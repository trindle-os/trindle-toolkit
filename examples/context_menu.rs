//! A card with a context menu: a right click, a held finger or the Menu key on it offers Pin and Remove (`cargo run -p trindle-toolkit --example context_menu`).

use trindle_toolkit::t;
use trindle_toolkit::ui::{App, Shown};
use trindle_toolkit::widget::{card, column, label, main_word, MenuRow, Node};
use trindle_toolkit::window::Kind;

struct Card {
    pinned: bool,
    removed: bool,
}

#[derive(Clone)]
enum Msg {
    Pin(bool),
    Remove,
    Back,
}

impl App for Card {
    type Msg = Msg;

    fn update(&mut self, msg: Msg) {
        match msg {
            Msg::Pin(on) => self.pinned = on,
            Msg::Remove => self.removed = true,
            Msg::Back => self.removed = false,
        }
    }

    fn view(&self) -> Node<Msg> {
        if self.removed {
            return column(vec![label(t!("kit-example-context-removed")), main_word(t!("kit-example-context-back")).on_press(Msg::Back)]).padding(24.0).gap(16.0);
        }
        let pinned = self.pinned;
        let said = if pinned { t!("kit-example-context-pinned") } else { t!("kit-example-context-unpinned") };
        let note = card(column(vec![label(t!("kit-example-context-note")), label(said)]).gap(8.0)).on_press(Msg::Pin(!pinned)).context_menu(move || vec![
            MenuRow::send("starred-symbolic", t!("kit-example-context-pin"), Msg::Pin(!pinned)).checked(pinned),
            MenuRow::send("user-trash-symbolic", t!("kit-example-context-remove"), Msg::Remove).rule(),
        ]);
        column(vec![label(t!("kit-example-context-how")), note]).padding(24.0).gap(16.0)
    }
}

fn main() {
    let kind = Kind::Window { title: t!("kit-example-context-title").into(), app_id: "org.example.ContextMenu".into() };
    if let Err(why) = Shown::run(kind, (360, 240), Card { pinned: false, removed: false }) {
        eprintln!("{why}");
        std::process::exit(1);
    }
}
