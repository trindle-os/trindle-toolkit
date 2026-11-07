//! Input waiting to be given to its window.

use super::Input;

#[derive(Default)]
pub(super) struct Inbox {
    waiting: std::collections::VecDeque<(u64, Input, Option<u32>)>,
}

impl Inbox {
    pub(super) fn push(&mut self, (win, input): (u64, Input)) {
        if let Some((before, was, None)) = self.waiting.back_mut() {
            let same = *before == win && match (&*was, &input) {
                (Input::PointerMove { .. }, Input::PointerMove { .. }) => true,
                (Input::TouchMove { id: a, .. }, Input::TouchMove { id: b, .. }) => a == b,
                _ => false,
            };
            if same {
                *was = input;
                return;
            }
        }
        self.waiting.push_back((win, input, None));
    }

    pub(super) fn push_pressed(&mut self, win: u64, input: Input, serial: u32) {
        self.waiting.push_back((win, input, Some(serial)));
    }

    pub(super) fn extend(&mut self, more: impl IntoIterator<Item = (u64, Input)>) {
        for one in more {
            self.push(one);
        }
    }

    pub(super) fn next(&mut self) -> Option<(u64, Input, Option<u32>)> {
        self.waiting.pop_front()
    }

    pub(super) fn forget(&mut self, win: u64) {
        self.waiting.retain(|(id, ..)| *id != win);
    }
}
