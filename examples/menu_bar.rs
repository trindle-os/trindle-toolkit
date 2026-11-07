//! A menu bar across the top of a window: submenus, ticks, radio rows and keys (`cargo run -p trindle-toolkit --example menu_bar`).

use trindle_toolkit::menubar::{BarDone, BarMsg, MenuBar};
use trindle_toolkit::popup::{row, Row};
use trindle_toolkit::t;
use trindle_toolkit::ui::{App, Shown};
use trindle_toolkit::widget::{column, label, note, Node};
use trindle_toolkit::window::Kind;

struct Bars {
    bar: MenuBar,
    wrap: bool,
    size: usize,
    done: String,
}

#[derive(Clone)]
enum Msg {
    Bar(BarMsg),
}

fn sizes() -> [String; 3] {
    [t!("kit-example-bar-small").into(), t!("kit-example-bar-medium").into(), t!("kit-example-bar-large").into()]
}

fn menus(wrap: bool, size: usize) -> Vec<(String, Vec<Row>)> {
    let sizes = sizes().into_iter().enumerate().map(|(i, words)| row("", words).radio(i == size)).collect();
    vec![
        (t!("kit-example-bar-file").into(), vec![
            row("document-new-symbolic", t!("kit-example-bar-new")).keys("Ctrl+N"),
            row("document-open-symbolic", t!("kit-example-bar-open")).keys("Ctrl+O"),
            row("window-close-symbolic", t!("kit-example-bar-quit")).keys("Ctrl+Q").rule(),
        ]),
        (t!("kit-example-bar-view").into(), vec![
            row("", t!("kit-example-bar-wrap")).check(wrap),
            row("", t!("kit-example-bar-size")).sub(sizes, 0.0).rule(),
        ]),
    ]
}

impl Bars {
    fn fresh(&self) -> MenuBar {
        let mut bar = MenuBar::new(menus(self.wrap, self.size));
        bar.lit = self.bar.lit;
        bar.open = self.bar.open.clone();
        bar
    }
}

impl App for Bars {
    type Msg = Msg;

    fn update(&mut self, msg: Msg) {
        let Msg::Bar(m) = msg;
        let what = match self.bar.update(m) {
            BarDone::Picked(0, r) => Some([t!("kit-example-bar-new"), t!("kit-example-bar-open"), t!("kit-example-bar-quit")][r].to_string()),
            BarDone::Picked(1, 0) => {
                self.wrap = !self.wrap;
                Some(t!("kit-example-bar-wrap").to_string())
            }
            BarDone::PickedIn(1, 1, s) => {
                self.size = s;
                Some(sizes()[s].clone())
            }
            _ => None,
        };
        if let Some(what) = what {
            self.done = t!("kit-example-bar-chose", what = what.as_str()).to_string();
        }
        self.bar = self.fresh();
    }

    fn view(&self) -> Node<Msg> {
        let page = column(vec![label(t!("kit-example-bar-how")), note(self.done.clone())]).padding(24.0).gap(12.0);
        self.fresh().window(page, Msg::Bar)
    }
}

fn main() {
    let kind = Kind::Window { title: t!("kit-example-bar-title").into(), app_id: "org.example.MenuBar".into() };
    let bars = Bars { bar: MenuBar::new(menus(false, 1)), wrap: false, size: 1, done: String::new() };
    if let Err(why) = Shown::run(kind, (420, 260), bars) {
        eprintln!("{why}");
        std::process::exit(1);
    }
}
