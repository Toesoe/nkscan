**SUPER COOLSCAN 9000 ED I/F PROTOCOL SPECIFICATIONS**

**1. FEATURES OF THE COMMUNICATION PROTOCOL IN THIS UNIT**

> The communication protocol of this unit conforms to the interface
> standard of IEEE1394 (SBP-2).

The specifications for each communication protocol are explained below.

**1-1. SBP-2 Protocol Specifications**

**1-1-1. Outline**

> The host reads the configuration ROM of the target device, and ensures
> that the device has SBP-2 functions.
>
> In SBP-2 protocol, the host only gives the pointer to the linked list
> of commands. The target should fetch these commands, and execute them
> in proper order. These SBP-2 transaction layers are supported in the
> link-layer-controller on this unit that conforms to IEEE1394-1995 and
> P1394a.
>
> These include login, logout and reconnection of a management agent,
> the fetch engine that gets the linked list of ORB (Operation Request
> Block) from the logged-in nodes, flow control for data transfer, and
> the support for target node page table management. Transaction layers
> analyze the ORB, and can extract CDB (Command Descriptor Block) and
> return it with status information to the logged-in node.

**1-1-2. Supported functions**

> The functions that are supported in this unit are described below.

- Order command model

- Unsolicited status

- ABORT_TASK_SET management function

- TARGET_RESET management function

- NODE RESET

> This unit does not support the following functions.

- Multiple host

- Unorder command model (command queue)

- ECA status

- SCSI command link function

- SET PASSWORD management function

- ABORT_TASK management function

- LOGICAL_UNIT_RESET management function

- Abort task in “rq_fmt=3” format

**1-1-3. Session management, device detection, and data type**

> The outline of the sessions that are performed by the initiator for
> this unit is shown below.

- Log-in

- The address of the operation request block (ORB) with SCSI inquiry
  command (INQUIRY) is written to the ORB_POINTER register, and the SCSI
  inquiry command (INQUIRY) is issued.

- The command set implementation is assumed to be the SCSI Primary
  Commands (SPC).

- The ORB_POINTER writing is performed only after login, because the
  operating system uses all the doorbells.

- Log-out

**1-1-3-1. Multiple logical unit**

> This unit is a single logical unit.

**  
1-1-3-2. Password**

> The password is not used at the time of login. Instead, when the
> initiator logs in, the password with 0 set to all figures is assumed.
>
> The initiator does not set the password for the current
> implementation. However, the command is supported for future use.

**1-1-3-3. Exclusive login**

> The host retrieves the devices in order by using SBP-2 Device Type.
>
> The target must abort the current task set in the very rare instance
> that the host log-out is terminated before the current task set.

**1-1-3-4. Other commands**

> In the same manner, the host uses the following commands;
> ABORT_TASK_SET, TARGET_RESET, and RECONNECT. These commands are used
> for the time-out request recovery between the sessions and
> reconnection after bus reset.

**1-1-3-5. Device detection**

> The operating system uses only the configuration ROM to detect the
> device. The target detection mechanism ‘depends on the command set’
> according to ANSI SBP-2 specifications. However, the host does not
> implement the mechanism.

**1-1-4. Command execution**

> For the command execution model of this unit, ‘order model (the target
> executes one ORB at a time, and after each ORB is executed, the
> completion status is sent out)’ is used.

**1-1-5. Task management**

> The host uses only ‘ABORT_TASK_SET’ when the task is aborted. (The
> host does not use ‘Abort Task ORB’ or ‘rq_fmt=3’ format.)
>
> When the target is reset, the host uses ‘TARGET_RESET’. (The Logical
> Unit Reset command in the ANSI SBP-2 specifications is not used.)

**1-1-6. Status responses**

> The SBP-2 module of the host always sets the notify bit to 1. For the
> status format, refer to SBP-2 Annex B.
>
> The host always sets the UNSOLICITED_STATUS_ENABLE register to
> ‘enable’.
>
> If one or more status blocks are reserved, one status block must be
> sent out.
>
> The UNSOLICITED_STATUS_ENABLE register is cleared.
>
> The host enables UNSOLICITED_STATUS_ENABLE again. The next UNSOLICITED
> STATUS BLOCK can be sent out at any time.
>
> ‘UNSOLICITED STATUS’ is the status that is transferred to the
> initiator by the target that is not related to a specific ORB. (Data
> or management command)

**1-1-7. CSR register**

> First, the host performs writing to the RESET_START register. When it
> is not terminated, the host executes the low-level NODE RESET
> (IEEE1394-1995 Control Status Registers (CSR)). The host does not use
> the STATE_CLEAR register. In the same manner, the host does not read
> the AGENT_STATE register value.
>
> If the request is out of time, the recovery starts the following
> steps.

1.  ABORT_TASK_SET is issued.

> If ABORT_TASK_SET is succeeded, the operating system issues the
> request again and the process progresses.

2.  If ABORT_TASK_SET is failed, the operating system outputs
    TARGET_RESET.

> If TARGET_RESET is succeeded, the request is sent again.

3.  If TARGET_RESET is failed, the operating system executes NODE RESET
    and after that, logs in again.

> When the login is never terminated, the device may be broken. And it
> is marked as the defective device.

Correspondence between SBP-2 and SCSI-2

<table>
<colgroup>
<col style="width: 23%" />
<col style="width: 51%" />
<col style="width: 25%" />
</colgroup>
<tbody>
<tr>
<td><blockquote>
<p>Action</p>
</blockquote></td>
<td><blockquote>
<p>SBP-2</p>
</blockquote></td>
<td><blockquote>
<p>SCSI-2</p>
</blockquote></td>
</tr>
<tr>
<td><blockquote>
<p>Command abort</p>
</blockquote></td>
<td><blockquote>
<p>ABORT_TASK_SET management function</p>
</blockquote></td>
<td><blockquote>
<p>ABORT message</p>
</blockquote></td>
</tr>
<tr>
<td><blockquote>
<p>Hard reset</p>
</blockquote></td>
<td><blockquote>
<p>Writing to the RESET_START register</p>
<p>TARGET_RESET management function</p>
<p>NODE_RESET</p>
</blockquote></td>
<td><blockquote>
<p>BUS DEVICE RESET message</p>
</blockquote></td>
</tr>
</tbody>
</table>

**1-1-8. Configuration ROM**

> This unit defines the configuration ROM as shown below based on the
> SBP-2 specifications.

Table 1-1-8-1 Configuration ROM format of this unit

<table style="width:100%;">
<colgroup>
<col style="width: 10%" />
<col style="width: 2%" />
<col style="width: 2%" />
<col style="width: 2%" />
<col style="width: 2%" />
<col style="width: 2%" />
<col style="width: 2%" />
<col style="width: 2%" />
<col style="width: 2%" />
<col style="width: 2%" />
<col style="width: 2%" />
<col style="width: 2%" />
<col style="width: 2%" />
<col style="width: 2%" />
<col style="width: 2%" />
<col style="width: 2%" />
<col style="width: 2%" />
<col style="width: 2%" />
<col style="width: 2%" />
<col style="width: 2%" />
<col style="width: 2%" />
<col style="width: 2%" />
<col style="width: 2%" />
<col style="width: 2%" />
<col style="width: 2%" />
<col style="width: 2%" />
<col style="width: 2%" />
<col style="width: 2%" />
<col style="width: 2%" />
<col style="width: 2%" />
<col style="width: 2%" />
<col style="width: 2%" />
<col style="width: 2%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: right;"><p>Bit</p>
<p>Quadlet</p></td>
<td style="text-align: center;">0</td>
<td style="text-align: center;">1</td>
<td style="text-align: center;">2</td>
<td style="text-align: center;">3</td>
<td style="text-align: center;">4</td>
<td style="text-align: center;">5</td>
<td style="text-align: center;">6</td>
<td style="text-align: center;">7</td>
<td style="text-align: center;">8</td>
<td style="text-align: center;">9</td>
<td style="text-align: center;"><p>1</p>
<p>0</p></td>
<td style="text-align: center;"><p>1</p>
<p>1</p></td>
<td style="text-align: center;"><p>1</p>
<p>2</p></td>
<td style="text-align: center;"><p>1</p>
<p>3</p></td>
<td style="text-align: center;"><p>1</p>
<p>4</p></td>
<td style="text-align: center;"><p>1</p>
<p>5</p></td>
<td style="text-align: center;"><p>1</p>
<p>6</p></td>
<td style="text-align: center;"><p>1</p>
<p>7</p></td>
<td style="text-align: center;"><p>1</p>
<p>8</p></td>
<td style="text-align: center;"><p>1</p>
<p>9</p></td>
<td style="text-align: center;"><p>2</p>
<p>0</p></td>
<td style="text-align: center;"><p>2</p>
<p>1</p></td>
<td style="text-align: center;"><p>2</p>
<p>2</p></td>
<td style="text-align: center;"><p>2</p>
<p>3</p></td>
<td style="text-align: center;"><p>2</p>
<p>4</p></td>
<td style="text-align: center;"><p>2</p>
<p>5</p></td>
<td style="text-align: center;"><p>2</p>
<p>6</p></td>
<td style="text-align: center;"><p>2</p>
<p>7</p></td>
<td style="text-align: center;"><p>2</p>
<p>8</p></td>
<td style="text-align: center;"><p>2</p>
<p>9</p></td>
<td style="text-align: center;"><p>3</p>
<p>0</p></td>
<td style="text-align: center;"><p>3</p>
<p>1</p></td>
</tr>
<tr>
<td style="text-align: center;">0</td>
<td colspan="8" style="text-align: center;">Length [04h]</td>
<td colspan="8" style="text-align: center;">CRC Length [11h]</td>
<td colspan="16" style="text-align: center;">ROM CRC</td>
</tr>
<tr>
<td style="text-align: center;">1</td>
<td colspan="32" style="text-align: center;">Bus Name=“1394”
[31333934h]</td>
</tr>
<tr>
<td style="text-align: center;">2</td>
<td colspan="32" style="text-align: center;">Node_Options
[00FF5012h]</td>
</tr>
<tr>
<td style="text-align: center;">3</td>
<td colspan="24" style="text-align: center;">Node_Vendor_ID
[0090B5h]</td>
<td colspan="8" style="text-align: center;">Chip_ID_Hi [40h]</td>
</tr>
<tr>
<td style="text-align: center;">4</td>
<td colspan="32" style="text-align: center;">Chip_ID_Lo [03h+Serial
Number]</td>
</tr>
<tr>
<td style="text-align: center;">5</td>
<td colspan="16" style="text-align: center;">Directory Length
[0005h]</td>
<td colspan="16" style="text-align: center;">Root Directory CRC</td>
</tr>
<tr>
<td style="text-align: center;">6</td>
<td colspan="8" style="text-align: center;">Key [03h]</td>
<td colspan="24" style="text-align: center;">Module_Vendor_ID
[0090B5h]</td>
</tr>
<tr>
<td style="text-align: center;">7</td>
<td colspan="8" style="text-align: center;">Key [0Ch]</td>
<td colspan="24" style="text-align: center;">Node_Capabilities
[0083C0h]</td>
</tr>
<tr>
<td style="text-align: center;">8</td>
<td colspan="8" style="text-align: center;">Key [81h]</td>
<td colspan="24" style="text-align: center;">Offset to Leaf of
Textual_Descriptor [00000Dh]</td>
</tr>
<tr>
<td style="text-align: center;">9</td>
<td colspan="8" style="text-align: center;">Key [04h]</td>
<td colspan="24" style="text-align: center;">Module_HW_Version
[00500Ah]</td>
</tr>
<tr>
<td style="text-align: center;">10</td>
<td colspan="8" style="text-align: center;">Key [D1h]</td>
<td colspan="24" style="text-align: center;">Offset to Directory of
Unit_Directory [000001h]</td>
</tr>
<tr>
<td style="text-align: center;">11</td>
<td colspan="16" style="text-align: center;">Directory Length
[0009h]</td>
<td colspan="16" style="text-align: center;">Unit Directory CRC
[44FEh]</td>
</tr>
<tr>
<td style="text-align: center;">12</td>
<td colspan="8" style="text-align: center;">Key [12h]</td>
<td colspan="24" style="text-align: center;">Unit_Spec_ID [00609Eh]</td>
</tr>
<tr>
<td style="text-align: center;">13</td>
<td colspan="8" style="text-align: center;">Key [13h]</td>
<td colspan="24" style="text-align: center;">Unit_SW_Version
[010483h]</td>
</tr>
<tr>
<td style="text-align: center;">14</td>
<td colspan="8" style="text-align: center;">Key [54h]</td>
<td colspan="24" style="text-align: center;">Management_Agent
[00C000h]</td>
</tr>
<tr>
<td style="text-align: center;">15</td>
<td colspan="8" style="text-align: center;">Key [3Ah]</td>
<td colspan="24" style="text-align: center;">Unit Characteristics
[000A08h]</td>
</tr>
<tr>
<td rowspan="2" style="text-align: center;">16</td>
<td colspan="8" rowspan="2" style="text-align: center;">Key [14h]</td>
<td colspan="24" style="text-align: center;">Logical_Unit_Number
[060000h]</td>
</tr>
<tr>
<td colspan="8" style="text-align: center;">Device_Type [06h]</td>
<td colspan="16" style="text-align: center;">LUN [0000h]</td>
</tr>
<tr>
<td style="text-align: center;">17</td>
<td colspan="8" style="text-align: center;">Key [38h]</td>
<td colspan="24" style="text-align: center;">Command_Set_Spec_ID
[00609h]</td>
</tr>
<tr>
<td style="text-align: center;">18</td>
<td colspan="8" style="text-align: center;">Key [39h]</td>
<td colspan="24" style="text-align: center;">Command_Set [0104D8h]</td>
</tr>
<tr>
<td style="text-align: center;">19</td>
<td colspan="8" style="text-align: center;">Key [17h]</td>
<td colspan="24" style="text-align: center;">Model_ID [004002h]</td>
</tr>
<tr>
<td style="text-align: center;">20</td>
<td colspan="8" style="text-align: center;">Key [81h]</td>
<td colspan="24" style="text-align: center;">Offset to Leaf of
Textual_Descriptor [000007h]</td>
</tr>
<tr>
<td style="text-align: center;">21</td>
<td colspan="16" style="text-align: center;">Leaf Length [0005h]</td>
<td colspan="16" style="text-align: center;">Text Leaf CRC</td>
</tr>
<tr>
<td style="text-align: center;">22</td>
<td colspan="8" style="text-align: center;">Spec_Type [00h]</td>
<td colspan="24" style="text-align: center;">Specifier_ID [000000h]</td>
</tr>
<tr>
<td style="text-align: center;">23</td>
<td colspan="32" style="text-align: center;">Language_ID
[00000000h]</td>
</tr>
<tr>
<td style="text-align: center;">24 to 26</td>
<td colspan="32" style="text-align: center;">(“ Nikon ”)</td>
</tr>
<tr>
<td style="text-align: center;">27</td>
<td colspan="16" style="text-align: center;">Leaf Length [0006h]</td>
<td colspan="16" style="text-align: center;">Text Leaf CRC [0FDDh]</td>
</tr>
<tr>
<td style="text-align: center;">28</td>
<td colspan="8" style="text-align: center;">Spec_Type [00h]</td>
<td colspan="24" style="text-align: center;">Specifier_ID [000000h]</td>
</tr>
<tr>
<td style="text-align: center;">29</td>
<td colspan="32" style="text-align: center;">Language_ID
[00000000h]</td>
</tr>
<tr>
<td style="text-align: center;">30 to 33</td>
<td colspan="32" style="text-align: center;">(“ LS-9000 ED ”)</td>
</tr>
</tbody>
</table>

Remarks: Directory structure of the configuration ROM

> Bus information block (Quadlet 0 to 4)
>
> \|----- Root directory (Quadlet 5 to 10)
>
> \|- Unit directory (Quadlet 11 to 20)
>
> \|----- Text descriptor leaf (Quadlet 21 to 26)
>
> \|- Text descriptor leaf (Quadlet 27 to 33)

**  
1-1-9. Commands of this unit**

> The commands that are executed by this unit are shown in table
> 1-1-9-1.

Table 1-1-9-1 List of the commands of this unit

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
<td style="text-align: center;">Service (phase) transition</td>
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
<p>RESERVE UNIT</p>
</blockquote></td>
<td style="text-align: center;">16h</td>
<td style="text-align: center;">M</td>
<td style="text-align: center;"><blockquote>
<p>C - S</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;"><blockquote>
<p>RELEASE UNIT</p>
</blockquote></td>
<td style="text-align: center;">17h</td>
<td style="text-align: center;">M</td>
<td style="text-align: center;"><blockquote>
<p>C - S</p>
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

Service (phase) explanation

C : Command service (phase)

Din : Data in service (phase)

Dout : Data out service (phase)

S : Status service (phase)

> Note) The presence of data service (phase) described in the example is
> in the case that the transfer length contains non-zero value.

**1-1-10. Status of this unit**

> The statuses that are transferred by this unit are shown below.
>
> The status is transferred when each command is terminated if the
> command is not terminated by any of the conditions below.

1)  In the case of power reset

2)  In the case of hard reset

- When writing to the RESET_START register is performed

- TARGET_RESET management function

- NODE RESET (IEEE1394-1995 Control Status Registers (CSR))

3)  When the ABORT_TASK_SET management function is received

4)  In the case of log-out

Table 1-1-10-1 Status byte code of this unit

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
<col style="width: 4%" />
</colgroup>
<tbody>
<tr>
<td colspan="9" style="text-align: center;">Bit of the status byte</td>
<td colspan="3" rowspan="2" style="text-align: center;">Status</td>
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
<td colspan="2" style="text-align: center;">R</td>
<td style="text-align: center;">R</td>
<td style="text-align: center;">0</td>
<td style="text-align: center;">0</td>
<td style="text-align: center;">1</td>
<td style="text-align: center;">0</td>
<td style="text-align: center;">0</td>
<td style="text-align: center;">R</td>
<td style="text-align: center;"><blockquote>
<p>BUSY</p>
</blockquote></td>
<td colspan="2" style="text-align: center;">[08h]</td>
</tr>
<tr>
<td colspan="2" style="text-align: center;">R</td>
<td style="text-align: center;">R</td>
<td style="text-align: center;">0</td>
<td style="text-align: center;">1</td>
<td style="text-align: center;">1</td>
<td style="text-align: center;">0</td>
<td style="text-align: center;">0</td>
<td style="text-align: center;">R</td>
<td style="text-align: center;"><blockquote>
<p>RESERVATION CONFLICT</p>
</blockquote></td>
<td colspan="2" style="text-align: center;">[18h]</td>
</tr>
<tr>
<td colspan="11" style="text-align: center;"><blockquote>
<p>Key: R - Reserved bit (set to 0)</p>
</blockquote></td>
<td style="text-align: center;"></td>
</tr>
</tbody>
</table>

> When the status is ‘GOOD’, only the first 2 quadlets of the status ORB
> are transferred.
>
> When the status is other than ‘GOOD’ (CHECK CONDITION and so on), the
> eight quadlets with sense data are transferred as shown below.

Table 1-1-10-2 Status of this unit

<table>
<colgroup>
<col style="width: 11%" />
<col style="width: 2%" />
<col style="width: 2%" />
<col style="width: 2%" />
<col style="width: 2%" />
<col style="width: 2%" />
<col style="width: 2%" />
<col style="width: 2%" />
<col style="width: 2%" />
<col style="width: 2%" />
<col style="width: 2%" />
<col style="width: 2%" />
<col style="width: 2%" />
<col style="width: 2%" />
<col style="width: 2%" />
<col style="width: 2%" />
<col style="width: 2%" />
<col style="width: 2%" />
<col style="width: 2%" />
<col style="width: 2%" />
<col style="width: 2%" />
<col style="width: 2%" />
<col style="width: 2%" />
<col style="width: 2%" />
<col style="width: 2%" />
<col style="width: 2%" />
<col style="width: 2%" />
<col style="width: 2%" />
<col style="width: 2%" />
<col style="width: 2%" />
<col style="width: 2%" />
<col style="width: 2%" />
<col style="width: 2%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: right;"><p>Bit</p>
<p>Quadlet</p></td>
<td style="text-align: center;">0</td>
<td style="text-align: center;">1</td>
<td style="text-align: center;">2</td>
<td style="text-align: center;">3</td>
<td style="text-align: center;">4</td>
<td style="text-align: center;">5</td>
<td style="text-align: center;">6</td>
<td style="text-align: center;">7</td>
<td style="text-align: center;">8</td>
<td style="text-align: center;">9</td>
<td style="text-align: center;"><p>1</p>
<p>0</p></td>
<td style="text-align: center;"><p>1</p>
<p>1</p></td>
<td style="text-align: center;"><p>1</p>
<p>2</p></td>
<td style="text-align: center;"><p>1</p>
<p>3</p></td>
<td style="text-align: center;"><p>1</p>
<p>4</p></td>
<td style="text-align: center;"><p>1</p>
<p>5</p></td>
<td style="text-align: center;"><p>1</p>
<p>6</p></td>
<td style="text-align: center;"><p>1</p>
<p>7</p></td>
<td style="text-align: center;"><p>1</p>
<p>8</p></td>
<td style="text-align: center;"><p>1</p>
<p>9</p></td>
<td style="text-align: center;"><p>2</p>
<p>0</p></td>
<td style="text-align: center;"><p>2</p>
<p>1</p></td>
<td style="text-align: center;"><p>2</p>
<p>2</p></td>
<td style="text-align: center;"><p>2</p>
<p>3</p></td>
<td style="text-align: center;"><p>2</p>
<p>4</p></td>
<td style="text-align: center;"><p>2</p>
<p>5</p></td>
<td style="text-align: center;"><p>2</p>
<p>6</p></td>
<td style="text-align: center;"><p>2</p>
<p>7</p></td>
<td style="text-align: center;"><p>2</p>
<p>8</p></td>
<td style="text-align: center;"><p>2</p>
<p>9</p></td>
<td style="text-align: center;"><p>3</p>
<p>0</p></td>
<td style="text-align: center;"><p>3</p>
<p>1</p></td>
</tr>
<tr>
<td style="text-align: center;">0</td>
<td colspan="2" style="text-align: center;">Src</td>
<td colspan="2" style="text-align: center;"><p>Re</p>
<p>sp</p></td>
<td style="text-align: center;">D</td>
<td colspan="3" style="text-align: center;">Len</td>
<td colspan="8" style="text-align: center;">Sbp_status</td>
<td colspan="16" style="text-align: center;">ORB_offset_hi</td>
</tr>
<tr>
<td style="text-align: center;">1</td>
<td colspan="30" style="text-align: center;">ORB_offset_lo</td>
<td colspan="2" style="text-align: center;">r</td>
</tr>
<tr>
<td style="text-align: center;">2</td>
<td colspan="2" style="text-align: center;">sfmt</td>
<td colspan="6" style="text-align: center;">Status</td>
<td style="text-align: center;">V</td>
<td style="text-align: center;">M</td>
<td style="text-align: center;">E</td>
<td style="text-align: center;">I</td>
<td colspan="4" style="text-align: center;">Sense_key</td>
<td colspan="8" style="text-align: center;">Sense_code</td>
<td colspan="8" style="text-align: center;">Sense_qualifier</td>
</tr>
<tr>
<td style="text-align: center;">3</td>
<td colspan="32" style="text-align: center;">Information</td>
</tr>
<tr>
<td style="text-align: center;">4</td>
<td colspan="32" style="text-align: center;">CDB-dependent</td>
</tr>
<tr>
<td style="text-align: center;">5</td>
<td colspan="8" style="text-align: center;">Fru</td>
<td colspan="24" style="text-align: center;">Sense_key-dependent</td>
</tr>
<tr>
<td style="text-align: center;">6, 7</td>
<td colspan="32" style="text-align: center;">Vendor-dependent</td>
</tr>
</tbody>
</table>

Quadlet 2

- Status (bit 2 to 7)

> Indicates the status code in table 1-1-10-1.

- Sense_key (bit 12 to 15)

> Indicates the sense key.

- Sense_code (bit 16 to 23)

> Indicates the additional sense code (ASC).

- Sense_qualifier (bit 24 to 31)

> Indicates the additional sense code qualifier (ASCQ).

Quadlet 5

- Fru (bit 0 to 7)

> Indicates the detailed error information.

**  
1-1-11. Command processing and exception status**

**1-1-11-1. CA status, auto sense, REQUEST SENSE command**

> For the sense data clearance, when the status block is returned by the
> auto sense function of the SBP-2 target implement, the CA status
> condition is automatically cleared.
>
> The SBP-2 target can transfer the status, together with the sense
> data.
>
> When the CA status occurs in the target (CHECK CONDITION status is
> returned), the status with the sense data is transferred.
>
> The initiator that detected the CA status (received the CHECK
> CONDITION status) does not have to execute the REQUEST SENSE command.

**1-1-11-2. I/O process in the queue**

> Not supported by this unit.

**1-1-11-3. Unit attention status**

> This unit generates the unit attention status for each initiator in
> the following cases.

1)  After the power is turned ON (UA without film is not transferred)

2)  When the attached holder is removed (UA without film is not
    transferred)

3)  When the holder is exchanged

**1-1-11-4. Log-out processing**

> When the log-out occurs, this unit moves the fetch agent of the target
> to RESET and operates as follows.

1)  The command is aborted and the status when the log-out is performed
    succeeds.

2)  Perform the same processing as that in the case of SCSI-ABORT
    command.

3)  Perform the same operations as TARGET_RESET (hard reset of the
    target by the host).

**2. COMMAND EXPLANATIONS**

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
<p>00h-00h-00h-00h (No error)</p></td>
</tr>
<tr>
<td style="text-align: center;">Check Condition</td>
<td style="text-align: center;"><p>Logical Unit Not Supported</p>
<blockquote>
<p>05h-25h-00h-00h (An LUN other than 0 was specified.)</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">Check Condition</td>
<td style="text-align: left;"><p>Logical Unit Is In Process Of Becoming
Ready</p>
<blockquote>
<p>02h-04h-01h-00h (During the execution of the operation activation
command)</p>
<p>02h-04h-01h-01h (During loading/ejection of the object to be
scanned)</p>
<p>02h-04h-01h-02h (During the measurement of the correction data)</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">Check Condition</td>
<td style="text-align: left;"><p>Logical Unit Not Ready, Cause Not
Reportable</p>
<blockquote>
<p>02h-04h-02h-00h (The internal mechanical error occurred.)</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">Check Condition</td>
<td style="text-align: left;"><p>Logical Unit Not Ready, Manual
Intervention Required</p>
<p>02h-04h-03h-06h (FH-869GR: The mask is not set.)</p>
<blockquote>
<p>02h-04h-03h-07h (Undefined holder)</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">Check Condition</td>
<td style="text-align: left;"><p>Logical Unit Does Not Respond To
Selection</p>
<blockquote>
<p>02h-05h-00h-00h (The operation is possible, but the initialization
operation in the unit is not completed because the power is just turned
ON.)</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">Check Condition</td>
<td style="text-align: left;"><p>Medium Not Present</p>
<blockquote>
<p>02h-3Ah-00h-01h (The holder is not inserted.)</p>
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
<p>0Bh-4Bh-00h-00h (Unexpected error during Data Phase)</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">Check Condition</td>
<td style="text-align: left;"><p>Overlapped Commands Attempted</p>
<blockquote>
<p>0Bh-4Eh-00h-00h (The unit is selected by the same initiator while
disconnecting.)</p>
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
    shall not clear the unit attention condition.

> Table 2-2-1-2 Page code field list

<table>
<colgroup>
<col style="width: 5%" />
<col style="width: 9%" />
<col style="width: 26%" />
<col style="width: 13%" />
<col style="width: 8%" />
<col style="width: 35%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: center;">VPD</td>
<td colspan="3" style="text-align: center;">Page code</td>
<td style="text-align: center;">Sub-section</td>
<td style="text-align: center;">Attached holder</td>
</tr>
<tr>
<td style="text-align: center;">0</td>
<td colspan="2" style="text-align: center;">Standard INQUIRY data</td>
<td style="text-align: center;">00h (*1)</td>
<td style="text-align: center;">2-2-1</td>
<td style="text-align: center;"></td>
</tr>
<tr>
<td rowspan="33" style="text-align: center;">1</td>
<td rowspan="33" style="text-align: center;">VPD informa-tion</td>
<td style="text-align: center;">Page code list</td>
<td style="text-align: center;">00h</td>
<td style="text-align: center;">2-2-2-1</td>
<td style="text-align: center;"></td>
</tr>
<tr>
<td rowspan="23" style="text-align: center;">FRU ASCII information</td>
<td style="text-align: center;">01h (unused)</td>
<td rowspan="23" style="text-align: center;">2-2-2-2</td>
<td style="text-align: center;">-</td>
</tr>
<tr>
<td style="text-align: center;">10h</td>
<td style="text-align: center;">Undefined holder</td>
</tr>
<tr>
<td style="text-align: center;">11h (unused)</td>
<td style="text-align: center;">-</td>
</tr>
<tr>
<td style="text-align: center;">12h</td>
<td style="text-align: center;">FH-816</td>
</tr>
<tr>
<td style="text-align: center;">13h</td>
<td style="text-align: center;">FH-8G1</td>
</tr>
<tr>
<td style="text-align: center;">14h</td>
<td style="text-align: center;">FH-835M</td>
</tr>
<tr>
<td style="text-align: center;">15h</td>
<td style="text-align: center;">FH-835S</td>
</tr>
<tr>
<td style="text-align: center;">16h</td>
<td style="text-align: center;">FH-869M</td>
</tr>
<tr>
<td style="text-align: center;">17h</td>
<td style="text-align: center;">FH-869S</td>
</tr>
<tr>
<td style="text-align: center;">18h</td>
<td style="text-align: center;">FH-869G</td>
</tr>
<tr>
<td style="text-align: center;">19h</td>
<td style="text-align: center;">FH-869GR 6*4.5</td>
</tr>
<tr>
<td style="text-align: center;">1Ah</td>
<td style="text-align: center;">FH-869GR 6*6</td>
</tr>
<tr>
<td style="text-align: center;">1Bh</td>
<td style="text-align: center;">FH-869GR 6*7</td>
</tr>
<tr>
<td style="text-align: center;">1Ch</td>
<td style="text-align: center;">FH-869GR 6*8</td>
</tr>
<tr>
<td style="text-align: center;">1Dh</td>
<td style="text-align: center;">FH-869GR 6*9</td>
</tr>
<tr>
<td style="text-align: center;">1Eh</td>
<td style="text-align: center;">FH-869GR panorama 58</td>
</tr>
<tr>
<td style="text-align: center;">1Fh</td>
<td style="text-align: center;">FH-869GR panorama 65</td>
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
<td style="text-align: center;">42h (unused)</td>
<td style="text-align: center;">-</td>
</tr>
<tr>
<td style="text-align: center;">50h (unused)</td>
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
<td style="text-align: center;">Address information</td>
<td style="text-align: center;">C1h</td>
<td style="text-align: center;">2-2-2-3</td>
<td style="text-align: center;"></td>
</tr>
<tr>
<td rowspan="4" style="text-align: center;">Additional address
information</td>
<td style="text-align: center;">C8h</td>
<td rowspan="4" style="text-align: center;">2-2-2-6</td>
<td style="text-align: center;"></td>
</tr>
<tr>
<td style="text-align: center;">C9h</td>
<td style="text-align: center;">FH-816</td>
</tr>
<tr>
<td style="text-align: center;">CAh</td>
<td style="text-align: center;">FH-816</td>
</tr>
<tr>
<td style="text-align: center;">CBh</td>
<td style="text-align: center;">FH-816</td>
</tr>
<tr>
<td style="text-align: center;">SET WINDOW function</td>
<td style="text-align: center;">D1h</td>
<td style="text-align: center;">2-2-2-4</td>
<td style="text-align: center;"></td>
</tr>
<tr>
<td style="text-align: center;">Other information</td>
<td style="text-align: center;">E1h</td>
<td style="text-align: center;">2-2-2-5</td>
<td style="text-align: center;"></td>
</tr>
<tr>
<td rowspan="2" style="text-align: center;">Unused page</td>
<td style="text-align: center;">F0h</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
</tr>
<tr>
<td style="text-align: center;">F8h</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
</tr>
</tbody>
</table>

\*1 Page code field value of the INQUIRY command that is transferred
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
<p>[LS-9000 ED ]</p></td>
</tr>
<tr>
<td style="text-align: center;">32 to 35</td>
<td colspan="8" style="text-align: center;"><p>Product Revision
Level</p>
<p>Example: [1.00]</p></td>
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
>
> This unit supports 00h, 01h, 40h, 41h, 42h, 50h, 60h, 61h, C1h, D1h,
> E1h, F0h, F8h, IDs of the attached holder (10h to 1Fh), and the
> additional address page.
>
> For the additional address page, C8h, C9h, CAh, and CBh are supported
> when the attached holder is FH-816; and C8h is supported in other
> cases.

Note) On the above supported pages, 01h, 40h, 41h, 42h, 50h, 60h, 61h,
F0h, and F8h are not used.

**  **
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

Table 2-2-2-2-1 Holder ID and holder name

|  |  |  |  |
|----|----|----|----|
| Page code (ID) | Attached holder | ASCII information | Descriptions |
| 10h | Undefined holder | Unknown | Undefined holder |
| 12h | FH-816 | 16mm Film | 16-mm film holder |
| 13h | FH-8G1 | Praparat | Praparat holder |
| 14h | FH-835M | 35mm Mount Film | 35-mm mount film holder |
| 15h | FH-835S | 35mm Strip Film | 35-mm strip film holder |
| 16h | FH-869M | Brownie Mount Film | Brownie mount film holder |
| 17h | FH-869S | Brownie Strip Film | Brownie strip film holder |
| 18h | FH-869G | Brownie Strip Film with G | Brownie strip film holder with glass |
| 19h | FH-869GR 6\*4.5 | 6\*4.5 Film | Rotation holder 6\*4.5 |
| 1Ah | FH-869GR 6\*6 | 6\*6 Film | Rotation holder 6\*6 |
| 1Bh | FH-869GR 6\*7 | 6\*7 Film | Rotation holder 6\*7 |
| 1Ch | FH-869GR 6\*8 | 6\*8 Film | Rotation holder 6\*8 |
| 1Dh | FH-869GR 6\*9 | 6\*9 Film | Rotation holder 6\*9 |
| 1Eh | FH-869GR panorama 58 | Panorama 58 | Rotation holder panorama 58 |
| 1Fh | FH-869GR panorama 65 | Panorama 65 | Rotation holder panorama 65 |

**  **
**2-2-2-3. Address information page**

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
<td colspan="8" style="text-align: center;">Page length [87d=57h]</td>
</tr>
<tr>
<td style="text-align: center;">4</td>
<td colspan="8" style="text-align: center;"><blockquote>
<p>SCSI function support (SCSI data transfer function) [1]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">5, 6</td>
<td colspan="8" style="text-align: center;"><blockquote>
<p>Window descriptor block length</p>
<p>[58=003Ah]</p>
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
<p>Image Buffer Size (Unit: KB)</p>
<p>[256=0100h]</p>
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
<p>Unit Name ID</p>
<p>[01h]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">15</td>
<td colspan="8" style="text-align: center;"><blockquote>
<p>Current Holder Name ID (ID number of the attached holder)</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">16</td>
<td colspan="8" style="text-align: center;"><blockquote>
<p>Coordinate base information (resolution type and scanning that are
supported)</p>
<p>[42h]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">17</td>
<td colspan="8" style="text-align: center;"><blockquote>
<p>Addressing Kind (addressing type that is supported)</p>
<p>[12h]</p>
</blockquote></td>
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
<p>[666=029Ah]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">24 to 27</td>
<td colspan="8" style="text-align: center;"><blockquote>
<p>X-Maximum Set Window Address</p>
<p>(Window descriptor X-axis offset address maximum value)</p>
<p>[9999=270Fh]</p>
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
<p>[10000=2710h]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">40, 41</td>
<td colspan="8" style="text-align: center;"><blockquote>
<p>Y-Optical Resolution (Unit: dpi)</p>
<p>[4000=0FA0h]</p>
</blockquote></td>
</tr>
</tbody>
</table>

<table>
<colgroup>
<col style="width: 11%" />
<col style="width: 88%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: center;">42, 43</td>
<td><blockquote>
<p>Y-Maximum Resolution (Unit: dpi)</p>
<p>[4000=0FA0h]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">44, 45</td>
<td><blockquote>
<p>Y-Minimum Resolution (Unit: dpi)</p>
<p>[333=014Dh]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">46 to 49</td>
<td><blockquote>
<p>Y-Maximum Set Window Address</p>
<p>(Window descriptor Y-axis offset address maximum value)</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">50 to 53</td>
<td><blockquote>
<p>Y-Minimum Set Window Address</p>
<p>(Window descriptor Y-axis offset address minimum value)</p>
<p>[0]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">54 to 57</td>
<td><blockquote>
<p>Y-Offset for first image’s address (Y-axis scanning start position
offset address)</p>
<p>[0]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">58 to 61</td>
<td><blockquote>
<p>Y-Set Window boundary</p>
<p>(Maximum window width value of the Y-axis window descriptor)</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">62 to 65</td>
<td><blockquote>
<p>Y-Another world maximum Address</p>
<p>(Maximum address in the sub-scanning direction outside the specified
address)</p>
<p>[0]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">66 to 69</td>
<td><blockquote>
<p>Y-Another world minimum Address</p>
<p>(Minimum address in the sub-scanning direction outside the specified
address)</p>
<p>[0]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">70, 71</td>
<td><blockquote>
<p>Maximum Thumbnail Resolution (maximum resolution in thumbnail
scanning. Unit: dpi)</p>
<p>[83=0053h]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">72, 73</td>
<td><blockquote>
<p>Minimum Thumbnail Resolution (minimum resolution in thumbnail
scanning. Unit: dpi)</p>
<p>[83=0053h]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">74</td>
<td><blockquote>
<p>Maximum Image count (maximum number of frames that can be
scanned)</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">75</td>
<td><blockquote>
<p>Actual including image count (the number of medium frames that are
currently set)</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">76, 77</td>
<td><blockquote>
<p>Minimum Focusing Address (minimum address of the focus position)</p>
<p>[0]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">78, 79</td>
<td><blockquote>
<p>Maximum Focusing Address (maximum address of the focus position)</p>
<p>[450=01C2h]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">80, 81</td>
<td><blockquote>
<p>Lamp warm-up maximum time (maximum time for lamp warming-up) [0]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">82</td>
<td><blockquote>
<p>A/D bit depth (depth of bits for an A/D converter)</p>
<p>[16=10h]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">83, 84</td>
<td><blockquote>
<p>CCD Pixel Number (the number of effective pixels in the CCD.</p>
<p>For the CCD in which the number of effective pixels differs in each
color, the maximum value is set.)</p>
<p>[10000=2710h]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">85</td>
<td><blockquote>
<p>Line Gap Count (the number of pixels between lines of the CCD)</p>
<p>[12=0Ch]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">86</td>
<td><blockquote>
<p>CCD Line Number (the number of lines in the CCD)</p>
<p>[03h]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">87, 88</td>
<td><blockquote>
<p>Reserved</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">89, 90</td>
<td><blockquote>
<p>Reserved</p>
</blockquote></td>
</tr>
</tbody>
</table>

\*1 When an invalid logical unit selection is performed

Byte 4 SCSI function support

> This field specifies the SCSI data transfer function.
>
> Setting each bit to zero indicates that the function is not supported
> by this unit, and setting each bit to one indicates that the function
> is supported by this unit. In this unit, this field is set to 01h.

<table>
<colgroup>
<col style="width: 12%" />
<col style="width: 71%" />
<col style="width: 16%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: center;">Bit</td>
<td style="text-align: center;">Descriptions</td>
<td style="text-align: center;"><p>Support of</p>
<p>this unit</p></td>
</tr>
<tr>
<td style="text-align: center;">0</td>
<td style="text-align: left;"><blockquote>
<p>Unused</p>
</blockquote></td>
<td style="text-align: center;">1</td>
</tr>
<tr>
<td style="text-align: center;">1</td>
<td style="text-align: left;"><blockquote>
<p>Image reading (READ command) must be performed in units of</p>
<p>[Data of one line in bytes * number of colors].</p>
</blockquote></td>
<td style="text-align: center;">0</td>
</tr>
<tr>
<td style="text-align: center;">2</td>
<td style="text-align: left;"><blockquote>
<p>Image reading (READ command) must be performed in units of</p>
<p>[Data of one line in bytes].</p>
</blockquote></td>
<td style="text-align: center;">0</td>
</tr>
<tr>
<td style="text-align: center;">3 to 6</td>
<td style="text-align: left;"><blockquote>
<p>Reserved</p>
</blockquote></td>
<td style="text-align: center;">0</td>
</tr>
<tr>
<td style="text-align: center;">7</td>
<td style="text-align: left;"><blockquote>
<p>Extend bit</p>
</blockquote></td>
<td style="text-align: center;">0</td>
</tr>
</tbody>
</table>

Byte 5, 6 Window descriptor block length

> This field specifies the length of the window descriptor in bytes. In
> this unit, this field is set to 58d.

Byte 7, 8 Set parameter descriptor block length

> This field specifies the length of the SET PARAMETER command parameter
> in bytes. In this unit, this field is set to 15d.

Byte 9, 10 General SCSI Buffer Size

> This field specifies the data size that is used for the SCSI data
> transfer in bytes. Zero indicates that the buffer size is not limited.
> In this unit, this field is set to zero.

Byte 11, 12 Image Buffer Size

> This field specifies the image buffer size in kilobytes. In this unit,
> this field is set to 256d.

Byte 13 Number of equipped Unit

> This field specifies the number of units that can be attached to this
> unit simultaneously. In this unit, this field is set to one.

Byte 14 Unit Name ID

> This field specifies the ID number of the adapter that is currently
> attached. In this unit, this field is set to 1.

Byte 15 Current Holder Name ID

> This field specifies the ID number of the attached holder.

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
<td>Resolution type [2]</td>
<td style="text-align: left;"><p>Setting this bit to 0 indicates that
reading can be performed in continuous resolution. Setting this bit to 1
indicates that reading can be performed only in the resolution of each
pitch. Setting this bit to 2 indicates that reading can be performed in
the Line Gap Count measure pitch.</p>
<p>For the pitch, refer to the explanation of the GET WINDOW
command.</p></td>
</tr>
<tr>
<td style="text-align: center;">Bit2</td>
<td><p>X Origin Reversed</p>
<blockquote>
<p>[0]</p>
</blockquote></td>
<td style="text-align: left;">Setting this bit to 1 indicates that the
main-scanning direction origin is reversed (at the right end of the
medium).</td>
</tr>
<tr>
<td style="text-align: center;">Bit3</td>
<td><p>Y Origin Reversed</p>
<blockquote>
<p>[0]</p>
</blockquote></td>
<td style="text-align: left;">Setting this bit to 1 indicates that the
sub-scanning direction origin is reversed (at the bottom end of the
medium).</td>
</tr>
<tr>
<td style="text-align: center;">Bit4</td>
<td><p>Thumbnail Order Reversed</p>
<blockquote>
<p>[0]</p>
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
<td>Additional Coordinate Information [1]</td>
<td style="text-align: left;">This bit is set to 1 in this unit.</td>
</tr>
<tr>
<td style="text-align: center;">Bit7</td>
<td>Extend bit [0]</td>
<td style="text-align: left;">This bit is set to 0 in this unit.</td>
</tr>
</tbody>
</table>

Byte 17 Addressing Kind

> This field specifies the addressing type that is supported. The
> addressing of the bit to which 1 is set is supported.

<table style="width:87%;">
<colgroup>
<col style="width: 8%" />
<col style="width: 4%" />
<col style="width: 8%" />
<col style="width: 46%" />
<col style="width: 12%" />
<col style="width: 7%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: center;">Bit</td>
<td colspan="3" style="text-align: center;">Descriptions</td>
<td colspan="2" style="text-align: center;">Support of this unit</td>
</tr>
<tr>
<td style="text-align: center;">0</td>
<td colspan="3" style="text-align: center;"><blockquote>
<p>The Set Window address is the same as the medium position
address.</p>
</blockquote></td>
<td colspan="2" style="text-align: center;">0</td>
</tr>
<tr>
<td style="text-align: center;">1</td>
<td colspan="3" style="text-align: center;"><blockquote>
<p>The Set Window address is the same as the address of the mechanical
block.</p>
</blockquote></td>
<td colspan="2" style="text-align: center;">1</td>
</tr>
<tr>
<td style="text-align: center;">2</td>
<td colspan="3" style="text-align: center;"><blockquote>
<p>Specifying the scanning range over two or more frames is
prohibited.</p>
</blockquote></td>
<td colspan="2" style="text-align: center;">0</td>
</tr>
<tr>
<td style="text-align: center;">3</td>
<td colspan="3" style="text-align: center;"><blockquote>
<p>Reserved</p>
</blockquote></td>
<td colspan="2" style="text-align: center;">0</td>
</tr>
<tr>
<td style="text-align: center;">4</td>
<td colspan="3" style="text-align: center;"><blockquote>
<p>The position of the medium can be operated.</p>
</blockquote></td>
<td colspan="2" style="text-align: center;">1</td>
</tr>
<tr>
<td style="text-align: center;">5</td>
<td colspan="3" style="text-align: center;"><blockquote>
<p>The mechanical block position can be operated.</p>
</blockquote></td>
<td colspan="2" style="text-align: center;">0</td>
</tr>
<tr>
<td style="text-align: center;">6</td>
<td colspan="3" style="text-align: center;"><blockquote>
<p>Reserved</p>
</blockquote></td>
<td colspan="2" style="text-align: center;">0</td>
</tr>
<tr>
<td style="text-align: center;">7</td>
<td colspan="3" style="text-align: center;"><blockquote>
<p>Extension bit</p>
</blockquote></td>
<td colspan="2" style="text-align: center;">0</td>
</tr>
</tbody>
</table>

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

Byte 86 CCD Line Number

> This field specifies the number of lines in the CCD. When 0 is set or
> no value is sent to this field, ‘3 lines’ is set. In this unit, this
> field is set to 3.

**2-2-2-4. SET WINDOW function page**

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
<p>[03h]</p></td>
</tr>
<tr>
<td style="text-align: center;">5</td>
<td colspan="8" style="text-align: left;"><blockquote>
<p>Scan Mode Support</p>
<p>[16h]</p>
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
<p>[0]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">9</td>
<td colspan="8" style="text-align: left;"><blockquote>
<p>Color Ordering2</p>
<p>[0]</p>
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
> This unit supports Image Scanning and Thumbnail Scanning.

<table style="width:100%;">
<colgroup>
<col style="width: 6%" />
<col style="width: 26%" />
<col style="width: 50%" />
<col style="width: 17%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: center;">Bit</td>
<td style="text-align: center;">Type</td>
<td style="text-align: center;">Explanations of operation</td>
<td style="text-align: center;">Support of this unit</td>
</tr>
<tr>
<td style="text-align: center;">0</td>
<td style="text-align: left;">Image Scanning</td>
<td style="text-align: left;">Normal image scanning</td>
<td style="text-align: center;">1</td>
</tr>
<tr>
<td style="text-align: center;">1</td>
<td style="text-align: left;">Thumbnail Scanning</td>
<td style="text-align: left;">Thumbnail image scanning</td>
<td style="text-align: center;">1</td>
</tr>
<tr>
<td style="text-align: center;">2</td>
<td style="text-align: left;">Set up Scanning</td>
<td style="text-align: left;"><p>Prescan</p>
<p>Scanning for deciding the optimal integral time and gain,
etc.</p></td>
<td style="text-align: center;">0</td>
</tr>
<tr>
<td style="text-align: center;">3</td>
<td style="text-align: left;">Reserved</td>
<td style="text-align: left;">Reserved</td>
<td style="text-align: center;">-</td>
</tr>
<tr>
<td style="text-align: center;">4</td>
<td style="text-align: left;">Reserved</td>
<td style="text-align: left;">Reserved</td>
<td style="text-align: center;">-</td>
</tr>
<tr>
<td style="text-align: center;">5</td>
<td style="text-align: left;">Auto Exposure Scanning</td>
<td style="text-align: left;">Scanning for deciding the integral time at
which the output value becomes the AE Value that is set in each
color</td>
<td style="text-align: center;">0</td>
</tr>
<tr>
<td style="text-align: center;">6</td>
<td style="text-align: left;">AE with WB Scanning</td>
<td style="text-align: left;">Scanning for deciding the integral time at
which the maximum value of the output values in each color becomes the
AE Value that is set with the white balance maintained</td>
<td style="text-align: center;">0</td>
</tr>
<tr>
<td style="text-align: center;">7</td>
<td style="text-align: left;">Extend bit</td>
<td style="text-align: center;">Extension bit [0]</td>
<td style="text-align: center;">0</td>
</tr>
</tbody>
</table>

Byte 5 Scan Mode Support

> This field specifies the scanning mode.
>
> Normal Quality Scan, High Speed Scan, Multiple Reading Scan, and
> Reverse direction Scanning Supported are supported.

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
<td style="text-align: center;">[1]</td>
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

Byte 6 Color Interleaving Support

> This field specifies the color order for data transfer.
>
> ‘Line without CCD distance’ and ‘Multi line Simultaneous reading’ are
> supported.

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
<p>Extend bit</p>
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
> case of C-M-Y, the setting is C=1, M=2, Y=3.
>
> Bit 0 to 3 specify the color that can be scanned as the first color. 0
> indicates that all colors can be scanned as the first color.
>
> Bit 4 to 7 specify the color that can be scanned as the second color.
> 0 indicates that all colors can be scanned as the second color.

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
> case of C-M-Y, the setting is C=1, M=2, Y=3.
>
> Bit 0 to 3 specify the color that can be scanned as the third color. 0
> indicates that all colors can be scanned as the third color.
>
> Bit 4 to 7 specify the color that can be scanned as the fourth color.
> 0 indicates that all colors can be scanned as the fourth color.

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
>
> Byte 17 to 20 Minimum Value for the First Control
>
> This specifies the minimum value that can be set.
>
> The minimum value of the integral time in this unit is 10 nsec.
>
> Byte 21 to 24 Maximum Value for the First Control
>
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
<p>Byte 4 [83h]</p>
<p>Byte 5 [05h]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">6 to 10</td>
<td colspan="8" style="text-align: left;"><blockquote>
<p>Send/Read supported information (SEND/READ command support data
transfer)</p>
<p>Byte 6 [ACh]</p>
<p>Byte 7 [00h]</p>
<p>Byte 8 [D0h]</p>
<p>Byte 9 [3Ah]</p>
<p>Byte 10 [48h]</p>
</blockquote></td>
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
<p>Bits per a Max Value Data (the number of bits of the AE maximum
value)</p>
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
<p>Bits per a Dark Current Data</p>
<p>(The number of bits in each data of the dark voltage correction
coefficient)</p>
<p>[16=10h]</p>
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
<p>Byte 22 [06h]</p>
<p>Byte 23 [0]</p>
</blockquote></td>
</tr>
</tbody>
</table>

<table>
<colgroup>
<col style="width: 12%" />
<col style="width: 87%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: center;">24, 25</td>
<td style="text-align: left;"><blockquote>
<p>Execute operation support A0</p>
<p>(Function that is supported by operation code Axh of Execute)</p>
<p>Byte 24 [01h]</p>
<p>Byte 25 [0]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">26, 27</td>
<td style="text-align: left;"><blockquote>
<p>Execute operation support B0</p>
<p>(Function that is supported by operation code Bxh of Execute)</p>
<p>Byte 26 [09h]</p>
<p>Byte 27 [0]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">28, 29</td>
<td style="text-align: left;"><blockquote>
<p>Execute operation support C0</p>
<p>(Function that is supported by operation code Cxh of Execute)</p>
<p>Byte 28 [02h]</p>
<p>Byte 29 [0]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">30, 31</td>
<td style="text-align: left;"><blockquote>
<p>Execute operation support D0</p>
<p>(Function that is supported by operation code Dxh of Execute)</p>
<p>Byte 30 [01h]</p>
<p>Byte 31 [0]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">32, 33</td>
<td style="text-align: left;"><blockquote>
<p>Execute operation support E0</p>
<p>(Function that is supported by operation code Exh of Execute)</p>
<p>Byte 32 [0]</p>
<p>Byte 33 [0]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">34, 35</td>
<td style="text-align: left;"><blockquote>
<p>Execute operation support F0</p>
<p>(Function that is supported by operation code Fxh of Execute)</p>
<p>Byte 34 [0]</p>
<p>Byte 35 [0]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">36</td>
<td style="text-align: left;"><blockquote>
<p>Additional Information (other additional information)</p>
<p>[02h]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">37</td>
<td style="text-align: left;"><blockquote>
<p>Volatile buffer for Initiator use (RAM buffer area)</p>
<p>[04h]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">38</td>
<td style="text-align: left;"><blockquote>
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
<col style="width: 12%" />
<col style="width: 12%" />
<col style="width: 59%" />
<col style="width: 15%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: center;">Byte</td>
<td style="text-align: center;">Bit</td>
<td style="text-align: center;">Descriptions</td>
<td style="text-align: center;">Support</td>
</tr>
<tr>
<td style="text-align: center;">4</td>
<td style="text-align: center;">0</td>
<td style="text-align: left;"><blockquote>
<p>Thumbnail created by driver</p>
</blockquote></td>
<td style="text-align: center;">1</td>
</tr>
<tr>
<td style="text-align: center;"></td>
<td style="text-align: center;">1</td>
<td style="text-align: left;"><blockquote>
<p>Averaging multiple reading by driver</p>
</blockquote></td>
<td style="text-align: center;">1</td>
</tr>
<tr>
<td style="text-align: center;"></td>
<td style="text-align: center;">2</td>
<td style="text-align: left;"><blockquote>
<p>Registration gap resolved by driver</p>
</blockquote></td>
<td style="text-align: center;">0</td>
</tr>
<tr>
<td style="text-align: center;"></td>
<td style="text-align: center;">3</td>
<td style="text-align: left;"><blockquote>
<p>Dark voltage data created by driver</p>
</blockquote></td>
<td style="text-align: center;">0</td>
</tr>
<tr>
<td style="text-align: center;"></td>
<td style="text-align: center;">4</td>
<td style="text-align: left;"><blockquote>
<p>Shading calibration data created by driver</p>
</blockquote></td>
<td style="text-align: center;">0</td>
</tr>
<tr>
<td style="text-align: center;"></td>
<td style="text-align: center;">5</td>
<td style="text-align: left;"><blockquote>
<p>Auto Focus by driver</p>
</blockquote></td>
<td style="text-align: center;">0</td>
</tr>
<tr>
<td style="text-align: center;"></td>
<td style="text-align: center;">6</td>
<td style="text-align: left;"><blockquote>
<p>Shading correction by driver</p>
</blockquote></td>
<td style="text-align: center;">0</td>
</tr>
<tr>
<td style="text-align: center;"></td>
<td style="text-align: center;">7</td>
<td style="text-align: left;"><blockquote>
<p>Extend bit</p>
</blockquote></td>
<td style="text-align: center;">1</td>
</tr>
<tr>
<td style="text-align: center;">5</td>
<td style="text-align: center;">0</td>
<td style="text-align: left;"><blockquote>
<p>Multi line simultaneous reading process by driver</p>
</blockquote></td>
<td style="text-align: center;">1</td>
</tr>
<tr>
<td style="text-align: center;"></td>
<td style="text-align: center;">1</td>
<td style="text-align: left;"><blockquote>
<p>Pitch in the main-scanning direction by driver</p>
</blockquote></td>
<td style="text-align: center;">0</td>
</tr>
<tr>
<td style="text-align: center;"></td>
<td style="text-align: center;">2</td>
<td style="text-align: left;"><blockquote>
<p>Truncated by driver</p>
</blockquote></td>
<td style="text-align: center;">1</td>
</tr>
<tr>
<td style="text-align: center;"></td>
<td style="text-align: center;">3</td>
<td style="text-align: left;"><blockquote>
<p>CCD Data Created by Driver</p>
</blockquote></td>
<td style="text-align: center;">1</td>
</tr>
<tr>
<td style="text-align: center;"></td>
<td style="text-align: center;">4 to 6</td>
<td style="text-align: left;"><blockquote>
<p>Reserved</p>
</blockquote></td>
<td style="text-align: center;">0</td>
</tr>
<tr>
<td style="text-align: center;"></td>
<td style="text-align: center;">7</td>
<td style="text-align: left;"><blockquote>
<p>Extend bit</p>
</blockquote></td>
<td style="text-align: center;">0</td>
</tr>
</tbody>
</table>

Byte 6 to 10 Send/Read supported information

> This field specifies the data transfer that is supported by the Send
> and the Read commands.
>
> The data transfer of the bit that is set to 1 is supported.

|        |      |                                  |       |
|--------|------|----------------------------------|:-----:|
| Byte 6 | Bit0 | Halftone mask reading supported  | \[0\] |
|        | Bit1 | Halftone mask writing supported  | \[0\] |
|        | Bit2 | Gamma function reading supported | \[0\] |
|        | Bit3 | Gamma function writing supported | \[0\] |
|        | Bit4 | Histogram Data reading supported | \[0\] |
|        | Bit5 | Max Value Data reading supported | \[0\] |
|        | Bit6 | Reserved                         | \[0\] |
|        | Bit7 | Extend bit                       | \[1\] |

|        |      |                                |       |
|:-------|:-----|:-------------------------------|:-----:|
| Byte 7 | Bit0 | Matrix Data reading supported  | \[0\] |
|        | Bit1 | Matrix Data writing supported  | \[0\] |
|        | Bit2 | Filter Data reading supported  | \[0\] |
|        | Bit3 | Filter Data writing supported  | \[0\] |
|        | Bit4 | Shading Data reading supported | \[0\] |
|        | Bit5 | Shading Data writing supported | \[0\] |
|        | Bit6 | Reserved                       | \[0\] |
|        | Bit7 | Extend bit                     | \[1\] |

|        |      |                                          |       |
|:-------|:-----|:-----------------------------------------|:-----:|
| Byte 8 | Bit0 | Dark Voltage Data reading supported      | \[0\] |
|        | Bit1 | Dark Voltage Data writing supported      | \[0\] |
|        | Bit2 | Magnetic Data reading supported          | \[0\] |
|        | Bit3 | Magnetic Data writing supported          | \[0\] |
|        | Bit4 | Cooperation parameters reading supported | \[1\] |
|        | Bit5 | Boundary data reading supported          | \[1\] |
|        | Bit6 | Boundary data writing supported          | \[1\] |
|        | Bit7 | Extend bit                               | \[1\] |

|        |      |                                           |       |
|:-------|:-----|:------------------------------------------|:-----:|
| Byte 9 | Bit0 | Analog Gamma reading supported            | \[0\] |
|        | Bit1 | Analog Gain reading supported             | \[1\] |
|        | Bit2 | Digital Gain reading supported            | \[0\] |
|        | Bit3 | Exposure Value reading supported          | \[1\] |
|        | Bit4 | Setup Information reading supported       | \[1\] |
|        | Bit5 | Setup Information writing supported       | \[1\] |
|        | Bit6 | Perforation Information reading supported | \[0\] |
|        | Bit7 | Extend bit                                | \[1\] |

|         |      |                                             |       |
|:--------|:-----|:--------------------------------------------|:-----:|
| Byte 10 | Bit0 | Boundary Type2 data reading supported       | \[0\] |
|         | Bit1 | Boundary Type2 data writing supported       | \[0\] |
|         | Bit2 | Initial WB Exposure Value reading supported | \[0\] |
|         | Bit3 | CCD data reading supported                  | \[1\] |
|         | Bit4 | Driver Soft Version reading supported       | \[0\] |
|         | Bit5 | Driver Soft Version writing supported       | \[0\] |
|         | Bit6 | Leak data reading supported                 | \[1\] |
|         | Bit7 | Extend bit                                  | \[0\] |

Byte 11 Bits per a halftone mask parameter

> This field specifies the length in bits of the halftone mask. This
> unit sets this field to 0.

Byte 12 and 13 X/Y bit depth of Download LUT

> This field specifies the length in bits of the input/output data in
> the LUT that is transferred from the initiator.
>
> This unit sets this field to 0.

Byte 14 Bits per a Histogram Data

> This field specifies the length in bits of each histogram data. This
> unit sets this field to 0.

Byte 15 Bits per a Max Value Data

> This field specifies the length in bits of the AE maximum value. This
> unit sets this field to 16d.

Byte 16 Bits per a Matrix Data

> This field specifies the length in bits of each matrix data. This unit
> sets this field to 0.

Byte 17 Bits per a Filter Data

> This field specifies the length in bits of each filter data. This unit
> sets this field to 0.

Byte 18 and 19 Bits per a Shading/Dark Current Data

> This field specifies the length in bits of each data for the shading
> correction coefficient/dark voltage correction coefficient.
>
> This unit sets this field to 16d.

Byte 20 and 21 Execute operation support 80

> This field specifies the function that is supported by operation code
> 8xh of EXECUTE command.
>
> This unit supports the initialization and the return to the origin.
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
> This unit supports the automatic execution of auto focus and the
> automatic execution of the shading white balance measurement.

<table style="width:99%;">
<colgroup>
<col style="width: 0%" />
<col style="width: 12%" />
<col style="width: 0%" />
<col style="width: 11%" />
<col style="width: 0%" />
<col style="width: 57%" />
<col style="width: 0%" />
<col style="width: 14%" />
</colgroup>
<tbody>
<tr>
<td colspan="2" style="text-align: center;">Byte</td>
<td colspan="2" style="text-align: center;">Bit</td>
<td colspan="2" style="text-align: center;">Operation</td>
<td colspan="2" style="text-align: center;">Value on this unit</td>
</tr>
<tr>
<td colspan="2" style="text-align: center;">22</td>
<td colspan="2" style="text-align: center;">0</td>
<td colspan="2" style="text-align: left;"><blockquote>
<p>Change Unit</p>
</blockquote></td>
<td colspan="2" style="text-align: center;">[0]</td>
</tr>
<tr>
<td colspan="2" style="text-align: center;"></td>
<td colspan="2" style="text-align: center;">1</td>
<td colspan="2" style="text-align: left;"><blockquote>
<p>AF auto execute</p>
</blockquote></td>
<td colspan="2" style="text-align: center;">[1]</td>
</tr>
<tr>
<td colspan="2" style="text-align: center;"></td>
<td colspan="2" style="text-align: center;">2</td>
<td colspan="2" style="text-align: left;"><blockquote>
<p>Calibration auto execute</p>
</blockquote></td>
<td colspan="2" style="text-align: center;">[1]</td>
</tr>
<tr>
<td colspan="2" style="text-align: center;"></td>
<td colspan="2" style="text-align: center;">3 to 7</td>
<td colspan="2" style="text-align: left;"><blockquote>
<p>Reserved</p>
</blockquote></td>
<td colspan="2" style="text-align: center;">[0]</td>
</tr>
<tr>
<td colspan="2" style="text-align: center;">23</td>
<td colspan="2" style="text-align: center;">0 to 7</td>
<td colspan="2" style="text-align: left;"><blockquote>
<p>Reserved</p>
</blockquote></td>
<td colspan="2" style="text-align: center;">[0]</td>
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
> This unit supports the shading measurement and the recording of the
> unit-specific data setting.

<table style="width:97%;">
<colgroup>
<col style="width: 0%" />
<col style="width: 2%" />
<col style="width: 10%" />
<col style="width: 0%" />
<col style="width: 2%" />
<col style="width: 9%" />
<col style="width: 0%" />
<col style="width: 2%" />
<col style="width: 54%" />
<col style="width: 0%" />
<col style="width: 2%" />
<col style="width: 12%" />
</colgroup>
<tbody>
<tr>
<td colspan="3" style="text-align: center;">Byte</td>
<td colspan="3" style="text-align: center;">Bit</td>
<td colspan="3" style="text-align: center;">Operation</td>
<td colspan="3" style="text-align: center;">Value on this unit</td>
</tr>
<tr>
<td colspan="3" style="text-align: center;">26</td>
<td colspan="3" style="text-align: center;">0</td>
<td colspan="3" style="text-align: center;"><blockquote>
<p>Setup Shading Data</p>
</blockquote></td>
<td colspan="3" style="text-align: center;">[1]</td>
</tr>
<tr>
<td colspan="3" style="text-align: center;"></td>
<td colspan="3" style="text-align: center;">1</td>
<td colspan="3" style="text-align: center;"><blockquote>
<p>Setup Dark Current Correction Data</p>
</blockquote></td>
<td colspan="3" style="text-align: center;">[0]</td>
</tr>
<tr>
<td colspan="3" style="text-align: center;"></td>
<td colspan="3" style="text-align: center;">2</td>
<td colspan="3" style="text-align: center;"><blockquote>
<p>Setup Offset Correction Data</p>
</blockquote></td>
<td colspan="3" style="text-align: center;">[0]</td>
</tr>
<tr>
<td colspan="3" style="text-align: center;"></td>
<td colspan="3" style="text-align: center;">3</td>
<td colspan="3" style="text-align: center;"><blockquote>
<p>Write Data On Device Dependence</p>
</blockquote></td>
<td colspan="3" style="text-align: center;">[1]</td>
</tr>
<tr>
<td colspan="3" style="text-align: center;"></td>
<td colspan="3" style="text-align: center;">4</td>
<td colspan="3" style="text-align: center;"><blockquote>
<p>Change of Auto Unload time</p>
</blockquote></td>
<td colspan="3" style="text-align: center;">[0]</td>
</tr>
<tr>
<td colspan="3" style="text-align: center;"></td>
<td colspan="3" style="text-align: center;">5 to 7</td>
<td colspan="3" style="text-align: center;"><blockquote>
<p>Reserved</p>
</blockquote></td>
<td colspan="3" style="text-align: center;">[0]</td>
</tr>
<tr>
<td colspan="3" style="text-align: center;">27</td>
<td colspan="3" style="text-align: center;">0 to 7</td>
<td colspan="3" style="text-align: left;"><blockquote>
<p>Reserved</p>
</blockquote></td>
<td colspan="3" style="text-align: center;">[0]</td>
</tr>
</tbody>
</table>

Byte 28 and 29 Execute operation support C0

> This field specifies the function that is supported by operation code
> Cxh of EXECUTE command.
>
> This unit supports the focus movement.

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
<td style="text-align: center;">[0]</td>
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

<table style="width:93%;">
<colgroup>
<col style="width: 7%" />
<col style="width: 3%" />
<col style="width: 7%" />
<col style="width: 4%" />
<col style="width: 7%" />
<col style="width: 42%" />
<col style="width: 7%" />
<col style="width: 12%" />
</colgroup>
<tbody>
<tr>
<td colspan="2" style="text-align: center;">Byte</td>
<td colspan="2" style="text-align: center;">Bit</td>
<td colspan="2" style="text-align: center;">Operation</td>
<td colspan="2">Value on this unit</td>
</tr>
<tr>
<td colspan="2" style="text-align: center;">30</td>
<td colspan="2" style="text-align: center;">0</td>
<td colspan="2" style="text-align: left;"><blockquote>
<p>Unload object</p>
</blockquote></td>
<td colspan="2" style="text-align: center;">[1]</td>
</tr>
<tr>
<td colspan="2" style="text-align: center;"></td>
<td colspan="2" style="text-align: center;">1</td>
<td colspan="2" style="text-align: left;"><blockquote>
<p>Load object</p>
</blockquote></td>
<td colspan="2" style="text-align: center;">[0]</td>
</tr>
<tr>
<td colspan="2" style="text-align: center;"></td>
<td colspan="2" style="text-align: center;">2</td>
<td colspan="2" style="text-align: left;"><blockquote>
<p>Absolute positioning</p>
</blockquote></td>
<td colspan="2" style="text-align: center;">[0]</td>
</tr>
<tr>
<td colspan="2" style="text-align: center;"></td>
<td colspan="2" style="text-align: center;">3</td>
<td colspan="2" style="text-align: left;"><blockquote>
<p>Relative positioning</p>
</blockquote></td>
<td colspan="2" style="text-align: center;">[0]</td>
</tr>
<tr>
<td colspan="2" style="text-align: center;"></td>
<td colspan="2" style="text-align: center;">4</td>
<td colspan="2" style="text-align: left;"><blockquote>
<p>Rotate</p>
</blockquote></td>
<td colspan="2" style="text-align: center;">[0]</td>
</tr>
<tr>
<td colspan="2" style="text-align: center;"></td>
<td colspan="2" style="text-align: center;">5</td>
<td colspan="2" style="text-align: left;"><blockquote>
<p>Reserved</p>
</blockquote></td>
<td colspan="2" style="text-align: center;">[0]</td>
</tr>
<tr>
<td colspan="2" style="text-align: center;"></td>
<td colspan="2" style="text-align: center;">6</td>
<td colspan="2" style="text-align: left;"><blockquote>
<p>Reserved</p>
</blockquote></td>
<td colspan="2" style="text-align: center;">[0]</td>
</tr>
<tr>
<td colspan="2" style="text-align: center;"></td>
<td colspan="2" style="text-align: center;">7</td>
<td colspan="2" style="text-align: left;"><blockquote>
<p>Reserved</p>
</blockquote></td>
<td colspan="2" style="text-align: center;">[0]</td>
</tr>
<tr>
<td colspan="2" style="text-align: center;">31</td>
<td colspan="2" style="text-align: center;">0 to 7</td>
<td colspan="2" style="text-align: left;"><blockquote>
<p>Reserved</p>
</blockquote></td>
<td colspan="2" style="text-align: center;">[0]</td>
</tr>
</tbody>
</table>

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
<td style="text-align: center;">[1]</td>
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
<td style="text-align: center;">[0]</td>
</tr>
<tr>
<td style="text-align: left;">Bit3</td>
<td style="text-align: left;">Scanned object exchangeable without
notice</td>
<td style="text-align: left;"><blockquote>
<p>The scanned object can be exchanged, but it is not possible to inform
the initiator that the object has been exchanged.</p>
</blockquote></td>
<td style="text-align: center;">[0]</td>
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
> This unit sets this field to 4 (1 kbyte).

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
**2-2-2-6. Additional address information page**

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
<td colspan="8" style="text-align: center;">Page code [C8h]</td>
</tr>
<tr>
<td style="text-align: center;">2</td>
<td colspan="8" style="text-align: center;">Reserved [0]</td>
</tr>
<tr>
<td style="text-align: center;">3</td>
<td colspan="8" style="text-align: center;">Page length [n-3]</td>
</tr>
<tr>
<td style="text-align: center;">4</td>
<td colspan="8" style="text-align: center;">Number of Images</td>
</tr>
<tr>
<td style="text-align: center;">5 to 8</td>
<td colspan="8" style="text-align: center;">(1) Left Address</td>
</tr>
<tr>
<td style="text-align: center;">9 to 12</td>
<td colspan="8" style="text-align: center;">(1) Top Address</td>
</tr>
<tr>
<td style="text-align: center;">13 to 16</td>
<td colspan="8" style="text-align: center;">(1) Width</td>
</tr>
<tr>
<td style="text-align: center;">17 to 20</td>
<td colspan="8" style="text-align: center;">(1) Length</td>
</tr>
<tr>
<td style="text-align: center;">:</td>
<td colspan="8" style="text-align: center;">:</td>
</tr>
<tr>
<td style="text-align: center;"><p>n-15</p>
<p>to n-12</p></td>
<td colspan="8" style="text-align: center;">((n-4)/16) Left Address</td>
</tr>
<tr>
<td style="text-align: center;"><p>n-11</p>
<p>to n-8</p></td>
<td colspan="8" style="text-align: center;">((n-4)/16) Top Address</td>
</tr>
<tr>
<td style="text-align: center;"><p>n-7</p>
<p>to n-4</p></td>
<td colspan="8" style="text-align: center;">((n-4)/16) Width</td>
</tr>
<tr>
<td style="text-align: center;"><p>n-3</p>
<p>to n</p></td>
<td colspan="8" style="text-align: center;">((n-4)/16) Length</td>
</tr>
</tbody>
</table>

\*1 When an invalid logical unit selection is performed

Byte 4 Number of Images

> This field specifies the number of images that are stored in the unit.
>
> In this unit, a value from 1 to 60 is set according to the attached
> holder.

Byte 5 to 8 Left Address

> This field specifies the X address at the left edge of the first
> image.

Byte 9 to 12 Left Address

> This field specifies the Y address at the top edge of the first image.

Byte 13 to 16 Width

> This field specifies the width of the first image in the X-axis
> direction.

Byte 17 to 20 Length

> This field specifies the length of the first image in the Y-axis
> direction.

Byte 21 and after

> The address information of the second and later images is stored.

The address information of 15 images can be stored in one page. When the
number of images exceeds 15, the address information of the image is
stored on the page next to page C8h.

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
<td colspan="8" style="text-align: center;">Page code [C9h]</td>
</tr>
<tr>
<td style="text-align: center;">2</td>
<td colspan="8" style="text-align: center;">Reserved [0]</td>
</tr>
<tr>
<td style="text-align: center;">3</td>
<td colspan="8" style="text-align: center;">Page length [n-3]</td>
</tr>
<tr>
<td style="text-align: center;">4 to7</td>
<td colspan="8" style="text-align: center;">(16) Left Address</td>
</tr>
<tr>
<td style="text-align: center;">8 to 11</td>
<td colspan="8" style="text-align: center;">(16) Top Address</td>
</tr>
<tr>
<td style="text-align: center;">12 to 15</td>
<td colspan="8" style="text-align: center;">(16) Width</td>
</tr>
<tr>
<td style="text-align: center;">16 to 19</td>
<td colspan="8" style="text-align: center;">(16) Length</td>
</tr>
<tr>
<td style="text-align: center;">:</td>
<td colspan="8" style="text-align: center;">:</td>
</tr>
<tr>
<td style="text-align: center;"><p>n-15</p>
<p>to n-12</p></td>
<td colspan="8" style="text-align: center;">((n-4)/16 + 15) Left
Address</td>
</tr>
<tr>
<td style="text-align: center;"><p>n-11</p>
<p>to n-8</p></td>
<td colspan="8" style="text-align: center;">((n-4)/16 + 15) Top
Address</td>
</tr>
<tr>
<td style="text-align: center;"><p>n-7</p>
<p>to n-4</p></td>
<td colspan="8" style="text-align: center;">((n-4)/16 + 15) Width</td>
</tr>
<tr>
<td style="text-align: center;"><p>n-3</p>
<p>to n</p></td>
<td colspan="8" style="text-align: center;">((n-4)/16 + 15) Length</td>
</tr>
</tbody>
</table>

\*1 When an invalid logical unit selection is performed

**  **
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
> measurement points set in byte 10.

**  
2-3. MODE SELECT (6) Command**

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

- Mode parameter of this unit

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
<p>[0]</p></td>
<td colspan="3" style="text-align: center;">Third party device ID</td>
<td style="text-align: center;"><p>Reserved</p>
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

The RESERVE UNIT command is used to reserve the logical units for
exclusive use of the initiator.

This command requests that the entire logical unit be reserved for the
exclusive use of the initiator until the reservation is superseded by
another valid RESERVE UNIT command from the initiator that made the
reservation, until released by a RELEASE UNIT command from the same
initiator, or released by the hard reset status or the power-on cycle.
It shall be permissible for the initiator that reserved a logical unit
to reserve a logical unit again.

**2-5. RELEASE UNIT Command**

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
<p>[0]</p></td>
<td colspan="3" style="text-align: center;">Third party device ID</td>
<td style="text-align: center;"><p>Reserved</p>
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

The RELEASE UNIT command is used to release the reservation of the
logical units that was made by the initiator which issued the command.

If a valid reservation exists, this unit shall release the reservation
and return GOOD status.

Only the initiator that made the reservation can release it. A command
that attempts to release the reservation that is not currently valid is
not regarded as an error. In this case, this unit shall return GOOD
status without changing any other reservation.

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
> The bit corresponding to the parameter that is not variable by the
> initiator is set to 0.
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
<td colspan="8" style="text-align: center;">Transfer length [0, 1, 2,
3]</td>
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
<col style="width: 38%" />
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
<td style="text-align: center;">When Multi Line Simultaneous Reading is
set</td>
<td style="text-align: center;"><p>MULTI LINE SIMULTANEOUS READING
PROCESS BY DRIVER</p>
<p>(The rearrangement processing during Multi Line Simultaneous Reading
is performed by the initiator.)</p>
<p>09h-80h-04h-01h</p></td>
<td style="text-align: center;">The initiator cooperative action
parameter is read by the READ command following the SCAN command and the
rearrangement processing is performed on the initiator side based on the
information. The initiator issues the SCAN command again after
performing the necessary operation.</td>
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
<td style="text-align: center;">When Thumbnail is set</td>
<td style="text-align: center;"><p>THUMBNAIL CREATED BY DRIVER</p>
<p>(The thumbnail image of the object to be scanned is created by the
initiator.)</p>
<p>09h-80h-01h-04h</p></td>
<td style="text-align: center;">The initiator cooperative action
parameter is read by the READ command following the SCAN command and the
thumbnail is created on the initiator side based on the information. The
initiator issues the SCAN command again after performing the necessary
operation.</td>
</tr>
<tr>
<td style="text-align: center;">When the pixel composition is 8 bits and
an odd value is set in the reading resolution</td>
<td style="text-align: center;"><p>TRUNCATED BY DRIVER</p>
<p>(The excess data that is sent during the odd width reading in 8 bits
is deleted by the initiator.)</p>
<p>09h-80h-06h-00h</p></td>
<td style="text-align: center;">The SCAN command is issued again. The
excess data is deleted on the initiator side by the READ command that is
issued following the SCAN command.</td>
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

**  **
**2-9. SET WINDOW Command**

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
value: 66d]</td>
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
<td colspan="8" style="text-align: right;">[Recommended value: 58d]
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
value: (58*the number of windows+8)d]</td>
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
<td colspan="8" style="text-align: right;">[Recommended value: (58*the
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
<td colspan="8" style="text-align: right;">[Recommended value: 58d]
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
3](The default is 2.)</td>
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
<p>X Resolution [666 to 4000]</p>
</blockquote></td>
</tr>
<tr>
<td style="text-align: center;">4, 5</td>
<td colspan="8" style="text-align: center;"><blockquote>
<p>Y Resolution [333 to 4000]</p>
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
> (1/Line Gap Count measure) of the maximum resolution.
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

<table style="width:67%;">
<colgroup>
<col style="width: 21%" />
<col style="width: 11%" />
<col style="width: 0%" />
<col style="width: 21%" />
<col style="width: 2%" />
<col style="width: 8%" />
<col style="width: 0%" />
</colgroup>
<tbody>
<tr>
<td colspan="3" style="text-align: center;">Set Window
specification</td>
<td colspan="4" style="text-align: center;">Scanning at the device</td>
</tr>
<tr>
<td colspan="3" style="text-align: center;">X resolution</td>
<td colspan="2" style="text-align: center;">Scanning resolution</td>
<td colspan="2" style="text-align: center;">Pitch</td>
</tr>
<tr>
<td colspan="3" style="text-align: center;">4000 to 2001</td>
<td colspan="2" style="text-align: center;">4000</td>
<td colspan="2" style="text-align: center;">1</td>
</tr>
<tr>
<td colspan="3" style="text-align: center;">2000 to 1334</td>
<td colspan="2" style="text-align: center;">2000</td>
<td colspan="2" style="text-align: center;">2</td>
</tr>
<tr>
<td colspan="3" style="text-align: center;">1333 to 1001</td>
<td colspan="2" style="text-align: center;">1333</td>
<td colspan="2" style="text-align: center;">3</td>
</tr>
<tr>
<td colspan="3" style="text-align: center;">1000 to 667</td>
<td colspan="2" style="text-align: center;">1000</td>
<td colspan="2" style="text-align: center;">4</td>
</tr>
<tr>
<td colspan="3" style="text-align: center;">666 to 334</td>
<td colspan="2" style="text-align: center;">666</td>
<td colspan="2" style="text-align: center;">6</td>
</tr>
<tr>
<td colspan="3" style="text-align: center;">333</td>
<td colspan="2" style="text-align: center;">333</td>
<td colspan="2" style="text-align: center;">12</td>
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
> The default is 01h. This field specifies the currently set value for
> the GET WINDOW command.
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
> This unit supports the normal scan, multiple reading scan, and the
> high-speed scan.
>
> The default is 02h. This field specifies the currently set value for
> the GET WINDOW command.

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

> This field specifies which ordering (pixel ordering, line ordering,
> plane ordering, or single-color three-line simultaneous reading) shall
> be used for reading. It also specifies whether the X and Y offsets
> include the CCD distance for the pixel ordering and the line ordering.
> A bit whose ordering is specified for reading is set to 1.
>
> This unit supports the line ordering without CCD distance and the
> single-color three-line simultaneous reading. The default is 2. This
> field specifies the currently set value for the GET WINDOW command.

|      |                             |
|:-----|:----------------------------|
| Bit0 | Pixel without CCD distance  |
| Bit1 | Line without CCD distance   |
| Bit2 | Plane                       |
| Bit3 | Reserved \[0\]              |
| Bit4 | Pixel with CCD distance     |
| Bit5 | Line with CCD distance      |
| Bit6 | 3 line Simultaneous reading |
| Bit7 | Reserved \[0\]              |

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
<td style="text-align: center;">R/S</td>
<td style="text-align: center;">2</td>
<td style="text-align: center;">16384</td>
<td style="text-align: center;">Not included</td>
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
<td style="text-align: center;">R</td>
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
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
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
<td style="text-align: center;">18</td>
<td style="text-align: center;">Included</td>
</tr>
<tr>
<td style="text-align: center;">88h</td>
<td style="text-align: left;">Boundary Information</td>
<td style="text-align: center;">R/S</td>
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
<td style="text-align: center;">R</td>
<td style="text-align: center;">4</td>
<td style="text-align: center;">2</td>
<td style="text-align: center;">Included</td>
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
<td style="text-align: left;">Reserved</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
</tr>
<tr>
<td style="text-align: center;">8Eh</td>
<td style="text-align: left;">Perforation information</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
</tr>
<tr>
<td style="text-align: center;">8Fh</td>
<td style="text-align: left;">Boundary Information Type2</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;">-</td>
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
> \*2 The valid number of pixels for CCD is 10000d.

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
<td>When the data type code is 03h, 80h, 81h, 84h, 85h, or 8Ch</td>
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

When the data type is 00h (image), in the READ command issued next to a
READ command, the data is transferred starting from the end of the image
data that is transferred by the previous READ command. When the data
type code is other than 00h, the data is transferred again starting from
the top of the data (the top of the header, if any).

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

**  
2-11-3. Image data transfer**

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

**2-11-3-1. Image data format**

The format of the transferred data changes variously depending on the
setting for the ASIC during the scanning.

Some of the examples of the one-line data formats in the typical setting
are shown below. The data in the following format is repeated as many
times as the number of scanning lines (sub-scanning).

1)  When the three colors are output in the order of R, G, and B in line
    ordering and transferred

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
    ordering and transferred for three-line reading

<table>
<colgroup>
<col style="width: 7%" />
<col style="width: 7%" />
<col style="width: 7%" />
<col style="width: 7%" />
<col style="width: 7%" />
<col style="width: 7%" />
<col style="width: 7%" />
<col style="width: 7%" />
<col style="width: 7%" />
<col style="width: 7%" />
<col style="width: 7%" />
<col style="width: 7%" />
<col style="width: 7%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: center;"><p>Line1</p>
<p>R1</p>
<p>H</p></td>
<td style="text-align: center;"><p>Line1</p>
<p>R1</p>
<p>L</p></td>
<td style="text-align: center;"><p>Line2</p>
<p>R1</p>
<p>H</p></td>
<td style="text-align: center;"><p>Line2</p>
<p>R1</p>
<p>L</p></td>
<td style="text-align: center;"><p>Line3</p>
<p>R1</p>
<p>H</p></td>
<td style="text-align: center;"><p>Line3</p>
<p>R1</p>
<p>L</p></td>
<td style="text-align: center;">…</td>
<td style="text-align: center;"><p>Line1</p>
<p>Rn</p>
<p>H</p></td>
<td style="text-align: center;"><p>Line1</p>
<p>Rn</p>
<p>L</p></td>
<td style="text-align: center;"><p>Line2</p>
<p>Rn</p>
<p>H</p></td>
<td style="text-align: center;"><p>Line2</p>
<p>Rn</p>
<p>L</p></td>
<td style="text-align: center;"><p>Line3</p>
<p>Rn</p>
<p>H</p></td>
<td style="text-align: center;"><p>Line3</p>
<p>Rn</p>
<p>L</p></td>
</tr>
</tbody>
</table>

<table>
<colgroup>
<col style="width: 3%" />
<col style="width: 7%" />
<col style="width: 7%" />
<col style="width: 7%" />
<col style="width: 7%" />
<col style="width: 7%" />
<col style="width: 7%" />
<col style="width: 7%" />
<col style="width: 7%" />
<col style="width: 7%" />
<col style="width: 7%" />
<col style="width: 7%" />
<col style="width: 7%" />
<col style="width: 7%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: center;"></td>
<td style="text-align: center;"><p>Line1</p>
<p>G1</p>
<p>H</p></td>
<td style="text-align: center;"><p>Line1</p>
<p>G1</p>
<p>L</p></td>
<td style="text-align: center;"><p>Line2</p>
<p>G1</p>
<p>H</p></td>
<td style="text-align: center;"><p>Line2</p>
<p>G1</p>
<p>L</p></td>
<td style="text-align: center;"><p>Line3</p>
<p>G1</p>
<p>H</p></td>
<td style="text-align: center;"><p>Line3</p>
<p>G1</p>
<p>L</p></td>
<td style="text-align: center;">…</td>
<td style="text-align: center;"><p>Line1</p>
<p>Gn</p>
<p>H</p></td>
<td style="text-align: center;"><p>Line1</p>
<p>Gn</p>
<p>L</p></td>
<td style="text-align: center;"><p>Line2</p>
<p>Gn</p>
<p>H</p></td>
<td style="text-align: center;"><p>Line2</p>
<p>Gn</p>
<p>L</p></td>
<td style="text-align: center;"><p>Line3</p>
<p>Gn</p>
<p>H</p></td>
<td style="text-align: center;"><p>Line3</p>
<p>Gn</p>
<p>L</p></td>
</tr>
</tbody>
</table>

<table>
<colgroup>
<col style="width: 7%" />
<col style="width: 7%" />
<col style="width: 7%" />
<col style="width: 7%" />
<col style="width: 7%" />
<col style="width: 7%" />
<col style="width: 6%" />
<col style="width: 7%" />
<col style="width: 7%" />
<col style="width: 7%" />
<col style="width: 7%" />
<col style="width: 7%" />
<col style="width: 7%" />
<col style="width: 7%" />
</colgroup>
<tbody>
<tr>
<td style="text-align: center;"></td>
<td style="text-align: center;"><p>Line1</p>
<p>B1</p>
<p>H</p></td>
<td style="text-align: center;"><p>Line1</p>
<p>B1</p>
<p>L</p></td>
<td style="text-align: center;"><p>Line2</p>
<p>B1</p>
<p>H</p></td>
<td style="text-align: center;"><p>Line2</p>
<p>B1</p>
<p>L</p></td>
<td style="text-align: center;"><p>Line3</p>
<p>B1</p>
<p>H</p></td>
<td style="text-align: center;"><p>Line3</p>
<p>B1</p>
<p>L</p></td>
<td style="text-align: center;">…</td>
<td style="text-align: center;"><p>Line1</p>
<p>Bn</p>
<p>H</p></td>
<td style="text-align: center;"><p>Line1</p>
<p>Bn</p>
<p>L</p></td>
<td style="text-align: center;"><p>Line2</p>
<p>Bn</p>
<p>H</p></td>
<td style="text-align: center;"><p>Line2</p>
<p>Bn</p>
<p>L</p></td>
<td style="text-align: center;"><p>Line3</p>
<p>Bn</p>
<p>H</p></td>
<td style="text-align: center;"><p>Line3</p>
<p>Bn</p>
<p>L</p></td>
</tr>
</tbody>
</table>

**2-11-4. LUT**

This unit does not support READ/SEND of the LUT.

**2-11-5. Initiator cooperative action parameter**

The READ command specifying data type code 87h is sent from the host
when one of the following scanning modes is selected: thumbnail
scanning, one-line multiple reading, three-line simultaneous reading, or
8-bit odd-width reading (in the main-scanning direction). When receiving
this command, this unit sends the data that conveys each information of
the unit.

The contents and the format of the data are shown below.

Operation type code

|  |  |  |
|:--:|----|:---|
| 1 | THUMBNAIL CREATED BY DRIVER | Thumbnail scanning |
| 2 | AVERAGING MULTIPLE READING BY DRIVER | Line averaging for multiple reading function |
| 4 | MULTI LINE SIMULTANEOUS READING PROCESS BY DRIVER | Rearrangement for three-line simultaneous reading function |
| 7 | CCD DATA CREATED BY DRIVER | CCD data reading |

Table 2-11-5-1 Format of THUMBNAIL CREATED BY DRIVER

|  |  |  |  |
|:--:|:--:|:--:|----|
| Byte | Name | Descriptions | Parameter |
| 0 | Type Code | Operation type code | 1 |
| 1 to 4 | Sense Data | Sense data that is set by the SCAN command | 09h-80h-01h-04h |
| 5, 6 | Bytes Per Line | The number of bytes per line | Depends on the scanning condition |
| 7, 8 | Entire Lines | The number of entire lines | Number of scanning lines\*Number of frames |
| 9 | Bits Per a Color of Dot | The number of bits per dot of one color | \[16d\] |
| 10, 11 | Lines Per an Image | The number of lines per image | The number of scanning lines |
| 12 | Reading Count Per a Line | Exposure counts per line | \- |
| 13 to 17 | Reserved | Reserved | 0 |

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

Table 2-11-5-3 Format of MULTI LINE SIMULTANEOUS READING PROCESS BY
DRIVER

|  |  |  |  |
|:--:|:--:|:--:|----|
| Byte | Name | Descriptions | Parameter |
| 0 | Type Code | Operation type code | 4 |
| 1 to 4 | Sense Data | Sense data that is set by the SCAN command | 09h-80h-04h-01h |
| 5, 6 | Bytes Per Line | The number of bytes per line | Depends on the scanning condition |
| 7, 8 | Entire Lines | The number of entire lines | The number of scanning lines |
| 9 | Bits Per a Color of Dot | The number of bits per dot of one color | Depends on the scanning condition |
| 10, 11 | Lines Per an Image | The number of lines per image | 0 |
| 12 | Reading Count Per a Line | Exposure counts per line | 0 |
| 13, 14 | Registration gap | Line Gap Count divided by Scanning pitch | Depends on the scanning condition |
| 15 to 17 | Reserved | Reserved | 0 |

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
| 12 | CCD Data Type of K Data | Type for CCD measurement of K color | \- |
| 13 to 17 | Reserved | Reserved | 0 |

Byte 5 to 12 CCD Data Type of color Data

> This field specifies the type that is used for the CCD measurement of
> each color.

**  
2-11-6. Boundary Information**

After the thumbnail scanning of the Brownie strip film, the coordinate
information of each frame is set in the unit by the host. The boundary
address (Top Address) specified by the host is represented by
inch\*maximum resolution (the number of lines in pitch 1).

<table style="width:100%;">
<colgroup>
<col style="width: 18%" />
<col style="width: 9%" />
<col style="width: 9%" />
<col style="width: 9%" />
<col style="width: 9%" />
<col style="width: 9%" />
<col style="width: 9%" />
<col style="width: 9%" />
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
<td colspan="8" style="text-align: center;">Address at the upper left of
the first frame in the sub-scanning direction</td>
</tr>
<tr>
<td style="text-align: center;">8 to 11</td>
<td colspan="8" style="text-align: center;">Address at the upper left of
the first frame in the main-scanning direction</td>
</tr>
<tr>
<td style="text-align: center;">12 to 15</td>
<td colspan="8" style="text-align: center;">Address at the lower right
of the first frame in the sub-scanning direction</td>
</tr>
<tr>
<td style="text-align: center;">16 to 19</td>
<td colspan="8" style="text-align: center;">Address at the lower right
of the first frame in the main-scanning direction</td>
</tr>
<tr>
<td style="text-align: center;">:</td>
<td colspan="8" style="text-align: center;">:</td>
</tr>
<tr>
<td style="text-align: center;"><p>n-15 to</p>
<p>n-12</p></td>
<td colspan="8" style="text-align: center;">Address at the upper left of
the mth frame in the sub-scanning direction</td>
</tr>
<tr>
<td style="text-align: center;"><p>n-11 to</p>
<p>n-8</p></td>
<td colspan="8" style="text-align: center;">Address at the upper left of
the mth frame in the main-scanning direction</td>
</tr>
<tr>
<td style="text-align: center;"><p>n-7 to</p>
<p>n-4</p></td>
<td colspan="8" style="text-align: center;">Address at the lower right
of the mth frame in the sub-scanning direction</td>
</tr>
<tr>
<td style="text-align: center;"><p>n-3 to</p>
<p>n</p></td>
<td colspan="8" style="text-align: center;">Address at the lower right
of the mth frame in the main-scanning direction</td>
</tr>
</tbody>
</table>

**2-11-7. Analog gain**

The gain value (magnification) of the analog gain is specified in the
floating decimal point format of ANSI.

The gain value is specified in order of the value set in the SET WINDOW.
However, since the default value (current value) is used when 0 is set,
the gain value when 0 is set is not included.

The number of parameters is the maximum number of parameters for the
analog gain on the Set Window function page of the Inquiry command.

|      |              |                                          |
|:----:|:------------:|:----------------------------------------:|
| Byte |  Parameter   |               Descriptions               |
| 0, 1 | Analog gain1 | The gain value when 1 is set 1.000000000 |
| 2, 3 | Analog gain2 |       The gain value when 2 is set       |

**2-11-8. WB exposure value**

The value decided by the measurement of the unit at the time of start-up
specifies the color according to the upper byte of the data type
qualifier, and 4-byte data is sent for each color.

**  
2-11-10. CCD data**

<table style="width:100%;">
<colgroup>
<col style="width: 44%" />
<col style="width: 6%" />
<col style="width: 7%" />
<col style="width: 7%" />
<col style="width: 7%" />
<col style="width: 7%" />
<col style="width: 7%" />
<col style="width: 6%" />
<col style="width: 7%" />
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
<td style="text-align: center;">：</td>
<td colspan="8" style="text-align: center;">：</td>
</tr>
<tr>
<td style="text-align: center;">2(m-2), 2(m-2)+1</td>
<td colspan="8" style="text-align: center;">The (m-1)th point data of
the first type in CCD first line</td>
</tr>
<tr>
<td style="text-align: center;">2(m-1), 2(m-1)+1</td>
<td colspan="8" style="text-align: center;">The mth point data of the
first type in CCD first line</td>
</tr>
<tr>
<td style="text-align: center;">2m, 2m+1</td>
<td colspan="8" style="text-align: center;">The first point data of the
second type in CCD first line</td>
</tr>
<tr>
<td style="text-align: center;">2(m+2), 2(m+2)+1</td>
<td colspan="8" style="text-align: center;">The second point data of the
second type in CCD first line</td>
</tr>
<tr>
<td style="text-align: center;">：</td>
<td colspan="8" style="text-align: center;">：</td>
</tr>
<tr>
<td style="text-align: center;">2(2m-1), 2(2m-1)+1</td>
<td colspan="8" style="text-align: center;">The mth point data of the
second type in CCD first line</td>
</tr>
<tr>
<td style="text-align: center;">2(2m), 2(2m)+1</td>
<td colspan="8" style="text-align: center;">The first point data of the
third type in CCD first line</td>
</tr>
<tr>
<td style="text-align: center;">：</td>
<td colspan="8" style="text-align: center;">：</td>
</tr>
<tr>
<td style="text-align: center;">2((n-1)m+(m-2)), 2((n-1)m+(m-2))+1</td>
<td colspan="8" style="text-align: center;">The (m-1)th point data of
the nth type in CCD first line</td>
</tr>
<tr>
<td style="text-align: center;">2((n-1)m+(m-1)), 2((n-1)m+(m-1))+1</td>
<td colspan="8" style="text-align: center;">The mth point data of the
nth type in CCD first line</td>
</tr>
<tr>
<td style="text-align: center;">2mn, 2mn+1</td>
<td colspan="8" style="text-align: center;">The first point data of the
first type in CCD second line</td>
</tr>
<tr>
<td style="text-align: center;">2(mn+1), 2(mn+1)+1</td>
<td colspan="8" style="text-align: center;">The second point data of the
first type in CCD second line</td>
</tr>
<tr>
<td style="text-align: center;">：</td>
<td colspan="8" style="text-align: center;">：</td>
</tr>
<tr>
<td style="text-align: center;">2(mn+(n-1)m+(m-2)),
2(mn+(n-1)m+(m-2))+1</td>
<td colspan="8" style="text-align: center;">The (m-1)th point data of
the nth type in CCD second line</td>
</tr>
<tr>
<td style="text-align: center;">2(mn+(n-1)m+(m-1)),
2(mn+(n-1)m+(m-1))+1</td>
<td colspan="8" style="text-align: center;">The mth point data of the
nth type in CCD second line</td>
</tr>
<tr>
<td style="text-align: center;">2(2mn), 2(2mn)+1</td>
<td colspan="8" style="text-align: center;">The first point data of the
first type in CCD third line</td>
</tr>
<tr>
<td style="text-align: center;">2(2mn+1), 2(2mn+1)+1</td>
<td colspan="8" style="text-align: center;">The second point data of the
first type in CCD third line</td>
</tr>
<tr>
<td style="text-align: center;">：</td>
<td colspan="8" style="text-align: center;">：</td>
</tr>
<tr>
<td style="text-align: center;">2(2mn+(n-1)m+(m-2)),
2(2mn+(n-1)m+(m-2))+1</td>
<td colspan="8" style="text-align: center;">The (m-1)th point data of
the nth type in CCD third line</td>
</tr>
<tr>
<td style="text-align: center;">2(2mn+(n-1)m+(m-1)),
2(2mn+(n-1)m+(m-1))+1</td>
<td colspan="8" style="text-align: center;">The mth point data of the
nth type in CCD third line</td>
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
<tr>
<td style="text-align: center;"><blockquote>
<p>When the operation is not terminated normally</p>
</blockquote></td>
<td style="text-align: center;"><p>LOGICAL UNIT NOT READY, CAUSE NOT
REPORTABLE</p>
<p>(The internal mechanical error occurred.)</p>
<p>02h-04h-02h-00h</p></td>
<td style="text-align: center;">The command terminates with the CHECK
CONDITION status for the TEST UNIT READY command that is received after
the operation is terminated.</td>
</tr>
<tr>
<td style="text-align: center;">When the EXECUTE command is received
before the operation parameter is set by the SET PARAMETER command</td>
<td style="text-align: center;"><p>COMMAND SEQUENCE ERROR</p>
<p>(The EXECUTE command is received before the parameter is set by the
SET PARAMETER command.)</p>
<p>05h-2Ch-00h-00h</p></td>
<td style="text-align: center;">The command terminates with the CHECK
CONDITION status.</td>
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

- The first setting value specifies the absolute address value when the
  specified operation code needs the address parameter.

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

- The second setting value field specifies the absolute address value
  when the specified operation code needs two address parameters.

> When AF (auto focusing) is performed (code A0h, A1h), the address on
> the medium where AF is performed in the sub-scanning direction is
> specified. Zero in this field specifies the address zero.

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
<td style="text-align: center;">This unit is initialized in the same
manner as that of power ON.</td>
<td style="text-align: center;">None</td>
<td style="text-align: center;">Yes</td>
</tr>
<tr>
<td style="text-align: center;">81h</td>
<td style="text-align: center;">Return to the origin</td>
<td style="text-align: center;">Return to the origin</td>
<td style="text-align: center;">None</td>
<td style="text-align: center;">Yes</td>
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
<td style="text-align: center;">92h</td>
<td style="text-align: center;">Auto Calibration</td>
<td style="text-align: center;">Automatic calibration execution
ON/OFF</td>
<td style="text-align: center;">1st Val</td>
<td style="text-align: center;">Yes</td>
</tr>
<tr>
<td style="text-align: center;">A0h</td>
<td style="text-align: center;">Auto Focus</td>
<td style="text-align: center;">Performs the auto focus</td>
<td style="text-align: center;">1st Val, 2nd Val</td>
<td style="text-align: center;">Yes</td>
</tr>
<tr>
<td style="text-align: center;">B0h</td>
<td style="text-align: center;">Setup Shading Data</td>
<td style="text-align: center;">Performs the shading measurement</td>
<td style="text-align: center;">None</td>
<td style="text-align: center;">Yes</td>
</tr>
<tr>
<td style="text-align: center;">C1h</td>
<td style="text-align: center;">Focus Move</td>
<td style="text-align: center;">Moves the scan block in the AF
direction</td>
<td style="text-align: center;">1st Val</td>
<td style="text-align: center;">Yes</td>
</tr>
<tr>
<td style="text-align: center;">D0h</td>
<td style="text-align: center;">Unload object</td>
<td style="text-align: center;">Unloads the object</td>
<td style="text-align: center;">None</td>
<td style="text-align: center;">Yes</td>
</tr>
</tbody>
</table>

1st Val: First setting value

2nd Val: Second setting value

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
<td style="text-align: center;"><p>Opera-tion</p>
<p>code</p></td>
<td><p>Color specifi-cation</p>
<p>(Color)</p></td>
<td style="text-align: center;"><p>First setting value</p>
<p>(1st Val)</p></td>
<td style="text-align: center;"><p>Second setting value</p>
<p>(2nd Val)</p></td>
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
<td style="text-align: center;">92h</td>
<td style="text-align: center;">-</td>
<td style="text-align: center;"><p>Automatic calibration execution</p>
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
<td style="text-align: center;">When the RECEIVE DIAGNOSTIC RESULTS
command is received independently</td>
<td style="text-align: center;"><p>COMMAND SEQUENCE ERROR</p>
<p>05h-2Ch-00h-00h</p></td>
<td style="text-align: center;">The command terminates with the CHECK
CONDITION status.</td>
</tr>
</tbody>
</table>
