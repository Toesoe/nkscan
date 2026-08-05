# One capability-driven protocol layer for all Nikon Coolscan models

## Context

`src/` has been stripped back to `scsi/`, `transport/`, and an empty `protocol/`.
The old per-model tree (`scanners/{ls40,ls50,ls4000,ls5000,ls8000,ls9000,nikon}/`)
was reverse-engineered guesswork; we now have two official Nikon interface specs
in `docs/` (LS-5000 = USB/35 mm, LS-9000 = FireWire/medium-format).

Reading both end to end shows they are **one protocol**, not two. Identical
opcode table, identical CDBs including the vendor ones (C0h/C1h/E0h/E1h),
identical mode pages, identical READ/SEND data header, and — the decisive part —
**window descriptor bytes 0–49 are byte-for-byte identical**.

More importantly, every behavioural difference between the two is *advertised by
the device itself* in the INQUIRY VPD pages. So the design is not "one command
layer plus two model profiles" — it is one command layer over a `Capabilities`
struct parsed at runtime, with no model variants at all. Model identity is
needed only to key a (normally empty) errata table.

---

## Every difference reduces to an advertised capability

This is the load-bearing claim, so here it is field by field. Left column is a
behaviour that would traditionally justify a model variant; middle column is the
VPD field that reports it.

| Behaviour | Advertised by | LS-5000 | LS-9000 |
|---|---|---|---|
| Can crop in X? | C1h X-Max vs X-Min Set Window Address (24–27 vs 28–31) | `0 == 0` → **no**, must use full boundary | `9999` vs `0` → yes |
| X/Y crop bounds | C1h 36–39 / 58–61 (Set Window boundary) | 3946 / 5959 | 10000 / holder-dep. |
| Resolution rule | C1h byte 16 bits 0–1 (resolution type) + byte 85 (Line Gap Count) | type 3 → pitch ∈ {1, even} | type 2 → pitch ∈ divisors(12) |
| Min/max resolution | C1h 18–23, 40–45 | 90–4000 | X 666–4000, Y 333–4000 |
| CCD lines → de-interleave layout | C1h byte 86 | 2 | 3 |
| Who re-registers CCD lines | E1h byte 5 bit 0 | 0 → scanner | 1 → **host** |
| Who strips invalid padding | E1h byte 5 bit 2 | 1 → host | 1 → host |
| Who builds thumbnails / averages | E1h byte 4 bits 0–1 | host | host |
| Which READ/SEND data types exist | E1h bytes 6–10 | shading 84h, setup 8Dh, perforation 8Eh, boundary-type2 8Fh | boundary 88h, analog gain 8Ah, max value 81h |
| Which EXECUTE ops exist | E1h bytes 20–35 | 14 ops | 8 ops |
| **Stage Move exists?** | E1h byte 28 bit 0 | **1** | **0** (SET WINDOW moves the stage) |
| Auto-calibration op exists? | E1h byte 22 bit 2 | 0 | **1** (SET PARAMETER 92h) |
| Frame-boundary mechanism | E1h byte 8 bits 5–6 vs byte 10 bits 0–1; byte 9 bit 6 | perforation + boundary-type2 | plain boundary |
| Scanner publishes frame rects? | C1h byte 16 bit 6 → VPD C8h/C9h | 0 | **1** |
| Media-change notification | E1h byte 36 bits 0–3 | without notice (poll) | with notice (Unit Attention) |
| Scan kinds / modes | D1h bytes 4–5 | + reverse direction | + high speed |
| Colour-ordering constraint | D1h bytes 8–9 | constrained (2=G, 3=B, 4th) | free (`0`/`0`) |
| Bit depth, exposure range | D1h bytes 10, 16–24 | 16-bit, 1..0x3FFFFFF | identical |
| Which VPD pages exist at all | page 00h list | incl. **E2h** | incl. **C8h–CBh**, holder IDs 10h–1Fh |
| EXECUTE parameter ranges | E2h (when present) | B4h unload time 60–3600 s | n/a |
| Media identity | FRU ASCII pages | adapter × holder | holder only |

Two consequences worth calling out, because they look like model logic but
aren't:

- **Frame boundaries.** The mechanism follows the film format — 135 film seeks by
  counting perforations (read 8Eh, write back 8Fh with perforation/decimal/pulse),
  120 film seeks by rectangle (scanner publishes C8h/C9h, host corrects via 88h).
  But *which one applies is advertised*, so this is a branch on capability, not
  on model.
- **Stage motion.** The LS-9000 has no Stage Move and no absolute positioning;
  SET WINDOW *is* the move command. That's `E1h[28].bit0 == 0`, discoverable —
  and it's the documented root of the existing stage-position-hazard note (a
  frame-kind SET WINDOW moves the mechanism, hence the long timeout).

Similarly, the host-side post-processing obligations don't need model knowledge
at runtime: SCAN returns a CHECK CONDITION whose ASCQ names the pending
cooperative action, and the READ 87h record's **byte 0 is its own type code**
with the length in the data header. Parse by type code, dispatch on that.

---

## What is *not* discoverable

Four residuals. Only the first is a real gap.

1. **The IR / Digital ICE channel is absent from both specs.** Table 2-10-4 in
   each documents window identifiers 0–3 (+4 neutral gray, unsupported) and
   nothing else — the word "infrared" appears nowhere in either document. The
   capture in `RE_FINDINGS.md` §6 shows Nikon Scan sending **four** SET WINDOW
   calls per pass with **window ID 9 = IR**, IR first. The LS-5000 spec hints
   obliquely (SCAN accepts a 4-entry window list; Colour Ordering2 defines a
   "fourth color") but never names it. **The RE work is still the only source
   here** — this should be modelled as an extension beyond the documented
   window-ID enum, not dropped.
2. **Window descriptor bytes 50–57.** Declared 58 (LS-9000) / 61 (LS-5000) in
   C1h, but both descriptor tables stop at byte 49. Moot in practice: Nikon Scan
   puts `0x0032` (50) in the SET WINDOW header and sends exactly the documented
   50 bytes. Emit 50; the spec's "lacking part shall be unchanged" rule makes it
   well-defined.
3. **Sense-block shape** — 8-byte `status/key/ASC/ASCQ/TSC` vs SBP-2 8-quadlet
   with `Information`/`FRU`. This is determined by *transport*, not model, so
   `transport/` already knows which to parse.
4. **Devices that misreport.** Proven necessary by the docs themselves: the
   LS-9000's E1h summary bytes 6–10 contradict its own per-bit tables, and E3h is
   documented but missing from the page-00h list. Needs a thin override table
   keyed on INQUIRY Product Identification + revision — an errata list, not a
   model variant. Starts empty except for known-bad fields.

---

## Structure for `src/protocol/`

```
src/protocol/
  mod.rs          Scanner facade over transport::Transport
  caps/
    mod.rs        Capabilities: parsed once at open, single source of truth
    address.rs    C1h -> geometry, CCD, resolution rule, buffer limits
    setwindow.rs  D1h -> scan kinds, modes, colour, bit depth, exposure range
    other.rs      E1h -> host-coop bits, data-type registry, EXECUTE support
    pages.rs      00h page list; C8h/C9h frame rects; E2h op ranges; FRU identity
    errata.rs     overrides keyed on product id + revision (near-empty)
  cdb.rs          all 17 CDB builders — no variants
  window.rs       50-byte descriptor codec + 8-byte header; window IDs incl. IR=9
  resolution.rs   PitchRule::{OnePlusEven, DivisorsOf(u8)} + rounding + ROUNDED sense
  data.rs         READ/SEND data types, gated on caps
  execute.rs      SET PARAMETER 13-byte op block, gated on caps
  coop.rs         cooperative records, dispatched on record type byte (1,2,4,6,7)
  sense.rs        sense-key/ASC/ASCQ decode (tail shape supplied by transport)
```

Design points:

- `Capabilities` is built by **parsing INQUIRY at open time**, then consulted for
  every decision. Nothing branches on model. This is what makes LS-40 / LS-50 /
  LS-4000 / LS-8000 work without their own specs — they self-describe.
- Unsupported operations fail from the capability check with a clear error
  (`Unsupported { op, reason }`), before any CDB is built. That replaces the old
  `capability/unsupported.rs` machinery.
- The X-crop rule lives in `window.rs` as spec logic: if
  `x_max_addr == x_min_addr`, force width to the X boundary and report the
  host-side crop back to the caller rather than silently scanning full width.
- `scsi/` already carries the SCSI-generic halves —
  `cdbs/{inquiry,mode_select,mode_sense,scan,send_diagnostic,test_unit_ready,
  window,read_send,reservation}.rs` and `mode_pages/measurement_units.rs`.
  `protocol/` layers Nikon semantics on those; it must not re-assemble CDBs.
- RESERVE/RELEASE and the BUSY / RESERVATION CONFLICT statuses only exist on
  SBP-2; on USB they cannot occur. Let `transport/` report which statuses are
  reachable rather than testing the model.

---

## Sense data: there is no sense-code chapter

LS-5000 §1-1-5-2 says the sense buffer is filled from **"table 4-1-1"** — but
neither document has a section 3 or 4; both end at §2-17-1. Verified against the
original `.docx`, so this isn't a conversion loss: the reference dangles. The
LS-9000 doesn't even make the promise — §1-1-10 defines the SBP-2 quadlet layout
and never gives a code list.

So the codes are **distributed across per-command tables**, and the only global
statement is a two-row table at the head of §2 in both docs:

- **"common error 1"** = `05h-24h-00h-00h` INVALID FIELD IN CDB
- **"common error 2"** = `05h-26h-00h-00h` INVALID FIELD IN PARAMETER LIST

Exhaustive sweep of both documents gives **35 distinct codes for the LS-5000 and
27 for the LS-9000**. Note that one code in each (`ROUNDED PARAMETER`) is written
as prose rather than as a tuple, so grepping for `NNh-NNh-NNh-NNh` alone misses
it. Tuples are `key-ASC-ASCQ-TSC`, matching LS-5000 bytes 1..4.

### Wire format is transport's problem

- **LS-5000 (USB)** — 8 bytes: `status, sense_key, ASC, ASCQ, TSC, 3× reserved`
- **LS-9000 (SBP-2)** — 8 quadlets: `sfmt/status/V/M/E/I/key/ASC/ASCQ`, then
  `Information`, `CDB-dependent`, `FRU`, vendor-dependent

**Neither device implements REQUEST SENSE.** Sense is always auto-delivered with
the status, so there is no CHECK CONDITION → REQUEST SENSE round trip to write.

**Unresolved:** §2-11 specifies that short reads return CHECK CONDITION with
`ILI=1`, `valid=1` and Information set to the residual — the mechanism for
detecting the end of a thumbnail scan of unknown length. SBP-2 has homes for all
three (`V`/`I` bits in quadlet 2, `Information` in quadlet 3). The LS-5000's
8-byte block has none of them; bytes 5–7 are explicitly Reserved. The USB
residual path is therefore unspecified and needs hardware.

### Reading sense on Linux (scsi-generic)

Host-side plumbing rather than spec content, but it determines what actually
lands in `sense.rs`.

**`hdr.status` and the sense buffer are different planes.** `hdr.status` is the
raw one-byte SCSI status from the target — the same `00h`/`02h`/`08h`/`18h`
values as the specs' table 1-1-5-1 / 1-1-10-1. It says *that* something happened.
`sbp` carries the sense data and says *what*. Neither substitutes for the other.
`masked_status` and `msg_status` are SCSI-1-era legacy (`masked_status` is just
`status >> 1`); ignore both.

**There are three independent status planes and conflating them is the usual
bug.** Check in this order:

```
ioctl() < 0     OS-level failure (errno); the command may never have been built
host_status     HBA / transport: DID_NO_CONNECT, DID_TIME_OUT, DID_ERROR …
driver_status   mid-layer: DRIVER_SENSE (0x08), DRIVER_TIMEOUT …
status          the target's SCSI status byte
sbp             sense data, valid only when DRIVER_SENSE is set
```

A `DID_TIME_OUT` with `status == 0` is not success — the command never came back
and `status` is meaningless. `hdr.info & SG_INFO_CHECK` is a cheap composite test
if a single early branch is wanted.

**Gate on `sb_len_wr`, not on a fixed length.** Set `mx_sb_len` generously (64)
and read `sb_len_wr` for how much was actually written. `RE_FINDINGS.md` §3
observed a 32-byte sense buffer on the Windows transport, so the 18-byte
assumption baked into a lot of pass-through code is wrong here.

**No REQUEST SENSE round trip.** The kernel mid-layer performs auto-sense, so
`sbp` is already populated on return — which lines up with both devices, neither
of which implements the REQUEST SENSE opcode.

**`hdr.resid`** = `dxfer_len - actually_transferred`, the Linux-side equivalent of
the ILI/Information residual above. Do not build end-of-thumbnail detection on it
without verifying: `resid` is not reliably populated by every transport, and
usb-storage in particular often leaves it zero.

**SBP-2 quadlets never reach userspace.** `firewire-sbp2` repacks the status
block into an ordinary fixed-format sense buffer first:

| SBP-2 status block | → fixed-format sense |
|---|---|
| byte 1 (sense key + valid/mark/eom/ili) | byte 2 |
| bytes 4–7 (Information) | bytes 3–6 |
| bytes 8–11 (CDB-dependent) | bytes 8–11 |
| byte 2 (sense_code) | byte 12 — **ASC** |
| byte 3 (sense_qualifier) | byte 13 — **ASCQ** |
| byte 12 (FRU) | byte 14 |
| bytes 13+ (sense-key dependent) | byte 15+ |

So the parse is ordinary SPC — `sense[2] & 0x0F` key, `sense[12]` ASC,
`sense[13]` ASCQ, `sense[14]` FRU — and §1-1-10's quadlet diagram only describes
what the device emitted, not what arrives.

**This gives a concrete test for the homeless 4th byte.** It must land in the
sense-key-dependent region, i.e. `sense[15]` onward after repacking. Trigger a
known 4-tuple — `09h-80h-04h-01h` is easy, any sub-4000 dpi three-line scan —
with `mx_sb_len = 64`, dump all `sb_len_wr` bytes, and look for the `01`.

**None of this applies to the LS-5000** unless it enumerates as usb-storage.
Nikon's USB protocol is its own phase model over bulk endpoints (§1-1-2), not
BOT, so that path parses the 8-byte status block directly off libusb and has no
`sg_io_hdr` at all.

### Five functional classes, not one error enum

Modelling these as a flat `Error` enum will produce a driver that fails on
success. They split cleanly by sense key.

#### 1. Key `02h` — the asynchronous progress channel

SCAN, EXECUTE and ABORT are *operation activation commands*: they return GOOD
**immediately** and perform the work in the background. Completion is discovered
by polling TEST UNIT READY. These codes are that poll's return channel.

| Code | Meaning |
|---|---|
| `00h-00h-00h-00h` | done, succeeded |
| `02h-04h-01h-00h..04h` | still working — `00` operation running, `01` load/eject, `02` correction-data measurement, `03` load operation, `04` auto shading/WB |
| `02h-04h-02h-00h` | done, **failed** (internal mechanical error) |
| `02h-04h-03h-xx` | needs physical intervention — LS-5000: `00` adapter ejected, `01` IA-20 LL door, `02` undefined adapter, `03` SA-30 film gate, `04` adapter unlocked. LS-9000: `06` FH-869GR mask unset, `07` undefined holder |
| `02h-3Ah-00h-xx` | medium not present — LS-5000 `00`, `01`, `03`, `04`; LS-9000 `01` only |
| `02h-05h-00h-00h` | operable, but still initialising after power-on |
| `02h-04h-00h-00h` | needs an initialising command (LS-5000 only) |

**Two-stage error retrieval.** `02h-04h-02h-00h` is deliberately generic. To get
the actual fault you then issue SEND DIAGNOSTIC, which reports the concrete error
— **and clears it** (§2-8). One shot; if you skip it the detail is gone.

#### 2. Key `05h` — programming errors

| Code | Meaning |
|---|---|
| `05h-1Ah-00h-00h` | PARAMETER LIST LENGTH ERROR |
| `05h-20h-00h-00h` | INVALID COMMAND OPERATION CODE (LS-5000 only) |
| `05h-24h-00h-00h` | INVALID FIELD IN CDB — common error 1 |
| `05h-25h-00h-00h` | LOGICAL UNIT NOT SUPPORTED (LUN ≠ 0) |
| `05h-26h-00h-00h` | INVALID FIELD IN PARAMETER LIST — common error 2 |
| `05h-2Ch-00h-00h` | COMMAND SEQUENCE ERROR — **overloaded, see below** |
| `05h-2Ch-02h-00h` | INVALID COMBINATION OF WINDOWS SPECIFIED |

**The `2Ch` trap.** `05h-2Ch-00h-00h` covers four unrelated situations: READ
image without a preceding SCAN; EXECUTE before SET PARAMETER; any non-basic
command issued mid-operation; **and reading past the end of the image data**.
That last one means *normal end-of-image is reported as a command sequence
error*. Treat `2Ch` as fatal and every completed scan looks like a failure.

#### 3. Key `09h` / ASC `80h` — cooperative handshake, not an error

**ASCQ is the cooperative operation type code, and it is the same value as
byte 0 of the READ 87h record.** A CHECK CONDITION here means "stop, read 87h,
do this post-processing, re-issue".

| ASCQ | Operation | LS-5000 4th byte | LS-9000 4th byte |
|---|---|---|---|
| `01h` | THUMBNAIL CREATED BY DRIVER | `02h` (IA-20), `06h` (SA-21/30) | `04h` |
| `02h` | AVERAGING MULTIPLE READING | `00h` | `00h` |
| `04h` | MULTI LINE SIMULTANEOUS READING | — | `01h` |
| `06h` | TRUNCATED BY DRIVER | `01h` | `00h` |
| `07h` | CCD DATA CREATED BY DRIVER | `00h` | `00h` |

`sense.rs` should classify on `(key, asc)` first and route `(09h, 80h)` to
`coop.rs` as a control-flow signal. Dispatch the actual work on the record's own
type byte rather than the ASCQ, and the 4th-byte variation never reaches you.

#### 4. Keys `06h` / `0Bh` — environment and contention

| Code | Meaning |
|---|---|
| `06h-xxh-xxh-xxh` | UNIT ATTENTION — LS-9000 triggers: power-on, holder removed, holder exchanged |
| `06h-2Ah-01h-00h` | MODE PARAMETERS CHANGED — raised to *other* initiators after a MODE SELECT |
| `0Bh-08h-00h-00h` | LU COMMUNICATION FAILURE — busy with an internal operation |
| `0Bh-4Bh-00h-00h` | DATA PHASE ERROR |
| `0Bh-4Eh-00h-00h` | OVERLAPPED COMMANDS ATTEMPTED |

INQUIRY must **not** clear a pending Unit Attention (LS-5000 §2-2 item 5).

#### 5. Key `01h` — success with adjustment

| Code | Meaning |
|---|---|
| `01h-37h-00h-00h` | **ROUNDED PARAMETER** — resolution snapped to the nearest legal pitch |

Sense key 1 is RECOVERED ERROR: the SET WINDOW succeeded. Read the actual value
back with GET WINDOW. This arrives as CHECK CONDITION and means "fine, carry
on" — miss it and every non-native resolution looks like a failure.

### Where the models actually differ

Only the ASCQ vocabulary: `02h-04h-03h-xx` (adapter states vs holder states),
`02h-3Ah-00h-xx` (four LS-5000 variants vs one), and the `09h-80h` 4th byte.
Everything structural is shared.

### Undefined terms and open questions

- **"Basic command"** is used three times in each spec — the rule being that
  issuing a non-basic command mid-operation *aborts the operation* — and is
  never defined. TEST UNIT READY is clearly one; beyond that it is guesswork,
  and guessing wrong destroys in-flight work.
- **"TSC"** is named once (LS-5000 §1-1-5-2) and defined nowhere. It is the 4th
  element of every tuple, so the per-command tables specify it implicitly.
- ~~**The 4th byte has no home in the SBP-2 status block**~~ — **settled**, see
  "TSC lives at sense byte 15" below. It rides in quadlet 5's
  `sense_key-dependent` field.

---

## What the hardware said

Everything below was read off a real **LS-9000 ED, firmware 1.00, FH-869GR
holder**, on 2026-08-04. Where it disagrees with either document, it wins.

The pattern across every disagreement is the same direction: **the firmware does
more than the documents admit.** Treat the specs as a lower bound on capability,
prefer per-bit tables over summary bytes, and prefer the device over both.

### TSC lives at sense byte 15

The 4th tuple element does exist on SBP-2, in quadlet 5's `sense_key-dependent`
field, which `firewire-sbp2` repacks to sense bytes 15–17.

Two tuples matched their documented TSC exactly:

| Observed | §2-1-2 |
|---|---|
| `02h-3Ah-00h` + byte 15 `01h` | `02h-3Ah-00h-01h` "the holder is not inserted" |
| `02h-04h-01h` + byte 15 `01h` | `02h-04h-01h-01h` "during loading/ejection" |

The decisive evidence is independent variation: inserting a holder queues **two**
unit attentions, both `06h-28h-00h`, differing only at byte 15 (`01h` then
`00h`). Nothing else in the buffer distinguishes them.

Note SKSV (byte 15 bit 7) is **clear**, so this is not SPC sense-key-specific
being used properly — Nikon is using the vendor half of the field. A conformant
reader would discard it, and an SPC progress indicator would have byte 15 ≥ `80h`.

Two caveats. That TSC rides in quadlet 5 is Nikon's choice and portable; that it
lands at *byte 15* is `firewire-sbp2`'s repack. Where `scsiscan.sys` puts it is
unknown — that driver's `SENSE_LENGTH = 32` exists because Nikon state was seen
past the end of fixed-format sense, which suggests a different layout.

### Values that disagree with the documents

| Field | Spec | Hardware |
|---|---|---|
| `C1h` window descriptor length (5,6) | 58 | **59** — and the real stride is 50 |
| `D1h` byte 4 scanning kind | `03h` | **`1Bh`** — bits 3,4 set, both called Reserved |
| `E1h` byte 5 cooperation | `05h` | **`0Dh`** — per-bit table was right |
| `E1h` byte 6 data types | `ACh` / `80h` | **`A0h`** — neither |
| `E1h` byte 15 max-value depth | `0` | **16** |

**`D1h` byte 4.** Bits 3 and 4 are `Reserved [-]` in §2-2-2-4's table, which also
gives the byte as `03h`. Both are set on hardware.

Bit 3 is **not actually undocumented** — the LS-9000 contradicts itself. Its own
§2-10 byte 42 table, which is bit-for-bit identical to the LS-5000's, names bit 3
*Set up Scanning2*. Byte 11 of the same `D1h` page corroborates it a third time:
§2-2-2-4 defines that field as "effective when Setup Scan2 of the Scanning Kind
support field is 1", so the page defines a field in terms of a bit it elsewhere
calls reserved. Hardware sides with §2-10.

Bit 4 is the real unknown. Both specs' byte-42 tables call it reserved; only the
LS-5000's `D1h` table names it, as *Histogram Scanning*. The bit is set on an
LS-9000 and that is all that is known.

**`E1h` byte 6.** `A0h` means gamma/LUT read and write are **off** — confirming
§2-11-4's prose over the `ACh` summary, and matching the no-hardware-LUT finding
— while Max Value Data reading is **on**, which both the summary and the per-bit
table deny. Corroborated independently by byte 15 reading 16 where the spec says 0.

### The infrared window is real, and the device says so

GET WINDOW at power-on returns **five** descriptors, identifiers `0, 1, 2, 3, 9`.
Identifier **9 is infrared** — neither spec mentions it (table 2-10-4 stops at 4,
with an unsupported neutral gray), so this is the device confirming what only the
RE captures previously showed.

Every window at rest: 4000 × 4000 dpi, origin 0,0, 10000 × 13860, 16 bpp, image
kind, normal quality, line-without-CCD-distance, AE `FFh`, `color_ordering` **0**
— the channel identity is in byte 0, not byte 40.

That size is **not a legal window**. 10000 is `C1h`'s CCD pixel count exactly, and
both figures exceed the boundaries the same page publishes (8964 × 13176). The
power-on descriptor describes the hardware maximum, not something SET WINDOW would
accept, so it is a poor template — clamp to the boundary before reusing it.

Exposures are per channel, in 10 ns units, and are usable seeds:

| Window | Exposure | | |
|---|---|---|---|
| 0 default | 50842 | 508.4 µs | identical to G |
| 1 R | 71125 | 711.3 µs | ×1.40 |
| 2 G | 50842 | 508.4 µs | reference |
| 3 B | 41480 | 414.8 µs | ×0.82 |
| 9 IR | 93004 | 930.0 µs | ×1.83 |

### GET WINDOW quirks

- **`Single = 1` is refused** with `05h-24h`, for identifiers 0 and 1 alike,
  despite §2-10 giving byte 1 bit 0 as `[0, 1]`. Only "all windows" works.
- **The transfer length must not exceed what exists.** 512 bytes was refused;
  8, 66 and 240 were accepted against a real total of 258. So the correct idiom
  is two-phase: read the 8-byte header, then re-issue for `2 + length`.
- The header carries **both** lengths — bytes 0,1 the total after themselves,
  bytes 6,7 one descriptor. Take the stride from there rather than assuming.
- This header shape differs from SET WINDOW's parameter header, which puts its
  length at bytes 6,7 and nothing at 0,1. Do not share a codec.

### Measurement units, and what `C1h` is counting in

The mode page behaves exactly as documented — the one place so far where it does.
Current and Default both read **1200**; the Variable mask is `ff ff` over the
divisor and `00` over the basic measurement unit, verbatim 2-6-2; the block
descriptor is the documented `density 0 / blocks 0 / block length 1`. MODE SELECT
to 4000 and back round-trips.

Two things the specs never say, both settled here:

- **`C1h` is quoted in maximum-resolution units, whatever the divisor is.** The X
  boundary reads 8964, which is 56.9 mm at 4000 dpi — the 6 cm film width. At 1200
  it would be 190 mm, wider than the machine. The divisor does not enter into it.
- **GET WINDOW reports stored values, not converted ones.** All five descriptors
  came back byte-identical at divisor 1200 and at 4000. So the numbers in a
  descriptor mean whatever the divisor was when they were *set*, and the page
  gives no way to tell which — GET WINDOW after a divisor change is ambiguous.

Together those argue for setting the divisor to the maximum resolution once at
open and leaving it: one step is then one pixel, and window coordinates, `C1h`
addresses and `C1h` boundaries are all in the same unit with no conversion
anywhere. The alternative, 1200, needs §2-10's second formula and two extra
roundings on every axis.

The divisor survives us — it holds until the next MODE SELECT, a reset or a power
cycle — so a session must read it rather than assume the documented default.

### What `C1h` says at rest

With adapter 1 / holder `17h` (FH-869GR) loaded:

| Field | X | Y |
|---|---|---|
| Optical dpi | 4000 | 4000 |
| Settable dpi | 666–4000 | 333–4000 |
| Offset address | 0–8963 | 0–34644 |
| Boundary (max window) | 8964 | 13176 |

Both axes are croppable, so the LS-5000's forced-full-width rule does not apply
here. The Y offset range runs to 34644 — 220 mm, the length of the strip in the
holder — while a single window may be at most 13176 long (83.6 mm). CCD 10000
pixels, 3 lines, line gap 12, `DivisorsOf(12)` pitch, 16-bit, image buffer 256 KB,
focus range 0–450, one frame loaded.

Two smaller notes from the same dump. `set_parameter_len` is **15**, not the 13
the EXECUTE block was assumed to be. And byte 4 bit 0 is **set** — the field the
LS-9000 spec calls unused and the LS-5000 calls microcode downloading.

### What Nikon Scan actually sends, read through the spec

The six sessions in `/mnt/storage/NikonScanDecomp/scan_captures/` were reverse
engineered before we had the documents. Reading their SET WINDOW payloads
against 2-10-3 for the first time settles several things the RE could not.

Every descriptor in the corpus is one of four shapes. Decoded into the flags in
`window.rs` and `caps/set_window.rs`, with the raw byte in brackets:

| Phase | dpi | `multiple_reading` | `flags` (41) | `scanning_kind` (42) | `scanning_mode` (43) | `color_interleaving` (44) |
|---|---|---|---|---|---|---|
| Calibration preamble | 4000 | 0 | `AVERAGING\|POSITIVE` [`81`] | `IMAGE` [`01`] | `NORMAL_QUALITY` [`02`] | `LINE_WITHOUT_DISTANCE` [`02`] |
| Thumbnail | 83 | 0 | `AVERAGING\|POSITIVE` [`81`] | `THUMBNAIL` [`02`] | `NORMAL_QUALITY` [`02`] | `LINE_WITHOUT_DISTANCE` [`02`] |
| Preview / prescan | 666 | n−1 | `POSITIVE` [`01`] | `IMAGE` [`01`] | `HIGH_SPEED` [`04`] | `MULTILINE_SIMULTANEOUS` [`40`] |
| Scan | 4000 | n−1 | `AVERAGING\|POSITIVE` [`81`] | `IMAGE` [`01`] | `NORMAL_QUALITY` [`02`] | either of the two above |

With multisampling on, `scanning_mode` gains `MULTI_READING` [`|10`] — so the
prescan of the 16× session is `HIGH_SPEED|MULTI_READING` [`14`] and its scan is
`NORMAL_QUALITY|MULTI_READING` [`12`].

`window.rs`'s tests decode three of these straight out of the corpus, so the
mapping is checked rather than asserted.

**Byte 44 is the sensor-mode choice, not a resolution consequence.** This is the
finding that matters, and it needed two sessions to see. `full_session_cold_start`
and `singleline_ccd` both scan at 4000 dpi, Normal Quality, averaging on, and
differ in byte 44 alone: `40h` for what Nikon Scan calls the normal CCD mode,
`02h` for what it calls **Super Fine**. So `40h` — 2-10's "3 line simultaneous
reading" — is the multi-line sensor read whole, and `02h` is the one-line read.
Nothing about it follows from dpi, and a three-line scan at full resolution
(`81/02/40`) is a combination no dpi-driven rule can produce.

That also means **Super Fine is the safe default**: `02h` needs no host
re-registration, so it never enters the `09h-80h-04h` cooperative path, and it
bins the bar correctly at every resolution, which the multi-line read does not.

**Multiple reading** is confirmed exactly as 2-10 words it — scans per line is
`multiple_reading + 1`, in byte 40's high nibble:

| Session | `multiple_reading` | raw byte 40 |
|---|---|---|
| 4× | 3 | `30` |
| 8× | 7 | `70` |
| 16× | 15 | `F0` |

The count and `ScanMode::MULTI_READING` always move together, which `validate`
now enforces rather than letting SCAN refuse the pair.

**Every one of these settings is gated by an `E1h` cooperation bit**, and that is
the useful way to read the page: each set bit is work the *host* must do before
the matching descriptor field can be used at all. Ours reports byte 4 = `83h`,
byte 5 = `0Dh`:

| Cooperation bit | Enables | Sense that arrives |
|---|---|---|
| `THUMBNAIL` | `ScanKind::THUMBNAIL` | `09h-80h-01h` |
| `AVERAGING` | `multiple_reading` > 0, i.e. multisampling | `09h-80h-02h` |
| `MULTI_LINE` | `MULTILINE_SIMULTANEOUS` in byte 44 | `09h-80h-04h` |
| `TRUNCATED` | 8-bit odd widths | `09h-80h-06h` |
| `CCD_DATA` | raw CCD readout | `09h-80h-07h` |

So multisampling is not a descriptor field we can simply set — the scanner returns
*n* readings per line and expects us to average them. Nothing in the corpus'
descriptor bytes says so; only `E1h` does. The one configuration needing no
cooperation at all is a plain single-line image scan, which is why that is what
`Session` should reach for first.

Three other things the descriptors show that the tables above do not:

- **`composition` is `MultilevelRGB` in every scan window**, even though each
  window carries a single color. Not `MultilevelBW`, which is what "one window is
  one channel" would suggest.
- **The Y resolution is sent as a different number from X** — the 666 dpi prescan
  sends 333 — confirming 2-10's claim that SET WINDOW ignores the field.
- **X offset and width can sum past the boundary.** A scan sends offset 518 with
  width 8964, which is the full boundary; 518 + 8964 = 9482, over the 8964 limit
  but inside the 10000-pixel CCD. So the two are bounded separately, and the sum
  is bounded by the sensor rather than by the boundary. `validate` checks them
  independently, which matches, but neither spec states the sum rule.

Two smaller confirmations. The thumbnail pass is `02h` in byte 42 — genuinely a
thumbnail scan, not an overview of some other kind — and it uses Normal Quality
and line ordering at 83 dpi, so low resolution alone never selects high speed.

And the IR window (id 9) is in the **scan** set only when Digital ICE is on:
`16x_multisample` and `singleline_ccd` send four descriptors, the ICE-off
`full_session_cold_start` sends three. That is what ties id 9 to ICE. Note it
appears in *every* prescan regardless, ICE off included, so the preview meters
infrared whether or not the scan will use it.

**SCAN's payload is the window id list.** `1B` with transfer length 3 sends
`01 02 03`; with ICE on it is length 4 and `09 01 02 03`, infrared first. That is
how a window set becomes a scan.

Two limits of the corpus itself, worth knowing before trusting it:

- **It contains no sense data.** The proxy dumps the sense buffer before the call,
  so all six traces show 32 zero bytes everywhere. `SRB_Status=0x84` marks where a
  CHECK CONDITION happened, but never what it said — so none of the `09h-80h`
  cooperative codes above can be confirmed from here, only from the spec and from
  our own hardware.
- **It has no reduced-resolution multi-line scan.** Nikon Scan only ever sends
  multi-line at 4000 dpi, so which of bytes 41, 43 and 44 actually makes the
  sensor bar bin below that is still open — the three always move together in
  this data. One NikonScan capture at 2000 dpi with the normal CCD mode would
  settle it. The
default above sidesteps the question entirely.

### Behaviour worth knowing

- **Unit attentions queue.** A holder insertion raises two, both `06h-28h-00h`
  (not-ready-to-ready, medium may have changed). A retry loop must expect the
  `StateChanged` arm to fire repeatedly.
- **Loading takes ~10 s** and reports `02h-04h-01h-01h` throughout, so a
  readiness wait needs a budget well past that.
- **INQUIRY works with a unit attention pending**, exactly as §2-2 note 5 says,
  which is what makes re-probing capabilities inside the retry loop safe.

---

## Spec errata to encode (with a comment citing the section)

1. **LS-9000 LUT.** Table 2-11-2 lists `03h` as R/S 2×16384; §2-11-4 says *"This
   unit does not support READ/SEND of the LUT."* Prose wins — and it matches the
   existing no-hardware-LUT finding (gamma 2.2 is host-side). Don't implement 03h.
2. **LS-9000 E1h bytes 6–10.** Summary bytes vs per-bit tables disagree (byte 6
   `ACh` vs `80h`; byte 7 `00h` vs `80h`; byte 8 `D0h` vs `F0h`; byte 9 `3Ah` vs
   `BAh`; byte 10 `48h` agrees). **Settled on hardware:** `A0 80 F0 BA 48`. The
   per-bit tables win for 7–10, and byte 6 matches *neither* — see below.
3. **LS-9000 E1h byte 5** declared `05h`, bit table implies `0Dh`.
   **Settled: `0Dh`.** CCD-data cooperation is real.
4. **LS-9000 8Dh** marked reserved in 2-11-2, but E1h byte 9 bit 5 says setup-info
   writing is supported.
5. **LS-9000 E3h** documented in §2-2-2-7, absent from the page-00h list.
   **Confirmed on hardware:** the page answers anyway, so `errata` must be able
   to *add* a page, not only override fields.
6. **LS-9000 analog gain** — 2-11-2 says 4 bytes × 2, §2-11-7's table shows
   2-byte fields. 4-byte IEEE-754 is the consistent reading.
7. **LS-5000 shading** — "47352 valid data × 2 bytes" is wrong; 47352 is the
   *byte* count (23676 × u16 = 3946 px × 3 line-modes × 2 gains).
8. **LS-5000 image buffer** — C1h table says 256 KB, prose says 64 KB.
9. **LS-5000 B1h** dark current listed "Yes" in 2-15-3 but E1h byte 26 bit 1 = 0.
10. **LS-5000 RESERVE/RELEASE** documented in §2-4/2-5, omitted from the §1-1-3
    command table.
11. **Descriptor length** declared 61/58 but only bytes 0–49 defined, in both.
    **Settled: 50.** Table 2-10-3 runs 0–49 in *both* documents, and an LS-9000
    reports a stride of 50 in its own GET WINDOW header. Every other number —
    58 in the headers' "recommended value", 59 from `C1h` — is wrong.
12. **Truncation ASCQ differs** — `09h-80h-06h-01h` (LS-5000, incl. the
    "not a multiple of 512 bytes" trigger) vs `09h-80h-06h-00h` (LS-9000, 8-bit
    odd-width only), and the LS-9000 defines no type-6 payload despite naming the
    trigger. Dispatch on the record's own type byte, not the ASCQ.

## The LS-9000 descriptor tail, resolved

Cross-checking `~/sync/Projects/Ghidra/NikonScan/RE_FINDINGS.md` §6 against the
official table (RE offsets are payload-relative — subtract 8) shows the capture
is **fully explained by the documented 50-byte descriptor**:

| RE `[n]` | Spec byte | Official meaning | Captured |
|---|---|---|---|
| `[6:8]` | — | descriptor length in header | **0x0032 = 50** |
| `[48]` | 40 | Multiple Reading Number \| Color Ordering | `00` |
| `[49]` | 41 | b7 Averaging, b0 Posi/Nega | `81` |
| `[50]` | 42 | **Scanning Kind** | `01` = Image Scanning |
| `[51]` | 43 | **Scanning Mode** | `02` = Normal Quality |
| `[52]` | 44 | **Color Interleaving** | `02` = line w/o CCD distance |
| `[53]` | 45 | **AE Value** | `FF` = 255, the documented default |
| `[54:58]` | 46–49 | **Exposure Value** u32, 10 ns units | `00 05 E1 01` ≈ 3.85 ms |

Three RE hypotheses are falsified: byte 42 is Scanning Kind (not a film-format
code), byte 44 is Colour Interleaving (not averaging), and averaging is bit 7 of
byte 41.

---

## Verification

1. ~~**INQUIRY sweep on the LS-9000**~~ — **done.** C1h byte 16 = `42h`, byte 85
   = 12, byte 86 = 3 all confirmed, and errata #2, #3, #5 and #11 are settled.
   See "What the hardware said". Real page dumps are now fixtures in
   `caps/{address,other,set_window}.rs`.
2. **Resolution rounding** — request 1333, 1000, 500, 333, 200 dpi; confirm
   GET WINDOW resolution and `ROUNDED PARAMETER (01h-37h-00h)` follow
   `DivisorsOf(12)`: 500 and 200 must round, 1333 must not.
3. **SET WINDOW byte-for-byte** — emit the 50-byte descriptor for window IDs
   9/1/2/3 and diff against the captured payload in `RE_FINDINGS.md` §6; expect
   an exact match including the header's `0x0032`. The encoder already
   round-trips the device's own GET WINDOW output byte for byte, so this checks
   the *choice* of values rather than the codec.
4. **Cooperative path** — run a sub-4000 dpi scan, confirm SCAN returns
   `09h-80h-04h-01h`, read 87h, check bytes 13–14 equal `12 / pitch`.
5. **Capability gating** — assert `Stage Move` and SET PARAMETER `D2h` are
   rejected pre-CDB on the LS-9000, and that X-crop is accepted (proving the
   rule reads C1h rather than assuming the LS-5000 restriction).
6. **Regression corpus** — replay the six sessions in
   `/mnt/storage/NikonScanDecomp/scan_captures/` (varying multi-sample, CCD
   format, DPI, crop, manual gain) through the new decode path; compare against
   the existing TIFFs.
7. LS-5000-family behaviour (X-crop refusal, 512-byte padding, perforation
   boundaries) can't be exercised without that hardware — implement to spec,
   cover with unit tests over synthesised VPD pages.

## Follow-ups

- Rewrite the `ls9000ed-window-vendor-bytes` memory: bytes 42/44 were
  misidentified and the "vendor tail" framing is wrong — it's all documented.
  It also conflates two mechanisms — byte 41's Averaging is about the
  *multiple-reading* count in byte 40 (`E1h` `AVERAGING`, ASCQ `02h`), while CCD
  line registration is byte 44's interleaving (`MULTI_LINE`, ASCQ `04h`). The
  4000-dpi captures move both at once, which is what made them look like one
  thing.
- ~~Record that IR (window ID 9) is absent from both official specs~~ — done, and
  the device reports it directly. See "What the hardware said".
- Find where `scsiscan.sys` puts TSC. Same experiment: pull the holder, expect
  `02h-3Ah-00h-01h`, and look for the `01h` in the 32-byte buffer. Byte 15 would
  mean both stacks repack identically; byte 18+ would explain that driver's
  `SENSE_LENGTH = 32`.
- Settle whether `D1h`'s dropout-colour bit means anything here. On a film
  scanner it would be "read one channel, return one greyscale plane", which is
  useful — but Nikon Scan sent R-G-B in all 128 captured descriptors, so it was
  never exercised.
