//! The wavy line under words a spell checker does not know, as a text box paints it.

use super::*;
use super::super::spelling::Speller;
use super::super::widget::{column, entry};

struct Knows;

impl Speller for Knows {
    fn known(&self, w: &str) -> bool {
        w == "good"
    }
}

fn waved(view: Node<()>) -> usize {
    let mut ui = Ui::new(column(vec![view.key("note")]).padding(20.0), Theme::new(false, "#fbc711"));
    ui.size = (400, 80);
    let mut pix = tiny_skia::Pixmap::new(400, 80).unwrap();
    ui.paint(&mut pix.as_mut(), 1.0);
    let r = ui.spot("note").unwrap().rect;
    let want = ui.theme.spelling;
    let near = |p: tiny_skia::PremultipliedColorU8| {
        let c = p.demultiply();
        (c.blue() as f32 - want.b * 255.0).abs() < 40.0 && (c.green() as f32 - want.g * 255.0).abs() < 40.0 && c.red() < 120
    };
    (r.x as u32..(r.x + r.w) as u32).flat_map(|x| (r.y as u32..(r.y + r.h) as u32).map(move |y| (x, y)))
        .filter(|&(x, y)| near(pix.pixel(x, y).unwrap())).count()
}

#[test]
fn an_unknown_word_is_waved_under_and_a_known_one_is_not() {
    assert!(waved(entry("good wrod", "Note", |_| ()).spelling(&Knows)) > 6);
    assert_eq!(waved(entry("good good", "Note", |_| ()).spelling(&Knows)), 0);
    assert_eq!(waved(entry("good wrod", "Note", |_| ())), 0, "no checker, no line");
}

#[test]
fn the_text_area_waves_under_unknown_words_and_tells_the_reader() {
    use super::super::editing::area::TextArea;
    use super::super::editing::editor::Look;
    let area = TextArea::new("good wrod\ngood", Look { font: None, size: 14.0, wrap: true, numbers: false });
    let told = |area: &TextArea| {
        let mut ui = Ui::new(column(vec![area.node("Notes", 60.0, |_| ()).key("notes")]), Theme::new(false, "#fbc711"));
        ui.size = (400, 80);
        ui.settle();
        ui.access("T").nodes.into_iter().find(|(_, p)| p.label() == Some("Notes")).map(|(_, p)| p.invalid())
    };
    assert_eq!(waved(area.node("Notes", 60.0, |_| ())), 0, "no checker, no line");
    assert_eq!(told(&area), Some(None));
    area.spelling(Some(std::rc::Rc::new(Knows)));
    assert!(waved(area.node("Notes", 60.0, |_| ())) > 6);
    assert_eq!(told(&area), Some(Some(accesskit::Invalid::Spelling)));
    area.set_text("good good");
    assert_eq!(waved(area.node("Notes", 60.0, |_| ())), 0);
    assert_eq!(told(&area), Some(None));
    area.set_text("wrod");
    area.spelling(None);
    assert_eq!(waved(area.node("Notes", 60.0, |_| ())), 0, "the checker taken away");
}
