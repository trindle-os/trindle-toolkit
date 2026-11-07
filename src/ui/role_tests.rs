//! Tests of a program's own widget told to a screen reader with the role and value it gives.

use accesskit::{Action, ActionRequest, NodeId, Role as Told, Toggled, TreeId};

use super::*;
use super::super::role::Role;
use super::super::widget::{column, drawing};

#[derive(Clone, Debug, PartialEq)]
enum Msg {
    Up,
    Down,
    Flip,
}

fn stars(role: Role) -> Node<Msg> {
    drawing((100.0, 24.0), "Rating", |_, _| {}).role(role).value("3 of 5 stars")
        .on_keys(|input| match input {
            Input::Key { sym: Keysym::Up, .. } => Some(Msg::Up),
            Input::Key { sym: Keysym::Down, .. } => Some(Msg::Down),
            _ => None,
        })
}

fn ui_of(tree: Node<Msg>) -> Ui<Msg> {
    let mut ui = Ui::new(column(vec![tree]), Theme::new(false, "#fbc711"));
    ui.size = (300, 200);
    ui.settle();
    ui
}

fn part(ui: &mut Ui<Msg>, name: &str) -> (NodeId, accesskit::Node) {
    ui.access("T").nodes.into_iter().find(|(_, p)| p.label() == Some(name)).expect("a part with that name")
}

#[test]
fn a_drawing_is_told_with_the_role_and_value_its_program_gives() {
    let (value, min, max) = (3.0, 0.0, 5.0);
    for (role, told) in [(Role::Slider { value, min, max }, Told::Slider), (Role::SpinButton { value, min, max }, Told::SpinButton), (Role::Meter { value, min, max }, Told::Meter)] {
        let mut ui = ui_of(stars(role));
        let (_, p) = part(&mut ui, "Rating");
        assert_eq!(p.role(), told);
        assert_eq!(p.value(), Some("3 of 5 stars"));
        assert_eq!((p.numeric_value(), p.max_numeric_value()), (Some(3.0), Some(5.0)));
        assert!(ui.unnamed_controls().is_empty());
    }
    let mut ui = ui_of(drawing((24.0, 24.0), "Bold", |_, _| {}).pressable(Msg::Flip).role(Role::Switch(true)));
    let (id, p) = part(&mut ui, "Bold");
    assert_eq!((p.role(), p.toggled()), (Told::Switch, Some(Toggled::True)));
    let reply = ui.act(&ActionRequest { action: Action::Click, target_tree: TreeId::ROOT, target_node: id, data: None });
    assert_eq!(reply.messages, vec![Msg::Flip]);
}

#[test]
fn the_screen_reader_steps_a_slider_or_spin_button_through_its_keys() {
    let mut ui = ui_of(stars(Role::SpinButton { value: 3.0, min: 0.0, max: 5.0 }));
    let (id, p) = part(&mut ui, "Rating");
    assert!(p.supports_action(Action::Increment) && p.supports_action(Action::Decrement));
    let ask = |action| ActionRequest { action, target_tree: TreeId::ROOT, target_node: id, data: None };
    assert_eq!(ui.act(&ask(Action::Increment)).messages, vec![Msg::Up]);
    assert_eq!(ui.act(&ask(Action::Decrement)).messages, vec![Msg::Down]);
    let mut ui = ui_of(stars(Role::Meter { value: 3.0, min: 0.0, max: 5.0 }));
    assert!(!part(&mut ui, "Rating").1.supports_action(Action::Increment));
}
