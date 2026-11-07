//! A grid of cells painted as a terminal does it: Ctrl+Shift+C copies, Ctrl+Shift+V pastes, Ctrl+G rings, the wheel scrolls back (`cargo run -p trindle-toolkit --example cells`).

use trindle_toolkit::t;
use trindle_toolkit::grid::cells;
use trindle_toolkit::input::{Input, Keysym};
use trindle_toolkit::text::Style;
use trindle_toolkit::ui::{App, Shown};
use trindle_toolkit::widget::{drawing, Length, Node};
use trindle_toolkit::window::Kind;

const SIZE: f32 = 16.0;

struct Cells {
    lines: Vec<String>,
    back: f32,
    copy: Option<String>,
    paste: bool,
    bell: bool,
}

#[derive(Clone)]
enum Msg {
    Type(String),
    Enter,
    Backspace,
    Copy,
    Paste,
    Pasted(String),
    Bell,
    Scroll(f32),
}

impl App for Cells {
    type Msg = Msg;

    fn update(&mut self, msg: Msg) {
        match msg {
            Msg::Type(t) => self.lines.last_mut().unwrap().push_str(&t),
            Msg::Enter => self.lines.push(String::new()),
            Msg::Backspace => {
                self.lines.last_mut().unwrap().pop();
            }
            Msg::Copy => self.copy = Some(self.lines.join("\n")),
            Msg::Paste => self.paste = true,
            Msg::Pasted(text) => {
                for (i, line) in text.lines().enumerate() {
                    if i > 0 {
                        self.lines.push(String::new());
                    }
                    self.lines.last_mut().unwrap().push_str(line);
                }
            }
            Msg::Bell => self.bell = true,
            Msg::Scroll(n) => self.back = (self.back - n).clamp(0.0, self.lines.len().saturating_sub(1) as f32),
        }
    }

    fn view(&self) -> Node<Msg> {
        let (lines, back) = (self.lines.clone(), self.back.floor() as usize);
        drawing((480.0, 240.0), "Cells", move |canvas, pen| {
            let style = Style::monospace(SIZE * pen.scale);
            let (cell, tall) = pen.fonts.cell_size(&style);
            let rows = (canvas.height() as f32 / tall) as usize;
            let end = lines.len().saturating_sub(back);
            for (row, line) in lines[end.saturating_sub(rows)..end].iter().enumerate() {
                let laid = pen.fonts.layout(line, &style, None).snapped(line, cell);
                pen.fonts.draw(canvas, &laid, 0.0, row as f32 * tall, pen.theme.fg);
            }
        })
        .on_keys(|input| match input {
            Input::Key { sym, mods, .. } if mods.ctrl && mods.shift && (*sym == Keysym::C || *sym == Keysym::c) => Some(Msg::Copy),
            Input::Key { sym, mods, .. } if mods.ctrl && mods.shift && (*sym == Keysym::V || *sym == Keysym::v) => Some(Msg::Paste),
            Input::Key { sym, mods, .. } if mods.ctrl && (*sym == Keysym::G || *sym == Keysym::g) => Some(Msg::Bell),
            Input::Key { sym, .. } if *sym == Keysym::Return => Some(Msg::Enter),
            Input::Key { sym, .. } if *sym == Keysym::BackSpace => Some(Msg::Backspace),
            Input::Key { text: Some(t), mods, .. } if !mods.ctrl && cells(t) > 0 => Some(Msg::Type(t.clone())),
            Input::Composed { commit, .. } if !commit.is_empty() => Some(Msg::Type(commit.clone())),
            _ => None,
        })
        .on_scroll(Msg::Scroll)
        .first()
        .width(Length::Fill)
        .height(Length::Fill)
    }

    fn to_clipboard(&mut self) -> Option<String> {
        self.copy.take()
    }

    fn wants_paste(&mut self) -> bool {
        std::mem::take(&mut self.paste)
    }

    fn pasted(&self, text: String) -> Option<Msg> {
        Some(Msg::Pasted(text))
    }

    fn wants_bell(&mut self) -> bool {
        std::mem::take(&mut self.bell)
    }

    fn window_title(&self) -> Option<String> {
        Some(self.lines.first().filter(|l| !l.is_empty()).cloned().unwrap_or_else(|| t!("kit-example-cells-title").into()))
    }
}

fn main() {
    let kind = Kind::Window { title: t!("kit-example-cells-title").into(), app_id: "org.example.Cells".into() };
    let app = Cells { lines: vec![String::new()], back: 0.0, copy: None, paste: false, bell: false };
    if let Err(why) = Shown::run(kind, (480, 240), app) {
        eprintln!("{why}");
        std::process::exit(1);
    }
}
