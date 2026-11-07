//! A main action whose act cannot be undone is red, and no other is.

use super::*;
use super::super::theme::{self as th, drawn, Theme};
use super::super::widget::{self as ww};

fn view(for_good: bool, off: bool) -> Node<()> {
    let delete = ww::main_action("edit-delete-symbolic", "Delete permanently").on_press(()).off(off).key("delete");
    let delete = if for_good { delete.cannot_be_undone() } else { delete };
    ww::column(vec![ww::label("This cannot be undone."), ww::row(vec![ww::space(), delete])]).padding(20.0).gap(12.0)
}

fn fill(for_good: bool, off: bool, light: bool) -> (Colour, Theme) {
    let mut ui = Ui::new(view(for_good, off), Theme::new(light, "#fbc711"));
    ui.lay_at(400, 120);
    let r = ui.spot("delete").unwrap().rect;
    let mut pix = tiny_skia::Pixmap::new(400, 120).unwrap();
    ui.paint(&mut pix.as_mut(), 1.0);
    let px = pix.pixel((r.x + 5.0) as u32, (r.y + r.h / 2.0) as u32).unwrap();
    let c = Colour { r: f32::from(px.red()) / 255.0, g: f32::from(px.green()) / 255.0, b: f32::from(px.blue()) / 255.0, a: 1.0 };
    (c, ui.theme.clone())
}

fn near(a: Colour, b: Colour) -> bool {
    (a.r - b.r).abs() < 0.012 && (a.g - b.g).abs() < 0.012 && (a.b - b.b).abs() < 0.012
}

#[test]
fn a_main_action_that_cannot_be_undone_is_red_and_any_other_is_the_highlight() {
    for light in [false, true] {
        let (plain, t) = fill(false, false, light);
        assert!(near(plain, t.accent), "light {light}: a main action that can be undone is no longer the highlight: {}", plain.css());
        let (red, t) = fill(true, false, light);
        assert!(near(red, t.danger), "light {light}: a main action that cannot be undone is {} and not the sheet's red {}", red.css(), t.danger.css());
        assert!(!near(red, t.accent));
        assert!(th::contrast(th::on(t.danger), t.danger) >= th::WORDS, "light {light}: words on the red");
        let (grey, t) = fill(true, true, light);
        assert!(near(grey, t.control), "light {light}: greyed out it is {}", grey.css());
    }
}

#[test]
fn a_red_main_action_draws_only_pairs_the_table_holds() {
    drawn::none(drawn::in_every_look("a main action that cannot be undone", 2, || {
        drawn::window("as it stands", view(true, false), Theme::chosen(), (400, 120), false);
        drawn::window("greyed out", view(true, true), Theme::chosen(), (400, 120), false);
    }));
}
