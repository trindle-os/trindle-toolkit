//! The seat: the pointer, keyboard and touch as the compositor tells of them, turned into waiting input.

use wayland_client::protocol::wl_seat;
use wayland_client::WEnum;
use wayland_protocols::wp::cursor_shape::v1::client::wp_cursor_shape_device_v1::Shape;

use super::*;

impl State {
    pub(super) fn stop_repeat(&mut self) {
        self.held = None;
        self.held_count += 1;
        self.start_repeat = false;
    }

    pub(super) fn show_cursor(&mut self) {
        let Some((id, serial)) = self.pointer_on else { return };
        let cursor = self.windows.iter().find(|w| w.id == id).map(|w| w.cursor).unwrap_or_default();
        if let Some(device) = &self.shape {
            device.set_shape(serial, to_shape(cursor));
        }
    }
}

fn to_shape(cursor: Cursor) -> Shape {
    match cursor {
        Cursor::Arrow => Shape::Default,
        Cursor::Hand => Shape::Pointer,
        Cursor::Text => Shape::Text,
        Cursor::Crosshair => Shape::Crosshair,
        Cursor::Move => Shape::Move,
        Cursor::NResize => Shape::NResize,
        Cursor::NeResize => Shape::NeResize,
        Cursor::EResize => Shape::EResize,
        Cursor::SeResize => Shape::SeResize,
        Cursor::SResize => Shape::SResize,
        Cursor::SwResize => Shape::SwResize,
        Cursor::WResize => Shape::WResize,
        Cursor::NwResize => Shape::NwResize,
        Cursor::ColResize => Shape::ColResize,
        Cursor::RowResize => Shape::RowResize,
    }
}

impl Dispatch<wl_seat::WlSeat, ()> for State {
    fn event(state: &mut Self, seat: &wl_seat::WlSeat, event: wl_seat::Event, _: &(), _: &Connection, qh: &QueueHandle<Self>) {
        let wl_seat::Event::Capabilities { capabilities: WEnum::Value(has) } = event else { return };
        let modern = seat.version() >= 3;
        let has_pointer = has.contains(wl_seat::Capability::Pointer);
        if has_pointer && state.pointer.is_none() {
            let pointer = seat.get_pointer(qh, ());
            state.shape = state.shapes.as_ref().map(|m| m.get_pointer(&pointer, qh, ()));
            state.gestures.follow(Some(&pointer), qh);
            state.pointer = Some(pointer);
        } else if !has_pointer {
            state.gestures.follow(None, qh);
            if let Some(d) = state.shape.take() {
                d.destroy();
            }
            if let Some(p) = state.pointer.take().filter(|_| modern) {
                p.release();
            }
            state.pointer_on = None;
        }
        let has_keyboard = has.contains(wl_seat::Capability::Keyboard);
        if has_keyboard && state.keyboard.is_none() {
            state.keyboard = Some(seat.get_keyboard(qh, ()));
        } else if !has_keyboard {
            if let Some(k) = state.keyboard.take().filter(|_| modern) {
                k.release();
            }
            state.keys_on = None;
            state.stop_repeat();
        }
        let has_touch = has.contains(wl_seat::Capability::Touch);
        if has_touch && state.touch.is_none() {
            state.touch = Some(seat.get_touch(qh, ()));
        } else if !has_touch {
            if let Some(t) = state.touch.take().filter(|_| modern) {
                t.release();
            }
            state.touches.clear();
        }
    }
}

impl Dispatch<wl_pointer::WlPointer, ()> for State {
    fn event(state: &mut Self, pointer: &wl_pointer::WlPointer, event: wl_pointer::Event, _: &(), _: &Connection,
             _: &QueueHandle<Self>) {
        use wl_pointer::Event as E;
        let mut whole = pointer.version() < 5;
        match event {
            E::Enter { serial, surface, surface_x, surface_y } => {
                let Some(id) = win_of(&surface) else { return };
                drag::pointer_back(state);
                state.pointer_on = Some((id, serial));
                state.pointer_at = (surface_x as f32, surface_y as f32);
                state.show_cursor();
                state.inbox.push((id, Input::PointerIn { x: surface_x as f32, y: surface_y as f32 }));
            }
            E::Leave { .. } => {
                if let Some((id, _)) = state.pointer_on.take() {
                    state.inbox.push((id, Input::PointerOut));
                }
            }
            E::Motion { surface_x, surface_y, .. } => {
                let (x, y) = (surface_x as f32, surface_y as f32);
                state.pointer_at = (x, y);
                if let Some((id, _)) = state.pointer_on {
                    state.inbox.push((id, Input::PointerMove { x, y }));
                }
            }
            E::Button { serial, button, state: pressed, .. } => {
                state.serial = serial;
                note_code(&CODES, serial, button);
                let down = pressed == WEnum::Value(wl_pointer::ButtonState::Pressed);
                state.pressed = down.then(|| (serial, Button::from_code(button)));
                let Some((id, _)) = state.pointer_on else { return };
                let (button, (x, y)) = (Button::from_code(button), state.pointer_at);
                let input = match pressed {
                    WEnum::Value(wl_pointer::ButtonState::Pressed) => Input::Press { button, x, y },
                    _ => Input::Release { button, x, y },
                };
                state.inbox.push_pressed(id, input, serial);
            }
            E::Axis { axis: WEnum::Value(axis), value, .. } => state.scrolled[axis_index(axis)] += value as f32,
            E::AxisDiscrete { axis: WEnum::Value(axis), discrete } => state.steps[axis_index(axis)] += discrete as f32,
            E::AxisValue120 { axis: WEnum::Value(axis), value120 } => {
                state.steps[axis_index(axis)] += value120 as f32 / 120.0;
            }
            E::AxisStop { .. } => state.stopped = true,
            E::Frame => whole = true,
            _ => {}
        }
        if whole {
            scroll_whole(state);
        }
    }
}

fn scroll_whole(state: &mut State) {
    let (scrolled, steps) = (std::mem::take(&mut state.scrolled), std::mem::take(&mut state.steps));
    let stopped = std::mem::take(&mut state.stopped);
    let Some((id, _)) = state.pointer_on else { return };
    let input = if steps != [0.0; 2] {
        Input::Scroll { x: steps[0], y: steps[1], steps: true }
    } else if scrolled != [0.0; 2] || stopped {
        Input::Scroll { x: scrolled[0], y: scrolled[1], steps: false }
    } else {
        return;
    };
    state.inbox.push((id, input));
}

fn axis_index(axis: wl_pointer::Axis) -> usize {
    match axis {
        wl_pointer::Axis::HorizontalScroll => 0,
        _ => 1,
    }
}

impl Dispatch<wl_keyboard::WlKeyboard, ()> for State {
    fn event(state: &mut Self, _: &wl_keyboard::WlKeyboard, event: wl_keyboard::Event, _: &(), _: &Connection,
             _: &QueueHandle<Self>) {
        use wl_keyboard::Event as E;
        match event {
            E::Keymap { format: WEnum::Value(wl_keyboard::KeymapFormat::XkbV1), fd, size } => {
                state.layout.get_or_insert_with(Keyboard::new).set_keymap(fd, size);
            }
            E::Enter { serial, surface, .. } => {
                state.serial = serial;
                let Some(id) = win_of(&surface) else { return };
                state.keys_on = Some(id);
                state.inbox.push((id, Input::Focus(true)));
            }
            E::Leave { .. } => {
                state.stop_repeat();
                if let Some(id) = state.keys_on.take() {
                    state.inbox.push((id, Input::Focus(false)));
                }
            }
            E::Key { serial, key, state: pressed, .. } => {
                state.serial = serial;
                note_code(&CODES, serial, key);
                let (Some(id), Some(layout)) = (state.keys_on, state.layout.as_mut()) else { return };
                if !layout.has_layout() {
                    if pressed == WEnum::Value(wl_keyboard::KeyState::Pressed) {
                        state.inbox.push((id, Input::NoLayout));
                    }
                    return;
                }
                if pressed == WEnum::Value(wl_keyboard::KeyState::Pressed) {
                    let Some(input) = layout.press(key, false) else { return };
                    let repeats = layout.repeats(key);
                    state.inbox.push_pressed(id, input, serial);
                    state.stop_repeat();
                    if repeats {
                        state.held_serial = serial;
                        state.held = Some(key);
                        state.start_repeat = true;
                    }
                } else {
                    let input = layout.release(key);
                    if state.held == Some(key) {
                        state.stop_repeat();
                    }
                    state.inbox.extend(input.map(|i| (id, i)));
                }
            }
            E::Modifiers { mods_depressed, mods_latched, mods_locked, group, .. } => {
                if let Some(k) = &mut state.layout {
                    k.set_mods(mods_depressed, mods_latched, mods_locked, group);
                    if let Some(id) = state.keys_on {
                        state.inbox.push((id, Input::Mods(k.mods())));
                    }
                }
            }
            E::RepeatInfo { rate, delay } => state.layout.get_or_insert_with(Keyboard::new).set_repeat(rate, delay),
            _ => {}
        }
    }
}

impl Dispatch<wl_touch::WlTouch, ()> for State {
    fn event(state: &mut Self, _: &wl_touch::WlTouch, event: wl_touch::Event, _: &(), _: &Connection, _: &QueueHandle<Self>) {
        use wl_touch::Event as E;
        let on = |state: &State, touch: i32| state.touches.iter().find(|(t, _)| *t == touch).map(|(_, w)| *w);
        match event {
            E::Down { serial, surface, id, x, y, .. } => {
                state.serial = serial;
                let Some(win) = win_of(&surface) else { return };
                state.touches.retain(|(t, _)| *t != id);
                state.touches.push((id, win));
                state.touched.retain(|(t, _)| *t != id);
                state.touched.push((id, serial));
                state.inbox.push_pressed(win, Input::TouchDown { id, x: x as f32, y: y as f32 }, serial);
            }
            E::Motion { id, x, y, .. } => {
                if let Some(win) = on(state, id) {
                    state.inbox.push((win, Input::TouchMove { id, x: x as f32, y: y as f32 }));
                }
            }
            E::Up { id, .. } => {
                if let Some(win) = on(state, id) {
                    state.touches.retain(|(t, _)| *t != id);
                    let down = state.touched.iter().position(|(t, _)| *t == id).map(|at| state.touched.remove(at).1);
                    match down {
                        Some(serial) => state.inbox.push_pressed(win, Input::TouchUp { id }, serial),
                        None => state.inbox.push((win, Input::TouchUp { id })),
                    }
                }
            }
            E::Cancel => {
                state.touched.clear();
                let mut wins: Vec<u64> = state.touches.drain(..).map(|(_, w)| w).collect();
                wins.sort_unstable();
                wins.dedup();
                state.inbox.extend(wins.into_iter().map(|w| (w, Input::TouchCancel)));
            }
            _ => {}
        }
    }
}
