#[derive(Debug, thiserror::Error)]
pub enum BinderError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    NixError(#[from] nix::Error),
    #[error(transparent)]
    Utf16Error(#[from] std::string::FromUtf16Error),
    #[error(transparent)]
    Utf8Error(#[from] std::string::FromUtf8Error),
    #[error("Failed parse parcel: {0}")]
    FailedParseParcel(String),
}

pub type BinderResult<T> = Result<T, BinderError>;
