use crate::DisplayInfo;
use core_graphics::display::{CGDirectDisplayID, CGDisplay, CGError, CGPoint, CGRect};

fn create_display_info(id: CGDirectDisplayID) -> DisplayInfo {
  let cg_display = CGDisplay::new(id);
  let CGRect { origin, size } = cg_display.bounds();
  let scale = match cg_display.display_mode() {
    Some(display_mode) => {
      let pixel_width = display_mode.pixel_width();

      (pixel_width as f32) / size.width as f32
    }
    None => 1.0,
  };

  let rotation = cg_display.rotation() as f32;

  DisplayInfo {
    id,
    x: origin.x as i32,
    y: origin.y as i32,
    width: size.width as u32,
    height: size.height as u32,
    scale,
    rotation,
    primary: cg_display.is_main(),
  }
}

pub fn get_all() -> Vec<DisplayInfo> {
  match CGDisplay::active_displays() {
    Ok(display_ids) => {
      let mut display_infos: Vec<DisplayInfo> = Vec::with_capacity(display_ids.len());

      for display_id in display_ids {
        display_infos.push(create_display_info(display_id));
      }

      display_infos
    }
    Err(_) => vec![],
  }
}

pub fn get_from_point(x: i32, y: i32) -> Option<DisplayInfo> {
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

  if cg_error != 0 {
    return None;
  }

  if display_count == 0 {
    return None;
  }
  
  let display_id = display_ids.get(0)?;

  Some(create_display_info(*display_id))
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
