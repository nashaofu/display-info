use crate::{DisplayInfo, DisplayInfoError};
use std::str;
use xcb::{
  randr::{GetCrtcInfo, GetMonitors, GetOutputInfo, MonitorInfo, Output, Rotation},
  x::{GetProperty, Screen, ATOM_RESOURCE_MANAGER, ATOM_STRING},
  Connection, Xid,
};

impl DisplayInfo {
  fn new(monitor_info: &MonitorInfo, output: &Output, rotation: f32, scale_factor: f32) -> Self {
    DisplayInfo {
      id: output.resource_id(),
      x: ((monitor_info.x() as f32) / scale_factor) as i32,
      y: ((monitor_info.y() as f32) / scale_factor) as i32,
      width: ((monitor_info.width() as f32) / scale_factor) as u32,
      height: ((monitor_info.height() as f32) / scale_factor) as u32,
      rotation,
      scale_factor,
      is_primary: monitor_info.primary(),
    }
  }
}

fn get_scale_factor(conn: &Connection, screen: &Screen) -> Result<f32, DisplayInfoError> {
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
    .unwrap_or_default()
    .strip_prefix(xft_dpi_prefix)
    .ok_or_else(|| DisplayInfoError::new("Can't get Xft.dpi"))?
    .parse::<f32>()?;

  Ok(xft_dpi / 96.0)
}

fn get_rotation(conn: &Connection, output: &Output) -> Result<f32, DisplayInfoError> {
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

  let rotation = match get_crtc_info_reply.rotation() {
    Rotation::ROTATE_0 => 0.0,
    Rotation::ROTATE_90 => 90.0,
    Rotation::ROTATE_180 => 180.0,
    Rotation::ROTATE_270 => 270.0,
    _ => 0.0,
  };

  Ok(rotation)
}

pub fn get_all() -> Result<Vec<DisplayInfo>, DisplayInfoError> {
  let (conn, index) = Connection::connect(None)?;

  let setup = conn.get_setup();

  let screen = setup
    .roots()
    .nth(index as usize)
    .ok_or_else(|| DisplayInfoError::new("Get screen error"))?;

  let scale_factor = get_scale_factor(&conn, screen).unwrap_or(1.0);

  let get_monitors_cookie = conn.send_request(&GetMonitors {
    window: screen.root(),
    get_active: true,
  });

  let get_monitors_reply = conn.wait_for_reply(get_monitors_cookie)?;

  let monitor_info_iterator = get_monitors_reply.monitors();

  let mut display_infos = Vec::new();

  for monitor_info in monitor_info_iterator {
    let output = monitor_info
      .outputs()
      .get(0)
      .ok_or_else(|| DisplayInfoError::new("Get output error"))?;

    let rotation = get_rotation(&conn, output).unwrap_or(0.0);

    display_infos.push(DisplayInfo::new(
      monitor_info,
      output,
      rotation,
      scale_factor,
    ));
  }

  Ok(display_infos)
}

pub fn get_from_point(x: i32, y: i32) -> Result<DisplayInfo, DisplayInfoError> {
  let display_infos = DisplayInfo::all()?;

  let display_info = display_infos
    .iter()
    .find(|&&display_info| {
      x >= display_info.x
        && x <= display_info.x + display_info.width as i32
        && y >= display_info.y
        && y <= display_info.y + display_info.height as i32
    })
    .ok_or_else(|| DisplayInfoError::new("Can't find display"))?;

  Ok(*display_info)
}
