//! The ready settings page for a program's own look, in a window.

use super::*;
use super::super::colour::Colour;
use super::super::picker::ColourBox;
use super::super::settings::{look_page, LookChoice, LookEdit, Shade};
use super::super::theme::Theme;

fn page(choice: &LookChoice) -> Ui<LookEdit> {
    let colour = ColourBox::new(Colour::parse("#fbc711").unwrap());
    let mut ui = Ui::new(look_page(choice, &colour, |e| e), Theme::new(true, "#fbc711"));
    ui.size = (720, 700);
    ui.settle();
    ui
}

#[test]
fn the_look_page_offers_the_shade_and_the_highlight_and_a_reader_hears_them() {
    let mut ui = page(&LookChoice::default());
    let told = ui.access("Test");
    let labels: Vec<&str> = told.nodes.iter().filter_map(|(_, n)| n.label()).collect();
    for want in ["Light or dark", "As the desktop", "Hue", "Hex code"] {
        assert!(labels.iter().any(|l| l.contains(want)), "{want} in {labels:?}");
    }
    assert!(!labels.iter().any(|l| l.contains("Use the desktop's")), "the desktop's highlight is already in use");
    let own = LookChoice { shade: Shade::Light, highlight: Colour::parse("#2295f1") };
    let told = page(&own).access("Test");
    assert!(told.nodes.iter().any(|(_, n)| n.label() == Some("Use the desktop's")));
}

#[test]
fn choosing_light_on_the_page_sends_the_change() {
    let mut ui = page(&LookChoice::default());
    let told = ui.access("Test");
    let (id, _) = told.nodes.iter().find(|(_, n)| n.label() == Some("Light")).expect("a Light choice");
    let ask = accesskit::ActionRequest { action: accesskit::Action::Click, target_tree: accesskit::TreeId::ROOT, target_node: *id, data: None };
    assert_eq!(ui.act(&ask).messages, vec![LookEdit::Shade(Shade::Light)]);
}
