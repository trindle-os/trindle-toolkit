//! The window's own undo and redo keys, after a text box has none of its own left.

use super::*;

impl<M: Clone> Ui<M> {
    pub fn on_undo(&mut self, undo: M, redo: M) {
        self.undo = Some((undo, redo));
    }

    pub(super) fn window_undo(&mut self, sym: Keysym, mods: Mods, reply: &mut Reply<M>) -> bool {
        let Some((undo, redo)) = self.undo.clone() else { return false };
        if !mods.ctrl || mods.alt || mods.logo {
            return false;
        }
        let back = match sym {
            Keysym::z | Keysym::Z => !mods.shift,
            Keysym::y | Keysym::Y if !mods.shift => false,
            _ => return false,
        };
        if let Some(id) = self.focus.clone().filter(|id| self.is_entry(id)) {
            if self.edit(&id).is_some_and(|e| if back { e.can_undo() } else { e.can_redo() }) {
                return false;
            }
        }
        reply.messages.push(if back { undo } else { redo });
        true
    }
}
