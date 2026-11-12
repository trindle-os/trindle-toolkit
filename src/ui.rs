//! A window full of widgets: the program says what to show from its own state, the toolkit paints it in Trindle's look, moves the keyboard between widgets with Tab, shows each widget's name briefly when the pointer rests on it, and hands back what the person asked for, which the program uses to change its state before saying again what to show.

use crate::colour::paint;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::ops::Range;
use std::rc::Rc;
use std::time::{Duration, Instant};

use tiny_skia::{Path, PathBuilder, Pixmap, PixmapMut, Stroke, Transform};

use super::ahead;
use super::colour::Colour;
use super::dnd::{Wanted, Way};
use super::edit::{self, Edit};
use super::event::Loop;
use super::icon::Icons;
use super::paint::Painter;
use super::shadow::Shadows;
use super::input::{Button, Cursor, Input, Keysym, Mods, Typing};
use super::text::{Fonts, Layout, Style};
use super::theme::{self, Theme};
use super::widget::{self as w, id_of, puzzle_names, Anchor, Area, Item, Kept, Lay, Node, Pen, Rect, Spot, Texts, Tone, What, Words};
use super::window::dmabuf::Drawn;
use crate::paint::remote::Answer;
use crate::paint::Painting;
use super::window::{self, Display, Dragged, Kind, Window};

mod access;
mod delivering;
mod following;
mod settling;
mod shown;
mod timing;
mod rereading;
mod places;
mod lists;
mod taking;
mod pressing;
mod keys;
mod radio;
mod entries;
mod painting;
mod parts;
mod controls;
mod tips;
mod copying;
mod dropdown;
mod eye;
mod fling;
mod rich;
pub(crate) mod outside;
mod sideways;
mod undoing;
#[cfg(test)]
mod sideways_tests;
#[cfg(test)]
mod between_lists_tests;
#[cfg(test)]
mod undoing_tests;
#[cfg(test)]
mod landmark_tests;
#[cfg(test)]
mod purpose_tests;
mod outline;
#[cfg(test)]
mod outline_tests;
#[cfg(test)]
mod panel_tests;
#[cfg(test)]
mod notices_tests;
#[cfg(test)]
mod look_tests;
mod inspect;
#[cfg(test)]
mod inspect_tests;
#[cfg(test)]
mod zero_config_tests;
#[cfg(test)]
mod scale_tests;
mod split;
pub use split::{load_splits, save_splits};
mod band;
#[cfg(test)]
mod outside_tests;
#[cfg(test)]
mod paste_bell_tests;
#[cfg(test)]
mod eye_tests;
#[cfg(test)]
mod dropdown_tests;
#[cfg(test)]
mod copying_tests;
#[cfg(test)]
mod spelling_tests;
#[cfg(test)]
mod shared_tests;
#[cfg(test)]
mod still_tests;
#[cfg(test)]
mod damage_tests;
#[cfg(test)]
mod rows_tests;
#[cfg(test)]
mod naming_tests;
#[cfg(test)]
mod leaving_tests;
#[cfg(test)]
mod cut_tests;
#[cfg(test)]
mod overflow_tests;
#[cfg(test)]
mod whole_tests;
#[cfg(test)]
mod apart_tests;
#[cfg(any(test, feature = "testing"))]
mod testing;
#[cfg(test)]
mod beside_long_words_tests;
#[cfg(test)]
mod for_good_tests;
#[cfg(test)]
mod shut_tests;
#[cfg(test)]
mod terminal_tests;
#[cfg(test)]
mod text_tests;
#[cfg(test)]
mod kit_controls_tests;
#[cfg(test)]
mod widgets_tests;
#[cfg(test)]
mod nav_tests;
#[cfg(test)]
mod tree_tests;
#[cfg(test)]
mod small_tests;
#[cfg(test)]
mod bars_tests;
#[cfg(test)]
mod context_tests;
#[cfg(test)]
mod charts_tests;
#[cfg(test)]
mod photos_tests;
#[cfg(test)]
mod frame_tests;
#[cfg(test)]
mod split_tests;
#[cfg(test)]
mod dock_tests;
#[cfg(test)]
mod band_tests;
#[cfg(test)]
mod role_tests;
#[cfg(test)]
mod segments_tests;
#[cfg(test)]
mod lines_tests;

use copying::CopyMenu;
#[cfg(test)]
use delivering::{announced, apply_messages, outgoing, paste_asked, show};
use dropdown::DropOpen;

pub use crate::common::tip::Labels;

pub fn tip_rest() -> Duration {
    theme::sheet().label_rest
}

pub fn tip_stays() -> Duration {
    theme::sheet().label_stays
}
const FRAME: Duration = Duration::from_millis(16);
const EDGE_STEP: Duration = Duration::from_millis(60);
const PART_ROOM: f32 = 64.0;
const UNSEEN_STEP: Duration = Duration::from_millis(250);
const SPIN: Duration = Duration::from_secs(1);
const PAGE_STEPS: f32 = 10.0;
pub const MULTI_CLICK: Duration = Duration::from_millis(400);
const TWICE_SLOP: f32 = 6.0;
const SWIPE_SLOP: f32 = 8.0;
const HOLD: Duration = Duration::from_millis(500);
const HOLD_SLOP: f32 = 10.0;
const RIGHT_SLOP: f32 = 6.0;
const FADED_ROOM: f32 = 24.0;
const BELL_GAP: Duration = Duration::from_millis(340);

pub trait App {
    type Msg;
    fn update(&mut self, msg: Self::Msg);
    fn unchanged(&self, _msg: &Self::Msg) -> bool {
        false
    }
    fn view(&self) -> Node<Self::Msg>;
    fn focused(&self) -> Option<Self::Msg> {
        None
    }
    fn to_clipboard(&mut self) -> Option<String> {
        None
    }
    fn wants_paste(&mut self) -> bool {
        false
    }
    fn pasted(&self, _text: String) -> Option<Self::Msg> {
        None
    }
    fn announcement(&mut self) -> Option<String> {
        None
    }
    fn wants_bell(&mut self) -> bool {
        false
    }
    fn window_title(&self) -> Option<String> {
        None
    }
    fn intercept(&self, _input: &Input, _keys: bool) -> Option<Self::Msg> {
        None
    }
    fn intercept_with(&self, input: &Input, at: Keyboard) -> Option<Self::Msg> {
        self.intercept(input, at.keys)
    }
    fn typing(&self) -> Option<Typing> {
        None
    }
    fn pressed(&mut self, _place: Rect) {}
    fn resized(&mut self, _w: f32, _h: f32) -> bool {
        false
    }
    fn blend(&self) -> Option<(u64, f32)> {
        None
    }
    fn dragging(&self) -> Option<Lifted> {
        None
    }
    fn drop_end(&mut self, _id: u64, _way: Way) -> DropEnd {
        DropEnd::Done
    }
}

const WHEEL_LINES: f32 = 3.0;
const TOUCHPAD_LINE: f32 = 24.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DropEnd {
    Done,
    Failed,
    Later,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Lifted {
    pub files: Vec<std::path::PathBuf>,
    pub typed: Vec<(String, Vec<u8>)>,
    pub right: bool,
    pub label: String,
    pub icon: Option<String>,
    pub folder: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Keyboard {
    pub keys: bool,
    pub text: bool,
    pub arrows: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum TipState {
    Waiting(Instant),
    Shown(Instant),
    Gone,
}

struct Rows {
    row: f32,
    first: usize,
    last: usize,
    hovered: Option<usize>,
    pad: f32,
    icon: f32,
    style: Style,
    dots: Rc<Layout>,
}

struct Tip {
    id: String,
    state: TipState,
}

pub struct Reply<M> {
    pub messages: Vec<M>,
    pub redraw: bool,
    pub copy: Option<String>,
    pub paste: bool,
    pub hold: Option<u64>,
    pub wanted: Option<Wanted>,
    pub dropped: Option<(u64, Way, bool)>,
    pub lift: Option<String>,
    pub(crate) patch: bool,
}

impl<M> Reply<M> {
    fn new() -> Reply<M> {
        Reply { messages: Vec::new(), redraw: false, copy: None, paste: false, hold: None, wanted: None, dropped: None, lift: None, patch: false }
    }
}

pub struct Ui<M> {
    fonts: Fonts,
    icons: Icons,
    texts: Texts,
    pub theme: Theme,
    tree: Node<M>,
    spots: Vec<Spot>,
    laid: Option<(u32, u32, f32, u32)>,
    size: (u32, u32),
    scale: f32,
    focus: Option<String>,
    hover: Option<String>,
    hovered: Option<String>,
    pressed: Option<String>,
    dragging: Option<String>,
    keyboard_seen: bool,
    tip: Option<Tip>,
    tip_part: Option<usize>,
    cuts: RefCell<HashMap<(String, Option<usize>), String>>,
    said: RefCell<HashMap<String, Vec<(Rect, Rc<str>)>>>,
    pointer: Option<(f32, f32)>,
    touch: Option<i32>,
    hold: Option<(f32, f32, u64)>,
    touches: u64,
    edits: HashMap<String, Edit>,
    last_click: Option<(Instant, String, u32)>,
    edge: Option<Instant>,
    last_down: Option<(Instant, String, (f32, f32))>,
    picked: Option<(String, Edit)>,
    right: Option<(String, (f32, f32), bool)>,
    copy_menu: Option<CopyMenu<M>>,
    drop: Option<DropOpen>,
    keys_here: bool,
    scrolls: HashMap<String, f32>,
    areas: Vec<String>,
    held: Kept,
    puzzles: Vec<String>,
    stays: w::Stays,
    chose: Option<(String, usize)>,
    bar_hover: Option<String>,
    thumb: Option<(String, f32)>,
    swipe: Option<(String, f32, f32, bool)>,
    fling: fling::Fling,
    swipe_x: Option<(String, (f32, f32), f32, bool)>,
    undo: Option<(M, M)>,
    last_row: Option<(Instant, String, usize)>,
    along: Option<(String, usize)>,
    gap: Option<(String, usize)>,
    splits: split::Splits,
    row_drag: Option<(String, usize, f32, bool)>,
    band: Option<band::Band>,
    mods: Mods,
    anchor: Option<(String, usize)>,
    ahead: Option<(String, String, Instant)>,
    preedit: Option<(String, String, Option<(usize, usize)>)>,
    pub see_through: bool,
    touched: RefCell<Vec<Rect>>,
    out_labels: bool,
    sent_out: RefCell<Option<Outside>>,
    part: Cell<bool>,
    pixels_at: Cell<Option<tiny_skia::IntRect>>,
    marking: Cell<bool>,
    under: RefCell<Vec<(usize, Option<Colour>)>>,
    pub shown: f32,
    blend: Option<(u64, f32)>,
    before: Option<(u64, Rc<Pixmap>)>,
    first_given: bool,
    grabbed: u32,
    sought: u32,
    nudged: u32,
    last_pressed: Cell<Option<Rect>>,
    pub(crate) news: String,
    #[cfg_attr(not(feature = "testing"), allow(dead_code))]
    audited: bool,
    #[cfg_attr(not(feature = "testing"), allow(dead_code))]
    audit_off: bool,
    shadows: Shadows,
    glows: HashMap<String, (u32, Option<Instant>)>,
    born: Instant,
    moving: bool,
    labels: Labels,
    refused: Option<(Option<String>, Instant)>,
    refused_words: &'static str,
    armed: Option<(String, Instant)>,
    eyes: eye::Eyes,
    outside: Option<super::dnd::Offered>,
    held_kinds: Vec<String>,
    lift: Option<outside::Lift>,
    bell: Option<Instant>,
    belled: Option<Instant>,
    frame: Cell<Instant>,
    dev: inspect::Dev,
}

impl<M: Clone> Ui<M> {
    pub fn new(tree: Node<M>, theme: Theme) -> Ui<M> {
        let mut ui = Ui {
            fonts: Fonts::new(), icons: Icons::new(), texts: Texts::default(), theme, tree, spots: Vec::new(),
            laid: None, size: (0, 0), scale: 1.0, focus: None, hover: None, hovered: None, pressed: None, dragging: None,
            keyboard_seen: false, tip: None, tip_part: None, cuts: RefCell::new(HashMap::new()), said: RefCell::new(HashMap::new()), pointer: None, touch: None, hold: None, touches: 0, edits: HashMap::new(), last_click: None, edge: None, last_down: None, picked: None, right: None, copy_menu: None, drop: None,
            keys_here: true, scrolls: HashMap::new(), areas: Vec::new(), held: Kept::default(), puzzles: Vec::new(), stays: w::Stays::default(), chose: None, bar_hover: None, thumb: None, swipe: None, band: None, fling: fling::Fling::default(), swipe_x: None, undo: None, last_row: None, along: None, gap: None, splits: split::Splits::default(), row_drag: None, mods: Mods::default(), anchor: None, ahead: None,
            preedit: None, see_through: false, out_labels: false, sent_out: RefCell::new(None), part: Cell::new(false), pixels_at: Cell::new(None), marking: Cell::new(false), touched: RefCell::new(Vec::new()), under: RefCell::new(Vec::new()), shown: 1.0, blend: None, before: None, first_given: false, grabbed: 0, sought: 0, nudged: 0,
            last_pressed: Cell::new(None), news: String::new(), audited: false, audit_off: false, shadows: Shadows::default(),
            glows: HashMap::new(), born: Instant::now(), moving: crate::host::host().moving(),
            labels: Labels::Brief, refused: None, refused_words: "", armed: None, eyes: eye::Eyes::default(), outside: None, held_kinds: Vec::new(), lift: None, bell: None, belled: None, frame: Cell::new(Instant::now()),
            dev: inspect::Dev::default(),
        };
        ui.follow_armed();
        ui
    }

    pub fn set_labels(&mut self, labels: Labels) {
        if self.labels != labels {
            self.labels = labels;
            self.laid = None;
            self.tip = None;
        }
    }

    pub fn natural_height(&self, width: f32) -> f32 {
        let lay = Lay { fonts: &self.fonts, texts: &self.texts, scale: 1.0, scrolled: &self.scrolls, splits: Some(&self.splits.places), natural: Default::default(), bento: Default::default(),
                        named: self.labels == Labels::Always, whole: Default::default(), puzzles: puzzle_names(&self.tree), held: Some(&self.held), stays: None };
        lay.whole.set(true);
        lay.measure(&self.tree, width).1
    }

    pub fn set_view(&mut self, tree: Node<M>) {
        let chosen = |ui: &Self| -> Vec<(String, Option<usize>)> {
            ui.spots.iter().filter_map(|s| match ui.tree.at(&s.path).map(|n| &n.what) {
                Some(What::List { chosen, .. }) => Some((s.id.clone(), *chosen)),
                _ => None,
            }).collect()
        };
        let before = chosen(self);
        self.eyes_before_view();
        self.tree = tree;
        self.audited = false;
        self.follow_armed();
        self.laid = None;
        self.start_glows(Instant::now());
        if let Some((turn, top, tall, path)) = self.tree.newest_seek(&mut Vec::new()).filter(|s| s.0 > self.sought) {
            self.sought = turn;
            self.settle();
            if let Some(id) = self.spots.iter().find(|s| s.path == path).map(|s| s.id.clone()) {
                let part = (self.px(top), self.px(tall));
                self.reveal(&id, Some(part));
            }
        }
        let mut nudges = Vec::new();
        self.tree.nudges(self.nudged, &mut Vec::new(), &mut nudges);
        for (turn, by, path) in nudges {
            self.nudged = self.nudged.max(turn);
            self.settle();
            let id = self.spots.iter().find(|s| s.path == path).map(|s| s.id.clone());
            if let Some(area) = id.and_then(|id| self.areas_round(&id).into_iter().next()) {
                if let Some(s) = self.spot(&area) {
                    let to = self.offset(s) + self.px(by);
                    self.scroll_area(&area, to);
                }
            }
        }
        if before.is_empty() {
            return;
        }
        self.settle();
        let own = self.chose.take();
        for (id, now) in chosen(self) {
            if let (Some(i), Some((_, was))) = (now, before.iter().find(|(b, _)| *b == id)) {
                if *was != Some(i) && own.as_ref().is_none_or(|(list, row)| *list != id || *row != i) {
                    let row = self.row_of(&id);
                    self.reveal(&id, Some((i as f32 * row, row)));
                }
            }
        }
    }

    pub fn ring_bell(&mut self, now: Instant) -> bool {
        if self.belled.is_some_and(|at| now.saturating_duration_since(at) < BELL_GAP) {
            return false;
        }
        self.bell = Some(now);
        self.belled = Some(now);
        true
    }

    fn control_at(&self, path: &[usize]) -> Colour {
        let t = &self.theme;
        match self.card_depth(path) {
            0 => t.control,
            1 => t.bg,
            2 => t.panel,
            _ => t.raised,
        }
    }

    fn card_depth(&self, path: &[usize]) -> usize {
        let mut n = &self.tree;
        let mut depth = 0;
        for &i in path {
            if matches!(n.what, What::Card(..)) && !n.flat && !n.chip && !n.sunk && !n.side && !n.band {
                depth += 1;
            }
            let Some(next) = n.items().get(i) else { break };
            n = next;
        }
        depth
    }

    fn grounds(&self, path: &[usize]) -> [Colour; 2] {
        let t = &self.theme;
        match self.card_depth(path) {
            0 => [t.bg, theme::over(t.side_bar, t.bg)],
            depth => [t.card(depth - 1); 2],
        }
    }

    fn row_icon_ink(&self, item: &Item, ink: Colour, fill: Option<Colour>, path: &[usize]) -> Colour {
        if !item.folder {
            return item.mark.unwrap_or(ink);
        }
        let under = self.grounds(path).map(|g| fill.map_or(g, |f| theme::over(f, g)));
        self.theme.folder_ink(item.mark, &under)
    }

    pub fn tick(&mut self, now: Instant) -> bool {
        let moving = self.spinning() || self.glows.values().any(|(_, s)| s.is_some());
        for (_, started) in self.glows.values_mut() {
            if started.is_some_and(|s| now >= s + theme::sheet().glow) {
                *started = None;
            }
        }
        let disarmed = self.armed.as_ref().is_some_and(|(_, at)| now >= *at + theme::sheet().armed);
        if disarmed {
            self.armed = None;
            self.follow_armed();
        }
        let belled = self.bell.is_some();
        if self.bell.is_some_and(|s| now >= s + theme::sheet().bell) {
            self.bell = None;
        }
        let moving = moving | belled | disarmed | self.eyes_tick(now);
        let unsaid = self.refused.as_ref().is_some_and(|(_, at)| now >= *at + tip_stays());
        if unsaid {
            self.refused = None;
        }
        let moving = moving | unsaid | self.coast(now) | self.drag_scroll(now);
        let stays = self.labels != Labels::Brief || self.tells_whole();
        let Some(t) = &mut self.tip else { return moving };
        moving | match t.state {
            TipState::Waiting(since) if now >= since + tip_rest() => {
                t.state = TipState::Shown(now);
                true
            }
            TipState::Shown(since) if !stays && now >= since + tip_stays() => {
                t.state = TipState::Gone;
                true
            }
            _ => false,
        }
    }

    pub fn only_turns(&self) -> bool {
        self.spinning() && !self.glows.values().any(|(_, s)| s.is_some()) && self.bell.is_none() && !self.fling.coasting() && self.drag_edge().is_none()
    }

    pub fn next_wake(&self, now: Instant) -> Option<Duration> {
        let frame = (self.spinning() || self.fling.coasting() || self.drag_edge().is_some() || self.bell.is_some() || self.glows.values().any(|(_, s)| s.is_some())).then_some(if theme::drawn_by_processor() { FRAME * 2 } else { FRAME });
        let tip = match self.tip.as_ref().map(|t| t.state) {
            Some(TipState::Waiting(since)) => Some((since + tip_rest()).saturating_duration_since(now)),
            Some(TipState::Shown(since)) if self.labels == Labels::Brief && !self.tells_whole() => Some((since + tip_stays()).saturating_duration_since(now)),
            _ => None,
        };
        let armed = self.armed.as_ref().map(|(_, at)| (*at + theme::sheet().armed).saturating_duration_since(now));
        let refused = self.refused.as_ref().map(|(_, at)| (*at + tip_stays()).saturating_duration_since(now));
        [frame, tip, armed, refused, self.eyes_wake(now)].into_iter().flatten().min()
    }
}

pub fn tip_box(canvas: &mut PixmapMut, _theme: &Theme, fonts: &Fonts, scale: f32, layout: &Layout, r: Rect, ch: f32) -> Rect {
    label_pill(canvas, fonts, scale, layout, r, ch, theme::sheet().label_ink)
}

pub fn tip_box_on<'a>(p: &mut Painter<'a>, _theme: &Theme, fonts: &'a Fonts, scale: f32, layout: &Layout, r: Rect, room: (f32, f32)) -> Rect {
    label_pill_on(p, fonts, scale, layout, r, room, theme::sheet().label_ink)
}

pub fn label_pill(canvas: &mut PixmapMut, fonts: &Fonts, scale: f32, layout: &Layout, r: Rect, ch: f32, ink: Colour) -> Rect {
    let mut p = Painter::default();
    let b = label_pill_on(&mut p, fonts, scale, layout, r, (canvas.width() as f32, ch), ink);
    p.play(canvas);
    b
}

pub fn label_pill_on<'a>(p: &mut Painter<'a>, fonts: &'a Fonts, scale: f32, layout: &Layout, r: Rect, room: (f32, f32), ink: Colour) -> Rect {
    label_pill_with(p, fonts, scale, layout, r, room, ink, label_corner(layout, scale))
}

pub fn label_corner(layout: &Layout, scale: f32) -> f32 {
    let pad_y = (theme::sheet().label_padding.1 * scale).round();
    theme::corner_px(theme::Part::Tooltip, (layout.height / layout.lines.len().max(1) as f32).ceil() + 2.0 * pad_y, scale)
}

fn label_pill_with<'a>(p: &mut Painter<'a>, fonts: &'a Fonts, scale: f32, layout: &Layout, r: Rect, room: (f32, f32), ink: Colour, radius: f32) -> Rect {
    let b = label_rect(layout, r, room, scale);
    let k = theme::sheet();
    let pad_x = (k.label_padding.0 * scale).round();
    let pad_y = (k.label_padding.1 * scale).round();
    let left = layout.left();
    let left = if left.is_finite() { left } else { 0.0 };
    let radius = radius.min(b.h / 2.0);
    let fall = k.label_shadow;
    let steps = 5;
    for i in 0..steps {
        let grow = fall.blur * scale * (i as f32 + 0.5) / steps as f32 / 2.0;
        let s = Rect { y: b.y + fall.y * scale, ..b }.inset(-grow);
        fill_on(p, s, radius + grow, theme::alpha(Colour { a: 1.0, ..k.label_outline }, fall.alpha / steps as f32));
    }
    let line = scale.round().max(1.0);
    fill_on(p, b.inset(-line), radius + line, k.label_outline);
    fill_on(p, b, radius, k.label_back);
    edge_on(p, b, radius, k.label_edge, scale);
    fonts.paint(p, layout, b.x + pad_x - left, b.y + pad_y, ink);
    b
}

pub fn label_rect(layout: &Layout, r: Rect, (cw, ch): (f32, f32), scale: f32) -> Rect {
    let k = theme::sheet();
    let (pad_x, pad_y) = ((k.label_padding.0 * scale).round(), (k.label_padding.1 * scale).round());
    let words = layout.widest();
    let (w, h) = (words.ceil() + 2.0 * pad_x, layout.height.ceil() + 2.0 * pad_y);
    let gap = (k.label_gap * scale).round();
    let below = r.y + r.h + gap;
    let y = if below + h <= ch || r.y - gap - h < 0.0 { below } else { r.y - gap - h };
    let x = (r.x + (r.w - w) / 2.0).clamp(0.0, (cw - w).max(0.0)).round();
    Rect { x, y: y.round(), w, h }
}

pub fn label_layout(fonts: &Fonts, text: &str, scale: f32, cw: f32) -> Layout {
    let k = theme::sheet();
    let style = tip_style(scale);
    let room = (cw - 2.0 * (k.label_padding.0 * scale).round()).max(1.0).min(k.label_width_most * scale);
    let most = k.label_lines_most.max(2);
    let text = text.trim_end();
    let first = text.find(|c: char| !c.is_whitespace()).unwrap_or(0);
    let text = &text[text[..first].rfind('\n').map_or(0, |i| i + 1)..];
    let head = label_head(text, most, most * (k.label_width_most / 2.0) as usize);
    let laid = fonts.layout(head, &style, Some(room));
    if head.len() == text.len() && laid.lines.len() <= most {
        return laid;
    }
    let end = if laid.lines.len() >= most { laid.lines[most - 2].shown_end.min(head.len()) } else { head.len() };
    let (shown, rest) = (head[..end].trim_end(), text[end..].trim_end());
    if rest.trim().is_empty() {
        return fonts.layout(shown, &style, Some(room));
    }
    let more = crate::common::ui::more_lines(rest.bytes().filter(|b| *b == b'\n').count());
    fonts.layout(&format!("{shown}\n{more}"), &style, Some(room))
}

fn label_head(text: &str, lines: usize, letters: usize) -> &str {
    let mut seen = 0;
    for (n, (i, c)) in text.char_indices().enumerate() {
        if n >= letters {
            return &text[..i];
        }
        if c == '\n' {
            seen += 1;
            if seen >= lines {
                return &text[..i];
            }
        }
    }
    text
}

pub fn tip_style(scale: f32) -> Style {
    w::name_style(scale)
}

fn notch(height: f32, scale: f32) -> f32 {
    (height / scale).powf(2.0 / 3.0)
}

fn toggled(picked: &[usize], row: usize) -> Vec<usize> {
    let mut p = picked.to_vec();
    match p.iter().position(|&r| r == row) {
        Some(at) => { p.remove(at); }
        None => p.push(row),
    }
    p.sort_unstable();
    p
}

fn run(from: usize, to: usize) -> Vec<usize> {
    (from.min(to)..=from.max(to)).collect()
}

fn held(mut mods: Mods, sym: Keysym, down: bool) -> Mods {
    match sym {
        Keysym::Control_L | Keysym::Control_R => mods.ctrl = down,
        Keysym::Shift_L | Keysym::Shift_R => mods.shift = down,
        _ => {}
    }
    mods
}

fn moved(s: &Spot, dx: f32, dy: f32) -> Spot {
    let by = |r: Rect| Rect { x: r.x + dx, y: r.y + dy, ..r };
    Spot { rect: by(s.rect), clip: s.clip.map(by), ..s.clone() }
}

fn snap(v: f32, min: f32, max: f32, step: f32) -> f32 {
    if step.is_nan() || step <= 0.0 {
        return v.clamp(min.min(max), max.max(min));
    }
    let snapped = min + ((v - min) / step).round() * step;
    let tidy = (snapped / step).round() * step;
    let v = if (tidy - snapped).abs() < step * 1e-3 { tidy } else { snapped };
    let v = (v * 1e5).round() / 1e5;
    v.clamp(min.min(max), max.max(min))
}

pub fn rounded(r: Rect, radius: f32) -> Option<Path> {
    let k = radius.min(r.w / 2.0).min(r.h / 2.0).max(0.0);
    let c = k * (1.0 - 0.552_284_8);
    let (x0, y0, x1, y1) = (r.x, r.y, r.x + r.w, r.y + r.h);
    let mut p = PathBuilder::new();
    p.move_to(x0 + k, y0);
    p.line_to(x1 - k, y0);
    p.cubic_to(x1 - c, y0, x1, y0 + c, x1, y0 + k);
    p.line_to(x1, y1 - k);
    p.cubic_to(x1, y1 - c, x1 - c, y1, x1 - k, y1);
    p.line_to(x0 + k, y1);
    p.cubic_to(x0 + c, y1, x0, y1 - c, x0, y1 - k);
    p.line_to(x0, y0 + k);
    p.cubic_to(x0, y0 + c, x0 + c, y0, x0 + k, y0);
    p.close();
    p.finish()
}

fn with_preedit(text: &str, at: usize, word: &str) -> String {
    let at = at.min(text.len());
    [&text[..at], word, &text[at..]].concat()
}

pub fn toned(t: &Theme, tone: Tone, plain: Colour) -> Colour {
    match tone {
        Tone::Problem => t.danger_text,
        _ => plain,
    }
}

pub fn state(t: &Theme, tone: Tone, plain: Colour) -> Colour {
    match tone {
        Tone::Plain => plain,
        Tone::Good => theme::good(),
        Tone::Warn => theme::warn(),
        Tone::Problem => t.danger,
        Tone::Calm => theme::calm(),
        Tone::Accent => t.soft,
    }
}

fn picked_bands(layout: &Layout, sel: Range<usize>) -> Vec<Rect> {
    if sel.is_empty() {
        return Vec::new();
    }
    layout.lines.iter().filter_map(|line| {
        let (a, b) = (sel.start.max(line.range.start), sel.end.min(line.range.end));
        if a >= b {
            return None;
        }
        let x0 = layout.caret(a).0;
        let x1 = if b >= line.range.end { line.x + line.width } else { layout.caret(b).0 };
        Some(Rect { x: x0.min(x1), y: line.top, w: (x1 - x0).abs().max(1.0), h: line.height })
    }).collect()
}

pub fn fill(canvas: &mut PixmapMut, r: Rect, radius: f32, c: Colour) {
    let mut p = Painter::default();
    fill_on(&mut p, r, radius, c);
    p.play(canvas);
}

pub fn fill_on(p: &mut Painter, r: Rect, radius: f32, c: Colour) {
    p.round(r, [radius; 4], c);
}

fn arc(cx: f32, cy: f32, radius: f32, share: f32) -> Option<Path> {
    let sweep = share.clamp(0.0, 1.0) * std::f32::consts::TAU;
    if sweep < 0.001 {
        return None;
    }
    let mut pb = PathBuilder::new();
    let top = -std::f32::consts::FRAC_PI_2;
    arc_to(&mut pb, (cx, cy), radius, top, top + sweep, true);
    pb.finish()
}

pub fn arc_to(pb: &mut PathBuilder, (cx, cy): (f32, f32), radius: f32, from: f32, to: f32, start: bool) {
    let sweep = to - from;
    let pieces = (sweep.abs() / std::f32::consts::FRAC_PI_2).ceil().max(1.0);
    let step = sweep / pieces;
    let k = 4.0 / 3.0 * (step / 4.0).tan() * radius;
    let at = |a: f32| (cx + radius * a.cos(), cy + radius * a.sin());
    let (x, y) = at(from);
    if start {
        pb.move_to(x, y);
    } else {
        pb.line_to(x, y);
    }
    let mut a = from;
    for _ in 0..pieces as usize {
        let b = a + step;
        let ((x0, y0), (x1, y1)) = (at(a), at(b));
        pb.cubic_to(x0 - k * a.sin(), y0 + k * a.cos(), x1 + k * b.sin(), y1 - k * b.cos(), x1, y1);
        a = b;
    }
}

fn edge(canvas: &mut PixmapMut, r: Rect, radius: f32, c: Colour, scale: f32) {
    let mut p = Painter::default();
    edge_on(&mut p, r, radius, c, scale);
    p.play(canvas);
}

fn edge_on(p: &mut Painter, r: Rect, radius: f32, c: Colour, scale: f32) {
    let width = scale.round().max(1.0);
    if let Some(path) = rounded(r.inset(width / 2.0), (radius - width / 2.0).max(0.0)) {
        p.stroke_path(path, Stroke { width, ..Stroke::default() }, c);
    }
}

pub(crate) fn knob(canvas: &mut PixmapMut, r: Rect, t: &Theme, scale: f32) {
    let k = theme::sheet();
    let radius = theme::corner_px(theme::Part::Knob, r.h, scale);
    fill(canvas, Rect { y: r.y + scale.round().max(1.0), ..r }, radius, k.knob_shadow);
    fill(canvas, r, radius, t.knob);
}

type Look = Rc<dyn Fn() -> Theme>;

struct Held<A: App> {
    app: A,
    ui: Ui<A::Msg>,
    look: Look,
    wake: Option<Instant>,
    link: Option<access::Link>,
    out: bool,
    outside: Option<OutsideCall>,
    told: Option<Outside>,
}

impl<A: App> Held<A> where A::Msg: Clone {
    fn tell(&mut self) {
        if let Some(link) = &mut self.link {
            link.tell(&mut self.ui);
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Outside {
    pub words: String,
    pub at: Rect,
    pub size: (f32, f32),
}

type OutsideCall = Rc<dyn Fn(Option<Outside>)>;

#[derive(Clone, Copy, Debug)]
pub struct Painted {
    pub began: Instant,
    pub took: Duration,
    pub size: (u32, u32),
    pub scale: f32,
    pub by_gpu: bool,
    pub by_service: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Drawer {
    Gpu,
    Service,
    Processor(Why),
}

pub use crate::paint::{drawn_by, DrawnBy, Why};

impl Drawer {
    pub fn line(&self, program: &str) -> String {
        match self {
            Drawer::Gpu => format!("trindle: {program} window drawn by gpu (Passed)"),
            Drawer::Service => format!("trindle: {program} window drawn by service (Passed)"),
            Drawer::Processor(why) => format!("trindle: {program} window drawn by processor ({why:?})"),
        }
    }
}

type PaintedCall = Rc<RefCell<Option<Box<dyn FnMut(&Painted)>>>>;

pub struct Shown<A: App> {
    pub window: Rc<Window>,
    held: Rc<RefCell<Held<A>>>,
    events: Loop,
    drawer: Drawer,
    painted: PaintedCall,
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod label_tests;

#[cfg(test)]
mod list_ops_tests;

#[cfg(test)]
mod float_ops_tests;
#[cfg(test)]
mod tip_ops_tests;

#[cfg(test)]
mod for_pairs {
    use super::*;

    #[test]
    fn selected_words_draw_only_pairs_the_table_holds() {
        use crate::theme::drawn;
        use crate::widget::{card, column, label, note, scroll, Length, What, Tone};
        use crate::edit::Edit;
        use crate::theme::Theme;
        let three = |at: &str| vec![label("Plain words to select").key(format!("{at}-words")), note("Quiet words to select").key(format!("{at}-quiet")),
                                    note("Red words to select").tone(Tone::Problem).key(format!("{at}-red"))];
        let view = || column::<()>(vec![
            column(three("page")),
            card(column(vec![column(three("card")), card(column(three("box")))])),
            scroll("s", column(three("scrolled"))).height(Length::Fixed(90.0)),
        ]).padding(10.0);
        let ids: Vec<String> = ["page", "card", "box", "scrolled"].iter().flat_map(|at| ["words", "quiet", "red"].map(|k| format!("{at}-{k}"))).collect();
        drawn::none(drawn::in_every_look("selected words", 12, || {
            for id in &ids {
                let mut ui: Ui<()> = Ui::new(view(), Theme::chosen());
                ui.size = (420, 420);
                ui.settle();
                let Some(What::Label { text, .. }) = ui.node(id).map(|n| &n.what) else { panic!("no label {id}") };
                let mut e = Edit::new(text);
                e.select(0..e.text.len());
                ui.picked = Some((id.clone(), e));
                drawn::page(id);
                let mut pix = tiny_skia::Pixmap::new(420, 420).unwrap();
                ui.paint(&mut pix.as_mut(), 1.0);
            }
        }));
    }
}
