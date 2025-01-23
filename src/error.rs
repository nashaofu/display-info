use std::sync::PoisonError;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Error(String),
    #[error("StdSyncPoisonError {0}")]
    StdSyncPoisonError(String),

    #[cfg(target_os = "linux")]
    #[error(transparent)]
    XcbError(#[from] xcb::Error),
    #[cfg(target_os = "linux")]
    #[error(transparent)]
    XcbConnError(#[from] xcb::ConnError),
    #[cfg(target_os = "linux")]
    #[error(transparent)]
    ImageImageError(#[from] image::ImageError),
    #[cfg(target_os = "linux")]
    #[error(transparent)]
    StdStrUtf8Error(#[from] std::str::Utf8Error),
    #[cfg(target_os = "linux")]
    #[error(transparent)]
    DbusError(#[from] dbus::Error),
    #[cfg(target_os = "linux")]
    #[error(transparent)]
    StdIOError(#[from] std::io::Error),
    #[cfg(target_os = "linux")]
    #[error(transparent)]
    StdTimeSystemTimeError(#[from] std::time::SystemTimeError),

    #[cfg(target_os = "macos")]
    #[error("CoreGraphicsDisplayCGError {0}")]
    CoreGraphicsDisplayCGError(core_graphics::display::CGError),

    #[cfg(target_os = "windows")]
    #[error(transparent)]
    WindowsCoreError(#[from] windows::core::Error),
    #[cfg(target_os = "windows")]
    #[error(transparent)]
    Utf16Error(#[from] widestring::error::Utf16Error),
}

impl Error {
    pub fn new<S: ToString>(err: S) -> Self {
        Error::Error(err.to_string())
    }
}

#[cfg(target_os = "macos")]
impl From<core_graphics::display::CGError> for Error {
    fn from(value: core_graphics::display::CGError) -> Self {
        Error::CoreGraphicsDisplayCGError(value)
    }
}

pub type XCapResult<T> = Result<T, Error>;

impl<T> From<PoisonError<T>> for Error {
    fn from(value: PoisonError<T>) -> Self {
        Error::StdSyncPoisonError(value.to_string())
    }
}
