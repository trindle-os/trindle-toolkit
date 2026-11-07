//! The keys of a table's rows: moving, opening and closing, and choosing several.

use super::super::input::{Keysym, Mods};
use super::{Table, Touch};

impl Table {
    pub fn key(&mut self, sym: Keysym, mods: Mods, tall: f32) -> Option<Touch> {
        let extend = self.multiple && mods.shift;
        let page = self.rows_in(tall).max(1) as isize;
        let by = match sym {
            Keysym::Up | Keysym::KP_Up => -1,
            Keysym::Down | Keysym::KP_Down => 1,
            Keysym::Page_Up | Keysym::KP_Page_Up => -page,
            Keysym::Page_Down | Keysym::KP_Page_Down => page,
            Keysym::Home | Keysym::KP_Home => isize::MIN / 2,
            Keysym::End | Keysym::KP_End => {
                self.chosen.get_or_insert(0);
                isize::MAX / 2
            }
            Keysym::a | Keysym::A if self.multiple && mods.ctrl => {
                self.pick_all();
                return Some(Touch::Picked);
            }
            _ => return self.branch_key(sym, tall),
        };
        let at = self.step_with(by, tall, extend)?;
        Some(if extend { Touch::Extended(at) } else { Touch::Chose(at) })
    }

    fn branch_key(&mut self, sym: Keysym, tall: f32) -> Option<Touch> {
        let row = self.chosen.filter(|&r| r < self.rows)?;
        let open = self.is_open(row);
        match sym {
            Keysym::Right | Keysym::KP_Right => match open? {
                false => Some(Touch::Open(row)),
                true => {
                    let first = row + 1;
                    let inside = self.branch(first).zip(self.branch(row)).is_some_and(|(c, b)| c.depth > b.depth);
                    inside.then(|| Touch::Chose(self.go_to(first, tall, false)))
                }
            },
            Keysym::Left | Keysym::KP_Left => {
                if open == Some(true) {
                    self.close_row(row);
                    return Some(Touch::Closed(row));
                }
                let parent = self.parent_of(row)?;
                Some(Touch::Chose(self.go_to(parent, tall, false)))
            }
            Keysym::space | Keysym::KP_Space if self.multiple => {
                self.pick_toggle(row);
                Some(Touch::Toggled(row))
            }
            Keysym::Return | Keysym::KP_Enter => Some(self.toggle_row(row).unwrap_or(Touch::Opened(row))),
            Keysym::space | Keysym::KP_Space if self.is_tree() => Some(self.toggle_row(row).unwrap_or(Touch::Opened(row))),
            _ => None,
        }
    }
}
