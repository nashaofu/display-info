use std::mem;

use scopeguard::{ScopeGuard, guard};
use widestring::U16CString;
use windows::{
    Win32::{
        Devices::Display::{
            DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME, DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME,
            DISPLAYCONFIG_DEVICE_INFO_HEADER, DISPLAYCONFIG_MODE_INFO, DISPLAYCONFIG_PATH_INFO,
            DISPLAYCONFIG_SOURCE_DEVICE_NAME, DISPLAYCONFIG_TARGET_DEVICE_NAME,
            DisplayConfigGetDeviceInfo, GetDisplayConfigBufferSizes, QDC_ONLY_ACTIVE_PATHS,
            QueryDisplayConfig,
        },
        Foundation::{FreeLibrary, GetLastError, HANDLE, HMODULE, LPARAM, RECT, TRUE},
        Graphics::Gdi::{
            DESKTOPHORZRES, DEVMODEW, DISPLAY_DEVICEW, ENUM_CURRENT_SETTINGS, EnumDisplayDevicesW,
            EnumDisplaySettingsW, GetDeviceCaps, HDC, HMONITOR, HORZRES, MONITORINFOEXW,
        },
        System::{
            LibraryLoader::{GetProcAddress, LoadLibraryW},
            Threading::GetCurrentProcess,
        },
    },
    core::{BOOL, HRESULT, PCWSTR, s, w},
};

use crate::error::{DIError, DIResult};

// 定义 GetProcessDpiAwareness 函数的类型
type GetProcessDpiAwareness =
    unsafe extern "system" fn(h_process: HANDLE, value: *mut u32) -> HRESULT;

pub(super) fn get_process_is_dpi_awareness(process: HANDLE) -> DIResult<bool> {
    unsafe {
        let scope_guard_hmodule = load_library(w!("Shcore.dll"))?;

        let get_process_dpi_awareness_proc_address =
            GetProcAddress(*scope_guard_hmodule, s!("GetProcessDpiAwareness"))
                .ok_or(DIError::new("GetProcAddress GetProcessDpiAwareness failed"))?;

        let get_process_dpi_awareness: GetProcessDpiAwareness =
            mem::transmute(get_process_dpi_awareness_proc_address);

        let mut process_dpi_awareness = 0;
        // https://learn.microsoft.com/zh-cn/windows/win32/api/shellscalingapi/nf-shellscalingapi-getprocessdpiawareness
        get_process_dpi_awareness(process, &mut process_dpi_awareness).ok()?;

        // 当前进程不感知 DPI，则回退到 GetDeviceCaps 获取 DPI
        Ok(process_dpi_awareness != 0)
    }
}

pub(super) fn load_library(
    lib_filename: PCWSTR,
) -> DIResult<ScopeGuard<HMODULE, impl FnOnce(HMODULE)>> {
    unsafe {
        let hmodule = LoadLibraryW(lib_filename)?;

        if hmodule.is_invalid() {
            return Err(DIError::new(format!(
                "LoadLibraryW error {:?}",
                GetLastError()
            )));
        }

        let scope_guard_hmodule = guard(hmodule, |val| {
            if let Err(err) = FreeLibrary(val) {
                log::error!("FreeLibrary {:?} failed {:?}", val, err);
            }
        });

        Ok(scope_guard_hmodule)
    }
}

pub(super) extern "system" fn monitor_enum_proc(
    h_monitor: HMONITOR,
    _: HDC,
    _: *mut RECT,
    state: LPARAM,
) -> BOOL {
    unsafe {
        let state = Box::leak(Box::from_raw(state.0 as *mut Vec<HMONITOR>));
        state.push(h_monitor);

        TRUE
    }
}

pub(super) fn get_dev_mode_w(monitor_info_exw: &MONITORINFOEXW) -> DIResult<DEVMODEW> {
    let sz_device = monitor_info_exw.szDevice.as_ptr();
    let mut dev_mode_w = DEVMODEW {
        dmSize: mem::size_of::<DEVMODEW>() as u16,
        ..DEVMODEW::default()
    };

    unsafe {
        EnumDisplaySettingsW(PCWSTR(sz_device), ENUM_CURRENT_SETTINGS, &mut dev_mode_w).ok()?;
    };

    Ok(dev_mode_w)
}

// 定义 GetDpiForMonitor 函数的类型
type GetDpiForMonitor = unsafe extern "system" fn(
    h_monitor: HMONITOR,
    dpi_type: u32,
    dpi_x: *mut u32,
    dpi_y: *mut u32,
) -> HRESULT;

pub(super) fn get_hi_dpi_scale_factor(h_monitor: HMONITOR) -> DIResult<f32> {
    unsafe {
        let current_process_is_dpi_awareness: bool =
            get_process_is_dpi_awareness(GetCurrentProcess())?;

        // 当前进程不感知 DPI，则回退到 GetDeviceCaps 获取 DPI
        if !current_process_is_dpi_awareness {
            return Err(DIError::new("Process not DPI aware"));
        }

        let scope_guard_hmodule = load_library(w!("Shcore.dll"))?;

        let get_dpi_for_monitor_proc_address =
            GetProcAddress(*scope_guard_hmodule, s!("GetDpiForMonitor"))
                .ok_or(DIError::new("GetProcAddress GetDpiForMonitor failed"))?;

        let get_dpi_for_monitor: GetDpiForMonitor =
            mem::transmute(get_dpi_for_monitor_proc_address);

        let mut dpi_x = 0;
        let mut dpi_y = 0;

        // https://learn.microsoft.com/zh-cn/windows/win32/api/shellscalingapi/ne-shellscalingapi-monitor_dpi_type
        get_dpi_for_monitor(h_monitor, 0, &mut dpi_x, &mut dpi_y).ok()?;

        Ok(dpi_x as f32 / 96.0)
    }
}

pub(super) fn get_scale_factor(
    h_monitor: HMONITOR,
    scope_guard_hdc: ScopeGuard<HDC, impl FnOnce(HDC)>,
) -> DIResult<f32> {
    let scale_factor = get_hi_dpi_scale_factor(h_monitor).unwrap_or_else(|err| {
        log::info!("{}", err);
        // https://learn.microsoft.com/zh-cn/windows/win32/api/wingdi/nf-wingdi-getdevicecaps
        unsafe {
            let physical_width = GetDeviceCaps(Some(*scope_guard_hdc), DESKTOPHORZRES);
            let logical_width = GetDeviceCaps(Some(*scope_guard_hdc), HORZRES);

            physical_width as f32 / logical_width as f32
        }
    });

    Ok(scale_factor)
}

fn get_display_device_string(monitor_info_ex_w: MONITORINFOEXW) -> DIResult<String> {
    unsafe {
        let mut display_device = DISPLAY_DEVICEW {
            cb: mem::size_of::<DISPLAY_DEVICEW>() as u32,
            ..DISPLAY_DEVICEW::default()
        };
        EnumDisplayDevicesW(
            PCWSTR(monitor_info_ex_w.szDevice.as_ptr()),
            0,
            &mut display_device,
            0,
        )
        .ok()?;

        let device_string =
            U16CString::from_vec_truncate(display_device.DeviceString).to_string()?;

        Ok(device_string)
    }
}

pub(super) fn get_display_friendly_name(monitor_info_ex_w: MONITORINFOEXW) -> DIResult<String> {
    unsafe {
        let mut number_of_paths = 0;
        let mut number_of_modes = 0;
        GetDisplayConfigBufferSizes(
            QDC_ONLY_ACTIVE_PATHS,
            &mut number_of_paths,
            &mut number_of_modes,
        )
        .ok()?;

        let mut paths = vec![DISPLAYCONFIG_PATH_INFO::default(); number_of_paths as usize];
        let mut modes = vec![DISPLAYCONFIG_MODE_INFO::default(); number_of_modes as usize];

        QueryDisplayConfig(
            QDC_ONLY_ACTIVE_PATHS,
            &mut number_of_paths,
            paths.as_mut_ptr(),
            &mut number_of_modes,
            modes.as_mut_ptr(),
            None,
        )
        .ok()?;

        for path in paths {
            let mut source = DISPLAYCONFIG_SOURCE_DEVICE_NAME {
                header: DISPLAYCONFIG_DEVICE_INFO_HEADER {
                    r#type: DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME,
                    size: mem::size_of::<DISPLAYCONFIG_SOURCE_DEVICE_NAME>() as u32,
                    adapterId: path.sourceInfo.adapterId,
                    id: path.sourceInfo.id,
                },
                ..DISPLAYCONFIG_SOURCE_DEVICE_NAME::default()
            };

            if DisplayConfigGetDeviceInfo(&mut source.header) != 0 {
                continue;
            }

            if source.viewGdiDeviceName != monitor_info_ex_w.szDevice {
                continue;
            }

            let mut target = DISPLAYCONFIG_TARGET_DEVICE_NAME {
                header: DISPLAYCONFIG_DEVICE_INFO_HEADER {
                    r#type: DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME,
                    size: mem::size_of::<DISPLAYCONFIG_TARGET_DEVICE_NAME>() as u32,
                    adapterId: path.sourceInfo.adapterId,
                    id: path.targetInfo.id,
                },
                ..DISPLAYCONFIG_TARGET_DEVICE_NAME::default()
            };

            if DisplayConfigGetDeviceInfo(&mut target.header) != 0 {
                continue;
            }

            let name =
                U16CString::from_vec_truncate(target.monitorFriendlyDeviceName).to_string()?;

            if name.is_empty() {
                return get_display_device_string(monitor_info_ex_w);
            }

            return Ok(name);
        }

        get_display_device_string(monitor_info_ex_w)
    }
}
