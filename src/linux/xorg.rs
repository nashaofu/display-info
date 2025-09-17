use std::str;
use xcb::x::{Atom, GetAtomName};
use xcb::{
    Connection, Xid,
    randr::{
        GetCrtcInfo, GetMonitors, GetOutputInfo, GetScreenResources, Mode, ModeFlag, ModeInfo,
        Output, Rotation,
    },
    x::{ATOM_RESOURCE_MANAGER, ATOM_STRING, GetProperty, Screen},
};

use crate::DisplayInfo;
use crate::error::{DIError, DIResult};

pub type ScreenRawHandle = Output;

fn get_name(conn: &Connection, atom: Atom) -> DIResult<String> {
    let get_atom_value = conn.send_request(&GetAtomName { atom });

    let get_atom_value_reply = conn.wait_for_reply(get_atom_value)?;
    Ok(get_atom_value_reply.name().to_string())
}

// per https://gitlab.freedesktop.org/xorg/app/xrandr/-/blob/master/xrandr.c#L576
fn get_current_frequency(mode_infos: &[ModeInfo], mode: Mode) -> f32 {
    let mode_info = match mode_infos.iter().find(|m| m.id == mode.resource_id()) {
        Some(mode_info) => mode_info,
        None => return 0.0,
    };

    let vtotal = {
        let mut val = mode_info.vtotal;
        if mode_info.mode_flags.contains(ModeFlag::DOUBLE_SCAN) {
            val *= 2;
        }
        if mode_info.mode_flags.contains(ModeFlag::INTERLACE) {
            val /= 2;
        }
        val
    };

    if vtotal != 0 && mode_info.htotal != 0 {
        (mode_info.dot_clock as f32) / (vtotal as f32 * mode_info.htotal as f32)
    } else {
        0.0
    }
}

fn get_scale_factor(conn: &Connection, screen: &Screen) -> DIResult<f32> {
    let xft_dpi_prefix = "Xft.dpi:\t";

    let get_property_cookie = conn.send_request(&GetProperty {
        delete: false,
        window: screen.root(),
        property: ATOM_RESOURCE_MANAGER,
        r#type: ATOM_STRING,
        long_offset: 0,
        long_length: 60,
    });

    let get_property_reply = conn.wait_for_reply(get_property_cookie)?;

    let resource_manager = str::from_utf8(get_property_reply.value())?;

    let xft_dpi = resource_manager
        .split('\n')
        .find(|s| s.starts_with(xft_dpi_prefix))
        .ok_or_else(|| DIError::new("Xft.dpi parse failed"))?
        .strip_prefix(xft_dpi_prefix)
        .ok_or_else(|| DIError::new("Xft.dpi parse failed"))?;

    let dpi = xft_dpi.parse::<f32>().map_err(DIError::new)?;

    Ok(dpi / 96.0)
}

fn get_rotation_frequency(
    conn: &Connection,
    mode_infos: &[ModeInfo],
    output: &Output,
) -> DIResult<(f32, f32)> {
    let get_output_info_cookie = conn.send_request(&GetOutputInfo {
        output: *output,
        config_timestamp: 0,
    });

    let get_output_info_reply = conn.wait_for_reply(get_output_info_cookie)?;

    let get_crtc_info_cookie = conn.send_request(&GetCrtcInfo {
        crtc: get_output_info_reply.crtc(),
        config_timestamp: 0,
    });

    let get_crtc_info_reply = conn.wait_for_reply(get_crtc_info_cookie)?;

    let mode = get_crtc_info_reply.mode();

    let rotation = match get_crtc_info_reply.rotation() {
        Rotation::ROTATE_0 => 0.0,
        Rotation::ROTATE_90 => 90.0,
        Rotation::ROTATE_180 => 180.0,
        Rotation::ROTATE_270 => 270.0,
        _ => 0.0,
    };

    let frequency = get_current_frequency(mode_infos, mode);

    Ok((rotation, frequency))
}

pub fn get_all() -> DIResult<Vec<DisplayInfo>> {
    let (conn, index) = Connection::connect(None)?;

    let setup = conn.get_setup();

    let screen = setup
        .roots()
        .nth(index as usize)
        .ok_or_else(|| DIError::new("Not found screen"))?;

    let scale_factor = get_scale_factor(&conn, screen).unwrap_or(1.0);

    let get_monitors_cookie = conn.send_request(&GetMonitors {
        window: screen.root(),
        get_active: true,
    });

    let get_monitors_reply = conn.wait_for_reply(get_monitors_cookie)?;

    let monitor_info_iterator = get_monitors_reply.monitors();

    let get_screen_resources_cookie = conn.send_request(&GetScreenResources {
        window: screen.root(),
    });

    let get_screen_resources_reply = conn.wait_for_reply(get_screen_resources_cookie)?;

    let mode_infos = get_screen_resources_reply.modes();

    let mut display_infos = Vec::new();

    for monitor_info in monitor_info_iterator {
        let output = monitor_info
            .outputs()
            .first()
            .ok_or_else(|| DIError::new("Not found output"))?;

        let (rotation, frequency) =
            get_rotation_frequency(&conn, mode_infos, output).unwrap_or((0.0, 0.0));

        let name = get_name(&conn, monitor_info.name())?;

        display_infos.push(DisplayInfo {
            id: output.resource_id(),
            name: name.clone(),
            friendly_name: name,
            raw_handle: *output,
            x: ((monitor_info.x() as f32) / scale_factor) as i32,
            y: ((monitor_info.y() as f32) / scale_factor) as i32,
            width: ((monitor_info.width() as f32) / scale_factor) as u32,
            height: ((monitor_info.height() as f32) / scale_factor) as u32,
            width_mm: monitor_info.width_in_millimeters() as i32,
            height_mm: monitor_info.height_in_millimeters() as i32,
            rotation,
            scale_factor,
            frequency,
            is_primary: monitor_info.primary(),
        });
    }

    Ok(display_infos)
}

pub fn get_from_point(x: i32, y: i32) -> DIResult<DisplayInfo> {
    let display_infos = DisplayInfo::all()?;

    display_infos
        .iter()
        .find(|&display_info| {
            x >= display_info.x
                && x < display_info.x + display_info.width as i32
                && y >= display_info.y
                && y < display_info.y + display_info.height as i32
        })
        .cloned()
        .ok_or_else(|| DIError::new("Get display info failed"))
}
