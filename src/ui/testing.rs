//! What the tests of the toolkit's and of other programs' windows need to see inside a Ui.

use super::*;

impl<M: Clone> Ui<M> {
    pub fn squeezed_or_touching(&mut self) -> Vec<String> {
        self.settle();
        let named = self.labels == Labels::Always;
        let mut parts: Vec<(String, Rect)> = Vec::new();
        let mut wrong = Vec::new();
        for s in &self.spots {
            let (name, least) = match self.tree.at(&s.path).map(|n| &n.what) {
                Some(What::Button { words: None, tip, size, .. }) => (tip.clone(), self.px(size.unwrap_or(theme::icon_button()))),
                Some(What::Switch { tip, .. }) => (tip.clone(), self.px(theme::sheet().switch.0)),
                Some(What::Tick { tip, .. }) => (tip.clone(), self.px(theme::sheet().tick)),
                _ => continue,
            };
            if s.rect.w + 0.5 < least {
                wrong.push(format!("{name:?} is drawn {least} wide in a room of {}{}", s.rect.w, if named { ", with labels kept" } else { "" }));
            }
            parts.push((name, s.rect));
        }
        for (i, (a, ra)) in parts.iter().enumerate() {
            for (b, rb) in &parts[i + 1..] {
                let over = ra.intersect(rb);
                if over.w > 0.5 && over.h > 0.5 {
                    wrong.push(format!("{a:?} and {b:?} overlap by {} across and {} down{}", over.w, over.h, if named { ", with labels kept" } else { "" }));
                }
            }
        }
        wrong
    }
}

impl<M: Clone> Ui<M> {
    pub fn round_and_worded(&mut self) -> Vec<(String, Rect)> {
        self.settle();
        self.spots.iter().filter_map(|s| match self.tree.at(&s.path).map(|n| &n.what) {
            Some(What::Button { tip, .. }) => Some((tip.clone(), s.rect)),
            _ => None,
        }).collect()
    }
}

impl<M: Clone> Ui<M> {
    pub fn lay_at(&mut self, w: u32, h: u32) {
        self.size = (w, h);
        self.laid = None;
        self.settle();
    }

    pub fn still(&mut self) {
        self.moving = false;
    }

    pub fn label_at(&mut self, key: &str) -> Option<String> {
        if self.spots.is_empty() {
            self.lay_at(1280, 720);
        }
        let id = self.spots.iter().find(|s| s.id.ends_with(key))?.id.clone();
        self.label_of(&id)
    }

    pub fn boxes(&self) -> Vec<(String, Rect, Option<Rect>)> {
        self.spots.iter().map(|s| (s.id.clone(), s.rect, s.clip)).collect()
    }

    pub fn pressables(&self) -> Vec<(String, &'static str)> {
        self.spots.iter().filter_map(|s| {
            let n = self.tree.at(&s.path).filter(|n| n.focusable())?;
            let kind = match n.what {
                What::Button { .. } => "button",
                What::Switch { .. } => "switch",
                What::Tick { .. } => "tick",
                What::Choice { .. } => "choice",
                What::Slider { .. } => "slider",
                What::Dropdown { .. } => "dropdown",
                _ => return None,
            };
            Some((s.id.clone(), kind))
        }).collect()
    }

    pub fn to_the_ends(&mut self) {
        self.settle();
        for (id, _, most) in self.places() {
            self.scroll_area(&id, most);
        }
    }

    pub fn moved_by(&mut self, next: Node<M>) -> Vec<String> {
        let (w, h) = self.size;
        let (boxes, places) = (self.boxes(), self.places());
        self.set_view(next);
        self.lay_at(w, h);
        let mut out = Vec::new();
        let now = self.places();
        for p in &places {
            match now.iter().find(|q| q.0 == p.0) {
                Some(q) if q.1 == p.1 => {}
                Some(q) => out.push(format!("{} scrolled from {} to {}", p.0, p.1, q.1)),
                None => out.push(format!("{} is gone", p.0)),
            }
        }
        let now = self.boxes();
        let named = |id: &str| id.chars().any(|c| !c.is_ascii_digit() && c != '.');
        for b in boxes.iter().filter(|b| named(&b.0)) {
            if now.iter().find(|n| n.0 == b.0).is_some_and(|n| n != b) {
                out.push(format!("{} moved", b.0));
            }
        }
        out
    }

    pub fn puzzle_cards(&self) -> Vec<Rect> {
        self.puzzle_cards_named().into_iter().map(|(_, r)| r).collect()
    }

    pub fn puzzle_cards_named(&self) -> Vec<(Option<String>, Rect)> {
        let in_puzzle = |path: &[usize]| path.split_last()
            .and_then(|(_, above)| self.tree.at(above))
            .is_some_and(|n| matches!(n.what, What::Grid { puzzle: true, even: false, .. }));
        let mut cards: Vec<&Spot> = self.spots.iter().filter(|s| in_puzzle(&s.path)).collect();
        cards.sort_by(|a, b| a.path.cmp(&b.path));
        cards.into_iter().map(|s| (self.tree.at(&s.path).and_then(|n| n.key.clone()), s.rect)).collect()
    }

    pub fn keyboard_on(&self) -> Option<String> {
        self.focus.clone()
    }

    pub fn places(&self) -> Vec<(String, f32, f32)> {
        self.spots.iter().filter(|s| self.is_scroll(s)).map(|s| (s.id.clone(), self.offset(s), (s.full - s.rect.h).max(0.0).round())).collect()
    }

    pub fn pill_boxes(&self, id: &str) -> Vec<Rect> {
        self.pill_rects(id)
    }

    pub fn open_rows(&self) -> Vec<String> {
        let Some(d) = &self.drop else { return Vec::new() };
        self.source_of(&d.id, d.part).map(|s| s.options.to_vec()).unwrap_or_default()
    }

    pub fn dropdowns(&self) -> Vec<(String, bool)> {
        self.spots.iter().filter_map(|s| {
            let n = self.tree.at(&s.path)?;
            matches!(n.what, What::Dropdown { .. }).then(|| (s.id.clone(), n.focusable()))
        }).collect()
    }

    pub fn keyboard_to(&mut self, id: &str) {
        self.settle();
        self.focus = Some(id.to_string());
        self.reveal(id, None);
        self.settle();
    }

    pub fn open_list(&self) -> Option<(Rect, Rect)> {
        let d = self.drop.as_ref()?;
        Some((self.drop_laid()?.panel, self.source_of(&d.id, d.part)?.anchor))
    }
}
