//! A menu bar: a row of titles across a window's top, each opening its menu below it.

use std::cell::RefCell;
use std::rc::Rc;

use tiny_skia::{FillRule, Transform};

use crate::colour::paint;
use crate::input::{Input, Keysym};
use crate::popup::{self, Done, Place, Popup, PopupMsg, Row};
use crate::text::Style;
use crate::theme;
use crate::ui::rounded;
use crate::role::Role;
use crate::widget::{column, drawing, Length, Node, Pointer, Rect};

#[derive(Clone, Debug)]
pub struct MenuBar {
    pub menus: Vec<(String, Vec<Row>)>,
    pub lit: Option<usize>,
    pub open: Option<Popup>,
    turn: u32,
    spans: Rc<RefCell<Vec<(f32, f32)>>>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BarMsg {
    Press(f32),
    Point(f32),
    Key(Keysym),
    Alt(char),
    Letter(char),
    Menu(PopupMsg),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BarDone {
    Open,
    Picked(usize, usize),
    PickedIn(usize, usize, usize),
    Closed,
}

pub fn height() -> f32 {
    theme::sheet().menu_row + 2.0 * theme::sheet().menu_padding
}

impl MenuBar {
    pub fn new(menus: Vec<(String, Vec<Row>)>) -> Self {
        MenuBar { menus, lit: None, open: None, turn: 0, spans: Rc::default() }
    }

    pub fn f10(&mut self) {
        self.turn += 1;
        self.open_at(0, true);
    }

    pub fn alt(&mut self, letter: char) -> bool {
        let at = self.menus.iter().position(|(t, _)| split(t).1.is_some_and(|(_, c)| c.to_lowercase().eq(letter.to_lowercase())));
        if let Some(i) = at {
            self.turn += 1;
            self.open_at(i, true);
        }
        at.is_some()
    }

    fn title_at(&self, x: f32) -> Option<usize> {
        self.spans.borrow().iter().position(|&(a, w)| x >= a && x < a + w)
    }

    fn open_at(&mut self, i: usize, lit: bool) {
        let Some((_, rows)) = self.menus.get(i) else { return };
        let x = self.spans.borrow().get(i).map_or(0.0, |s| s.0);
        let menu = Popup::new(rows.clone(), Place::At(x, height()));
        self.open = Some(if lit { menu.lit() } else { menu });
        self.lit = Some(i);
    }

    fn along(&mut self, by: i32) {
        let n = self.menus.len() as i32;
        if n > 0 {
            let to = self.lit.map_or(0, |l| (l as i32 + by).rem_euclid(n)) as usize;
            if self.open.is_some() {
                self.open_at(to, true);
            } else {
                self.lit = Some(to);
            }
        }
    }

    pub fn update(&mut self, m: BarMsg) -> BarDone {
        match m {
            BarMsg::Press(x) => match self.title_at(x) {
                Some(i) if self.open.is_some() && self.lit == Some(i) => self.shut(),
                Some(i) => {
                    self.open_at(i, false);
                    BarDone::Open
                }
                None => BarDone::Open,
            },
            BarMsg::Point(x) => {
                if let Some(i) = self.title_at(x).filter(|&i| self.open.is_some() && self.lit != Some(i)) {
                    self.open_at(i, false);
                }
                BarDone::Open
            }
            BarMsg::Key(sym) => self.key(sym),
            BarMsg::Alt(c) => {
                self.alt(c);
                BarDone::Open
            }
            BarMsg::Letter(c) => self.in_menu(PopupMsg::Letter(c)),
            BarMsg::Menu(m) => self.in_menu(m),
        }
    }

    fn shut(&mut self) -> BarDone {
        self.open = None;
        BarDone::Closed
    }

    fn key(&mut self, sym: Keysym) -> BarDone {
        let Some(menu) = &self.open else {
            match sym {
                Keysym::Left | Keysym::KP_Left => self.along(-1),
                Keysym::Right | Keysym::KP_Right => self.along(1),
                Keysym::F10 => self.open_at(0, true),
                Keysym::Down | Keysym::KP_Down | Keysym::Up | Keysym::KP_Up | Keysym::Return | Keysym::KP_Enter | Keysym::space => {
                    self.open_at(self.lit.unwrap_or(0), true);
                }
                _ => {}
            }
            return BarDone::Open;
        };
        let sub_open = menu.beside.open.is_some();
        let on_sub = !sub_open && menu.hover.and_then(|h| menu.rows.get(h)).is_some_and(|r| !r.sub.is_empty());
        match sym {
            Keysym::Left | Keysym::KP_Left if !sub_open => {
                self.along(-1);
                BarDone::Open
            }
            Keysym::Right | Keysym::KP_Right if !on_sub && menu.beside.hover.is_none() => {
                self.along(1);
                BarDone::Open
            }
            Keysym::F10 => self.shut(),
            _ => self.in_menu(popup::key(sym)),
        }
    }

    fn in_menu(&mut self, m: PopupMsg) -> BarDone {
        let (Some(menu), Some(i)) = (&mut self.open, self.lit) else { return BarDone::Open };
        match menu.update(m) {
            Done::Open => BarDone::Open,
            Done::Picked(r) => {
                self.open = None;
                BarDone::Picked(i, r)
            }
            Done::PickedIn(r, j) => {
                self.open = None;
                BarDone::PickedIn(i, r, j)
            }
            Done::Closed => self.shut(),
        }
    }

    pub fn node<M: 'static>(&self, wrap: impl Fn(BarMsg) -> M + Clone + 'static) -> Node<M> {
        let titles: Vec<String> = self.menus.iter().map(|m| m.0.clone()).collect();
        let plain: Vec<String> = titles.iter().map(|t| split(t).0).collect();
        let (lit, open) = (self.lit, self.open.is_some());
        let spans = self.spans.clone();
        let told = plain.iter().zip(spans.borrow().iter()).map(|(t, &(x, w))| (t.clone(), x, w)).collect();
        let (w1, w2) = (wrap.clone(), wrap);
        drawing((0.0, height()), "", move |canvas, pen| {
            let (t, s) = (pen.theme, pen.scale);
            let (pad, title_pad) = (theme::sheet().menu_padding, theme::sheet().menu_title_pad);
            let style = Style::new((theme::sheet().text.0 * s).round());
            let mut x = pad;
            let mut laid = Vec::new();
            for (i, words) in titles.iter().enumerate() {
                let (shown, letter) = split(words);
                let l = pen.fonts.layout(&shown, &style, None);
                let w = l.width / s + 2.0 * title_pad;
                if lit == Some(i) {
                    let r = Rect { x: x * s, y: pad * s, w: w * s, h: theme::sheet().menu_row * s };
                    if let Some(path) = rounded(r, theme::sheet().corner_of(theme::Part::Row) * s) {
                        canvas.fill_path(&path, &paint(if open { t.raised_more } else { t.hover }), FillRule::Winding, Transform::identity(), None);
                    }
                }
                let (left, top) = (((x + title_pad) * s).round(), ((height() * s - l.height) / 2.0).round());
                match letter {
                    Some((at, c)) => {
                        let end = at + c.len_utf8();
                        let mut from = left;
                        for (part, colour) in [(&shown[..at], t.fg), (&shown[at..end], t.accent_text), (&shown[end..], t.fg)] {
                            let piece = pen.fonts.layout(part, &style, None);
                            pen.fonts.draw(canvas, &piece, from, top, colour);
                            from += piece.width;
                        }
                    }
                    None => pen.fonts.draw(canvas, &l, left, top, t.fg),
                }
                laid.push((x, w));
                x += w + pad;
            }
            *spans.borrow_mut() = laid;
        })
        .width(Length::Fill)
        .role(Role::MenuBar { titles: told, lit, open })
        .called(crate::t!("kit-menu-bar"))
        .on_pointer_when(move |p| match p {
            Pointer::Down(x, _) => Some(w1(BarMsg::Press(x))),
            Pointer::Hover(x, _) | Pointer::Move(x, _) => Some(w1(BarMsg::Point(x))),
            _ => None,
        })
        .on_hover()
        .on_keys(move |i| bar_input(i, open).map(&w2))
        .grab(self.turn)
    }

    pub fn window<M: 'static>(&self, page: Node<M>, wrap: impl Fn(BarMsg) -> M + Clone + 'static) -> Node<M> {
        let all = column(vec![self.node(wrap.clone()), page]).gap(0.0);
        match &self.open {
            Some(menu) => menu.over(all, move |m| wrap(BarMsg::Menu(m))),
            None => all,
        }
    }
}

fn bar_key(sym: Keysym, open: bool) -> bool {
    matches!(sym, Keysym::Left | Keysym::KP_Left | Keysym::Right | Keysym::KP_Right | Keysym::Up | Keysym::KP_Up | Keysym::Down | Keysym::KP_Down
        | Keysym::Return | Keysym::KP_Enter | Keysym::space | Keysym::F10) || (open && sym == Keysym::Escape)
}

fn split(title: &str) -> (String, Option<(usize, char)>) {
    match title.find('&') {
        Some(i) if title[i + 1..].chars().next().is_some_and(char::is_alphanumeric) => {
            let shown = format!("{}{}", &title[..i], &title[i + 1..]);
            let c = shown[i..].chars().next().unwrap();
            (shown, Some((i, c)))
        }
        _ => (title.to_string(), title.char_indices().find(|(_, c)| c.is_alphanumeric())),
    }
}

fn bar_input(i: &Input, open: bool) -> Option<BarMsg> {
    match *i {
        Input::Key { sym, mods, .. } if mods.alt => sym.key_char().filter(|c| c.is_alphanumeric()).map(BarMsg::Alt),
        Input::Key { sym, .. } if bar_key(sym, open) => Some(BarMsg::Key(sym)),
        Input::Key { ref text, mods, .. } if open => match popup::typed(text.as_deref(), mods) {
            PopupMsg::Letter(c) => Some(BarMsg::Letter(c)),
            _ => None,
        },
        _ => None,
    }
}
