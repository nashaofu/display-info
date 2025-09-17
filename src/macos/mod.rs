use objc2::MainThreadMarker;
use objc2_app_kit::NSScreen;
use objc2_core_foundation::{CGPoint, CGRect};
use objc2_core_graphics::{
    CGDirectDisplayID, CGDisplayBounds, CGDisplayCopyDisplayMode, CGDisplayIsMain, CGDisplayMode,
    CGDisplayRotation, CGDisplayScreenSize, CGError, CGGetActiveDisplayList,
    CGGetDisplaysWithPoint,
};
use objc2_foundation::{NSNumber, NSString};

use crate::{
    DisplayInfo,
    error::{DIError, DIResult},
};

pub type ScreenRawHandle = CGDirectDisplayID;

fn get_display_friendly_name(display_id: CGDirectDisplayID) -> DIResult<String> {
    let screens = NSScreen::screens(unsafe { MainThreadMarker::new_unchecked() });
    for screen in screens {
        let device_description = screen.deviceDescription();
        let screen_number = device_description
            .objectForKey(&NSString::from_str("NSScreenNumber"))
            .ok_or(DIError::new("Get NSScreenNumber failed"))?;

        let screen_id = screen_number
            .downcast::<NSNumber>()
            .map_err(|err| DIError::new(format!("{:?}", err)))?
            .unsignedIntValue();

        if screen_id == display_id {
            unsafe { return Ok(screen.localizedName().to_string()) };
        }
    }

    Err(DIError::new(format!(
        "Get display {} friendly name failed",
        display_id
    )))
}

impl DisplayInfo {
    fn new(id: CGDirectDisplayID) -> DIResult<Self> {
        unsafe {
            let CGRect { origin, size } = CGDisplayBounds(id);

            let rotation = CGDisplayRotation(id) as f32;

            let display_mode = CGDisplayCopyDisplayMode(id);
            let pixel_width = CGDisplayMode::pixel_width(display_mode.as_deref());
            let scale_factor = pixel_width as f32 / size.width as f32;
            let frequency = CGDisplayMode::refresh_rate(display_mode.as_deref()) as f32;

            let size_mm = CGDisplayScreenSize(id);
            let is_primary = CGDisplayIsMain(id);

            Ok(DisplayInfo {
                id,
                name: format!("Display {id}"),
                friendly_name: get_display_friendly_name(id)
                    .unwrap_or(format!("Unknown Display {}", id)),
                raw_handle: id,
                x: origin.x as i32,
                y: origin.y as i32,
                width: size.width as u32,
                height: size.height as u32,
                width_mm: size_mm.width as i32,
                height_mm: size_mm.height as i32,
                rotation,
                frequency,
                scale_factor,
                is_primary,
            })
        }
    }

    pub fn all() -> DIResult<Vec<DisplayInfo>> {
        let max_displays: u32 = 16;
        let mut active_displays: Vec<CGDirectDisplayID> = vec![0; max_displays as usize];
        let mut display_count: u32 = 0;

        let cg_error = unsafe {
            CGGetActiveDisplayList(
                max_displays,
                active_displays.as_mut_ptr(),
                &mut display_count,
            )
        };

        if cg_error != CGError::Success {
            return Err(DIError::new(format!(
                "CGGetActiveDisplayList failed: {:?}",
                cg_error
            )));
        }

        active_displays.truncate(display_count as usize);

        let mut display_infos = Vec::with_capacity(active_displays.len());

        for display in active_displays {
            display_infos.push(DisplayInfo::new(display)?);
        }

        Ok(display_infos)
    }

    pub fn from_point(x: i32, y: i32) -> DIResult<DisplayInfo> {
        let point = CGPoint {
            x: x as f64,
            y: y as f64,
        };

        let max_displays: u32 = 16;
        let mut display_ids: Vec<CGDirectDisplayID> = vec![0; max_displays as usize];
        let mut display_count: u32 = 0;

        let cg_error = unsafe {
            CGGetDisplaysWithPoint(
                point,
                max_displays,
                display_ids.as_mut_ptr(),
                &mut display_count,
            )
        };

        if cg_error != CGError::Success {
            return Err(DIError::new(format!(
                "CGGetDisplaysWithPoint failed: {:?}",
                cg_error
            )));
        }

        if let Some(&display_id) = display_ids.first() {
            DisplayInfo::new(display_id)
        } else {
            Err(DIError::new("Display not found"))
        }
    }
}
