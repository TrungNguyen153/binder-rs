use crate::parcelable::Parcelable;

use super::binder_type::BinderType;

#[derive(Debug)]
pub struct BinderFlatObject {
    pub(crate) binder_type: BinderType,
    flags: u32,
    pub(crate) handle: usize,
    cookie: usize,
    stability: u32, // stability  == SYSTEM
}

impl BinderFlatObject {
    pub fn new(binder_type: BinderType, handle: usize, cookie: usize, flags: u32) -> Self {
        Self {
            binder_type,
            flags,
            handle,
            cookie,
            stability: 0xc,
        }
    }
}

impl Parcelable for BinderFlatObject {
    fn deserialize(parcel: &mut crate::parcel::Parcel) -> crate::error::BinderResult<Self>
    where
        Self: Sized,
    {
        Ok(Self {
            binder_type: BinderType::deserialize(parcel)?,
            flags: u32::deserialize(parcel)?,
            handle: usize::deserialize(parcel)?,
            cookie: usize::deserialize(parcel)?,
            stability: u32::deserialize(parcel)?,
        })
    }

    fn serialize(&self, parcel: &mut crate::parcel::Parcel) -> crate::error::BinderResult<()> {
        parcel.push_object();
        self.binder_type.serialize(parcel)?;
        parcel.write_u32(self.flags)?;
        parcel.write_usize(self.handle)?;
        parcel.write_usize(self.cookie)?;
        parcel.write_u32(self.stability)?;
        Ok(())
    }
}
