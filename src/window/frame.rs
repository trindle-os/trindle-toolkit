//! What a window needs to draw its own title bar: whether the compositor wants it to, moving and resizing by the person's press, the window menu, and the shadow's room kept out of its geometry.

use wayland_client::protocol::wl_compositor::WlCompositor;
use wayland_client::{Connection, Dispatch, QueueHandle, WEnum};
use wayland_protocols::xdg::decoration::zv1::client::zxdg_toplevel_decoration_v1::{self, Mode, ZxdgToplevelDecorationV1};
use wayland_protocols::xdg::shell::client::xdg_toplevel::{self, ResizeEdge};

use super::{has_state, Edges, Role, State, Win, Window};
use crate::frame::Frame;

pub(super) fn resize_edge(e: Edges) -> Option<ResizeEdge> {
    Some(match e {
        Edges::TOP => ResizeEdge::Top,
        Edges::BOTTOM => ResizeEdge::Bottom,
        Edges::LEFT => ResizeEdge::Left,
        Edges::RIGHT => ResizeEdge::Right,
        e if e == Edges::TOP | Edges::LEFT => ResizeEdge::TopLeft,
        e if e == Edges::TOP | Edges::RIGHT => ResizeEdge::TopRight,
        e if e == Edges::BOTTOM | Edges::LEFT => ResizeEdge::BottomLeft,
        e if e == Edges::BOTTOM | Edges::RIGHT => ResizeEdge::BottomRight,
        _ => return None,
    })
}

pub(super) fn snug(states: &[u8]) -> bool {
    use xdg_toplevel::State as S;
    [S::Maximized, S::Fullscreen, S::TiledLeft, S::TiledRight, S::TiledTop, S::TiledBottom].iter().any(|s| has_state(states, *s as u32))
}

fn room(win: &Win) -> u32 {
    if win.snug { 0 } else { win.shadow }
}

pub(super) fn whole(win: &Win) -> (u32, u32) {
    grown(win.size, room(win))
}

pub(super) fn tell_opaque(win: &mut Win, compositor: &WlCompositor, qh: &QueueHandle<State>) {
    let (w, h) = win.size;
    let room = room(win);
    let c = win.corner.min(w / 2).min(h / 2);
    let now = [w, h, room, c, u32::from(win.solid)];
    if win.opaque_for == Some(now) {
        return;
    }
    win.opaque_for = Some(now);
    if !win.solid {
        win.surface.set_opaque_region(None);
        return;
    }
    let region = compositor.create_region(qh, ());
    let at = |v: u32| i32::try_from(v).unwrap_or(i32::MAX);
    if c > 0 {
        region.add(at(room + c), at(room), at(w - 2 * c), at(c));
        region.add(at(room + c), at(room + h - c), at(w - 2 * c), at(c));
    }
    region.add(at(room), at(room + c), at(w), at(h - 2 * c));
    win.surface.set_opaque_region(Some(&region));
    region.destroy();
}

fn grown((w, h): (u32, u32), room: u32) -> (u32, u32) {
    (w.saturating_add(room.saturating_mul(2)), h.saturating_add(room.saturating_mul(2)))
}

fn geometry((w, h): (u32, u32), room: u32) -> [i32; 4] {
    let side = |v: u32| i32::try_from(v).unwrap_or(i32::MAX);
    [side(room), side(room), side(w), side(h)]
}

pub(super) fn tell_geometry(win: &mut Win, compositor: &WlCompositor, qh: &QueueHandle<State>) {
    let now = (room(win), win.size);
    if (win.shadow == 0 && win.geometry_told.is_none()) || win.geometry_told == Some(now) {
        return;
    }
    let Role::Window { xdg, .. } = &win.role else { return };
    let [x, y, w, h] = geometry(win.size, now.0);
    xdg.set_window_geometry(x, y, w, h);
    if now.0 == 0 {
        win.surface.set_input_region(None);
    } else {
        let region = compositor.create_region(qh, ());
        region.add(x, y, w, h);
        win.surface.set_input_region(Some(&region));
        region.destroy();
    }
    win.geometry_told = Some(now);
}

impl Window {
    pub fn needs_frame(&self) -> bool {
        let mut state = self.display.state.borrow_mut();
        state.win(self.id).is_some_and(|w| matches!(&w.role, Role::Window { title_bar, .. } if title_bar.is_none() || w.frame_asked))
    }

    pub fn begin_move(&self) -> bool {
        let state = self.display.state.borrow();
        let asked = if let (Some(seat), Some(Role::Window { top, .. })) = (&state.seat, state.windows.iter().find(|w| w.id == self.id).map(|w| &w.role)) {
            top._move(seat, state.serial);
            true
        } else {
            false
        };
        drop(state);
        let _ = self.display.conn.flush();
        asked
    }

    pub fn show_window_menu(&self, x: f32, y: f32) -> bool {
        let state = self.display.state.borrow();
        let asked = if let (Some(seat), Some(Role::Window { top, .. })) = (&state.seat, state.windows.iter().find(|w| w.id == self.id).map(|w| &w.role)) {
            top.show_window_menu(seat, state.serial, x as i32, y as i32);
            true
        } else {
            false
        };
        drop(state);
        let _ = self.display.conn.flush();
        asked
    }

    pub fn set_shadow(&self, room: u32) {
        let mut state = self.display.state.borrow_mut();
        let Some(w) = state.win(self.id) else { return };
        if matches!(w.role, Role::Window { .. }) && w.shadow != room {
            w.shadow = room;
            w.dirty = true;
        }
        drop(state);
        self.display.paint();
    }

    pub fn set_corner(&self, radius: u32) {
        let mut state = self.display.state.borrow_mut();
        let Some(w) = state.win(self.id) else { return };
        w.corner = radius;
        w.dirty = true;
        drop(state);
        self.display.paint();
    }

    pub fn set_solid(&self, solid: bool) {
        if let Some(w) = self.display.state.borrow_mut().win(self.id) {
            w.solid = solid;
        }
    }

    pub fn frame(&self) -> Frame {
        let mut state = self.display.state.borrow_mut();
        state.win(self.id).map_or_else(Frame::default, |w| Frame { maximised: w.maximised, shadow: room(w) as f32, compositor_menu: w.compositor_menu })
    }

    pub fn begin_resize(&self, edges: Edges) -> bool {
        let state = self.display.state.borrow();
        let found = state.windows.iter().find(|w| w.id == self.id).map(|w| &w.role);
        let asked = if let (Some(seat), Some(Role::Window { top, .. }), Some(edge)) = (&state.seat, found, resize_edge(edges)) {
            top.resize(seat, state.serial, edge);
            true
        } else {
            false
        };
        drop(state);
        let _ = self.display.conn.flush();
        asked
    }
}

impl Dispatch<ZxdgToplevelDecorationV1, ()> for State {
    fn event(state: &mut Self, bar: &ZxdgToplevelDecorationV1, event: zxdg_toplevel_decoration_v1::Event, _: &(), _: &Connection, _: &QueueHandle<Self>) {
        let zxdg_toplevel_decoration_v1::Event::Configure { mode } = event else { return };
        let ours = state.windows.iter_mut().find(|w| matches!(&w.role, Role::Window { title_bar: Some(t), .. } if t == bar));
        if let Some(w) = ours {
            w.frame_asked = mode == WEnum::Value(Mode::ClientSide);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_edge_and_corner_has_its_resize_edge() {
        assert_eq!(resize_edge(Edges::TOP), Some(ResizeEdge::Top));
        assert_eq!(resize_edge(Edges::BOTTOM | Edges::RIGHT), Some(ResizeEdge::BottomRight));
        assert_eq!(resize_edge(Edges::LEFT | Edges::TOP), Some(ResizeEdge::TopLeft));
        assert_eq!(resize_edge(Edges::NONE), None);
        assert_eq!(resize_edge(Edges::LEFT | Edges::RIGHT), None);
    }

    fn states(list: &[u32]) -> Vec<u8> {
        list.iter().flat_map(|s| s.to_ne_bytes()).collect()
    }

    #[test]
    fn the_shadow_room_is_round_the_window_and_out_of_its_geometry() {
        assert_eq!(grown((200, 100), 24), (248, 148));
        assert_eq!(geometry((200, 100), 24), [24, 24, 200, 100]);
        assert_eq!(grown((200, 100), 0), (200, 100));
        assert_eq!(geometry((200, 100), 0), [0, 0, 200, 100]);
        assert_eq!(grown((u32::MAX, 1), 4), (u32::MAX, 9), "a huge size saturates");
    }

    #[test]
    fn maximised_full_screen_or_tiled_has_no_shadow() {
        for s in [1, 2, 5, 6, 7, 8] {
            assert!(snug(&states(&[4, s])), "state {s}");
        }
        assert!(!snug(&states(&[3, 4])), "resizing and active keep the shadow");
        assert!(!snug(&[]));
    }
}
