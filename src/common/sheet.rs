//! The one style sheet (share/style.css), with a theme and the person's own laid over it name by name; shared with the compositor, so it needs nothing of the toolkit.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

pub const CSS: &str = include_str!("../../share/style.css");
const OWN_MOST: u64 = 256 * 1024;
pub const THEME_MOST: u64 = 64 * 1024;
pub const THEME_KEY: &str = "theme_name";
pub const TRIAL_KEY: &str = "theme_trial";
pub const TRIAL_FOR: u64 = 30;
pub const NAMING_KEY: &str = "theme_naming";
pub const NAMING: &str = "2";
const LOOK: &str = "trindle-theme-look";
pub const LOOK_KEY: &str = "theme";
pub const TITLE_CONTRAST: f32 = 4.5;
const OWN_ONLY: &[&str] = &["trindle-highlight"];

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Values {
    plain: HashMap<String, String>,
    light: HashMap<String, String>,
}

impl Values {
    pub fn parse(css: &str) -> Values {
        let mut out = Values::default();
        for d in declarations(css) {
            if !d.value.is_empty() {
                if d.light { &mut out.light } else { &mut out.plain }.insert(d.name, d.value);
            }
        }
        out
    }

    pub fn get(&self, name: &str, light: bool) -> Option<&str> {
        light.then(|| self.light.get(name)).flatten().or_else(|| self.plain.get(name)).map(String::as_str)
    }

    pub fn has(&self, name: &str) -> bool {
        self.plain.contains_key(name) || self.light.contains_key(name)
    }

    pub fn is_empty(&self) -> bool {
        self.plain.is_empty() && self.light.is_empty()
    }

    fn keep_known(mut self, known: &Values, own: bool) -> Values {
        let keep = |n: &String| if OWN_ONLY.contains(&n.as_str()) { own } else { known.has(n) };
        self.plain.retain(|n, _| keep(n));
        self.light.retain(|n, _| keep(n));
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Declared {
    pub line: usize,
    pub name: String,
    pub value: String,
    pub light: bool,
}

pub fn declarations(css: &str) -> Vec<Declared> {
    let mut out = Vec::new();
    let mut within: Vec<String> = Vec::new();
    let mut words = String::new();
    let (mut line, mut began) = (1, 1);
    let mut blank = true;
    let mut rest = css;
    while !rest.is_empty() {
        if let Some(after) = rest.strip_prefix("/*") {
            let end = after.find("*/").map_or(after.len(), |end| end + 2);
            line += after[..end].matches('\n').count();
            rest = &after[end..];
            continue;
        }
        let c = rest.chars().next().unwrap_or(' ');
        rest = &rest[c.len_utf8()..];
        match c {
            '{' => {
                within.push(std::mem::take(&mut words).trim().to_string());
                blank = true;
            }
            '}' | ';' => {
                emit(&within, &words, began, &mut out);
                words.clear();
                blank = true;
                if c == '}' {
                    within.pop();
                }
            }
            _ => {
                if c == '\n' {
                    line += 1;
                }
                if blank && !c.is_whitespace() {
                    began = line;
                    blank = false;
                }
                words.push(c);
            }
        }
    }
    emit(&within, &words, began, &mut out);
    out
}

fn light_media(s: &str) -> bool {
    let s = s.to_ascii_lowercase().replace(' ', "");
    s.starts_with("@media") && s.contains("prefers-color-scheme:light")
}

fn emit(within: &[String], words: &str, began: usize, out: &mut Vec<Declared>) {
    let light = match within {
        [root] if root == ":root" => Some(false),
        [media, root] if root == ":root" && light_media(media) => Some(true),
        _ => None,
    };
    if let (Some(light), Some((name, value))) = (light, words.trim().strip_prefix("--").and_then(|d| d.split_once(':'))) {
        out.push(Declared { line: began, name: name.trim().to_string(), value: value.trim().to_string(), light });
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Layers {
    pub default: Values,
    pub theme: Values,
    pub own: Values,
    pub held: Values,
    pub only: Option<bool>,
}

fn only_of(theme: &Values) -> Option<bool> {
    let said = theme.plain.get(LOOK).map(|l| l.trim().trim_matches('"').to_ascii_lowercase());
    match said.as_deref() {
        _ if theme.is_empty() => None,
        Some("light") => Some(true),
        Some("dark") => Some(false),
        Some("both") => None,
        _ => theme.light.is_empty().then_some(false),
    }
}

pub fn one_look(css: &str) -> Option<bool> {
    only_of(&Values::parse(css).keep_known(&Values::parse(CSS), false))
}

impl Layers {
    pub fn new(default: &str, own: &str) -> Layers {
        Layers::themed(default, "", own)
    }

    pub fn themed(default: &str, theme: &str, own: &str) -> Layers {
        let default = Values::parse(default);
        let theme = Values::parse(theme).keep_known(&default, false);
        let own = Values::parse(own).keep_known(&default, true);
        let only = only_of(&theme);
        Layers { default, theme, own, only, held: Values::default() }.titles_held()
    }

    fn titles_held(mut self) -> Layers {
        let asks: &[bool] = if self.only.is_some() { &[false] } else { &[false, true] };
        for &asked in asks {
            let light = self.light(asked);
            for (ink, bar) in TITLES {
                let Some(mut b) = self.colour(bar, light) else { continue };
                let best = |b: [f32; 4]| plain_colour(title_ink(b)).map_or(0.0, |i| title_contrast(i, b));
                if best(b) < TITLE_CONTRAST && !self.own_title(bar, light) {
                    let more = (1..=20).map(|i| format!("rgba({}, {}, {}, {})", b[0] * 255.0, b[1] * 255.0, b[2] * 255.0, b[3] + (1.0 - b[3]) * i as f32 / 20.0))
                        .find(|said| plain_colour(said).is_some_and(|c| best(c) >= TITLE_CONTRAST));
                    let Some(more) = more else { continue };
                    b = plain_colour(&more).unwrap_or(b);
                    if light { &mut self.held.light } else { &mut self.held.plain }.insert(bar.to_string(), more);
                }
                let Some(i) = self.colour(ink, light) else { continue };
                if self.own_title(ink, light) || title_contrast(i, b) >= TITLE_CONTRAST {
                    continue;
                }
                if light { &mut self.held.light } else { &mut self.held.plain }.insert(ink.to_string(), title_ink(b).to_string());
            }
        }
        self
    }

    pub fn own_title(&self, name: &str, light: bool) -> bool {
        self.own.get(name, light).is_some_and(|said| match said.trim().strip_prefix("var(--").and_then(|v| v.strip_suffix(')')) {
            Some(other) => self.own_readable(other.trim(), light, 1),
            None => plain_colour(said).is_some(),
        })
    }

    fn own_readable(&self, name: &str, light: bool, depth: u8) -> bool {
        depth <= 8
            && [&self.own, &self.default].into_iter().filter_map(|v| v.get(name, light)).any(|text| {
                match text.trim().strip_prefix("var(--").and_then(|v| v.strip_suffix(')')) {
                    Some(other) => self.own_readable(other.trim(), light, depth + 1),
                    None => plain_colour(text).is_some(),
                }
            })
    }

    pub fn light(&self, asked: bool) -> bool {
        self.only.unwrap_or(asked)
    }

    pub fn each<'a>(&'a self, name: &'a str, light: bool) -> impl Iterator<Item = &'a str> + 'a {
        let light = self.light(light);
        let held = if light { &self.held.light } else { &self.held.plain }.get(name).map(String::as_str);
        self.own.get(name, light).into_iter().chain(held).chain([&self.theme, &self.default].into_iter().filter_map(move |v| v.get(name, light)))
    }

    #[cfg(any(test, feature = "testing"))]
    pub fn get<'a>(&'a self, name: &'a str, light: bool) -> Option<&'a str> {
        self.each(name, light).next()
    }

    pub fn colour(&self, name: &str, light: bool) -> Option<[f32; 4]> {
        self.colour_at(name, light, 0)
    }

    fn colour_at(&self, name: &str, light: bool, depth: u8) -> Option<[f32; 4]> {
        if depth > 8 {
            return None;
        }
        let read = |text: &str| match text.trim().strip_prefix("var(--").and_then(|v| v.strip_suffix(')')) {
            Some(other) => self.colour_at(other.trim(), light, depth + 1),
            None => plain_colour(text),
        };
        self.each(name, light).find_map(read)
    }

    pub fn shadow(&self, name: &str, light: bool) -> Option<Vec<[f32; 4]>> {
        let read = |text: &str| shadow_layers(text, |c| plain_colour(c).map(|c| c[3]));
        self.each(name, light).find_map(read)
    }

    pub fn number(&self, name: &str, light: bool, min: f32, max: f32) -> Option<f32> {
        self.number_at(name, light, 0).map(|n| n.clamp(min, max))
    }

    fn number_at(&self, name: &str, light: bool, depth: u8) -> Option<f32> {
        if depth > 8 {
            return None;
        }
        let read = |text: &str| match text.trim().strip_prefix("var(--").and_then(|v| v.strip_suffix(')')) {
            Some(other) => self.number_at(other.trim(), light, depth + 1),
            None => number(text),
        };
        self.each(name, light).find_map(read)
    }

    pub fn step(&self, name: &str, light: bool) -> Option<f32> {
        let (kind, light) = (step_kind(name)?, self.light(light));
        let said = self.default.get(name, light)?.trim();
        let ours = number(said).or_else(|| self.stepped(kind, 0.0, said, light))?;
        let theirs = [&self.own, &self.theme].into_iter().filter_map(|v| v.get(name, light)).find_map(|s| self.stepped(kind, ours, s.trim(), light));
        Some(theirs.unwrap_or(ours))
    }

    fn stepped(&self, kind: Step, ours: f32, said: &str, light: bool) -> Option<f32> {
        match kind {
            Step::Gap if said == "none" => Some(0.0),
            Step::Gap if GAPS.contains(&said) => self.default.get(&format!("trindle-gap-{said}"), light).and_then(number),
            Step::Gap => None,
            Step::Size => SIZES.iter().find(|s| s.0 == said).map(|s| if s.1 == 1.0 { ours } else { (ours * s.1 / UNIT).round() * UNIT }),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    Gap,
    Size,
}

pub const GAPS: [&str; 7] = ["none", "tiny", "small", "medium", "roomy", "large", "huge"];
pub const SIZES: [(&str, f32); 5] = [("smaller", 0.75), ("small", 0.875), ("normal", 1.0), ("large", 1.25), ("larger", 1.5)];
pub const UNIT: f32 = 2.0;
pub const GUIDE: &str = "docs/style-sheet-for-ai.md";

pub fn step_kind(name: &str) -> Option<Step> {
    static KINDS: std::sync::OnceLock<HashMap<String, Step>> = std::sync::OnceLock::new();
    KINDS.get_or_init(|| CSS.lines().filter_map(|l| {
        let (n, rest) = l.trim_start().strip_prefix("--")?.split_once(':')?;
        let kind = if rest.contains("/* (gap step)") { Step::Gap } else if rest.contains("/* (size step)") { Step::Size } else { return None };
        Some((n.trim().to_string(), kind))
    }).collect()).get(name).copied()
}

pub fn is_step(kind: Step, said: &str) -> bool {
    let said = said.trim();
    match kind {
        Step::Gap => GAPS.contains(&said),
        Step::Size => SIZES.iter().any(|s| s.0 == said),
    }
}

pub fn steps(kind: Step) -> String {
    match kind {
        Step::Gap => GAPS.join(", "),
        Step::Size => SIZES.map(|s| s.0).join(", "),
    }
}

fn luminance(c: [f32; 4]) -> f32 {
    let lin = |v: f32| if v <= 0.039_28 { v / 12.92 } else { ((v + 0.055) / 1.055).powf(2.4) };
    0.2126 * lin(c[0]) + 0.7152 * lin(c[1]) + 0.0722 * lin(c[2])
}

fn kept(c: [f32; 4]) -> [f32; 4] {
    let k = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() / 255.0;
    [k(c[0]), k(c[1]), k(c[2]), c[3]]
}

pub fn contrast(a: [f32; 4], b: [f32; 4]) -> f32 {
    let (x, y) = (luminance(kept(a)), luminance(kept(b)));
    (x.max(y) + 0.05) / (x.min(y) + 0.05)
}

pub const TITLES: [(&str, &str); 2] =
    [("trindle-title-ink", "trindle-title-background"), ("trindle-title-away-ink", "trindle-title-away-background")];

fn laid_over(top: [f32; 4], under: [f32; 3]) -> [f32; 4] {
    let a = top[3].clamp(0.0, 1.0);
    let mix = |i: usize| top[i] * a + under[i] * (1.0 - a);
    [mix(0), mix(1), mix(2), 1.0]
}

pub fn title_contrast(ink: [f32; 4], bar: [f32; 4]) -> f32 {
    [[0.0; 3], [1.0; 3]]
        .into_iter()
        .map(|behind| {
            let bar = kept(laid_over(bar, behind));
            contrast(laid_over(ink, [bar[0], bar[1], bar[2]]), bar)
        })
        .fold(f32::INFINITY, f32::min)
}

pub fn title_ink(bar: [f32; 4]) -> &'static str {
    let far = |ink: [f32; 4]| title_contrast(ink, bar);
    if far([0.0, 0.0, 0.0, 1.0]) >= far([1.0, 1.0, 1.0, 1.0]) { "#000000" } else { "#ffffff" }
}

pub fn number(text: &str) -> Option<f32> {
    let v = text.trim();
    let (digits, times) = if let Some(n) = v.strip_suffix("ms") {
        (n, 1.0)
    } else if let Some(n) = v.strip_suffix('s') {
        (n, 1000.0)
    } else {
        (v.strip_suffix("px").or_else(|| v.strip_suffix("pt")).unwrap_or(v), 1.0)
    };
    digits.trim().parse::<f32>().ok().filter(|n| n.is_finite()).map(|n| n * times)
}

pub fn own_sheet() -> PathBuf {
    super::files::config_path("style.css")
}

const READ_WITHIN: std::time::Duration = std::time::Duration::from_millis(400);
const MOST_WAITING: usize = 4;

pub fn within<T: Send + 'static>(long: std::time::Duration, work: impl FnOnce() -> T + Send + 'static) -> Option<T> {
    static WAITING: super::run::Slots = super::run::Slots::new(MOST_WAITING);
    super::run::on_slot(WAITING.take()?, "sheet", long, work)
}

fn read_own() -> (String, Option<Unread>) {
    if cfg!(any(test, feature = "testing")) {
        return (std::env::var_os("TRINDLE_SHOT_SHEET").and_then(|f| std::fs::read_to_string(f).ok()).unwrap_or_default(), None);
    }
    let path = own_sheet();
    within(READ_WITHIN, move || own_words(&path, OWN_MOST)).unwrap_or_else(|| {
        eprintln!("trindle: {} took too long to read and is not used", own_sheet().display());
        (String::new(), Some(Unread::TooSlow))
    })
}

fn own_words(path: &std::path::Path, most: u64) -> (String, Option<Unread>) {
    match std::fs::symlink_metadata(path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return (String::new(), None),
        Ok(about) if about.file_type().is_symlink() && matches!(std::fs::metadata(path), Err(e) if e.kind() == std::io::ErrorKind::NotFound) => {
            eprintln!("trindle: {} is not used: it is a link to a file that is not there", path.display());
            return (String::new(), Some(Unread::NoTarget));
        }
        _ => {}
    }
    match words_of(path, most, true) {
        Ok(words) => (words, None),
        Err(why) => {
            eprintln!("trindle: {} is not used: {}", path.display(), match why {
                Unread::TooBig => format!("it is over {} KB", most / 1024),
                _ => "it is not an ordinary file of words".to_string(),
            });
            (String::new(), Some(why))
        }
    }
}

pub fn own_themes() -> PathBuf {
    super::files::config_path("themes")
}

pub fn system_themes() -> PathBuf {
    super::files::share("themes")
}

pub const OWN_NAMES: [&str; 2] = ["default", "trindle"];

pub fn theme_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 40
        && id.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        && !OWN_NAMES.contains(&id)
}

#[cfg(any(test, feature = "testing"))]
pub fn theme_file(id: &str) -> Option<PathBuf> {
    theme_file_in(id, &own_themes(), &system_themes())
}

pub fn theme_file_in(id: &str, own: &std::path::Path, system: &std::path::Path) -> Option<PathBuf> {
    if !theme_id(id) {
        return None;
    }
    [own, system].into_iter().find_map(|d| theme_in(d, id))
}

pub fn theme_in(folder: &std::path::Path, id: &str) -> Option<PathBuf> {
    let file = folder.join(format!("{id}.css"));
    std::fs::symlink_metadata(&file).ok().filter(|about| !about.is_dir()).map(|_| file)
}

pub fn theme_words(id: &str) -> Result<String, Unread> {
    let (id, own, system) = (id.to_string(), own_themes(), system_themes());
    within(READ_WITHIN, move || theme_words_in(&id, &own, &system)).unwrap_or(Err(Unread::TooSlow))
}

pub fn theme_words_in(id: &str, own: &std::path::Path, system: &std::path::Path) -> Result<String, Unread> {
    match theme_file_in(id, own, system) {
        Some(file) => read_sheet(&file, THEME_MOST),
        None => Err(Unread::Gone),
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Unread {
    TooBig,
    NotASheet,
    TooSlow,
    Gone,
    NoTarget,
}

pub fn read_sheet(path: &std::path::Path, most: u64) -> Result<String, Unread> {
    words_of(path, most, false)
}

fn words_of(path: &std::path::Path, most: u64, links: bool) -> Result<String, Unread> {
    use std::io::Read;
    use std::os::unix::fs::OpenOptionsExt;
    let file = std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NONBLOCK | if links { 0 } else { libc::O_NOFOLLOW })
        .open(path)
        .map_err(|_| Unread::NotASheet)?;
    let about = file.metadata().map_err(|_| Unread::NotASheet)?;
    if !about.is_file() {
        return Err(Unread::NotASheet);
    }
    if about.len() > most {
        return Err(Unread::TooBig);
    }
    let mut bytes = Vec::new();
    file.take(most + 1).read_to_end(&mut bytes).map_err(|_| Unread::NotASheet)?;
    if bytes.len() as u64 > most {
        return Err(Unread::TooBig);
    }
    String::from_utf8(bytes).map_err(|_| Unread::NotASheet)
}

static NO_THEMES: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub fn no_themes() {
    NO_THEMES.store(true, std::sync::atomic::Ordering::Relaxed);
}

fn now() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs())
}

fn trial_parts(said: &str, now: u64) -> Option<(String, u64, Option<bool>)> {
    let mut parts = said.trim().split('@');
    let (id, until, look) = (parts.next()?, parts.next()?, parts.next());
    if parts.next().is_some() {
        return None;
    }
    let until: u64 = until.parse().ok()?;
    until.checked_sub(now).filter(|left| (1..=TRIAL_FOR).contains(left))?;
    let look = match look {
        None => None,
        Some("light") => Some(true),
        Some("dark") => Some(false),
        Some(_) => return None,
    };
    match id {
        "default" => Some((String::new(), until, look)),
        id if theme_id(id) => Some((id.to_string(), until, look)),
        _ => None,
    }
}

pub fn trial_of(said: &str, now: u64) -> Option<(String, u64)> {
    trial_parts(said, now).map(|(id, until, _)| (id, until))
}

pub fn trial_look_of(said: &str, now: u64) -> Option<bool> {
    trial_parts(said, now)?.2
}

pub fn trial() -> Option<(String, u64)> {
    if cfg!(any(test, feature = "testing")) {
        return None;
    }
    trial_of(&super::files::saved(TRIAL_KEY), now())
}

pub fn trial_look() -> Option<bool> {
    if cfg!(any(test, feature = "testing")) {
        return None;
    }
    trial_look_of(&super::files::saved(TRIAL_KEY), now())
}

pub fn asked_light() -> bool {
    trial_look().unwrap_or_else(|| super::files::saved(LOOK_KEY) == "light")
}

pub fn trial_left() -> Option<std::time::Duration> {
    trial().map(|(_, until)| std::time::Duration::from_secs(until.saturating_sub(now())))
}

pub fn chosen_theme() -> String {
    if cfg!(any(test, feature = "testing")) {
        return String::new();
    }
    kept_theme(&super::files::saved(THEME_KEY), &super::files::saved(NAMING_KEY))
}

pub fn kept_theme(id: &str, naming: &str) -> String {
    if !theme_id(id) || (id == "midnight" && naming.is_empty()) { String::new() } else if id == "compact" { "snug".to_string() } else { id.to_string() }
}

pub fn theme_in_use() -> String {
    trial().map_or_else(chosen_theme, |(id, _)| id)
}

fn read_theme() -> (String, Option<(String, Unread)>) {
    if NO_THEMES.load(std::sync::atomic::Ordering::Relaxed) {
        return (String::new(), None);
    }
    if cfg!(any(test, feature = "testing")) {
        let words = std::env::var_os("TRINDLE_TEST_THEME").and_then(|p| read_sheet(std::path::Path::new(&p), THEME_MOST).ok());
        return (words.unwrap_or_default(), None);
    }
    let id = theme_in_use();
    if id.is_empty() {
        return (String::new(), None);
    }
    match theme_words(&id) {
        Ok(words) => (words, None),
        Err(why) => {
            eprintln!("trindle: the theme {id} cannot be used ({why:?}), so Trindle's own look is drawn");
            (String::new(), Some((id, why)))
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Refused {
    pub theme: Option<(String, Unread)>,
    pub own: Option<Unread>,
}

static LAYERS: RwLock<Option<(Arc<Layers>, Refused)>> = RwLock::new(None);
static REREADING: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn read_all() -> (Arc<Layers>, Refused) {
    let ((theme, no_theme), (own, no_own)) = (read_theme(), read_own());
    (Arc::new(Layers::themed(CSS, &theme, &own)), Refused { theme: no_theme, own: no_own })
}

fn read() -> (Arc<Layers>, Refused) {
    if let Some(l) = LAYERS.read().unwrap_or_else(|e| e.into_inner()).as_ref() {
        return l.clone();
    }
    let fresh = read_all();
    LAYERS.write().unwrap_or_else(|e| e.into_inner()).get_or_insert(fresh).clone()
}

pub fn layers() -> Arc<Layers> {
    read().0
}

pub fn refused() -> Refused {
    read().1
}

pub fn reread() -> bool {
    let _one = REREADING.lock().unwrap_or_else(|e| e.into_inner());
    let fresh = read_all();
    let mut held = LAYERS.write().unwrap_or_else(|e| e.into_inner());
    if held.as_ref().is_some_and(|h| *h.0 == *fresh.0 && h.1 == fresh.1) {
        return false;
    }
    *held = Some(fresh);
    true
}

pub const MOST_LAYERS: usize = 3;

pub fn shadow_layers(text: &str, alpha: impl Fn(&str) -> Option<f32>) -> Option<Vec<[f32; 4]>> {
    if text.trim().eq_ignore_ascii_case("none") {
        return Some(Vec::new());
    }
    let mut out = Vec::with_capacity(MOST_LAYERS);
    for layer in split_top(text, ',').into_iter().take(MOST_LAYERS) {
        let layer = layer.split_whitespace().collect::<Vec<_>>().join(" ");
        let words: Vec<&str> = split_top(&layer, ' ').into_iter().filter(|w| !w.is_empty() && *w != "inset").collect();
        let lengths: Vec<f32> = words.iter().map_while(|w| number(w).filter(|_| !w.ends_with("ms") && !w.ends_with('s'))).collect();
        let strength = match &words[lengths.len()..] {
            [] => 1.0,
            [c] => alpha(c)?,
            _ => return None,
        };
        let (y, blur, spread) = match lengths.as_slice() {
            [_, y] => (*y, 0.0, 0.0),
            [_, y, b] => (*y, *b, 0.0),
            [_, y, b, s] => (*y, *b, *s),
            _ => return None,
        };
        out.push([y.clamp(0.0, 32.0), blur.clamp(0.0, 64.0), spread.clamp(-32.0, 16.0), strength.clamp(0.0, 1.0)]);
    }
    (!out.is_empty()).then_some(out)
}

pub fn split_top(text: &str, at: char) -> Vec<&str> {
    let (mut out, mut depth, mut from) = (Vec::new(), 0i32, 0);
    for (i, c) in text.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => depth -= 1,
            c if c == at && depth == 0 => {
                out.push(text[from..i].trim());
                from = i + c.len_utf8();
            }
            _ => {}
        }
    }
    out.push(text[from..].trim());
    out
}

pub fn plain_colour(text: &str) -> Option<[f32; 4]> {
    let t = text.trim().to_ascii_lowercase();
    if let Some(hex) = t.strip_prefix('#') {
        if !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
            return None;
        }
        let one = |i: usize| u8::from_str_radix(&hex[i..=i], 16).ok().map(|d| (d * 17) as f32 / 255.0);
        let two = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).ok().map(|d| d as f32 / 255.0);
        return match hex.len() {
            3 => Some([one(0)?, one(1)?, one(2)?, 1.0]),
            4 => Some([one(0)?, one(1)?, one(2)?, one(3)?]),
            6 => Some([two(0)?, two(2)?, two(4)?, 1.0]),
            8 => Some([two(0)?, two(2)?, two(4)?, two(6)?]),
            _ => None,
        };
    }
    let inner = t.strip_prefix("rgba(").or_else(|| t.strip_prefix("rgb(")).and_then(|s| s.strip_suffix(')'))?;
    let parts: Vec<f32> = inner.split(',').map(|p| p.trim().parse::<f32>().ok()).collect::<Option<_>>()?;
    let byte = |v: f32| (0.0..=255.0).contains(&v).then_some(v / 255.0);
    match parts.as_slice() {
        [r, g, b] => Some([byte(*r)?, byte(*g)?, byte(*b)?, 1.0]),
        [r, g, b, a] if (0.0..=1.0).contains(a) => Some([byte(*r)?, byte(*g)?, byte(*b)?, *a]),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{step_kind, Layers, Step, CSS, GAPS, UNIT};

    #[test]
    fn a_step_is_taken_from_any_sheet_and_a_number_only_from_trindles_own() {
        let l = |own: &str| Layers::new(CSS, own);
        assert_eq!(l("").step("trindle-gap-small", false), Some(4.0));
        assert_eq!(l(":root { --trindle-gap-small: large; }").step("trindle-gap-small", false), Some(16.0));
        assert_eq!(l(":root { --trindle-gap-small: none; }").step("trindle-gap-small", false), Some(0.0));
        assert_eq!(l(":root { --trindle-gap-small: 5; }").step("trindle-gap-small", false), Some(4.0), "a number is passed over");
        assert_eq!(l(":root { --trindle-board-lane: 75px; }").step("trindle-board-lane", false), Some(260.0));
        assert_eq!(l(":root { --trindle-board-lane: larger; }").step("trindle-board-lane", false), Some(390.0));
        assert_eq!(l(":root { --trindle-board-lane: smaller; }").step("trindle-board-lane", false), Some(196.0), "a whole number of units");
        let theme = Layers::themed(CSS, ":root { --trindle-gap-small: medium; }", ":root { --trindle-gap-small: 9; }");
        assert_eq!(theme.step("trindle-gap-small", false), Some(8.0), "the person's number is passed over, the theme's step taken");
        assert_eq!(l("").step("trindle-gap", false), None, "a value with limits takes no steps");
    }

    #[test]
    fn every_gap_step_trindle_gives_is_a_whole_number_of_units() {
        let l = Layers::new(CSS, "");
        for g in &GAPS[1..] {
            let name = format!("trindle-gap-{g}");
            assert_eq!(step_kind(&name), Some(Step::Gap), "{name}");
            let v = l.number(&name, false, f32::MIN, f32::MAX).unwrap();
            assert!(v > 0.0 && v % UNIT == 0.0, "{name} is {v}");
        }
        assert!(super::declarations(CSS).iter().filter(|d| step_kind(&d.name) == Some(Step::Size)).all(|d| super::number(&d.value).is_some()));
    }

    #[test]
    fn every_light_value_replaces_one_for_either_look() {
        let v = super::Values::parse(super::CSS);
        assert!(!v.light.is_empty());
        for name in v.light.keys() {
            assert!(v.plain.contains_key(name), "{name}");
        }
    }

    #[test]
    fn the_float_shadow_reads_as_two_layers_in_either_look() {
        let l = super::layers();
        assert_eq!(l.shadow("trindle-float-shadow", false).map(|v| v.len()), Some(2));
        assert_eq!(l.shadow("trindle-float-shadow", true).map(|v| v.len()), Some(1));
        assert_eq!(l.shadow("trindle-no-such-shadow", false), None);
    }

    #[test]
    fn named_values_are_read_and_kept_within_their_limits() {
        let l = super::layers();
        assert_eq!(l.get("trindle-title-ink", false), Some("var(--trindle-text)"));
        assert_eq!(l.colour("trindle-title-ink", false), Some([236.0 / 255.0, 242.0 / 255.0, 251.0 / 255.0, 1.0]));
        assert_eq!(l.number("trindle-entry-height", false, 0.0, 100.0), Some(34.0));
        assert_eq!(l.number("trindle-label-rest", false, 0.0, 5000.0), Some(400.0));
        assert_eq!(l.number("trindle-remove-armed", false, 0.0, 10000.0), Some(3000.0));
        assert_eq!(l.number("trindle-button-height", false, 20.0, 30.0), Some(30.0));
        assert_eq!(l.number("no-such-value", false, 0.0, 10.0), None);
        assert_eq!(l.number("trindle-card-gap", false, 0.0, 100.0), Some(12.0));
    }

    #[test]
    fn the_light_block_changes_only_the_light_look() {
        let l = super::layers();
        assert_eq!(l.get("trindle-page", false), Some("#0a111e"));
        assert_eq!(l.get("trindle-page", true), Some("#ededed"));
        assert_eq!(l.get("trindle-gap", true), Some("12"));
    }

    #[test]
    fn a_sheet_is_read_as_css_is_written() {
        let v = super::Values::parse("/* a { --x: 1; } */ :root{--trindle-a:1;--trindle-b: rgba(0, 0, 0, 0.5) ;}
            a { --trindle-c: 2; } @media (prefers-color-scheme: dark) { :root { --trindle-d: 3; } }
            @media (prefers-color-scheme : light) { :root { --trindle-a: 4; } } :root { --trindle-e: ; --trindle-f: 5 }");
        assert_eq!(v.get("trindle-a", false), Some("1"));
        assert_eq!(v.get("trindle-a", true), Some("4"));
        assert_eq!(v.get("trindle-b", false), Some("rgba(0, 0, 0, 0.5)"));
        assert_eq!(v.get("trindle-f", false), Some("5"));
        for none in ["x", "trindle-c", "trindle-d", "trindle-e"] {
            assert_eq!(v.get(none, false), None, "{none}");
        }
        assert_eq!(super::Values::parse(":root { --trindle-a: 1; } } :root { --trindle-b: 2").get("trindle-a", false), Some("1"));
        assert_eq!(super::Values::parse("/* never closed :root { --trindle-a: 1; }").get("trindle-a", false), None);
    }

    #[test]
    fn plain_colours_and_names_of_others_are_worked_out_without_the_toolkit() {
        assert_eq!(super::plain_colour("#ff0000"), Some([1.0, 0.0, 0.0, 1.0]));
        assert_eq!(super::plain_colour("rgba(0, 0, 0, 0.5)"), Some([0.0, 0.0, 0.0, 0.5]));
        assert_eq!(super::plain_colour("rgb(300, 0, 0)"), None);
        let l = super::Layers::new(":root { --trindle-page: #000000; --trindle-a: var(--trindle-page); --trindle-b: #ffffff; }",
                                   ":root { --trindle-b: nonsense; --trindle-page: var(--trindle-page); }");
        assert_eq!(l.colour("trindle-a", false), Some([0.0, 0.0, 0.0, 1.0]));
        assert_eq!(l.colour("trindle-b", false), Some([1.0, 1.0, 1.0, 1.0]));
        assert_eq!(l.colour("trindle-page", false), Some([0.0, 0.0, 0.0, 1.0]));
        assert_eq!(super::layers().colour("trindle-calm", false).map(|c| (c[0] * 255.0).round()), Some(34.0));
    }

    fn titled(bar: &str, ink: &str) -> super::Layers {
        let theme = format!(":root {{ --trindle-title-background: {bar}; --trindle-title-away-background: {bar};
            --trindle-title-ink: {ink}; --trindle-title-away-ink: {ink}; }}");
        super::Layers::themed(super::CSS, &theme, "")
    }

    #[test]
    fn black_or_white_can_be_read_on_a_solid_bar_of_any_colour() {
        let levels: Vec<u8> = (0..=255u8).step_by(3).collect();
        let greys = (0..=255u8).map(|g| [g, g, g]);
        let colours = levels.iter().flat_map(|r| levels.iter().flat_map(|g| levels.iter().map(|b| [*r, *g, *b])));
        let mut least = f32::INFINITY;
        for [r, g, b] in greys.chain(colours) {
            let bar = [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0];
            let ink = super::plain_colour(super::title_ink(bar)).unwrap();
            let got = super::title_contrast(ink, bar);
            assert!(got >= super::TITLE_CONTRAST, "#{r:02x}{g:02x}{b:02x}: {got}");
            least = least.min(got);
        }
        assert!((4.57..4.62).contains(&least), "{least}");
        let grey = super::plain_colour("#767676").unwrap();
        assert!(super::title_contrast(super::plain_colour("#111111").unwrap(), grey) < super::TITLE_CONTRAST);
        let grey = super::plain_colour("#7a7a7a").unwrap();
        assert!(super::title_contrast(super::plain_colour("#f2f2f2").unwrap(), grey) < super::TITLE_CONTRAST);
    }

    #[test]
    fn a_value_at_the_very_end_of_a_sheet_is_read() {
        let v = super::Values::parse(":root { --trindle-gap: 14; --trindle-card: #223344");
        assert_eq!((v.get("trindle-gap", false), v.get("trindle-card", false)), (Some("14"), Some("#223344")));
        let v = super::Values::parse(":root { --trindle-card: #223344 }\n@media (prefers-color-scheme: light) { :root { --trindle-page: #f0f4f8");
        assert_eq!((v.get("trindle-page", true), v.get("trindle-page", false)), (Some("#f0f4f8"), None));
        let d = super::declarations(":root {\n  --trindle-gap: 14;\n  --trindle-card:\n    #223344\n");
        assert_eq!(d.last().map(|d| (d.line, d.name.as_str(), d.value.as_str(), d.light)), Some((3, "trindle-card", "#223344", false)));
        for not in ["--trindle-card: #223344", "body { --trindle-card: #223344", ":root { --trindle-card", ":root { --trindle-card:",
                    ":root { } --trindle-card: #223344", ":root { --trindle-card: #223344 /* never closed"] {
            let v = super::Values::parse(not);
            let ok = not.ends_with("never closed");
            assert_eq!(v.get("trindle-card", false).is_some(), ok, "{not}");
        }
    }

    #[test]
    fn a_read_that_never_comes_back_is_left_after_its_time() {
        use std::time::{Duration, Instant};
        assert_eq!(super::within(Duration::from_secs(5), || 7), Some(7));
        let since = Instant::now();
        let (hold, held) = std::sync::mpsc::channel::<()>();
        let stuck = move || held.recv_timeout(Duration::from_secs(60)).is_ok();
        assert_eq!(super::within(Duration::from_millis(150), stuck), None);
        assert!(since.elapsed() < Duration::from_secs(5), "{:?}", since.elapsed());
        assert_eq!(super::within(Duration::from_secs(5), || 8), Some(8));
        drop(hold);
    }

    fn room(name: &str) -> std::path::PathBuf {
        super::super::files::private_temp(&format!("trindle-sheet-{name}")).unwrap()
    }

    #[test]
    fn a_sheet_that_is_a_link_to_nothing_is_said_as_that() {
        let dir = room("no-target");
        let link = dir.join("style.css");
        std::os::unix::fs::symlink(dir.join("never-there.css"), &link).unwrap();
        assert_eq!(own_within(link.clone()), Some((String::new(), Some(super::Unread::NoTarget))));
        std::os::unix::fs::symlink(&link, dir.join("second.css")).unwrap();
        assert_eq!(own_within(dir.join("second.css")), Some((String::new(), Some(super::Unread::NoTarget))));
        std::fs::write(dir.join("never-there.css"), ":root { --trindle-gap: 14; }").unwrap();
        assert_eq!(own_within(link), Some((":root { --trindle-gap: 14; }".to_string(), None)));
        let _ = std::fs::remove_dir_all(dir);
    }

    fn own_within(path: std::path::PathBuf) -> Option<(String, Option<super::Unread>)> {
        let (tell, heard) = std::sync::mpsc::channel();
        std::thread::spawn(move || tell.send(super::own_words(&path, super::OWN_MOST)));
        heard.recv_timeout(std::time::Duration::from_secs(5)).ok()
    }

    #[test]
    fn a_persons_own_sheet_that_is_no_file_of_words_holds_nothing_up_and_is_said() {
        use super::Unread::{NotASheet, TooBig};
        let dir = room("own");
        let pipe = dir.join("pipe.css");
        let c = std::ffi::CString::new(pipe.to_str().unwrap()).unwrap();
        assert_eq!(crate::sys::os::mkfifo(&c, 0o600), 0);
        assert_eq!(own_within(pipe), Some((String::new(), Some(NotASheet))), "a pipe");
        assert_eq!(own_within("/dev/zero".into()), Some((String::new(), Some(NotASheet))), "a device");
        assert_eq!(own_within(dir.clone()), Some((String::new(), Some(NotASheet))), "a folder");
        std::fs::write(dir.join("binary.css"), [0xff, 0xfe, 0x00, 0x41]).unwrap();
        assert_eq!(own_within(dir.join("binary.css")), Some((String::new(), Some(NotASheet))), "not words");
        std::fs::write(dir.join("big.css"), vec![b' '; super::OWN_MOST as usize + 1]).unwrap();
        assert_eq!(own_within(dir.join("big.css")), Some((String::new(), Some(TooBig))));
        assert_eq!(own_within(dir.join("none.css")), Some((String::new(), None)));
        std::fs::write(dir.join("style.css"), ":root { --trindle-gap: 14; }").unwrap();
        std::os::unix::fs::symlink(dir.join("style.css"), dir.join("link.css")).unwrap();
        assert_eq!(own_within(dir.join("link.css")), Some((":root { --trindle-gap: 14; }".to_string(), None)));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn a_title_can_be_read_on_a_bar_whichever_sheet_set_the_bar() {
        let levels = [0u8, 51, 102, 118, 122, 153, 204, 255];
        let greys = (0..=255u8).map(|g| [g, g, g]);
        let colours = levels.iter().flat_map(|r| levels.iter().flat_map(|g| levels.iter().map(|b| [*r, *g, *b])));
        for [r, g, b] in greys.chain(colours) {
            let bar = format!("#{r:02x}{g:02x}{b:02x}");
            let bars = format!(":root {{ --trindle-title-background: {bar}; --trindle-title-away-background: {bar}; }}");
            let words = ":root { --trindle-title-ink: #808080; --trindle-title-away-ink: rgba(255, 255, 255, 0.05); }";
            let cases = [("own bar", super::Layers::themed(super::CSS, "", &bars)),
                         ("own bar, a theme's words", super::Layers::themed(super::CSS, words, &bars)),
                         ("a theme's bar", super::Layers::themed(super::CSS, &bars, ""))];
            for (what, l) in cases {
                for light in [false, true] {
                    for (words, on) in super::TITLES {
                        let (i, b) = (l.colour(words, light).unwrap(), l.colour(on, light).unwrap());
                        let got = super::title_contrast(i, b);
                        assert!(got >= super::TITLE_CONTRAST, "{what}: {words} on {bar}, light {light}: {got}");
                    }
                }
            }
        }
        let own = super::Layers::themed(super::CSS, "", ":root { --trindle-title-background: #767676; }");
        assert!(own.theme.is_empty() && own.only.is_none());
        assert_ne!(own, super::Layers::new(super::CSS, ""));
        let own = super::Layers::themed(super::CSS, "", ":root { --trindle-title-background: #767676; --trindle-title-ink: #777777; }");
        assert_eq!(own.get("trindle-title-ink", false), Some("#777777"));
        let plain = super::Layers::new(super::CSS, "");
        for (words, _) in super::TITLES {
            assert_eq!(plain.get(words, false), plain.default.get(words, false));
            assert_eq!(plain.get(words, true), plain.default.get(words, true));
        }
    }

    #[test]
    fn a_themes_title_can_be_read_on_its_bar_whatever_the_bar() {
        let levels = [0u8, 51, 102, 118, 122, 153, 204, 255];
        let greys = (0..=255u8).map(|g| [g, g, g]);
        let colours = levels.iter().flat_map(|r| levels.iter().flat_map(|g| levels.iter().map(|b| [*r, *g, *b])));
        for [r, g, b] in greys.chain(colours) {
            let bar = format!("#{r:02x}{g:02x}{b:02x}");
            for ink in [bar.as_str(), "#111111", "#f2f2f2", "rgba(255, 255, 255, 0.05)", "rgba(0, 0, 0, 0.05)"] {
                let l = titled(&bar, ink);
                for light in [false, true] {
                    for (words, on) in super::TITLES {
                        let (i, b) = (l.colour(words, light).unwrap(), l.colour(on, light).unwrap());
                        let got = super::title_contrast(i, b);
                        assert!(got >= super::TITLE_CONTRAST, "{ink} on {bar}, light {light}: {got}");
                    }
                }
            }
        }
        let l = titled("#102030", "#e0e8f0");
        assert_eq!(l.get("trindle-title-ink", false), Some("#e0e8f0"));
        let own = super::Layers::themed(super::CSS, ":root { --trindle-title-background: #767676; }", ":root { --trindle-title-ink: #777777; }");
        assert_eq!(own.get("trindle-title-ink", false), Some("#777777"));
        assert!(super::Layers::new(super::CSS, "").theme.is_empty());
    }

    #[test]
    fn title_words_set_for_one_look_only_are_held_in_the_other() {
        let own = ":root { --trindle-title-background: #767676; --trindle-title-away-background: #767676; }
            @media (prefers-color-scheme: light) { :root { --trindle-title-ink: #222222; --trindle-title-away-ink: #222222; } }";
        for theme in ["", ":root { --trindle-theme-look: both; --trindle-title-ink: #808080; --trindle-title-away-ink: #7a7a7a; }"] {
            let l = super::Layers::themed(super::CSS, theme, own);
            for (words, on) in super::TITLES {
                let (i, b) = (l.colour(words, false).unwrap(), l.colour(on, false).unwrap());
                assert!(super::title_contrast(i, b) >= super::TITLE_CONTRAST, "{words} in the dark look: {}", super::title_contrast(i, b));
                assert!(!l.own_title(words, false) && l.own_title(words, true));
                assert_eq!(l.get(words, true), Some("#222222"));
            }
        }
        let l = super::Layers::themed(super::CSS, "", ":root { --trindle-title-background: #767676; --trindle-title-ink: shade(#ffffff, 0.9); }");
        let (i, b) = (l.colour("trindle-title-ink", false).unwrap(), l.colour("trindle-title-background", false).unwrap());
        assert!(super::title_contrast(i, b) >= super::TITLE_CONTRAST, "{}", super::title_contrast(i, b));
    }

    #[test]
    fn words_put_right_for_one_look_leave_the_other_look_as_the_theme_wrote_it() {
        let theme = ":root { --trindle-title-background: #767676; --trindle-title-ink: #808080; }
            @media (prefers-color-scheme: light) { :root { --trindle-title-background: #ffffff; --trindle-title-ink: #1a2b3c; } }";
        let l = super::Layers::themed(super::CSS, theme, "");
        assert_eq!(l.get("trindle-title-ink", false), Some("#000000"));
        assert_eq!(l.get("trindle-title-ink", true), Some("#1a2b3c"));
        let theme = ":root { --trindle-title-background: #101820; --trindle-title-ink: #e8eef4; }
            @media (prefers-color-scheme: light) { :root { --trindle-title-background: #767676; --trindle-title-ink: #808080; } }";
        let l = super::Layers::themed(super::CSS, theme, "");
        assert_eq!(l.get("trindle-title-ink", false), Some("#e8eef4"));
        assert_eq!(l.get("trindle-title-ink", true), Some("#000000"));
    }

    #[test]
    fn a_see_through_bar_is_made_solid_enough_for_its_words() {
        let mut dice = 0x2545_f491u32;
        let mut next = move || { dice ^= dice << 13; dice ^= dice >> 17; dice ^= dice << 5; dice };
        let mut bars: Vec<String> = ["rgba(0, 0, 0, 0.5)", "rgba(255, 255, 255, 0.5)", "rgba(128, 128, 128, 0.3)", "rgba(0, 0, 0, 0)", "#00000080"].iter()
            .map(|b| b.to_string()).collect();
        bars.extend((0..400).map(|_| format!("rgba({}, {}, {}, 0.{:02})", next() % 256, next() % 256, next() % 256, next() % 100)));
        for bar in &bars {
            for ink in ["#ffffff", "#808080", "rgba(0, 0, 0, 0.3)"] {
                let theme = format!(":root {{ --trindle-title-background: {bar}; --trindle-title-away-background: {bar};
                    --trindle-title-ink: {ink}; --trindle-title-away-ink: {ink}; }}");
                let l = super::Layers::themed(super::CSS, &theme, "");
                for light in [false, true] {
                    for (words, on) in super::TITLES {
                        let (i, b) = (l.colour(words, light).unwrap(), l.colour(on, light).unwrap());
                        assert!(super::title_contrast(i, b) >= super::TITLE_CONTRAST, "{ink} on {bar}, light {light}: {}", super::title_contrast(i, b));
                        let was = super::plain_colour(bar).unwrap();
                        assert!((0..3).all(|n| (b[n] - was[n]).abs() < 0.003) && b[3] >= was[3] - 0.001, "{bar} became {b:?}");
                    }
                }
            }
        }
        let l = super::Layers::themed(super::CSS, ":root { --trindle-title-background: rgba(0, 0, 0, 0.9); }", "");
        assert_eq!(l.get("trindle-title-background", false), Some("rgba(0, 0, 0, 0.9)"));
        let l = super::Layers::themed(super::CSS, "", ":root { --trindle-title-background: rgba(0, 0, 0, 0.5); }");
        assert_eq!(l.get("trindle-title-background", false), Some("rgba(0, 0, 0, 0.5)"));
        assert_eq!(l.get("trindle-title-ink", false), Some("#ffffff"));
    }

    #[test]
    fn a_bar_that_is_not_solid_gets_the_better_of_black_and_white() {
        for (bar, want) in [("rgba(0, 0, 0, 0.9)", "#ffffff"), ("rgba(255, 255, 255, 0.9)", "#000000"), ("rgba(20, 30, 60, 0.9)", "#ffffff")] {
            let l = titled(bar, bar);
            assert_eq!(l.get("trindle-title-ink", false), Some(want), "{bar}");
            let b = super::plain_colour(bar).unwrap();
            let (black, white) = ([0.0, 0.0, 0.0, 1.0], [1.0, 1.0, 1.0, 1.0]);
            let chosen = super::plain_colour(want).unwrap();
            assert!(super::title_contrast(chosen, b) >= super::title_contrast(black, b).max(super::title_contrast(white, b)) - 0.001);
        }
    }

    #[test]
    fn a_persons_sheet_wins_name_by_name_as_a_later_sheet_does() {
        let default = ":root { --trindle-page: #111111; --trindle-gap: 12; }
            @media (prefers-color-scheme: light) { :root { --trindle-page: #eeeeee; } }";
        let l = super::Layers::new(default, ":root { --trindle-page: #222222; --trindle-new: 1; --trindle-highlight: #b4543a; }");
        assert_eq!(l.get("trindle-page", true), Some("#222222"));
        assert_eq!(l.get("trindle-new", false), None);
        assert_eq!(l.get("trindle-highlight", false), Some("#b4543a"));
        let l = super::Layers::new(default, ":root { --trindle-page: #222222; }
            @media (prefers-color-scheme: light) { :root { --trindle-page: #dddddd; } }");
        assert_eq!((l.get("trindle-page", false), l.get("trindle-page", true)), (Some("#222222"), Some("#dddddd")));
        let l = super::Layers::new(default, ":root { --trindle-gap: wide; }");
        assert_eq!(l.number("trindle-gap", false, 4.0, 36.0), Some(12.0));
        let l = super::Layers::new(default, ":root { --trindle-gap: 99px; }");
        assert_eq!(l.number("trindle-gap", false, 4.0, 36.0), Some(36.0));
    }
}
