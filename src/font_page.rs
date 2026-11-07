//! A font chooser as a page in the window: the families found by name, a size, and some words drawn in the chosen font.

use crate::dialog::dialog;
use crate::spin::spin;
use crate::t;
use crate::text::{FontFamily, Style};
use crate::theme;
use crate::widget::{drawing, entry, form_label, item, list, main_action, note, row, Length, Node, Tone};

const SIZES: (f32, f32) = (6.0, 96.0);

#[derive(Clone, Debug, PartialEq)]
pub enum FontStep {
    Search(String),
    Family(String),
    Size(f32),
    Done,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FontPage {
    pub search: String,
    pub family: Option<String>,
    pub size: f32,
    pub problem: Option<String>,
}

impl FontPage {
    pub fn new(family: Option<String>, size: f32) -> FontPage {
        FontPage { search: String::new(), family, size: size.clamp(SIZES.0, SIZES.1), problem: None }
    }

    pub fn shown<'a>(&self, all: &'a [FontFamily]) -> Vec<&'a FontFamily> {
        let wanted = self.search.trim().to_lowercase();
        all.iter().filter(|f| f.name.to_lowercase().contains(&wanted)).collect()
    }

    pub fn update(&mut self, step: FontStep) -> Option<(String, f32)> {
        match step {
            FontStep::Search(s) => self.search = s,
            FontStep::Family(f) => (self.family, self.problem) = (Some(f), None),
            FontStep::Size(s) => self.size = s.clamp(SIZES.0, SIZES.1),
            FontStep::Done => match &self.family {
                Some(f) => return Some((f.clone(), self.size)),
                None => self.problem = Some(t!("kit-fonts-choose-first").to_string()),
            },
        }
        None
    }

    pub fn view<M: Clone + 'static>(&self, families: &[FontFamily], title: impl Into<String>, wrap: impl Fn(FontStep) -> M + Clone + 'static, cancel: M) -> Node<M> {
        let shown = self.shown(families);
        let names: Vec<String> = shown.iter().map(|f| f.name.clone()).collect();
        let chosen = self.family.as_ref().and_then(|c| names.iter().position(|n| n == c));
        let rows = shown.iter().map(|f| {
            let one = item(f.name.clone());
            if f.monospace { one.detail(t!("kit-fonts-monospace")) } else { one }
        }).collect();
        let (search, pick, open, size, done) = (wrap.clone(), wrap.clone(), wrap.clone(), wrap.clone(), wrap);
        let mut page = dialog(title, t!("kit-fonts-lead"), cancel)
            .holds(entry(self.search.clone(), t!("kit-fonts-search"), move |s| search(FontStep::Search(s))).key("fonts-search"))
            .holds(list(rows, chosen, move |i| pick(FontStep::Family(names[i].clone()))).on_open(move |_| open(FontStep::Done))
                .called(t!("kit-fonts-list")).height(Length::Fill).key("fonts-list"));
        if shown.is_empty() {
            page = page.holds(note(t!("kit-fonts-none")));
        }
        if let Some(problem) = &self.problem {
            page = page.holds(note(problem.clone()).tone(Tone::Problem));
        }
        let words = t!("kit-fonts-sample").to_string();
        let (family, points) = (self.family.clone(), self.size);
        let tip = t!("kit-fonts-sample-tip", family = family.clone().unwrap_or_default(), words = words.clone());
        let sample = drawing((1.0, (points * 1.6).ceil()), tip, move |canvas, pen| {
            let style = Style { family: family.clone(), ..Style::new((points * pen.scale).round()) };
            let laid = pen.fonts.layout(&words, &style, Some(canvas.width() as f32));
            pen.fonts.draw(canvas, &laid, 0.0, ((canvas.height() as f32 - laid.height) / 2.0).max(0.0).round(), pen.theme.fg);
        }).width(Length::Fill);
        page.holds(row(vec![
                form_label(t!("kit-fonts-size")),
                spin(self.size, SIZES.0, SIZES.1, 1.0, t!("kit-fonts-size"), move |s| size(FontStep::Size(s))),
            ]).gap(theme::gaps().medium))
            .holds(sample)
            .answer(main_action("", t!("kit-fonts-use")).on_press(done(FontStep::Done)))
            .node()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::Theme;
    use crate::ui::Ui;

    fn families() -> Vec<FontFamily> {
        [("Noto Sans", false), ("Noto Sans Mono", true), ("Serif Example", false)].into_iter()
            .map(|(name, monospace)| FontFamily { name: name.into(), faces: vec![(400, false)], monospace, colour: false }).collect()
    }

    #[test]
    fn a_search_narrows_the_list_and_an_answer_needs_a_font() {
        let all = families();
        let mut page = FontPage::new(None, 200.0);
        assert_eq!(page.size, 96.0);
        page.update(FontStep::Search(" MONO ".into()));
        assert_eq!(page.shown(&all).iter().map(|f| f.name.as_str()).collect::<Vec<_>>(), ["Noto Sans Mono"]);
        assert_eq!(page.update(FontStep::Done), None);
        assert!(page.problem.is_some());
        page.update(FontStep::Family("Noto Sans Mono".into()));
        page.update(FontStep::Size(14.0));
        assert_eq!((page.update(FontStep::Done), page.problem.clone()), (Some(("Noto Sans Mono".into(), 14.0)), None));
    }

    #[test]
    fn the_page_is_named_draws_its_sample_and_sends_the_family_chosen() {
        use crate::input::{Input, Keysym};
        use std::time::Instant;
        let page = FontPage::new(Some("Noto Sans".into()), 12.0);
        let mut ui = Ui::new(page.view(&families(), "Choose a font", |s| s, FontStep::Done), Theme::new(false, "#fbc711"));
        ui.lay_at(800, 600);
        assert!(ui.unnamed_controls().is_empty());
        let mut pix = tiny_skia::Pixmap::new(800, 600).unwrap();
        ui.paint(&mut pix.as_mut(), 1.0);
        ui.keyboard_to("fonts-list");
        let key = |sym| Input::Key { sym, text: None, mods: Default::default(), repeat: false };
        assert_eq!(ui.input(&key(Keysym::Down), Instant::now()).messages, vec![FontStep::Family("Noto Sans Mono".into())]);
        assert_eq!(ui.input(&key(Keysym::Return), Instant::now()).messages, vec![FontStep::Done]);
    }
}
