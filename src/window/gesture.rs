//! A touchpad's pinch and twist, as the compositor tells them (pointer gestures), given to the window as `Input::PadPinch`.

use wayland_client::globals::GlobalList;
use wayland_client::protocol::wl_pointer::WlPointer;
use wayland_client::{Connection, Dispatch, Proxy, QueueHandle};
use wayland_protocols::wp::pointer_gestures::zv1::client::{
    zwp_pointer_gesture_pinch_v1::{self, ZwpPointerGesturePinchV1},
    zwp_pointer_gestures_v1::ZwpPointerGesturesV1,
};

use super::{win_of, Input, State};
use crate::pinch::Twist;

#[derive(Default)]
pub(super) struct Pad {
    on: Option<u64>,
    scale: f32,
}

impl Pad {
    fn begin(&mut self, win: Option<u64>) {
        (self.on, self.scale) = (win, 1.0);
    }

    fn update(&mut self, moved: (f32, f32), scale: f32, degrees: f32, at: (f32, f32)) -> Option<(u64, Twist)> {
        let id = self.on?;
        let by = if self.scale > 0.0 && scale > 0.0 { scale / self.scale } else { 1.0 };
        if scale > 0.0 {
            self.scale = scale;
        }
        Some((id, Twist { scale: by, turn: degrees.to_radians(), centre: at, moved }))
    }

    fn end(&mut self, cancelled: bool) -> Option<(u64, Input)> {
        Some((self.on.take()?, Input::PadPinchEnd { cancelled }))
    }
}

#[derive(Default)]
pub(super) struct Gestures {
    manager: Option<ZwpPointerGesturesV1>,
    pinch: Option<ZwpPointerGesturePinchV1>,
    pad: Pad,
}

impl Gestures {
    pub(super) fn bind(globals: &GlobalList, qh: &QueueHandle<State>) -> Gestures {
        Gestures { manager: globals.bind(qh, 1..=3, ()).ok(), ..Gestures::default() }
    }

    pub(super) fn follow(&mut self, pointer: Option<&WlPointer>, qh: &QueueHandle<State>) {
        if let Some(p) = self.pinch.take() {
            p.destroy();
        }
        self.pad = Pad::default();
        if let (Some(m), Some(pointer)) = (&self.manager, pointer) {
            self.pinch = Some(m.get_pinch_gesture(pointer, qh, ()));
        }
    }
}

impl Dispatch<ZwpPointerGesturePinchV1, ()> for State {
    fn event(state: &mut Self, _: &ZwpPointerGesturePinchV1, event: zwp_pointer_gesture_pinch_v1::Event, _: &(), _: &Connection, _: &QueueHandle<Self>) {
        use zwp_pointer_gesture_pinch_v1::Event as E;
        let pad = &mut state.gestures.pad;
        match event {
            E::Begin { surface, .. } => pad.begin(win_of(&surface)),
            E::Update { dx, dy, scale, rotation, .. } => {
                if let Some((id, t)) = pad.update((dx as f32, dy as f32), scale as f32, rotation as f32, state.pointer_at) {
                    state.inbox.push((id, Input::PadPinch(t)));
                }
            }
            E::End { cancelled, .. } => {
                if let Some(end) = pad.end(cancelled != 0) {
                    state.inbox.push(end);
                }
            }
            _ => {}
        }
    }
}

impl Dispatch<ZwpPointerGesturesV1, ()> for State {
    fn event(_: &mut Self, _: &ZwpPointerGesturesV1, _: <ZwpPointerGesturesV1 as Proxy>::Event, _: &(), _: &Connection, _: &QueueHandle<Self>) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_touchpad_pinch_is_a_twist_from_the_last_update_about_the_pointer() {
        let mut pad = Pad::default();
        assert!(pad.update((1.0, 0.0), 2.0, 0.0, (0.0, 0.0)).is_none(), "nothing before it begins");
        pad.begin(Some(4));
        let (id, t) = pad.update((2.0, -1.0), 1.5, 90.0, (30.0, 40.0)).unwrap();
        assert_eq!(id, 4);
        assert!((t.scale - 1.5).abs() < 1e-6 && (t.turn - std::f32::consts::FRAC_PI_2).abs() < 1e-6);
        assert_eq!((t.centre, t.moved), ((30.0, 40.0), (2.0, -1.0)));
        let (_, t) = pad.update((0.0, 0.0), 3.0, -45.0, (30.0, 40.0)).unwrap();
        assert!((t.scale - 2.0).abs() < 1e-6, "the compositor's scale is from the start, the twist's from the last");
        assert!((t.turn + std::f32::consts::FRAC_PI_4).abs() < 1e-6);
        let (_, t) = pad.update((0.0, 0.0), 0.0, 0.0, (30.0, 40.0)).unwrap();
        assert_eq!(t.scale, 1.0, "a scale of nothing is no change");
        assert_eq!(pad.end(true), Some((4, Input::PadPinchEnd { cancelled: true })));
        assert_eq!(pad.end(false), None, "ended once");
        pad.begin(None);
        assert!(pad.update((0.0, 0.0), 2.0, 0.0, (0.0, 0.0)).is_none(), "a pinch on no window of ours is not heard");
    }
}
