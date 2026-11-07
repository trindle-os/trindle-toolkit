//! Drives a program's window with no desktop, for the program's own tests: click a widget by its name, type, read the words it shows, take a picture. See docs/guides/testing.md.

use std::path::Path;
use std::time::Instant;

use accesskit::{Node as Part, Role, TreeUpdate};

use crate::input::{Button, Input, Keysym, Mods};
use crate::theme::Theme;
use crate::ui::{App, Ui};

pub struct Harness<A: App> {
    app: A,
    ui: Ui<A::Msg>,
    size: (u32, u32),
}

impl<A: App> Harness<A> where A::Msg: Clone {
    pub fn new(app: A, (w, h): (u32, u32)) -> Harness<A> {
        crate::text::built_in_mono_only();
        let mut ui = Ui::new(app.view(), Theme::new(false, crate::common::style::DEFAULT_ACCENT));
        ui.lay_at(w, h);
        Harness { app, ui, size: (w, h) }
    }

    pub fn app(&self) -> &A {
        &self.app
    }

    pub fn ui(&mut self) -> &mut Ui<A::Msg> {
        &mut self.ui
    }

    pub fn send(&mut self, input: &Input) {
        let reply = self.ui.input(input, Instant::now());
        for m in reply.messages {
            self.app.update(m);
        }
        self.ui.set_view(self.app.view());
        self.ui.lay_at(self.size.0, self.size.1);
    }

    fn parts(&mut self) -> TreeUpdate {
        self.ui.access("")
    }

    pub fn click(&mut self, name: &str) -> bool {
        let found = self.parts().nodes.into_iter().find(|(_, p)| said(p).as_deref() == Some(name) && p.role() != Role::Window);
        let Some(b) = found.and_then(|(_, p)| p.bounds()) else { return false };
        let (x, y) = (((b.x0 + b.x1) / 2.0) as f32, ((b.y0 + b.y1) / 2.0) as f32);
        self.send(&Input::PointerMove { x, y });
        self.send(&Input::Press { button: Button::Left, x, y });
        self.send(&Input::Release { button: Button::Left, x, y });
        true
    }

    pub fn key(&mut self, sym: Keysym, mods: Mods) {
        self.send(&Input::Key { sym, text: None, mods, repeat: false });
    }

    pub fn type_text(&mut self, text: &str) {
        for c in text.chars() {
            self.send(&Input::Key { sym: Keysym::NoSymbol, text: Some(c.to_string()), mods: Mods::default(), repeat: false });
        }
    }

    pub fn text(&mut self) -> Vec<String> {
        self.parts().nodes.iter().filter(|(_, p)| p.role() != Role::Window).flat_map(|(_, p)| [p.label(), p.value()])
            .flatten().filter(|s| !s.trim().is_empty()).map(str::to_string).collect()
    }

    pub fn picture(&mut self, scale: f32) -> tiny_skia::Pixmap {
        let (w, h) = ((self.size.0 as f32 * scale).round() as u32, (self.size.1 as f32 * scale).round() as u32);
        let mut pix = tiny_skia::Pixmap::new(w.max(1), h.max(1)).expect("a window has a size");
        self.ui.paint(&mut pix.as_mut(), scale);
        pix
    }

    pub fn save_picture(&mut self, scale: f32, path: &Path) -> image::ImageResult<()> {
        let pix = self.picture(scale);
        let rgba: Vec<u8> = pix.pixels().iter().flat_map(|p| {
            let c = p.demultiply();
            [c.red(), c.green(), c.blue(), c.alpha()]
        }).collect();
        image::save_buffer(path, &rgba, pix.width(), pix.height(), image::ExtendedColorType::Rgba8)
    }
}

fn said(p: &Part) -> Option<String> {
    p.label().or(p.value()).map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widget::{column, entry, label, main_word, Node};

    struct Counter {
        pressed: u32,
        name: String,
    }

    #[derive(Clone)]
    enum Msg {
        Press,
        Name(String),
    }

    impl App for Counter {
        type Msg = Msg;
        fn update(&mut self, msg: Msg) {
            match msg {
                Msg::Press => self.pressed += 1,
                Msg::Name(n) => self.name = n,
            }
        }
        fn view(&self) -> Node<Msg> {
            column(vec![label(format!("Pressed {} times", self.pressed)), main_word("Add one").on_press(Msg::Press),
                        entry(self.name.clone(), "Your name", Msg::Name)]).padding(16.0).gap(12.0)
        }
    }

    fn harness() -> Harness<Counter> {
        Harness::new(Counter { pressed: 0, name: String::new() }, (320, 200))
    }

    #[test]
    fn a_click_by_name_reaches_the_program_and_its_words_are_read_back() {
        let mut h = harness();
        assert!(h.text().contains(&"Pressed 0 times".to_string()), "{:?}", h.text());
        assert!(h.click("Add one"));
        assert!(h.click("Add one"));
        assert_eq!(h.app().pressed, 2);
        assert!(h.text().contains(&"Pressed 2 times".to_string()), "{:?}", h.text());
        assert!(!h.click("Nothing called this"));
    }

    #[test]
    fn typing_goes_where_the_keyboard_is() {
        let mut h = harness();
        assert!(h.click("Your name"));
        h.type_text("John Smith");
        assert_eq!(h.app().name, "John Smith");
        h.key(Keysym::BackSpace, Mods::default());
        assert_eq!(h.app().name, "John Smit");
    }

    #[test]
    fn a_picture_is_the_window_at_its_scale_and_saves_as_a_png() {
        let mut h = harness();
        let pix = h.picture(2.0);
        assert_eq!((pix.width(), pix.height()), (640, 400));
        assert!(pix.pixels().iter().any(|p| *p != pix.pixels()[0]), "something is drawn");
        let at = std::env::temp_dir().join(format!("trindle-harness-{}.png", std::process::id()));
        h.save_picture(1.0, &at).unwrap();
        let back = image::open(&at).unwrap();
        std::fs::remove_file(&at).unwrap();
        assert_eq!((back.width(), back.height()), (320, 200));
    }
}
