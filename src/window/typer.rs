//! The input method's way in: what it is told of the text box with the keyboard, and what it sends back.

use wayland_protocols::wp::text_input::zv3::client::zwp_text_input_v3::{self, ChangeCause, ContentHint, ContentPurpose};

use super::*;
use crate::input::Purpose;

fn content_type(p: Purpose) -> (ContentHint, ContentPurpose) {
    match p {
        Purpose::Words => (ContentHint::None, ContentPurpose::Normal),
        Purpose::Digits => (ContentHint::None, ContentPurpose::Digits),
        Purpose::Number => (ContentHint::None, ContentPurpose::Number),
        Purpose::Phone => (ContentHint::None, ContentPurpose::Phone),
        Purpose::Address => (ContentHint::None, ContentPurpose::Url),
        Purpose::Email => (ContentHint::None, ContentPurpose::Email),
        Purpose::Name => (ContentHint::AutoCapitalization, ContentPurpose::Name),
        Purpose::Date => (ContentHint::None, ContentPurpose::Date),
        Purpose::Time => (ContentHint::None, ContentPurpose::Time),
        Purpose::Pin => (ContentHint::SensitiveData | ContentHint::HiddenText, ContentPurpose::Pin),
    }
}

impl State {
    pub(super) fn tell_typer(&mut self) {
        let Some(typer) = self.input_method.typer.clone() else { return };
        let typing = self.input_method.typer_on.and_then(|id| self.windows.iter().find(|w| w.id == id)).and_then(|w| w.typing.clone());
        if typing == self.input_method.typer_sent {
            self.input_method.by_method = false;
            return;
        }
        match &typing {
            None => typer.disable(),
            Some(t) => {
                if self.input_method.typer_sent.as_ref().is_some_and(|s| s.id != t.id) {
                    typer.disable();
                    typer.commit();
                }
                if self.input_method.typer_sent.as_ref().is_none_or(|s| s.id != t.id) {
                    typer.enable();
                }
                let (text, caret, anchor) = if t.secret { (String::new(), 0, 0) } else { around(&t.text, t.caret, t.anchor) };
                typer.set_surrounding_text(text, caret as i32, anchor as i32);
                typer.set_text_change_cause(if self.input_method.by_method { ChangeCause::InputMethod } else { ChangeCause::Other });
                if t.secret {
                    typer.set_content_type(ContentHint::SensitiveData | ContentHint::HiddenText, ContentPurpose::Password);
                } else {
                    let (hint, purpose) = content_type(t.purpose);
                    typer.set_content_type(hint, purpose);
                }
                let (x, y, w, h) = t.area;
                typer.set_cursor_rectangle(x, y, w, h);
            }
        }
        typer.commit();
        self.input_method.typer_sent = typing;
        self.input_method.by_method = false;
    }
}

pub(super) fn around(text: &str, caret: usize, anchor: usize) -> (String, usize, usize) {
    const MOST: usize = 4000;
    let (caret, anchor) = (caret.min(text.len()), anchor.min(text.len()));
    if text.len() < MOST {
        return (text.to_string(), caret, anchor);
    }
    let before = |mut at: usize| {
        while !text.is_char_boundary(at) {
            at -= 1;
        }
        at
    };
    let after = |mut at: usize| {
        while !text.is_char_boundary(at) {
            at += 1;
        }
        at
    };
    let (low, high) = (caret.min(anchor), caret.max(anchor));
    let (low, high) = if high - low < MOST / 2 { (low, high) } else { (caret, caret) };
    let spare = (MOST - 1 - (high - low)) / 2;
    let start = after(low.saturating_sub(spare));
    let end = before((high + spare).min(text.len()));
    let keep = |at: usize| at.clamp(start, end) - start;
    (text[start..end].to_string(), keep(caret), keep(anchor))
}

impl Dispatch<ZwpTextInputV3, ()> for State {
    fn event(state: &mut Self, _: &ZwpTextInputV3, event: zwp_text_input_v3::Event, _: &(), _: &Connection, _: &QueueHandle<Self>) {
        use zwp_text_input_v3::Event;
        match event {
            Event::Enter { surface } => {
                state.input_method.typer_on = win_of(&surface);
                state.input_method.typer_sent = None;
                state.tell_typer();
            }
            Event::Leave { .. } => {
                if let Some(id) = state.input_method.typer_on.take() {
                    state.inbox.push((id, Input::Composed { delete: (0, 0), commit: String::new(), preedit: String::new(), caret: None }));
                }
                state.input_method.typer_sent = None;
                state.input_method.composing = Composing::default();
            }
            Event::PreeditString { text, cursor_begin, cursor_end } => {
                state.input_method.composing.preedit = text.unwrap_or_default();
                state.input_method.composing.caret = (cursor_begin >= 0 && cursor_end >= 0).then_some((cursor_begin as usize, cursor_end as usize));
            }
            Event::CommitString { text } => state.input_method.composing.commit = text.unwrap_or_default(),
            Event::DeleteSurroundingText { before_length, after_length } => {
                state.input_method.composing.delete = (before_length, after_length);
            }
            Event::Done { .. } => {
                let c = std::mem::take(&mut state.input_method.composing);
                if let Some(id) = state.input_method.typer_on {
                    state.inbox.push((id, Input::Composed { delete: c.delete, commit: c.commit, preedit: c.preedit, caret: c.caret }));
                    state.input_method.by_method = true;
                }
            }
            _ => {}
        }
    }
}
