//! The spell check hook for text boxes: the program says which words it knows, and the toolkit draws a wavy line under the rest and tells the screen reader.

use std::ops::Range;

use tiny_skia::{PathBuilder, PixmapMut, Stroke};
use unicode_segmentation::UnicodeSegmentation;

use super::colour::Colour;
use super::paint::Painter;
use super::widget::{Node, What};

pub trait Speller {
    fn known(&self, word: &str) -> bool;
}

pub fn misspelt(text: &str, speller: &dyn Speller) -> Vec<Range<usize>> {
    text.unicode_word_indices()
        .filter(|(_, w)| w.chars().all(|c| c.is_alphabetic() || c == '\'' || c == '\u{2019}') && !speller.known(w))
        .map(|(at, w)| at..at + w.len())
        .collect()
}

impl<M> Node<M> {
    pub fn spelling(mut self, speller: &dyn Speller) -> Self {
        if let What::Entry { text, secret: false, misspelt: marks, .. } = &mut self.what {
            *marks = misspelt(text, speller);
        }
        self
    }
}

pub fn squiggle(canvas: &mut PixmapMut, x0: f32, x1: f32, y: f32, thick: f32, c: Colour) {
    let mut p = Painter::default();
    squiggle_on(&mut p, x0, x1, y, thick, c);
    p.play(canvas);
}

pub fn squiggle_on(p: &mut Painter, x0: f32, x1: f32, y: f32, thick: f32, c: Colour) {
    let (half, swing) = (thick * 2.0, thick * 2.0);
    let mut pb = PathBuilder::new();
    pb.move_to(x0, y + swing);
    let mut x = x0;
    let mut up = true;
    while x < x1 {
        let next = (x + half).min(x1);
        pb.line_to(next, if up { y + thick / 2.0 } else { y + swing });
        up = !up;
        x = next;
    }
    if let Some(path) = pb.finish() {
        p.stroke_path(path, Stroke { width: thick, line_join: tiny_skia::LineJoin::Round, ..Stroke::default() }, c);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widget::entry;

    struct Few;

    impl Speller for Few {
        fn known(&self, word: &str) -> bool {
            ["the", "cat", "sat", "don't"].contains(&word.to_lowercase().as_str())
        }
    }

    #[test]
    fn unknown_words_are_found_and_numbers_left_alone() {
        let text = "The catt sat on 42 mats, don't x2";
        let found: Vec<&str> = misspelt(text, &Few).into_iter().map(|r| &text[r]).collect();
        assert_eq!(found, ["catt", "on", "mats"]);
        assert!(misspelt("café naïve", &Few).iter().all(|r| "café naïve".is_char_boundary(r.start) && "café naïve".is_char_boundary(r.end)));
    }

    #[test]
    fn a_text_box_keeps_its_marks_and_a_secret_one_is_never_checked() {
        let n: Node<()> = entry("The catt", "Note", |_| ()).spelling(&Few);
        assert!(matches!(&n.what, What::Entry { misspelt, .. } if *misspelt == [4..8]));
        let s: Node<()> = entry("hunter2 catt", "Password", |_| ()).secret().spelling(&Few);
        assert!(matches!(&s.what, What::Entry { misspelt, .. } if misspelt.is_empty()));
    }

    #[test]
    fn the_line_can_be_found_on_a_typing_box_in_every_look() {
        for (look, t) in crate::theme::tests::every_look() {
            let c = crate::theme::contrast(t.spelling, crate::theme::over(t.field, crate::theme::over(t.panel, t.bg)));
            assert!(c >= 3.0, "{look}: {c}");
        }
    }

    #[test]
    fn the_squiggle_is_a_wavy_line_in_its_colour() {
        let mut pix = tiny_skia::Pixmap::new(40, 10).unwrap();
        squiggle(&mut pix.as_mut(), 2.0, 38.0, 2.0, 1.5, Colour::rgb(0.0, 0.0, 1.0));
        let lit = |x: u32| (0..10).filter(|&y| pix.pixel(x, y).unwrap().alpha() > 100).min();
        let tops: Vec<Option<u32>> = (4..36).map(lit).collect();
        assert!(tops.iter().all(Option::is_some), "unbroken");
        assert!(tops.iter().min() != tops.iter().max(), "it goes up and down");
        assert!(pix.pixel(20, 4).map_or(0, |p| p.blue()) > 0 || pix.pixel(20, 5).map_or(0, |p| p.blue()) > 0);
        assert_eq!(pix.pixels().iter().map(|p| p.red()).max(), Some(0), "never red");
    }
}
