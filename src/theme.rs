//! Trindle's look for the toolkit's widgets, every value read from the one style sheet and held within safe limits.

use std::cell::RefCell;
use std::sync::RwLock;
use std::time::{Duration, Instant};

use crate::common::sheet;
use crate::common::style::{self, Layers};

use super::colour::Colour;

#[cfg(any(test, feature = "testing"))]
pub mod drawn;
mod look;
mod pairs;
mod reader;
pub mod lint;
#[cfg(any(test, feature = "testing"))]
pub use drawn::laid_over;

#[cfg(not(any(test, feature = "testing")))]
pub struct Laid;

#[cfg(not(any(test, feature = "testing")))]
impl Drop for Laid {
    fn drop(&mut self) {}
}

#[cfg(not(any(test, feature = "testing")))]
#[inline(always)]
pub fn laid_over(_under: &tiny_skia::PixmapMut, _at: (i32, i32, u32, u32), _opacity: f32) -> Laid {
    Laid
}
#[cfg(any(test, feature = "testing"))]
pub use pairs::{painted, painted_in, Ink};
use reader::Reader;

#[derive(Clone, Debug, PartialEq)]
pub struct Bento {
    pub gap: f32,
    pub page_gap: f32,
    pub card: (u8, u8),
    pub figure: (u8, u8),
    pub picture: (u8, u8),
    pub words: f32,
    pub figure_words: f32,
    pub picture_least: f32,
}

pub fn bento() -> &'static Bento {
    &sheet().bento
}

#[derive(Clone, Debug, PartialEq)]
pub struct Welcome {
    pub width: f32,
    pub height: f32,
    pub least_width: f32,
    pub least_height: f32,
    pub header: f32,
    pub tile: f32,
    pub tile_least: f32,
    pub run: f32,
}

pub fn welcome() -> &'static Welcome {
    &sheet().welcome
}

#[derive(Clone, Debug, PartialEq)]
pub struct AppPage {
    pub narrow: f32,
    pub icon: f32,
    pub action: f32,
    pub picture_shape: f32,
    pub picture_least: f32,
    pub picture_most: f32,
    pub ring: f32,
    pub figure: f32,
    pub bar: f32,
    pub bar_name: f32,
    pub bar_count: f32,
}

pub fn app_page() -> &'static AppPage {
    &sheet().app_page
}

#[derive(Clone, Debug, PartialEq)]
pub struct AppsList {
    pub tile_least: f32,
    pub tile_icon: f32,
    pub row_icon: f32,
    pub card_least: f32,
    pub held: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FoldersRoom {
    pub row: f32,
    pub rich_row: f32,
    pub line: f32,
    pub side_row: f32,
    pub gap: f32,
    pub icon_room: (f32, f32),
}

pub fn compact_room() -> &'static FoldersRoom {
    &sheet().folders[0]
}

pub fn apps_list() -> &'static AppsList {
    &sheet().apps_list
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Gaps {
    pub tiny: f32,
    pub small: f32,
    pub medium: f32,
    pub roomy: f32,
    pub large: f32,
    pub huge: f32,
}

pub fn gaps() -> &'static Gaps {
    &sheet().gaps
}

pub const TOUCH_TARGET: f32 = 44.0;
pub const TARGET_LEAST: f32 = 24.0;
pub const ROW_WORDS: f32 = 160.0;
pub const SLIDER_MIN_WIDTH: f32 = 120.0;
pub const ENTRY_MIN_WIDTH: f32 = 160.0;
pub const SCROLLBAR_MIN: f32 = 32.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Palette {
    Red,
    Pink,
    Purple,
    DeepPurple,
    Indigo,
    Blue,
    LightBlue,
    Cyan,
    Teal,
    Green,
    LightGreen,
    Lime,
    Yellow,
    Amber,
    Orange,
    DeepOrange,
    Brown,
    Grey,
    BlueGrey,
}

impl Palette {
    pub const ALL: [Palette; 19] = [Palette::Red, Palette::Pink, Palette::Purple, Palette::DeepPurple, Palette::Indigo,
                                    Palette::Blue, Palette::LightBlue, Palette::Cyan, Palette::Teal, Palette::Green,
                                    Palette::LightGreen, Palette::Lime, Palette::Yellow, Palette::Amber, Palette::Orange,
                                    Palette::DeepOrange, Palette::Brown, Palette::Grey, Palette::BlueGrey];

    pub fn name(self) -> &'static str {
        match self {
            Palette::Red => "red",
            Palette::Pink => "pink",
            Palette::Purple => "purple",
            Palette::DeepPurple => "deep-purple",
            Palette::Indigo => "indigo",
            Palette::Blue => "blue",
            Palette::LightBlue => "light-blue",
            Palette::Cyan => "cyan",
            Palette::Teal => "teal",
            Palette::Green => "green",
            Palette::LightGreen => "light-green",
            Palette::Lime => "lime",
            Palette::Yellow => "yellow",
            Palette::Amber => "amber",
            Palette::Orange => "orange",
            Palette::DeepOrange => "deep-orange",
            Palette::Brown => "brown",
            Palette::Grey => "grey",
            Palette::BlueGrey => "blue-grey",
        }
    }

    pub fn swatch(self) -> Colour {
        sheet().palette[self as usize]
    }

    pub fn colour(self) -> Colour {
        match self {
            Palette::Red | Palette::Green | Palette::Grey | Palette::BlueGrey | Palette::Brown => sheet().palette[self as usize],
            Palette::Blue => pair()[0],
            _ => pair()[1],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Safe {
    pub states: [Colour; 4],
    pub series: [Colour; 6],
}

static SAFE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub fn colour_blind_safe(on: bool) {
    SAFE.store(on, std::sync::atomic::Ordering::Relaxed);
}

pub(crate) fn safe_now() -> Option<Safe> {
    SAFE.load(std::sync::atomic::Ordering::Relaxed).then(|| sheet().safe[usize::from(style::light())])
}

pub fn calm() -> Colour {
    if let Some(s) = safe_now() { return s.states[0] }
    if style::light() { sheet().calm_light } else { sheet().calm }
}
pub fn good() -> Colour {
    if let Some(s) = safe_now() { return s.states[1] }
    if style::light() { sheet().good_light } else { sheet().good }
}
pub fn warn() -> Colour {
    if let Some(s) = safe_now() { return s.states[2] }
    if style::light() { sheet().warn_light } else { sheet().warn }
}
pub fn problem() -> Colour {
    safe_now().map_or(sheet().problem, |s| s.states[3])
}
pub fn quiet() -> Colour {
    sheet().quiet
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Fall {
    pub y: f32,
    pub blur: f32,
    pub spread: f32,
    pub alpha: f32,
}

const fn fall(y: f32, blur: f32, spread: f32, alpha: f32) -> Fall {
    Fall { y, blur, spread, alpha }
}

pub const MOST_FALLS: usize = style::MOST_LAYERS;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Falls {
    each: [Fall; MOST_FALLS],
    count: usize,
}

impl std::ops::Deref for Falls {
    type Target = [Fall];
    fn deref(&self) -> &[Fall] {
        &self.each[..self.count]
    }
}

impl Falls {
    pub fn parse(text: &str) -> Option<Falls> {
        let layers = style::shadow_layers(text, |c| Colour::parse(c).map(|c| c.a))?;
        let mut out = Falls { each: [fall(0.0, 0.0, 0.0, 0.0); MOST_FALLS], count: 0 };
        for [y, blur, spread, alpha] in layers {
            out.each[out.count] = fall(y, blur, spread, alpha);
            out.count += 1;
        }
        Some(out)
    }
}

const WHITE: Colour = Colour::rgb(1.0, 1.0, 1.0);
const BLACK: Colour = Colour::rgb(0.0, 0.0, 0.0);
const UNREAD: Colour = Colour::rgb(1.0, 0.0, 1.0);

fn default_accent() -> Colour {
    Colour::parse(style::DEFAULT_ACCENT).unwrap_or(UNREAD)
}

pub const HIGH_WORDS: f32 = 7.0;
pub const HIGH_EDGES: f32 = 4.5;

const TERM: [&str; 16] = ["trindle-term-black", "trindle-term-red", "trindle-term-green", "trindle-term-yellow", "trindle-term-blue", "trindle-term-magenta",
                          "trindle-term-cyan", "trindle-term-white", "trindle-term-bright-black", "trindle-term-bright-red", "trindle-term-bright-green",
                          "trindle-term-bright-yellow", "trindle-term-bright-blue", "trindle-term-bright-magenta", "trindle-term-bright-cyan", "trindle-term-bright-white"];

#[derive(Clone, Debug, PartialEq)]
pub struct Sheet {
    pub button: f32,
    pub icon: f32,
    pub ring_button: f32,
    pub ring_app_edge: f32,
    pub ring_button_gap: f32,
    pub ring_band_room: f32,
    pub ring_band_gap: f32,
    pub ring_job_programs: f32,
    pub ring_job_computer: f32,
    pub ring_job_tools: f32,
    pub ring_job_tray: f32,
    pub ring_windows_width: f32,
    pub ring_windows_width_least: f32,
    pub ring_windows_gap: f32,
    pub ring_windows_part_gap: f32,
    pub ring_windows_icon_alone: f32,
    pub ring_windows_track: f32,
    pub ring_edge_width: f32,
    pub ring_edge_strength: f32,
    pub ring_windows_tab_firm: f32,
    pub ring_windows_tab_front: f32,
    pub ring_windows_tab_pad: f32,
    pub ring_windows_name_width: f32,
    pub ring_windows_title: f32,
    pub ring_windows_title_gap: f32,
    pub ring_windows_dot: f32,
    pub ring_windows_picture: f32,
    pub ring_windows_picture_margin: f32,
    pub ring_windows_picture_corner: f32,
    pub ring_windows_picture_icon: f32,
    pub label_back: Colour,
    pub label_ink: Colour,
    pub label_edge: Colour,
    pub label_outline: Colour,
    pub label_shadow: Fall,
    pub label_text: f32,
    pub label_weight: u16,
    pub label_padding: (f32, f32),
    pub label_gap: f32,
    pub label_width_most: f32,
    pub label_lines_most: usize,
    pub label_rest: Duration,
    pub label_stays: Duration,
    pub slide_time: Duration,
    pub fade_time: Duration,
    pub reduce_motion: bool,
    pub notice_stays: Duration,
    pub hover_edge: f32,
    pub armed: Duration,
    pub work_done: Duration,
    pub work_heavy: (f32, Duration),
    pub bento: Bento,
    pub welcome: Welcome,
    pub app_page: AppPage,
    pub apps_list: AppsList,
    pub folders: [FoldersRoom; 3],
    pub palette: [Colour; 19],
    pub term: [Colour; 16],
    pub pairs: [[Colour; 2]; 19],
    pub info: f32,
    pub calm: Colour,
    pub good: Colour,
    pub warn: Colour,
    pub calm_light: Colour,
    pub good_light: Colour,
    pub warn_light: Colour,
    pub problem: Colour,
    pub quiet: Colour,
    pub colour_blind_safe: bool,
    pub safe: [Safe; 2],
    pub code: [Colour; 5],
    pub corner: f32,
    pub corners: [f32; Part::ALL.len()],
    pub panel_gap: f32,
    pub page_padding: f32,
    pub gap: f32,
    pub card_padding: f32,
    pub snug_padding: f32,
    pub sidebar_room: f32,
    pub line_padding: f32,
    pub chip_padding: (f32, f32),
    pub tabs_padding: f32,
    pub focus_width: f32,
    pub focus_offset: f32,
    pub switch: (f32, f32),
    pub knob: f32,
    pub knob_colour: Colour,
    pub knob_shadow: Colour,
    pub knob_off: Colour,
    pub track: f32,
    pub entry_height: f32,
    pub entry_padding: f32,
    pub entry_eye: f32,
    pub caret: f32,
    pub selection: f32,
    pub mark: f32,
    pub row: f32,
    pub rich_row: f32,
    pub rich_gap: f32,
    pub rich_icon: f32,
    pub rich_weight: u16,
    pub tree_indent: f32,
    pub title_bar_room: f32,
    pub frame_edge: f32,
    pub frame_shadow: f32,
    pub divider: f32,
    pub divider_lit: f32,
    pub scrollbar: f32,
    pub scrollbar_hover: f32,
    pub scrollbar_rest: f32,
    pub scrollbar_lit: f32,
    pub scrollbar_inset: f32,
    pub spinner: f32,
    pub spinner_line: f32,
    pub preview_panel: f32,
    pub preview_picture: f32,
    pub sidebar_width: f32,
    pub split_narrow: f32,
    pub drawer_width: f32,
    pub drawer_room: f32,
    pub icon_tile: f32,
    pub icon_tile_icon: f32,
    pub status_icon: f32,
    pub dot: f32,
    pub dot_target: f32,
    pub fling_friction: f32,
    pub drive_card: f32,
    pub fill_bar: f32,
    pub chart_line: f32,
    pub radio: f32,
    pub tick: f32,
    pub gaps: Gaps,
    pub colour_square: (f32, f32),
    pub colour_swatch: f32,
    pub colour_swatch_corner: f32,
    pub colour_code: f32,
    pub gradient_bar: f32,
    pub volume: f32,
    pub thumb: f32,
    pub lane: (f32, f32, f32),
    pub table: (f32, f32),
    pub info_icon: f32,
    pub bars: f32,
    pub bars_width: f32,
    pub bars_least: f32,
    pub bars_gap: f32,
    pub bars_corner: f32,
    pub skeleton_line: f32,
    pub skeleton_gap: f32,
    pub skeleton_last: f32,
    pub hidden_drive: f32,
    pub glow: Duration,
    pub glow_width: f32,
    pub bell: Duration,
    pub glow_strength: f32,
    pub lit_width: f32,
    pub working_line: f32,
    pub action_height: f32,
    pub action_padding: f32,
    pub action_gap: f32,
    pub action_text: f32,
    pub action_weight: u16,
    pub word_button: (f32, f32),
    pub word_button_padding: f32,
    pub word_button_text: f32,
    pub page_title: (f32, u16),
    pub section: (f32, u16),
    pub lead: (f32, u16),
    pub heading: (f32, u16),
    pub headline: (f32, u16),
    pub figure: (f32, u16),
    pub big_figure: (f32, u16),
    pub text: (f32, u16),
    pub form_label: (f32, u16),
    pub note: (f32, u16),
    pub column_heading: (f32, u16),
    pub side_heading: (f32, u16),
    pub tighten: f32,
    pub menu_width: f32,
    pub menu_row: f32,
    pub menu_padding: f32,
    pub menu_icon: f32,
    pub menu_sub_pause: Duration,
    pub menu_sub_overlap: f32,
    pub menu_sub_arrow: f32,
    pub menu_title_pad: f32,
    pub menu_group_gap: f32,
    pub menu_edge: f32,
    pub menu_row_least: f32,
    pub tab_height: f32,
    pub tab_least: f32,
    pub tab_most: f32,
    pub tab_close: f32,
    pub tab_more: f32,
    pub tab_pad: f32,
    pub tab_slop: f32,
    pub tab_tear: f32,
    pub tab_count: f32,
    pub pill_height: f32,
    pub pill_side: f32,
    pub pill_widest: f32,
    pub pill_room: f32,
    pub pill_foot: f32,
    pub pill_words: f32,
    pub dropdown_height: f32,
    pub dropdown_row: f32,
    pub dropdown_rows: f32,
    pub dropdown_width: f32,
    pub state_dot: f32,
    pub badge: f32,
    pub badge_icon: f32,
    pub area_line: f32,
    pub section_line_width: f32,
    pub trial_foot: f32,
    pub trial_foot_padding: f32,
    pub current_edge: f32,
    pub pane_strip: f32,
    pub note_title_weight: u16,
    pub note_slide: f32,
    pub circle: f32,
    pub circle_rest_least: f32,
    pub circle_gap: f32,
    pub circle_margin: f32,
    pub circle_button: f32,
    pub circle_most: f32,
    pub circle_pointed_most: f32,
    pub circle_hold: f32,
    pub circle_step: f32,
}

#[cfg(any(test, feature = "testing"))]
thread_local! {
    pub static WASH_AS_WAS: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static READ: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}

static SHEET: RwLock<Option<&'static Sheet>> = RwLock::new(None);

pub const FULL_ROUND: f32 = 999.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Part {
    Card, Box, Row, Menu, Button, IconButton, RingButton, Switch, Knob, Slider, Tab, Badge, Chip, Segmented, Tooltip, Scrollbar, Picture, Radio, Tick,
}

impl Part {
    pub const ALL: [Part; 19] = [
        Part::Card, Part::Box, Part::Row, Part::Menu, Part::Button, Part::IconButton, Part::RingButton, Part::Switch, Part::Knob, Part::Slider,
        Part::Tab, Part::Badge, Part::Chip, Part::Segmented, Part::Tooltip, Part::Scrollbar, Part::Picture, Part::Radio, Part::Tick,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Part::Card => "trindle-card-corner",
            Part::Box => "trindle-box-corner",
            Part::Row => "trindle-row-corner",
            Part::Menu => "trindle-float-corner",
            Part::Button => "trindle-button-corner",
            Part::IconButton => "trindle-icon-button-corner",
            Part::RingButton => "trindle-ring-corner",
            Part::Switch => "trindle-switch-corner",
            Part::Knob => "trindle-knob-corner",
            Part::Slider => "trindle-slider-corner",
            Part::Tab => "trindle-tab-corner",
            Part::Badge => "trindle-badge-corner",
            Part::Chip => "trindle-chip-corner",
            Part::Segmented => "trindle-segmented-corner",
            Part::Tooltip => "trindle-tooltip-corner",
            Part::Scrollbar => "trindle-scrollbar-corner",
            Part::Picture => "trindle-picture-corner",
            Part::Radio => "trindle-radio-corner",
            Part::Tick => "trindle-tick-corner",
        }
    }
}

impl Sheet {
    pub fn corner_of(&self, part: Part) -> f32 {
        self.corners[part as usize]
    }
}

pub fn corner_px(part: Part, shorter: f32, scale: f32) -> f32 {
    (sheet().corner_of(part) * scale).min(shorter / 2.0).max(0.0)
}

pub fn sheet() -> &'static Sheet {
    if let Some(s) = *SHEET.read().unwrap_or_else(|e| e.into_inner()) {
        return s;
    }
    let mut held = SHEET.write().unwrap_or_else(|e| e.into_inner());
    held.get_or_insert_with(|| Box::leak(Box::new(Sheet::from(&style::layers()))))
}

static REREADING: std::sync::Mutex<()> = std::sync::Mutex::new(());
static READING: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

static LOOK: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0);

pub fn reread() {
    let _one = REREADING.lock().unwrap_or_else(|e| e.into_inner());
    let sheets = style::reread();
    let look = 1 + u8::from(style::light());
    let before = LOOK.swap(look, std::sync::atomic::Ordering::Relaxed);
    if !sheets {
        if before != 0 && before != look {
            READING.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
        return;
    }
    let fresh = Sheet::from(&style::layers());
    let mut held = SHEET.write().unwrap_or_else(|e| e.into_inner());
    if held.is_none_or(|s| *s != fresh) {
        *held = Some(Box::leak(Box::new(fresh)));
    }
    READING.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
}

pub fn reading() -> u64 {
    READING.load(std::sync::atomic::Ordering::Relaxed)
}

pub fn icon_button() -> f32 {
    sheet().button
}

pub fn icon() -> f32 {
    sheet().icon
}

#[derive(Clone, Debug, PartialEq)]
pub struct Theme {
    pub light: bool,
    pub bg: Colour,
    pub panel: Colour,
    pub ring: Colour,
    pub ring_edge: Colour,
    pub raised: Colour,
    pub raised_more: Colour,
    pub hover: Colour,
    pub control: Colour,
    pub control_edge: Colour,
    pub field: Colour,
    pub field_edge: Colour,
    pub well: Colour,
    pub fg: Colour,
    pub muted: Colour,
    pub accent: Colour,
    pub lit_edge: Colour,
    pub on_accent: Colour,
    pub accent_text: Colour,
    pub picked: Colour,
    pub on_picked: Colour,
    pub folder: Colour,
    pub soft: Colour,
    pub icon: Colour,
    pub danger: Colour,
    pub danger_text: Colour,
    pub edge: Colour,
    pub card_edge: Colour,
    pub corrected: bool,
    pub band: Colour,
    pub side_line: Colour,
    pub section_line: Colour,
    pub top_bar: Colour,
    pub side_bar: Colour,
    pub foot: Colour,
    pub area_line: Colour,
    pub trial_foot: Colour,
    pub trial_foot_line: Colour,
    pub current_edge: Colour,
    pub accent_hover: Colour,
    pub knob: Colour,
    pub knob_off: Colour,
    pub spelling: Colour,
    pub shadow: Colour,
    pub float: Falls,
    pub panel_falls: Falls,
    pub raised_falls: Falls,
    pub field_inset: Option<Fall>,
}

pub const WORDS: f32 = 4.5;
pub const EDGES: f32 = 3.0;

pub const SPARE: f32 = 0.05;
pub const AIM_WORDS: f32 = WORDS + SPARE;
pub const AIM_EDGES: f32 = EDGES + SPARE;

pub fn over(top: Colour, under: Colour) -> Colour {
    if top.a >= 1.0 {
        return top;
    }
    Colour { a: 1.0, ..mix(Colour { a: 1.0, ..under }, Colour { a: 1.0, ..top }, top.a.clamp(0.0, 1.0)) }
}

pub fn clear(c: Colour, on: &[Colour], at: f32) -> Colour {
    let least = |c: Colour| on.iter().map(|o| contrast(c, *o)).fold(f32::INFINITY, f32::min);
    if least(c) >= at {
        return c;
    }
    let black = Colour::rgb(0.0, 0.0, 0.0);
    let to = if least(WHITE) >= least(black) { WHITE } else { black };
    let to = Colour { a: c.a, ..to };
    (1..=20).map(|i| mix(c, to, i as f32 / 20.0)).find(|m| least(*m) >= at).unwrap_or(to)
}

fn builtin() -> &'static Layers {
    static OWN: std::sync::OnceLock<Layers> = std::sync::OnceLock::new();
    OWN.get_or_init(|| Layers::new(style::CSS, ""))
}

const DARK_BELOW: f32 = 0.17;
const LIGHT_ABOVE: f32 = 0.19;

fn sided(c: Colour, dark: bool) -> Colour {
    let fits = |c: &Colour| if dark { luminance(*c) <= DARK_BELOW } else { luminance(*c) >= LIGHT_ABOVE };
    let to = if dark { BLACK } else { WHITE };
    (0..=20).map(|i| mix(c, to, i as f32 / 20.0)).find(fits).unwrap_or(to)
}

fn kin(c: Colour, like: Colour) -> Colour {
    let ((h, _, s), (want, _, _)) = (to_hls(c), to_hls(like));
    let apart = ((h - want + 540.0).rem_euclid(360.0) - 180.0).abs();
    if apart <= 40.0 && s >= 0.3 && c.a >= 1.0 { c } else { like }
}

pub fn safe() -> Theme {
    Theme::from(builtin(), false, style::DEFAULT_ACCENT)
}

pub fn held() -> Theme {
    #[cfg(any(test, feature = "testing"))]
    if let Some((light, accent)) = style::asked_for_test() {
        return Theme::from(builtin(), light, &accent);
    }
    Theme::from(builtin(), crate::common::sheet::asked_light(), &style::chosen_accent())
}

pub fn safe_sheet() -> &'static Sheet {
    static OWN: std::sync::OnceLock<Sheet> = std::sync::OnceLock::new();
    OWN.get_or_init(|| Sheet::from(builtin()))
}

#[derive(Clone, Debug, PartialEq)]
pub struct Sample {
    pub theme: Theme,
    pub card_corner: f32,
    pub box_corner: f32,
    pub both: bool,
}

pub fn sample(css: &str, light: bool, accent: &str) -> Sample {
    let layers = Layers::themed(style::CSS, css, "");
    let sheet = Sheet::from(&layers);
    Sample { theme: Theme::from(&layers, light, accent), card_corner: sheet.corner_of(Part::Card), box_corner: sheet.corner_of(Part::Box),
             both: layers.only.is_none() }
}

pub fn own_colours_fall_short(layers: &Layers, accent: &str) -> bool {
    if layers.own.is_empty() {
        return false;
    }
    [false, true].into_iter().any(|light| {
        pairs::painted(layers, light, accent).iter().any(|p| p.short() && pairs::theirs(layers, light, p.ink))
    })
}

#[derive(Clone, Debug, PartialEq)]
pub struct Fault {
    pub line: usize,
    pub why: String,
}

impl Fault {
    pub fn words(&self) -> String {
        if self.line == 0 { self.why.clone() } else { crate::t!("kit-theme-line", line = self.line, why = &self.why) }
    }
}

pub const ABOUT: [&str; 5] = ["trindle-theme-name", "trindle-theme-about", "trindle-theme-author", "trindle-theme-version",
                              "trindle-theme-look"];

pub fn check(css: &str) -> Result<(), Fault> {
    let fault = |line: usize, why: &str| Err(Fault { line, why: why.to_string() });
    if css.len() as u64 > sheet::THEME_MOST {
        return fault(0, crate::t!("kit-theme-too-big"));
    }
    if css.chars().any(|c| c.is_control() && !matches!(c, '\n' | '\r' | '\t')) {
        return fault(0, crate::t!("kit-theme-not-sheet"));
    }
    let mut bare = String::with_capacity(css.len());
    let mut rest = css;
    while let Some(at) = rest.find("/*") {
        bare.push_str(&rest[..at]);
        let after = &rest[at..];
        let end = after[2..].find("*/").map_or(after.len(), |e| e + 4);
        bare.extend(after[..end].chars().map(|c| if c == '\n' { '\n' } else { ' ' }));
        rest = &after[end..];
    }
    bare.push_str(rest);
    for (i, line) in bare.lines().enumerate() {
        let mut l = line.to_ascii_lowercase();
        l.retain(|c| !c.is_whitespace());
        if l.contains("@import") || l.contains("url(") || l.contains("://") || l.contains("@font-face") || l.contains('\\') {
            return fault(i + 1, crate::t!("kit-theme-imports"));
        }
    }
    let default = builtin();
    let layers = Layers::themed(style::CSS, css, "");
    let accent = Colour::parse(style::DEFAULT_ACCENT).unwrap_or(UNREAD);
    let mut known = 0;
    for d in style::declarations(css) {
        let name = d.name.as_str();
        if ABOUT.contains(&name) || !default.default.has(name) {
            continue;
        }
        known += 1;
        if d.value.len() > 200 {
            return fault(d.line, &crate::t!("kit-theme-too-long", name = name));
        }
        let ours = Reader::new(default, d.light, accent);
        let theirs = Reader::new(&layers, d.light, accent);
        let own = default.default.get(name, d.light).unwrap_or("");
        let (is, reads) = if let Some(kind) = sheet::step_kind(name) {
            (crate::t!("kit-theme-not-step", name = name, steps = sheet::steps(kind), guide = sheet::GUIDE), css == style::CSS || sheet::is_step(kind, &d.value))
        } else if ours.colour(own, 0).is_some() {
            (crate::t!("kit-theme-not-colour", name = name), theirs.colour(&d.value, 0).is_some())
        } else if default.number(name, d.light, f32::MIN, f32::MAX).is_some() {
            let another = d.value.strip_prefix("var(--").and_then(|v| v.strip_suffix(')'));
            (crate::t!("kit-theme-not-number", name = name), another.map_or(sheet::number(&d.value).is_some(), |n| layers.number(n.trim(), d.light, f32::MIN, f32::MAX).is_some()))
        } else {
            (crate::t!("kit-theme-not-shadow", name = name), Falls::parse(&d.value).is_some())
        };
        if !reads {
            return fault(d.line, &is);
        }
    }
    if known == 0 {
        return fault(0, crate::t!("kit-theme-nothing-known"));
    }
    Ok(())
}

pub fn named(name: &str, light: bool) -> Colour {
    named_in(&style::layers(), name, light, &style::accent())
}

fn named_in(layers: &Layers, name: &str, light: bool, accent: &str) -> Colour {
    let t = Theme::from(layers, light, accent);
    match name {
        "trindle-page" => t.bg,
        "trindle-card" => t.panel,
        "trindle-box" => t.raised,
        "trindle-box-more" => t.raised_more,
        "trindle-typing-box" => t.field,
        "trindle-control" => t.control,
        "trindle-hover" => t.hover,
        "trindle-well" => t.well,
        "trindle-ring-edge" => t.ring_edge,
        "trindle-top-bar" => t.top_bar,
        "trindle-side-bar" => t.side_bar,
        "trindle-foot" => t.foot,
        "trindle-trial-foot" => t.trial_foot,
        "trindle-band" => t.band,
        "trindle-picked" => t.picked,
        "trindle-highlight-hover" => t.accent_hover,
        "trindle-text" => t.fg,
        "trindle-text-quiet" => t.muted,
        "trindle-shade-red" => t.danger_text,
        "trindle-bad" => t.danger,
        "trindle-highlight-text" => t.accent_text,
        "trindle-on-picked" => t.on_picked,
        "trindle-highlight" => t.accent,
        "trindle-typing-box-edge" => t.field_edge,
        "trindle-current-card-edge" => t.current_edge,
        "trindle-folder-icon" => t.folder,
        "trindle-highlight-soft" => t.soft,
        "trindle-icon" => t.icon,
        "trindle-knob" => t.knob,
        "trindle-knob-off" => t.knob_off,
        _ => Reader::new(layers, t.light, t.accent).c(name),
    }
}

pub const HIDDEN_LEAST: f32 = 0.65;

fn ring_edge_seen(mut c: Colour) -> Colour {
    let (white, black) = (Colour::rgb(1.0, 1.0, 1.0), Colour::rgb(0.0, 0.0, 0.0));
    for _ in 0..40 {
        if contrast(c, white) < AIM_EDGES {
            c = mix(c, black, 0.05);
        } else if contrast(c, black) < AIM_EDGES {
            c = mix(c, white, 0.05);
        } else {
            return c;
        }
    }
    Colour::rgb(0.5, 0.5, 0.5)
}

pub fn luminance(c: Colour) -> f32 {
    let lin = |v: f32| if v <= 0.039_28 { v / 12.92 } else { ((v + 0.055) / 1.055).powf(2.4) };
    0.2126 * lin(c.r) + 0.7152 * lin(c.g) + 0.0722 * lin(c.b)
}

pub const LAID: f32 = 2.0;

fn as_laid(under: &[Colour]) -> Vec<Colour> {
    let mixed = |c: &Colour| [c.r, c.g, c.b].iter().any(|v| (v * 255.0 - (v * 255.0).round()).abs() > 1e-3);
    under.iter().flat_map(|u| if mixed(u) { near(*u).to_vec() } else { vec![kept(*u)] }).collect()
}

fn near(c: Colour) -> [Colour; 3] {
    let by = |c: Colour, d: f32| Colour { r: (c.r + d).clamp(0.0, 1.0), g: (c.g + d).clamp(0.0, 1.0), b: (c.b + d).clamp(0.0, 1.0), a: c.a };
    let k = kept(c);
    [k, by(k, LAID / 255.0), by(k, -LAID / 255.0)]
}

pub fn kept(c: Colour) -> Colour {
    let k = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() / 255.0;
    Colour { r: k(c.r), g: k(c.g), b: k(c.b), a: c.a }
}

pub fn contrast(a: Colour, b: Colour) -> f32 {
    apart(kept(a), kept(b))
}

fn apart(a: Colour, b: Colour) -> f32 {
    let (x, y) = (luminance(a), luminance(b));
    (x.max(y) + 0.05) / (x.min(y) + 0.05)
}

fn dim_share(ink: Colour, share: f32, under: &[Colour], floor: f32) -> f32 {
    let fits = |s: f32| under.iter().all(|g| contrast(over(alpha(ink, s), kept(*g)), *g) >= floor);
    let share = share.clamp(0.0, 1.0);
    (0..=20).map(|i| share + (1.0 - share) * i as f32 / 20.0).find(|s| fits(*s)).unwrap_or(1.0)
}

pub fn dimmed(ink: Colour, share: f32, under: &[Colour], floor: f32) -> Colour {
    vouched(alpha(ink, dim_share(ink, share, &as_laid(under), floor)), under)
}

#[cfg_attr(not(test), inline(always))]
fn vouched(ink: Colour, under: &[Colour]) -> Colour {
    #[cfg(any(test, feature = "testing"))]
    drawn::vouch(ink, under);
    #[cfg(not(any(test, feature = "testing")))]
    let _ = under;
    ink
}

const NEAR_BLACK: Colour = Colour::rgb(0x11 as f32 / 255.0, 0x11 as f32 / 255.0, 0x11 as f32 / 255.0);

pub fn on(c: Colour) -> Colour {
    let c = Colour { a: 1.0, ..c };
    let best = if contrast(NEAR_BLACK, c) >= contrast(WHITE, c) { NEAR_BLACK } else { WHITE };
    vouched(if contrast(best, c) >= AIM_WORDS { best } else { BLACK }, &[c])
}

pub fn on_as_written(c: Colour) -> Colour {
    let c = Colour { a: 1.0, ..c };
    vouched(Colour::parse(style::on_colour(&c.css())).unwrap_or(BLACK), &[c])
}

fn usual(c: Colour) -> Colour {
    if luminance(c) > 0.36 { NEAR_BLACK } else { WHITE }
}

pub fn shade(c: Colour, by: f32) -> Colour {
    let (h, l, s) = to_hls(c);
    let (r, g, b) = from_hls(h, (l * by).clamp(0.0, 1.0), (s * by).clamp(0.0, 1.0));
    Colour { r, g, b, a: c.a }
}

pub fn alpha(c: Colour, by: f32) -> Colour {
    Colour { a: (c.a * by).clamp(0.0, 1.0), ..c }
}

pub fn mix(a: Colour, b: Colour, by: f32) -> Colour {
    let m = |x: f32, y: f32| x + (y - x) * by;
    Colour { r: m(a.r, b.r), g: m(a.g, b.g), b: m(a.b, b.b), a: m(a.a, b.a) }
}

fn to_hls(c: Colour) -> (f32, f32, f32) {
    let max = c.r.max(c.g).max(c.b);
    let min = c.r.min(c.g).min(c.b);
    let l = (max + min) / 2.0;
    if max == min {
        return (0.0, l, 0.0);
    }
    let d = max - min;
    let s = if l <= 0.5 { d / (max + min) } else { d / (2.0 - max - min) };
    let h = if max == c.r { (c.g - c.b) / d } else if max == c.g { 2.0 + (c.b - c.r) / d } else { 4.0 + (c.r - c.g) / d };
    ((h * 60.0).rem_euclid(360.0), l, s)
}

fn from_hls(h: f32, l: f32, s: f32) -> (f32, f32, f32) {
    if s == 0.0 {
        return (l, l, l);
    }
    let m2 = if l <= 0.5 { l * (1.0 + s) } else { l + s - l * s };
    let m1 = 2.0 * l - m2;
    let part = |h: f32| {
        let h = h.rem_euclid(360.0);
        if h < 60.0 { m1 + (m2 - m1) * h / 60.0 } else if h < 180.0 { m2 } else if h < 240.0 { m1 + (m2 - m1) * (240.0 - h) / 60.0 } else { m1 }
    };
    (part(h + 120.0), part(h), part(h - 120.0))
}

#[cfg(any(test, feature = "testing"))]
pub fn hostile_sheets_for_tests(count: usize) -> Vec<String> {
    tests::hostile_sheets(count, 0x51ab_90c3)
}

#[cfg(any(test, feature = "testing"))]
#[cfg_attr(not(test), allow(dead_code, unused_imports))]
pub mod tests;

pub fn pair() -> [Colour; 2] {
    thread_local! {
        static LAST: RefCell<Option<(Instant, [Colour; 2])>> = const { RefCell::new(None) };
    }
    LAST.with(|l| {
        #[cfg(any(test, feature = "testing"))]
        if style::asked_for_test().is_some() {
            *l.borrow_mut() = None;
        }
        if let Some((at, p)) = *l.borrow() {
            if at.elapsed() < Duration::from_millis(250) {
                return p;
            }
        }
        let accent = Colour::parse(&style::accent()).unwrap_or_else(default_accent);
        let p = pair_for(accent);
        *l.borrow_mut() = Some((Instant::now(), p));
        p
    })
}

pub fn pair_for(accent: Colour) -> [Colour; 2] {
    let s = sheet();
    let near = |c: &Colour| (c.r - accent.r).powi(2) + (c.g - accent.g).powi(2) + (c.b - accent.b).powi(2);
    let at = (0..Palette::ALL.len()).min_by(|&a, &b| near(&s.palette[a]).total_cmp(&near(&s.palette[b]))).unwrap_or_default();
    s.pairs[at]
}

pub fn drawn_by_processor() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("TRINDLE_DRAWN_BY_PROCESSOR").is_some_and(|v| v == "1"))
}
