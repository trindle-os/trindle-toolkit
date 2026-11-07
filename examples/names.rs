//! A list of twenty made-up names that is only read; built, not run on screen, by `cargo build --release --examples`.

use trindle_toolkit::t;
use trindle_toolkit::ui::{App, Shown};
use trindle_toolkit::widget::{column, item, list, page_title, Field, Node};
use trindle_toolkit::window::Kind;

const NAMES: [&str; 20] = [
    "John Smith", "Jane Brown", "Alan Taylor", "Susan Wilson", "David Evans",
    "Mary Thomas", "Peter Roberts", "Helen Walker", "Colin Wright", "Karen Hall",
    "Brian Green", "Alice Wood", "Simon Clarke", "Julie Hughes", "Neil Baker",
    "Sarah King", "Paul Turner", "Emma Hill", "Gary Cooper", "Linda Moore",
];

struct Names;

#[derive(Clone)]
enum Msg {
    Nothing,
}

impl App for Names {
    type Msg = Msg;

    fn update(&mut self, msg: Msg) {
        match msg {
            Msg::Nothing => {}
        }
    }

    fn view(&self) -> Node<Msg> {
        let rows = NAMES.iter().map(|name| item(*name).marked(Field::Text, &[0..1])).collect();
        column(vec![page_title(t!("kit-example-names-title")), list(rows, None, |_| Msg::Nothing)]).padding(24.0).gap(16.0)
    }
}

fn main() {
    let kind = Kind::Window { title: t!("kit-example-names-title").into(), app_id: "org.example.Names".into() };
    if let Err(why) = Shown::run(kind, (360, 480), Names) {
        eprintln!("{why}");
        std::process::exit(1);
    }
}
