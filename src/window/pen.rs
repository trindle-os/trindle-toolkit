//! A tablet's pen (where, how hard, at what slant, and whether it is the eraser end), and a pointer held still in a window that reports only how far it is moved.

use std::sync::atomic::{AtomicBool, Ordering};

use wayland_client::globals::GlobalList;
use wayland_client::{event_created_child, Connection, Dispatch, QueueHandle, WEnum};
use wayland_protocols::wp::cursor_shape::v1::client::wp_cursor_shape_device_v1::{Shape, WpCursorShapeDeviceV1};
use wayland_protocols::wp::pointer_constraints::zv1::client::{
    zwp_locked_pointer_v1::ZwpLockedPointerV1,
    zwp_pointer_constraints_v1::{Lifetime, ZwpPointerConstraintsV1},
};
use wayland_protocols::wp::relative_pointer::zv1::client::{
    zwp_relative_pointer_manager_v1::ZwpRelativePointerManagerV1,
    zwp_relative_pointer_v1::{self, ZwpRelativePointerV1},
};
use wayland_protocols::wp::tablet::zv2::client::{
    zwp_tablet_manager_v2::ZwpTabletManagerV2,
    zwp_tablet_pad_group_v2::{self, ZwpTabletPadGroupV2},
    zwp_tablet_pad_ring_v2::ZwpTabletPadRingV2,
    zwp_tablet_pad_strip_v2::ZwpTabletPadStripV2,
    zwp_tablet_pad_v2::{self, ZwpTabletPadV2},
    zwp_tablet_seat_v2::{self, ZwpTabletSeatV2},
    zwp_tablet_tool_v2::{self, ZwpTabletToolV2},
    zwp_tablet_v2::ZwpTabletV2,
};

use super::{win_of, Button, Input, State, Window};

#[derive(Default)]
pub(super) struct Stroke {
    on: Option<u64>,
    at: (f32, f32),
    pressure: f32,
    tilt: (f32, f32),
    touching: bool,
    came: bool,
    went_down: Option<u32>,
    went_up: bool,
    left: bool,
}

impl Stroke {
    pub(super) fn frame(&mut self, eraser: bool) -> Vec<(u64, Input, Option<u32>)> {
        let Some(id) = self.on else { return Vec::new() };
        let (x, y) = self.at;
        let mut out = Vec::new();
        if std::mem::take(&mut self.came) {
            out.push((id, Input::PointerIn { x, y }, None));
        } else {
            out.push((id, Input::PointerMove { x, y }, None));
        }
        if let Some(serial) = self.went_down.take() {
            self.touching = true;
            out.push((id, Input::Press { button: Button::Left, x, y }, Some(serial)));
        }
        if std::mem::take(&mut self.went_up) {
            self.touching = false;
            out.push((id, Input::Release { button: Button::Left, x, y }, None));
        }
        if std::mem::take(&mut self.left) {
            out.retain(|(_, i, _)| !matches!(i, Input::PointerMove { .. }));
            out.push((id, Input::PenOut, None));
            out.push((id, Input::PointerOut, None));
            *self = Stroke::default();
            return out;
        }
        out.push((id, Input::Pen { x, y, pressure: self.pressure, tilt: self.tilt, touching: self.touching, eraser }, None));
        out
    }
}

#[derive(Default)]
pub(super) struct Pens {
    stroke: Stroke,
    shapes: Vec<WpCursorShapeDeviceV1>,
    constraints: Option<ZwpPointerConstraintsV1>,
    relative: Option<ZwpRelativePointerManagerV1>,
    held: Option<(ZwpLockedPointerV1, ZwpRelativePointerV1)>,
}

impl Pens {
    pub(super) fn bind(globals: &GlobalList, qh: &QueueHandle<State>, seat: Option<&wayland_client::protocol::wl_seat::WlSeat>) -> Pens {
        let tablets: Option<ZwpTabletManagerV2> = globals.bind(qh, 1..=1, ()).ok();
        if let (Some(m), Some(s)) = (tablets, seat) {
            m.get_tablet_seat(s, qh, ());
        }
        Pens {
            constraints: globals.bind(qh, 1..=1, ()).ok(),
            relative: globals.bind(qh, 1..=1, ()).ok(),
            ..Pens::default()
        }
    }

    pub(super) fn let_go(&mut self, win: u64) {
        if self.held.as_ref().is_some_and(|(_, by)| wayland_client::Proxy::data::<u64>(by) == Some(&win)) {
            if let Some((lock, by)) = self.held.take() {
                lock.destroy();
                by.destroy();
            }
        }
    }
}

impl Window {
    pub fn hold_pointer(&self, on: bool) -> bool {
        let qh = self.display.queue.borrow().handle();
        let mut state = self.display.state.borrow_mut();
        if let Some((lock, by)) = state.pens.held.take() {
            lock.destroy();
            by.destroy();
        }
        if on {
            let surface = state.win(self.id).map(|w| w.surface.clone());
            let (Some(c), Some(r), Some(p), Some(surface)) = (&state.pens.constraints, &state.pens.relative, &state.pointer, surface) else { return false };
            let lock = c.lock_pointer(&surface, p, None, Lifetime::Persistent, &qh, ());
            let by = r.get_relative_pointer(p, &qh, self.id);
            state.pens.held = Some((lock, by));
        }
        drop(state);
        let _ = self.display.conn.flush();
        true
    }
}

#[derive(Default)]
pub(super) struct Tool {
    eraser: AtomicBool,
}

impl Dispatch<ZwpTabletToolV2, Tool> for State {
    fn event(state: &mut Self, tool: &ZwpTabletToolV2, event: zwp_tablet_tool_v2::Event, data: &Tool, _: &Connection, qh: &QueueHandle<Self>) {
        use zwp_tablet_tool_v2::Event as E;
        let s = &mut state.pens.stroke;
        match event {
            E::Type { tool_type } => data.eraser.store(tool_type == WEnum::Value(zwp_tablet_tool_v2::Type::Eraser), Ordering::Relaxed),
            E::ProximityIn { serial, surface, .. } => {
                (s.on, s.came) = (win_of(&surface), true);
                if let Some(m) = &state.shapes {
                    let device = m.get_tablet_tool_v2(tool, qh, ());
                    device.set_shape(serial, Shape::Default);
                    state.pens.shapes.push(device);
                }
            }
            E::ProximityOut => s.left = true,
            E::Down { serial } => {
                s.went_down = Some(serial);
                state.serial = serial;
            }
            E::Up => s.went_up = true,
            E::Motion { x, y } => s.at = (x as f32, y as f32),
            E::Pressure { pressure } => s.pressure = pressure as f32 / f32::from(u16::MAX),
            E::Tilt { tilt_x, tilt_y } => s.tilt = (tilt_x as f32, tilt_y as f32),
            E::Frame { .. } => {
                for (id, input, serial) in s.frame(data.eraser.load(Ordering::Relaxed)) {
                    match serial {
                        Some(serial) => state.inbox.push_pressed(id, input, serial),
                        None => state.inbox.push((id, input)),
                    }
                }
                if state.pens.stroke.on.is_none() {
                    state.pens.shapes.drain(..).for_each(|d| d.destroy());
                }
            }
            E::Removed => tool.destroy(),
            _ => {}
        }
    }
}

impl Dispatch<ZwpTabletSeatV2, ()> for State {
    fn event(_: &mut Self, _: &ZwpTabletSeatV2, _: zwp_tablet_seat_v2::Event, _: &(), _: &Connection, _: &QueueHandle<Self>) {}

    event_created_child!(State, ZwpTabletSeatV2, [
        zwp_tablet_seat_v2::EVT_TABLET_ADDED_OPCODE => (ZwpTabletV2, ()),
        zwp_tablet_seat_v2::EVT_TOOL_ADDED_OPCODE => (ZwpTabletToolV2, Tool::default()),
        zwp_tablet_seat_v2::EVT_PAD_ADDED_OPCODE => (ZwpTabletPadV2, ()),
    ]);
}

impl Dispatch<ZwpTabletPadV2, ()> for State {
    fn event(_: &mut Self, pad: &ZwpTabletPadV2, event: zwp_tablet_pad_v2::Event, _: &(), _: &Connection, _: &QueueHandle<Self>) {
        if let zwp_tablet_pad_v2::Event::Removed = event {
            pad.destroy();
        }
    }

    event_created_child!(State, ZwpTabletPadV2, [
        zwp_tablet_pad_v2::EVT_GROUP_OPCODE => (ZwpTabletPadGroupV2, ()),
    ]);
}

impl Dispatch<ZwpTabletPadGroupV2, ()> for State {
    fn event(_: &mut Self, _: &ZwpTabletPadGroupV2, _: zwp_tablet_pad_group_v2::Event, _: &(), _: &Connection, _: &QueueHandle<Self>) {}

    event_created_child!(State, ZwpTabletPadGroupV2, [
        zwp_tablet_pad_group_v2::EVT_RING_OPCODE => (ZwpTabletPadRingV2, ()),
        zwp_tablet_pad_group_v2::EVT_STRIP_OPCODE => (ZwpTabletPadStripV2, ()),
    ]);
}

impl Dispatch<ZwpRelativePointerV1, u64> for State {
    fn event(state: &mut Self, _: &ZwpRelativePointerV1, event: zwp_relative_pointer_v1::Event, id: &u64, _: &Connection, _: &QueueHandle<Self>) {
        if let zwp_relative_pointer_v1::Event::RelativeMotion { dx_unaccel, dy_unaccel, .. } = event {
            state.inbox.push((*id, Input::PointerBy { x: dx_unaccel as f32, y: dy_unaccel as f32 }));
        }
    }
}

macro_rules! ignore {
    ($($t:ty),*) => {$(
        impl Dispatch<$t, ()> for State {
            fn event(_: &mut Self, _: &$t, _: <$t as wayland_client::Proxy>::Event, _: &(), _: &Connection, _: &QueueHandle<Self>) {}
        }
    )*};
}
ignore!(ZwpTabletManagerV2, ZwpTabletV2, ZwpTabletPadRingV2, ZwpTabletPadStripV2, ZwpPointerConstraintsV1, ZwpLockedPointerV1, ZwpRelativePointerManagerV1);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_pen_stroke_is_a_press_and_drag_with_pressure_and_slant() {
        let mut s = Stroke { on: Some(3), came: true, at: (10.0, 20.0), ..Stroke::default() };
        assert_eq!(s.frame(false).into_iter().map(|e| e.1).collect::<Vec<_>>(),
                   [Input::PointerIn { x: 10.0, y: 20.0 }, Input::Pen { x: 10.0, y: 20.0, pressure: 0.0, tilt: (0.0, 0.0), touching: false, eraser: false }]);
        (s.went_down, s.pressure, s.tilt) = (Some(77), 0.5, (10.0, -5.0));
        let down = s.frame(false);
        assert_eq!(down[1], (3, Input::Press { button: Button::Left, x: 10.0, y: 20.0 }, Some(77)), "the press carries its serial");
        assert_eq!(down[2].1, Input::Pen { x: 10.0, y: 20.0, pressure: 0.5, tilt: (10.0, -5.0), touching: true, eraser: false });
        (s.at, s.went_up) = ((12.0, 22.0), true);
        let up = s.frame(true);
        assert_eq!(up.iter().map(|e| e.1.clone()).collect::<Vec<_>>(), [Input::PointerMove { x: 12.0, y: 22.0 },
                   Input::Release { button: Button::Left, x: 12.0, y: 22.0 },
                   Input::Pen { x: 12.0, y: 22.0, pressure: 0.5, tilt: (10.0, -5.0), touching: false, eraser: true }]);
        s.left = true;
        assert_eq!(s.frame(false).into_iter().map(|e| e.1).collect::<Vec<_>>(), [Input::PenOut, Input::PointerOut]);
        assert!(s.frame(false).is_empty(), "nothing once the pen has gone");
    }
}
