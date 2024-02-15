use std::env::var_os;

use anyhow::Result;

use crate::DisplayInfo;

mod wayland;
mod xorg;

pub use xorg::ScreenRawHandle;

fn is_wayland() -> bool {
    var_os("WAYLAND_DISPLAY")
        .or(var_os("XDG_SESSION_TYPE"))
        .is_some_and(|v| {
            v.to_str()
                .unwrap_or_default()
                .to_lowercase()
                .contains("wayland")
        })
}

pub fn get_all() -> Result<Vec<DisplayInfo>> {
    if is_wayland() {
        wayland::get_all()
    } else {
        xorg::get_all()
    }
}

pub fn get_from_point(x: i32, y: i32) -> Result<DisplayInfo> {
    if is_wayland() {
        wayland::get_from_point(x, y)
    } else {
        xorg::get_from_point(x, y)
    }
}
