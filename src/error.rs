use thiserror::Error;

#[cfg(target_os = "windows")]
#[derive(Debug, Error)]
pub enum DIError {
    #[error("{0}")]
    Error(String),
    #[error(transparent)]
    WindowsCoreError(#[from] windows::core::Error),
    #[error(transparent)]
    Utf16Error(#[from] widestring::error::Utf16Error),
}

impl DIError {
    pub fn new<S: ToString>(err: S) -> Self {
        DIError::Error(err.to_string())
    }
}

pub type DIResult<T> = Result<T, DIError>;
