//! What the person does to a window as plain events in points, keys read through their layout.

use std::os::fd::OwnedFd;
use std::time::Duration;

use xkbcommon::xkb;
pub use xkbcommon::xkb::Keysym;

use super::dnd::{Given, Offered, Way};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Button {
    Left,
    Right,
    Middle,
    Back,
    Forward,
    Other(u32),
}

impl Button {
    pub fn from_code(code: u32) -> Button {
        match code {
            0x110 => Button::Left,
            0x111 => Button::Right,
            0x112 => Button::Middle,
            0x113 | 0x116 => Button::Back,
            0x114 | 0x115 => Button::Forward,
            c => Button::Other(c),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Mods {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub logo: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Input {
    PointerIn { x: f32, y: f32 },
    PointerMove { x: f32, y: f32 },
    PointerOut,
    Pen { x: f32, y: f32, pressure: f32, tilt: (f32, f32), touching: bool, eraser: bool },
    PenOut,
    PointerBy { x: f32, y: f32 },
    Press { button: Button, x: f32, y: f32 },
    Release { button: Button, x: f32, y: f32 },
    Scroll { x: f32, y: f32, steps: bool },
    Key { sym: Keysym, text: Option<String>, mods: Mods, repeat: bool },
    KeyUp { sym: Keysym, mods: Mods },
    Mods(Mods),
    Focus(bool),
    TouchDown { id: i32, x: f32, y: f32 },
    TouchMove { id: i32, x: f32, y: f32 },
    TouchUp { id: i32 },
    TouchCancel,
    PadPinch(crate::pinch::Twist),
    PadPinchEnd { cancelled: bool },
    Composed { delete: (u32, u32), commit: String, preedit: String, caret: Option<(usize, usize)> },
    DragIn { x: f32, y: f32, offered: Offered, kinds: Vec<String> },
    DragOver { x: f32, y: f32 },
    DragGone,
    Dropped { x: f32, y: f32, given: Given, way: Way, id: u64 },
    DragEnded { took: Option<super::dnd::Taken>, short: Option<super::dnd::Short> },
    Refused,
    NoLayout,
    Placed,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Typing {
    pub id: String,
    pub text: String,
    pub caret: usize,
    pub anchor: usize,
    pub area: (i32, i32, i32, i32),
    pub secret: bool,
    pub purpose: Purpose,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Purpose {
    #[default]
    Words,
    Digits,
    Number,
    Phone,
    Address,
    Email,
    Name,
    Date,
    Time,
    Pin,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Cursor {
    #[default]
    Arrow,
    Hand,
    Text,
    Crosshair,
    Move,
    NResize,
    NeResize,
    EResize,
    SeResize,
    SResize,
    SwResize,
    WResize,
    NwResize,
    ColResize,
    RowResize,
}

pub struct Keyboard {
    context: xkb::Context,
    state: Option<xkb::State>,
    keymap: Option<xkb::Keymap>,
    compose: Option<xkb::compose::State>,
    pub repeat: Option<(Duration, Duration)>,
}

impl Default for Keyboard {
    fn default() -> Self {
        Self::new()
    }
}

const KEYMAP_MOST: u32 = 4 << 20;

fn keymap_text(fd: OwnedFd, size: u32) -> Option<String> {
    use std::os::unix::fs::FileExt;
    let file = std::fs::File::from(fd);
    let mut text = vec![0u8; size.min(KEYMAP_MOST) as usize];
    let mut got = 0;
    while got < text.len() {
        match file.read_at(&mut text[got..], got as u64) {
            Ok(0) => break,
            Ok(n) => got += n,
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {}
            Err(_) => return None,
        }
    }
    text.truncate(got);
    let text = String::from_utf8(text).ok()?;
    Some(text.trim_end_matches('\0').to_string())
}

const REPEAT_DELAY: Duration = Duration::from_millis(600);
const REPEAT_EVERY: Duration = Duration::from_millis(40);

impl Keyboard {
    pub fn new() -> Keyboard {
        let context = xkb::Context::new(xkb::CONTEXT_NO_FLAGS);
        let compose = compose_for(&context, &locale());
        Keyboard { context, state: None, keymap: None, compose, repeat: Some((REPEAT_DELAY, REPEAT_EVERY)) }
    }

    pub fn set_keymap(&mut self, fd: OwnedFd, size: u32) -> bool {
        let map = keymap_text(fd, size).and_then(|text| xkb::Keymap::new_from_string(&self.context, text, xkb::KEYMAP_FORMAT_TEXT_V1, xkb::KEYMAP_COMPILE_NO_FLAGS));
        match map {
            Some(map) => {
                self.use_keymap(map);
                true
            }
            None => {
                eprintln!("trindle: the keyboard layout could not be read");
                false
            }
        }
    }

    pub fn has_layout(&self) -> bool {
        self.state.is_some()
    }

    fn use_keymap(&mut self, map: xkb::Keymap) {
        self.state = Some(xkb::State::new(&map));
        self.keymap = Some(map);
    }

    pub fn set_mods(&mut self, depressed: u32, latched: u32, locked: u32, group: u32) {
        if let Some(s) = &mut self.state {
            s.update_mask(depressed, latched, locked, 0, 0, group);
        }
    }

    pub fn set_repeat(&mut self, rate: i32, delay: i32) {
        self.repeat = (rate > 0).then(|| (Duration::from_millis(delay.max(0) as u64),
                                          Duration::from_micros(1_000_000 / rate as u64)));
    }

    pub fn mods(&self) -> Mods {
        let Some(s) = &self.state else { return Mods::default() };
        let on = |name: &str| s.mod_name_is_active(name, xkb::STATE_MODS_EFFECTIVE);
        Mods { shift: on(xkb::MOD_NAME_SHIFT), ctrl: on(xkb::MOD_NAME_CTRL), alt: on(xkb::MOD_NAME_ALT),
               logo: on(xkb::MOD_NAME_LOGO) }
    }

    pub fn repeats(&self, key: u32) -> bool {
        self.repeat.is_some() && self.keymap.as_ref().is_some_and(|m| m.key_repeats(code(key)))
    }

    pub fn press(&mut self, key: u32, repeat: bool) -> Option<Input> {
        let state = self.state.as_ref()?;
        let sym = state.key_get_one_sym(code(key));
        let mods = self.mods();
        let mut text = Some(state.key_get_utf8(code(key)));
        let mut sym_out = sym;
        if let Some(c) = &mut self.compose {
            if c.feed(sym) == xkb::compose::FeedResult::Accepted {
                match c.status() {
                    xkb::compose::Status::Composing => return None,
                    xkb::compose::Status::Composed => {
                        text = c.utf8();
                        sym_out = c.keysym().unwrap_or(sym);
                        c.reset();
                    }
                    xkb::compose::Status::Cancelled => {
                        c.reset();
                        return None;
                    }
                    xkb::compose::Status::Nothing => {}
                }
            }
        }
        if (mods.ctrl || mods.alt) && !is_latin(sym_out) {
            sym_out = self.latin_sym(key, mods.shift).unwrap_or(sym_out);
        }
        let text = text.filter(|t| !t.is_empty() && !t.chars().any(char::is_control));
        Some(Input::Key { sym: sym_out, text, mods, repeat })
    }

    fn latin_sym(&self, key: u32, shifted: bool) -> Option<Keysym> {
        let map = self.keymap.as_ref()?;
        (0..map.num_layouts()).find_map(|l| map.key_get_syms_by_level(code(key), l, u32::from(shifted)).first().copied().filter(|s| is_latin(*s)))
    }

    pub fn release(&self, key: u32) -> Option<Input> {
        let sym = self.state.as_ref()?.key_get_one_sym(code(key));
        Some(Input::KeyUp { sym, mods: self.mods() })
    }
}

fn is_latin(sym: Keysym) -> bool {
    (0x20..0x7f).contains(&sym.raw())
}

fn code(key: u32) -> xkb::Keycode {
    xkb::Keycode::new(key.saturating_add(8))
}

fn locale() -> String {
    ["LC_ALL", "LC_CTYPE", "LANG"].iter()
        .find_map(|v| std::env::var(v).ok().filter(|s| !s.is_empty()))
        .unwrap_or_else(|| "C".into())
}

fn compose_for(context: &xkb::Context, locale: &str) -> Option<xkb::compose::State> {
    let table = xkb::compose::Table::new_from_locale(context, locale.as_ref(), xkb::compose::COMPILE_NO_FLAGS).ok()?;
    Some(xkb::compose::State::new(&table, xkb::compose::STATE_NO_FLAGS))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn british() -> Keyboard {
        let mut k = Keyboard::new();
        let map = xkb::Keymap::new_from_names(&k.context, "", "pc105", "gb", "", None, xkb::KEYMAP_COMPILE_NO_FLAGS)
            .expect("the gb layout");
        k.use_keymap(map);
        k.compose = compose_for(&k.context, "en_GB.UTF-8");
        k
    }

    const A: u32 = 30;
    const ENTER: u32 = 28;
    const LEFT_SHIFT: u32 = 42;

    #[test]
    fn keys_type_through_the_layout() {
        let mut k = british();
        assert_eq!(k.press(A, false), Some(Input::Key { sym: Keysym::a, text: Some("a".into()), mods: Mods::default(), repeat: false }));
        k.set_mods(1, 0, 0, 0);
        let Some(Input::Key { sym, text, mods, .. }) = k.press(A, true) else { panic!() };
        assert_eq!((sym, text.as_deref(), mods.shift), (Keysym::A, Some("A"), true));
        k.set_mods(0, 0, 0, 0);
        let Some(Input::Key { sym, text, .. }) = k.press(ENTER, false) else { panic!() };
        assert_eq!((sym, text), (Keysym::Return, None));
        assert!(k.repeats(A) && !k.repeats(LEFT_SHIFT));
        assert!(matches!(k.release(A), Some(Input::KeyUp { sym: Keysym::a, .. })));
    }

    #[test]
    fn a_shortcut_types_nothing() {
        let mut k = british();
        k.set_mods(4, 0, 0, 0);
        let Some(Input::Key { sym, text, mods, .. }) = k.press(A, false) else { panic!() };
        assert_eq!((sym, text, mods.ctrl), (Keysym::a, None, true));
    }

    #[test]
    fn a_shortcut_on_a_non_latin_layout_reports_the_latin_letter() {
        let mut k = Keyboard::new();
        let map = xkb::Keymap::new_from_names(&k.context, "", "pc105", "us,ru", "", None, xkb::KEYMAP_COMPILE_NO_FLAGS).expect("the us and ru layouts");
        k.use_keymap(map);
        k.set_mods(4, 0, 0, 1);
        let Some(Input::Key { sym, mods, .. }) = k.press(A, false) else { panic!() };
        assert_eq!((sym, mods.ctrl), (Keysym::a, true));
        k.set_mods(0, 0, 0, 1);
        let Some(Input::Key { text, .. }) = k.press(A, false) else { panic!() };
        assert_eq!(text.as_deref(), Some("ф"));
    }

    #[test]
    fn a_dead_key_joins_the_next_letter() {
        let mut k = british();
        crate::common::needs::file(std::path::Path::new("/usr/share/X11/locale/en_US.UTF-8/Compose"), "the X11 compose tables (Debian's libx11-data)");
        let Some(c) = &mut k.compose else { panic!("no dead-key table could be built for en_GB.UTF-8") };
        assert_eq!(c.feed(Keysym::dead_acute), xkb::compose::FeedResult::Accepted);
        assert_eq!(c.status(), xkb::compose::Status::Composing);
        let got = k.press(18, false);
        let Some(Input::Key { text, .. }) = got else { panic!("{got:?}") };
        assert_eq!(text.as_deref(), Some("é"));
    }

    #[test]
    fn repeat_speed_comes_from_the_compositor() {
        let mut k = Keyboard::new();
        k.set_repeat(25, 400);
        assert_eq!(k.repeat, Some((Duration::from_millis(400), Duration::from_millis(40))));
        k.set_repeat(0, 400);
        assert_eq!(k.repeat, None);
    }

    #[test]
    fn buttons_have_names() {
        assert_eq!(Button::from_code(0x110), Button::Left);
        assert_eq!(Button::from_code(0x111), Button::Right);
        assert_eq!(Button::from_code(0x200), Button::Other(0x200));
    }

    #[test]
    fn a_layout_file_left_at_its_end_is_still_read() {
        use std::io::Write;
        let k = british();
        let mut text = k.keymap.as_ref().unwrap().get_as_string(xkb::KEYMAP_FORMAT_TEXT_V1).into_bytes();
        text.push(0);
        let path = std::env::temp_dir().join(format!("trindle-keymap-{}", std::process::id()));
        let mut file = std::fs::File::options().read(true).write(true).create(true).truncate(true).open(&path).unwrap();
        file.write_all(&text).unwrap();
        std::fs::remove_file(&path).ok();
        let mut fresh = Keyboard::new();
        assert!(!fresh.has_layout());
        assert!(fresh.set_keymap(file.into(), text.len() as u32));
        assert!(fresh.has_layout());
        let empty = std::fs::File::open("/dev/null").unwrap();
        let mut none = Keyboard::new();
        assert!(!none.set_keymap(empty.into(), 100));
        assert!(!none.has_layout());
    }
}
