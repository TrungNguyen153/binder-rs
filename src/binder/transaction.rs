use num_derive::FromPrimitive;

use crate::pack_chars;

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
#[derive(Debug, FromPrimitive)]
pub enum Transaction {
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
