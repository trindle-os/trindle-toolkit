//! Random keys, composed words and pointer moves from a fixed seed: nothing may panic and the caret, the text and the layout stay sane.

use std::time::Instant;

use crate::editing::doc::Doc;
use crate::editing::editor::{Look, Pane};
use crate::input::{Button, Input, Keysym, Mods};
use crate::theme::Theme;
use crate::ui::Ui;
use crate::widget::{column, dropdown, entry, list, item, row, slider, switch, tick, Length, Node};

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
    fn pick<'a, T>(&mut self, from: &'a [T]) -> &'a T {
        &from[self.below(from.len())]
    }
}

const WORDS: [&str; 10] = ["a", "Z", " ", "\n", "é", "日本", "👩‍👩‍👧", "\u{200f}", "tab\there", ""];
const KEYS: [Keysym; 16] = [Keysym::BackSpace, Keysym::Delete, Keysym::Left, Keysym::Right, Keysym::Up, Keysym::Down, Keysym::Home, Keysym::End, Keysym::Return,
                            Keysym::Tab, Keysym::Page_Up, Keysym::Page_Down, Keysym::space, Keysym::a, Keysym::z, Keysym::Escape];

fn random_key(rng: &mut Rng) -> Input {
    let sym = *rng.pick(&KEYS);
    let mods = Mods { shift: rng.below(3) == 0, ctrl: rng.below(4) == 0, alt: rng.below(8) == 0, logo: false };
    let text = (rng.below(2) == 0).then(|| rng.pick(&WORDS).to_string());
    Input::Key { sym, text, mods, repeat: rng.below(5) == 0 }
}

fn random_composed(rng: &mut Rng) -> Input {
    let caret = (rng.below(2) == 0).then(|| (rng.below(8), rng.below(8)));
    Input::Composed { delete: (rng.below(6) as u32, rng.below(6) as u32), commit: rng.pick(&WORDS).to_string(), preedit: rng.pick(&WORDS).to_string(), caret }
}

#[test]
fn a_text_area_keeps_its_caret_inside_the_text_whatever_is_typed() {
    let mut rng = Rng(0x2545_f491_4f6c_dd1d);
    for round in 0..40 {
        let look = Look { font: None, size: 15.0, wrap: round % 2 == 0, numbers: round % 3 == 0 };
        let mut pane = Pane::new(Doc::new("héllo wörld\nsecond 日本 line\n".into()), look);
        pane.editing = true;
        for step in 0..400 {
            let input = if rng.below(4) == 0 { random_composed(&mut rng) } else { random_key(&mut rng) };
            pane.key(&input);
            let d = &pane.doc;
            let text = d.text();
            assert!(d.caret <= d.len() && d.anchor <= d.len(), "round {round} step {step}: caret {} anchor {} past {} bytes after {input:?}", d.caret, d.anchor, d.len());
            assert!(text.is_char_boundary(d.caret) && text.is_char_boundary(d.anchor), "round {round} step {step}: caret inside a letter after {input:?}");
            assert_eq!(d.line_range(d.lines() - 1).end, d.len(), "round {round} step {step}: the line table lost the end");
        }
        while pane.doc.undo() {}
        assert_eq!(pane.doc.text(), "héllo wörld\nsecond 日本 line\n", "round {round}: undoing everything must give back the first text");
    }
}

fn form() -> Node<()> {
    column(vec![
        entry("John Smith", "Name", |_| ()).on_submit(()),
        entry("hidden", "Password", |_| ()).secret().key("secret"),
        row(vec![switch(true, "On", |_| ()), tick(false, "Agree", |_| ())]).gap(8.0),
        slider(40.0, 0.0, 100.0, 1.0, "Volume", |_| ()).width(Length::Fixed(200.0)),
        dropdown(vec!["Ely".to_string(), "Brandon".to_string(), "Thetford".to_string()], Some(1), "Town", |_| ()),
        list(vec![item("Documents"), item("Pictures"), item("notes.txt")], Some(0), |_| ()),
    ]).gap(8.0).padding(16.0)
}

#[test]
fn a_form_survives_random_keys_pointers_and_composed_words() {
    let (w, h) = (360u32, 480u32);
    let mut rng = Rng(0x9e37_79b9_7f4a_7c15);
    let mut ui: Ui<()> = Ui::new(form(), Theme::new(false, "#fbc711"));
    ui.lay_at(w, h);
    let mut pix = tiny_skia::Pixmap::new(w, h).unwrap();
    let mut now = Instant::now();
    for step in 0..3000 {
        let (x, y) = (rng.below(w as usize + 40) as f32 - 20.0, rng.below(h as usize + 40) as f32 - 20.0);
        let input = match rng.below(9) {
            0 => Input::PointerMove { x, y },
            1 => Input::Press { button: Button::Left, x, y },
            2 => Input::Release { button: Button::Left, x, y },
            3 => Input::Scroll { x: rng.below(5) as f32 - 2.0, y: rng.below(5) as f32 - 2.0, steps: rng.below(2) == 0 },
            4 => Input::Composed { delete: (rng.below(4) as u32, rng.below(4) as u32), commit: rng.pick(&WORDS).to_string(), preedit: rng.pick(&WORDS).to_string(), caret: None },
            5 => Input::Focus(rng.below(4) != 0),
            6 => Input::PointerOut,
            _ => random_key(&mut rng),
        };
        now += std::time::Duration::from_millis(rng.below(400) as u64);
        ui.input(&input, now);
        ui.tick(now);
        if step % 50 == 0 {
            ui.paint(&mut pix.as_mut(), 1.0);
            for (id, r, _) in ui.boxes() {
                assert!(r.x.is_finite() && r.y.is_finite() && r.w.is_finite() && r.h.is_finite() && r.w >= 0.0 && r.h >= 0.0, "step {step}: {id} has a broken box after {input:?}");
            }
        }
    }
}
