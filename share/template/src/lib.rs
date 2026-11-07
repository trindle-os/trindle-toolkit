//! The program's window: a line of words and a button that counts its presses. Made by trindle-new.

use trindle_toolkit::host::{self, Host};
use trindle_toolkit::t;
use trindle_toolkit::ui::{App, Shown};
use trindle_toolkit::widget::{column, label, main_word, Node};
use trindle_toolkit::window::Kind;

pub struct Window {
    pub pressed: u32,
}

#[derive(Clone)]
pub enum Msg {
    Press,
}

impl App for Window {
    type Msg = Msg;

    fn update(&mut self, msg: Msg) {
        match msg {
            Msg::Press => self.pressed += 1,
        }
    }

    fn view(&self) -> Node<Msg> {
        column(vec![label(t!("app-pressed", n = self.pressed)), main_word(t!("app-button")).on_press(Msg::Press)]).padding(24.0).gap(16.0).centred()
    }
}

pub struct Words;

impl Host for Words {
    fn words(&self) -> &'static [(&'static str, &'static str)] {
        &[("app.ftl", include_str!("../share/lang/en/app.ftl"))]
    }
}

pub fn run(app_id: &str) -> Result<(), String> {
    host::set(&Words);
    let kind = Kind::Window { title: t!("app-title").into(), app_id: app_id.into() };
    Shown::run(kind, (320, 160), Window { pressed: 0 })
}

#[cfg(test)]
mod tests {
    use super::*;
    use trindle_toolkit::lang::{self, Bundle, Catalogue};
    use trindle_toolkit::testing::Harness;

    fn with_words<R>(during: impl FnOnce() -> R) -> R {
        host::with(&Words, || lang::with_catalogue(Catalogue::new(Bundle::default(), lang::english()), during))
    }

    #[test]
    fn the_button_counts_its_presses() {
        with_words(|| {
            let mut h = Harness::new(Window { pressed: 0 }, (320, 160));
            assert!(h.text().contains(&"Not pressed yet".to_string()), "{:?}", h.text());
            assert!(h.click("Press me"));
            assert_eq!(h.app().pressed, 1);
            assert!(h.text().contains(&"Pressed once".to_string()), "{:?}", h.text());
            assert!(h.ui().unnamed_controls().is_empty());
        });
    }
}
