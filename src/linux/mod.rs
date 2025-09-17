use std::env::var_os;

pub use xorg::ScreenRawHandle;

use crate::{DisplayInfo, error::DIResult};

mod wayland;
mod xorg;

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

impl DisplayInfo {
    pub fn all() -> DIResult<Vec<DisplayInfo>> {
        if is_wayland() {
            wayland::get_all()
        } else {
            xorg::get_all()
        }
    }

    pub fn from_point(x: i32, y: i32) -> DIResult<DisplayInfo> {
        if is_wayland() {
            wayland::get_from_point(x, y)
        } else {
            xorg::get_from_point(x, y)
        }
    }
}
