use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::FromPrimitive;

use crate::{_io, _ior, _iow, error::BinderError, parcelable::Parcelable};

const BC_TRANSACTION: u32 = _iow!(b'c', 0, 0x40);
const BC_REPLY: u32 = _iow!(b'c', 1, 0x40);
const BC_ACQUIRE_RESULT: u32 = _iow!(b'c', 2, 0x4);
const BC_FREE_BUFFER: u32 = _iow!(b'c', 3, 0x8);
const BC_INCREFS: u32 = _iow!(b'c', 4, 0x4);
const BC_ACQUIRE: u32 = _iow!(b'c', 5, 0x4);
const BC_RELEASE: u32 = _iow!(b'c', 6, 0x4);
const BC_DECREFS: u32 = _iow!(b'c', 7, 0x4);
const BC_INCREFS_DONE: u32 = _iow!(b'c', 8, 0x10);
const BC_ACQUIRE_DONE: u32 = _iow!(b'c', 9, 0x10);
const BC_ATTEMPT_ACQUIRE: u32 = _iow!(b'c', 10, 0x10);
const BC_REGISTER_LOOPER: u32 = _io!(b'c', 11);
const BC_ENTER_LOOPER: u32 = _io!(b'c', 12);
const BC_EXIT_LOOPER: u32 = _io!(b'c', 13);
const BC_REQUEST_DEATH_NOTIFICATION: u32 = _iow!(b'c', 14, 0xc);
const BC_CLEAR_DEATH_NOTIFICATION: u32 = _iow!(b'c', 15, 0x0c);
const BC_DEAD_BINDER_DONE: u32 = _iow!(b'c', 16, 0x8);
const BC_TRANSACTION_SG: u32 = _iow!(b'c', 17, 0x48);
const BC_REPLY_SG: u32 = _iow!(b'c', 18, 0x48);

#[derive(Debug, Clone, Copy, FromPrimitive, ToPrimitive)]
#[repr(u32)]
pub enum BinderCommand {
    Transaction = BC_TRANSACTION,
    Reply = BC_REPLY,
    AcquireResult = BC_ACQUIRE_RESULT,
    FreeBuffer = BC_FREE_BUFFER,
    IncRefs = BC_INCREFS,
    Acquire = BC_ACQUIRE,
    Release = BC_RELEASE,
    DecRefs = BC_DECREFS,
    IncRefsDone = BC_INCREFS_DONE,
    AcquireDone = BC_ACQUIRE_DONE,
    AttemptAcquire = BC_ATTEMPT_ACQUIRE,
    RegisterLooper = BC_REGISTER_LOOPER,
    EnterLooper = BC_ENTER_LOOPER,
    ExitLooper = BC_EXIT_LOOPER,
    RequestDeathNotification = BC_REQUEST_DEATH_NOTIFICATION,
    ClearDeathNotification = BC_CLEAR_DEATH_NOTIFICATION,
    DeadBinderDone = BC_DEAD_BINDER_DONE,
    TransactionSG = BC_TRANSACTION_SG,
    ReplySG = BC_REPLY_SG,
}

impl Parcelable for BinderCommand {
    fn deserialize(parcel: &mut crate::parcel::Parcel) -> crate::error::BinderResult<Self>
    where
        Self: Sized,
    {
        let v = parcel.read_u32()?;
        match BinderCommand::from_u32(v) {
            Some(b) => Ok(b),
            None => Err(BinderError::FailedParseParcel(format!(
                "BinderCommand: {v}"
            ))),
        }
    }

    fn serialize(&self, parcel: &mut crate::parcel::Parcel) -> crate::error::BinderResult<()> {
        parcel.write_u32(*self as u32)
    }
}

const BR_ERROR: u32 = _ior!(b'r', 0, 4);
const BR_OK: u32 = _ior!(b'r', 1, 0);
const BR_TRANSACTION: u32 = _ior!(b'r', 2, 0x40);
const BR_REPLY: u32 = _ior!(b'r', 3, 0x40);
const BR_ACQUIRE_RESULT: u32 = _ior!(b'r', 4, 0x4);
const BR_DEAD_REPLY: u32 = _io!(b'r', 5);
const BR_TRANSACTION_COMPLETE: u32 = _io!(b'r', 6);
const BR_INCREFS: u32 = _ior!(b'r', 7, 0x10);
const BR_ACQUIRE: u32 = _ior!(b'r', 8, 0x10);
const BR_RELEASE: u32 = _ior!(b'r', 9, 0x10);
const BR_DECREFS: u32 = _ior!(b'r', 10, 0x10);
const BR_ATTEMPT_ACQUIRE: u32 = _ior!(b'r', 11, 0xc);
const BR_NOOP: u32 = _io!(b'r', 12);
const BR_SPAWN_LOOPER: u32 = _io!(b'r', 13);
const BR_FINISHED: u32 = _io!(b'r', 14);
const BR_DEAD_BINDER: u32 = _ior!(b'r', 15, 0x8);
const BR_CLEAR_DEATH_NOTIFICATION_DONE: u32 = _ior!(b'r', 16, 0x8);
const BR_FAILED_REPLY: u32 = _io!(b'r', 17);
const BR_FROZEN_REPLY: u32 = _io!(b'r', 18);
const BR_ONEWAY_SPAM_SUSPECT: u32 = _io!(b'r', 19);

#[repr(u32)]
#[derive(Debug, Clone, Copy, FromPrimitive, ToPrimitive)]
pub enum BinderReturn {
    Error = BR_ERROR,
    Ok = BR_OK,
    Transaction = BR_TRANSACTION,
    Reply = BR_REPLY,
    AcquireResult = BR_ACQUIRE_RESULT,
    DeadReply = BR_DEAD_REPLY,
    TransactionComplete = BR_TRANSACTION_COMPLETE,
    IncRefs = BR_INCREFS,
    Acquire = BR_ACQUIRE,
    Release = BR_RELEASE,
    DecRefs = BR_DECREFS,
    AttemptAcquire = BR_ATTEMPT_ACQUIRE,
    Noop = BR_NOOP,
    SpawnLooper = BR_SPAWN_LOOPER,
    Finished = BR_FINISHED,
    DeadBinder = BR_DEAD_BINDER,
    ClearDeathNotification = BR_CLEAR_DEATH_NOTIFICATION_DONE,
    FailedReply = BR_FAILED_REPLY,
    FrozenReply = BR_FROZEN_REPLY,
    OnwaySpamSuspect = BR_ONEWAY_SPAM_SUSPECT,
}

impl Parcelable for BinderReturn {
    fn deserialize(parcel: &mut crate::parcel::Parcel) -> crate::error::BinderResult<Self>
    where
        Self: Sized,
    {
        let v = parcel.read_u32()?;
        match BinderReturn::from_u32(v) {
            Some(b) => Ok(b),
            None => Err(BinderError::FailedParseParcel(format!("BinderReturn: {v}"))),
        }
    }

    fn serialize(&self, parcel: &mut crate::parcel::Parcel) -> crate::error::BinderResult<()> {
        parcel.write_u32(*self as u32)
    }
}
