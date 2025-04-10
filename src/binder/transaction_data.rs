use nix::libc;

use crate::parcel::Parcel;

#[repr(C)]
pub struct BinderTransactionData {
    pub target: *mut libc::c_void,
    pub cookie: *mut libc::c_void,
    pub code: libc::c_uint,
    pub flags: libc::c_uint,
    pub sender_pid: libc::pid_t,
    pub sender_euid: libc::uid_t,
    pub data_size: libc::size_t,
    pub offsets_size: libc::size_t,
    pub buffer: *mut libc::c_void,
    pub offset: *mut libc::c_void,
}

impl BinderTransactionData {
    pub fn to_parcel(&self) -> Parcel {
        unsafe {
            Parcel::from_slice(std::slice::from_raw_parts(
                self.buffer as *const u8,
                self.data_size,
            ))
        }
    }
}
