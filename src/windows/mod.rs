use std::{mem, ptr};

use scopeguard::guard;
use utils::{get_dev_mode_w, get_display_friendly_name, get_scale_factor, monitor_enum_proc};
use widestring::U16CString;
use windows::{
    Win32::{
        Foundation::{LPARAM, POINT},
        Graphics::Gdi::{
            CreateDCW, DMDO_90, DMDO_180, DMDO_270, DMDO_DEFAULT, DeleteDC, EnumDisplayMonitors,
            GetDeviceCaps, GetMonitorInfoW, HMONITOR, HORZSIZE, MONITOR_DEFAULTTONULL, MONITORINFO,
            MONITORINFOEXW, MonitorFromPoint, VERTSIZE,
        },
        UI::WindowsAndMessaging::MONITORINFOF_PRIMARY,
    },
    core::PCWSTR,
};

use crate::{
    DisplayInfo,
    error::{DIError, DIResult},
};

mod utils;

pub type ScreenRawHandle = HMONITOR;

impl DisplayInfo {
    fn new(h_monitor: HMONITOR) -> DIResult<DisplayInfo> {
        let mut monitor_info_ex_w = MONITORINFOEXW::default();
        monitor_info_ex_w.monitorInfo.cbSize = mem::size_of::<MONITORINFOEXW>() as u32;
        let monitor_info_ex_w_ptr =
            &mut monitor_info_ex_w as *mut MONITORINFOEXW as *mut MONITORINFO;

        // https://learn.microsoft.com/zh-cn/windows/win32/api/winuser/nf-winuser-getmonitorinfoa
        unsafe { GetMonitorInfoW(h_monitor, monitor_info_ex_w_ptr).ok()? };

        let name = U16CString::from_vec_truncate(monitor_info_ex_w.szDevice).to_string()?;

        let dev_mode_w = get_dev_mode_w(&monitor_info_ex_w)?;

        let dm_position = unsafe { dev_mode_w.Anonymous1.Anonymous2.dmPosition };
        let dm_pels_width = dev_mode_w.dmPelsWidth;
        let dm_pels_height = dev_mode_w.dmPelsHeight;

        let scope_guard_hdc = guard(
            unsafe {
                CreateDCW(
                    PCWSTR(monitor_info_ex_w.szDevice.as_ptr()),
                    PCWSTR(monitor_info_ex_w.szDevice.as_ptr()),
                    PCWSTR(ptr::null()),
                    None,
                )
            },
            |val| unsafe {
                if !DeleteDC(val).as_bool() {
                    log::error!("DeleteDC {:?} failed", val)
                }
            },
        );

        let (width_mm, height_mm) = unsafe {
            (
                GetDeviceCaps(Some(*scope_guard_hdc), HORZSIZE),
                GetDeviceCaps(Some(*scope_guard_hdc), VERTSIZE),
            )
        };

        let dm_display_orientation =
            unsafe { dev_mode_w.Anonymous1.Anonymous2.dmDisplayOrientation };
        let rotation = match dm_display_orientation {
            DMDO_90 => 90.0,
            DMDO_180 => 180.0,
            DMDO_270 => 270.0,
            DMDO_DEFAULT => 0.0,
            _ => 0.0,
        };

        let scale_factor = get_scale_factor(h_monitor, scope_guard_hdc)?;

        Ok(DisplayInfo {
            id: h_monitor.0 as u32,
            name,
            friendly_name: get_display_friendly_name(monitor_info_ex_w)
                .unwrap_or(format!("Unknown Display {}", h_monitor.0 as u32)),
            raw_handle: h_monitor,
            x: dm_position.x,
            y: dm_position.y,
            width: dm_pels_width,
            height: dm_pels_height,
            width_mm,
            height_mm,
            rotation,
            scale_factor,
            frequency: dev_mode_w.dmDisplayFrequency as f32,
            is_primary: monitor_info_ex_w.monitorInfo.dwFlags == MONITORINFOF_PRIMARY,
        })
    }

    pub fn all() -> DIResult<Vec<DisplayInfo>> {
        let h_monitors_mut_ptr: *mut Vec<HMONITOR> = Box::into_raw(Box::default());

        let h_monitors = unsafe {
            EnumDisplayMonitors(
                None,
                None,
                Some(monitor_enum_proc),
                LPARAM(h_monitors_mut_ptr as isize),
            )
            .ok()?;
            Box::from_raw(h_monitors_mut_ptr)
        };

        let mut impl_monitors = Vec::with_capacity(h_monitors.len());

        for &h_monitor in h_monitors.iter() {
            if let Ok(impl_monitor) = DisplayInfo::new(h_monitor) {
                impl_monitors.push(impl_monitor);
            } else {
                log::error!("ImplMonitor::new({:?}) failed", h_monitor);
            }
        }

        Ok(impl_monitors)
    }

    pub fn from_point(x: i32, y: i32) -> DIResult<DisplayInfo> {
        let point = POINT { x, y };
        let h_monitor = unsafe { MonitorFromPoint(point, MONITOR_DEFAULTTONULL) };

        if h_monitor.is_invalid() {
            return Err(DIError::new("Not found monitor"));
        }

        DisplayInfo::new(h_monitor)
    }
}
