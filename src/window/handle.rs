//! What a program asks of a window it holds, and how the window goes when it is dropped.

use wayland_protocols::xdg::decoration::zv1::client::zxdg_toplevel_decoration_v1::Mode;

use super::*;

const UNSEEN_AFTER: std::time::Duration = std::time::Duration::from_secs(1);

impl Window {
    pub fn display(&self) -> &Rc<Display> {
        &self.display
    }

    pub fn redraw(&self) {
        if let Some(w) = self.display.state.borrow_mut().win(self.id) {
            w.dirty = true;
        }
        self.display.paint();
    }

    pub fn patch(&self) {
        if let Some(w) = self.display.state.borrow_mut().win(self.id) {
            if w.gpu.is_some() {
                w.dirty = true;
            } else {
                w.patch = true;
            }
        }
        self.display.paint();
    }

    pub fn patch_moves(&self) {
        if let Some(w) = self.display.state.borrow_mut().win(self.id) {
            w.moves = true;
        }
    }

    pub fn on_patch(&self, call: impl FnMut(&mut PixmapMut, f32) -> Option<Vec<[i32; 4]>> + 'static) {
        if let Some(w) = self.display.state.borrow_mut().win(self.id) {
            w.patcher = Some(Box::new(call));
        }
    }

    pub fn on_close(&self, call: impl FnMut() + 'static) {
        if let Some(w) = self.display.state.borrow_mut().win(self.id) {
            w.closing = false;
            w.on_close = Some(Box::new(call));
        }
    }

    pub fn on_suspended(&self, call: impl FnMut(bool) + 'static) {
        if let Some(w) = self.display.state.borrow_mut().win(self.id) {
            w.told_suspended = w.suspended;
            w.on_suspended = Some(Box::new(call));
        }
    }

    pub fn on_input(&self, call: impl FnMut(Input) + 'static) {
        if let Some(w) = self.display.state.borrow_mut().win(self.id) {
            w.on_input = Some(Box::new(call));
        }
    }

    pub fn set_cursor(&self, cursor: Cursor) {
        let mut state = self.display.state.borrow_mut();
        let Some(w) = state.win(self.id) else { return };
        if w.cursor == cursor {
            return;
        }
        w.cursor = cursor;
        if state.pointer_on.is_some_and(|(id, _)| id == self.id) {
            state.show_cursor();
            drop(state);
            let _ = self.display.conn.flush();
        }
    }

    pub fn set_typing(&self, typing: Option<Typing>) {
        let mut state = self.display.state.borrow_mut();
        let Some(w) = state.win(self.id) else { return };
        if w.typing == typing {
            return;
        }
        w.typing = typing;
        if state.input_method.typer_on == Some(self.id) {
            state.tell_typer();
            drop(state);
            let _ = self.display.conn.flush();
        }
    }

    pub fn place(&self) -> Option<[i32; 4]> {
        let mut state = self.display.state.borrow_mut();
        let on = state.win(self.id)?.on.first().copied()?;
        state.screens.iter().find(|s| s.id == on).map(|s| s.place).filter(|p| p[2] > 0 && p[3] > 0)
    }

    pub fn room_tall(&self) -> Option<u32> {
        const EDGE_ROOM: u32 = 48;
        const LEAST: u32 = 120;
        self.display.places().iter().map(|p| p[3]).max().filter(|h| *h > 0).map(|h| (h as u32).saturating_sub(EDGE_ROOM).max(LEAST))
    }

    pub fn size(&self) -> (u32, u32) {
        self.display.state.borrow_mut().win(self.id).map_or((0, 0), |w| w.size)
    }

    pub fn resize(&self, size: (u32, u32)) {
        let mut state = self.display.state.borrow_mut();
        let Some(w) = state.win(self.id) else { return };
        match &w.role {
            Role::Panel(layer) => {
                layer.set_size(size.0, size.1);
                w.surface.commit();
            }
            Role::Window { .. } => {
                w.size = size;
                w.dirty = true;
            }
            Role::Lock(..) => {}
        }
        drop(state);
        self.display.paint();
    }

    pub fn set_keys(&self, keys: Keys) {
        let mut state = self.display.state.borrow_mut();
        let Some(w) = state.win(self.id) else { return };
        if let Role::Panel(layer) = &w.role {
            layer.set_keyboard_interactivity(keyboard_interactivity(keys));
            w.keys = keys;
            w.surface.commit();
        }
        drop(state);
        self.display.keys_held();
        let _ = self.display.conn.flush();
    }

    pub fn set_margins(&self, margins: [i32; 4]) {
        let mut state = self.display.state.borrow_mut();
        let Some(w) = state.win(self.id) else { return };
        if let Role::Panel(layer) = &w.role {
            let [top, right, bottom, left] = margins;
            layer.set_margin(top, right, bottom, left);
            w.surface.commit();
        }
        drop(state);
        let _ = self.display.conn.flush();
    }
}

impl Window {
    pub fn set_margins_placed(&self, margins: [i32; 4]) {
        self.set_margins(margins);
        let qh = self.display.queue.borrow().handle();
        self.display.conn.display().sync(&qh, Placed(self.id));
        let _ = self.display.conn.flush();
    }
}

impl Window {
    pub fn set_input_region(&self, rects: Option<&[[i32; 4]]>) {
        let qh = self.display.queue.borrow().handle();
        let mut state = self.display.state.borrow_mut();
        let compositor = state.compositor.clone();
        let Some(w) = state.win(self.id) else { return };
        match rects {
            Some(rects) => {
                let region = compositor.create_region(&qh, ());
                for [x, y, width, height] in rects {
                    region.add(*x, *y, *width, *height);
                }
                w.surface.set_input_region(Some(&region));
                region.destroy();
            }
            None => w.surface.set_input_region(None),
        }
        w.surface.commit();
        drop(state);
        let _ = self.display.conn.flush();
    }

    pub fn set_title(&self, title: &str) {
        let mut state = self.display.state.borrow_mut();
        let Some(w) = state.win(self.id) else { return };
        if let Role::Window { top, names, .. } = &mut w.role {
            if names.0 != no_nul(title) {
                names.0 = no_nul(title);
                top.set_title(names.0.clone());
            }
        }
        drop(state);
        let _ = self.display.conn.flush();
    }

    pub fn set_parent(&self, handle: &str) {
        let qh = self.display.queue.borrow().handle();
        let mut state = self.display.state.borrow_mut();
        let Some(importer) = state.importer.clone() else { return };
        let Some(w) = state.win(self.id) else { return };
        if !matches!(w.role, Role::Window { .. }) {
            return;
        }
        let imported = importer.import_toplevel(no_nul(handle), &qh, ());
        imported.set_parent_of(&w.surface);
        if let Some(old) = w.parent.replace(imported) {
            old.destroy();
        }
        drop(state);
        let _ = self.display.conn.flush();
    }

    pub fn set_fullscreen(&self, on: bool) {
        self.with_toplevel(|top, _| if on { top.set_fullscreen(None) } else { top.unset_fullscreen() });
    }

    pub fn set_maximised(&self, on: bool) {
        self.with_toplevel(|top, _| if on { top.set_maximized() } else { top.unset_maximized() });
    }

    pub fn maximised(&self) -> bool {
        self.display.state.borrow_mut().win(self.id).is_some_and(|w| w.maximised)
    }

    pub fn minimise(&self) {
        self.with_toplevel(|top, _| top.set_minimized());
    }

    pub fn limit(&self, least: Option<(u32, u32)>, most: Option<(u32, u32)>) {
        let (least, most) = (least.unwrap_or((0, 0)), most.unwrap_or((0, 0)));
        let up = |m: u32, l: u32| if m == 0 { 0 } else { m.max(l) };
        let most = (up(most.0, least.0), up(most.1, least.1));
        self.with_toplevel(|top, surface| {
            let side = |v: u32| i32::try_from(v).unwrap_or(i32::MAX);
            top.set_min_size(side(least.0), side(least.1));
            top.set_max_size(side(most.0), side(most.1));
            surface.commit();
        });
    }

    pub fn set_child_of(&self, parent: Option<&Window>) {
        let tops = parent.and_then(|p| match self.display.state.borrow_mut().win(p.id).map(|w| &w.role) {
            Some(Role::Window { top, .. }) => Some(top.clone()),
            _ => None,
        });
        if parent.is_some() && tops.is_none() {
            return;
        }
        self.with_toplevel(|top, _| top.set_parent(tops.as_ref()));
    }

    pub fn set_modal(&self, on: bool) {
        let qh = self.display.queue.borrow().handle();
        let mut state = self.display.state.borrow_mut();
        let Some(manager) = state.dialogs.clone() else { return };
        let Some(w) = state.win(self.id) else { return };
        let Role::Window { top, .. } = &w.role else { return };
        let dialog = w.dialog.get_or_insert_with(|| manager.get_xdg_dialog(top, &qh, ()));
        if on { dialog.set_modal() } else { dialog.unset_modal() }
        drop(state);
        let _ = self.display.conn.flush();
    }

    pub fn keep_awake(&self, on: bool) -> bool {
        let qh = self.display.queue.borrow().handle();
        let mut state = self.display.state.borrow_mut();
        let Some(manager) = state.inhibits.clone() else { return false };
        let Some(w) = state.win(self.id) else { return false };
        match (on, w.awake.take()) {
            (true, None) => w.awake = Some(manager.create_inhibitor(&w.surface, &qh, ())),
            (true, held) => w.awake = held,
            (false, held) => held.into_iter().for_each(|a| a.destroy()),
        }
        drop(state);
        let _ = self.display.conn.flush();
        true
    }

    pub fn kept_awake(&self) -> bool {
        self.display.state.borrow_mut().win(self.id).is_some_and(|w| w.awake.is_some())
    }

    pub fn set_icon(&self, name: &str, pixels: Option<&tiny_skia::Pixmap>) {
        let qh = self.display.queue.borrow().handle();
        let mut state = self.display.state.borrow_mut();
        let Some(manager) = state.icons.clone() else { return };
        let shm = state.shm.clone();
        let Some(w) = state.win(self.id) else { return };
        let Role::Window { top, .. } = &w.role else { return };
        let icon = manager.create_icon(&qh, ());
        icon.set_name(no_nul(name));
        if let Some(px) = pixels.filter(|p| p.width() == p.height()) {
            let (side, len) = (px.width(), px.data().len());
            if let Some(mut memory) = shm::Memory::new(c"trindle-icon", len) {
                memory.bytes_mut().copy_from_slice(px.data());
                to_compositor(memory.bytes_mut());
                let pool = shm.create_pool(memory.fd(), len as i32, &qh, ());
                let buffer = pool.create_buffer(0, side as i32, side as i32, side as i32 * 4, wl_shm::Format::Argb8888, &qh, (0, 0));
                icon.add_buffer(&buffer, 1);
                pool.destroy();
                manager.set_icon(top, Some(&icon));
                icon.destroy();
                buffer.destroy();
                drop(state);
                let _ = self.display.conn.flush();
                return;
            }
        }
        manager.set_icon(top, Some(&icon));
        icon.destroy();
        drop(state);
        let _ = self.display.conn.flush();
    }

    fn with_toplevel(&self, f: impl FnOnce(&xdg_toplevel::XdgToplevel, &wl_surface::WlSurface)) {
        let mut state = self.display.state.borrow_mut();
        if let Some(Win { role: Role::Window { top, .. }, surface, .. }) = state.win(self.id) {
            f(top, surface);
        }
        drop(state);
        let _ = self.display.conn.flush();
    }

    pub fn seen(&self) -> bool {
        let mut state = self.display.state.borrow_mut();
        let Some(w) = state.win(self.id) else { return false };
        let stalled = w.waiting && w.sent.is_some_and(|t| t.elapsed() > UNSEEN_AFTER);
        !w.hidden && w.configured && !w.suspended && !stalled
    }

    pub fn expect_pass(&self, pass: &str) {
        let plain = !pass.is_empty() && pass.len() <= 256 && pass.bytes().all(|b| b.is_ascii_graphic());
        self.display.state.borrow_mut().pass = plain.then(|| pass.to_string());
    }

    pub fn raise(&self) {
        let used = {
            let mut state = self.display.state.borrow_mut();
            let pass = state.pass.take();
            let surface = state.win(self.id).map(|w| w.surface.clone());
            match (pass, &state.activation, surface) {
                (Some(pass), Some(m), Some(surface)) => {
                    m.activate(pass, &surface);
                    true
                }
                _ => false,
            }
        };
        if used {
            let _ = self.display.conn.flush();
        } else {
            self.set_visible(false);
            self.set_visible(true);
        }
    }

    pub fn set_visible(&self, yes: bool) {
        let mut state = self.display.state.borrow_mut();
        let Some(w) = state.win(self.id) else { return };
        w.hidden = !yes;
        if yes {
            if let Role::Window { top, title_bar, names: (title, app_id), .. } = &w.role {
                top.set_title(title.clone());
                top.set_app_id(app_id.clone());
                if let Some(t) = title_bar {
                    t.set_mode(Mode::ServerSide);
                }
            }
            w.surface.commit();
        } else {
            w.surface.attach(None, 0, 0);
            w.surface.commit();
            if std::mem::take(&mut w.mapped) {
                w.configured = false;
                w.void += 1;
                self.display.conn.display().sync(&self.display.queue.borrow().handle(), Unvoid(self.id));
            }
            w.waiting = false;
            w.sent = None;
        }
        drop(state);
        self.display.trim_unseen();
        self.display.keys_held();
        let _ = self.display.conn.flush();
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        let mut state = self.display.state.borrow_mut();
        let Some(at) = state.windows.iter().position(|w| w.id == self.id) else { return };
        let w = state.windows.remove(at);
        state.inbox.forget(self.id);
        state.touches.retain(|(_, id)| *id != self.id);
        if state.pointer_on.is_some_and(|(id, _)| id == self.id) {
            state.pointer_on = None;
        }
        if state.keys_on == Some(self.id) {
            state.keys_on = None;
            state.stop_repeat();
        }
        if state.input_method.typer_on == Some(self.id) {
            state.tell_typer();
            state.input_method.typer_on = None;
        }
        drag::forget(&mut state, self.id);
        state.pens.let_go(self.id);
        drop(state);
        if let Some(p) = w.parent {
            p.destroy();
        }
        if let Some(f) = w.fractional {
            f.destroy();
        }
        if let Some(v) = w.viewport {
            v.destroy();
        }
        if let Some(d) = w.dialog {
            d.destroy();
        }
        if let Some(a) = w.awake {
            a.destroy();
        }
        match w.role {
            Role::Window { xdg, top, title_bar, .. } => {
                if let Some(t) = title_bar {
                    t.destroy();
                }
                top.destroy();
                xdg.destroy();
            }
            Role::Panel(layer) => layer.destroy(),
            Role::Lock(lock, screen) => {
                lock.destroy();
                if let Some(h) = self.display.state.borrow_mut().lock.as_mut() {
                    h.seq.uncovered(&screen);
                }
            }
        }
        w.surface.destroy();
        if let Some(b) = w.buffers {
            b.buffers.iter().for_each(|b| b.destroy());
        }
        let _ = self.display.conn.flush();
    }
}
