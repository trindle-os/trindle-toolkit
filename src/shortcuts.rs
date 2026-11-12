//! A table of keyboard shortcuts a program keeps: each key chord and what it does, found from a key press, written out for a menu or a help page, and checked for two doing the same chord.

use super::input::{Input, Keysym, Mods};
use super::widget::{column, label, note, page_title, row, space, Node, Tone};
use crate::t;
use xkbcommon::xkb;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Chord {
    pub sym: Keysym,
    pub mods: Mods,
}

impl Chord {
    pub fn parse(text: &str) -> Option<Chord> {
        let mut mods = Mods::default();
        let (head, key) = match text.trim().strip_suffix('+') {
            Some(h) if h.is_empty() || h.ends_with('+') => (h.strip_suffix('+').unwrap_or(h), "plus"),
            _ => text.trim().rsplit_once('+').map_or(("", text.trim()), |(h, k)| (h, k.trim())),
        };
        if key.is_empty() {
            return None;
        }
        let words: Vec<&str> = if head.is_empty() { Vec::new() } else { head.split('+').map(str::trim).collect() };
        for word in words {
            match word.to_ascii_lowercase().as_str() {
                "ctrl" | "control" => mods.ctrl = true,
                "shift" => mods.shift = true,
                "alt" => mods.alt = true,
                "logo" | "super" => mods.logo = true,
                _ => return None,
            }
        }
        let sym = xkb::keysym_from_name(key, xkb::KEYSYM_CASE_INSENSITIVE);
        (sym.raw() != 0).then_some(Chord { sym: lower(sym), mods })
    }

    pub fn pressed(&self, input: &Input) -> bool {
        self.is(input) && !matches!(input, Input::Key { repeat: true, .. })
    }

    pub fn is(&self, input: &Input) -> bool {
        let Input::Key { sym, mods, .. } = input else { return false };
        let (mut sym, mut shift) = (lower(*sym), mods.shift);
        if sym == Keysym::ISO_Left_Tab {
            (sym, shift) = (Keysym::Tab, true);
        } else if shift && !self.mods.shift && char::from_u32(xkb::keysym_to_utf32(sym)).is_some_and(|c| !c.is_alphanumeric() && !c.is_control()) {
            shift = false;
        }
        sym == self.sym && (mods.ctrl, shift, mods.alt, mods.logo) == mods_of(self.mods)
    }

    pub fn text(&self) -> String {
        let mut out = String::new();
        for (on, name) in [(self.mods.ctrl, t!("kit-key-ctrl")), (self.mods.shift, t!("kit-key-shift")), (self.mods.alt, t!("kit-key-alt")), (self.mods.logo, t!("kit-key-logo"))] {
            if on {
                out.push_str(name);
                out.push('+');
            }
        }
        match said(self.sym) {
            Some(name) => out.push_str(name),
            None => {
                let name = xkb::keysym_get_name(self.sym);
                if name.chars().count() == 1 {
                    out.extend(name.chars().flat_map(char::to_uppercase));
                } else {
                    out.push_str(&name);
                }
            }
        }
        out
    }

    pub fn kept(&self) -> Option<Kept> {
        let m = mods_of(self.mods);
        let plain = (false, false, false, false);
        let shift = (false, true, false, false);
        let ctrl = (true, false, false, false);
        let ctrl_shift = (true, true, false, false);
        match self.sym {
            Keysym::Tab if m == plain || m == shift => Some(Kept::Window),
            Keysym::Escape | Keysym::Menu if m == plain => Some(Kept::Window),
            Keysym::F10 if m == shift => Some(Kept::Window),
            Keysym::a | Keysym::c | Keysym::x | Keysym::v | Keysym::z | Keysym::y if m == ctrl => Some(Kept::Typing),
            Keysym::z if m == ctrl_shift => Some(Kept::Typing),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kept {
    Window,
    Typing,
}

fn said(sym: Keysym) -> Option<&'static str> {
    Some(match sym {
        Keysym::Left => t!("kit-key-left"),
        Keysym::Right => t!("kit-key-right"),
        Keysym::Up => t!("kit-key-up"),
        Keysym::Down => t!("kit-key-down"),
        Keysym::Page_Up => t!("kit-key-page-up"),
        Keysym::Page_Down => t!("kit-key-page-down"),
        Keysym::Home => t!("kit-key-home"),
        Keysym::End => t!("kit-key-end"),
        Keysym::Return => t!("kit-key-enter"),
        Keysym::Escape => t!("kit-key-escape"),
        Keysym::Tab => t!("kit-key-tab"),
        Keysym::space => t!("kit-key-space"),
        Keysym::BackSpace => t!("kit-key-backspace"),
        Keysym::Delete => t!("kit-key-delete"),
        Keysym::Insert => t!("kit-key-insert"),
        Keysym::plus => "+",
        Keysym::minus => "-",
        _ => return None,
    })
}

fn lower(sym: Keysym) -> Keysym {
    match char::from_u32(xkb::keysym_to_utf32(sym)).filter(|c| c.is_alphabetic()) {
        Some(c) => xkb::utf32_to_keysym(c.to_lowercase().next().unwrap_or(c) as u32),
        None => sym,
    }
}

fn mods_of(m: Mods) -> (bool, bool, bool, bool) {
    (m.ctrl, m.shift, m.alt, m.logo)
}

#[derive(Clone, Debug)]
pub struct Shortcuts<A> {
    list: Vec<(Chord, A)>,
}

impl<A> Default for Shortcuts<A> {
    fn default() -> Self {
        Shortcuts { list: Vec::new() }
    }
}

impl<A> Shortcuts<A> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn bind(&mut self, chord: &str, action: A) -> Result<Option<A>, A> {
        let Some(chord) = Chord::parse(chord) else { return Err(action) };
        let old = self.list.iter().position(|(c, _)| *c == chord).map(|i| self.list.remove(i).1);
        self.list.push((chord, action));
        Ok(old)
    }

    pub fn find(&self, input: &Input) -> Option<&A> {
        self.list.iter().find(|(c, _)| c.is(input)).map(|(_, a)| a)
    }

    pub fn all(&self) -> impl Iterator<Item = (&Chord, &A)> {
        self.list.iter().map(|(c, a)| (c, a))
    }

    pub fn chord_of(&self, action: &A) -> Option<Chord>
    where
        A: PartialEq,
    {
        self.list.iter().find(|(_, a)| a == action).map(|(c, _)| *c)
    }

    pub fn clashes(&self) -> Vec<(Chord, &A, Kept)> {
        self.list.iter().filter_map(|(c, a)| c.kept().map(|k| (*c, a, k))).collect()
    }

    pub fn page<M>(&self, says: impl Fn(&A) -> String) -> Node<M> {
        let mut items = vec![page_title(t!("kit-shortcuts"))];
        for (c, a) in &self.list {
            items.push(row(vec![label(says(a)), space(), label(c.text()).key(format!("shortcut-{}", c.text()))]));
            match c.kept() {
                Some(Kept::Window) => items.push(note(t!("kit-shortcut-kept-window")).tone(Tone::Problem)),
                Some(Kept::Typing) => items.push(note(t!("kit-shortcut-kept-typing")).tone(Tone::Problem)),
                None => {}
            }
        }
        column(items)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn press(sym: Keysym, mods: Mods) -> Input {
        Input::Key { sym, text: None, mods, repeat: false }
    }

    #[test]
    fn chords_are_read_matched_and_written() {
        let c = Chord::parse("Ctrl+Shift+C").unwrap();
        assert_eq!(c.text(), "Ctrl+Shift+C");
        let both = Mods { ctrl: true, shift: true, ..Mods::default() };
        assert!(c.is(&press(Keysym::C, both)) && c.is(&press(Keysym::c, both)));
        assert!(!c.is(&press(Keysym::c, Mods { ctrl: true, ..Mods::default() })));
        assert_eq!(Chord::parse("alt+left").unwrap().text(), "Alt+Left");
        assert_eq!(Chord::parse("Ctrl++"), Chord::parse("Ctrl+plus"));
        assert!(Chord::parse("Ctrl++").is_some());
        assert!(Chord::parse("Hyper+X").is_none() && Chord::parse("Ctrl+").is_none() && Chord::parse("Ctrl+Nothing").is_none());
    }

    #[test]
    fn a_held_key_repeats_for_is_but_not_for_pressed() {
        let c = Chord::parse("Ctrl+S").unwrap();
        let ctrl = Mods { ctrl: true, ..Mods::default() };
        let again = Input::Key { sym: Keysym::s, text: None, mods: ctrl, repeat: true };
        assert!(c.is(&again) && !c.pressed(&again) && c.pressed(&press(Keysym::s, ctrl)));
    }

    #[test]
    fn shift_tab_and_shifted_symbols_match_as_the_keyboard_reports_them() {
        let shift = Mods { shift: true, ..Mods::default() };
        assert!(Chord::parse("Shift+Tab").unwrap().is(&press(Keysym::ISO_Left_Tab, shift)));
        assert!(!Chord::parse("Tab").unwrap().is(&press(Keysym::ISO_Left_Tab, shift)));
        let both = Mods { ctrl: true, shift: true, ..Mods::default() };
        assert!(Chord::parse("Ctrl+plus").unwrap().is(&press(Keysym::plus, both)));
        assert!(Chord::parse("Ctrl+question").unwrap().is(&press(Keysym::question, both)));
        assert!(!Chord::parse("Ctrl+plus").unwrap().is(&press(Keysym::plus, Mods { alt: true, ..both })));
    }

    #[test]
    fn a_table_finds_actions_replaces_a_chord_and_lists_them() {
        let mut t = Shortcuts::new();
        assert_eq!(t.bind("Ctrl+S", "save"), Ok(None));
        assert_eq!(t.bind("F5", "reload"), Ok(None));
        assert_eq!(t.bind("ctrl+s", "save as"), Ok(Some("save")));
        assert_eq!(t.bind("Nonsense+Q", "quit"), Err("quit"));
        let ctrl = Mods { ctrl: true, ..Mods::default() };
        assert_eq!(t.find(&press(Keysym::s, ctrl)), Some(&"save as"));
        assert_eq!(t.find(&press(Keysym::F5, Mods::default())), Some(&"reload"));
        assert_eq!(t.find(&press(Keysym::F6, Mods::default())), None);
        assert_eq!(t.chord_of(&"reload").map(|c| c.text()).as_deref(), Some("F5"));
        assert_eq!(t.all().count(), 2);
    }

    #[test]
    fn named_keys_are_written_from_the_language_file() {
        assert_eq!(Chord::parse("Ctrl+Page_Down").unwrap().text(), "Ctrl+Page down");
        assert_eq!(Chord::parse("Alt+Return").unwrap().text(), "Alt+Enter");
        assert_eq!(Chord::parse("Ctrl+plus").unwrap().text(), "Ctrl++");
        assert_eq!(Chord::parse("F5").unwrap().text(), "F5");
    }

    #[test]
    fn chords_the_toolkit_takes_first_are_reported() {
        let mut t = Shortcuts::new();
        for (c, a) in [("Ctrl+C", "copy"), ("Escape", "close"), ("Ctrl+Shift+Z", "redo"), ("Ctrl+Shift+C", "copy path"), ("Alt+Escape", "leave")] {
            assert_eq!(t.bind(c, a), Ok(None));
        }
        let found: Vec<(&str, Kept)> = t.clashes().into_iter().map(|(_, a, k)| (*a, k)).collect();
        assert_eq!(found, vec![("copy", Kept::Typing), ("close", Kept::Window), ("redo", Kept::Typing)]);
        assert_eq!(Chord::parse("Shift+Tab").unwrap().kept(), Some(Kept::Window));
        assert_eq!(Chord::parse("Shift+F10").unwrap().kept(), Some(Kept::Window));
        assert_eq!(Chord::parse("F10").unwrap().kept(), None);
    }

    #[test]
    fn the_page_lists_each_shortcut_and_says_which_clash() {
        use crate::ui::Ui;
        use crate::theme::Theme;
        let mut t = Shortcuts::new();
        let _ = t.bind("F5", "Reload");
        let _ = t.bind("Escape", "Close");
        let mut ui: Ui<()> = Ui::new(t.page(|a| a.to_string()), Theme::new(false, "#fbc711"));
        ui.lay_at(500, 400);
        let words: Vec<String> = ui.access("Shortcuts").nodes.iter().filter_map(|(_, p)| p.label().or(p.value()).map(str::to_string)).collect();
        for w in ["Keyboard shortcuts", "Reload", "F5", "Close", "Escape", "This key is reserved for moving between controls and closing the window."] {
            assert!(words.iter().any(|s| s == w), "{w} missing from {words:?}");
        }
        assert!(!words.iter().any(|s| s.contains("text box")));
    }
}
