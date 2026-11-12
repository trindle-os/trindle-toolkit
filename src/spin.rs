//! Numbers stepped up and down: a number box that is typed in as well (`Spin`), a plain spin box, a time as two of them, and a month and year picked from a grid.

use std::rc::Rc;

use super::dial::{snap, value_after_key};
use super::role::Role;
use super::when::{month_name, Time};
use super::widget::{button, column, drawing, entry, even_grid, label, note, row, word_button, Length, Node, Tone, Words};
use crate::t;

#[derive(Clone, Debug, PartialEq)]
pub struct Spin {
    pub value: f64,
    pub min: f64,
    pub max: f64,
    pub step: f64,
    pub unit: String,
    pub decimals: Option<usize>,
    pub latin: bool,
    pub typed: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SpinMsg {
    Typed(String),
    Done,
    Step(i32),
}

impl Spin {
    pub fn new(value: f64, min: f64, max: f64, step: f64) -> Self {
        let (min, max) = if min <= max { (min, max) } else { (max, min) };
        let mut s = Spin { value: 0.0, min, max, step, unit: String::new(), decimals: None, latin: false, typed: None };
        s.value = s.fit(value);
        s
    }

    pub fn unit(mut self, unit: impl Into<String>) -> Self {
        self.unit = unit.into();
        self
    }

    pub fn decimals(mut self, places: usize) -> Self {
        self.decimals = Some(places);
        self.value = self.fit(self.value);
        self
    }

    pub fn latin(mut self) -> Self {
        self.latin = true;
        self
    }

    fn fit(&self, v: f64) -> f64 {
        let v = v.clamp(self.min, self.max);
        if self.step <= 0.0 {
            return v;
        }
        let snapped = self.min + ((v - self.min) / self.step).round() * self.step;
        let scale = 10f64.powi(self.decimals.map_or(6, |d| d.min(6)) as i32);
        ((snapped * scale).round() / scale).clamp(self.min, self.max)
    }

    pub fn text(&self) -> String {
        let places = self.decimals.unwrap_or_else(|| format!("{}", self.step).split_once('.').map_or(0, |(_, f)| f.len()));
        let plain = crate::lang::format::marked(self.value, places);
        if self.latin { plain } else { crate::lang::format::digits(&plain) }
    }

    fn read(&self) -> Option<f64> {
        let text = self.typed.as_deref()?;
        let (group, mark) = (crate::t!("kit-number-group"), crate::t!("kit-number-decimal"));
        match text.trim().split_once(group).filter(|(_, after)| !group.trim().is_empty() && !text.contains(mark) && text.matches(group).count() == 1 && !after.is_empty() && after.chars().all(char::is_alphanumeric) && after.chars().count() != 3) {
            Some((before, after)) => crate::numbers::read(&format!("{before}{mark}{after}")),
            None => crate::numbers::read(text),
        }
    }

    pub fn update(&mut self, m: SpinMsg) -> bool {
        let was = self.value;
        match m {
            SpinMsg::Typed(t) => self.typed = Some(t),
            SpinMsg::Done => {
                if let Some(v) = self.read() {
                    self.value = self.fit(v);
                    self.typed = None;
                }
            }
            SpinMsg::Step(n) => {
                let from = self.read().unwrap_or(self.value);
                self.value = self.fit(from + f64::from(n) * self.step);
                self.typed = None;
            }
        }
        self.value != was
    }

    pub fn node<M: 'static>(&self, tip: &str, wrap: impl Fn(SpinMsg) -> M + Clone + 'static) -> Node<M> {
        let (w1, w2, w3) = (wrap.clone(), wrap.clone(), wrap.clone());
        let shown = self.typed.clone().unwrap_or_else(|| self.text());
        let mut line = vec![
            entry(shown, tip, move |t| w1(SpinMsg::Typed(t)))
                .on_submit(wrap(SpinMsg::Done))
                .on_leave(w2(SpinMsg::Done))
                .on_step(move |n| w3(SpinMsg::Step(n)))
                .role(Role::SpinButton { value: self.value, min: self.min, max: self.max })
                .width(Length::Fill),
        ];
        if !self.unit.is_empty() {
            line.push(label(self.unit.clone()).width(Length::Shrink));
        }
        let (down, up) = (wrap.clone(), wrap.clone());
        line.push(button("list-remove-symbolic", t!("kit-spin-less", what = tip)).on_press(down(SpinMsg::Step(-1))).off(self.value <= self.min));
        line.push(button("list-add-symbolic", t!("kit-spin-more", what = tip)).on_press(up(SpinMsg::Step(1))).off(self.value >= self.max));
        let line = row(line);
        if self.typed.is_some() && !self.read().is_some_and(|v| (self.min..=self.max).contains(&v)) {
            let why = t!("kit-spin-not-number", min = self.min_text().as_str(), max = self.max_text().as_str());
            column(vec![line, note(why).tone(Tone::Problem)])
        } else {
            line
        }
    }

    fn min_text(&self) -> String {
        Spin { value: self.min, ..self.clone() }.text()
    }

    fn max_text(&self) -> String {
        Spin { value: self.max, ..self.clone() }.text()
    }
}

const NUMBER_WIDTH: f32 = 56.0;

pub fn number_text(value: f32, step: f32) -> String {
    let places = (0..3).find(|&p| (step * 10f32.powi(p)).fract().abs() < 1e-4).unwrap_or(3) as usize;
    crate::lang::format::decimal(f64::from(value), places)
}

pub fn spin_with<M: 'static>(value: f32, min: f32, max: f32, step: f32, tip: impl Into<String>, text: impl Fn(f32) -> String, change: impl Fn(f32) -> M + 'static) -> Node<M> {
    let tip = tip.into();
    let change = Rc::new(change);
    let (by_key, by_wheel) = (change.clone(), change.clone());
    let words = text(value);
    let size = (NUMBER_WIDTH, super::theme::sheet().entry_height);
    let number = drawing(size, tip.clone(), move |canvas, pen| {
        let style = Words::Plain.style(pen.scale);
        let layout = pen.fonts.layout(&words, &style, None);
        let x = ((canvas.width() as f32 - layout.width) / 2.0).max(0.0);
        let y = (canvas.height() as f32 - layout.height) / 2.0;
        pen.fonts.draw(canvas, &layout, x, y, pen.theme.fg);
    })
    .on_keys(move |input| value_after_key(input, value, min, max, step).map(|v| by_key(v)))
    .on_scroll(move |lines| by_wheel(snap(value - if lines == 0.0 { 0.0 } else { lines.signum() } * step, min, max, step)))
    .role(Role::Slider { value: f64::from(value), min: f64::from(min), max: f64::from(max) });
    row(vec![
        button("list-remove-symbolic", t!("kit-spin-less", what = tip.as_str())).on_press(change(snap(value - step, min, max, step))).off(value <= min),
        number,
        button("list-add-symbolic", t!("kit-spin-more", what = tip.as_str())).on_press(change(snap(value + step, min, max, step))).off(value >= max),
    ]).gap(crate::theme::gaps().small).width(Length::Shrink)
}

pub fn spin<M: 'static>(value: f32, min: f32, max: f32, step: f32, tip: impl Into<String>, change: impl Fn(f32) -> M + 'static) -> Node<M> {
    spin_with(value, min, max, step, tip, move |v| number_text(v, step), change)
}

pub fn time_spin<M: 'static>(time: Time, change: impl Fn(Time) -> M + 'static) -> Node<M> {
    let change = Rc::new(change);
    let (h, m) = (change.clone(), change);
    let two = |v: f32| crate::lang::format::digits(&format!("{:02}", v as u32));
    row(vec![
        spin_with(time.hour as f32, 0.0, 23.0, 1.0, t!("kit-time-hours"), two, move |v| h(Time { hour: v as u32, ..time })),
        label(":"),
        spin_with(time.minute as f32, 0.0, 59.0, 1.0, t!("kit-time-minutes"), two, move |v| m(Time { minute: v as u32, ..time })),
    ]).gap(crate::theme::gaps().medium).width(Length::Shrink)
}

pub fn month_year<M: 'static>(year: i32, month: u32, change: impl Fn(i32, u32) -> M + 'static) -> Node<M> {
    let change = Rc::new(change);
    let by_year = change.clone();
    let months = (1..=12).map(|m| word_button(month_name(m)).on(m == month).on_press(change(year, m))).collect();
    column(vec![
        spin_with(year as f32, 1.0, 9999.0, 1.0, t!("kit-year"), |v| crate::lang::format::digits(&format!("{}", v as i32)), move |v| by_year(v as i32, month)),
        even_grid(96.0, months).gap(crate::theme::gaps().medium),
    ]).gap(crate::theme::gaps().roomy).width(Length::Shrink)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lang::{with_catalogue, Bundle, Catalogue, KIT_ENGLISH};

    #[test]
    fn a_number_has_the_places_its_step_has() {
        assert_eq!(number_text(3.0, 1.0), "3");
        assert_eq!(number_text(0.5, 0.1), "0.5");
        assert_eq!(number_text(0.25, 0.05), "0.25");
    }

    fn language(text: &str) -> Catalogue {
        Catalogue::new(Bundle::from_text("xx", text), Bundle::from_files("en", KIT_ENGLISH))
    }

    #[test]
    fn numbers_are_shown_and_read_the_languages_way() {
        let german = "kit-number-decimal = ,\nkit-number-group = .";
        with_catalogue(language(german), || {
            let mut s = Spin::new(3.5, 0.0, 5000.0, 0.5);
            assert_eq!(s.text(), "3,5");
            assert_eq!(s.clone().latin().text(), "3,5", "0 to 9 with the language's mark");
            assert_eq!(number_text(0.5, 0.1), "0,5");
            s.update(SpinMsg::Typed("1.234,5".into()));
            s.update(SpinMsg::Done);
            assert_eq!(s.value, 1234.5);
            s.update(SpinMsg::Typed("2,5".into()));
            assert!(s.update(SpinMsg::Step(1)), "a step starts from what is typed");
            assert_eq!(s.value, 3.0, "2,5 is two and a half in German");
        });
        let english = Spin::new(1.0, 0.0, 5000.0, 0.5);
        with_catalogue(language(""), || {
            let mut s = english.clone();
            s.update(SpinMsg::Typed("1,234.5".into()));
            s.update(SpinMsg::Done);
            assert_eq!(s.value, 1234.5);
            assert_eq!(s.text(), "1234.5");
            s.update(SpinMsg::Typed("1,234".into()));
            s.update(SpinMsg::Done);
            assert_eq!(s.value, 1234.0, "three figures after the comma: a thousands mark");
            s.update(SpinMsg::Typed("3,4".into()));
            s.update(SpinMsg::Done);
            assert_eq!(s.value, 3.5, "a comma that cannot group is the point, rounded to the step");
        });
        let arabic = "kit-number-digits = ٠١٢٣٤٥٦٧٨٩\nkit-number-decimal = ٫\nkit-number-group = ٬";
        with_catalogue(language(arabic), || {
            let mut s = english.clone();
            assert_eq!(s.text(), "١٫٠");
            s.update(SpinMsg::Typed("٣٫٥".into()));
            s.update(SpinMsg::Done);
            assert_eq!(s.value, 3.5);
        });
    }

    #[test]
    fn the_buttons_step_and_grey_out_at_the_ends() {
        let s = spin(0.0, 0.0, 10.0, 2.0, "Copies", |v| v);
        let presses: Vec<(String, bool, f32)> = s.presses().into_iter().map(|(t, off, m)| (t, off, *m)).collect();
        assert_eq!(presses, vec![("Decrease Copies".into(), true, 0.0), ("Increase Copies".into(), false, 2.0)]);
        let t = time_spin(Time { hour: 23, minute: 5 }, |t| t);
        let sent: Vec<Time> = t.presses().into_iter().map(|(_, _, m)| *m).collect();
        assert_eq!(sent, vec![Time { hour: 22, minute: 5 }, Time { hour: 23, minute: 5 }, Time { hour: 23, minute: 4 }, Time { hour: 23, minute: 6 }]);
    }

    #[test]
    fn a_month_is_chosen_in_the_shown_year() {
        let g = month_year(2026, 10, |y, m| (y, m));
        let sent: Vec<(i32, u32)> = g.presses().into_iter().map(|(_, _, m)| *m).collect();
        assert_eq!(&sent[..3], &[(2025, 10), (2027, 10), (2026, 1)]);
        assert_eq!(sent.len(), 14);
    }
}
