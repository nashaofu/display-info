use std::error::Error;

#[cfg(target_os = "macos")]
mod darwin;
#[cfg(target_os = "macos")]
use darwin::*;

#[cfg(target_os = "windows")]
mod win32;
#[cfg(target_os = "windows")]
use win32::*;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
use linux::*;

#[derive(Debug, Clone)]
pub struct DisplayInfoError(String);

impl<T: Error> From<T> for DisplayInfoError {
  fn from(err: T) -> DisplayInfoError {
    DisplayInfoError(err.to_string())
  }
}

impl DisplayInfoError {
  pub fn new(str: &str) -> Self {
    DisplayInfoError(str.to_string())
  }
}

#[derive(Debug, Clone, Copy)]
pub struct DisplayInfo {
  pub id: u32,
  pub x: i32,
  pub y: i32,
  pub width: u32,
  pub height: u32,
  pub rotation: f32,
  pub scale_factor: f32,
  pub is_primary: bool,
}

impl DisplayInfo {
  pub fn all() -> Result<Vec<DisplayInfo>, DisplayInfoError> {
    get_all()
  }

  pub fn from_point(x: i32, y: i32) -> Result<DisplayInfo, DisplayInfoError> {
    get_from_point(x, y)
  }
}
