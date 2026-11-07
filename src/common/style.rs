//! The look: the theme and highlight colour the person chose, light or dark, and the one style sheet they are drawn with (see sheet.rs). On Trindle these are the person's own, from Settings; anywhere else the built-in sheet and the yellow highlight.

pub use super::sheet::{layers, reread, shadow_layers, split_top, Layers, MOST_LAYERS};
pub use super::sheet::{declarations, CSS};
use super::sheet;

pub const DEFAULT_ACCENT: &str = "#fbc711";

fn valid_hex(s: &str) -> bool {
    (s.len() == 7 || s.len() == 9) && s.starts_with('#') && s[1..].chars().all(|c| c.is_ascii_hexdigit())
}

pub fn on_colour(hex: &str) -> &'static str {
    let fill = sheet::plain_colour(hex).unwrap_or([0.0, 0.0, 0.0, 1.0]);
    let stands = |ink: &str| sheet::contrast(sheet::plain_colour(ink).unwrap_or(fill), fill);
    let best = if stands("#09090b") >= stands("#fafafa") { "#09090b" } else { "#fafafa" };
    let plain = if stands("#000000") >= stands("#ffffff") { "#000000" } else { "#ffffff" };
    if stands(best) >= crate::theme::AIM_WORDS { best } else { plain }
}

#[cfg(any(test, feature = "testing"))]
thread_local! {
    pub static ASKED_FOR_TEST: std::cell::RefCell<Option<(bool, String)>> = const { std::cell::RefCell::new(None) };
}

#[cfg(any(test, feature = "testing"))]
pub fn asked_for_test() -> Option<(bool, String)> {
    ASKED_FOR_TEST.with(|a| a.borrow().clone())
}

pub fn accent() -> String {
    #[cfg(any(test, feature = "testing"))]
    if let Some((_, accent)) = asked_for_test() {
        return accent;
    }
    if let Some(own) = layers().own.get("trindle-highlight", false).map(str::trim).filter(|v| valid_hex(v)) {
        return own.to_ascii_lowercase();
    }
    chosen_accent()
}

pub fn chosen_accent() -> String {
    #[cfg(any(test, feature = "testing"))]
    if let Some((_, accent)) = asked_for_test() {
        return accent;
    }
    let a = super::files::saved("accent");
    if valid_hex(&a) { a } else { DEFAULT_ACCENT.to_string() }
}

pub fn own_sheet() -> std::path::PathBuf {
    super::sheet::own_sheet()
}

pub fn light() -> bool {
    #[cfg(any(test, feature = "testing"))]
    if let Some((light, _)) = asked_for_test() {
        return layers().light(light);
    }
    layers().only.unwrap_or_else(super::sheet::asked_light)
}

pub fn kept_light() -> bool {
    use super::sheet;
    let kept = sheet::theme_words(&sheet::chosen_theme()).unwrap_or_default();
    told_light(&kept, &super::files::saved(sheet::LOOK_KEY))
}

pub fn told_light(kept: &str, saved: &str) -> bool {
    super::sheet::one_look(kept).unwrap_or(saved == "light")
}

pub fn link_label_most() -> usize {
    layers().number("trindle-link-label-most", light(), 24.0, 80.0).unwrap_or(48.0) as usize
}
