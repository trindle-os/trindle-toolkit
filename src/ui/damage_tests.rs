//! A window the GPU draws is drawn again only where it changed: the pointer moving from one button to another changes those two.

use tiny_skia::IntRect;

use super::*;
use super::super::paint::{changes, Seen};
use super::super::theme::Theme;
use super::super::widget::{self as ww};

const W: u32 = 1200;
const H: u32 = 400;

fn changed(ui: &mut Ui<usize>, seen: &mut Seen) -> Option<Vec<IntRect>> {
    ui.ready((W, H), 1.0);
    let mut p = Painter::default();
    ui.record(&mut p, None);
    changes(p, (W, H), seen, &mut None).1
}

fn room(ui: &Ui<usize>, id: &str) -> IntRect {
    let r = ui.spot(id).unwrap().rect;
    let k = PART_ROOM;
    IntRect::from_ltrb((r.x - k).floor() as i32, (r.y - k).floor() as i32, (r.x + r.w + k).ceil() as i32, (r.y + r.h + k).ceil() as i32)
        .and_then(|b| b.intersect(&IntRect::from_xywh(0, 0, W, H).unwrap())).unwrap()
}

#[test]
fn a_pointer_move_from_one_button_to_another_redraws_only_those_two() {
    let buttons = (0..4).map(|i| ww::word_button(format!("Go {i}")).on_press(i).key(format!("b{i}"))).collect();
    let view = ww::column(vec![ww::label("Words that stay as they are"), ww::row(buttons).gap(160.0)]).padding(100.0).gap(12.0);
    let mut ui = Ui::new(view, Theme::new(true, "#fbc711"));
    ui.lay_at(W, H);
    let mut seen = Seen::new();
    assert_eq!(changed(&mut ui, &mut seen), None, "the first frame is drawn whole");
    assert_eq!(changed(&mut ui, &mut seen), Some(vec![]), "nothing changed");
    let middle = |ui: &Ui<usize>, id: &str| {
        let r = ui.spot(id).unwrap().rect;
        (r.x + r.w / 2.0, r.y + r.h / 2.0)
    };
    let (x, y) = middle(&ui, "b0");
    ui.input(&Input::PointerIn { x, y }, Instant::now());
    assert_eq!(changed(&mut ui, &mut seen), Some(vec![room(&ui, "b0")]), "the button pointed at");
    let (x, y) = middle(&ui, "b2");
    ui.input(&Input::PointerMove { x, y }, Instant::now());
    assert_eq!(changed(&mut ui, &mut seen), Some(vec![room(&ui, "b0"), room(&ui, "b2")]), "the one left and the one pointed at");
}
