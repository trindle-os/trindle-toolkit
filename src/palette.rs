//! The command palette: a page with a box to type in and the actions that match, any of which runs with Enter or a click.

use super::action::Actions;
use super::widget::{column, entry, item, list, note, Length, Node};
use crate::t;

#[derive(Clone, Debug, PartialEq)]
pub enum Edit {
    Typed(String),
    Chosen(usize),
    Run,
    Open(usize),
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Palette {
    pub query: String,
    pub chosen: usize,
}

impl Palette {
    pub fn update<A: Clone + PartialEq>(&mut self, edit: Edit, actions: &Actions<A>) -> Option<A> {
        let row = match edit {
            Edit::Typed(text) => {
                self.query = text;
                self.chosen = 0;
                return None;
            }
            Edit::Chosen(i) => {
                self.chosen = i;
                return None;
            }
            Edit::Run => self.chosen,
            Edit::Open(i) => i,
        };
        let id = actions.search(&self.query).get(row).map(|a| a.id.clone())?;
        *self = Palette::default();
        Some(id)
    }

    pub fn view<A: Clone + PartialEq, M: 'static>(&self, actions: &Actions<A>, wrap: impl Fn(Edit) -> M + Clone + 'static) -> Node<M> {
        let found = actions.search(&self.query);
        let (typed, chosen, open) = (wrap.clone(), wrap.clone(), wrap.clone());
        let input = entry(self.query.clone(), t!("kit-palette-search"), move |s| typed(Edit::Typed(s))).on_submit(wrap(Edit::Run)).first().width(Length::Fill);
        let under = if found.is_empty() {
            note(t!("kit-palette-none")).room_for(1)
        } else {
            let rows = found.iter().map(|a| {
                let mut row = item(a.label.clone());
                if let Some(c) = a.chord {
                    row = row.detail(c.text());
                }
                if !a.icon.is_empty() {
                    row = row.icon(a.icon.clone());
                }
                row
            }).collect();
            list(rows, Some(self.chosen.min(found.len() - 1)), move |i| chosen(Edit::Chosen(i))).on_open(move |i| open(Edit::Open(i))).called(t!("kit-palette-list")).width(Length::Fill)
        };
        column(vec![input, under]).gap(crate::theme::gaps().roomy)
    }
}
