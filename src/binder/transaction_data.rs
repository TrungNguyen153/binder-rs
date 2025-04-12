use std::fmt::Debug;

use nix::libc;

use crate::parcel::{FnFreeBuffer, Parcel};

use super::transaction::TransactionFlag;

#[derive(Clone, Copy)]
#[repr(C)]
pub union TargetUnion {
    pub handle: u32,
    pub ptr: *mut libc::c_void,
}

impl TargetUnion {
    pub fn new_handle(handle: u32) -> Self {
        let mut s: Self = unsafe { std::mem::zeroed() };
        s.handle = handle;
        s
    }

    pub fn new_ptr(ptr: *mut libc::c_void) -> Self {
        Self { ptr }
    }
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
#[repr(C, packed(4))]
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
}

impl BinderTransactionData {
    pub fn to_parcel(&self, free_buffer: Option<FnFreeBuffer>) -> Parcel {
        Parcel::from_ipc_parts(
            self.data,
            self.data_size,
            self.offsets,
            self.offsets_size / size_of::<usize>(),
            free_buffer,
        )
    }
}
