//! A title bar with the window buttons, edges to pull and a shadow round it, drawn from the sheet, for a window the compositor does not frame (`Window::needs_frame`).

use std::cell::Cell;
use std::rc::Rc;

use super::input::Cursor;
use super::text::Style;
use super::theme;
use super::widget::{button, column, drawing, layers, row, Length, MenuRow, Node, Pointer, Rect};
use super::window::Edges;
use crate::t;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Framing {
    Move,
    Resize(Edges),
    Minimise,
    Maximise,
    Close,
    Menu(f32, f32),
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Frame {
    pub maximised: bool,
    pub shadow: f32,
    pub compositor_menu: bool,
}

fn cursor_for(e: Edges) -> Cursor {
    match e {
        e if e == Edges::TOP || e == Edges::BOTTOM => Cursor::NResize,
        e if e == Edges::LEFT || e == Edges::RIGHT => Cursor::EResize,
        e if e == Edges::TOP | Edges::LEFT || e == Edges::BOTTOM | Edges::RIGHT => Cursor::NwResize,
        _ => Cursor::NeResize,
    }
}

fn edge<M: 'static>(edges: Edges, size: (f32, f32), act: Rc<dyn Fn(Framing) -> M>) -> Node<M> {
    let along = if edges == Edges::TOP || edges == Edges::BOTTOM { Length::Fill } else { Length::Fixed(size.0) };
    let down = if edges == Edges::LEFT || edges == Edges::RIGHT { Length::Fill } else { Length::Fixed(size.1) };
    drawing(size, "", |_, _| {})
        .on_pointer_when(move |p| matches!(p, Pointer::Down(..)).then(|| act(Framing::Resize(edges))))
        .cursor_with(move |_, _| cursor_for(edges))
        .width(along)
        .height(down)
}

fn own_menu<M>(maximised: bool, act: &dyn Fn(Framing) -> M) -> Vec<MenuRow<M>> {
    let max = if maximised { t!("kit-frame-restore") } else { t!("kit-frame-maximise") };
    let mut rows = vec![
        MenuRow::send("", max, act(Framing::Maximise)),
        MenuRow::send("", t!("kit-frame-minimise"), act(Framing::Minimise)),
        MenuRow::send("", t!("kit-frame-move"), act(Framing::Move)),
    ];
    if !maximised {
        rows.push(MenuRow::send("", t!("kit-frame-resize"), act(Framing::Resize(Edges::BOTTOM | Edges::RIGHT))));
    }
    rows.push(MenuRow::send("", t!("kit-frame-close"), act(Framing::Close)).rule());
    rows
}

pub fn framed<M: 'static>(title: &str, frame: Frame, body: Node<M>, act: impl Fn(Framing) -> M + 'static) -> Node<M> {
    let sheet = theme::sheet();
    let (bar_high, thick, room) = (sheet.button + 2.0 * sheet.title_bar_room, sheet.frame_edge, frame.shadow.max(0.0));
    let act: Rc<dyn Fn(Framing) -> M> = Rc::new(act);
    let (max_icon, max_tip) = if frame.maximised { ("window-restore-symbolic", t!("kit-frame-restore")) } else { ("window-maximize-symbolic", t!("kit-frame-maximise")) };
    let from = room + if frame.maximised { 0.0 } else { thick };
    let a = act.clone();
    let (name, pad, cut) = (title.to_string(), sheet.title_bar_room, Rc::new(Cell::new(false)));
    let shown = cut.clone();
    let mut grip = drawing((1.0, bar_high), title, move |canvas, pen| {
        let (w, h, k) = (canvas.width() as f32, canvas.height() as f32, pen.scale);
        let laid = pen.fonts.layout(&name, &Style::new((theme::sheet().text.0 * k).round()), None);
        shown.set(laid.width > w - 2.0 * pad * k);
        pen.fonts.draw(canvas, &laid, (pad * k).round(), ((h - laid.height) / 2.0).round(), pen.theme.fg);
    })
        .named(true)
        .tip_if_cut(cut)
        .on_pointer_when(move |p| match p {
            Pointer::Down(..) => Some(a(Framing::Move)),
            Pointer::Menu(x, y) if frame.compositor_menu => Some(a(Framing::Menu(from + x, from + y))),
            _ => None,
        })
        .on_double_press(act(Framing::Maximise))
        .width(Length::Fill)
        .height(Length::Fill);
    if !frame.compositor_menu {
        let a = act.clone();
        grip = grip.menu(move |_, _| own_menu(frame.maximised, &*a));
    }
    let back = drawing((1.0, bar_high), "", |canvas, pen| {
        if let Some(r) = tiny_skia::Rect::from_xywh(0.0, 0.0, canvas.width() as f32, canvas.height() as f32) {
            canvas.fill_rect(r, &super::colour::paint(pen.theme.panel), tiny_skia::Transform::identity(), None);
        }
    }).passes().width(Length::Fill).height(Length::Fill);
    let buttons = row(vec![
        button("window-minimize-symbolic", t!("kit-frame-minimise")).on_press(act(Framing::Minimise)).key("frame-minimise"),
        button(max_icon, max_tip).on_press(act(Framing::Maximise)).key("frame-maximise"),
        button("window-close-symbolic", t!("kit-frame-close")).on_press(act(Framing::Close)).key("frame-close"),
    ]).gap(sheet.gap).padding(sheet.title_bar_room).width(Length::Shrink);
    let bar = layers(vec![back, row(vec![grip, buttons]).gap(0.0)]).height(Length::Fixed(bar_high));
    let inside = column(vec![bar, body.width(Length::Fill).height(Length::Fill)]).gap(0.0).width(Length::Fill).height(Length::Fill);
    let whole = if frame.maximised { inside } else {
        let side = |e: Edges| edge(e, (thick, thick), act.clone());
        column(vec![
            row(vec![side(Edges::TOP | Edges::LEFT), side(Edges::TOP), side(Edges::TOP | Edges::RIGHT)]).gap(0.0).height(Length::Fixed(thick)),
            row(vec![side(Edges::LEFT), inside, side(Edges::RIGHT)]).gap(0.0).height(Length::Fill),
            row(vec![side(Edges::BOTTOM | Edges::LEFT), side(Edges::BOTTOM), side(Edges::BOTTOM | Edges::RIGHT)]).gap(0.0).height(Length::Fixed(thick)),
        ]).gap(0.0).width(Length::Fill).height(Length::Fill)
    };
    layers(vec![page_under(room), whole.padding(room)]).width(Length::Fill).height(Length::Fill)
}

fn page_under<M>(room: f32) -> Node<M> {
    drawing((1.0, 1.0), "", move |canvas, pen| {
        let k = room * pen.scale;
        let r = Rect { x: k, y: k, w: canvas.width() as f32 - 2.0 * k, h: canvas.height() as f32 - 2.0 * k };
        if room > 0.0 {
            super::shadow::below_for_drawing(canvas, pen.theme, &pen.theme.float, r, 0.0, pen.scale);
        }
        if let Some(r) = tiny_skia::Rect::from_xywh(r.x, r.y, r.w, r.h) {
            canvas.fill_rect(r, &super::colour::paint(pen.theme.bg), tiny_skia::Transform::identity(), None);
        }
    }).passes().width(Length::Fill).height(Length::Fill)
}
