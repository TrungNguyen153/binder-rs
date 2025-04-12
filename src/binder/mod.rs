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
    error::Result,
    parcel::{Parcel, parcelable::Serialize},
};

pub mod binder_type;
pub mod command_protocol;
pub mod constant;
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
    pub fn new(device: BinderDevice) -> Result<Self> {
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

    pub fn become_context_manager(&self) -> Result<()> {
        unsafe { binder_set_context_mgr(self.fd.as_raw_fd(), std::ptr::null_mut())? };
        Ok(())
    }

    pub fn binder_write(&self, buffer: &mut Parcel) -> Result<()> {
        if buffer.data_size() == 0 {
            // warn!("[BinderWrite] trying write buffer size 0.");
            return Ok(());
        }
        // we expect:
        // + Driver consume all our buffer

        info!("[BinderWrite] size: {}", buffer.data_size());
        let mut data = BinderWriteRead {
            write_size: buffer.data_size(),
            write_consumed: 0,
            write_buffer: buffer.as_ptr(),
            read_size: 0,
            read_consumed: 0,
            read_buffer: std::ptr::null_mut(),
        };

        let _ret = unsafe { binder_write_read(self.fd.as_raw_fd(), &mut data)? };

        // had consume
        if data.write_consumed > 0 {
            if data.write_consumed < buffer.data_size() {
                panic!(
                    "Driver did not consume write buffer. consumed: {} of {}",
                    data.write_consumed,
                    buffer.data_size()
                )
            } else {
                // yep remove data pos
                buffer.set_data_size(0);
            }
        }

        Ok(())
    }

    pub fn binder_read(&self, buffer: &mut Parcel) -> Result<()> {
        if buffer.capacity() == 0 {
            warn!("[BinderRead] Trying read driver with buffer capacity 0");
            return Ok(());
        }
        let mut data = BinderWriteRead {
            write_size: 0,
            write_consumed: 0,
            write_buffer: std::ptr::null(),
            read_size: buffer.capacity(),
            read_consumed: 0,
            read_buffer: buffer.as_mut_ptr(),
        };

        unsafe { binder_write_read(self.fd.as_raw_fd(), &mut data)? };

        info!(
            "[BinderRead] consumed: {}/{}",
            data.read_consumed, data.read_size
        );

        // set size for it then reset cursor for progress data
        buffer.set_data_size(data.read_consumed);
        buffer.set_data_position(0);

        Ok(())
    }

    pub fn enter_loop(&self) -> Result<()> {
        info!("[EnterLoopCmd]");
        let mut parcel = Parcel::default();
        parcel.write(&BinderCommand::EnterLooper)?;

        self.binder_write(&mut parcel)
    }

    pub fn exit_loop(&self) -> Result<()> {
        info!("[ExitLoopCmd]");
        let mut parcel = Parcel::default();
        parcel.write(&BinderCommand::ExitLooper)?;
        self.binder_write(&mut parcel)
    }

    pub fn binder_parse<F>(&self, parcel: &mut Parcel, mut handler: F) -> Result<bool>
    where
        F: FnMut(&Binder, BinderReturn, &mut Parcel) -> Result<bool>,
    {
        let mut handler_progressed = false;

        while parcel.has_unread_data() {
            // info!(
            //     "[BinderParse] Data unread left: {} (data size: {})",
            //     parcel.unread_data_size(),
            //     parcel.data_size()
            // );
            let cmd_value = parcel.read::<u32>()?;

            let cmd = BinderReturn::from_u32(cmd_value);

            if cmd.is_none() {
                warn!("Unknown BinderReturn value: {cmd_value}");
                continue;
            }

            let cmd = cmd.unwrap();
            info!("[BinderParse] Got cmd: {cmd:#?}");

            // if handler success handle this
            // we move to another
            if !handler_progressed {
                match handler(self, cmd, parcel) {
                    Ok(ret) => {
                        if ret {
                            handler_progressed = true;
                            continue;
                        }
                    }
                    Err(e) => {
                        error!("[BinderParse] handler: {e}");
                        return Err(e);
                    }
                }
            }

            // fallback to default handler

            match cmd {
                BinderReturn::Error => {
                    error!("[BinderParse] BR_Error: {}", parcel.read::<i32>()?);
                }
                BinderReturn::Ok => {}
                BinderReturn::Transaction | BinderReturn::Reply => {
                    let tx = parcel.read::<BinderTransactionData>()?;
                    info!("[BinderParse] Transaction data: \n{tx:#?}");
                }
                BinderReturn::AcquireResult => {
                    info!("[BinderParse] AcquireResult: {}", parcel.read::<i32>()?);
                }
                BinderReturn::DeadReply => {
                    panic!("[BinderParse] Got a DEAD_REPLY");
                }
                BinderReturn::TransactionComplete => {}
                BinderReturn::IncRefs => {
                    // 4 * 4 = 16
                    let ptr = parcel.read::<usize>()?;
                    info!("[BinderParse] IncRefs: {ptr:#X}");
                    let _ptr = parcel.read::<usize>()?;
                }
                BinderReturn::Acquire => {
                    // 4 * 4 = 16
                    let ptr = parcel.read::<usize>()?;
                    info!("[BinderParse] Acquire: {ptr:#X}");
                    let _ptr = parcel.read::<usize>()?;
                }
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

        Ok(handler_progressed)
    }

    pub fn transaction(
        &self,
        handle: u32,
        code: u32,
        flags: TransactionFlag,
        data: &mut Parcel,
    ) -> Result<()> {
        let mut parcel = Parcel::default();

        let transaction_data_out = BinderTransactionData {
            target: TargetUnion::new_handle(handle),
            code,
            flags: TransactionFlag::AcceptFds | flags,
            cookie: std::ptr::null_mut(),
            sender_pid: 0,
            sender_euid: 0,
            data_size: data.data_size(),
            offsets_size: (data.objects.len() * size_of::<usize>()),
            data: data.as_ptr() as _,
            offsets: data.objects.as_ptr() as _,
        };

        info!("[Transaction]\n{transaction_data_out:#?}");

        parcel.write(&BinderCommand::Transaction)?;
        parcel.write_aligned(&transaction_data_out);
        self.binder_write(&mut parcel)
    }

    pub fn transaction_with_parse<F>(
        &self,
        handle: u32,
        code: u32,
        flags: TransactionFlag,
        data: &mut Parcel,
        mut handler: F,
    ) -> Result<()>
    where
        F: FnMut(&Binder, BinderReturn, &mut Parcel) -> Result<bool>,
    {
        self.transaction(handle, code, flags, data)?;

        let mut parcel = Parcel::with_capacity(32 * 8);

        loop {
            info!("[TransactionWithParse] Looping");
            self.binder_read(&mut parcel)?;
            match self.binder_parse(&mut parcel, &mut handler) {
                Ok(progressed) => {
                    if progressed {
                        info!("Progressed");
                        break;
                    }
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
        Ok(())
    }

    pub fn reply(&self, data: &mut Parcel, flags: TransactionFlag) -> Result<()> {
        let mut parcel = Parcel::default();

        let transaction_data_out = BinderTransactionData {
            target: TargetUnion {
                ptr: 0xffffffff as usize as _,
            },
            code: Transaction::None.into(),
            flags,
            cookie: std::ptr::null_mut(),
            sender_pid: 0,
            sender_euid: 0,
            data_size: data.data_size(),
            offsets_size: (data.objects.len() * size_of::<usize>()),
            data: data.as_ptr() as _,
            offsets: data.objects.as_ptr() as _,
        };

        parcel.write(&BinderCommand::Reply)?;
        parcel.write_aligned(&transaction_data_out);
        self.binder_write(&mut parcel)
    }
}
