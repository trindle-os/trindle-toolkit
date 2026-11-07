//! The screens: how they come and go, and what the compositor says of each.

use wayland_client::globals::GlobalListContents;
use wayland_client::protocol::wl_registry;
use wayland_protocols::xdg::xdg_output::zv1::client::zxdg_output_v1;

use super::*;

pub(super) fn add_screen(state: &mut State, qh: &QueueHandle<State>, registry: &wl_registry::WlRegistry, id: u32, version: u32) {
    let output = registry.bind::<wl_output::WlOutput, _, _>(id, version.min(4), qh, id);
    let placer = state.places.as_ref().map(|m| m.get_xdg_output(&output, qh, id));
    let fresh = output.version() >= 2;
    state.screens_changed |= !fresh;
    state.screens.push(Screen { id, output, name: String::new(), scale: 1, place: [0; 4], placer, fresh });
}

fn remove_screen(state: &mut State, id: u32) {
    if let Some(at) = state.screens.iter().position(|s| s.id == id) {
        let s = state.screens.remove(at);
        state.screens_changed = true;
        if let Some(p) = s.placer {
            p.destroy();
        }
        if s.output.version() >= 3 {
            s.output.release();
        }
    }
    for w in &mut state.windows {
        w.on.retain(|&o| o != id);
    }
}

impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for State {
    fn event(state: &mut Self, registry: &wl_registry::WlRegistry, event: wl_registry::Event, _: &GlobalListContents,
             _: &Connection, qh: &QueueHandle<Self>) {
        match event {
            wl_registry::Event::Global { name, interface, version } if interface == "wl_output" => {
                add_screen(state, qh, registry, name, version);
            }
            wl_registry::Event::GlobalRemove { name } => remove_screen(state, name),
            _ => {}
        }
    }
}

impl Dispatch<wl_output::WlOutput, u32> for State {
    fn event(state: &mut Self, _: &wl_output::WlOutput, event: wl_output::Event, id: &u32, _: &Connection, _: &QueueHandle<Self>) {
        let Some(s) = state.screens.iter_mut().find(|s| s.id == *id) else { return };
        match event {
            wl_output::Event::Name { name } => s.name = name,
            wl_output::Event::Done if s.fresh => {
                s.fresh = false;
                state.screens_changed = true;
            }
            wl_output::Event::Scale { factor } if s.scale != factor.max(1) => {
                s.scale = factor.max(1);
                for w in state.windows.iter_mut().filter(|w| w.on.contains(id)) {
                    w.dirty = true;
                }
            }
            _ => {}
        }
    }
}

impl Dispatch<ZxdgOutputV1, u32> for State {
    fn event(state: &mut Self, _: &ZxdgOutputV1, event: zxdg_output_v1::Event, id: &u32, _: &Connection, _: &QueueHandle<Self>) {
        let Some(s) = state.screens.iter_mut().find(|s| s.id == *id) else { return };
        match event {
            zxdg_output_v1::Event::LogicalPosition { x, y } => (s.place[0], s.place[1]) = (x, y),
            zxdg_output_v1::Event::LogicalSize { width, height } => (s.place[2], s.place[3]) = (width, height),
            _ => {}
        }
    }
}
