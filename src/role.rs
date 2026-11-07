//! What a drawing or a group is to a screen reader, when the toolkit has no widget of its own for it: the roles given with `Node::role`.

#[derive(Clone, Debug, PartialEq)]
pub enum Role {
    Button,
    CheckBox(bool),
    Switch(bool),
    Slider { value: f64, min: f64, max: f64 },
    SpinButton { value: f64, min: f64, max: f64 },
    Meter { value: f64, min: f64, max: f64 },
    Image,
    Group,
    Dialog,
    MenuBar { titles: Vec<(String, f32, f32)>, lit: Option<usize>, open: bool },
    Toolbar { tools: Vec<(String, f32, f32)>, lit: Option<usize>, on: Vec<Option<bool>> },
    Status,
    Path,
    Range { low: f64, high: f64, min: f64, max: f64, lit: usize },
    Segments { parts: Vec<(String, f64, f64, f32, f32)>, lit: usize },
}

impl Role {
    pub(crate) fn steps(&self) -> bool {
        matches!(self, Role::Slider { .. } | Role::SpinButton { .. })
    }
}
