//! The widget gallery that is also the docs: every widget beside the code that makes it (`cargo run --example widgets`).

use trindle_toolkit::showcase::{page, Msg};
use trindle_toolkit::t;
use trindle_toolkit::ui::{App, Shown};
use trindle_toolkit::widget::Node;
use trindle_toolkit::window::Kind;

struct Widgets;

impl App for Widgets {
    type Msg = Msg;

    fn update(&mut self, _msg: Msg) {}

    fn view(&self) -> Node<Msg> {
        page()
    }
}

fn main() {
    let kind = Kind::Window { title: t!("kit-showcase-title").into(), app_id: "org.example.Widgets".into() };
    if let Err(why) = Shown::run(kind, (900, 700), Widgets) {
        eprintln!("{why}");
        std::process::exit(1);
    }
}
