//! The house style's empty-space test, on the toolkit's own layout.

use super::*;
use crate::widget::{Spot, Texts};
use crate::text::Fonts;

pub fn loose_tiles<M>(fonts: &Fonts, tree: &Node<M>, w: f32, h: f32) -> Vec<(Vec<usize>, f32)> {
    let texts = Texts::default();
    let scrolled = HashMap::new();
    let lay = Lay::bare(fonts, &texts, &scrolled);
    *lay.bento.log.borrow_mut() = Some(Vec::new());
    let mut spots: Vec<Spot> = Vec::new();
    lay.place(tree, Rect { x: 0.0, y: 0.0, w, h }, &mut Vec::new(), &mut spots);
    let tiles = lay.bento.log.borrow_mut().take().unwrap_or_default();
    let mut loose = Vec::new();
    for t in tiles {
        let Some(node) = tree.at(&t.path) else { continue };
        let pad = match node.what {
            What::Card(..) => lay.card_padding(node).1,
            _ => 0.0,
        };
        let inside = spots.iter().filter(|s| s.path.len() > t.path.len() && s.path.starts_with(&t.path));
        let Some(bottom) = inside.map(|s| s.rect.y + s.rect.h).reduce(f32::max) else { continue };
        let spare = t.rect.y + t.rect.h - pad - bottom;
        if gives(node) || node.span.is_some_and(|s| s.grows) {
            continue;
        }
        let allowed = if node.span.is_some() { LOOSE.max(GROWN_MOST * t.rect.h) } else { LOOSE };
        if spare > allowed + 0.5 {
            loose.push((t.path, spare));
        }
    }
    loose
}
