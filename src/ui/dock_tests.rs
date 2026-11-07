//! Tests of dragging a docked panel by its handle: the messages, the line where it would land, and the screen reader's name for the handle.

use super::*;
use super::super::dock::{Change, Dock, Edge, Moved};
use super::super::widget::label;

fn view(dock: &Dock) -> Node<Change> {
    dock.view(label("Main"), |_| label("Body"), |c| c)
}

fn shot(ui: &mut Ui<Change>) -> tiny_skia::Pixmap {
    let mut pix = tiny_skia::Pixmap::new(800, 600).unwrap();
    ui.paint(&mut pix.as_mut(), 1.0);
    pix
}

fn accent_at(ui: &Ui<Change>, pix: &tiny_skia::Pixmap, x: u32, y: u32) -> bool {
    let (p, a) = (pix.pixel(x, y).unwrap(), ui.theme.accent.to_skia().to_color_u8());
    (p.red(), p.green(), p.blue()) == (a.red(), a.green(), a.blue())
}

fn follow(ui: &mut Ui<Change>, dock: &mut Dock, reply: Reply<Change>) -> Vec<Moved> {
    let moved = reply.messages.into_iter().filter_map(|c| dock.update(c)).collect();
    ui.set_view(view(dock));
    ui.settle();
    moved
}

#[test]
fn a_panel_dragged_by_its_handle_shows_the_edge_and_lands_there() {
    let mut dock = Dock::default();
    dock.add("files", "Files", Edge::Left);
    let mut ui = Ui::new(view(&dock), Theme::new(false, "#fbc711"));
    ui.size = (800, 600);
    ui.settle();
    let names: Vec<String> = ui.access("T").nodes.iter().filter_map(|(_, n)| n.label().map(String::from)).collect();
    assert!(names.contains(&"Drag to another edge".to_string()), "{names:?}");
    let g = ui.spot("dock-grip-files").unwrap().rect;
    let (gx, gy) = (g.x + g.w / 2.0, g.y + g.h / 2.0);
    let now = Instant::now();
    ui.input(&Input::PointerMove { x: gx, y: gy }, now);
    let reply = ui.input(&Input::Press { button: Button::Left, x: gx, y: gy }, now);
    assert_eq!(reply.messages, vec![Change::Lift("files".into())]);
    follow(&mut ui, &mut dock, reply);
    assert!(ui.spot("dock-guide").is_some_and(|s| s.rect.w >= 790.0 && s.rect.h >= 590.0), "the guide covers the whole area");
    let pix = shot(&mut ui);
    assert!(!accent_at(&ui, &pix, 1, 300));
    let reply = ui.input(&Input::PointerMove { x: 790.0, y: 300.0 }, now);
    assert_eq!(reply.messages, vec![Change::Over(Edge::Right)]);
    follow(&mut ui, &mut dock, reply);
    let pix = shot(&mut ui);
    assert!(accent_at(&ui, &pix, 799, 300) && !accent_at(&ui, &pix, 400, 300));
    let reply = ui.input(&Input::Release { button: Button::Left, x: 790.0, y: 300.0 }, now);
    assert!(reply.messages.contains(&Change::Drop), "{:?}", reply.messages);
    assert_eq!(follow(&mut ui, &mut dock, reply), vec![Moved { id: "files".into(), edge: Edge::Right }]);
    assert!(ui.spot("dock-guide").is_none());
    assert_eq!(dock.panels[0].edge, Edge::Right);
}

#[test]
fn a_floating_panel_sits_at_the_top_right() {
    let mut dock = Dock::default();
    dock.add("files", "Files", Edge::Float);
    let mut ui = Ui::new(view(&dock), Theme::new(false, "#fbc711"));
    ui.size = (800, 600);
    ui.settle();
    let panel = ui.spot("dock-files").unwrap().rect;
    assert_eq!((panel.x + panel.w, panel.y), (784.0, 16.0));
}

#[test]
fn the_marks_under_a_scale_sit_under_their_values() {
    let mut ui = Ui::new(crate::dial::scale(0.5, 0.0, 1.0, 0.1, &[(0.0, "Low"), (1.0, "High")], "Level", |_| ()), Theme::new(false, "#fbc711"));
    ui.size = (800, 100);
    ui.settle();
    let told = ui.access("T");
    let left = |word: &str| told.nodes.iter().find(|(_, n)| n.label() == Some(word) || n.value() == Some(word)).and_then(|(_, n)| n.bounds()).unwrap().x0;
    assert!(left("Low") < 40.0 && left("High") > 700.0, "{} {}", left("Low"), left("High"));
}
