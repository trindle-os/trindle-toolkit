//! What the compositor says of each window's surface and of the frames it was given.

use wayland_client::protocol::wl_callback;
use wayland_protocols::wp::fractional_scale::v1::client::wp_fractional_scale_v1;
use wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_surface_v1;

use super::*;

pub(super) fn is_suspended(states: &[u8]) -> bool {
    has_state(states, xdg_toplevel::State::Suspended as u32)
}

pub(super) fn has_state(states: &[u8], state: u32) -> bool {
    states.chunks_exact(4).any(|c| u32::from_ne_bytes([c[0], c[1], c[2], c[3]]) == state)
}

impl Dispatch<wl_surface::WlSurface, u64> for State {
    fn event(state: &mut Self, _: &wl_surface::WlSurface, event: wl_surface::Event, id: &u64, _: &Connection, _: &QueueHandle<Self>) {
        let screen = |o: &wl_output::WlOutput| state.screens.iter().find(|s| &s.output == o).map(|s| s.id);
        let (entered, left) = match &event {
            wl_surface::Event::Enter { output } => (screen(output), None),
            wl_surface::Event::Leave { output } => (None, screen(output)),
            _ => (None, None),
        };
        let Some(w) = state.win(*id) else { return };
        if let Some(s) = entered.filter(|s| !w.on.contains(s)) {
            w.on.push(s);
            w.dirty = true;
        }
        if let Some(s) = left {
            w.on.retain(|&o| o != s);
            w.dirty = true;
        }
    }
}

impl Dispatch<WpFractionalScaleV1, u64> for State {
    fn event(state: &mut Self, _: &WpFractionalScaleV1, event: wp_fractional_scale_v1::Event, id: &u64,
             _: &Connection, _: &QueueHandle<Self>) {
        if let (wp_fractional_scale_v1::Event::PreferredScale { scale }, Some(w)) = (event, state.win(*id)) {
            if w.scale120 != scale {
                w.scale120 = scale;
                w.dirty = true;
            }
        }
    }
}

impl Dispatch<xdg_wm_base::XdgWmBase, ()> for State {
    fn event(_: &mut Self, wm: &xdg_wm_base::XdgWmBase, event: xdg_wm_base::Event, _: &(), _: &Connection, _: &QueueHandle<Self>) {
        if let xdg_wm_base::Event::Ping { serial } = event {
            wm.pong(serial);
        }
    }
}

impl Dispatch<xdg_surface::XdgSurface, u64> for State {
    fn event(state: &mut Self, xdg: &xdg_surface::XdgSurface, event: xdg_surface::Event, id: &u64,
             _: &Connection, _: &QueueHandle<Self>) {
        if let xdg_surface::Event::Configure { serial } = event {
            if state.win(*id).is_some_and(|w| w.void > 0) {
                return;
            }
            xdg.ack_configure(serial);
            if let Some(w) = state.win(*id) {
                if let Some(size) = w.proposed.take() {
                    w.size = size;
                }
                w.configured = true;
                w.dirty = true;
            }
        }
    }
}

impl Dispatch<xdg_toplevel::XdgToplevel, u64> for State {
    fn event(state: &mut Self, _: &xdg_toplevel::XdgToplevel, event: xdg_toplevel::Event, id: &u64,
             _: &Connection, _: &QueueHandle<Self>) {
        let Some(w) = state.win(*id) else { return };
        match event {
            xdg_toplevel::Event::Configure { width, height, states } => {
                let suspended = is_suspended(&states);
                if w.suspended && !suspended && w.waiting {
                    w.sent = Some(std::time::Instant::now());
                }
                w.suspended = suspended;
                w.maximised = has_state(&states, xdg_toplevel::State::Maximized as u32);
                w.snug = frame::snug(&states);
                if width > 0 && height > 0 {
                    w.proposed = Some(((width as u32).min(MOST_SIDE), (height as u32).min(MOST_SIDE)));
                }
            }
            xdg_toplevel::Event::ConfigureBounds { width, height } => {
                if !w.configured && w.proposed.is_none() {
                    w.proposed = fit(w.size, (width, height));
                }
            }
            xdg_toplevel::Event::Close => w.closing = true,
            xdg_toplevel::Event::WmCapabilities { capabilities } => w.compositor_menu = has_state(&capabilities, xdg_toplevel::WmCapabilities::WindowMenu as u32),
            _ => {}
        }
    }
}

pub(super) fn fit(size: (u32, u32), (width, height): (i32, i32)) -> Option<(u32, u32)> {
    let cut = |want: u32, room: i32| if room > 0 { want.min(room as u32) } else { want };
    let fitted = (cut(size.0, width), cut(size.1, height));
    (fitted != size).then_some(fitted)
}

impl Dispatch<ZwlrLayerSurfaceV1, u64> for State {
    fn event(state: &mut Self, _: &ZwlrLayerSurfaceV1, event: zwlr_layer_surface_v1::Event, id: &u64,
             _: &Connection, _: &QueueHandle<Self>) {
        let Some(w) = state.win(*id) else { return };
        match event {
            zwlr_layer_surface_v1::Event::Configure { serial, width, height } => {
                w.ack = Some(serial);
                let size = ((if width > 0 { width } else { w.size.0 }).min(MOST_SIDE), (if height > 0 { height } else { w.size.1 }).min(MOST_SIDE));
                if !(w.configured && w.size == size && !w.dirty) {
                    w.size = size;
                    w.configured = true;
                    w.dirty = true;
                }
            }
            zwlr_layer_surface_v1::Event::Closed => w.closing = true,
            _ => {}
        }
    }
}

impl Dispatch<wl_buffer::WlBuffer, (u64, usize)> for State {
    fn event(state: &mut Self, _: &wl_buffer::WlBuffer, event: wl_buffer::Event, (id, slot): &(u64, usize),
             _: &Connection, _: &QueueHandle<Self>) {
        if let (wl_buffer::Event::Release, Some(w)) = (event, state.win(*id)) {
            w.busy[*slot] = false;
        }
    }
}

impl Dispatch<wl_callback::WlCallback, u64> for State {
    fn event(state: &mut Self, _: &wl_callback::WlCallback, event: wl_callback::Event, id: &u64,
             _: &Connection, _: &QueueHandle<Self>) {
        if let (wl_callback::Event::Done { .. }, Some(w)) = (event, state.win(*id)) {
            w.waiting = false;
            w.sent = None;
        }
    }
}

pub(super) struct Placed(pub(super) u64);

impl Dispatch<wl_callback::WlCallback, Placed> for State {
    fn event(state: &mut Self, _: &wl_callback::WlCallback, event: wl_callback::Event, id: &Placed,
             _: &Connection, _: &QueueHandle<Self>) {
        if let wl_callback::Event::Done { .. } = event {
            state.inbox.push((id.0, Input::Placed));
        }
    }
}

pub(super) struct Unvoid(pub(super) u64);

impl Dispatch<wl_callback::WlCallback, Unvoid> for State {
    fn event(state: &mut Self, _: &wl_callback::WlCallback, event: wl_callback::Event, id: &Unvoid,
             _: &Connection, _: &QueueHandle<Self>) {
        if let (wl_callback::Event::Done { .. }, Some(w)) = (event, state.win(id.0)) {
            w.void = w.void.saturating_sub(1);
        }
    }
}
