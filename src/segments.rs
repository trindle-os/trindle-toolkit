//! A time in one box as two segments, hours and minutes, each stepped by the keys and told as a spin button.

use std::cell::RefCell;
use std::rc::Rc;

use tiny_skia::{FillRule, Transform};

use crate::colour::paint;
use crate::input::{Input, Keysym};
use crate::text::Style;
use crate::theme;
use crate::ui::rounded;
use crate::when::Time;
use crate::role::Role;
use crate::widget::{drawing, Length, Node, Pointer, Rect};

#[derive(Clone, Debug)]
pub struct SegmentedTime {
    pub time: Time,
    pub lit: usize,
    typed: Option<u32>,
    spans: Rc<RefCell<Vec<(f32, f32)>>>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SegMsg {
    Press(f32),
    Step(i32),
    Along(i32),
    End(bool),
    Digit(u32),
}

const MOST: [u32; 2] = [23, 59];

impl SegmentedTime {
    pub fn new(time: Time) -> Self {
        SegmentedTime { time, lit: 0, typed: None, spans: Rc::default() }
    }

    fn value(&self, segment: usize) -> u32 {
        if segment == 0 { self.time.hour } else { self.time.minute }
    }

    fn put(&mut self, v: u32) {
        if self.lit == 0 { self.time.hour = v } else { self.time.minute = v }
    }

    pub fn update(&mut self, m: SegMsg) -> bool {
        let was = self.time;
        let range = MOST[self.lit.min(1)] as i32 + 1;
        match m {
            SegMsg::Press(x) => {
                let spans = self.spans.borrow();
                let off = |i: usize| (spans[i].0 + spans[i].1 / 2.0 - x).abs();
                self.lit = (0..spans.len()).min_by(|&p, &q| off(p).total_cmp(&off(q))).unwrap_or(self.lit);
                self.typed = None;
            }
            SegMsg::Step(n) => {
                self.typed = None;
                self.put((self.value(self.lit) as i32 + n).rem_euclid(range) as u32);
            }
            SegMsg::Along(n) => {
                self.typed = None;
                self.lit = (self.lit as i32 + n).clamp(0, 1) as usize;
            }
            SegMsg::End(most) => {
                self.typed = None;
                self.put(if most { MOST[self.lit] } else { 0 });
            }
            SegMsg::Digit(d) => match self.typed.take() {
                None if d * 10 > MOST[self.lit] => {
                    self.put(d);
                    self.lit = (self.lit + 1).min(1);
                }
                None => {
                    self.put(d);
                    self.typed = Some(d);
                }
                Some(first) => {
                    self.put((first * 10 + d).min(MOST[self.lit]));
                    self.lit = (self.lit + 1).min(1);
                }
            },
        }
        self.time != was
    }

    pub fn node<M: 'static>(&self, tip: &str, wrap: impl Fn(SegMsg) -> M + Clone + 'static) -> Node<M> {
        let (time, lit) = (self.time, self.lit);
        let spans = self.spans.clone();
        let told = [crate::t!("kit-time-hours"), crate::t!("kit-time-minutes")].into_iter().enumerate().map(|(i, name)| {
            let (x, w) = spans.borrow().get(i).copied().unwrap_or_default();
            (name.to_string(), f64::from(if i == 0 { time.hour } else { time.minute }), f64::from(MOST[i]), x, w)
        }).collect();
        let (w1, w2) = (wrap.clone(), wrap);
        let pad = theme::sheet().entry_padding;
        let width = theme::sheet().text.0 * 5.0 + 4.0 * pad;
        drawing((width, theme::sheet().entry_height), "", move |canvas, pen| {
            let (t, s) = (pen.theme, pen.scale);
            let (w, h) = (canvas.width() as f32, canvas.height() as f32);
            let whole = Rect { x: 0.0, y: 0.0, w, h };
            let radius = theme::corner_px(theme::Part::Box, h, s);
            let shape = |canvas: &mut tiny_skia::PixmapMut, r: Rect, radius: f32, c| {
                if let Some(path) = rounded(r, radius) {
                    canvas.fill_path(&path, &paint(c), FillRule::Winding, Transform::identity(), None);
                }
            };
            shape(canvas, whole, radius, t.field_edge);
            let line = s.round().max(1.0);
            shape(canvas, Rect { x: line, y: line, w: w - 2.0 * line, h: h - 2.0 * line }, (radius - line).max(0.0), t.field);
            let style = Style::new((theme::sheet().text.0 * s).round());
            let figures = |i: usize| crate::lang::format::digits(&format!("{:02}", if i == 0 { time.hour } else { time.minute }));
            let (digits, colon) = (pen.fonts.layout("00", &style, None), pen.fonts.layout(":", &style, None));
            let room = pad * s;
            let (across, gap) = (digits.width + 2.0 * room, colon.width);
            let mut x = (w - 2.0 * across - gap) / 2.0;
            let mut laid = Vec::new();
            for i in 0..2 {
                let l = pen.fonts.layout(&figures(i), &style, None);
                let ink = if lit == i {
                    shape(canvas, Rect { x, y: line * 2.0, w: across, h: h - 4.0 * line }, (radius - 2.0 * line).max(0.0), t.accent);
                    t.on_accent
                } else {
                    t.fg
                };
                pen.fonts.draw(canvas, &l, (x + room).round(), ((h - l.height) / 2.0).round(), ink);
                laid.push((x / s, across / s));
                x += across;
                if i == 0 {
                    pen.fonts.draw(canvas, &colon, x.round(), ((h - colon.height) / 2.0).round(), t.fg);
                    x += gap;
                }
            }
            *spans.borrow_mut() = laid;
        })
        .width(Length::Shrink)
        .role(Role::Segments { parts: told, lit })
        .called(tip)
        .on_pointer_when(move |p| match p {
            Pointer::Down(x, _) => Some(w1(SegMsg::Press(x))),
            _ => None,
        })
        .on_keys(move |i| seg_key(i, lit).map(&w2))
    }
}

fn seg_key(i: &Input, lit: usize) -> Option<SegMsg> {
    let Input::Key { sym, mods, text, .. } = i else { return None };
    let page = if lit == 0 { 6 } else { 10 };
    let m = match *sym {
        Keysym::Up | Keysym::KP_Up => SegMsg::Step(1),
        Keysym::Down | Keysym::KP_Down => SegMsg::Step(-1),
        Keysym::Page_Up | Keysym::KP_Page_Up => SegMsg::Step(page),
        Keysym::Page_Down | Keysym::KP_Page_Down => SegMsg::Step(-page),
        Keysym::Home | Keysym::KP_Home => SegMsg::End(false),
        Keysym::End | Keysym::KP_End => SegMsg::End(true),
        Keysym::Left | Keysym::KP_Left => SegMsg::Along(-1),
        Keysym::Right | Keysym::KP_Right => SegMsg::Along(1),
        Keysym::Tab if !mods.shift && lit == 0 => SegMsg::Along(1),
        Keysym::ISO_Left_Tab | Keysym::Tab if mods.shift && lit == 1 => SegMsg::Along(-1),
        _ => match text.as_deref().and_then(|t| t.chars().next().filter(|_| t.chars().count() == 1)).and_then(|c| crate::lang::numerals::digit_value(c).map(u32::from)) {
            Some(d) if !mods.ctrl && !mods.alt => SegMsg::Digit(d),
            _ => return None,
        },
    };
    Some(m)
}
