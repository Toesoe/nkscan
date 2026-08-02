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

+:------------:+:-----------:+:----------------:+:----------------------:+
| Name         | Transfer    | Transfer         | Uses                   |
|              | type        | direction        |                        |
+--------------+-------------+------------------+------------------------+
| End point 0  | Control     | Initiator -\>    | Transmission/reception |
|              | IN/OUT      | This unit/       | of the standard        |
|              |             |                  | descriptor             |
|              |             | This unit -\>    |                        |
|              |             | Initiator        |                        |
+--------------+-------------+------------------+------------------------+
| End point 1  | Bulk OUT    | Initiator -\>    | Transmission of the    |
|              |             | This unit        | data/ command          |
+--------------+-------------+------------------+------------------------+
| End point 2  | Bulk IN     | This unit -\>    | Reception of the data/ |
|              |             | Initiator        | command                |
+--------------+-------------+------------------+------------------------+
| End point 3  | Interrupt   | This unit -\>    | Not used in this unit  |
|              | IN          | Initiator        |                        |
+--------------+-------------+------------------+------------------------+

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

  ----------------- ------------ -----------------------------------------
        Phase           Code                      Status

      No phase          00h        Nothing is received (a command can be
                                               transmitted).

       STATUS           01h                   Status IN phase

      DATA OUT          02h                   Data OUT phase

       DATA IN          03h                    Data IN phase

        BUSY            04h          A command is being executed (the
                                     processing that is being executed
                                                continues).
  ----------------- ------------ -----------------------------------------

**\
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

**\
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

**\
1-1-3. Commands of this unit**

> The commands that are executed by this unit are shown below.

Table 1-1-3-1 List of the commands of this unit

+:---------------------------:+:---------:+:------:+:----------------:+
| Command name                | Operation | Type   | Phase transition |
|                             | code      |        |                  |
+-----------------------------+-----------+--------+------------------+
| > TEST UNIT READY           | 00h       | M      | > C - S          |
+-----------------------------+-----------+--------+------------------+
| > INQUIRY                   | 12h       | M      | > C - Din - S    |
+-----------------------------+-----------+--------+------------------+
| > MODE SELECT (6)           | 15h       | O      | > C - Dout - S   |
+-----------------------------+-----------+--------+------------------+
| > MODE SENSE (6)            | 1Ah       | O      | > C - Din - S    |
+-----------------------------+-----------+--------+------------------+
| > SCAN                      | 1Bh       | M      | > C - S          |
+-----------------------------+-----------+--------+------------------+
| > RECEIVER DIAGNOSTIC       | 1Ch       | M      | > C - S          |
| > RESULTS                   |           |        |                  |
+-----------------------------+-----------+--------+------------------+
| > SEND DIAGNOSTIC           | 1Dh       | M      | > C - S          |
+-----------------------------+-----------+--------+------------------+
| > SET WINDOW                | 24h       | M      | > C - Dout - S   |
+-----------------------------+-----------+--------+------------------+
| > GET WINDOW                | 25h       | O      | > C - Din - S    |
+-----------------------------+-----------+--------+------------------+
| > READ                      | 28h       | M      | > C - Din - S    |
+-----------------------------+-----------+--------+------------------+
| > SEND                      | 2Ah       | O      | > C - Dout - S   |
+-----------------------------+-----------+--------+------------------+
| > ABORT                     | C0h       | V      | > C - S          |
+-----------------------------+-----------+--------+------------------+
| > EXECUTE                   | C1h       | V      | > C - S          |
+-----------------------------+-----------+--------+------------------+
| > SET PARAMETER             | E0h       | V      | > C - Dout - S   |
+-----------------------------+-----------+--------+------------------+
| > GET PARAMETER             | E1h       | V      | > C - Din - S    |
+-----------------------------+-----------+--------+------------------+

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

+:-----------------------:+:----------:+:----------:+:----------:+:----------:+:----------:+:----------:+:----------:+:------------------------:+:-------------:+:-------------:+
| Bit of the status byte                                                                                             | Status                                                   |
+-------------------------+------------+------------+------------+------------+------------+------------+------------+--------------------------+-------------------------------+
| 7                       | 6          | 5          | 4          | 3          | 2          | 1          | 0          |                          |                               |
+-------------------------+------------+------------+------------+------------+------------+------------+------------+--------------------------+-------------------------------+
| R                       | R          | 0          | 0          | 0          | 0          | 0          | R          | > GOOD                   | \[00h\]                       |
+-------------------------+------------+------------+------------+------------+------------+------------+------------+--------------------------+-------------------------------+
| R                       | R          | 0          | 0          | 0          | 0          | 1          | R          | > CHECK CONDITION        | \[02h\]                       |
+-------------------------+------------+------------+------------+------------+------------+------------+------------+--------------------------+---------------+---------------+
| > Key: R - Reserved bit (set to 0)                                                                                                                            |               |
+---------------------------------------------------------------------------------------------------------------------------------------------------------------+---------------+

**1-1-5-2. Format of the status**

> The status and the sense data are synthesized in the status phase and
> output. The format is shown below. The 8-byte status data is always
> transmitted.
>
> The status code in table 1-1-5-1 is set in byte 0. The sense data in
> table 4-1-1 is set for the sense key, ASC, ASCQ, and TSC in byte 1 to
> 4.

Table 1-1-5-2 Format of the status

+------------:+:-------:+:-------------------:+:-------------------:+:-------------------:+:-------------------:+:--------:+:--------:+:--------:+:--------:+
| Bit         | 7       | 6                   | 5                   | 4                                         | 3        | 2        | 1        | 0        |
|             |         |                     |                     |                                           |          |          |          |          |
| Byte        |         |                     |                     |                                           |          |          |          |          |
+-------------+---------+---------------------+---------------------+-------------------------------------------+----------+----------+----------+----------+
| 0           | \[0\]                         | Status                                                                                           | \[0\]    |
+-------------+-------------------------------+-------------------------------------------+------------------------------------------------------+----------+
| 1           | \[0\]                                                                     | Sense key                                                       |
+-------------+---------+-----------------------------------------------------------------+-----------------------------------------------------------------+
| 2                     | ASC                                                                                                                               |
+-----------------------+-----------------------------------------------------------------------------------------------------------------------------------+
| 3                     | ASCQ                                                                                                                              |
+-----------------------+-----------------------------------------------------------------------------------------------------------------------------------+
| 4                     | TSC                                                                                                                               |
+-----------------------+-----------------------------------------------------------------------------------------------------------------------------------+
| 5                     | Reserved \[00h\]                                                                                                                  |
+-----------------------+-----------------------------------------------------------------------------------------------------------------------------------+
| 6                     | Reserved \[00h\]                                                                                                                  |
+-----------------------+-----------------------------------------------------------------------------------------------------------------------------------+
| 7                     | Reserved \[00h\]                                                                                                                  |
+-----------------------+-----------------------------------------------------------------------------------------------------------------------------------+

**\
1-1-6. USB-specific additional specifications**

**1-1-6-1. Standard device requests**

> The standard device requests are shown below.

Table 1-1-6-1-1 Standard device request

  ------------------------ ------- ------------------------ ---------------
         ｂRequest          Value          Meaning          Support of this
                                                                 unit

         GET_STATUS           0       Status acquisition          Yes

       CLEAR_FEATURE          1       Function clearance          Yes

  Reserved for future use     2            Reserved              Stall

        SET_FEATURE           3        Function setting           Yes

  Reserved for future use     4            Reserved              Stall

        SET_ADDRESS           5        Address setting            Yes

       GET_DESCRIPTOR         6     Descriptor acquisition        Yes

       SET_DESCRIPTOR         7       Descriptor setting         Stall

     GET_CONFIGURATION        8         Configuration             Yes
                                         acquisition        

     SET_CONFIGURATION        9     Configuration setting         Yes

       GET_INTERFACE         10     Interface acquisition         Yes

       SET_INTERFACE         11       Interface setting           Yes

        SYNCH_FRAME          12     Synchronization frame        Stall
  ------------------------ ------- ------------------------ ---------------

Table 1-1-6-1-2 Descriptor type

  ---------------------- ------------------------------------------------
           Type                               Value

            1                                 DEVICE

            2                             CONFIGURATION

            3                                 STRING

            4                               INTERFACE

            5                                ENDPOINT

            6                            DEVICE_QUALIFIER

            7                       OTHER_SPEED_CONFIGURATION

            8                            INTERFACE_POWER
  ---------------------- ------------------------------------------------

> Remarks: The upper byte of the value indicates the descriptor type and
> the lower byte indicates the string descriptor index.

**\
1-1-6-2. Device descriptors in this unit**

> The lists of the descriptors for GET_DESCRIPTOR in this unit are shown
> below.

Table 1-1-6-2-1 DEVICE descriptor

  ------- ------- -------------------------------------- -------------------
   Byte    Size                    Item                       Set value

     0       1           Size of this descriptor             12h (fixed)

     1       1          Type of DEVICE descriptor            01h (fixed)

     2       2          Release number of the USB               0200h
                          specifications (2.00)          

     4       1                  Class code                  FFh (vendor)

     5       1                Sub-class code                     FFh

     6       1                Protocol code                      FFh
                                                          (vendor-specific)

     7       1      Maximum buffer size of end point 0     40h (64 bytes)

     8       2                  Vendor ID                       04B0h

    10       2                  Product ID                      4002h

    12       2            Device release number                 xxxxh

    14       1    Index to the string descriptor of the          01h
                               manufacturer              

    15       1        Index to the string descriptor             02h
                         representing the product        

    16       1        Index to the string descriptor             00h
                  representing the product number of the 
                                  device                 

    17       1      The number that can be configured            01h
  ------- ------- -------------------------------------- -------------------

Table 1-1-6-2-2 CONFIGURATION descriptor

  ------- ------- -------------------------------------- -----------------
   Byte    Size                    Item                      Set value

     0       1           Size of this descriptor            09h (fixed)

     1       1               Descriptor type                02h (fixed)

     2       2      Length of the entire configuration         0020h

     4       1       The number of interfaces of the            01h
                              configuration              

     5       1     Configuration selection argument in          01h
                                SetConfig                

     6       1    Configuration string descriptor index         00h

     7       1        Configuration characteristics       C0h (self power
                                                           supply only)

     8       1      Maximum bus power consumption (in       01h (2 mA)
                              units of 2 mA)             
  ------- ------- -------------------------------------- -----------------

Table 1-1-6-2-3 INTERFACE descriptor

+:-----:+:----:+:-----------------------------------:+:-----------------:+
| Byte  | Size | Item                                | Set value         |
+-------+------+-------------------------------------+-------------------+
| 0     | 1    | Size of this descriptor             | 09h (fixed)       |
+-------+------+-------------------------------------+-------------------+
| 1     | 1    | Descriptor type                     | 04h (fixed)       |
+-------+------+-------------------------------------+-------------------+
| 2     | 1    | Number of this interface in the     | 00h               |
|       |      | configuration                       |                   |
+-------+------+-------------------------------------+-------------------+
| 3     | 1    | Substitute selection argument for   | 00h               |
|       |      | SetInterface                        |                   |
+-------+------+-------------------------------------+-------------------+
| 4     | 1    | The number of end points of the     | 02h               |
|       |      | interface                           |                   |
|       |      |                                     |                   |
|       |      | (End point 0 is not included.)      |                   |
+-------+------+-------------------------------------+-------------------+
| 5     | 1    | Class code                          | FFh (vendor)      |
+-------+------+-------------------------------------+-------------------+
| 6     | 1    | Sub-class code                      | FFh               |
+-------+------+-------------------------------------+-------------------+
| 7     | 1    | Protocol code                       | FFh               |
|       |      |                                     | (vendor-specific) |
+-------+------+-------------------------------------+-------------------+
| 8     | 1    | Index to the string descriptor of   | 00h               |
|       |      | this interface                      |                   |
+-------+------+-------------------------------------+-------------------+

Table 1-1-6-2-4 ENDPOINT descriptor

+:-----:+:----:+:----:+:-----------------------:+:------------:+:------------:+
| End   | Byte | Size | Item                    | Set value                   |
|       |      |      |                         |                             |
| point |      |      |                         |                             |
|       |      |      |                         +--------------+--------------+
|       |      |      |                         | 2.0          | 1.1          |
+-------+------+------+-------------------------+--------------+--------------+
|       | 0    | 1    | Size of this descriptor | 07h (fixed)  | 07h (fixed)  |
+-------+------+------+-------------------------+--------------+--------------+
| 1     | 1    | 1    | Descriptor type         | 05h (fixed)  | 05h (fixed)  |
+-------+------+------+-------------------------+--------------+--------------+
|       | 2    | 1    | End point               | 01h (OUT)    | 01h (OUT)    |
|       |      |      | address/direction       |              |              |
+-------+------+------+-------------------------+--------------+--------------+
|       | 3    | 1    | Attribute (transfer     | 02h (bulk)   | 02h (bulk)   |
|       |      |      | type)                   |              |              |
+-------+------+------+-------------------------+--------------+--------------+
|       | 4    | 2    | Maximum packet size     | 0200h        | 0040h (64    |
|       |      |      |                         |              | bytes)       |
|       |      |      |                         | (512 bytes)  |              |
+-------+------+------+-------------------------+--------------+--------------+
|       | 6    | 1    | Polling interval (in    | 00h          | 00h          |
|       |      |      | units of ms)            |              |              |
+-------+------+------+-------------------------+--------------+--------------+
|       | 0    | 1    | Size of this descriptor | 07h (fixed)  | 07h (fixed)  |
+-------+------+------+-------------------------+--------------+--------------+
| 2     | 1    | 1    | Descriptor type         | 05h (fixed)  | 05h (fixed)  |
+-------+------+------+-------------------------+--------------+--------------+
|       | 2    | 1    | End point               | 82h (IN)     | 82h (IN)     |
|       |      |      | address/direction       |              |              |
+-------+------+------+-------------------------+--------------+--------------+
|       | 3    | 1    | Attribute (transfer     | 02h (bulk)   | 02h (bulk)   |
|       |      |      | type)                   |              |              |
+-------+------+------+-------------------------+--------------+--------------+
|       | 4    | 2    | Maximum packet size     | 0200h        | 0040h (64    |
|       |      |      |                         |              | bytes)       |
|       |      |      |                         | (512 bytes)  |              |
+-------+------+------+-------------------------+--------------+--------------+
|       | 6    | 1    | Polling interval (in    | 00h          | 00h          |
|       |      |      | units of ms)            |              |              |
+-------+------+------+-------------------------+--------------+--------------+

Table 1-1-6-2-5 Example of the STRING descriptor

+:---------------------:+:---------:+:---------:+:----------:+:---------------------------:+:-----------------------:+
| Request command from the host                 | Response from the device                                           |
+-----------------------+-----------+-----------+------------+-----------------------------+-------------------------+
| Value                 | Index/    | Requested size         | Contents                    | Remarks                 |
|                       |           |                        |                             |                         |
|                       | LANGID    | (N+2)                  | (hex)                       |                         |
+-----------------------+-----------+------------------------+-----------------------------+-------------------------+
| 0300h                 | 0000h     | 04h                    | \[04 03 09 04\]             | LANGID=0409h            |
+-----------------------+-----------+------------------------+-----------------------------+-------------------------+
| 0301h                 | 0409h     | 04h                    | > "N"                       | First character of      |
|                       |           |                        |                             | Nikon                   |
|                       |           |                        | \[0C 03 4E 00\]             |                         |
+-----------------------+-----------+------------------------+-----------------------------+-------------------------+
| 0301h                 | 0409h     | 0Ch                    | > "Nikon"                   | Manufacturer            |
|                       |           |                        |                             |                         |
|                       |           |                        | \[0C 03 4E 00 69 00 6B 00   |                         |
|                       |           |                        | 6F 00 6E 00\]               |                         |
+-----------------------+-----------+------------------------+-----------------------------+-------------------------+
| 0302h                 | 0409h     | 04h                    | > "L"                       | Product name            |
|                       |           |                        |                             |                         |
|                       |           |                        | \[16 03 4C 00\]             | First character of the  |
|                       |           |                        |                             | model name              |
+-----------------------+-----------+------------------------+-----------------------------+-------------------------+
| 0302h                 | 0409h     | 10h                    | > "LS-5000 ED"              | Model name              |
|                       |           |                        |                             |                         |
|                       |           |                        | \[16 03 4C 00 53 00 2D 00   |                         |
|                       |           |                        | 35 00 30 00                 |                         |
|                       |           |                        |                             |                         |
|                       |           |                        | 30 00 30 00 20 00 45 00 44  |                         |
|                       |           |                        | 00\]                        |                         |
+-----------------------+-----------+------------------------+-----------------------------+-------------------------+
| 0303h                 | 0409h     | 04h                    | > "x"                       | First character of the  |
|                       |           |                        |                             | version                 |
|                       |           |                        |                             |                         |
|                       |           |                        |                             | (Only when the serial   |
|                       |           |                        |                             | No. is written)         |
+-----------------------+-----------+------------------------+-----------------------------+-------------------------+
| 0303h                 | 0409h     | 12h                    | > "xxxxxxxx"                | Version of the model    |
|                       |           |                        |                             |                         |
|                       |           |                        |                             | Product number of the   |
|                       |           |                        |                             | device                  |
|                       |           |                        |                             |                         |
|                       |           |                        |                             | (Only when the serial   |
|                       |           |                        |                             | No. is written)         |
+-----------------------+-----------+------------------------+-----------------------------+-------------------------+

Table 1-1-6-2-6 DEVICE_QUALIFIER descriptor

  -------- ------- -------------------------------------- -----------------
    Byte    Size                    Item                      Set value

     0        1           Size of this descriptor                0Ah

     1        1               Descriptor type                    06h

     2        2          Release number of the USB              0200h
                            specifications (2.0)          

     4        1                  Class code                      FFh

     5        1                Sub-class code                    FFh

     6        1                Protocol code                     FFh

     7        1          Maximum packet size of EP0              40h

     8        1      The number that can be configured           01h

     9        1                   Reserved                       00h
  -------- ------- -------------------------------------- -----------------

Table 1-1-6-2-7 OTHER_SPEED_CONFIGURATION descriptor

  ------- ------- -------------------------------------- -----------------
   Byte    Size                    Item                      Set value

     0       1           Size of this descriptor                09h

     1       1               Descriptor type                    02h

     2       2      Length of the entire configuration         0020h

     4       1       The number of interfaces of the            01h
                              configuration              

     5       1         Argument for selecting this              01h
                              configuration              

     6       1        Index to the string descriptor            00h

     7       1     Specification of each characteristic         C0h
                    (self power supply/remote wake-up)   

     8       1        Maximum bus power consumption             01h
  ------- ------- -------------------------------------- -----------------

**\
2. COMMAND EXPLANATIONS**

Each command is explained below.

In the explanations, the common error responses are as shown in the
table below.

+:------------:+:-----------------------------:+:---------------------:+
| Common error | Sense data                    | Remarks               |
+--------------+-------------------------------+-----------------------+
| 1            | INVALID FIELD IN CDB          | Terminates with CHECK |
|              |                               | CONDITION status.     |
|              | (Some illegal data exists in  |                       |
|              | the CDB.)                     |                       |
|              |                               |                       |
|              | 05h-24h-00h-00h               |                       |
+--------------+-------------------------------+-----------------------+
| 2            | INVALID FIELD IN PARAMETER    | Terminates with CHECK |
|              | LIST                          | CONDITION status.     |
|              |                               |                       |
|              | (Some illegal data exists in  |                       |
|              | the parameter.)               |                       |
|              |                               |                       |
|              | 05h-26h-00h-00h               |                       |
+--------------+-------------------------------+-----------------------+

Other error responses are explained individually in the explanations
below.

The values in \[ \] in the table show the permissible values or
recommended values of this unit in the command description block and in
the parameter, or the values that are returned by this unit in the
response data.

**2-1. TEST UNIT READY Command**

Table 2-1-1 TEST UNIT READY command

+-------:+:----------:+:----------:+:----------:+:----------:+:----------:+:----------:+:----------:+:----------:+
| Bit    | 7          | 6          | 5          | 4          | 3          | 2          | 1          | 0          |
|        |            |            |            |            |            |            |            |            |
| Byte   |            |            |            |            |            |            |            |            |
+--------+------------+------------+------------+------------+------------+------------+------------+------------+
| 0      | Operation code \[00h\]                                                                                |
+--------+--------------------------------------+----------------------------------------------------------------+
| 1      | Logical unit number                  | Reserved                                                       |
|        |                                      |                                                                |
|        | \[0\]                                | \[0\]                                                          |
+--------+--------------------------------------+----------------------------------------------------------------+
| 2 to 4 | Reserved \[0\]                                                                                        |
+--------+-------------------------------------------------------------------------------------------------------+
| 5      | Control byte \[0\]                                                                                    |
+--------+-------------------------------------------------------------------------------------------------------+

The TEST UNIT READY command provides a means to check if the logical
unit is ready.

Table 2-1-2 shows the responses corresponding to the TEST UNIT READY
command. A response that has higher priority (RESERVATION CONFLICT, for
example) may be made.

Table 2-1-2 Preferred Test Unit Ready Responses

+:----------:+:-------------------------------------------------------:+
| Status     | Sense code                                              |
+------------+---------------------------------------------------------+
| GOOD       | No Additional Sense Information                         |
|            |                                                         |
|            | 00h-00h-00h-00h (Common: No error)                      |
+------------+---------------------------------------------------------+
| Check      | Logical Unit Not Supported                              |
| Condition  |                                                         |
|            | > 05h-25h-00h-00h (Common: An LUN other than 0 was      |
|            | > specified.)                                           |
+------------+---------------------------------------------------------+
| Check      | Logical Unit Is In Process Of Becoming Ready            |
| Condition  |                                                         |
|            | > 02h-04h-01h-00h (Common: During the execution of the  |
|            | > operation activation command)                         |
|            | >                                                       |
|            | > 02h-04h-01h-01h (MA-21: During the adapter            |
|            | > initialization operation)                             |
|            | >                                                       |
|            | > (Other than MA-21: During the adapter initialization  |
|            | > operation or during loading/ejection of the object to |
|            | > be scanned)                                           |
|            | >                                                       |
|            | > 02h-04h-01h-02h (Common: During the measurement of    |
|            | > the correction data)                                  |
|            | >                                                       |
|            | > 02h-04h-01h-03h (MA-21: During the execution of       |
|            | > operation for loading the object to be scanned)       |
|            | >                                                       |
|            | > 02h-04h-01h-04h (Common: During the execution of      |
|            | > automatic shading or white balance measurement)       |
+------------+---------------------------------------------------------+
| Check      | Logical Unit Not Ready, Cause Not Reportable            |
| Condition  |                                                         |
|            | > 02h-04h-02h-00h (Common: The internal mechanical      |
|            | > error occurred.)                                      |
+------------+---------------------------------------------------------+
| Check      | Logical Unit Not Ready, Initializing Command Required   |
| Condition  |                                                         |
|            | > 02h-04h-00h-00h (The initialization is not complete   |
|            | > because an object is inserted at the time of power    |
|            | > ON.)                                                  |
+------------+---------------------------------------------------------+
| Check      | Logical Unit Not Ready, Manual Intervention Required    |
| Condition  |                                                         |
|            | 02h-04h-03h-00h (Common: The adapter is ejected.)       |
|            |                                                         |
|            | > 02h-04h-03h-01h (IA-20: The LL door is not completely |
|            | > opened when the 240 adapter is attached.)             |
|            | >                                                       |
|            | > 02h-04h-03h-02h (Common: Undefined adapter)           |
|            | >                                                       |
|            | > 02h-04h-03h-03h (SA-30: The film of 6 frames or more  |
|            | > is loaded with the film gate closed)                  |
|            | >                                                       |
|            | > 02h-04h-03h-04h (SA-21/SA-30: The adapter is pulled   |
|            | > out a little in the locked status.)                   |
+------------+---------------------------------------------------------+
| Check      | Logical Unit Does Not Respond To Selection              |
| Condition  |                                                         |
|            | > 02h-05h-00h-00h (Common: The operation is possible,   |
|            | > but the initialization operation in the unit is not   |
|            | > completed because the power is just turned ON.)       |
+------------+---------------------------------------------------------+
| Check      | Medium Not Present                                      |
| Condition  |                                                         |
|            | > 02h-3Ah-00h-00h (SF-210: The loading command is sent  |
|            | > without an object to be scanned.)                     |
|            | >                                                       |
|            | > 02h-3Ah-00h-01h (MA-21: (a) only) (IA-20: (a), (b),   |
|            | > (c), or (d))                                          |
|            | >                                                       |
|            | > (Other: (a), (b), or (c))                             |
|            |                                                         |
|            | (a) A medium is not supplied to the adapter.            |
|            |                                                         |
|            | (b) The film is ejected when the power supply is turned |
|            |     ON or the adapter is exchanged.                     |
|            |                                                         |
|            | (c) The medium is ejected by the eject command.         |
|            |                                                         |
|            | (d) The LL door is opened, but the loading switch is    |
|            |     not ON.                                             |
|            |                                                         |
|            | > 02h-3Ah-00h-03h (SA-21/SA-30: Reading cannot be       |
|            | > performed because a film that is out of standard is   |
|            | > inserted.)                                            |
|            | >                                                       |
|            | > 02h-3Ah-00h-04h (The frame position of a larger       |
|            | > number than the number of frames in the inserted film |
|            | > is specified.)                                        |
+------------+---------------------------------------------------------+
| Check      | 06h-xxh-xxh-xxh                                         |
| Condition  |                                                         |
|            | > Unit Attention                                        |
+------------+---------------------------------------------------------+
| Check      | Data Phase Error                                        |
| Condition  |                                                         |
|            | > 0Bh-4Bh-00h-00h (Common: Unexpected error during Data |
|            | > Phase)                                                |
+------------+---------------------------------------------------------+
| Check      | Overlapped Commands Attempted                           |
| Condition  |                                                         |
|            | > 0Bh-4Eh-00h-00h (Common: The unit is selected by the  |
|            | > same initiator while disconnecting.)                  |
+------------+---------------------------------------------------------+

**2-2. INQUIRY Command**

Table 2-2-1-1 INQUIRY command

+-------:+:-----------:+:-----------:+:-----------:+:-----------:+:-----------:+:-----------:+:-----------:+:-----------:+:-----------:+
| Bit    | 7           | 6           | 5                         | 4           | 3           | 2           | 1           | 0           |
|        |             |             |                           |             |             |             |             |             |
| Byte   |             |             |                           |             |             |             |             |             |
+--------+-------------+-------------+---------------------------+-------------+-------------+-------------+-------------+-------------+
| 0      | Operation code \[12h\]                                                                                                      |
+--------+-------------------------------------------------------+-------------------------------------------------------+-------------+
| 1      | Logical unit number                                   | Reserved                                              | EVPD        |
|        |                                                       |                                                       |             |
|        | \[0\]                                                 | \[0\]                                                 | \[0, 1\]    |
+--------+-------------------------------------------------------+-------------------------------------------------------+-------------+
| 2      | Page code \[0\]                                                                                                             |
+--------+-----------------------------------------------------------------------------------------------------------------------------+
| 3      | Reserved \[0\]                                                                                                              |
+--------+-----------------------------------------------------------------------------------------------------------------------------+
| 4      | Allocation length \[Recommended value 36d\]                                                                                 |
+--------+-----------------------------------------+-----------------------------------------------------------------------------------+
| 5      | Reserved \[0\]                          | Control byte \[0\]                                                                |
+--------+-----------------------------------------+-----------------------------------------------------------------------------------+

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

+:---:+:-------------:+:-----------------:+:------------:+:-----------:+:-------------------:+
| VPD | Page code                                        | Sub-section | Attached adapter    |
|     |                                                  |             | (\*1)               |
+-----+-----------------------------------+--------------+-------------+---------------------+
| 0   | Standard INQUIRY data             | 00h (\*2)    | 2-2-1       | MA-21, SA-21,       |
|     |                                   |              |             | SA-30, IA-20,       |
|     |                                   |              |             | SF-210, Non         |
+-----+---------------+-------------------+--------------+-------------+---------------------+
| 1   | VPD           | Page code list    | 00h          | 2-2-2-1     | MA-21, SA-21,       |
|     | informa-tion  |                   |              |             | SA-30, IA-20,       |
|     |               |                   |              |             | SF-210, Non         |
|     |               +-------------------+--------------+-------------+---------------------+
|     |               | FRU ASCII         | 01h          | 2-2-2-2     | MA-21, SA-21,       |
|     |               | information       |              |             | SA-30, IA-20,       |
|     |               |                   |              |             | SF-210              |
|     |               |                   +--------------+             +---------------------+
|     |               |                   | 10h          |             | MA-21               |
|     |               |                   +--------------+             +---------------------+
|     |               |                   | 40h (unused) |             | \-                  |
|     |               |                   +--------------+             +---------------------+
|     |               |                   | 41h (unused) |             | \-                  |
|     |               |                   +--------------+             +---------------------+
|     |               |                   | 43h (unused) |             | \-                  |
|     |               |                   +--------------+             +---------------------+
|     |               |                   | 44h (unused) |             | \-                  |
|     |               |                   +--------------+             +---------------------+
|     |               |                   | 45h (unused) |             | \-                  |
|     |               |                   +--------------+             +---------------------+
|     |               |                   | 46h (unused) |             | \-                  |
|     |               |                   +--------------+             +---------------------+
|     |               |                   | 47h (unused) |             | \-                  |
|     |               |                   +--------------+             +---------------------+
|     |               |                   | 50h (unused) |             | \-                  |
|     |               |                   +--------------+             +---------------------+
|     |               |                   | 51h (unused) |             | \-                  |
|     |               |                   +--------------+             +---------------------+
|     |               |                   | 60h (unused) |             | \-                  |
|     |               |                   +--------------+             +---------------------+
|     |               |                   | 61h (unused) |             | \-                  |
|     |               |                   +--------------+             +---------------------+
|     |               |                   | 62h (unused) |             | \-                  |
|     |               +-------------------+--------------+-------------+---------------------+
|     |               | Address           | C1h          | 2-2-2-3     | MA-21, SA-21,       |
|     |               | information       |              |             | SA-30, IA-20,       |
|     |               |                   |              |             | SF-210, Non         |
|     |               +-------------------+--------------+-------------+---------------------+
|     |               | SET WINDOW        | D1h          | 2-2-2-4     | MA-21, SA-21,       |
|     |               | function          |              |             | SA-30, IA-20,       |
|     |               |                   |              |             | SF-210              |
|     |               +-------------------+--------------+-------------+---------------------+
|     |               | Other information | E1h          | 2-2-2-5     | MA-21, SA-21,       |
|     |               |                   |              |             | SA-30, IA-20,       |
|     |               |                   |              |             | SF-210              |
|     |               +-------------------+--------------+-------------+---------------------+
|     |               | Operation code    | E2h          | 2-2-2-6     | SA-21, SA-30, IA-20 |
|     |               | setting page      |              |             |                     |
|     |               +-------------------+--------------+-------------+---------------------+
|     |               | CCD measurement   | E3h          | 2-2-2-7     | MA-21, SA-21,       |
|     |               | setting page      |              |             | SA-30, IA-20,       |
|     |               |                   |              |             | SF-210              |
|     |               +-------------------+--------------+-------------+---------------------+
|     |               | Unused page       | F0h          | \-          | \-                  |
|     |               |                   +--------------+-------------+---------------------+
|     |               |                   | F1h          | \-          | \-                  |
|     |               |                   +--------------+-------------+---------------------+
|     |               |                   | F8h          | \-          | \-                  |
|     |               |                   +--------------+-------------+---------------------+
|     |               |                   | FAh          | \-          | \-                  |
|     |               |                   +--------------+-------------+---------------------+
|     |               |                   | FBh          | \-          | \-                  |
|     |               |                   +--------------+-------------+---------------------+
|     |               |                   | FCh          | \-          | \-                  |
+-----+---------------+-------------------+--------------+-------------+---------------------+

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

+-------:+:----------:+:----------:+:----------:+:----------:+:----------:+:----------:+:----------:+:----------:+
| Bit    | 7          | 6          | 5          | 4          | 3          | 2          | 1          | 0          |
|        |            |            |            |            |            |            |            |            |
| Byte   |            |            |            |            |            |            |            |            |
+--------+------------+------------+------------+------------+------------+------------+------------+------------+
| 0      | Peripheral Qualifier                 | Peripheral Device Type                                         |
|        |                                      |                                                                |
|        | \[0\]                                | \[6=00110b\]                                                   |
|        |                                      |                                                                |
|        | \[011b\](\*1)                        | \[1Fh=11111b\](\*1)                                            |
+--------+------------+-------------------------+----------------------------------------------------------------+
| 1      | RMB        | Device-Type Modifier                                                                     |
|        |            |                                                                                          |
|        | \[1\]      | \[0\]                                                                                    |
+--------+------------+------------+--------------------------------------+--------------------------------------+
| 2      | ISO Version             | ECMA Version                         | ANSI-Approved Version                |
|        |                         |                                      |                                      |
|        | \[0\]                   | \[0\]                                | \[2=010b\]                           |
+--------+------------+------------+-------------------------+------------+--------------------------------------+
| 3      | AENC       | TrmIOP     | Reserved                | Response Data Format                              |
|        |            |            |                         |                                                   |
|        | \[0\]      | \[0\]      | \[0\]                   | \[2=0010b\]                                       |
+--------+------------+------------+-------------------------+---------------------------------------------------+
| 4      | Additional Length (n-4)                                                                               |
|        |                                                                                                       |
|        | \[1Fh=31d\]                                                                                           |
+--------+-------------------------------------------------------------------------------------------------------+
| 5, 6   | Reserved \[0\]                                                                                        |
+--------+------------+------------+------------+------------+------------+------------+------------+------------+
| 7      | RelAdr     | WBus32     | WBus16     | Sync       | Linked     | Reserved   | CmdQue     | SftRe      |
|        |            |            |            |            |            |            |            |            |
|        | \[0\]      | \[0\]      | \[0\]      | \[0\]      | \[0\]      | \[0\]      | \[0\]      | \[0\]      |
+--------+------------+------------+------------+------------+------------+------------+------------+------------+
| 8 to   | Vendor Identification                                                                                 |
| 15     |                                                                                                       |
|        | \[Nikon\]                                                                                             |
+--------+-------------------------------------------------------------------------------------------------------+
| 16 to  | Product Identification                                                                                |
| 31     |                                                                                                       |
|        | \[ \]                                                                                                 |
+--------+-------------------------------------------------------------------------------------------------------+
| 32 to  | Product Revision Level                                                                                |
| 35     |                                                                                                       |
|        | Example: \[0.01\]                                                                                     |
+--------+-------------------------------------------------------------------------------------------------------+

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

**\
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

+------:+:-----------:+:-----------:+:-----------:+:-----------:+:-----------:+:-----------:+:-----------:+:-----------:+
| Bit   | 7           | 6           | 5           | 4           | 3           | 2           | 1           | 0           |
|       |             |             |             |             |             |             |             |             |
| Byte  |             |             |             |             |             |             |             |             |
+-------+-------------+-------------+-------------+-------------+-------------+-------------+-------------+-------------+
| 0     | Peripheral Qualifier                    | Peripheral Device Type                                              |
|       |                                         |                                                                     |
|       | \[0\]                                   | \[6=00110b\]                                                        |
|       |                                         |                                                                     |
|       | \[011b\](\*1)                           | \[1Fh=11111b\](\*1)                                                 |
+-------+-----------------------------------------+---------------------------------------------------------------------+
| 1     | Page code \[00h\]                                                                                             |
+-------+---------------------------------------------------------------------------------------------------------------+
| 2     | Reserved \[0\]                                                                                                |
+-------+---------------------------------------------------------------------------------------------------------------+
| 3     | Page length \[m-3\]                                                                                           |
+-------+---------------------------------------------------------------------------------------------------------------+
| 4 to  | Page code list \[m-4\]                                                                                        |
| m     |                                                                                                               |
+-------+---------------------------------------------------------------------------------------------------------------+

> \*1 When an invalid logical unit selection is performed

This shows the list of information page codes that are supported by this
unit.

Byte 4 Page code list

> In this field, the information page codes that are supported by this
> unit are shown in units of one byte length in order starting from page
> code 00h.

+:---------------------------:+:--------------------------------------:+
| Attached adapter            | Supported page (hex)                   |
+-----------------------------+----------------------------------------+
| Common to all adapters      | 00, 01, 40, 41, 50, 51, 60, 61, 62,    |
|                             | C1, D1, E1, E3, F0, F8, FB, FC         |
+-----------------------------+----------------------------------------+
| Mount adapter               | Common to all adapters + 10            |
|                             |                                        |
| (when a holder is attached) |                                        |
+-----------------------------+----------------------------------------+
| 6-frame strip adapter       | Common to all adapters + 46, E2        |
+-----------------------------+----------------------------------------+
| 36-frame strip adapter      | Common to all adapters + 47, E2        |
+-----------------------------+----------------------------------------+
| 240 adapter                 | Common to all adapters + 43, E2        |
+-----------------------------+----------------------------------------+
| Slide feeder                | Common to all adapters + 45, F1        |
+-----------------------------+----------------------------------------+
| None/Undefined              | 00, C1, FB, FC                         |
+-----------------------------+----------------------------------------+

Note) On the above supported pages, 40, 41, 43, 45, 46, 47, 50, 51, 60,
61, 62, F0, F1, F8, FA, FB, and FC are not used.

**2-2-2-2. FRU ASCII information page**

+------:+:-----------:+:-----------:+:-----------:+:-----------:+:-----------:+:-----------:+:-----------:+:-----------:+
| Bit   | 7           | 6           | 5           | 4           | 3           | 2           | 1           | 0           |
|       |             |             |             |             |             |             |             |             |
| Byte  |             |             |             |             |             |             |             |             |
+-------+-------------+-------------+-------------+-------------+-------------+-------------+-------------+-------------+
| 0     | Peripheral Qualifier                    | Peripheral Device Type                                              |
|       |                                         |                                                                     |
|       | \[0\]                                   | \[6=00110b\]                                                        |
|       |                                         |                                                                     |
|       | \[011b\](\*1)                           | \[1Fh=11111b\](\*1)                                                 |
+-------+-----------------------------------------+---------------------------------------------------------------------+
| 1     | Page code \[01 to 7Fh\]                                                                                       |
+-------+---------------------------------------------------------------------------------------------------------------+
| 2     | Reserved \[0\]                                                                                                |
+-------+---------------------------------------------------------------------------------------------------------------+
| 3     | Page length \[m-3\]                                                                                           |
+-------+---------------------------------------------------------------------------------------------------------------+
| 4     | ASCII data length \[m-4\]                                                                                     |
+-------+---------------------------------------------------------------------------------------------------------------+
| 5 to  | ASCII information                                                                                             |
| m     |                                                                                                               |
+-------+---------------------------------------------------------------------------------------------------------------+

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

+:-----------:+:------------------:+:-----------:+:--------------------:+
| Page code   | Attached adapter   | ASCII       | Descriptions         |
| (ID)        |                    | information |                      |
+-------------+--------------------+-------------+----------------------+
| 01h         | Mount adapter      | Mount       | Mount adapter        |
|             +--------------------+-------------+----------------------+
|             | 6-frame strip      | 6Strip      | 6-frame strip        |
|             | adapter            |             | adapter              |
|             +--------------------+-------------+----------------------+
|             | 36-frame strip     | 36Strip     | 36-frame strip       |
|             | adapter            |             | adapter              |
|             +--------------------+-------------+----------------------+
|             | 240 adapter        | 240         | 240 adapter          |
|             +--------------------+-------------+----------------------+
|             | Slide feeder       | Feeder      | Slide feeder         |
+-------------+--------------------+-------------+----------------------+
| 10h         | Mount adapter      | FH3         | 6-frame strip holder |
|             |                    +-------------+----------------------+
|             |                    | FHG1        | Praparat holder      |
|             |                    +-------------+----------------------+
|             |                    | FHA1        | APS holder           |
+-------------+--------------------+-------------+----------------------+

**\
2-2-2-3. Address information page**

Address information page

+-----:+:------------:+:------------:+:------------:+:------------:+:------------:+:------------:+:------------:+:------------:+
| Bit  | 7            | 6            | 5            | 4            | 3            | 2            | 1            | 0            |
|      |              |              |              |              |              |              |              |              |
| Byte |              |              |              |              |              |              |              |              |
+------+--------------+--------------+--------------+--------------+--------------+--------------+--------------+--------------+
| 0    | Peripheral Qualifier                       | Peripheral Device Type                                                   |
|      |                                            |                                                                          |
|      | \[0\]                                      | \[6=00110b\]                                                             |
|      |                                            |                                                                          |
|      | \[011b\](\*1)                              | \[1Fh=11111b\](\*1)                                                      |
+------+--------------------------------------------+--------------------------------------------------------------------------+
| 1    | Page code \[C1h\]                                                                                                     |
+------+-----------------------------------------------------------------------------------------------------------------------+
| 2    | Reserved \[0\]                                                                                                        |
+------+-----------------------------------------------------------------------------------------------------------------------+
| 3    | Page length \[83d=53h\]                                                                                               |
+------+-----------------------------------------------------------------------------------------------------------------------+
| 4    | > SCSI function support (SCSI data transfer function)                                                                 |
|      | >                                                                                                                     |
|      | > \[03h\] (Adapters other than the IA-20 adapter)                                                                     |
|      | >                                                                                                                     |
|      | > \[0Bh\] (IA-20 adapter)                                                                                             |
+------+-----------------------------------------------------------------------------------------------------------------------+
| 5, 6 | > Window descriptor block length \[61=003Dh\]                                                                         |
+------+-----------------------------------------------------------------------------------------------------------------------+
| 7, 8 | > Set parameter descriptor block length                                                                               |
|      | >                                                                                                                     |
|      | > (Length of the SET PARAMETER command parameter in bytes)                                                            |
|      | >                                                                                                                     |
|      | > \[15=000Fh\]                                                                                                        |
+------+-----------------------------------------------------------------------------------------------------------------------+
| 9,   | > General SCSI Buffer Size (SCSI data transfer buffer size. Unit: byte) \[0\]                                         |
| 10   |                                                                                                                       |
+------+-----------------------------------------------------------------------------------------------------------------------+
| 11,  | > Image Buffer Size (Unit: KB) \[256=0100h\]                                                                          |
| 12   |                                                                                                                       |
+------+-----------------------------------------------------------------------------------------------------------------------+
| 13   | > Number of equipped Unit (the number of units that can be attached simultaneously) \[1\]                             |
+------+-----------------------------------------------------------------------------------------------------------------------+
| 14   | > Unit Name ID (ID numbers of the attached adapter and the attached holder)                                           |
|      | >                                                                                                                     |
|      | > \[01h\] (When an adapter is attached)                                                                               |
|      | >                                                                                                                     |
|      | > \[0\] (When an adapter is not attached or an undefined adapter is attached)                                         |
+------+-----------------------------------------------------------------------------------------------------------------------+
| 15   | > Current Holder Name ID (the current holder name)                                                                    |
|      | >                                                                                                                     |
|      | > \[10h\] (When a holder is attached)                                                                                 |
|      | >                                                                                                                     |
|      | > \[0\] (When a holder is not attached or an undefined holder is attached)                                            |
+------+-----------------------------------------------------------------------------------------------------------------------+
| 16   | > Coordinate base information (resolution type and scanning that are supported)                                       |
|      | >                                                                                                                     |
|      | > \[0Fh\] (The FH3 holder is inserted reversely.)                                                                     |
|      | >                                                                                                                     |
|      | > \[13h\] (IA-20)                                                                                                     |
|      | >                                                                                                                     |
|      | > \[03h\] (Adapter and holder other than the above)                                                                   |
+------+-----------------------------------------------------------------------------------------------------------------------+
| 17   | > Addressing Kind (addressing type that is supported)                                                                 |
|      | >                                                                                                                     |
|      | > \[31h\] (SA-21/SA-30)                                                                                               |
|      | >                                                                                                                     |
|      | > \[35h\] (IA-20)                                                                                                     |
|      |                                                                                                                       |
|      | \[32h\] (SF-210)                                                                                                      |
|      |                                                                                                                       |
|      | \[22h\] (Adapter and holder other than the above)                                                                     |
+------+-----------------------------------------------------------------------------------------------------------------------+
| 18,  | > X-Optical Resolution (Unit: dpi)                                                                                    |
| 19   | >                                                                                                                     |
|      | > \[4000=0FA0h\]                                                                                                      |
+------+-----------------------------------------------------------------------------------------------------------------------+
| 20,  | > X-Maximum Resolution (Unit: dpi)                                                                                    |
| 21   | >                                                                                                                     |
|      | > \[4000=0FA0h\]                                                                                                      |
+------+-----------------------------------------------------------------------------------------------------------------------+
| 22,  | > X-Minimum Resolution (Unit: dpi)                                                                                    |
| 23   | >                                                                                                                     |
|      | > \[90=005Ah\]                                                                                                        |
+------+-----------------------------------------------------------------------------------------------------------------------+
| 24   | > X-Maximum Set Window Address                                                                                        |
| to   | >                                                                                                                     |
| 27   | > (Window descriptor X-axis offset address maximum value)                                                             |
|      | >                                                                                                                     |
|      | > \[0\]                                                                                                               |
+------+-----------------------------------------------------------------------------------------------------------------------+
| 28   | > X-Minimum Set Window Address                                                                                        |
| to   | >                                                                                                                     |
| 31   | > (Window descriptor X-axis offset address minimum value)                                                             |
|      | >                                                                                                                     |
|      | > \[0\]                                                                                                               |
+------+-----------------------------------------------------------------------------------------------------------------------+
| 32   | > X-Offset for first image's address (X-axis scanning start position offset address)                                  |
| to   | >                                                                                                                     |
| 35   | > \[0\]                                                                                                               |
+------+-----------------------------------------------------------------------------------------------------------------------+
| 36   | > X-Set Window boundary                                                                                               |
| to   | >                                                                                                                     |
| 39   | > (Maximum window width value of the X-axis window descriptor)                                                        |
|      | >                                                                                                                     |
|      | > \[2916=00000B64h\] (IA-20)                                                                                          |
|      | >                                                                                                                     |
|      | > \[3946=00000F6Ah\] (Adapter and holder other than the above)                                                        |
+------+-----------------------------------------------------------------------------------------------------------------------+
| 40,  | > Y-Optical Resolution (Unit: dpi)                                                                                    |
| 41   | >                                                                                                                     |
|      | > \[4000=0FA0h\]                                                                                                      |
+------+-----------------------------------------------------------------------------------------------------------------------+
| 42,  | > Y-Maximum Resolution (Unit: dpi)                                                                                    |
| 43   | >                                                                                                                     |
|      | > \[4000=0FA0h\]                                                                                                      |
+------+-----------------------------------------------------------------------------------------------------------------------+
| 44,  | > Y-Minimum Resolution (Unit: dpi)                                                                                    |
| 45   | >                                                                                                                     |
|      | > \[90=005Ah\]                                                                                                        |
+------+-----------------------------------------------------------------------------------------------------------------------+
| 46   | > Y-Maximum Set Window Address                                                                                        |
| to   | >                                                                                                                     |
| 49   | > (Window descriptor Y-axis offset address maximum value)                                                             |
|      | >                                                                                                                     |
|      | > \[\*2\] (SA-21/SA-30)                                                                                               |
|      | >                                                                                                                     |
|      | > \[\*3\] (IA-20)                                                                                                     |
|      | >                                                                                                                     |
|      | > \[5781=00001695h\] (Adapter and holder other than the above)                                                        |
+------+-----------------------------------------------------------------------------------------------------------------------+
| 50   | > Y-Minimum Set Window Address                                                                                        |
| to   | >                                                                                                                     |
| 53   | > (Window descriptor Y-axis offset address minimum value) \[0\]                                                       |
+------+-----------------------------------------------------------------------------------------------------------------------+
| 54   | > Y-Offset for first image's address (Y-axis scanning start position offset address)                                  |
| to   | >                                                                                                                     |
| 57   | > \[0\]                                                                                                               |
+------+-----------------------------------------------------------------------------------------------------------------------+
| 58   | > Y-Set Window boundary                                                                                               |
| to   | >                                                                                                                     |
| 61   | > (Maximum window width value of the Y-axis window descriptor)                                                        |
|      | >                                                                                                                     |
|      | > \[5959=00001747h\] (SA-21/SA-30)                                                                                    |
|      | >                                                                                                                     |
|      | > \[4453=00001165h\] (IA-20)                                                                                          |
|      | >                                                                                                                     |
|      | > \[5782=00001696h\] (Adapter and holder other than the above)                                                        |
+------+-----------------------------------------------------------------------------------------------------------------------+
| 62   | > Y-Another world maximum Address                                                                                     |
| to   | >                                                                                                                     |
| 65   | > (Maximum address in the sub-scanning direction outside the specified address)                                       |
|      | >                                                                                                                     |
|      | > \[5959=00001747h\] (SA-21/SA-30 adapter)                                                                            |
|      | >                                                                                                                     |
|      | > \[5782=00001696h\] (IA-20)                                                                                          |
+------+-----------------------------------------------------------------------------------------------------------------------+
| 66   | > Y-Another world minimum Address                                                                                     |
| to   | >                                                                                                                     |
| 69   | > (Minimum address in the sub-scanning direction outside the specified address)                                       |
|      | >                                                                                                                     |
|      | > \[0\] (SA-21/SA-30/IA-20)                                                                                           |
+------+-----------------------------------------------------------------------------------------------------------------------+
| 70,  | > Maximum Thumbnail Resolution                                                                                        |
| 71   | >                                                                                                                     |
|      | > (Maximum resolution in thumbnail scanning. Unit: dpi)                                                               |
|      | >                                                                                                                     |
|      | > \[97=0061h\] (SA-21/SA-30)                                                                                          |
|      | >                                                                                                                     |
|      | > \[90=005Ah\] (IA-20)                                                                                                |
+------+-----------------------------------------------------------------------------------------------------------------------+
| 72,  | > Minimum Thumbnail Resolution                                                                                        |
| 73   | >                                                                                                                     |
|      | > (Minimum resolution in thumbnail scanning. Unit: dpi)                                                               |
|      | >                                                                                                                     |
|      | > \[97=0061h\] (SA-21/SA-30)                                                                                          |
|      | >                                                                                                                     |
|      | > \[90=005Ah\] (IA-20)                                                                                                |
+------+-----------------------------------------------------------------------------------------------------------------------+
| 74   | > Maximum Image count (maximum number of frames that can be scanned)                                                  |
|      | >                                                                                                                     |
|      | > \[6\] (SA-21)                                                                                                       |
|      | >                                                                                                                     |
|      | > \[40=28h\] (SA-30/IA-20)                                                                                            |
|      | >                                                                                                                     |
|      | > \[1\] (Adapter and holder other than the above)                                                                     |
+------+-----------------------------------------------------------------------------------------------------------------------+
| 75   | Actual including image count (the number of medium frames that are currently set)                                     |
|      |                                                                                                                       |
|      | > \[\*4\] (SA-21/SA-30)                                                                                               |
|      | >                                                                                                                     |
|      | > \[1 to 40d\] (IA-20)                                                                                                |
|      | >                                                                                                                     |
|      | > \[6\] (When the number of frames is not known in SA-21. Ex.: When the initialization of SA-21 is performed before   |
|      | > the number of frames is detected)                                                                                   |
|      | >                                                                                                                     |
|      | > \[0\] (When a medium is not inserted in SA-21/SA-30/IA-20)                                                          |
|      | >                                                                                                                     |
|      | > \[40=28h\] (When the number of frames is not known in SA-30/IA-20)                                                  |
|      | >                                                                                                                     |
|      | > \[1\] (Adapter and holder other than the above)                                                                     |
+------+-----------------------------------------------------------------------------------------------------------------------+
| 76,  | Minimum Focusing Address (minimum address of the focus position) \[0\]                                                |
| 77   |                                                                                                                       |
+------+-----------------------------------------------------------------------------------------------------------------------+
| 78,  | > Maximum Focusing Address (maximum address of the focus position) \[323=0143h\]                                      |
| 79   |                                                                                                                       |
+------+-----------------------------------------------------------------------------------------------------------------------+
| 80,  | > Lamp warm-up maximum time (maximum time for lamp warming-up) \[0\]                                                  |
| 81   |                                                                                                                       |
+------+-----------------------------------------------------------------------------------------------------------------------+
| 82   | > A/D bit depth (depth of bits for an A/D converter) \[16=10h\]                                                       |
+------+-----------------------------------------------------------------------------------------------------------------------+
| 83,  | > CCD Pixel Number                                                                                                    |
| 84   | >                                                                                                                     |
|      | > (The number of effective pixels in the CCD. For the CCD in which the number of effective pixels differs in each     |
|      | > color, the maximum value is set.)                                                                                   |
|      | >                                                                                                                     |
|      | > \[3946=0F6Ah\]                                                                                                      |
+------+-----------------------------------------------------------------------------------------------------------------------+
| 85   | > Line Gap Count (the number of gaps between lines) \[01h\]                                                           |
+------+-----------------------------------------------------------------------------------------------------------------------+
| 86   | > CCD Line Number (the number of lines in the CCD) \[02h\]                                                            |
+------+-----------------------------------------------------------------------------------------------------------------------+

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

+:----:+:--------------------------------------:+:----------:+:----------:+
| Bit  |                                        | Support of this unit    |
|      |                                        +------------+------------+
|      |                                        | Adapter    | IA-20      |
|      |                                        | other than |            |
|      |                                        | IA-20      |            |
+------+----------------------------------------+------------+------------+
| 0    | > Microcode downloading function       | 1          | 1          |
+------+----------------------------------------+------------+------------+
| 1    | > Image reading (READ command) must be | 1          | 1          |
|      | > performed in units of \[Data of one  |            |            |
|      | > line in bytes \* number of colors\]. |            |            |
+------+----------------------------------------+------------+------------+
| 2    | > Image reading (READ command) must be | 0          | 0          |
|      | > performed in units of \[Data of one  |            |            |
|      | > line in bytes\].                     |            |            |
+------+----------------------------------------+------------+------------+
| 3    | > Thumbnail reading (READ command)     | 0          | 1          |
|      | > must be performed in units of \[The  |            |            |
|      | > number of bytes in one frame\*       |            |            |
|      | > number of colors\].                  |            |            |
+------+----------------------------------------+------------+------------+
| 4 to | > Reserved                             | 0          | 0          |
| 6    |                                        |            |            |
+------+----------------------------------------+------------+------------+
| 7    | > Extend bit                           | 0          | 0          |
+------+----------------------------------------+------------+------------+

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

+:-----:+-------------------------+:--------------------------------------+
| Bit0, | Resolution type \[3\]   | Setting this bit to 0 indicates that  |
| 1     |                         | reading can be performed in           |
|       |                         | continuous resolution.                |
|       |                         |                                       |
|       |                         | Setting this bit to 1 indicates that  |
|       |                         | reading can be performed only in the  |
|       |                         | resolution of each pitch.             |
|       |                         |                                       |
|       |                         | Setting this bit to 2 indicates that  |
|       |                         | reading can be performed only in the  |
|       |                         | resolution of the pitch which is the  |
|       |                         | measure of the maximum pitch. (\*1)   |
|       |                         |                                       |
|       |                         | Setting this bit to 3 indicates that  |
|       |                         | reading can be performed only in the  |
|       |                         | resolution of pitch 1 and an even     |
|       |                         | pitch. (\*2)                          |
+-------+-------------------------+---------------------------------------+
| Bit2  | X Origin Reversed       | Setting this bit to 1 indicates that  |
|       |                         | the main-scanning direction origin is |
|       | > \[FH3 reverse         | reversed (at the right end of the     |
|       | > direction=1/          | medium).                              |
|       | >                       |                                       |
|       | > Other=0\]             |                                       |
+-------+-------------------------+---------------------------------------+
| Bit3  | Y Origin Reversed       | Setting this bit to 1 indicates that  |
|       |                         | the sub-scanning direction origin is  |
|       | > \[FH3 reverse         | reversed (at the bottom end of the    |
|       | > direction=1/          | medium).                              |
|       | >                       |                                       |
|       | > Other=0\]             |                                       |
+-------+-------------------------+---------------------------------------+
| Bit4  | Thumbnail Order         | Setting this bit to 0 indicates that  |
|       | Reversed                | the thumbnail image is stored in the  |
|       |                         | normal direction (first frame-\>last  |
|       | > \[IA-20=1/SA-21,      | frame). Setting this bit to 1         |
|       | > SA-30=0\]             | indicates that the thumbnail image is |
|       |                         | stored in the reversed direction      |
|       |                         | (last frame-\>first frame).           |
+-------+-------------------------+---------------------------------------+
| Bit5  | Reserved \[0\]          | This bit is set to 0 in this unit.    |
+-------+-------------------------+---------------------------------------+
| Bit6  | Additional Coordinate   | This bit is set to 0 in this unit.    |
|       | Information \[0\]       |                                       |
+-------+-------------------------+---------------------------------------+
| Bit7  | Extend bit \[0\]        | This bit is set to 0 in this unit.    |
+-------+-------------------------+---------------------------------------+

> \*1: When the maximum pitch is 12, the pitches in which reading can be
> performed are 1, 2, 3, 4, 6, and 12.
>
> \*2: However, by way of exception, only for reading the thumbnail in
> SA-21/SA-30, reading is performed in the odd pitch (pitch 41) relative
> to the film movement length.

Byte 17 Addressing Kind

> This field specifies the addressing type that is supported. The
> addressing of the bit to which 1 is set is supported.

+:---:+:---------------------------------------:+:------:+:---------------:+:------:+:------:+:------:+
| Bit | Descriptions                            | Adapter                                             |
+-----+-----------------------------------------+--------+-----------------+--------+--------+--------+
|     |                                         | MA-21  | SA-21           | SA-30  | IA-20  | SF-210 |
+-----+-----------------------------------------+--------+-----------------+--------+--------+--------+
| 0   | > The Set Window address is the same as | 0      | 1               | 1      | 1      | 0      |
|     | > the medium position address.          |        |                 |        |        |        |
+-----+-----------------------------------------+--------+-----------------+--------+--------+--------+
| 1   | > The Set Window address is the same as | 1      | 0               | 0      | 0      | 1      |
|     | > the address of the mechanical block.  |        |                 |        |        |        |
+-----+-----------------------------------------+--------+-----------------+--------+--------+--------+
| 2   | > Specifying the scanning range over    | 0      | 0               | 0      | 1      | 0      |
|     | > two or more frames is prohibited.     |        |                 |        |        |        |
+-----+-----------------------------------------+--------+-----------------+--------+--------+--------+
| 3   | > Reserved                              | 0      | 0               | 0      | 0      | 0      |
+-----+-----------------------------------------+--------+-----------------+--------+--------+--------+
| 4   | > The position of the medium can be     | 0      | 1               | 1      | 1      | 1      |
|     | > operated.                             |        |                 |        |        |        |
+-----+-----------------------------------------+--------+-----------------+--------+--------+--------+
| 5   | > The mechanical block position can be  | 1      | 1               | 1      | 1      | 1      |
|     | > operated.                             |        |                 |        |        |        |
+-----+-----------------------------------------+--------+-----------------+--------+--------+--------+
| 6   | > Reserved                              | 0      | 0               | 0      | 0      | 0      |
+-----+-----------------------------------------+--------+-----------------+--------+--------+--------+
| 7   | > Extension bit                         | 0      | 0               | 0      | 0      | 0      |
+-----+-----------------------------------------+--------+-----------------+--------+--------+--------+

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
> no value is sent to this field, '3 lines' is set.

Note) If this page is requested when the adapter is not attached or an
undefined adapter is attached, the data up to byte 14 (allocation length
15 bytes) is returned.

Address information page set value

+:----:+----------------------+:---------:+:---------:+:---------:+:---------:+:---------:+:---------:+
| Byte |                      | Set value                                                             |
|      |                      +-----------+-----------+-----------+-----------+-----------------------+
|      |                      | MA-21     | SA-21     | SA-30     | SF-210    | IA-20                 |
|      |                      | (\*3)     |           |           |           |                       |
+------+----------------------+-----------+-----------+-----------+-----------+-----------------------+
| 18,  | X-Optical Resolution | 4000                                                                  |
| 19   |                      |                                                                       |
+------+----------------------+-----------------------------------------------------------------------+
| 20,  | X-Maximum Resolution | 4000                                                                  |
| 21   |                      |                                                                       |
+------+----------------------+-----------------------------------------------------------------------+
| 22,  | X-Minimum Resolution | 90                                                                    |
| 23   |                      |                                                                       |
+------+----------------------+-----------------------------------------------------------------------+
| 24   | X-Maximum Set Window | 0                                                                     |
| to   | Address              |                                                                       |
| 27   |                      |                                                                       |
+------+----------------------+-----------------------------------------------------------------------+
| 28   | X-Minimum Set Window | 0                                                                     |
| to   | Address              |                                                                       |
| 31   |                      |                                                                       |
+------+----------------------+-----------------------------------------------------------+-----------+
| 36   | X-Set Window         | 3946                                                      | 2916      |
| to   | boundary             |                                                           |           |
| 39   |                      |                                                           |           |
+------+----------------------+-----------------------------------------------------------+-----------+
| 40,  | Y-Optical Resolution | 4000                                                                  |
| 41   |                      |                                                                       |
+------+----------------------+-----------------------------------------------------------------------+
| 42,  | Y-Maximum Resolution | 4000                                                                  |
| 43   |                      |                                                                       |
+------+----------------------+-----------------------------------------------------------------------+
| 44,  | Y-Minimum Resolution | 90                                                                    |
| 45   |                      |                                                                       |
+------+----------------------+-----------+-----------+-----------+-----------+-----------------------+
| 46   | Y-Maximum Set Window | 5781      | (\*1)     | (\*1)     | 5781      | (\*2)                 |
| to   | Address              |           |           |           |           |                       |
| 49   |                      |           |           |           |           |                       |
+------+----------------------+-----------+-----------+-----------+-----------+-----------------------+
| 50   | Y-Minimum Set Window | 0         | 0         | 0         | 0         | 0                     |
| to   | Address              |           |           |           |           |                       |
| 53   |                      |           |           |           |           |                       |
+------+----------------------+-----------+-----------+-----------+-----------+-----------------------+
| 58   | Y-Set Window         | 5782      | 5959      | 5959      | 5782      | 4453                  |
| to   | boundary             |           |           |           |           |                       |
| 61   |                      |           |           |           |           |                       |
+------+----------------------+-----------+-----------+-----------+-----------+-----------------------+
| 62   | Y-Another world      | \-        | 5959      | 5959      | \-        | 5782                  |
| to   | maximum Address      |           |           |           |           |                       |
| 65   |                      |           |           |           |           |                       |
+------+----------------------+-----------+-----------+-----------+-----------+-----------------------+
| 66   | Y-Another world      | \-        | 0         | 0         | \-        | 0                     |
| to   | minimum Address      |           |           |           |           |                       |
| 69   |                      |           |           |           |           |                       |
+------+----------------------+-----------+-----------+-----------+-----------+-----------------------+
| 70,  | Maximum Thumbnail    | \-        | 97        | 97        | \-        | 90                    |
| 71   | Resolution           |           |           |           |           |                       |
+------+----------------------+-----------+-----------+-----------+-----------+-----------------------+
| 72,  | Minimum Thumbnail    | \-        | 97        | 97        | \-        | 90                    |
| 73   | Resolution           |           |           |           |           |                       |
+------+----------------------+-----------+-----------+-----------+-----------+-----------------------+
| 76,  | Minimum Focusing     | 0                                                                     |
| 77   | Address              |                                                                       |
+------+----------------------+-----------------------------------------------------------------------+
| 78,  | Maximum Focusing     | 323                                                                   |
| 79   | Address              |                                                                       |
+------+----------------------+-----------------------------------------------------------------------+

\*1 Y-Maximum Set Window Address=(Actual Including Image Count+2)\*5959

\*2 Y-Maximum Set Window Address=(Actual Including Image Count)\*4453-1

\*3 Each holder of FH3, FHG1, and FHA1 is included.

**2-2-2-4. SET WINDOW function page**

SET WINDOW function page

+------:+:-----------:+:-----------:+:-----------:+:-----------:+:-----------:+:-----------:+:-----------:+:-----------:+
| Bit   | 7           | 6           | 5           | 4           | 3           | 2           | 1           | 0           |
|       |             |             |             |             |             |             |             |             |
| Byte  |             |             |             |             |             |             |             |             |
+-------+-------------+-------------+-------------+-------------+-------------+-------------+-------------+-------------+
| 0     | Peripheral Qualifier                    | Peripheral Device Type                                              |
|       |                                         |                                                                     |
|       | \[0\]                                   | \[6=00110b\]                                                        |
|       |                                         |                                                                     |
|       | \[011b\](\*1)                           | \[1Fh=11111b\](\*1)                                                 |
+-------+-----------------------------------------+---------------------------------------------------------------------+
| 1     | Page code \[D1h\]                                                                                             |
+-------+---------------------------------------------------------------------------------------------------------------+
| 2     | Reserved \[0\]                                                                                                |
+-------+---------------------------------------------------------------------------------------------------------------+
| 3     | Page length \[24d=18h\]                                                                                       |
+-------+---------------------------------------------------------------------------------------------------------------+
| 4     | > Scanning Kind Support                                                                                       |
|       |                                                                                                               |
|       | \[03h\] (SA-21/SA-30/IA-20)                                                                                   |
|       |                                                                                                               |
|       | \[01h\] (Adapter and holder other than the above)                                                             |
+-------+---------------------------------------------------------------------------------------------------------------+
| 5     | > Scan Mode Support                                                                                           |
|       | >                                                                                                             |
|       | > \[52h\]                                                                                                     |
+-------+---------------------------------------------------------------------------------------------------------------+
| 6     | > Color Interleaving Support (color order for data transfer)                                                  |
|       | >                                                                                                             |
|       | > \[42h\]                                                                                                     |
+-------+---------------------------------------------------------------------------------------------------------------+
| 7     | > Color Component                                                                                             |
|       | >                                                                                                             |
|       | > \[06h\]                                                                                                     |
+-------+---------------------------------------------------------------------------------------------------------------+
| 8     | > Color Ordering1                                                                                             |
|       | >                                                                                                             |
|       | > \[20h\]                                                                                                     |
+-------+---------------------------------------------------------------------------------------------------------------+
| 9     | > Color Ordering2                                                                                             |
|       | >                                                                                                             |
|       | > \[43h\]                                                                                                     |
+-------+---------------------------------------------------------------------------------------------------------------+
| 10    | > Output Bit Depth/Dot a Color Support (the number of bits in one-color data)                                 |
|       | >                                                                                                             |
|       | > \[20h\]                                                                                                     |
+-------+---------------------------------------------------------------------------------------------------------------+
| 11    | > Number of Setup Mode                                                                                        |
|       | >                                                                                                             |
|       | > \[0\]                                                                                                       |
+-------+---------------------------------------------------------------------------------------------------------------+
| 12    | > Digital Image Control Support                                                                               |
|       | >                                                                                                             |
|       | > \[0\]                                                                                                       |
+-------+---------------------------------------------------------------------------------------------------------------+
| 13    | > Additional length for Digital Control Information                                                           |
|       | >                                                                                                             |
|       | > \[0\]                                                                                                       |
+-------+---------------------------------------------------------------------------------------------------------------+
| 14    | > Analog Control Support                                                                                      |
|       | >                                                                                                             |
|       | > \[40h\]                                                                                                     |
+-------+---------------------------------------------------------------------------------------------------------------+
| 15    | > Additional length for Analog Control Information                                                            |
|       | >                                                                                                             |
|       | > \[9\]                                                                                                       |
+-------+---------------------------------------------------------------------------------------------------------------+
| 16 to | > for the First Supported Control (exposure value control parameter)                                          |
| 24    | >                                                                                                             |
|       | > Byte 16 Bytes a Value for the control (parameter length in bytes)                                           |
|       | >                                                                                                             |
|       | > \[4\]                                                                                                       |
|       | >                                                                                                             |
|       | > Byte 17 to 20 Minimum Value for the First Control                                                           |
|       | >                                                                                                             |
|       | > \[00000001h\]                                                                                               |
|       | >                                                                                                             |
|       | > Byte 21 to 24 Maximum Value for the First Control                                                           |
|       | >                                                                                                             |
|       | > \[03FFFFFFh\]                                                                                               |
+-------+---------------------------------------------------------------------------------------------------------------+
| 25    | > Filter Support                                                                                              |
|       | >                                                                                                             |
|       | > \[0\]                                                                                                       |
+-------+---------------------------------------------------------------------------------------------------------------+
| 26    | > Matrix Support                                                                                              |
|       | >                                                                                                             |
|       | > \[0\]                                                                                                       |
+-------+---------------------------------------------------------------------------------------------------------------+
| 27    | > Halftone Support                                                                                            |
|       | >                                                                                                             |
|       | > \[0\]                                                                                                       |
+-------+---------------------------------------------------------------------------------------------------------------+

\*1 When an invalid logical unit selection is performed

Byte 4 Scanning Kind Support

> This field specifies the image scanning types that are supported.
>
> For this unit, all adapters support Image Scanning, and the 6-frame
> strip adapter (6SA), 36-frame strip adapter (36SA), and 240 adapter
> support Thumbnail Scanning in addition.

+:---:+:--------------:+:-------------------------------:+:------------------+:---------+
| Bit | Type           | Explanations of operation       | IA-20/SA-21/SA-30 | Other    |
|     |                |                                 |                   | adapters |
+-----+----------------+---------------------------------+-------------------+----------+
| 0   | Image Scanning | Normal image scanning           | 1                 | 1        |
+-----+----------------+---------------------------------+-------------------+----------+
| 1   | Thumbnail      | Thumbnail image scanning        | 1                 | 0        |
|     | Scanning       |                                 |                   |          |
+-----+----------------+---------------------------------+-------------------+----------+
| 2   | Set up         | Prescan                         | 0                 | 0        |
|     | Scanning       |                                 |                   |          |
|     |                | Scanning for deciding the       |                   |          |
|     |                | optimal integral time and gain, |                   |          |
|     |                | etc.                            |                   |          |
+-----+----------------+---------------------------------+-------------------+----------+
| 3   | Set up         | Prescan                         | 0                 | 0        |
|     | Scanning2      |                                 |                   |          |
|     |                | Scanning for deciding the       |                   |          |
|     |                | optimal integral time and gain, |                   |          |
|     |                | etc. The                        |                   |          |
|     |                | low-density/high-density limit  |                   |          |
|     |                | values are used instead of the  |                   |          |
|     |                | maximum value and the minimum   |                   |          |
|     |                | value. When the bit is 1, Setup |                   |          |
|     |                | Mode in the window descriptor   |                   |          |
|     |                | of SET WINDOW is supported. For |                   |          |
|     |                | the number of supports, refer   |                   |          |
|     |                | to 'Number of Setup mode'       |                   |          |
|     |                | field.                          |                   |          |
+-----+----------------+---------------------------------+-------------------+----------+
| 4   | Histogram      | Scanning for creating the image | 0                 | 0        |
|     | Scanning       | data histogram                  |                   |          |
+-----+----------------+---------------------------------+-------------------+----------+
| 5   | Auto Exposure  | Scanning for deciding the       | 0                 | 0        |
|     | Scanning       | integral time at which the      |                   |          |
|     |                | output value becomes the AE     |                   |          |
|     |                | Value that is set in each color |                   |          |
+-----+----------------+---------------------------------+-------------------+----------+
| 6   | AE with WB     | Scanning for deciding the       | 0                 | 0        |
|     | Scanning       | integral time at which the      |                   |          |
|     |                | maximum value of the output     |                   |          |
|     |                | values in each color becomes    |                   |          |
|     |                | the AE Value that is set with   |                   |          |
|     |                | the white balance maintained    |                   |          |
+-----+----------------+---------------------------------+-------------------+----------+
| 7   | Extend bit     | Extension bit \[0\]             | 0                 | 0        |
+-----+----------------+---------------------------------+-------------------+----------+

\[03h\] \[01h\]

Byte 5 Scan Mode Support

> This field specifies the scanning mode.
>
> Normal Quality Scan, Multiple Reading Scan, and Reverse direction
> Scanning Supported are supported.

+:-----:+-----------------------------------------------------+:-----:+
| Bit0  | > High Quality Scan                                 | \[0\] |
+-------+-----------------------------------------------------+-------+
| Bit1  | > Normal Quality Scan                               | \[1\] |
+-------+-----------------------------------------------------+-------+
| Bit2  | > High Speed Scan                                   | \[0\] |
+-------+-----------------------------------------------------+-------+
| Bit3  | > Reserved                                          | \[0\] |
+-------+-----------------------------------------------------+-------+
| Bit4  | > Multiple Reading Scan                             | \[1\] |
+-------+-----------------------------------------------------+-------+
| Bit5  | > Reserved                                          | \[0\] |
+-------+-----------------------------------------------------+-------+
| Bit6  | > Reverse direction Scanning Supported              | \[1\] |
+-------+-----------------------------------------------------+-------+
| Bit7  | > Extend bit                                        | \[0\] |
+-------+-----------------------------------------------------+-------+

Byte 6 Color Interleaving Support

> This field specifies the color order for data transfer.
>
> This unit supports 'Line without CCD distance' and 'Multi line
> Simultaneous reading'.

+:------:+----------------------------------------------------+:-----:+
| Bit0   | > Pixel without CCD distance                       | \[0\] |
+--------+----------------------------------------------------+-------+
| Bit1   | > Line without CCD distance                        | \[1\] |
+--------+----------------------------------------------------+-------+
| Bit2   | > Plane                                            | \[0\] |
+--------+----------------------------------------------------+-------+
| Bit3   | > Reserved                                         | \[0\] |
+--------+----------------------------------------------------+-------+
| Bit4   | > Pixel with CCD distance                          | \[0\] |
+--------+----------------------------------------------------+-------+
| Bit5   | > Line with CCD distance                           | \[0\] |
+--------+----------------------------------------------------+-------+
| Bit6   | > Multi line Simultaneous reading                  | \[1\] |
+--------+----------------------------------------------------+-------+
| Bit7   | > Reserved                                         | \[0\] |
+--------+----------------------------------------------------+-------+

Byte 7 Color Component

> This field specifies the color composition to be scanned. Dropout
> Color and R-G-B are supported.

+:------:+---------------------------------------------------+:-------:+
| Bit0   | > Neutral Gray Scale                              | \[0\]   |
+--------+---------------------------------------------------+---------+
| Bit1   | > Dropout Color                                   | \[1\]   |
+--------+---------------------------------------------------+---------+
| Bit2   | > R-G-B                                           | \[1\]   |
+--------+---------------------------------------------------+---------+
| Bit3   | > C-M-Y                                           | \[0\]   |
+--------+---------------------------------------------------+---------+
| Bit4   | > Reserved                                        | \[0\]   |
+--------+---------------------------------------------------+---------+
| Bit5   | > Reserved                                        | \[0\]   |
+--------+---------------------------------------------------+---------+
| Bit6   | > Reserved                                        | \[0\]   |
+--------+---------------------------------------------------+---------+
| Bit7   | > Extend bit                                      | \[0\]   |
+--------+---------------------------------------------------+---------+

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

+:------:+-------------------------------------------------------------+
| Bit0-3 | > First component color                                     |
+--------+-------------------------------------------------------------+
| Bit4-7 | > Second component color                                    |
+--------+-------------------------------------------------------------+

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

+:------:+-------------------------------------------------------------+
| Bit0-3 | > Third component color                                     |
+--------+-------------------------------------------------------------+
| Bit4-7 | > Fourth component color                                    |
+--------+-------------------------------------------------------------+

Byte 10 Output Bit Depth / Dot a Color Support

> This field specifies the number of bits of a single color data. This
> unit supports 16bit.

+:------:+----------------------------------------------------+:-----:+
| Bit0   | > 1bit a color                                     | \[0\] |
+--------+----------------------------------------------------+-------+
| Bit1   | > 8bit a color                                     | \[0\] |
+--------+----------------------------------------------------+-------+
| Bit2   | > 10bit a color                                    | \[0\] |
+--------+----------------------------------------------------+-------+
| Bit3   | > 12bit a color                                    | \[0\] |
+--------+----------------------------------------------------+-------+
| Bit4   | > 14bit a color                                    | \[0\] |
+--------+----------------------------------------------------+-------+
| Bit5   | > 16bit a color                                    | \[1\] |
+--------+----------------------------------------------------+-------+
| Bit6   | > Reserved                                         | \[0\] |
+--------+----------------------------------------------------+-------+
| Bit7   | > Extend bit                                       | \[0\] |
+--------+----------------------------------------------------+-------+

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

+:-------:+---------------------------------------------------+:-----:+
| Bit0    | > Analog Gamma                                    | \[0\] |
+---------+---------------------------------------------------+-------+
| Bit1    | > Exposure Time                                   | \[0\] |
+---------+---------------------------------------------------+-------+
| Bit2    | > Analog Gain                                     | \[0\] |
+---------+---------------------------------------------------+-------+
| Bit3    | > Digital Gain                                    | \[0\] |
+---------+---------------------------------------------------+-------+
| Bit4    | > Analog Shift                                    | \[0\] |
+---------+---------------------------------------------------+-------+
| Bit5    | > Analog Offset                                   | \[0\] |
+---------+---------------------------------------------------+-------+
| Bit6    | > Exposure Value                                  | \[1\] |
+---------+---------------------------------------------------+-------+
| Bit7    | > Extend bit                                      | \[0\] |
+---------+---------------------------------------------------+-------+

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

**\**
**2-2-2-5. Other information page**

Other information page

+------:+:-----------:+:-----------:+:-----------:+:-----------:+:-----------:+:-----------:+:-----------:+:-----------:+
| Bit   | 7           | 6           | 5           | 4           | 3           | 2           | 1           | 0           |
|       |             |             |             |             |             |             |             |             |
| Byte  |             |             |             |             |             |             |             |             |
+-------+-------------+-------------+-------------+-------------+-------------+-------------+-------------+-------------+
| 0     | Peripheral Qualifier                    | Peripheral Device Type                                              |
|       |                                         |                                                                     |
|       | \[0\]                                   | \[6=00110b\]                                                        |
|       |                                         |                                                                     |
|       | \[011b\](\*1)                           | \[1Fh=11111b\](\*1)                                                 |
+-------+-----------------------------------------+---------------------------------------------------------------------+
| 1     | Page code \[E1h\]                                                                                             |
+-------+---------------------------------------------------------------------------------------------------------------+
| 2     | Reserved \[0\]                                                                                                |
+-------+---------------------------------------------------------------------------------------------------------------+
| 3     | Page length \[35d=23h\]                                                                                       |
+-------+---------------------------------------------------------------------------------------------------------------+
| 4, 5  | > Host cooperation function (initiator cooperation execution processing)                                      |
|       | >                                                                                                             |
|       | > Byte 4 \[83h\] (SA-21/SA-30)                                                                                |
|       | >                                                                                                             |
|       | > \[82h\] (Adapter and holder other than the above)                                                           |
|       | >                                                                                                             |
|       | > Byte 5 \[0Ch\]                                                                                              |
+-------+---------------------------------------------------------------------------------------------------------------+
| 6 to  | > Send/Read supported information (SEND/READ command support data transfer)                                   |
| 10    | >                                                                                                             |
|       | > Byte 6 \[80h\]                                                                                              |
|       | >                                                                                                             |
|       | > Byte 7 \[B0h\]                                                                                              |
|       | >                                                                                                             |
|       | > Byte 8 \[90h\]                                                                                              |
|       | >                                                                                                             |
|       | > Byte 9 \[DAh\] (SA-21/SA-30)                                                                                |
|       |                                                                                                               |
|       | \[9Ah\] (Adapter and holder other than the above)                                                             |
|       |                                                                                                               |
|       | Byte 10 \[7Bh\] (SA-21/SA-30)                                                                                 |
|       |                                                                                                               |
|       | \[78h\] (IA-21/SF-210)                                                                                        |
|       |                                                                                                               |
|       | \[7Ch\] (Adapter and holder other than the above)                                                             |
+-------+---------------------------------------------------------------------------------------------------------------+
| 11    | > Bits per a halftone mask parameter                                                                          |
|       | >                                                                                                             |
|       | > \[0\]                                                                                                       |
+-------+---------------------------------------------------------------------------------------------------------------+
| 12    | > X bit depth of Download LUT                                                                                 |
|       | >                                                                                                             |
|       | > (The number of bits in the input data of the LUT that is downloaded from the initiator)                     |
|       | >                                                                                                             |
|       | > \[0\]                                                                                                       |
+-------+---------------------------------------------------------------------------------------------------------------+
| 13    | > Y bit depth of Download LUT                                                                                 |
|       | >                                                                                                             |
|       | > (The number of bits in the output data of the LUT that is downloaded from the initiator)                    |
|       | >                                                                                                             |
|       | > \[0\]                                                                                                       |
+-------+---------------------------------------------------------------------------------------------------------------+
| 14    | > Bits per a Histogram Data                                                                                   |
|       | >                                                                                                             |
|       | > \[0\]                                                                                                       |
+-------+---------------------------------------------------------------------------------------------------------------+
| 15    | > Bits per a Max Value Data                                                                                   |
|       | >                                                                                                             |
|       | > (The number of bits of the AE maximum value)                                                                |
|       | >                                                                                                             |
|       | > \[0\]                                                                                                       |
+-------+---------------------------------------------------------------------------------------------------------------+
| 16    | > Bits per a Matrix Data                                                                                      |
|       | >                                                                                                             |
|       | > \[0\]                                                                                                       |
+-------+---------------------------------------------------------------------------------------------------------------+
| 17    | > Bits per a Filter Data                                                                                      |
|       | >                                                                                                             |
|       | > \[0\]                                                                                                       |
+-------+---------------------------------------------------------------------------------------------------------------+
| 18    | > Bits per a Shading Data                                                                                     |
|       | >                                                                                                             |
|       | > (The number of bits in each data of the shading correction coefficient)                                     |
|       | >                                                                                                             |
|       | > \[16=10h\]                                                                                                  |
+-------+---------------------------------------------------------------------------------------------------------------+
| 19    | > Bits per a Dark Current Data (The number of bits in each data of the dark voltage correction coefficient)   |
|       | >                                                                                                             |
|       | > \[0\]                                                                                                       |
+-------+---------------------------------------------------------------------------------------------------------------+
| 20,   | > Execute operation support 80                                                                                |
| 21    | >                                                                                                             |
|       | > (Function that is supported by operation code 8xh of Execute)                                               |
|       | >                                                                                                             |
|       | > Byte 20 \[03h\]                                                                                             |
|       | >                                                                                                             |
|       | > Byte 21 \[0\]                                                                                               |
+-------+---------------------------------------------------------------------------------------------------------------+
| 22,   | > Execute operation support 90                                                                                |
| 23    | >                                                                                                             |
|       | > (Function that is supported by operation code 9xh of Execute)                                               |
|       | >                                                                                                             |
|       | > Byte 22 \[02h\] (MA-21)                                                                                     |
|       | >                                                                                                             |
|       | > \[0\] (Adapter other than the above)                                                                        |
|       | >                                                                                                             |
|       | > Byte 23 \[0\]                                                                                               |
+-------+---------------------------------------------------------------------------------------------------------------+
| 24,   | > Execute operation support A0                                                                                |
| 25    | >                                                                                                             |
|       | > (Function that is supported by operation code Axh of Execute)                                               |
|       | >                                                                                                             |
|       | > Byte 24 \[01h\]                                                                                             |
|       | >                                                                                                             |
|       | > Byte 25 \[0\]                                                                                               |
+-------+---------------------------------------------------------------------------------------------------------------+
| 26,   | > Execute operation support B0                                                                                |
| 27    | >                                                                                                             |
|       | > (Function that is supported by operation code Bxh of Execute)                                               |
|       | >                                                                                                             |
|       | > Byte 26 \[19h\] (SA-21/SA-30/IA-20)                                                                         |
|       | >                                                                                                             |
|       | > \[09h\] (Adapter and holder other than the above)                                                           |
|       | >                                                                                                             |
|       | > Byte 27 \[0\]                                                                                               |
+-------+---------------------------------------------------------------------------------------------------------------+
| 28,   | > Execute operation support C0                                                                                |
| 29    | >                                                                                                             |
|       | > (Function that is supported by operation code Cxh of Execute)                                               |
|       | >                                                                                                             |
|       | > Byte 28 \[03h\]                                                                                             |
|       | >                                                                                                             |
|       | > Byte 29 \[0\]                                                                                               |
+-------+---------------------------------------------------------------------------------------------------------------+
| 30,   | > Execute operation support D0                                                                                |
| 31    | >                                                                                                             |
|       | > (Function that is supported by operation code Dxh of Execute)                                               |
|       | >                                                                                                             |
|       | > Byte 30 \[45h\] (SA-21/SA-30)                                                                               |
|       | >                                                                                                             |
|       | > \[07h\] (IA-20)                                                                                             |
|       | >                                                                                                             |
|       | > \[23h\] (SF-210)                                                                                            |
|       | >                                                                                                             |
|       | > \[0\] (Adapter and holder other than the above)                                                             |
|       | >                                                                                                             |
|       | > Byte 31 \[0\]                                                                                               |
+-------+---------------------------------------------------------------------------------------------------------------+
| 32,   | > Execute operation support E0                                                                                |
| 33    | >                                                                                                             |
|       | > (Function that is supported by operation code Exh of Execute)                                               |
|       | >                                                                                                             |
|       | > Byte 32 \[0\]                                                                                               |
|       | >                                                                                                             |
|       | > Byte 33 \[0\]                                                                                               |
+-------+---------------------------------------------------------------------------------------------------------------+
| 34,   | > Execute operation support F0                                                                                |
| 35    | >                                                                                                             |
|       | > (Function that is supported by operation code Fxh of Execute)                                               |
|       | >                                                                                                             |
|       | > Byte 34 \[0\]                                                                                               |
|       | >                                                                                                             |
|       | > Byte 35 \[0\]                                                                                               |
+-------+---------------------------------------------------------------------------------------------------------------+
| 36    | > Additional Information (other additional information)                                                       |
|       | >                                                                                                             |
|       | > \[0Ch\]                                                                                                     |
+-------+---------------------------------------------------------------------------------------------------------------+
| 37    | > Volatile buffer for Initiator use (RAM buffer area)                                                         |
|       | >                                                                                                             |
|       | > \[4\]                                                                                                       |
+-------+---------------------------------------------------------------------------------------------------------------+
| 38    | > Non Volatile buffer for Initiator use (non-volatile memory buffer area)                                     |
|       | >                                                                                                             |
|       | > \[0\]                                                                                                       |
+-------+---------------------------------------------------------------------------------------------------------------+

\*1 When an invalid logical unit selection is performed

Byte 4 and 5 Host cooperation function

> This field specifies the processing that is executed in cooperation
> with the initiator.
>
> The initiator performs the processing of the bit that is set to 1.

+:----:+:----:+:-------------------------------:+:-----------:+:--------:+
| Byte | Bit  |                                 | SA-21/SA-30 | Other    |
|      |      |                                 |             | adapters |
|      |      |                                 | /IA-20      |          |
+------+------+---------------------------------+-------------+----------+
| 4    | 0    | > Thumbnail created by driver   | 1           | 0        |
+------+------+---------------------------------+-------------+----------+
|      | 1    | > Averaging multiple reading by | 1           | 1        |
|      |      | > driver                        |             |          |
+------+------+---------------------------------+-------------+----------+
|      | 2    | > Registration gap resolved by  | 0           | 0        |
|      |      | > driver                        |             |          |
+------+------+---------------------------------+-------------+----------+
|      | 3    | > Dark voltage data created by  | 0           | 0        |
|      |      | > driver                        |             |          |
+------+------+---------------------------------+-------------+----------+
|      | 4    | > Shading calibration data      | 0           | 0        |
|      |      | > created by driver             |             |          |
+------+------+---------------------------------+-------------+----------+
|      | 5    | > Auto Focus by driver          | 0           | 0        |
+------+------+---------------------------------+-------------+----------+
|      | 6    | > Shading correction by driver  | 0           | 0        |
+------+------+---------------------------------+-------------+----------+
|      | 7    | > Extend bit                    | 1           | 1        |
+------+------+---------------------------------+-------------+----------+
| 5    | 0    | > 3 line simultaneous reading   | 0           | 0        |
|      |      | > process by driver             |             |          |
+------+------+---------------------------------+-------------+----------+
|      | 1    | > Pitch in the main-scanning    | 0           | 0        |
|      |      | > direction by driver           |             |          |
+------+------+---------------------------------+-------------+----------+
|      | 2    | > Truncated by driver           | 1           | 1        |
+------+------+---------------------------------+-------------+----------+
|      | 3    | > CCD Data Created by Driver    | 1           | 1        |
+------+------+---------------------------------+-------------+----------+
|      | 4 to | > Reserved                      | 0           | 0        |
|      | 6    |                                 |             |          |
+------+------+---------------------------------+-------------+----------+
|      | 7    | > Extend bit                    | 0           | 0        |
+------+------+---------------------------------+-------------+----------+

Byte 6 to 10 Send/Read supported information

> This field specifies the data transfer that is supported by the Send
> and the Read commands.
>
> The data transfer of the bit that is set to 1 is supported.
>
> However, setting byte 7 bit5 'Shading Data writing supported' to \[0\]
> when the shading correction that is being performed by the Set
> Parameter command becomes an error indicates that the recovery
> operation such as transferring the previous shading data from the host
> to the unit is not necessary and the previous shading data can be
> recovered in the unit.

+------+------+-------------------------------+:--------:+:--------:+:---------:+
|      |      |                               | \[SA-21/ | \[IA-20/ | \[Other\] |
|      |      |                               |          |          |           |
|      |      |                               | SA-30\]  | SF-210\] |           |
+------+------+-------------------------------+----------+----------+-----------+
| Byte | Bit0 | Halftone mask reading         | \[0\]    | \[0\]    | \[0\]     |
| 6    |      | supported                     |          |          |           |
+------+------+-------------------------------+----------+----------+-----------+
|      | Bit1 | Halftone mask writing         | \[0\]    | \[0\]    | \[0\]     |
|      |      | supported                     |          |          |           |
+------+------+-------------------------------+----------+----------+-----------+
|      | Bit2 | Gamma function reading        | \[0\]    | \[0\]    | \[0\]     |
|      |      | supported                     |          |          |           |
+------+------+-------------------------------+----------+----------+-----------+
|      | Bit3 | Gamma function writing        | \[0\]    | \[0\]    | \[0\]     |
|      |      | supported                     |          |          |           |
+------+------+-------------------------------+----------+----------+-----------+
|      | Bit4 | Histogram Data reading        | \[0\]    | \[0\]    | \[0\]     |
|      |      | supported                     |          |          |           |
+------+------+-------------------------------+----------+----------+-----------+
|      | Bit5 | Max Value Data reading        | \[0\]    | \[0\]    | \[0\]     |
|      |      | supported                     |          |          |           |
+------+------+-------------------------------+----------+----------+-----------+
|      | Bit6 | Reserved                      | \[0\]    | \[0\]    | \[0\]     |
+------+------+-------------------------------+----------+----------+-----------+
|      | Bit7 | Extend bit                    | \[1\]    | \[1\]    | \[1\]     |
+------+------+-------------------------------+----------+----------+-----------+

+:-----+:-----+:------------------------------+:--------:+:--------:+:---------:+
|      |      |                               | \[SA-21/ | \[IA-20/ | \[Other\] |
|      |      |                               |          |          |           |
|      |      |                               | SA-30\]  | SF-210\] |           |
+------+------+-------------------------------+----------+----------+-----------+
| Byte | Bit0 | Matrix Data reading supported | \[0\]    | \[0\]    | \[0\]     |
| 7    |      |                               |          |          |           |
+------+------+-------------------------------+----------+----------+-----------+
|      | Bit1 | Matrix Data writing supported | \[0\]    | \[0\]    | \[0\]     |
+------+------+-------------------------------+----------+----------+-----------+
|      | Bit2 | Filter Data reading supported | \[0\]    | \[0\]    | \[0\]     |
+------+------+-------------------------------+----------+----------+-----------+
|      | Bit3 | Filter Data writing supported | \[0\]    | \[0\]    | \[0\]     |
+------+------+-------------------------------+----------+----------+-----------+
|      | Bit4 | Shading Data reading          | \[1\]    | \[1\]    | \[1\]     |
|      |      | supported                     |          |          |           |
+------+------+-------------------------------+----------+----------+-----------+
|      | Bit5 | Shading Data writing          | \[1\]    | \[1\]    | \[1\]     |
|      |      | supported                     |          |          |           |
+------+------+-------------------------------+----------+----------+-----------+
|      | Bit6 | Reserved                      | \[0\]    | \[0\]    | \[0\]     |
+------+------+-------------------------------+----------+----------+-----------+
|      | Bit7 | Extend bit                    | \[1\]    | \[1\]    | \[1\]     |
+------+------+-------------------------------+----------+----------+-----------+

+:-----+:-----+:------------------------------+:--------:+:--------:+:---------:+
|      |      |                               | \[SA-21/ | \[IA-20/ | \[Other\] |
|      |      |                               |          |          |           |
|      |      |                               | SA-30\]  | SF-210\] |           |
+------+------+-------------------------------+----------+----------+-----------+
| Byte | Bit0 | Dark Voltage Data reading     | \[0\]    | \[0\]    | \[0\]     |
| 8    |      | supported                     |          |          |           |
+------+------+-------------------------------+----------+----------+-----------+
|      | Bit1 | Dark Voltage Data writing     | \[0\]    | \[0\]    | \[0\]     |
|      |      | supported                     |          |          |           |
+------+------+-------------------------------+----------+----------+-----------+
|      | Bit2 | Magnetic Data reading         | \[0\]    | \[0\]    | \[0\]     |
|      |      | supported                     |          |          |           |
+------+------+-------------------------------+----------+----------+-----------+
|      | Bit3 | Magnetic Data writing         | \[0\]    | \[0\]    | \[0\]     |
|      |      | supported                     |          |          |           |
+------+------+-------------------------------+----------+----------+-----------+
|      | Bit4 | Cooperation parameters        | \[1\]    | \[1\]    | \[1\]     |
|      |      | reading supported             |          |          |           |
+------+------+-------------------------------+----------+----------+-----------+
|      | Bit5 | Boundary data reading         | \[0\]    | \[0\]    | \[0\]     |
|      |      | supported                     |          |          |           |
+------+------+-------------------------------+----------+----------+-----------+
|      | Bit6 | Boundary data writing         | \[0\]    | \[0\]    | \[0\]     |
|      |      | supported                     |          |          |           |
+------+------+-------------------------------+----------+----------+-----------+
|      | Bit7 | Extend bit                    | \[1\]    | \[1\]    | \[1\]     |
+------+------+-------------------------------+----------+----------+-----------+

+:-----+:-----+:------------------------------+:--------:+:--------:+:---------:+
|      |      |                               | \[SA-21/ | \[IA-20/ | \[Other\] |
|      |      |                               |          |          |           |
|      |      |                               | SA-30\]  | SF-210\] |           |
+------+------+-------------------------------+----------+----------+-----------+
| Byte | Bit0 | Analog Gamma reading          | \[0\]    | \[0\]    | \[0\]     |
| 9    |      | supported                     |          |          |           |
+------+------+-------------------------------+----------+----------+-----------+
|      | Bit1 | Analog Gain reading supported | \[1\]    | \[1\]    | \[1\]     |
+------+------+-------------------------------+----------+----------+-----------+
|      | Bit2 | Digital Gain reading          | \[0\]    | \[0\]    | \[0\]     |
|      |      | supported                     |          |          |           |
+------+------+-------------------------------+----------+----------+-----------+
|      | Bit3 | Exposure Value reading        | \[1\]    | \[1\]    | \[1\]     |
|      |      | supported                     |          |          |           |
+------+------+-------------------------------+----------+----------+-----------+
|      | Bit4 | Setup Information reading     | \[1\]    | \[1\]    | \[1\]     |
|      |      | supported                     |          |          |           |
+------+------+-------------------------------+----------+----------+-----------+
|      | Bit5 | Setup Information writing     | \[0\]    | \[0\]    | \[0\]     |
|      |      | supported                     |          |          |           |
+------+------+-------------------------------+----------+----------+-----------+
|      | Bit6 | Perforation Information       | \[1\]    | \[0\]    | \[0\]     |
|      |      | reading supported             |          |          |           |
+------+------+-------------------------------+----------+----------+-----------+
|      | Bit7 | Extend bit                    | \[1\]    | \[1\]    | \[1\]     |
+------+------+-------------------------------+----------+----------+-----------+

+:-----+:-----+:------------------------------+:--------:+:--------:+:---------:+
|      |      |                               | \[SA-21/ | \[IA-20/ | \[Other\] |
|      |      |                               |          |          |           |
|      |      |                               | SA-30\]  | SF-210\] |           |
+------+------+-------------------------------+----------+----------+-----------+
| Byte | Bit0 | Boundary Type2 data reading   | \[1\]    | \[0\]    | \[0\]     |
| 10   |      | supported                     |          |          |           |
+------+------+-------------------------------+----------+----------+-----------+
|      | Bit1 | Boundary Type2 data writing   | \[1\]    | \[0\]    | \[0\]     |
|      |      | supported                     |          |          |           |
+------+------+-------------------------------+----------+----------+-----------+
|      | Bit2 | Initial WB Exposure Value     | \[0\]    | \[0\]    | \[1\]     |
|      |      | reading supported             |          |          |           |
+------+------+-------------------------------+----------+----------+-----------+
|      | Bit3 | CCD data reading supported    | \[1\]    | \[1\]    | \[1\]     |
+------+------+-------------------------------+----------+----------+-----------+
|      | Bit4 | Driver Soft Version reading   | \[1\]    | \[1\]    | \[1\]     |
|      |      | supported                     |          |          |           |
+------+------+-------------------------------+----------+----------+-----------+
|      | Bit5 | Driver Soft Version writing   | \[1\]    | \[1\]    | \[1\]     |
|      |      | supported                     |          |          |           |
+------+------+-------------------------------+----------+----------+-----------+
|      | Bit6 | Leak data reading supported   | \[1\]    | \[1\]    | \[1\]     |
+------+------+-------------------------------+----------+----------+-----------+
|      | Bit7 | Extend bit                    | \[0\]    | \[0\]    | \[0\]     |
+------+------+-------------------------------+----------+----------+-----------+

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
> This unit supports 'Initialize' and 'Return to origin'.
>
> 'Initialize' performs the unit initialization in the same manner as
> that is performed at the start of power supply.
>
> 'Return to origin' moves the object to be scanned or the stage
> (mechanical block) to the origin position.

+:------:+:------:+:---------------------------------------:+:--------:+
| Byte   | Bit    | Operation                               | Value on |
|        |        |                                         | this     |
|        |        |                                         | unit     |
+--------+--------+-----------------------------------------+----------+
| 20     | 0      | > Initialize                            | \[1\]    |
+--------+--------+-----------------------------------------+----------+
|        | 1      | > Return to origin                      | \[1\]    |
+--------+--------+-----------------------------------------+----------+
|        | 2 to 7 | > Reserved                              | \[0\]    |
+--------+--------+-----------------------------------------+----------+
| 21     | 0 to 7 | > Reserved                              | \[0\]    |
+--------+--------+-----------------------------------------+----------+

Byte 22 and 23 Execute operation support 90

> This field specifies the function that is supported by operation code
> 9xh of EXECUTE command.
>
> This unit supports the automatic execution of auto focus.

+:---------:+:---------:+:-----------------------------------:+:---------------:+:------:+
| Byte      | Bit       | Operation                           | Value on this unit       |
+-----------+-----------+-------------------------------------+-----------------+--------+
|           |           |                                     | MA-21           | Other  |
|           |           |                                     |                 | than   |
|           |           |                                     |                 | MA-21  |
+-----------+-----------+-------------------------------------+-----------------+--------+
| 22        | 0         | > Change Unit                       | \[0\]           | \[0\]  |
+-----------+-----------+-------------------------------------+-----------------+--------+
|           | 1         | > AF Autoexec                       | \[1\]           | \[0\]  |
+-----------+-----------+-------------------------------------+-----------------+--------+
|           | 2 to 7    | > Reserved                          | \[0\]           | \[0\]  |
+-----------+-----------+-------------------------------------+-----------------+--------+
| 23        | 0 to 7    | > Reserved                          | \[0\]           | \[0\]  |
+-----------+-----------+-------------------------------------+-----------------+--------+

Byte 24 and 25 Execute operation support A0

> This field specifies the function that is supported by operation code
> Axh of EXECUTE command.
>
> This unit supports the auto focus.

+:------:+:------:+:-------------------------------------:+:---------:+
| Byte   | Bit    | Operation                             | Value on  |
|        |        |                                       | this unit |
+--------+--------+---------------------------------------+-----------+
| 24     | 0      | > Auto Focus                          | \[1\]     |
+--------+--------+---------------------------------------+-----------+
|        | 1      | > Color oriented Auto Focus           | \[0\]     |
+--------+--------+---------------------------------------+-----------+
|        | 2 to 7 | > Reserved                            | \[0\]     |
+--------+--------+---------------------------------------+-----------+
| 25     | 0 to 7 | > Reserved                            | \[0\]     |
+--------+--------+---------------------------------------+-----------+

Byte 26 and 27 Execute operation support B0

> This field specifies the function that is supported by operation code
> Bxh of EXECUTE command.
>
> This unit supports the shading measurement, dark voltage measurement,
> recording of the unit-specific data setting, and changing the
> automatic ejection time of the film.

+:---------------------:+:---------------------:+:---------------------------------------------:+:---------------------------------:+:---------------------------:+
| Byte                  | Bit                   | Operation                                     | Value on this unit                                              |
+-----------------------+-----------------------+-----------------------------------------------+-----------------------------------+-----------------------------+
|                       |                       |                                               | SA-21/SA-30                       | Other than                  |
|                       |                       |                                               |                                   |                             |
|                       |                       |                                               | /IA-20                            | SA-21/SA-30/IA-20           |
+-----------------------+-----------------------+-----------------------------------------------+-----------------------------------+-----------------------------+
| 26                    | 0                     | > Setup Shading Data                          | \[1\]                             | \[1\]                       |
+-----------------------+-----------------------+-----------------------------------------------+-----------------------------------+-----------------------------+
|                       | 1                     | > Setup Dark Current Correction Data          | \[0\]                             | \[0\]                       |
+-----------------------+-----------------------+-----------------------------------------------+-----------------------------------+-----------------------------+
|                       | 2                     | > Setup Offset Correction Data                | \[0\]                             | \[0\]                       |
+-----------------------+-----------------------+-----------------------------------------------+-----------------------------------+-----------------------------+
|                       | 3                     | > Write Data On Device Dependence             | \[1\]                             | \[1\]                       |
+-----------------------+-----------------------+-----------------------------------------------+-----------------------------------+-----------------------------+
|                       | 4                     | > Change of Auto Unload time                  | \[1\]                             | \[0\]                       |
+-----------------------+-----------------------+-----------------------------------------------+-----------------------------------+-----------------------------+
|                       | 5 to 7                | > Reserved                                    | \[0\]                             | \[0\]                       |
+-----------------------+-----------------------+-----------------------------------------------+-----------------------------------+-----------------------------+
| 27                    | 0 to 7                | > Reserved                                    | \[0\]                             | \[0\]                       |
+-----------------------+-----------------------+-----------------------------------------------+-----------------------------------+-----------------------------+

Byte 28 and 29 Execute operation support C0

> This field specifies the function that is supported by operation code
> Cxh of EXECUTE command.
>
> This unit supports the stage movement and the focus movement.

+:------:+:------:+:--------------------------------------:+:--------:+
| Byte   | Bit    | Operation                              | Value on |
|        |        |                                        | this     |
|        |        |                                        | unit     |
+--------+--------+----------------------------------------+----------+
| 28     | 0      | > Stage Move                           | \[1\]    |
+--------+--------+----------------------------------------+----------+
|        | 1      | > Focus Move                           | \[1\]    |
+--------+--------+----------------------------------------+----------+
|        | 2 to 7 | > Reserved                             | \[0\]    |
+--------+--------+----------------------------------------+----------+
| 29     | 0 to 7 | > Reserved                             | \[0\]    |
+--------+--------+----------------------------------------+----------+

Byte 30 and 31 Execute operation support D0

> This field specifies the function that is supported by operation code
> Dxh of EXECUTE command.
>
> This unit supports the loading/unloading of the object to be scanned.
> For the movement, the absolute address specification is supported.

+:-------:+:-------:+:-----------------------:+:-------------------------:+:-------------------------:+:-------------------------:+:-------------------------:+:-------------------------:+
| Byte    | Bit     | Operation               | Value on this unit                                                                                                                        |
+---------+---------+-------------------------+---------------------------+---------------------------+---------------------------+---------------------------+---------------------------+
|         |         |                         | MA-21                     | SA-21                     | SA-30                     | IA-20                     | SF-210                    |
|         |         |                         |                           |                           |                           |                           |                           |
|         |         |                         | (\*1)                     |                           |                           |                           |                           |
+---------+---------+-------------------------+---------------------------+---------------------------+---------------------------+---------------------------+---------------------------+
| 30      | 0       | > Unload object         | \[0\]                     | \[1\]                     | \[1\]                     | \[1\]                     | \[1\]                     |
+---------+---------+-------------------------+---------------------------+---------------------------+---------------------------+---------------------------+---------------------------+
|         | 1       | > Load object           | \[0\]                     | \[0\]                     | \[0\]                     | \[1\]                     | \[1\]                     |
+---------+---------+-------------------------+---------------------------+---------------------------+---------------------------+---------------------------+---------------------------+
|         | 2       | > Absolute positioning  | \[0\]                     | \[1\]                     | \[1\]                     | \[1\]                     | \[0\]                     |
+---------+---------+-------------------------+---------------------------+---------------------------+---------------------------+---------------------------+---------------------------+
|         | 3       | > Relative positioning  | \[0\]                     | \[0\]                     | \[0\]                     | \[0\]                     | \[0\]                     |
+---------+---------+-------------------------+---------------------------+---------------------------+---------------------------+---------------------------+---------------------------+
|         | 4       | > Rotate                | \[0\]                     | \[0\]                     | \[0\]                     | \[0\]                     | \[0\]                     |
+---------+---------+-------------------------+---------------------------+---------------------------+---------------------------+---------------------------+---------------------------+
|         | 5       | > FD Move               | \[0\]                     | \[0\]                     | \[0\]                     | \[0\]                     | \[1\]                     |
+---------+---------+-------------------------+---------------------------+---------------------------+---------------------------+---------------------------+---------------------------+
|         | 6       | > SA Lock               | \[0\]                     | \[1\]                     | \[1\]                     | \[0\]                     | \[0\]                     |
+---------+---------+-------------------------+---------------------------+---------------------------+---------------------------+---------------------------+---------------------------+
|         | 7       | > Reserved              | \[0\]                     | \[0\]                     | \[0\]                     | \[0\]                     | \[0\]                     |
+---------+---------+-------------------------+---------------------------+---------------------------+---------------------------+---------------------------+---------------------------+
| 31      | 0 to 7  | > Reserved              | \[0\]                     | \[0\]                     | \[0\]                     | \[0\]                     | \[0\]                     |
+---------+---------+-------------------------+---------------------------+---------------------------+---------------------------+---------------------------+---------------------------+

\*1 Each holder of FH3, FHG1, and FHA1 is included.

Byte 32 and 33 Execute operation support E0

> This field specifies the function that is supported by operation code
> Exh of EXECUTE command.

Byte 34 and 35 Execute operation support F0

> This field specifies the function that is supported by operation code
> Fxh of EXECUTE command.

Byte 36 Additional Information

> This field specifies the other additional information.

+:-----+:--------------------+:----------------------------:+:--------:+
| Bit  |                     | Explanations of operation    | Value on |
|      |                     |                              | this     |
|      |                     |                              | unit     |
+------+---------------------+------------------------------+----------+
| Bit0 | Hot exchangeable to | > The attached adapter can   | \[0\]    |
|      | unequipped unit     | > be exchanged with the      |          |
|      | with notice         | > power turned ON, and it is |          |
|      |                     | > possible to inform the     |          |
|      |                     | > initiator that the adapter |          |
|      |                     | > has been exchanged.        |          |
+------+---------------------+------------------------------+----------+
| Bit1 | Scanned object      | > The scanned object can be  | \[0\]    |
|      | exchangeable with   | > exchanged, and it is       |          |
|      | notice              | > possible to inform the     |          |
|      |                     | > initiator that the object  |          |
|      |                     | > has been exchanged.        |          |
+------+---------------------+------------------------------+----------+
| Bit2 | Hot exchangeable to | > The attached adapter can   | \[1\]    |
|      | unequipped unit     | > be exchanged with the      |          |
|      | without notice      | > power turned ON, but it is |          |
|      |                     | > not possible to inform the |          |
|      |                     | > initiator that the adapter |          |
|      |                     | > has been exchanged.        |          |
+------+---------------------+------------------------------+----------+
| Bit3 | Scanned object      | > The scanned object can be  | \[1\]    |
|      | exchangeable        | > exchanged, but it is not   |          |
|      | without notice      | > possible to inform the     |          |
|      |                     | > initiator that the object  |          |
|      |                     | > has been exchanged.        |          |
+------+---------------------+------------------------------+----------+
| Bit4 | Histogram Scanning  | > Scanning for creating the  | \[0\]    |
| to 6 |                     | > histogram of the image     |          |
|      |                     | > data                       |          |
+------+---------------------+------------------------------+----------+
| Bit7 | Extend bit          | > Extension bit              | \[0\]    |
+------+---------------------+------------------------------+----------+

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

**\**
**2-2-2-6. Operation code setting page**

+----------:+:---------:+:---------:+:---------:+:---------:+:---------:+:---------:+:---------:+:---------:+
| Bit       | 7         | 6         | 5         | 4         | 3         | 2         | 1         | 0         |
|           |           |           |           |           |           |           |           |           |
| Byte      |           |           |           |           |           |           |           |           |
+-----------+-----------+-----------+-----------+-----------+-----------+-----------+-----------+-----------+
| 0         | Peripheral Qualifier              | Peripheral Device Type                                    |
|           |                                   |                                                           |
|           | \[0\]                             | \[6=00110b\]                                              |
|           |                                   |                                                           |
|           | \[011b\](\*1)                     | \[1Fh=11111b\](\*1)                                       |
+-----------+-----------------------------------+-----------------------------------------------------------+
| 1         | Page code \[E2h\]                                                                             |
+-----------+-----------------------------------------------------------------------------------------------+
| 2         | Reserved \[0\]                                                                                |
+-----------+-----------------------------------------------------------------------------------------------+
| 3         | Page length \[m-3\]                                                                           |
+-----------+-----------------------------------------------------------------------------------------------+
| 4         | Number of Operation code (=n)                                                                 |
|           |                                                                                               |
|           | (The number of operation codes for which setting of each value is necessary)                  |
|           |                                                                                               |
|           | \[1\]                                                                                         |
+-----------+-----------------------------------------------------------------------------------------------+
| 5         | Operation Code                                                                                |
+-----------+-----------------------------------------------------------------------------------------------+
| 6 to 9    | Minimum value of 1st Value                                                                    |
+-----------+-----------------------------------------------------------------------------------------------+
| 10 to 13  | Maximum value of 1st Value                                                                    |
+-----------+-----------------------------------------------------------------------------------------------+
| 14 to 17  | Minimum value of 2nd Value                                                                    |
+-----------+-----------------------------------------------------------------------------------------------+
| 18 to 21  | Maximum value of 2nd Value                                                                    |
+-----------+-----------------------------------------------------------------------------------------------+
| 22 to 25  | Minimum value of Speed                                                                        |
+-----------+-----------------------------------------------------------------------------------------------+
| 26 to 29  | Maximum value of Speed                                                                        |
+-----------+-----------------------------------------------------------------------------------------------+
| m=5+25\*n | \[n-1 times, repetition of byte 5 to 29\]                                                     |
+-----------+-----------------------------------------------------------------------------------------------+

\*1 When an invalid logical unit selection is performed

Byte 4 Number of Operation code

> This field specifies the number of operation codes for which each
> value is set. This field is set to 1 in this unit.

Byte 5 and after (5+25\*n)

> This field specifies the operation codes for which each value is set.
> The 24 bytes following this field indicate the operation parameter for
> the unit of the ID specified in this field. The operation code that is
> used in this unit is shown below.

  ------------------------------------------ ----------------------------
  Contents of operation                             Operation code

  Setting of the medium ejection time                    B4h
  ------------------------------------------ ----------------------------

Set value of the operation code setting page

+---------------------------------------+:--------------:+:--------------:+
| Operation code                        | B4                              |
|                                       +----------------+----------------+
|                                       | Byte           | Set value      |
+---------------------------------------+----------------+----------------+
| Operation Code                        | 5              | B4h            |
+---------------------------------------+----------------+----------------+
| Minimum value of 1^st^ Value          | 6 to 9         | 60             |
+---------------------------------------+----------------+----------------+
| Maximum value of 1^st^ Value          | 10 to 13       | 3600           |
+---------------------------------------+----------------+----------------+
| Minimum value of 2^nd^ Value          | 14 to 17       | 0              |
+---------------------------------------+----------------+----------------+
| Maximum value of 2^nd^ Value          | 18 to 21       | 1              |
+---------------------------------------+----------------+----------------+
| Minimum value of Speed                | 22 to 25       | 0              |
+---------------------------------------+----------------+----------------+
| Maximum value of Speed                | 26 to 29       | 0              |
+---------------------------------------+----------------+----------------+

**2-2-2-7. CCD measurement setting page**

+-------:+:-----------:+:-----------:+:-----------:+:-----------:+:-----------:+:-----------:+:-----------:+:-----------:+
| Bit    | 7           | 6           | 5           | 4           | 3           | 2           | 1           | 0           |
|        |             |             |             |             |             |             |             |             |
| Byte   |             |             |             |             |             |             |             |             |
+--------+-------------+-------------+-------------+-------------+-------------+-------------+-------------+-------------+
| 0      | Peripheral Qualifier                    | Peripheral Device Type                                              |
|        |                                         |                                                                     |
|        | \[0\]                                   | \[6=00110b\]                                                        |
|        |                                         |                                                                     |
|        | \[011b\](\*1)                           | \[1Fh=11111b\](\*1)                                                 |
+--------+-----------------------------------------+---------------------------------------------------------------------+
| 1      | Page code \[E3h\]                                                                                             |
+--------+---------------------------------------------------------------------------------------------------------------+
| 2      | Reserved \[0\]                                                                                                |
+--------+---------------------------------------------------------------------------------------------------------------+
| 3      | Page length \[\]                                                                                              |
+--------+---------------------------------------------------------------------------------------------------------------+
| 4, 5   | Color of CCD Data                                                                                             |
+--------+---------------------------------------------------------------------------------------------------------------+
| 6, 7   | Resolution of CCD Data                                                                                        |
+--------+---------------------------------------------------------------------------------------------------------------+
| 8      | Scanning Number of CCD Data                                                                                   |
|        |                                                                                                               |
|        | (The number of scanning times for the CCD measurement)                                                        |
+--------+---------------------------------------------------------------------------------------------------------------+
| 9      | Type of CCD Data                                                                                              |
|        |                                                                                                               |
|        | (The number of types for the CCD measurement)                                                                 |
+--------+---------------------------------------------------------------------------------------------------------------+
| 10     | A number of CCD Data \[n\]                                                                                    |
|        |                                                                                                               |
|        | (The number of measurement points for the CCD measurement)                                                    |
+--------+---------------------------------------------------------------------------------------------------------------+
| 11, 12 | First value of CCD Data                                                                                       |
|        |                                                                                                               |
|        | (Ratio of the first point for the CCD measurement)                                                            |
+--------+---------------------------------------------------------------------------------------------------------------+
| 13, 14 | Second value of CCD Data                                                                                      |
|        |                                                                                                               |
|        | (Ratio of the second point for the CCD measurement)                                                           |
+--------+---------------------------------------------------------------------------------------------------------------+
| :      | :                                                                                                             |
+--------+---------------------------------------------------------------------------------------------------------------+
| n+10,  | nth value of CCD Data                                                                                         |
| n+11   |                                                                                                               |
|        | (Ratio of the nth point for the CCD measurement)                                                              |
+--------+---------------------------------------------------------------------------------------------------------------+

\*1 When an invalid logical unit selection is performed

Byte 4 and 5 Color of CCD Data

> This field specifies the color for the CCD measurement. The color in
> which 1 is set is used for the CCD measurement. (Two or more colors
> may be specified simultaneously.)
>
> Byte 4

  ------------- ---------------------------------------------------------
  Bit 0         R \[0:OFF/1:ON\]

  Bit 1         G \[0:OFF/1:ON\]

  Bit 2         B \[0:OFF/1:ON\]

  Bit 3         NG \[0:OFF/1:ON\]

  Bit 4         C \[0:OFF/1:ON\]

  Bit 5         M \[0:OFF/1:ON\]

  Bit 6         Y \[0:OFF/1:ON\]

  Bit 7         K \[0:OFF/1:ON\]
  ------------- ---------------------------------------------------------

Byte 5

  ------------------ ----------------------------------------------------
  Bit 0 to 7         Reserved

  ------------------ ----------------------------------------------------

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

+-------:+:----------:+:----------:+:----------:+:----------:+:----------:+:----------:+:----------:+:----------:+
| Bit    | 7          | 6          | 5          | 4          | 3          | 2          | 1          | 0          |
|        |            |            |            |            |            |            |            |            |
| Byte   |            |            |            |            |            |            |            |            |
+--------+------------+------------+------------+------------+------------+------------+------------+------------+
| 0      | Operation code \[15h\]                                                                                |
+--------+--------------------------------------+------------+--------------------------------------+------------+
| 1      | Logical unit number                  | PF         | Reserved                             | SP         |
|        |                                      |            |                                      |            |
|        | \[0\]                                | \[1\]      | \[0\]                                | \[0\]      |
+--------+--------------------------------------+------------+--------------------------------------+------------+
| 2, 3   | Reserved \[0\]                                                                                        |
+--------+-------------------------------------------------------------------------------------------------------+
| 4      | Parameter list length \[0,4,12,20\]                                                                   |
+--------+-------------------------------------------------------------------------------------------------------+
| 5      | Control byte \[0\]                                                                                    |
+--------+-------------------------------------------------------------------------------------------------------+

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

+:-----------------------:+:---------------------:+:------------------:+
| Status                  | Sense data            | Remarks            |
+-------------------------+-----------------------+--------------------+
| > When an initiator     | MODE PARAMETERS       | Creates the UNIT   |
| > sends a MODE SELECT   | CHANGED               | ATTENTION status   |
| > command that changes  |                       | for all initiators |
| > the parameter         | > (The MODE parameter | other than the     |
| > applicable to other   | > is changed by other | initiator that     |
| > initiators            | > initiator when the  | issued the MODE    |
|                         | > multi-initiator is  | SELECT command.    |
|                         | > set.)               |                    |
|                         |                       |                    |
|                         | 06h-2Ah-01h-00h       |                    |
+-------------------------+-----------------------+--------------------+
| When a parameter list   | PARAMETER LIST LENGTH | Terminates with    |
| length that results in  | ERROR                 | the CHECK          |
| truncation of any       |                       | CONDITION status.  |
| parameter for           | (The parameter length |                    |
| descriptor, header, or  | is illegal.)          |                    |
| page is specified       |                       |                    |
|                         | 05h-1Ah-00h-00h       |                    |
+-------------------------+-----------------------+--------------------+
| a)  When the initiator  | INVALID FIELD IN      | Terminates the     |
|     changes the field   | PARAMETER LIST        | MODE SELECT        |
|     that is not         |                       | command with the   |
|     changeable as       | (Some illegal data    | CHECK CONDITION    |
|     reported by this    | exists in the         | status without     |
|     unit to the value   | parameter.)           | changing any mode  |
|     other than the      |                       | parameter.         |
|     current value       | 05h-26h-00h-00h       |                    |
|                         |                       |                    |
| b)  When the initiator  |                       |                    |
|     sends a MODE SELECT |                       |                    |
|     header, block       |                       |                    |
|     descriptor, or page |                       |                    |
|     header for which a  |                       |                    |
|     non-supported value |                       |                    |
|     is set in the       |                       |                    |
|     reserved field      |                       |                    |
|                         |                       |                    |
| c)  When the initiator  |                       |                    |
|     sends a page of the |                       |                    |
|     length that is      |                       |                    |
|     different from the  |                       |                    |
|     parameter length    |                       |                    |
|     reported for that   |                       |                    |
|     page by the MODE    |                       |                    |
|     SENSE command       |                       |                    |
|                         |                       |                    |
| d)  When the initiator  |                       |                    |
|     sends a parameter   |                       |                    |
|     that has a value    |                       |                    |
|     exceeding the       |                       |                    |
|     support range of    |                       |                    |
|     this unit           |                       |                    |
|                         |                       |                    |
| e)  When the initiator  |                       |                    |
|     sets a value other  |                       |                    |
|     than 0 in the       |                       |                    |
|     reserved field of   |                       |                    |
|     the mode parameter  |                       |                    |
+-------------------------+-----------------------+--------------------+

\- Mode parameter of this unit

Table 2-3-2 Mode parameter header

+-------:+:----------:+:----------:+:----------:+:----------:+:----------:+:----------:+:----------:+:----------:+
| Bit    | 7          | 6          | 5          | 4          | 3          | 2          | 1          | 0          |
|        |            |            |            |            |            |            |            |            |
| Byte   |            |            |            |            |            |            |            |            |
+--------+------------+------------+------------+------------+------------+------------+------------+------------+
| 0      | Mode Data Length                                                                                      |
+--------+-------------------------------------------------------------------------------------------------------+
| 1      | Medium Type \[0\]                                                                                     |
+--------+-------------------------------------------------------------------------------------------------------+
| 2      | Device-Specific Parameter \[0\]                                                                       |
+--------+-------------------------------------------------------------------------------------------------------+
| 3      | Block Descriptor Length \[0, 8\]                                                                      |
+--------+-------------------------------------------------------------------------------------------------------+

1)  When the MODE SENSE command is used, the Mode Data Length field
    specifies the length in bytes of the following data that can be
    transferred. Mode Data Length does not include itself. When using
    the MODE SELECT command, the Mode Data Length field is set to
    'Reserved'.

2)  Medium Type is always set to 0.

3)  Device-Specific Parameter is always set to 0.

4)  Block Descriptor Length specifies the length in bytes of all the
    block descriptors. In this unit, 0 or 8 is set. Block Descriptor
    Length of 0 means that the block descriptor is not included in the
    mode parameter list; however, this is not regarded as an error.

\- Mode parameter block descriptor

Table 2-3-3 Mode parameter block descriptor

+-------:+:----------:+:----------:+:----------:+:----------:+:----------:+:----------:+:----------:+:----------:+
| Bit    | 7          | 6          | 5          | 4          | 3          | 2          | 1          | 0          |
|        |            |            |            |            |            |            |            |            |
| Byte   |            |            |            |            |            |            |            |            |
+--------+------------+------------+------------+------------+------------+------------+------------+------------+
| 0      | Density Code \[0\]                                                                                    |
+--------+-------------------------------------------------------------------------------------------------------+
| 1      | (MSB)                                                                                                 |
+--------+-------------------------------------------------------------------------------------------------------+
| 2      | Number of Blocks \[0\]                                                                                |
+--------+-------------------------------------------------------------------------------------------------------+
| 3      | (LSB)                                                                                                 |
+--------+-------------------------------------------------------------------------------------------------------+
| 4      | Reserved \[0\]                                                                                        |
+--------+-------------------------------------------------------------------------------------------------------+
| 5      | (MSB)                                                                                                 |
+--------+-------------------------------------------------------------------------------------------------------+
| 6      | Block Length \[1\]                                                                                    |
+--------+-------------------------------------------------------------------------------------------------------+
| 7      | (LSB)                                                                                                 |
+--------+-------------------------------------------------------------------------------------------------------+

1)  Density Code field is always set to 0.

2)  Number of Blocks field is always set to 0.

3)  Block Length field is always set to 1.

\- Measurement Units page

This unit supports only the Measurement Units page.

Table 2-3-4 Measurement Units page

+-------:+:----------:+:----------:+:----------:+:----------:+:----------:+:----------:+:----------:+:----------:+
| Bit    | 7          | 6          | 5          | 4          | 3          | 2          | 1          | 0          |
|        |            |            |            |            |            |            |            |            |
| Byte   |            |            |            |            |            |            |            |            |
+--------+------------+------------+------------+------------+------------+------------+------------+------------+
| 0      | PS         | Reserved   | Operation code \[03h\]                                                      |
|        |            |            |                                                                             |
|        | \[0\]      | \[0\]      |                                                                             |
+--------+------------+------------+-----------------------------------------------------------------------------+
| 1      | Parameter length \[06h\]                                                                              |
+--------+-------------------------------------------------------------------------------------------------------+
| 2      | Basic measurement unit \[00h\]                                                                        |
+--------+-------------------------------------------------------------------------------------------------------+
| 3      | Reserved \[0\]                                                                                        |
+--------+-------------------------------------------------------------------------------------------------------+
| 4      | (MSB)                                                                                                 |
+--------+-------------------------------------------------------------------------------------------------------+
| 5      | Measurement unit divisor \[1200/Maximum resolution\]                                                  |
|        |                                                                                                       |
|        | (LSB)                                                                                                 |
+--------+-------------------------------------------------------------------------------------------------------+
| 6, 7   | Reserved \[0\]                                                                                        |
+--------+-------------------------------------------------------------------------------------------------------+

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

**\**
**2-4. RESERVE UNIT Command**

Table 2-4-1 RESERVE UNIT command

+-------:+:----------:+:----------:+:----------:+:----------:+:----------:+:----------:+:----------:+:----------:+
| Bit    | 7          | 6          | 5          | 4          | 3          | 2          | 1          | 0          |
|        |            |            |            |            |            |            |            |            |
| Byte   |            |            |            |            |            |            |            |            |
+--------+------------+------------+------------+------------+------------+------------+------------+------------+
| 0      | Operation code \[16h\]                                                                                |
+--------+--------------------------------------+------------+--------------------------------------+------------+
| 1      | Logical unit number                  | Third      | Third party device ID                | Re-served  |
|        |                                      | party      |                                      |            |
|        | \[0\]                                |            |                                      | \[0\]      |
|        |                                      | \[0 or 1\] |                                      |            |
+--------+--------------------------------------+------------+--------------------------------------+------------+
| 2 to 4 | Reserved \[0\]                                                                                        |
+--------+-------------------------------------------------------------------------------------------------------+
| 5      | Control byte \[0\]                                                                                    |
+--------+-------------------------------------------------------------------------------------------------------+

The RESERVE UNIT command is used to reserve the logical unit for the
exclusive use by the initiator.

This command requests the reservation of the entire logical unit for the
exclusive use by the initiator until it is replaced with any other valid
RESERVE UNIT command from the initiator that made the reservation, it is
released by the RELEASE UNIT command from the same initiator, or it is
released by the hard reset status or the power-ON cycle. It is
permissible for the initiator that currently makes reservation to
reserve the logical unit again.

**\
2-5. RELEASE UNIT Command**

Table 2-5-1 RELEASE UNIT command

+-------:+:----------:+:----------:+:----------:+:----------:+:----------:+:----------:+:----------:+:----------:+
| Bit    | 7          | 6          | 5          | 4          | 3          | 2          | 1          | 0          |
|        |            |            |            |            |            |            |            |            |
| Byte   |            |            |            |            |            |            |            |            |
+--------+------------+------------+------------+------------+------------+------------+------------+------------+
| 0      | Operation code \[17h\]                                                                                |
+--------+--------------------------------------+------------+--------------------------------------+------------+
| 1      | Logical unit number                  | Third      | Third party device ID                | Re-served  |
|        |                                      | party      |                                      |            |
|        | \[0\]                                |            |                                      | \[0\]      |
|        |                                      | \[0 or 1\] |                                      |            |
+--------+--------------------------------------+------------+--------------------------------------+------------+
| 2 to 4 | Reserved \[0\]                                                                                        |
+--------+-------------------------------------------------------------------------------------------------------+
| 5      | Control byte \[0\]                                                                                    |
+--------+-------------------------------------------------------------------------------------------------------+

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

+-------:+:----------:+:----------:+:----------:+:----------:+:----------:+:----------:+:----------:+:----------:+
| Bit    | 7          | 6          | 5          | 4          | 3          | 2          | 1          | 0          |
|        |            |            |            |            |            |            |            |            |
| Byte   |            |            |            |            |            |            |            |            |
+--------+------------+------------+------------+------------+------------+------------+------------+------------+
| 0      | Operation code \[1Ah\]                                                                                |
+--------+--------------------------------------+------------+------------+--------------------------------------+
| 1      | Logical unit number                  | PF         | DBD        | Reserved                             |
|        |                                      |            |            |                                      |
|        | \[0\]                                | \[1\]      | \[0 or 1\] | \[0\]                                |
+--------+-------------------------+------------+------------+------------+--------------------------------------+
| 2      | PC                      | Page code                                                                   |
+--------+-------------------------+-----------------------------------------------------------------------------+
| 3      | Reserved \[0\]                                                                                        |
+--------+-------------------------------------------------------------------------------------------------------+
| 4      | Allocation length                                                                                     |
+--------+-------------------------------------------------------------------------------------------------------+
| 5      | Control byte \[0\]                                                                                    |
+--------+-------------------------------------------------------------------------------------------------------+

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

+:------------------:+:-----------------------------------------------:+
| Code               | Parameter type                                  |
+--------------------+-------------------------------------------------+
| 00b                | Current value                                   |
|                    |                                                 |
| 01b                | Variable value                                  |
|                    |                                                 |
| 10b                | Default value                                   |
+--------------------+-------------------------------------------------+

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

+:--------------------:+:---------------------------------------------:+
| Page code            | Descriptions                                  |
+----------------------+-----------------------------------------------+
| 03h                  | Returns the Measurement Units page            |
|                      |                                               |
| 3Fh                  | Returns all pages                             |
+----------------------+-----------------------------------------------+

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

+-------:+:----------:+:----------:+:----------:+:----------:+:----------:+:----------:+:----------:+:----------:+
| Bit    | 7          | 6          | 5          | 4          | 3          | 2          | 1          | 0          |
|        |            |            |            |            |            |            |            |            |
| Byte   |            |            |            |            |            |            |            |            |
+--------+------------+------------+------------+------------+------------+------------+------------+------------+
| 0      | Operation code \[1Bh\]                                                                                |
+--------+--------------------------------------+----------------------------------------------------------------+
| 1      | Logical unit number                  | Reserved                                                       |
|        |                                      |                                                                |
|        | \[0\]                                | \[0\]                                                          |
+--------+--------------------------------------+----------------------------------------------------------------+
| 2, 3   | Reserved \[0\]                                                                                        |
+--------+-------------------------------------------------------------------------------------------------------+
| 4      | Transfer length \[0, 1, 2, 3, 4\]                                                                     |
+--------+-------------------------------------------------------------------------------------------------------+
| 5      | Control byte \[0\]                                                                                    |
+--------+-------------------------------------------------------------------------------------------------------+

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

+:------------------:+:-------------------------:+:-------------------:+
| Status             | Sense data                | Remarks             |
+--------------------+---------------------------+---------------------+
| When the default   | INVALID COMBINATION OF    | Terminates with the |
| color is specified | WINDOWS SPECIFIED         | CHECK CONDITION     |
| with other color   |                           | status.             |
| in the window      | 05h-2Ch-02h-00h           |                     |
+--------------------+---------------------------+---------------------+
| When the           | INVALID COMBINATION OF    | Terminates with the |
| overlapped setting | WINDOWS SPECIFIED         | CHECK CONDITION     |
| is performed       |                           | status.             |
| between two or     | 05h-2Ch-02h-00h           |                     |
| more windows (a    |                           |                     |
| different setting  |                           |                     |
| is performed in    |                           |                     |
| the parameter      |                           |                     |
| common to all      |                           |                     |
| windows)           |                           |                     |
+--------------------+---------------------------+---------------------+
| When Multiple      | AVERAGING MULTIPLE        | The initiator       |
| Reading is set     | READING BY DRIVER         | cooperative action  |
|                    |                           | parameter is read   |
|                    | (The averaging processing | by the READ command |
|                    | during Multiple Reading   | following the SCAN  |
|                    | is performed by the       | command and the     |
|                    | initiator.)               | averaging           |
|                    |                           | processing is       |
|                    | 09h-80h-02h-00h           | performed on the    |
|                    |                           | initiator side      |
|                    |                           | based on the        |
|                    |                           | information.        |
+--------------------+---------------------------+---------------------+
| When Thumbnail is  | THUMBNAIL CREATED BY      | The initiator       |
| set                | DRIVER                    | cooperative action  |
|                    |                           | parameter is read   |
| \(240\)            | (The thumbnail image of   | by the READ command |
|                    | the 240 film is created   | following the SCAN  |
|                    | by the initiator.)        | command and the     |
|                    |                           | thumbnail is        |
|                    | 09h-80h-01h-02h           | created on the      |
|                    |                           | initiator side      |
|                    |                           | based on the        |
|                    |                           | information. The    |
|                    |                           | initiator issues    |
|                    |                           | the SCAN command    |
|                    |                           | again after         |
|                    |                           | performing the      |
|                    |                           | necessary           |
|                    |                           | operation.          |
+--------------------+---------------------------+                     |
| (6-frame strip)    | THUMBNAIL CREATED BY      |                     |
|                    | DRIVER                    |                     |
| (36-frame strip)   |                           |                     |
|                    | (The thumbnail image of   |                     |
|                    | the strip film is created |                     |
|                    | by the initiator.)        |                     |
|                    |                           |                     |
|                    | 09h-80h-01h-06h           |                     |
+--------------------+---------------------------+---------------------+
| For two-line       | TRUNCATED BY DRIVER       | The SCAN command is |
| reading, when a    |                           | issued again. The   |
| setting other than | (The invalid data that is | excess data is      |
| the combination of | sent excessively is       | deleted on the      |
| an even-number     | deleted by the            | initiator side by   |
| start address and  | initiator.)               | the READ command    |
| an odd-number end  |                           | that is issued      |
| address is made    | 09h-80h-06h-01h           | following the SCAN  |
|                    |                           | command.            |
| When the sent data |                           |                     |
| is not a multiple  |                           |                     |
| of 512 bytes       |                           |                     |
+--------------------+---------------------------+---------------------+
| When the CCD DATA  | CCD DATA CREATED BY       | The initiator       |
| is ON while Image  | DRIVER                    | cooperative action  |
| Scanning is set    |                           | parameter is read   |
|                    | 9h-80h-07h-00h            | by the READ command |
|                    |                           | following the SCAN  |
|                    |                           | command.            |
+--------------------+---------------------------+---------------------+
| If Set up Scanning | LOGICAL UNIT NOT READY,   | Terminates with the |
| is set, after the  | CAUSE NOT REPORTABLE      | CHECK CONDITION     |
| operation is       |                           | status. If the      |
| activated by the   | (The internal mechanical  | operation is        |
| SCAN command, when | error occurred.)          | terminated          |
| the completion of  |                           | normally, after the |
| reading and the    | 02h-04h-02h-00h           | operation           |
| device internal    |                           | completion is       |
| processing         |                           | confirmed, Max      |
| operation is       |                           | Value can be read   |
| confirmed by the   |                           | by the READ         |
| TEST UNIT READY    |                           | command.            |
| command, and the   |                           |                     |
| operation is not   |                           |                     |
| terminated         |                           |                     |
| normally           |                           |                     |
+--------------------+---------------------------+---------------------+
| After the SCAN     | LOGICAL UNIT IS IN        | Terminates with     |
| command is         | PROCESS OF BECOMING READY | GOOD status after   |
| terminated with    |                           | the preparation is  |
| GOOD status, until | (During the execution of  | completed even in   |
| the scan           | the operation activation  | the scanning        |
| preparation such   | command)                  | status.             |
| as the stage       |                           |                     |
| movement is        | 02h-04h-01h-00h           |                     |
| completed (for     |                           |                     |
| TEST UNIT READY)   | (During loading/ejection  |                     |
|                    | of the object to be       |                     |
|                    | scanned)                  |                     |
|                    |                           |                     |
|                    | 02h-04h-01h-01h           |                     |
|                    |                           |                     |
|                    | (During the measurement   |                     |
|                    | of the correction data)   |                     |
|                    |                           |                     |
|                    | 02h-04h-01h-02h           |                     |
|                    |                           |                     |
|                    | (During the execution of  |                     |
|                    | operation for loading the |                     |
|                    | object to be scanned)     |                     |
|                    |                           |                     |
|                    | 02h-04h-01h-03h           |                     |
|                    |                           |                     |
|                    | (During the execution of  |                     |
|                    | automatic shading or      |                     |
|                    | white balance             |                     |
|                    | measurement)              |                     |
|                    |                           |                     |
|                    | 02h-04h-01h-04h           |                     |
+--------------------+---------------------------+---------------------+

**2-8. SEND DIAGNOSTIC Command**

Table 2-8-1 SEND DIAGNOSTIC command

+-------:+:----------:+:----------:+:----------:+:----------:+:----------:+:----------:+:----------:+:----------:+
| Bit    | 7          | 6          | 5          | 4          | 3          | 2          | 1          | 0          |
|        |            |            |            |            |            |            |            |            |
| Byte   |            |            |            |            |            |            |            |            |
+--------+------------+------------+------------+------------+------------+------------+------------+------------+
| 0      | Operation code \[1Dh\]                                                                                |
+--------+--------------------------------------+------------+------------+------------+------------+------------+
| 1      | Logical unit number                  | PF         | Re-served  | Self       | DevOfL     | Unit       |
|        |                                      |            |            |            |            |            |
|        | \[0\]                                | \[0 or 1\] | \[0\]      | Test       | \[0\]      | OfL        |
|        |                                      |            |            |            |            |            |
|        |                                      |            |            | \[0 or 1\] |            | \[0\]      |
+--------+--------------------------------------+------------+------------+------------+------------+------------+
| 2      | Reserved \[0\]                                                                                        |
+--------+-------------------------------------------------------------------------------------------------------+
| 3, 4   | Parameter list length                                                                                 |
+--------+-------------------------------------------------------------------------------------------------------+
| 5      | Control byte \[0\]                                                                                    |
+--------+-------------------------------------------------------------------------------------------------------+

The SEND DIAGNOSTIC command performs the self-test for this unit itself.

For the self-test of this unit, 'Parameter exists' or 'Parameter does
not exist' can be selected.

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

+:-----------------------:+:------------------------:+:--------------:+
| Status                  | Sense data               | Remarks        |
+-------------------------+--------------------------+----------------+
| An error occurred in    | Logical Unit Not Ready,  |                |
| the operation           | Cause Not Reportable     |                |
| activation command.     |                          |                |
|                         | (The internal mechanical |                |
|                         | error occurred.)         |                |
|                         |                          |                |
|                         | 02h-04h-02h-00h          |                |
+-------------------------+--------------------------+----------------+

- If the parameter exists

1)  The page format (PF) bit is set to 1, the Self Test bit is set to 0,
    and the parameter list length is set to the transferred parameter
    length in bytes. This unit does not support this status.

    The SCSI device off-line (DevOfl) bit and the logical unit off-line
    (UnitOfl) bit must be set to 0 regardless of whether the parameter
    exists or not.

**\
2-9. SET WINDOW Command**

Table 2-9-1 SET WINDOW command

+----------:+:-----------:+:-----------:+:-----------:+:-----------:+:-----------:+:-----------:+:-----------:+:-----------:+:-----------:+
| Bit       | 7           | 6           | 5           | 4                         | 3           | 2           | 1           | 0           |
|           |             |             |             |                           |             |             |             |             |
| Byte      |             |             |             |                           |             |             |             |             |
+-----------+-------------+-------------+-------------+---------------------------+-------------+-------------+-------------+-------------+
| 0         | Operation code \[24h\]                                                                                                      |
+-----------+-------------------------------------------------------+---------------------------------------------------------------------+
| 1         | Logical unit number                                   | Reserved                                                            |
|           |                                                       |                                                                     |
|           | \[0\]                                                 | \[0\]                                                               |
+-----------+-------------------------------------------------------+---------------------------------------------------------------------+
| 2 to 5    | Reserved \[0\]                                                                                                              |
+-----------+-----------------------------------------------------------------------------------------------------------------------------+
|           | (MSB)                                                                                                                       |
+-----------+-----------------------------------------------------------------------------------------------------------------------------+
| 6 to 8    | Transfer length \[Recommended value: 58d\]                                                                                  |
+-----------+-----------------------------------------------------------------------------------------------------------------------------+
|           | (LSB)                                                                                                                       |
+-----------+-------------+---------------------------------------------------------------------------------------------------------------+
| 9         | Reserved    | Control byte \[0\]                                                                                            |
|           |             |                                                                                                               |
|           | \[0\]       |                                                                                                               |
+-----------+-------------+---------------------------------------------------------------------------------------------------------------+

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

+-------:+:----------:+:----------:+:----------:+:----------:+:----------:+:----------:+:----------:+:----------:+
| Bit    | 7          | 6          | 5          | 4          | 3          | 2          | 1          | 0          |
|        |            |            |            |            |            |            |            |            |
| Byte   |            |            |            |            |            |            |            |            |
+--------+------------+------------+------------+------------+------------+------------+------------+------------+
| 0 to 5 | Reserved                                                                                              |
+--------+-------------------------------------------------------------------------------------------------------+
| 6      | (MSB) Window descriptor length                                                                        |
+--------+-------------------------------------------------------------------------------------------------------+
| 7      | \[Recommended value: 50d\] (LSB)                                                                      |
+--------+-------------------------------------------------------------------------------------------------------+

- The window parameter data consists of a header followed by one or more
  window descriptors (refer to table 2-10-3). Each window descriptor
  specifies the location, size, and the scanning method of the window.

The window descriptor length specifies the length in bytes of a single
window descriptor.

**\
2-10. GET WINDOW Command**

Table 2-10-1 GET WINDOW command

+-------:+:-----------:+:-----------:+:-----------:+:-----------:+:-----------:+:-----------:+:-----------:+:-----------:+:-----------:+
| Bit    | 7           | 6           | 5                         | 4           | 3           | 2           | 1           | 0           |
|        |             |             |                           |             |             |             |             |             |
| Byte   |             |             |                           |             |             |             |             |             |
+--------+-------------+-------------+---------------------------+-------------+-------------+-------------+-------------+-------------+
| 0      | Operation code \[25h\]                                                                                                      |
+--------+-------------------------------------------------------+-------------------------------------------------------+-------------+
| 1      | Logical unit number                                   | Reserved                                              | Single      |
|        |                                                       |                                                       |             |
|        | \[0\]                                                 | \[0\]                                                 | \[0, 1\]    |
+--------+-------------------------------------------------------+-------------------------------------------------------+-------------+
| 2 to 4 | Reserved \[0\]                                                                                                              |
+--------+-----------------------------------------------------------------------------------------------------------------------------+
| 5      | Window identifier \[0, 1, 2, 3\]                                                                                            |
+--------+-----------------------------------------------------------------------------------------------------------------------------+
|        | (MSB)                                                                                                                       |
+--------+-----------------------------------------------------------------------------------------------------------------------------+
| 6 to 8 | Transfer length \[Recommended value: (50\*the number of windows+8)d\]                                                       |
+--------+-----------------------------------------------------------------------------------------------------------------------------+
|        | (LSB)                                                                                                                       |
+--------+-----------------------------------------+-----------------------------------------------------------------------------------+
| 9      | Reserved \[0\]                          | Control byte \[0\]                                                                |
+--------+-----------------------------------------+-----------------------------------------------------------------------------------+

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

+-------:+:----------:+:----------:+:----------:+:----------:+:----------:+:----------:+:----------:+:----------:+
| Bit    | 7          | 6          | 5          | 4          | 3          | 2          | 1          | 0          |
|        |            |            |            |            |            |            |            |            |
| Byte   |            |            |            |            |            |            |            |            |
+--------+------------+------------+------------+------------+------------+------------+------------+------------+
| 0      | (MSB) Window data length                                                                              |
+--------+-------------------------------------------------------------------------------------------------------+
| 1      | \[Recommended value: (50\*the number of windows+6)d\] (LSB)                                           |
+--------+-------------------------------------------------------------------------------------------------------+
| 2 to 5 | Reserved                                                                                              |
+--------+-------------------------------------------------------------------------------------------------------+
| 6      | (MSB) Window descriptor length                                                                        |
+--------+-------------------------------------------------------------------------------------------------------+
| 7      | \[Recommended value: 50d\] (LSB)                                                                      |
+--------+-------------------------------------------------------------------------------------------------------+

The window data length specifies the length in bytes of the data that is
transferred following it. The window data length does not include
itself. Even if the allocated length is not enough to return all the GET
WINDOW data, the window data length is not adjusted for sending the cut
data again.

The window descriptor length specifies the window descriptor length in
bytes for a single window.

Table 2-10-3 Window Descriptor Byte

+-------+-------------+-------------+-------------+-------------+-------------+-------------+-------------+-------------+
| Bit   | 7           | 6           | 5           | 4           | 3           | 2           | 1           | 0           |
|       |             |             |             |             |             |             |             |             |
| Byte  |             |             |             |             |             |             |             |             |
+:=====:+:===========:+:===========:+:===========:+:===========:+:===========:+:===========:+:===========:+:===========:+
| 0     | Window Identifier \[0, 1, 2, 3\] (The default is 2.)                                                          |
+-------+-------------------------------------------------------------------------------------------------+-------------+
| 1     | Reserved                                                                                        | Auto        |
|       |                                                                                                 |             |
|       | \[0\]                                                                                           | \[0\]       |
+-------+-------------------------------------------------------------------------------------------------+-------------+
| 2, 3  | > X Resolution \[90 to 4000\]                                                                                 |
+-------+---------------------------------------------------------------------------------------------------------------+
| 4, 5  | > Y Resolution \[90 to 4000\]                                                                                 |
+-------+---------------------------------------------------------------------------------------------------------------+
| 6 to  | > Upper Left X Offset (The default is 0.)                                                                     |
| 9     |                                                                                                               |
+-------+---------------------------------------------------------------------------------------------------------------+
| 10 to | > Upper Left Y Offset (The default is 0.)                                                                     |
| 13    |                                                                                                               |
+-------+---------------------------------------------------------------------------------------------------------------+
| 14 to | > Window Width                                                                                                |
| 17    |                                                                                                               |
+-------+---------------------------------------------------------------------------------------------------------------+
| 18 to | > Window Length                                                                                               |
| 21    |                                                                                                               |
+-------+---------------------------------------------------------------------------------------------------------------+
| 22    | > Brightness \[0\]                                                                                            |
+-------+---------------------------------------------------------------------------------------------------------------+
| 23    | > Threshold \[0\]                                                                                             |
+-------+---------------------------------------------------------------------------------------------------------------+
| 24    | > Contrast \[0\]                                                                                              |
+-------+---------------------------------------------------------------------------------------------------------------+
| 25    | > Image Composition \[2 or 5\] (The default is 2.)                                                            |
+-------+---------------------------------------------------------------------------------------------------------------+
| 26    | > Pixel Composition \[16d\]                                                                                   |
+-------+---------------------------------------------------------------------------------------------------------------+
| 27,   | > Halftone Pattern \[0\]                                                                                      |
| 28    |                                                                                                               |
+-------+-------------+-------------------------------------------------------+-----------------------------------------+
| 29    | Reverse     | Reserved                                              | Padding Type                            |
|       |             |                                                       |                                         |
|       | \[0\]       | \[0\]                                                 | \[0\]                                   |
+-------+-------------+-------------------------------------------------------+-----------------------------------------+
| 30,   | > Bit Ordering \[0\]                                                                                          |
| 31    |                                                                                                               |
+-------+---------------------------------------------------------------------------------------------------------------+
| 32    | > Compression Type \[0\]                                                                                      |
+-------+---------------------------------------------------------------------------------------------------------------+
| 33    | > Compression Argument \[0\]                                                                                  |
+-------+---------------------------------------------------------------------------------------------------------------+
| 34 to | > Reserved \[0\]                                                                                              |
| 39    |                                                                                                               |
+-------+-------------------------------------------------------+-------------------------------------------------------+
| 40    | Multiple Reading Number \[0 to 15\]                   | Color Ordering \[0, 1, 2, 3\]                         |
|       |                                                       |                                                       |
|       | (The default is 0.)                                   | (The default is R=1, G=2, B=3)                        |
+-------+-------------+-------------+-------------+-------------+-----------------------------------------+-------------+
| 41    | Averag-ing  | Matrix      | Filter      | Reserved    | Setup Mode                              | Object      |
|       |             |             |             |             |                                         |             |
|       | 1: ON       | \[0\]       | \[0\]       | \[0\]       |                                         | 1: Posi\*   |
|       |             |             |             |             |                                         |             |
|       | 0: OFF\*    |             |             |             |                                         | 0: Nega     |
+-------+-------------+-------------+-------------+-------------+-----------------------------------------+-------------+
| 42    | > Scanning Kind (The default is 1.)                                                                           |
+-------+---------------------------------------------------------------------------------------------------------------+
| 43    | > Scanning Mode (The default is 2.)                                                                           |
+-------+---------------------------------------------------------------------------------------------------------------+
| 44    | > Color interleaving (The default is 2.)                                                                      |
+-------+---------------------------------------------------------------------------------------------------------------+
| 45    | > AE Value (The default is 255d.)                                                                             |
+-------+---------------------------------------------------------------------------------------------------------------+
| 46 to | > Exposure Value \[0 to 3FFFFFFh\]                                                                            |
| 49    |                                                                                                               |
+-------+---------------------------------------------------------------------------------------------------------------+

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

  ------------------------ ----------------------- -----------------------
     Window identifier         Scanning color       Support of this unit

             0                Default color (G)              Yes

             1                        R                      Yes

             2                        G                      Yes

             3                        B                      Yes

             4                  Neutral gray                 No
  ------------------------ ----------------------- -----------------------

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

+:-----------------------:+:---------------:+:------:+:------:+:------:+
| Set Window              | Scanning at the device                     |
| specification           |                                            |
+-------------------------+--------------------------+-----------------+
| X resolution            | Scanning resolution      | Pitch           |
+-------------------------+-----------------+--------+--------+--------+
| 4000 to 2001            | 4000            | 1               |        |
+-------------------------+-----------------+-----------------+--------+
| 2000 to 1001            | 2000            | 2               |        |
+-------------------------+-----------------+-----------------+--------+
| 1000 to 667             | 1000            | 4               |        |
+-------------------------+-----------------+-----------------+--------+
| 666 to 501              | 666             | 6               |        |
+-------------------------+-----------------+-----------------+--------+
| 500 to 401              | 500             | 8               |        |
+-------------------------+-----------------+-----------------+--------+
| 400 to 334              | 400             | 10              |        |
+-------------------------+-----------------+-----------------+--------+
| 333 to 286              | 333             | 12              |        |
+-------------------------+-----------------+-----------------+--------+
| 285 to 251              | 285             | 14              |        |
+-------------------------+-----------------+-----------------+--------+
| 250 to 223              | 250             | 16              |        |
+-------------------------+-----------------+-----------------+--------+
| 222 to 201              | 222             | 18              |        |
+-------------------------+-----------------+-----------------+--------+
| 200 to 182              | 200             | 20              |        |
+-------------------------+-----------------+-----------------+--------+
| 181 to 167              | 181             | 22              |        |
+-------------------------+-----------------+-----------------+--------+
| 166 to 154              | 166             | 24              |        |
+-------------------------+-----------------+-----------------+--------+
| 153 to 143              | 153             | 26              |        |
+-------------------------+-----------------+-----------------+--------+
| 142 to 134              | 142             | 28              |        |
+-------------------------+-----------------+-----------------+--------+
| 133 to 126              | 133             | 30              |        |
+-------------------------+-----------------+-----------------+--------+
| 125 to 118              | 125             | 32              |        |
+-------------------------+-----------------+-----------------+--------+
| 117 to 112              | 117             | 34              |        |
+-------------------------+-----------------+-----------------+--------+
| 111 to 106              | 111             | 36              |        |
+-------------------------+-----------------+-----------------+--------+
| 105 to 101              | 105             | 38              |        |
+-------------------------+-----------------+-----------------+--------+
| 100 to 96               | 100             | 40              |        |
+-------------------------+-----------------+-----------------+--------+
| 95 to 91                | 95              | 42              |        |
+-------------------------+-----------------+-----------------+--------+
| 90                      | 90              | 44              |        |
+-------------------------+-----------------+-----------------+--------+

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

+:------------:+:---------------------------------------:+:----------:+
| Code         | Descriptions                            | Support    |
+--------------+-----------------------------------------+------------+
| 00h          | Bi-level black & white                  | No         |
|              |                                         |            |
| 01h          | Dithered/halftone black & white         | No         |
|              |                                         |            |
| 02h          | Multi-level black & white               | Yes        |
|              |                                         |            |
| 03h          | Bi-level RGB color                      | No         |
|              |                                         |            |
| 04h          | Dithered/halftone RGB color             | No         |
|              |                                         |            |
| 05h          | Multi-level RGB color                   | Yes        |
|              |                                         |            |
| 06h to FFh   | Reserved                                |            |
+--------------+-----------------------------------------+------------+

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

+:----------------------------:+:--------------------:+:--------------:+
| Status                       | Sense data           | Remarks        |
+------------------------------+----------------------+----------------+
| In the case that the         | INVALID COMBINATION  | The command is |
| overlapped setting of a      | OF WINDOWS SPECIFIED | terminated     |
| value other than 0 is        |                      | with the CHECK |
| performed in this field for  | 05h-2Ch-02h-00h      | CONDITION      |
| two or more windows that are |                      | status.        |
| set when the SCAN command is |                      |                |
| received, and that 0 and a   |                      |                |
| value other than 0 are set   |                      |                |
| in this field                |                      |                |
+------------------------------+----------------------+----------------+

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
> status and sets 'Thumbnail created by driver' in the sense data. The
> initiator can read the parameter (Cooperation parameter) for creating
> the thumbnail by the READ command. This unit executes reading at the
> second SCAN command.

  ------------------------- ---------- -----------------------------------
  Scanning Kind                        

                                  Bit0                      Image Scanning

                                  Bit1                  Thumbnail Scanning

                                  Bit2                     Set up Scanning

                                  Bit3                    Set up Scanning2

                                  Bit4                      Reserved \[0\]

                                  Bit5              Auto Exposure Scanning

                                  Bit6                 AE with WB Scanning

                                  Bit7                      Reserved \[0\]
  ------------------------- ---------- -----------------------------------

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

  ------------------------- ---------- -----------------------------------
  Scan Mode Support                    

                                  Bit0                   High Quality Scan

                                  Bit1                 Normal Quality Scan

                                  Bit2                     High Speed Scan

                                  Bit3                      Reserved \[0\]

                                  Bit4               Multiple Reading Scan

                                  Bit5                      Reserved \[0\]

                                  Bit6          Reverse direction Scanning

                                  Bit7                      Reserved \[0\]
  ------------------------- ---------- -----------------------------------

Byte 44

> This field specifies which ordering (pixel ordering, line ordering, or
> plane ordering) shall be used for reading. It also specifies whether
> the X and Y offsets include the CCD distance for the pixel ordering
> and the line ordering. A bit whose ordering is specified for reading
> is set to 1.
>
> This field specifies the currently set value for the GET WINDOW
> command.

  --------------- -------------------------------------------------------
  Bit0            Pixel without CCD distance

  Bit1            Line without CCD distance

  Bit2            Plane

  Bit3            Reserved \[0\]

  Bit4            Pixel with CCD distance

  Bit5            Line with CCD distance

  Bit6            Multi line Simultaneous reading

  Bit7            Reserved \[0\]
  --------------- -------------------------------------------------------

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

+-------:+:----------:+:----------:+:----------:+:----------:+:----------:+:----------:+:----------:+:----------:+
| Bit    | 7          | 6          | 5          | 4          | 3          | 2          | 1          | 0          |
|        |            |            |            |            |            |            |            |            |
| Byte   |            |            |            |            |            |            |            |            |
+--------+------------+------------+------------+------------+------------+------------+------------+------------+
| 0      | Operation code \[28h\]                                                                                |
+--------+--------------------------------------+----------------------------------------------------------------+
| 1      | Logical unit number                  | Reserved                                                       |
|        |                                      |                                                                |
|        | \[0\]                                | \[0\]                                                          |
+--------+--------------------------------------+----------------------------------------------------------------+
| 2      | Data type code                                                                                        |
+--------+-------------------------------------------------------------------------------------------------------+
| 3      | Reserved \[0\]                                                                                        |
+--------+-------------------------------------------------------------------------------------------------------+
| 4      | Data type qualifier (upper byte)                                                                      |
+--------+-------------------------------------------------------------------------------------------------------+
| 5      | Data type qualifier (lower byte)                                                                      |
+--------+-------------------------------------------------------------------------------------------------------+
| 6      | (MSB)                                                                                                 |
+--------+-------------------------------------------------------------------------------------------------------+
| 7      | Transfer length                                                                                       |
+--------+-------------------------------------------------------------------------------------------------------+
| 8      | (LSB)                                                                                                 |
+--------+------------+------------------------------------------------------------------------------------------+
| 9      | Reserved   | Control bit \[0\]                                                                        |
|        |            |                                                                                          |
|        | \[0\]      |                                                                                          |
+--------+------------+------------------------------------------------------------------------------------------+

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

+-------------------+--------------------------------------------------+
| Data type code    | Recommended value (Refer to table 2-11-2.)       |
+-------------------+--------------------------------------------------+
| 00h to 7Fh        | Data length in bytes \* the number of valid data |
+-------------------+--------------------------------------------------+
| 80h and after     | Data length in bytes \* the number of valid      |
|                   | data + header length in bytes                    |
|                   |                                                  |
|                   | (In the case of the magnetic data, the magnetic  |
|                   | data header is included.)                        |
+-------------------+--------------------------------------------------+

Table 2-11-2 Data type code (common to READ/SEND)

+:-------:+:-------------------:+:-----------:+:-------:+:----------:+:--------:+
| Code    | Descriptions        | Support by  | Length  | Number of  | Header   |
|         |                     | this        | in      | valid data | included |
|         |                     | system^\*1^ | bytes   |            | or not   |
|         |                     |             | of each | (Number of |          |
|         |                     |             | valid   | elements)  |          |
|         |                     |             | data    |            |          |
+---------+---------------------+-------------+---------+------------+----------+
| 00h     | Image               | R           | 1 or 2  | Variable   | Not      |
|         |                     |             |         |            | included |
+---------+---------------------+-------------+---------+------------+----------+
| 02h     | Halftone mask       | \-          | \-      | \-         | \-       |
+---------+---------------------+-------------+---------+------------+----------+
| 03h     | LUT                 | \-          | \-      | \-         | \-       |
+---------+---------------------+-------------+---------+------------+----------+
| 80h     | Histogram data      | \-          | \-      | \-         | \-       |
+---------+---------------------+-------------+---------+------------+----------+
| 81h     | Maximum value       | \-          | 2       | 1          | Included |
+---------+---------------------+-------------+---------+------------+----------+
| 82h     | Matrix data         | \-          | \-      | \-         | \-       |
+---------+---------------------+-------------+---------+------------+----------+
| 83h     | Filter data         | \-          | \-      | \-         | \-       |
+---------+---------------------+-------------+---------+------------+----------+
| 84h     | Shading data        | R/S         | 2       | 47352      | Included |
+---------+---------------------+-------------+---------+------------+----------+
| 85h     | Dark voltage        | \-          | \-      | \-         | \-       |
|         | correction data     |             |         |            |          |
+---------+---------------------+-------------+---------+------------+----------+
| 86h     | Magnetic data       | \-          | \-      | \-         | \-       |
+---------+---------------------+-------------+---------+------------+----------+
| 87h     | Initiator           | R           | 1       | Variable   | Included |
|         | cooperative action  |             |         |            |          |
|         | parameter           |             |         |            |          |
+---------+---------------------+-------------+---------+------------+----------+
| 88h     | Boundary            | \-          | 4       | Variable   | Included |
|         | Information         |             |         |            |          |
+---------+---------------------+-------------+---------+------------+----------+
| 89h     | Analog gamma        | \-          | \-      | \-         | \-       |
+---------+---------------------+-------------+---------+------------+----------+
| 8Ah     | Analog gain         | \-          | \-      | \-         | \-       |
+---------+---------------------+-------------+---------+------------+----------+
| 8Bh     | Digital gain        | \-          | \-      | \-         | \-       |
+---------+---------------------+-------------+---------+------------+----------+
| 8Ch     | WB exposure value   | R           | 4       | 1          | Included |
+---------+---------------------+-------------+---------+------------+----------+
| 8Dh     | Setup information   | R           | 1, 2,   | Variable   | Included |
|         |                     |             | or 4    |            |          |
+---------+---------------------+-------------+---------+------------+----------+
| 8Eh     | Perforation         | R           | 1 or 2  | Variable   | Included |
|         | information         |             |         |            |          |
+---------+---------------------+-------------+---------+------------+----------+
| 8Fh     | Boundary            | R/S         | 1, 2,   | Variable   | Included |
|         | Information Type2   |             | or 4    |            |          |
+---------+---------------------+-------------+---------+------------+----------+
| 90h     | WB exposure value   | \-          | \-      | \-         | \-       |
|         | at the time of      |             |         |            |          |
|         | shipment            |             |         |            |          |
+---------+---------------------+-------------+---------+------------+----------+
| 91h     | CCD data            | R           | 2       | Variable   | Included |
+---------+---------------------+-------------+---------+------------+----------+
| 92h     | Driver software     | \-          | \-      | \-         | \-       |
|         | version information |             |         |            |          |
+---------+---------------------+-------------+---------+------------+----------+
| 93h     | Leak volume         | R           | 2       | 3          | Included |
+---------+---------------------+-------------+---------+------------+----------+
| 94h-DFh | Reserved            | \-          | \-      | \-         | \-       |
+---------+---------------------+-------------+---------+------------+----------+
| E0h     | Initiator RAM       | R/S         | 1, 2,   | Variable   | Included |
|         | buffer              |             | or 4    | (max 1 KB) |          |
+---------+---------------------+-------------+---------+------------+----------+
| E1h     | Initiator EEPROM    | \-          | \-      | \-         | \-       |
|         | buffer              |             |         |            |          |
+---------+---------------------+-------------+---------+------------+----------+

> \*1 R means that the code is supported only for the READ command. R/S
> means that the code is supported for both the READ and the SEND
> commands.
>
> \*2 The valid number of pixels for CCD is 3946d.

Table 2-11-3 Data type qualifier (upper byte)

+--------------------+--------+----------------------------------------+
|                    | Code   | Descriptions                           |
+--------------------+--------+----------------------------------------+
| When the data type | 00h    | Default color (G-component element)    |
| code is 03h, 80h,  |        |                                        |
| 81h, 84h, 85h,     | 01h    | R-component element                    |
| 8Ch, 8Dh, or 91h   |        |                                        |
|                    | 02h    | G-component element                    |
|                    |        |                                        |
|                    | 03h    | B-component element                    |
+--------------------+--------+----------------------------------------+
| Case other than    | \*\*h  | No meaning                             |
| the above          |        |                                        |
+--------------------+--------+----------------------------------------+

Table 2-11-4 Data type qualifier (lower byte)

+-------------------+--------------------------------------------------+
| Code              | Descriptions                                     |
+-------------------+--------------------------------------------------+
| 00h               | 1-byte data                                      |
|                   |                                                  |
| 01h               | 2-byte data                                      |
|                   |                                                  |
| 02h               | Reserved                                         |
|                   |                                                  |
| 03h               | 4-byte data                                      |
|                   |                                                  |
| 04h and after     | Reserved                                         |
+-------------------+--------------------------------------------------+

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

+:------------------------:+:-----------------------:+:---------------:+
| Status                   | Sense data              | Remarks         |
+--------------------------+-------------------------+-----------------+
| - When the READ command  | COMMAND SEQUENCE ERROR  | The command     |
|   of the image is        |                         | terminates with |
|   received without       | (A command that makes   | the CHECK       |
|   receiving the SCAN     | the previous SCAN       | CONDITION       |
|   command                | command invalid is      | status.         |
|                          | received while the      |                 |
| - When the READ command  | scanning operation is   |                 |
|   is received after all  | valid)                  |                 |
|   image data is          |                         |                 |
|   transferred            | 05h-2Ch-00h-00h         |                 |
+--------------------------+-------------------------+-----------------+

**2-11-1. 2-byte data transfer**

For the 2-byte data, upper byte and lower byte are transferred, in that
order. For the data of three bytes or more, the upper byte, the middle
byte, and the lower byte are transferred, in that order.

**2-11-2. Data header**

If the data type code is over 80h, the following READ data header is
added at the top of the valid data.

Table 2-11-6 READ data header

+-------:+:----------:+:----------:+:----------:+:----------:+:----------:+:----------:+:----------:+:----------:+
| Bit    | 7          | 6          | 5          | 4          | 3          | 2          | 1          | 0          |
|        |            |            |            |            |            |            |            |            |
| Byte   |            |            |            |            |            |            |            |            |
+--------+------------+------------+------------+------------+------------+------------+------------+------------+
| 0      | Data type code                                                                                        |
+--------+-------------------------------------------------------------------------------------------------------+
| 1      | The number of bits in each valid data                                                                 |
+--------+-------------------------------------------------------------------------------------------------------+
|        | (MSB)                                                                                                 |
+--------+-------------------------------------------------------------------------------------------------------+
| 2 to 5 | Valid data length                                                                                     |
+--------+-------------------------------------------------------------------------------------------------------+
|        | (LSB)                                                                                                 |
+--------+-------------------------------------------------------------------------------------------------------+

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

+:------------------:+:------------------------------:+:--------------:+
| Status             | Sense data                     | Remarks        |
+--------------------+--------------------------------+----------------+
| When the READ      | COMMAND SEQUENCE ERROR         | The command    |
| command is         |                                | terminates     |
| received after all | (A command that makes the      | with the CHECK |
| the image data is  | previous SCAN command invalid  | CONDITION      |
| transferred        | is received while the scanning | status.        |
|                    | operation is valid)            |                |
|                    |                                |                |
|                    | 05h-2Ch-00h-00h                |                |
+--------------------+--------------------------------+----------------+

**Precautions:**

As shown in Byte 4 "SCSI function support" of 2-2-2-3. "Address
information page", the image reading is performed in units of \[Data of
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

+-------:+:----------:+:----------:+:----------:+:----------:+:----------:+:----------:+:----------:+:----------:+
| Bit    | 7          | 6          | 5          | 4          | 3          | 2          | 1          | 0          |
|        |            |            |            |            |            |            |            |            |
| Byte   |            |            |            |            |            |            |            |            |
+--------+------------+------------+------------+------------+------------+------------+------------+------------+
| 0      | (MSB) Data for the first pixel in gain 1, 2-line mode, CCD line A                                     |
+--------+-------------------------------------------------------------------------------------------------------+
| 1      | (LSB)                                                                                                 |
+--------+-------------------------------------------------------------------------------------------------------+
| 2      | (MSB) Data for the first pixel in gain 1, 2-line mode, CCD line B                                     |
+--------+-------------------------------------------------------------------------------------------------------+
| 3      | (LSB)                                                                                                 |
+--------+-------------------------------------------------------------------------------------------------------+
| :      | :                                                                                                     |
+--------+-------------------------------------------------------------------------------------------------------+
| 15782  | (MSB) Data for the 3946th pixel in gain 1, 2-line mode, CCD line B                                    |
+--------+-------------------------------------------------------------------------------------------------------+
| 15783  | (LSB)                                                                                                 |
+--------+-------------------------------------------------------------------------------------------------------+
| 15784  | (MSB) Data for the first pixel in gain 1, 1-line mode, CCD line A                                     |
+--------+-------------------------------------------------------------------------------------------------------+
| 15785  | (LSB)                                                                                                 |
+--------+-------------------------------------------------------------------------------------------------------+
| :      | :                                                                                                     |
+--------+-------------------------------------------------------------------------------------------------------+
| 23674  | (MSB) Data for the 3946th pixel in gain 1, 1-line mode, CCD line A                                    |
+--------+-------------------------------------------------------------------------------------------------------+
| 23675  | (LSB)                                                                                                 |
+--------+-------------------------------------------------------------------------------------------------------+
| 23676  | (MSB) Data for the first pixel in gain 2, 2-line mode, CCD line A                                     |
+--------+-------------------------------------------------------------------------------------------------------+
| 23677  | (LSB)                                                                                                 |
+--------+-------------------------------------------------------------------------------------------------------+
| :      | :                                                                                                     |
+--------+-------------------------------------------------------------------------------------------------------+
| 39458  | (MSB) Data for the 3946th pixel in gain 2, 2-line mode, CCD line B                                    |
+--------+-------------------------------------------------------------------------------------------------------+
| 39459  | (LSB)                                                                                                 |
+--------+-------------------------------------------------------------------------------------------------------+
| 39460  | (MSB) Data for the first pixel in gain 2, 1-line mode, CCD line A                                     |
+--------+-------------------------------------------------------------------------------------------------------+
| 39461  | (LSB)                                                                                                 |
+--------+-------------------------------------------------------------------------------------------------------+
| :      | :                                                                                                     |
+--------+-------------------------------------------------------------------------------------------------------+
| 47350  | (MSB) Data for the 3946th pixel in gain 2, 1-line mode, CCD line A                                    |
+--------+-------------------------------------------------------------------------------------------------------+
| 47351  | (LSB)                                                                                                 |
+--------+-------------------------------------------------------------------------------------------------------+

Note) For the shading data measured with the 240 adapter attached, the
shading data corresponding to the range outside the aperture becomes
invalid, but this invalid data is also read at the same time in the
reading by the READ command.

**\
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

  ------ ------------------------------- --------------------------------
    1    THUMBNAIL CREATED BY DRIVER     Thumbnail scanning

    2    AVERAGING MULTIPLE READING BY   Line averaging of the multiple
         DRIVER                          reading function

    6    TRUNCATED BY DRIVER             Deletion of invalid data

    7    CCD DATA CREATED BY DRIVER      CCD data reading
  ------ ------------------------------- --------------------------------

Table 2-11-5-1 Format of THUMBNAIL CREATED BY DRIVER

+:----:+:-------------:+:-----------------:+---------------------------+
| Byte | Name          | Descriptions      | Parameter                 |
+------+---------------+-------------------+---------------------------+
| 0    | Type Code     | Operation type    | 1                         |
|      |               | code              |                           |
+------+---------------+-------------------+---------------------------+
| 1 to | Sense Data    | Sense data that   | 09h-80h-01h-02h (IA)      |
| 4    |               | is set by the     |                           |
|      |               | SCAN command      | 09h-80h-01h-06h (SA)      |
+------+---------------+-------------------+---------------------------+
| 5, 6 | Bytes Per     | The number of     | Depends on the scanning   |
|      | Line          | bytes per line    | condition                 |
+------+---------------+-------------------+---------------------------+
| 7, 8 | Entire Lines  | The number of     | Number of scanning        |
|      |               | entire lines      | lines\*Number of frames   |
+------+---------------+-------------------+---------------------------+
| 9    | Bits Per a    | The number of     | \[16\]                    |
|      | Color of Dot  | bits per dot of   |                           |
|      |               | one color         |                           |
+------+---------------+-------------------+---------------------------+
| 10,  | Lines Per an  | The number of     | The number of scanning    |
| 11   | Image         | lines per image   | lines                     |
+------+---------------+-------------------+---------------------------+
| 12   | Reading Count | Exposure counts   | \-                        |
|      | Per a Line    | per line          |                           |
+------+---------------+-------------------+---------------------------+
| 13   | Reserved      | Reserved          | 0                         |
| to   |               |                   |                           |
| 17   |               |                   |                           |
+------+---------------+-------------------+---------------------------+

Table 2-11-5-2 Format of AVERAGING MULTIPLE READING BY DRIVER

  ------ ---------------- -------------------- ---------------------------
   Byte        Name           Descriptions     Parameter

    0       Type Code     Operation type code  2

  1 to 4    Sense Data     Sense data that is  09h-80h-02h-00h
                            set by the SCAN    
                                command        

   5, 6   Bytes Per Line  The number of bytes  \-
                                per line       

   7, 8    Entire Lines   The number of entire \-
                                 lines         

    9    Bits Per a Color  The number of bits  \-
              of Dot      per dot of one color 

  10, 11   Lines Per an   The number of lines  \-
              Image            per image       

    12    Reading Count   Exposure counts per  Depends on the scanning
            Per a Line            line         condition

  13 to      Reserved           Reserved       0
    17                                         
  ------ ---------------- -------------------- ---------------------------

Table 2-11-5-3 Format of TRUNCATED BY DRIVER TYPE2

+:----:+:-------------:+:-----------------:+---------------------------+
| Byte | Name          | Descriptions      | Parameter                 |
+------+---------------+-------------------+---------------------------+
| 0    | Type Code     | Operation type    | 6                         |
|      |               | code (06h)        |                           |
+------+---------------+-------------------+---------------------------+
| 1 to | Sense Data    | Sense data that   | 09h-80h-06h-01h           |
| 4    |               | is set by the     |                           |
|      |               | SCAN command      |                           |
|      |               |                   |                           |
|      |               | (9h-80h-06h-01h)  |                           |
+------+---------------+-------------------+---------------------------+
| 5, 6 | Invalid Data  | Invalid data      | Depends on the scanning   |
|      | Position      | attaching         | condition                 |
|      |               | position          |                           |
+------+---------------+-------------------+---------------------------+
| 7, 8 | Byte of       | Invalid data      | Depends on the scanning   |
|      | invalid data  | length in bytes   | condition                 |
|      | of Left of    | that is attached  |                           |
|      | each color    | to the            |                           |
|      |               | first-pixel side  |                           |
|      |               | in the scan line  |                           |
|      |               | direction with    |                           |
|      |               | the origin of the |                           |
|      |               | image in each     |                           |
|      |               | color set to the  |                           |
|      |               | standard          |                           |
+------+---------------+-------------------+---------------------------+
| 9,   | Byte of       | Invalid data      | Depends on the scanning   |
| 10   | invalid data  | length in bytes   | condition                 |
|      | of Last of    | that is attached  |                           |
|      | each color    | to the last-pixel |                           |
|      |               | side in the scan  |                           |
|      |               | line direction    |                           |
|      |               | with the origin   |                           |
|      |               | of the image in   |                           |
|      |               | each color set to |                           |
|      |               | the standard      |                           |
+------+---------------+-------------------+---------------------------+
| 11,  | Byte of       | Invalid data      | Depends on the scanning   |
| 12   | invalid data  | length in bytes   | condition                 |
|      | of Left of    | that is attached  |                           |
|      | all color     | to the            |                           |
|      |               | first-pixel side  |                           |
|      |               | in the scan line  |                           |
|      |               | direction with    |                           |
|      |               | the origin of the |                           |
|      |               | image in all      |                           |
|      |               | colors set to the |                           |
|      |               | standard          |                           |
+------+---------------+-------------------+---------------------------+
| 13,  | Byte of       | Invalid data      | Depends on the scanning   |
| 14   | invalid data  | length in bytes   | condition                 |
|      | of Last of    | that is attached  |                           |
|      | all color     | to the last-pixel |                           |
|      |               | side in the scan  |                           |
|      |               | line direction    |                           |
|      |               | with the origin   |                           |
|      |               | of the image in   |                           |
|      |               | all colors set to |                           |
|      |               | the standard      |                           |
+------+---------------+-------------------+---------------------------+
| 15,  | Reserved      | \-                | \-                        |
| 16   |               |                   |                           |
+------+---------------+-------------------+---------------------------+
| 17,  | Reserved      | \-                | \-                        |
| 18   |               |                   |                           |
+------+---------------+-------------------+---------------------------+
| 19,  | Line of       | The number of     | Depends on the scanning   |
| 20   | invalid data  | invalid data      | condition                 |
|      | of Top        | lines that is     |                           |
|      |               | attached to the   |                           |
|      |               | first-line side   |                           |
|      |               | in the base line  |                           |
|      |               | direction with    |                           |
|      |               | the origin of the |                           |
|      |               | image set to the  |                           |
|      |               | standard          |                           |
+------+---------------+-------------------+---------------------------+
| 21,  | Line of       | The number of     | Depends on the scanning   |
| 22   | invalid data  | invalid data      | condition                 |
|      | of End        | lines that is     |                           |
|      |               | attached to the   |                           |
|      |               | last-line side in |                           |
|      |               | the base line     |                           |
|      |               | direction with    |                           |
|      |               | the origin of the |                           |
|      |               | image set to the  |                           |
|      |               | standard          |                           |
+------+---------------+-------------------+---------------------------+
| 23,  | Byte of       | Invalid data      | Depends on the scanning   |
| 24   | invalid data  | length in bytes   | condition                 |
|      | of Top of one | that is attached  |                           |
|      | frame         | to the            |                           |
|      |               | first-pixel side  |                           |
|      |               | in the scan line  |                           |
|      |               | direction with    |                           |
|      |               | the origin of the |                           |
|      |               | one-frame image   |                           |
|      |               | data set to the   |                           |
|      |               | standard          |                           |
+------+---------------+-------------------+---------------------------+
| 25,  | Byte of       | Invalid data      | Depends on the scanning   |
| 26   | invalid data  | length in bytes   | condition                 |
|      | of End of one | that is attached  |                           |
|      | frame         | to the last-pixel |                           |
|      |               | side in the scan  |                           |
|      |               | line direction    |                           |
|      |               | with the origin   |                           |
|      |               | of the one-frame  |                           |
|      |               | image data set to |                           |
|      |               | the standard      |                           |
+------+---------------+-------------------+---------------------------+

Byte 5 and 6 Invalid Data Position

> This field specifies the position to which the invalid data is
> attached. The invalid data is attached to the position of the bit to
> which 1 is set.

  --------- --------- ----------------------------------------------------
  Byte 5    Bit0      The first-pixel side in the scan line direction with
                      theorigin of data in each color set to the standard

            Bit1      The last-pixel side in the scan line direction with
                      theorigin of data in each color set to the standard

            Bit2      The first-pixel side in the scan line direction with
                      theorigin of data in all colors set to the standard

            Bit3      The last-pixel side in the scan line direction with
                      theorigin of data in all colors set to the standard

            Bit4      Reserved

            Bit5      Reserved

            Bit6      The first-line side in the base line direction with
                      the origin set to the standard

            Bit7      The last-line side in the base line direction with
                      the origin set to the standard
  --------- --------- ----------------------------------------------------

  --------- --------- ----------------------------------------------------
  Byte 6    Bit0      The first-pixel side in the scan line direction with
                      theorigin of one-frame image data set to the
                      standard

            Bit1      The last-pixel side in the scan line direction with
                      theorigin of one-frame image data set to the
                      standard

            Bit2      Reserved

            Bit3      Reserved

            Bit4      Reserved

            Bit5      Reserved

            Bit6      Reserved

            Bit7      Reserved
  --------- --------- ----------------------------------------------------

Byte 7 to 26 Byte of invalid data

> This field specifies the invalid data length in bytes that is attached
> to each position.

Some of the positions may be included in both the scan-line side and the
base-line side depending on the condition. In this case, it is handled
as a part of the line in the base line direction.

Table 2-11-5-4 Format of CCD DATA CREATED BY DRIVER

  ------ ---------------- --------------------- --------------------------
   Byte        Name           Descriptions      Parameter

    0       Type Code      Operation type code  7

  1 to 4    Sense Data     Sense data that is   09h-80h-07h-00h
                             set by the SCAN    
                                 command        

    5    CCD Data Type of     Type for CCD      Depends on the scanning
              R Data        measurement of R    condition
                                  color         

    6    CCD Data Type of     Type for CCD      Depends on the scanning
              G Data        measurement of G    condition
                                  color         

    7    CCD Data Type of     Type for CCD      Depends on the scanning
              B Data        measurement of B    condition
                                  color         

    8    CCD Data Type of     Type for CCD      \-
             NG Data        measurement of NG   
                                  color         

    9    CCD Data Type of     Type for CCD      \-
              C Data        measurement of C    
                                  color         

    10   CCD Data Type of     Type for CCD      \-
              M Data        measurement of M    
                                  color         

    11   CCD Data Type of     Type for CCD      \-
              Y Data        measurement of Y    
                                  color         

    12   CCD Data Type of     Type for CCD      \-
              B Data        measurement of B    
                                  color         

  13 to      Reserved           Reserved        0
    17                                          
  ------ ---------------- --------------------- --------------------------

Byte 5 to 12 CCD Data Type of color Data

> This field specifies the type that is used for the CCD measurement of
> each color.

**2-11-6. WB exposure value**

The value decided by the measurement of the unit at the time of start-up
specifies the color according to the upper byte of the data type
qualifier, and 4-byte data is sent for each color.

**\
2-11-7. Setup information**

+-------:+:----------:+:----------:+:----------:+:----------:+:----------:+:----------:+:----------:+:----------:+
| Bit    | 7          | 6          | 5          | 4          | 3          | 2          | 1          | 0          |
|        |            |            |            |            |            |            |            |            |
| Byte   |            |            |            |            |            |            |            |            |
+--------+------------+------------+------------+------------+------------+------------+------------+------------+
| 0, 1   | (MSB) Parameter length \[n-2\] (LSB)                                                                  |
+--------+-------------------------------------------------------------------------------------------------------+
| 2      | Format Identifier \[0\]                                                                               |
+--------+-------------------------------------------------------------------------------------------------------+
| 3, 4   | Base Level (Base level value of the film)                                                             |
+--------+-------------------------------------------------------------------------------------------------------+
| 5 to 8 | Exposure Value for Base Level                                                                         |
|        |                                                                                                       |
|        | (Exposure value when the base level value of the film is decided)                                     |
+--------+-------------------------------------------------------------------------------------------------------+
| 9 to   | Exposure Value for White balance at base measurement                                                  |
| 12     |                                                                                                       |
|        | (Exposure value for white balance when the base level value of the film is decided)                   |
+--------+-------------------------------------------------------------------------------------------------------+
| 13     | The number of information retaining images                                                            |
+--------+-------------------------------------------------------------------------------------------------------+
| 14     | 1st Index (The first image)                                                                           |
+--------+-------------------------------------------------------------------------------------------------------+
| 15 to  | Exposure Value for 1st index image                                                                    |
| 18     |                                                                                                       |
|        | (Exposure value after prescan of the first image)                                                     |
+--------+-------------------------------------------------------------------------------------------------------+
| 19 to  | Exposure Value for White balance at 1st image setup                                                   |
| 22     |                                                                                                       |
|        | (Exposure value for white balance in the prescan of the first image)                                  |
+--------+-------------------------------------------------------------------------------------------------------+
| 23, 24 | Minimum Level for the 1st index image                                                                 |
|        |                                                                                                       |
|        | (Minimum level of the image detected after prescan of the first image)                                |
+--------+-------------------------------------------------------------------------------------------------------+
| 25, 26 | Maximum Level for the 1st index image                                                                 |
|        |                                                                                                       |
|        | (Maximum level of the image detected after prescan of the first image)                                |
+--------+-------------------------------------------------------------------------------------------------------+
| 27     | 2nd Index (The second image)                                                                          |
+--------+-------------------------------------------------------------------------------------------------------+
| 28 to  | Exposure Value for 2nd index image                                                                    |
| 31     |                                                                                                       |
|        | (Exposure value after prescan of the 2nd image)                                                       |
+--------+-------------------------------------------------------------------------------------------------------+
| 32 to  | Exposure Value for White balance at 2nd image setup                                                   |
| 35     |                                                                                                       |
|        | (Exposure value for white balance in the prescan of the 2nd image)                                    |
+--------+-------------------------------------------------------------------------------------------------------+
| 36, 37 | Minimum Level for the 2nd index image                                                                 |
|        |                                                                                                       |
|        | (Minimum level of the image detected after prescan of the 2nd image)                                  |
+--------+-------------------------------------------------------------------------------------------------------+
| 38, 39 | Maximum Level for the 2nd index image                                                                 |
|        |                                                                                                       |
|        | (Maximum level of the image detected after prescan of the 2nd image)                                  |
+--------+-------------------------------------------------------------------------------------------------------+
| :      | :                                                                                                     |
+--------+-------------------------------------------------------------------------------------------------------+
| n-3,   | Minimum Level for the last index image                                                                |
| n-2    |                                                                                                       |
|        | (Minimum level of the image detected after prescan of the last image)                                 |
+--------+-------------------------------------------------------------------------------------------------------+
| n-1, n | Maximum Level for the last index image                                                                |
|        |                                                                                                       |
|        | (Maximum level of the image detected after prescan of the last image)                                 |
+--------+-------------------------------------------------------------------------------------------------------+

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

**\
2-11-8. Perforation information**

After the thumbnail scanning of the strip film, the READ command
specifying data type code 8Eh is sent from the initiator again. This
unit receives this command, and transfers the data for the number of
lines between each perforation.

The contents of the data and the format are as shown below.

+:-----:+:-----------:+:-----------:+:-----------:+:-----------:+:-----------:+:-----------:+:-----------:+:-----------:+
| Bit   | 7           | 6           | 5           | 4           | 3           | 2           | 1           | 0           |
|       |             |             |             |             |             |             |             |             |
| Byte  |             |             |             |             |             |             |             |             |
+-------+-------------+-------------+-------------+-------------+-------------+-------------+-------------+-------------+
| 0 to  | (MSB) Parameter length \[4n+1\]                                                                               |
| 2     |                                                                                                               |
+-------+---------------------------------------------------------------------------------------------------------------+
|       | (LSB)                                                                                                         |
+-------+---------------------------------------------------------------------------------------------------------------+
| 3     | Bytes per parameter (The number of bytes in each line of the absolute position information)                   |
|       |                                                                                                               |
|       | \[4\]                                                                                                         |
+-------+---------------------------------------------------------------------------------------------------------------+
| 4, 5  | (MSB) Perforation number for the 1st line                                                                     |
|       |                                                                                                               |
|       | (LSB)                                                                                                         |
+-------+-------------+-------------------------------------------------------------------------------------------------+
| 6     | Count       | Number of Pattern for the 1st line                                                              |
|       | switching   |                                                                                                 |
|       | flag        |                                                                                                 |
|       |             |                                                                                                 |
|       | \[0, 1\]    |                                                                                                 |
+-------+-------------+-------------------------------------------------------------------------------------------------+
| 7     | Pulse number for the 1st line                                                                                 |
+-------+---------------------------------------------------------------------------------------------------------------+
| :     | :                                                                                                             |
+-------+---------------------------------------------------------------------------------------------------------------+
| 4n,   | (MSB) Perforation number for the nth line                                                                     |
|       |                                                                                                               |
| 4n+1  | (LSB)                                                                                                         |
+-------+-------------+-------------------------------------------------------------------------------------------------+
| 4n+2  | Count       | Number of Pattern for the nth line                                                              |
|       | switching   |                                                                                                 |
|       | flag        |                                                                                                 |
|       |             |                                                                                                 |
|       | \[0, 1\]    |                                                                                                 |
+-------+-------------+-------------------------------------------------------------------------------------------------+
| 4n+3  | Pulse number for the nth line                                                                                 |
+-------+---------------------------------------------------------------------------------------------------------------+

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

**\
2-11-9. Boundary Information Type2**

+-------:+:----------:+:----------:+:----------:+:----------:+:----------:+:----------:+:----------:+:----------:+
| Bit    | 7          | 6          | 5          | 4          | 3          | 2          | 1          | 0          |
|        |            |            |            |            |            |            |            |            |
| Byte   |            |            |            |            |            |            |            |            |
+--------+------------+------------+------------+------------+------------+------------+------------+------------+
| 0, 1   | (MSB) Parameter length \[n-1\] (LSB)                                                                  |
+--------+-------------------------------------------------------------------------------------------------------+
| 2      | The actual number of images                                                                           |
+--------+-------------------------------------------------------------------------------------------------------+
| 3      | Reserved \[0\]                                                                                        |
+--------+-------------------------------------------------------------------------------------------------------+
| 4 to 7 | 1st image Top (Y) address                                                                             |
+--------+-------------------------------------------------------------------------------------------------------+
| 8, 9   | 1st image Perforation number                                                                          |
+--------+-------------------------------------------------------------------------------------------------------+
| 10     | 1st image Perforation decimal                                                                         |
+--------+-------------------------------------------------------------------------------------------------------+
| 11     | 1st image Pulse number                                                                                |
+--------+-------------------------------------------------------------------------------------------------------+
| 12 to  | 2nd image Top (Y) address                                                                             |
| 15     |                                                                                                       |
+--------+-------------------------------------------------------------------------------------------------------+
| 16, 17 | 2nd image Perforation number                                                                          |
+--------+-------------------------------------------------------------------------------------------------------+
| 18     | 2nd image Perforation decimal                                                                         |
+--------+-------------------------------------------------------------------------------------------------------+
| 19     | 2nd image Pulse number                                                                                |
+--------+-------------------------------------------------------------------------------------------------------+
| :      | :                                                                                                     |
+--------+-------------------------------------------------------------------------------------------------------+
| n-7 to | mth (\*) image Top (Y) address                                                                        |
| n-4    |                                                                                                       |
+--------+-------------------------------------------------------------------------------------------------------+
| n-3,   | mth image Perforation number                                                                          |
| n-2    |                                                                                                       |
+--------+-------------------------------------------------------------------------------------------------------+
| n-1    | mth image Perforation decimal                                                                         |
+--------+-------------------------------------------------------------------------------------------------------+
| n      | mth image Pulse number                                                                                |
+--------+-------------------------------------------------------------------------------------------------------+

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

+------------:+:------------:+:------------:+:------------:+:------------:+:------------:+:------------:+:------------:+:------------:+
| Bit         | 7            | 6            | 5            | 4            | 3            | 2            | 1            | 0            |
|             |              |              |              |              |              |              |              |              |
| Byte        |              |              |              |              |              |              |              |              |
+-------------+--------------+--------------+--------------+--------------+--------------+--------------+--------------+--------------+
| 0, 1        | The first point data of the first type in CCD first line                                                              |
+-------------+-----------------------------------------------------------------------------------------------------------------------+
| 2, 3        | The second point data of the first type in CCD first line                                                             |
+-------------+-----------------------------------------------------------------------------------------------------------------------+
| :           | :                                                                                                                     |
+-------------+-----------------------------------------------------------------------------------------------------------------------+
| 2mn-4,      | The (m-1)th point data of the nth type in CCD first line                                                              |
| 2mn-3       |                                                                                                                       |
+-------------+-----------------------------------------------------------------------------------------------------------------------+
| 2mn-2,      | The mth point data of the nth type in CCD first line                                                                  |
| 2mn-1       |                                                                                                                       |
+-------------+-----------------------------------------------------------------------------------------------------------------------+
| 2mn, 2mn+1  | The first point data of the first type in CCD second line                                                             |
+-------------+-----------------------------------------------------------------------------------------------------------------------+
| :           | :                                                                                                                     |
+-------------+-----------------------------------------------------------------------------------------------------------------------+
| 2lmn-2,     | The mth point data of the nth type in CCD second line                                                                 |
| 2lmn-1      |                                                                                                                       |
+-------------+-----------------------------------------------------------------------------------------------------------------------+

**2-11-11. Leak volume**

Leak_g, Leak_s, and Leak_k (2 bytes each) are sent to the initiator, in
that order.

'FFFFh' is sent for all of the three kinds when they are not recorded
once in the scanner.

The host should use the default value when 'FFFFh' is sent.

Because the value multiplied by 1,000,000 is recorded in the scanner,
the value divided by 1,000,000 should be used.

**\
2-12. SEND Command**

Table 2-12-1 SEND command

+-------:+:----------:+:----------:+:----------:+:----------:+:----------:+:----------:+:----------:+:----------:+
| Bit    | 7          | 6          | 5          | 4          | 3          | 2          | 1          | 0          |
|        |            |            |            |            |            |            |            |            |
| Byte   |            |            |            |            |            |            |            |            |
+--------+------------+------------+------------+------------+------------+------------+------------+------------+
| 0      | Operation code \[2Ah\]                                                                                |
+--------+--------------------------------------+----------------------------------------------------------------+
| 1      | Logical unit number                  | Reserved                                                       |
|        |                                      |                                                                |
|        | \[0\]                                | \[0\]                                                          |
+--------+--------------------------------------+----------------------------------------------------------------+
| 2      | Data type code                                                                                        |
+--------+-------------------------------------------------------------------------------------------------------+
| 3      | Reserved \[0\]                                                                                        |
+--------+-------------------------------------------------------------------------------------------------------+
| 4      | Data type qualifier (upper byte)                                                                      |
+--------+-------------------------------------------------------------------------------------------------------+
| 5      | Data type qualifier (lower byte)                                                                      |
+--------+-------------------------------------------------------------------------------------------------------+
| 6      | (MSB)                                                                                                 |
+--------+-------------------------------------------------------------------------------------------------------+
| 7      | Transfer length                                                                                       |
+--------+-------------------------------------------------------------------------------------------------------+
| 8      | (LSB)                                                                                                 |
+--------+-------------------------------------------------------------------------------------------------------+
| 9      | Control byte \[0\]                                                                                    |
+--------+-------------------------------------------------------------------------------------------------------+

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

+-------:+:----------:+:----------:+:----------:+:----------:+:----------:+:----------:+:----------:+:----------:+
| Bit    | 7          | 6          | 5          | 4          | 3          | 2          | 1          | 0          |
|        |            |            |            |            |            |            |            |            |
| Byte   |            |            |            |            |            |            |            |            |
+--------+------------+------------+------------+------------+------------+------------+------------+------------+
| 0      | Operation code \[C0h\]                                                                                |
+--------+--------------------------------------+----------------------------------------------------------------+
| 1      | Logical unit number                  | Reserved                                                       |
|        |                                      |                                                                |
|        | \[0\]                                | \[0\]                                                          |
+--------+--------------------------------------+----------------------------------------------------------------+
| 2 to 4 | Reserved \[0\]                                                                                        |
+--------+-------------------------------------------------------------------------------------------------------+
| 5      | Control byte \[0\]                                                                                    |
+--------+-------------------------------------------------------------------------------------------------------+

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

+-------:+:----------:+:----------:+:----------:+:----------:+:----------:+:----------:+:----------:+:----------:+
| Bit    | 7          | 6          | 5          | 4          | 3          | 2          | 1          | 0          |
|        |            |            |            |            |            |            |            |            |
| Byte   |            |            |            |            |            |            |            |            |
+--------+------------+------------+------------+------------+------------+------------+------------+------------+
| 0      | Operation code \[C1h\]                                                                                |
+--------+--------------------------------------+----------------------------------------------------------------+
| 1      | Logical unit number                  | Reserved                                                       |
|        |                                      |                                                                |
|        | \[0\]                                | \[0\]                                                          |
+--------+--------------------------------------+----------------------------------------------------------------+
| 2 to 4 | Reserved \[0\]                                                                                        |
+--------+-------------------------------------------------------------------------------------------------------+
| 5      | Control byte \[0\]                                                                                    |
+--------+-------------------------------------------------------------------------------------------------------+

The EXECUTE command performs the operation specified by the SET
PARAMETER command. The EXECUTE command is an operation activation
command.

The EXECUTE command performs the specified operation after returning
GOOD status.

A command other than the basic command must not be issued from the same
initiator to this unit before the operation termination is confirmed by
the TEST UNIT READY command.

Table 2-14-2 Sense data that is set in each status

+:----------------:+:---------------------------:+:-------------------:+
| Status           | Sense data                  | Remarks             |
+------------------+-----------------------------+---------------------+
| When the TEST    | LOGICAL UNIT IS IN PROCESS  | The command         |
| UNIT READY       | OF BECOMING READY           | terminates with the |
| command is       |                             | CHECK CONDITION     |
| received during  | (During the execution of    | status.             |
| operation        | the operation activation    |                     |
|                  | command)                    |                     |
|                  |                             |                     |
|                  | 02h-04h-01h-00h             |                     |
|                  |                             |                     |
|                  | (During loading/ejection of |                     |
|                  | the object to be scanned)   |                     |
|                  |                             |                     |
|                  | 02h-04h-01h-01h             |                     |
|                  |                             |                     |
|                  | (During the measurement of  |                     |
|                  | the correction data)        |                     |
|                  |                             |                     |
|                  | 02h-04h-01h-02h             |                     |
|                  |                             |                     |
|                  | (During the execution of    |                     |
|                  | operation for loading the   |                     |
|                  | object to be scanned)       |                     |
|                  |                             |                     |
|                  | 02h-04h-01h-03h             |                     |
|                  |                             |                     |
|                  | (During the execution of    |                     |
|                  | automatic shading or white  |                     |
|                  | balance measurement)        |                     |
|                  |                             |                     |
|                  | 02h-04h-01h-04h             |                     |
+------------------+-----------------------------+---------------------+
| When the TEST    | NO ADDITIONAL SENSE         | The command         |
| UNIT READY       | INFORMATION                 | terminates with     |
| command is       |                             | GOOD status.        |
| received after   | (No error)                  |                     |
| operation is     |                             |                     |
| terminated       | 00h-00h-00h-00h             |                     |
| normally         |                             |                     |
+------------------+-----------------------------+---------------------+
| When a command   | COMMAND SEQUENCE ERROR      | The command         |
| other than the   |                             | terminates with the |
| basic command is | (A command that makes the   | CHECK CONDITION     |
| received from    | previous SCAN command       | status. The         |
| the same         | invalid is received while   | measurement-related |
| initiator before | the scanning operation is   | operation that is   |
| the operation    | valid)                      | being performed is  |
| termination is   |                             | aborted.            |
| confirmed by the | 05h-2Ch-00h-00h             |                     |
| TEST UNIT READY  |                             |                     |
| command          |                             |                     |
+------------------+-----------------------------+---------------------+
| When a command   | LOGICAL UNIT COMMUNICATION  | The command         |
| other than the   | FAILURE                     | terminates with the |
| basic command is |                             | CHECK CONDITION     |
| received from    | (The command cannot be      | status. The         |
| the other        | executed because the        | operation that is   |
| initiator during | internal operation is being | being performed     |
| operation        | performed.)                 | continues without   |
|                  |                             | any influence.      |
|                  | 0Bh-08h-00h-00h             |                     |
+------------------+-----------------------------+---------------------+

+------------------+-----------------------------+--------------------+
| > When the       | LOGICAL UNIT NOT READY,     | The command        |
| > operation is   | CAUSE NOT REPORTABLE        | terminates with    |
| > not terminated |                             | the CHECK          |
| > normally       | (The internal mechanical    | CONDITION status   |
|                  | error occurred.)            | for the TEST UNIT  |
|                  |                             | READY command that |
|                  | 02h-04h-02h-00h             | is received after  |
|                  |                             | the operation is   |
|                  |                             | terminated.        |
+------------------+-----------------------------+--------------------+
| When the EXECUTE | COMMAND SEQUENCE ERROR      | The command        |
| command is       |                             | terminates with    |
| received before  | (The EXECUTE command is     | the CHECK          |
| the operation    | received before the         | CONDITION status.  |
| parameter is set | parameter is set by the SET |                    |
| by the SET       | PARAMETER command.)         |                    |
| PARAMETER        |                             |                    |
| command          | 05h-2Ch-00h-00h             |                    |
+------------------+-----------------------------+--------------------+

**2-15. SET PARAMETER Command**

Table 2-15-1 SET PARAMETER command

+-------:+:----------:+:----------:+:----------:+:----------:+:----------:+:----------:+:----------:+:----------:+
| Bit    | 7          | 6          | 5          | 4          | 3          | 2          | 1          | 0          |
|        |            |            |            |            |            |            |            |            |
| Byte   |            |            |            |            |            |            |            |            |
+--------+------------+------------+------------+------------+------------+------------+------------+------------+
| 0      | Operation code \[E0h\]                                                                                |
+--------+--------------------------------------+----------------------------------------------------------------+
| 1      | Logical unit number                  | Reserved                                                       |
|        |                                      |                                                                |
|        | \[0\]                                | \[0\]                                                          |
+--------+--------------------------------------+----------------------------------------------------------------+
| 2      | Operation code                                                                                        |
+--------+-------------------------------------------------------------------------------------------------------+
| 3 to 5 | Reserved \[0\]                                                                                        |
+--------+-------------------------------------------------------------------------------------------------------+
| 6      | (MSB)                                                                                                 |
+--------+-------------------------------------------------------------------------------------------------------+
| 7      | Parameter length \[Recommended value: 13d\]                                                           |
+--------+-------------------------------------------------------------------------------------------------------+
| 8      | (LSB)                                                                                                 |
+--------+-------------------------------------------------------------------------------------------------------+
| 9      | Control byte \[0\]                                                                                    |
+--------+-------------------------------------------------------------------------------------------------------+

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

+------:+:-----------:+:-----------:+:-----------:+:-----------:+:-----------:+:-----------:+:-----------:+:-----------:+
| Bit   | 7           | 6           | 5           | 4           | 3           | 2           | 1           | 0           |
|       |             |             |             |             |             |             |             |             |
| Byte  |             |             |             |             |             |             |             |             |
+-------+-------------+-------------+-------------+-------------+-------------+-------------+-------------+-------------+
| 0     | Color specification                                                                                           |
+-------+---------------------------------------------------------------------------------------------------------------+
| 1 to  | First setting value                                                                                           |
| 4     |                                                                                                               |
+-------+---------------------------------------------------------------------------------------------------------------+
| 5 to  | Second setting value                                                                                          |
| 8     |                                                                                                               |
+-------+---------------------------------------------------------------------------------------------------------------+
| 9, 10 | Speed                                                                                                         |
+-------+---------------------------------------------------------------------------------------------------------------+
| 11    | Torque                                                                                                        |
+-------+---------------------------------------------------------------------------------------------------------------+
| 12    | Driving method                                                                                                |
+-------+---------------------------------------------------------------------------------------------------------------+

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
  performing auto focus when the operation code is 'Color oriented Auto
  Focus'.

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

+:---------:+:--------------------:+:----------------:+:----------:+:-------:+
| Operation | Internal operation   | Contents of      | Valid      | Support |
|           | to be set            | operation        | parameters | of this |
| code      |                      |                  |            | unit    |
+-----------+----------------------+------------------+------------+---------+
| 80h       | Initialize           | This unit is     | None       | Yes     |
|           |                      | initialized in   |            |         |
|           |                      | the same manner  |            |         |
|           |                      | as that of power |            |         |
|           |                      | ON.              |            |         |
+-----------+----------------------+------------------+------------+---------+
| 81h       | Return to the origin | Return to the    | None       | Yes     |
|           |                      | origin           |            |         |
+-----------+----------------------+------------------+------------+---------+
| 90h       | Change Unit          |                  | 1st Val    | No      |
+-----------+----------------------+------------------+------------+---------+
| 91h       | Auto AF              | Automatic AF     | 1st Val    | Yes     |
|           |                      | execution        |            |         |
|           |                      |                  |            |         |
|           |                      | ON/OFF           |            |         |
+-----------+----------------------+------------------+------------+---------+
| A0h       | Auto Focus           | Performs the     | 1st Val,   | Yes     |
|           |                      | auto focus       | 2nd Val    |         |
+-----------+----------------------+------------------+------------+---------+
| A1h       | Color oriented Auto  |                  | 1st Val,   | No      |
|           | Focus                |                  | 2nd Val,   |         |
|           |                      |                  | color      |         |
+-----------+----------------------+------------------+------------+---------+
| B0h       | Setup Shading Data   | Performs the     | None       | Yes     |
|           |                      | shading          |            |         |
|           |                      | measurement      |            |         |
+-----------+----------------------+------------------+------------+---------+
| B1h       | Setup Dark Current   | Performs the     | None       | Yes     |
|           | Correction Data      | dark voltage     |            |         |
|           |                      | measurement      |            |         |
+-----------+----------------------+------------------+------------+---------+
| B2h       | Setup Offset         |                  | None       | No      |
|           | Correction Data      |                  |            |         |
+-----------+----------------------+------------------+------------+---------+
| B4h       | Unload time set      | Setting the      | 1st Val,   | Yes     |
|           |                      | object unloading | 2nd Val    |         |
|           |                      | time             |            |         |
+-----------+----------------------+------------------+------------+---------+
| C0h       | Stage Move           | Moves the scan   | 1st Val    | Yes     |
|           |                      | block in the     |            |         |
|           |                      | scanning         |            |         |
|           |                      | direction        |            |         |
+-----------+----------------------+------------------+------------+---------+
| C1h       | Focus Move           | Moves the scan   | 1st Val    | Yes     |
|           |                      | block in the AF  |            |         |
|           |                      | direction        |            |         |
+-----------+----------------------+------------------+------------+---------+
| D0h       | Unload object        | Unloads the      | None       | Yes     |
|           |                      | object           |            |         |
+-----------+----------------------+------------------+------------+---------+
| D1h       | Load object          | Loads the object | None       | Yes     |
+-----------+----------------------+------------------+------------+---------+
| D2h       | Absolute positioning | Absolute         | 1st Val    | Yes     |
|           |                      | positioning of   |            |         |
|           |                      | the object       |            |         |
+-----------+----------------------+------------------+------------+---------+
| D3h       | Relative positioning | Relative         | 1st Val    | No      |
|           |                      | positioning      |            |         |
+-----------+----------------------+------------------+------------+---------+
| D4h       | Rotate               | Rotation         | 1st Val    | No      |
+-----------+----------------------+------------------+------------+---------+
| D5h       | FD                   | FD movement time | 1st Val,   | Yes     |
|           |                      | setting          | 2nd Val    |         |
+-----------+----------------------+------------------+------------+---------+
| D6h       | SA Lock              | SA lock          | 1^st^ Val  | Yes     |
|           |                      | mechanism ON/OFF |            |         |
+-----------+----------------------+------------------+------------+---------+

1^st^ Val: First setting value

2^nd^ Val: Second setting value

Color: Color specification

Speed: Speed specification

Torque: Torque

Drive: Driving method

Table 2-15-4 Descriptions of each parameter for the operation codes

+------------+----------------+:-------------:+:------------:+:--------------:+:--------:+:-------:+:---------:+
| Opera-tion | Color          | First setting | Second       | Speed          | Torque   | Driving | Remarks   |
| code       | specifi-cation | value         | setting      |                |          | method  |           |
|            |                |               | value        | specifi-cation | (Torque) |         |           |
|            | (Color)        | (1^st^ Val)   |              |                |          | (Drive) |           |
|            |                |               | (2^nd^ Val)  | (Speed)        |          |         |           |
+------------+----------------+---------------+--------------+----------------+----------+---------+-----------+
| 80h        | \-             | \-            | \-           | \-             | \-       | \-      | No        |
|            |                |               |              |                |          |         | parameter |
+------------+----------------+---------------+--------------+----------------+----------+---------+-----------+
| 81h        | \-             | \-            | \-           | \-             | \-       | \-      | No        |
|            |                |               |              |                |          |         | parameter |
+------------+----------------+---------------+--------------+----------------+----------+---------+-----------+
| 90h        | \-             | \-            | \-           | \-             | \-       | \-      | Not       |
|            |                |               |              |                |          |         | supported |
+------------+----------------+---------------+--------------+----------------+----------+---------+-----------+
| 91h        | \-             | Automatic AF  | \-           | \-             | \-       | \-      |           |
|            |                | execution     |              |                |          |         |           |
|            |                |               |              |                |          |         |           |
|            |                | 0: OFF        |              |                |          |         |           |
|            |                |               |              |                |          |         |           |
|            |                | 1: ON         |              |                |          |         |           |
+------------+----------------+---------------+--------------+----------------+----------+---------+-----------+
| A0h        | \-             | Address on    | Address on   | \-             | \-       | \-      |           |
|            |                | the medium    | the medium   |                |          |         |           |
|            |                | where AF is   | where AF is  |                |          |         |           |
|            |                | performed in  | performed in |                |          |         |           |
|            |                | the           | the          |                |          |         |           |
|            |                | main-scanning | sub-scanning |                |          |         |           |
|            |                | direction     | direction    |                |          |         |           |
+------------+----------------+---------------+--------------+----------------+----------+---------+-----------+
| A1h        | \-             | \-            | \-           | \-             | \-       | \-      | Not       |
|            |                |               |              |                |          |         | supported |
+------------+----------------+---------------+--------------+----------------+----------+---------+-----------+
| B0h        | \-             | \-            | \-           | \-             | \-       | \-      | No        |
|            |                |               |              |                |          |         | parameter |
+------------+----------------+---------------+--------------+----------------+----------+---------+-----------+
| B1h        | \-             | \-            | \-           | \-             | \-       | \-      | No        |
|            |                |               |              |                |          |         | parameter |
+------------+----------------+---------------+--------------+----------------+----------+---------+-----------+
| B2h        | \-             | \-            | \-           | \-             | \-       | \-      | Not       |
|            |                |               |              |                |          |         | supported |
+------------+----------------+---------------+--------------+----------------+----------+---------+-----------+
| B4h        | \-             | Setting value | 0: Timer OFF | \-             | \-       | \-      |           |
|            |                | of the        |              |                |          |         |           |
|            |                | unloading     | 1: Timer ON  |                |          |         |           |
|            |                | time          |              |                |          |         |           |
|            |                |               |              |                |          |         |           |
|            |                | (unit \[s\],  |              |                |          |         |           |
|            |                | default 600   |              |                |          |         |           |
|            |                | \[s\])        |              |                |          |         |           |
+------------+----------------+---------------+--------------+----------------+----------+---------+-----------+
| C0h        | \-             | Address in    | \-           | \-             | \-       | \-      |           |
|            |                | the scanning  |              |                |          |         |           |
|            |                | direction     |              |                |          |         |           |
+------------+----------------+---------------+--------------+----------------+----------+---------+-----------+
| C1h        | \-             | Address in    | \-           | \-             | \-       | \-      |           |
|            |                | the AF        |              |                |          |         |           |
|            |                | direction     |              |                |          |         |           |
+------------+----------------+---------------+--------------+----------------+----------+---------+-----------+
| D0h        | \-             | \-            | \-           | \-             | \-       | \-      | No        |
|            |                |               |              |                |          |         | parameter |
+------------+----------------+---------------+--------------+----------------+----------+---------+-----------+
| D1h        | \-             | \-            | \-           | \-             | \-       | \-      | No        |
|            |                |               |              |                |          |         | parameter |
+------------+----------------+---------------+--------------+----------------+----------+---------+-----------+
| D2h        | \-             | Address in    | \-           | \-             | \-       | \-      |           |
|            |                | the           |              |                |          |         |           |
|            |                | main-scanning |              |                |          |         |           |
|            |                | direction     |              |                |          |         |           |
+------------+----------------+---------------+--------------+----------------+----------+---------+-----------+
| D3h        | \-             | \-            | \-           | \-             | \-       | \-      | Not       |
|            |                |               |              |                |          |         | supported |
+------------+----------------+---------------+--------------+----------------+----------+---------+-----------+
| D4h        | \-             | \-            | \-           | \-             | \-       | \-      | Not       |
|            |                |               |              |                |          |         | supported |
+------------+----------------+---------------+--------------+----------------+----------+---------+-----------+
| D5h        | \-             | From 0 to     | 0: Loads the | \-             | \-       | \-      |           |
|            |                | 3200          | object       |                |          |         |           |
|            |                |               |              |                |          |         |           |
|            |                | (in units of  | 1: Unloads   |                |          |         |           |
|            |                | 10 ms, 1 ms   | the object   |                |          |         |           |
|            |                | for 0)        |              |                |          |         |           |
+------------+----------------+---------------+--------------+----------------+----------+---------+-----------+
| D6h        | \-             | 0: OFF        | \-           | \-             | \-       | \-      |           |
|            |                |               |              |                |          |         |           |
|            |                | 1: ON         |              |                |          |         |           |
+------------+----------------+---------------+--------------+----------------+----------+---------+-----------+

Note) The address is shown in units of 4000 dpi.

**2-16. GET PARAMETER Command**

Table 2-16-1 GET PARAMETER command

+-------:+:----------:+:----------:+:----------:+:----------:+:----------:+:----------:+:----------:+:----------:+
| Bit    | 7          | 6          | 5          | 4          | 3          | 2          | 1          | 0          |
|        |            |            |            |            |            |            |            |            |
| Byte   |            |            |            |            |            |            |            |            |
+--------+------------+------------+------------+------------+------------+------------+------------+------------+
| 0      | Operation code \[E1h\]                                                                                |
+--------+--------------------------------------+----------------------------------------------------------------+
| 1      | Logical unit number                  | Reserved                                                       |
|        |                                      |                                                                |
|        | \[0\]                                | \[0\]                                                          |
+--------+--------------------------------------+----------------------------------------------------------------+
| 2      | Operation code                                                                                        |
+--------+-------------------------------------------------------------------------------------------------------+
| 3 to 5 | Reserved \[0\]                                                                                        |
+--------+-------------------------------------------------------------------------------------------------------+
| 6      | (MSB)                                                                                                 |
+--------+-------------------------------------------------------------------------------------------------------+
| 7      | Parameter length                                                                                      |
+--------+-------------------------------------------------------------------------------------------------------+
| 8      | (LSB)                                                                                                 |
+--------+-------------------------------------------------------------------------------------------------------+
| 9      | Control byte \[0\]                                                                                    |
+--------+-------------------------------------------------------------------------------------------------------+

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

**\
2-17. RECEIVE DIAGNOSTIC RESULTS Command**

Table 2-17-1 RECEIVE DIAGNOSTIC RESULTS command

+--------------:+:-------------:+:-------------:+:-------------:+:-------------:+:-------------:+:-------------:+:-------------:+:-------------:+
| Bit           | 7             | 6             | 5             | 4             | 3             | 2             | 1             | 0             |
|               |               |               |               |               |               |               |               |               |
| Byte          |               |               |               |               |               |               |               |               |
+---------------+---------------+---------------+---------------+---------------+---------------+---------------+---------------+---------------+
| 0             | Operation code \[1Ch\]                                                                                                        |
+---------------+-----------------------------------------------+-------------------------------------------------------------------------------+
| 1             | Logical unit number                           | Reserved                                                                      |
|               |                                               |                                                                               |
|               | \[0\]                                         | \[0\]                                                                         |
+---------------+-----------------------------------------------+-------------------------------------------------------------------------------+
| 2             | Reserved \[0\]                                                                                                                |
+---------------+-------------------------------------------------------------------------------------------------------------------------------+
| 3             | (MSB) Allocation length                                                                                                       |
|               |                                                                                                                               |
|               | (LSB)                                                                                                                         |
+---------------+                                                                                                                               |
| 4             |                                                                                                                               |
+---------------+-------------------------------------------------------------------------------------------------------------------------------+
| 5             | Reserved \[0\]                                                                                                                |
+---------------+-------------------------------------------------------------------------------------------------------------------------------+
|                                                                                                                                               |
+-----------------------------------------------------------------------------------------------------------------------------------------------+

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

+:---------------------:+:-------------------------:+:----------------:+
| Status                | Sense data                | Remarks          |
+-----------------------+---------------------------+------------------+
| When the SEND         | INVALID FIELD IN CDB      | The command      |
| DIAGNOSTIC command is |                           | terminates with  |
| received with the     | (Some illegal data exists | the CHECK        |
| specification of      | in the CDB.)              | CONDITION        |
| parameter when the    |                           | status.          |
| adapter is not        | 05h-24h-00h-00h           |                  |
| attached              |                           |                  |
+-----------------------+---------------------------+------------------+
| When the RECEIVE      | INVALID COMMAND OPERATION | The command      |
| DIAGNOSTIC RESULTS    | CODE                      | terminates with  |
| command is received   |                           | the CHECK        |
| independently when    | (Op-Code that is not      | CONDITION        |
| the adapter is not    | supported is received.)   | status.          |
| attached              |                           |                  |
|                       | 05h-20h-00h-00h           |                  |
+-----------------------+---------------------------+------------------+
| When the RECEIVE      | COMMAND SEQUENCE ERROR    | The command      |
| DIAGNOSTIC RESULTS    |                           | terminates with  |
| command is received   | (The RECEIVE DIAGNOSTIC   | the CHECK        |
| independently when    | RESULT command is         | CONDITION        |
| the adapter is        | received independently    | status.          |
| attached              | when the adapter for      |                  |
|                       | inspection is attached.)  |                  |
|                       |                           |                  |
|                       | 05h-2Ch-00h-00h           |                  |
+-----------------------+---------------------------+------------------+
