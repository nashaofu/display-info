use thiserror::Error;

#[cfg(all(target_family = "unix", not(target_os = "macos")))]
#[derive(Debug, Error)]
pub enum DIError {
    #[error("{0}")]
    Error(String),
    #[error(transparent)]
    StdStrUtf8Error(#[from] std::str::Utf8Error),
    #[error(transparent)]
    XcbError(#[from] xcb::Error),
    #[error(transparent)]
    XcbConnError(#[from] xcb::ConnError),
    #[error(transparent)]
    SmithayClientToolkitClientDispatchError(
        #[from] smithay_client_toolkit::reexports::client::DispatchError,
    ),
    #[error(transparent)]
    SmithayClientToolkitClientConnectError(
        #[from] smithay_client_toolkit::reexports::client::ConnectError,
    ),
}

#[cfg(target_os = "macos")]
#[derive(Debug, Error)]
pub enum DIError {
    #[error("{0}")]
    Error(String),
}

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
