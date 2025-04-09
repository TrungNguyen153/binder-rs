use std::{
    ffi::c_void,
    num::NonZero,
    os::fd::{AsFd, AsRawFd, FromRawFd, OwnedFd},
    ptr::NonNull,
};

use constant::{BINDER_VM_SIZE, DEFAULT_MAX_BINDER_THREADS};
use devices::BinderDevice;
use nix::{
    fcntl::{OFlag, open},
    ioctl_readwrite, ioctl_write_ptr,
    sys::{
        mman::{MapFlags, ProtFlags, mmap},
        stat::Mode,
    },
};

use crate::error::BinderResult;

pub mod binder_type;
mod constant;
pub mod devices;
pub mod transaction;

#[macro_export]
macro_rules! pack_chars {
    ($c1:expr, $c2:expr, $c3:expr, $c4:expr) => {
        ((($c1 as u32) << 24) | (($c2 as u32) << 16) | (($c3 as u32) << 8) | ($c4 as u32))
    };
}

/// A structure representing the binder version
#[derive(Default, Debug)]
#[repr(C)]
pub struct BinderVersion {
    protocol_version: i32,
}

ioctl_write_ptr!(binder_set_max_threads, b'b', 5, u32);
ioctl_readwrite!(binder_read_version, b'b', 9, BinderVersion);

pub struct Binder {
    fd: OwnedFd,
    mem: NonNull<c_void>,
}

impl Binder {
    pub fn new(device: BinderDevice) -> BinderResult<Self> {
        let flags = OFlag::O_RDWR | OFlag::O_CLOEXEC | OFlag::O_NONBLOCK;

        let fd = open(device.to_string().as_str(), flags, Mode::empty())?;

        let fd = unsafe { OwnedFd::from_raw_fd(fd) };

        let mut binder_version = BinderVersion::default();

        unsafe {
            binder_read_version(fd.as_raw_fd(), &mut binder_version)?;
        }

        info!("{binder_version:#?}");

        let mem = unsafe {
            mmap(
                None,
                NonZero::new(BINDER_VM_SIZE).unwrap(),
                ProtFlags::PROT_READ,
                MapFlags::MAP_PRIVATE | MapFlags::MAP_NORESERVE,
                fd.as_fd(),
                0,
            )?
        };

        unsafe { binder_set_max_threads(fd.as_raw_fd(), &DEFAULT_MAX_BINDER_THREADS)? };

        Ok(Self { fd, mem })
    }
}
