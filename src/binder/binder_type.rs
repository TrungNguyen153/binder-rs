use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};

use crate::{error::BinderError, pack_chars, parcelable::Parcelable};

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

impl Parcelable for BinderType {
    fn deserialize(parcel: &mut crate::parcel::Parcel) -> crate::error::BinderResult<Self>
    where
        Self: Sized,
    {
        let v = parcel.read_u32()?;
        match BinderType::from_u32(v) {
            Some(b) => Ok(b),
            None => Err(BinderError::FailedParseParcel(format!("BinderType: {v}"))),
        }
    }

    fn serialize(&self, parcel: &mut crate::parcel::Parcel) -> crate::error::BinderResult<()> {
        parcel.write_u32(self.to_u32().unwrap())
    }
}
