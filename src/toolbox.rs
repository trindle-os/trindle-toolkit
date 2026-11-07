//! A toolbox of icon buttons with one tool in use, and pages behind a row of tabs, which nest: a page may hold tabs of its own.

use super::widget::{button, choice, column, even_grid, Node};

pub struct Tool<'a> {
    pub icon: &'a str,
    pub name: &'a str,
}

pub fn toolbox<M>(tools: &[Tool], chosen: usize, choose: impl Fn(usize) -> M) -> Node<M> {
    let buttons = tools.iter().enumerate().map(|(i, t)| button(t.icon, t.name).on(i == chosen).on_press(choose(i))).collect();
    even_grid(super::theme::TOUCH_TARGET + 8.0, buttons).gap(crate::theme::gaps().small)
}

pub fn tab_pages<M>(names: Vec<String>, chosen: usize, tip: &str, choose: impl Fn(usize) -> M + 'static, page: Node<M>) -> Node<M> {
    column(vec![choice(names, chosen, tip, choose), page]).gap(crate::theme::gaps().roomy)
}
