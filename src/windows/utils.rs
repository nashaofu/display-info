use std::mem;

use anyhow::{anyhow, Result};
use scopeguard::{guard, ScopeGuard};
use windows::{
    core::{s, w, HRESULT, PCWSTR},
    Win32::{
        Foundation::{FreeLibrary, GetLastError, BOOL, HANDLE, HMODULE, LPARAM, RECT, TRUE},
        Graphics::Gdi::{
            EnumDisplaySettingsW, GetDeviceCaps, DESKTOPHORZRES, DEVMODEW, ENUM_CURRENT_SETTINGS,
            HDC, HMONITOR, HORZRES, MONITORINFOEXW,
        },
        System::{
            LibraryLoader::{GetProcAddress, LoadLibraryW},
            Threading::GetCurrentProcess,
        },
    },
};

// 定义 GetProcessDpiAwareness 函数的类型
type GetProcessDpiAwareness =
    unsafe extern "system" fn(hprocess: HANDLE, value: *mut u32) -> HRESULT;

pub(super) fn get_process_is_dpi_awareness(process: HANDLE) -> Result<bool> {
    unsafe {
        let scope_guard_hmodule = load_library(w!("Shcore.dll"))?;

        let get_process_dpi_awareness_proc_address =
            GetProcAddress(*scope_guard_hmodule, s!("GetProcessDpiAwareness"))
                .ok_or(anyhow!("GetProcAddress GetProcessDpiAwareness failed"))?;

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
) -> Result<ScopeGuard<HMODULE, impl FnOnce(HMODULE)>> {
    unsafe {
        let hmodule = LoadLibraryW(lib_filename)?;

        if hmodule.is_invalid() {
            return Err(anyhow!("LoadLibraryW error {:?}", GetLastError()));
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
    hmonitor: HMONITOR,
    _: HDC,
    _: *mut RECT,
    state: LPARAM,
) -> BOOL {
    unsafe {
        let state = Box::leak(Box::from_raw(state.0 as *mut Vec<HMONITOR>));
        state.push(hmonitor);

        TRUE
    }
}

pub(super) fn get_dev_mode_w(monitor_info_exw: &MONITORINFOEXW) -> Result<DEVMODEW> {
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
    hmonitor: HMONITOR,
    dpi_type: u32,
    dpi_x: *mut u32,
    dpi_y: *mut u32,
) -> HRESULT;

pub(super) fn get_hi_dpi_scale_factor(hmonitor: HMONITOR) -> Result<f32> {
    unsafe {
        let current_process_is_dpi_awareness: bool =
            get_process_is_dpi_awareness(GetCurrentProcess())?;

        // 当前进程不感知 DPI，则回退到 GetDeviceCaps 获取 DPI
        if !current_process_is_dpi_awareness {
            return Err(anyhow!("Process not DPI aware"));
        }

        let scope_guard_hmodule = load_library(w!("Shcore.dll"))?;

        let get_dpi_for_monitor_proc_address =
            GetProcAddress(*scope_guard_hmodule, s!("GetDpiForMonitor"))
                .ok_or(anyhow!("GetProcAddress GetDpiForMonitor failed"))?;

        let get_dpi_for_monitor: GetDpiForMonitor =
            mem::transmute(get_dpi_for_monitor_proc_address);

        let mut dpi_x = 0;
        let mut dpi_y = 0;

        // https://learn.microsoft.com/zh-cn/windows/win32/api/shellscalingapi/ne-shellscalingapi-monitor_dpi_type
        get_dpi_for_monitor(hmonitor, 0, &mut dpi_x, &mut dpi_y).ok()?;

        Ok(dpi_x as f32 / 96.0)
    }
}

pub(super) fn get_scale_factor(
    hmonitor: HMONITOR,
    scope_guard_hdc: ScopeGuard<HDC, impl FnOnce(HDC)>,
) -> Result<f32> {
    let scale_factor = get_hi_dpi_scale_factor(hmonitor).unwrap_or_else(|err| {
        log::info!("{}", err);
        // https://learn.microsoft.com/zh-cn/windows/win32/api/wingdi/nf-wingdi-getdevicecaps
        unsafe {
            let physical_width = GetDeviceCaps(*scope_guard_hdc, DESKTOPHORZRES);
            let logical_width = GetDeviceCaps(*scope_guard_hdc, HORZRES);

            physical_width as f32 / logical_width as f32
        }
    });

    Ok(scale_factor)
}
