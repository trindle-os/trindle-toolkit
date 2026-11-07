//! Nothing touches its neighbour: a row of things that cannot be squeezed goes onto more lines when it lacks the room.

use super::*;
use super::super::theme::Theme;
use super::super::widget::{self as ww, Length};

fn swatches(n: usize) -> Node<()> {
    ww::column(vec![
        ww::label("Highlight"),
        ww::row((0..n).map(|i| ww::button("", format!("Colour number {i}"))).collect()).gap(8.0).width(Length::Shrink),
    ]).padding(16.0)
}

fn buttons(ui: &Ui<()>) -> Vec<Rect> {
    ui.spots.iter().filter(|s| matches!(ui.tree.at(&s.path).map(|n| &n.what), Some(What::Button { .. }))).map(|s| s.rect).collect()
}

#[test]
fn a_row_with_room_is_one_line() {
    let mut ui = Ui::new(swatches(10), Theme::new(true, "#fbc711"));
    ui.lay_at(900, 400);
    let at = buttons(&ui);
    assert_eq!(at.len(), 10);
    assert!(at.iter().all(|r| r.y == at[0].y), "a row with room for all its buttons was broken into lines");
    assert!(ui.squeezed_or_touching().is_empty());
}

#[test]
fn a_row_without_the_room_goes_onto_more_lines_and_squeezes_nothing() {
    for labels in [Labels::Brief, Labels::Always] {
        for width in [520u32, 400, 300, 200] {
            let mut ui = Ui::new(ww::column(vec![swatches(19), ww::label("After")]), Theme::new(true, "#fbc711"));
            ui.set_labels(labels);
            ui.lay_at(width, 900);
            let wrong = ui.squeezed_or_touching();
            assert!(wrong.is_empty(), "at {width}: {wrong:?}");
            let at = buttons(&ui);
            assert_eq!(at.len(), 19);
            assert!(at.iter().all(|r| r.x >= 0.0 && r.x + r.w <= width as f32 + 0.5), "at {width} a button was put outside the window");
            let mut lines: Vec<(f32, usize)> = Vec::new();
            for r in &at {
                let middle = r.y + r.h / 2.0;
                match lines.last_mut() {
                    Some((y, n)) if (*y - middle).abs() <= 1.0 => *n += 1,
                    _ => lines.push((middle, 1)),
                }
            }
            assert!(lines.len() > 1, "at {width} nineteen buttons were kept on one line");
            let (most, least) = (lines.iter().map(|l| l.1).max().unwrap(), lines.iter().map(|l| l.1).min().unwrap());
            assert!(most - least <= 1, "at {width} the lines are not as full as each other: {lines:?}");
            assert!(least > 1 || most <= 2, "at {width} one button was left on a line of its own: {lines:?}");
            let after = ui.spots.iter().rfind(|s| matches!(ui.tree.at(&s.path).map(|n| &n.what), Some(What::Label { .. }))).unwrap().rect;
            let bottom = at.iter().map(|r| r.y + r.h).fold(0.0, f32::max);
            assert!(after.y >= bottom, "at {width} the words after the row are drawn over its last line");
        }
    }
}

#[test]
fn a_row_with_words_or_room_to_spare_is_laid_out_as_before() {
    let view = || -> Node<()> { ww::row(vec![ww::label("Name").width(Length::Fill), ww::button("", "One"), ww::button("", "Two")]).gap(8.0) };
    let mut ui = Ui::new(view(), Theme::new(true, "#fbc711"));
    ui.lay_at(300, 100);
    let at = buttons(&ui);
    assert!(at[0].y == at[1].y && at[1].x + at[1].w <= 300.5 && at[1].x + at[1].w >= 299.0, "the buttons after a row's words are no longer at its far end");
    let spaced = || -> Node<()> { ww::row(vec![ww::button("", "One"), ww::space(), ww::button("", "Two")]).gap(8.0) };
    let mut ui = Ui::new(spaced(), Theme::new(true, "#fbc711"));
    ui.lay_at(300, 100);
    let at = buttons(&ui);
    assert!(at[0].x < 1.0 && at[1].x + at[1].w >= 299.0, "a row's own space no longer pushes its buttons apart");
}
