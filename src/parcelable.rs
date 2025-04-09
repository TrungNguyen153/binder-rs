use crate::{error::BinderResult, parcel::Parcel};

pub trait Parcelable: std::fmt::Debug {
    fn deserialize(parcel: &mut Parcel) -> BinderResult<Self>
    where
        Self: Sized;

    fn serialize(&self, parcel: &mut Parcel) -> BinderResult<()>;
}
