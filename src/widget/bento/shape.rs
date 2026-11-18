//! The shape check of Trindle's desktop style, on the toolkit's own layout: a page's cards are a puzzle, at most one card wider than three quarters of the page, every card a box from 1:1 to 3:1, the cards ending level so together they make one rectangle, and no line of quiet words that needs more than two lines in the room it has.

use super::*;
use crate::widget::{Spot, Texts, Words};
use crate::text::Fonts;

pub const LINES: usize = 2;

#[derive(Debug, Default, PartialEq)]
pub struct Shape {
    pub cards: usize,
    pub wide: usize,
    pub bars: Vec<String>,
    pub long: Vec<String>,
    pub puzzle: bool,
}

fn long_lines<M>(lay: &Lay, n: &Node<M>, path: &mut Vec<usize>, spots: &[Spot], out: &mut Vec<String>) {
    if let What::Label { text, words: Words::Note, .. } = &n.what {
        let room = spots.iter().find(|s| s.path == *path).map(|s| s.rect.w);
        let lines = room.map_or(1, |w| lay.texts.get(lay.fonts, text, &Words::Note.style(lay.scale), Some(w)).lines.len());
        if lines > LINES {
            out.push(text.clone());
        }
    }
    for (i, item) in n.items().iter().enumerate() {
        path.push(i);
        long_lines(lay, item, path, spots, out);
        path.pop();
    }
}

pub fn check<M>(fonts: &Fonts, tree: &Node<M>, w: f32, h: f32) -> Shape {
    let texts = Texts::default();
    let scrolled = HashMap::new();
    let lay = Lay::bare(fonts, &texts, &scrolled);
    *lay.bento.log.borrow_mut() = Some(Vec::new());
    let mut spots: Vec<Spot> = Vec::new();
    lay.place(tree, Rect { x: 0.0, y: 0.0, w, h }, &mut Vec::new(), &mut spots);
    let tiles = lay.bento.log.borrow_mut().take().unwrap_or_default();
    let mut shape = Shape::default();
    let mut grids: HashMap<Vec<usize>, Vec<Placed>> = HashMap::new();
    for t in tiles {
        grids.entry(t.path[..t.path.len() - 1].to_vec()).or_default().push(t);
    }
    for (path, tiles) in grids {
        let Some(grid) = tree.at(&path) else { continue };
        let What::Grid { puzzle, .. } = grid.what else { continue };
        if !tiles.iter().any(|t| tree.at(&t.path).is_some_and(|n| n.beside)) {
            continue;
        }
        shape.puzzle |= puzzle;
        let left = tiles.iter().map(|t| t.rect.x).fold(f32::MAX, f32::min);
        let right = tiles.iter().map(|t| t.rect.x + t.rect.w).fold(0.0, f32::max);
        for t in &tiles {
            let Some(n) = tree.at(&t.path) else { continue };
            shape.cards += 1;
            if t.rect.w > 0.75 * (right - left) && tiles.len() > 1 {
                shape.wide += 1;
            }
            if t.rect.w > 3.05 * t.rect.h || t.rect.h > 3.05 * t.rect.w {
                shape.bars.push(format!("{:?} {}x{}", t.path, t.rect.w.round(), t.rect.h.round()));
            }
            long_lines(&lay, n, &mut t.path.clone(), &spots, &mut shape.long);
        }
    }
    shape
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widget::{bento, card, column, heading, note};

    fn page(text: &str) -> Node<()> {
        let b = crate::theme::bento();
        let mk = |n: Node<()>| card(column(vec![heading("Card"), n]).gap(16.0)).height(Length::Fill).beside().span(b.card.0, b.card.1, b.words);
        bento(b.words, b.gap, vec![mk(note(text.to_string())), mk(note("Short")), mk(note("Short"))])
    }

    #[test]
    fn a_quiet_line_is_long_by_its_laid_out_width_in_every_language() {
        let fonts = Fonts::new();
        crate::lang::each::each_language(|tag, long| {
            let words = &long[0];
            let wide = check(&fonts, &page(words), 4000.0, 800.0);
            assert!(wide.long.is_empty(), "{tag}: {words:?} fits two lines on a wide page: {:?}", wide.long);
            let narrow = check(&fonts, &page(&format!("{words} {words} {words} {words}")), 700.0, 800.0);
            assert_eq!(narrow.long.len(), 1, "{tag}: four times the words take more than two lines on a narrow page");
            assert!(check(&fonts, &page("Hi"), 700.0, 800.0).long.is_empty(), "{tag}");
        });
    }
}
