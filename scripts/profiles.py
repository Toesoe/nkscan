#!/usr/bin/env python3
"""Turn Nikon Scan's scanner profiles into standard ICC ones that take linear data.

Nikon Scan ships one input profile per scanner per film type. Two things stop us
using them as they are:

1. The profile class is `nkpf`, a Nikon private class. littlecms refuses to open
   the file at all. Everything else in it is standard ICC v2, so the class is
   patched to `scnr`.

2. They expect the gamma 2.2 encoded values Nikon Scan's driver hands its CMS,
   not the linear samples the scanner returns. Verified with littlecms against
   NKLS9000_P: feeding 0.5^(1/2.2) gives Y = 0.495, feeding 0.5 gives Y = 0.21.

   The encoding lives in the A2B0 input curves for the LUT profiles and in the
   TRC tags for the matrix-only monochrome ones. Composing the encode into those
   curves, new[L] = old[L^(1/2.2)], leaves Nikon's own measurements intact and
   moves the profile onto linear input.

Run from the repo root. Reads profiles/*.icm, writes profiles/*.icc.
"""

import ctypes
import ctypes.util
import glob
import os
import struct
import sys

# What Nikon Scan encodes with before its CMS sees the data
GAMMA = 2.2


def tags(b):
    n = struct.unpack(">I", b[128:132])[0]
    return {
        b[132 + 12 * i : 136 + 12 * i].decode("latin1"): struct.unpack(
            ">II", b[136 + 12 * i : 144 + 12 * i]
        )
        for i in range(n)
    }


def recurve(table):
    """`table` sampled at L^(1/gamma), so it takes linear input"""
    n = len(table)
    out = []
    for i in range(n):
        at = (i / (n - 1)) ** (1.0 / GAMMA) * (n - 1)
        lo = int(at)
        hi = min(lo + 1, n - 1)
        f = at - lo
        out.append(round(table[lo] * (1 - f) + table[hi] * f))
    return out


def linearize(b):
    """A copy of `b` as a standard input profile taking linear samples"""
    b = bytearray(b)
    b[12:16] = b"scnr"
    t = tags(b)

    if "A2B0" in t:
        off, _ = t["A2B0"]
        if bytes(b[off : off + 4]) != b"mft2":
            raise SystemExit(f"A2B0 is {bytes(b[off:off+4])}, not the mft2 this handles")
        channels = b[off + 8]
        entries = struct.unpack(">H", b[off + 48 : off + 50])[0]
        base = off + 52
        for c in range(channels):
            at = base + c * entries * 2
            old = list(struct.unpack(f">{entries}H", b[at : at + entries * 2]))
            b[at : at + entries * 2] = struct.pack(f">{entries}H", *recurve(old))
        return bytes(b), f"A2B0 {channels}x{entries}"

    # The channels usually share one block of curve data, and curving it once
    # per tag that points at it would apply the encode three times over
    curves = [tag for tag in ("rTRC", "gTRC", "bTRC", "kTRC") if tag in t]
    seen = set()
    for tag in curves:
        off, _ = t[tag]
        if off in seen:
            continue
        seen.add(off)
        if bytes(b[off : off + 4]) != b"curv":
            raise SystemExit(f"{tag} is {bytes(b[off:off+4])}, not a curv")
        count = struct.unpack(">I", b[off + 8 : off + 12])[0]
        if count < 2:
            raise SystemExit(f"{tag} is a gamma rather than a table, which this does not handle")
        at = off + 12
        old = list(struct.unpack(f">{count}H", b[at : at + count * 2]))
        b[at : at + count * 2] = struct.pack(f">{count}H", *recurve(old))
    return bytes(b), " ".join(curves)


def lcms():
    """littlecms, for checking the result rather than trusting it"""
    # A nix store holds builds for other architectures too, so try until one loads
    candidates = [ctypes.util.find_library("lcms2")]
    candidates += sorted(glob.glob("/nix/store/*/lib/liblcms2.so.2"), reverse=True)
    lib = None
    for path in candidates:
        if not path:
            continue
        try:
            lib = ctypes.CDLL(path)
            break
        except OSError:
            continue
    if not lib:
        return None
    lib.cmsOpenProfileFromMem.restype = ctypes.c_void_p
    lib.cmsOpenProfileFromMem.argtypes = [ctypes.c_char_p, ctypes.c_uint32]
    lib.cmsCreateLab4Profile.restype = ctypes.c_void_p
    lib.cmsCreateLab4Profile.argtypes = [ctypes.c_void_p]
    lib.cmsCreateTransform.restype = ctypes.c_void_p
    lib.cmsCreateTransform.argtypes = [
        ctypes.c_void_p,
        ctypes.c_uint32,
        ctypes.c_void_p,
        ctypes.c_uint32,
        ctypes.c_uint32,
        ctypes.c_uint32,
    ]
    return lib


def luminance(lib, data, code):
    """Y of a neutral `code` through this profile, 0 to 1"""
    h = lib.cmsOpenProfileFromMem(data, len(data))
    if not h:
        return None
    lab = lib.cmsCreateLab4Profile(None)
    rgb16 = (4 << 16) | (3 << 3) | 2
    lab_dbl = (1 << 22) | (10 << 16) | (3 << 3)
    t = lib.cmsCreateTransform(ctypes.c_void_p(h), rgb16, ctypes.c_void_p(lab), lab_dbl, 1, 0)
    if not t:
        return None
    src = (ctypes.c_uint16 * 3)(code, code, code)
    dst = (ctypes.c_double * 3)()
    lib.cmsDoTransform(ctypes.c_void_p(t), src, dst, 1)
    star = dst[0]
    return ((star + 16) / 116) ** 3 if star > 8 else star / 903.3


def main():
    sources = sorted(glob.glob("profiles/*.icm"))
    if not sources:
        raise SystemExit("no profiles/*.icm to convert; run from the repo root")
    lib = lcms()
    if not lib:
        print("no littlecms found, writing without checking", file=sys.stderr)

    for src in sources:
        raw = open(src, "rb").read()
        out, what = linearize(raw)
        dest = os.path.splitext(src)[0] + ".icc"
        open(dest, "wb").write(out)

        note = ""
        if lib:
            # A linear half should come back as half the luminance
            got = luminance(lib, out, 32768)
            note = f"  0.5 linear -> Y {got:.3f}" if got else "  littlecms refused it"
        print(f"{os.path.basename(dest):22s} {what:16s}{note}")


if __name__ == "__main__":
    main()
