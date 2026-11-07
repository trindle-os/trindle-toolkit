//! A dialog as a page: shown in place of the window's page, never over it.

use crate::theme;
use crate::role::Role;
use crate::widget::{column, entry, lead, main_word, page_title, quiet_action, row, space, Length, Node};

pub struct Dialog<M> {
    title: String,
    lead: String,
    content: Vec<Node<M>>,
    actions: Vec<Node<M>>,
    cancel: M,
    turn: u32,
    typing: bool,
}

pub fn dialog<M>(title: impl Into<String>, lead: impl Into<String>, cancel: M) -> Dialog<M> {
    Dialog { title: title.into(), lead: lead.into(), content: Vec::new(), actions: Vec::new(), cancel, turn: 0, typing: false }
}

impl<M: Clone + 'static> Dialog<M> {
    pub fn holds(mut self, n: Node<M>) -> Self {
        self.content.push(n);
        self
    }

    pub fn answer(mut self, n: Node<M>) -> Self {
        self.actions.push(n);
        self
    }

    pub fn turn(mut self, turn: u32) -> Self {
        self.turn = turn;
        self
    }

    pub fn node(self) -> Node<M> {
        let mut cancel = quiet_action("", crate::t!("kit-cancel")).on_press(self.cancel.clone());
        if !self.typing {
            cancel = cancel.grab(self.turn).first();
        }
        let mut foot = vec![space(), cancel];
        foot.extend(self.actions);
        let mut items = vec![page_title(self.title.clone()), lead(self.lead)];
        items.extend(self.content);
        items.push(space().height(Length::Fill));
        items.push(row(foot));
        column(items)
            .padding(theme::sheet().page_padding)
            .height(Length::Fill)
            .role(Role::Dialog)
            .called(self.title)
            .on_escape(self.cancel)
    }
}

pub fn confirm<M: Clone + 'static>(title: impl Into<String>, message: impl Into<String>, yes: impl Into<String>, go: M, cancel: M, turn: u32) -> Node<M> {
    dialog(title, message, cancel).answer(main_word(yes).on_press(go).key("dialog-yes")).turn(turn).node()
}

pub fn choose<M: Clone + 'static>(title: impl Into<String>, message: impl Into<String>, options: Vec<(String, M)>, cancel: M, turn: u32) -> Node<M> {
    let buttons = options.into_iter().enumerate()
        .map(|(i, (words, m))| quiet_action("", words).on_press(m).key(format!("dialog-option-{i}")))
        .collect();
    dialog(title, message, cancel).holds(column(buttons).gap(theme::gaps().small)).turn(turn).node()
}

pub fn ask<M: Clone + 'static>(title: impl Into<String>, message: impl Into<String>, text: impl Into<String>, hint: impl Into<String>,
                               change: impl Fn(String) -> M + 'static, yes: impl Into<String>, go: M, cancel: M, turn: u32) -> Node<M> {
    let mut d = dialog(title, message, cancel).holds(entry(text, hint, change).on_submit(go.clone()).key("dialog-text").grab(turn).first())
        .answer(main_word(yes).on_press(go).key("dialog-yes"));
    d.typing = true;
    d.turn(turn).node()
}
