//! A length of time typed as hours, minutes and seconds in three small boxes.

use super::widget::{entry, label, row, Length, Node};
use crate::lang::format::digits;
use crate::lang::numerals::latin_digits;
use crate::t;

#[derive(Clone, Debug, PartialEq)]
pub enum Typed {
    Hours(String),
    Minutes(String),
    Seconds(String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct DurationBox {
    pub hours: String,
    pub minutes: String,
    pub seconds: String,
}

impl DurationBox {
    pub fn new(total: u64) -> DurationBox {
        DurationBox { hours: (total / 3600).to_string(), minutes: format!("{:02}", total / 60 % 60), seconds: format!("{:02}", total % 60) }
    }

    pub fn update(&mut self, typed: Typed) {
        let figures = |s: String, most| latin_digits(&s).chars().filter(char::is_ascii_digit).take(most).collect::<String>();
        match typed {
            Typed::Hours(s) => self.hours = figures(s, 4),
            Typed::Minutes(s) => self.minutes = figures(s, 2),
            Typed::Seconds(s) => self.seconds = figures(s, 2),
        }
    }

    pub fn total(&self) -> Option<u64> {
        let (h, m, s): (u64, u64, u64) = (self.hours.parse().ok()?, self.minutes.parse().ok()?, self.seconds.parse().ok()?);
        (m < 60 && s < 60).then_some(h * 3600 + m * 60 + s)
    }

    pub fn view<M: 'static>(&self, wrap: impl Fn(Typed) -> M + Clone + 'static) -> Node<M> {
        let (h, m, s) = (wrap.clone(), wrap.clone(), wrap);
        row(vec![
            entry(digits(&self.hours), t!("kit-duration-hours"), move |x| h(Typed::Hours(x))).width(Length::Fixed(72.0)),
            label(":"),
            entry(digits(&self.minutes), t!("kit-duration-minutes"), move |x| m(Typed::Minutes(x))).width(Length::Fixed(64.0)),
            label(":"),
            entry(digits(&self.seconds), t!("kit-duration-seconds"), move |x| s(Typed::Seconds(x))).width(Length::Fixed(64.0)),
        ]).gap(crate::theme::gaps().medium).width(Length::Shrink)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_duration_is_read_in_seconds_and_range_checked() {
        let mut d = DurationBox::new(3725);
        assert_eq!((d.hours.as_str(), d.minutes.as_str(), d.seconds.as_str()), ("1", "02", "05"));
        assert_eq!(d.total(), Some(3725));
        d.update(Typed::Minutes("6x0".into()));
        assert_eq!((d.minutes.as_str(), d.total()), ("60", None));
        d.update(Typed::Minutes("59".into()));
        d.update(Typed::Hours("123456".into()));
        assert_eq!((d.hours.as_str(), d.total()), ("1234", Some(1234 * 3600 + 59 * 60 + 5)));
        d.update(Typed::Seconds(String::new()));
        assert_eq!(d.total(), None);
    }
}
