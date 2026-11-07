//! Vector pictures (SVG) drawn from their shapes at whatever size they are shown, so they stay sharp at any zoom and any screen.

use std::rc::Rc;

use super::paint::Painter;
use super::picture::SvgTree as Tree;
use tiny_skia::{PixmapMut, Transform};

use super::widget::{drawing, Node, Rect};

pub fn draw(canvas: &mut PixmapMut, tree: &Tree, r: Rect) {
    let s = tree.size();
    let k = (r.w / s.width()).min(r.h / s.height());
    if !(k.is_finite() && k > 0.0) {
        return;
    }
    let at = (r.x + (r.w - s.width() * k) / 2.0, r.y + (r.h - s.height() * k) / 2.0);
    resvg::render(tree, Transform::from_row(k, 0.0, 0.0, k, at.0, at.1), canvas);
}

#[allow(dead_code)]
pub(crate) fn draw_on<'a>(p: &mut Painter<'a>, tree: &'a Tree, r: Rect) {
    let Some(at) = tiny_skia::IntRect::from_ltrb(r.x.floor() as i32, r.y.floor() as i32, (r.x + r.w).ceil() as i32, (r.y + r.h).ceil() as i32) else { return };
    p.pixels(at, move |c| draw(c, tree, r));
}

pub fn vector<M>(tree: Rc<Tree>, size: (f32, f32), tip: impl Into<String>) -> Node<M> {
    drawing(size, tip, move |canvas, _| {
        let r = Rect { x: 0.0, y: 0.0, w: canvas.width() as f32, h: canvas.height() as f32 };
        draw(canvas, &tree, r);
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::picture::svg_tree;

    const SQUARE: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10"><circle cx="5" cy="5" r="5" fill="#ff0000"/></svg>"##;

    fn drawn(w: u32, h: u32) -> tiny_skia::Pixmap {
        let tree = svg_tree(SQUARE).unwrap();
        let mut pix = tiny_skia::Pixmap::new(w, h).unwrap();
        draw(&mut pix.as_mut(), &tree, Rect { x: 0.0, y: 0.0, w: w as f32, h: h as f32 });
        pix
    }

    #[test]
    fn a_small_picture_is_drawn_large_from_its_shapes_with_a_sharp_edge() {
        let p = drawn(400, 400);
        assert_eq!(p.pixel(200, 200).unwrap().red(), 255);
        let soft = (0..400).filter(|&x| (1..255).contains(&p.pixel(x, 200).unwrap().alpha())).count();
        assert!(soft <= 2, "an edge drawn from the shape is one pixel soft at most, not stretched: {soft}");
    }

    #[test]
    fn it_keeps_its_shape_centred_in_a_wide_box() {
        let p = drawn(300, 100);
        assert_eq!(p.pixel(150, 50).unwrap().red(), 255);
        assert_eq!(p.pixel(20, 50).unwrap().alpha(), 0, "left of the circle is empty");
    }

    #[test]
    fn recorded_it_plays_as_it_draws() {
        let tree = svg_tree(SQUARE).unwrap();
        let r = Rect { x: 3.4, y: 2.6, w: 30.3, h: 21.7 };
        let mut played = tiny_skia::Pixmap::new(40, 30).unwrap();
        let mut p = Painter::default();
        draw_on(&mut p, &tree, r);
        crate::paint::cpu::play(p.ops, &mut played.as_mut(), &crate::shadow::Shadows::default());
        let mut direct = tiny_skia::Pixmap::new(40, 30).unwrap();
        draw(&mut direct.as_mut(), &tree, r);
        assert!(played.data() == direct.data());
    }

    #[test]
    fn the_widget_tells_the_screen_reader_what_it_shows() {
        let n: Node<()> = vector(Rc::new(svg_tree(SQUARE).unwrap()), (24.0, 24.0), "A red dot");
        assert!(matches!(&n.what, crate::widget::What::Drawing { tip, .. } if tip == "A red dot"));
    }
}
