//! Sideways scrolling areas by finger and keys.

use super::*;

impl<M: Clone> Ui<M> {
    pub(super) fn areas_x_round(&self, id: &str) -> Vec<String> {
        let Some(path) = self.spot(id).map(|s| s.path.clone()) else { return Vec::new() };
        self.spots.iter().rev()
            .filter(|s| s.path.len() < path.len() && path.starts_with(&s.path) && self.is_scroll_x(s))
            .map(|s| s.id.clone()).collect()
    }

    pub(super) fn along_area_key(&mut self, sym: Keysym, mods: Mods, reply: &mut Reply<M>) -> bool {
        let area = self.focus.as_deref().and_then(|f| self.areas_x_round(f).into_iter().next())
            .or_else(|| self.pointer.and_then(|(x, y)| self.areas_x_at(x, y).into_iter().next()));
        let Some(area) = area else { return false };
        let Some(s) = self.spot(&area) else { return false };
        let (off, w) = (self.offset(s), s.rect.w);
        let step = notch(w, self.scale) * self.scale;
        let to = match sym {
            Keysym::Left | Keysym::KP_Left if !mods.shift => off - step,
            Keysym::Right | Keysym::KP_Right if !mods.shift => off + step,
            Keysym::Page_Up | Keysym::KP_Page_Up if mods.shift => off - self.page(w),
            Keysym::Page_Down | Keysym::KP_Page_Down if mods.shift => off + self.page(w),
            Keysym::Home | Keysym::KP_Home if mods.shift => 0.0,
            Keysym::End | Keysym::KP_End if mods.shift => s.full,
            _ => return false,
        };
        if self.scroll_area(&area, to) {
            reply.redraw = true;
            self.rehover(reply);
        }
        true
    }

    pub(super) fn touch_down_x(&mut self, x: f32, y: f32) {
        self.swipe_x = None;
        if self.dragging.is_none() && self.thumb.is_none() {
            self.swipe_x = self.areas_x_at(x, y).into_iter().next()
                .and_then(|a| self.spot(&a).map(|s| (a.clone(), (x, y), self.offset(s), false)));
        }
    }

    pub(super) fn touch_move_x(&mut self, x: f32, y: f32, reply: &mut Reply<M>) -> bool {
        let Some((area, (x0, y0), off0, started)) = self.swipe_x.clone() else { return false };
        if !started {
            let (dx, dy) = ((x - x0).abs(), (y - y0).abs());
            if self.swipe.as_ref().is_some_and(|s| s.3) || dx <= SWIPE_SLOP * self.scale || dx < dy {
                return false;
            }
            self.swipe_x = Some((area.clone(), (x0, y0), off0, true));
            self.swipe = None;
            self.pressed = None;
            self.eyes.down = None;
            self.hover = None;
            reply.redraw = true;
        }
        self.pointer = Some((x, y));
        reply.redraw |= self.scroll_area(&area, off0 + x0 - x);
        true
    }
}
