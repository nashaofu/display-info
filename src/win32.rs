use crate::DisplayInfo;
use sfhash::digest;
use std::{mem, ptr};
use widestring::U16CString;
use windows::{
  core::PCWSTR,
  Win32::{
    Foundation::{BOOL, LPARAM, POINT, RECT},
    Graphics::Gdi::{
      CreateDCW, CreatedHDC, DeleteDC, EnumDisplayMonitors, EnumDisplaySettingsExW, GetDeviceCaps,
      GetMonitorInfoW, MonitorFromPoint, DESKTOPHORZRES, DEVMODEW, ENUM_CURRENT_SETTINGS,
      GET_DEVICE_CAPS_INDEX, HDC, HMONITOR, HORZRES, MONITORINFOEXW, MONITOR_DEFAULTTONULL,
    },
  },
};

impl DisplayInfo {
  fn new(monitor_info_exw: &MONITORINFOEXW) -> Self {
    let sz_device = monitor_info_exw.szDevice.as_ptr();

    let sz_device_string = unsafe { U16CString::from_ptr_str(sz_device).to_string_lossy() };
    let rc_monitor = monitor_info_exw.monitorInfo.rcMonitor;
    let dw_flags = monitor_info_exw.monitorInfo.dwFlags;

    DisplayInfo {
      id: digest(sz_device_string.as_bytes()),
      x: rc_monitor.left,
      y: rc_monitor.top,
      width: (rc_monitor.right - rc_monitor.left) as u32,
      height: (rc_monitor.bottom - rc_monitor.top) as u32,
      rotation: get_rotation(sz_device).unwrap_or(0.0),
      scale_factor: get_scale_factor(sz_device),
      is_primary: dw_flags == 1u32,
    }
  }
}

struct CreatedHDCBox(CreatedHDC);

impl Drop for CreatedHDCBox {
  fn drop(&mut self) {
    unsafe {
      DeleteDC(self.0);
    };
  }
}

impl CreatedHDCBox {
  fn new(sz_device: *const u16) -> Self {
    let h_dc = unsafe {
      CreateDCW(
        PCWSTR(sz_device),
        PCWSTR(sz_device),
        PCWSTR(ptr::null()),
        ptr::null(),
      )
    };

    CreatedHDCBox(h_dc)
  }

  fn get_device_caps(&self, index: GET_DEVICE_CAPS_INDEX) -> i32 {
    unsafe { GetDeviceCaps(self.0, index) }
  }
}

fn get_monitor_info_exw(h_monitor: HMONITOR) -> Option<MONITORINFOEXW> {
  let mut monitor_info_exw: MONITORINFOEXW = unsafe { mem::zeroed() };
  monitor_info_exw.monitorInfo.cbSize = mem::size_of::<MONITORINFOEXW>() as u32;
  let monitor_info_exw_ptr = <*mut _>::cast(&mut monitor_info_exw);

  unsafe { GetMonitorInfoW(h_monitor, monitor_info_exw_ptr).ok()? };

  Some(monitor_info_exw)
}

fn get_rotation(sz_device: *const u16) -> Option<f32> {
  let mut dev_modew: DEVMODEW = DEVMODEW::default();
  dev_modew.dmSize = mem::size_of::<DEVMODEW>() as u16;
  let dev_modew_ptr = <*mut _>::cast(&mut dev_modew);

  unsafe {
    EnumDisplaySettingsExW(PCWSTR(sz_device), ENUM_CURRENT_SETTINGS, dev_modew_ptr, 0).ok()?;
  };

  let dm_display_orientation = unsafe { dev_modew.Anonymous1.Anonymous2.dmDisplayOrientation };

  let rotation = match dm_display_orientation {
    0 => 0.0,
    1 => 90.0,
    2 => 180.0,
    3 => 270.0,
    _ => 0.0,
  };

  Some(rotation)
}

fn get_scale_factor(sz_device: *const u16) -> f32 {
  let wh_dc = CreatedHDCBox::new(sz_device);
  let logical_width = wh_dc.get_device_caps(HORZRES);
  let physical_width = wh_dc.get_device_caps(DESKTOPHORZRES);

  physical_width as f32 / logical_width as f32
}

pub fn get_all() -> Option<Vec<DisplayInfo>> {
  let h_monitors_mut_ptr = Box::into_raw(Box::new(Vec::new()));

  unsafe {
    EnumDisplayMonitors(
      HDC::default(),
      ptr::null_mut(),
      Some(monitor_enum_proc),
      LPARAM(h_monitors_mut_ptr as isize),
    )
    .ok()?
  };

  let h_monitors = unsafe { Box::from_raw(h_monitors_mut_ptr) };

  let display_infos = h_monitors
    .iter()
    .map(|monitor_info_exw| DisplayInfo::new(monitor_info_exw))
    .collect::<Vec<DisplayInfo>>();

  Some(display_infos)
}

pub fn get_from_point(x: i32, y: i32) -> Option<DisplayInfo> {
  let point = POINT { x, y };
  let h_monitor = unsafe { MonitorFromPoint(point, MONITOR_DEFAULTTONULL) };

  if h_monitor.is_invalid() {
    return None;
  }

  let monitor_info_exw = get_monitor_info_exw(h_monitor)?;

  Some(DisplayInfo::new(&monitor_info_exw))
}

extern "system" fn monitor_enum_proc(
  h_monitor: HMONITOR,
  _: HDC,
  _: *mut RECT,
  state: LPARAM,
) -> BOOL {
  unsafe {
    let state = Box::leak(Box::from_raw(state.0 as *mut Vec<MONITORINFOEXW>));

    match get_monitor_info_exw(h_monitor) {
      Ok(monitor_info_exw) => {
        state.push(monitor_info_exw);
        BOOL::from(true)
      }
      Err(_) => BOOL::from(false),
    }
  }
}
