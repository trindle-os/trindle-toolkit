//! Tests that a text box's purpose reaches the input method.

use super::*;
use super::super::input::Purpose;
use super::super::widget::{column, entry};

#[test]
fn the_input_method_is_told_what_a_box_is_for() {
    let page = column(vec![
        entry("", "Phone", |_| ()).purpose(Purpose::Phone).key("phone"),
        entry("", "Notes", |_| ()).key("notes"),
    ]);
    let mut ui: Ui<()> = Ui::new(page, Theme::new(false, "#fbc711"));
    ui.lay_at(400, 300);
    ui.keyboard_to("phone");
    assert_eq!(ui.typing().map(|t| t.purpose), Some(Purpose::Phone));
    ui.keyboard_to("notes");
    assert_eq!(ui.typing().map(|t| t.purpose), Some(Purpose::Words));
}
