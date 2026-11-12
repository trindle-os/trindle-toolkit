//! Everything a node can be told after it is made, and what it can be asked.

use std::cell::Cell;
use std::ops::Range;
use std::rc::Rc;

use tiny_skia::PixmapMut;

use crate::colour::Colour;
use crate::dnd::{Given, Way};
use crate::input::{Cursor, Input, Mods};

const MOST_LINES: usize = 100;

use super::{Area, Arrows, Held, Length, Many, MenuRow, Node, Outside, Pen, Pointer, Reading, Rect, SaidRow, SaidTable, SaidTabs, Says, Secret, Span, Sub, Tone, Urgency, What, card};

impl<M> Node<M> {
    pub fn width(mut self, w: Length) -> Self {
        self.width = w;
        self
    }

    pub fn height(mut self, h: Length) -> Self {
        self.height = h;
        self
    }

    pub fn key(mut self, key: impl Into<String>) -> Self {
        self.key = Some(key.into());
        self
    }

    pub fn stays(mut self, name: impl Into<String>) -> Self {
        if let What::Placed { stays, .. } = &mut self.what {
            *stays = true;
            self.key = Some(name.into());
        }
        self
    }

    pub fn within(mut self) -> Self {
        if let What::Placed { within, .. } = &mut self.what {
            *within = true;
        }
        self
    }

    pub fn on_escape(mut self, m: M) -> Self {
        self.escape = Some(m);
        self
    }

    pub fn first(mut self) -> Self {
        self.first = true;
        self
    }

    pub fn grab(mut self, turn: u32) -> Self {
        self.grab = turn;
        self
    }

    pub fn seek(mut self, turn: u32, top: f32, tall: f32) -> Self {
        self.seek = Some((turn, top, tall));
        self
    }

    pub fn nudge(mut self, turn: u32, by: f32) -> Self {
        self.nudge = Some((turn, by));
        self
    }

    pub fn nudges(&self, after: u32, path: &mut Vec<usize>, out: &mut Vec<(u32, f32, Vec<usize>)>) {
        if let Some((turn, by)) = self.nudge.filter(|n| n.0 > after) {
            out.push((turn, by, path.clone()));
        }
        for (i, item) in self.items().iter().enumerate() {
            path.push(i);
            item.nudges(after, path, out);
            path.pop();
        }
    }

    #[cfg(any(test, feature = "testing"))]
    pub fn lists_scroll(&self) -> bool {
        self.lists_scroll_inside(false)
    }

    #[cfg(any(test, feature = "testing"))]
    pub fn said(&self) -> Vec<String> {
        let mut out = Vec::new();
        self.each(&mut |n| match &n.what {
            What::Label { text, .. } => out.push(text.clone()),
            What::Rich { spans, .. } => out.push(spans.iter().map(|s| s.text.as_str()).collect()),
            What::Button { tip, .. } => out.push(format!("button: {tip}")),
            What::Switch { tip, .. } => out.push(format!("switch: {tip}")),
            What::Tick { tip, .. } => out.push(format!("tick: {tip}")),
            What::Entry { tip, secret, .. } => out.push(format!("{}: {tip}", if *secret { "secret" } else { "entry" })),
            What::Dropdown { tip, .. } => out.push(format!("dropdown: {tip}")),
            What::List { rows, .. } => out.extend(rows.iter().map(|r| r.text.clone())),
            _ => {}
        });
        out
    }

    #[cfg(any(test, feature = "testing"))]
    pub fn buttons(&self) -> Vec<(String, bool)> {
        let mut out = Vec::new();
        self.each(&mut |n| if let What::Button { tip, press, .. } = &n.what {
            out.push((tip.clone(), press.is_some() && !n.off));
        });
        out
    }

    #[cfg(any(test, feature = "testing"))]
    pub fn presses(&self) -> Vec<(String, bool, &M)> {
        let mut out = Vec::new();
        if let What::Button { tip, press: Some(m), .. } = &self.what {
            out.push((tip.clone(), self.off, m));
        }
        for i in self.items() {
            out.extend(i.presses());
        }
        out
    }

    #[cfg(any(test, feature = "testing"))]
    pub fn submits(&self) -> Vec<&M> {
        let mut out = Vec::new();
        if let What::Entry { submit: Some(m), .. } = &self.what {
            out.push(m);
        }
        for i in self.items() {
            out.extend(i.submits());
        }
        out
    }

    #[cfg(any(test, feature = "testing"))]
    pub fn shut(&self) -> Vec<(String, bool, Option<String>)> {
        let mut out = Vec::new();
        self.each(&mut |n| if let What::Button { tip, press: Some(_), .. } = &n.what {
            out.push((tip.clone(), n.off, n.why.clone()));
        });
        out
    }

    #[cfg(any(test, feature = "testing"))]
    pub fn worded(&self) -> Vec<(String, Option<String>)> {
        let mut out = Vec::new();
        self.each(&mut |n| if let What::Button { tip, words, .. } = &n.what {
            out.push((tip.clone(), words.clone()));
        });
        out
    }

    #[cfg(any(test, feature = "testing"))]
    pub fn for_good(&self) -> Vec<String> {
        let mut out = Vec::new();
        self.each(&mut |n| if let What::Button { tip, for_good: true, .. } = &n.what {
            out.push(tip.clone());
        });
        out
    }

    #[cfg(any(test, feature = "testing"))]
    pub fn iconed(&self) -> Vec<(String, String)> {
        let mut out = Vec::new();
        self.each(&mut |n| if let What::Button { tip, icon, .. } = &n.what {
            if !icon.is_empty() {
                out.push((tip.clone(), icon.clone()));
            }
        });
        out
    }

    #[cfg(any(test, feature = "testing"))]
    fn each<'a>(&'a self, visit: &mut impl FnMut(&'a Node<M>)) {
        visit(self);
        for i in self.items() {
            i.each(visit);
        }
    }

    #[cfg(any(test, feature = "testing"))]
    fn lists_scroll_inside(&self, scrolled: bool) -> bool {
        match &self.what {
            What::List { .. } => scrolled,
            What::Scroll(inner) | What::ScrollX(inner) => inner.lists_scroll_inside(true),
            _ => self.items().iter().all(|i| i.lists_scroll_inside(scrolled)),
        }
    }

    pub fn newest_seek(&self, path: &mut Vec<usize>) -> Option<(u32, f32, f32, Vec<usize>)> {
        let mut best = self.seek.map(|(t, top, tall)| (t, top, tall, path.clone()));
        for (i, item) in self.items().iter().enumerate() {
            path.push(i);
            if let Some(found) = item.newest_seek(path) {
                if best.as_ref().is_none_or(|b| found.0 > b.0) {
                    best = Some(found);
                }
            }
            path.pop();
        }
        best
    }

    pub fn gap(mut self, g: f32) -> Self {
        if let What::Column { gap, .. } | What::Row { gap, .. } | What::Grid { gap, .. } = &mut self.what {
            *gap = g;
        }
        self
    }

    pub fn padding(mut self, p: f32) -> Self {
        if let What::Column { padding, .. } | What::Row { padding, .. } = &mut self.what {
            *padding = p;
        }
        self
    }

    pub fn centred(mut self) -> Self {
        if let What::Column { centre, .. } | What::Entry { centre, .. } = &mut self.what {
            *centre = true;
        }
        self
    }

    pub fn in_circle(mut self, across: f32, margin: f32) -> Self {
        if let What::Column { centre, .. } = &mut self.what {
            *centre = true;
            self.round = Some((across.max(0.0), margin.max(0.0)));
        }
        self
    }

    pub fn on_press(mut self, m: M) -> Self {
        match &mut self.what {
            What::Button { press, .. } | What::Card(_, press) => *press = Some(m),
            _ => {}
        }
        self
    }

    pub fn flat(mut self) -> Self {
        self.flat = true;
        self
    }

    pub fn snug(mut self) -> Self {
        self.snug = true;
        self
    }

    pub fn side(mut self) -> Self {
        self.side = true;
        self
    }

    pub fn area(mut self, which: Area, out: [f32; 4]) -> Self {
        self.area = Some((which, out));
        self.side = true;
        self
    }

    pub fn floats(mut self) -> Self {
        self.floats = true;
        self
    }

    pub fn chip(mut self) -> Self {
        self.flat = true;
        self.chip = true;
        self
    }

    pub fn opacity(mut self, share: f32) -> Self {
        self.opacity = share.clamp(0.0, 1.0);
        self
    }

    pub fn lit(mut self, share: f32) -> Self {
        self.lit = share.clamp(0.0, 1.0);
        self
    }

    pub fn current(mut self, is: bool) -> Self {
        self.current = is;
        self
    }

    pub fn on_press_within(mut self, m: M) -> Self {
        self.claims = Some(m);
        self
    }

    pub fn glow(mut self, turn: u32) -> Self {
        self.glow = turn;
        self
    }

    pub fn holds_box(&self) -> bool {
        match &self.what {
            What::Entry { .. } => true,
            What::Card(..) => !(self.sunk || self.chip || self.flat || self.floats || self.side || self.band),
            What::Row { items, .. } | What::Column { items, .. } => items.iter().any(Node::holds_box),
            _ => false,
        }
    }

    pub fn only_buttons(&self) -> bool {
        match &self.what {
            What::Button { .. } | What::Label { .. } | What::Space | What::Ring { .. } | What::Spinner { .. } => true,
            What::Card(inner, None) => inner.only_buttons(),
            What::Row { items, .. } | What::Column { items, .. } => items.iter().all(Node::only_buttons),
            _ => false,
        }
    }

    pub fn glow_kept(mut self, turn: u32, buttons: bool) -> Self {
        let shows = match &self.what {
            What::Entry { .. } => !buttons,
            What::Card(..) => !buttons && self.holds_box(),
            What::Button { words: None, .. } => buttons,
            _ => false,
        };
        if shows {
            self.glow = turn;
        }
        if let What::Row { items, .. } | What::Column { items, .. } = &mut self.what {
            *items = std::mem::take(items).into_iter().map(|i| i.glow_kept(turn, buttons)).collect();
        }
        self
    }

    pub fn sunk(mut self) -> Self {
        self.sunk = true;
        self
    }

    pub fn tinted(mut self, share: f32) -> Self {
        self.tint = share.clamp(0.001, 1.0);
        self
    }

    pub fn filled(mut self) -> Self {
        self.filled = true;
        self
    }

    pub fn grows(mut self) -> Self {
        if let What::Grid { grow, even: true, .. } = &mut self.what {
            *grow = true;
        }
        self
    }

    pub fn beside(mut self) -> Self {
        self.beside = true;
        self
    }

    pub fn span(mut self, least: u8, most: u8, words: f32) -> Self {
        self.span = Some(Span { least, most, words, grows: false, flat: false });
        self
    }

    pub fn wide_by_choice(mut self) -> Self {
        if let Some(span) = &mut self.span {
            span.flat = true;
        }
        self
    }

    pub fn span_grows(mut self, least: u8, most: u8, words: f32) -> Self {
        self.span = Some(Span { least, most, words, grows: true, flat: false });
        self
    }

    pub fn banded(self) -> Self {
        let mut c = card(self).snug();
        c.band = true;
        c
    }

    pub fn wraps(mut self) -> Self {
        self.wraps = true;
        self
    }

    pub fn stacks(mut self) -> Self {
        self.stacks = true;
        self
    }

    pub fn cut(mut self) -> Self {
        self.cut = 1;
        self
    }

    pub fn cut_after(mut self, lines: u8) -> Self {
        self.cut = lines.max(1);
        self
    }

    pub fn room_for(mut self, lines: u8) -> Self {
        self.cut = lines.max(1);
        self.keep_lines = lines.max(1);
        self
    }

    pub fn whole(mut self, text: impl Into<String>) -> Self {
        self.whole = Some(text.into());
        self
    }

    pub fn when_armed(mut self, remove_key: impl Into<String>) -> Self {
        self.when_armed = Some(remove_key.into());
        self
    }

    pub(crate) fn follow_armed(&mut self, armed: &dyn Fn(&str) -> bool) {
        if let Some(k) = &self.when_armed {
            if armed(k) {
                self.each_below(&mut |n| if let Some((off, opacity)) = n.kept.take() { (n.off, n.opacity) = (off, opacity) });
            } else {
                self.each_below(&mut |n| { n.kept.get_or_insert((n.off, n.opacity)); n.off = true; });
                self.opacity = 0.0;
                return;
            }
        }
        match &mut self.what {
            What::Column { items, .. } | What::Row { items, .. } | What::Grid { items, .. } | What::Layers(items) | What::Split { items, .. } =>
                items.iter_mut().for_each(|n| n.follow_armed(armed)),
            What::Card(inner, _) | What::Scroll(inner) | What::ScrollX(inner) | What::Placed { inner, .. } | What::Slide { inner, .. } | What::Aspect { inner, .. } => inner.follow_armed(armed),
            _ => {}
        }
    }

    pub fn own_labels(mut self) -> Self {
        self.own_labels = true;
        self
    }

    pub fn copies(self, words: impl Into<String>) -> Self {
        let words = words.into();
        self.menu(move |_, _| vec![MenuRow::copy(words.clone())])
    }

    pub fn menu(mut self, f: impl Fn(f32, f32) -> Vec<MenuRow<M>> + 'static) -> Self {
        if let What::Drawing { menu, .. } = &mut self.what {
            *menu = Some(Box::new(f));
        }
        self
    }

    pub fn cannot_be_undone(mut self) -> Self {
        if let What::Button { for_good, .. } = &mut self.what {
            *for_good = true;
        }
        self
    }

    pub fn off(mut self, yes: bool) -> Self {
        self.set_off(yes);
        self
    }

    pub fn off_because(mut self, why: Option<&str>) -> Self {
        if let Some(why) = why {
            self.set_off(true);
            self.each_below(&mut |n| {
                n.why.get_or_insert_with(|| why.to_string());
            });
        }
        self
    }

    fn set_off(&mut self, yes: bool) {
        self.each_below(&mut |n| n.off = yes);
    }

    fn each_below(&mut self, f: &mut dyn FnMut(&mut Node<M>)) {
        f(self);
        match &mut self.what {
            What::Column { items, .. } | What::Row { items, .. } | What::Grid { items, .. } | What::Layers(items) | What::Split { items, .. } =>
                items.iter_mut().for_each(|n| n.each_below(f)),
            What::Card(inner, _) | What::Scroll(inner) | What::ScrollX(inner) | What::Placed { inner, .. } | What::Slide { inner, .. } | What::Aspect { inner, .. } => inner.each_below(f),
            _ => {}
        }
    }

    pub fn on(mut self, yes: bool) -> Self {
        if let What::Button { on, .. } = &mut self.what {
            *on = Some(yes);
        }
        self
    }

    pub fn marked(mut self, ranges: &[Range<usize>]) -> Self {
        if let What::Label { text, marks, .. } = &mut self.what {
            *marks = crate::text::clean_marks(text, ranges);
        }
        self
    }

    pub fn on_link(mut self, f: impl Fn(String) -> M + 'static) -> Self {
        if let What::Rich { follow, .. } = &mut self.what {
            *follow = Some(Box::new(f));
        }
        self
    }

    pub fn tone(mut self, t: Tone) -> Self {
        match &mut self.what {
            What::Label { tone, .. } | What::Ring { tone, .. } => *tone = t,
            _ => {}
        }
        self
    }

    pub fn on_step(mut self, f: impl Fn(i32) -> M + 'static) -> Self {
        if let What::Entry { steps, .. } = &mut self.what {
            *steps = Some(Box::new(f));
        }
        self
    }

    pub fn lines(mut self, n: usize) -> Self {
        if let What::Entry { lines, secret: false, .. } = &mut self.what {
            *lines = n.clamp(1, MOST_LINES);
        }
        self
    }

    pub fn on_submit(mut self, m: M) -> Self {
        if let What::Entry { submit, .. } = &mut self.what {
            *submit = Some(m);
        }
        self
    }

    pub fn selects(mut self, part: Range<usize>) -> Self {
        if let What::Entry { select, .. } = &mut self.what {
            *select = Some(part);
        }
        self
    }

    pub fn weight(mut self, weight: u16) -> Self {
        if let What::Entry { weight: w, .. } = &mut self.what {
            *w = Some(weight);
        }
        self
    }

    pub fn on_leave(mut self, m: M) -> Self {
        if let What::Entry { leave, .. } = &mut self.what {
            *leave = Some(m);
        }
        self
    }

    pub fn on_backspace_empty(mut self, m: M) -> Self {
        if let What::Entry { back, .. } = &mut self.what {
            *back = Some(m);
        }
        self
    }

    pub fn on_tab(mut self, f: impl Fn(bool) -> M + 'static) -> Self {
        if let What::Entry { tab, .. } = &mut self.what {
            *tab = Some(Box::new(f));
        }
        self
    }

    pub fn on_up_down(mut self, f: impl Fn(bool) -> M + 'static) -> Self {
        if let What::Entry { updown, .. } = &mut self.what {
            *updown = Some(Box::new(f));
        }
        self
    }

    pub fn secret(self) -> Self {
        self.secret_of(Secret::Password)
    }

    pub fn secret_of(mut self, what: Secret) -> Self {
        if let What::Entry { secret, kind, misspelt, lines, .. } = &mut self.what {
            *secret = true;
            *kind = what;
            *lines = 1;
            misspelt.clear();
        }
        self
    }

    pub fn shown(mut self, turn: u32) -> Self {
        if let What::Entry { show, .. } = &mut self.what {
            *show = turn;
        }
        self
    }

    pub fn guarded(mut self) -> Self {
        if let What::Entry { guarded, .. } = &mut self.what {
            *guarded = true;
        }
        self
    }

    pub fn leads_to(mut self, address: &str) -> Self {
        if let What::Button { tip, spoken, .. } = &mut self.what {
            let read = crate::link::read(address, crate::common::style::link_label_most());
            *spoken = Some(format!("{tip}. {}", read.spoken));
            self.leads = Some(read.label);
        }
        self
    }

    pub fn spoken(mut self, name: impl Into<String>) -> Self {
        if let What::Entry { spoken, .. } | What::Button { spoken, .. } = &mut self.what {
            *spoken = Some(name.into());
        }
        self
    }

    pub fn size(mut self, points: f32) -> Self {
        match &mut self.what {
            What::Button { size, .. } => *size = Some(points),
            What::Spinner { size } => *size = points,
            _ => {}
        }
        self
    }

    pub fn working(mut self, share: f32, tone: Tone) -> Self {
        if let What::Button { working, .. } = &mut self.what {
            *working = Some((share.clamp(0.0, 1.0), tone));
        }
        self
    }

    pub fn face(mut self, f: impl Fn(&mut PixmapMut, Rect, Colour, &Pen) + 'static) -> Self {
        if let What::Button { face, .. } = &mut self.what {
            *face = Some(Box::new(f));
        }
        self
    }

    pub fn on_pointer(self, f: impl Fn(Pointer) -> M + 'static) -> Self {
        self.on_pointer_mods(move |p, _| f(p))
    }

    pub fn on_pointer_mods(self, f: impl Fn(Pointer, Mods) -> M + 'static) -> Self {
        self.on_pointer_mods_when(move |p, m| Some(f(p, m)))
    }

    pub fn on_pointer_when(self, f: impl Fn(Pointer) -> Option<M> + 'static) -> Self {
        self.on_pointer_mods_when(move |p, _| f(p))
    }

    fn on_pointer_mods_when(mut self, f: impl Fn(Pointer, Mods) -> Option<M> + 'static) -> Self {
        if let What::Drawing { pointer, .. } = &mut self.what {
            *pointer = Some(Box::new(f));
        }
        self
    }

    pub fn alive(mut self) -> Self {
        if let What::Drawing { alive, .. } = &mut self.what {
            *alive = true;
        }
        self
    }

    pub(super) fn says_mut(&mut self) -> &mut Says {
        self.says.get_or_insert_with(Default::default)
    }

    pub fn called(mut self, name: impl Into<String>) -> Self {
        self.says_mut().name = Some(name.into());
        self
    }

    pub fn radio(mut self, chosen: bool) -> Self {
        self.says_mut().radio = Some(chosen);
        self
    }

    pub fn polite(mut self) -> Self {
        self.says_mut().live = Some(Urgency::Polite);
        self
    }

    pub fn assertive(mut self) -> Self {
        self.says_mut().live = Some(Urgency::Assertive);
        self
    }

    pub fn alert(mut self) -> Self {
        self.says_mut().alert = true;
        self
    }

    pub fn busy(mut self) -> Self {
        self.says_mut().busy = true;
        self
    }

    pub fn role(mut self, role: crate::role::Role) -> Self {
        self.says_mut().role = Some(role);
        self
    }

    pub fn value(mut self, words: impl Into<String>) -> Self {
        self.says_mut().words = Some(words.into());
        self
    }

    pub fn misspelt(mut self, any: bool) -> Self {
        self.says_mut().misspelt = any;
        self
    }

    pub fn progress(mut self, share: Option<f32>) -> Self {
        self.says_mut().progress = Some(share.map(|v| v.clamp(0.0, 1.0)));
        self
    }

    pub fn reads(mut self, reading: Reading) -> Self {
        self.says_mut().reading = Some(reading);
        self
    }

    pub fn menu_rows(mut self, rows: Vec<SaidRow>) -> Self {
        self.says_mut().menu = Some(rows);
        self
    }

    pub fn table_rows(mut self, table: SaidTable) -> Self {
        self.says_mut().table = Some(table);
        self
    }

    pub fn tab_list(mut self, tabs: SaidTabs) -> Self {
        self.says_mut().tabs = Some(tabs);
        self
    }

    pub fn unseen(mut self) -> Self {
        self.says_mut().unseen = true;
        self
    }

    pub fn dialog_message(mut self) -> Self {
        self.says_mut().dialog = Some(false);
        self
    }

    pub fn alert_message(mut self) -> Self {
        self.says_mut().dialog = Some(true);
        self
    }

    pub fn named(mut self, yes: bool) -> Self {
        if let What::Drawing { named, .. } = &mut self.what {
            *named = yes;
        }
        self
    }

    pub fn tip_if_cut(mut self, flag: Rc<Cell<bool>>) -> Self {
        if let What::Drawing { cut, .. } = &mut self.what {
            *cut = Some(flag);
        }
        self
    }

    pub fn only_when_seen(mut self) -> Self {
        if let What::Drawing { seen_only, .. } = &mut self.what {
            *seen_only = true;
        }
        self
    }

    pub fn passes(mut self) -> Self {
        if let What::Drawing { passes, .. } = &mut self.what {
            *passes = true;
        }
        self
    }

    pub fn on_hover(mut self) -> Self {
        if let What::Drawing { hovers, .. } = &mut self.what {
            *hovers = true;
        }
        self
    }

    pub fn on_right_drag(mut self) -> Self {
        if let What::Drawing { right_drags, .. } = &mut self.what {
            *right_drags = true;
        }
        self
    }

    pub fn on_drop(mut self, f: impl Fn(usize) -> M + 'static) -> Self {
        if let What::List { landed, .. } | What::Choice { landed, .. } = &mut self.what {
            *landed = Some(Box::new(f));
        }
        self
    }

    pub fn on_drop_where(mut self, takes: impl Fn(usize) -> bool + 'static, f: impl Fn(usize) -> M + 'static) -> Self {
        if let What::List { drop_on, .. } | What::Choice { drop_on, .. } = &mut self.what {
            *drop_on = Some(Box::new(takes));
        }
        self.on_drop(f)
    }

    pub fn on_drop_from_outside(mut self, would: impl Fn(&Held) -> Option<Way> + 'static, take: impl Fn(&Held, Given, Way) -> M + 'static) -> Self {
        self.outside = Some(Outside { kinds: Vec::new(), would: Box::new(would), take: Box::new(take) });
        self
    }

    pub fn on_drop_of(mut self, kinds: &[&str], would: impl Fn(&Held) -> Option<Way> + 'static, take: impl Fn(&Held, Given, Way) -> M + 'static) -> Self {
        let kinds = kinds.iter().map(|k| k.to_string()).collect();
        self.outside = Some(Outside { kinds, would: Box::new(would), take: Box::new(take) });
        self
    }

    pub fn arrows(mut self, subs: Vec<Sub>, opening: impl Fn(usize) -> M + 'static, pick: impl Fn(usize, usize) -> M + 'static) -> Self {
        if let What::Choice { arrows, .. } = &mut self.what {
            *arrows = Some(Arrows { subs, opening: Box::new(opening), pick: Box::new(pick) });
        }
        self
    }

    pub fn on_drop_between(mut self, f: impl Fn(usize) -> M + 'static) -> Self {
        if let What::List { between, .. } = &mut self.what {
            *between = Some(Box::new(f));
        }
        self
    }

    pub fn on_reorder(mut self, f: impl Fn(usize, usize) -> M + 'static) -> Self {
        if let What::List { reorder, .. } = &mut self.what {
            *reorder = Some(Box::new(f));
        }
        self
    }

    pub fn on_move_in(mut self, f: impl Fn(String, usize, usize) -> M + 'static) -> Self {
        if let What::List { move_in, .. } = &mut self.what {
            *move_in = Some(Box::new(f));
        }
        self
    }

    pub fn on_middle(mut self, f: impl Fn(usize) -> M + 'static) -> Self {
        if let What::List { middle, .. } | What::Choice { middle, .. } = &mut self.what {
            *middle = Some(Box::new(f));
        }
        self
    }

    pub fn on_wheel(mut self, f: impl Fn(f32) -> M + 'static) -> Self {
        if let What::Slide { wheel, .. } = &mut self.what {
            *wheel = Some(Box::new(f));
        }
        self
    }

    pub fn on_keys(mut self, f: impl Fn(&Input) -> Option<M> + 'static) -> Self {
        if let What::Drawing { keys, .. } = &mut self.what {
            *keys = Some(Box::new(f));
        }
        self
    }

    pub fn on_scroll(mut self, f: impl Fn(f32) -> M + 'static) -> Self {
        if let What::Drawing { scroll, .. } = &mut self.what {
            *scroll = Some(Box::new(f));
        }
        self
    }

    pub fn pressable(mut self, m: M) -> Self {
        if let What::Drawing { click, .. } = &mut self.what {
            *click = Some(m);
        }
        self
    }

    pub fn on_double_press(mut self, m: M) -> Self {
        if let What::Drawing { twice, .. } = &mut self.what {
            *twice = Some(m);
        }
        self
    }

    pub fn cursor_with(mut self, f: impl Fn(f32, f32) -> Cursor + 'static) -> Self {
        if let What::Drawing { cursor, .. } = &mut self.what {
            *cursor = Some(Box::new(f));
        }
        self
    }

    pub fn row_height(mut self, h: f32) -> Self {
        if let What::List { row, .. } = &mut self.what {
            *row = h;
        }
        self
    }

    pub fn many(mut self, picked: Vec<usize>, pick: impl Fn(Vec<usize>) -> M + 'static) -> Self {
        if let What::List { many, .. } = &mut self.what {
            *many = Some(Many { picked, pick: Box::new(pick) });
        }
        self
    }

    pub fn on_open(mut self, f: impl Fn(usize) -> M + 'static) -> Self {
        if let What::List { open, .. } = &mut self.what {
            *open = Some(Box::new(f));
        }
        self
    }

    pub fn on_menu(mut self, f: impl Fn(usize, f32, f32) -> M + 'static) -> Self {
        if let What::List { menu, .. } = &mut self.what {
            *menu = Some(Box::new(f));
        }
        self
    }

    pub fn here(mut self) -> Self {
        if let What::List { here, .. } = &mut self.what {
            *here = true;
        }
        self
    }

    pub fn type_ahead(mut self) -> Self {
        if let What::List { ahead, .. } = &mut self.what {
            *ahead = true;
        }
        self
    }

    pub fn items(&self) -> &[Node<M>] {
        match &self.what {
            What::Column { items, .. } | What::Row { items, .. } | What::Grid { items, .. } | What::Layers(items) | What::Split { items, .. } => items,
            What::Card(inner, _) | What::Scroll(inner) | What::ScrollX(inner) | What::Placed { inner, .. } | What::Slide { inner, .. } | What::Aspect { inner, .. } => std::slice::from_ref(inner),
            _ => &[],
        }
    }

    pub(crate) fn at(&self, path: &[usize]) -> Option<&Node<M>> {
        match path.split_first() {
            None => Some(self),
            Some((&i, rest)) => self.items().get(i)?.at(rest),
        }
    }

    pub(crate) fn shown_words(&self) -> String {
        fn gather<'a, M>(n: &'a Node<M>, out: &mut Vec<&'a str>) {
            match &n.what {
                What::Label { text, .. } => out.push(text),
                What::Drawing { tip, .. } if !tip.is_empty() => out.push(tip),
                _ => n.items().iter().for_each(|i| gather(i, out)),
            }
        }
        let mut out = Vec::new();
        gather(self, &mut out);
        out.join("\n")
    }

    pub(crate) fn focusable(&self) -> bool {
        if self.off {
            return false;
        }
        match &self.what {
            What::Button { press, .. } | What::Card(_, press) => press.is_some(),
            What::Switch { .. } | What::Tick { .. } | What::Slider { .. } | What::Entry { .. } | What::Divider { .. } => true,
            What::Choice { options, .. } | What::Dropdown { options, .. } => !options.is_empty(),
            What::List { rows, .. } => !rows.is_empty(),
            What::Drawing { keys, click, .. } => keys.is_some() || click.is_some(),
            What::Rich { spans, follow, .. } => follow.is_some() && spans.iter().any(|s| s.link.is_some()),
            _ => false,
        }
    }

    pub fn tip(&self) -> Option<&str> {
        match &self.what {
            What::Button { words: Some(_), .. } => None,
            What::Button { tip, .. } | What::Switch { tip, .. } | What::Tick { tip, .. } | What::Slider { tip, .. } | What::Choice { tip, .. }
            | What::Dropdown { tip, .. }
                if !tip.is_empty() => Some(tip),
            What::Drawing { tip, click: Some(_), named, cut, .. } if !tip.is_empty() && (!*named || cut.as_ref().is_some_and(|c| c.get())) => Some(tip),
            _ => None,
        }
    }
}
