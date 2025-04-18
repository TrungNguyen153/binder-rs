#[derive(Debug, thiserror::Error)]
pub enum BinderError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    SliceError(#[from] std::array::TryFromSliceError),
    #[error(transparent)]
    NixError(#[from] nix::Error),
    #[error(transparent)]
    Utf16Error(#[from] std::string::FromUtf16Error),
    #[error(transparent)]
    Utf8Error(#[from] std::string::FromUtf8Error),
    #[error("Failed parse parcel: {0}")]
    FailedParseParcel(String),
    #[error("Bad value")]
    BadValue,
    #[error("UnexpectedNull")]
    UnexpectedNull,
    #[error("NotEnoughData")]
    NotEnoughData,
    #[error("BadType")]
    BadType,
    #[error("InvalidOperation")]
    InvalidOperation,
}

pub type Result<T> = std::result::Result<T, BinderError>;
