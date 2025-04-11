// https://android.googlesource.com/platform/frameworks/native/+/idea133/cmds/servicemanager/binder.c
// https://github.com/rong1129/android-binder-ipc/blob/master/module/binder.h
// https://android.googlesource.com/platform/frameworks/native/+/master/libs/binder/rust/src/binder.rs
use std::{
    ffi::c_void,
    num::NonZero,
    os::fd::{AsFd, AsRawFd, FromRawFd, OwnedFd},
    ptr::NonNull,
};

use command_protocol::{BinderCommand, BinderReturn};
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
use num_traits::FromPrimitive;
use transaction::{Transaction, TransactionFlag};
use transaction_data::{BinderTransactionData, TargetUnion};

use crate::{
    error::BinderResult,
    parcel::{self, Parcel},
    parcelable::Parcelable,
};

pub mod binder_type;
pub mod command_protocol;
mod constant;
pub mod devices;
pub mod flat_object;
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

    pub fn binder_read(&self, mut buffer: impl AsMut<[u8]>) -> BinderResult<usize> {
        let buffer = buffer.as_mut();
        let mut data = BinderWriteRead {
            write_size: 0,
            write_consumed: 0,
            write_buffer: std::ptr::null(),
            read_size: buffer.len(),
            read_consumed: 0,
            read_buffer: buffer.as_mut_ptr(),
        };

        unsafe { binder_write_read(self.fd.as_raw_fd(), &mut data)? };

        Ok(data.read_consumed)
    }

    pub fn enter_loop(&self) -> BinderResult<()> {
        let mut parcel = Parcel::default();
        BinderCommand::EnterLooper.serialize(&mut parcel)?;
        self.binder_write(parcel)
    }

    pub fn exit_loop(&self) -> BinderResult<()> {
        let mut parcel = Parcel::default();
        BinderCommand::ExitLooper.serialize(&mut parcel)?;
        self.binder_write(&parcel)
    }

    pub fn binder_loop<
        F: FnMut(&Binder, &mut Parcel, Option<&BinderTransactionData>) -> bool + Copy + Clone,
    >(
        &self,
        handler: F,
    ) -> BinderResult<!> {
        let mut parcel = Parcel::with_capacity(32 * 8);

        loop {
            let read_consumed = self.binder_read(&mut parcel)?;
            parcel.resize_data(read_consumed);
            parcel.reset_cursor();
            self.binder_parse(&mut parcel, handler)?;
            parcel.reset_cursor();
        }
    }

    fn binder_parse<F: FnMut(&Binder, &mut Parcel, Option<&BinderTransactionData>) -> bool>(
        &self,
        parcel: &mut Parcel,
        mut handler: F,
    ) -> BinderResult<bool> {
        while parcel.has_unread_data() {
            let cmd_value = parcel.read_u32()?;
            let cmd = BinderReturn::from_u32(cmd_value);
            if cmd.is_none() {
                error!("Unknown BinderReturn command: {cmd_value}");
                return Ok(false);
            }

            let cmd = cmd.unwrap();
            info!("Got cmd: {cmd:#?}");

            match cmd {
                BinderReturn::Error => {
                    error!("Error: {}", parcel.read_i32()?);
                }
                BinderReturn::Ok => {}
                BinderReturn::Transaction | BinderReturn::Reply => {
                    let transaction_data_in = parcel.read_transaction_data()?;
                    let mut parcel = unsafe {
                        Parcel::from_data_and_offsets(
                            transaction_data_in.data,
                            transaction_data_in.data_size as usize,
                            transaction_data_in.offsets,
                            transaction_data_in.offsets_size as usize / size_of::<usize>(),
                        )
                    };
                    handler(self, &mut parcel, Some(&transaction_data_in));
                }
                BinderReturn::AcquireResult => {
                    info!("Result: {}", parcel.read_i32()?);
                }
                BinderReturn::DeadReply => {
                    panic!("Got a DEAD_REPLY");
                }
                BinderReturn::TransactionComplete => {}
                BinderReturn::IncRefs => {}
                BinderReturn::Acquire => {}
                BinderReturn::Release => {}
                BinderReturn::DecRefs => {}
                BinderReturn::AttemptAcquire => {}
                BinderReturn::Noop => {}
                BinderReturn::SpawnLooper => {}
                BinderReturn::Finished => {}
                BinderReturn::DeadBinder => {}
                BinderReturn::ClearDeathNotification => {}
                BinderReturn::FailedReply => {
                    panic!("Got a FailedReply");
                }
                BinderReturn::FrozenReply => {}
                BinderReturn::OnwaySpamSuspect => {}
            }
        }

        Ok(true)
    }

    pub fn transaction(
        &self,
        handle: u32,
        code: u32,
        flags: TransactionFlag,
        data: &mut Parcel,
    ) -> BinderResult<()> {
        let mut parcel = Parcel::default();
        BinderCommand::Transaction.serialize(&mut parcel)?;

        let transaction_data_out = BinderTransactionData {
            target: TargetUnion { handle },
            code,
            flags,
            cookie: std::ptr::null_mut(),
            sender_pid: 0,
            sender_euid: 0,
            data_size: data.len(),
            offsets_size: (data.offsets_len() * size_of::<usize>()),
            data: if !data.is_empty() {
                data.as_slice_mut().as_mut_ptr()
            } else {
                std::ptr::null_mut()
            },
            offsets: if data.offsets_len() != 0 {
                data.offsets_mut().as_mut_ptr()
            } else {
                std::ptr::null_mut()
            },
        };

        parcel.write_transaction_data(&transaction_data_out)?;
        self.binder_write(parcel)
    }

    pub fn transaction_with_parse<F>(
        &self,
        handle: u32,
        code: u32,
        flags: TransactionFlag,
        data: &mut Parcel,
        handler: F,
    ) -> BinderResult<()>
    where
        F: FnMut(&Binder, &mut Parcel, Option<&BinderTransactionData>) -> bool + Copy + Clone,
    {
        self.transaction(handle, code, flags, data)?;

        let mut parcel = Parcel::with_capacity(32 * 8);

        loop {
            let read_consumed = self.binder_read(&mut parcel)?;
            parcel.resize_data(read_consumed);
            parcel.reset_cursor();
            self.binder_parse(&mut parcel, handler)?;
            parcel.reset_cursor();
        }
        Ok(())
    }

    pub fn reply(&self, data: &mut Parcel, flags: TransactionFlag) -> BinderResult<()> {
        let mut parcel = Parcel::default();
        BinderCommand::Reply.serialize(&mut parcel)?;
        let transaction_data_out = BinderTransactionData {
            target: TargetUnion {
                ptr: 0xffffffff as usize as _,
            },
            code: Transaction::None.into(),
            flags,
            cookie: std::ptr::null_mut(),
            sender_pid: 0,
            sender_euid: 0,
            data_size: data.len(),
            offsets_size: (data.offsets_len() * size_of::<usize>()),
            data: if !data.is_empty() {
                data.as_slice_mut().as_mut_ptr()
            } else {
                std::ptr::null_mut()
            },
            offsets: if data.offsets_len() != 0 {
                data.offsets_mut().as_mut_ptr()
            } else {
                std::ptr::null_mut()
            },
        };
        parcel.write_transaction_data(&transaction_data_out)?;
        self.binder_write(parcel)
    }
}
