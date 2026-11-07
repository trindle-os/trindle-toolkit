//! Panels docked round a main area, as in an editor: each panel sits at the left, the right or the foot, or floats over the main area, and is moved by the buttons on its head or by dragging its handle.

use std::cell::Cell;
use std::rc::Rc;

use super::theme;
use super::widget::{button, card, column, drawing, heading, icon_image, layers, placed, row, Length, Node, Pointer, Rect, Tone};
use crate::t;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Edge {
    Left,
    Right,
    Bottom,
    Float,
}

impl Edge {
    const ALL: [Edge; 4] = [Edge::Left, Edge::Right, Edge::Bottom, Edge::Float];

    fn icon(self) -> &'static str {
        match self {
            Edge::Left => "go-first-symbolic",
            Edge::Right => "go-last-symbolic",
            Edge::Bottom => "go-bottom-symbolic",
            Edge::Float => "window-new-symbolic",
        }
    }

    fn tip(self) -> &'static str {
        match self {
            Edge::Left => t!("kit-dock-left"),
            Edge::Right => t!("kit-dock-right"),
            Edge::Bottom => t!("kit-dock-bottom"),
            Edge::Float => t!("kit-dock-float"),
        }
    }
}

pub fn edge_for(x: f32, y: f32, w: f32, h: f32) -> Edge {
    let near = [(x / w.max(1.0), Edge::Left), (1.0 - x / w.max(1.0), Edge::Right), (1.0 - y / h.max(1.0), Edge::Bottom)];
    near.into_iter().filter(|(d, _)| *d <= 0.2).min_by(|a, b| a.0.total_cmp(&b.0)).map_or(Edge::Float, |(_, e)| e)
}

#[derive(Clone, Debug, PartialEq)]
pub struct Panel {
    pub id: String,
    pub title: String,
    pub edge: Edge,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Dock {
    pub panels: Vec<Panel>,
    held: Option<(String, Option<Edge>)>,
    area: Rc<Cell<(f32, f32)>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Moved {
    pub id: String,
    pub edge: Edge,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Change {
    Move(Moved),
    Lift(String),
    Over(Edge),
    Drop,
}

impl Dock {
    pub fn add(&mut self, id: impl Into<String>, title: impl Into<String>, edge: Edge) {
        self.panels.push(Panel { id: id.into(), title: title.into(), edge });
    }

    pub fn update(&mut self, change: Change) -> Option<Moved> {
        let moved = match change {
            Change::Move(m) => m,
            Change::Lift(id) => {
                self.held = Some((id, None));
                return None;
            }
            Change::Over(edge) => {
                if let Some((_, over)) = &mut self.held {
                    *over = Some(edge);
                }
                return None;
            }
            Change::Drop => {
                let (id, edge) = self.held.take()?;
                Moved { id, edge: edge? }
            }
        };
        let i = self.panels.iter().position(|p| p.id == moved.id && p.edge != moved.edge)?;
        let mut p = self.panels.remove(i);
        p.edge = moved.edge;
        self.panels.push(p);
        Some(moved)
    }

    pub fn landing(&self) -> Option<Edge> {
        self.held.as_ref().and_then(|(_, e)| *e)
    }

    pub fn view<M: 'static>(&self, main: Node<M>, body: impl Fn(&str) -> Node<M>, wrap: impl Fn(Change) -> M + Clone + 'static) -> Node<M> {
        let panel = |p: &Panel| {
            let (id, lift) = (p.id.clone(), wrap.clone());
            let grip = icon_image("list-drag-handle-symbolic", GRIP, Tone::Calm).called(t!("kit-dock-grip")).on_pointer_when(move |at| match at {
                Pointer::Down(..) => Some(lift(Change::Lift(id.clone()))),
                Pointer::Up(..) => Some(lift(Change::Drop)),
                _ => None,
            }).key(format!("dock-grip-{}", p.id));
            let mut head = vec![grip, heading(p.title.clone()).width(Length::Fill)];
            head.extend(Edge::ALL.into_iter().filter(|e| *e != p.edge).map(|e| {
                button(e.icon(), e.tip()).spoken(format!("{}: {}", p.title, e.tip())).on_press(wrap(Change::Move(Moved { id: p.id.clone(), edge: e })))
            }));
            card(column(vec![row(head).gap(crate::theme::gaps().small), body(&p.id)]).gap(crate::theme::gaps().medium)).key(format!("dock-{}", p.id))
        };
        let at = |edge: Edge| self.panels.iter().filter(|p| p.edge == edge).map(panel).collect::<Vec<_>>();
        let side = |items: Vec<Node<M>>| column(items).gap(crate::theme::gaps().medium).width(Length::Fixed(PANEL_WIDTH));
        let mut middle = vec![main.width(Length::Fill)];
        middle.extend(at(Edge::Float).into_iter().map(|p| placed(p.width(Length::Fixed(PANEL_WIDTH)), |(iw, _), (w, _)| ((w - iw - 16.0).max(0.0), 16.0))));
        let mut across = Vec::new();
        let (left, right) = (at(Edge::Left), at(Edge::Right));
        if !left.is_empty() {
            across.push(side(left));
        }
        across.push(layers(middle).width(Length::Fill).height(Length::Fill));
        if !right.is_empty() {
            across.push(side(right));
        }
        let mut down = vec![row(across).gap(crate::theme::gaps().medium).height(Length::Fill)];
        let bottom = at(Edge::Bottom);
        if !bottom.is_empty() {
            down.push(row(bottom.into_iter().map(|p| p.width(Length::Fill)).collect()).gap(crate::theme::gaps().medium));
        }
        let whole = column(down).gap(crate::theme::gaps().medium);
        if self.held.is_none() {
            return whole;
        }
        let (area, read, landing) = (self.area.clone(), self.area.clone(), self.landing());
        let guide = drawing((0.0, 0.0), "", move |canvas, pen| {
            let (w, h) = (canvas.width() as f32 / pen.scale, canvas.height() as f32 / pen.scale);
            area.set((w, h));
            let thick = theme::sheet().focus_width.max(2.0);
            let line = match landing {
                Some(Edge::Left) => Rect { x: 0.0, y: 0.0, w: thick, h },
                Some(Edge::Right) => Rect { x: w - thick, y: 0.0, w: thick, h },
                Some(Edge::Bottom) => Rect { x: 0.0, y: h - thick, w, h: thick },
                Some(Edge::Float) => Rect { x: (w - PANEL_WIDTH - 16.0).max(0.0), y: 16.0, w: PANEL_WIDTH.min(w), h: thick },
                None => return,
            };
            super::draw::fill(canvas, pen, line, pen.theme.accent);
        }).on_hover().on_pointer_when(move |at| match at {
            Pointer::Hover(x, y) => {
                let (w, h) = read.get();
                Some(wrap(Change::Over(edge_for(x, y, w, h))))
            }
            _ => None,
        }).key("dock-guide").width(Length::Fill).height(Length::Fill);
        layers(vec![whole, guide])
    }
}

pub const PANEL_WIDTH: f32 = 240.0;
const GRIP: f32 = 24.0;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_drop_near_an_edge_docks_there_and_elsewhere_floats() {
        assert_eq!(edge_for(10.0, 300.0, 800.0, 600.0), Edge::Left);
        assert_eq!(edge_for(790.0, 300.0, 800.0, 600.0), Edge::Right);
        assert_eq!(edge_for(400.0, 590.0, 800.0, 600.0), Edge::Bottom);
        assert_eq!(edge_for(8.0, 597.0, 800.0, 600.0), Edge::Bottom, "the nearer edge wins");
        assert_eq!(edge_for(400.0, 300.0, 800.0, 600.0), Edge::Float);
    }

    #[test]
    fn a_panel_moves_to_the_end_of_its_new_edge() {
        let mut d = Dock::default();
        d.add("files", "Files", Edge::Left);
        d.add("find", "Find", Edge::Left);
        let moved = d.update(Change::Move(Moved { id: "files".into(), edge: Edge::Bottom }));
        assert_eq!(moved, Some(Moved { id: "files".into(), edge: Edge::Bottom }));
        assert_eq!(d.panels.iter().map(|p| (p.id.as_str(), p.edge)).collect::<Vec<_>>(), vec![("find", Edge::Left), ("files", Edge::Bottom)]);
        let v = d.view(super::super::widget::label("Main"), |_| super::super::widget::label("Body"), |m| m);
        let moves: Vec<Edge> = v.presses().into_iter().filter_map(|(_, _, m)| match m {
            Change::Move(m) => Some(m.edge),
            _ => None,
        }).collect();
        assert_eq!(moves, vec![Edge::Right, Edge::Bottom, Edge::Float, Edge::Left, Edge::Right, Edge::Float]);
    }

    #[test]
    fn a_panel_dragged_by_its_handle_lands_on_the_last_edge_it_was_over() {
        let mut d = Dock::default();
        d.add("files", "Files", Edge::Left);
        assert_eq!(d.update(Change::Over(Edge::Right)), None, "nothing is held yet");
        assert_eq!(d.landing(), None);
        d.update(Change::Lift("files".into()));
        assert_eq!(d.update(Change::Drop), None);
        d.update(Change::Lift("files".into()));
        d.update(Change::Over(Edge::Right));
        d.update(Change::Over(Edge::Bottom));
        assert_eq!(d.landing(), Some(Edge::Bottom));
        assert_eq!(d.update(Change::Drop), Some(Moved { id: "files".into(), edge: Edge::Bottom }));
        assert_eq!((d.panels[0].edge, d.landing()), (Edge::Bottom, None));
        d.update(Change::Lift("files".into()));
        d.update(Change::Over(Edge::Bottom));
        assert_eq!(d.update(Change::Drop), None);
    }
}
