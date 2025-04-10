// https://android.googlesource.com/platform/frameworks/native/+/idea133/cmds/servicemanager/binder.c
// https://github.com/rong1129/android-binder-ipc/blob/master/module/binder.h
use std::{
    ffi::c_void,
    num::NonZero,
    os::fd::{AsFd, AsRawFd, FromRawFd, OwnedFd},
    ptr::NonNull,
};

use command_protocol::BinderCommand;
use constant::{BINDER_VM_SIZE, DEFAULT_MAX_BINDER_THREADS};
use devices::BinderDevice;
use nix::{
    fcntl::{OFlag, open},
    ioctl_readwrite, ioctl_write_ptr, libc,
    sys::{
        mman::{MapFlags, ProtFlags, mmap, munmap},
        stat::Mode,
    },
};

use crate::{
    error::BinderResult,
    parcel::{self, Parcel},
};

pub mod binder_type;
pub mod command_protocol;
mod constant;
pub mod devices;
pub mod transaction;
pub mod transaction_data;

/// A structure representing the binder version
#[derive(Default, Debug)]
#[repr(C)]
pub struct BinderVersion(i32);

#[repr(C)]
struct BinderWriteRead {
    write_size: libc::size_t,
    write_consumed: libc::size_t,
    write_buffer: *const u8,
    read_size: libc::size_t,
    read_consumed: libc::size_t,
    read_buffer: *mut u8,
}
ioctl_readwrite!(binder_write_read, b'b', 1, BinderWriteRead);
ioctl_write_ptr!(binder_set_max_threads, b'b', 5, u32);
ioctl_write_ptr!(binder_set_context_mgr, b'b', 7, i32);
ioctl_readwrite!(binder_read_version, b'b', 9, BinderVersion);

pub struct Binder {
    fd: OwnedFd,
    mem: NonNull<c_void>,
}

impl Drop for Binder {
    fn drop(&mut self) {
        unsafe {
            if let Err(e) = munmap(self.mem, BINDER_VM_SIZE) {
                error!("[DropBinder] {e}")
            }
        }
    }
}

impl Binder {
    pub fn new(device: BinderDevice) -> BinderResult<Self> {
        let flags = OFlag::O_RDWR | OFlag::O_CLOEXEC;

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

    pub fn become_context_manager(&self) -> BinderResult<()> {
        unsafe { binder_set_context_mgr(self.fd.as_raw_fd(), std::ptr::null_mut())? };
        Ok(())
    }

    pub fn binder_write(&self, data: impl AsRef<[u8]>) -> BinderResult<()> {
        let data = data.as_ref();
        // write in c implement not use write_consumed

        info!("[BinderWrite]");
        let mut data = BinderWriteRead {
            write_size: data.len(),
            write_consumed: 0,
            write_buffer: data.as_ptr(),
            read_size: 0,
            read_consumed: 0,
            read_buffer: std::ptr::null_mut(),
        };

        let ret = unsafe { binder_write_read(self.fd.as_raw_fd(), &mut data)? };
        info!("[BinderWrite] {ret}");
        Ok(())
    }

    pub fn binder_write_parcel(&self, parcel: &Parcel) -> BinderResult<()> {
        self.binder_write(parcel)
    }

    pub fn binder_read(&self, mut buffer: impl AsMut<[u8]>) -> BinderResult<libc::c_int> {
        let buffer = buffer.as_mut();
        let mut data = BinderWriteRead {
            write_size: 0,
            write_consumed: 0,
            write_buffer: std::ptr::null(),
            read_size: buffer.len(),
            read_consumed: 0,
            read_buffer: buffer.as_mut_ptr(),
        };

        unsafe { Ok(binder_write_read(self.fd.as_raw_fd(), &mut data)?) }
    }

    pub fn binder_read_parcel(&self, parcel: &mut Parcel) -> BinderResult<()> {
        self.binder_read(parcel)?;
        Ok(())
    }

    pub fn enter_loop(&self) -> BinderResult<()> {
        let mut parcel = Parcel::default();
        parcel.write_object(BinderCommand::EnterLooper)?;
        self.binder_write_parcel(&parcel)
    }

    pub fn exit_loop(&self) -> BinderResult<()> {
        let mut parcel = Parcel::default();
        parcel.write_object(BinderCommand::ExitLooper)?;
        self.binder_write_parcel(&parcel)
    }

    pub fn binder_loop(&self) -> BinderResult<!> {
        self.enter_loop()?;
        let mut parcel = Parcel::with_capacity(32 * 8);

        loop {
            self.binder_read_parcel(&mut parcel)?;
            self.binder_parse(&mut parcel)?;
            todo!()
        }
    }

    fn binder_parse(&self, parcel: &Parcel) -> BinderResult<bool> {
        todo!()
    }
}
