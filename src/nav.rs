//! Moving between pages: a stack with its switcher, a navigation view with a back button and history, a split view, and panels laid over a page.

use super::input::{Button, Input, Keysym};
use super::theme;
use super::widget::{button, card, choice, column, layers, list, row, space, title, Item, Length, Node};

pub fn stack<M>(names: Vec<String>, current: usize, choose: impl Fn(usize) -> M + 'static, page: Node<M>) -> Node<M> {
    column(vec![choice(names, current, crate::t!("kit-pages"), choose), page]).gap(theme::sheet().gap)
}

pub fn stack_beside<M>(pages: Vec<Item>, current: usize, choose: impl Fn(usize) -> M + 'static, page: Node<M>) -> Node<M> {
    let k = theme::sheet();
    row(vec![card(list(pages, Some(current), choose).here().called(crate::t!("kit-pages"))).side().width(Length::Fixed(k.sidebar_width)), page.width(Length::Fill)]).gap(k.gap)
}

pub fn page_key(input: &Input, current: usize, count: usize) -> Option<usize> {
    let Input::Key { sym, mods, .. } = input else { return None };
    if !mods.ctrl || mods.alt || count == 0 {
        return None;
    }
    let on = match *sym {
        Keysym::Page_Down => true,
        Keysym::Tab => !mods.shift,
        Keysym::Page_Up | Keysym::ISO_Left_Tab => false,
        _ => return None,
    };
    Some(if on { (current + 1) % count } else { (current + count - 1) % count })
}

#[derive(Clone, Debug, PartialEq)]
pub struct History<P> {
    pages: Vec<P>,
}

impl<P> History<P> {
    pub fn new(first: P) -> Self {
        History { pages: vec![first] }
    }

    pub fn push(&mut self, page: P) {
        self.pages.push(page);
    }

    pub fn back(&mut self) -> bool {
        self.pages.len() > 1 && self.pages.pop().is_some()
    }

    pub fn home(&mut self) {
        self.pages.truncate(1);
    }

    pub fn current(&self) -> &P {
        &self.pages[self.pages.len() - 1]
    }

    pub fn can_go_back(&self) -> bool {
        self.pages.len() > 1
    }

    pub fn trail(&self) -> &[P] {
        &self.pages
    }
}

pub fn nav_view<M: Clone>(name: impl Into<String>, back: Option<M>, page: Node<M>) -> Node<M> {
    let gap = theme::sheet().gap;
    let mut bar = Vec::new();
    if let Some(m) = back.clone() {
        bar.push(button("go-previous-symbolic", crate::t!("kit-back")).on_press(m));
    }
    bar.push(title(name));
    let view = column(vec![row(bar).gap(gap), page.height(Length::Fill)]).gap(gap);
    match back {
        Some(m) => view.on_escape(m),
        None => view,
    }
}

pub fn is_narrow(width: f32) -> bool {
    width < theme::sheet().split_narrow
}

pub fn split_view<M: Clone>(width: f32, side: Node<M>, content: Node<M>, name: impl Into<String>, showing: bool, back: M) -> Node<M> {
    let k = theme::sheet();
    match (is_narrow(width), showing) {
        (false, _) => row(vec![card(side).side().width(Length::Fixed(k.sidebar_width)), content.width(Length::Fill)]).gap(k.gap),
        (true, true) => nav_view(name, Some(back), content),
        (true, false) => side,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Edge {
    Start,
    End,
}

pub fn drawer<M: Clone>(page: Node<M>, panel: Node<M>, open: bool, edge: Edge, width: f32, shut: M) -> Node<M> {
    if !open {
        return page;
    }
    let k = theme::sheet();
    let wide = k.drawer_width.min(width - k.drawer_room).max(0.0);
    let close = row(vec![space(), button("window-close-symbolic", crate::t!("kit-shut-panel")).on_press(shut.clone())]);
    let card = card(column(vec![close, panel.height(Length::Fill)]).gap(k.gap)).floats().width(Length::Fixed(wide)).height(Length::Fill);
    let across = match edge {
        Edge::Start => vec![card, space()],
        Edge::End => vec![space(), card],
    };
    layers(vec![page, row(across).height(Length::Fill)]).on_escape(shut)
}

pub fn bottom_sheet<M: Clone>(page: Node<M>, panel: Node<M>, open: bool, shut: M) -> Node<M> {
    if !open {
        return page;
    }
    let gap = theme::sheet().gap;
    let close = row(vec![space(), button("window-close-symbolic", crate::t!("kit-shut-panel")).on_press(shut.clone())]);
    let sheet = card(column(vec![close, panel]).gap(gap)).floats();
    layers(vec![page, column(vec![space().height(Length::Fill), sheet]).height(Length::Fill)]).on_escape(shut)
}

pub fn back_key(input: &Input) -> bool {
    match input {
        Input::Key { sym: Keysym::Left, mods, .. } => mods.alt && !mods.ctrl && !mods.shift,
        Input::Key { sym: Keysym::XF86_Back, .. } => true,
        Input::Press { button: Button::Back, .. } => true,
        _ => false,
    }
}
