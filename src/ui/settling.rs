//! A new view taken in, and the widgets laid out again for it or for a new size.

use super::*;

impl<M: Clone> Ui<M> {
    pub(super) fn settle(&mut self) {
        let now = (self.size.0, self.size.1, self.scale, self.fonts.generation());
        if self.laid == Some(now) {
            return;
        }
        self.texts.turn();
        let lay = Lay { fonts: &self.fonts, texts: &self.texts, scale: self.scale, scrolled: &self.scrolls, splits: Some(&self.splits.places), natural: Default::default(), bento: Default::default(),
                        named: self.labels == Labels::Always, whole: Default::default(), puzzles: puzzle_names(&self.tree), held: Some(&self.held), stays: Some(&self.stays) };
        let mut spots = Vec::new();
        let rect = Rect { x: 0.0, y: 0.0, w: self.size.0 as f32, h: self.size.1 as f32 };
        lay.place(&self.tree, rect, &mut Vec::new(), &mut spots);
        let puzzles: Vec<String> = lay.puzzles.into_values().collect();
        self.stays.settle();
        self.spots = spots;
        self.laid = Some(now);
        self.forget_what_went();
        self.keep_places(puzzles);
        self.eyes_settle();
        self.settle_edits();
        self.settle_focus();
        self.keep_drop();
    }

    fn forget_what_went(&mut self) {
        let mut lost_arm = false;
        let ids: Vec<&str> = self.spots.iter().map(|s| s.id.as_str()).collect();
        for kept in [&mut self.focus, &mut self.hover, &mut self.pressed, &mut self.dragging] {
            if kept.as_deref().is_some_and(|id| !ids.contains(&id)) {
                *kept = None;
            }
        }
        if self.tip.as_ref().is_some_and(|t| !ids.contains(&t.id.as_str())) {
            self.tip = None;
        }
        if self.armed.as_ref().is_some_and(|(id, _)| !ids.contains(&id.as_str())) {
            self.armed = None;
            lost_arm = true;
        }
        if let Some((id, e)) = &self.picked {
            let same = matches!(self.node(id).map(|n| &n.what), Some(What::Label { text, .. }) if *text == e.text);
            if !same {
                self.picked = None;
            }
        }
        if self.bar_hover.as_deref().is_some_and(|id| !ids.contains(&id)) {
            self.bar_hover = None;
        }
        if self.thumb.as_ref().is_some_and(|(id, _)| !ids.contains(&id.as_str())) {
            self.thumb = None;
        }
        if self.swipe.as_ref().is_some_and(|(id, ..)| !ids.contains(&id.as_str())) {
            self.swipe = None;
        }
        if self.swipe_x.as_ref().is_some_and(|(id, ..)| !ids.contains(&id.as_str())) {
            self.swipe_x = None;
        }
        if lost_arm {
            self.follow_armed();
        }
    }

    fn settle_edits(&mut self) {
        let tree = &self.tree;
        let entry = |id: &str| self.spots.iter().find(|s| s.id == id)
            .is_some_and(|s| matches!(tree.at(&s.path).map(|n| &n.what), Some(What::Entry { .. })));
        self.edits.retain(|id, _| entry(id));
        for s in &self.spots {
            if let Some(What::Entry { text, select: Some(part), .. }) = self.tree.at(&s.path).map(|n| &n.what) {
                if !self.edits.contains_key(&s.id) {
                    let mut e = Edit::new(text);
                    let edge = |at: usize| (0..=at.min(text.len())).rev().find(|&i| text.is_char_boundary(i)).unwrap_or(0);
                    e.select(edge(part.start)..edge(part.end));
                    self.edits.insert(s.id.clone(), e);
                }
            }
        }
    }

    fn settle_focus(&mut self) {
        if self.focus.is_none() && !self.first_given {
            self.first_given = true;
            self.focus = self.spots.iter().find(|s| self.tree.at(&s.path).is_some_and(|n| n.first && n.focusable())).map(|s| s.id.clone());
        }
        let grab = self.spots.iter()
            .filter_map(|s| self.tree.at(&s.path).filter(|n| n.grab > self.grabbed && n.focusable()).map(|n| (n.grab, s.id.clone())))
            .max_by_key(|(turn, _)| *turn);
        if let Some((turn, id)) = grab {
            self.grabbed = turn;
            self.first_given = true;
            self.focus = Some(id);
        }
        if self.eyes.on.is_some() && self.eyes.on != self.focus {
            self.eyes.on = None;
        }
    }
}
