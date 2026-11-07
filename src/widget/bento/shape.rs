//! The shape check of Trindle's desktop style, on the toolkit's own layout: a page's cards are a puzzle, at most one card wider than three quarters of the page, every card a box from 1:1 to 3:1, the cards ending level so together they make one rectangle, and no line of quiet words longer than a short line.

use super::*;
use crate::widget::{Spot, Texts, Words};
use crate::text::Fonts;

pub const SHORT: usize = 60;

#[derive(Debug, Default, PartialEq)]
pub struct Shape {
    pub cards: usize,
    pub wide: usize,
    pub bars: Vec<String>,
    pub long: Vec<String>,
    pub puzzle: bool,
}

fn long_lines<M>(n: &Node<M>, out: &mut Vec<String>) {
    if let What::Label { text, words: Words::Note, .. } = &n.what {
        if text.chars().count() > SHORT {
            out.push(text.clone());
        }
    }
    n.items().iter().for_each(|i| long_lines(i, out));
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
            long_lines(n, &mut shape.long);
        }
    }
    shape
}
