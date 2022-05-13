use crate::DisplayInfo;
use std::{ffi::CString, ptr, slice};
use x11::{
  xlib::{XDefaultRootWindow, XOpenDisplay, XResourceManagerString},
  xrandr::{
    XRRFreeCrtcInfo, XRRFreeScreenResources, XRRGetCrtcInfo, XRRGetOutputInfo,
    XRRGetScreenResourcesCurrent,
  },
};

pub fn get_all() -> Vec<DisplayInfo> {
  unsafe {
    let display_ptr = XOpenDisplay(ptr::null_mut());

    if display_ptr.is_null() {
      return vec![];
    }

    let window_id = XDefaultRootWindow(display_ptr);

    let screen_resources_ptr = XRRGetScreenResourcesCurrent(display_ptr, window_id);
    let screen_resources = *screen_resources_ptr;

    let noutput = screen_resources.noutput as usize;
    let outputs = slice::from_raw_parts(screen_resources.outputs, noutput);

    let resource_manager_string_cstring = CString::from_raw(XResourceManagerString(display_ptr));
    let resource_manager_string = resource_manager_string_cstring.to_string_lossy();

    let prefix = "Xft.dpi:\t";

    let xft_dpi = resource_manager_string
      .split("\n")
      .find(|str| str.starts_with(prefix))
      .map(|str| str.replace(prefix, ""))
      .map(|dpi| dpi.parse::<f32>().unwrap_or(96.0))
      .unwrap_or(96.0);

    let scale = xft_dpi / 96.0;

    let mut display_infos = Vec::new();

    for output in outputs.iter() {
      let output_info_ptr = XRRGetOutputInfo(display_ptr, screen_resources_ptr, *output);
      let output_info = *output_info_ptr;

      if output_info.connection != 0 {
        continue;
      }

      let crtc_info_ptr = XRRGetCrtcInfo(display_ptr, screen_resources_ptr, output_info.crtc);
      let crtc_info = *crtc_info_ptr;

      let rotation = match crtc_info.rotation {
        2 => 90.0,
        4 => 180.0,
        8 => 270.0,
        _ => 0.0,
      };

      let display_info = DisplayInfo {
        id: *output as u32,
        x: ((crtc_info.x as f32) / scale) as i32,
        y: ((crtc_info.y as f32) / scale) as i32,
        width: ((crtc_info.width as f32) / scale) as u32,
        height: ((crtc_info.height as f32) / scale) as u32,
        scale,
        rotation,
      };

      XRRFreeCrtcInfo(crtc_info_ptr);

      display_infos.push(display_info);
    }

    XRRFreeScreenResources(screen_resources_ptr);

    display_infos
  }
}

pub fn get_from_point(x: i32, y: i32) -> Option<DisplayInfo> {
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
