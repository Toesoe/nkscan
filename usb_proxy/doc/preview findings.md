```markdown
# Nikon Scanner SCSI Reverse Engineering Notes

## Overview

The Nikon scanner communicates over SCSI using a mixture of standard SCSI commands and Nikon vendor-specific commands.

Current investigation focuses on the preview scan workflow and how the driver enumerates frames on a film strip.

---

# SCSI Command Decoder

A basic CDB decoder has been implemented with support for:

- TEST UNIT READY (`0x00`)
- REQUEST SENSE (`0x03`)
- INQUIRY (`0x12`)
- MODE SELECT (`0x15`)
- READ CAPACITY (`0x25`)
- READ(10) (`0x28`)
- WRITE(10) (`0x2A`)
- START STOP UNIT (`0x1B`)
- Nikon vendor commands:
  - Vendor WRITE (`0xE0`)
  - Vendor READ (`0xE1`)

The Nikon vendor decoder currently identifies subcommands from `CDB[2]`.

Known vendor subcommands:

| Register | Description |
|----------|-------------|
| `0x40` | Scan parameters |
| `0x41` | Calibration data |
| `0x42` | Gain values |
| `0x43` | Offset values |
| `0x44` | Motor position |
| `0x45` | Exposure time |
| `0x46` | Focus position |
| `0x47` | Lamp settings |
| `0x80` | Lamp on/off trigger |
| `0x81` | Motor initialization trigger |
| `0x91` | Motor step (direction + count) |
| `0xA0` | CCD setup |
| `0xB0` | State change trigger |
| `0xB1` | State change trigger |
| `0xB3` | Configuration write |
| `0xB4` | Extended configuration |
| `0xC0` | Gain calibration |
| `0xC1` | Offset calibration |
| `0xD0` | Diagnostic trigger |
| `0xD1` | Diagnostic trigger |
| `0xD2` | Diagnostic data |
| `0xD5` | Extended diagnostic |
| `0xD6` | Persistent settings |

Further work:
- Add parameter dumping for all vendor commands.
- Decode transfer length and payload contents.
- Correlate vendor writes with subsequent mechanical actions.

---

# Preview Scan Observation

A preview operation was captured for a short strip containing only four actual frames.

The scanner still appears to enumerate a full 40-frame carrier.

Observed behavior:

1. Scanner initializes.
2. Driver reads a small number of blocks from the scanner.
3. Several SCSI writes occur.
4. Frames are accessed sequentially.
5. Preview completes.
6. Empty frames are removed afterwards.

---

# Relevant Command Sequence

## Initial frame reads

Example:

```

READ(10)
LBA=2348810499 blocks=10

READ(10)
LBA=2348810755 blocks=10

READ(10)
LBA=2348811011 blocks=10

```

These appear to be preview image data blocks.

The LBA spacing:

```

2348810499
2348810755
2348811011

```

Difference:

```

256 blocks

```

This suggests a fixed-size frame allocation.

Possible interpretation:

```

frame N = base + (N * 256 blocks)

```

The preview storage area may reserve space for every possible frame regardless of whether film exists.

---

# Unknown Command

Observed:

```

CDB:
24 00 00 00 00 00 00 00 3A 80

Direction:
OUT

Transfer:
58 bytes

```

Opcode `0x24` is currently unknown.

Needs investigation.

Possibilities:

- Nikon vendor-like command using standard opcode space
- MODE-related command
- Scanner-specific metadata upload

Payload dump is required.

---

# Preview Buffer Enumeration Behavior

Later sequence:

```

READ CAPACITY(10)

```

followed by repeated:

```

READ(10)

LBA=0
TransferLen=131072

```

Repeated approximately 40 times.

The repeated reads appear to correspond to frame enumeration.

Observed:

```

frame 1
frame 2
frame 3
...
frame 40

```

The driver appears to request all 40 possible frame slots.

For empty positions, the scanner returns garbage/uninitialized data.

The Nikon software then removes empty frames after determining which slots contain valid images.

---

# Current Hypothesis

The scanner preview pipeline likely works as follows:

```

1. Initialize scanner
   |
   v
2. Move film / detect strip
   |
   v
3. Allocate preview slots for maximum carrier capacity
   |
   v
4. Scan or read preview data into fixed frame slots
   |
   v
5. Enumerate all possible frame positions (40 max)
   |
   v
6. Validate frame contents
   |
   v
7. Remove empty frames from UI

```

The software likely does not know the number of frames beforehand.

Instead:

- The scanner exposes a fixed maximum frame map.
- The driver probes every slot.
- Frame validity is determined afterwards.

---

# Frame Addressing Hypothesis

The observed spacing:

```

256 blocks/frame

```

suggests a frame table:

```

slot 0:
LBA = BASE + 0*256

slot 1:
LBA = BASE + 1*256

slot 2:
LBA = BASE + 2*256

...

slot 39:
LBA = BASE + 39*256

```

This should be verified by:

- Capturing a full 40-frame preview.
- Comparing LBA values.
- Checking whether valid frames always align to 256-block boundaries.

---

# Next Steps

## 1. Add complete CDB payload dumping

For every command:

```

CDB
direction
transfer length
payload before/after transfer

```

Especially:

- `0xE0`
- `0xE1`
- opcode `0x24`

---

## 2. Decode Nikon vendor parameters

For each vendor register:

Example:

```

E0 00 40 xx xx xx xx xx xx xx

```

dump:

- raw bytes
- endian interpretation
- signed/unsigned values

Likely candidates:

- resolution
- exposure
- motor positions
- lamp power
- CCD timing

---

## 3. Investigate frame validity markers

The empty frames may contain metadata indicating:

- no film detected
- frame border detection failed
- exposure histogram invalid
- checksum

Compare:

- valid frame slot
- empty frame slot

---

## 4. Capture a full strip preview

A complete 40-frame capture should reveal:

- exact frame indexing
- whether enumeration is sequential
- whether empty slots are skipped by the scanner or software
- preview storage layout

---

# Current Confidence

High confidence:

- Preview uses fixed frame slots.
- Software probes all possible frame positions.
- Empty frames are discarded after enumeration.
- Frame slots are likely separated by fixed-size allocation.

Medium confidence:

- 256-block spacing corresponds to frame slots.
- The LBA region represents a preview cache.

Low confidence:

- Meaning of opcode `0x24`.
- Exact frame validity mechanism.
- Nikon vendor command payload structures.
```