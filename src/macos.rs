use crate::DisplayInfo;
use anyhow::{anyhow, Result};
use core_graphics::display::{CGDirectDisplayID, CGDisplay, CGError, CGPoint, CGRect};

pub type ScreenRawHandle = CGDisplay;

impl DisplayInfo {
    fn new(id: CGDirectDisplayID) -> Self {
        let cg_display = CGDisplay::new(id);
        let CGRect { origin, size } = cg_display.bounds();

        let rotation = cg_display.rotation() as f32;
        let (scale_factor, frequency) = cg_display
            .display_mode()
            .map(|display_mode| {
                let pixel_width = display_mode.pixel_width();
                let scale_factor = pixel_width as f32 / size.width as f32;
                let refresh_rate = display_mode.refresh_rate() as f32;

                (scale_factor, refresh_rate)
            })
            .unwrap_or((1.0, 0.0));

        DisplayInfo {
            id,
            name: format!("Display {id}"),
            raw_handle: cg_display,
            x: origin.x as i32,
            y: origin.y as i32,
            width: size.width as u32,
            height: size.height as u32,
            rotation,
            frequency,
            scale_factor,
            is_primary: cg_display.is_main(),
        }
    }
}

pub fn get_all() -> Result<Vec<DisplayInfo>> {
    let display_ids =
        CGDisplay::active_displays().map_err(|e| anyhow!("Get active displays error: {}", e))?;

    let mut display_infos: Vec<DisplayInfo> = Vec::with_capacity(display_ids.len());

    for display_id in display_ids {
        display_infos.push(DisplayInfo::new(display_id));
    }

    Ok(display_infos)
}

pub fn get_from_point(x: i32, y: i32) -> Result<DisplayInfo> {
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

    if cg_error != 0 || display_count == 0 {
        return Err(anyhow!("Get displays with point failed"));
    }

    if let Some(&display_id) = display_ids.first() {
        Ok(DisplayInfo::new(display_id))
    } else {
        Err(anyhow!("Display not found"))
    }
}

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGGetDisplaysWithPoint(
        point: CGPoint,
        max_displays: u32,
        displays: *mut CGDirectDisplayID,
        display_count: *mut u32,
    ) -> CGError;
}
