#[derive(Debug, thiserror::Error)]
pub enum BinderError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    NixError(#[from] nix::Error),
    #[error("Failed parse parcel: {0}")]
    FailedParseParcel(String),
}

pub type BinderResult<T> = Result<T, BinderError>;
