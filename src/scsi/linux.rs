//! Safe-rust wrapper over the "SCSI-Generic" linux userspace layer

use std::os::raw::{c_int, c_uchar, c_uint, c_ushort, c_void};

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct sg_io_hdr {
    /// Interface ID [Input]: Must be 'S'
    interface_id: c_int,
    /// Data Trasfer Direction [Input]
    dxfer_direction: DxferDirection,
    cmd_len: c_uchar,
    mx_sb_len: c_uchar,
    iovec_count: c_ushort,
    dxfer_len: c_uint,
    dxferp: *mut c_void,
    cmdp: *mut c_uchar,
    sbp: *mut c_uchar,
    timeout: c_uint,
    flags: c_uint,
    pack_id: c_int,
    usr_ptr: *mut c_void,
    status: c_uchar,
    masked_status: c_uchar,
    msg_status: c_uchar,
    sb_len_wr: c_uchar,
    host_status: c_ushort,
    driver_status: c_ushort,
    resid: c_int,
    duration: c_uint,
    info: c_uint,
}

#[repr(C)]
enum DxferDirection {
    /// SCSI Test Unit Ready
    None = -1,
    /// WRITE
    ToDev = -2,
    /// READ
    FromDev = -3,
    ToFromDev = -4,
    /// Unknown data direction (probably unused)
    Unknown = -5
}
