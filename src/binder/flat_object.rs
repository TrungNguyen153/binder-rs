use std::os::fd::{FromRawFd, OwnedFd};

use crate::parcelable::Parcelable;

use super::binder_type::BinderType;

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

#[derive(Debug)]
pub struct BinderFlatObject {
    pub(crate) binder_type: BinderType,
    flags: u32,
    pub(crate) data: UnionFlatObject,
    cookie: usize,
    stability: u32, // stability  == SYSTEM
}

impl BinderFlatObject {
    pub fn new(binder_type: BinderType, handle: usize, cookie: usize, flags: u32) -> Self {
        Self {
            binder_type,
            flags,
            data: UnionFlatObject { binder: handle },
            cookie,
            stability: 0xc,
        }
    }

    pub unsafe fn ref_from_raw(ptr: *const u8, offset: usize) -> &'static Self {
        unsafe { std::mem::transmute(&*ptr.add(offset)) }
    }

    pub unsafe fn mut_from_raw(ptr: *mut u8, offset: usize) -> &'static mut Self {
        unsafe { std::mem::transmute(&*ptr.add(offset)) }
    }

    pub(crate) fn handle(&self) -> u32 {
        unsafe { self.data.handle }
    }

    pub(crate) fn set_handle(&mut self, handle: u32) {
        unsafe { self.data.handle = handle }
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

        Some(unsafe { OwnedFd::from_raw_fd(self.data.handle as _) })
    }

    pub fn set_cookie(&mut self, cookie: usize) {
        self.cookie = cookie;
    }
}

impl Parcelable for BinderFlatObject {
    fn deserialize(parcel: &mut crate::parcel::Parcel) -> crate::error::Result<Self>
    where
        Self: Sized,
    {
        Ok(Self {
            binder_type: BinderType::deserialize(parcel)?,
            flags: u32::deserialize(parcel)?,
            data: usize::deserialize(parcel)?,
            cookie: usize::deserialize(parcel)?,
            stability: u32::deserialize(parcel)?,
        })
    }

    fn serialize(&self, parcel: &mut crate::parcel::Parcel) -> crate::error::Result<()> {
        parcel.push_object();
        self.binder_type.serialize(parcel)?;
        parcel.write_u32(self.flags)?;
        parcel.write_usize(self.data)?;
        parcel.write_usize(self.cookie)?;
        parcel.write_u32(self.stability)?;
        Ok(())
    }
}
