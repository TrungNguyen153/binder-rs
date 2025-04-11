use std::fmt::Debug;

use nix::libc;

use crate::parcel::Parcel;

use super::transaction::TransactionFlag;

#[repr(C)]
pub union TargetUnion {
    pub handle: u32,
    pub ptr: *mut libc::c_void,
}

impl Debug for TargetUnion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TargetUnion {{\n")?;
        write!(f, "\t handle: {},\n", unsafe { self.handle })?;
        write!(f, "\t ptr: {:#?},\n", unsafe { self.ptr })?;
        write!(f, "}}")
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct BinderTransactionData {
    pub target: TargetUnion,
    pub cookie: *mut libc::c_void,
    pub code: u32,
    pub flags: TransactionFlag,

    pub sender_pid: libc::pid_t,
    pub sender_euid: libc::uid_t,

    /// in bytes
    pub data_size: libc::size_t,
    /// in bytes
    pub offsets_size: libc::size_t,
    pub data: *mut u8,
    pub offsets: *mut usize,
    // pub sec_ctx: *mut libc::c_void,
}

impl BinderTransactionData {
    pub fn to_parcel(&self) -> Parcel {
        unsafe {
            Parcel::from_slice(std::slice::from_raw_parts(
                self.data as *const u8,
                self.data_size,
            ))
        }
    }
}
