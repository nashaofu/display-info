use crate::DisplayInfo;
use sfhash::digest;
use std::{mem, ptr};
use widestring::U16CString;
use windows::{
  core::PCWSTR,
  Win32::{
    Foundation::{BOOL, LPARAM, POINT, RECT},
    Graphics::Gdi::{
      CreateDCW, DeleteDC, EnumDisplayMonitors, EnumDisplaySettingsExW, GetDeviceCaps,
      GetMonitorInfoW, MonitorFromPoint, DESKTOPHORZRES, DEVMODEW, ENUM_CURRENT_SETTINGS, HDC,
      HMONITOR, HORZRES, MONITORINFOEXW, MONITOR_DEFAULTTONULL,
    },
  },
};

fn get_monitor_info_exw(h_monitor: HMONITOR) -> Option<MONITORINFOEXW> {
  unsafe {
    let mut monitor_info_exw: MONITORINFOEXW = mem::zeroed();
    monitor_info_exw.monitorInfo.cbSize = mem::size_of::<MONITORINFOEXW>() as u32;
    let monitor_info_exw_ptr = <*mut _>::cast(&mut monitor_info_exw);

    match GetMonitorInfoW(h_monitor, monitor_info_exw_ptr) {
      BOOL(0) => None,
      _ => Some(monitor_info_exw),
    }
  }
}

fn get_scale(sz_device: *const u16) -> f32 {
  unsafe {
    let h_dc = CreateDCW(
      PCWSTR(sz_device),
      PCWSTR(sz_device),
      PCWSTR(ptr::null()),
      ptr::null(),
    );
    let logical_width = GetDeviceCaps(h_dc, HORZRES);
    let physical_width = GetDeviceCaps(h_dc, DESKTOPHORZRES);

    DeleteDC(h_dc);

    physical_width as f32 / logical_width as f32
  }
}

fn get_display_rotation(sz_device: *const u16) -> f32 {
  unsafe {
    let mut dev_modew: DEVMODEW = DEVMODEW::default();
    dev_modew.dmSize = mem::size_of::<DEVMODEW>() as u16;
    let dev_modew_ptr = <*mut _>::cast(&mut dev_modew);

    let dm_display_orientation =
      match EnumDisplaySettingsExW(PCWSTR(sz_device), ENUM_CURRENT_SETTINGS, dev_modew_ptr, 0) {
        BOOL(0) => None,
        _ => Some(dev_modew.Anonymous1.Anonymous2.dmDisplayOrientation),
      };

    match dm_display_orientation {
      Some(1) => 90.0,
      Some(2) => 180.0,
      Some(3) => 270.0,
      _ => 0.0,
    }
  }
}

fn create_display_info(monitor_info_exw: MONITORINFOEXW) -> DisplayInfo {
  unsafe {
    let sz_device = monitor_info_exw.szDevice.as_ptr();
    let sz_device_string = U16CString::from_ptr_str(sz_device).to_string_lossy();
    let rc_monitor = monitor_info_exw.monitorInfo.rcMonitor;
    let dw_flags = monitor_info_exw.monitorInfo.dwFlags;

    DisplayInfo {
      id: digest(sz_device_string.as_bytes()),
      x: rc_monitor.left,
      y: rc_monitor.top,
      width: (rc_monitor.right - rc_monitor.left) as u32,
      height: (rc_monitor.bottom - rc_monitor.top) as u32,
      scale: get_scale(sz_device),
      rotation: get_display_rotation(sz_device),
      primary: dw_flags == 1u32,
    }
  }
}

pub fn get_all() -> Vec<DisplayInfo> {
  unsafe {
    let h_monitors = Box::into_raw(Box::new(Vec::<MONITORINFOEXW>::new()));
    match EnumDisplayMonitors(
      HDC::default(),
      ptr::null_mut(),
      Some(monitor_enum_proc),
      LPARAM(h_monitors as isize),
    ) {
      BOOL(0) => vec![],
      _ => {
        let display_infos = Box::from_raw(h_monitors)
          .iter()
          .map(|monitor_info_exw| create_display_info(*monitor_info_exw))
          .collect();

        display_infos
      }
    }
  }
}

pub fn get_from_point(x: i32, y: i32) -> Option<DisplayInfo> {
  let point = POINT { x, y };
  unsafe {
    let h_monitor = MonitorFromPoint(point, MONITOR_DEFAULTTONULL);

    if h_monitor.is_invalid() {
      return None;
    }

    match get_monitor_info_exw(h_monitor) {
      Some(monitor_info_exw) => Some(create_display_info(monitor_info_exw)),
      None => None,
    }
  }
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
      Some(monitor_info_exw) => {
        state.push(monitor_info_exw);
        BOOL::from(true)
      }
      None => BOOL::from(false),
    }
  }
}
