//! Landmarks: the big parts of a window a screen reader can jump between.

use super::Node;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Landmark {
    Main,
    Navigation,
    Beside,
    Search,
    Form,
    Top,
    Foot,
    Region,
}

impl<M> Node<M> {
    pub fn landmark(mut self, kind: Landmark) -> Self {
        self.says_mut().landmark = Some(kind);
        self
    }
}
