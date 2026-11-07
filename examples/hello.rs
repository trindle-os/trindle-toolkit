//! The smallest program on the Trindle Toolkit: one window with a button that counts its presses (`cargo run -p trindle-toolkit --example hello`).

use trindle_toolkit::t;
use trindle_toolkit::ui::{App, Shown};
use trindle_toolkit::widget::{column, label, main_word, Node};
use trindle_toolkit::window::Kind;

struct Hello {
    pressed: u32,
}

#[derive(Clone)]
enum Msg {
    Press,
}

impl App for Hello {
    type Msg = Msg;

    fn update(&mut self, msg: Msg) {
        match msg {
            Msg::Press => self.pressed += 1,
        }
    }

    fn view(&self) -> Node<Msg> {
        let said = match self.pressed {
            0 => t!("kit-example-hello-none").to_string(),
            n => t!("kit-example-hello-pressed", n = n),
        };
        column(vec![label(said), main_word(t!("kit-example-hello-button")).on_press(Msg::Press)]).padding(24.0).gap(16.0).centred()
    }
}

fn main() {
    let kind = Kind::Window { title: t!("kit-example-hello-title").into(), app_id: "org.example.Hello".into() };
    if let Err(why) = Shown::run(kind, (320, 160), Hello { pressed: 0 }) {
        eprintln!("{why}");
        std::process::exit(1);
    }
}
