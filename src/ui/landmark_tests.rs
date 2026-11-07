//! Tests of landmarks and of progress told as it changes.

use accesskit::{Live, Role};

use super::*;
use super::super::widget::{button, column, label, ring, row, Landmark};

fn told(tree: Node<()>) -> Vec<(Role, Option<String>, Option<Live>)> {
    let mut ui = Ui::new(tree, Theme::new(false, "#fbc711"));
    ui.lay_at(600, 400);
    ui.access("Notes").nodes.iter().map(|(_, p)| (p.role(), p.label().map(str::to_string), p.live())).collect()
}

#[test]
fn landmarks_are_told_with_their_roles_and_names() {
    let page = column(vec![
        row(vec![label("Notes")]).landmark(Landmark::Top),
        row(vec![
            column(vec![button("go-home-symbolic", "Home").on_press(())]).landmark(Landmark::Navigation).called("Places"),
            column(vec![label("Shopping list")]).landmark(Landmark::Main),
            column(vec![label("Details")]).landmark(Landmark::Beside),
        ]),
        row(vec![label("Saved")]).landmark(Landmark::Foot),
    ]);
    let parts = told(page);
    let has = |role: Role, name: Option<&str>| parts.iter().any(|(r, n, _)| *r == role && n.as_deref() == name);
    assert!(has(Role::Navigation, Some("Places")), "{parts:?}");
    for role in [Role::Banner, Role::Main, Role::Complementary, Role::ContentInfo] {
        assert!(parts.iter().any(|(r, ..)| *r == role), "{role:?} missing");
    }
    let regions = told(column(vec![label("Today")]).landmark(Landmark::Region).called("Today's jobs"));
    assert!(regions.iter().any(|(r, n, _)| *r == Role::Region && n.as_deref() == Some("Today's jobs")));
}

#[test]
fn a_ring_told_politely_is_progress_spoken_as_it_moves() {
    let parts = told(column(vec![ring(0.4, 40.0).polite()]));
    assert!(parts.iter().any(|(r, _, l)| *r == Role::ProgressIndicator && *l == Some(Live::Polite)));
}
