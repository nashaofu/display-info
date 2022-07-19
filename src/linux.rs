use crate::DisplayInfo;
use std::{
  ffi::{CStr, CString},
  os::raw::c_char,
  ptr, slice,
};

use x11::{
  xlib::{
    Display, XCloseDisplay, XDefaultRootWindow, XOpenDisplay, XResourceManagerString,
    XrmDestroyDatabase, XrmGetResource, XrmGetStringDatabase, XrmValue,
  },
  xrandr::{
    XRRFreeCrtcInfo, XRRFreeOutputInfo, XRRFreeScreenResources, XRRGetCrtcInfo, XRRGetOutputInfo,
    XRRGetOutputPrimary, XRRGetScreenResourcesCurrent,
  },
};

fn get_xft_dpi(display_ptr: *mut Display) -> f32 {
  unsafe {
    let resource_manager_string_ptr = XResourceManagerString(display_ptr);

    if resource_manager_string_ptr.is_null() {
      return 96.0;
    }

    let string_database_ptr = XrmGetStringDatabase(resource_manager_string_ptr);

    if string_database_ptr.is_null() {
      return 96.0;
    }

    let mut xrm_value = XrmValue {
      size: 0,
      addr: ptr::null_mut(),
    };

    let mut str_type: *mut c_char = ptr::null_mut();
    let str_name = CString::new("Xft.dpi").unwrap();
    let str_class = CString::new("Xft.Dpi").unwrap();

    let result = XrmGetResource(
      string_database_ptr,
      str_name.as_ptr(),
      str_class.as_ptr(),
      &mut str_type,
      &mut xrm_value,
    );

    XrmDestroyDatabase(string_database_ptr);

    if result == 0 || xrm_value.addr.is_null() {
      return 96.0;
    }

    CStr::from_ptr(xrm_value.addr)
      .to_str()
      .unwrap_or("96.0")
      .parse::<f32>()
      .unwrap_or(96.0)
  }
}

pub fn get_all() -> Vec<DisplayInfo> {
  let mut display_infos = Vec::new();

  unsafe {
    let display_ptr = XOpenDisplay(ptr::null_mut());

    if display_ptr.is_null() {
      return display_infos;
    }

    let window_id = XDefaultRootWindow(display_ptr);

    let screen_resources_ptr = XRRGetScreenResourcesCurrent(display_ptr, window_id);

    if screen_resources_ptr.is_null() {
      XCloseDisplay(display_ptr);
      return display_infos;
    }

    let screen_resources = *screen_resources_ptr;

    let noutput = screen_resources.noutput as usize;
    let outputs = slice::from_raw_parts(screen_resources.outputs, noutput);

    let xft_dpi = get_xft_dpi(display_ptr);

    let scale_factor = xft_dpi / 96.0;

    let primary_output = XRRGetOutputPrimary(display_ptr, window_id);

    for output in outputs.iter() {
      let output_info_ptr = XRRGetOutputInfo(display_ptr, screen_resources_ptr, *output);

      if output_info_ptr.is_null() {
        continue;
      }

      let output_info = *output_info_ptr;

      if output_info.connection != 0 {
        XRRFreeOutputInfo(output_info_ptr);
        continue;
      }

      let crtc_info_ptr = XRRGetCrtcInfo(display_ptr, screen_resources_ptr, output_info.crtc);

      if crtc_info_ptr.is_null() {
        XRRFreeOutputInfo(output_info_ptr);
        continue;
      }

      let crtc_info = *crtc_info_ptr;

      let rotation = match crtc_info.rotation {
        2 => 90.0,
        4 => 180.0,
        8 => 270.0,
        _ => 0.0,
      };

      let display_info = DisplayInfo {
        id: *output as u32,
        x: ((crtc_info.x as f32) / scale_factor) as i32,
        y: ((crtc_info.y as f32) / scale_factor) as i32,
        width: ((crtc_info.width as f32) / scale_factor) as u32,
        height: ((crtc_info.height as f32) / scale_factor) as u32,
        rotation,
        scale_factor,
        is_primary: primary_output == *output,
      };

      XRRFreeOutputInfo(output_info_ptr);
      XRRFreeCrtcInfo(crtc_info_ptr);

      display_infos.push(display_info);
    }

    XRRFreeScreenResources(screen_resources_ptr);
    XCloseDisplay(display_ptr);
  }

  display_infos
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
