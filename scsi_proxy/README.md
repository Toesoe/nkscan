# NKDSBP2.dll Proxy — SCSI CDB Capture

Windows proxy DLL that intercepts SCSI traffic between Nikon Scan and the
scanner. Used to capture the exact CDBs (especially SET WINDOW)
the Nikon driver sends.

## Install

1. On the Windows machine hosting the scanner, find the real NKDSBP2.dll
   (Nikon Scan installs it to
   `C:\Program Files (x86)\Common Files\Nikon\MaidMods\Scanners\`).
2. Rename it: `NKDSBP2.dll` -> `Nkdsbp2_real.dll`.
3. Copy the proxy `NKDSBP2.dll` next to `Nkdsbp2_real.dll`.
4. Run Nikon Scan (proably as administrator), trigger a scan (preview or full).
It would be useful to keep a log of the actions you perform in Nikon Scan and any buttons your press/settings you apply.
5. Check `C:\scsi_trace.log` for the CDB trace.

Please note that if you contribute these logs, they will contain image data, so make sure you don't send something sensitive.

## What it captures

The proxy writes **two files**:

- `C:\scsi_trace.log` — human-readable text (for eyeballing). Its per-line
  hex dumps are still capped at ~160 bytes, so large payloads are truncated
  here.
- `C:\scsi_trace.bin` — **complete, untruncated** binary records of every
  IOCTL_SCSISCAN_CMD exchange. This is the one to keep: it captures the full
  32 KB LUT writes and image reads that the text log truncates. Format in
  the "Binary trace format" section below.

The proxy hooks **two layers** and writes both to the text log:

1. **`NkDriverEntry` (high-level MAID dispatch)** — logs `op/param2/param3`
   for every call, plus a hex dump of the `param2`/`param3` buffers before
   and after. Same as the prior working proxy.
2. **`DeviceIoControl(IOCTL_SCSISCAN_CMD)` (low-level SCSI)** — logs the
   CDB, direction, transfer length, and the actual data buffer:
   - **Data-OUT (0x80, e.g. SET WINDOW)** dumped *before* the call — this is
     the payload we need to fix the SET WINDOW 0x26 error.
   - **Data-IN (0x40, e.g. GET WINDOW / READ)** dumped *after* the call.
   - **Sense data** if present (error responses).

Each entry is tagged with a monotonic `[#N]` sequence number so the two
layers can be correlated (the NkDriverEntry call and the DeviceIoControl
calls it triggers share a single counter).

## Build

The proxy cross-compiles to i686 Windows with the MinGW toolchain. A
dedicated cross-compiler is needed for i686-w64-mingw32-gcc.

```bash
cd tools/scsi_proxy
i686-w64-mingw32-gcc -shared -O2 -Wall -o NKDSBP2.dll \
    proxy.c -lkernel32 -Wl,--enable-stdcall-fixup nkdsbp2.def
```

The build produces a 34 KB PE32 i386 DLL. Dependency profile: imports from
`KERNEL32.dll` (all the hook helpers), `USER32.dll` (`wsprintfA`/
`wvsprintfA`), and `msvcrt.dll` (mingw runtime) — all standard Windows DLLs,
no extra runtime to ship.

## Log format

NkDriverEntry entries:
```
[#42] NkDriverEntry(op=2, param2=0x19F104, param3=0x62A2978)
  param2 (64 bytes): 00 10 00 00 ...
  param3 (64 bytes): C8 29 2A 06 ...
  -> result=0
```

SCSI CDB entries:
```
[#43] --- IOCTL_SCSISCAN_CMD ---
  CdbLength=10 SrbFlags=0x00000080 (dir:OUT) TransferLen=58 InBufSize=44 OutBufSize=58
  CDB (10 bytes): 24 00 00 00 00 00 00 00 3A 80
  DATA-OUT (58 bytes): 00 00 00 00 ...   ← SET WINDOW descriptor bytes
  SRB_Status=0x01
  Result=1 BytesReturned=0
```

The CDB opcode `0x24` is SET WINDOW. The DATA-OUT bytes that follow are the
exact window descriptor Nikon sends — this is what we need to match in Rust.

## Binary trace format (`C:\scsi_trace.bin`)

One record per IOCTL_SCSISCAN_CMD exchange, appended in order. All integers
are little-endian:

| Field | Size | Notes |
|-------|------|-------|
| magic | 4 | ASCII `"SREC"` |
| seq | u32 | correlates with the text log's `[#N]` |
| cdb_len | u8 | |
| direction | u8 | 0=none, 1=IN (dev→host), 2=OUT (host→dev) |
| srb_status | u8 | |
| sense_len | u8 | |
| srb_flags | u32 | |
| transfer_len | u32 | |
| result | i32 | DeviceIoControl BOOL return |
| cdb | 16 | fixed-width CDB |
| data_len | u32 | actual bytes captured (== transfer_len when readable) |
| data | data_len | the full SCSI data buffer — **not truncated** |
| sense | sense_len | sense buffer (Nikon uses 32 bytes) |

Unlike the text `DATA-OUT`/`DATA-IN` dumps, `data` here is the complete
payload, so a 32 KB WRITE DTC=0x03 (gamma LUT) is captured in full.
