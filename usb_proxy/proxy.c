/*
 * NKDSBP2.dll proxy — logs NkDriverEntry calls AND SCSI CDBs.
 *
 * NkDriverEntry: __stdcall, 3 params (12 bytes), confirmed from RET 0xc.
 *   int NkDriverEntry(DWORD op, DWORD param2, DWORD param3)
 *
 * SCSI capture: IAT-hooks DeviceIoControl in Nkdsbp2_real.dll ONLY (not the
 * whole process), so only calls originating from the real Nikon transport DLL
 * are intercepted. This avoids the global inline-patch problems seen earlier.
 *
 * For IOCTL_SCSISCAN_CMD (0x190012) the SCSISCAN_CMD control struct carries:
 *   - Cdb[16]        : the SCSI command descriptor block
 *   - Direction      : 0=none, 0x40=IN(device->host), 0x80=OUT(host->device)
 *   - TransferLength : byte count of the data buffer
 *
 * Data-OUT (0x80, e.g. SET WINDOW) bytes are logged BEFORE the call.
 * Data-IN  (0x40, e.g. GET WINDOW / READ) bytes are logged AFTER the call.
 *
 * Overlapped I/O handling:
 *   The Nikon driver issues ALL SCSI commands via overlapped (async) I/O —
 *   DeviceIoControl returns FALSE with ERROR_IO_PENDING. Small transfers
 *   (INQUIRY, GET WINDOW, etc.) complete fast enough that the buffer is
 *   filled by the time our hook reads it. Large transfers (image reads,
 *   376KB-804KB) do NOT complete in time — the DMA is still in flight when
 *   we try to read lpOutBuffer. The fix: after calling realDeviceIoControl,
 *   if the call returned FALSE (pending) and lpOverlapped is provided, we
 *   POLL HasOverlappedIoCompleted(lpOverlapped) (reads OVERLAPPED.Internal,
 *   set directly by the OS) until it flips, instead of waiting on the
 *   completion event ourselves. Waiting on the event would consume it
 *   (auto-reset) or force us to manually re-signal it for the caller —
 *   either way we'd be perturbing a synchronization object the caller also
 *   depends on. Polling never touches it, so the caller's own subsequent
 *   wait on that event behaves exactly as it would against the real DLL.
 *   We also save/restore GetLastError() around all of this — WriteFile,
 *   Sleep, etc. all clobber it as a side effect, and the caller's overlapped
 *   pattern depends on seeing ERROR_IO_PENDING survive on a pending call.
 *   (An earlier version of this fix used WaitForSingleObject + SetEvent and
 *   didn't preserve last-error — it broke device detection entirely: every
 *   SCSI command succeeded, but Nikon Scan saw a clobbered error code after
 *   each one and gave up, reporting "device not found".)
 *
 * Build:
 *   i686-w64-mingw32-gcc -shared -O2 -o NKDSBP2.dll proxy.c \
 *       -lkernel32 -Wl,--enable-stdcall-fixup nkdsbp2.def
 */

#define WIN32_LEAN_AND_MEAN
#include <stdarg.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#include <windows.h>

/* GetProcAddress returns FARPROC; casting to a specific function pointer type
 * is the documented usage but trips -Wcast-function-type. */
#pragma GCC diagnostic ignored "-Wcast-function-type"

/* IOCTL_SCSISCAN_CMD = FILE_DEVICE_SCANNER(0x22) << 16 | METHOD_OUT_DIRECT(2) |
 * function(4) << 2 */
#define IOCTL_SCSISCAN_CMD 0x00190012

/* SCSISCAN_CMD struct layout — from Microsoft's scsiscan.h (Windows DDK).
 * This is the PUBLIC spec, not reverse-engineered. */
#pragma pack(push, 4)
typedef struct {
  ULONG Reserved1;      /* +0x00                                   */
  ULONG Size;           /* +0x04: sizeof(SCSISCAN_CMD) = 0x2C      */
  ULONG SrbFlags;       /* +0x08: SRB_FLAGS_DATA_IN=0x40, _OUT=0x80 */
  UCHAR CdbLength;      /* +0x0C: 6, 10, or 16                     */
  UCHAR SenseLength;    /* +0x0D: sense buffer size                */
  UCHAR Reserved2;      /* +0x0E                                   */
  UCHAR Reserved3;      /* +0x0F                                   */
  ULONG TransferLength; /* +0x10: byte count of data buffer        */
  UCHAR Cdb[16];        /* +0x14: SCSI CDB                         */
  PUCHAR pSrbStatus;    /* +0x24: ptr to SRB status byte           */
  PUCHAR pSenseBuffer;  /* +0x28: ptr to sense data buffer         */
} SCSISCAN_CMD;
#pragma pack(pop)

typedef struct {
  DWORD data_buffer_ls;   // +0x00: data buffer pointer (set by LS5000.md3, NOT
                          // read by NKDUSCAN)
  DWORD direction;        // +0x04: 1=data-in, 2=data-out, other=no-data
  DWORD cdb_size;         // +0x08: CDB size (0x20=standard, 0x40=extended)
  DWORD callback;         // +0x0C: error callback function pointer (or NULL)
  DWORD callback_context; // +0x10: callback context (command object pointer,
                          // set by FUN_100ae410)
  DWORD cdb_length;       // +0x14: CDB length (from CDB builder return value)
  DWORD cdb_data;         // +0x18: pointer to CDB bytes
  DWORD transfer_length;  // +0x1C: total data transfer size in bytes
  DWORD data_buffer;      // +0x20: pointer to data buffer (used by NKDUSCAN for
                          // ReadFile/WriteFile)
  DWORD sense_buffer_size; // +0x24: sense data buffer size constant (0x20)
  DWORD additional_params; // +0x28: pointer to additional params buffer
} Op5CommandParams;

/* SRB flag bits (from scsiscan.h) */
#define SRB_FLAGS_DATA_IN 0x00000040
#define SRB_FLAGS_DATA_OUT 0x00000080

/* --- Globals --- */
static HANDLE g_scannerHandle = INVALID_HANDLE_VALUE;
static HANDLE g_logFile = INVALID_HANDLE_VALUE; /* human-readable text trace */
static HANDLE g_binFile =
    INVALID_HANDLE_VALUE; /* full binary trace (untruncated) */
static HMODULE g_realDll = NULL;
static DWORD g_seq = 0; /* monotonic call index for log correlation */

/* 3 params, __stdcall — matches RET 0xc in the real DLL */
typedef int(WINAPI *pNkDriverEntry)(DWORD op, DWORD param2, DWORD param3);

/* Real DeviceIoControl pointer (resolved via GetProcAddress; IAT slot is
 * overwritten to point at our hook). */
typedef BOOL(WINAPI *pDeviceIoControl)(HANDLE hDevice, DWORD dwIoControlCode,
                                       LPVOID lpInBuffer, DWORD nInBufferSize,
                                       LPVOID lpOutBuffer, DWORD nOutBufferSize,
                                       LPDWORD lpBytesReturned,
                                       LPOVERLAPPED lpOverlapped);

typedef HANDLE(WINAPI *pCreateFileA)(LPCSTR, DWORD, DWORD,
                                     LPSECURITY_ATTRIBUTES, DWORD, DWORD,
                                     HANDLE);

typedef HANDLE(WINAPI *pCreateFileW)(LPCWSTR, DWORD, DWORD,
                                     LPSECURITY_ATTRIBUTES, DWORD, DWORD,
                                     HANDLE);

typedef BOOL(WINAPI *pReadFile)(HANDLE, LPVOID, DWORD, LPDWORD, LPOVERLAPPED);

typedef BOOL(WINAPI *pWriteFile)(HANDLE, LPCVOID, DWORD, LPDWORD, LPOVERLAPPED);

static pNkDriverEntry g_realNkDriverEntry = NULL;

static pDeviceIoControl realDeviceIoControl = NULL;
static pCreateFileA realCreateFileA = NULL;
static pCreateFileW realCreateFileW = NULL;
static pReadFile realReadFile = NULL;
static pWriteFile realWriteFile = NULL;

static LARGE_INTEGER g_qpcFreq = {0};
static LARGE_INTEGER g_qpcStart = {0};

void timestamp_init(void) {
  QueryPerformanceFrequency(&g_qpcFreq);
  QueryPerformanceCounter(&g_qpcStart);
}

static double timestamp_ms(void) {
  LARGE_INTEGER now;
  QueryPerformanceCounter(&now);

  return (double)(now.QuadPart - g_qpcStart.QuadPart) * 1000.0 /
         (double)g_qpcFreq.QuadPart;
}

/* --- Logging helpers --- */

static void log_write(const char *fmt, ...) {
  if (g_logFile == INVALID_HANDLE_VALUE)
    return;
  char buf[4096];

  double ms = timestamp_ms();

  SYSTEMTIME st;
  GetLocalTime(&st);

  int prefix = snprintf(buf, sizeof(buf), "[%02d:%02d:%02d.%03d +%10.3fms] ",
                        st.wHour, st.wMinute, st.wSecond, st.wMilliseconds, ms);

  va_list args;
  va_start(args, fmt);
  vsnprintf(buf + prefix, sizeof(buf) - prefix, fmt, args);
  va_end(args);

  DWORD written;
  WriteFile(g_logFile, buf, strlen(buf), &written, NULL);
  FlushFileBuffers(g_logFile);
}

static void log_printf(const char *fmt, ...) {
  if (g_logFile == INVALID_HANDLE_VALUE)
    return;
  char buf[4096];
  va_list args;
  va_start(args, fmt);
  vsnprintf(buf, sizeof(buf), fmt, args);
  va_end(args);

  DWORD written;
  WriteFile(g_logFile, buf, strlen(buf), &written, NULL);
  FlushFileBuffers(g_logFile);
}

static void log_hex(const char *prefix, const BYTE *data, int len) {
  if (g_logFile == INVALID_HANDLE_VALUE)
    return;
  if (data == NULL || len <= 0)
    return;
  if (IsBadReadPtr(data, len))
    return; /* avoid crashes on bad pointers */
  char line[512];
  int pos = 0;
  pos += wsprintfA(line + pos, "%s (%d bytes):", prefix, len);
  for (int i = 0; i < len && pos < 500; i++) {
    pos += wsprintfA(line + pos, " %02X", data[i]);
  }
  pos += wsprintfA(line + pos, "\r\n");
  DWORD written;
  WriteFile(g_logFile, line, pos, &written, NULL);
  FlushFileBuffers(g_logFile);
}

static void dump_sense(const unsigned char *sense, int len) {
  if (!sense || len < 14)
    return;

  log_write("SENSE: ");

  for (int i = 0; i < len; i++)
    log_write("%02X ", sense[i]);

  log_write("\n");

  unsigned char response = sense[0] & 0x7f;
  unsigned char key = sense[2] & 0x0f;
  unsigned char asc = sense[12];
  unsigned char ascq = sense[13];

  log_write("  response=0x%02X key=0x%02X ASC=0x%02X ASCQ=0x%02X\n", response,
            key, asc, ascq);

  switch (key) {
  case 0x00:
    log_write("  NO SENSE\n");
    break;

  case 0x02:
    log_write("  NOT READY\n");
    break;

  case 0x06:
    log_write("  UNIT ATTENTION\n");
    break;

  case 0x05:
    log_write("  ILLEGAL REQUEST\n");
    break;

  default:
    log_write("  UNKNOWN SENSE KEY\n");
    break;
  }

  if (asc == 0x3A && ascq == 0x00)
    log_write("  -> NO MEDIUM\n");

  if (asc == 0x28 && ascq == 0x00)
    log_write("  -> MEDIUM CHANGED\n");
}

static void decode_nikon_vendor(const unsigned char *cdb, int len, int write) {
  if (len < 4)
    return;

  unsigned char reg = cdb[2];

  switch (reg) {

  case 0x80:
    log_write("    register 0x80: LAMP\n");
    break;

  case 0x91:
    log_write("    register 0x91: STATUS?\n");
    break;

  case 0xA0:
    log_write("    register 0xA0: AUTOFOCUS\n");
    break;

  case 0xB4:
    log_write("    register 0xB4: UNKNOWN CONTROL\n");
    break;

  case 0xC1:
    log_write("    register 0xC1: FOCUS POSITION\n");
    break;

  case 0xD0:
    log_write("    register 0xD0: EJECT\n");
    break;

  case 0xF0:
    log_write("    register 0xF0: UNKNOWN\n");
    break;

  default:
    log_write("    register 0x%02X unknown\n", reg);
    break;
  }

  if (len >= 10) {
    unsigned int transfer = ((unsigned int)cdb[6] << 16) |
                            ((unsigned int)cdb[7] << 8) | (unsigned int)cdb[8];

    log_write("    transfer=%u bytes\n", transfer);
  }
}

static void dump_cdb(const unsigned char *cdb, int len) {
  if (!cdb || len == 0)
    return;

  log_write("CDB: ");

  for (int i = 0; i < len; i++)
    log_printf("%02X ", cdb[i]);

  log_printf("\n");

  switch (cdb[0]) {

  case 0x00:
    log_write("  TEST UNIT READY\n");
    break;

  case 0x03:
    log_write("  REQUEST SENSE\n");
    if (len >= 6)
      log_write("    alloc_len=%u\n", cdb[4]);
    break;

  case 0x12:
    log_write("  INQUIRY\n");
    if (len >= 6)
      log_write("    alloc_len=%u\n", cdb[4]);
    break;

  case 0x15:
    log_write("  MODE SELECT(6)\n");
    break;

  case 0x25:
    log_write("  READ CAPACITY(10)\n");
    break;

  case 0x28:
    log_write("  READ(10)\n");
    if (len >= 10) {
      unsigned int lba = ((unsigned int)cdb[2] << 24) |
                         ((unsigned int)cdb[3] << 16) |
                         ((unsigned int)cdb[4] << 8) | (unsigned int)cdb[5];

      unsigned int blocks = ((unsigned int)cdb[7] << 8) | (unsigned int)cdb[8];

      log_write("    LBA=%u blocks=%u\n", lba, blocks);
    }
    break;

  case 0x2A:
    log_write("  WRITE(10)\n");
    break;

  case 0xE0:
    log_write("  NIKON VENDOR WRITE\n");
    decode_nikon_vendor(cdb, len, 1);
    break;

  case 0xE1:
    log_write("  NIKON VENDOR READ\n");
    decode_nikon_vendor(cdb, len, 0);
    break;

  default:
    log_write("  UNKNOWN opcode\n");
    break;
  }
}

/* Append one COMPLETE SCSI exchange to the binary trace — no truncation, so
 * 32 KB LUT writes and full image reads are captured intact (the text log's
 * log_hex still caps its line, which is why we need this).
 *
 * Record layout (all integers little-endian):
 *   "SREC"        4 bytes  magic
 *   seq           u32      correlates with the text log's [#N]
 *   cdb_len       u8
 *   direction     u8       0=none, 1=IN (device->host), 2=OUT (host->device)
 *   srb_status    u8
 *   sense_len     u8
 *   srb_flags     u32
 *   transfer_len  u32
 *   result        i32      DeviceIoControl BOOL return
 *   cdb           16 bytes
 *   data_len      u32
 *   data          data_len bytes  (the full SCSI data buffer)
 *   sense         sense_len bytes
 *
 * Parsed by the Rust nktrace/nkdiff tooling (see docs/PLAN.md). */
static void bin_write_scsi(DWORD seq, const SCSISCAN_CMD *cmd, BOOL result,
                           const BYTE *data, DWORD dataLen, BYTE srbStatus,
                           const BYTE *sense, DWORD senseLen) {
  if (g_binFile == INVALID_HANDLE_VALUE)
    return;
  DWORD w;
  BYTE dir = (cmd->SrbFlags & SRB_FLAGS_DATA_IN)    ? 1
             : (cmd->SrbFlags & SRB_FLAGS_DATA_OUT) ? 2
                                                    : 0;
  LONG res = (LONG)result;
  BYTE meta[4] = {cmd->CdbLength, dir, srbStatus, (BYTE)senseLen};

  WriteFile(g_binFile, "SREC", 4, &w, NULL);
  WriteFile(g_binFile, &seq, 4, &w, NULL);
  WriteFile(g_binFile, meta, 4, &w, NULL);
  WriteFile(g_binFile, &cmd->SrbFlags, 4, &w, NULL);
  WriteFile(g_binFile, &cmd->TransferLength, 4, &w, NULL);
  WriteFile(g_binFile, &res, 4, &w, NULL);
  WriteFile(g_binFile, cmd->Cdb, 16, &w, NULL);
  WriteFile(g_binFile, &dataLen, 4, &w, NULL);
  if (data && dataLen)
    WriteFile(g_binFile, data, dataLen, &w, NULL);
  if (sense && senseLen)
    WriteFile(g_binFile, sense, senseLen, &w, NULL);
  FlushFileBuffers(g_binFile);
}

/* --- Hooked DeviceIoControl ---
 * Only installed in Nkdsbp2_real.dll's IAT, so only the real Nikon transport
 * DLL's DeviceIoControl calls are intercepted. Intercepts IOCTL_SCSISCAN_CMD
 * and dumps the CDB + data buffer (data-OUT before, data-IN after).
 *
 * With METHOD_OUT_DIRECT, the SCSI data buffer is lpOutBuffer (the driver
 * maps it via MDL for either direction). Direction comes from SrbFlags.
 *
 * Overlapped I/O handling: The Nikon driver issues ALL SCSI commands via
 * overlapped (async) I/O. DeviceIoControl returns FALSE (pending). Small
 * transfers complete fast enough that the buffer is filled when we read it.
 * Large transfers (image reads, 376KB-804KB) do NOT complete in time.
 * Fix: for data-IN commands, if the call returned FALSE and lpOverlapped is
 * provided, wait on the overlapped event before reading the buffer. We then
 * re-signal the event so the caller's own WaitForSingleObject also succeeds. */
static BOOL WINAPI hookedDeviceIoControl(HANDLE hDevice, DWORD dwIoControlCode,
                                         LPVOID lpInBuffer, DWORD nInBufferSize,
                                         LPVOID lpOutBuffer,
                                         DWORD nOutBufferSize,
                                         LPDWORD lpBytesReturned,
                                         LPOVERLAPPED lpOverlapped) {
  log_write("DeviceIoControl called code=%08lX\r\n", dwIoControlCode);
  BOOL result;
  DWORD savedErr = 0;
  DWORD seq = ++g_seq;

  if (dwIoControlCode == IOCTL_SCSISCAN_CMD && lpInBuffer &&
      nInBufferSize >= sizeof(SCSISCAN_CMD)) {
    SCSISCAN_CMD *cmd = (SCSISCAN_CMD *)lpInBuffer;
    BOOL isDataIn = (cmd->SrbFlags & SRB_FLAGS_DATA_IN) != 0;
    BOOL isDataOut = (cmd->SrbFlags & SRB_FLAGS_DATA_OUT) != 0;

    log_write("[#%lu] --- IOCTL_SCSISCAN_CMD ---\r\n", seq);
    log_write("  CdbLength=%d SrbFlags=0x%08X (dir:%s) TransferLen=%d "
              "InBufSize=%d OutBufSize=%d\r\n",
              cmd->CdbLength, cmd->SrbFlags,
              isDataIn ? "IN" : (isDataOut ? "OUT" : "none"),
              cmd->TransferLength, nInBufferSize, nOutBufferSize);
    log_hex("  CDB", cmd->Cdb, cmd->CdbLength);

    /* Data-OUT (host -> device, e.g. SET WINDOW): dump BEFORE the call.
     * The data is in lpOutBuffer (METHOD_OUT_DIRECT uses it for both
     * directions — the driver reads from or writes to this buffer). */
    if (isDataOut && cmd->TransferLength > 0 && lpOutBuffer) {
      int dumpLen = (int)cmd->TransferLength;
      if (dumpLen > 512)
        dumpLen = 512;
      log_hex("  DATA-OUT", (BYTE *)lpOutBuffer, dumpLen);
    }

    /* Call the real DeviceIoControl */
    result = realDeviceIoControl(hDevice, dwIoControlCode, lpInBuffer,
                                 nInBufferSize, lpOutBuffer, nOutBufferSize,
                                 lpBytesReturned, lpOverlapped);
    /* Capture last-error IMMEDIATELY — every Win32 call below (Sleep,
     * WriteFile, FlushFileBuffers, ...) resets the thread's last-error
     * as a side effect. The caller's overlapped pattern is
     * `if (!DeviceIoControl(...)) { if (GetLastError()==ERROR_IO_PENDING)
     * wait; else fail; }` — if anything overwrites it before we return,
     * the caller sees a bogus non-pending error on a perfectly good
     * pending I/O and treats it as a hard failure. This is what broke
     * device detection: every INQUIRY succeeded at the SCSI level but
     * Nikon Scan saw a clobbered error code and gave up. Restored right
     * before the return at the bottom of this function. */
    savedErr = GetLastError();

    /* For data-IN commands: wait for overlapped completion before reading.
     * The Nikon driver uses overlapped I/O for ALL SCSI commands —
     * DeviceIoControl returns FALSE (pending). Small transfers (INQUIRY,
     * GET WINDOW) complete fast enough that the buffer is already filled.
     * Large transfers (image reads) do NOT complete in time.
     *
     * Poll HasOverlappedIoCompleted() (reads OVERLAPPED.Internal, set
     * directly by the OS) instead of waiting on the completion event.
     * Waiting on the event ourselves would CONSUME it (auto-reset) or
     * require manually re-signaling it for the caller — either way we'd
     * be perturbing a synchronization object the caller also depends on.
     * Polling never touches it: the OS still signals the event/file
     * handle on its own, so the caller's own wait afterward sees it
     * completed normally, exactly as with the real DLL. */
    if (isDataIn && !result && lpOverlapped) {
      DWORD start = GetTickCount();
      while (!HasOverlappedIoCompleted(lpOverlapped)) {
        if (GetTickCount() - start > 10000)
          break; /* 10s safety cap */
        Sleep(1);
      }
      savedErr = GetLastError();
    }

    /* actualLen = the TRUE number of bytes the device wrote into
     * lpOutBuffer, which is NOT the same as cmd->TransferLength (the
     * requested size) on short transfers — e.g. the final chunk of an
     * image read, which legitimately ends with a CHECK CONDITION
     * SK=0x0B/ASC=0x4B short-transfer status. Trusting TransferLength
     * there means dumping/recording the tail of the buffer that the
     * device never wrote — exactly the zero-padding bug already found
     * (and fixed) on the Rust transport side. For an overlapped call
     * that completed (per the poll above), OVERLAPPED.InternalHigh
     * already holds the real transferred byte count — read it directly
     * instead of calling GetOverlappedResult() (which would reset
     * last-error again for no benefit). For a call that completed
     * synchronously, lpBytesReturned is already authoritative. */
    DWORD actualLen = cmd->TransferLength;
    if (isDataIn) {
      if (!result && lpOverlapped) {
        if (HasOverlappedIoCompleted(lpOverlapped)) {
          actualLen = (DWORD)lpOverlapped->InternalHigh;
        }
      } else if (lpBytesReturned) {
        actualLen = *lpBytesReturned;
      }
    }

    /* Data-IN (device -> host, e.g. GET WINDOW / READ): dump AFTER.
     * By now the overlapped I/O has completed (we polled above). */
    if (isDataIn && actualLen > 0 && lpOutBuffer) {
      int dumpLen = (int)actualLen;
      if (dumpLen > 512)
        dumpLen = 512;
      log_hex("  DATA-IN", (BYTE *)lpOutBuffer, dumpLen);
    }

    /* Sense data: pSenseBuffer is a pointer to a buffer of SenseLength
     * bytes. Dereference it carefully. */
    if (cmd->pSenseBuffer && cmd->SenseLength > 0 &&
        !IsBadReadPtr(cmd->pSenseBuffer, cmd->SenseLength)) {
      log_hex("  SENSE", cmd->pSenseBuffer, cmd->SenseLength);
    }

    /* SRB status: pSrbStatus points to a single status byte. */
    BYTE srbStatus = 0;
    if (cmd->pSrbStatus && !IsBadReadPtr(cmd->pSrbStatus, 1)) {
      srbStatus = *cmd->pSrbStatus;
      log_write("  SRB_Status=0x%02X\r\n", srbStatus);
    }

    /* Full untruncated binary record — read buffer inline.
     * By now the DMA has completed (we polled for it above). */
    {
      const BYTE *sensePtr = NULL;
      DWORD senseLen = 0;
      if (cmd->pSenseBuffer && cmd->SenseLength > 0 &&
          !IsBadReadPtr(cmd->pSenseBuffer, cmd->SenseLength)) {
        sensePtr = cmd->pSenseBuffer;
        senseLen = cmd->SenseLength;
      }

      /* isDataOut still uses cmd->TransferLength: that buffer was
       * prepared by the host itself before the call, so its full
       * requested length is always valid (no DMA race on writes). */
      DWORD outLen = isDataOut ? cmd->TransferLength : actualLen;
      const BYTE *dataPtr = NULL;
      DWORD dataLen = 0;
      if ((isDataIn || isDataOut) && outLen > 0 && lpOutBuffer &&
          !IsBadReadPtr(lpOutBuffer, outLen)) {
        dataPtr = (const BYTE *)lpOutBuffer;
        dataLen = outLen;
      }
      bin_write_scsi(seq, cmd, result, dataPtr, dataLen, srbStatus, sensePtr,
                     senseLen);
    }

    DWORD returned =
        isDataIn ? actualLen : (lpBytesReturned ? *lpBytesReturned : 0);
    log_write("  Result=%d BytesReturned=%d\r\n\r\n", result, returned);
  } else {
    /* Not our IOCTL — pass straight through, no logging noise. */
    result = realDeviceIoControl(hDevice, dwIoControlCode, lpInBuffer,
                                 nInBufferSize, lpOutBuffer, nOutBufferSize,
                                 lpBytesReturned, lpOverlapped);
    savedErr = GetLastError();
  }

  /* Restore whatever last-error the real call (or our completion poll)
   * produced, undoing any clobbering from the logging calls above. */
  SetLastError(savedErr);
  return result;
}

HANDLE WINAPI hookedCreateFileA(LPCSTR name, DWORD access, DWORD share,
                                LPSECURITY_ATTRIBUTES sec, DWORD creation,
                                DWORD flags, HANDLE template) {
  log_write("CreateFileA(%s)\r\n", name);
  HANDLE h =
      realCreateFileA(name, access, share, sec, creation, flags, template);

  if (h != INVALID_HANDLE_VALUE && strstr(name, "usbscan") != NULL) {
    g_scannerHandle = h;
    log_write("Scanner opened: %s handle=%p\n", name, h);
  }

  return h;
}

HANDLE WINAPI hookedCreateFileW(LPCWSTR name, DWORD access, DWORD share,
                                LPSECURITY_ATTRIBUTES sec, DWORD creation,
                                DWORD flags, HANDLE template) {
  HANDLE h =
      realCreateFileW(name, access, share, sec, creation, flags, template);

  if (h != INVALID_HANDLE_VALUE && wcsstr(name, L"usbscan")) {
    g_scannerHandle = h;
    log_write("Scanner opened (W) handle=%p\r\n", h);
  }

  return h;
}

BOOL WINAPI hookedWriteFile(HANDLE h, LPCVOID buffer, DWORD len,
                            LPDWORD written, LPOVERLAPPED ov) {
  if (h == g_scannerHandle)
    log_hex("USB OUT", buffer, len);

  return realWriteFile(h, buffer, len, written, ov);
}

BOOL WINAPI hookedReadFile(HANDLE h, LPVOID buffer, DWORD len, LPDWORD read,
                           LPOVERLAPPED ov) {
  BOOL r = realReadFile(h, buffer, len, read, ov);

  if (h == g_scannerHandle && r)
    log_hex("USB IN", buffer, *read);

  return r;
}

/* --- IAT hook installation --- */
static void hookIat(HMODULE mod, const char *dll, const char *func,
                    FARPROC replacement, FARPROC *original) {
  log_write("Scanning imports of %p for %s!%s\r\n", mod, dll, func);

  BYTE *base = (BYTE *)mod;
  IMAGE_DOS_HEADER *dos = (IMAGE_DOS_HEADER *)base;
  IMAGE_NT_HEADERS *nt = (IMAGE_NT_HEADERS *)(base + dos->e_lfanew);

  DWORD rva = nt->OptionalHeader.DataDirectory[IMAGE_DIRECTORY_ENTRY_IMPORT]
                  .VirtualAddress;

  IMAGE_IMPORT_DESCRIPTOR *imp = (IMAGE_IMPORT_DESCRIPTOR *)(base + rva);

  for (; imp->Name; imp++) {

    const char *name = (char *)(base + imp->Name);

    log_write("Import DLL: %s\r\n", name);

    if (lstrcmpiA(name, dll))
      continue;

    IMAGE_THUNK_DATA *orig =
        (IMAGE_THUNK_DATA *)(base + imp->OriginalFirstThunk);

    IMAGE_THUNK_DATA *iat = (IMAGE_THUNK_DATA *)(base + imp->FirstThunk);

    for (; orig->u1.AddressOfData; orig++, iat++) {

      IMAGE_IMPORT_BY_NAME *entry =
          (IMAGE_IMPORT_BY_NAME *)(base + orig->u1.AddressOfData);

      if (strcmp((char *)entry->Name, func) != 0)
        continue;

      *original = (FARPROC)(UINT_PTR)iat->u1.Function;

      DWORD old;
      VirtualProtect(&iat->u1.Function, sizeof(iat->u1.Function),
                     PAGE_READWRITE, &old);

      iat->u1.Function = (ULONG_PTR)replacement;

      VirtualProtect(&iat->u1.Function, sizeof(iat->u1.Function), old, &old);

      FlushInstructionCache(GetCurrentProcess(), &iat->u1.Function,
                            sizeof(iat->u1.Function));

      log_write("Hooked %s!%s\r\n", dll, func);
      return;
    }
  }

  log_write("Could not hook %s!%s\r\n", dll, func);
}

/* --- Exported NkDriverEntry (3 params, __stdcall) --- */
/* Opcodes
 * (https://github.com/kevihiiin/Nikon-Coolscan-RE/blob/main/docs/kb/components/nkduscan/api.md)
 * 1 - initialize/open session
 * 2 - close session
 * 3 - close command
 * 4 - release resource
 * 5 - execute SCSI command
 * 6 - get command status
 * 7 - shutdown/release all
 * 8 - query command
 * 9 - execute and retrieve result
 */
__declspec(dllexport) int WINAPI NkDriverEntry(DWORD op, DWORD param2,
                                               DWORD param3) {
  DWORD seq = ++g_seq;

  log_write("[#%lu] NkDriverEntry(op=%lu, param2=0x%08lX, param3=0x%08lX)\r\n",
            seq, op, param2, param3);

  if (!g_realNkDriverEntry) {
    log_write("ERROR: real NkDriverEntry missing\r\n");
    return -1;
  }

  int result = 0;

  if (op == 5) {
    Op5CommandParams *p = (Op5CommandParams *)param2;

    SCSISCAN_CMD fake = {0};

    fake.CdbLength = p->cdb_length;
    fake.SrbFlags = p->direction == 1   ? SRB_FLAGS_DATA_IN
                    : p->direction == 2 ? SRB_FLAGS_DATA_OUT
                                        : 0;
    fake.TransferLength = p->transfer_length;

    memcpy(fake.Cdb, (void *)p->cdb_data,
           p->cdb_length > sizeof(fake.Cdb) ? sizeof(fake.Cdb) : p->cdb_length);

    log_write("[#%lu] --- FC05 SCSI CMD ---\r\n", seq);
    log_write("  CdbLength=%d SrbFlags=0x%08X (dir:%s) TransferLen=%d ",
              fake.CdbLength, fake.SrbFlags,
              p->direction == 1   ? "IN"
              : p->direction == 2 ? "OUT"
                                  : "none",
              fake.TransferLength);
    dump_cdb(fake.Cdb, fake.CdbLength);

    if (p->direction == 1) {
      result = g_realNkDriverEntry(op, param2, param3);

      log_write("  DATA IN (%lu bytes): ", p->transfer_length);

      for (DWORD i = 0; i < p->transfer_length; i++) {
        log_printf("%02X ", ((BYTE *)p->data_buffer)[i]);
      }
      log_printf("\r\n");

      bin_write_scsi(seq, &fake, result, (BYTE *)p->data_buffer,
                     p->transfer_length, 0, NULL, 0);

      return result;
    }

    log_write("  DATA OUT (%lu bytes): ", p->transfer_length);

    for (DWORD i = 0; i < p->transfer_length; i++) {
      log_printf("%02X ", ((BYTE *)p->data_buffer)[i]);
    }
    log_printf("\r\n");

    bin_write_scsi(seq, &fake, 0, (BYTE *)p->data_buffer, p->transfer_length, 0,
                   NULL, 0);

    result = g_realNkDriverEntry(op, param2, param3);

    return result;
  }

  result = g_realNkDriverEntry(op, param2, param3);

  log_write("  -> result=%d\r\n", result);

  return result;
}

/* --- DLL lifecycle --- */

static BOOL buildRealDllPath(HINSTANCE hinstDLL, char *outPath,
                             DWORD outPathSize) {
  char path[MAX_PATH];
  DWORD len = GetModuleFileNameA(hinstDLL, path, MAX_PATH);
  if (len == 0 || len == MAX_PATH)
    return FALSE;
  char *slash = strrchr(path, '\\');
  if (!slash)
    return FALSE;
  *(slash + 1) = '\0';
  lstrcpynA(outPath, path, outPathSize);
  lstrcatA(outPath, "NKDUSCAN_real.dll");
  return TRUE;
}

BOOL WINAPI DllMain(HINSTANCE hinstDLL, DWORD fdwReason, LPVOID lpvReserved) {
  (void)lpvReserved;
  switch (fdwReason) {
  case DLL_PROCESS_ATTACH: {
    g_logFile = CreateFileA("C:\\usb_trace.log", GENERIC_WRITE, FILE_SHARE_READ,
                            NULL, CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL, NULL);
    g_binFile = CreateFileA("C:\\usb_trace.bin", GENERIC_WRITE, FILE_SHARE_READ,
                            NULL, CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL, NULL);

    timestamp_init();

    log_write("=== NKDUSCAN.dll proxy (IAT-hook build) ===\r\n");

    char path[MAX_PATH];
    if (buildRealDllPath(hinstDLL, path, sizeof(path))) {
      log_write("Real DLL: %s\r\n", path);
      g_realDll = LoadLibraryA(path);
    }
    if (!g_realDll)
      g_realDll = LoadLibraryA("NKDUSCAN_real.dll");
    if (!g_realDll) {
      log_write("ERROR: Could not load NKDUSCAN_real.dll, GetLastError=%d\r\n",
                GetLastError());
      log_write("=== Proxy unloaded (load failure) ===\r\n");
      CloseHandle(g_logFile);
      g_logFile = INVALID_HANDLE_VALUE;
      return FALSE;
    }
    log_write("Loaded real DLL at 0x%08X\r\n", (DWORD)(UINT_PTR)g_realDll);

    g_realNkDriverEntry =
        (pNkDriverEntry)GetProcAddress(g_realDll, "NkDriverEntry");
    if (!g_realNkDriverEntry)
      g_realNkDriverEntry =
          (pNkDriverEntry)GetProcAddress(g_realDll, (LPCSTR)1);
    if (!g_realNkDriverEntry) {
      log_write("ERROR: NkDriverEntry not found in real DLL\r\n");
      return FALSE;
    }
    log_write("NkDriverEntry: 0x%08X\r\n",
              (DWORD)(UINT_PTR)g_realNkDriverEntry);

    hookIat(g_realDll, "kernel32.dll", "CreateFileA",
            (FARPROC)hookedCreateFileA, (FARPROC *)&realCreateFileA);

    hookIat(g_realDll, "kernel32.dll", "CreateFileW",
            (FARPROC)hookedCreateFileW, (FARPROC *)&realCreateFileW);

    hookIat(g_realDll, "kernel32.dll", "ReadFile", (FARPROC)hookedReadFile,
            (FARPROC *)&realReadFile);

    hookIat(g_realDll, "kernel32.dll", "WriteFile", (FARPROC)hookedWriteFile,
            (FARPROC *)&realWriteFile);

    hookIat(g_realDll, "kernel32.dll", "DeviceIoControl",
            (FARPROC)hookedDeviceIoControl, (FARPROC *)&realDeviceIoControl);

    log_write("=== Ready ===\r\n\r\n");
    break;
  }
  case DLL_PROCESS_DETACH:
    log_write("=== Proxy unloaded (seq=%lu) ===\r\n", g_seq);
    if (g_logFile != INVALID_HANDLE_VALUE)
      CloseHandle(g_logFile);
    if (g_binFile != INVALID_HANDLE_VALUE)
      CloseHandle(g_binFile);
    if (g_realDll)
      FreeLibrary(g_realDll);
    break;
  }
  return TRUE;
}
