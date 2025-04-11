use crate::pack_chars;

/// The default maximum number of threads to support
pub const DEFAULT_MAX_BINDER_THREADS: u32 = 15;
pub const PAGE_SIZE: usize = 0x1000;
pub const BINDER_VM_SIZE: usize = (1 * 1024 * 1024) - PAGE_SIZE * 2;

pub const INTERFACE_HEADER: u32 = pack_chars!('S', 'Y', 'S', 'T');
