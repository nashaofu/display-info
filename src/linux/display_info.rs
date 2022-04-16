use crate::DisplayInfo;
use std::{ptr, slice};
use x11::{
  xlib::{XCloseDisplay, XDefaultRootWindow, XOpenDisplay},
  xrandr::{
    XRRFreeCrtcInfo, XRRFreeMonitors, XRRFreeScreenResources, XRRGetCrtcInfo, XRRGetMonitors,
    XRRGetScreenResourcesCurrent,
  },
};

impl DisplayInfo {
  pub fn all() -> Vec<DisplayInfo> {
    unsafe {
      let display_ptr = XOpenDisplay(ptr::null_mut());

      if display_ptr.is_null() {
        return vec![];
      }

      let window_id = XDefaultRootWindow(display_ptr);

      let mut n_monitors = 0;
      let xrr_monitor_info_ptr = XRRGetMonitors(display_ptr, window_id, 1, &mut n_monitors);
      let xrr_monitor_infos = slice::from_raw_parts_mut(xrr_monitor_info_ptr, n_monitors as usize);

      let xrr_screen_resources_ptr = XRRGetScreenResourcesCurrent(display_ptr, window_id);
      let xrr_screen_resources = *xrr_screen_resources_ptr;
      let crtcs = slice::from_raw_parts_mut(xrr_screen_resources.crtcs, n_monitors as usize);

      let mut display_infos = Vec::new();

      for i in 0..n_monitors {
        if let Some(xrr_monitor_info) = xrr_monitor_infos.get(i as usize) {
          let xrr_crtc_info =
            XRRGetCrtcInfo(display_ptr, xrr_screen_resources_ptr, crtcs[i as usize]);

          let outputs =
            slice::from_raw_parts_mut(xrr_monitor_info.outputs, xrr_monitor_info.noutput as usize);

          let rotation = match (*xrr_crtc_info).rotation {
            8 => 90.0,
            2 => 270.0,
            4 => 180.0,
            _ => 0.0,
          };

          let display_info = DisplayInfo {
            id: outputs[0] as u32,
            x: xrr_monitor_info.x,
            y: xrr_monitor_info.y,
            width: xrr_monitor_info.width as u32,
            height: xrr_monitor_info.height as u32,
            scale: 1.0,
            rotation,
          };

          XRRFreeCrtcInfo(xrr_crtc_info);

          display_infos.push(display_info);
        }
      }

      XRRFreeMonitors(xrr_monitor_info_ptr);
      XRRFreeScreenResources(xrr_screen_resources_ptr);
      XCloseDisplay(display_ptr);

      display_infos
    }
  }

  pub fn from_point(x: i32, y: i32) -> Option<DisplayInfo> {
    let display_infos = DisplayInfo::all();
    let display_info = display_infos.iter().find(|&&display_info| {
      x >= display_info.x
        && x <= display_info.x + display_info.width as i32
        && y >= display_info.y
        && y <= display_info.y + display_info.height as i32
    });

    match display_info {
      Some(display_info) => Some(*display_info),
      None => None,
    }
  }
}
