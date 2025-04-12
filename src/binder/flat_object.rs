use std::os::fd::{BorrowedFd, FromRawFd, OwnedFd};

use crate::error::{BinderError, Result};

use super::binder_type::BinderType;
pub const FLAT_BINDER_FLAG_PRIORITY_MASK: u32 = 255;
pub const FLAT_BINDER_FLAG_ACCEPTS_FDS: u32 = 256;
#[derive(Clone, Copy)]
#[repr(C)]
pub union UnionFlatObject {
    pub handle: u32,
    pub binder: usize,
}

impl std::fmt::Debug for UnionFlatObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FlatObjectUnion {{\n")?;
        write!(f, "\t handle: {},\n", unsafe { self.handle })?;
        write!(f, "\t binder: {},\n", unsafe { self.binder })?;
        write!(f, "}}")
    }
}

impl Default for UnionFlatObject {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

#[repr(C, align(4))]
#[derive(Debug, Clone, Copy)]
pub struct BinderFlatObject {
    pub(crate) binder_type: BinderType,
    flags: u32,
    pub(crate) data: UnionFlatObject,
    cookie: usize,
}

impl Default for BinderFlatObject {
    fn default() -> Self {
        Self {
            binder_type: BinderType::Binder,
            flags: 0,
            data: UnionFlatObject { binder: 0 },
            cookie: 0,
        }
    }
}

impl BinderFlatObject {
    pub fn new_with_fd(raw_fd: i32, take_ownership: bool) -> Self {
        Self {
            binder_type: BinderType::Fd,
            flags: 0x7F & FLAT_BINDER_FLAG_ACCEPTS_FDS,
            data: UnionFlatObject {
                handle: raw_fd as _,
            },
            cookie: if take_ownership { 1 } else { 0 },
        }
    }

    pub unsafe fn ref_from_raw(ptr: *const u8, offset: usize) -> &'static Self {
        unsafe { std::mem::transmute(&*ptr.add(offset)) }
    }

    pub unsafe fn mut_from_raw(ptr: *mut u8, offset: usize) -> &'static mut Self {
        unsafe { std::mem::transmute(&mut *ptr.add(offset)) }
    }

    pub(crate) fn handle(&self) -> u32 {
        unsafe { self.data.handle }
    }

    pub(crate) fn set_handle(&mut self, handle: u32) {
        self.data.handle = handle
    }

    pub(crate) fn pointer(&self) -> usize {
        unsafe { self.data.binder }
    }

    pub(crate) fn cookie(&self) -> usize {
        self.cookie
    }

    pub(crate) fn header_type(&self) -> BinderType {
        self.binder_type
    }

    pub(crate) fn owned_fd(&self) -> Option<OwnedFd> {
        if self.binder_type != BinderType::Fd {
            return None;
        }

        Some(unsafe { OwnedFd::from_raw_fd(self.handle() as _) })
    }

    pub(crate) fn borrowed_fd(&self) -> Option<BorrowedFd> {
        if self.binder_type != BinderType::Fd {
            return None;
        }

        Some(unsafe { BorrowedFd::borrow_raw(self.handle() as _) })
    }

    pub fn set_cookie(&mut self, cookie: usize) {
        self.cookie = cookie;
    }

    pub(crate) fn acquire(&self) -> Result<()> {
        match self.binder_type {
            BinderType::Binder => {
                todo!()
                // if self.pointer() != 0 {
                //     let strong = raw_pointer_to_strong_binder((self.pointer(), self.cookie()));
                //     strong.increase()?;
                // }
                // Ok(())
            }
            BinderType::Handle => {
                todo!()
                // process_state::ProcessState::as_self()
                // .strong_proxy_for_handle(self.handle())?
                // .increase()
            }
            BinderType::Fd => {
                // Notion to do.
                Ok(())
            }
            _ => {
                error!("Invalid object type {:?}", self.binder_type);
                Err(BinderError::InvalidOperation)
            }
        }
    }

    pub(crate) fn release(&self) -> Result<()> {
        match self.binder_type {
            BinderType::Binder => {
                todo!()
                // if self.pointer() != 0 {
                //     let strong = raw_pointer_to_strong_binder((self.pointer(), self.cookie()));
                //     strong.decrease()?;
                // }
                // Ok(())
            }
            BinderType::Handle => {
                todo!()
                // process_state::ProcessState::as_self()
                // .strong_proxy_for_handle(self.handle())?
                // .decrease()
            }
            BinderType::Fd => {
                if self.cookie != 0 {
                    // Get owned fd and close it.
                    self.owned_fd();
                }

                Ok(())
            }
            _ => {
                error!("Invalid object type {:?}", self.binder_type);
                Err(BinderError::InvalidOperation)
            }
        }
    }
}
