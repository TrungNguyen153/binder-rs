use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::FromPrimitive;

use crate::{
    error::BinderError,
    pack_chars,
    parcel::parcelable::{Deserialize, Serialize},
};

const BINDER_TYPE_LARGE: u8 = 0x85;

const TF_BINDER: u32 = pack_chars!(b's', b'b', b'*', BINDER_TYPE_LARGE);
const TF_WEAKBINDER: u32 = pack_chars!(b'w', b'b', b'*', BINDER_TYPE_LARGE);
const TF_HANDLE: u32 = pack_chars!(b's', b'h', b'*', BINDER_TYPE_LARGE);
const TF_WEAKHANDLE: u32 = pack_chars!(b'w', b'h', b'*', BINDER_TYPE_LARGE);
const TF_FD: u32 = pack_chars!(b'f', b'd', b'*', BINDER_TYPE_LARGE);
const TF_FDA: u32 = pack_chars!(b'f', b'd', b'a', BINDER_TYPE_LARGE);
const TF_PTR: u32 = pack_chars!(b'p', b't', b'*', BINDER_TYPE_LARGE);

#[derive(Debug, Hash, Clone, Copy, PartialEq, FromPrimitive, ToPrimitive)]
#[repr(u32)]
pub enum BinderType {
    Binder = TF_BINDER,
    WeakBinder = TF_WEAKBINDER,
    Handle = TF_HANDLE,
    WeakHandle = TF_WEAKHANDLE,
    Fd = TF_FD,
    Fda = TF_FDA,
    Ptr = TF_PTR,
}

impl Serialize for BinderType {
    fn serialize(&self, parcel: &mut crate::parcel::Parcel) -> crate::error::Result<()> {
        u32::from_u32(*self as _).serialize(parcel)
    }
}

impl Deserialize for BinderType {
    fn deserialize(parcel: &mut crate::parcel::Parcel) -> crate::error::Result<Self> {
        let v = <u32>::deserialize(parcel)?;
        match BinderType::from_u32(v) {
            Some(b) => Ok(b),
            None => Err(BinderError::FailedParseParcel(format!("BinderType: {v}"))),
        }
    }
}
