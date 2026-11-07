//! The widget gallery that is also the docs: each widget beside the code that makes it, written once so the two cannot differ (`cargo run --example widgets`).

use crate::t;
use crate::widget::{button, card, choice, column, dropdown, entry, heading, label, main_word, note, ring, row, scroll, slider, spinner, switch, tick, Length, Node};

#[derive(Clone, Debug, PartialEq)]
pub enum Msg {
    Pressed,
    Switched(bool),
    Ticked(bool),
    Slid(f32),
    Typed(String),
    Chose(usize),
    Picked(usize),
}

pub struct Entry {
    pub code: &'static str,
    pub node: Node<Msg>,
}

macro_rules! shown {
    ($e:expr) => {
        Entry { code: stringify!($e), node: $e }
    };
}

pub fn entries() -> Vec<Entry> {
    vec![
        shown!(label(t!("kit-showcase-words"))),
        shown!(heading(t!("kit-showcase-heading"))),
        shown!(note(t!("kit-showcase-note"))),
        shown!(button("list-add-symbolic", t!("kit-showcase-add")).on_press(Msg::Pressed)),
        shown!(main_word(t!("kit-showcase-save")).on_press(Msg::Pressed)),
        shown!(switch(true, t!("kit-showcase-wifi"), Msg::Switched)),
        shown!(tick(false, t!("kit-showcase-agree"), Msg::Ticked)),
        shown!(slider(0.4, 0.0, 1.0, 0.1, t!("kit-showcase-volume"), Msg::Slid)),
        shown!(entry("", t!("kit-showcase-name"), Msg::Typed)),
        shown!(choice(vec![t!("kit-showcase-small").into(), t!("kit-showcase-large").into()], 0, t!("kit-showcase-size"), Msg::Chose)),
        shown!(dropdown(vec![t!("kit-showcase-town-first").into(), t!("kit-showcase-town-second").into()], None, t!("kit-showcase-town"), Msg::Picked)),
        shown!(ring(0.6, 48.0)),
        shown!(spinner()),
        shown!(card(label(t!("kit-showcase-in-card")))),
    ]
}

pub fn page() -> Node<Msg> {
    let rows = entries().into_iter().map(|e| row(vec![e.node.width(Length::Fill), note(e.code).width(Length::Fill)]).gap(crate::theme::gaps().huge).width(Length::Fill)).collect();
    scroll("showcase", column(rows).gap(crate::theme::gaps().large).padding(crate::theme::gaps().huge))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::Theme;
    use crate::ui::Ui;

    #[test]
    fn each_widget_s_code_is_the_code_that_made_it() {
        let all = entries();
        let names = ["label", "heading", "note", "button", "main_word", "switch", "tick", "slider", "entry", "choice", "dropdown", "ring", "spinner", "card"];
        assert_eq!(all.len(), names.len());
        let source: String = crate::common::source::code(include_str!("showcase.rs")).split_whitespace().collect();
        for (e, name) in all.iter().zip(names) {
            assert!(e.code.starts_with(&format!("{name}(")), "{}", e.code);
            let code: String = e.code.split_whitespace().collect();
            assert!(source.contains(&format!("shown!({code})")), "{} is not written so in the source", e.code);
        }
    }

    #[test]
    fn the_page_shows_every_widget_named_and_apart() {
        let mut ui = Ui::new(page(), Theme::new(false, "#fbc711"));
        ui.lay_at(900, 700);
        assert_eq!(ui.unnamed_controls(), Vec::<String>::new());
        assert_eq!(ui.squeezed_or_touching(), Vec::<String>::new());
        let mut pix = tiny_skia::Pixmap::new(900, 700).unwrap();
        ui.paint(&mut pix.as_mut(), 1.0);
    }
}
