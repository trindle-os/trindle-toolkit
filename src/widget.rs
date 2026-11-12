//! What a window shows, described afresh from the program's own state each time it changes: words, round icon buttons, switches and sliders, set in columns, rows and a bento grid of cards. The toolkit keeps the rest (where the pointer is, which widget has the keyboard) itself, matched up by each widget's place in the description or by the key it is given.

use std::cell::Cell;
use std::ops::Range;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use tiny_skia::{Pixmap, PixmapMut};

use super::colour::Colour;
use super::icon::Icons;
use super::dnd::{Given, Offered, Way};
use super::input::{Cursor, Input, Mods};
use super::text::{Fonts, Style};
use super::theme::{self, Theme};

mod bento;
mod flow;
mod landmark;
mod lay;
mod map;
mod measure;
mod node;
mod pills;
mod place;
mod purpose;
mod split;
mod texts;
pub use landmark::Landmark;
pub use split::{split, split_down, Start, SPLIT_LEAST, SPLIT_STEP};
pub(crate) use measure::lines_pad;
pub(crate) use lay::{id_of, puzzle_names, Anchor, Lay};
pub(crate) use pills::{has_arrow, pill_at, pill_widths, pills_in};
pub(crate) use place::Stays;
pub(crate) use texts::{action_fixed, action_room, action_words, word_button_style, Texts};
#[cfg(test)]
pub(crate) use texts::{action_fixed_in, action_words_in};
pub(crate) use split::first_side;
pub(crate) use bento::Kept;
#[cfg(any(test, feature = "testing"))]
pub use bento::check::loose_tiles;
#[cfg(any(test, feature = "testing"))]
pub use bento::shape;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Length {
    Fill,
    Shrink,
    Fixed(f32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Words {
    Plain,
    Title,
    Note,
    Page,
    Lead,
    Heading,
    Headline,
    Section,
    Label,
    Column,
    Side,
    Figure,
}

pub const WINDOW_MARGIN: f32 = 8.0;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Tone {
    #[default]
    Plain,
    Good,
    Warn,
    Problem,
    Calm,
    Accent,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Secret {
    #[default]
    Password,
    Pin,
    Key,
}

impl Secret {
    pub fn show(self) -> &'static str {
        match self {
            Secret::Password => crate::t!("kit-show-password"),
            Secret::Pin => crate::t!("kit-show-pin"),
            Secret::Key => crate::t!("kit-show-key"),
        }
    }

    pub fn hide(self) -> &'static str {
        match self {
            Secret::Password => crate::t!("kit-hide-password"),
            Secret::Pin => crate::t!("kit-hide-pin"),
            Secret::Key => crate::t!("kit-hide-key"),
        }
    }

    pub fn told(self, shown: bool) -> &'static str {
        match (self, shown) {
            (Secret::Password, true) => crate::t!("kit-password-shown"),
            (Secret::Password, false) => crate::t!("kit-password-hidden"),
            (Secret::Pin, true) => crate::t!("kit-pin-shown"),
            (Secret::Pin, false) => crate::t!("kit-pin-hidden"),
            (Secret::Key, true) => crate::t!("kit-key-shown"),
            (Secret::Key, false) => crate::t!("kit-key-hidden"),
        }
    }
}

pub enum What<M> {
    Column { items: Vec<Node<M>>, gap: f32, padding: f32, centre: bool },
    Row { items: Vec<Node<M>>, gap: f32, padding: f32 },
    Grid { items: Vec<Node<M>>, min: f32, gap: f32, even: bool, puzzle: bool,
           grow: bool,
           flow: bool },
    Card(Box<Node<M>>, Option<M>),
    Space,
    Label { text: String, words: Words, tone: Tone,
             marks: Vec<Range<usize>> },
    Rich { spans: Vec<crate::rich::Span>, words: Words, follow: Option<Box<dyn Fn(String) -> M>> },
    Ring { share: f32, size: f32, tone: Tone },
    Button { icon: String, tip: String, press: Option<M>, on: Option<bool>, face: Option<Face>, spoken: Option<String>, size: Option<f32>,
             working: Option<(f32, Tone)>,
             words: Option<String>,
             for_good: bool },
    Switch { on: bool, tip: String, change: Box<dyn Fn(bool) -> M> },
    Tick { on: bool, tip: String, change: Box<dyn Fn(bool) -> M> },
    Slider { value: f32, min: f32, max: f32, step: f32, tip: String, change: Box<dyn Fn(f32) -> M> },
    Entry { text: String, tip: String, change: Box<dyn Fn(String) -> M>, submit: Option<M>, secret: bool, kind: Secret, guarded: bool,
            show: u32, spoken: Option<String>,
            lines: usize,
            select: Option<Range<usize>>,
            leave: Option<M>,
            tab: Option<Box<dyn Fn(bool) -> M>>,
            updown: Option<Box<dyn Fn(bool) -> M>>,
            centre: bool,
            weight: Option<u16>,
            misspelt: Vec<Range<usize>>,
            back: Option<M>,
            steps: Option<Box<dyn Fn(i32) -> M>> },
    Choice { options: Vec<String>, chosen: usize, tip: String, choose: Box<dyn Fn(usize) -> M>,
             landed: Option<Box<dyn Fn(usize) -> M>>,
             drop_on: Option<Box<dyn Fn(usize) -> bool>>,
             middle: Option<Box<dyn Fn(usize) -> M>>,
             arrows: Option<Arrows<M>> },
    Dropdown { options: Vec<String>, chosen: Option<usize>, tip: String, choose: Box<dyn Fn(usize) -> M> },
    Scroll(Box<Node<M>>),
    ScrollX(Box<Node<M>>),
    List { rows: Vec<Item>, chosen: Option<usize>, choose: Box<dyn Fn(usize) -> M>, open: Option<Box<dyn Fn(usize) -> M>>, row: f32, many: Option<Many<M>>,
           menu: Option<Box<dyn Fn(usize, f32, f32) -> M>>,
           landed: Option<Box<dyn Fn(usize) -> M>>,
           drop_on: Option<Box<dyn Fn(usize) -> bool>>,
           middle: Option<Box<dyn Fn(usize) -> M>>,
           between: Option<Box<dyn Fn(usize) -> M>>,
           reorder: Option<Box<dyn Fn(usize, usize) -> M>>,
           move_in: Option<Box<dyn Fn(String, usize, usize) -> M>>,
           ahead: bool,
           here: bool,
           tree: Option<Branches<M>> },
    Spinner { size: f32 },
    Drawing { size: (f32, f32), tip: String, paint: Box<dyn Fn(&mut PixmapMut, &Pen)>,
              pointer: Option<Box<dyn Fn(Pointer, Mods) -> Option<M>>>, cursor: Option<Box<dyn Fn(f32, f32) -> Cursor>>, hovers: bool,
              passes: bool,
              right_drags: bool,
              keys: Option<Box<dyn Fn(&Input) -> Option<M>>>,
              scroll: Option<Box<dyn Fn(f32) -> M>>,
              click: Option<M>,
              twice: Option<M>,
              seen_only: bool,
              named: bool,
              cut: Option<Rc<Cell<bool>>>,
              live: Option<Frames>,
              alive: bool,
              menu: Option<Box<dyn Fn(f32, f32) -> Vec<MenuRow<M>>>> },
    Layers(Vec<Node<M>>),
    Split { items: Vec<Node<M>>, down: bool, start: Start, least: (f32, f32) },
    Divider { down: bool },
    Placed { inner: Box<Node<M>>, at: Box<dyn Fn((f32, f32), (f32, f32)) -> (f32, f32)>, stays: bool, within: bool },
    Slide { inner: Box<Node<M>>, by: f32, wheel: Option<Box<dyn Fn(f32) -> M>> },
    Aspect { inner: Box<Node<M>>, ratio: f32 },
}

impl<M> Drop for What<M> {
    fn drop(&mut self) {
        if let What::Entry { text, .. } = self {
            zeroize::Zeroize::zeroize(text);
        }
    }
}

pub type Face = Box<dyn Fn(&mut PixmapMut, Rect, Colour, &Pen)>;

pub struct MenuRow<M> {
    pub icon: String,
    pub words: String,
    pub keys: String,
    pub does: MenuDo<M>,
    pub rule: bool,
    pub check: Option<bool>,
    pub radio: bool,
    pub sub: Vec<MenuRow<M>>,
}

pub enum MenuDo<M> {
    Copy(String),
    Send(M),
}

impl<M> MenuRow<M> {
    pub fn send(icon: impl Into<String>, words: impl Into<String>, m: M) -> Self {
        MenuRow { icon: icon.into(), words: words.into(), keys: String::new(), does: MenuDo::Send(m), rule: false, check: None, radio: false, sub: Vec::new() }
    }

    pub fn copy(text: impl Into<String>) -> Self {
        MenuRow { icon: "edit-copy-symbolic".into(), words: crate::t!("kit-copy").into(), keys: "Ctrl+C".into(), does: MenuDo::Copy(text.into()), rule: false, check: None, radio: false, sub: Vec::new() }
    }

    pub fn keys(mut self, keys: impl Into<String>) -> Self {
        self.keys = keys.into();
        self
    }

    pub fn words(mut self, words: impl Into<String>) -> Self {
        self.words = words.into();
        self
    }

    pub fn rule(mut self) -> Self {
        self.rule = true;
        self
    }

    pub fn checked(mut self, on: bool) -> Self {
        self.check = Some(on);
        self
    }

    pub fn radio(mut self, on: bool) -> Self {
        self.check = Some(on);
        self.radio = true;
        self
    }

    pub fn submenu(mut self, rows: Vec<MenuRow<M>>) -> Self {
        self.sub = rows;
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Pointer {
    Down(f32, f32),
    Move(f32, f32),
    Up(f32, f32),
    Hover(f32, f32),
    Leave,
    Menu(f32, f32),
    Middle(f32, f32),
    RightDown(f32, f32),
    RightMove(f32, f32),
    RightUp(f32, f32),
}

pub struct Pen<'a> {
    pub theme: &'a Theme,
    pub fonts: &'a Fonts,
    pub icons: &'a Icons,
    pub scale: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Area {
    Top,
    Side,
    Foot,
    Controls,
    Between,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Item {
    pub icon: Option<String>,
    pub text: String,
    pub detail: Option<String>,
    pub under: Option<String>,
    pub mark: Option<Colour>,
    pub folder: bool,
    pub marks: Vec<Range<usize>>,
    pub under_marks: Vec<Range<usize>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Field {
    Text,
    Under,
}

pub fn item(text: impl Into<String>) -> Item {
    Item { icon: None, text: text.into(), detail: None, under: None, mark: None, folder: false, marks: Vec::new(), under_marks: Vec::new() }
}

impl Item {
    pub fn icon(mut self, name: impl Into<String>) -> Self {
        self.icon = Some(name.into());
        self
    }

    pub fn marked(mut self, field: Field, ranges: &[Range<usize>]) -> Self {
        match field {
            Field::Text => self.marks = crate::text::clean_marks(&self.text, ranges),
            Field::Under => self.under_marks = self.under.as_deref().map_or_else(Vec::new, |u| crate::text::clean_marks(u, ranges)),
        }
        self
    }

    pub fn detail(mut self, words: impl Into<String>) -> Self {
        self.detail = Some(words.into());
        self
    }

    pub fn under(mut self, words: impl Into<String>) -> Self {
        self.under = Some(words.into());
        self
    }

    pub fn mark(mut self, colour: Colour) -> Self {
        self.mark = Some(colour);
        self
    }

    pub fn folder(mut self) -> Self {
        self.folder = true;
        self
    }
}

pub struct Many<M> {
    pub(crate) picked: Vec<usize>,
    pub(crate) pick: Box<dyn Fn(Vec<usize>) -> M>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TreeRow {
    pub item: Item,
    pub depth: usize,
    pub open: Option<bool>,
}

pub fn tree_row(item: Item, depth: usize) -> TreeRow {
    TreeRow { item, depth, open: None }
}

impl TreeRow {
    pub fn branch(mut self, open: bool) -> Self {
        self.open = Some(open);
        self
    }
}

pub struct Branches<M> {
    pub(crate) rows: Vec<(usize, Option<bool>)>,
    pub(crate) toggle: Box<dyn Fn(usize) -> M>,
}

impl<M> Branches<M> {
    pub(crate) fn parent(&self, r: usize) -> Option<usize> {
        let d = self.rows.get(r)?.0;
        (0..r).rev().find(|&i| self.rows[i].0 < d)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Sub {
    None,
    Rows(Rc<Vec<String>>, Option<usize>),
    Words(String),
}

pub struct Arrows<M> {
    pub(crate) subs: Vec<Sub>,
    pub(crate) opening: Box<dyn Fn(usize) -> M>,
    pub(crate) pick: Box<dyn Fn(usize, usize) -> M>,
}

pub(crate) const ARROW: f32 = 14.0;
pub(crate) const ARROW_GAP: f32 = 2.0;

pub struct Node<M> {
    pub what: What<M>,
    pub width: Length,
    pub height: Length,
    pub key: Option<String>,
    pub escape: Option<M>,
    pub first: bool,
    pub grab: u32,
    pub off: bool,
    pub flat: bool,
    pub chip: bool,
    pub snug: bool,
    pub area: Option<(Area, [f32; 4])>,
    pub opacity: f32,
    pub lit: f32,
    pub cut: u8,
    pub keep_lines: u8,
    pub seek: Option<(u32, f32, f32)>,
    pub nudge: Option<(u32, f32)>,
    pub glow: u32,
    pub sunk: bool,
    pub filled: bool,
    pub tint: f32,
    pub beside: bool,
    pub stacks: bool,
    pub wraps: bool,
    pub twice: bool,
    pub own_act: Option<&'static str>,
    pub side: bool,
    pub floats: bool,
    pub span: Option<Span>,
    pub band: bool,
    pub outside: Option<Outside<M>>,
    pub current: bool,
    pub claims: Option<M>,
    pub context: Option<Box<dyn Fn() -> Vec<MenuRow<M>>>>,
    pub round: Option<(f32, f32)>,
    pub leads: Option<String>,
    pub why: Option<String>,
    pub whole: Option<String>,
    pub own_labels: bool,
    pub quiet: bool,
    pub when_armed: Option<String>,
    pub(crate) kept: Option<(bool, f32)>,
    pub says: Option<Box<Says>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Urgency {
    Polite,
    Assertive,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Reading {
    pub lines: Vec<String>,
    pub caret: Option<(usize, usize)>,
    pub selection: Option<((usize, usize), (usize, usize))>,
    pub editable: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SaidRow {
    pub words: String,
    pub top: f32,
    pub height: f32,
    pub lit: bool,
    pub check: Option<bool>,
    pub radio: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SaidCells {
    pub index: usize,
    pub cells: Vec<String>,
    pub top: f32,
    pub height: f32,
    pub chosen: bool,
    pub group: Option<String>,
    pub branch: Option<SaidBranch>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SaidBranch {
    pub level: usize,
    pub open: Option<bool>,
    pub place: usize,
    pub of: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SaidTable {
    pub headings: Vec<String>,
    pub columns: Vec<(f32, f32)>,
    pub sort: Option<(usize, bool)>,
    pub total: usize,
    pub head_height: f32,
    pub rows: Vec<SaidCells>,
    pub column: Option<usize>,
    pub many: bool,
    pub tree: bool,
    pub cursor: Option<usize>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SaidTab {
    pub index: usize,
    pub words: String,
    pub chosen: bool,
    pub closable: bool,
    pub x: f32,
    pub width: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SaidTabs {
    pub total: usize,
    pub tabs: Vec<SaidTab>,
    pub height: f32,
    pub more: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Says {
    pub name: Option<String>,
    pub live: Option<Urgency>,
    pub alert: bool,
    pub busy: bool,
    pub progress: Option<Option<f32>>,
    pub reading: Option<Reading>,
    pub menu: Option<Vec<SaidRow>>,
    pub table: Option<SaidTable>,
    pub tabs: Option<SaidTabs>,
    pub dialog: Option<bool>,
    pub unseen: bool,
    pub radio: Option<bool>,
    pub landmark: Option<Landmark>,
    pub purpose: Option<crate::input::Purpose>,
    pub role: Option<super::role::Role>,
    pub words: Option<String>,
    pub misspelt: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Held {
    pub offered: Offered,
    pub part: Part,
    pub mods: Mods,
}

impl Held {
    pub fn usual(&self) -> Option<Way> {
        super::dnd::usual_way(self.offered.ways, self.mods.shift, self.mods.ctrl)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Part {
    Row(usize),
    Gap(usize),
    Point(f32, f32),
    Whole,
}

pub struct Outside<M> {
    pub(crate) kinds: Vec<String>,
    pub(crate) would: Box<dyn Fn(&Held) -> Option<Way>>,
    pub(crate) take: Box<dyn Fn(&Held, Given, Way) -> M>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Span {
    pub least: u8,
    pub most: u8,
    pub words: f32,
    pub grows: bool,
    pub flat: bool,
}

const SLIDE_ROOM: f32 = 100_000.0;

fn node<M>(what: What<M>, width: Length) -> Node<M> {
    Node { what, width, height: Length::Shrink, key: None, escape: None, first: false, grab: 0, off: false, flat: false, chip: false, snug: false, opacity: 1.0, lit: 0.0, cut: 0, keep_lines: 0, seek: None, nudge: None, glow: 0, sunk: false, filled: false, tint: 0.0, beside: false, stacks: false, wraps: false, twice: false, own_act: None, side: false, area: None, floats: false, span: None, band: false, outside: None, current: false, claims: None, context: None, round: None, leads: None, why: None, whole: None, own_labels: false, quiet: false, when_armed: None, kept: None, says: None }
}

pub fn column<M>(items: Vec<Node<M>>) -> Node<M> {
    node(What::Column { items, gap: theme::sheet().gap, padding: 0.0, centre: false }, Length::Fill)
}

pub fn row<M>(items: Vec<Node<M>>) -> Node<M> {
    node(What::Row { items, gap: theme::sheet().gap, padding: 0.0 }, Length::Fill)
}

pub fn grid<M>(min: f32, items: Vec<Node<M>>) -> Node<M> {
    node(What::Grid { items, min, gap: theme::bento().gap, even: false, puzzle: false, grow: false, flow: false }, Length::Fill)
}

pub fn even_grid<M>(min: f32, items: Vec<Node<M>>) -> Node<M> {
    node(What::Grid { items, min, gap: theme::bento().gap, even: true, puzzle: false, grow: false, flow: false }, Length::Fill)
}

pub fn flow<M>(items: Vec<Node<M>>) -> Node<M> {
    node(What::Grid { items, min: 0.0, gap: theme::sheet().gap, even: false, puzzle: false, grow: false, flow: true }, Length::Fill)
}

pub fn bento<M>(min: f32, gap: f32, items: Vec<Node<M>>) -> Node<M> {
    let mut out: Vec<Node<M>> = Vec::new();
    let mut run: Vec<Node<M>> = Vec::new();
    for mut item in items {
        if item.beside {
            if let What::Grid { items, even: false, flow: false, .. } = &mut item.what {
                if items.iter().all(|i| i.beside) {
                    run.append(items);
                    continue;
                }
            }
            run.push(item);
            continue;
        }
        if !run.is_empty() {
            out.push(node(What::Grid { items: std::mem::take(&mut run), min, gap, even: false, puzzle: true, grow: false, flow: false }, Length::Fill));
        }
        out.push(item);
    }
    if !run.is_empty() {
        out.push(node(What::Grid { items: run, min, gap, even: false, puzzle: true, grow: false, flow: false }, Length::Fill));
    }
    column(out).gap(gap)
}

pub fn spinner<M>() -> Node<M> {
    node(What::Spinner { size: theme::sheet().spinner }, Length::Shrink)
}

pub fn card<M>(inner: Node<M>) -> Node<M> {
    node(What::Card(Box::new(inner), None), Length::Fill)
}

pub fn banded_list<M>(rows: Vec<Node<M>>) -> Vec<Node<M>> {
    rows.into_iter().enumerate().map(|(i, r)| if i % 2 == 1 { r.banded() } else { card(r).snug().flat() }).collect()
}

pub fn space<M>() -> Node<M> {
    node(What::Space, Length::Fill)
}

fn words<M>(text: String, words: Words) -> Node<M> {
    node(What::Label { text, words, tone: Tone::Plain, marks: Vec::new() }, Length::Shrink)
}

pub fn rich<M>(spans: Vec<crate::rich::Span>) -> Node<M> {
    node(What::Rich { spans, words: Words::Plain, follow: None }, Length::Shrink)
}

pub fn label<M>(text: impl Into<String>) -> Node<M> {
    words(text.into(), Words::Plain)
}

pub fn title<M>(text: impl Into<String>) -> Node<M> {
    words(text.into(), Words::Title)
}

pub fn page_title<M>(text: impl Into<String>) -> Node<M> {
    words(text.into(), Words::Page)
}

pub fn lead<M>(text: impl Into<String>) -> Node<M> {
    words(text.into(), Words::Lead)
}

pub fn heading<M>(text: impl Into<String>) -> Node<M> {
    words(text.into(), Words::Heading)
}

pub fn headline<M>(text: impl Into<String>) -> Node<M> {
    words(text.into(), Words::Headline)
}

pub fn section<M>(text: impl Into<String>) -> Node<M> {
    words(text.into(), Words::Section)
}

pub fn form_label<M>(text: impl Into<String>) -> Node<M> {
    words(text.into(), Words::Label)
}

pub fn column_heading<M>(text: impl Into<String>) -> Node<M> {
    words(text.into(), Words::Column)
}

pub fn side_heading<M>(text: impl Into<String>) -> Node<M> {
    words(text.into(), Words::Side)
}

pub fn figure<M>(text: impl Into<String>) -> Node<M> {
    words(text.into(), Words::Figure)
}

pub fn note<M>(text: impl Into<String>) -> Node<M> {
    words(text.into(), Words::Note)
}

pub fn ring<M>(share: f32, size: f32) -> Node<M> {
    node(What::Ring { share: share.clamp(0.0, 1.0), size, tone: Tone::Plain }, Length::Shrink)
}

pub fn icon_circle<M>(icon: impl Into<String>) -> Node<M> {
    let icon = icon.into();
    let across = theme::sheet().badge;
    drawing((across, across), "", move |canvas, pen| {
        let (t, s) = (pen.theme, pen.scale);
        let inner = theme::sheet().badge_icon;
        let r = across * s / 2.0;
        let (cx, cy) = (canvas.width() as f32 / 2.0, canvas.height() as f32 / 2.0);
        if let Some(path) = crate::ui::rounded(Rect { x: cx - r, y: cy - r, w: 2.0 * r, h: 2.0 * r }, theme::corner_px(theme::Part::Badge, 2.0 * r, s)) {
            let mut paint = tiny_skia::Paint::default();
            paint.set_color(t.bg.to_skia());
            paint.anti_alias = true;
            canvas.fill_path(&path, &paint, tiny_skia::FillRule::Winding, tiny_skia::Transform::identity(), None);
        }
        let (whole, inner) = (s.ceil() as u32, inner.round().max(1.0) as u32);
        let px = (inner * whole) as f32;
        let at = |c: f32| (c - px / 2.0).round() as i32;
        pen.icons.draw(canvas, &icon, at(cx), at(cy), inner, whole, t.fg);
    }).only_when_seen()
}

pub fn icon_image<M>(icon: impl Into<String>, size: f32, tone: Tone) -> Node<M> {
    let icon = icon.into();
    drawing((size, size), "", move |canvas, pen| {
        let colour = match tone {
            Tone::Calm => pen.theme.muted,
            Tone::Problem => pen.theme.danger_text,
            _ => pen.theme.fg,
        };
        let whole = pen.scale.ceil() as u32;
        let px = (size * pen.scale).round() as u32;
        pen.icons.draw(canvas, &icon, 0, 0, (px / whole.max(1)).max(1), whole.max(1), colour);
    }).only_when_seen()
}

pub fn avatar<M>(picture: Option<Rc<image::DynamicImage>>, name: impl Into<String>, size: f32) -> Node<M> {
    let name = name.into();
    let said = name.clone();
    drawing((size, size), "", move |canvas, pen| {
        let side = canvas.width().min(canvas.height());
        let Some(mut round) = Pixmap::new(side, side) else { return };
        match picture.as_ref().and_then(|img| super::picture::shared(img, true, side, side)) {
            Some(ready) => super::picture::draw_prepared(&mut round.as_mut(), &ready, 0, 0),
            None => {
                round.fill(pen.theme.bg.to_skia());
                let letters: String = name.split_whitespace().filter_map(|w| w.chars().next()).take(2).flat_map(char::to_uppercase).collect();
                let style = Style { weight: 600, ..Style::new(side as f32 * 0.4) };
                let laid = pen.fonts.layout(&letters, &style, None);
                pen.fonts.draw(&mut round.as_mut(), &laid, (side as f32 - laid.width) / 2.0, (side as f32 - laid.height) / 2.0, pen.theme.fg);
            }
        }
        let mut mask = tiny_skia::Mask::new(side, side).unwrap();
        let across = side as f32;
        if let Some(path) = crate::ui::rounded(Rect { x: 0.0, y: 0.0, w: across, h: across }, theme::corner_px(theme::Part::Picture, across, pen.scale)) {
            mask.fill_path(&path, tiny_skia::FillRule::Winding, true, tiny_skia::Transform::identity());
        }
        canvas.draw_pixmap(0, 0, round.as_ref(), &tiny_skia::PixmapPaint::default(), tiny_skia::Transform::identity(), Some(&mask));
    }).only_when_seen().called(said)
}

pub fn image_view<M>(img: Rc<image::DynamicImage>, name: impl Into<String>, size: (f32, f32)) -> Node<M> {
    drawing(size, "", move |canvas, _| {
        let (iw, ih) = image::GenericImageView::dimensions(img.as_ref());
        if iw == 0 || ih == 0 {
            return;
        }
        let k = (canvas.width() as f32 / iw as f32).min(canvas.height() as f32 / ih as f32);
        let (w, h) = (((iw as f32 * k).round() as u32).max(1), ((ih as f32 * k).round() as u32).max(1));
        if let Some(ready) = super::picture::shared(&img, false, w, h) {
            super::picture::draw_prepared(canvas, &ready, ((canvas.width() - w.min(canvas.width())) / 2) as i32, ((canvas.height() - h.min(canvas.height())) / 2) as i32);
        }
    }).only_when_seen().called(name)
}

pub fn section_rule<M>() -> Node<M> {
    drawing((1.0, theme::sheet().section_line_width), "", |canvas, pen| {
        let r = tiny_skia::Rect::from_xywh(0.0, 0.0, canvas.width() as f32, canvas.height() as f32);
        if let Some(r) = r {
            canvas.fill_rect(r, &super::colour::paint(pen.theme.section_line), tiny_skia::Transform::identity(), None);
        }
    }).width(Length::Fill)
}

pub fn drawing<M>(size: (f32, f32), tip: impl Into<String>, paint: impl Fn(&mut PixmapMut, &Pen) + 'static) -> Node<M> {
    node(What::Drawing { size, tip: tip.into(), paint: Box::new(paint), pointer: None, cursor: None, hovers: false, passes: false, right_drags: false, keys: None, scroll: None, click: None, twice: None, seen_only: false, named: false, cut: None,
                         live: None, alive: false, menu: None }, Length::Shrink)
}

pub type Frames = Arc<Mutex<Option<Pixmap>>>;

pub fn moving<M>(size: (f32, f32), tip: impl Into<String>, frames: Frames) -> Node<M> {
    let f = frames.clone();
    let mut n = drawing(size, tip, move |canvas, _| {
        let r = Rect { x: 0.0, y: 0.0, w: canvas.width() as f32, h: canvas.height() as f32 };
        put_frame(canvas, r, &f, false);
    });
    if let What::Drawing { live, .. } = &mut n.what {
        *live = Some(frames);
    }
    n
}

pub fn put_frame(canvas: &mut PixmapMut, r: Rect, frames: &Frames, bgra: bool) {
    let (rx, ry) = (r.x.round() as i64, r.y.round() as i64);
    let (rw, rh) = (r.w.round().max(0.0) as i64, r.h.round().max(0.0) as i64);
    let (cw, ch) = (canvas.width() as usize, canvas.height() as usize);
    let (left, top) = (rx.max(0), ry.max(0));
    let (right, bottom) = (rx.saturating_add(rw).min(cw as i64), ry.saturating_add(rh).min(ch as i64));
    if right <= left || bottom <= top {
        return;
    }
    let (cut_x, cut_y) = ((left - rx) as usize, (top - ry) as usize);
    let (x0, y0, w, h) = (left as usize, top as usize, (right - left) as usize, (bottom - top) as usize);
    let Ok(slot) = frames.lock() else { return };
    let Some(px) = slot.as_ref() else {
        let data = canvas.data_mut();
        for y in y0..y0 + h {
            for p in data[(y * cw + x0) * 4..(y * cw + x0 + w) * 4].chunks_exact_mut(4) {
                p.copy_from_slice(&[0, 0, 0, 255]);
            }
        }
        return;
    };
    let whole = (rw as usize, rh as usize);
    if (px.width() as usize, px.height() as usize) == whole {
        let (from, data) = (px.data(), canvas.data_mut());
        for y in 0..h {
            let at = ((cut_y + y) * whole.0 + cut_x) * 4;
            let src = &from[at..at + w * 4];
            let dst = &mut data[((y0 + y) * cw + x0) * 4..((y0 + y) * cw + x0 + w) * 4];
            if !bgra {
                for (d, s) in dst.chunks_exact_mut(4).zip(src.chunks_exact(4)) {
                    d.copy_from_slice(&[s[2], s[1], s[0], s[3]]);
                }
            } else {
                dst.copy_from_slice(src);
            }
        }
        return;
    }
    let (kx, ky) = (whole.0 as f32 / px.width() as f32, whole.1 as f32 / px.height() as f32);
    let paint = tiny_skia::PixmapPaint { quality: tiny_skia::FilterQuality::Bilinear, ..Default::default() };
    canvas.draw_pixmap(0, 0, px.as_ref(), &paint, tiny_skia::Transform::from_row(kx, 0.0, 0.0, ky, rx as f32, ry as f32), None);
    if !bgra {
        let data = canvas.data_mut();
        for y in y0..y0 + h {
            for p in data[(y * cw + x0) * 4..(y * cw + x0 + w) * 4].chunks_exact_mut(4) {
                p.swap(0, 2);
            }
        }
    }
}

pub fn put_frame_on(p: &mut crate::paint::Painter, r: Rect, frames: &Frames, bgra: bool) {
    let frames = frames.clone();
    p.pixels(crate::draw::around(r.x, r.y, r.w, r.h, 1.0), move |c| put_frame(c, r, &frames, bgra));
}

pub fn layers<M>(items: Vec<Node<M>>) -> Node<M> {
    Node { height: Length::Fill, ..node(What::Layers(items), Length::Fill) }
}

pub fn placed<M>(inner: Node<M>, at: impl Fn((f32, f32), (f32, f32)) -> (f32, f32) + 'static) -> Node<M> {
    Node { height: Length::Fill, ..node(What::Placed { inner: Box::new(inner), at: Box::new(at), stays: false, within: false }, Length::Fill) }
}

pub struct InPlace<M> {
    pub place: Rect,
    pub entry: Node<M>,
    pub problem: Option<String>,
}

pub fn typing_over<M>(under: Node<M>, edit: Option<InPlace<M>>) -> Node<M> {
    let (width, height) = (under.width, under.height);
    let mut items = vec![under];
    if let Some(InPlace { place, entry, problem }) = edit {
        let (x, y) = (place.x, place.y);
        items.push(placed(entry.width(Length::Fixed(place.w)).height(Length::Fixed(place.h)).snug(), move |_, _| (x, y)));
        if let Some(why) = problem {
            let under = y + place.h + theme::sheet().rich_gap;
            items.push(placed(card(note(why).tone(Tone::Problem)).chip(), move |(w, _), (room, _)| (x.min(room - w).max(0.0), under)));
        }
    }
    let mut all = layers(items);
    all.width = width;
    all.height = height;
    all
}

pub fn slide<M>(inner: Node<M>, width: f32, by: f32) -> Node<M> {
    node(What::Slide { inner: Box::new(inner), by, wheel: None }, Length::Fixed(width))
}

pub fn aspect<M>(ratio: f32, inner: Node<M>) -> Node<M> {
    node(What::Aspect { inner: Box::new(inner), ratio: ratio.max(f32::EPSILON) }, Length::Fill)
}

pub(crate) const NAME_WIDTH: f32 = 96.0;
pub(crate) const NAME_GAP: f32 = 4.0;

pub(crate) fn name_style(scale: f32) -> Style {
    let k = theme::sheet();
    Style { weight: k.label_weight, ..Style::new(k.label_text * scale) }
}

pub fn remove_by_hand<M>(icon: impl Into<String>, tip: impl Into<String>, act: &'static str) -> Node<M> {
    let mut b = remove(icon, tip);
    b.own_act = Some(act);
    b
}

pub fn remove<M>(icon: impl Into<String>, tip: impl Into<String>) -> Node<M> {
    let mut b = button(icon, tip);
    b.twice = true;
    b
}

pub fn button<M>(icon: impl Into<String>, tip: impl Into<String>) -> Node<M> {
    node(What::Button { icon: icon.into(), tip: tip.into(), press: None, on: None, face: None, spoken: None, size: None, working: None, words: None, for_good: false }, Length::Shrink)
}

pub fn link<M>(address: &str) -> Node<M> {
    let read = super::link::read(address, crate::common::style::link_label_most());
    button(super::link::ICON, read.label).spoken(read.spoken)
}

pub fn link_named<M>(name: &str, address: &str) -> Node<M> {
    button(super::link::ICON, name).leads_to(address)
}

pub fn main_action<M>(icon: impl Into<String>, words: impl Into<String>) -> Node<M> {
    let words = words.into();
    node(What::Button { icon: icon.into(), tip: words.clone(), press: None, on: None, face: None, spoken: None, size: None, working: None,
                        words: Some(words), for_good: false }, Length::Shrink)
}

pub fn quiet_action<M>(icon: impl Into<String>, words: impl Into<String>) -> Node<M> {
    let mut n = main_action(icon, words);
    n.quiet = true;
    n
}

pub(crate) const QUIET_PADDING: f32 = 12.0;

pub fn main_word<M>(words: impl Into<String>) -> Node<M> {
    main_action("", words)
}

pub(crate) fn is_word_button(icon: &str, on: Option<bool>) -> bool {
    icon.is_empty() && on.is_some()
}

pub(crate) fn action_icon_room(scale: f32) -> f32 {
    let s = theme::sheet();
    (s.action_gap * scale).round() + (s.icon * scale).round()
}

pub fn word_button<M>(words: impl Into<String>) -> Node<M> {
    let words = words.into();
    node(What::Button { icon: String::new(), tip: words.clone(), press: None, on: Some(false), face: None, spoken: None, size: None, working: None,
                        words: Some(words), for_good: false }, Length::Shrink)
}

pub const TICK_SIZE: f32 = 18.0;

pub fn tick<M>(on: bool, tip: impl Into<String>, change: impl Fn(bool) -> M + 'static) -> Node<M> {
    node(What::Tick { on, tip: tip.into(), change: Box::new(change) }, Length::Shrink)
}

pub fn switch<M>(on: bool, tip: impl Into<String>, change: impl Fn(bool) -> M + 'static) -> Node<M> {
    node(What::Switch { on, tip: tip.into(), change: Box::new(change) }, Length::Shrink)
}

pub fn slider<M>(value: f32, min: f32, max: f32, step: f32, tip: impl Into<String>,
                 change: impl Fn(f32) -> M + 'static) -> Node<M> {
    node(What::Slider { value, min, max, step: step.max(f32::EPSILON), tip: tip.into(), change: Box::new(change) }, Length::Fill)
}

pub fn entry<M>(text: impl Into<String>, tip: impl Into<String>, change: impl Fn(String) -> M + 'static) -> Node<M> {
    node(What::Entry { text: text.into(), tip: tip.into(), change: Box::new(change), submit: None, secret: false, kind: Secret::Password, guarded: false, show: 0, spoken: None, lines: 1,
                       select: None, leave: None, tab: None, updown: None, centre: false, weight: None, misspelt: Vec::new(), back: None, steps: None }, Length::Fill)
}

pub fn choice<M>(options: Vec<String>, chosen: usize, tip: impl Into<String>, choose: impl Fn(usize) -> M + 'static) -> Node<M> {
    let chosen = chosen.min(options.len().saturating_sub(1));
    node(What::Choice { options, chosen, tip: tip.into(), choose: Box::new(choose), landed: None, drop_on: None, middle: None, arrows: None }, Length::Shrink)
}

pub fn nothing_chosen() -> &'static str {
    crate::t!("kit-choose-one")
}

pub(crate) const CHEVRON: f32 = 16.0;
pub(crate) const CHEVRON_GAP: f32 = 8.0;

pub fn dropdown<M>(options: Vec<String>, chosen: Option<usize>, tip: impl Into<String>, choose: impl Fn(usize) -> M + 'static) -> Node<M> {
    let chosen = chosen.filter(|&i| i < options.len());
    node(What::Dropdown { options, chosen, tip: tip.into(), choose: Box::new(choose) }, Length::Shrink)
}

pub fn is_short(options: &[String]) -> bool {
    options.len() == 2 || (options.len() <= 4 && options.iter().map(|o| o.chars().count()).sum::<usize>() <= 40)
}

pub fn one_of<M>(options: Vec<String>, chosen: Option<usize>, tip: impl Into<String>, choose: impl Fn(usize) -> M + 'static) -> Node<M> {
    match chosen.filter(|&i| i < options.len()) {
        Some(at) if is_short(&options) => choice(options, at, tip, choose),
        chosen => dropdown(options, chosen, tip, choose),
    }
}

pub fn scroll<M>(key: impl Into<String>, inner: Node<M>) -> Node<M> {
    Node { height: Length::Fill, key: Some(key.into()), ..node(What::Scroll(Box::new(inner)), Length::Fill) }
}

pub fn scroll_x<M>(key: impl Into<String>, inner: Node<M>) -> Node<M> {
    Node { height: Length::Fill, key: Some(key.into()), ..node(What::ScrollX(Box::new(inner)), Length::Fill) }
}

pub fn list<M>(rows: Vec<Item>, chosen: Option<usize>, choose: impl Fn(usize) -> M + 'static) -> Node<M> {
    node(What::List { rows, chosen, choose: Box::new(choose), open: None, row: theme::sheet().row, many: None, menu: None, landed: None, drop_on: None, middle: None, between: None, reorder: None, move_in: None, ahead: false, here: false, tree: None }, Length::Fill)
}

pub fn tree<M>(rows: Vec<TreeRow>, chosen: Option<usize>, choose: impl Fn(usize) -> M + 'static,
               toggle: impl Fn(usize) -> M + 'static, open: impl Fn(usize) -> M + 'static) -> Node<M> {
    let at = rows.iter().map(|r| (r.depth, r.open)).collect();
    let mut n = list(rows.into_iter().map(|r| r.item).collect(), chosen, choose).on_open(open);
    if let What::List { tree, .. } = &mut n.what {
        *tree = Some(Branches { rows: at, toggle: Box::new(toggle) });
    }
    n
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub fn contains(&self, x: f32, y: f32) -> bool {
        x >= self.x && x < self.x + self.w && y >= self.y && y < self.y + self.h
    }

    pub fn inset(&self, by: f32) -> Rect {
        Rect { x: self.x + by, y: self.y + by, w: (self.w - 2.0 * by).max(0.0), h: (self.h - 2.0 * by).max(0.0) }
    }

    pub fn intersect(&self, o: &Rect) -> Rect {
        let (x, y) = (self.x.max(o.x), self.y.max(o.y));
        let (r, b) = ((self.x + self.w).min(o.x + o.w), (self.y + self.h).min(o.y + o.h));
        Rect { x, y, w: (r - x).max(0.0), h: (b - y).max(0.0) }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Spot {
    pub id: String,
    pub path: Vec<usize>,
    pub rect: Rect,
    pub clip: Option<Rect>,
    pub full: f32,
}

impl Spot {
    pub fn shows(&self, x: f32, y: f32) -> bool {
        self.rect.contains(x, y) && self.clip.is_none_or(|c| c.contains(x, y))
    }
}

#[cfg(test)]
mod tests;
