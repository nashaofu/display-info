use anyhow::Result;

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
  pub fn all() -> Result<Vec<DisplayInfo>> {
    get_all()
  }

  pub fn from_point(x: i32, y: i32) -> Result<DisplayInfo> {
    get_from_point(x, y)
  }
}
