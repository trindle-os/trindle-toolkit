//! A settings page the way every Trindle settings page is laid out: a title and a lead sentence, then each group as a section name over a card of rows, each row the setting's name and help on the left and its control at the right; and a program's own look, kept in its own settings file, with a ready page to change it.

use std::path::{Path, PathBuf};

use super::colour::Colour;
use super::common::files::config_home;
use super::common::plumbing::Ini;
use super::common::style::DEFAULT_ACCENT;
use super::desktop;
use super::picker::{ColourBox, Edit};
use super::theme::Theme;
use crate::t;
use super::widget::{card, column, form_label, lead, note, one_of, page_title, row, scroll, section, word_button, Length, Node};

pub fn setting<M>(name: impl Into<String>, help: impl Into<String>, control: Node<M>) -> Node<M> {
    let (name, help) = (name.into(), help.into());
    let mut words = vec![form_label(name)];
    if !help.is_empty() {
        words.push(note(help));
    }
    row(vec![column(words).gap(crate::theme::gaps().tiny).width(Length::Fill), control.width(Length::Shrink)]).gap(crate::theme::gaps().large)
}

pub fn setting_page<M>(key: impl Into<String>, title: impl Into<String>, lead_in: impl Into<String>, groups: Vec<(String, Vec<Node<M>>)>) -> Node<M> {
    let mut items = vec![page_title(title), lead(lead_in)];
    for (name, rows) in groups.into_iter().filter(|(_, rows)| !rows.is_empty()) {
        items.push(section(name));
        items.push(card(column(rows).gap(crate::theme::gaps().large)));
    }
    scroll(key, column(items).gap(crate::theme::gaps().large).padding(crate::theme::gaps().large))
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Shade {
    #[default]
    Desktop,
    Dark,
    Light,
}

impl Shade {
    pub const ALL: [Shade; 3] = [Shade::Desktop, Shade::Dark, Shade::Light];

    fn word(self) -> &'static str {
        match self {
            Shade::Desktop => "desktop",
            Shade::Dark => "dark",
            Shade::Light => "light",
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct LookChoice {
    pub shade: Shade,
    pub highlight: Option<Colour>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum LookEdit {
    Shade(Shade),
    Highlight(Edit),
    DesktopHighlight,
}

impl LookChoice {
    pub fn file(app_id: &str) -> PathBuf {
        config_home().join(app_id).join("settings.ini")
    }

    pub fn from_ini(ini: &Ini) -> LookChoice {
        let shade = Shade::ALL.into_iter().find(|s| ini.get("Look", "Shade") == Some(s.word())).unwrap_or_default();
        LookChoice { shade, highlight: ini.get("Look", "Highlight").and_then(Colour::parse).map(|c| Colour { a: 1.0, ..c }) }
    }

    pub fn to_ini(&self, ini: &mut Ini) {
        ini.set("Look", "Shade", self.shade.word());
        ini.set("Look", "Highlight", &self.highlight.map(Colour::css).unwrap_or_default());
    }

    pub fn load(path: &Path) -> LookChoice {
        Ini::read(path).map(|ini| LookChoice::from_ini(&ini)).unwrap_or_default()
    }

    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        let mut ini = Ini::read(path)?;
        self.to_ini(&mut ini);
        if let Some(folder) = path.parent() {
            std::fs::create_dir_all(folder)?;
        }
        ini.write(path)
    }

    pub fn theme(&self, desk: &desktop::Look) -> Theme {
        let light = match self.shade {
            Shade::Desktop => desk.light.unwrap_or(false),
            Shade::Dark => false,
            Shade::Light => true,
        };
        let accent = self.highlight.or(desk.accent).map_or_else(|| DEFAULT_ACCENT.to_string(), Colour::css);
        let mut theme = Theme::new(light, &accent);
        if desk.more_contrast {
            theme.raise_contrast();
        }
        theme
    }

    pub fn apply(&mut self, edit: LookEdit, colour: &mut ColourBox) {
        match edit {
            LookEdit::Shade(s) => self.shade = s,
            LookEdit::Highlight(e) => {
                colour.update(e);
                self.highlight = Some(colour.colour());
            }
            LookEdit::DesktopHighlight => self.highlight = None,
        }
    }
}

pub fn look_page<M: 'static>(choice: &LookChoice, colour: &ColourBox, change: impl Fn(LookEdit) -> M + Clone + 'static) -> Node<M> {
    let shades = [t!("kit-look-desktop"), t!("kit-look-dark"), t!("kit-look-light")].map(str::to_string).to_vec();
    let chosen = Shade::ALL.iter().position(|s| *s == choice.shade);
    let pick = change.clone();
    let mut highlight = vec![colour.view(move |e| pick(LookEdit::Highlight(e)))];
    if choice.highlight.is_some() {
        highlight.push(word_button(t!("kit-look-highlight-desktop")).on_press(change(LookEdit::DesktopHighlight)));
    }
    let shade = change;
    setting_page("look", t!("kit-look-title"), t!("kit-look-lead"), vec![(t!("kit-look-group").to_string(), vec![
        setting(t!("kit-look-shade"), t!("kit-look-shade-help"), one_of(shades, chosen, t!("kit-look-shade"), move |i| shade(LookEdit::Shade(Shade::ALL[i])))),
        setting(t!("kit-look-highlight"), t!("kit-look-highlight-help"), column(highlight).gap(crate::theme::gaps().medium)),
    ])])
}

pub fn numerals_row<M: 'static>(chosen: crate::lang::numerals::Numerals, change: impl Fn(crate::lang::numerals::Numerals) -> M + 'static) -> Node<M> {
    use crate::lang::numerals::Numerals;
    let words = Numerals::ALL.map(|n| n.label().to_string()).to_vec();
    let at = Numerals::ALL.iter().position(|n| *n == chosen);
    setting(t!("kit-numerals-title"), t!("kit-numerals-help"), one_of(words, at, t!("kit-numerals-title"), move |i| change(Numerals::ALL[i])))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_file_that_cannot_be_read_is_not_overwritten_by_a_save() {
        let folder = std::env::temp_dir().join(format!("trindle-look-bad-{}", std::process::id()));
        std::fs::create_dir_all(&folder).unwrap();
        let path = folder.join("settings.ini");
        std::fs::write(&path, [0xff, 0xfe]).unwrap();
        assert!(LookChoice::default().save(&path).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), [0xff, 0xfe]);
        std::fs::remove_dir_all(&folder).unwrap();
    }

    #[test]
    fn a_look_is_kept_in_the_programs_own_file_beside_its_other_settings() {
        let folder = std::env::temp_dir().join(format!("trindle-look-{}", std::process::id()));
        let path = folder.join("org.example.Notes").join("settings.ini");
        assert_eq!(LookChoice::load(&path), LookChoice::default(), "nothing saved is the desktop's look");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, "[Window]\nWidth=800\n").unwrap();
        let choice = LookChoice { shade: Shade::Light, highlight: Colour::parse("#2295f1") };
        choice.save(&path).unwrap();
        assert_eq!(LookChoice::load(&path), choice);
        assert_eq!(Ini::read(&path).unwrap().get("Window", "Width"), Some("800"), "the rest of the file is kept");
        std::fs::remove_dir_all(&folder).unwrap();
        assert!(LookChoice::file("org.example.Notes").ends_with("org.example.Notes/settings.ini"));
    }

    #[test]
    fn the_programs_choice_wins_and_the_desktop_fills_the_rest() {
        let desk = desktop::Look { light: Some(true), accent: Colour::parse("#e8207e"), ..desktop::Look::default() };
        assert_eq!(LookChoice::default().theme(&desk), Theme::new(true, "#e8207e"));
        let dark = LookChoice { shade: Shade::Dark, highlight: Colour::parse("#4bae4f") }.theme(&desk);
        assert_eq!(dark, Theme::new(false, "#4bae4f"));
        assert_eq!(LookChoice::default().theme(&desktop::Look::default()), Theme::new(false, DEFAULT_ACCENT));
    }

    #[test]
    fn the_desktops_high_contrast_holds_words_to_seven_to_one() {
        use crate::theme::{contrast, HIGH_WORDS};
        for light in [false, true] {
            let desk = desktop::Look { light: Some(light), more_contrast: true, ..desktop::Look::default() };
            let t = LookChoice::default().theme(&desk);
            assert!(contrast(t.muted, t.panel) >= HIGH_WORDS, "light {light}: {:.2}", contrast(t.muted, t.panel));
        }
    }

    #[test]
    fn edits_from_the_page_change_the_choice() {
        let mut choice = LookChoice::default();
        let mut colour = ColourBox::new(Colour::parse(DEFAULT_ACCENT).unwrap());
        choice.apply(LookEdit::Shade(Shade::Light), &mut colour);
        choice.apply(LookEdit::Highlight(Edit::Hex("#2295f1".into())), &mut colour);
        assert_eq!((choice.shade, choice.highlight.map(Colour::css)), (Shade::Light, Some("#2295f1".to_string())));
        choice.apply(LookEdit::DesktopHighlight, &mut colour);
        assert_eq!(choice.highlight, None);
    }
}
