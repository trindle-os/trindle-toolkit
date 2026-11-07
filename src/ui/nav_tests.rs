//! Tests of moving between pages (nav.rs) in a real window tree.

use super::*;
use super::shared_tests::{click, key, middle};
use super::super::nav;
use super::super::theme::Theme;
use super::super::widget::{self as ww, item, Length};

#[derive(Clone, Debug, PartialEq)]
enum Msg {
    Page(usize),
}

fn ui_of(tree: Node<Msg>, size: (u32, u32)) -> Ui<Msg> {
    let mut ui = Ui::new(tree, Theme::new(true, "#fbc711"));
    ui.size = size;
    ui.settle();
    ui
}

fn found(ui: &Ui<Msg>, is: impl Fn(&What<Msg>) -> bool) -> Spot {
    ui.spots.iter().find(|s| ui.tree.at(&s.path).is_some_and(|n| is(&n.what))).cloned().expect("laid out")
}

fn ctrl(sym: Keysym, shift: bool) -> Input {
    Input::Key { sym, text: None, mods: Mods { ctrl: true, shift, ..Mods::default() }, repeat: false }
}

fn names() -> Vec<String> {
    vec!["General".into(), "Sound".into(), "Display".into()]
}

#[test]
fn a_stacks_strip_picks_a_page_and_is_read_as_its_pages() {
    let mut ui = ui_of(nav::stack(names(), 0, Msg::Page, ww::label("General page")).padding(10.0), (400, 300));
    let strip = found(&ui, |w| matches!(w, What::Choice { .. }));
    let r = ui.part_rect(&strip.id, 2).unwrap();
    let (x, y) = middle(r);
    assert_eq!(click(&mut ui, x, y), vec![Msg::Page(2)]);
    let told = ui.access("Test");
    assert!(told.nodes.iter().any(|(_, n)| n.label() == Some("Pages")));
    assert!(told.nodes.iter().any(|(_, n)| n.label() == Some("General page") || n.value() == Some("General page")));
}

#[test]
fn a_side_switcher_is_the_sheets_width_and_its_rows_pick_pages() {
    let pages = names().into_iter().map(|n| item(n).icon("document-open-symbolic")).collect();
    let mut ui = ui_of(nav::stack_beside(pages, 1, Msg::Page, ww::label("Sound page")), (600, 300));
    let side = found(&ui, |w| matches!(w, What::Card(..)));
    assert_eq!(side.rect.w, theme::sheet().sidebar_width);
    let rows = found(&ui, |w| matches!(w, What::List { .. }));
    let r = ui.part_rect(&rows.id, 2).unwrap();
    let (x, y) = middle(r);
    assert_eq!(click(&mut ui, x, y), vec![Msg::Page(2)]);
}

#[test]
fn ctrl_page_keys_and_ctrl_tab_go_round_the_pages() {
    assert_eq!(nav::page_key(&ctrl(Keysym::Page_Down, false), 2, 3), Some(0));
    assert_eq!(nav::page_key(&ctrl(Keysym::Page_Up, false), 0, 3), Some(2));
    assert_eq!(nav::page_key(&ctrl(Keysym::Tab, false), 0, 3), Some(1));
    assert_eq!(nav::page_key(&ctrl(Keysym::ISO_Left_Tab, true), 1, 3), Some(0));
    assert_eq!(nav::page_key(&key(Keysym::Page_Down), 0, 3), None);
    assert_eq!(nav::page_key(&ctrl(Keysym::Page_Down, false), 0, 0), None);
}

#[test]
fn history_goes_on_and_back_and_never_leaves_the_first_page() {
    let mut h = nav::History::new("Home");
    assert!(!h.can_go_back() && !h.back());
    h.push("Wi-Fi");
    h.push("Network");
    assert_eq!((h.current(), h.trail()), (&"Network", &["Home", "Wi-Fi", "Network"][..]));
    assert!(h.back());
    assert_eq!(*h.current(), "Wi-Fi");
    h.push("Saved");
    h.home();
    assert_eq!((*h.current(), h.can_go_back()), ("Home", false));
}

#[test]
fn a_navigation_views_back_button_and_escape_go_back_and_the_first_page_has_none() {
    let view = |back| nav::nav_view("Wi-Fi", back, ww::label("Networks"));
    let mut ui = ui_of(view(Some(Msg::Page(0))), (400, 300));
    let back = found(&ui, |w| matches!(w, What::Button { .. }));
    let (x, y) = middle(back.rect);
    assert_eq!(click(&mut ui, x, y), vec![Msg::Page(0)]);
    assert_eq!(ui.input(&key(Keysym::Escape), Instant::now()).messages, vec![Msg::Page(0)]);
    let told = ui.access("Test");
    assert!(told.nodes.iter().any(|(_, n)| n.role() == accesskit::Role::Button && n.label() == Some("Back")));
    let first = ui_of(view(None), (400, 300));
    assert!(!first.spots.iter().any(|s| matches!(first.tree.at(&s.path).map(|n| &n.what), Some(What::Button { .. }))));
}

fn split(width: f32, showing: bool) -> Node<Msg> {
    let side = ww::list(names().into_iter().map(item).collect(), Some(0), Msg::Page);
    nav::split_view(width, side, ww::label("Sound page"), "Sound", showing, Msg::Page(99))
}

fn words(ui: &mut Ui<Msg>) -> Vec<String> {
    let told = ui.access("Test");
    told.nodes.iter().filter_map(|(_, n)| n.label().or(n.value()).map(str::to_string)).collect()
}

#[test]
fn a_split_view_shows_both_sides_when_wide_and_one_with_a_way_back_when_narrow() {
    let narrow = theme::sheet().split_narrow;
    assert!(nav::is_narrow(narrow - 1.0) && !nav::is_narrow(narrow));
    let mut wide = ui_of(split(800.0, false), (800, 400));
    let said = words(&mut wide);
    assert!(said.iter().any(|w| w == "Sound page") && said.iter().any(|w| w == "General"), "{said:?}");
    assert_eq!(found(&wide, |w| matches!(w, What::Card(..))).rect.w, theme::sheet().sidebar_width);
    let mut side = ui_of(split(400.0, false), (400, 400));
    let said = words(&mut side);
    assert!(said.iter().any(|w| w == "General") && !said.iter().any(|w| w == "Sound page"), "{said:?}");
    let mut content = ui_of(split(400.0, true), (400, 400));
    let said = words(&mut content);
    assert!(said.iter().any(|w| w == "Sound page") && !said.iter().any(|w| w == "General"), "{said:?}");
    let back = found(&content, |w| matches!(w, What::Button { .. }));
    let (x, y) = middle(back.rect);
    assert_eq!(click(&mut content, x, y), vec![Msg::Page(99)]);
}

fn page_and_drawer(open: bool, edge: nav::Edge, width: f32) -> Node<Msg> {
    let page = ww::column(vec![ww::button("list-add-symbolic", "Add").on_press(Msg::Page(1))]);
    nav::drawer(page, ww::label("Details"), open, edge, width, Msg::Page(0))
}

#[test]
fn a_drawer_lies_over_the_page_at_its_edge_and_shuts_by_button_or_escape() {
    let k = theme::sheet();
    let mut shut = ui_of(page_and_drawer(false, nav::Edge::End, 800.0), (800, 400));
    assert!(!words(&mut shut).iter().any(|w| w == "Details"));
    let mut ui = ui_of(page_and_drawer(true, nav::Edge::End, 800.0), (800, 400));
    let panel = ui.spots.iter().rfind(|s| matches!(ui.tree.at(&s.path).map(|n| &n.what), Some(What::Card(..)))).cloned().unwrap();
    assert_eq!((panel.rect.w, panel.rect.x + panel.rect.w, panel.rect.h), (k.drawer_width, 800.0, 400.0));
    let add = ui.spots.iter().find(|s| matches!(ui.tree.at(&s.path).map(|n| &n.what), Some(What::Button { tip, .. }) if tip == "Add")).cloned().unwrap();
    let (x, y) = middle(add.rect);
    assert_eq!(click(&mut ui, x, y), vec![Msg::Page(1)]);
    let close = ui.spots.iter().find(|s| matches!(ui.tree.at(&s.path).map(|n| &n.what), Some(What::Button { tip, .. }) if tip == "Close the panel")).cloned().unwrap();
    assert!(close.rect.x >= panel.rect.x);
    let (x, y) = middle(close.rect);
    assert_eq!(click(&mut ui, x, y), vec![Msg::Page(0)]);
    assert_eq!(ui.input(&key(Keysym::Escape), Instant::now()).messages, vec![Msg::Page(0)]);
    let start = ui_of(page_and_drawer(true, nav::Edge::Start, 300.0), (300, 400));
    let panel = start.spots.iter().rfind(|s| matches!(start.tree.at(&s.path).map(|n| &n.what), Some(What::Card(..)))).cloned().unwrap();
    assert_eq!((panel.rect.x, panel.rect.w), (0.0, 300.0 - k.drawer_room));
}

#[test]
fn a_flow_box_keeps_its_items_own_widths_and_wraps_them_like_words() {
    let chip = |w: f32| ww::drawing((w, 30.0), "Tag", |_, _| {});
    let gap = theme::sheet().gap;
    let ui = ui_of(ww::column(vec![ww::flow(vec![chip(150.0), chip(100.0), chip(120.0), chip(60.0)])]), (300, 300));
    let rects: Vec<Rect> = ui.spots.iter().filter(|s| matches!(ui.tree.at(&s.path).map(|n| &n.what), Some(What::Drawing { .. }))).map(|s| s.rect).collect();
    assert_eq!(rects.iter().map(|r| r.w).collect::<Vec<_>>(), [150.0, 100.0, 120.0, 60.0]);
    assert_eq!((rects[0].x, rects[0].y, rects[1].x, rects[1].y), (0.0, 0.0, 150.0 + gap, 0.0));
    assert_eq!((rects[2].x, rects[2].y), (0.0, 30.0 + gap), "120 more does not fit after 250 and a gap");
    assert_eq!((rects[3].x, rects[3].y), (120.0 + gap, 30.0 + gap));
    assert_eq!(ui.natural_height(300.0), 60.0 + gap);
}

#[test]
fn an_icon_grid_paints_chosen_tiles_and_is_read_tile_by_tile() {
    use super::super::icongrid::{paint_icons, IconGrid};
    let size = (400.0, 300.0);
    let shot = |grid: IconGrid| {
        let mut ui = ui_of(ww::drawing(size, "Files", move |canvas, pen| {
            paint_icons(canvas, pen, &grid, None, true, |i| ("folder-symbolic".into(), format!("File {i}")));
        }), (400, 300));
        let mut pix = tiny_skia::Pixmap::new(400, 300).unwrap();
        ui.paint(&mut pix.as_mut(), 1.0);
        pix
    };
    let mut grid = IconGrid::new(6);
    let plain = shot(grid.clone());
    grid.selection.press(1, Mods::default());
    let chosen = shot(grid.clone());
    let r = grid.tile_rect(1, size.0);
    let at = |p: &tiny_skia::Pixmap| p.pixel(r.x as u32 + 6, r.y as u32 + 6).unwrap();
    assert_ne!(at(&plain), at(&chosen));
    let other = grid.tile_rect(3, size.0);
    let away = |p: &tiny_skia::Pixmap| p.pixel(other.x as u32 + 6, other.y as u32 + 6).unwrap();
    assert_eq!(away(&plain), away(&chosen));
    let said = grid.said(size, |i| format!("File {i}"));
    assert_eq!((said.total, said.rows.len(), said.rows[1].chosen, said.rows[0].chosen), (6, 6, true, false));
    assert_eq!(said.rows[1].cells, ["File 1"]);
}

#[test]
fn a_preferences_window_is_a_split_view_of_its_pages() {
    use super::super::pages::preferences;
    let pages = || names().into_iter().map(item).collect();
    let page = || ww::label("Volume");
    let mut wide = ui_of(preferences(800.0, pages(), 1, Msg::Page, false, Msg::Page(99), page()), (800, 400));
    let said = words(&mut wide);
    assert!(said.iter().any(|w| w == "Display") && said.iter().any(|w| w == "Volume"), "{said:?}");
    let rows = found(&wide, |w| matches!(w, What::List { .. }));
    let (x, y) = middle(wide.part_rect(&rows.id, 2).unwrap());
    assert_eq!(click(&mut wide, x, y), vec![Msg::Page(2)]);
    let mut narrow = ui_of(preferences(400.0, pages(), 1, Msg::Page, true, Msg::Page(99), page()), (400, 400));
    let said = words(&mut narrow);
    assert!(said.iter().any(|w| w == "Sound") && said.iter().any(|w| w == "Back") && !said.iter().any(|w| w == "Display"), "{said:?}");
}

#[test]
fn a_status_page_and_a_welcome_page_say_what_they_are_and_offer_one_action() {
    use super::super::pages::{status_page, welcome_page};
    let mut empty = ui_of(status_page::<Msg>("folder-symbolic", "Nothing here", "This folder is empty.", None), (400, 400));
    let said = words(&mut empty);
    assert!(said.iter().any(|w| w == "Nothing here") && said.iter().any(|w| w == "This folder is empty."), "{said:?}");
    assert!(empty.round_and_worded().is_empty());
    let mut welcome = ui_of(welcome_page("folder-symbolic", "Notes", "Write things down.", "Start", Msg::Page(0)), (400, 400));
    let start = welcome.round_and_worded();
    assert_eq!(start.len(), 1);
    let (x, y) = middle(start[0].1);
    assert_eq!(click(&mut welcome, x, y), vec![Msg::Page(0)]);
    let title = welcome.spots.iter().find(|s| matches!(welcome.tree.at(&s.path).map(|n| &n.what), Some(What::Label { text, .. }) if text == "Notes")).unwrap().rect;
    assert!((title.x + title.w / 2.0 - 200.0).abs() < 1.0, "{title:?}");
    assert!(title.y > 100.0, "{title:?}");
}

#[test]
fn an_about_page_gives_the_version_website_credits_and_licence() {
    use super::super::pages::{about_page, About};
    let about = About {
        name: "Notes".into(), icon: "folder-symbolic".into(), version: "1.2".into(), summary: "Write things down.".into(),
        website: Some("https://example.com".into()), credits: vec![("Made by".into(), vec!["John Smith".into(), "Jane Smith".into()])],
        licence: "GPL 3 or later".into(), copyright: "2026 John Smith".into(),
    };
    let mut ui = ui_of(about_page::<Msg>("about", &about), (500, 800));
    let said = words(&mut ui);
    for w in ["Notes", "Version 1.2", "Website", "John Smith, Jane Smith", "Licence", "GPL 3 or later", "2026 John Smith"] {
        assert!(said.iter().any(|s| s.contains(w)), "{w} in {said:?}");
    }
}

fn spot_named(ui: &Ui<Msg>, tip_is: &str) -> Spot {
    ui.spots.iter().find(|s| matches!(ui.tree.at(&s.path).map(|n| &n.what), Some(What::Button { tip, .. }) if tip == tip_is)).cloned().expect(tip_is)
}

#[test]
fn a_frame_names_its_card_and_a_fixed_layout_puts_things_where_it_is_told() {
    use super::super::frames::{fixed, frame};
    let mut ui = ui_of(frame("Sound", ww::label("Volume")), (400, 300));
    let said = words(&mut ui);
    assert!(said.iter().any(|w| w == "Sound") && said.iter().any(|w| w == "Volume"));
    let ui = ui_of(fixed(vec![((30.0, 40.0), ww::button("list-add-symbolic", "Add")), ((200.0, 10.0), ww::button("list-add-symbolic", "More"))]), (400, 300));
    let (add, more) = (spot_named(&ui, "Add").rect, spot_named(&ui, "More").rect);
    assert_eq!((add.x, add.y, more.x, more.y), (30.0, 40.0, 200.0, 10.0));
}

#[test]
fn an_aspect_frame_keeps_its_shape_in_the_middle_of_any_room() {
    let pic = || ww::drawing((10.0, 10.0), "Picture", |_, _| {}).width(Length::Fill).height(Length::Fill);
    let wide = ui_of(ww::aspect(2.0, pic()).height(Length::Fill), (400, 100));
    let r = found(&wide, |w| matches!(w, What::Drawing { .. })).rect;
    assert_eq!((r.x, r.y, r.w, r.h), (100.0, 0.0, 200.0, 100.0));
    let tall = ui_of(ww::aspect(2.0, pic()).height(Length::Fill), (400, 600));
    let r = found(&tall, |w| matches!(w, What::Drawing { .. })).rect;
    assert_eq!((r.x, r.y, r.w, r.h), (0.0, 200.0, 400.0, 200.0));
    let col = ui_of(ww::column(vec![ww::aspect(4.0, pic()), ww::label("Under")]), (400, 600));
    assert_eq!(found(&col, |w| matches!(w, What::Drawing { .. })).rect.h, 100.0);
}

#[test]
fn a_badge_sits_on_the_corner_it_is_given_and_both_are_heard() {
    use super::super::frames::{badged, Corner};
    let inbox = || ww::button("document-open-symbolic", "Inbox").size(48.0);
    let badge = || ww::card(ww::note("3")).chip();
    let mut ui = ui_of(ww::column(vec![badged(inbox(), badge(), Corner::TopEnd)]).padding(20.0), (400, 300));
    let marked = spot_named(&ui, "Inbox").rect;
    let label = ui.spots.iter().find(|s| matches!(ui.tree.at(&s.path).map(|n| &n.what), Some(What::Label { text, .. }) if text == "3")).unwrap().rect;
    assert!(label.y < marked.y + marked.h / 2.0 && label.x > marked.x + marked.w / 2.0, "{label:?} on {marked:?}");
    let said = words(&mut ui);
    assert!(said.iter().any(|w| w == "Inbox") && said.iter().any(|w| w == "3"));
}

#[test]
fn a_revealer_fades_in_and_is_gone_at_nothing() {
    use super::super::frames::revealer;
    let view = |share| ww::column(vec![revealer(ww::button("list-add-symbolic", "Add").on_press(Msg::Page(1)), share), ww::label("Below")]);
    let mut gone = ui_of(view(0.0), (400, 300));
    assert!(!words(&mut gone).iter().any(|w| w == "Add"));
    let below = gone.spots.iter().find(|s| matches!(gone.tree.at(&s.path).map(|n| &n.what), Some(What::Label { .. }))).unwrap().rect;
    assert!(below.y <= theme::sheet().gap, "it takes no room of its own: {below:?}");
    let half = ui_of(view(0.5), (400, 300));
    assert_eq!(half.tree.at(&spot_named(&half, "Add").path).map(|n| n.opacity), Some(0.5));
}

#[test]
fn a_pagers_dots_say_their_page_mark_the_current_and_pick_one() {
    use super::super::frames::pager;
    let pages = || vec![ww::label("One"), ww::label("Two"), ww::label("Three")];
    let mut ui = ui_of(pager(pages(), 300.0, 1.0, Msg::Page), (400, 300));
    let third = spot_named(&ui, "Page 3 of 3");
    assert!(third.rect.w >= theme::sheet().dot_target);
    let (x, y) = middle(third.rect);
    assert_eq!(click(&mut ui, x, y), vec![Msg::Page(2)]);
    let told = ui.access("Test");
    let dot = told.nodes.iter().find(|(_, n)| n.label() == Some("Page 2 of 3")).map(|(_, n)| n).unwrap();
    assert_eq!(dot.toggled(), Some(accesskit::Toggled::True));
    let two = ui.spots.iter().find(|s| matches!(ui.tree.at(&s.path).map(|n| &n.what), Some(What::Label { text, .. }) if text == "Two")).unwrap().rect;
    assert_eq!(two.x, 0.0);
}

#[test]
fn a_bottom_sheet_lies_over_the_foot_of_the_page_and_shuts() {
    let page = ww::column(vec![ww::button("list-add-symbolic", "Add").on_press(Msg::Page(1))]);
    let mut ui = ui_of(nav::bottom_sheet(page, ww::label("Share with"), true, Msg::Page(0)), (400, 600));
    let label = ui.spots.iter().find(|s| matches!(ui.tree.at(&s.path).map(|n| &n.what), Some(What::Label { text, .. }) if text == "Share with")).unwrap().rect;
    assert!(label.y > 450.0, "{label:?}");
    let (x, y) = middle(spot_named(&ui, "Add").rect);
    assert_eq!(click(&mut ui, x, y), vec![Msg::Page(1)]);
    let (x, y) = middle(spot_named(&ui, "Close the panel").rect);
    assert_eq!(click(&mut ui, x, y), vec![Msg::Page(0)]);
    assert_eq!(ui.input(&key(Keysym::Escape), Instant::now()).messages, vec![Msg::Page(0)]);
}

fn long() -> Ui<Msg> {
    let rows = (0..200).map(|i| ww::label(format!("Row {i}"))).collect();
    ui_of(ww::scroll("area", ww::column(rows)), (400, 300))
}

fn scrolled(ui: &Ui<Msg>) -> f32 {
    ui.scrolls.get("area").copied().unwrap_or(0.0)
}

fn flick(ui: &mut Ui<Msg>, start: Instant) -> Instant {
    ui.input(&Input::TouchDown { id: 1, x: 200.0, y: 250.0 }, start);
    let mut t = start;
    for step in 1..=5 {
        t = start + Duration::from_millis(10 * step);
        ui.input(&Input::TouchMove { id: 1, x: 200.0, y: 250.0 - 40.0 * step as f32 }, t);
    }
    ui.input(&Input::TouchUp { id: 1 }, t);
    t
}

#[test]
fn a_flicked_area_coasts_on_slowing_and_stops_and_a_touch_stops_it_at_once() {
    let mut ui = long();
    let start = Instant::now();
    let up = flick(&mut ui, start);
    let let_go = scrolled(&ui);
    assert_eq!(let_go, 200.0);
    assert!(ui.next_wake(up).is_some());
    assert!(ui.tick(up + Duration::from_millis(16)));
    let first = scrolled(&ui) - let_go;
    assert!(first > 0.0, "it coasts on the way it was going");
    ui.tick(up + Duration::from_millis(32));
    let second = scrolled(&ui) - let_go - first;
    assert!(second > 0.0 && second < first, "and slows: {first} then {second}");
    let mut t = up + Duration::from_millis(32);
    for _ in 0..400 {
        t += Duration::from_millis(16);
        ui.tick(t);
    }
    assert!(!ui.fling.coasting() && ui.next_wake(t).is_none());
    let mut ui = long();
    let up = flick(&mut ui, start);
    ui.tick(up + Duration::from_millis(16));
    ui.input(&Input::TouchDown { id: 2, x: 200.0, y: 100.0 }, up + Duration::from_millis(20));
    let held = scrolled(&ui);
    assert!(!ui.tick(up + Duration::from_millis(40)));
    assert_eq!(scrolled(&ui), held);
}

#[test]
fn a_slow_drag_or_less_movement_does_not_coast() {
    let mut ui = long();
    let start = Instant::now();
    ui.input(&Input::TouchDown { id: 1, x: 200.0, y: 250.0 }, start);
    ui.input(&Input::TouchMove { id: 1, x: 200.0, y: 200.0 }, start + Duration::from_millis(10));
    ui.input(&Input::TouchMove { id: 1, x: 200.0, y: 199.0 }, start + Duration::from_millis(100));
    ui.input(&Input::TouchUp { id: 1 }, start + Duration::from_millis(100));
    assert!(!ui.fling.coasting());
    super::super::motion::reduce(true);
    let mut still = long();
    flick(&mut still, start);
    super::super::motion::reduce(false);
    assert!(!still.fling.coasting());
}

#[test]
fn a_touchpad_flick_coasts_once_the_fingers_come_off() {
    let mut ui = long();
    let start = Instant::now();
    ui.input(&Input::PointerMove { x: 200.0, y: 150.0 }, start);
    for step in 1..=4 {
        ui.input(&Input::Scroll { x: 0.0, y: 30.0, steps: false }, start + Duration::from_millis(10 * step));
    }
    assert!(!ui.fling.coasting(), "not while the fingers are on it");
    let off = start + Duration::from_millis(45);
    ui.input(&Input::Scroll { x: 0.0, y: 0.0, steps: false }, off);
    assert!(ui.fling.coasting());
    let was = scrolled(&ui);
    ui.tick(off + Duration::from_millis(16));
    assert!(scrolled(&ui) > was);
    ui.input(&Input::Scroll { x: 0.0, y: 1.0, steps: true }, off + Duration::from_millis(20));
    assert!(!ui.fling.coasting());
}

#[test]
fn alt_left_the_back_key_and_the_mouses_back_button_go_back() {
    let alt = Input::Key { sym: Keysym::Left, text: None, mods: Mods { alt: true, ..Mods::default() }, repeat: false };
    assert!(nav::back_key(&alt));
    assert!(!nav::back_key(&key(Keysym::Left)));
    assert!(nav::back_key(&key(Keysym::XF86_Back)));
    assert!(nav::back_key(&Input::Press { button: Button::Back, x: 1.0, y: 1.0 }));
    assert!(!nav::back_key(&Input::Press { button: Button::Left, x: 1.0, y: 1.0 }));
}

#[test]
fn a_press_on_a_floating_card_does_not_reach_the_button_under_it() {
    let page = ww::row(vec![ww::button("x", "Under").on_press(Msg::Page(1)), ww::label("").width(Length::Fill), ww::button("x", "Beside").on_press(Msg::Page(2))]);
    let menu = ww::placed(ww::card(ww::label("A menu card")).floats(), |(_, h), (_, ah)| (0.0, (ah - h) / 2.0));
    let mut ui = ui_of(ww::layers(vec![page, menu]), (600, 300));
    let button = |ui: &Ui<Msg>, name: &str| found(ui, |w| matches!(w, What::Button { tip, .. } if tip == name)).rect;
    let (under, beside) = (button(&ui, "Under"), button(&ui, "Beside"));
    let card = ui.spots.iter().rfind(|s| matches!(ui.tree.at(&s.path).map(|n| &n.what), Some(What::Card(..)))).cloned().unwrap().rect;
    let (x, y) = middle(under);
    assert!(card.contains(x, y) && !card.contains(middle(beside).0, middle(beside).1), "card {card:?}, under {under:?}, beside {beside:?}");
    assert_eq!(click(&mut ui, x, y), vec![]);
    let (x, y) = middle(beside);
    assert_eq!(click(&mut ui, x, y), vec![Msg::Page(2)]);
}
