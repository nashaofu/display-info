//! # example
//! Get all display info
//! ```
//! use display_info::DisplayInfo;
//! use std::time::Instant;
//!
//! let start = Instant::now();
//!
//! let display_infos = DisplayInfo::all().unwrap();
//! for display_info in display_infos {
//!   println!("display_info {display_info:?}");
//! }
//! let display_info = DisplayInfo::from_point(100, 100).unwrap();
//! println!("display_info {display_info:?}");
//! println!("运行耗时: {:?}", start.elapsed());
//! ```

pub mod error;
use error::{DIError, DIResult};

#[cfg(all(target_family = "unix", not(target_os = "macos")))]
mod linux;
#[cfg(all(target_family = "unix", not(target_os = "macos")))]
use linux::ScreenRawHandle;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
use macos::ScreenRawHandle;

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
use windows::ScreenRawHandle;

#[derive(Debug, Clone)]
pub struct DisplayInfo {
    /// Unique identifier associated with the display.
    pub id: u32,
    /// The display name
    pub name: String,
    /// The display friendly name
    pub friendly_name: String,
    /// Native screen raw handle
    pub raw_handle: ScreenRawHandle,
    /// The display x coordinate.
    pub x: i32,
    /// The display x coordinate.
    pub y: i32,
    /// The display pixel width.
    pub width: u32,
    /// The display pixel height.
    pub height: u32,
    /// The width of a display in millimeters. This value may be 0.
    pub width_mm: i32,
    /// The height of a display in millimeters. This value may be 0.
    pub height_mm: i32,
    /// Can be 0, 90, 180, 270, represents screen rotation in clock-wise degrees.
    pub rotation: f32,
    /// Output device's pixel scale factor.
    pub scale_factor: f32,
    /// The display refresh rate.
    pub frequency: f32,
    /// Whether the screen is the main screen
    pub is_primary: bool,
}

impl DisplayInfo {
    pub fn from_name(name: impl ToString) -> DIResult<DisplayInfo> {
        let name = name.to_string();
        let display_infos = DisplayInfo::all()?;

        display_infos
            .iter()
            .find(|&d| d.name == name)
            .cloned()
            .ok_or_else(|| DIError::new("Get display info failed"))
    }
}
