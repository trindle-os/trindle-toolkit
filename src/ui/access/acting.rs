//! What a screen reader asks done to a part, done as if the person had done it.

use super::*;

impl<M: Clone> Ui<M> {
    pub(super) fn find(&self, target: NodeId) -> Option<(String, Piece)> {
        if let Some(m) = &self.copy_menu {
            if let Some(i) = (0..m.acts.len()).find(|&i| part_id(COPY_MENU, Piece::Item(i)) == target) {
                return Some((COPY_MENU.into(), Piece::Item(i)));
            }
            if let Some(j) = (0..m.popup.sub_rows().len()).find(|&j| part_id(COPY_SUB, Piece::Item(j)) == target) {
                return Some((COPY_SUB.into(), Piece::Item(j)));
            }
        }
        for s in &self.spots {
            if part_id(&s.id, Piece::Widget) == target {
                return Some((s.id.clone(), Piece::Widget));
            }
            if let Some(piece) = self.piece_of(s, target) {
                return Some((s.id.clone(), piece));
            }
        }
        None
    }

    fn piece_of(&self, s: &Spot, target: NodeId) -> Option<Piece> {
        let is = |piece: Piece| part_id(&s.id, piece) == target;
        match self.tree.at(&s.path).map(|n| &n.what) {
            Some(What::Choice { options, .. }) => (0..options.len()).find(|&r| is(Piece::Row(r))).map(Piece::Row),
            Some(What::List { rows, chosen, row, .. }) => self.list_span(s, rows.len(), *chosen, self.px(*row)).find(|&r| is(Piece::Row(r))).map(Piece::Row),
            Some(What::Dropdown { .. }) => self.told_options(&s.id).into_iter().find(|&r| is(Piece::Row(r))).map(Piece::Row),
            Some(What::Entry { text, .. }) => {
                if is(Piece::Eye) && self.eye_of(&s.id).is_some() {
                    return Some(Piece::Eye);
                }
                let pieces = text.graphemes(true).count().div_ceil(RUN).max(1);
                (0..pieces).find(|&k| is(Piece::Run(k))).map(Piece::Run)
            }
            Some(What::Rich { spans, .. }) => (0..spans.len()).find(|&i| is(Piece::Item(i))).map(Piece::Item),
            Some(What::Drawing { .. }) => self.drawing_piece(s, target),
            _ => None,
        }
    }

    fn drawing_piece(&self, s: &Spot, target: NodeId) -> Option<Piece> {
        let is = |piece: Piece| part_id(&s.id, piece) == target;
        let says = self.node(&s.id).and_then(|n| n.says.as_deref());
        if let Some(tabs) = says.and_then(|k| k.tabs.as_ref()) {
            for t in &tabs.tabs {
                for piece in [Piece::Tab(t.index), Piece::Close(t.index)] {
                    if is(piece) {
                        return Some(piece);
                    }
                }
            }
            if tabs.more && is(Piece::More) {
                return Some(Piece::More);
            }
        }
        let items = match says.and_then(|k| k.role.as_ref()) {
            Some(crate::role::Role::Toolbar { tools, .. }) => tools.len(),
            Some(crate::role::Role::Range { .. }) => 2,
            Some(crate::role::Role::Segments { parts, .. }) => parts.len(),
            _ => 0,
        };
        if let Some(i) = (0..items).find(|&i| is(Piece::Item(i))) {
            return Some(Piece::Item(i));
        }
        let table = says.and_then(|k| k.table.as_ref())?;
        table.rows.iter().find(|r| is(Piece::TableRow(r.index))).map(|r| Piece::TableRow(r.index))
    }

    fn centre_of(&self, id: &str, piece: Piece) -> Option<(f32, f32)> {
        let says = self.node(id)?.says.as_deref()?;
        if let (Some(crate::role::Role::Toolbar { tools, .. }), Piece::Item(i)) = (&says.role, piece) {
            let (_, x, w) = tools.get(i)?;
            return Some((x + w / 2.0, self.spot(id)?.rect.h / self.k() / 2.0));
        }
        if let Some(tabs) = &says.tabs {
            let y = tabs.height / 2.0;
            let last = tabs.tabs.last().map_or(0.0, |t| t.x + t.width);
            return match piece {
                Piece::Tab(i) => tabs.tabs.iter().find(|t| t.index == i).map(|t| (t.x + tabs::pad(), y)),
                Piece::Close(i) => tabs.tabs.iter().find(|t| t.index == i).map(|t| {
                    let c = tabs::close_box(Rect { x: t.x, y: 0.0, w: t.width, h: tabs.height });
                    (c.x + c.w / 2.0, y)
                }),
                Piece::More => Some((last + tabs::more() / 2.0, y)),
                _ => None,
            };
        }
        let table = says.table.as_ref()?;
        let Piece::TableRow(i) = piece else { return None };
        let r = table.rows.iter().find(|r| r.index == i)?;
        Some((table.columns.first().map_or(0.0, |c| c.0 + 4.0), r.top + r.height / 2.0))
    }

    fn act_slider(&mut self, id: &str, req: &ActionRequest, reply: &mut Reply<M>) {
        let Some(What::Slider { value, min, max, step, change, .. }) = self.node(id).map(|n| &n.what) else { return };
        let to = match (req.action, &req.data) {
            (Action::Increment, _) => value + step,
            (Action::Decrement, _) => value - step,
            (_, Some(ActionData::NumericValue(v))) => *v as f32,
            (_, Some(ActionData::Value(v))) => match v.trim().parse::<f32>() {
                Ok(v) => v,
                Err(_) => return,
            },
            _ => return,
        };
        if !to.is_finite() {
            return;
        }
        let v = snap(to, *min, *max, *step);
        if v != *value {
            reply.messages.push(change(v));
        }
    }

    fn act_set_value(&mut self, id: &str, req: &ActionRequest, reply: &mut Reply<M>) {
        let Some(ActionData::Value(v)) = &req.data else { return };
        let line = self.entry_words(id, v);
        let replace = req.action == Action::ReplaceSelectedText;
        let Some(e) = self.edit(id) else { return };
        let before = e.stamp();
        if !replace {
            e.select(0..e.text.len());
        }
        if line.is_empty() {
            e.delete_to(e.caret);
        } else {
            e.insert(&line);
        }
        self.changed(id, before, reply);
    }

    fn act_selection(&mut self, id: &str, req: &ActionRequest) {
        let Some(ActionData::SetTextSelection(sel)) = &req.data else { return };
        let (Some(anchor), Some(caret)) = (self.byte_at(id, sel.anchor), self.byte_at(id, sel.focus)) else { return };
        self.focus = Some(id.to_string());
        if let Some(e) = self.edit(id) {
            e.move_to(anchor, false);
            e.move_to(caret, true);
        }
    }

    pub(super) fn handle_lit(&self, id: &str) -> Option<usize> {
        match self.node(id)?.says.as_deref()?.role {
            Some(crate::role::Role::Range { lit, .. } | crate::role::Role::Segments { lit, .. }) => Some(lit),
            _ => None,
        }
    }

    pub(super) fn act_handle(&mut self, id: &str, i: usize, action: Action, reply: &mut Reply<M>) {
        let Some(What::Drawing { keys: Some(keys), .. }) = self.node(id).map(|n| &n.what) else { return };
        let key = |sym, shift| Input::Key { sym, text: None, mods: Mods { shift, ..Default::default() }, repeat: false };
        let lit = self.handle_lit(id);
        let mut sent = Vec::new();
        if lit != Some(i) {
            let segments = matches!(self.node(id).and_then(|n| n.says.as_deref()).and_then(|s| s.role.as_ref()), Some(crate::role::Role::Segments { .. }));
            sent.push(match (segments, i) {
                (true, 0) => key(Keysym::Left, false),
                (true, _) => key(Keysym::Right, false),
                (false, 0) => key(Keysym::Tab, true),
                (false, _) => key(Keysym::Tab, false),
            });
        }
        match action {
            Action::Increment => sent.push(key(Keysym::Up, false)),
            Action::Decrement => sent.push(key(Keysym::Down, false)),
            _ => {}
        }
        reply.messages.extend(sent.iter().filter_map(keys));
        self.focus = Some(id.to_string());
    }

    pub(super) fn act_tap(&mut self, id: &str, piece: Piece, reply: &mut Reply<M>) {
        if let (Some((x, y)), Some(What::Drawing { pointer: Some(f), .. })) = (self.centre_of(id, piece), self.node(id).map(|n| &n.what)) {
            reply.messages.extend(f(w::Pointer::Down(x, y), Default::default()));
            reply.messages.extend(f(w::Pointer::Up(x, y), Default::default()));
        }
    }

    fn act_branch(&mut self, id: &str, r: usize, expand: bool, reply: &mut Reply<M>) {
        let open = self.node(id).and_then(|n| n.says.as_deref()).and_then(|s| s.table.as_ref())
            .and_then(|t| t.rows.iter().find(|x| x.index == r)).and_then(|x| x.branch).and_then(|b| b.open);
        if open.is_some_and(|o| o != expand) {
            self.act_tap(id, Piece::TableRow(r), reply);
            let sym = if expand { Keysym::Right } else { Keysym::Left };
            let key = Input::Key { sym, text: None, mods: Mods::default(), repeat: false };
            reply.messages.extend(self.keys_of(id).and_then(|f| f(&key)));
        }
    }

    pub(super) fn act_row_click(&mut self, id: &str, r: usize, reply: &mut Reply<M>) {
        if let Some(What::List { chosen, choose, open, .. }) = self.node(id).map(|n| &n.what) {
            if *chosen != Some(r) {
                reply.messages.push(choose(r));
            } else if let Some(open) = open {
                reply.messages.push(open(r));
            }
        }
        self.focus = Some(id.to_string());
    }

    fn act_scroll(&mut self, id: &str, up: bool) {
        if let Some(s) = self.spot(id) {
            let by = self.page(s.rect.h);
            let to = self.offset(s) + if up { -by } else { by };
            self.scroll_area(id, to);
        }
    }

    pub(super) fn act_choice(&mut self, id: &str, r: usize, focus: bool, reply: &mut Reply<M>) {
        self.focus = Some(id.to_string());
        self.keyboard_seen = focus;
        if let Some(What::Choice { chosen, choose, .. }) = self.node(id).map(|n| &n.what) {
            if *chosen != r {
                reply.messages.push(choose(r));
            }
        }
    }

    pub(super) fn act_list_focus(&mut self, id: &str, r: usize, row: f32, reply: &mut Reply<M>) {
        self.focus = Some(id.to_string());
        self.keyboard_seen = true;
        if let Some(What::List { chosen, choose, .. }) = self.node(id).map(|n| &n.what) {
            if *chosen != Some(r) {
                reply.messages.push(choose(r));
            }
        }
        self.reveal(id, Some((r as f32 * row, row)));
    }

    pub(super) fn act_focus(&mut self, id: &str, eye: bool) {
        self.eyes.on = eye.then(|| id.to_string());
        self.focus = Some(id.to_string());
        self.keyboard_seen = true;
        self.reveal(id, None);
    }

    pub(super) fn act_on_the_whole(&mut self, id: &str, piece: Piece, row: f32, req: &ActionRequest, reply: &mut Reply<M>) {
        match (req.action, piece) {
            (Action::Expand | Action::Collapse, Piece::TableRow(r)) => self.act_branch(id, r, req.action == Action::Expand, reply),
            (Action::Expand | Action::Collapse, Piece::Row(r)) => {
                let open = match self.node(id).map(|n| &n.what) {
                    Some(What::List { tree: Some(t), .. }) => t.rows.get(r).and_then(|x| x.1),
                    _ => None,
                };
                if open.is_some_and(|o| o != (req.action == Action::Expand)) {
                    self.tree_toggle(id, r, reply);
                }
            }
            (Action::ShowContextMenu, _) => {
                self.context_menu_of(id);
            }
            (Action::Click, _) => self.activate(id, std::time::Instant::now(), reply),
            (Action::ScrollIntoView, Piece::Row(r)) => self.reveal(id, Some((r as f32 * row, row))),
            (Action::ScrollIntoView, _) => self.reveal(id, None),
            (Action::ScrollUp | Action::ScrollDown, _) => {
                self.act_scroll(id, req.action == Action::ScrollUp);
            }
            (Action::Increment | Action::Decrement | Action::SetValue, _) if self.is_slider(id) => {
                self.act_slider(id, req, reply);
            }
            (Action::Increment | Action::Decrement, _) if self.keys_of(id).is_some() => {
                let sym = if req.action == Action::Increment { Keysym::Up } else { Keysym::Down };
                let key = Input::Key { sym, text: None, mods: Mods::default(), repeat: false };
                reply.messages.extend(self.keys_of(id).and_then(|f| f(&key)));
            }
            (Action::SetValue | Action::ReplaceSelectedText, _) if self.is_entry(id) => {
                self.act_set_value(id, req, reply);
            }
            (Action::SetTextSelection, _) if self.is_entry(id) => {
                self.act_selection(id, req);
            }
            _ => reply.redraw = false,
        }
    }

    pub(super) fn is_choice(&self, id: &str) -> bool {
        matches!(self.node(id).map(|n| &n.what), Some(What::Choice { .. }))
    }

    fn is_slider(&self, id: &str) -> bool {
        matches!(self.node(id).map(|n| &n.what), Some(What::Slider { .. }))
    }

    fn byte_at(&self, id: &str, at: TextPosition) -> Option<usize> {
        let (owner, Piece::Run(k)) = self.find(at.node)? else { return None };
        if owner != id {
            return None;
        }
        let What::Entry { text, .. } = &self.node(id)?.what else { return None };
        let n = k * RUN + at.character_index.min(RUN);
        Some(text.grapheme_indices(true).nth(n).map_or(text.len(), |(i, _)| i))
    }
}
