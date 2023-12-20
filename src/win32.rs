use crate::DisplayInfo;
use anyhow::{anyhow, Result};
use fxhash::hash32;
use std::{mem, ops::Deref, ptr};
use widestring::U16CString;
use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::{BOOL, LPARAM, POINT, RECT},
        Graphics::Gdi::{
            CreateDCW, CreatedHDC, DeleteDC, EnumDisplayMonitors, EnumDisplaySettingsExW,
            GetDeviceCaps, GetMonitorInfoW, MonitorFromPoint, DESKTOPHORZRES, DEVMODEW,
            DEVMODE_DISPLAY_ORIENTATION, EDS_RAWMODE, ENUM_CURRENT_SETTINGS, HDC, HMONITOR,
            HORZRES, MONITORINFOEXW, MONITOR_DEFAULTTONULL,
        },
    },
};

// 自动释放资源
macro_rules! drop_box {
    ($type:tt, $value:expr, $drop:expr) => {{
        struct DropBox($type);

        impl Deref for DropBox {
            type Target = $type;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl Drop for DropBox {
            fn drop(&mut self) {
                $drop(self.0);
            }
        }

        DropBox($value)
    }};
}

impl DisplayInfo {
    fn new(monitor_info_exw: &MONITORINFOEXW) -> Self {
        let sz_device = monitor_info_exw.szDevice.as_ptr();

        let sz_device_string = unsafe { U16CString::from_ptr_str(sz_device).to_string_lossy() };
        let rc_monitor = monitor_info_exw.monitorInfo.rcMonitor;
        let dw_flags = monitor_info_exw.monitorInfo.dwFlags;

        let (rotation, frequency) = get_rotation_frequency(sz_device).unwrap_or((0.0, 0.0));

        DisplayInfo {
            id: hash32(sz_device_string.as_bytes()),
            x: rc_monitor.left,
            y: rc_monitor.top,
            width: (rc_monitor.right - rc_monitor.left) as u32,
            height: (rc_monitor.bottom - rc_monitor.top) as u32,
            rotation,
            frequency,
            scale_factor: get_scale_factor(sz_device),
            is_primary: dw_flags == 1u32,
        }
    }
}

fn get_monitor_info_exw(h_monitor: HMONITOR) -> Result<MONITORINFOEXW> {
    let mut monitor_info_exw: MONITORINFOEXW = unsafe { mem::zeroed() };
    monitor_info_exw.monitorInfo.cbSize = mem::size_of::<MONITORINFOEXW>() as u32;
    let monitor_info_exw_ptr = <*mut _>::cast(&mut monitor_info_exw);

    unsafe { GetMonitorInfoW(h_monitor, monitor_info_exw_ptr).ok()? };

    Ok(monitor_info_exw)
}

fn get_rotation_frequency(sz_device: *const u16) -> Result<(f32, f32)> {
    let mut dev_modew: DEVMODEW = DEVMODEW {
        dmSize: mem::size_of::<DEVMODEW>() as u16,
        ..DEVMODEW::default()
    };

    let dev_modew_ptr = <*mut _>::cast(&mut dev_modew);

    unsafe {
        EnumDisplaySettingsExW(
            PCWSTR(sz_device),
            ENUM_CURRENT_SETTINGS,
            dev_modew_ptr,
            EDS_RAWMODE,
        )
        .ok()?;
    };

    let dm_display_orientation = unsafe { dev_modew.Anonymous1.Anonymous2.dmDisplayOrientation };

    let rotation = match dm_display_orientation {
        DEVMODE_DISPLAY_ORIENTATION(0) => 0.0,
        DEVMODE_DISPLAY_ORIENTATION(1) => 90.0,
        DEVMODE_DISPLAY_ORIENTATION(2) => 180.0,
        DEVMODE_DISPLAY_ORIENTATION(3) => 270.0,
        _ => dm_display_orientation.0 as f32,
    };

    let frequency = dev_modew.dmDisplayFrequency as f32;

    Ok((rotation, frequency))
}

fn get_scale_factor(sz_device: *const u16) -> f32 {
    let dcw_drop_box = drop_box!(
        CreatedHDC,
        unsafe {
            CreateDCW(
                PCWSTR(sz_device),
                PCWSTR(sz_device),
                PCWSTR(ptr::null()),
                None,
            )
        },
        |dcw| unsafe { DeleteDC(dcw) }
    );

    let logical_width = unsafe { GetDeviceCaps(*dcw_drop_box, HORZRES) };
    let physical_width = unsafe { GetDeviceCaps(*dcw_drop_box, DESKTOPHORZRES) };

    let scale_factor = (physical_width as f32 / logical_width as f32) * 10.0;

    scale_factor.round() / 10.0
}

pub fn get_all() -> Result<Vec<DisplayInfo>> {
    let h_monitors_mut_ptr: *mut Vec<MONITORINFOEXW> = Box::into_raw(Box::default());

    unsafe {
        EnumDisplayMonitors(
            HDC::default(),
            None,
            Some(monitor_enum_proc),
            LPARAM(h_monitors_mut_ptr as isize),
        )
        .ok()?;
    };

    let h_monitors = unsafe { Box::from_raw(h_monitors_mut_ptr) };

    let display_infos = h_monitors
        .iter()
        .map(DisplayInfo::new)
        .collect::<Vec<DisplayInfo>>();

    Ok(display_infos)
}

pub fn get_from_point(x: i32, y: i32) -> Result<DisplayInfo> {
    let point = POINT { x, y };
    let h_monitor = unsafe { MonitorFromPoint(point, MONITOR_DEFAULTTONULL) };

    if h_monitor.is_invalid() {
        return Err(anyhow!("Monitor is invalid"));
    }

    let monitor_info_exw = get_monitor_info_exw(h_monitor)?;

    Ok(DisplayInfo::new(&monitor_info_exw))
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
