//! Keys that move, press or change what has the keyboard.

use super::*;

impl<M: Clone> Ui<M> {
    pub(super) fn key(&mut self, sym: Keysym, text: Option<&str>, mods: Mods, now: Instant, reply: &mut Reply<M>) {
        if sym == Keysym::Tab || sym == Keysym::ISO_Left_Tab {
            self.tab_key(sym, mods, now, reply);
            return;
        }
        if sym == Keysym::Escape {
            if let Some(m) = self.escape() {
                reply.messages.push(m);
            }
            return;
        }
        let focus = self.focus.clone();
        if focus.as_deref().is_some_and(|id| self.eye_key(id, sym, text, mods, now, reply)) {
            return;
        }
        let in_entry = focus.as_deref().is_some_and(|id| self.is_entry(id));
        if (sym == Keysym::Menu || (sym == Keysym::F10 && mods.shift && !mods.ctrl)) && self.menu_from_key() {
            reply.redraw = true;
            return;
        }
        if !in_entry && (self.pick_key(sym, mods, reply) || self.pick_move(sym, mods, reply)) {
            return;
        }
        if self.window_undo(sym, mods, reply) {
            return;
        }
        if focus.as_deref().is_some_and(|id| self.widget_key(id, sym, text, mods, now, reply)) {
            return;
        }
        if focus.as_deref().is_some_and(|id| self.along_key(id, sym, reply)) {
            return;
        }
        if focus.as_deref().is_some_and(|id| self.radio_key(id, sym, reply)) {
            return;
        }
        if focus.as_deref().is_some_and(|id| self.choice_key(id, sym, reply)) {
            return;
        }
        if let Some(id) = focus.as_deref().filter(|id| self.is_dropdown(id)) {
            if matches!(sym, Keysym::Down | Keysym::KP_Down | Keysym::Up | Keysym::KP_Up) {
                self.open_drop(id);
                reply.redraw = true;
                return;
            }
        }
        if self.along_area_key(sym, mods, reply) || self.page_key(sym, reply) {
            return;
        }
        if let Some(id) = focus {
            if matches!(sym, Keysym::space | Keysym::Return | Keysym::KP_Enter | Keysym::KP_Space) {
                if let Some(r) = self.spot(&id).map(|s| s.rect).filter(|_| matches!(self.node(&id).map(|n| &n.what), Some(What::Drawing { keys: Some(_), click: None, .. }))) {
                    if self.drawing_menu(&id, r.x + r.w / 2.0, r.y + r.h / 2.0, true) {
                        reply.redraw = true;
                        return;
                    }
                }
                self.activate(&id, now, reply);
            }
        }
    }

    fn widget_key(&mut self, id: &str, sym: Keysym, text: Option<&str>, mods: Mods, now: Instant, reply: &mut Reply<M>) -> bool {
        if self.is_entry(id) {
            if let Some(m) = self.step_key(id, sym, mods) {
                reply.messages.push(m);
                reply.redraw = true;
            } else {
                self.edit_key(id, sym, text, mods, reply);
            }
            return true;
        }
        if self.is_list(id) && !mods.ctrl && !mods.alt && !mods.logo
            && text.is_some_and(|t| self.list_ahead(id, t, now, reply)) {
            return true;
        }
        if self.is_list(id) && self.list_key(id, sym, mods, reply) {
            return true;
        }
        self.slider_key(id, sym, reply) || self.divider_key(id, sym, reply)
    }

    pub(super) fn tab_key(&mut self, sym: Keysym, mods: Mods, now: Instant, reply: &mut Reply<M>) {
        let forward = !mods.shift && sym == Keysym::Tab;
        if let Some(What::Entry { tab: Some(f), .. }) = self.focus.as_deref().and_then(|id| self.node(id)).map(|n| &n.what) {
            if !mods.ctrl && !mods.alt && !mods.logo {
                reply.messages.push(f(forward));
                reply.redraw = true;
                return;
            }
        }
        if self.rich_tab(forward) {
            reply.redraw = true;
            return;
        }
        self.move_focus(forward);
        self.rich_arrive(forward);
        if let Some(id) = self.focus.clone() {
            self.reveal(&id, None);
        }
        self.tip = self.focus.clone().filter(|id| self.node(id).is_some_and(|n| n.tip().is_some())).map(|id| Tip { id, state: TipState::Shown(now) });
        reply.redraw = true;
    }

    fn step_key(&self, id: &str, sym: Keysym, mods: Mods) -> Option<M> {
        let Some(What::Entry { steps: Some(f), .. }) = self.node(id).map(|n| &n.what) else { return None };
        let by = match sym {
            Keysym::Up | Keysym::KP_Up => 1,
            Keysym::Down | Keysym::KP_Down => -1,
            Keysym::Page_Up | Keysym::KP_Page_Up => PAGE_STEPS as i32,
            Keysym::Page_Down | Keysym::KP_Page_Down => -(PAGE_STEPS as i32),
            _ => return None,
        };
        (!mods.ctrl && !mods.alt && !mods.shift).then(|| f(by))
    }

    pub(super) fn slider_key(&self, id: &str, sym: Keysym, reply: &mut Reply<M>) -> bool {
        let Some(What::Slider { value, min, max, step, change, .. }) = self.node(id).map(|n| &n.what) else { return false };
        let to = match sym {
            Keysym::Left | Keysym::Down | Keysym::KP_Left | Keysym::KP_Down => Some(value - step),
            Keysym::Right | Keysym::Up | Keysym::KP_Right | Keysym::KP_Up => Some(value + step),
            Keysym::Page_Down | Keysym::KP_Page_Down => Some(value - step * PAGE_STEPS),
            Keysym::Page_Up | Keysym::KP_Page_Up => Some(value + step * PAGE_STEPS),
            Keysym::Home | Keysym::KP_Home => Some(*min),
            Keysym::End | Keysym::KP_End => Some(*max),
            _ => None,
        };
        if let Some(to) = to {
            let v = snap(to, *min, *max, *step);
            if v != *value {
                reply.messages.push(change(v));
            }
            return true;
        }
        false
    }

    pub(super) fn choice_key(&self, id: &str, sym: Keysym, reply: &mut Reply<M>) -> bool {
        let Some(What::Choice { options, chosen, choose, .. }) = self.node(id).map(|n| &n.what) else { return false };
        let last = options.len().saturating_sub(1);
        let to = match sym {
            Keysym::Left | Keysym::Up | Keysym::KP_Left | Keysym::KP_Up => Some(chosen.saturating_sub(1)),
            Keysym::Right | Keysym::Down | Keysym::KP_Right | Keysym::KP_Down => Some((chosen + 1).min(last)),
            Keysym::Home | Keysym::KP_Home => Some(0),
            Keysym::End | Keysym::KP_End => Some(last),
            _ => None,
        };
        if let Some(to) = to {
            if to != *chosen {
                reply.messages.push(choose(to));
            }
            return true;
        }
        false
    }

    pub(super) fn escape(&self) -> Option<M> {
        let path = self.focus.as_deref().and_then(|id| self.spot(id)).map(|s| s.path.clone()).unwrap_or_default();
        (0..=path.len()).rev().find_map(|n| self.tree.at(&path[..n]).and_then(|w| w.escape.clone()))
    }
}
