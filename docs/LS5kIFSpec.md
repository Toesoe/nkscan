**SUPER COOLSCAN 5000 ED I/F PROTOCOL SPECIFICATIONS**

**1. FEATURES OF THE COMMUNICATION PROTOCOL IN THIS UNIT**

> The communication protocol of this unit conforms to the interface
> standard of USB 2.0.
>
> The specifications for each communication protocol are explained
> below.

**1-1. USB Protocol Specifications**

**1-1-1. Outline**

**1-1-1-1. Composition of the USB**

> The composition and the uses of the end points in the USB
> communication are defined as shown below.

<table>
<colgroup>
<col style="width: 21%" />
<col style="width: 20%" />
<col style="width: 26%" />
<col style="width: 31%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: center;">Name</td>
<td style="text-align: center;">Transfer type</td>
<td style="text-align: center;">Transfer direction</td>
<td style="text-align: center;">Uses</td>
</tr>
<tr>
<td style="text-align: center;">End point 0</td>
<td style="text-align: center;">Control IN/OUT</td>
<td style="text-align: center;"><p>Initiator -&gt; This unit/</p>
<p>This unit -&gt; Initiator</p></td>
<td style="text-align: center;">Transmission/reception of the standard
descriptor</td>
</tr>
<tr>
<td style="text-align: center;">End point 1</td>
<td style="text-align: center;">Bulk OUT</td>
<td style="text-align: center;">Initiator -&gt; This unit</td>
<td style="text-align: center;">Transmission of the data/ command</td>
</tr>
<tr>
<td style="text-align: center;">End point 2</td>
<td style="text-align: center;">Bulk IN</td>
<td style="text-align: center;">This unit -&gt; Initiator</td>
<td style="text-align: center;">Reception of the data/ command</td>
</tr>
<tr>
<td style="text-align: center;">End point 3</td>
<td style="text-align: center;">Interrupt IN</td>
<td style="text-align: center;">This unit -&gt; Initiator</td>
<td style="text-align: center;">Not used in this unit</td>
</tr>
</tbody>
</table>

**1-1-2. Communication phase specifications**

**1-1-2-1. Correspondence to the SCSI phase**

> In order to make the USB communication protocol have the phase
> conception for the SCSI, this unit manages the phase and the initiator
> checks the phase for operation.
>
> However, the standard device request processing by the control
> transfer should be performed according to the USB specifications, and
> the phase check is not used (the response is made with the response
> descriptor to the descriptor request).

1)  The communication is started by the command issuance of the
    initiator.

2)  This unit receives the command and then sets the next operation
    phase.

3)  When the initiator issues D0h (phase check code), this unit responds
    with the following phase codes.

> After the command is issued, the phase check must be executed without
> fail.
>
> It is supposed that the status phase is executed without fail after
> the phase processing (DATA IN, DATA OUT).

4)  The initiator performs the communication corresponding to the
    received phase code. (DATA IN, DATA OUT, STATUS)

Table 1-1-2-1 Phase codes

|  |  |  |
|:--:|:--:|:--:|
| Phase | Code | Status |
| No phase | 00h | Nothing is received (a command can be transmitted). |
| STATUS | 01h | Status IN phase |
| DATA OUT | 02h | Data OUT phase |
| DATA IN | 03h | Data IN phase |
| BUSY | 04h | A command is being executed (the processing that is being executed continues). |

**  
1-1-2-2. Data IN/OUT phase**

> The operation at the data IN/OUT is shown below.

1)  When the command is received from the initiator, the phase is set
    according to table 1-1-2-1.

2)  After receiving D0h (phase check code) from the initiator, the phase
    response is made.

3)  This unit transmits or receives the data.

4)  When this unit receives 06H (status reception code) indicating that
    the status reception is enabled from the initiator, the status
    transmission is started.

![](media/image1.wmf)

**1-1-2-3. Status IN phase**

> The operation in the status IN is shown below.

1)  When the command is received from the initiator, the phase is set
    according to table 1-1-2-1.

2)  After receiving D0h (phase check code) from the initiator, the phase
    response is made.

3)  When this unit receives 06H (status reception code) indicating that
    the status reception is enabled from the initiator, the status
    transmission is started.

4)  The status for the command that is received in 1) is transmitted.

![](media/image2.wmf)

**  
1-1-2-4. Abort processing of the operation activation command**

> The abort processing of the operation activation command is performed
> by issuing the ABORT command.
>
> An example of the abort processing for the scanning operation is shown
> below.

1)  When the SET WINDOW command is received, the phase is set.

2)  After D0h (phase check code) is received, the phase response is
    made.

3)  When this unit receives 06H (status reception code) indicating that
    the status reception is enabled from the initiator, the status
    transmission is started.

4)  After the data is received, the status for the SET WINDOW command is
    transmitted.

5)  When the SCAN command is received, the phase is set.

6)  After D0h (phase check code) is received, the phase response is
    made.

7)  When this unit receives 06H (status reception code) indicating that
    the status reception is enabled from the initiator, the status
    transmission is started.

8)  The status for the SCAN command is transmitted.

9)  When the first READ command is received, the phase is set.

10) After D0h (phase check code) is received, the phase response is
    made.

11) When this unit receives 06H (status reception code) indicating that
    the status reception is enabled from the initiator, the status
    transmission is started.

12) After the data is transmitted, the status for the READ command is
    transmitted.

13) When the second READ command is received, the phase is set.

14) After D0h (phase check code) is received, the phase response is
    made.

15) When this unit receives 06H (status reception code) indicating that
    the status reception is enabled from the initiator, the status
    transmission is started.

16) After the data is transmitted, the status for the READ command is
    transmitted.

17) When the ABORT command is received, the phase is set.

18) After D0h (phase check code) is received, the phase response is
    made.

19) When this unit receives 06H (status reception code) indicating that
    the status reception is enabled from the initiator, the status
    transmission is started.

20) The status for the ABORT command is transmitted.

21) The ABORT processing is performed.

**  
1-1-3. Commands of this unit**

> The commands that are executed by this unit are shown below.

Table 1-1-3-1 List of the commands of this unit

<table style="width:100%;">
<colgroup>
<col style="width: 42%" />
<col style="width: 16%" />
<col style="width: 12%" />
<col style="width: 27%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: center;">Command name</td>
<td style="text-align: center;">Operation code</td>
<td style="text-align: center;">Type</td>
<td style="text-align: center;">Phase transition</td>
</tr>
<tr>
<td style="text-align: center;"><blockquote>
<p>TEST UNIT READY</p>
</blockquote></td>
<td style="text-align: center;">00h</td>
<td style="text-align: center;">M</td>
<td style="text-align: center;"><blockquote>
<p>C - S</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;"><blockquote>
<p>INQUIRY</p>
</blockquote></td>
<td style="text-align: center;">12h</td>
<td style="text-align: center;">M</td>
<td style="text-align: center;"><blockquote>
<p>C - Din - S</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;"><blockquote>
<p>MODE SELECT (6)</p>
</blockquote></td>
<td style="text-align: center;">15h</td>
<td style="text-align: center;">O</td>
<td style="text-align: center;"><blockquote>
<p>C - Dout - S</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;"><blockquote>
<p>MODE SENSE (6)</p>
</blockquote></td>
<td style="text-align: center;">1Ah</td>
<td style="text-align: center;">O</td>
<td style="text-align: center;"><blockquote>
<p>C - Din - S</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;"><blockquote>
<p>SCAN</p>
</blockquote></td>
<td style="text-align: center;">1Bh</td>
<td style="text-align: center;">M</td>
<td style="text-align: center;"><blockquote>
<p>C - S</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;"><blockquote>
<p>RECEIVER DIAGNOSTIC RESULTS</p>
</blockquote></td>
<td style="text-align: center;">1Ch</td>
<td style="text-align: center;">M</td>
<td style="text-align: center;"><blockquote>
<p>C - S</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;"><blockquote>
<p>SEND DIAGNOSTIC</p>
</blockquote></td>
<td style="text-align: center;">1Dh</td>
<td style="text-align: center;">M</td>
<td style="text-align: center;"><blockquote>
<p>C - S</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;"><blockquote>
<p>SET WINDOW</p>
</blockquote></td>
<td style="text-align: center;">24h</td>
<td style="text-align: center;">M</td>
<td style="text-align: center;"><blockquote>
<p>C - Dout - S</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;"><blockquote>
<p>GET WINDOW</p>
</blockquote></td>
<td style="text-align: center;">25h</td>
<td style="text-align: center;">O</td>
<td style="text-align: center;"><blockquote>
<p>C - Din - S</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;"><blockquote>
<p>READ</p>
</blockquote></td>
<td style="text-align: center;">28h</td>
<td style="text-align: center;">M</td>
<td style="text-align: center;"><blockquote>
<p>C - Din - S</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;"><blockquote>
<p>SEND</p>
</blockquote></td>
<td style="text-align: center;">2Ah</td>
<td style="text-align: center;">O</td>
<td style="text-align: center;"><blockquote>
<p>C - Dout - S</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;"><blockquote>
<p>ABORT</p>
</blockquote></td>
<td style="text-align: center;">C0h</td>
<td style="text-align: center;">V</td>
<td style="text-align: center;"><blockquote>
<p>C - S</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;"><blockquote>
<p>EXECUTE</p>
</blockquote></td>
<td style="text-align: center;">C1h</td>
<td style="text-align: center;">V</td>
<td style="text-align: center;"><blockquote>
<p>C - S</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;"><blockquote>
<p>SET PARAMETER</p>
</blockquote></td>
<td style="text-align: center;">E0h</td>
<td style="text-align: center;">V</td>
<td style="text-align: center;"><blockquote>
<p>C - Dout - S</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;"><blockquote>
<p>GET PARAMETER</p>
</blockquote></td>
<td style="text-align: center;">E1h</td>
<td style="text-align: center;">V</td>
<td style="text-align: center;"><blockquote>
<p>C - Din - S</p>
</blockquote></td>
</tr>
</tbody>
</table>

Remarks

M : Mandatory command in the SCSI-2 standard

O : Option command in the SCSI-2 standard

V : Command that is originally specified in this unit

Explanation of the phase

C : Command phase

Din : DATA IN phase

Dout : DATA OUT phase

S : Status phase

Phase : 1-byte phase code is returned.

> Note) The presence of data phase described in the example is in the
> case that the transfer length contains non-zero value.

**1-1-4. Message code**

> The message phase is not supported in the USB.

**1-1-5. Status**

**1-1-5-1. Status supported in this unit**

> The status supported by this unit is shown below.
>
> Since the BUSY status and the RESERVATION CONFLICT status cannot exist
> in the USB, they are not supported.

Table 1-1-5-1 Status byte code in this unit

<table>
<colgroup>
<col style="width: 3%" />
<col style="width: 5%" />
<col style="width: 5%" />
<col style="width: 5%" />
<col style="width: 5%" />
<col style="width: 5%" />
<col style="width: 5%" />
<col style="width: 5%" />
<col style="width: 5%" />
<col style="width: 38%" />
<col style="width: 10%" />
<col style="width: 3%" />
</colgroup>
<tbody>
<tr>
<td colspan="9" style="text-align: center;">Bit of the status byte</td>
<td colspan="3" style="text-align: center;">Status</td>
</tr>
<tr>
<td colspan="2" style="text-align: center;">7</td>
<td style="text-align: center;">6</td>
<td style="text-align: center;">5</td>
<td style="text-align: center;">4</td>
<td style="text-align: center;">3</td>
<td style="text-align: center;">2</td>
<td style="text-align: center;">1</td>
<td style="text-align: center;">0</td>
<td style="text-align: center;"></td>
<td colspan="2" style="text-align: center;"></td>
</tr>
<tr>
<td colspan="2" style="text-align: center;">R</td>
<td style="text-align: center;">R</td>
<td style="text-align: center;">0</td>
<td style="text-align: center;">0</td>
<td style="text-align: center;">0</td>
<td style="text-align: center;">0</td>
<td style="text-align: center;">0</td>
<td style="text-align: center;">R</td>
<td style="text-align: center;"><blockquote>
<p>GOOD</p>
</blockquote></td>
<td colspan="2" style="text-align: center;">[00h]</td>
</tr>
<tr>
<td colspan="2" style="text-align: center;">R</td>
<td style="text-align: center;">R</td>
<td style="text-align: center;">0</td>
<td style="text-align: center;">0</td>
<td style="text-align: center;">0</td>
<td style="text-align: center;">0</td>
<td style="text-align: center;">1</td>
<td style="text-align: center;">R</td>
<td style="text-align: center;"><blockquote>
<p>CHECK CONDITION</p>
</blockquote></td>
<td colspan="2" style="text-align: center;">[02h]</td>
</tr>
<tr>
<td colspan="11" style="text-align: center;"><blockquote>
<p>Key: R - Reserved bit (set to 0)</p>
</blockquote></td>
<td style="text-align: center;"></td>
</tr>
</tbody>
</table>

**1-1-5-2. Format of the status**

> The status and the sense data are synthesized in the status phase and
> output. The format is shown below. The 8-byte status data is always
> transmitted.
>
> The status code in table 1-1-5-1 is set in byte 0. The sense data in
> table 4-1-1 is set for the sense key, ASC, ASCQ, and TSC in byte 1 to
> 4.

Table 1-1-5-2 Format of the status

<table style="width:73%;">
<colgroup>
<col style="width: 6%" />
<col style="width: 4%" />
<col style="width: 6%" />
<col style="width: 5%" />
<col style="width: 2%" />
<col style="width: 6%" />
<col style="width: 2%" />
<col style="width: 2%" />
<col style="width: 4%" />
<col style="width: 0%" />
<col style="width: 7%" />
<col style="width: 7%" />
<col style="width: 7%" />
<col style="width: 6%" />
<col style="width: 0%" />
</colgroup>
<tbody>
<tr>
<td colspan="2" style="text-align: right;"><p>Bit</p>
<p>Byte</p></td>
<td style="text-align: center;">7</td>
<td colspan="2" style="text-align: center;">6</td>
<td colspan="2" style="text-align: center;">5</td>
<td colspan="4" style="text-align: center;">4</td>
<td style="text-align: center;">3</td>
<td style="text-align: center;">2</td>
<td style="text-align: center;">1</td>
<td style="text-align: center;">0</td>
</tr>
<tr>
<td colspan="2" style="text-align: center;">0</td>
<td colspan="3" style="text-align: center;">[0]</td>
<td colspan="9" style="text-align: center;">Status</td>
<td style="text-align: center;">[0]</td>
</tr>
<tr>
<td colspan="2" style="text-align: center;">1</td>
<td colspan="7" style="text-align: center;">[0]</td>
<td colspan="6" style="text-align: center;">Sense key</td>
</tr>
<tr>
<td colspan="3" style="text-align: center;">2</td>
<td colspan="12" style="text-align: center;">ASC</td>
</tr>
<tr>
<td colspan="3" style="text-align: center;">3</td>
<td colspan="12" style="text-align: center;">ASCQ</td>
</tr>
<tr>
<td colspan="3" style="text-align: center;">4</td>
<td colspan="12" style="text-align: center;">TSC</td>
</tr>
<tr>
<td colspan="3" style="text-align: center;">5</td>
<td colspan="12" style="text-align: center;">Reserved [00h]</td>
</tr>
<tr>
<td colspan="3" style="text-align: center;">6</td>
<td colspan="12" style="text-align: center;">Reserved [00h]</td>
</tr>
<tr>
<td colspan="3" style="text-align: center;">7</td>
<td colspan="12" style="text-align: center;">Reserved [00h]</td>
</tr>
</tbody>
</table>

**  
1-1-6. USB-specific additional specifications**

**1-1-6-1. Standard device requests**

> The standard device requests are shown below.

Table 1-1-6-1-1 Standard device request

|  |  |  |  |
|:--:|:--:|:--:|:--:|
| ｂRequest | Value | Meaning | Support of this unit |
| GET_STATUS | 0 | Status acquisition | Yes |
| CLEAR_FEATURE | 1 | Function clearance | Yes |
| Reserved for future use | 2 | Reserved | Stall |
| SET_FEATURE | 3 | Function setting | Yes |
| Reserved for future use | 4 | Reserved | Stall |
| SET_ADDRESS | 5 | Address setting | Yes |
| GET_DESCRIPTOR | 6 | Descriptor acquisition | Yes |
| SET_DESCRIPTOR | 7 | Descriptor setting | Stall |
| GET_CONFIGURATION | 8 | Configuration acquisition | Yes |
| SET_CONFIGURATION | 9 | Configuration setting | Yes |
| GET_INTERFACE | 10 | Interface acquisition | Yes |
| SET_INTERFACE | 11 | Interface setting | Yes |
| SYNCH_FRAME | 12 | Synchronization frame | Stall |

Table 1-1-6-1-2 Descriptor type

|      |                           |
|:----:|:-------------------------:|
| Type |           Value           |
|  1   |          DEVICE           |
|  2   |       CONFIGURATION       |
|  3   |          STRING           |
|  4   |         INTERFACE         |
|  5   |         ENDPOINT          |
|  6   |     DEVICE_QUALIFIER      |
|  7   | OTHER_SPEED_CONFIGURATION |
|  8   |      INTERFACE_POWER      |

> Remarks: The upper byte of the value indicates the descriptor type and
> the lower byte indicates the string descriptor index.

**  
1-1-6-2. Device descriptors in this unit**

> The lists of the descriptors for GET_DESCRIPTOR in this unit are shown
> below.

Table 1-1-6-2-1 DEVICE descriptor

|  |  |  |  |
|:--:|:--:|:--:|:--:|
| Byte | Size | Item | Set value |
| 0 | 1 | Size of this descriptor | 12h (fixed) |
| 1 | 1 | Type of DEVICE descriptor | 01h (fixed) |
| 2 | 2 | Release number of the USB specifications (2.00) | 0200h |
| 4 | 1 | Class code | FFh (vendor) |
| 5 | 1 | Sub-class code | FFh |
| 6 | 1 | Protocol code | FFh (vendor-specific) |
| 7 | 1 | Maximum buffer size of end point 0 | 40h (64 bytes) |
| 8 | 2 | Vendor ID | 04B0h |
| 10 | 2 | Product ID | 4002h |
| 12 | 2 | Device release number | xxxxh |
| 14 | 1 | Index to the string descriptor of the manufacturer | 01h |
| 15 | 1 | Index to the string descriptor representing the product | 02h |
| 16 | 1 | Index to the string descriptor representing the product number of the device | 00h |
| 17 | 1 | The number that can be configured | 01h |

Table 1-1-6-2-2 CONFIGURATION descriptor

|  |  |  |  |
|:--:|:--:|:--:|:--:|
| Byte | Size | Item | Set value |
| 0 | 1 | Size of this descriptor | 09h (fixed) |
| 1 | 1 | Descriptor type | 02h (fixed) |
| 2 | 2 | Length of the entire configuration | 0020h |
| 4 | 1 | The number of interfaces of the configuration | 01h |
| 5 | 1 | Configuration selection argument in SetConfig | 01h |
| 6 | 1 | Configuration string descriptor index | 00h |
| 7 | 1 | Configuration characteristics | C0h (self power supply only) |
| 8 | 1 | Maximum bus power consumption (in units of 2 mA) | 01h (2 mA) |

Table 1-1-6-2-3 INTERFACE descriptor

<table>
<colgroup>
<col style="width: 11%" />
<col style="width: 9%" />
<col style="width: 54%" />
<col style="width: 24%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: center;">Byte</td>
<td style="text-align: center;">Size</td>
<td style="text-align: center;">Item</td>
<td style="text-align: center;">Set value</td>
</tr>
<tr>
<td style="text-align: center;">0</td>
<td style="text-align: center;">1</td>
<td style="text-align: center;">Size of this descriptor</td>
<td style="text-align: center;">09h (fixed)</td>
</tr>
<tr>
<td style="text-align: center;">1</td>
<td style="text-align: center;">1</td>
<td style="text-align: center;">Descriptor type</td>
<td style="text-align: center;">04h (fixed)</td>
</tr>
<tr>
<td style="text-align: center;">2</td>
<td style="text-align: center;">1</td>
<td style="text-align: center;">Number of this interface in the
configuration</td>
<td style="text-align: center;">00h</td>
</tr>
<tr>
<td style="text-align: center;">3</td>
<td style="text-align: center;">1</td>
<td style="text-align: center;">Substitute selection argument for
SetInterface</td>
<td style="text-align: center;">00h</td>
</tr>
<tr>
<td style="text-align: center;">4</td>
<td style="text-align: center;">1</td>
<td style="text-align: center;"><p>The number of end points of the
interface</p>
<p>(End point 0 is not included.)</p></td>
<td style="text-align: center;">02h</td>
</tr>
<tr>
<td style="text-align: center;">5</td>
<td style="text-align: center;">1</td>
<td style="text-align: center;">Class code</td>
<td style="text-align: center;">FFh (vendor)</td>
</tr>
<tr>
<td style="text-align: center;">6</td>
<td style="text-align: center;">1</td>
<td style="text-align: center;">Sub-class code</td>
<td style="text-align: center;">FFh</td>
</tr>
<tr>
<td style="text-align: center;">7</td>
<td style="text-align: center;">1</td>
<td style="text-align: center;">Protocol code</td>
<td style="text-align: center;">FFh (vendor-specific)</td>
</tr>
<tr>
<td style="text-align: center;">8</td>
<td style="text-align: center;">1</td>
<td style="text-align: center;">Index to the string descriptor of this
interface</td>
<td style="text-align: center;">00h</td>
</tr>
</tbody>
</table>

Table 1-1-6-2-4 ENDPOINT descriptor

<table>
<colgroup>
<col style="width: 9%" />
<col style="width: 8%" />
<col style="width: 8%" />
<col style="width: 36%" />
<col style="width: 18%" />
<col style="width: 18%" />
</colgroup>
<tbody>
<tr>
<td rowspan="2" style="text-align: center;"><p>End</p>
<p>point</p></td>
<td rowspan="2" style="text-align: center;">Byte</td>
<td rowspan="2" style="text-align: center;">Size</td>
<td rowspan="2" style="text-align: center;">Item</td>
<td colspan="2" style="text-align: center;">Set value</td>
</tr>
<tr>
<td style="text-align: center;">2.0</td>
<td style="text-align: center;">1.1</td>
</tr>
<tr>
<td style="text-align: center;"></td>
<td style="text-align: center;">0</td>
<td style="text-align: center;">1</td>
<td style="text-align: center;">Size of this descriptor</td>
<td style="text-align: center;">07h (fixed)</td>
<td style="text-align: center;">07h (fixed)</td>
</tr>
<tr>
<td style="text-align: center;">1</td>
<td style="text-align: center;">1</td>
<td style="text-align: center;">1</td>
<td style="text-align: center;">Descriptor type</td>
<td style="text-align: center;">05h (fixed)</td>
<td style="text-align: center;">05h (fixed)</td>
</tr>
<tr>
<td style="text-align: center;"></td>
<td style="text-align: center;">2</td>
<td style="text-align: center;">1</td>
<td style="text-align: center;">End point address/direction</td>
<td style="text-align: center;">01h (OUT)</td>
<td style="text-align: center;">01h (OUT)</td>
</tr>
<tr>
<td style="text-align: center;"></td>
<td style="text-align: center;">3</td>
<td style="text-align: center;">1</td>
<td style="text-align: center;">Attribute (transfer type)</td>
<td style="text-align: center;">02h (bulk)</td>
<td style="text-align: center;">02h (bulk)</td>
</tr>
<tr>
<td style="text-align: center;"></td>
<td style="text-align: center;">4</td>
<td style="text-align: center;">2</td>
<td style="text-align: center;">Maximum packet size</td>
<td style="text-align: center;"><p>0200h</p>
<p>(512 bytes)</p></td>
<td style="text-align: center;">0040h (64 bytes)</td>
</tr>
<tr>
<td style="text-align: center;"></td>
<td style="text-align: center;">6</td>
<td style="text-align: center;">1</td>
<td style="text-align: center;">Polling interval (in units of ms)</td>
<td style="text-align: center;">00h</td>
<td style="text-align: center;">00h</td>
</tr>
<tr>
<td style="text-align: center;"></td>
<td style="text-align: center;">0</td>
<td style="text-align: center;">1</td>
<td style="text-align: center;">Size of this descriptor</td>
<td style="text-align: center;">07h (fixed)</td>
<td style="text-align: center;">07h (fixed)</td>
</tr>
<tr>
<td style="text-align: center;">2</td>
<td style="text-align: center;">1</td>
<td style="text-align: center;">1</td>
<td style="text-align: center;">Descriptor type</td>
<td style="text-align: center;">05h (fixed)</td>
<td style="text-align: center;">05h (fixed)</td>
</tr>
<tr>
<td style="text-align: center;"></td>
<td style="text-align: center;">2</td>
<td style="text-align: center;">1</td>
<td style="text-align: center;">End point address/direction</td>
<td style="text-align: center;">82h (IN)</td>
<td style="text-align: center;">82h (IN)</td>
</tr>
<tr>
<td style="text-align: center;"></td>
<td style="text-align: center;">3</td>
<td style="text-align: center;">1</td>
<td style="text-align: center;">Attribute (transfer type)</td>
<td style="text-align: center;">02h (bulk)</td>
<td style="text-align: center;">02h (bulk)</td>
</tr>
<tr>
<td style="text-align: center;"></td>
<td style="text-align: center;">4</td>
<td style="text-align: center;">2</td>
<td style="text-align: center;">Maximum packet size</td>
<td style="text-align: center;"><p>0200h</p>
<p>(512 bytes)</p></td>
<td style="text-align: center;">0040h (64 bytes)</td>
</tr>
<tr>
<td style="text-align: center;"></td>
<td style="text-align: center;">6</td>
<td style="text-align: center;">1</td>
<td style="text-align: center;">Polling interval (in units of ms)</td>
<td style="text-align: center;">00h</td>
<td style="text-align: center;">00h</td>
</tr>
</tbody>
</table>

Table 1-1-6-2-5 Example of the STRING descriptor

<table style="width:91%;">
<colgroup>
<col style="width: 8%" />
<col style="width: 0%" />
<col style="width: 10%" />
<col style="width: 0%" />
<col style="width: 11%" />
<col style="width: 0%" />
<col style="width: 2%" />
<col style="width: 8%" />
<col style="width: 0%" />
<col style="width: 36%" />
<col style="width: 0%" />
<col style="width: 12%" />
</colgroup>
<tbody>
<tr>
<td colspan="7" style="text-align: center;">Request command from the
host</td>
<td colspan="5" style="text-align: center;">Response from the
device</td>
</tr>
<tr>
<td colspan="3" style="text-align: center;">Value</td>
<td colspan="2" style="text-align: center;"><p>Index/</p>
<p>LANGID</p></td>
<td colspan="3" style="text-align: center;"><p>Requested size</p>
<p>(N+2)</p></td>
<td colspan="2" style="text-align: center;"><p>Contents</p>
<p>(hex)</p></td>
<td colspan="2" style="text-align: center;">Remarks</td>
</tr>
<tr>
<td colspan="3" style="text-align: center;">0300h</td>
<td colspan="2" style="text-align: center;">0000h</td>
<td colspan="3" style="text-align: center;">04h</td>
<td colspan="2" style="text-align: center;">[04 03 09 04]</td>
<td colspan="2" style="text-align: center;">LANGID=0409h</td>
</tr>
<tr>
<td colspan="3" style="text-align: center;">0301h</td>
<td colspan="2" style="text-align: center;">0409h</td>
<td colspan="3" style="text-align: center;">04h</td>
<td colspan="2" style="text-align: center;"><blockquote>
<p>“N”</p>
</blockquote>
<p>[0C 03 4E 00]</p></td>
<td colspan="2" style="text-align: center;">First character of
Nikon</td>
</tr>
<tr>
<td colspan="3" style="text-align: center;">0301h</td>
<td colspan="2" style="text-align: center;">0409h</td>
<td colspan="3" style="text-align: center;">0Ch</td>
<td colspan="2" style="text-align: center;"><blockquote>
<p>“Nikon”</p>
</blockquote>
<p>[0C 03 4E 00 69 00 6B 00 6F 00 6E 00]</p></td>
<td colspan="2" style="text-align: center;">Manufacturer</td>
</tr>
<tr>
<td colspan="3" style="text-align: center;">0302h</td>
<td colspan="2" style="text-align: center;">0409h</td>
<td colspan="3" style="text-align: center;">04h</td>
<td colspan="2" style="text-align: center;"><blockquote>
<p>“L”</p>
</blockquote>
<p>[16 03 4C 00]</p></td>
<td colspan="2" style="text-align: center;"><p>Product name</p>
<p>First character of the model name</p></td>
</tr>
<tr>
<td colspan="3" style="text-align: center;">0302h</td>
<td colspan="2" style="text-align: center;">0409h</td>
<td colspan="3" style="text-align: center;">10h</td>
<td colspan="2" style="text-align: center;"><blockquote>
<p>“LS-5000 ED”</p>
</blockquote>
<p>[16 03 4C 00 53 00 2D 00 35 00 30 00</p>
<p>30 00 30 00 20 00 45 00 44 00]</p></td>
<td colspan="2" style="text-align: center;">Model name</td>
</tr>
<tr>
<td colspan="3" style="text-align: center;">0303h</td>
<td colspan="2" style="text-align: center;">0409h</td>
<td colspan="3" style="text-align: center;">04h</td>
<td colspan="2" style="text-align: center;"><blockquote>
<p>“x”</p>
</blockquote></td>
<td colspan="2" style="text-align: center;"><p>First character of the
version</p>
<p>(Only when the serial No. is written)</p></td>
</tr>
<tr>
<td colspan="3" style="text-align: center;">0303h</td>
<td colspan="2" style="text-align: center;">0409h</td>
<td colspan="3" style="text-align: center;">12h</td>
<td colspan="2" style="text-align: center;"><blockquote>
<p>“xxxxxxxx”</p>
</blockquote></td>
<td colspan="2" style="text-align: center;"><p>Version of the model</p>
<p>Product number of the device</p>
<p>(Only when the serial No. is written)</p></td>
</tr>
</tbody>
</table>

Table 1-1-6-2-6 DEVICE_QUALIFIER descriptor

|      |      |                                                |           |
|:----:|:----:|:----------------------------------------------:|:---------:|
| Byte | Size |                      Item                      | Set value |
|  0   |  1   |            Size of this descriptor             |    0Ah    |
|  1   |  1   |                Descriptor type                 |    06h    |
|  2   |  2   | Release number of the USB specifications (2.0) |   0200h   |
|  4   |  1   |                   Class code                   |    FFh    |
|  5   |  1   |                 Sub-class code                 |    FFh    |
|  6   |  1   |                 Protocol code                  |    FFh    |
|  7   |  1   |           Maximum packet size of EP0           |    40h    |
|  8   |  1   |       The number that can be configured        |    01h    |
|  9   |  1   |                    Reserved                    |    00h    |

Table 1-1-6-2-7 OTHER_SPEED_CONFIGURATION descriptor

|  |  |  |  |
|:--:|:--:|:--:|:--:|
| Byte | Size | Item | Set value |
| 0 | 1 | Size of this descriptor | 09h |
| 1 | 1 | Descriptor type | 02h |
| 2 | 2 | Length of the entire configuration | 0020h |
| 4 | 1 | The number of interfaces of the configuration | 01h |
| 5 | 1 | Argument for selecting this configuration | 01h |
| 6 | 1 | Index to the string descriptor | 00h |
| 7 | 1 | Specification of each characteristic (self power supply/remote wake-up) | C0h |
| 8 | 1 | Maximum bus power consumption | 01h |

**  
2. COMMAND EXPLANATIONS**

Each command is explained below.

In the explanations, the common error responses are as shown in the
table below.

<table>
<colgroup>
<col style="width: 21%" />
<col style="width: 44%" />
<col style="width: 34%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: center;">Common error</td>
<td style="text-align: center;">Sense data</td>
<td style="text-align: center;">Remarks</td>
</tr>
<tr>
<td style="text-align: center;">1</td>
<td style="text-align: center;"><p>INVALID FIELD IN CDB</p>
<p>(Some illegal data exists in the CDB.)</p>
<p>05h-24h-00h-00h</p></td>
<td style="text-align: center;">Terminates with CHECK CONDITION
status.</td>
</tr>
<tr>
<td style="text-align: center;">2</td>
<td style="text-align: center;"><p>INVALID FIELD IN PARAMETER LIST</p>
<p>(Some illegal data exists in the parameter.)</p>
<p>05h-26h-00h-00h</p></td>
<td style="text-align: center;">Terminates with CHECK CONDITION
status.</td>
</tr>
</tbody>
</table>

Other error responses are explained individually in the explanations
below.

The values in \[ \] in the table show the permissible values or
recommended values of this unit in the command description block and in
the parameter, or the values that are returned by this unit in the
response data.

**2-1. TEST UNIT READY Command**

Table 2-1-1 TEST UNIT READY command

<table>
<colgroup>
<col style="width: 13%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: right;"><p>Bit</p>
<p>Byte</p></td>
<td style="text-align: center;">7</td>
<td style="text-align: center;">6</td>
<td style="text-align: center;">5</td>
<td style="text-align: center;">4</td>
<td style="text-align: center;">3</td>
<td style="text-align: center;">2</td>
<td style="text-align: center;">1</td>
<td style="text-align: center;">0</td>
</tr>
<tr>
<td style="text-align: center;">0</td>
<td colspan="8" style="text-align: center;">Operation code [00h]</td>
</tr>
<tr>
<td style="text-align: center;">1</td>
<td colspan="3" style="text-align: center;"><p>Logical unit number</p>
<p>[0]</p></td>
<td colspan="5" style="text-align: center;"><p>Reserved</p>
<p>[0]</p></td>
</tr>
<tr>
<td style="text-align: center;">2 to 4</td>
<td colspan="8" style="text-align: center;">Reserved [0]</td>
</tr>
<tr>
<td style="text-align: center;">5</td>
<td colspan="8" style="text-align: center;">Control byte [0]</td>
</tr>
</tbody>
</table>

The TEST UNIT READY command provides a means to check if the logical
unit is ready.

Table 2-1-2 shows the responses corresponding to the TEST UNIT READY
command. A response that has higher priority (RESERVATION CONFLICT, for
example) may be made.

Table 2-1-2 Preferred Test Unit Ready Responses

<table>
<colgroup>
<col style="width: 18%" />
<col style="width: 81%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: center;">Status</td>
<td style="text-align: center;">Sense code</td>
</tr>
<tr>
<td style="text-align: center;">GOOD</td>
<td style="text-align: center;"><p>No Additional Sense Information</p>
<p>00h-00h-00h-00h (Common: No error)</p></td>
</tr>
<tr>
<td style="text-align: center;">Check Condition</td>
<td style="text-align: center;"><p>Logical Unit Not Supported</p>
<blockquote>
<p>05h-25h-00h-00h (Common: An LUN other than 0 was specified.)</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">Check Condition</td>
<td style="text-align: left;"><p>Logical Unit Is In Process Of Becoming
Ready</p>
<blockquote>
<p>02h-04h-01h-00h (Common: During the execution of the operation
activation command)</p>
<p>02h-04h-01h-01h (MA-21: During the adapter initialization
operation)</p>
<p>(Other than MA-21: During the adapter initialization operation or
during loading/ejection of the object to be scanned)</p>
<p>02h-04h-01h-02h (Common: During the measurement of the correction
data)</p>
<p>02h-04h-01h-03h (MA-21: During the execution of operation for loading
the object to be scanned)</p>
<p>02h-04h-01h-04h (Common: During the execution of automatic shading or
white balance measurement)</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">Check Condition</td>
<td style="text-align: left;"><p>Logical Unit Not Ready, Cause Not
Reportable</p>
<blockquote>
<p>02h-04h-02h-00h (Common: The internal mechanical error occurred.)</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">Check Condition</td>
<td style="text-align: left;"><p>Logical Unit Not Ready, Initializing
Command Required</p>
<blockquote>
<p>02h-04h-00h-00h (The initialization is not complete because an object
is inserted at the time of power ON.)</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">Check Condition</td>
<td style="text-align: left;"><p>Logical Unit Not Ready, Manual
Intervention Required</p>
<p>02h-04h-03h-00h (Common: The adapter is ejected.)</p>
<blockquote>
<p>02h-04h-03h-01h (IA-20: The LL door is not completely opened when the
240 adapter is attached.)</p>
<p>02h-04h-03h-02h (Common: Undefined adapter)</p>
<p>02h-04h-03h-03h (SA-30: The film of 6 frames or more is loaded with
the film gate closed)</p>
<p>02h-04h-03h-04h (SA-21/SA-30: The adapter is pulled out a little in
the locked status.)</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">Check Condition</td>
<td style="text-align: left;"><p>Logical Unit Does Not Respond To
Selection</p>
<blockquote>
<p>02h-05h-00h-00h (Common: The operation is possible, but the
initialization operation in the unit is not completed because the power
is just turned ON.)</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">Check Condition</td>
<td style="text-align: left;"><p>Medium Not Present</p>
<blockquote>
<p>02h-3Ah-00h-00h (SF-210: The loading command is sent without an
object to be scanned.)</p>
<p>02h-3Ah-00h-01h (MA-21: (a) only) (IA-20: (a), (b), (c), or (d))</p>
<p>(Other: (a), (b), or (c))</p>
</blockquote>
<ol type="a">
<li><p>A medium is not supplied to the adapter.</p></li>
<li><p>The film is ejected when the power supply is turned ON or the
adapter is exchanged.</p></li>
<li><p>The medium is ejected by the eject command.</p></li>
<li><p>The LL door is opened, but the loading switch is not ON.</p></li>
</ol>
<blockquote>
<p>02h-3Ah-00h-03h (SA-21/SA-30: Reading cannot be performed because a
film that is out of standard is inserted.)</p>
<p>02h-3Ah-00h-04h (The frame position of a larger number than the
number of frames in the inserted film is specified.)</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">Check Condition</td>
<td style="text-align: left;"><p>06h-xxh-xxh-xxh</p>
<blockquote>
<p>Unit Attention</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">Check Condition</td>
<td style="text-align: left;"><p>Data Phase Error</p>
<blockquote>
<p>0Bh-4Bh-00h-00h (Common: Unexpected error during Data Phase)</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">Check Condition</td>
<td style="text-align: left;"><p>Overlapped Commands Attempted</p>
<blockquote>
<p>0Bh-4Eh-00h-00h (Common: The unit is selected by the same initiator
while disconnecting.)</p>
</blockquote></td>
</tr>
</tbody>
</table>

**2-2. INQUIRY Command**

Table 2-2-1-1 INQUIRY command

<table>
<colgroup>
<col style="width: 13%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 0%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 11%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: right;"><p>Bit</p>
<p>Byte</p></td>
<td style="text-align: center;">7</td>
<td style="text-align: center;">6</td>
<td colspan="2" style="text-align: center;">5</td>
<td style="text-align: center;">4</td>
<td style="text-align: center;">3</td>
<td style="text-align: center;">2</td>
<td style="text-align: center;">1</td>
<td style="text-align: center;">0</td>
</tr>
<tr>
<td style="text-align: center;">0</td>
<td colspan="9" style="text-align: center;">Operation code [12h]</td>
</tr>
<tr>
<td style="text-align: center;">1</td>
<td colspan="4" style="text-align: center;"><p>Logical unit number</p>
<p>[0]</p></td>
<td colspan="4" style="text-align: center;"><p>Reserved</p>
<p>[0]</p></td>
<td style="text-align: center;"><p>EVPD</p>
<p>[0, 1]</p></td>
</tr>
<tr>
<td style="text-align: center;">2</td>
<td colspan="9" style="text-align: center;">Page code [0]</td>
</tr>
<tr>
<td style="text-align: center;">3</td>
<td colspan="9" style="text-align: center;">Reserved [0]</td>
</tr>
<tr>
<td style="text-align: center;">4</td>
<td colspan="9" style="text-align: center;">Allocation length
[Recommended value 36d]</td>
</tr>
<tr>
<td style="text-align: center;">5</td>
<td colspan="3" style="text-align: center;">Reserved [0]</td>
<td colspan="6" style="text-align: center;">Control byte [0]</td>
</tr>
</tbody>
</table>

1)  The INQUIRY command sends information regarding parameters of this
    unit and its attached logical units to the initiator.

2)  An EVPD (Enable Vital Product Data) bit of zero specifies that this
    unit shall send the standard INQUIRY data to the initiator. An EVPD
    bit of one indicates that this unit shall send the VPD (Vital
    Product Data) specified by the page code field to the initiator.

3)  The page code field specifies the kind of VPD information when EVPD
    bit has a value of one. An EVPD bit of zero specifies that the page
    code is invalid. Table 2-2-1-2 shows the list of page codes
    supported in this unit.

4)  The INQUIRY command shall return with CHECK CONDITION status only
    when this unit cannot return the requested INQUIRY data.

5)  If an INQUIRY command is received from an initiator with a pending
    unit attention condition (i.e., before this unit reports CHECK
    CONDITION status), this unit shall perform the INQUIRY command and
    shall not clear the unit attention condition (refer to standard
    6.9).

> Table 2-2-1-2 Page code field list

<table>
<colgroup>
<col style="width: 7%" />
<col style="width: 10%" />
<col style="width: 28%" />
<col style="width: 13%" />
<col style="width: 8%" />
<col style="width: 31%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: center;">VPD</td>
<td colspan="3" style="text-align: center;">Page code</td>
<td style="text-align: center;">Sub-section</td>
<td style="text-align: center;">Attached adapter (*1)</td>
</tr>
<tr>
<td style="text-align: center;">0</td>
<td colspan="2" style="text-align: center;">Standard INQUIRY data</td>
<td style="text-align: center;">00h (*2)</td>
<td style="text-align: center;">2-2-1</td>
<td style="text-align: center;">MA-21, SA-21, SA-30, IA-20, SF-210,
Non</td>
</tr>
<tr>
<td rowspan="26" style="text-align: center;">1</td>
<td rowspan="26" style="text-align: center;">VPD informa-tion</td>
<td style="text-align: center;">Page code list</td>
<td style="text-align: center;">00h</td>
<td style="text-align: center;">2-2-2-1</td>
<td style="text-align: center;">MA-21, SA-21, SA-30, IA-20, SF-210,
Non</td>
</tr>
<tr>
<td rowspan="14" style="text-align: center;">FRU ASCII information</td>
<td style="text-align: center;">01h</td>
<td rowspan="14" style="text-align: center;">2-2-2-2</td>
<td style="text-align: center;">MA-21, SA-21, SA-30, IA-20, SF-210</td>
</tr>
<tr>
<td style="text-align: center;">10h</td>
<td style="text-align: center;">MA-21</td>
</tr>
<tr>
<td style="text-align: center;">40h (unused)</td>
<td style="text-align: center;">-</td>
</tr>
<tr>
<td style="text-align: center;">41h (unused)</td>
<td style="text-align: center;">-</td>
</tr>
<tr>
<td style="text-align: center;">43h (unused)</td>
<td style="text-align: center;">-</td>
</tr>
<tr>
<td style="text-align: center;">44h (unused)</td>
<td style="text-align: center;">-</td>
</tr>
<tr>
<td style="text-align: center;">45h (unused)</td>
<td style="text-align: center;">-</td>
</tr>
<tr>
<td style="text-align: center;">46h (unused)</td>
<td style="text-align: center;">-</td>
</tr>
<tr>
<td style="text-align: center;">47h (unused)</td>
<td style="text-align: center;">-</td>
</tr>
<tr>
<td style="text-align: center;">50h (unused)</td>
<td style="text-align: center;">-</td>
</tr>
<tr>
<td style="text-align: center;">51h (unused)</td>
<td style="text-align: center;">-</td>
</tr>
<tr>
<td style="text-align: center;">60h (unused)</td>
<td style="text-align: center;">-</td>
</tr>
<tr>
<td style="text-align: center;">61h (unused)</td>
<td style="text-align: center;">-</td>
</tr>
<tr>
<td style="text-align: center;">62h (unused)</td>
<td style="text-align: center;">-</td>
</tr>
<tr>
<td style="text-align: center;">Address information</td>
<td style="text-align: center;">C1h</td>
<td style="text-align: center;">2-2-2-3</td>
<td style="text-align: center;">MA-21, SA-21, SA-30, IA-20, SF-210,
Non</td>
</tr>
<tr>
<td style="text-align: center;">SET WINDOW function</td>
<td style="text-align: center;">D1h</td>
<td style="text-align: center;">2-2-2-4</td>
<td style="text-align: center;">MA-21, SA-21, SA-30, IA-20, SF-210</td>
</tr>
<tr>
<td style="text-align: center;">Other information</td>
<td style="text-align: center;">E1h</td>
<td style="text-align: center;">2-2-2-5</td>
<td style="text-align: center;">MA-21, SA-21, SA-30, IA-20, SF-210</td>
</tr>
<tr>
<td style="text-align: center;">Operation code setting page</td>
<td style="text-align: center;">E2h</td>
<td style="text-align: center;">2-2-2-6</td>
<td style="text-align: center;">SA-21, SA-30, IA-20</td>
</tr>
<tr>
<td style="text-align: center;">CCD measurement setting page</td>
<td style="text-align: center;">E3h</td>
<td style="text-align: center;">2-2-2-7</td>
<td style="text-align: center;">MA-21, SA-21, SA-30, IA-20, SF-210</td>
</tr>
<tr>
<td rowspan="6" style="text-align: center;">Unused page</td>
<td style="text-align: center;">F0h</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
</tr>
<tr>
<td style="text-align: center;">F1h</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
</tr>
<tr>
<td style="text-align: center;">F8h</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
</tr>
<tr>
<td style="text-align: center;">FAh</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
</tr>
<tr>
<td style="text-align: center;">FBh</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
</tr>
<tr>
<td style="text-align: center;">FCh</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
</tr>
</tbody>
</table>

\*1 The correspondence between the model names in the above table and
the adapter names is shown below.

> MA-21: Mount adapter, SA-21: 6-frame strip adapter, SA-30: 36-frame
> strip adapter,
>
> IA-20: 240 adapter, SF-210: Slide feeder, Non: None/Undefined

\*2 Page code field value of the INQUIRY command that is transferred
from the initiator

**2-2-1. Standard INQUIRY data of this unit**

The standard INQUIRY data of this unit is the mandatory data of 36-byte
length only.

Table 2-2-2 Standard INQUIRY data format of this unit

<table>
<colgroup>
<col style="width: 13%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 11%" />
<col style="width: 10%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: right;"><p>Bit</p>
<p>Byte</p></td>
<td style="text-align: center;">7</td>
<td style="text-align: center;">6</td>
<td style="text-align: center;">5</td>
<td style="text-align: center;">4</td>
<td style="text-align: center;">3</td>
<td style="text-align: center;">2</td>
<td style="text-align: center;">1</td>
<td style="text-align: center;">0</td>
</tr>
<tr>
<td style="text-align: center;">0</td>
<td colspan="3" style="text-align: center;"><p>Peripheral Qualifier</p>
<p>[0]</p>
<p>[011b](*1)</p></td>
<td colspan="5" style="text-align: center;"><p>Peripheral Device
Type</p>
<p>[6=00110b]</p>
<p>[1Fh=11111b](*1)</p></td>
</tr>
<tr>
<td style="text-align: center;">1</td>
<td style="text-align: center;"><p>RMB</p>
<p>[1]</p></td>
<td colspan="7" style="text-align: center;"><p>Device-Type Modifier</p>
<p>[0]</p></td>
</tr>
<tr>
<td style="text-align: center;">2</td>
<td colspan="2" style="text-align: center;"><p>ISO Version</p>
<p>[0]</p></td>
<td colspan="3" style="text-align: center;"><p>ECMA Version</p>
<p>[0]</p></td>
<td colspan="3" style="text-align: center;"><p>ANSI-Approved Version</p>
<p>[2=010b]</p></td>
</tr>
<tr>
<td style="text-align: center;">3</td>
<td style="text-align: center;"><p>AENC</p>
<p>[0]</p></td>
<td style="text-align: center;"><p>TrmIOP</p>
<p>[0]</p></td>
<td colspan="2" style="text-align: center;"><p>Reserved</p>
<p>[0]</p></td>
<td colspan="4" style="text-align: center;"><p>Response Data Format</p>
<p>[2=0010b]</p></td>
</tr>
<tr>
<td style="text-align: center;">4</td>
<td colspan="8" style="text-align: center;"><p>Additional Length
(n-4)</p>
<p>[1Fh=31d]</p></td>
</tr>
<tr>
<td style="text-align: center;">5, 6</td>
<td colspan="8" style="text-align: center;">Reserved [0]</td>
</tr>
<tr>
<td style="text-align: center;">7</td>
<td style="text-align: center;"><p>RelAdr</p>
<p>[0]</p></td>
<td style="text-align: center;"><p>WBus32</p>
<p>[0]</p></td>
<td style="text-align: center;"><p>WBus16</p>
<p>[0]</p></td>
<td style="text-align: center;"><p>Sync</p>
<p>[0]</p></td>
<td style="text-align: center;"><p>Linked</p>
<p>[0]</p></td>
<td style="text-align: center;"><p>Reserved</p>
<p>[0]</p></td>
<td style="text-align: center;"><p>CmdQue</p>
<p>[0]</p></td>
<td style="text-align: center;"><p>SftRe</p>
<p>[0]</p></td>
</tr>
<tr>
<td style="text-align: center;">8 to 15</td>
<td colspan="8" style="text-align: center;"><p>Vendor Identification</p>
<p>[Nikon]</p></td>
</tr>
<tr>
<td style="text-align: center;">16 to 31</td>
<td colspan="8" style="text-align: center;"><p>Product
Identification</p>
<p>[ ]</p></td>
</tr>
<tr>
<td style="text-align: center;">32 to 35</td>
<td colspan="8" style="text-align: center;"><p>Product Revision
Level</p>
<p>Example: [0.01]</p></td>
</tr>
</tbody>
</table>

> \*1 When an invalid logical unit selection is performed

The RMB (Removable medium) bit is set to one.

The Vendor Identification field contains eight bytes of ASCII data
identifying the vendor of the product. In this field, the data shall be
left aligned and unused bytes shall be filled with space characters
(20h).

The Product Identification field contains sixteen bytes of ASCII data
defined by the vendor. In this field, the data shall be left aligned and
unused bytes shall be filled with space characters (20h).

The Product Revision Level field contains four bytes of ASCII data
defined by the vendor. In this field, the data shall be left aligned and
unused bytes shall be filled with space characters (20h).

**  
2-2-2. VPD information**

> When the EVPD (Enable Vital Product Data) bit is set to one, this unit
> sends VPD (Vital Product Data) specified in the page code field to the
> initiator.

Byte 0

> This byte has the same Peripheral Qualifier and Peripheral Device Type
> as the standard Inquiry data.

Byte 1

> It contains a page code. Cxh, Dxh, Exh, and Fxh of the page code
> indicate the adapter information, Set Window information, other
> information, and the experimental information, respectively.

Byte 3

> It contains the length of the page data from byte 4 onwards.
>
> Even if all the page data cannot be transferred because the CDB data
> length is short, this byte indicates the whole byte length of the
> defined page data.

Byte 4 and after

> These bytes are defined on each page.

If the VPD information is requested when the adapter is not installed,
this unit transfers the information with Peripheral Qualifier set to
one, Peripheral Device Type set to 6, page code that is requested, and
the page length set to one. The VPD information of byte 5 is the invalid
data.

**2-2-2-1. Page code list page**

<table>
<colgroup>
<col style="width: 12%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: right;"><p>Bit</p>
<p>Byte</p></td>
<td style="text-align: center;">7</td>
<td style="text-align: center;">6</td>
<td style="text-align: center;">5</td>
<td style="text-align: center;">4</td>
<td style="text-align: center;">3</td>
<td style="text-align: center;">2</td>
<td style="text-align: center;">1</td>
<td style="text-align: center;">0</td>
</tr>
<tr>
<td style="text-align: center;">0</td>
<td colspan="3" style="text-align: center;"><p>Peripheral Qualifier</p>
<p>[0]</p>
<p>[011b](*1)</p></td>
<td colspan="5" style="text-align: center;"><p>Peripheral Device
Type</p>
<p>[6=00110b]</p>
<p>[1Fh=11111b](*1)</p></td>
</tr>
<tr>
<td style="text-align: center;">1</td>
<td colspan="8" style="text-align: center;">Page code [00h]</td>
</tr>
<tr>
<td style="text-align: center;">2</td>
<td colspan="8" style="text-align: center;">Reserved [0]</td>
</tr>
<tr>
<td style="text-align: center;">3</td>
<td colspan="8" style="text-align: center;">Page length [m-3]</td>
</tr>
<tr>
<td style="text-align: center;">4 to m</td>
<td colspan="8" style="text-align: center;">Page code list [m-4]</td>
</tr>
</tbody>
</table>

> \*1 When an invalid logical unit selection is performed

This shows the list of information page codes that are supported by this
unit.

Byte 4 Page code list

> In this field, the information page codes that are supported by this
> unit are shown in units of one byte length in order starting from page
> code 00h.

<table>
<colgroup>
<col style="width: 42%" />
<col style="width: 57%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: center;">Attached adapter</td>
<td style="text-align: center;">Supported page (hex)</td>
</tr>
<tr>
<td style="text-align: left;">Common to all adapters</td>
<td style="text-align: center;">00, 01, 40, 41, 50, 51, 60, 61, 62, C1,
D1, E1, E3, F0, F8, FB, FC</td>
</tr>
<tr>
<td style="text-align: center;"><p>Mount adapter</p>
<p>(when a holder is attached)</p></td>
<td style="text-align: center;">Common to all adapters + 10</td>
</tr>
<tr>
<td style="text-align: center;">6-frame strip adapter</td>
<td style="text-align: center;">Common to all adapters + 46, E2</td>
</tr>
<tr>
<td style="text-align: center;">36-frame strip adapter</td>
<td style="text-align: center;">Common to all adapters + 47, E2</td>
</tr>
<tr>
<td style="text-align: center;">240 adapter</td>
<td style="text-align: center;">Common to all adapters + 43, E2</td>
</tr>
<tr>
<td style="text-align: center;">Slide feeder</td>
<td style="text-align: center;">Common to all adapters + 45, F1</td>
</tr>
<tr>
<td style="text-align: center;">None/Undefined</td>
<td style="text-align: center;">00, C1, FB, FC</td>
</tr>
</tbody>
</table>

Note) On the above supported pages, 40, 41, 43, 45, 46, 47, 50, 51, 60,
61, 62, F0, F1, F8, FA, FB, and FC are not used.

**2-2-2-2. FRU ASCII information page**

<table>
<colgroup>
<col style="width: 12%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: right;"><p>Bit</p>
<p>Byte</p></td>
<td style="text-align: center;">7</td>
<td style="text-align: center;">6</td>
<td style="text-align: center;">5</td>
<td style="text-align: center;">4</td>
<td style="text-align: center;">3</td>
<td style="text-align: center;">2</td>
<td style="text-align: center;">1</td>
<td style="text-align: center;">0</td>
</tr>
<tr>
<td style="text-align: center;">0</td>
<td colspan="3" style="text-align: center;"><p>Peripheral Qualifier</p>
<p>[0]</p>
<p>[011b](*1)</p></td>
<td colspan="5" style="text-align: center;"><p>Peripheral Device
Type</p>
<p>[6=00110b]</p>
<p>[1Fh=11111b](*1)</p></td>
</tr>
<tr>
<td style="text-align: center;">1</td>
<td colspan="8" style="text-align: center;">Page code [01 to 7Fh]</td>
</tr>
<tr>
<td style="text-align: center;">2</td>
<td colspan="8" style="text-align: center;">Reserved [0]</td>
</tr>
<tr>
<td style="text-align: center;">3</td>
<td colspan="8" style="text-align: center;">Page length [m-3]</td>
</tr>
<tr>
<td style="text-align: center;">4</td>
<td colspan="8" style="text-align: center;">ASCII data length [m-4]</td>
</tr>
<tr>
<td style="text-align: center;">5 to m</td>
<td colspan="8" style="text-align: center;">ASCII information</td>
</tr>
</tbody>
</table>

> \*1 When an invalid logical unit selection is performed
>
> This page contains the unit information such as the adapter name, the
> unit name of a stage or a motor, and the name of the experimental
> parameter as the ASCII character strings.

The page code becomes the ID of the unit information that is shown in
the ASCII information.

Byte 4 ASCII data length

> This field specifies the byte length of the ASCII information.

Byte 5 and after ASCII information

> This field specifies the unit information as the ASCII character
> strings.
>
> Only the graphic codes (20h to 7Eh) are used in the ASCII character
> strings, and the last one character in each line is set to the NULL
> code (00h).
>
> This unit supports the unit information shown in table 2-2-2-2-1.

Table 2-2-2-2-1 Adapter ID, adapter name, holder ID, and holder name

<table>
<colgroup>
<col style="width: 20%" />
<col style="width: 29%" />
<col style="width: 18%" />
<col style="width: 32%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: center;">Page code (ID)</td>
<td style="text-align: center;">Attached adapter</td>
<td style="text-align: center;">ASCII information</td>
<td style="text-align: center;">Descriptions</td>
</tr>
<tr>
<td rowspan="5" style="text-align: center;">01h</td>
<td style="text-align: center;">Mount adapter</td>
<td style="text-align: center;">Mount</td>
<td style="text-align: center;">Mount adapter</td>
</tr>
<tr>
<td style="text-align: center;">6-frame strip adapter</td>
<td style="text-align: center;">6Strip</td>
<td style="text-align: center;">6-frame strip adapter</td>
</tr>
<tr>
<td style="text-align: center;">36-frame strip adapter</td>
<td style="text-align: center;">36Strip</td>
<td style="text-align: center;">36-frame strip adapter</td>
</tr>
<tr>
<td style="text-align: center;">240 adapter</td>
<td style="text-align: center;">240</td>
<td style="text-align: center;">240 adapter</td>
</tr>
<tr>
<td style="text-align: center;">Slide feeder</td>
<td style="text-align: center;">Feeder</td>
<td style="text-align: center;">Slide feeder</td>
</tr>
<tr>
<td rowspan="3" style="text-align: center;">10h</td>
<td rowspan="3" style="text-align: center;">Mount adapter</td>
<td style="text-align: center;">FH3</td>
<td style="text-align: center;">6-frame strip holder</td>
</tr>
<tr>
<td style="text-align: center;">FHG1</td>
<td style="text-align: center;">Praparat holder</td>
</tr>
<tr>
<td style="text-align: center;">FHA1</td>
<td style="text-align: center;">APS holder</td>
</tr>
</tbody>
</table>

**  
2-2-2-3. Address information page**

Address information page

<table>
<colgroup>
<col style="width: 10%" />
<col style="width: 12%" />
<col style="width: 11%" />
<col style="width: 11%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: right;"><p>Bit</p>
<p>Byte</p></td>
<td style="text-align: center;">7</td>
<td style="text-align: center;">6</td>
<td style="text-align: center;">5</td>
<td style="text-align: center;">4</td>
<td style="text-align: center;">3</td>
<td style="text-align: center;">2</td>
<td style="text-align: center;">1</td>
<td style="text-align: center;">0</td>
</tr>
<tr>
<td style="text-align: center;">0</td>
<td colspan="3" style="text-align: center;"><p>Peripheral Qualifier</p>
<p>[0]</p>
<p>[011b](*1)</p></td>
<td colspan="5" style="text-align: center;"><p>Peripheral Device
Type</p>
<p>[6=00110b]</p>
<p>[1Fh=11111b](*1)</p></td>
</tr>
<tr>
<td style="text-align: center;">1</td>
<td colspan="8" style="text-align: center;">Page code [C1h]</td>
</tr>
<tr>
<td style="text-align: center;">2</td>
<td colspan="8" style="text-align: center;">Reserved [0]</td>
</tr>
<tr>
<td style="text-align: center;">3</td>
<td colspan="8" style="text-align: center;">Page length [83d=53h]</td>
</tr>
<tr>
<td style="text-align: center;">4</td>
<td colspan="8" style="text-align: center;"><blockquote>
<p>SCSI function support (SCSI data transfer function)</p>
<p>[03h] (Adapters other than the IA-20 adapter)</p>
<p>[0Bh] (IA-20 adapter)</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">5, 6</td>
<td colspan="8" style="text-align: center;"><blockquote>
<p>Window descriptor block length [61=003Dh]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">7, 8</td>
<td colspan="8" style="text-align: center;"><blockquote>
<p>Set parameter descriptor block length</p>
<p>(Length of the SET PARAMETER command parameter in bytes)</p>
<p>[15=000Fh]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">9, 10</td>
<td colspan="8" style="text-align: center;"><blockquote>
<p>General SCSI Buffer Size (SCSI data transfer buffer size. Unit: byte)
[0]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">11, 12</td>
<td colspan="8" style="text-align: center;"><blockquote>
<p>Image Buffer Size (Unit: KB) [256=0100h]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">13</td>
<td colspan="8" style="text-align: center;"><blockquote>
<p>Number of equipped Unit (the number of units that can be attached
simultaneously) [1]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">14</td>
<td colspan="8" style="text-align: center;"><blockquote>
<p>Unit Name ID (ID numbers of the attached adapter and the attached
holder)</p>
<p>[01h] (When an adapter is attached)</p>
<p>[0] (When an adapter is not attached or an undefined adapter is
attached)</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">15</td>
<td colspan="8" style="text-align: center;"><blockquote>
<p>Current Holder Name ID (the current holder name)</p>
<p>[10h] (When a holder is attached)</p>
<p>[0] (When a holder is not attached or an undefined holder is
attached)</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">16</td>
<td colspan="8" style="text-align: center;"><blockquote>
<p>Coordinate base information (resolution type and scanning that are
supported)</p>
<p>[0Fh] (The FH3 holder is inserted reversely.)</p>
<p>[13h] (IA-20)</p>
<p>[03h] (Adapter and holder other than the above)</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">17</td>
<td colspan="8" style="text-align: center;"><blockquote>
<p>Addressing Kind (addressing type that is supported)</p>
<p>[31h] (SA-21/SA-30)</p>
<p>[35h] (IA-20)</p>
</blockquote>
<p>[32h] (SF-210)</p>
<p>[22h] (Adapter and holder other than the above)</p></td>
</tr>
<tr>
<td style="text-align: center;">18, 19</td>
<td colspan="8" style="text-align: center;"><blockquote>
<p>X-Optical Resolution (Unit: dpi)</p>
<p>[4000=0FA0h]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">20, 21</td>
<td colspan="8" style="text-align: center;"><blockquote>
<p>X-Maximum Resolution (Unit: dpi)</p>
<p>[4000=0FA0h]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">22, 23</td>
<td colspan="8" style="text-align: center;"><blockquote>
<p>X-Minimum Resolution (Unit: dpi)</p>
<p>[90=005Ah]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">24 to 27</td>
<td colspan="8" style="text-align: center;"><blockquote>
<p>X-Maximum Set Window Address</p>
<p>(Window descriptor X-axis offset address maximum value)</p>
<p>[0]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">28 to 31</td>
<td colspan="8" style="text-align: center;"><blockquote>
<p>X-Minimum Set Window Address</p>
<p>(Window descriptor X-axis offset address minimum value)</p>
<p>[0]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">32 to 35</td>
<td colspan="8" style="text-align: center;"><blockquote>
<p>X-Offset for first image’s address (X-axis scanning start position
offset address)</p>
<p>[0]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">36 to 39</td>
<td colspan="8" style="text-align: center;"><blockquote>
<p>X-Set Window boundary</p>
<p>(Maximum window width value of the X-axis window descriptor)</p>
<p>[2916=00000B64h] (IA-20)</p>
<p>[3946=00000F6Ah] (Adapter and holder other than the above)</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">40, 41</td>
<td colspan="8" style="text-align: center;"><blockquote>
<p>Y-Optical Resolution (Unit: dpi)</p>
<p>[4000=0FA0h]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">42, 43</td>
<td colspan="8" style="text-align: center;"><blockquote>
<p>Y-Maximum Resolution (Unit: dpi)</p>
<p>[4000=0FA0h]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">44, 45</td>
<td colspan="8" style="text-align: center;"><blockquote>
<p>Y-Minimum Resolution (Unit: dpi)</p>
<p>[90=005Ah]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">46 to 49</td>
<td colspan="8" style="text-align: center;"><blockquote>
<p>Y-Maximum Set Window Address</p>
<p>(Window descriptor Y-axis offset address maximum value)</p>
<p>[*2] (SA-21/SA-30)</p>
<p>[*3] (IA-20)</p>
<p>[5781=00001695h] (Adapter and holder other than the above)</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">50 to 53</td>
<td colspan="8" style="text-align: center;"><blockquote>
<p>Y-Minimum Set Window Address</p>
<p>(Window descriptor Y-axis offset address minimum value) [0]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">54 to 57</td>
<td colspan="8" style="text-align: center;"><blockquote>
<p>Y-Offset for first image’s address (Y-axis scanning start position
offset address)</p>
<p>[0]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">58 to 61</td>
<td colspan="8" style="text-align: center;"><blockquote>
<p>Y-Set Window boundary</p>
<p>(Maximum window width value of the Y-axis window descriptor)</p>
<p>[5959=00001747h] (SA-21/SA-30)</p>
<p>[4453=00001165h] (IA-20)</p>
<p>[5782=00001696h] (Adapter and holder other than the above)</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">62 to 65</td>
<td colspan="8" style="text-align: center;"><blockquote>
<p>Y-Another world maximum Address</p>
<p>(Maximum address in the sub-scanning direction outside the specified
address)</p>
<p>[5959=00001747h] (SA-21/SA-30 adapter)</p>
<p>[5782=00001696h] (IA-20)</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">66 to 69</td>
<td colspan="8" style="text-align: center;"><blockquote>
<p>Y-Another world minimum Address</p>
<p>(Minimum address in the sub-scanning direction outside the specified
address)</p>
<p>[0] (SA-21/SA-30/IA-20)</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">70, 71</td>
<td colspan="8" style="text-align: center;"><blockquote>
<p>Maximum Thumbnail Resolution</p>
<p>(Maximum resolution in thumbnail scanning. Unit: dpi)</p>
<p>[97=0061h] (SA-21/SA-30)</p>
<p>[90=005Ah] (IA-20)</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">72, 73</td>
<td colspan="8" style="text-align: center;"><blockquote>
<p>Minimum Thumbnail Resolution</p>
<p>(Minimum resolution in thumbnail scanning. Unit: dpi)</p>
<p>[97=0061h] (SA-21/SA-30)</p>
<p>[90=005Ah] (IA-20)</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">74</td>
<td colspan="8" style="text-align: center;"><blockquote>
<p>Maximum Image count (maximum number of frames that can be
scanned)</p>
<p>[6] (SA-21)</p>
<p>[40=28h] (SA-30/IA-20)</p>
<p>[1] (Adapter and holder other than the above)</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">75</td>
<td colspan="8" style="text-align: center;"><p>Actual including image
count (the number of medium frames that are currently set)</p>
<blockquote>
<p>[*4] (SA-21/SA-30)</p>
<p>[1 to 40d] (IA-20)</p>
<p>[6] (When the number of frames is not known in SA-21. Ex.: When the
initialization of SA-21 is performed before the number of frames is
detected)</p>
<p>[0] (When a medium is not inserted in SA-21/SA-30/IA-20)</p>
<p>[40=28h] (When the number of frames is not known in SA-30/IA-20)</p>
<p>[1] (Adapter and holder other than the above)</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">76, 77</td>
<td colspan="8" style="text-align: center;">Minimum Focusing Address
(minimum address of the focus position) [0]</td>
</tr>
<tr>
<td style="text-align: center;">78, 79</td>
<td colspan="8" style="text-align: center;"><blockquote>
<p>Maximum Focusing Address (maximum address of the focus position)
[323=0143h]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">80, 81</td>
<td colspan="8" style="text-align: center;"><blockquote>
<p>Lamp warm-up maximum time (maximum time for lamp warming-up) [0]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">82</td>
<td colspan="8" style="text-align: center;"><blockquote>
<p>A/D bit depth (depth of bits for an A/D converter) [16=10h]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">83, 84</td>
<td colspan="8" style="text-align: center;"><blockquote>
<p>CCD Pixel Number</p>
<p>(The number of effective pixels in the CCD. For the CCD in which the
number of effective pixels differs in each color, the maximum value is
set.)</p>
<p>[3946=0F6Ah]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">85</td>
<td colspan="8" style="text-align: center;"><blockquote>
<p>Line Gap Count (the number of gaps between lines) [01h]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">86</td>
<td colspan="8" style="text-align: center;"><blockquote>
<p>CCD Line Number (the number of lines in the CCD) [02h]</p>
</blockquote></td>
</tr>
</tbody>
</table>

\*1 When an invalid logical unit selection is performed

\*2 Y-Maximum Set Window Address=(Actual including image count+2)\*5959

\*3 Y-Maximum Set Window Address=(Actual including image count)\*4453-1

> \*4 The number of frames of the strip adapter=Whole length of the
> strip film/Length of one frame

Byte 4 SCSI function support

> This field specifies the SCSI data transfer function.
>
> Setting each bit to zero indicates that the function is not supported
> by this unit, and setting each bit to one indicates that the function
> is supported by this unit. In this unit, this field is set to 0Bh for
> the IA-20, and 03h for the adapters other than the IA-20.

<table>
<colgroup>
<col style="width: 10%" />
<col style="width: 57%" />
<col style="width: 15%" />
<col style="width: 15%" />
</colgroup>
<tbody>
<tr>
<td rowspan="2" style="text-align: center;">Bit</td>
<td rowspan="2" style="text-align: center;"></td>
<td colspan="2" style="text-align: center;">Support of this unit</td>
</tr>
<tr>
<td style="text-align: center;">Adapter other than IA-20</td>
<td style="text-align: center;">IA-20</td>
</tr>
<tr>
<td style="text-align: center;">0</td>
<td style="text-align: left;"><blockquote>
<p>Microcode downloading function</p>
</blockquote></td>
<td style="text-align: center;">1</td>
<td style="text-align: center;">1</td>
</tr>
<tr>
<td style="text-align: center;">1</td>
<td style="text-align: left;"><blockquote>
<p>Image reading (READ command) must be performed in units of [Data of
one line in bytes * number of colors].</p>
</blockquote></td>
<td style="text-align: center;">1</td>
<td style="text-align: center;">1</td>
</tr>
<tr>
<td style="text-align: center;">2</td>
<td style="text-align: left;"><blockquote>
<p>Image reading (READ command) must be performed in units of [Data of
one line in bytes].</p>
</blockquote></td>
<td style="text-align: center;">0</td>
<td style="text-align: center;">0</td>
</tr>
<tr>
<td style="text-align: center;">3</td>
<td style="text-align: left;"><blockquote>
<p>Thumbnail reading (READ command) must be performed in units of [The
number of bytes in one frame* number of colors].</p>
</blockquote></td>
<td style="text-align: center;">0</td>
<td style="text-align: center;">1</td>
</tr>
<tr>
<td style="text-align: center;">4 to 6</td>
<td style="text-align: left;"><blockquote>
<p>Reserved</p>
</blockquote></td>
<td style="text-align: center;">0</td>
<td style="text-align: center;">0</td>
</tr>
<tr>
<td style="text-align: center;">7</td>
<td style="text-align: left;"><blockquote>
<p>Extend bit</p>
</blockquote></td>
<td style="text-align: center;">0</td>
<td style="text-align: center;">0</td>
</tr>
</tbody>
</table>

Byte 5, 6 Window descriptor block length

> This field specifies the length of the window descriptor in bytes. In
> this unit, this field is set to 61d.

Byte 7, 8 Set parameter descriptor block length

> This field specifies the length of the SET PARAMETER command parameter
> in bytes. In this unit, this field is set to 15d.

Byte 9, 10 General SCSI Buffer Size

> This field specifies the data size that is used for the SCSI data
> transfer in bytes. Zero indicates that the buffer size is not limited.
> In this unit, this field is set to zero.

Byte 11, 12 Image Buffer Size

> This field specifies the image buffer size in kilobytes. In this unit,
> this field is set to 64d.

Byte 13 Number of equipped Unit

> This field specifies the number of units that can be attached to this
> unit simultaneously. In this unit, this field is set to one.

Byte 14 Unit Name ID

> This field specifies the ID number of the adapter that is currently
> attached. In this unit, this field is set to 1 when the adapter is
> attached, and set to 0 when the adapter is not attached or an
> undefined adapter is attached.

Byte 15 Current Holder Name ID

> This field specifies the current holder name. In this unit, this field
> is set to 10h when the holder is attached, and set to 0 when the
> holder is not attached or an undefined holder is attached.

Byte 16 Coordinate base information

> Each bit in this field specifies the resolution type and the reading
> method that are supported.

<table>
<colgroup>
<col style="width: 6%" />
<col style="width: 37%" />
<col style="width: 56%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: center;">Bit0, 1</td>
<td>Resolution type [3]</td>
<td style="text-align: left;"><p>Setting this bit to 0 indicates that
reading can be performed in continuous resolution.</p>
<p>Setting this bit to 1 indicates that reading can be performed only in
the resolution of each pitch.</p>
<p>Setting this bit to 2 indicates that reading can be performed only in
the resolution of the pitch which is the measure of the maximum pitch.
(*1)</p>
<p>Setting this bit to 3 indicates that reading can be performed only in
the resolution of pitch 1 and an even pitch. (*2)</p></td>
</tr>
<tr>
<td style="text-align: center;">Bit2</td>
<td><p>X Origin Reversed</p>
<blockquote>
<p>[FH3 reverse direction=1/</p>
<p>Other=0]</p>
</blockquote></td>
<td style="text-align: left;">Setting this bit to 1 indicates that the
main-scanning direction origin is reversed (at the right end of the
medium).</td>
</tr>
<tr>
<td style="text-align: center;">Bit3</td>
<td><p>Y Origin Reversed</p>
<blockquote>
<p>[FH3 reverse direction=1/</p>
<p>Other=0]</p>
</blockquote></td>
<td style="text-align: left;">Setting this bit to 1 indicates that the
sub-scanning direction origin is reversed (at the bottom end of the
medium).</td>
</tr>
<tr>
<td style="text-align: center;">Bit4</td>
<td><p>Thumbnail Order Reversed</p>
<blockquote>
<p>[IA-20=1/SA-21, SA-30=0]</p>
</blockquote></td>
<td style="text-align: left;">Setting this bit to 0 indicates that the
thumbnail image is stored in the normal direction (first frame-&gt;last
frame). Setting this bit to 1 indicates that the thumbnail image is
stored in the reversed direction (last frame-&gt;first frame).</td>
</tr>
<tr>
<td style="text-align: center;">Bit5</td>
<td>Reserved [0]</td>
<td style="text-align: left;">This bit is set to 0 in this unit.</td>
</tr>
<tr>
<td style="text-align: center;">Bit6</td>
<td>Additional Coordinate Information [0]</td>
<td style="text-align: left;">This bit is set to 0 in this unit.</td>
</tr>
<tr>
<td style="text-align: center;">Bit7</td>
<td>Extend bit [0]</td>
<td style="text-align: left;">This bit is set to 0 in this unit.</td>
</tr>
</tbody>
</table>

> \*1: When the maximum pitch is 12, the pitches in which reading can be
> performed are 1, 2, 3, 4, 6, and 12.
>
> \*2: However, by way of exception, only for reading the thumbnail in
> SA-21/SA-30, reading is performed in the odd pitch (pitch 41) relative
> to the film movement length.

Byte 17 Addressing Kind

> This field specifies the addressing type that is supported. The
> addressing of the bit to which 1 is set is supported.

<table style="width:90%;">
<colgroup>
<col style="width: 6%" />
<col style="width: 3%" />
<col style="width: 6%" />
<col style="width: 38%" />
<col style="width: 6%" />
<col style="width: 3%" />
<col style="width: 3%" />
<col style="width: 6%" />
<col style="width: 6%" />
<col style="width: 6%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: center;">Bit</td>
<td colspan="3" style="text-align: center;">Descriptions</td>
<td colspan="6" style="text-align: center;">Adapter</td>
</tr>
<tr>
<td style="text-align: center;"></td>
<td colspan="3" style="text-align: center;"></td>
<td style="text-align: center;">MA-21</td>
<td colspan="2" style="text-align: center;">SA-21</td>
<td style="text-align: center;">SA-30</td>
<td style="text-align: center;">IA-20</td>
<td style="text-align: center;">SF-210</td>
</tr>
<tr>
<td style="text-align: center;">0</td>
<td colspan="3" style="text-align: left;"><blockquote>
<p>The Set Window address is the same as the medium position
address.</p>
</blockquote></td>
<td style="text-align: center;">0</td>
<td colspan="2" style="text-align: center;">1</td>
<td style="text-align: center;">1</td>
<td style="text-align: center;">1</td>
<td style="text-align: center;">0</td>
</tr>
<tr>
<td style="text-align: center;">1</td>
<td colspan="3" style="text-align: left;"><blockquote>
<p>The Set Window address is the same as the address of the mechanical
block.</p>
</blockquote></td>
<td style="text-align: center;">1</td>
<td colspan="2" style="text-align: center;">0</td>
<td style="text-align: center;">0</td>
<td style="text-align: center;">0</td>
<td style="text-align: center;">1</td>
</tr>
<tr>
<td style="text-align: center;">2</td>
<td colspan="3" style="text-align: left;"><blockquote>
<p>Specifying the scanning range over two or more frames is
prohibited.</p>
</blockquote></td>
<td style="text-align: center;">0</td>
<td colspan="2" style="text-align: center;">0</td>
<td style="text-align: center;">0</td>
<td style="text-align: center;">1</td>
<td style="text-align: center;">0</td>
</tr>
<tr>
<td style="text-align: center;">3</td>
<td colspan="3" style="text-align: left;"><blockquote>
<p>Reserved</p>
</blockquote></td>
<td style="text-align: center;">0</td>
<td colspan="2" style="text-align: center;">0</td>
<td style="text-align: center;">0</td>
<td style="text-align: center;">0</td>
<td style="text-align: center;">0</td>
</tr>
<tr>
<td style="text-align: center;">4</td>
<td colspan="3" style="text-align: left;"><blockquote>
<p>The position of the medium can be operated.</p>
</blockquote></td>
<td style="text-align: center;">0</td>
<td colspan="2" style="text-align: center;">1</td>
<td style="text-align: center;">1</td>
<td style="text-align: center;">1</td>
<td style="text-align: center;">1</td>
</tr>
<tr>
<td style="text-align: center;">5</td>
<td colspan="3" style="text-align: left;"><blockquote>
<p>The mechanical block position can be operated.</p>
</blockquote></td>
<td style="text-align: center;">1</td>
<td colspan="2" style="text-align: center;">1</td>
<td style="text-align: center;">1</td>
<td style="text-align: center;">1</td>
<td style="text-align: center;">1</td>
</tr>
<tr>
<td style="text-align: center;">6</td>
<td colspan="3" style="text-align: left;"><blockquote>
<p>Reserved</p>
</blockquote></td>
<td style="text-align: center;">0</td>
<td colspan="2" style="text-align: center;">0</td>
<td style="text-align: center;">0</td>
<td style="text-align: center;">0</td>
<td style="text-align: center;">0</td>
</tr>
<tr>
<td style="text-align: center;">7</td>
<td colspan="3" style="text-align: left;"><blockquote>
<p>Extension bit</p>
</blockquote></td>
<td style="text-align: center;">0</td>
<td colspan="2" style="text-align: center;">0</td>
<td style="text-align: center;">0</td>
<td style="text-align: center;">0</td>
<td style="text-align: center;">0</td>
</tr>
</tbody>
</table>

MA-21: Mount adapter (including each holder), SA-21: 6-frame strip
adapter,

SA-30: 36-frame strip adapter, IA-20: 240 adapter, SF-210: Slide feeder

Byte 24 to 27 X-Maximum Set Window Address

> This field specifies the maximum value of the X-axis offset address
> for the window descriptor.
>
> If this address is the same as X-Minimum Set Window Address in byte 28
> to 31, it means that this unit cannot specify the optional reading
> range in the X-axis direction.
>
> The initiator must set the reading range in the X-axis direction to
> the value of X-Set Window boundary (Maximum window width value of the
> X-axis window descriptor) in byte 36 to 39 when reading is performed.

Byte 46 to 49 Y-Maximum Set Window Address

> This field specifies the maximum value of the Y-axis offset address
> for the window descriptor.
>
> If this address is the same as Y-Minimum Set Window Address in byte 50
> to 53, it means that this unit cannot specify the optional reading
> range in the Y-axis direction.
>
> The initiator must set the reading range in the Y-axis direction to
> the value of Y-Set Window boundary (Maximum window width value of the
> Y-axis window descriptor) in byte 58 to 61 when reading is performed.

Byte 75 Actual including image count

> This field specifies the number of frames of the medium that is
> currently set.
>
> For the 240 adapter, this field is set to 0 when initialization is
> performed before the number of frames is detected (the number of
> frames is unknown), and set to the number of frames (1 to 40) after it
> is detected.
>
> For the 6-frame strip adapter, this field is set to 6 when
> initialization is performed before the number of frames is detected,
> and set to the number of frames after it is detected.
>
> For the 36-frame strip adapter, this field is set to 40 when
> initialization is performed before the number of frames is detected,
> and set to the number of frames after it is detected.
>
> The number of frames for the 6- or 36-frame strip adapter is (Whole
> length of the strip film/Length of one frame) counting decimals as
> one.
>
> For the adapters other than the above, this field is set to 1.

Byte 86 CCD Line Number

> This field specifies the number of lines in the CCD. When 0 is set or
> no value is sent to this field, ‘3 lines’ is set.

Note) If this page is requested when the adapter is not attached or an
undefined adapter is attached, the data up to byte 14 (allocation length
15 bytes) is returned.

Address information page set value

<table style="width:100%;">
<colgroup>
<col style="width: 9%" />
<col style="width: 32%" />
<col style="width: 11%" />
<col style="width: 11%" />
<col style="width: 11%" />
<col style="width: 11%" />
<col style="width: 0%" />
<col style="width: 11%" />
</colgroup>
<tbody>
<tr>
<td rowspan="2" style="text-align: center;">Byte</td>
<td rowspan="2"></td>
<td colspan="6" style="text-align: center;">Set value</td>
</tr>
<tr>
<td style="text-align: center;">MA-21 (*3)</td>
<td style="text-align: center;">SA-21</td>
<td style="text-align: center;">SA-30</td>
<td style="text-align: center;">SF-210</td>
<td colspan="2" style="text-align: center;">IA-20</td>
</tr>
<tr>
<td style="text-align: center;">18, 19</td>
<td>X-Optical Resolution</td>
<td colspan="6" style="text-align: center;">4000</td>
</tr>
<tr>
<td style="text-align: center;">20, 21</td>
<td>X-Maximum Resolution</td>
<td colspan="6" style="text-align: center;">4000</td>
</tr>
<tr>
<td style="text-align: center;">22, 23</td>
<td>X-Minimum Resolution</td>
<td colspan="6" style="text-align: center;">90</td>
</tr>
<tr>
<td style="text-align: center;">24 to 27</td>
<td>X-Maximum Set Window Address</td>
<td colspan="6" style="text-align: center;">0</td>
</tr>
<tr>
<td style="text-align: center;">28 to 31</td>
<td>X-Minimum Set Window Address</td>
<td colspan="6" style="text-align: center;">0</td>
</tr>
<tr>
<td style="text-align: center;">36 to 39</td>
<td>X-Set Window boundary</td>
<td colspan="5" style="text-align: center;">3946</td>
<td style="text-align: center;">2916</td>
</tr>
<tr>
<td style="text-align: center;">40, 41</td>
<td>Y-Optical Resolution</td>
<td colspan="6" style="text-align: center;">4000</td>
</tr>
<tr>
<td style="text-align: center;">42, 43</td>
<td>Y-Maximum Resolution</td>
<td colspan="6" style="text-align: center;">4000</td>
</tr>
<tr>
<td style="text-align: center;">44, 45</td>
<td>Y-Minimum Resolution</td>
<td colspan="6" style="text-align: center;">90</td>
</tr>
<tr>
<td style="text-align: center;">46 to 49</td>
<td>Y-Maximum Set Window Address</td>
<td style="text-align: center;">5781</td>
<td style="text-align: center;">(*1)</td>
<td style="text-align: center;">(*1)</td>
<td style="text-align: center;">5781</td>
<td colspan="2" style="text-align: center;">(*2)</td>
</tr>
<tr>
<td style="text-align: center;">50 to 53</td>
<td>Y-Minimum Set Window Address</td>
<td style="text-align: center;">0</td>
<td style="text-align: center;">0</td>
<td style="text-align: center;">0</td>
<td style="text-align: center;">0</td>
<td colspan="2" style="text-align: center;">0</td>
</tr>
<tr>
<td style="text-align: center;">58 to 61</td>
<td>Y-Set Window boundary</td>
<td style="text-align: center;">5782</td>
<td style="text-align: center;">5959</td>
<td style="text-align: center;">5959</td>
<td style="text-align: center;">5782</td>
<td colspan="2" style="text-align: center;">4453</td>
</tr>
<tr>
<td style="text-align: center;">62 to 65</td>
<td>Y-Another world maximum Address</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">5959</td>
<td style="text-align: center;">5959</td>
<td style="text-align: center;">-</td>
<td colspan="2" style="text-align: center;">5782</td>
</tr>
<tr>
<td style="text-align: center;">66 to 69</td>
<td>Y-Another world minimum Address</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">0</td>
<td style="text-align: center;">0</td>
<td style="text-align: center;">-</td>
<td colspan="2" style="text-align: center;">0</td>
</tr>
<tr>
<td style="text-align: center;">70, 71</td>
<td>Maximum Thumbnail Resolution</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">97</td>
<td style="text-align: center;">97</td>
<td style="text-align: center;">-</td>
<td colspan="2" style="text-align: center;">90</td>
</tr>
<tr>
<td style="text-align: center;">72, 73</td>
<td>Minimum Thumbnail Resolution</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">97</td>
<td style="text-align: center;">97</td>
<td style="text-align: center;">-</td>
<td colspan="2" style="text-align: center;">90</td>
</tr>
<tr>
<td style="text-align: center;">76, 77</td>
<td>Minimum Focusing Address</td>
<td colspan="6" style="text-align: center;">0</td>
</tr>
<tr>
<td style="text-align: center;">78, 79</td>
<td>Maximum Focusing Address</td>
<td colspan="6" style="text-align: center;">323</td>
</tr>
</tbody>
</table>

\*1 Y-Maximum Set Window Address=(Actual Including Image Count+2)\*5959

\*2 Y-Maximum Set Window Address=(Actual Including Image Count)\*4453-1

\*3 Each holder of FH3, FHG1, and FHA1 is included.

**2-2-2-4. SET WINDOW function page**

SET WINDOW function page

<table>
<colgroup>
<col style="width: 12%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: right;"><p>Bit</p>
<p>Byte</p></td>
<td style="text-align: center;">7</td>
<td style="text-align: center;">6</td>
<td style="text-align: center;">5</td>
<td style="text-align: center;">4</td>
<td style="text-align: center;">3</td>
<td style="text-align: center;">2</td>
<td style="text-align: center;">1</td>
<td style="text-align: center;">0</td>
</tr>
<tr>
<td style="text-align: center;">0</td>
<td colspan="3" style="text-align: center;"><p>Peripheral Qualifier</p>
<p>[0]</p>
<p>[011b](*1)</p></td>
<td colspan="5" style="text-align: center;"><p>Peripheral Device
Type</p>
<p>[6=00110b]</p>
<p>[1Fh=11111b](*1)</p></td>
</tr>
<tr>
<td style="text-align: center;">1</td>
<td colspan="8" style="text-align: center;">Page code [D1h]</td>
</tr>
<tr>
<td style="text-align: center;">2</td>
<td colspan="8" style="text-align: center;">Reserved [0]</td>
</tr>
<tr>
<td style="text-align: center;">3</td>
<td colspan="8" style="text-align: center;">Page length [24d=18h]</td>
</tr>
<tr>
<td style="text-align: center;">4</td>
<td colspan="8" style="text-align: left;"><blockquote>
<p>Scanning Kind Support</p>
</blockquote>
<p>[03h] (SA-21/SA-30/IA-20)</p>
<p>[01h] (Adapter and holder other than the above)</p></td>
</tr>
<tr>
<td style="text-align: center;">5</td>
<td colspan="8" style="text-align: left;"><blockquote>
<p>Scan Mode Support</p>
<p>[52h]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">6</td>
<td colspan="8" style="text-align: left;"><blockquote>
<p>Color Interleaving Support (color order for data transfer)</p>
<p>[42h]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">7</td>
<td colspan="8" style="text-align: left;"><blockquote>
<p>Color Component</p>
<p>[06h]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">8</td>
<td colspan="8" style="text-align: left;"><blockquote>
<p>Color Ordering1</p>
<p>[20h]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">9</td>
<td colspan="8" style="text-align: left;"><blockquote>
<p>Color Ordering2</p>
<p>[43h]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">10</td>
<td colspan="8" style="text-align: center;"><blockquote>
<p>Output Bit Depth/Dot a Color Support (the number of bits in one-color
data)</p>
<p>[20h]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">11</td>
<td colspan="8" style="text-align: left;"><blockquote>
<p>Number of Setup Mode</p>
<p>[0]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">12</td>
<td colspan="8" style="text-align: left;"><blockquote>
<p>Digital Image Control Support</p>
<p>[0]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">13</td>
<td colspan="8" style="text-align: left;"><blockquote>
<p>Additional length for Digital Control Information</p>
<p>[0]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">14</td>
<td colspan="8" style="text-align: left;"><blockquote>
<p>Analog Control Support</p>
<p>[40h]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">15</td>
<td colspan="8" style="text-align: left;"><blockquote>
<p>Additional length for Analog Control Information</p>
<p>[9]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">16 to 24</td>
<td colspan="8" style="text-align: left;"><blockquote>
<p>for the First Supported Control (exposure value control
parameter)</p>
<p>Byte 16 Bytes a Value for the control (parameter length in bytes)</p>
<p>[4]</p>
<p>Byte 17 to 20 Minimum Value for the First Control</p>
<p>[00000001h]</p>
<p>Byte 21 to 24 Maximum Value for the First Control</p>
<p>[03FFFFFFh]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">25</td>
<td colspan="8" style="text-align: left;"><blockquote>
<p>Filter Support</p>
<p>[0]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">26</td>
<td colspan="8" style="text-align: left;"><blockquote>
<p>Matrix Support</p>
<p>[0]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">27</td>
<td colspan="8" style="text-align: left;"><blockquote>
<p>Halftone Support</p>
<p>[0]</p>
</blockquote></td>
</tr>
</tbody>
</table>

\*1 When an invalid logical unit selection is performed

Byte 4 Scanning Kind Support

> This field specifies the image scanning types that are supported.
>
> For this unit, all adapters support Image Scanning, and the 6-frame
> strip adapter (6SA), 36-frame strip adapter (36SA), and 240 adapter
> support Thumbnail Scanning in addition.

<table>
<colgroup>
<col style="width: 6%" />
<col style="width: 23%" />
<col style="width: 47%" />
<col style="width: 11%" />
<col style="width: 11%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: center;">Bit</td>
<td style="text-align: center;">Type</td>
<td style="text-align: center;">Explanations of operation</td>
<td style="text-align: left;">IA-20/SA-21/SA-30</td>
<td style="text-align: left;">Other adapters</td>
</tr>
<tr>
<td style="text-align: center;">0</td>
<td style="text-align: left;">Image Scanning</td>
<td style="text-align: left;">Normal image scanning</td>
<td style="text-align: center;">1</td>
<td style="text-align: center;">1</td>
</tr>
<tr>
<td style="text-align: center;">1</td>
<td style="text-align: left;">Thumbnail Scanning</td>
<td style="text-align: left;">Thumbnail image scanning</td>
<td style="text-align: center;">1</td>
<td style="text-align: center;">0</td>
</tr>
<tr>
<td style="text-align: center;">2</td>
<td style="text-align: left;">Set up Scanning</td>
<td style="text-align: left;"><p>Prescan</p>
<p>Scanning for deciding the optimal integral time and gain,
etc.</p></td>
<td style="text-align: center;">0</td>
<td style="text-align: center;">0</td>
</tr>
<tr>
<td style="text-align: center;">3</td>
<td style="text-align: left;">Set up Scanning2</td>
<td style="text-align: left;"><p>Prescan</p>
<p>Scanning for deciding the optimal integral time and gain, etc. The
low-density/high-density limit values are used instead of the maximum
value and the minimum value. When the bit is 1, Setup Mode in the window
descriptor of SET WINDOW is supported. For the number of supports, refer
to ‘Number of Setup mode’ field.</p></td>
<td style="text-align: center;">0</td>
<td style="text-align: center;">0</td>
</tr>
<tr>
<td style="text-align: center;">4</td>
<td style="text-align: left;">Histogram Scanning</td>
<td style="text-align: left;">Scanning for creating the image data
histogram</td>
<td style="text-align: center;">0</td>
<td style="text-align: center;">0</td>
</tr>
<tr>
<td style="text-align: center;">5</td>
<td style="text-align: left;">Auto Exposure Scanning</td>
<td style="text-align: left;">Scanning for deciding the integral time at
which the output value becomes the AE Value that is set in each
color</td>
<td style="text-align: center;">0</td>
<td style="text-align: center;">0</td>
</tr>
<tr>
<td style="text-align: center;">6</td>
<td style="text-align: left;">AE with WB Scanning</td>
<td style="text-align: left;">Scanning for deciding the integral time at
which the maximum value of the output values in each color becomes the
AE Value that is set with the white balance maintained</td>
<td style="text-align: center;">0</td>
<td style="text-align: center;">0</td>
</tr>
<tr>
<td style="text-align: center;">7</td>
<td style="text-align: left;">Extend bit</td>
<td style="text-align: center;">Extension bit [0]</td>
<td style="text-align: center;">0</td>
<td style="text-align: center;">0</td>
</tr>
</tbody>
</table>

\[03h\] \[01h\]

Byte 5 Scan Mode Support

> This field specifies the scanning mode.
>
> Normal Quality Scan, Multiple Reading Scan, and Reverse direction
> Scanning Supported are supported.

<table>
<colgroup>
<col style="width: 12%" />
<col style="width: 75%" />
<col style="width: 11%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: center;">Bit0</td>
<td><blockquote>
<p>High Quality Scan</p>
</blockquote></td>
<td style="text-align: center;">[0]</td>
</tr>
<tr>
<td style="text-align: center;">Bit1</td>
<td><blockquote>
<p>Normal Quality Scan</p>
</blockquote></td>
<td style="text-align: center;">[1]</td>
</tr>
<tr>
<td style="text-align: center;">Bit2</td>
<td><blockquote>
<p>High Speed Scan</p>
</blockquote></td>
<td style="text-align: center;">[0]</td>
</tr>
<tr>
<td style="text-align: center;">Bit3</td>
<td><blockquote>
<p>Reserved</p>
</blockquote></td>
<td style="text-align: center;">[0]</td>
</tr>
<tr>
<td style="text-align: center;">Bit4</td>
<td><blockquote>
<p>Multiple Reading Scan</p>
</blockquote></td>
<td style="text-align: center;">[1]</td>
</tr>
<tr>
<td style="text-align: center;">Bit5</td>
<td><blockquote>
<p>Reserved</p>
</blockquote></td>
<td style="text-align: center;">[0]</td>
</tr>
<tr>
<td style="text-align: center;">Bit6</td>
<td><blockquote>
<p>Reverse direction Scanning Supported</p>
</blockquote></td>
<td style="text-align: center;">[1]</td>
</tr>
<tr>
<td style="text-align: center;">Bit7</td>
<td><blockquote>
<p>Extend bit</p>
</blockquote></td>
<td style="text-align: center;">[0]</td>
</tr>
</tbody>
</table>

Byte 6 Color Interleaving Support

> This field specifies the color order for data transfer.
>
> This unit supports ‘Line without CCD distance’ and ‘Multi line
> Simultaneous reading’.

<table>
<colgroup>
<col style="width: 13%" />
<col style="width: 74%" />
<col style="width: 12%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: center;">Bit0</td>
<td><blockquote>
<p>Pixel without CCD distance</p>
</blockquote></td>
<td style="text-align: center;">[0]</td>
</tr>
<tr>
<td style="text-align: center;">Bit1</td>
<td><blockquote>
<p>Line without CCD distance</p>
</blockquote></td>
<td style="text-align: center;">[1]</td>
</tr>
<tr>
<td style="text-align: center;">Bit2</td>
<td><blockquote>
<p>Plane</p>
</blockquote></td>
<td style="text-align: center;">[0]</td>
</tr>
<tr>
<td style="text-align: center;">Bit3</td>
<td><blockquote>
<p>Reserved</p>
</blockquote></td>
<td style="text-align: center;">[0]</td>
</tr>
<tr>
<td style="text-align: center;">Bit4</td>
<td><blockquote>
<p>Pixel with CCD distance</p>
</blockquote></td>
<td style="text-align: center;">[0]</td>
</tr>
<tr>
<td style="text-align: center;">Bit5</td>
<td><blockquote>
<p>Line with CCD distance</p>
</blockquote></td>
<td style="text-align: center;">[0]</td>
</tr>
<tr>
<td style="text-align: center;">Bit6</td>
<td><blockquote>
<p>Multi line Simultaneous reading</p>
</blockquote></td>
<td style="text-align: center;">[1]</td>
</tr>
<tr>
<td style="text-align: center;">Bit7</td>
<td><blockquote>
<p>Reserved</p>
</blockquote></td>
<td style="text-align: center;">[0]</td>
</tr>
</tbody>
</table>

Byte 7 Color Component

> This field specifies the color composition to be scanned. Dropout
> Color and R-G-B are supported.

<table>
<colgroup>
<col style="width: 13%" />
<col style="width: 72%" />
<col style="width: 14%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: center;">Bit0</td>
<td><blockquote>
<p>Neutral Gray Scale</p>
</blockquote></td>
<td style="text-align: center;">[0]</td>
</tr>
<tr>
<td style="text-align: center;">Bit1</td>
<td><blockquote>
<p>Dropout Color</p>
</blockquote></td>
<td style="text-align: center;">[1]</td>
</tr>
<tr>
<td style="text-align: center;">Bit2</td>
<td><blockquote>
<p>R-G-B</p>
</blockquote></td>
<td style="text-align: center;">[1]</td>
</tr>
<tr>
<td style="text-align: center;">Bit3</td>
<td><blockquote>
<p>C-M-Y</p>
</blockquote></td>
<td style="text-align: center;">[0]</td>
</tr>
<tr>
<td style="text-align: center;">Bit4</td>
<td><blockquote>
<p>Reserved</p>
</blockquote></td>
<td style="text-align: center;">[0]</td>
</tr>
<tr>
<td style="text-align: center;">Bit5</td>
<td><blockquote>
<p>Reserved</p>
</blockquote></td>
<td style="text-align: center;">[0]</td>
</tr>
<tr>
<td style="text-align: center;">Bit6</td>
<td><blockquote>
<p>Reserved</p>
</blockquote></td>
<td style="text-align: center;">[0]</td>
</tr>
<tr>
<td style="text-align: center;">Bit7</td>
<td><blockquote>
<p>Extend bit</p>
</blockquote></td>
<td style="text-align: center;">[0]</td>
</tr>
</tbody>
</table>

Byte 8 Color Ordering1

> This field specifies the color ordering in which this unit can read
> the data.
>
> In the case of R-G-B scanning, the setting is R=1, G=2, B=3. In the
> case of C-M-Y, the setting is C=1, M=2, Y=3. 0 indicates that all
> colors can be scanned.
>
> Bit 0 to 3 specify the color that can be scanned as the first color.
> This field is set to 0 in this unit.
>
> Bit 4 to 7 specify the color that can be scanned as the second color.
> This field is set to 2 in this unit.

<table>
<colgroup>
<col style="width: 13%" />
<col style="width: 86%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: center;">Bit0-3</td>
<td><blockquote>
<p>First component color</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">Bit4-7</td>
<td><blockquote>
<p>Second component color</p>
</blockquote></td>
</tr>
</tbody>
</table>

Byte 9 Color Ordering2

> This field specifies the color ordering in which this unit can read
> the data.
>
> In the case of R-G-B scanning, the setting is R=1, G=2, B=3. In the
> case of C-M-Y, the setting is C=1, M=2, Y=3. 0 indicates that all
> colors can be scanned.
>
> Bit 0 to 3 specify the color that can be scanned as the third color.
> This field is set to 3 in this unit.
>
> Bit 4 to 7 specify the color that can be scanned as the fourth color.
> This field is set to 4 in this unit.

<table>
<colgroup>
<col style="width: 13%" />
<col style="width: 86%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: center;">Bit0-3</td>
<td><blockquote>
<p>Third component color</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">Bit4-7</td>
<td><blockquote>
<p>Fourth component color</p>
</blockquote></td>
</tr>
</tbody>
</table>

Byte 10 Output Bit Depth / Dot a Color Support

> This field specifies the number of bits of a single color data. This
> unit supports 16bit.

<table>
<colgroup>
<col style="width: 13%" />
<col style="width: 74%" />
<col style="width: 12%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: center;">Bit0</td>
<td><blockquote>
<p>1bit a color</p>
</blockquote></td>
<td style="text-align: center;">[0]</td>
</tr>
<tr>
<td style="text-align: center;">Bit1</td>
<td><blockquote>
<p>8bit a color</p>
</blockquote></td>
<td style="text-align: center;">[0]</td>
</tr>
<tr>
<td style="text-align: center;">Bit2</td>
<td><blockquote>
<p>10bit a color</p>
</blockquote></td>
<td style="text-align: center;">[0]</td>
</tr>
<tr>
<td style="text-align: center;">Bit3</td>
<td><blockquote>
<p>12bit a color</p>
</blockquote></td>
<td style="text-align: center;">[0]</td>
</tr>
<tr>
<td style="text-align: center;">Bit4</td>
<td><blockquote>
<p>14bit a color</p>
</blockquote></td>
<td style="text-align: center;">[0]</td>
</tr>
<tr>
<td style="text-align: center;">Bit5</td>
<td><blockquote>
<p>16bit a color</p>
</blockquote></td>
<td style="text-align: center;">[1]</td>
</tr>
<tr>
<td style="text-align: center;">Bit6</td>
<td><blockquote>
<p>Reserved</p>
</blockquote></td>
<td style="text-align: center;">[0]</td>
</tr>
<tr>
<td style="text-align: center;">Bit7</td>
<td><blockquote>
<p>Extend bit</p>
</blockquote></td>
<td style="text-align: center;">[0]</td>
</tr>
</tbody>
</table>

Byte 11 Number of Setup Mode

> This field specifies the number of setup modes that are supported. It
> becomes effective when Setup Scan2 of the Scanning Kind support field
> is 1. The Setup Scan of (the number specified here + 1) types can be
> set.
>
> 0 is set in this unit.

Byte 12 Digital Image Control Support

> This field specifies the digital image control function that is
> supported.
>
> This unit does not support the digital image control.

Byte 13 Additional length for Digital Control Information

> This field specifies the additional information length (in bytes) of
> digital image control.
>
> This unit sets this field to 0.

Byte 14 Analog Control Support

> This field specifies the analog image control function that is
> supported.
>
> This unit supports the exposure value.

<table>
<colgroup>
<col style="width: 14%" />
<col style="width: 73%" />
<col style="width: 12%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: center;">Bit0</td>
<td><blockquote>
<p>Analog Gamma</p>
</blockquote></td>
<td style="text-align: center;">[0]</td>
</tr>
<tr>
<td style="text-align: center;">Bit1</td>
<td><blockquote>
<p>Exposure Time</p>
</blockquote></td>
<td style="text-align: center;">[0]</td>
</tr>
<tr>
<td style="text-align: center;">Bit2</td>
<td><blockquote>
<p>Analog Gain</p>
</blockquote></td>
<td style="text-align: center;">[0]</td>
</tr>
<tr>
<td style="text-align: center;">Bit3</td>
<td><blockquote>
<p>Digital Gain</p>
</blockquote></td>
<td style="text-align: center;">[0]</td>
</tr>
<tr>
<td style="text-align: center;">Bit4</td>
<td><blockquote>
<p>Analog Shift</p>
</blockquote></td>
<td style="text-align: center;">[0]</td>
</tr>
<tr>
<td style="text-align: center;">Bit5</td>
<td><blockquote>
<p>Analog Offset</p>
</blockquote></td>
<td style="text-align: center;">[0]</td>
</tr>
<tr>
<td style="text-align: center;">Bit6</td>
<td><blockquote>
<p>Exposure Value</p>
</blockquote></td>
<td style="text-align: center;">[1]</td>
</tr>
<tr>
<td style="text-align: center;">Bit7</td>
<td><blockquote>
<p>Extend bit</p>
</blockquote></td>
<td style="text-align: center;">[0]</td>
</tr>
</tbody>
</table>

Byte 15 Additional length for Analog Control Information

> This field specifies the analog image control function that is
> supported.
>
> This unit sets this field to 9.

Byte 16 to 24 First support function control parameter

> The first support function in this unit is the exposure value control.

Byte 16 Bytes a Value for the Control

> This specifies the length (in bytes) of the parameter for this
> function.
>
> The unit of the integral time setting is 10 nsec.

Byte 17 to 20 Minimum Value for the First Control

> This specifies the minimum value that can be set.
>
> The minimum value of the integral time in this unit is 10 nsec.

Byte 21 to 24 Maximum Value for the First Control

> This specifies the maximum value that can be set.
>
> The maximum value of the integral time in this unit is 03FFFFFFh\*10
> nsec.

Byte 25 Filter Support

> This unit does not support the filter.

Byte 26 Matrix Support

> This unit does not support the matrix.

Byte 27 Halftone Support

> This field specifies the halftone that is supported.
>
> This unit does not support the halftone.

**  **
**2-2-2-5. Other information page**

Other information page

<table>
<colgroup>
<col style="width: 12%" />
<col style="width: 11%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: right;"><p>Bit</p>
<p>Byte</p></td>
<td style="text-align: center;">7</td>
<td style="text-align: center;">6</td>
<td style="text-align: center;">5</td>
<td style="text-align: center;">4</td>
<td style="text-align: center;">3</td>
<td style="text-align: center;">2</td>
<td style="text-align: center;">1</td>
<td style="text-align: center;">0</td>
</tr>
<tr>
<td style="text-align: center;">0</td>
<td colspan="3" style="text-align: center;"><p>Peripheral Qualifier</p>
<p>[0]</p>
<p>[011b](*1)</p></td>
<td colspan="5" style="text-align: center;"><p>Peripheral Device
Type</p>
<p>[6=00110b]</p>
<p>[1Fh=11111b](*1)</p></td>
</tr>
<tr>
<td style="text-align: center;">1</td>
<td colspan="8" style="text-align: center;">Page code [E1h]</td>
</tr>
<tr>
<td style="text-align: center;">2</td>
<td colspan="8" style="text-align: center;">Reserved [0]</td>
</tr>
<tr>
<td style="text-align: center;">3</td>
<td colspan="8" style="text-align: center;">Page length [35d=23h]</td>
</tr>
<tr>
<td style="text-align: center;">4, 5</td>
<td colspan="8" style="text-align: left;"><blockquote>
<p>Host cooperation function (initiator cooperation execution
processing)</p>
<p>Byte 4 [83h] (SA-21/SA-30)</p>
<p>[82h] (Adapter and holder other than the above)</p>
<p>Byte 5 [0Ch]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">6 to 10</td>
<td colspan="8" style="text-align: left;"><blockquote>
<p>Send/Read supported information (SEND/READ command support data
transfer)</p>
<p>Byte 6 [80h]</p>
<p>Byte 7 [B0h]</p>
<p>Byte 8 [90h]</p>
<p>Byte 9 [DAh] (SA-21/SA-30)</p>
</blockquote>
<p>[9Ah] (Adapter and holder other than the above)</p>
<p>Byte 10 [7Bh] (SA-21/SA-30)</p>
<p>[78h] (IA-21/SF-210)</p>
<p>[7Ch] (Adapter and holder other than the above)</p></td>
</tr>
<tr>
<td style="text-align: center;">11</td>
<td colspan="8" style="text-align: left;"><blockquote>
<p>Bits per a halftone mask parameter</p>
<p>[0]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">12</td>
<td colspan="8" style="text-align: left;"><blockquote>
<p>X bit depth of Download LUT</p>
<p>(The number of bits in the input data of the LUT that is downloaded
from the initiator)</p>
<p>[0]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">13</td>
<td colspan="8" style="text-align: left;"><blockquote>
<p>Y bit depth of Download LUT</p>
<p>(The number of bits in the output data of the LUT that is downloaded
from the initiator)</p>
<p>[0]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">14</td>
<td colspan="8" style="text-align: left;"><blockquote>
<p>Bits per a Histogram Data</p>
<p>[0]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">15</td>
<td colspan="8" style="text-align: left;"><blockquote>
<p>Bits per a Max Value Data</p>
<p>(The number of bits of the AE maximum value)</p>
<p>[0]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">16</td>
<td colspan="8" style="text-align: left;"><blockquote>
<p>Bits per a Matrix Data</p>
<p>[0]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">17</td>
<td colspan="8" style="text-align: left;"><blockquote>
<p>Bits per a Filter Data</p>
<p>[0]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">18</td>
<td colspan="8" style="text-align: left;"><blockquote>
<p>Bits per a Shading Data</p>
<p>(The number of bits in each data of the shading correction
coefficient)</p>
<p>[16=10h]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">19</td>
<td colspan="8" style="text-align: left;"><blockquote>
<p>Bits per a Dark Current Data (The number of bits in each data of the
dark voltage correction coefficient)</p>
<p>[0]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">20, 21</td>
<td colspan="8" style="text-align: left;"><blockquote>
<p>Execute operation support 80</p>
<p>(Function that is supported by operation code 8xh of Execute)</p>
<p>Byte 20 [03h]</p>
<p>Byte 21 [0]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">22, 23</td>
<td colspan="8" style="text-align: left;"><blockquote>
<p>Execute operation support 90</p>
<p>(Function that is supported by operation code 9xh of Execute)</p>
<p>Byte 22 [02h] (MA-21)</p>
<p>[0] (Adapter other than the above)</p>
<p>Byte 23 [0]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">24, 25</td>
<td colspan="8" style="text-align: left;"><blockquote>
<p>Execute operation support A0</p>
<p>(Function that is supported by operation code Axh of Execute)</p>
<p>Byte 24 [01h]</p>
<p>Byte 25 [0]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">26, 27</td>
<td colspan="8" style="text-align: left;"><blockquote>
<p>Execute operation support B0</p>
<p>(Function that is supported by operation code Bxh of Execute)</p>
<p>Byte 26 [19h] (SA-21/SA-30/IA-20)</p>
<p>[09h] (Adapter and holder other than the above)</p>
<p>Byte 27 [0]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">28, 29</td>
<td colspan="8" style="text-align: left;"><blockquote>
<p>Execute operation support C0</p>
<p>(Function that is supported by operation code Cxh of Execute)</p>
<p>Byte 28 [03h]</p>
<p>Byte 29 [0]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">30, 31</td>
<td colspan="8" style="text-align: left;"><blockquote>
<p>Execute operation support D0</p>
<p>(Function that is supported by operation code Dxh of Execute)</p>
<p>Byte 30 [45h] (SA-21/SA-30)</p>
<p>[07h] (IA-20)</p>
<p>[23h] (SF-210)</p>
<p>[0] (Adapter and holder other than the above)</p>
<p>Byte 31 [0]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">32, 33</td>
<td colspan="8" style="text-align: left;"><blockquote>
<p>Execute operation support E0</p>
<p>(Function that is supported by operation code Exh of Execute)</p>
<p>Byte 32 [0]</p>
<p>Byte 33 [0]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">34, 35</td>
<td colspan="8" style="text-align: left;"><blockquote>
<p>Execute operation support F0</p>
<p>(Function that is supported by operation code Fxh of Execute)</p>
<p>Byte 34 [0]</p>
<p>Byte 35 [0]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">36</td>
<td colspan="8" style="text-align: left;"><blockquote>
<p>Additional Information (other additional information)</p>
<p>[0Ch]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">37</td>
<td colspan="8" style="text-align: left;"><blockquote>
<p>Volatile buffer for Initiator use (RAM buffer area)</p>
<p>[4]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">38</td>
<td colspan="8" style="text-align: left;"><blockquote>
<p>Non Volatile buffer for Initiator use (non-volatile memory buffer
area)</p>
<p>[0]</p>
</blockquote></td>
</tr>
</tbody>
</table>

\*1 When an invalid logical unit selection is performed

Byte 4 and 5 Host cooperation function

> This field specifies the processing that is executed in cooperation
> with the initiator.
>
> The initiator performs the processing of the bit that is set to 1.

<table>
<colgroup>
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 47%" />
<col style="width: 15%" />
<col style="width: 16%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: center;">Byte</td>
<td style="text-align: center;">Bit</td>
<td style="text-align: center;"></td>
<td style="text-align: center;"><p>SA-21/SA-30</p>
<p>/IA-20</p></td>
<td style="text-align: center;">Other adapters</td>
</tr>
<tr>
<td style="text-align: center;">4</td>
<td style="text-align: center;">0</td>
<td style="text-align: left;"><blockquote>
<p>Thumbnail created by driver</p>
</blockquote></td>
<td style="text-align: center;">1</td>
<td style="text-align: center;">0</td>
</tr>
<tr>
<td style="text-align: center;"></td>
<td style="text-align: center;">1</td>
<td style="text-align: left;"><blockquote>
<p>Averaging multiple reading by driver</p>
</blockquote></td>
<td style="text-align: center;">1</td>
<td style="text-align: center;">1</td>
</tr>
<tr>
<td style="text-align: center;"></td>
<td style="text-align: center;">2</td>
<td style="text-align: left;"><blockquote>
<p>Registration gap resolved by driver</p>
</blockquote></td>
<td style="text-align: center;">0</td>
<td style="text-align: center;">0</td>
</tr>
<tr>
<td style="text-align: center;"></td>
<td style="text-align: center;">3</td>
<td style="text-align: left;"><blockquote>
<p>Dark voltage data created by driver</p>
</blockquote></td>
<td style="text-align: center;">0</td>
<td style="text-align: center;">0</td>
</tr>
<tr>
<td style="text-align: center;"></td>
<td style="text-align: center;">4</td>
<td style="text-align: left;"><blockquote>
<p>Shading calibration data created by driver</p>
</blockquote></td>
<td style="text-align: center;">0</td>
<td style="text-align: center;">0</td>
</tr>
<tr>
<td style="text-align: center;"></td>
<td style="text-align: center;">5</td>
<td style="text-align: left;"><blockquote>
<p>Auto Focus by driver</p>
</blockquote></td>
<td style="text-align: center;">0</td>
<td style="text-align: center;">0</td>
</tr>
<tr>
<td style="text-align: center;"></td>
<td style="text-align: center;">6</td>
<td style="text-align: left;"><blockquote>
<p>Shading correction by driver</p>
</blockquote></td>
<td style="text-align: center;">0</td>
<td style="text-align: center;">0</td>
</tr>
<tr>
<td style="text-align: center;"></td>
<td style="text-align: center;">7</td>
<td style="text-align: left;"><blockquote>
<p>Extend bit</p>
</blockquote></td>
<td style="text-align: center;">1</td>
<td style="text-align: center;">1</td>
</tr>
<tr>
<td style="text-align: center;">5</td>
<td style="text-align: center;">0</td>
<td style="text-align: left;"><blockquote>
<p>3 line simultaneous reading process by driver</p>
</blockquote></td>
<td style="text-align: center;">0</td>
<td style="text-align: center;">0</td>
</tr>
<tr>
<td style="text-align: center;"></td>
<td style="text-align: center;">1</td>
<td style="text-align: left;"><blockquote>
<p>Pitch in the main-scanning direction by driver</p>
</blockquote></td>
<td style="text-align: center;">0</td>
<td style="text-align: center;">0</td>
</tr>
<tr>
<td style="text-align: center;"></td>
<td style="text-align: center;">2</td>
<td style="text-align: left;"><blockquote>
<p>Truncated by driver</p>
</blockquote></td>
<td style="text-align: center;">1</td>
<td style="text-align: center;">1</td>
</tr>
<tr>
<td style="text-align: center;"></td>
<td style="text-align: center;">3</td>
<td style="text-align: left;"><blockquote>
<p>CCD Data Created by Driver</p>
</blockquote></td>
<td style="text-align: center;">1</td>
<td style="text-align: center;">1</td>
</tr>
<tr>
<td style="text-align: center;"></td>
<td style="text-align: center;">4 to 6</td>
<td style="text-align: left;"><blockquote>
<p>Reserved</p>
</blockquote></td>
<td style="text-align: center;">0</td>
<td style="text-align: center;">0</td>
</tr>
<tr>
<td style="text-align: center;"></td>
<td style="text-align: center;">7</td>
<td style="text-align: left;"><blockquote>
<p>Extend bit</p>
</blockquote></td>
<td style="text-align: center;">0</td>
<td style="text-align: center;">0</td>
</tr>
</tbody>
</table>

Byte 6 to 10 Send/Read supported information

> This field specifies the data transfer that is supported by the Send
> and the Read commands.
>
> The data transfer of the bit that is set to 1 is supported.
>
> However, setting byte 7 bit5 ‘Shading Data writing supported’ to \[0\]
> when the shading correction that is being performed by the Set
> Parameter command becomes an error indicates that the recovery
> operation such as transferring the previous shading data from the host
> to the unit is not necessary and the previous shading data can be
> recovered in the unit.

<table style="width:100%;">
<colgroup>
<col style="width: 9%" />
<col style="width: 9%" />
<col style="width: 45%" />
<col style="width: 11%" />
<col style="width: 11%" />
<col style="width: 11%" />
</colgroup>
<tbody>
<tr>
<td></td>
<td></td>
<td></td>
<td style="text-align: center;"><p>[SA-21/</p>
<p>SA-30]</p></td>
<td style="text-align: center;"><p>[IA-20/</p>
<p>SF-210]</p></td>
<td style="text-align: center;">[Other]</td>
</tr>
<tr>
<td>Byte 6</td>
<td>Bit0</td>
<td>Halftone mask reading supported</td>
<td style="text-align: center;">[0]</td>
<td style="text-align: center;">[0]</td>
<td style="text-align: center;">[0]</td>
</tr>
<tr>
<td></td>
<td>Bit1</td>
<td>Halftone mask writing supported</td>
<td style="text-align: center;">[0]</td>
<td style="text-align: center;">[0]</td>
<td style="text-align: center;">[0]</td>
</tr>
<tr>
<td></td>
<td>Bit2</td>
<td>Gamma function reading supported</td>
<td style="text-align: center;">[0]</td>
<td style="text-align: center;">[0]</td>
<td style="text-align: center;">[0]</td>
</tr>
<tr>
<td></td>
<td>Bit3</td>
<td>Gamma function writing supported</td>
<td style="text-align: center;">[0]</td>
<td style="text-align: center;">[0]</td>
<td style="text-align: center;">[0]</td>
</tr>
<tr>
<td></td>
<td>Bit4</td>
<td>Histogram Data reading supported</td>
<td style="text-align: center;">[0]</td>
<td style="text-align: center;">[0]</td>
<td style="text-align: center;">[0]</td>
</tr>
<tr>
<td></td>
<td>Bit5</td>
<td>Max Value Data reading supported</td>
<td style="text-align: center;">[0]</td>
<td style="text-align: center;">[0]</td>
<td style="text-align: center;">[0]</td>
</tr>
<tr>
<td></td>
<td>Bit6</td>
<td>Reserved</td>
<td style="text-align: center;">[0]</td>
<td style="text-align: center;">[0]</td>
<td style="text-align: center;">[0]</td>
</tr>
<tr>
<td></td>
<td>Bit7</td>
<td>Extend bit</td>
<td style="text-align: center;">[1]</td>
<td style="text-align: center;">[1]</td>
<td style="text-align: center;">[1]</td>
</tr>
</tbody>
</table>

<table>
<colgroup>
<col style="width: 10%" />
<col style="width: 9%" />
<col style="width: 44%" />
<col style="width: 11%" />
<col style="width: 11%" />
<col style="width: 11%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: left;"></td>
<td style="text-align: left;"></td>
<td style="text-align: left;"></td>
<td style="text-align: center;"><p>[SA-21/</p>
<p>SA-30]</p></td>
<td style="text-align: center;"><p>[IA-20/</p>
<p>SF-210]</p></td>
<td style="text-align: center;">[Other]</td>
</tr>
<tr>
<td style="text-align: left;">Byte 7</td>
<td style="text-align: left;">Bit0</td>
<td style="text-align: left;">Matrix Data reading supported</td>
<td style="text-align: center;">[0]</td>
<td style="text-align: center;">[0]</td>
<td style="text-align: center;">[0]</td>
</tr>
<tr>
<td style="text-align: left;"></td>
<td style="text-align: left;">Bit1</td>
<td style="text-align: left;">Matrix Data writing supported</td>
<td style="text-align: center;">[0]</td>
<td style="text-align: center;">[0]</td>
<td style="text-align: center;">[0]</td>
</tr>
<tr>
<td style="text-align: left;"></td>
<td style="text-align: left;">Bit2</td>
<td style="text-align: left;">Filter Data reading supported</td>
<td style="text-align: center;">[0]</td>
<td style="text-align: center;">[0]</td>
<td style="text-align: center;">[0]</td>
</tr>
<tr>
<td style="text-align: left;"></td>
<td style="text-align: left;">Bit3</td>
<td style="text-align: left;">Filter Data writing supported</td>
<td style="text-align: center;">[0]</td>
<td style="text-align: center;">[0]</td>
<td style="text-align: center;">[0]</td>
</tr>
<tr>
<td style="text-align: left;"></td>
<td style="text-align: left;">Bit4</td>
<td style="text-align: left;">Shading Data reading supported</td>
<td style="text-align: center;">[1]</td>
<td style="text-align: center;">[1]</td>
<td style="text-align: center;">[1]</td>
</tr>
<tr>
<td style="text-align: left;"></td>
<td style="text-align: left;">Bit5</td>
<td style="text-align: left;">Shading Data writing supported</td>
<td style="text-align: center;">[1]</td>
<td style="text-align: center;">[1]</td>
<td style="text-align: center;">[1]</td>
</tr>
<tr>
<td style="text-align: left;"></td>
<td style="text-align: left;">Bit6</td>
<td style="text-align: left;">Reserved</td>
<td style="text-align: center;">[0]</td>
<td style="text-align: center;">[0]</td>
<td style="text-align: center;">[0]</td>
</tr>
<tr>
<td style="text-align: left;"></td>
<td style="text-align: left;">Bit7</td>
<td style="text-align: left;">Extend bit</td>
<td style="text-align: center;">[1]</td>
<td style="text-align: center;">[1]</td>
<td style="text-align: center;">[1]</td>
</tr>
</tbody>
</table>

<table>
<colgroup>
<col style="width: 9%" />
<col style="width: 9%" />
<col style="width: 44%" />
<col style="width: 11%" />
<col style="width: 11%" />
<col style="width: 11%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: left;"></td>
<td style="text-align: left;"></td>
<td style="text-align: left;"></td>
<td style="text-align: center;"><p>[SA-21/</p>
<p>SA-30]</p></td>
<td style="text-align: center;"><p>[IA-20/</p>
<p>SF-210]</p></td>
<td style="text-align: center;">[Other]</td>
</tr>
<tr>
<td style="text-align: left;">Byte 8</td>
<td style="text-align: left;">Bit0</td>
<td style="text-align: left;">Dark Voltage Data reading supported</td>
<td style="text-align: center;">[0]</td>
<td style="text-align: center;">[0]</td>
<td style="text-align: center;">[0]</td>
</tr>
<tr>
<td style="text-align: left;"></td>
<td style="text-align: left;">Bit1</td>
<td style="text-align: left;">Dark Voltage Data writing supported</td>
<td style="text-align: center;">[0]</td>
<td style="text-align: center;">[0]</td>
<td style="text-align: center;">[0]</td>
</tr>
<tr>
<td style="text-align: left;"></td>
<td style="text-align: left;">Bit2</td>
<td style="text-align: left;">Magnetic Data reading supported</td>
<td style="text-align: center;">[0]</td>
<td style="text-align: center;">[0]</td>
<td style="text-align: center;">[0]</td>
</tr>
<tr>
<td style="text-align: left;"></td>
<td style="text-align: left;">Bit3</td>
<td style="text-align: left;">Magnetic Data writing supported</td>
<td style="text-align: center;">[0]</td>
<td style="text-align: center;">[0]</td>
<td style="text-align: center;">[0]</td>
</tr>
<tr>
<td style="text-align: left;"></td>
<td style="text-align: left;">Bit4</td>
<td style="text-align: left;">Cooperation parameters reading
supported</td>
<td style="text-align: center;">[1]</td>
<td style="text-align: center;">[1]</td>
<td style="text-align: center;">[1]</td>
</tr>
<tr>
<td style="text-align: left;"></td>
<td style="text-align: left;">Bit5</td>
<td style="text-align: left;">Boundary data reading supported</td>
<td style="text-align: center;">[0]</td>
<td style="text-align: center;">[0]</td>
<td style="text-align: center;">[0]</td>
</tr>
<tr>
<td style="text-align: left;"></td>
<td style="text-align: left;">Bit6</td>
<td style="text-align: left;">Boundary data writing supported</td>
<td style="text-align: center;">[0]</td>
<td style="text-align: center;">[0]</td>
<td style="text-align: center;">[0]</td>
</tr>
<tr>
<td style="text-align: left;"></td>
<td style="text-align: left;">Bit7</td>
<td style="text-align: left;">Extend bit</td>
<td style="text-align: center;">[1]</td>
<td style="text-align: center;">[1]</td>
<td style="text-align: center;">[1]</td>
</tr>
</tbody>
</table>

<table>
<colgroup>
<col style="width: 9%" />
<col style="width: 9%" />
<col style="width: 44%" />
<col style="width: 11%" />
<col style="width: 11%" />
<col style="width: 11%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: left;"></td>
<td style="text-align: left;"></td>
<td style="text-align: left;"></td>
<td style="text-align: center;"><p>[SA-21/</p>
<p>SA-30]</p></td>
<td style="text-align: center;"><p>[IA-20/</p>
<p>SF-210]</p></td>
<td style="text-align: center;">[Other]</td>
</tr>
<tr>
<td style="text-align: left;">Byte 9</td>
<td style="text-align: left;">Bit0</td>
<td style="text-align: left;">Analog Gamma reading supported</td>
<td style="text-align: center;">[0]</td>
<td style="text-align: center;">[0]</td>
<td style="text-align: center;">[0]</td>
</tr>
<tr>
<td style="text-align: left;"></td>
<td style="text-align: left;">Bit1</td>
<td style="text-align: left;">Analog Gain reading supported</td>
<td style="text-align: center;">[1]</td>
<td style="text-align: center;">[1]</td>
<td style="text-align: center;">[1]</td>
</tr>
<tr>
<td style="text-align: left;"></td>
<td style="text-align: left;">Bit2</td>
<td style="text-align: left;">Digital Gain reading supported</td>
<td style="text-align: center;">[0]</td>
<td style="text-align: center;">[0]</td>
<td style="text-align: center;">[0]</td>
</tr>
<tr>
<td style="text-align: left;"></td>
<td style="text-align: left;">Bit3</td>
<td style="text-align: left;">Exposure Value reading supported</td>
<td style="text-align: center;">[1]</td>
<td style="text-align: center;">[1]</td>
<td style="text-align: center;">[1]</td>
</tr>
<tr>
<td style="text-align: left;"></td>
<td style="text-align: left;">Bit4</td>
<td style="text-align: left;">Setup Information reading supported</td>
<td style="text-align: center;">[1]</td>
<td style="text-align: center;">[1]</td>
<td style="text-align: center;">[1]</td>
</tr>
<tr>
<td style="text-align: left;"></td>
<td style="text-align: left;">Bit5</td>
<td style="text-align: left;">Setup Information writing supported</td>
<td style="text-align: center;">[0]</td>
<td style="text-align: center;">[0]</td>
<td style="text-align: center;">[0]</td>
</tr>
<tr>
<td style="text-align: left;"></td>
<td style="text-align: left;">Bit6</td>
<td style="text-align: left;">Perforation Information reading
supported</td>
<td style="text-align: center;">[1]</td>
<td style="text-align: center;">[0]</td>
<td style="text-align: center;">[0]</td>
</tr>
<tr>
<td style="text-align: left;"></td>
<td style="text-align: left;">Bit7</td>
<td style="text-align: left;">Extend bit</td>
<td style="text-align: center;">[1]</td>
<td style="text-align: center;">[1]</td>
<td style="text-align: center;">[1]</td>
</tr>
</tbody>
</table>

<table>
<colgroup>
<col style="width: 9%" />
<col style="width: 9%" />
<col style="width: 44%" />
<col style="width: 11%" />
<col style="width: 11%" />
<col style="width: 11%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: left;"></td>
<td style="text-align: left;"></td>
<td style="text-align: left;"></td>
<td style="text-align: center;"><p>[SA-21/</p>
<p>SA-30]</p></td>
<td style="text-align: center;"><p>[IA-20/</p>
<p>SF-210]</p></td>
<td style="text-align: center;">[Other]</td>
</tr>
<tr>
<td style="text-align: left;">Byte 10</td>
<td style="text-align: left;">Bit0</td>
<td style="text-align: left;">Boundary Type2 data reading supported</td>
<td style="text-align: center;">[1]</td>
<td style="text-align: center;">[0]</td>
<td style="text-align: center;">[0]</td>
</tr>
<tr>
<td style="text-align: left;"></td>
<td style="text-align: left;">Bit1</td>
<td style="text-align: left;">Boundary Type2 data writing supported</td>
<td style="text-align: center;">[1]</td>
<td style="text-align: center;">[0]</td>
<td style="text-align: center;">[0]</td>
</tr>
<tr>
<td style="text-align: left;"></td>
<td style="text-align: left;">Bit2</td>
<td style="text-align: left;">Initial WB Exposure Value reading
supported</td>
<td style="text-align: center;">[0]</td>
<td style="text-align: center;">[0]</td>
<td style="text-align: center;">[1]</td>
</tr>
<tr>
<td style="text-align: left;"></td>
<td style="text-align: left;">Bit3</td>
<td style="text-align: left;">CCD data reading supported</td>
<td style="text-align: center;">[1]</td>
<td style="text-align: center;">[1]</td>
<td style="text-align: center;">[1]</td>
</tr>
<tr>
<td style="text-align: left;"></td>
<td style="text-align: left;">Bit4</td>
<td style="text-align: left;">Driver Soft Version reading supported</td>
<td style="text-align: center;">[1]</td>
<td style="text-align: center;">[1]</td>
<td style="text-align: center;">[1]</td>
</tr>
<tr>
<td style="text-align: left;"></td>
<td style="text-align: left;">Bit5</td>
<td style="text-align: left;">Driver Soft Version writing supported</td>
<td style="text-align: center;">[1]</td>
<td style="text-align: center;">[1]</td>
<td style="text-align: center;">[1]</td>
</tr>
<tr>
<td style="text-align: left;"></td>
<td style="text-align: left;">Bit6</td>
<td style="text-align: left;">Leak data reading supported</td>
<td style="text-align: center;">[1]</td>
<td style="text-align: center;">[1]</td>
<td style="text-align: center;">[1]</td>
</tr>
<tr>
<td style="text-align: left;"></td>
<td style="text-align: left;">Bit7</td>
<td style="text-align: left;">Extend bit</td>
<td style="text-align: center;">[0]</td>
<td style="text-align: center;">[0]</td>
<td style="text-align: center;">[0]</td>
</tr>
</tbody>
</table>

Byte 11 Bits per a halftone mask parameter

> This field specifies the length in bits of the halftone mask. This
> unit sets this field to 0.

Byte 12 and 13 X/Y bit depth of Download LUT

> This field specifies the length in bits of the input/output data in
> the LUT that is transferred from the initiator.
>
> This unit does not support this field.

Byte 14 Bits per a Histogram Data

> This field specifies the length in bits of each histogram data. This
> unit sets this field to 0.

Byte 15 Bits per a Max Value Data

> This field specifies the length in bits of the AE maximum value. This
> unit does not support this field.

Byte 16 Bits per a Matrix Data

> This field specifies the length in bits of each matrix data. This unit
> sets this field to 0.

Byte 17 Bits per a Filter Data

> This field specifies the length in bits of each filter data. This unit
> sets this field to 0.

Byte 18 Bits per a Shading

> This field specifies the length in bits of each data for the shading
> correction coefficient.
>
> This unit sets this field to 10h.

Byte 19 Dark Current Data

> This field specifies the length in bits of each data for the dark
> voltage correction coefficient.
>
> This unit does not support this field.

Byte 20 and 21 Execute operation support 80

> This field specifies the function that is supported by operation code
> 8xh of EXECUTE command.
>
> This unit supports ‘Initialize’ and ‘Return to origin’.
>
> ‘Initialize’ performs the unit initialization in the same manner as
> that is performed at the start of power supply.
>
> ‘Return to origin’ moves the object to be scanned or the stage
> (mechanical block) to the origin position.

<table>
<colgroup>
<col style="width: 12%" />
<col style="width: 12%" />
<col style="width: 58%" />
<col style="width: 15%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: center;">Byte</td>
<td style="text-align: center;">Bit</td>
<td style="text-align: center;">Operation</td>
<td style="text-align: center;">Value on this unit</td>
</tr>
<tr>
<td style="text-align: center;">20</td>
<td style="text-align: center;">0</td>
<td style="text-align: left;"><blockquote>
<p>Initialize</p>
</blockquote></td>
<td style="text-align: center;">[1]</td>
</tr>
<tr>
<td style="text-align: center;"></td>
<td style="text-align: center;">1</td>
<td style="text-align: left;"><blockquote>
<p>Return to origin</p>
</blockquote></td>
<td style="text-align: center;">[1]</td>
</tr>
<tr>
<td style="text-align: center;"></td>
<td style="text-align: center;">2 to 7</td>
<td style="text-align: left;"><blockquote>
<p>Reserved</p>
</blockquote></td>
<td style="text-align: center;">[0]</td>
</tr>
<tr>
<td style="text-align: center;">21</td>
<td style="text-align: center;">0 to 7</td>
<td style="text-align: left;"><blockquote>
<p>Reserved</p>
</blockquote></td>
<td style="text-align: center;">[0]</td>
</tr>
</tbody>
</table>

Byte 22 and 23 Execute operation support 90

> This field specifies the function that is supported by operation code
> 9xh of EXECUTE command.
>
> This unit supports the automatic execution of auto focus.

<table style="width:99%;">
<colgroup>
<col style="width: 0%" />
<col style="width: 11%" />
<col style="width: 0%" />
<col style="width: 11%" />
<col style="width: 0%" />
<col style="width: 47%" />
<col style="width: 0%" />
<col style="width: 12%" />
<col style="width: 13%" />
</colgroup>
<tbody>
<tr>
<td colspan="2" style="text-align: center;">Byte</td>
<td colspan="2" style="text-align: center;">Bit</td>
<td colspan="2" style="text-align: center;">Operation</td>
<td colspan="3" style="text-align: center;">Value on this unit</td>
</tr>
<tr>
<td colspan="2" style="text-align: center;"></td>
<td colspan="2" style="text-align: center;"></td>
<td colspan="2" style="text-align: center;"></td>
<td colspan="2" style="text-align: center;">MA-21</td>
<td style="text-align: center;">Other than MA-21</td>
</tr>
<tr>
<td colspan="2" style="text-align: center;">22</td>
<td colspan="2" style="text-align: center;">0</td>
<td colspan="2" style="text-align: left;"><blockquote>
<p>Change Unit</p>
</blockquote></td>
<td colspan="2" style="text-align: center;">[0]</td>
<td style="text-align: center;">[0]</td>
</tr>
<tr>
<td colspan="2" style="text-align: center;"></td>
<td colspan="2" style="text-align: center;">1</td>
<td colspan="2" style="text-align: left;"><blockquote>
<p>AF Autoexec</p>
</blockquote></td>
<td colspan="2" style="text-align: center;">[1]</td>
<td style="text-align: center;">[0]</td>
</tr>
<tr>
<td colspan="2" style="text-align: center;"></td>
<td colspan="2" style="text-align: center;">2 to 7</td>
<td colspan="2" style="text-align: left;"><blockquote>
<p>Reserved</p>
</blockquote></td>
<td colspan="2" style="text-align: center;">[0]</td>
<td style="text-align: center;">[0]</td>
</tr>
<tr>
<td colspan="2" style="text-align: center;">23</td>
<td colspan="2" style="text-align: center;">0 to 7</td>
<td colspan="2" style="text-align: left;"><blockquote>
<p>Reserved</p>
</blockquote></td>
<td colspan="2" style="text-align: center;">[0]</td>
<td style="text-align: center;">[0]</td>
</tr>
</tbody>
</table>

Byte 24 and 25 Execute operation support A0

> This field specifies the function that is supported by operation code
> Axh of EXECUTE command.
>
> This unit supports the auto focus.

<table>
<colgroup>
<col style="width: 13%" />
<col style="width: 12%" />
<col style="width: 56%" />
<col style="width: 18%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: center;">Byte</td>
<td style="text-align: center;">Bit</td>
<td style="text-align: center;">Operation</td>
<td style="text-align: center;">Value on this unit</td>
</tr>
<tr>
<td style="text-align: center;">24</td>
<td style="text-align: center;">0</td>
<td style="text-align: left;"><blockquote>
<p>Auto Focus</p>
</blockquote></td>
<td style="text-align: center;">[1]</td>
</tr>
<tr>
<td style="text-align: center;"></td>
<td style="text-align: center;">1</td>
<td style="text-align: left;"><blockquote>
<p>Color oriented Auto Focus</p>
</blockquote></td>
<td style="text-align: center;">[0]</td>
</tr>
<tr>
<td style="text-align: center;"></td>
<td style="text-align: center;">2 to 7</td>
<td style="text-align: left;"><blockquote>
<p>Reserved</p>
</blockquote></td>
<td style="text-align: center;">[0]</td>
</tr>
<tr>
<td style="text-align: center;">25</td>
<td style="text-align: center;">0 to 7</td>
<td style="text-align: left;"><blockquote>
<p>Reserved</p>
</blockquote></td>
<td style="text-align: center;">[0]</td>
</tr>
</tbody>
</table>

Byte 26 and 27 Execute operation support B0

> This field specifies the function that is supported by operation code
> Bxh of EXECUTE command.
>
> This unit supports the shading measurement, dark voltage measurement,
> recording of the unit-specific data setting, and changing the
> automatic ejection time of the film.

<table style="width:98%;">
<colgroup>
<col style="width: 0%" />
<col style="width: 0%" />
<col style="width: 1%" />
<col style="width: 6%" />
<col style="width: 0%" />
<col style="width: 0%" />
<col style="width: 1%" />
<col style="width: 6%" />
<col style="width: 0%" />
<col style="width: 0%" />
<col style="width: 1%" />
<col style="width: 39%" />
<col style="width: 0%" />
<col style="width: 0%" />
<col style="width: 1%" />
<col style="width: 15%" />
<col style="width: 0%" />
<col style="width: 1%" />
<col style="width: 18%" />
</colgroup>
<tbody>
<tr>
<td colspan="4" style="text-align: center;">Byte</td>
<td colspan="4" style="text-align: center;">Bit</td>
<td colspan="4" style="text-align: center;">Operation</td>
<td colspan="7" style="text-align: center;">Value on this unit</td>
</tr>
<tr>
<td colspan="4" style="text-align: center;"></td>
<td colspan="4" style="text-align: center;"></td>
<td colspan="4" style="text-align: center;"></td>
<td colspan="4" style="text-align: center;"><p>SA-21/SA-30</p>
<p>/IA-20</p></td>
<td colspan="3" style="text-align: center;"><p>Other than</p>
<p>SA-21/SA-30/IA-20</p></td>
</tr>
<tr>
<td colspan="4" style="text-align: center;">26</td>
<td colspan="4" style="text-align: center;">0</td>
<td colspan="4" style="text-align: center;"><blockquote>
<p>Setup Shading Data</p>
</blockquote></td>
<td colspan="4" style="text-align: center;">[1]</td>
<td colspan="3" style="text-align: center;">[1]</td>
</tr>
<tr>
<td colspan="4" style="text-align: center;"></td>
<td colspan="4" style="text-align: center;">1</td>
<td colspan="4" style="text-align: center;"><blockquote>
<p>Setup Dark Current Correction Data</p>
</blockquote></td>
<td colspan="4" style="text-align: center;">[0]</td>
<td colspan="3" style="text-align: center;">[0]</td>
</tr>
<tr>
<td colspan="4" style="text-align: center;"></td>
<td colspan="4" style="text-align: center;">2</td>
<td colspan="4" style="text-align: center;"><blockquote>
<p>Setup Offset Correction Data</p>
</blockquote></td>
<td colspan="4" style="text-align: center;">[0]</td>
<td colspan="3" style="text-align: center;">[0]</td>
</tr>
<tr>
<td colspan="4" style="text-align: center;"></td>
<td colspan="4" style="text-align: center;">3</td>
<td colspan="4" style="text-align: center;"><blockquote>
<p>Write Data On Device Dependence</p>
</blockquote></td>
<td colspan="4" style="text-align: center;">[1]</td>
<td colspan="3" style="text-align: center;">[1]</td>
</tr>
<tr>
<td colspan="4" style="text-align: center;"></td>
<td colspan="4" style="text-align: center;">4</td>
<td colspan="4" style="text-align: center;"><blockquote>
<p>Change of Auto Unload time</p>
</blockquote></td>
<td colspan="4" style="text-align: center;">[1]</td>
<td colspan="3" style="text-align: center;">[0]</td>
</tr>
<tr>
<td colspan="4" style="text-align: center;"></td>
<td colspan="4" style="text-align: center;">5 to 7</td>
<td colspan="4" style="text-align: center;"><blockquote>
<p>Reserved</p>
</blockquote></td>
<td colspan="4" style="text-align: center;">[0]</td>
<td colspan="3" style="text-align: center;">[0]</td>
</tr>
<tr>
<td colspan="4" style="text-align: center;">27</td>
<td colspan="4" style="text-align: center;">0 to 7</td>
<td colspan="4" style="text-align: left;"><blockquote>
<p>Reserved</p>
</blockquote></td>
<td colspan="4" style="text-align: center;">[0]</td>
<td colspan="3" style="text-align: center;">[0]</td>
</tr>
</tbody>
</table>

Byte 28 and 29 Execute operation support C0

> This field specifies the function that is supported by operation code
> Cxh of EXECUTE command.
>
> This unit supports the stage movement and the focus movement.

<table>
<colgroup>
<col style="width: 13%" />
<col style="width: 12%" />
<col style="width: 58%" />
<col style="width: 15%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: center;">Byte</td>
<td style="text-align: center;">Bit</td>
<td style="text-align: center;">Operation</td>
<td style="text-align: center;">Value on this unit</td>
</tr>
<tr>
<td style="text-align: center;">28</td>
<td style="text-align: center;">0</td>
<td style="text-align: left;"><blockquote>
<p>Stage Move</p>
</blockquote></td>
<td style="text-align: center;">[1]</td>
</tr>
<tr>
<td style="text-align: center;"></td>
<td style="text-align: center;">1</td>
<td style="text-align: left;"><blockquote>
<p>Focus Move</p>
</blockquote></td>
<td style="text-align: center;">[1]</td>
</tr>
<tr>
<td style="text-align: center;"></td>
<td style="text-align: center;">2 to 7</td>
<td style="text-align: left;"><blockquote>
<p>Reserved</p>
</blockquote></td>
<td style="text-align: center;">[0]</td>
</tr>
<tr>
<td style="text-align: center;">29</td>
<td style="text-align: center;">0 to 7</td>
<td style="text-align: left;"><blockquote>
<p>Reserved</p>
</blockquote></td>
<td style="text-align: center;">[0]</td>
</tr>
</tbody>
</table>

Byte 30 and 31 Execute operation support D0

> This field specifies the function that is supported by operation code
> Dxh of EXECUTE command.
>
> This unit supports the loading/unloading of the object to be scanned.
> For the movement, the absolute address specification is supported.

<table style="width:94%;">
<colgroup>
<col style="width: 6%" />
<col style="width: 2%" />
<col style="width: 6%" />
<col style="width: 2%" />
<col style="width: 6%" />
<col style="width: 25%" />
<col style="width: 6%" />
<col style="width: 2%" />
<col style="width: 6%" />
<col style="width: 2%" />
<col style="width: 6%" />
<col style="width: 2%" />
<col style="width: 6%" />
<col style="width: 2%" />
<col style="width: 6%" />
<col style="width: 2%" />
</colgroup>
<tbody>
<tr>
<td colspan="2" style="text-align: center;">Byte</td>
<td colspan="2" style="text-align: center;">Bit</td>
<td colspan="2" style="text-align: center;">Operation</td>
<td colspan="10" style="text-align: center;">Value on this unit</td>
</tr>
<tr>
<td colspan="2" style="text-align: center;"></td>
<td colspan="2" style="text-align: center;"></td>
<td colspan="2" style="text-align: center;"></td>
<td colspan="2" style="text-align: center;"><p>MA-21</p>
<p>(*1)</p></td>
<td colspan="2" style="text-align: center;">SA-21</td>
<td colspan="2" style="text-align: center;">SA-30</td>
<td colspan="2" style="text-align: center;">IA-20</td>
<td colspan="2" style="text-align: center;">SF-210</td>
</tr>
<tr>
<td colspan="2" style="text-align: center;">30</td>
<td colspan="2" style="text-align: center;">0</td>
<td colspan="2" style="text-align: left;"><blockquote>
<p>Unload object</p>
</blockquote></td>
<td colspan="2" style="text-align: center;">[0]</td>
<td colspan="2" style="text-align: center;">[1]</td>
<td colspan="2" style="text-align: center;">[1]</td>
<td colspan="2" style="text-align: center;">[1]</td>
<td colspan="2" style="text-align: center;">[1]</td>
</tr>
<tr>
<td colspan="2" style="text-align: center;"></td>
<td colspan="2" style="text-align: center;">1</td>
<td colspan="2" style="text-align: left;"><blockquote>
<p>Load object</p>
</blockquote></td>
<td colspan="2" style="text-align: center;">[0]</td>
<td colspan="2" style="text-align: center;">[0]</td>
<td colspan="2" style="text-align: center;">[0]</td>
<td colspan="2" style="text-align: center;">[1]</td>
<td colspan="2" style="text-align: center;">[1]</td>
</tr>
<tr>
<td colspan="2" style="text-align: center;"></td>
<td colspan="2" style="text-align: center;">2</td>
<td colspan="2" style="text-align: left;"><blockquote>
<p>Absolute positioning</p>
</blockquote></td>
<td colspan="2" style="text-align: center;">[0]</td>
<td colspan="2" style="text-align: center;">[1]</td>
<td colspan="2" style="text-align: center;">[1]</td>
<td colspan="2" style="text-align: center;">[1]</td>
<td colspan="2" style="text-align: center;">[0]</td>
</tr>
<tr>
<td colspan="2" style="text-align: center;"></td>
<td colspan="2" style="text-align: center;">3</td>
<td colspan="2" style="text-align: left;"><blockquote>
<p>Relative positioning</p>
</blockquote></td>
<td colspan="2" style="text-align: center;">[0]</td>
<td colspan="2" style="text-align: center;">[0]</td>
<td colspan="2" style="text-align: center;">[0]</td>
<td colspan="2" style="text-align: center;">[0]</td>
<td colspan="2" style="text-align: center;">[0]</td>
</tr>
<tr>
<td colspan="2" style="text-align: center;"></td>
<td colspan="2" style="text-align: center;">4</td>
<td colspan="2" style="text-align: left;"><blockquote>
<p>Rotate</p>
</blockquote></td>
<td colspan="2" style="text-align: center;">[0]</td>
<td colspan="2" style="text-align: center;">[0]</td>
<td colspan="2" style="text-align: center;">[0]</td>
<td colspan="2" style="text-align: center;">[0]</td>
<td colspan="2" style="text-align: center;">[0]</td>
</tr>
<tr>
<td colspan="2" style="text-align: center;"></td>
<td colspan="2" style="text-align: center;">5</td>
<td colspan="2" style="text-align: left;"><blockquote>
<p>FD Move</p>
</blockquote></td>
<td colspan="2" style="text-align: center;">[0]</td>
<td colspan="2" style="text-align: center;">[0]</td>
<td colspan="2" style="text-align: center;">[0]</td>
<td colspan="2" style="text-align: center;">[0]</td>
<td colspan="2" style="text-align: center;">[1]</td>
</tr>
<tr>
<td colspan="2" style="text-align: center;"></td>
<td colspan="2" style="text-align: center;">6</td>
<td colspan="2" style="text-align: left;"><blockquote>
<p>SA Lock</p>
</blockquote></td>
<td colspan="2" style="text-align: center;">[0]</td>
<td colspan="2" style="text-align: center;">[1]</td>
<td colspan="2" style="text-align: center;">[1]</td>
<td colspan="2" style="text-align: center;">[0]</td>
<td colspan="2" style="text-align: center;">[0]</td>
</tr>
<tr>
<td colspan="2" style="text-align: center;"></td>
<td colspan="2" style="text-align: center;">7</td>
<td colspan="2" style="text-align: left;"><blockquote>
<p>Reserved</p>
</blockquote></td>
<td colspan="2" style="text-align: center;">[0]</td>
<td colspan="2" style="text-align: center;">[0]</td>
<td colspan="2" style="text-align: center;">[0]</td>
<td colspan="2" style="text-align: center;">[0]</td>
<td colspan="2" style="text-align: center;">[0]</td>
</tr>
<tr>
<td colspan="2" style="text-align: center;">31</td>
<td colspan="2" style="text-align: center;">0 to 7</td>
<td colspan="2" style="text-align: left;"><blockquote>
<p>Reserved</p>
</blockquote></td>
<td colspan="2" style="text-align: center;">[0]</td>
<td colspan="2" style="text-align: center;">[0]</td>
<td colspan="2" style="text-align: center;">[0]</td>
<td colspan="2" style="text-align: center;">[0]</td>
<td colspan="2" style="text-align: center;">[0]</td>
</tr>
</tbody>
</table>

\*1 Each holder of FH3, FHG1, and FHA1 is included.

Byte 32 and 33 Execute operation support E0

> This field specifies the function that is supported by operation code
> Exh of EXECUTE command.

Byte 34 and 35 Execute operation support F0

> This field specifies the function that is supported by operation code
> Fxh of EXECUTE command.

Byte 36 Additional Information

> This field specifies the other additional information.

<table>
<colgroup>
<col style="width: 9%" />
<col style="width: 31%" />
<col style="width: 44%" />
<col style="width: 15%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: left;">Bit</td>
<td style="text-align: left;"></td>
<td style="text-align: center;">Explanations of operation</td>
<td style="text-align: center;">Value on this unit</td>
</tr>
<tr>
<td style="text-align: left;">Bit0</td>
<td style="text-align: left;">Hot exchangeable to unequipped unit with
notice</td>
<td style="text-align: left;"><blockquote>
<p>The attached adapter can be exchanged with the power turned ON, and
it is possible to inform the initiator that the adapter has been
exchanged.</p>
</blockquote></td>
<td style="text-align: center;">[0]</td>
</tr>
<tr>
<td style="text-align: left;">Bit1</td>
<td style="text-align: left;">Scanned object exchangeable with
notice</td>
<td style="text-align: left;"><blockquote>
<p>The scanned object can be exchanged, and it is possible to inform the
initiator that the object has been exchanged.</p>
</blockquote></td>
<td style="text-align: center;">[0]</td>
</tr>
<tr>
<td style="text-align: left;">Bit2</td>
<td style="text-align: left;">Hot exchangeable to unequipped unit
without notice</td>
<td style="text-align: center;"><blockquote>
<p>The attached adapter can be exchanged with the power turned ON, but
it is not possible to inform the initiator that the adapter has been
exchanged.</p>
</blockquote></td>
<td style="text-align: center;">[1]</td>
</tr>
<tr>
<td style="text-align: left;">Bit3</td>
<td style="text-align: left;">Scanned object exchangeable without
notice</td>
<td style="text-align: left;"><blockquote>
<p>The scanned object can be exchanged, but it is not possible to inform
the initiator that the object has been exchanged.</p>
</blockquote></td>
<td style="text-align: center;">[1]</td>
</tr>
<tr>
<td style="text-align: left;">Bit4 to 6</td>
<td style="text-align: left;">Histogram Scanning</td>
<td style="text-align: left;"><blockquote>
<p>Scanning for creating the histogram of the image data</p>
</blockquote></td>
<td style="text-align: center;">[0]</td>
</tr>
<tr>
<td style="text-align: left;">Bit7</td>
<td style="text-align: left;">Extend bit</td>
<td style="text-align: center;"><blockquote>
<p>Extension bit</p>
</blockquote></td>
<td style="text-align: center;">[0]</td>
</tr>
</tbody>
</table>

Byte 37 Volatile buffer for Initiator use

> This field specifies the buffer size that can be read/written freely
> by the initiator. The unit is 256 bytes.
>
> This area is preserved on the RAM of the scanner, and the written data
> is maintained while the power of the scanner is ON.
>
> This unit sets this field to 4 (1 Kbyte).

Byte 38 Non Volatile buffer for Initiator use

> This field specifies the buffer size that can be read/written freely
> by the initiator. The unit is one byte.
>
> This area is preserved in the non-volatile memory of the scanner, and
> the written data is maintained permanently.
>
> This unit sets this field to 0, that is, this unit does not support
> the non-volatile memory buffer area.

**  **
**2-2-2-6. Operation code setting page**

<table>
<colgroup>
<col style="width: 14%" />
<col style="width: 10%" />
<col style="width: 9%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: right;"><p>Bit</p>
<p>Byte</p></td>
<td style="text-align: center;">7</td>
<td style="text-align: center;">6</td>
<td style="text-align: center;">5</td>
<td style="text-align: center;">4</td>
<td style="text-align: center;">3</td>
<td style="text-align: center;">2</td>
<td style="text-align: center;">1</td>
<td style="text-align: center;">0</td>
</tr>
<tr>
<td style="text-align: center;">0</td>
<td colspan="3" style="text-align: center;"><p>Peripheral Qualifier</p>
<p>[0]</p>
<p>[011b](*1)</p></td>
<td colspan="5" style="text-align: center;"><p>Peripheral Device
Type</p>
<p>[6=00110b]</p>
<p>[1Fh=11111b](*1)</p></td>
</tr>
<tr>
<td style="text-align: center;">1</td>
<td colspan="8" style="text-align: center;">Page code [E2h]</td>
</tr>
<tr>
<td style="text-align: center;">2</td>
<td colspan="8" style="text-align: center;">Reserved [0]</td>
</tr>
<tr>
<td style="text-align: center;">3</td>
<td colspan="8" style="text-align: center;">Page length [m-3]</td>
</tr>
<tr>
<td style="text-align: center;">4</td>
<td colspan="8" style="text-align: center;"><p>Number of Operation code
(=n)</p>
<p>(The number of operation codes for which setting of each value is
necessary)</p>
<p>[1]</p></td>
</tr>
<tr>
<td style="text-align: center;">5</td>
<td colspan="8" style="text-align: center;">Operation Code</td>
</tr>
<tr>
<td style="text-align: center;">6 to 9</td>
<td colspan="8" style="text-align: center;">Minimum value of 1st
Value</td>
</tr>
<tr>
<td style="text-align: center;">10 to 13</td>
<td colspan="8" style="text-align: center;">Maximum value of 1st
Value</td>
</tr>
<tr>
<td style="text-align: center;">14 to 17</td>
<td colspan="8" style="text-align: center;">Minimum value of 2nd
Value</td>
</tr>
<tr>
<td style="text-align: center;">18 to 21</td>
<td colspan="8" style="text-align: center;">Maximum value of 2nd
Value</td>
</tr>
<tr>
<td style="text-align: center;">22 to 25</td>
<td colspan="8" style="text-align: center;">Minimum value of Speed</td>
</tr>
<tr>
<td style="text-align: center;">26 to 29</td>
<td colspan="8" style="text-align: center;">Maximum value of Speed</td>
</tr>
<tr>
<td style="text-align: right;">m=5+25*n</td>
<td colspan="8" style="text-align: center;">[n-1 times, repetition of
byte 5 to 29]</td>
</tr>
</tbody>
</table>

\*1 When an invalid logical unit selection is performed

Byte 4 Number of Operation code

> This field specifies the number of operation codes for which each
> value is set. This field is set to 1 in this unit.

Byte 5 and after (5+25\*n)

> This field specifies the operation codes for which each value is set.
> The 24 bytes following this field indicate the operation parameter for
> the unit of the ID specified in this field. The operation code that is
> used in this unit is shown below.

|                                     |                |
|-------------------------------------|:--------------:|
| Contents of operation               | Operation code |
| Setting of the medium ejection time |      B4h       |

Set value of the operation code setting page

<table>
<colgroup>
<col style="width: 55%" />
<col style="width: 20%" />
<col style="width: 23%" />
</colgroup>
<tbody>
<tr>
<td rowspan="2">Operation code</td>
<td colspan="2" style="text-align: center;">B4</td>
</tr>
<tr>
<td style="text-align: center;">Byte</td>
<td style="text-align: center;">Set value</td>
</tr>
<tr>
<td>Operation Code</td>
<td style="text-align: center;">5</td>
<td style="text-align: center;">B4h</td>
</tr>
<tr>
<td>Minimum value of 1<sup>st</sup> Value</td>
<td style="text-align: center;">6 to 9</td>
<td style="text-align: center;">60</td>
</tr>
<tr>
<td>Maximum value of 1<sup>st</sup> Value</td>
<td style="text-align: center;">10 to 13</td>
<td style="text-align: center;">3600</td>
</tr>
<tr>
<td>Minimum value of 2<sup>nd</sup> Value</td>
<td style="text-align: center;">14 to 17</td>
<td style="text-align: center;">0</td>
</tr>
<tr>
<td>Maximum value of 2<sup>nd</sup> Value</td>
<td style="text-align: center;">18 to 21</td>
<td style="text-align: center;">1</td>
</tr>
<tr>
<td>Minimum value of Speed</td>
<td style="text-align: center;">22 to 25</td>
<td style="text-align: center;">0</td>
</tr>
<tr>
<td>Maximum value of Speed</td>
<td style="text-align: center;">26 to 29</td>
<td style="text-align: center;">0</td>
</tr>
</tbody>
</table>

**2-2-2-7. CCD measurement setting page**

<table>
<colgroup>
<col style="width: 12%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: right;"><p>Bit</p>
<p>Byte</p></td>
<td style="text-align: center;">7</td>
<td style="text-align: center;">6</td>
<td style="text-align: center;">5</td>
<td style="text-align: center;">4</td>
<td style="text-align: center;">3</td>
<td style="text-align: center;">2</td>
<td style="text-align: center;">1</td>
<td style="text-align: center;">0</td>
</tr>
<tr>
<td style="text-align: center;">0</td>
<td colspan="3" style="text-align: center;"><p>Peripheral Qualifier</p>
<p>[0]</p>
<p>[011b](*1)</p></td>
<td colspan="5" style="text-align: center;"><p>Peripheral Device
Type</p>
<p>[6=00110b]</p>
<p>[1Fh=11111b](*1)</p></td>
</tr>
<tr>
<td style="text-align: center;">1</td>
<td colspan="8" style="text-align: center;">Page code [E3h]</td>
</tr>
<tr>
<td style="text-align: center;">2</td>
<td colspan="8" style="text-align: center;">Reserved [0]</td>
</tr>
<tr>
<td style="text-align: center;">3</td>
<td colspan="8" style="text-align: center;">Page length []</td>
</tr>
<tr>
<td style="text-align: center;">4, 5</td>
<td colspan="8" style="text-align: center;">Color of CCD Data</td>
</tr>
<tr>
<td style="text-align: center;">6, 7</td>
<td colspan="8" style="text-align: center;">Resolution of CCD Data</td>
</tr>
<tr>
<td style="text-align: center;">8</td>
<td colspan="8" style="text-align: center;"><p>Scanning Number of CCD
Data</p>
<p>(The number of scanning times for the CCD measurement)</p></td>
</tr>
<tr>
<td style="text-align: center;">9</td>
<td colspan="8" style="text-align: center;"><p>Type of CCD Data</p>
<p>(The number of types for the CCD measurement)</p></td>
</tr>
<tr>
<td style="text-align: center;">10</td>
<td colspan="8" style="text-align: center;"><p>A number of CCD Data
[n]</p>
<p>(The number of measurement points for the CCD measurement)</p></td>
</tr>
<tr>
<td style="text-align: center;">11, 12</td>
<td colspan="8" style="text-align: center;"><p>First value of CCD
Data</p>
<p>(Ratio of the first point for the CCD measurement)</p></td>
</tr>
<tr>
<td style="text-align: center;">13, 14</td>
<td colspan="8" style="text-align: center;"><p>Second value of CCD
Data</p>
<p>(Ratio of the second point for the CCD measurement)</p></td>
</tr>
<tr>
<td style="text-align: center;">:</td>
<td colspan="8" style="text-align: center;">:</td>
</tr>
<tr>
<td style="text-align: center;">n+10, n+11</td>
<td colspan="8" style="text-align: center;"><p>nth value of CCD Data</p>
<p>(Ratio of the nth point for the CCD measurement)</p></td>
</tr>
</tbody>
</table>

\*1 When an invalid logical unit selection is performed

Byte 4 and 5 Color of CCD Data

> This field specifies the color for the CCD measurement. The color in
> which 1 is set is used for the CCD measurement. (Two or more colors
> may be specified simultaneously.)
>
> Byte 4

|       |                   |
|-------|-------------------|
| Bit 0 | R \[0:OFF/1:ON\]  |
| Bit 1 | G \[0:OFF/1:ON\]  |
| Bit 2 | B \[0:OFF/1:ON\]  |
| Bit 3 | NG \[0:OFF/1:ON\] |
| Bit 4 | C \[0:OFF/1:ON\]  |
| Bit 5 | M \[0:OFF/1:ON\]  |
| Bit 6 | Y \[0:OFF/1:ON\]  |
| Bit 7 | K \[0:OFF/1:ON\]  |

Byte 5

|            |          |
|------------|----------|
| Bit 0 to 7 | Reserved |

> Byte 6 and 7 Resolution of CCD Data
>
> This field specifies the resolution for the CCD measurement.

Byte 8 Scanning Number of CCD Data

> This field specifies the number of scanning times for the CCD
> measurement.

Byte 9 Type of CCD Data

> This field specifies the number of types for the CCD measurement. The
> CCD measurement is necessary as many times as the number of
> measurement colors that is specified in byte 4 above multiplied by the
> number of types.

Byte 10 A number of CCD Data

> This field specifies the number of measurement points for the CCD
> measurement.

Byte 11 and after nth value of CCD Data

> This field specifies the ratio of each point for the number of
> measurement points set in byte 6.

**2-3. MODE SELECT (6) Command**

Table 2-3-1 MODE SELECT (6) command

<table style="width:100%;">
<colgroup>
<col style="width: 13%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: right;"><p>Bit</p>
<p>Byte</p></td>
<td style="text-align: center;">7</td>
<td style="text-align: center;">6</td>
<td style="text-align: center;">5</td>
<td style="text-align: center;">4</td>
<td style="text-align: center;">3</td>
<td style="text-align: center;">2</td>
<td style="text-align: center;">1</td>
<td style="text-align: center;">0</td>
</tr>
<tr>
<td style="text-align: center;">0</td>
<td colspan="8" style="text-align: center;">Operation code [15h]</td>
</tr>
<tr>
<td style="text-align: center;">1</td>
<td colspan="3" style="text-align: center;"><p>Logical unit number</p>
<p>[0]</p></td>
<td style="text-align: center;"><p>PF</p>
<p>[1]</p></td>
<td colspan="3" style="text-align: center;"><p>Reserved</p>
<p>[0]</p></td>
<td style="text-align: center;"><p>SP</p>
<p>[0]</p></td>
</tr>
<tr>
<td style="text-align: center;">2, 3</td>
<td colspan="8" style="text-align: center;">Reserved [0]</td>
</tr>
<tr>
<td style="text-align: center;">4</td>
<td colspan="8" style="text-align: center;">Parameter list length
[0,4,12,20]</td>
</tr>
<tr>
<td style="text-align: center;">5</td>
<td colspan="8" style="text-align: center;">Control byte [0]</td>
</tr>
</tbody>
</table>

1)  The MODE SELECT (6) command provides a means for the initiator to
    specify the device parameter to this unit.

2)  The PF (Page Format) bit must be set to 1.

> It means that the MODE SELECT parameters following the header and the
> block descriptor(s) are structured as the pages of the related
> parameters and are as specified in the SCSI-2 standard.

3)  The SP (Save Pages) bit must be set to 0.

> It means that this unit does not have the page save function. If this
> bit is set to 1, this unit responds with common error 1.

4)  The parameter list length field specifies the length in bytes of the
    mode parameter list that is transferred from the initiator to this
    unit during the DATA OUT phase. Setting the parameter list length to
    0 indicates that the data is not transferred. This status is not
    regarded as an error. The effective values of the parameter list
    length are 0, 4, 12, and 20.

> Table 2-3-2 Sense data that is set in each status

<table>
<colgroup>
<col style="width: 36%" />
<col style="width: 33%" />
<col style="width: 29%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: center;">Status</td>
<td style="text-align: center;">Sense data</td>
<td style="text-align: center;">Remarks</td>
</tr>
<tr>
<td style="text-align: center;"><blockquote>
<p>When an initiator sends a MODE SELECT command that changes the
parameter applicable to other initiators</p>
</blockquote></td>
<td style="text-align: center;"><p>MODE PARAMETERS CHANGED</p>
<blockquote>
<p>(The MODE parameter is changed by other initiator when the
multi-initiator is set.)</p>
</blockquote>
<p>06h-2Ah-01h-00h</p></td>
<td style="text-align: center;">Creates the UNIT ATTENTION status for
all initiators other than the initiator that issued the MODE SELECT
command.</td>
</tr>
<tr>
<td style="text-align: center;">When a parameter list length that
results in truncation of any parameter for descriptor, header, or page
is specified</td>
<td style="text-align: center;"><p>PARAMETER LIST LENGTH ERROR</p>
<p>(The parameter length is illegal.)</p>
<p>05h-1Ah-00h-00h</p></td>
<td style="text-align: center;">Terminates with the CHECK CONDITION
status.</td>
</tr>
<tr>
<td style="text-align: center;"><ol type="a">
<li><p>When the initiator changes the field that is not changeable as
reported by this unit to the value other than the current value</p></li>
<li><p>When the initiator sends a MODE SELECT header, block descriptor,
or page header for which a non-supported value is set in the reserved
field</p></li>
<li><p>When the initiator sends a page of the length that is different
from the parameter length reported for that page by the MODE SENSE
command</p></li>
<li><p>When the initiator sends a parameter that has a value exceeding
the support range of this unit</p></li>
<li><p>When the initiator sets a value other than 0 in the reserved
field of the mode parameter</p></li>
</ol></td>
<td style="text-align: center;"><p>INVALID FIELD IN PARAMETER LIST</p>
<p>(Some illegal data exists in the parameter.)</p>
<p>05h-26h-00h-00h</p></td>
<td style="text-align: center;">Terminates the MODE SELECT command with
the CHECK CONDITION status without changing any mode parameter.</td>
</tr>
</tbody>
</table>

\- Mode parameter of this unit

Table 2-3-2 Mode parameter header

<table style="width:100%;">
<colgroup>
<col style="width: 13%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: right;"><p>Bit</p>
<p>Byte</p></td>
<td style="text-align: center;">7</td>
<td style="text-align: center;">6</td>
<td style="text-align: center;">5</td>
<td style="text-align: center;">4</td>
<td style="text-align: center;">3</td>
<td style="text-align: center;">2</td>
<td style="text-align: center;">1</td>
<td style="text-align: center;">0</td>
</tr>
<tr>
<td style="text-align: center;">0</td>
<td colspan="8" style="text-align: center;">Mode Data Length</td>
</tr>
<tr>
<td style="text-align: center;">1</td>
<td colspan="8" style="text-align: center;">Medium Type [0]</td>
</tr>
<tr>
<td style="text-align: center;">2</td>
<td colspan="8" style="text-align: center;">Device-Specific Parameter
[0]</td>
</tr>
<tr>
<td style="text-align: center;">3</td>
<td colspan="8" style="text-align: center;">Block Descriptor Length [0,
8]</td>
</tr>
</tbody>
</table>

1)  When the MODE SENSE command is used, the Mode Data Length field
    specifies the length in bytes of the following data that can be
    transferred. Mode Data Length does not include itself. When using
    the MODE SELECT command, the Mode Data Length field is set to
    ‘Reserved’.

2)  Medium Type is always set to 0.

3)  Device-Specific Parameter is always set to 0.

4)  Block Descriptor Length specifies the length in bytes of all the
    block descriptors. In this unit, 0 or 8 is set. Block Descriptor
    Length of 0 means that the block descriptor is not included in the
    mode parameter list; however, this is not regarded as an error.

\- Mode parameter block descriptor

Table 2-3-3 Mode parameter block descriptor

<table style="width:100%;">
<colgroup>
<col style="width: 13%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: right;"><p>Bit</p>
<p>Byte</p></td>
<td style="text-align: center;">7</td>
<td style="text-align: center;">6</td>
<td style="text-align: center;">5</td>
<td style="text-align: center;">4</td>
<td style="text-align: center;">3</td>
<td style="text-align: center;">2</td>
<td style="text-align: center;">1</td>
<td style="text-align: center;">0</td>
</tr>
<tr>
<td style="text-align: center;">0</td>
<td colspan="8" style="text-align: center;">Density Code [0]</td>
</tr>
<tr>
<td style="text-align: center;">1</td>
<td colspan="8" style="text-align: left;">(MSB)</td>
</tr>
<tr>
<td style="text-align: center;">2</td>
<td colspan="8" style="text-align: center;">Number of Blocks [0]</td>
</tr>
<tr>
<td style="text-align: center;">3</td>
<td colspan="8" style="text-align: right;">(LSB)</td>
</tr>
<tr>
<td style="text-align: center;">4</td>
<td colspan="8" style="text-align: center;">Reserved [0]</td>
</tr>
<tr>
<td style="text-align: center;">5</td>
<td colspan="8" style="text-align: left;">(MSB)</td>
</tr>
<tr>
<td style="text-align: center;">6</td>
<td colspan="8" style="text-align: center;">Block Length [1]</td>
</tr>
<tr>
<td style="text-align: center;">7</td>
<td colspan="8" style="text-align: right;">(LSB)</td>
</tr>
</tbody>
</table>

1)  Density Code field is always set to 0.

2)  Number of Blocks field is always set to 0.

3)  Block Length field is always set to 1.

\- Measurement Units page

This unit supports only the Measurement Units page.

Table 2-3-4 Measurement Units page

<table>
<colgroup>
<col style="width: 13%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 12%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: right;"><p>Bit</p>
<p>Byte</p></td>
<td style="text-align: center;">7</td>
<td style="text-align: center;">6</td>
<td style="text-align: center;">5</td>
<td style="text-align: center;">4</td>
<td style="text-align: center;">3</td>
<td style="text-align: center;">2</td>
<td style="text-align: center;">1</td>
<td style="text-align: center;">0</td>
</tr>
<tr>
<td style="text-align: center;">0</td>
<td style="text-align: center;"><p>PS</p>
<p>[0]</p></td>
<td style="text-align: center;"><p>Reserved</p>
<p>[0]</p></td>
<td colspan="6" style="text-align: center;">Operation code [03h]</td>
</tr>
<tr>
<td style="text-align: center;">1</td>
<td colspan="8" style="text-align: center;">Parameter length [06h]</td>
</tr>
<tr>
<td style="text-align: center;">2</td>
<td colspan="8" style="text-align: center;">Basic measurement unit
[00h]</td>
</tr>
<tr>
<td style="text-align: center;">3</td>
<td colspan="8" style="text-align: center;">Reserved [0]</td>
</tr>
<tr>
<td style="text-align: center;">4</td>
<td colspan="8" style="text-align: left;">(MSB)</td>
</tr>
<tr>
<td style="text-align: center;">5</td>
<td colspan="8" style="text-align: center;"><p>Measurement unit divisor
[1200/Maximum resolution]</p>
<p>(LSB)</p></td>
</tr>
<tr>
<td style="text-align: center;">6, 7</td>
<td colspan="8" style="text-align: center;">Reserved [0]</td>
</tr>
</tbody>
</table>

1)  The Measurement Units page specifies the units of measurement used
    for setting the window position and for positioning an object. The
    measurement units are independent of the scan resolutions in the
    horizontal and the vertical directions.

2)  The Parameters Savable (PS) bit is used only with the MODE SENSE
    command. This bit is reserved for the MODE SELECT command.

> This bit is set to 0 in both the MODE SENSE and the MODE SELECT
> commands. (It is reserved in the MODE SELECT command by the standard.)
>
> In other words, this unit does not have the Parameters Savable
> function. When this bit is 1 in the MODE SELECT command, this unit
> responds with common error 2.

3)  The parameter length field is set to 6.

4)  The basic measurement unit field is set to 0. It means that this
    unit uses only inches as the basic measurement unit. When this bit
    is 1 in the MODE SELECT command, this unit responds with common
    error 2.

5)  The measurement unit divisor specifies the value that is necessary
    to correspond with the basic measurement unit. This unit uses only
    1200 and the maximum resolution value (dpi) of this unit as the
    measurement unit divisor. When this field is set to a value other
    than 1200 or the maximum resolution value (dpi) in the MODE SELECT
    command, this unit responds with common error 2. The default value
    is 1200.

**  **
**2-4. RESERVE UNIT Command**

Table 2-4-1 RESERVE UNIT command

<table style="width:100%;">
<colgroup>
<col style="width: 13%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: right;"><p>Bit</p>
<p>Byte</p></td>
<td style="text-align: center;">7</td>
<td style="text-align: center;">6</td>
<td style="text-align: center;">5</td>
<td style="text-align: center;">4</td>
<td style="text-align: center;">3</td>
<td style="text-align: center;">2</td>
<td style="text-align: center;">1</td>
<td style="text-align: center;">0</td>
</tr>
<tr>
<td style="text-align: center;">0</td>
<td colspan="8" style="text-align: center;">Operation code [16h]</td>
</tr>
<tr>
<td style="text-align: center;">1</td>
<td colspan="3" style="text-align: center;"><p>Logical unit number</p>
<p>[0]</p></td>
<td style="text-align: center;"><p>Third party</p>
<p>[0 or 1]</p></td>
<td colspan="3" style="text-align: center;">Third party device ID</td>
<td style="text-align: center;"><p>Re-served</p>
<p>[0]</p></td>
</tr>
<tr>
<td style="text-align: center;">2 to 4</td>
<td colspan="8" style="text-align: center;">Reserved [0]</td>
</tr>
<tr>
<td style="text-align: center;">5</td>
<td colspan="8" style="text-align: center;">Control byte [0]</td>
</tr>
</tbody>
</table>

The RESERVE UNIT command is used to reserve the logical unit for the
exclusive use by the initiator.

This command requests the reservation of the entire logical unit for the
exclusive use by the initiator until it is replaced with any other valid
RESERVE UNIT command from the initiator that made the reservation, it is
released by the RELEASE UNIT command from the same initiator, or it is
released by the hard reset status or the power-ON cycle. It is
permissible for the initiator that currently makes reservation to
reserve the logical unit again.

**  
2-5. RELEASE UNIT Command**

Table 2-5-1 RELEASE UNIT command

<table style="width:100%;">
<colgroup>
<col style="width: 13%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: right;"><p>Bit</p>
<p>Byte</p></td>
<td style="text-align: center;">7</td>
<td style="text-align: center;">6</td>
<td style="text-align: center;">5</td>
<td style="text-align: center;">4</td>
<td style="text-align: center;">3</td>
<td style="text-align: center;">2</td>
<td style="text-align: center;">1</td>
<td style="text-align: center;">0</td>
</tr>
<tr>
<td style="text-align: center;">0</td>
<td colspan="8" style="text-align: center;">Operation code [17h]</td>
</tr>
<tr>
<td style="text-align: center;">1</td>
<td colspan="3" style="text-align: center;"><p>Logical unit number</p>
<p>[0]</p></td>
<td style="text-align: center;"><p>Third party</p>
<p>[0 or 1]</p></td>
<td colspan="3" style="text-align: center;">Third party device ID</td>
<td style="text-align: center;"><p>Re-served</p>
<p>[0]</p></td>
</tr>
<tr>
<td style="text-align: center;">2 to 4</td>
<td colspan="8" style="text-align: center;">Reserved [0]</td>
</tr>
<tr>
<td style="text-align: center;">5</td>
<td colspan="8" style="text-align: center;">Control byte [0]</td>
</tr>
</tbody>
</table>

The RELEASE UNIT command is used for the initiator that issued the
command to release the reservation of the logical unit that has already
been reserved.

If there is any valid reservation, this unit releases the reservation
and returns the GOOD status.

Only the initiator that executed the reservation can release the
reservation. The command that attempts to release the reservation which
is not currently valid is not regarded as an error. At this time, this
unit returns the GOOD status without changing any other reservation.

**2-6. MODE SENSE (6) Command**

Table 2-6-1 MODE SENSE command

<table style="width:100%;">
<colgroup>
<col style="width: 13%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: right;"><p>Bit</p>
<p>Byte</p></td>
<td style="text-align: center;">7</td>
<td style="text-align: center;">6</td>
<td style="text-align: center;">5</td>
<td style="text-align: center;">4</td>
<td style="text-align: center;">3</td>
<td style="text-align: center;">2</td>
<td style="text-align: center;">1</td>
<td style="text-align: center;">0</td>
</tr>
<tr>
<td style="text-align: center;">0</td>
<td colspan="8" style="text-align: center;">Operation code [1Ah]</td>
</tr>
<tr>
<td style="text-align: center;">1</td>
<td colspan="3" style="text-align: center;"><p>Logical unit number</p>
<p>[0]</p></td>
<td style="text-align: center;"><p>PF</p>
<p>[1]</p></td>
<td style="text-align: center;"><p>DBD</p>
<p>[0 or 1]</p></td>
<td colspan="3" style="text-align: center;"><p>Reserved</p>
<p>[0]</p></td>
</tr>
<tr>
<td style="text-align: center;">2</td>
<td colspan="2" style="text-align: center;">PC</td>
<td colspan="6" style="text-align: center;">Page code</td>
</tr>
<tr>
<td style="text-align: center;">3</td>
<td colspan="8" style="text-align: center;">Reserved [0]</td>
</tr>
<tr>
<td style="text-align: center;">4</td>
<td colspan="8" style="text-align: center;">Allocation length</td>
</tr>
<tr>
<td style="text-align: center;">5</td>
<td colspan="8" style="text-align: center;">Control byte [0]</td>
</tr>
</tbody>
</table>

The MODE SENSE (6) command provides a means for this unit to report the
parameters to the initiator.

1)  For the PF (Page Format) bit, refer to the subsection describing the
    MODE SELECT command.

2)  A DBD (Disable Block Descriptors) bit of zero indicates that this
    unit returns the block descriptors in the mode sense data. When this
    bit is set to 1, this unit does not return the block descriptors in
    the mode sense data.

3)  The PC (Page Control) field defines the type of the mode page
    parameter value that is returned. The PC field is defined in table
    2-6-2. When the PC field is 3h (saved value in the SCSI-2 standard),
    this unit sets the sense key and the additional sense code to
    ILLEGAL REQUEST and SAVING PARAMETERS NOT SUPPORTED, respectively
    and terminates the command with the CHECK CONDITION status.

Table 2-6-2 PC field

<table>
<colgroup>
<col style="width: 30%" />
<col style="width: 70%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: center;">Code</td>
<td style="text-align: center;">Parameter type</td>
</tr>
<tr>
<td style="text-align: center;"><p>00b</p>
<p>01b</p>
<p>10b</p></td>
<td style="text-align: center;"><p>Current value</p>
<p>Variable value</p>
<p>Default value</p></td>
</tr>
</tbody>
</table>

> Current value
>
> The PC field value of 00b requests this unit to return the current
> parameter value for the specified page code of the logical unit. The
> current values returned are:
>
> a\) The parameters set in the last successful MODE SELECT command
>
> b\) The default values if a MODE SELECT command has not been executed
> normally since the last power-on, hard reset condition, or the BUS
> DEVICE RESET message
>
> Variable value
>
> When the PC field is set to 01b, the variable parameter mask value for
> the page code specified by this unit is returned. The information
> indicating which parameters are variable in the requested page is
> returned. The bit corresponding to the variable parameter is set to 1.
> The bit corresponding to the parameter that is not variable is set to
> 0 by the initiator.
>
> In this unit, of the parameters in the Measurement Units page, all
> bits in the measurement unit divisor field are set to 1 and all bits
> in the basic measurement unit field are set to 0, because the
> measurement unit divisor is variable but the basic measurement unit is
> not.
>
> Default value
>
> When the PC field is set to 10b, the default value for the page code
> specified by this unit is returned. This unit sets 0 for the
> parameters that are not supported.

4)  The page code field specifies which page or pages to return. The
    page code usage is defined in table 2-6-3.

Table 2-6-3 Page code usage

<table>
<colgroup>
<col style="width: 32%" />
<col style="width: 67%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: center;">Page code</td>
<td style="text-align: center;">Descriptions</td>
</tr>
<tr>
<td style="text-align: center;"><p>03h</p>
<p>3Fh</p></td>
<td style="text-align: center;"><p>Returns the Measurement Units
page</p>
<p>Returns all pages</p></td>
</tr>
</tbody>
</table>

> An initiator may request one or all of the pages supported by this
> unit. If an initiator issues a MODE SENSE command with a page code
> value that is not implemented by this unit, this unit shall respond
> with common error 1.
>
> A page code of 3Fh indicates that all pages implemented by this unit
> (however, this unit supports the Measurement Units page only) shall be
> returned to the initiator.

If a MODE SENSE command with both the PC field and the page code field
set to 0 is received, this unit returns only the mode parameter header
and the block descriptor.

**2-7. SCAN Command**

Table 2-7-1 SCAN command

<table>
<colgroup>
<col style="width: 13%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: right;"><p>Bit</p>
<p>Byte</p></td>
<td style="text-align: center;">7</td>
<td style="text-align: center;">6</td>
<td style="text-align: center;">5</td>
<td style="text-align: center;">4</td>
<td style="text-align: center;">3</td>
<td style="text-align: center;">2</td>
<td style="text-align: center;">1</td>
<td style="text-align: center;">0</td>
</tr>
<tr>
<td style="text-align: center;">0</td>
<td colspan="8" style="text-align: center;">Operation code [1Bh]</td>
</tr>
<tr>
<td style="text-align: center;">1</td>
<td colspan="3" style="text-align: center;"><p>Logical unit number</p>
<p>[0]</p></td>
<td colspan="5" style="text-align: center;"><p>Reserved</p>
<p>[0]</p></td>
</tr>
<tr>
<td style="text-align: center;">2, 3</td>
<td colspan="8" style="text-align: center;">Reserved [0]</td>
</tr>
<tr>
<td style="text-align: center;">4</td>
<td colspan="8" style="text-align: center;">Transfer length [0, 1, 2, 3,
4]</td>
</tr>
<tr>
<td style="text-align: center;">5</td>
<td colspan="8" style="text-align: center;">Control byte [0]</td>
</tr>
</tbody>
</table>

The SCAN command requests this unit to start the scanning operation.

1)  This command is an operation activation command. When receiving this
    command, this unit shall send the status and then start the scanning
    operation.

> After this command is executed, the image data reading can be
> performed by the READ command.
>
> The transfer length specifies the length in bytes of the window
> identifier list that shall be sent during the DATA OUT phase. If the
> transfer length is 0, the data is not transferred. This is not an
> error.

2)  The window identifier list consists of the window identifiers that
    define the window to be scanned. This unit has three windows for
    each of the default color, and R, G, and B color. This unit performs
    the scanning operation for the window that is specified by the
    window identifier.

> The default color is valid when only the default color is read.

The sense data that is set in each status is shown in table 2-7-2.

Table 2-7-2 Sense data that is set in each status

<table>
<colgroup>
<col style="width: 29%" />
<col style="width: 39%" />
<col style="width: 31%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: center;">Status</td>
<td style="text-align: center;">Sense data</td>
<td style="text-align: center;">Remarks</td>
</tr>
<tr>
<td style="text-align: center;">When the default color is specified with
other color in the window</td>
<td style="text-align: center;"><p>INVALID COMBINATION OF WINDOWS
SPECIFIED</p>
<p>05h-2Ch-02h-00h</p></td>
<td style="text-align: center;">Terminates with the CHECK CONDITION
status.</td>
</tr>
<tr>
<td style="text-align: center;">When the overlapped setting is performed
between two or more windows (a different setting is performed in the
parameter common to all windows)</td>
<td style="text-align: center;"><p>INVALID COMBINATION OF WINDOWS
SPECIFIED</p>
<p>05h-2Ch-02h-00h</p></td>
<td style="text-align: center;">Terminates with the CHECK CONDITION
status.</td>
</tr>
<tr>
<td style="text-align: center;">When Multiple Reading is set</td>
<td style="text-align: center;"><p>AVERAGING MULTIPLE READING BY
DRIVER</p>
<p>(The averaging processing during Multiple Reading is performed by the
initiator.)</p>
<p>09h-80h-02h-00h</p></td>
<td style="text-align: center;">The initiator cooperative action
parameter is read by the READ command following the SCAN command and the
averaging processing is performed on the initiator side based on the
information.</td>
</tr>
<tr>
<td style="text-align: center;"><p>When Thumbnail is set</p>
<p>(240)</p></td>
<td style="text-align: center;"><p>THUMBNAIL CREATED BY DRIVER</p>
<p>(The thumbnail image of the 240 film is created by the
initiator.)</p>
<p>09h-80h-01h-02h</p></td>
<td rowspan="2" style="text-align: center;">The initiator cooperative
action parameter is read by the READ command following the SCAN command
and the thumbnail is created on the initiator side based on the
information. The initiator issues the SCAN command again after
performing the necessary operation.</td>
</tr>
<tr>
<td style="text-align: right;"><p>(6-frame strip)</p>
<p>(36-frame strip)</p></td>
<td style="text-align: center;"><p>THUMBNAIL CREATED BY DRIVER</p>
<p>(The thumbnail image of the strip film is created by the
initiator.)</p>
<p>09h-80h-01h-06h</p></td>
</tr>
<tr>
<td style="text-align: center;"><p>For two-line reading, when a setting
other than the combination of an even-number start address and an
odd-number end address is made</p>
<p>When the sent data is not a multiple of 512 bytes</p></td>
<td style="text-align: center;"><p>TRUNCATED BY DRIVER</p>
<p>(The invalid data that is sent excessively is deleted by the
initiator.)</p>
<p>09h-80h-06h-01h</p></td>
<td style="text-align: center;">The SCAN command is issued again. The
excess data is deleted on the initiator side by the READ command that is
issued following the SCAN command.</td>
</tr>
<tr>
<td style="text-align: center;">When the CCD DATA is ON while Image
Scanning is set</td>
<td style="text-align: center;"><p>CCD DATA CREATED BY DRIVER</p>
<p>9h-80h-07h-00h</p></td>
<td style="text-align: center;">The initiator cooperative action
parameter is read by the READ command following the SCAN command.</td>
</tr>
<tr>
<td style="text-align: center;">If Set up Scanning is set, after the
operation is activated by the SCAN command, when the completion of
reading and the device internal processing operation is confirmed by the
TEST UNIT READY command, and the operation is not terminated
normally</td>
<td style="text-align: center;"><p>LOGICAL UNIT NOT READY, CAUSE NOT
REPORTABLE</p>
<p>(The internal mechanical error occurred.)</p>
<p>02h-04h-02h-00h</p></td>
<td style="text-align: center;">Terminates with the CHECK CONDITION
status. If the operation is terminated normally, after the operation
completion is confirmed, Max Value can be read by the READ command.</td>
</tr>
<tr>
<td style="text-align: center;">After the SCAN command is terminated
with GOOD status, until the scan preparation such as the stage movement
is completed (for TEST UNIT READY)</td>
<td style="text-align: center;"><p>LOGICAL UNIT IS IN PROCESS OF
BECOMING READY</p>
<p>(During the execution of the operation activation command)</p>
<p>02h-04h-01h-00h</p>
<p>(During loading/ejection of the object to be scanned)</p>
<p>02h-04h-01h-01h</p>
<p>(During the measurement of the correction data)</p>
<p>02h-04h-01h-02h</p>
<p>(During the execution of operation for loading the object to be
scanned)</p>
<p>02h-04h-01h-03h</p>
<p>(During the execution of automatic shading or white balance
measurement)</p>
<p>02h-04h-01h-04h</p></td>
<td style="text-align: center;">Terminates with GOOD status after the
preparation is completed even in the scanning status.</td>
</tr>
</tbody>
</table>

**2-8. SEND DIAGNOSTIC Command**

Table 2-8-1 SEND DIAGNOSTIC command

<table>
<colgroup>
<col style="width: 13%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: right;"><p>Bit</p>
<p>Byte</p></td>
<td style="text-align: center;">7</td>
<td style="text-align: center;">6</td>
<td style="text-align: center;">5</td>
<td style="text-align: center;">4</td>
<td style="text-align: center;">3</td>
<td style="text-align: center;">2</td>
<td style="text-align: center;">1</td>
<td style="text-align: center;">0</td>
</tr>
<tr>
<td style="text-align: center;">0</td>
<td colspan="8" style="text-align: center;">Operation code [1Dh]</td>
</tr>
<tr>
<td style="text-align: center;">1</td>
<td colspan="3" style="text-align: center;"><p>Logical unit number</p>
<p>[0]</p></td>
<td style="text-align: center;"><p>PF</p>
<p>[0 or 1]</p></td>
<td style="text-align: center;"><p>Re-served</p>
<p>[0]</p></td>
<td style="text-align: center;"><p>Self</p>
<p>Test</p>
<p>[0 or 1]</p></td>
<td style="text-align: center;"><p>DevOfL</p>
<p>[0]</p></td>
<td style="text-align: center;"><p>Unit</p>
<p>OfL</p>
<p>[0]</p></td>
</tr>
<tr>
<td style="text-align: center;">2</td>
<td colspan="8" style="text-align: center;">Reserved [0]</td>
</tr>
<tr>
<td style="text-align: center;">3, 4</td>
<td colspan="8" style="text-align: center;">Parameter list length</td>
</tr>
<tr>
<td style="text-align: center;">5</td>
<td colspan="8" style="text-align: center;">Control byte [0]</td>
</tr>
</tbody>
</table>

The SEND DIAGNOSTIC command performs the self-test for this unit itself.

For the self-test of this unit, ‘Parameter exists’ or ‘Parameter does
not exist’ can be selected.

●If the parameter does not exist

1)  The page format (PF) bit is set to 0, the Self Test bit is set to 1,
    and the parameter list length is set to 0.

2)  After the power ON, when this command is received without executing
    the operation activation command once, the command is terminated
    with GOOD status if the diagnostic result is normal during power ON,
    or the sense data is set according to each abnormal status and the
    command is terminated with CHECK CONDITION status if the diagnostic
    result is abnormal.

3)  If the operation activation command was executed, in order to notice
    the hardware error information, the sense data is set and the
    command is terminated with the CHECK CONDITION status if an error
    information exists, and the command is terminated with GOOD status
    if an error information does not exist.

> If an error occurs in the operation activation command, when the
> operation is completed, the sense data is set as shown in the table
> below in the CHECK CONDITION status for TEST UNIT READY. Then the
> concrete error information is set in the sense data by executing this
> command.
>
> If the error information is set in the sense data by this command, the
> error information occurred in the operation activation command is
> cleared.

<table>
<colgroup>
<col style="width: 36%" />
<col style="width: 38%" />
<col style="width: 24%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: center;">Status</td>
<td style="text-align: center;">Sense data</td>
<td style="text-align: center;">Remarks</td>
</tr>
<tr>
<td style="text-align: center;">An error occurred in the operation
activation command.</td>
<td style="text-align: center;"><p>Logical Unit Not Ready, Cause Not
Reportable</p>
<p>(The internal mechanical error occurred.)</p>
<p>02h-04h-02h-00h</p></td>
<td style="text-align: center;"></td>
</tr>
</tbody>
</table>

- If the parameter exists

1)  The page format (PF) bit is set to 1, the Self Test bit is set to 0,
    and the parameter list length is set to the transferred parameter
    length in bytes. This unit does not support this status.

    The SCSI device off-line (DevOfl) bit and the logical unit off-line
    (UnitOfl) bit must be set to 0 regardless of whether the parameter
    exists or not.

**  
2-9. SET WINDOW Command**

Table 2-9-1 SET WINDOW command

<table style="width:100%;">
<colgroup>
<col style="width: 0%" />
<col style="width: 13%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 0%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
</colgroup>
<tbody>
<tr>
<td colspan="2" style="text-align: right;"><p>Bit</p>
<p>Byte</p></td>
<td style="text-align: center;">7</td>
<td style="text-align: center;">6</td>
<td style="text-align: center;">5</td>
<td colspan="2" style="text-align: center;">4</td>
<td style="text-align: center;">3</td>
<td style="text-align: center;">2</td>
<td style="text-align: center;">1</td>
<td style="text-align: center;">0</td>
</tr>
<tr>
<td colspan="2" style="text-align: center;">0</td>
<td colspan="9" style="text-align: center;">Operation code [24h]</td>
</tr>
<tr>
<td colspan="2" style="text-align: center;">1</td>
<td colspan="4" style="text-align: center;"><p>Logical unit number</p>
<p>[0]</p></td>
<td colspan="5" style="text-align: center;"><p>Reserved</p>
<p>[0]</p></td>
</tr>
<tr>
<td colspan="2" style="text-align: center;">2 to 5</td>
<td colspan="9" style="text-align: center;">Reserved [0]</td>
</tr>
<tr>
<td colspan="2" style="text-align: center;"></td>
<td colspan="9" style="text-align: left;">(MSB)</td>
</tr>
<tr>
<td colspan="2" style="text-align: center;">6 to 8</td>
<td colspan="9" style="text-align: center;">Transfer length [Recommended
value: 58d]</td>
</tr>
<tr>
<td colspan="2" style="text-align: center;"></td>
<td colspan="9" style="text-align: right;">(LSB)</td>
</tr>
<tr>
<td colspan="2" style="text-align: center;">9</td>
<td style="text-align: center;"><p>Reserved</p>
<p>[0]</p></td>
<td colspan="8" style="text-align: center;">Control byte [0]</td>
</tr>
</tbody>
</table>

- The SET WINDOW command provides a means for the initiator to define
  the windows within the scanning range of the device. The windows are
  defined by each color.

- The transfer length specifies the length in bytes of the data that
  shall be transferred during the DATA OUT phase. A transfer length of 0
  indicates that no window parameter data shall be transferred. This
  status is not regarded as an error.

- When the transfer length is shorter than the full window parameter
  length of this unit, the lacking part of the parameter shall be
  unchanged. If the multi-byte parameter is incompletely transferred,
  the suspended data shall be considered invalid and the original
  parameter shall be unchanged.

- When the transfer length is longer than the full window parameter
  length, this unit responds with common error 1. If an illegal data is
  included, this unit responds with common error 2.

Table 2-9-2 SET WINDOW data header

<table style="width:100%;">
<colgroup>
<col style="width: 12%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: right;"><p>Bit</p>
<p>Byte</p></td>
<td style="text-align: center;">7</td>
<td style="text-align: center;">6</td>
<td style="text-align: center;">5</td>
<td style="text-align: center;">4</td>
<td style="text-align: center;">3</td>
<td style="text-align: center;">2</td>
<td style="text-align: center;">1</td>
<td style="text-align: center;">0</td>
</tr>
<tr>
<td style="text-align: center;">0 to 5</td>
<td colspan="8" style="text-align: center;">Reserved</td>
</tr>
<tr>
<td style="text-align: center;">6</td>
<td colspan="8" style="text-align: center;">(MSB) Window descriptor
length</td>
</tr>
<tr>
<td style="text-align: center;">7</td>
<td colspan="8" style="text-align: right;">[Recommended value: 50d]
(LSB)</td>
</tr>
</tbody>
</table>

- The window parameter data consists of a header followed by one or more
  window descriptors (refer to table 2-10-3). Each window descriptor
  specifies the location, size, and the scanning method of the window.

The window descriptor length specifies the length in bytes of a single
window descriptor.

**  
2-10. GET WINDOW Command**

Table 2-10-1 GET WINDOW command

<table>
<colgroup>
<col style="width: 13%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 0%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 11%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: right;"><p>Bit</p>
<p>Byte</p></td>
<td style="text-align: center;">7</td>
<td style="text-align: center;">6</td>
<td colspan="2" style="text-align: center;">5</td>
<td style="text-align: center;">4</td>
<td style="text-align: center;">3</td>
<td style="text-align: center;">2</td>
<td style="text-align: center;">1</td>
<td style="text-align: center;">0</td>
</tr>
<tr>
<td style="text-align: center;">0</td>
<td colspan="9" style="text-align: center;">Operation code [25h]</td>
</tr>
<tr>
<td style="text-align: center;">1</td>
<td colspan="4" style="text-align: center;"><p>Logical unit number</p>
<p>[0]</p></td>
<td colspan="4" style="text-align: center;"><p>Reserved</p>
<p>[0]</p></td>
<td style="text-align: center;"><p>Single</p>
<p>[0, 1]</p></td>
</tr>
<tr>
<td style="text-align: center;">2 to 4</td>
<td colspan="9" style="text-align: center;">Reserved [0]</td>
</tr>
<tr>
<td style="text-align: center;">5</td>
<td colspan="9" style="text-align: center;">Window identifier [0, 1, 2,
3]</td>
</tr>
<tr>
<td style="text-align: center;"></td>
<td colspan="9" style="text-align: left;">(MSB)</td>
</tr>
<tr>
<td style="text-align: center;">6 to 8</td>
<td colspan="9" style="text-align: center;">Transfer length [Recommended
value: (50*the number of windows+8)d]</td>
</tr>
<tr>
<td style="text-align: center;"></td>
<td colspan="9" style="text-align: right;">(LSB)</td>
</tr>
<tr>
<td style="text-align: center;">9</td>
<td colspan="3" style="text-align: center;">Reserved [0]</td>
<td colspan="6" style="text-align: center;">Control byte [0]</td>
</tr>
</tbody>
</table>

The GET WINDOW command reports the values that are currently set in the
scanner for each window parameter.

- The Single bit specifies the type of the requested window definition
  information.

> When this bit is set to 1, the definition information about a single
> window specified by byte 5 is transferred to the initiator.
>
> When this bit is set to 0, the definition information about all the
> windows that are defined by SET WINDOW or that are set in this unit as
> the default is transferred to the initiator.

- For the parameters to which 0 (default value) is set in SET WINDOW,
  the values that are currently set in this unit are reported.

<!-- -->

- The window identifier specifies the window for which the definition
  information is requested when the Single bit is set to 1. The window
  identifier is defined in each color. When the Single bit is 0, this
  byte is invalid.

<!-- -->

- For the window identifier, the default color, R, G, B, and Neutral
  Gray can be set to 0, 1, 2, 3, and 4, respectively.

<!-- -->

- The GET WINDOW data consists of a header followed by one or more
  window descriptors. Each window descriptor specifies the location,
  size, and the scanning method of the window.

Table 2-10-2 GET WINDOW data header

<table style="width:100%;">
<colgroup>
<col style="width: 13%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: right;"><p>Bit</p>
<p>Byte</p></td>
<td style="text-align: center;">7</td>
<td style="text-align: center;">6</td>
<td style="text-align: center;">5</td>
<td style="text-align: center;">4</td>
<td style="text-align: center;">3</td>
<td style="text-align: center;">2</td>
<td style="text-align: center;">1</td>
<td style="text-align: center;">0</td>
</tr>
<tr>
<td style="text-align: center;">0</td>
<td colspan="8" style="text-align: left;">(MSB) Window data length</td>
</tr>
<tr>
<td style="text-align: center;">1</td>
<td colspan="8" style="text-align: right;">[Recommended value: (50*the
number of windows+6)d] (LSB)</td>
</tr>
<tr>
<td style="text-align: center;">2 to 5</td>
<td colspan="8" style="text-align: center;">Reserved</td>
</tr>
<tr>
<td style="text-align: center;">6</td>
<td colspan="8" style="text-align: left;">(MSB) Window descriptor
length</td>
</tr>
<tr>
<td style="text-align: center;">7</td>
<td colspan="8" style="text-align: right;">[Recommended value: 50d]
(LSB)</td>
</tr>
</tbody>
</table>

The window data length specifies the length in bytes of the data that is
transferred following it. The window data length does not include
itself. Even if the allocated length is not enough to return all the GET
WINDOW data, the window data length is not adjusted for sending the cut
data again.

The window descriptor length specifies the window descriptor length in
bytes for a single window.

Table 2-10-3 Window Descriptor Byte

<table>
<colgroup>
<col style="width: 12%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 11%" />
</colgroup>
<thead>
<tr>
<th style="text-align: right;"><p>Bit</p>
<p>Byte</p></th>
<th style="text-align: center;">7</th>
<th style="text-align: center;">6</th>
<th style="text-align: center;">5</th>
<th style="text-align: center;">4</th>
<th style="text-align: center;">3</th>
<th style="text-align: center;">2</th>
<th style="text-align: center;">1</th>
<th style="text-align: center;">0</th>
</tr>
</thead>
<tbody>
<tr>
<td style="text-align: center;">0</td>
<td colspan="8" style="text-align: center;">Window Identifier [0, 1, 2,
3] (The default is 2.)</td>
</tr>
<tr>
<td style="text-align: center;">1</td>
<td colspan="7" style="text-align: center;"><p>Reserved</p>
<p>[0]</p></td>
<td style="text-align: center;"><p>Auto</p>
<p>[0]</p></td>
</tr>
<tr>
<td style="text-align: center;">2, 3</td>
<td colspan="8" style="text-align: center;"><blockquote>
<p>X Resolution [90 to 4000]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">4, 5</td>
<td colspan="8" style="text-align: center;"><blockquote>
<p>Y Resolution [90 to 4000]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">6 to 9</td>
<td colspan="8" style="text-align: center;"><blockquote>
<p>Upper Left X Offset (The default is 0.)</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">10 to 13</td>
<td colspan="8" style="text-align: center;"><blockquote>
<p>Upper Left Y Offset (The default is 0.)</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">14 to 17</td>
<td colspan="8" style="text-align: center;"><blockquote>
<p>Window Width</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">18 to 21</td>
<td colspan="8" style="text-align: center;"><blockquote>
<p>Window Length</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">22</td>
<td colspan="8" style="text-align: center;"><blockquote>
<p>Brightness [0]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">23</td>
<td colspan="8" style="text-align: center;"><blockquote>
<p>Threshold [0]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">24</td>
<td colspan="8" style="text-align: center;"><blockquote>
<p>Contrast [0]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">25</td>
<td colspan="8" style="text-align: center;"><blockquote>
<p>Image Composition [2 or 5] (The default is 2.)</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">26</td>
<td colspan="8" style="text-align: center;"><blockquote>
<p>Pixel Composition [16d]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">27, 28</td>
<td colspan="8" style="text-align: center;"><blockquote>
<p>Halftone Pattern [0]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">29</td>
<td style="text-align: center;"><p>Reverse</p>
<p>[0]</p></td>
<td colspan="4" style="text-align: center;"><p>Reserved</p>
<p>[0]</p></td>
<td colspan="3" style="text-align: center;"><p>Padding Type</p>
<p>[0]</p></td>
</tr>
<tr>
<td style="text-align: center;">30, 31</td>
<td colspan="8" style="text-align: center;"><blockquote>
<p>Bit Ordering [0]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">32</td>
<td colspan="8" style="text-align: center;"><blockquote>
<p>Compression Type [0]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">33</td>
<td colspan="8" style="text-align: center;"><blockquote>
<p>Compression Argument [0]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">34 to 39</td>
<td colspan="8" style="text-align: center;"><blockquote>
<p>Reserved [0]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">40</td>
<td colspan="4" style="text-align: center;"><p>Multiple Reading Number
[0 to 15]</p>
<p>(The default is 0.)</p></td>
<td colspan="4" style="text-align: center;"><p>Color Ordering [0, 1, 2,
3]</p>
<p>(The default is R=1, G=2, B=3)</p></td>
</tr>
<tr>
<td style="text-align: center;">41</td>
<td style="text-align: center;"><p>Averag-ing</p>
<p>1: ON</p>
<p>0: OFF*</p></td>
<td style="text-align: center;"><p>Matrix</p>
<p>[0]</p></td>
<td style="text-align: center;"><p>Filter</p>
<p>[0]</p></td>
<td style="text-align: center;"><p>Reserved</p>
<p>[0]</p></td>
<td colspan="3" style="text-align: center;">Setup Mode</td>
<td style="text-align: center;"><p>Object</p>
<p>1: Posi*</p>
<p>0: Nega</p></td>
</tr>
<tr>
<td style="text-align: center;">42</td>
<td colspan="8" style="text-align: center;"><blockquote>
<p>Scanning Kind (The default is 1.)</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">43</td>
<td colspan="8" style="text-align: center;"><blockquote>
<p>Scanning Mode (The default is 2.)</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">44</td>
<td colspan="8" style="text-align: center;"><blockquote>
<p>Color interleaving (The default is 2.)</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">45</td>
<td colspan="8" style="text-align: center;"><blockquote>
<p>AE Value (The default is 255d.)</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">46 to 49</td>
<td colspan="8" style="text-align: center;"><blockquote>
<p>Exposure Value [0 to 3FFFFFFh]</p>
</blockquote></td>
</tr>
</tbody>
</table>

\* Default value

> Each field is described below.

The values are the absolute values to the standard, and not the relative
values to the current settings.

The upper byte is first in multi-byte parameters.

A value of zero specifies the default value in the device.

Byte 0

> The window identifier specifies the window defined by the window
> descriptor.
>
> This unit defines the window in each scanning color by using the
> window identifier as shown in table 2-10-4. The default is 2.

Table 2-10-4 Relation between the window identifier and the scanning
color

|                   |                   |                      |
|:-----------------:|:-----------------:|:--------------------:|
| Window identifier |  Scanning color   | Support of this unit |
|         0         | Default color (G) |         Yes          |
|         1         |         R         |         Yes          |
|         2         |         G         |         Yes          |
|         3         |         B         |         Yes          |
|         4         |   Neutral gray    |          No          |

Byte 2 and 3

> The X Resolution field specifies the resolution in the scan line
> direction.
>
> In this unit, scanning can be performed only with the resolution of
> (1/Integer) of the maximum resolution.
>
> Therefore, if a resolution other than the recommended resolution shown
> in the attached material is specified, the resolution is rounded to
> the supported resolution for processing by the following formulas. In
> this case, Rounded parameter (sense key 01h, ASC = 37h, ASCQ = 00h) is
> set to the sense data and the command is terminated with the CHECK
> CONDITION status.
>
> Scanning pitch = Maximum resolution/Specified resolution (Fractions
> are rounded off)
>
> Scanning resolution = Maximum resolution/Scanning pitch
>
> It is possible to obtain the resolution range that can be set in this
> unit by the Inquiry command. For the correspondence between the
> specified resolution and the scanning resolution, and the recommended
> resolution, refer to table 2-10-5.
>
> The default is the maximum resolution.

Table 2-10-5 Relation between the specified resolution and the scanning
resolution

<table style="width:68%;">
<colgroup>
<col style="width: 23%" />
<col style="width: 8%" />
<col style="width: 3%" />
<col style="width: 18%" />
<col style="width: 1%" />
<col style="width: 9%" />
<col style="width: 2%" />
</colgroup>
<tbody>
<tr>
<td colspan="2" style="text-align: center;">Set Window
specification</td>
<td colspan="5" style="text-align: center;">Scanning at the device</td>
</tr>
<tr>
<td colspan="2" style="text-align: center;">X resolution</td>
<td colspan="3" style="text-align: center;">Scanning resolution</td>
<td colspan="2" style="text-align: center;">Pitch</td>
</tr>
<tr>
<td colspan="2" style="text-align: center;">4000 to 2001</td>
<td colspan="2" style="text-align: center;">4000</td>
<td colspan="2" style="text-align: center;">1</td>
<td style="text-align: center;"></td>
</tr>
<tr>
<td colspan="2" style="text-align: center;">2000 to 1001</td>
<td colspan="2" style="text-align: center;">2000</td>
<td colspan="2" style="text-align: center;">2</td>
<td style="text-align: center;"></td>
</tr>
<tr>
<td colspan="2" style="text-align: center;">1000 to 667</td>
<td colspan="2" style="text-align: center;">1000</td>
<td colspan="2" style="text-align: center;">4</td>
<td style="text-align: center;"></td>
</tr>
<tr>
<td colspan="2" style="text-align: center;">666 to 501</td>
<td colspan="2" style="text-align: center;">666</td>
<td colspan="2" style="text-align: center;">6</td>
<td style="text-align: center;"></td>
</tr>
<tr>
<td colspan="2" style="text-align: center;">500 to 401</td>
<td colspan="2" style="text-align: center;">500</td>
<td colspan="2" style="text-align: center;">8</td>
<td style="text-align: center;"></td>
</tr>
<tr>
<td colspan="2" style="text-align: center;">400 to 334</td>
<td colspan="2" style="text-align: center;">400</td>
<td colspan="2" style="text-align: center;">10</td>
<td style="text-align: center;"></td>
</tr>
<tr>
<td colspan="2" style="text-align: center;">333 to 286</td>
<td colspan="2" style="text-align: center;">333</td>
<td colspan="2" style="text-align: center;">12</td>
<td style="text-align: center;"></td>
</tr>
<tr>
<td colspan="2" style="text-align: center;">285 to 251</td>
<td colspan="2" style="text-align: center;">285</td>
<td colspan="2" style="text-align: center;">14</td>
<td style="text-align: center;"></td>
</tr>
<tr>
<td colspan="2" style="text-align: center;">250 to 223</td>
<td colspan="2" style="text-align: center;">250</td>
<td colspan="2" style="text-align: center;">16</td>
<td style="text-align: center;"></td>
</tr>
<tr>
<td colspan="2" style="text-align: center;">222 to 201</td>
<td colspan="2" style="text-align: center;">222</td>
<td colspan="2" style="text-align: center;">18</td>
<td style="text-align: center;"></td>
</tr>
<tr>
<td colspan="2" style="text-align: center;">200 to 182</td>
<td colspan="2" style="text-align: center;">200</td>
<td colspan="2" style="text-align: center;">20</td>
<td style="text-align: center;"></td>
</tr>
<tr>
<td colspan="2" style="text-align: center;">181 to 167</td>
<td colspan="2" style="text-align: center;">181</td>
<td colspan="2" style="text-align: center;">22</td>
<td style="text-align: center;"></td>
</tr>
<tr>
<td colspan="2" style="text-align: center;">166 to 154</td>
<td colspan="2" style="text-align: center;">166</td>
<td colspan="2" style="text-align: center;">24</td>
<td style="text-align: center;"></td>
</tr>
<tr>
<td colspan="2" style="text-align: center;">153 to 143</td>
<td colspan="2" style="text-align: center;">153</td>
<td colspan="2" style="text-align: center;">26</td>
<td style="text-align: center;"></td>
</tr>
<tr>
<td colspan="2" style="text-align: center;">142 to 134</td>
<td colspan="2" style="text-align: center;">142</td>
<td colspan="2" style="text-align: center;">28</td>
<td style="text-align: center;"></td>
</tr>
<tr>
<td colspan="2" style="text-align: center;">133 to 126</td>
<td colspan="2" style="text-align: center;">133</td>
<td colspan="2" style="text-align: center;">30</td>
<td style="text-align: center;"></td>
</tr>
<tr>
<td colspan="2" style="text-align: center;">125 to 118</td>
<td colspan="2" style="text-align: center;">125</td>
<td colspan="2" style="text-align: center;">32</td>
<td style="text-align: center;"></td>
</tr>
<tr>
<td colspan="2" style="text-align: center;">117 to 112</td>
<td colspan="2" style="text-align: center;">117</td>
<td colspan="2" style="text-align: center;">34</td>
<td style="text-align: center;"></td>
</tr>
<tr>
<td colspan="2" style="text-align: center;">111 to 106</td>
<td colspan="2" style="text-align: center;">111</td>
<td colspan="2" style="text-align: center;">36</td>
<td style="text-align: center;"></td>
</tr>
<tr>
<td colspan="2" style="text-align: center;">105 to 101</td>
<td colspan="2" style="text-align: center;">105</td>
<td colspan="2" style="text-align: center;">38</td>
<td style="text-align: center;"></td>
</tr>
<tr>
<td colspan="2" style="text-align: center;">100 to 96</td>
<td colspan="2" style="text-align: center;">100</td>
<td colspan="2" style="text-align: center;">40</td>
<td style="text-align: center;"></td>
</tr>
<tr>
<td colspan="2" style="text-align: center;">95 to 91</td>
<td colspan="2" style="text-align: center;">95</td>
<td colspan="2" style="text-align: center;">42</td>
<td style="text-align: center;"></td>
</tr>
<tr>
<td colspan="2" style="text-align: center;">90</td>
<td colspan="2" style="text-align: center;">90</td>
<td colspan="2" style="text-align: center;">44</td>
<td style="text-align: center;"></td>
</tr>
</tbody>
</table>

Byte 4 and 5

> The Y Resolution field specifies the resolution in the base line
> direction.
>
> This field is not referred to because the X-axis resolution and the
> Y-axis resolution are the same in this unit.
>
> This field is ignored in the SET WINDOW command. The same value as the
> X-axis resolution is returned in the GET WINDOW command.
>
> If the X-axis resolution is changed, the Y-axis resolution also
> changes according to it.

Byte 6 to 9

> The Upper Left X Offset field specifies the X-axis coordinate of the
> upper left corner of the window. This coordinate is shown by the value
> of the distance (in inch) from the left end of the object multiplied
> by the specified unit divisor. It is possible to obtain the range that
> can be set in this unit by the Inquiry command.
>
> The default is 0.

Byte 10 to 13

> The Upper Left Y Offset field specifies the Y-axis coordinate of the
> upper left corner of the window. This coordinate is shown by the value
> of the distance (in inch) from the top end of the object multiplied by
> the specified unit divisor. It is possible to obtain the range that
> can be set in this unit by the Inquiry command.
>
> The default is 0.

\[Notes\]

> This unit obtains the number of scanning pixels from the scanning
> range and the resolution that are set by the SET WINDOW command by
> using the formulas below.
>
> The X offset specified by the initiator is X_off, the Y offset is
> Y_off, the X-axis resolution is Xdpi, the window width is W, and the
> window length is L. All variables are integers.

1.  When the unit divisor is the maximum resolution

> Scanning pitch P = (Maximum resolution/Xdpi) The fractions are rounded
> off.
>
> The number of X-axis pixels = W/P The fractions are rounded off.
>
> The number of Y-axis lines = L/P The fractions are rounded off.

2.  When the unit divisor is 1200

> X-axis scanning start pixel = (X_offset\*Maximum resolution)/1200 The
> fractions are rounded off.
>
> Y-axis scanning start line = (Y_offset\*Maximum resolution)/1200 The
> fractions are rounded off.
>
> Scanning pitch P = (Maximum resolution/Xdpi) The fractions are rounded
> off.
>
> The number of X-axis pixels = (W\*Maximum resolution)/(1200\*P) The
> fractions are rounded off.
>
> The number of Y-axis lines = (L\*Maximum resolution)/(1200\*P) The
> fractions are rounded off.

Byte 14 to 17

> The Window Width field specifies the width of the window in the scan
> line direction. The window width in inch multiplied by the specified
> unit divisor makes this value. The default is the maximum window width
> divided by the specified unit divisor. The maximum window width of
> this unit can be obtained by the Inquiry command.

Byte 18 to 21

> The Window Length field specifies the length of the window in the base
> line direction. The window length in inch multiplied by the specified
> unit divisor makes this value. The default is the maximum window
> length divided by the specified unit divisor. The maximum window
> length of this unit can be obtained by the Inquiry command.

Byte 25

> The Image Composition field specifies the type of the image. The image
> composition is shown in table 2-10-6.
>
> The default is 02h.

Table 2-10-6 Image composition code

<table>
<colgroup>
<col style="width: 22%" />
<col style="width: 59%" />
<col style="width: 18%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: center;">Code</td>
<td style="text-align: center;">Descriptions</td>
<td style="text-align: center;">Support</td>
</tr>
<tr>
<td style="text-align: center;"><p>00h</p>
<p>01h</p>
<p>02h</p>
<p>03h</p>
<p>04h</p>
<p>05h</p>
<p>06h to FFh</p></td>
<td style="text-align: center;"><p>Bi-level black &amp; white</p>
<p>Dithered/halftone black &amp; white</p>
<p>Multi-level black &amp; white</p>
<p>Bi-level RGB color</p>
<p>Dithered/halftone RGB color</p>
<p>Multi-level RGB color</p>
<p>Reserved</p></td>
<td style="text-align: center;"><p>No</p>
<p>No</p>
<p>Yes</p>
<p>No</p>
<p>No</p>
<p>Yes</p></td>
</tr>
</tbody>
</table>

Byte 26

> The Pixel Composition field specifies the number of bits used to
> represent the intensity of one pixel.
>
> 16d is set in this unit.

Byte 29

> The padding bit specifies how this unit shall pad the image data
> transferred to the initiator if it is not an integral number of bytes.
>
> This bit is set to 0.

Byte 30 and 31

> The Bit Ordering field is set to 0.

Byte 32

> The Compression Type and the Compression Argument fields are both set
> to 0. The compression is not performed in this unit.

Byte 40

- Multiple Reading Number

> This field specifies whether the multiple reading is performed or not
> and how many times one line is scanned in the multiple reading. The
> number of scanning times per one line is (the value that is set in
> this field + 1).
>
> If 0 is set in this field, the number of scanning times per one line
> becomes one and the normal scanning instead of the multiple reading is
> performed.
>
> The default is 0. The currently set value is specified for the GET
> WINDOW command.

- Color Ordering

> This field specifies the order for reading the color specified in this
> window.
>
> Setting this field to 0 specifies the default color ordering in the
> unit. The default color ordering means R=1, G=2, and B=3.
>
> If the overlapped setting of a value other than 0 is performed in this
> field for two or more windows that are set when the SCAN command is
> received, or if 0 and a value other than 0 are set in this field, the
> sense data is set as shown in the table below and the SCAN command is
> terminated with the CHECK CONDITION status.
>
> The currently set value is specified for the GET WINDOW command.

<table>
<colgroup>
<col style="width: 43%" />
<col style="width: 32%" />
<col style="width: 23%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: center;">Status</td>
<td style="text-align: center;">Sense data</td>
<td style="text-align: center;">Remarks</td>
</tr>
<tr>
<td style="text-align: center;">In the case that the overlapped setting
of a value other than 0 is performed in this field for two or more
windows that are set when the SCAN command is received, and that 0 and a
value other than 0 are set in this field</td>
<td style="text-align: center;"><p>INVALID COMBINATION OF WINDOWS
SPECIFIED</p>
<p>05h-2Ch-02h-00h</p></td>
<td style="text-align: center;">The command is terminated with the CHECK
CONDITION status.</td>
</tr>
</tbody>
</table>

Byte 41

- Averaging

> When 1 is set in this field, the averaging in the main scanning
> direction is performed. When 0 is set in this field, the averaging is
> not performed.
>
> The default is 0. This field specifies the currently set value for the
> GET WINDOW command.

- Setup Mode

> This field is valid when Setup Scan2 of the Scanning Kind byte on the
> Set Window function page in Inquiry is supported. The Setup mode can
> be specified with a value shown in Number of setup mode on the Set
> Window function page in Inquiry set as the upper limit.

- Negative/Positive

> When 1 is set in this field, the medium is positive. When 0 is set in
> this field, the medium is negative.
>
> The default is 1. This field specifies the currently set value for the
> GET WINDOW command.

Byte 42

> A bit whose kind of scanning is specified should be set to 1.
>
> The scanning kinds that are supported by this unit can be obtained by
> the Inquiry command.
>
> The default is 1. This field specifies the currently set value for the
> GET WINDOW command.
>
> When the thumbnail scanning is set, this unit terminates the first
> SCAN command after the SET WINDOW command with the CHECK CONDITION
> status and sets ‘Thumbnail created by driver’ in the sense data. The
> initiator can read the parameter (Cooperation parameter) for creating
> the thumbnail by the READ command. This unit executes reading at the
> second SCAN command.

|               |      |                        |
|:--------------|-----:|-----------------------:|
| Scanning Kind |      |                        |
|               | Bit0 |         Image Scanning |
|               | Bit1 |     Thumbnail Scanning |
|               | Bit2 |        Set up Scanning |
|               | Bit3 |       Set up Scanning2 |
|               | Bit4 |         Reserved \[0\] |
|               | Bit5 | Auto Exposure Scanning |
|               | Bit6 |    AE with WB Scanning |
|               | Bit7 |         Reserved \[0\] |

Byte 43

> A bit whose mode is specified should be set to 1.
>
> This unit supports the Normal Quality Scan, Multiple Reading Scan, and
> Reverse direction Scanning.
>
> If Reverse direction Scanning is specified with Normal Quality Scan or
> Multiple Reading Scan, the normal scanning and the reverse-direction
> scanning are performed for the sub-scanning direction.
>
> The default is 2. This field specifies the currently set value for the
> GET WINDOW command.

|                   |      |                            |
|:------------------|-----:|---------------------------:|
| Scan Mode Support |      |                            |
|                   | Bit0 |          High Quality Scan |
|                   | Bit1 |        Normal Quality Scan |
|                   | Bit2 |            High Speed Scan |
|                   | Bit3 |             Reserved \[0\] |
|                   | Bit4 |      Multiple Reading Scan |
|                   | Bit5 |             Reserved \[0\] |
|                   | Bit6 | Reverse direction Scanning |
|                   | Bit7 |             Reserved \[0\] |

Byte 44

> This field specifies which ordering (pixel ordering, line ordering, or
> plane ordering) shall be used for reading. It also specifies whether
> the X and Y offsets include the CCD distance for the pixel ordering
> and the line ordering. A bit whose ordering is specified for reading
> is set to 1.
>
> This field specifies the currently set value for the GET WINDOW
> command.

|      |                                 |
|:-----|:--------------------------------|
| Bit0 | Pixel without CCD distance      |
| Bit1 | Line without CCD distance       |
| Bit2 | Plane                           |
| Bit3 | Reserved \[0\]                  |
| Bit4 | Pixel with CCD distance         |
| Bit5 | Line with CCD distance          |
| Bit6 | Multi line Simultaneous reading |
| Bit7 | Reserved \[0\]                  |

Byte 45

> This field specifies the adjustment target value (AE Value) when the
> exposure adjustment (AE) is performed. If Auto Exposure Scanning or AE
> with WB Scanning is set in byte 42, adjustment is performed so that
> the output value becomes the specified value when AE is executed.
>
> The default in this unit is 255d. This field specifies the currently
> set value for the GET WINDOW command.
>
> When the parameter is 0, the default value (255d) is set. This value
> is also returned for the GET WINDOW command.

Byte 46 to 49

> This field sets a value corresponding to the exposure time in units of
> 10 nsec.
>
> In this unit, the analog gain and the exposure time are set according
> to this value.
>
> This field specifies the currently set value for the GET WINDOW
> command.
>
> The range of the value that can be set is from 0 to 3FFFFFFh. When 0
> is set, the value that is decided in the unit is returned.

**2-11. READ Command**

Table 2-11-1 READ command

<table>
<colgroup>
<col style="width: 13%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 11%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: right;"><p>Bit</p>
<p>Byte</p></td>
<td style="text-align: center;">7</td>
<td style="text-align: center;">6</td>
<td style="text-align: center;">5</td>
<td style="text-align: center;">4</td>
<td style="text-align: center;">3</td>
<td style="text-align: center;">2</td>
<td style="text-align: center;">1</td>
<td style="text-align: center;">0</td>
</tr>
<tr>
<td style="text-align: center;">0</td>
<td colspan="8" style="text-align: center;">Operation code [28h]</td>
</tr>
<tr>
<td style="text-align: center;">1</td>
<td colspan="3" style="text-align: center;"><p>Logical unit number</p>
<p>[0]</p></td>
<td colspan="5" style="text-align: center;"><p>Reserved</p>
<p>[0]</p></td>
</tr>
<tr>
<td style="text-align: center;">2</td>
<td colspan="8" style="text-align: center;">Data type code</td>
</tr>
<tr>
<td style="text-align: center;">3</td>
<td colspan="8" style="text-align: center;">Reserved [0]</td>
</tr>
<tr>
<td style="text-align: center;">4</td>
<td colspan="8" style="text-align: center;">Data type qualifier (upper
byte)</td>
</tr>
<tr>
<td style="text-align: center;">5</td>
<td colspan="8" style="text-align: center;">Data type qualifier (lower
byte)</td>
</tr>
<tr>
<td style="text-align: center;">6</td>
<td colspan="8" style="text-align: left;">(MSB)</td>
</tr>
<tr>
<td style="text-align: center;">7</td>
<td colspan="8" style="text-align: center;">Transfer length</td>
</tr>
<tr>
<td style="text-align: center;">8</td>
<td colspan="8" style="text-align: right;">(LSB)</td>
</tr>
<tr>
<td style="text-align: center;">9</td>
<td style="text-align: center;"><p>Reserved</p>
<p>[0]</p></td>
<td colspan="7" style="text-align: center;">Control bit [0]</td>
</tr>
</tbody>
</table>

The READ command requests that this unit transfer the data to the
initiator.

- Data type code

> This is for distinguishing the types of data that is transferred
> between the initiator and this unit. The details are shown in table
> 2-11-2.

- Data type qualifier

> This provides a means to differentiate the data transfers of the same
> data type code except image data.
>
> The data type qualifier is shown in tables 2-11-3 and -4.
>
> In the following case, however, this unit responds with common error 1
> (INVALID FIELD IN CDB (5h-24h-00h-00h)).
>
> \- When the lower byte of the data type qualifier is different from
> the data length in bytes shown in table 2-11-2

- Transfer length

> This specifies the number of blocks of the data that is transferred by
> this unit to the initiator during the DATA IN phase.
>
> (1 block = 1 byte in this unit)
>
> The recommended values are as shown below.

<table>
<colgroup>
<col style="width: 28%" />
<col style="width: 71%" />
</colgroup>
<tbody>
<tr>
<td>Data type code</td>
<td>Recommended value (Refer to table 2-11-2.)</td>
</tr>
<tr>
<td>00h to 7Fh</td>
<td>Data length in bytes * the number of valid data</td>
</tr>
<tr>
<td>80h and after</td>
<td><p>Data length in bytes * the number of valid data + header length
in bytes</p>
<p>(In the case of the magnetic data, the magnetic data header is
included.)</p></td>
</tr>
</tbody>
</table>

Table 2-11-2 Data type code (common to READ/SEND)

<table>
<colgroup>
<col style="width: 10%" />
<col style="width: 31%" />
<col style="width: 13%" />
<col style="width: 14%" />
<col style="width: 19%" />
<col style="width: 11%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: center;">Code</td>
<td style="text-align: center;">Descriptions</td>
<td style="text-align: center;">Support by this system<sup>*1</sup></td>
<td style="text-align: center;">Length in bytes of each valid data</td>
<td style="text-align: center;"><p>Number of valid data</p>
<p>(Number of elements)</p></td>
<td style="text-align: center;">Header included or not</td>
</tr>
<tr>
<td style="text-align: center;">00h</td>
<td style="text-align: left;">Image</td>
<td style="text-align: center;">R</td>
<td style="text-align: center;">1 or 2</td>
<td style="text-align: center;">Variable</td>
<td style="text-align: center;">Not included</td>
</tr>
<tr>
<td style="text-align: center;">02h</td>
<td style="text-align: left;">Halftone mask</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
</tr>
<tr>
<td style="text-align: center;">03h</td>
<td style="text-align: left;">LUT</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
</tr>
<tr>
<td style="text-align: center;">80h</td>
<td style="text-align: left;">Histogram data</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
</tr>
<tr>
<td style="text-align: center;">81h</td>
<td style="text-align: left;">Maximum value</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">2</td>
<td style="text-align: center;">1</td>
<td style="text-align: center;">Included</td>
</tr>
<tr>
<td style="text-align: center;">82h</td>
<td style="text-align: left;">Matrix data</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
</tr>
<tr>
<td style="text-align: center;">83h</td>
<td style="text-align: left;">Filter data</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
</tr>
<tr>
<td style="text-align: center;">84h</td>
<td style="text-align: left;">Shading data</td>
<td style="text-align: center;">R/S</td>
<td style="text-align: center;">2</td>
<td style="text-align: center;">47352</td>
<td style="text-align: center;">Included</td>
</tr>
<tr>
<td style="text-align: center;">85h</td>
<td style="text-align: left;">Dark voltage correction data</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
</tr>
<tr>
<td style="text-align: center;">86h</td>
<td style="text-align: left;">Magnetic data</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
</tr>
<tr>
<td style="text-align: center;">87h</td>
<td style="text-align: left;">Initiator cooperative action
parameter</td>
<td style="text-align: center;">R</td>
<td style="text-align: center;">1</td>
<td style="text-align: center;">Variable</td>
<td style="text-align: center;">Included</td>
</tr>
<tr>
<td style="text-align: center;">88h</td>
<td style="text-align: left;">Boundary Information</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">4</td>
<td style="text-align: center;">Variable</td>
<td style="text-align: center;">Included</td>
</tr>
<tr>
<td style="text-align: center;">89h</td>
<td style="text-align: left;">Analog gamma</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
</tr>
<tr>
<td style="text-align: center;">8Ah</td>
<td style="text-align: left;">Analog gain</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
</tr>
<tr>
<td style="text-align: center;">8Bh</td>
<td style="text-align: left;">Digital gain</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
</tr>
<tr>
<td style="text-align: center;">8Ch</td>
<td style="text-align: left;">WB exposure value</td>
<td style="text-align: center;">R</td>
<td style="text-align: center;">4</td>
<td style="text-align: center;">1</td>
<td style="text-align: center;">Included</td>
</tr>
<tr>
<td style="text-align: center;">8Dh</td>
<td style="text-align: left;">Setup information</td>
<td style="text-align: center;">R</td>
<td style="text-align: center;">1, 2, or 4</td>
<td style="text-align: center;">Variable</td>
<td style="text-align: center;">Included</td>
</tr>
<tr>
<td style="text-align: center;">8Eh</td>
<td style="text-align: left;">Perforation information</td>
<td style="text-align: center;">R</td>
<td style="text-align: center;">1 or 2</td>
<td style="text-align: center;">Variable</td>
<td style="text-align: center;">Included</td>
</tr>
<tr>
<td style="text-align: center;">8Fh</td>
<td style="text-align: left;">Boundary Information Type2</td>
<td style="text-align: center;">R/S</td>
<td style="text-align: center;">1, 2, or 4</td>
<td style="text-align: center;">Variable</td>
<td style="text-align: center;">Included</td>
</tr>
<tr>
<td style="text-align: center;">90h</td>
<td style="text-align: left;">WB exposure value at the time of
shipment</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
</tr>
<tr>
<td style="text-align: center;">91h</td>
<td style="text-align: left;">CCD data</td>
<td style="text-align: center;">R</td>
<td style="text-align: center;">2</td>
<td style="text-align: center;">Variable</td>
<td style="text-align: center;">Included</td>
</tr>
<tr>
<td style="text-align: center;">92h</td>
<td style="text-align: left;">Driver software version information</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
</tr>
<tr>
<td style="text-align: center;">93h</td>
<td style="text-align: left;">Leak volume</td>
<td style="text-align: center;">R</td>
<td style="text-align: center;">2</td>
<td style="text-align: center;">3</td>
<td style="text-align: center;">Included</td>
</tr>
<tr>
<td style="text-align: center;">94h-DFh</td>
<td style="text-align: left;">Reserved</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
</tr>
<tr>
<td style="text-align: center;">E0h</td>
<td style="text-align: left;">Initiator RAM buffer</td>
<td style="text-align: center;">R/S</td>
<td style="text-align: center;">1, 2, or 4</td>
<td style="text-align: center;">Variable (max 1 KB)</td>
<td style="text-align: center;">Included</td>
</tr>
<tr>
<td style="text-align: center;">E1h</td>
<td style="text-align: left;">Initiator EEPROM buffer</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
</tr>
</tbody>
</table>

> \*1 R means that the code is supported only for the READ command. R/S
> means that the code is supported for both the READ and the SEND
> commands.
>
> \*2 The valid number of pixels for CCD is 3946d.

Table 2-11-3 Data type qualifier (upper byte)

<table>
<colgroup>
<col style="width: 29%" />
<col style="width: 13%" />
<col style="width: 57%" />
</colgroup>
<tbody>
<tr>
<td></td>
<td>Code</td>
<td>Descriptions</td>
</tr>
<tr>
<td>When the data type code is 03h, 80h, 81h, 84h, 85h, 8Ch, 8Dh, or
91h</td>
<td><p>00h</p>
<p>01h</p>
<p>02h</p>
<p>03h</p></td>
<td><p>Default color (G-component element)</p>
<p>R-component element</p>
<p>G-component element</p>
<p>B-component element</p></td>
</tr>
<tr>
<td>Case other than the above</td>
<td>**h</td>
<td>No meaning</td>
</tr>
</tbody>
</table>

Table 2-11-4 Data type qualifier (lower byte)

<table>
<colgroup>
<col style="width: 28%" />
<col style="width: 71%" />
</colgroup>
<tbody>
<tr>
<td>Code</td>
<td>Descriptions</td>
</tr>
<tr>
<td><p>00h</p>
<p>01h</p>
<p>02h</p>
<p>03h</p>
<p>04h and after</p></td>
<td><p>1-byte data</p>
<p>2-byte data</p>
<p>Reserved</p>
<p>4-byte data</p>
<p>Reserved</p></td>
</tr>
</tbody>
</table>

When this unit sends data that is smaller than the transfer length, the
CHECK CONDITION status is returned. Set the ILI bit to 1, valid bit to
1, and information byte to the difference between the requested transfer
length and the number of blocks actually transferred. For image data
transfer, the response shall be made in the same manner.

When the data type code is other than 00h, the data is transferred again
starting from the top of the data (the top of the header, if any).

This command terminates with the RESERVATION CONFLICT status when there
is a reserved access conflict and there is no data to be transferred.

The sense data set in each status is shown in the table below.

Table 2-11-5 Sense data set in each status

<table>
<colgroup>
<col style="width: 38%" />
<col style="width: 36%" />
<col style="width: 25%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: center;">Status</td>
<td style="text-align: center;">Sense data</td>
<td style="text-align: center;">Remarks</td>
</tr>
<tr>
<td style="text-align: center;"><ul>
<li><p>When the READ command of the image is received without receiving
the SCAN command</p></li>
<li><p>When the READ command is received after all image data is
transferred</p></li>
</ul></td>
<td style="text-align: center;"><p>COMMAND SEQUENCE ERROR</p>
<p>(A command that makes the previous SCAN command invalid is received
while the scanning operation is valid)</p>
<p>05h-2Ch-00h-00h</p></td>
<td style="text-align: center;">The command terminates with the CHECK
CONDITION status.</td>
</tr>
</tbody>
</table>

**2-11-1. 2-byte data transfer**

For the 2-byte data, upper byte and lower byte are transferred, in that
order. For the data of three bytes or more, the upper byte, the middle
byte, and the lower byte are transferred, in that order.

**2-11-2. Data header**

If the data type code is over 80h, the following READ data header is
added at the top of the valid data.

Table 2-11-6 READ data header

<table style="width:100%;">
<colgroup>
<col style="width: 13%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: right;"><p>Bit</p>
<p>Byte</p></td>
<td style="text-align: center;">7</td>
<td style="text-align: center;">6</td>
<td style="text-align: center;">5</td>
<td style="text-align: center;">4</td>
<td style="text-align: center;">3</td>
<td style="text-align: center;">2</td>
<td style="text-align: center;">1</td>
<td style="text-align: center;">0</td>
</tr>
<tr>
<td style="text-align: center;">0</td>
<td colspan="8" style="text-align: center;">Data type code</td>
</tr>
<tr>
<td style="text-align: center;">1</td>
<td colspan="8" style="text-align: center;">The number of bits in each
valid data</td>
</tr>
<tr>
<td style="text-align: center;"></td>
<td colspan="8" style="text-align: left;">(MSB)</td>
</tr>
<tr>
<td style="text-align: center;">2 to 5</td>
<td colspan="8" style="text-align: center;">Valid data length</td>
</tr>
<tr>
<td style="text-align: center;"></td>
<td colspan="8" style="text-align: right;">(LSB)</td>
</tr>
</tbody>
</table>

\- Data type code

> This field specifies the type of data, and the same value as the data
> type for the READ command is set.

\- The number of bits in each valid data

> This field specifies the length in bits of the valid data to be
> transferred.
>
> For example, in the case of the 14-bit data, the transferred data is 2
> bytes (=16 bits). However, because the valid data is 14 bits, 0Eh
> (=14d) is returned for this byte.

\- Valid data length

> This field specifies the length in bytes of the valid data that is
> currently retained in the unit. The valid data length that is handled
> as the standard in this unit is the length in bytes of each valid data
> multiplied by the number of valid data in table 2-11-2.
>
> In this unit, when the number of valid data is the fixed value, the
> length in bytes of each valid data multiplied by the number of valid
> data is set for the valid data length. When the number of valid data
> is variable, the length in bytes of each valid data multiplied by the
> number of valid data that is currently retained is set.
>
> Even if the data transfer length for the READ command is not enough to
> transfer all the valid data, the valid data length is not adjusted.

**2-11-3. Image data transfer**

The image data output of this unit is 16 bits.

Each data consists of 2 bytes and the data transfer is performed by
setting 8 bits on the MSB side in the first byte, then the remaining 8
bits are transferred.

When the thumbnail scanning is performed for the strip film, the length
is also measured at the same time. Therefore there is no way for the
initiator to check when the end of the film comes. So it is possible
that the data becomes smaller than the transfer length. In this case,
the CHECK CONDITION status shall be returned. Set the ILI bit to 1,
valid bit to 1, and information byte to the difference between the
requested transfer length and the number of blocks actually transferred.
After that, the following sense data shall be returned when the next
READ command is received.

<table>
<colgroup>
<col style="width: 30%" />
<col style="width: 45%" />
<col style="width: 23%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: center;">Status</td>
<td style="text-align: center;">Sense data</td>
<td style="text-align: center;">Remarks</td>
</tr>
<tr>
<td style="text-align: center;">When the READ command is received after
all the image data is transferred</td>
<td style="text-align: center;"><p>COMMAND SEQUENCE ERROR</p>
<p>(A command that makes the previous SCAN command invalid is received
while the scanning operation is valid)</p>
<p>05h-2Ch-00h-00h</p></td>
<td style="text-align: center;">The command terminates with the CHECK
CONDITION status.</td>
</tr>
</tbody>
</table>

**Precautions:**

As shown in Byte 4 “SCSI function support” of 2-2-2-3. “Address
information page”, the image reading is performed in units of \[Data of
one line in bytes \* number of colors\]. Therefore the image data size
is calculated by the following formula.

Image data length

= (Window width/Scanning pitch) x Number of lines x Number of colors +
Invalid data

Remarks): The invalid data is the number of bytes set in TRUNCATED BY
DRIVER TYPE2.

Note that a communication error may occur when the value of the data
length in the READ command differs from that of the actual image data
length because the error processing is not executed.

**2-11-3-1. Image data format**

The format of the transferred data changes variously depending on the
setting for the ASIC during the scanning.

Some of the examples of the one-line data formats in the typical setting
are shown below. The data in the following format is repeated as many
times as the number of scanning lines (sub-scanning).

1)  When the three colors are output in the order of R, G, and B in line
    ordering and transferred for one-line reading

<table>
<colgroup>
<col style="width: 6%" />
<col style="width: 6%" />
<col style="width: 6%" />
<col style="width: 6%" />
<col style="width: 6%" />
<col style="width: 6%" />
<col style="width: 6%" />
<col style="width: 6%" />
<col style="width: 6%" />
<col style="width: 6%" />
<col style="width: 6%" />
<col style="width: 6%" />
<col style="width: 6%" />
<col style="width: 6%" />
<col style="width: 6%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: center;"><p>R1</p>
<p>H</p></td>
<td style="text-align: center;"><p>R1</p>
<p>L</p></td>
<td style="text-align: center;">…</td>
<td style="text-align: center;"><p>Rn</p>
<p>H</p></td>
<td style="text-align: center;"><p>Rn</p>
<p>L</p></td>
<td style="text-align: center;"><p>G1</p>
<p>H</p></td>
<td style="text-align: center;"><p>G1</p>
<p>L</p></td>
<td style="text-align: center;">…</td>
<td style="text-align: center;"><p>Gn</p>
<p>H</p></td>
<td style="text-align: center;"><p>Gn</p>
<p>L</p></td>
<td style="text-align: center;"><p>B1</p>
<p>H</p></td>
<td style="text-align: center;"><p>B1</p>
<p>L</p></td>
<td style="text-align: center;">…</td>
<td style="text-align: center;"><p>Bn</p>
<p>H</p></td>
<td style="text-align: center;"><p>Bn</p>
<p>L</p></td>
</tr>
</tbody>
</table>

2)  When the three colors are output in the order of R, G, and B in line
    ordering and transferred for two-line reading

<table>
<colgroup>
<col style="width: 3%" />
<col style="width: 3%" />
<col style="width: 3%" />
<col style="width: 3%" />
<col style="width: 3%" />
<col style="width: 3%" />
<col style="width: 3%" />
<col style="width: 3%" />
<col style="width: 3%" />
<col style="width: 3%" />
<col style="width: 3%" />
<col style="width: 3%" />
<col style="width: 3%" />
<col style="width: 3%" />
<col style="width: 3%" />
<col style="width: 3%" />
<col style="width: 3%" />
<col style="width: 3%" />
<col style="width: 3%" />
<col style="width: 3%" />
<col style="width: 3%" />
<col style="width: 3%" />
<col style="width: 3%" />
<col style="width: 3%" />
<col style="width: 3%" />
<col style="width: 3%" />
<col style="width: 3%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: center;"><p>A</p>
<p>R1</p>
<p>H</p></td>
<td style="text-align: center;"><p>A</p>
<p>R1</p>
<p>L</p></td>
<td style="text-align: center;"><p>B</p>
<p>R1</p>
<p>H</p></td>
<td style="text-align: center;"><p>B</p>
<p>R1</p>
<p>L</p></td>
<td style="text-align: center;">…</td>
<td style="text-align: center;"><p>A</p>
<p>Rn</p>
<p>H</p></td>
<td style="text-align: center;"><p>A</p>
<p>Rn</p>
<p>L</p></td>
<td style="text-align: center;"><p>B</p>
<p>Rn</p>
<p>H</p></td>
<td style="text-align: center;"><p>B</p>
<p>Rn</p>
<p>L</p></td>
<td style="text-align: center;"><p>A</p>
<p>G1</p>
<p>H</p></td>
<td style="text-align: center;"><p>A</p>
<p>G1</p>
<p>L</p></td>
<td style="text-align: center;"><p>B</p>
<p>G1</p>
<p>H</p></td>
<td style="text-align: center;"><p>B</p>
<p>G1</p>
<p>L</p></td>
<td style="text-align: center;">…</td>
<td style="text-align: center;"><p>A</p>
<p>Gn</p>
<p>H</p></td>
<td style="text-align: center;"><p>A</p>
<p>Gn</p>
<p>L</p></td>
<td style="text-align: center;"><p>B</p>
<p>Gn</p>
<p>H</p></td>
<td style="text-align: center;"><p>B</p>
<p>Gn</p>
<p>L</p></td>
<td style="text-align: center;"><p>A</p>
<p>B1</p>
<p>H</p></td>
<td style="text-align: center;"><p>A</p>
<p>B1</p>
<p>L</p></td>
<td style="text-align: center;"><p>B</p>
<p>B1</p>
<p>H</p></td>
<td style="text-align: center;"><p>B</p>
<p>B1</p>
<p>L</p></td>
<td style="text-align: center;">…</td>
<td style="text-align: center;"><p>A</p>
<p>Bn</p>
<p>H</p></td>
<td style="text-align: center;"><p>A</p>
<p>Bn</p>
<p>L</p></td>
<td style="text-align: center;"><p>B</p>
<p>Bn</p>
<p>H</p></td>
<td style="text-align: center;"><p>B</p>
<p>Bn</p>
<p>L</p></td>
</tr>
</tbody>
</table>

**2-11-4. Shading data format**

The shading data is represented by the 16-bit data.

The format is shown below.

<table style="width:100%;">
<colgroup>
<col style="width: 13%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: right;"><p>Bit</p>
<p>Byte</p></td>
<td style="text-align: center;">7</td>
<td style="text-align: center;">6</td>
<td style="text-align: center;">5</td>
<td style="text-align: center;">4</td>
<td style="text-align: center;">3</td>
<td style="text-align: center;">2</td>
<td style="text-align: center;">1</td>
<td style="text-align: center;">0</td>
</tr>
<tr>
<td style="text-align: center;">0</td>
<td colspan="8" style="text-align: left;">(MSB) Data for the first pixel
in gain 1, 2-line mode, CCD line A</td>
</tr>
<tr>
<td style="text-align: center;">1</td>
<td colspan="8" style="text-align: right;">(LSB)</td>
</tr>
<tr>
<td style="text-align: center;">2</td>
<td colspan="8" style="text-align: left;">(MSB) Data for the first pixel
in gain 1, 2-line mode, CCD line B</td>
</tr>
<tr>
<td style="text-align: center;">3</td>
<td colspan="8" style="text-align: right;">(LSB)</td>
</tr>
<tr>
<td style="text-align: center;">:</td>
<td colspan="8" style="text-align: center;">:</td>
</tr>
<tr>
<td style="text-align: center;">15782</td>
<td colspan="8" style="text-align: left;">(MSB) Data for the 3946th
pixel in gain 1, 2-line mode, CCD line B</td>
</tr>
<tr>
<td style="text-align: center;">15783</td>
<td colspan="8" style="text-align: right;">(LSB)</td>
</tr>
<tr>
<td style="text-align: center;">15784</td>
<td colspan="8" style="text-align: left;">(MSB) Data for the first pixel
in gain 1, 1-line mode, CCD line A</td>
</tr>
<tr>
<td style="text-align: center;">15785</td>
<td colspan="8" style="text-align: right;">(LSB)</td>
</tr>
<tr>
<td style="text-align: center;">:</td>
<td colspan="8" style="text-align: center;">:</td>
</tr>
<tr>
<td style="text-align: center;">23674</td>
<td colspan="8" style="text-align: left;">(MSB) Data for the 3946th
pixel in gain 1, 1-line mode, CCD line A</td>
</tr>
<tr>
<td style="text-align: center;">23675</td>
<td colspan="8" style="text-align: right;">(LSB)</td>
</tr>
<tr>
<td style="text-align: center;">23676</td>
<td colspan="8" style="text-align: left;">(MSB) Data for the first pixel
in gain 2, 2-line mode, CCD line A</td>
</tr>
<tr>
<td style="text-align: center;">23677</td>
<td colspan="8" style="text-align: right;">(LSB)</td>
</tr>
<tr>
<td style="text-align: center;">:</td>
<td colspan="8" style="text-align: center;">:</td>
</tr>
<tr>
<td style="text-align: center;">39458</td>
<td colspan="8" style="text-align: left;">(MSB) Data for the 3946th
pixel in gain 2, 2-line mode, CCD line B</td>
</tr>
<tr>
<td style="text-align: center;">39459</td>
<td colspan="8" style="text-align: right;">(LSB)</td>
</tr>
<tr>
<td style="text-align: center;">39460</td>
<td colspan="8" style="text-align: left;">(MSB) Data for the first pixel
in gain 2, 1-line mode, CCD line A</td>
</tr>
<tr>
<td style="text-align: center;">39461</td>
<td colspan="8" style="text-align: right;">(LSB)</td>
</tr>
<tr>
<td style="text-align: center;">:</td>
<td colspan="8" style="text-align: center;">:</td>
</tr>
<tr>
<td style="text-align: center;">47350</td>
<td colspan="8" style="text-align: left;">(MSB) Data for the 3946th
pixel in gain 2, 1-line mode, CCD line A</td>
</tr>
<tr>
<td style="text-align: center;">47351</td>
<td colspan="8" style="text-align: right;">(LSB)</td>
</tr>
</tbody>
</table>

Note) For the shading data measured with the 240 adapter attached, the
shading data corresponding to the range outside the aperture becomes
invalid, but this invalid data is also read at the same time in the
reading by the READ command.

**  
2-11-5. Initiator cooperative action parameter**

The READ command specifying data type code 87h is sent from the host
when one of the following scanning modes is selected: thumbnail
scanning, one-line multiple reading, or 8-bit odd-width reading (in the
scan line direction). When receiving this command, this unit sends the
data that conveys each information of the unit.

Because the valid data length becomes variable depending on the
operation type code, the host needs to read the data header once and
then the valid data length.

The contents and the format of the data are shown below.

Operation type code

|  |  |  |
|:--:|----|:---|
| 1 | THUMBNAIL CREATED BY DRIVER | Thumbnail scanning |
| 2 | AVERAGING MULTIPLE READING BY DRIVER | Line averaging of the multiple reading function |
| 6 | TRUNCATED BY DRIVER | Deletion of invalid data |
| 7 | CCD DATA CREATED BY DRIVER | CCD data reading |

Table 2-11-5-1 Format of THUMBNAIL CREATED BY DRIVER

<table>
<colgroup>
<col style="width: 9%" />
<col style="width: 22%" />
<col style="width: 28%" />
<col style="width: 39%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: center;">Byte</td>
<td style="text-align: center;">Name</td>
<td style="text-align: center;">Descriptions</td>
<td>Parameter</td>
</tr>
<tr>
<td style="text-align: center;">0</td>
<td style="text-align: center;">Type Code</td>
<td style="text-align: center;">Operation type code</td>
<td>1</td>
</tr>
<tr>
<td style="text-align: center;">1 to 4</td>
<td style="text-align: center;">Sense Data</td>
<td style="text-align: center;">Sense data that is set by the SCAN
command</td>
<td><p>09h-80h-01h-02h (IA)</p>
<p>09h-80h-01h-06h (SA)</p></td>
</tr>
<tr>
<td style="text-align: center;">5, 6</td>
<td style="text-align: center;">Bytes Per Line</td>
<td style="text-align: center;">The number of bytes per line</td>
<td>Depends on the scanning condition</td>
</tr>
<tr>
<td style="text-align: center;">7, 8</td>
<td style="text-align: center;">Entire Lines</td>
<td style="text-align: center;">The number of entire lines</td>
<td>Number of scanning lines*Number of frames</td>
</tr>
<tr>
<td style="text-align: center;">9</td>
<td style="text-align: center;">Bits Per a Color of Dot</td>
<td style="text-align: center;">The number of bits per dot of one
color</td>
<td>[16]</td>
</tr>
<tr>
<td style="text-align: center;">10, 11</td>
<td style="text-align: center;">Lines Per an Image</td>
<td style="text-align: center;">The number of lines per image</td>
<td>The number of scanning lines</td>
</tr>
<tr>
<td style="text-align: center;">12</td>
<td style="text-align: center;">Reading Count Per a Line</td>
<td style="text-align: center;">Exposure counts per line</td>
<td>-</td>
</tr>
<tr>
<td style="text-align: center;">13 to 17</td>
<td style="text-align: center;">Reserved</td>
<td style="text-align: center;">Reserved</td>
<td>0</td>
</tr>
</tbody>
</table>

Table 2-11-5-2 Format of AVERAGING MULTIPLE READING BY DRIVER

|  |  |  |  |
|:--:|:--:|:--:|----|
| Byte | Name | Descriptions | Parameter |
| 0 | Type Code | Operation type code | 2 |
| 1 to 4 | Sense Data | Sense data that is set by the SCAN command | 09h-80h-02h-00h |
| 5, 6 | Bytes Per Line | The number of bytes per line | \- |
| 7, 8 | Entire Lines | The number of entire lines | \- |
| 9 | Bits Per a Color of Dot | The number of bits per dot of one color | \- |
| 10, 11 | Lines Per an Image | The number of lines per image | \- |
| 12 | Reading Count Per a Line | Exposure counts per line | Depends on the scanning condition |
| 13 to 17 | Reserved | Reserved | 0 |

Table 2-11-5-3 Format of TRUNCATED BY DRIVER TYPE2

<table>
<colgroup>
<col style="width: 9%" />
<col style="width: 22%" />
<col style="width: 28%" />
<col style="width: 39%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: center;">Byte</td>
<td style="text-align: center;">Name</td>
<td style="text-align: center;">Descriptions</td>
<td>Parameter</td>
</tr>
<tr>
<td style="text-align: center;">0</td>
<td style="text-align: center;">Type Code</td>
<td style="text-align: center;">Operation type code (06h)</td>
<td>6</td>
</tr>
<tr>
<td style="text-align: center;">1 to 4</td>
<td style="text-align: center;">Sense Data</td>
<td style="text-align: center;"><p>Sense data that is set by the SCAN
command</p>
<p>(9h-80h-06h-01h)</p></td>
<td>09h-80h-06h-01h</td>
</tr>
<tr>
<td style="text-align: center;">5, 6</td>
<td style="text-align: center;">Invalid Data Position</td>
<td style="text-align: center;">Invalid data attaching position</td>
<td>Depends on the scanning condition</td>
</tr>
<tr>
<td style="text-align: center;">7, 8</td>
<td style="text-align: center;">Byte of invalid data of Left of each
color</td>
<td style="text-align: center;">Invalid data length in bytes that is
attached to the first-pixel side in the scan line direction with the
origin of the image in each color set to the standard</td>
<td>Depends on the scanning condition</td>
</tr>
<tr>
<td style="text-align: center;">9, 10</td>
<td style="text-align: center;">Byte of invalid data of Last of each
color</td>
<td style="text-align: center;">Invalid data length in bytes that is
attached to the last-pixel side in the scan line direction with the
origin of the image in each color set to the standard</td>
<td>Depends on the scanning condition</td>
</tr>
<tr>
<td style="text-align: center;">11, 12</td>
<td style="text-align: center;">Byte of invalid data of Left of all
color</td>
<td style="text-align: center;">Invalid data length in bytes that is
attached to the first-pixel side in the scan line direction with the
origin of the image in all colors set to the standard</td>
<td>Depends on the scanning condition</td>
</tr>
<tr>
<td style="text-align: center;">13, 14</td>
<td style="text-align: center;">Byte of invalid data of Last of all
color</td>
<td style="text-align: center;">Invalid data length in bytes that is
attached to the last-pixel side in the scan line direction with the
origin of the image in all colors set to the standard</td>
<td>Depends on the scanning condition</td>
</tr>
<tr>
<td style="text-align: center;">15, 16</td>
<td style="text-align: center;">Reserved</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
</tr>
<tr>
<td style="text-align: center;">17, 18</td>
<td style="text-align: center;">Reserved</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
</tr>
<tr>
<td style="text-align: center;">19, 20</td>
<td style="text-align: center;">Line of invalid data of Top</td>
<td style="text-align: center;">The number of invalid data lines that is
attached to the first-line side in the base line direction with the
origin of the image set to the standard</td>
<td>Depends on the scanning condition</td>
</tr>
<tr>
<td style="text-align: center;">21, 22</td>
<td style="text-align: center;">Line of invalid data of End</td>
<td style="text-align: center;">The number of invalid data lines that is
attached to the last-line side in the base line direction with the
origin of the image set to the standard</td>
<td>Depends on the scanning condition</td>
</tr>
<tr>
<td style="text-align: center;">23, 24</td>
<td style="text-align: center;">Byte of invalid data of Top of one
frame</td>
<td style="text-align: center;">Invalid data length in bytes that is
attached to the first-pixel side in the scan line direction with the
origin of the one-frame image data set to the standard</td>
<td>Depends on the scanning condition</td>
</tr>
<tr>
<td style="text-align: center;">25, 26</td>
<td style="text-align: center;">Byte of invalid data of End of one
frame</td>
<td style="text-align: center;">Invalid data length in bytes that is
attached to the last-pixel side in the scan line direction with the
origin of the one-frame image data set to the standard</td>
<td>Depends on the scanning condition</td>
</tr>
</tbody>
</table>

Byte 5 and 6 Invalid Data Position

> This field specifies the position to which the invalid data is
> attached. The invalid data is attached to the position of the bit to
> which 1 is set.

|  |  |  |
|----|----|----|
| Byte 5 | Bit0 | The first-pixel side in the scan line direction with theorigin of data in each color set to the standard |
|  | Bit1 | The last-pixel side in the scan line direction with theorigin of data in each color set to the standard |
|  | Bit2 | The first-pixel side in the scan line direction with theorigin of data in all colors set to the standard |
|  | Bit3 | The last-pixel side in the scan line direction with theorigin of data in all colors set to the standard |
|  | Bit4 | Reserved |
|  | Bit5 | Reserved |
|  | Bit6 | The first-line side in the base line direction with the origin set to the standard |
|  | Bit7 | The last-line side in the base line direction with the origin set to the standard |

|  |  |  |
|----|----|----|
| Byte 6 | Bit0 | The first-pixel side in the scan line direction with theorigin of one-frame image data set to the standard |
|  | Bit1 | The last-pixel side in the scan line direction with theorigin of one-frame image data set to the standard |
|  | Bit2 | Reserved |
|  | Bit3 | Reserved |
|  | Bit4 | Reserved |
|  | Bit5 | Reserved |
|  | Bit6 | Reserved |
|  | Bit7 | Reserved |

Byte 7 to 26 Byte of invalid data

> This field specifies the invalid data length in bytes that is attached
> to each position.

Some of the positions may be included in both the scan-line side and the
base-line side depending on the condition. In this case, it is handled
as a part of the line in the base line direction.

Table 2-11-5-4 Format of CCD DATA CREATED BY DRIVER

|  |  |  |  |
|:--:|:--:|:--:|----|
| Byte | Name | Descriptions | Parameter |
| 0 | Type Code | Operation type code | 7 |
| 1 to 4 | Sense Data | Sense data that is set by the SCAN command | 09h-80h-07h-00h |
| 5 | CCD Data Type of R Data | Type for CCD measurement of R color | Depends on the scanning condition |
| 6 | CCD Data Type of G Data | Type for CCD measurement of G color | Depends on the scanning condition |
| 7 | CCD Data Type of B Data | Type for CCD measurement of B color | Depends on the scanning condition |
| 8 | CCD Data Type of NG Data | Type for CCD measurement of NG color | \- |
| 9 | CCD Data Type of C Data | Type for CCD measurement of C color | \- |
| 10 | CCD Data Type of M Data | Type for CCD measurement of M color | \- |
| 11 | CCD Data Type of Y Data | Type for CCD measurement of Y color | \- |
| 12 | CCD Data Type of B Data | Type for CCD measurement of B color | \- |
| 13 to 17 | Reserved | Reserved | 0 |

Byte 5 to 12 CCD Data Type of color Data

> This field specifies the type that is used for the CCD measurement of
> each color.

**2-11-6. WB exposure value**

The value decided by the measurement of the unit at the time of start-up
specifies the color according to the upper byte of the data type
qualifier, and 4-byte data is sent for each color.

**  
2-11-7. Setup information**

<table>
<colgroup>
<col style="width: 13%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: right;"><p>Bit</p>
<p>Byte</p></td>
<td style="text-align: center;">7</td>
<td style="text-align: center;">6</td>
<td style="text-align: center;">5</td>
<td style="text-align: center;">4</td>
<td style="text-align: center;">3</td>
<td style="text-align: center;">2</td>
<td style="text-align: center;">1</td>
<td style="text-align: center;">0</td>
</tr>
<tr>
<td style="text-align: center;">0, 1</td>
<td colspan="8" style="text-align: center;">(MSB) Parameter length [n-2]
(LSB)</td>
</tr>
<tr>
<td style="text-align: center;">2</td>
<td colspan="8" style="text-align: center;">Format Identifier [0]</td>
</tr>
<tr>
<td style="text-align: center;">3, 4</td>
<td colspan="8" style="text-align: center;">Base Level (Base level value
of the film)</td>
</tr>
<tr>
<td style="text-align: center;">5 to 8</td>
<td colspan="8" style="text-align: center;"><p>Exposure Value for Base
Level</p>
<p>(Exposure value when the base level value of the film is
decided)</p></td>
</tr>
<tr>
<td style="text-align: center;">9 to 12</td>
<td colspan="8" style="text-align: center;"><p>Exposure Value for White
balance at base measurement</p>
<p>(Exposure value for white balance when the base level value of the
film is decided)</p></td>
</tr>
<tr>
<td style="text-align: center;">13</td>
<td colspan="8" style="text-align: center;">The number of information
retaining images</td>
</tr>
<tr>
<td style="text-align: center;">14</td>
<td colspan="8" style="text-align: center;">1st Index (The first
image)</td>
</tr>
<tr>
<td style="text-align: center;">15 to 18</td>
<td colspan="8" style="text-align: center;"><p>Exposure Value for 1st
index image</p>
<p>(Exposure value after prescan of the first image)</p></td>
</tr>
<tr>
<td style="text-align: center;">19 to 22</td>
<td colspan="8" style="text-align: center;"><p>Exposure Value for White
balance at 1st image setup</p>
<p>(Exposure value for white balance in the prescan of the first
image)</p></td>
</tr>
<tr>
<td style="text-align: center;">23, 24</td>
<td colspan="8" style="text-align: center;"><p>Minimum Level for the 1st
index image</p>
<p>(Minimum level of the image detected after prescan of the first
image)</p></td>
</tr>
<tr>
<td style="text-align: center;">25, 26</td>
<td colspan="8" style="text-align: center;"><p>Maximum Level for the 1st
index image</p>
<p>(Maximum level of the image detected after prescan of the first
image)</p></td>
</tr>
<tr>
<td style="text-align: center;">27</td>
<td colspan="8" style="text-align: center;">2nd Index (The second
image)</td>
</tr>
<tr>
<td style="text-align: center;">28 to 31</td>
<td colspan="8" style="text-align: center;"><p>Exposure Value for 2nd
index image</p>
<p>(Exposure value after prescan of the 2nd image)</p></td>
</tr>
<tr>
<td style="text-align: center;">32 to 35</td>
<td colspan="8" style="text-align: center;"><p>Exposure Value for White
balance at 2nd image setup</p>
<p>(Exposure value for white balance in the prescan of the 2nd
image)</p></td>
</tr>
<tr>
<td style="text-align: center;">36, 37</td>
<td colspan="8" style="text-align: center;"><p>Minimum Level for the 2nd
index image</p>
<p>(Minimum level of the image detected after prescan of the 2nd
image)</p></td>
</tr>
<tr>
<td style="text-align: center;">38, 39</td>
<td colspan="8" style="text-align: center;"><p>Maximum Level for the 2nd
index image</p>
<p>(Maximum level of the image detected after prescan of the 2nd
image)</p></td>
</tr>
<tr>
<td style="text-align: center;">:</td>
<td colspan="8" style="text-align: center;">:</td>
</tr>
<tr>
<td style="text-align: center;">n-3, n-2</td>
<td colspan="8" style="text-align: center;"><p>Minimum Level for the
last index image</p>
<p>(Minimum level of the image detected after prescan of the last
image)</p></td>
</tr>
<tr>
<td style="text-align: center;">n-1, n</td>
<td colspan="8" style="text-align: center;"><p>Maximum Level for the
last index image</p>
<p>(Maximum level of the image detected after prescan of the last
image)</p></td>
</tr>
</tbody>
</table>

Byte 2 Format Identifier

Identifier for specifying the format type in byte 3 and after. This must
be 0 under the present condition.

Byte 3, 4 Base Level

Base level value of the film. It is represented by the number of A/D
bits in the unit with the lower-bit aligned.

Byte 5 to 8 Exposure Value for Base Level

Exposure value when the base level of the film is decided.

Byte 9 to 12 Exposure Value for White balance at base measurement

Exposure value for white balance when the base level of the film is
decided

Byte 13 The number of information retaining images

For the number of films that retain the information, when the
information of only one frame can be retained depending on the condition
of the scanner, the number of information retaining images is set to 1.

Byte 14 and after

This field specifies the Setup information for each image. This unit
does not support it.

nth Index

Index value of the film. Index is assigned in order of lower number of
the Y address. If images are placed in both X and Y directions, Y
address has advantage to assign the index. The information of images are
generated with this index every time prescan is done. Therefore the
index values are not necessarily sequential numbers.

Exposure Value for nth index image

> Exposure value that is decided as a result of prescan of the nth
> image.

Exposure Value for White balance at nth image setup

Exposure value for white balance when the prescan of the nth image is
performed.

Minimum Level for the last index image

Minimum level of the image that is detected as a result of the prescan
of the nth image.

Maximum Level for the last index image

Maximum level of the image that is detected as a result of the prescan
of the nth image.

**  
2-11-8. Perforation information**

After the thumbnail scanning of the strip film, the READ command
specifying data type code 8Eh is sent from the initiator again. This
unit receives this command, and transfers the data for the number of
lines between each perforation.

The contents of the data and the format are as shown below.

<table style="width:100%;">
<colgroup>
<col style="width: 12%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: center;"><p>Bit</p>
<p>Byte</p></td>
<td style="text-align: center;">7</td>
<td style="text-align: center;">6</td>
<td style="text-align: center;">5</td>
<td style="text-align: center;">4</td>
<td style="text-align: center;">3</td>
<td style="text-align: center;">2</td>
<td style="text-align: center;">1</td>
<td style="text-align: center;">0</td>
</tr>
<tr>
<td style="text-align: center;">0 to 2</td>
<td colspan="8" style="text-align: center;">(MSB) Parameter length
[4n+1]</td>
</tr>
<tr>
<td style="text-align: center;"></td>
<td colspan="8" style="text-align: right;">(LSB)</td>
</tr>
<tr>
<td style="text-align: center;">3</td>
<td colspan="8" style="text-align: center;"><p>Bytes per parameter (The
number of bytes in each line of the absolute position information)</p>
<p>[4]</p></td>
</tr>
<tr>
<td style="text-align: center;">4, 5</td>
<td colspan="8" style="text-align: center;"><p>(MSB) Perforation number
for the 1st line</p>
<p>(LSB)</p></td>
</tr>
<tr>
<td style="text-align: center;">6</td>
<td style="text-align: center;"><p>Count switching flag</p>
<p>[0, 1]</p></td>
<td colspan="7" style="text-align: center;">Number of Pattern for the
1st line</td>
</tr>
<tr>
<td style="text-align: center;">7</td>
<td colspan="8" style="text-align: center;">Pulse number for the 1st
line</td>
</tr>
<tr>
<td style="text-align: center;">:</td>
<td colspan="8" style="text-align: center;">:</td>
</tr>
<tr>
<td style="text-align: center;"><p>4n,</p>
<p>4n+1</p></td>
<td colspan="8" style="text-align: center;"><p>(MSB) Perforation number
for the nth line</p>
<p>(LSB)</p></td>
</tr>
<tr>
<td style="text-align: center;">4n+2</td>
<td style="text-align: center;"><p>Count switching flag</p>
<p>[0, 1]</p></td>
<td colspan="7" style="text-align: center;">Number of Pattern for the
nth line</td>
</tr>
<tr>
<td style="text-align: center;">4n+3</td>
<td colspan="8" style="text-align: center;">Pulse number for the nth
line</td>
</tr>
</tbody>
</table>

Byte 3 Bytes per parameter

> This field specifies the number of bytes in each line of the absolute
> position information.

Byte 4 and after Perforation Address for the nth line

> This field specifies the absolute position information of the nth line
> of the thumbnail.
>
> \- Count switching flag
>
> This is the flag to be set when the sensor that counts the perforation
> is switched.

**  
2-11-9. Boundary Information Type2**

<table>
<colgroup>
<col style="width: 13%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: right;"><p>Bit</p>
<p>Byte</p></td>
<td style="text-align: center;">7</td>
<td style="text-align: center;">6</td>
<td style="text-align: center;">5</td>
<td style="text-align: center;">4</td>
<td style="text-align: center;">3</td>
<td style="text-align: center;">2</td>
<td style="text-align: center;">1</td>
<td style="text-align: center;">0</td>
</tr>
<tr>
<td style="text-align: center;">0, 1</td>
<td colspan="8" style="text-align: center;">(MSB) Parameter length [n-1]
(LSB)</td>
</tr>
<tr>
<td style="text-align: center;">2</td>
<td colspan="8" style="text-align: center;">The actual number of
images</td>
</tr>
<tr>
<td style="text-align: center;">3</td>
<td colspan="8" style="text-align: center;">Reserved [0]</td>
</tr>
<tr>
<td style="text-align: center;">4 to 7</td>
<td colspan="8" style="text-align: center;">1st image Top (Y)
address</td>
</tr>
<tr>
<td style="text-align: center;">8, 9</td>
<td colspan="8" style="text-align: center;">1st image Perforation
number</td>
</tr>
<tr>
<td style="text-align: center;">10</td>
<td colspan="8" style="text-align: center;">1st image Perforation
decimal</td>
</tr>
<tr>
<td style="text-align: center;">11</td>
<td colspan="8" style="text-align: center;">1st image Pulse number</td>
</tr>
<tr>
<td style="text-align: center;">12 to 15</td>
<td colspan="8" style="text-align: center;">2nd image Top (Y)
address</td>
</tr>
<tr>
<td style="text-align: center;">16, 17</td>
<td colspan="8" style="text-align: center;">2nd image Perforation
number</td>
</tr>
<tr>
<td style="text-align: center;">18</td>
<td colspan="8" style="text-align: center;">2nd image Perforation
decimal</td>
</tr>
<tr>
<td style="text-align: center;">19</td>
<td colspan="8" style="text-align: center;">2nd image Pulse number</td>
</tr>
<tr>
<td style="text-align: center;">:</td>
<td colspan="8" style="text-align: center;">:</td>
</tr>
<tr>
<td style="text-align: center;">n-7 to n-4</td>
<td colspan="8" style="text-align: center;">mth (*) image Top (Y)
address</td>
</tr>
<tr>
<td style="text-align: center;">n-3, n-2</td>
<td colspan="8" style="text-align: center;">mth image Perforation
number</td>
</tr>
<tr>
<td style="text-align: center;">n-1</td>
<td colspan="8" style="text-align: center;">mth image Perforation
decimal</td>
</tr>
<tr>
<td style="text-align: center;">n</td>
<td colspan="8" style="text-align: center;">mth image Pulse number</td>
</tr>
</tbody>
</table>

\*: m=(n-3)/8

Byte 0, 1 Parameter length

This field specifies the length of the boundary information following
this field in bytes.

Byte 2 The actual number of images

This field specifies the number of thumbnail images that are stored in
this boundary information.

Byte 4 to 7 1st image Top (Y) address

The Y address at the top of the first thumbnail image detected by the
host

Byte 8, 9 1st image Perforation number

The perforation number at the top position of the first thumbnail image
detected by the host

Byte 10 1st image Perforation decimal

The perforation decimal at the top position of the first thumbnail image
detected by the host

Byte 11 1st image Pulse number

The pulse number at the top position of the first thumbnail image
detected by the host

Byte 12 and after

When the boundary information of the second and later image exists, the
top positions of the thumbnail images are stored in succession in the
same format.

After the thumbnail scanning of the strip film, the coordinate
information of each frame is set in the unit from the host.

The coordinate specified by the host is indicated by the top position
address (Top Address) and the perforation information of the top
position. When the images are scanned, the Y address specified in SET
WINDOW and the top position address of each frame are compared and the
frame which includes the specified Y address is obtained. Then the film
is moved so that the top position address of the frame matches the top
position of the range in which scanning is possible. For this film
movement, the perforation is detected and the movement is performed as
much as the perforation number, the perforation decimal, and the pulse
number that are sent by Boundary Information Type2.

**2-11-10. CCD data**

<table>
<colgroup>
<col style="width: 20%" />
<col style="width: 9%" />
<col style="width: 9%" />
<col style="width: 9%" />
<col style="width: 9%" />
<col style="width: 9%" />
<col style="width: 9%" />
<col style="width: 9%" />
<col style="width: 9%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: right;"><p>Bit</p>
<p>Byte</p></td>
<td style="text-align: center;">7</td>
<td style="text-align: center;">6</td>
<td style="text-align: center;">5</td>
<td style="text-align: center;">4</td>
<td style="text-align: center;">3</td>
<td style="text-align: center;">2</td>
<td style="text-align: center;">1</td>
<td style="text-align: center;">0</td>
</tr>
<tr>
<td style="text-align: center;">0, 1</td>
<td colspan="8" style="text-align: center;">The first point data of the
first type in CCD first line</td>
</tr>
<tr>
<td style="text-align: center;">2, 3</td>
<td colspan="8" style="text-align: center;">The second point data of the
first type in CCD first line</td>
</tr>
<tr>
<td style="text-align: center;">:</td>
<td colspan="8" style="text-align: center;">:</td>
</tr>
<tr>
<td style="text-align: center;">2mn-4, 2mn-3</td>
<td colspan="8" style="text-align: center;">The (m-1)th point data of
the nth type in CCD first line</td>
</tr>
<tr>
<td style="text-align: center;">2mn-2, 2mn-1</td>
<td colspan="8" style="text-align: center;">The mth point data of the
nth type in CCD first line</td>
</tr>
<tr>
<td style="text-align: center;">2mn, 2mn+1</td>
<td colspan="8" style="text-align: center;">The first point data of the
first type in CCD second line</td>
</tr>
<tr>
<td style="text-align: center;">:</td>
<td colspan="8" style="text-align: center;">:</td>
</tr>
<tr>
<td style="text-align: center;">2lmn-2, 2lmn-1</td>
<td colspan="8" style="text-align: center;">The mth point data of the
nth type in CCD second line</td>
</tr>
</tbody>
</table>

**2-11-11. Leak volume**

Leak_g, Leak_s, and Leak_k (2 bytes each) are sent to the initiator, in
that order.

‘FFFFh’ is sent for all of the three kinds when they are not recorded
once in the scanner.

The host should use the default value when ‘FFFFh’ is sent.

Because the value multiplied by 1,000,000 is recorded in the scanner,
the value divided by 1,000,000 should be used.

**  
2-12. SEND Command**

Table 2-12-1 SEND command

<table>
<colgroup>
<col style="width: 13%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: right;"><p>Bit</p>
<p>Byte</p></td>
<td style="text-align: center;">7</td>
<td style="text-align: center;">6</td>
<td style="text-align: center;">5</td>
<td style="text-align: center;">4</td>
<td style="text-align: center;">3</td>
<td style="text-align: center;">2</td>
<td style="text-align: center;">1</td>
<td style="text-align: center;">0</td>
</tr>
<tr>
<td style="text-align: center;">0</td>
<td colspan="8" style="text-align: center;">Operation code [2Ah]</td>
</tr>
<tr>
<td style="text-align: center;">1</td>
<td colspan="3" style="text-align: center;"><p>Logical unit number</p>
<p>[0]</p></td>
<td colspan="5" style="text-align: center;"><p>Reserved</p>
<p>[0]</p></td>
</tr>
<tr>
<td style="text-align: center;">2</td>
<td colspan="8" style="text-align: center;">Data type code</td>
</tr>
<tr>
<td style="text-align: center;">3</td>
<td colspan="8" style="text-align: center;">Reserved [0]</td>
</tr>
<tr>
<td style="text-align: center;">4</td>
<td colspan="8" style="text-align: center;">Data type qualifier (upper
byte)</td>
</tr>
<tr>
<td style="text-align: center;">5</td>
<td colspan="8" style="text-align: center;">Data type qualifier (lower
byte)</td>
</tr>
<tr>
<td style="text-align: center;">6</td>
<td colspan="8" style="text-align: left;">(MSB)</td>
</tr>
<tr>
<td style="text-align: center;">7</td>
<td colspan="8" style="text-align: center;">Transfer length</td>
</tr>
<tr>
<td style="text-align: center;">8</td>
<td colspan="8" style="text-align: right;">(LSB)</td>
</tr>
<tr>
<td style="text-align: center;">9</td>
<td colspan="8" style="text-align: center;">Control byte [0]</td>
</tr>
</tbody>
</table>

The SEND command transfers the data from the initiator to this unit.

The data type code and the data type qualifier are defined in the READ
command.

The transfer length specifies the number of blocks that are transferred
from the initiator to this unit during the DATA OUT phase. The block
size is the current block size specified by the mode parameter block
descriptor. The transfer length of zero is not regarded as an error, but
indicates that there is no data to be transferred.

This command shall be terminated with the RESERVATION CONFLICT status if
any reserved access conflict exists and there is no data to be
transferred.

**2-13. ABORT Command**

Table 2-13-1 ABORT command

<table>
<colgroup>
<col style="width: 13%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: right;"><p>Bit</p>
<p>Byte</p></td>
<td style="text-align: center;">7</td>
<td style="text-align: center;">6</td>
<td style="text-align: center;">5</td>
<td style="text-align: center;">4</td>
<td style="text-align: center;">3</td>
<td style="text-align: center;">2</td>
<td style="text-align: center;">1</td>
<td style="text-align: center;">0</td>
</tr>
<tr>
<td style="text-align: center;">0</td>
<td colspan="8" style="text-align: center;">Operation code [C0h]</td>
</tr>
<tr>
<td style="text-align: center;">1</td>
<td colspan="3" style="text-align: center;"><p>Logical unit number</p>
<p>[0]</p></td>
<td colspan="5" style="text-align: center;"><p>Reserved</p>
<p>[0]</p></td>
</tr>
<tr>
<td style="text-align: center;">2 to 4</td>
<td colspan="8" style="text-align: center;">Reserved [0]</td>
</tr>
<tr>
<td style="text-align: center;">5</td>
<td colspan="8" style="text-align: center;">Control byte [0]</td>
</tr>
</tbody>
</table>

The ABORT command aborts the scanning operation that is started by the
SCAN command.

This command is an operation activation command.

After returning GOOD status, the abort operation is performed.

When this command is received, the scan block movement for reading the
next line is not performed and the scan block stops at that position.

In order to read the image data after the above situation, the SCAN
command must be executed and the image must be read again from the
beginning.

Even if this command is accepted while the scanning operation is not
performed, GOOD status shall be returned.

**2-14. EXECUTE Command**

Table 2-14-1 EXECUTE command

<table>
<colgroup>
<col style="width: 13%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: right;"><p>Bit</p>
<p>Byte</p></td>
<td style="text-align: center;">7</td>
<td style="text-align: center;">6</td>
<td style="text-align: center;">5</td>
<td style="text-align: center;">4</td>
<td style="text-align: center;">3</td>
<td style="text-align: center;">2</td>
<td style="text-align: center;">1</td>
<td style="text-align: center;">0</td>
</tr>
<tr>
<td style="text-align: center;">0</td>
<td colspan="8" style="text-align: center;">Operation code [C1h]</td>
</tr>
<tr>
<td style="text-align: center;">1</td>
<td colspan="3" style="text-align: center;"><p>Logical unit number</p>
<p>[0]</p></td>
<td colspan="5" style="text-align: center;"><p>Reserved</p>
<p>[0]</p></td>
</tr>
<tr>
<td style="text-align: center;">2 to 4</td>
<td colspan="8" style="text-align: center;">Reserved [0]</td>
</tr>
<tr>
<td style="text-align: center;">5</td>
<td colspan="8" style="text-align: center;">Control byte [0]</td>
</tr>
</tbody>
</table>

The EXECUTE command performs the operation specified by the SET
PARAMETER command. The EXECUTE command is an operation activation
command.

The EXECUTE command performs the specified operation after returning
GOOD status.

A command other than the basic command must not be issued from the same
initiator to this unit before the operation termination is confirmed by
the TEST UNIT READY command.

Table 2-14-2 Sense data that is set in each status

<table>
<colgroup>
<col style="width: 27%" />
<col style="width: 41%" />
<col style="width: 30%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: center;">Status</td>
<td style="text-align: center;">Sense data</td>
<td style="text-align: center;">Remarks</td>
</tr>
<tr>
<td style="text-align: center;">When the TEST UNIT READY command is
received during operation</td>
<td style="text-align: center;"><p>LOGICAL UNIT IS IN PROCESS OF
BECOMING READY</p>
<p>(During the execution of the operation activation command)</p>
<p>02h-04h-01h-00h</p>
<p>(During loading/ejection of the object to be scanned)</p>
<p>02h-04h-01h-01h</p>
<p>(During the measurement of the correction data)</p>
<p>02h-04h-01h-02h</p>
<p>(During the execution of operation for loading the object to be
scanned)</p>
<p>02h-04h-01h-03h</p>
<p>(During the execution of automatic shading or white balance
measurement)</p>
<p>02h-04h-01h-04h</p></td>
<td style="text-align: center;">The command terminates with the CHECK
CONDITION status.</td>
</tr>
<tr>
<td style="text-align: center;">When the TEST UNIT READY command is
received after operation is terminated normally</td>
<td style="text-align: center;"><p>NO ADDITIONAL SENSE INFORMATION</p>
<p>(No error)</p>
<p>00h-00h-00h-00h</p></td>
<td style="text-align: center;">The command terminates with GOOD
status.</td>
</tr>
<tr>
<td style="text-align: center;">When a command other than the basic
command is received from the same initiator before the operation
termination is confirmed by the TEST UNIT READY command</td>
<td style="text-align: center;"><p>COMMAND SEQUENCE ERROR</p>
<p>(A command that makes the previous SCAN command invalid is received
while the scanning operation is valid)</p>
<p>05h-2Ch-00h-00h</p></td>
<td style="text-align: center;">The command terminates with the CHECK
CONDITION status. The measurement-related operation that is being
performed is aborted.</td>
</tr>
<tr>
<td style="text-align: center;">When a command other than the basic
command is received from the other initiator during operation</td>
<td style="text-align: center;"><p>LOGICAL UNIT COMMUNICATION
FAILURE</p>
<p>(The command cannot be executed because the internal operation is
being performed.)</p>
<p>0Bh-08h-00h-00h</p></td>
<td style="text-align: center;">The command terminates with the CHECK
CONDITION status. The operation that is being performed continues
without any influence.</td>
</tr>
</tbody>
</table>

<table>
<colgroup>
<col style="width: 27%" />
<col style="width: 41%" />
<col style="width: 30%" />
</colgroup>
<tbody>
<tr>
<td><blockquote>
<p>When the operation is not terminated normally</p>
</blockquote></td>
<td><p>LOGICAL UNIT NOT READY, CAUSE NOT REPORTABLE</p>
<p>(The internal mechanical error occurred.)</p>
<p>02h-04h-02h-00h</p></td>
<td>The command terminates with the CHECK CONDITION status for the TEST
UNIT READY command that is received after the operation is
terminated.</td>
</tr>
<tr>
<td>When the EXECUTE command is received before the operation parameter
is set by the SET PARAMETER command</td>
<td><p>COMMAND SEQUENCE ERROR</p>
<p>(The EXECUTE command is received before the parameter is set by the
SET PARAMETER command.)</p>
<p>05h-2Ch-00h-00h</p></td>
<td>The command terminates with the CHECK CONDITION status.</td>
</tr>
</tbody>
</table>

**2-15. SET PARAMETER Command**

Table 2-15-1 SET PARAMETER command

<table style="width:100%;">
<colgroup>
<col style="width: 13%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: right;"><p>Bit</p>
<p>Byte</p></td>
<td style="text-align: center;">7</td>
<td style="text-align: center;">6</td>
<td style="text-align: center;">5</td>
<td style="text-align: center;">4</td>
<td style="text-align: center;">3</td>
<td style="text-align: center;">2</td>
<td style="text-align: center;">1</td>
<td style="text-align: center;">0</td>
</tr>
<tr>
<td style="text-align: center;">0</td>
<td colspan="8" style="text-align: center;">Operation code [E0h]</td>
</tr>
<tr>
<td style="text-align: center;">1</td>
<td colspan="3" style="text-align: center;"><p>Logical unit number</p>
<p>[0]</p></td>
<td colspan="5" style="text-align: center;"><p>Reserved</p>
<p>[0]</p></td>
</tr>
<tr>
<td style="text-align: center;">2</td>
<td colspan="8" style="text-align: center;">Operation code</td>
</tr>
<tr>
<td style="text-align: center;">3 to 5</td>
<td colspan="8" style="text-align: center;">Reserved [0]</td>
</tr>
<tr>
<td style="text-align: center;">6</td>
<td colspan="8" style="text-align: left;">(MSB)</td>
</tr>
<tr>
<td style="text-align: center;">7</td>
<td colspan="8" style="text-align: center;">Parameter length
[Recommended value: 13d]</td>
</tr>
<tr>
<td style="text-align: center;">8</td>
<td colspan="8" style="text-align: right;">(LSB)</td>
</tr>
<tr>
<td style="text-align: center;">9</td>
<td colspan="8" style="text-align: center;">Control byte [0]</td>
</tr>
</tbody>
</table>

The SET PARAMETER command is used to set the parameters for the internal
operation of the unit.

The operation code field specifies the code that indicates the internal
operation to be set. The initiator can obtain the operation codes
supported by this unit and the parameter length for each operation code
by the INQUIRY command.

The operation parameter of the length specified by the parameter length
field is transferred.

This unit starts the specified operation when the EXECUTE command is
received after the parameters are set by the SET PARAMETER command.

Table 2-15-2 Operation parameter

<table>
<colgroup>
<col style="width: 12%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: right;"><p>Bit</p>
<p>Byte</p></td>
<td style="text-align: center;">7</td>
<td style="text-align: center;">6</td>
<td style="text-align: center;">5</td>
<td style="text-align: center;">4</td>
<td style="text-align: center;">3</td>
<td style="text-align: center;">2</td>
<td style="text-align: center;">1</td>
<td style="text-align: center;">0</td>
</tr>
<tr>
<td style="text-align: center;">0</td>
<td colspan="8" style="text-align: center;">Color specification</td>
</tr>
<tr>
<td style="text-align: center;">1 to 4</td>
<td colspan="8" style="text-align: center;">First setting value</td>
</tr>
<tr>
<td style="text-align: center;">5 to 8</td>
<td colspan="8" style="text-align: center;">Second setting value</td>
</tr>
<tr>
<td style="text-align: center;">9, 10</td>
<td colspan="8" style="text-align: center;">Speed</td>
</tr>
<tr>
<td style="text-align: center;">11</td>
<td colspan="8" style="text-align: center;">Torque</td>
</tr>
<tr>
<td style="text-align: center;">12</td>
<td colspan="8" style="text-align: center;">Driving method</td>
</tr>
</tbody>
</table>

The parameter of 2 bytes or more is transferred starting from the upper
byte.

- The first setting value field specifies the absolute address value
  when the specified operation code needs the address parameter.

> When AF (auto focusing) is performed (code A0h, A1h), the address on
> the medium where AF is performed in the main-scanning direction is
> specified. Zero in this field specifies the address zero.
>
> This field specifies the direction when the specified operation code
> has no address parameter and can switch the direction. A value of zero
> specifies the normal direction, and a value of one specifies the
> reverse direction.
>
> This field specifies the ON/OFF switching when the specified operation
> code has no address parameter and need to switch ON/OFF. A value of
> zero indicates the OFF status, and a value of one specifies the ON
> status.
>
> \* When the operation code is D5h, the value range is from 0 to 3200
> (in units of 10 ms; 1 ms when 0 is set).

- The second setting value field specifies the absolute address value
  when the specified operation code needs two address parameters.

> When AF (auto focusing) is performed (code A0h, A1h), the address on
> the medium where AF is performed in the sub-scanning direction is
> specified. Zero in this field specifies the address zero.
>
> \* When the operation code is D5h, 0 indicates film loading and 1
> indicates film ejection.

- The color specification field specifies which color is used for
  performing auto focus when the operation code is ’Color oriented Auto
  Focus’.

> The setting method of color specification is the same as that for the
> SET WINDOW command. This field has no meaning when the other operation
> code is specified.

- The speed field specifies the movement speed when the operation code
  of variable movement speed is set.

> Zero in this field specifies the default speed in the unit. This field
> has no meaning when the other operation code is specified.

- The torque field specifies the torque value when the specified
  operation code accepts various torques.

> Zero in this field indicates the default in the unit. This field has
> no meaning when the other operation code is specified.

- The driving method field specifies the drive actuation when the
  specified operation code accepts various driving methods.

> Zero in this field indicates the default in the unit. This field has
> no meaning when the other operation code is specified.

Table 2-15-3 List of operation codes

<table>
<colgroup>
<col style="width: 12%" />
<col style="width: 32%" />
<col style="width: 27%" />
<col style="width: 17%" />
<col style="width: 10%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: center;"><p>Operation</p>
<p>code</p></td>
<td style="text-align: center;">Internal operation to be set</td>
<td style="text-align: center;">Contents of operation</td>
<td style="text-align: center;">Valid parameters</td>
<td style="text-align: center;">Support of this unit</td>
</tr>
<tr>
<td style="text-align: center;">80h</td>
<td style="text-align: center;">Initialize</td>
<td style="text-align: left;">This unit is initialized in the same
manner as that of power ON.</td>
<td style="text-align: center;">None</td>
<td style="text-align: center;">Yes</td>
</tr>
<tr>
<td style="text-align: center;">81h</td>
<td style="text-align: center;">Return to the origin</td>
<td style="text-align: left;">Return to the origin</td>
<td style="text-align: center;">None</td>
<td style="text-align: center;">Yes</td>
</tr>
<tr>
<td style="text-align: center;">90h</td>
<td style="text-align: center;">Change Unit</td>
<td style="text-align: left;"></td>
<td style="text-align: center;">1st Val</td>
<td style="text-align: center;">No</td>
</tr>
<tr>
<td style="text-align: center;">91h</td>
<td style="text-align: center;">Auto AF</td>
<td style="text-align: center;"><p>Automatic AF execution</p>
<p>ON/OFF</p></td>
<td style="text-align: center;">1st Val</td>
<td style="text-align: center;">Yes</td>
</tr>
<tr>
<td style="text-align: center;">A0h</td>
<td style="text-align: center;">Auto Focus</td>
<td style="text-align: left;">Performs the auto focus</td>
<td style="text-align: center;">1st Val, 2nd Val</td>
<td style="text-align: center;">Yes</td>
</tr>
<tr>
<td style="text-align: center;">A1h</td>
<td style="text-align: center;">Color oriented Auto Focus</td>
<td style="text-align: left;"></td>
<td style="text-align: center;">1st Val, 2nd Val, color</td>
<td style="text-align: center;">No</td>
</tr>
<tr>
<td style="text-align: center;">B0h</td>
<td style="text-align: center;">Setup Shading Data</td>
<td style="text-align: left;">Performs the shading measurement</td>
<td style="text-align: center;">None</td>
<td style="text-align: center;">Yes</td>
</tr>
<tr>
<td style="text-align: center;">B1h</td>
<td style="text-align: center;">Setup Dark Current Correction Data</td>
<td style="text-align: left;">Performs the dark voltage measurement</td>
<td style="text-align: center;">None</td>
<td style="text-align: center;">Yes</td>
</tr>
<tr>
<td style="text-align: center;">B2h</td>
<td style="text-align: left;">Setup Offset Correction Data</td>
<td style="text-align: left;"></td>
<td style="text-align: center;">None</td>
<td style="text-align: center;">No</td>
</tr>
<tr>
<td style="text-align: center;">B4h</td>
<td style="text-align: center;">Unload time set</td>
<td style="text-align: left;">Setting the object unloading time</td>
<td style="text-align: center;">1st Val, 2nd Val</td>
<td style="text-align: center;">Yes</td>
</tr>
<tr>
<td style="text-align: center;">C0h</td>
<td style="text-align: center;">Stage Move</td>
<td style="text-align: left;">Moves the scan block in the scanning
direction</td>
<td style="text-align: center;">1st Val</td>
<td style="text-align: center;">Yes</td>
</tr>
<tr>
<td style="text-align: center;">C1h</td>
<td style="text-align: center;">Focus Move</td>
<td style="text-align: left;">Moves the scan block in the AF
direction</td>
<td style="text-align: center;">1st Val</td>
<td style="text-align: center;">Yes</td>
</tr>
<tr>
<td style="text-align: center;">D0h</td>
<td style="text-align: center;">Unload object</td>
<td style="text-align: left;">Unloads the object</td>
<td style="text-align: center;">None</td>
<td style="text-align: center;">Yes</td>
</tr>
<tr>
<td style="text-align: center;">D1h</td>
<td style="text-align: center;">Load object</td>
<td style="text-align: left;">Loads the object</td>
<td style="text-align: center;">None</td>
<td style="text-align: center;">Yes</td>
</tr>
<tr>
<td style="text-align: center;">D2h</td>
<td style="text-align: center;">Absolute positioning</td>
<td style="text-align: left;">Absolute positioning of the object</td>
<td style="text-align: center;">1st Val</td>
<td style="text-align: center;">Yes</td>
</tr>
<tr>
<td style="text-align: center;">D3h</td>
<td style="text-align: center;">Relative positioning</td>
<td style="text-align: left;">Relative positioning</td>
<td style="text-align: center;">1st Val</td>
<td style="text-align: center;">No</td>
</tr>
<tr>
<td style="text-align: center;">D4h</td>
<td style="text-align: center;">Rotate</td>
<td style="text-align: left;">Rotation</td>
<td style="text-align: center;">1st Val</td>
<td style="text-align: center;">No</td>
</tr>
<tr>
<td style="text-align: center;">D5h</td>
<td style="text-align: center;">FD</td>
<td style="text-align: left;">FD movement time setting</td>
<td style="text-align: center;">1st Val, 2nd Val</td>
<td style="text-align: center;">Yes</td>
</tr>
<tr>
<td style="text-align: center;">D6h</td>
<td style="text-align: center;">SA Lock</td>
<td style="text-align: left;">SA lock mechanism ON/OFF</td>
<td style="text-align: center;">1<sup>st</sup> Val</td>
<td style="text-align: center;">Yes</td>
</tr>
</tbody>
</table>

1<sup>st</sup> Val: First setting value

2<sup>nd</sup> Val: Second setting value

Color: Color specification

Speed: Speed specification

Torque: Torque

Drive: Driving method

Table 2-15-4 Descriptions of each parameter for the operation codes

<table style="width:100%;">
<colgroup>
<col style="width: 9%" />
<col style="width: 9%" />
<col style="width: 18%" />
<col style="width: 18%" />
<col style="width: 9%" />
<col style="width: 9%" />
<col style="width: 9%" />
<col style="width: 14%" />
</colgroup>
<tbody>
<tr>
<td>Opera-tion code</td>
<td><p>Color specifi-cation</p>
<p>(Color)</p></td>
<td style="text-align: center;"><p>First setting value</p>
<p>(1<sup>st</sup> Val)</p></td>
<td style="text-align: center;"><p>Second setting value</p>
<p>(2<sup>nd</sup> Val)</p></td>
<td style="text-align: center;"><p>Speed</p>
<p>specifi-cation</p>
<p>(Speed)</p></td>
<td style="text-align: center;"><p>Torque</p>
<p>(Torque)</p></td>
<td style="text-align: center;"><p>Driving method</p>
<p>(Drive)</p></td>
<td style="text-align: center;">Remarks</td>
</tr>
<tr>
<td style="text-align: center;">80h</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">No parameter</td>
</tr>
<tr>
<td style="text-align: center;">81h</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">No parameter</td>
</tr>
<tr>
<td style="text-align: center;">90h</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">Not supported</td>
</tr>
<tr>
<td style="text-align: center;">91h</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;"><p>Automatic AF execution</p>
<p>0: OFF</p>
<p>1: ON</p></td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;"></td>
</tr>
<tr>
<td style="text-align: center;">A0h</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">Address on the medium where AF is
performed in the main-scanning direction</td>
<td style="text-align: center;">Address on the medium where AF is
performed in the sub-scanning direction</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;"></td>
</tr>
<tr>
<td style="text-align: center;">A1h</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">Not supported</td>
</tr>
<tr>
<td style="text-align: center;">B0h</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">No parameter</td>
</tr>
<tr>
<td style="text-align: center;">B1h</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">No parameter</td>
</tr>
<tr>
<td style="text-align: center;">B2h</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">Not supported</td>
</tr>
<tr>
<td style="text-align: center;">B4h</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;"><p>Setting value of the unloading
time</p>
<p>(unit [s], default 600 [s])</p></td>
<td style="text-align: center;"><p>0: Timer OFF</p>
<p>1: Timer ON</p></td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;"></td>
</tr>
<tr>
<td style="text-align: center;">C0h</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">Address in the scanning direction</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;"></td>
</tr>
<tr>
<td style="text-align: center;">C1h</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">Address in the AF direction</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;"></td>
</tr>
<tr>
<td style="text-align: center;">D0h</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">No parameter</td>
</tr>
<tr>
<td style="text-align: center;">D1h</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">No parameter</td>
</tr>
<tr>
<td style="text-align: center;">D2h</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">Address in the main-scanning
direction</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;"></td>
</tr>
<tr>
<td style="text-align: center;">D3h</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">Not supported</td>
</tr>
<tr>
<td style="text-align: center;">D4h</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">Not supported</td>
</tr>
<tr>
<td style="text-align: center;">D5h</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;"><p>From 0 to 3200</p>
<p>(in units of 10 ms, 1 ms for 0)</p></td>
<td style="text-align: center;"><p>0: Loads the object</p>
<p>1: Unloads the object</p></td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;"></td>
</tr>
<tr>
<td style="text-align: center;">D6h</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;"><p>0: OFF</p>
<p>1: ON</p></td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;"></td>
</tr>
</tbody>
</table>

Note) The address is shown in units of 4000 dpi.

**2-16. GET PARAMETER Command**

Table 2-16-1 GET PARAMETER command

<table>
<colgroup>
<col style="width: 13%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: right;"><p>Bit</p>
<p>Byte</p></td>
<td style="text-align: center;">7</td>
<td style="text-align: center;">6</td>
<td style="text-align: center;">5</td>
<td style="text-align: center;">4</td>
<td style="text-align: center;">3</td>
<td style="text-align: center;">2</td>
<td style="text-align: center;">1</td>
<td style="text-align: center;">0</td>
</tr>
<tr>
<td style="text-align: center;">0</td>
<td colspan="8" style="text-align: center;">Operation code [E1h]</td>
</tr>
<tr>
<td style="text-align: center;">1</td>
<td colspan="3" style="text-align: center;"><p>Logical unit number</p>
<p>[0]</p></td>
<td colspan="5" style="text-align: center;"><p>Reserved</p>
<p>[0]</p></td>
</tr>
<tr>
<td style="text-align: center;">2</td>
<td colspan="8" style="text-align: center;">Operation code</td>
</tr>
<tr>
<td style="text-align: center;">3 to 5</td>
<td colspan="8" style="text-align: center;">Reserved [0]</td>
</tr>
<tr>
<td style="text-align: center;">6</td>
<td colspan="8" style="text-align: left;">(MSB)</td>
</tr>
<tr>
<td style="text-align: center;">7</td>
<td colspan="8" style="text-align: center;">Parameter length</td>
</tr>
<tr>
<td style="text-align: center;">8</td>
<td colspan="8" style="text-align: right;">(LSB)</td>
</tr>
<tr>
<td style="text-align: center;">9</td>
<td colspan="8" style="text-align: center;">Control byte [0]</td>
</tr>
</tbody>
</table>

The current settings for the operation specified by the operation code
can be read by using the GET PARAMETER command.

For the details of the parameters to be transferred, refer to table
2-15-4.

The parameters of the items that are not set contain a value of zero.

Even if the specified operation code differs from the operation code
that is set in the previous SET PARAMETER command, the parameter of the
specified code is returned.

The data returned for each operation code is the value that is currently
set in the unit.

**  
2-17. RECEIVE DIAGNOSTIC RESULTS Command**

Table 2-17-1 RECEIVE DIAGNOSTIC RESULTS command

<table>
<colgroup>
<col style="width: 13%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
<col style="width: 10%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: right;"><p>Bit</p>
<p>Byte</p></td>
<td style="text-align: center;">7</td>
<td style="text-align: center;">6</td>
<td style="text-align: center;">5</td>
<td style="text-align: center;">4</td>
<td style="text-align: center;">3</td>
<td style="text-align: center;">2</td>
<td style="text-align: center;">1</td>
<td style="text-align: center;">0</td>
</tr>
<tr>
<td style="text-align: center;">0</td>
<td colspan="8" style="text-align: center;">Operation code [1Ch]</td>
</tr>
<tr>
<td style="text-align: center;">1</td>
<td colspan="3" style="text-align: center;"><p>Logical unit number</p>
<p>[0]</p></td>
<td colspan="5" style="text-align: center;"><p>Reserved</p>
<p>[0]</p></td>
</tr>
<tr>
<td style="text-align: center;">2</td>
<td colspan="8" style="text-align: center;">Reserved [0]</td>
</tr>
<tr>
<td style="text-align: center;">3</td>
<td colspan="8" rowspan="2" style="text-align: center;"><p>(MSB)
Allocation length</p>
<p>(LSB)</p></td>
</tr>
<tr>
<td style="text-align: center;">4</td>
</tr>
<tr>
<td style="text-align: center;">5</td>
<td colspan="8" style="text-align: center;">Reserved [0]</td>
</tr>
<tr>
<td colspan="9" style="text-align: left;"></td>
</tr>
</tbody>
</table>

The RECEIVE DIAGNOSTIC RESULTS command is used to read the diagnostic
data after the SEND DIAGNOSTIC command is executed. When this command is
received, this unit transfers the diagnostic results of the SEND
DIAGNOSTIC command that was executed previously to the initiator.

\[Caution\]

It is recommended that this unit be reserved in order to guarantee that
the information about the diagnostic results will not be broken by the
command output from the other initiator.

**2-17-1. Error handling**

Table 2-17-2 Error handling

<table>
<colgroup>
<col style="width: 34%" />
<col style="width: 39%" />
<col style="width: 26%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: center;">Status</td>
<td style="text-align: center;">Sense data</td>
<td style="text-align: center;">Remarks</td>
</tr>
<tr>
<td style="text-align: center;">When the SEND DIAGNOSTIC command is
received with the specification of parameter when the adapter is not
attached</td>
<td style="text-align: center;"><p>INVALID FIELD IN CDB</p>
<p>(Some illegal data exists in the CDB.)</p>
<p>05h-24h-00h-00h</p></td>
<td style="text-align: center;">The command terminates with the CHECK
CONDITION status.</td>
</tr>
<tr>
<td style="text-align: center;">When the RECEIVE DIAGNOSTIC RESULTS
command is received independently when the adapter is not attached</td>
<td style="text-align: center;"><p>INVALID COMMAND OPERATION CODE</p>
<p>(Op-Code that is not supported is received.)</p>
<p>05h-20h-00h-00h</p></td>
<td style="text-align: center;">The command terminates with the CHECK
CONDITION status.</td>
</tr>
<tr>
<td style="text-align: center;">When the RECEIVE DIAGNOSTIC RESULTS
command is received independently when the adapter is attached</td>
<td style="text-align: center;"><p>COMMAND SEQUENCE ERROR</p>
<p>(The RECEIVE DIAGNOSTIC RESULT command is received independently when
the adapter for inspection is attached.)</p>
<p>05h-2Ch-00h-00h</p></td>
<td style="text-align: center;">The command terminates with the CHECK
CONDITION status.</td>
</tr>
</tbody>
</table>
