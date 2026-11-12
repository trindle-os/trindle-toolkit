//! Boxes that take only what fits a pattern (a phone number, a date, a hex colour) and say what is wrong in red words under the box, in room kept for them so nothing moves.

use super::widget::{column, entry, label, Length, Node, Tone};
use crate::lang::numerals::{digit_value, latin_digits};
use crate::t;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mask {
    Phone,
    Date,
    Hex,
}

impl Mask {
    pub fn shape(self, typed: &str) -> String {
        match self {
            Mask::Phone => latin_digits(typed).chars().enumerate()
                .filter(|&(i, c)| c.is_ascii_digit() || " ()-".contains(c) || (i == 0 && c == '+'))
                .map(|(_, c)| c).take(24).collect(),
            Mask::Date => latin_digits(typed).chars().filter(|c| c.is_ascii_digit() || *c == '-').take(10).collect(),
            Mask::Hex => {
                let hash = typed.starts_with('#');
                let digits = typed.chars().filter(char::is_ascii_hexdigit).take(8);
                (if hash { "#" } else { "" }).chars().chain(digits).collect()
            }
        }
    }

    pub fn problem(self, text: &str) -> Option<&'static str> {
        if text.is_empty() {
            return None;
        }
        let right = match self {
            Mask::Phone => {
                let figures = text.chars().filter(|c| digit_value(*c).is_some()).count();
                (7..=15).contains(&figures)
            }
            Mask::Date => super::when::Date::parse(text).is_some(),
            Mask::Hex => {
                let d = text.strip_prefix('#').unwrap_or(text);
                matches!(d.len(), 3 | 6 | 8) && d.chars().all(|c| c.is_ascii_hexdigit())
            }
        };
        (!right).then(|| match self {
            Mask::Phone => t!("kit-check-phone"),
            Mask::Date => t!("kit-check-date"),
            Mask::Hex => t!("kit-check-hex"),
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Edit {
    Typed(String),
    Left,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CheckedBox {
    pub mask: Mask,
    pub text: String,
    pub left: bool,
}

impl CheckedBox {
    pub fn new(mask: Mask, text: &str) -> CheckedBox {
        CheckedBox { mask, text: mask.shape(text), left: false }
    }

    pub fn update(&mut self, edit: Edit) {
        match edit {
            Edit::Typed(s) => self.text = self.mask.shape(&s),
            Edit::Left => self.left = true,
        }
    }

    pub fn problem(&self) -> Option<&'static str> {
        self.left.then(|| self.mask.problem(&self.text)).flatten()
    }

    pub fn view<M: 'static>(&self, tip: impl Into<String>, wrap: impl Fn(Edit) -> M + Clone + 'static) -> Node<M> {
        let typed = wrap.clone();
        let input = entry(self.text.clone(), tip, move |s| typed(Edit::Typed(s))).on_leave(wrap(Edit::Left)).width(Length::Fill);
        let under = match self.problem() {
            Some(words) => label(words).tone(Tone::Problem),
            None => label(""),
        };
        column(vec![input, under.room_for(1)]).gap(crate::theme::gaps().small)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_mask_drops_what_cannot_be_in_it_and_adds_its_marks() {
        assert_eq!(Mask::Phone.shape("+44 (0)7700-900123x+"), "+44 (0)7700-900123");
        assert_eq!(Mask::Date.shape("2026/10-30x"), "202610-30");
        assert_eq!(Mask::Date.shape("2026-10-301"), "2026-10-30");
        assert_eq!(Mask::Hex.shape("#FBC711zz"), "#FBC711");
        assert_eq!(Mask::Hex.shape("fb#c"), "fbc");
    }

    #[test]
    fn a_problem_is_said_only_for_what_is_wrong() {
        assert_eq!(Mask::Phone.problem("07700 900123"), None);
        assert_eq!(Mask::Phone.problem("123"), Some("Enter a phone number with 7 to 15 digits."));
        assert_eq!(Mask::Date.problem("2026-02-30"), Some("Enter a date as year-month-day, such as 2026-10-30."));
        assert_eq!(Mask::Date.problem("2024-02-29"), None);
        assert_eq!((Mask::Hex.problem("#fff"), Mask::Hex.problem("fbc711cc")), (None, None));
        assert!(Mask::Hex.problem("#fbc7").is_some());
        assert_eq!(Mask::Hex.problem(""), None);
    }

    #[test]
    fn a_half_typed_box_is_not_called_wrong_until_it_is_left() {
        let mut b = CheckedBox::new(Mask::Date, "");
        b.update(Edit::Typed("2026-1".into()));
        assert_eq!(b.problem(), None);
        b.update(Edit::Left);
        assert!(b.problem().is_some());
        b.update(Edit::Typed("2026-10-30".into()));
        assert_eq!(b.problem(), None);
    }
}
