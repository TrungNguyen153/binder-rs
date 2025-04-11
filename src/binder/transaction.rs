use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

use crate::{error::BinderError, pack_chars, parcelable::Parcelable};

const PING_TRANSCATION: u32 = pack_chars!(b'_', b'P', b'N', b'G');
const DUMP_TRANSACTION: u32 = pack_chars!(b'_', b'D', b'M', b'P');
const SHELL_COMMAND_TRANSACTION: u32 = pack_chars!(b'_', b'C', b'M', b'D');
const INTERFACE_TRANSACTION: u32 = pack_chars!(b'_', b'N', b'T', b'F');
const SYSPROPS_TRANSACTION: u32 = pack_chars!(b'_', b'S', b'P', b'R');
const EXTENSION_TRANSACTION: u32 = pack_chars!(b'_', b'E', b'X', b'T');
const DEBUG_PID_TRANSACTION: u32 = pack_chars!(b'_', b'P', b'I', b'D');
const TWEET_TRANSACTION: u32 = pack_chars!(b'_', b'T', b'W', b'T');
const LIKE_TRANSACTION: u32 = pack_chars!(b'_', b'L', b'I', b'K');

#[repr(u32)]
#[derive(Debug, Clone, Copy, FromPrimitive)]
pub enum Transaction {
    None = 0,
    FirstCall = 1,
    LastCall = 0xffffff,
    Ping = PING_TRANSCATION,
    Dump = DUMP_TRANSACTION,
    ShellCommand = SHELL_COMMAND_TRANSACTION,
    Interface = INTERFACE_TRANSACTION,
    Sysprops = SYSPROPS_TRANSACTION,
    Extension = EXTENSION_TRANSACTION,
    DebugPid = DEBUG_PID_TRANSACTION,
    Tweet = TWEET_TRANSACTION,
    Like = LIKE_TRANSACTION,
}

impl Into<u32> for Transaction {
    fn into(self) -> u32 {
        self as _
    }
}

impl Parcelable for Transaction {
    fn deserialize(parcel: &mut crate::parcel::Parcel) -> crate::error::BinderResult<Self>
    where
        Self: Sized,
    {
        let v = parcel.read_u32()?;
        match Transaction::from_u32(v) {
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

bitflags::bitflags! {
    #[derive(Debug)]
    pub struct TransactionFlag: u32 {
        const OneWay = 1;
        const CollectNotedAppOps = 2;
        const RootObject = 4;
        const StatusCode = 8;
        const AcceptFds = 0x10;
        const ClearBuf = 0x20;
    }
}
