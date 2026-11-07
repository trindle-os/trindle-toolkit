# Painter

A `Painter` records a drawing as a list of shapes (`fill`, `round`, `line`, `fill_path`, `stroke_path`, clips and layers) and then plays it onto a canvas with `play`. A drawing's paint closure makes one, records on it and plays it; the toolkit's own paint functions each have an `_on` version, such as `Scene::paint_on`, that records on a `Painter` you pass, so a frame can be built as one list. Everything on a `Painter` is in pixels, so multiply points by `pen.scale`.

![Five rounded columns above a line](../images/widgets/painter.png)

**The screen reader hears:** the painter says nothing itself; the `drawing` it is used in is an image named by its tip.

For the drawing around it, see [drawing](drawing.md).

<!-- picture width 320 -->
```rust
use trindle_toolkit::widget::{column, drawing, label, Node, Pen, Rect};
use trindle_toolkit::Painter;

fn columns_on(p: &mut Painter, pen: &Pen, values: &[f32]) {
    let k = pen.scale;
    for (i, v) in values.iter().enumerate() {
        let x = (8.0 + i as f32 * 24.0) * k;
        let (h, floor) = (v * 40.0 * k, 52.0 * k);
        p.round(Rect { x, y: floor - h, w: 16.0 * k, h }, [4.0 * k; 4], pen.theme.accent);
    }
    p.line((4.0 * k, 54.0 * k), (124.0 * k, 54.0 * k), 2.0 * k, pen.theme.well);
}

fn view() -> Node<()> {
    let values = [0.4, 0.7, 0.5, 1.0, 0.8];
    column(vec![
        label("Visits"),
        drawing((128.0, 60.0), "Visits, five columns", move |canvas, pen| {
            let mut p = Painter::default();
            columns_on(&mut p, pen, &values);
            p.play(canvas);
        }),
    ]).gap(12.0).padding(24.0)
}
# let _ = view();
```
