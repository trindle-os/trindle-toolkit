//! Bars: a progress bar, a toolbar whose last tools go into a menu, a status bar and a path bar.

use std::cell::Cell;
use std::rc::Rc;
use std::sync::OnceLock;
use std::time::Instant;

use tiny_skia::{FillRule, PixmapMut, Transform};

use crate::colour::{paint, Colour};
use crate::input::{Input, Keysym};
use crate::theme;
use crate::ui::rounded;
use crate::role::Role;
use crate::widget::{card, drawing, dropdown, icon_image, label, quiet_action, row, Area, Length, MenuRow, Node, Pen, Pointer, Rect, Tone};

const CROSSING: f32 = 1.6;

fn fill(canvas: &mut PixmapMut, r: Rect, corner: f32, colour: Colour) {
    if let Some(path) = rounded(r, corner) {
        canvas.fill_path(&path, &paint(colour), FillRule::Winding, Transform::identity(), None);
    }
}

pub fn progress_bar<M>(share: Option<f32>, name: impl Into<String>) -> Node<M> {
    let share = share.map(|s| s.clamp(0.0, 1.0));
    let moving = share.is_none() && !crate::motion::reduced();
    let bar = drawing((theme::SLIDER_MIN_WIDTH, theme::sheet().knob), "", move |canvas, pen| {
        let (t, s) = (pen.theme, pen.scale);
        let (w, h) = (canvas.width() as f32, canvas.height() as f32);
        let thick = (theme::sheet().track * s).round().max(1.0);
        let line = Rect { x: 0.0, y: ((h - thick) / 2.0).round(), w, h: thick };
        let corner = theme::sheet().corner_of(theme::Part::Slider) * s;
        fill(canvas, line, corner, t.control);
        let (from, to) = match share {
            Some(v) => (0.0, v),
            None => {
                static START: OnceLock<Instant> = OnceLock::new();
                let along = if moving { (START.get_or_init(Instant::now).elapsed().as_secs_f32() / CROSSING).fract() } else { 0.0 };
                (along * 4.0 / 3.0 - 1.0 / 3.0, along * 4.0 / 3.0)
            }
        };
        let (a, b) = (from.max(0.0) * w, to.min(1.0) * w);
        if b > a {
            fill(canvas, Rect { x: a, w: (b - a).max(thick), ..line }, corner, t.soft);
        }
    })
    .width(Length::Fill)
    .progress(share)
    .called(name);
    if moving { bar.alive() } else { bar }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Tool {
    pub icon: String,
    pub tip: String,
    pub rule: bool,
    pub on: Option<bool>,
}

pub fn tool(icon: impl Into<String>, tip: impl Into<String>) -> Tool {
    Tool { icon: icon.into(), tip: tip.into(), rule: false, on: None }
}

impl Tool {
    pub fn rule(mut self) -> Self {
        self.rule = true;
        self
    }

    pub fn toggle(mut self, on: bool) -> Self {
        self.on = Some(on);
        self
    }
}

#[derive(Clone, Debug)]
pub struct Toolbar {
    pub tools: Vec<Tool>,
    pub lit: Option<usize>,
    seen: Rc<Cell<f32>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToolMsg {
    Press(usize),
    Light(Option<usize>),
}

fn lay(tools: &[Tool], width: f32) -> Vec<(usize, f32, Option<f32>)> {
    let (b, gap) = (theme::sheet().button, theme::sheet().gap);
    let all = |n: usize| tools.iter().take(n).enumerate().fold(0.0, |x, (i, t)| x + b + if t.rule && i > 0 { 2.0 * gap } else if i > 0 { gap } else { 0.0 });
    let fits = if all(tools.len()) <= width { tools.len() } else { (0..tools.len()).rev().find(|&n| all(n) + gap + b <= width).unwrap_or(0) };
    let mut out = Vec::new();
    let mut x = 0.0;
    for (i, t) in tools.iter().take(fits).enumerate() {
        let mut line = None;
        if i > 0 {
            x += gap;
            if t.rule {
                line = Some(x);
                x += gap;
            }
        }
        out.push((i, x, line));
        x += b;
    }
    if fits < tools.len() {
        out.push((tools.len(), if fits > 0 { x + gap } else { 0.0 }, None));
    }
    out
}

impl Toolbar {
    pub fn new(tools: Vec<Tool>) -> Self {
        Toolbar { tools, lit: None, seen: Rc::new(Cell::new(f32::MAX)) }
    }

    pub fn update(&mut self, m: ToolMsg) -> Option<usize> {
        match m {
            ToolMsg::Press(i) => {
                let on = &mut self.tools.get_mut(i)?.on;
                *on = on.map(|o| !o);
                Some(i)
            }
            ToolMsg::Light(l) => {
                self.lit = l;
                None
            }
        }
    }

    pub fn node<M: 'static>(&self, wrap: impl Fn(ToolMsg) -> M + Clone + 'static) -> Node<M> {
        let (n, lit, b) = (self.tools.len(), self.lit, theme::sheet().button);
        let slots = {
            let (tools, seen) = (self.tools.clone(), self.seen.clone());
            Rc::new(move || lay(&tools, seen.get()))
        };
        let told = slots().iter().map(|&(i, x, _)| (self.tools.get(i).map_or_else(|| crate::t!("kit-more-tools").to_string(), |t| t.tip.clone()), x, b)).collect();
        let at = {
            let slots = slots.clone();
            move |x: f32| slots().into_iter().find(|&(_, a, _)| x >= a && x < a + b).map(|s| s.0)
        };
        let (at1, at2, keys_slots, menu_slots) = (at.clone(), at, slots.clone(), slots);
        let (w1, w2, w3) = (wrap.clone(), wrap.clone(), wrap);
        let (tools, seen, menu_tools) = (self.tools.clone(), self.seen.clone(), self.tools.clone());
        drawing((b, b), "", move |canvas, pen| {
            seen.set(canvas.width() as f32 / pen.scale);
            paint_tools(canvas, pen, &tools, seen.get(), lit, b);
        })
        .width(Length::Fill)
        .role(Role::Toolbar { tools: told, lit, on: self.tools.iter().map(|t| t.on).collect() })
        .called(crate::t!("kit-toolbar"))
        .on_pointer_when(move |p| match p {
            Pointer::Hover(x, _) | Pointer::Move(x, _) => Some(at1(x)).filter(|&l| l != lit).map(|l| w1(ToolMsg::Light(l))),
            Pointer::Leave => lit.map(|_| w1(ToolMsg::Light(None))),
            Pointer::Up(x, _) => at1(x).filter(|&i| i < n).map(|i| w1(ToolMsg::Press(i))),
            _ => None,
        })
        .on_hover()
        .on_keys(move |i| {
            let Input::Key { sym, .. } = *i else { return None };
            let order: Vec<usize> = keys_slots().iter().map(|s| s.0).collect();
            let last = order.len().saturating_sub(1);
            let here = lit.and_then(|l| order.iter().position(|&o| o == l));
            let go = |to: usize| Some(w2(ToolMsg::Light(order.get(to).copied())));
            match sym {
                Keysym::Left | Keysym::KP_Left => go(here.map_or(0, |h| h.saturating_sub(1))),
                Keysym::Right | Keysym::KP_Right => go(here.map_or(0, |h| (h + 1).min(last))),
                Keysym::Home | Keysym::KP_Home => go(0),
                Keysym::End | Keysym::KP_End => go(last),
                Keysym::Return | Keysym::KP_Enter | Keysym::space => lit.filter(|&l| l < n).map(|l| w2(ToolMsg::Press(l))),
                _ => None,
            }
        })
        .menu(move |x, _| {
            if at2(x) != Some(n) && lit != Some(n) {
                return Vec::new();
            }
            let shown = menu_slots().iter().filter(|s| s.0 < n).count();
            menu_tools.iter().enumerate().skip(shown).map(|(i, t)| {
                let row = MenuRow::send(t.icon.clone(), t.tip.clone(), w3(ToolMsg::Press(i)));
                if let Some(on) = t.on { row.checked(on) } else { row }
            }).collect()
        })
    }
}

pub fn status_bar<M>(items: Vec<Node<M>>) -> Node<M> {
    card(row(items)).area(Area::Foot, [0.0; 4]).role(Role::Status).called(crate::t!("kit-status"))
}

pub fn path_bar<M: 'static>(parts: &[String], keep: usize, go: impl Fn(usize) -> M + Clone + 'static) -> Node<M> {
    let n = parts.len();
    let hidden = if n > keep.max(1) + 1 { 1..n - keep.max(1) } else { 1..1 };
    let mut items = Vec::new();
    for (i, name) in parts.iter().enumerate() {
        if hidden.contains(&i) && i != hidden.start {
            continue;
        }
        if i > 0 {
            items.push(icon_image("go-next-symbolic", theme::sheet().icon, Tone::Calm));
        }
        if i == hidden.start && !hidden.is_empty() {
            let (from, go) = (hidden.start, go.clone());
            items.push(dropdown(parts[hidden.clone()].to_vec(), None, crate::t!("kit-path-more"), move |j| go(from + j)));
        } else if i + 1 == n {
            items.push(label(name.clone()).cut());
        } else {
            items.push(quiet_action("", name.clone()).on_press(go(i)));
        }
    }
    row(items).width(Length::Shrink).role(Role::Path).called(crate::t!("kit-path"))
}

fn paint_tools(canvas: &mut PixmapMut, pen: &Pen, tools: &[Tool], seen: f32, lit: Option<usize>, b: f32) {
    let (t, s) = (pen.theme, pen.scale);
    let h = canvas.height() as f32;
    let icon = (theme::sheet().icon * s).round();
    for (i, x, line) in lay(tools, seen) {
        if let Some(lx) = line {
            let thick = (theme::sheet().section_line_width * s).max(1.0);
            fill(canvas, Rect { x: (lx * s - thick / 2.0).round(), y: ((h - icon) / 2.0).round(), w: thick, h: icon }, 0.0, t.section_line);
        }
        let slot = Rect { x: x * s, y: ((h - b * s) / 2.0).round(), w: b * s, h: b * s };
        let on = tools.get(i).and_then(|t| t.on) == Some(true);
        let square = match (lit == Some(i), on) {
            (true, true) => Some(theme::mix(t.card(1), t.hover, 0.5)),
            (true, false) => Some(t.hover),
            (false, true) => Some(t.card(1)),
            _ => None,
        };
        if let Some(square) = square {
            fill(canvas, slot, theme::sheet().corner_of(theme::Part::IconButton) * s, square);
        }
        let name = tools.get(i).map_or("view-more-symbolic", |t| t.icon.as_str());
        let (ix, iy) = ((slot.x + (slot.w - icon) / 2.0).round() as i32, (slot.y + (slot.h - icon) / 2.0).round() as i32);
        pen.icons.draw(canvas, name, ix, iy, icon as u32, 1, t.fg);
    }
}
