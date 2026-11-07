//! What a text box is for, told to the input method for an on-screen keyboard.

use super::Node;
use crate::input::Purpose;

impl<M> Node<M> {
    pub fn purpose(mut self, p: Purpose) -> Self {
        self.says_mut().purpose = Some(p);
        self
    }
}
