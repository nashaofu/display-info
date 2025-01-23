use std::{mem, ptr};

use anyhow::{anyhow, Result};
use scopeguard::guard;
use utils::{get_dev_mode_w, get_scale_factor, monitor_enum_proc};
use widestring::U16CString;
use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::{LPARAM, POINT},
        Graphics::Gdi::{
            CreateDCW, DeleteDC, EnumDisplayMonitors, GetDeviceCaps, GetMonitorInfoW,
            MonitorFromPoint, DMDO_180, DMDO_270, DMDO_90, DMDO_DEFAULT, HDC, HMONITOR, HORZSIZE,
            MONITORINFO, MONITORINFOEXW, MONITOR_DEFAULTTONULL, VERTSIZE,
        },
        UI::WindowsAndMessaging::MONITORINFOF_PRIMARY,
    },
};

use crate::DisplayInfo;

mod utils;

pub type ScreenRawHandle = HMONITOR;

impl DisplayInfo {
    pub fn new(hmonitor: HMONITOR) -> Result<DisplayInfo> {
        let mut monitor_info_ex_w = MONITORINFOEXW::default();
        monitor_info_ex_w.monitorInfo.cbSize = mem::size_of::<MONITORINFOEXW>() as u32;
        let monitor_info_ex_w_ptr =
            &mut monitor_info_ex_w as *mut MONITORINFOEXW as *mut MONITORINFO;

        // https://learn.microsoft.com/zh-cn/windows/win32/api/winuser/nf-winuser-getmonitorinfoa
        unsafe { GetMonitorInfoW(hmonitor, monitor_info_ex_w_ptr).ok()? };

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
                GetDeviceCaps(*scope_guard_hdc, HORZSIZE),
                GetDeviceCaps(*scope_guard_hdc, VERTSIZE),
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

        let scale_factor = get_scale_factor(hmonitor, scope_guard_hdc)?;

        Ok(DisplayInfo {
            id: hmonitor.0 as u32,
            name,
            raw_handle: hmonitor,
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

    pub fn all() -> Result<Vec<DisplayInfo>> {
        let hmonitors_mut_ptr: *mut Vec<HMONITOR> = Box::into_raw(Box::default());

        let hmonitors = unsafe {
            EnumDisplayMonitors(
                HDC::default(),
                None,
                Some(monitor_enum_proc),
                LPARAM(hmonitors_mut_ptr as isize),
            )
            .ok()?;
            Box::from_raw(hmonitors_mut_ptr)
        };

        let mut impl_monitors = Vec::with_capacity(hmonitors.len());

        for &hmonitor in hmonitors.iter() {
            if let Ok(impl_monitor) = DisplayInfo::new(hmonitor) {
                impl_monitors.push(impl_monitor);
            } else {
                log::error!("ImplMonitor::new({:?}) failed", hmonitor);
            }
        }

        Ok(impl_monitors)
    }

    pub fn from_point(x: i32, y: i32) -> Result<DisplayInfo> {
        let point = POINT { x, y };
        let hmonitor = unsafe { MonitorFromPoint(point, MONITOR_DEFAULTTONULL) };

        if hmonitor.is_invalid() {
            return Err(anyhow!("Not found monitor"));
        }

        DisplayInfo::new(hmonitor)
    }
}
