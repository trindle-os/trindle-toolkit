//! A text box of five lines that saves as you type, the saved words shown back under it; built, not run on screen, by `cargo build --release --examples`.

use trindle_toolkit::t;
use trindle_toolkit::ui::{App, Shown};
use trindle_toolkit::widget::{column, entry, form_label, label, Node};
use trindle_toolkit::window::Kind;

struct Notes {
    typing: String,
}

#[derive(Clone)]
enum Msg {
    Typed(String),
}

impl App for Notes {
    type Msg = Msg;

    fn update(&mut self, msg: Msg) {
        match msg {
            Msg::Typed(text) => self.typing = text,
        }
    }

    fn view(&self) -> Node<Msg> {
        let said = if self.typing.is_empty() { t!("kit-example-notes-none").to_string() } else { t!("kit-example-notes-saved", words = self.typing.as_str()) };
        column(vec![form_label(t!("kit-example-notes-label")), entry(self.typing.clone(), t!("kit-example-notes-hint"), Msg::Typed).lines(5), label(said)])
        .padding(24.0)
        .gap(16.0)
    }
}

fn main() {
    let kind = Kind::Window { title: t!("kit-example-notes-title").into(), app_id: "org.example.Notes".into() };
    if let Err(why) = Shown::run(kind, (360, 300), Notes { typing: String::new() }) {
        eprintln!("{why}");
        std::process::exit(1);
    }
}
