//! The usual rows of a settings page, each a `settings::setting` with its control: a switch, a dropdown, a text box or a spin box, named by the row's own name.

use super::settings::setting;
use super::spin::spin;
use super::widget::{dropdown, entry, switch, Node};

pub fn toggle_row<M>(name: &str, help: &str, on: bool, change: impl Fn(bool) -> M + 'static) -> Node<M> {
    setting(name, help, switch(on, name, change))
}

pub fn combo_row<M>(name: &str, help: &str, options: Vec<String>, chosen: Option<usize>, choose: impl Fn(usize) -> M + 'static) -> Node<M> {
    setting(name, help, dropdown(options, chosen, name, choose))
}

pub fn entry_row<M>(name: &str, help: &str, text: &str, change: impl Fn(String) -> M + 'static) -> Node<M> {
    setting(name, help, entry(text, name, change))
}

pub fn spin_row<M: 'static>(name: &str, help: &str, value: f32, (min, max, step): (f32, f32, f32), change: impl Fn(f32) -> M + 'static) -> Node<M> {
    setting(name, help, spin(value, min, max, step, name, change))
}
