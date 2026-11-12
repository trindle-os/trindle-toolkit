//! Selecting words anywhere in a window, and the right-click menu that copies them.

use super::*;
use super::super::popup::{self, Place, Popup, Row};
use super::super::widget::{MenuDo, MenuRow};

pub(super) enum Act<M> {
    Copy(String),
    Entry(String, Keysym),
    SelectAll(String),
    Send(M),
}

pub(super) struct CopyMenu<M> {
    pub(super) popup: Popup,
    pub(super) acts: Vec<Act<M>>,
    pub(super) subs: Vec<Vec<Act<M>>>,
}

fn copy_row() -> Row {
    popup::row("edit-copy-symbolic", crate::t!("kit-copy")).keys("Ctrl+C")
}

fn row_of(icon: String, words: String, keys: String, rule: bool, check: Option<bool>, radio: bool) -> Row {
    let mut row = popup::row(icon, words).keys(keys);
    row.rule = rule;
    row.check = check;
    row.radio = radio;
    row
}

fn act_of<M>(does: MenuDo<M>) -> Act<M> {
    match does {
        MenuDo::Copy(t) => Act::Copy(t),
        MenuDo::Send(m) => Act::Send(m),
    }
}

fn ctrl() -> Mods {
    Mods { ctrl: true, ..Mods::default() }
}

impl<M: Clone> Ui<M> {
    pub fn menu_open(&self) -> bool {
        self.copy_menu.is_some() || self.dropdown_open()
    }

    pub(super) fn menu_rect(&self) -> Option<Rect> {
        self.menu_rects().map(|r| r.0)
    }

    pub(super) fn menu_rects(&self) -> Option<(Rect, Option<Rect>)> {
        let m = self.copy_menu.as_ref()?;
        let k = self.scale.max(f32::EPSILON);
        let px = |r: Rect| Rect { x: r.x * k, y: r.y * k, w: r.w * k, h: r.h * k };
        let (menu, sub) = m.popup.spots((self.size.0 as f32 / k, self.size.1 as f32 / k));
        Some((px(menu), sub.map(px)))
    }

    fn sub_row_under(&self, x: f32, y: f32) -> Option<usize> {
        let (m, (_, r)) = (self.copy_menu.as_ref()?, self.menu_rects()?);
        let r = r.filter(|r| r.contains(x, y))?;
        let rules: Vec<bool> = m.popup.sub_rows().iter().map(|r| r.rule).collect();
        popup::row_at(&rules, (y - r.y) / self.scale.max(f32::EPSILON))
    }

    fn menu_hover(&mut self, x: f32, y: f32, from: Option<(f32, f32)>) {
        let k = self.scale.max(f32::EPSILON);
        let (Some((menu, sub)), row) = (self.menu_rects(), self.menu_row_under(x, y)) else { return };
        if sub.is_some_and(|r| r.contains(x, y)) {
            let j = self.sub_row_under(x, y);
            if let Some(m) = &mut self.copy_menu {
                m.popup.update(popup::PopupMsg::SubHover(j));
            }
            return;
        }
        let inside = |(x, y): (f32, f32)| (((x - menu.x) / k).round() as i32, ((y - menu.y) / k).round() as i32);
        let sub_here = sub.map(|r| Rect { x: (r.x - menu.x) / k, y: (r.y - menu.y) / k, w: r.w / k, h: r.h / k });
        let Some(m) = &mut self.copy_menu else { return };
        let open = m.popup.beside.open;
        if open.is_some() && row != open && popup::toward(from.map(inside), ((x - menu.x) / k, (y - menu.y) / k), sub_here, 8.0) {
            return;
        }
        match row {
            Some(r) if !m.popup.rows[r].sub.is_empty() => {
                m.popup.update(popup::PopupMsg::Pick(r));
            }
            Some(r) => {
                m.popup.update(popup::PopupMsg::Out);
                m.popup.hover = Some(r);
            }
            None => {
                m.popup.hover = open;
            }
        }
    }

    fn menu_row_under(&self, x: f32, y: f32) -> Option<usize> {
        let (m, r) = (self.copy_menu.as_ref()?, self.menu_rect()?);
        if !r.contains(x, y) {
            return None;
        }
        let rules: Vec<bool> = m.popup.rows.iter().map(|r| r.rule).collect();
        popup::row_at(&rules, (y - r.y) / self.scale.max(f32::EPSILON))
    }

    pub(super) fn close_menu(&mut self) -> bool {
        self.copy_menu.take().is_some()
    }

    fn open_menu(&mut self, x: f32, y: f32, rows: Vec<(Row, Act<M>)>, lit: bool) {
        if rows.is_empty() {
            return;
        }
        let (rows, acts): (Vec<Row>, Vec<Act<M>>) = rows.into_iter().unzip();
        let k = self.scale.max(f32::EPSILON);
        let mut popup = Popup::new(rows, Place::At(x / k, y / k));
        if lit {
            popup = popup.lit();
        }
        let subs = acts.iter().map(|_| Vec::new()).collect();
        self.copy_menu = Some(CopyMenu { popup, acts, subs });
        self.tip = None;
    }

    fn run_act(&mut self, act: Act<M>, reply: &mut Reply<M>) {
        match act {
            Act::Copy(text) => reply.copy = Some(text),
            Act::Entry(id, sym) => {
                self.focus = Some(id.clone());
                self.edit_key(&id, sym, None, ctrl(), reply);
            }
            Act::SelectAll(id) => {
                if let Some((p, e)) = &mut self.picked {
                    if *p == id {
                        let all = 0..e.text.len();
                        e.select(all);
                    }
                }
            }
            Act::Send(m) => reply.messages.push(m),
        }
        reply.redraw = true;
    }

    pub(super) fn pick_menu_row(&mut self, i: usize, reply: &mut Reply<M>) {
        if let Some(m) = self.copy_menu.as_mut().filter(|m| m.subs.get(i).is_some_and(|s| !s.is_empty())) {
            m.popup.update(popup::PopupMsg::Pick(i));
            reply.redraw = true;
            return;
        }
        let Some(mut m) = self.copy_menu.take() else { return };
        if i < m.acts.len() {
            let act = m.acts.swap_remove(i);
            self.run_act(act, reply);
        }
    }

    pub(super) fn pick_sub_row(&mut self, i: usize, j: usize, reply: &mut Reply<M>) {
        let Some(mut m) = self.copy_menu.take() else { return };
        if let Some(acts) = m.subs.get_mut(i).filter(|a| j < a.len()) {
            let act = acts.swap_remove(j);
            self.run_act(act, reply);
        }
    }

    fn menu_click(&mut self, row: usize, button: Button, pressed: bool, reply: &mut Reply<M>) {
        if let Some(m) = &mut self.copy_menu {
            m.popup.hover = Some(row);
        }
        if (button == Button::Left && !pressed) || (button == Button::Right && pressed) {
            self.pick_menu_row(row, reply);
        }
    }

    fn menu_key(&mut self, sym: Keysym, text: Option<&str>, mods: Mods, reply: &mut Reply<M>) {
        if mods.ctrl && matches!(sym, Keysym::c | Keysym::C | Keysym::Insert) {
            let at = self.copy_menu.as_ref().and_then(|m| m.acts.iter().position(|a| matches!(a, Act::Copy(_))));
            if let Some(i) = at {
                self.pick_menu_row(i, reply);
                return;
            }
        }
        let msg = match popup::key(sym) {
            popup::PopupMsg::Stay => popup::typed(text, mods),
            keyed => keyed,
        };
        let done = self.copy_menu.as_mut().map(|m| m.popup.update(msg));
        match done {
            Some(popup::Done::Picked(i)) => self.pick_menu_row(i, reply),
            Some(popup::Done::PickedIn(i, j)) => self.pick_sub_row(i, j, reply),
            Some(popup::Done::Closed) => {
                self.close_menu();
            }
            _ => {}
        }
    }

    pub(super) fn menu_input(&mut self, input: &Input) -> Option<Reply<M>> {
        let k = self.scale.max(f32::EPSILON);
        let mut reply = Reply::new();
        reply.redraw = true;
        let (under, in_sub) = match *input {
            Input::Press { x, y, .. } | Input::Release { x, y, .. } => (self.menu_row_under(x * k, y * k), self.sub_row_under(x * k, y * k)),
            _ => (None, None),
        };
        let open = self.copy_menu.as_ref().and_then(|m| m.popup.beside.open);
        match *input {
            Input::PointerIn { x, y } | Input::PointerMove { x, y } => {
                let from = self.pointer.replace((x * k, y * k));
                self.menu_hover(x * k, y * k, from);
                Some(reply)
            }
            Input::Press { button, .. } | Input::Release { button, .. } if in_sub.is_some() => {
                if (button == Button::Left) != matches!(input, Input::Press { .. }) {
                    self.pick_sub_row(open?, in_sub?, &mut reply);
                }
                Some(reply)
            }
            Input::Press { button, .. } | Input::Release { button, .. } if under.is_some() => {
                self.menu_click(under?, button, matches!(input, Input::Press { .. }), &mut reply);
                Some(reply)
            }
            Input::Press { button, .. } => {
                self.close_menu();
                (button != Button::Right).then_some(reply)
            }
            Input::Release { .. } => Some(reply),
            Input::TouchDown { x, y, .. } => {
                match (self.sub_row_under(x * k, y * k), self.menu_row_under(x * k, y * k)) {
                    (Some(j), _) => self.pick_sub_row(open?, j, &mut reply),
                    (None, Some(row)) => self.pick_menu_row(row, &mut reply),
                    (None, None) => {
                        self.close_menu();
                    }
                }
                Some(reply)
            }
            Input::TouchMove { .. } | Input::TouchUp { .. } | Input::TouchCancel => Some(reply),
            Input::Key { sym, ref text, mods, .. } => {
                self.menu_key(sym, text.as_deref(), mods, &mut reply);
                Some(reply)
            }
            Input::KeyUp { .. } | Input::Mods(_) | Input::Composed { .. } => Some(reply),
            Input::Focus(false) => {
                self.close_menu();
                None
            }
            Input::Scroll { .. } => {
                self.close_menu();
                None
            }
            _ => None,
        }
    }

    pub(super) fn paint_copy_menu(&self, canvas: &mut PixmapMut) {
        let mut p = Painter::default();
        self.copy_menu_on(&mut p);
        crate::paint::cpu::play(p.ops, canvas, &self.shadows);
    }

    pub(super) fn copy_menu_on(&self, p: &mut Painter) {
        let (Some(m), Some((r, sub))) = (&self.copy_menu, self.menu_rects()) else { return };
        self.floating_menu_on(p, r, &m.popup.rows, m.popup.hover);
        if let Some(sub) = sub {
            self.floating_menu_on(p, sub, m.popup.sub_rows(), m.popup.beside.hover);
        }
    }

    fn floating_menu_on(&self, p: &mut Painter, r: Rect, rows: &[Row], hover: Option<usize>) {
        let (w, h) = (r.w.round().max(1.0) as u32, r.h.round().max(1.0) as u32);
        let Some(at) = tiny_skia::IntRect::from_xywh(r.x.round() as i32, r.y.round() as i32, w, h) else { return };
        let t = &self.theme;
        let falls: Vec<theme::Fall> = t.float.iter().map(|f| self.scaled(f)).collect();
        if !falls.is_empty() {
            p.shadow(r, self.corner(theme::Part::Menu), &falls, t.shadow);
        }
        p.push_clip(at, 0.0);
        popup::paint_menu_on(p, &Pen { theme: t, fonts: &self.fonts, icons: &self.icons, scale: self.scale }, (w as f32, h as f32), rows, hover);
        p.pop();
    }

    pub(super) fn content_at(&self, x: f32, y: f32) -> Option<&Spot> {
        self.spots.iter().rev().find(|s| s.shows(x, y) && self.tree.at(&s.path).is_some_and(|n| {
            !matches!(n.what, What::Column { .. } | What::Row { .. } | What::Grid { .. } | What::Card(..) | What::Space
                | What::Scroll(_) | What::ScrollX(_) | What::Layers(_) | What::Split { .. } | What::Divider { .. } | What::Placed { .. } | What::Slide { .. } | What::Aspect { .. } | What::Ring { .. }
                | What::Spinner { .. })
        }))
    }

    pub(super) fn menu_for(&mut self, x: f32, y: f32) -> bool {
        let spot = self.content_at(x, y).cloned();
        let in_entry = spot.as_ref().and_then(|s| self.tree.at(&s.path)).is_some_and(|n| matches!(n.what, What::Entry { .. }));
        if !in_entry && self.context_menu_at(x, y) {
            return true;
        }
        let Some(spot) = spot else { return false };
        let Some(node) = self.tree.at(&spot.path) else { return false };
        if node.off {
            return false;
        }
        let id = spot.id.clone();
        match &node.what {
            What::Label { text, .. } => {
                let text = text.clone();
                self.words_menu(&id, &text, x, y, false);
                true
            }
            What::Entry { secret, .. } => {
                let secret = *secret;
                self.entry_menu(&id, secret, x, y, false);
                true
            }
            What::List { rows, menu: None, .. } => {
                let Some(row) = self.row_at(&id, y).and_then(|i| rows.get(i)) else { return false };
                let mut text = row.text.clone();
                if let Some(under) = &row.under {
                    text.push('\n');
                    text.push_str(under);
                }
                if let Some(detail) = &row.detail {
                    text.push('\t');
                    text.push_str(detail);
                }
                self.open_menu(x, y, vec![(copy_row(), Act::Copy(text))], false);
                true
            }
            What::Choice { options, .. } => {
                let Some(word) = self.pill(&id, x).and_then(|i| options.get(i)).cloned() else { return false };
                self.open_menu(x, y, vec![(copy_row(), Act::Copy(word))], false);
                true
            }
            What::Drawing { .. } => self.drawing_menu(&id, x, y, false),
            What::Button { words: Some(words), .. } => {
                let words = words.clone();
                self.open_menu(x, y, vec![(copy_row(), Act::Copy(words))], false);
                true
            }
            _ => false,
        }
    }

    fn words_menu(&mut self, id: &str, text: &str, x: f32, y: f32, lit: bool) {
        let chosen = self.picked.as_ref().filter(|(p, e)| p == id && !e.selection().is_empty()).map(|(_, e)| e.selected() == text);
        if chosen.is_none() {
            let mut e = Edit::new(text);
            e.select(0..text.len());
            self.picked = Some((id.to_string(), e));
        }
        self.picked_menu(id, chosen == Some(true), x, y, lit);
    }

    fn picked_menu(&mut self, id: &str, whole: bool, x: f32, y: f32, lit: bool) {
        let Some(copy) = self.picked.as_ref().filter(|(p, _)| p == id).map(|(_, e)| e.selected().to_string()) else { return };
        let mut rows = vec![(copy_row(), Act::Copy(copy))];
        if !whole {
            rows.push((popup::row("edit-select-all-symbolic", crate::t!("kit-select-all")).keys("Ctrl+A"), Act::SelectAll(id.to_string())));
        }
        self.open_menu(x, y, rows, lit);
    }

    fn entry_menu(&mut self, id: &str, secret: bool, x: f32, y: f32, lit: bool) {
        self.focus = Some(id.to_string());
        self.picked = None;
        let Some(e) = self.edit(id) else { return };
        let (has_text, has_pick) = (!e.text.is_empty(), !e.selection().is_empty());
        let key = |sym| Act::Entry(id.to_string(), sym);
        let mut rows = Vec::new();
        if has_pick && !secret {
            rows.push((popup::row("edit-cut-symbolic", crate::t!("kit-cut")).keys("Ctrl+X"), key(Keysym::x)));
            rows.push((copy_row(), key(Keysym::c)));
        }
        rows.push((popup::row("edit-paste-symbolic", crate::t!("kit-paste")).keys("Ctrl+V"), key(Keysym::v)));
        if has_text {
            rows.push((popup::row("edit-select-all-symbolic", crate::t!("kit-select-all")).keys("Ctrl+A"), key(Keysym::a)));
        }
        self.open_menu(x, y, rows, lit);
    }

    pub(super) fn drawing_menu(&mut self, id: &str, x: f32, y: f32, lit: bool) -> bool {
        let (Some(spot), Some(What::Drawing { menu: Some(f), .. })) = (self.spot(id), self.node(id).map(|n| &n.what)) else { return false };
        let k = self.scale.max(f32::EPSILON);
        let rows: Vec<MenuRow<M>> = f((x - spot.rect.x) / k, (y - spot.rect.y) / k);
        self.open_given(x, y, rows, lit)
    }

    pub(super) fn context_rows(&self, path: &[usize]) -> Option<Vec<MenuRow<M>>> {
        (0..=path.len()).rev().find_map(|n| self.tree.at(&path[..n]).filter(|n| !n.off).and_then(|n| n.context.as_ref()).map(|f| f()))
    }

    pub(super) fn has_context_at(&self, x: f32, y: f32) -> bool {
        self.spots.iter().rev().filter(|s| s.shows(x, y)).find_map(|s| self.context_rows(&s.path)).is_some()
    }

    pub(super) fn context_menu_at(&mut self, x: f32, y: f32) -> bool {
        let rows = self.spots.iter().rev().filter(|s| s.shows(x, y)).find_map(|s| self.context_rows(&s.path));
        rows.is_some_and(|rows| self.open_given(x, y, rows, false))
    }

    pub(super) fn context_menu_of(&mut self, id: &str) -> bool {
        let Some((r, rows)) = self.spot(id).and_then(|s| self.context_rows(&s.path).map(|rows| (s.rect, rows))) else { return false };
        self.open_given(r.x + r.w / 2.0, r.y + r.h / 2.0, rows, true)
    }

    fn open_given(&mut self, x: f32, y: f32, rows: Vec<MenuRow<M>>, lit: bool) -> bool {
        if rows.is_empty() {
            return false;
        }
        let (rows, subs): (Vec<(Row, Act<M>)>, Vec<Vec<Act<M>>>) = rows.into_iter().map(|r| {
            let (sub, acts): (Vec<Row>, Vec<Act<M>>) = r.sub.into_iter().map(|s| (row_of(s.icon, s.words, s.keys, s.rule, s.check, s.radio), act_of(s.does))).unzip();
            ((row_of(r.icon, r.words, r.keys, r.rule, r.check, r.radio).sub(sub, 0.0), act_of(r.does)), acts)
        }).unzip();
        self.open_menu(x, y, rows, lit);
        if let Some(m) = &mut self.copy_menu {
            m.subs = subs;
        }
        true
    }

    pub(super) fn menu_from_key(&mut self) -> bool {
        if let Some((id, r)) = self.focus.clone().and_then(|id| self.spot(&id).map(|s| (id, s.rect))) {
            if matches!(self.node(&id).map(|n| &n.what), Some(What::Drawing { menu: Some(_), .. })) {
                return self.drawing_menu(&id, r.x + r.w / 2.0, r.y + r.h / 2.0, true);
            }
        }
        if let Some(id) = self.focus.clone().filter(|id| !self.is_entry(id) && !matches!(self.node(id).map(|n| &n.what), Some(What::List { menu: Some(_), .. }))) {
            if self.context_menu_of(&id) {
                return true;
            }
        }
        if let Some(id) = self.focus.clone().filter(|id| self.is_entry(id)) {
            let Some(s) = self.spot(&id) else { return false };
            let (x, y) = (s.rect.x + s.rect.w / 2.0, s.rect.y + s.rect.h);
            let secret = self.entry_secret(&id);
            self.entry_menu(&id, secret, x, y, true);
            return true;
        }
        let Some((id, e)) = self.picked.as_ref().filter(|(_, e)| !e.selection().is_empty()) else { return false };
        let (id, whole) = (id.clone(), e.selected() == e.text);
        let Some(s) = self.spot(&id) else { return false };
        let (x, y) = (s.rect.x, s.rect.y + s.rect.h);
        self.picked_menu(&id, whole, x, y, true);
        true
    }

    pub(super) fn pick_move(&mut self, sym: Keysym, mods: Mods, reply: &mut Reply<M>) -> bool {
        let Some((_, e)) = &mut self.picked else { return false };
        if mods.ctrl && !mods.shift && !matches!(sym, Keysym::Left | Keysym::Right | Keysym::KP_Left | Keysym::KP_Right) {
            return false;
        }
        let collapse = !mods.shift && e.caret != e.anchor;
        let to = match crate::edit::facing(sym, &e.text) {
            Keysym::Left | Keysym::KP_Left => {
                if mods.ctrl { e.prev_word(e.caret) } else if collapse { e.selection().start } else { e.prev(e.caret) }
            }
            Keysym::Right | Keysym::KP_Right => {
                if mods.ctrl { e.next_word(e.caret) } else if collapse { e.selection().end } else { e.next(e.caret) }
            }
            Keysym::Home | Keysym::KP_Home | Keysym::Up | Keysym::KP_Up => 0,
            Keysym::End | Keysym::KP_End | Keysym::Down | Keysym::KP_Down => e.text.len(),
            _ => return false,
        };
        e.move_to(to, mods.shift);
        reply.redraw = true;
        true
    }
}
