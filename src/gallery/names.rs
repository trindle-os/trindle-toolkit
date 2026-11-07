//! Every widget the gallery shows that the keyboard can reach has a name a screen reader speaks.

use accesskit::Action;

use super::{gallery, SIZE};
use crate::theme::Theme;
use crate::ui::Ui;

#[test]
fn every_focusable_widget_in_the_gallery_has_a_name() {
    for dark in [false, true] {
        let mut ui = Ui::new(gallery(0), Theme::new(dark, "#fbc711"));
        ui.lay_at(SIZE.0, SIZE.1);
        let tree = ui.access("Every control");
        let focusable: Vec<_> = tree.nodes.iter().filter(|(_, p)| p.supports_action(Action::Focus)).collect();
        assert!(focusable.len() >= 15, "only {} focusable parts found", focusable.len());
        let unnamed: Vec<String> = focusable.iter()
            .filter(|(_, p)| ![p.label(), p.value()].into_iter().flatten().any(|n| !n.trim().is_empty()))
            .map(|(id, p)| format!("{id:?} {:?}", p.role())).collect();
        assert!(unnamed.is_empty(), "focusable with no name: {unnamed:?}");
        assert!(ui.unnamed_controls().is_empty());
    }
}

#[test]
fn every_list_in_the_gallery_has_a_name() {
    let mut ui = Ui::new(gallery(0), Theme::new(true, "#fbc711"));
    ui.lay_at(SIZE.0, SIZE.1);
    let lists = ui.access("Every control").nodes.iter().filter(|(_, p)| p.role() == accesskit::Role::ListBox).count();
    assert!(lists >= 1, "the gallery shows no list");
    assert_eq!(ui.unnamed_lists(), Vec::<String>::new());
}

#[test]
fn a_list_with_no_name_is_reported_and_naming_it_clears_it() {
    use crate::widget::{item, list};
    let rows = || vec![item("One"), item("Two")];
    let mut bare = Ui::new(list(rows(), None, |_| ()), Theme::new(true, "#fbc711"));
    bare.lay_at(300, 200);
    assert_eq!(bare.unnamed_lists().len(), 1);
    let mut named = Ui::new(list(rows(), None, |_| ()).called("Numbers"), Theme::new(true, "#fbc711"));
    named.lay_at(300, 200);
    assert!(named.unnamed_lists().is_empty());
}
