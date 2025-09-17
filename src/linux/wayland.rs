use smithay_client_toolkit::output::{OutputHandler, OutputInfo, OutputState};
use smithay_client_toolkit::reexports::client::globals::registry_queue_init;
use smithay_client_toolkit::reexports::client::protocol::wl_output;
use smithay_client_toolkit::reexports::client::{Connection, QueueHandle};
use smithay_client_toolkit::registry::{ProvidesRegistryState, RegistryState};
use smithay_client_toolkit::{delegate_output, delegate_registry, registry_handlers};
use xcb::XidNew;

use crate::DisplayInfo;
use crate::error::{DIError, DIResult};

impl From<&OutputInfo> for DisplayInfo {
    fn from(info: &OutputInfo) -> Self {
        let scale_factor = info.scale_factor as f32;
        let rotation = match info.transform {
            wl_output::Transform::_90 | wl_output::Transform::Flipped90 => 90.,
            wl_output::Transform::_180 | wl_output::Transform::Flipped180 => 180.,
            wl_output::Transform::_270 | wl_output::Transform::Flipped270 => 270.,
            _ => 0.,
        };
        let frequency = info
            .modes
            .iter()
            .find(|m| m.current || m.preferred)
            .map(|m| m.refresh_rate as f32 / 1000.0)
            .unwrap_or(0.);
        let (x, y) = info.logical_position.unwrap_or(info.location);
        let (w, h) = info.logical_size.unwrap_or(info.physical_size);
        let (width_mm, height_mm) = info.physical_size;
        DisplayInfo {
            id: info.id,
            name: info.name.clone().unwrap_or_default(),
            friendly_name: info
                .name
                .clone()
                .unwrap_or(format!("Unknown Display {}", info.id)),
            raw_handle: unsafe { xcb::randr::Output::new(info.id) },
            x: ((x as f32) / scale_factor) as i32,
            y: ((y as f32) / scale_factor) as i32,
            width: ((w as f32) / scale_factor) as u32,
            height: ((h as f32) / scale_factor) as u32,
            width_mm,
            height_mm,
            rotation,
            scale_factor,
            frequency,
            is_primary: false,
        }
    }
}

/// Application data.
struct ListOutputs {
    registry_state: RegistryState,
    output_state: OutputState,
}

impl OutputHandler for ListOutputs {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }

    fn new_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }

    fn update_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }

    fn output_destroyed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }
}

delegate_output!(ListOutputs);
delegate_registry!(ListOutputs);

impl ProvidesRegistryState for ListOutputs {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }

    registry_handlers! {
        OutputState,
    }
}

pub fn get_all() -> DIResult<Vec<DisplayInfo>> {
    let conn = Connection::connect_to_env()?;

    let (globals, mut event_queue) = registry_queue_init(&conn).unwrap();
    let qh = event_queue.handle();

    let registry_state = RegistryState::new(&globals);

    let output_delegate = OutputState::new(&globals, &qh);

    let mut list_outputs = ListOutputs {
        registry_state,
        output_state: output_delegate,
    };

    event_queue.roundtrip(&mut list_outputs)?;

    list_outputs
        .output_state
        .outputs()
        .map(|output| {
            list_outputs
                .output_state
                .info(&output)
                .map(|o| DisplayInfo::from(&o))
                .ok_or(DIError::new("Cannot get info from Output in Wayland"))
        })
        .collect::<DIResult<Vec<DisplayInfo>>>()
}

pub fn get_from_point(x: i32, y: i32) -> DIResult<DisplayInfo> {
    let display_infos = get_all()?;

    display_infos
        .iter()
        .find(|&d| {
            x >= d.x
                && x - (d.width as i32) < d.x + d.width as i32
                && y >= d.y
                && y - (d.height as i32) < d.y + d.height as i32
        })
        .cloned()
        .ok_or_else(|| DIError::new("Get display info failed"))
}
