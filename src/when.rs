//! A date to pick from a month grid and a time to type, each kept in a small state the program holds. The grid starts the week on the day the language says (`kit-date-first-weekday`) and writes its own figures.

use super::widget::{column, entry, form_label, label, note, row, word_button, Length, Node};
use crate::lang::format::{self, Moment, Pattern};
use crate::lang::numerals::latin_digits;
use crate::t;

const CELL: f32 = 40.0;
const MOST_YEAR: i32 = 9999;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Date {
    pub year: i32,
    pub month: u32,
    pub day: u32,
}

impl Date {
    pub fn new(year: i32, month: u32, day: u32) -> Option<Date> {
        ((1..=MOST_YEAR).contains(&year) && (1..=12).contains(&month) && (1..=days_in_month(year, month)).contains(&day)).then_some(Date { year, month, day })
    }

    fn days(self) -> i32 {
        let (y, m) = if self.month <= 2 { (self.year - 1, self.month + 9) } else { (self.year, self.month - 3) };
        let era = y.div_euclid(400);
        let yoe = y.rem_euclid(400);
        let doy = (153 * m as i32 + 2) / 5 + self.day as i32 - 1;
        era * 146_097 + yoe * 365 + yoe / 4 - yoe / 100 + doy - 719_468
    }

    pub fn weekday(self) -> u32 {
        (self.days() + 3).rem_euclid(7) as u32
    }

    pub fn week(self) -> (i32, u32) {
        iso_week(self.days() - self.weekday() as i32 + 3, self.year)
    }

    pub fn iso(self) -> String {
        format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }

    pub fn parse(text: &str) -> Option<Date> {
        let text = latin_digits(text.trim());
        let b = text.as_bytes();
        let shaped = b.len() == 10 && b[4] == b'-' && b[7] == b'-' && b.iter().enumerate().all(|(i, c)| i == 4 || i == 7 || c.is_ascii_digit());
        let n = |from: usize, to: usize| std::str::from_utf8(&b[from..to]).ok()?.parse::<u32>().ok();
        if !shaped {
            return None;
        }
        Date::new(n(0, 4)? as i32, n(5, 7)?, n(8, 10)?)
    }

    pub fn written(self, kind: Pattern) -> String {
        let m = Moment { year: self.year, month: self.month, day: self.day, weekday: (self.weekday() + 1) % 7, hour: 0, minute: 0, second: 0 };
        format::digits(&format::date(&m, kind))
    }
}

pub fn first_weekday() -> u32 {
    t!("kit-date-first-weekday").trim().parse::<u32>().ok().filter(|d| (1..=7).contains(d)).map_or(0, |d| d - 1)
}

fn iso_week(thursday: i32, near: i32) -> (i32, u32) {
    let jan1 = |year| Date { year, month: 1, day: 1 }.days();
    let year = [near + 1, near].into_iter().find(|&y| jan1(y) <= thursday).unwrap_or(near - 1);
    (year, ((thursday - jan1(year)) / 7 + 1) as u32)
}

pub fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        2 if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        _ => 0,
    }
}

pub(crate) fn month_name(month: u32) -> &'static str {
    match month {
        1 => t!("kit-month-1"),
        2 => t!("kit-month-2"),
        3 => t!("kit-month-3"),
        4 => t!("kit-month-4"),
        5 => t!("kit-month-5"),
        6 => t!("kit-month-6"),
        7 => t!("kit-month-7"),
        8 => t!("kit-month-8"),
        9 => t!("kit-month-9"),
        10 => t!("kit-month-10"),
        11 => t!("kit-month-11"),
        _ => t!("kit-month-12"),
    }
}

pub(crate) fn month_in_date(month: u32) -> &'static str {
    match month {
        1 => t!("kit-month-in-date-1"),
        2 => t!("kit-month-in-date-2"),
        3 => t!("kit-month-in-date-3"),
        4 => t!("kit-month-in-date-4"),
        5 => t!("kit-month-in-date-5"),
        6 => t!("kit-month-in-date-6"),
        7 => t!("kit-month-in-date-7"),
        8 => t!("kit-month-in-date-8"),
        9 => t!("kit-month-in-date-9"),
        10 => t!("kit-month-in-date-10"),
        11 => t!("kit-month-in-date-11"),
        _ => t!("kit-month-in-date-12"),
    }
}

fn weekday_name(day: u32) -> &'static str {
    match day {
        0 => t!("kit-weekday-1"),
        1 => t!("kit-weekday-2"),
        2 => t!("kit-weekday-3"),
        3 => t!("kit-weekday-4"),
        4 => t!("kit-weekday-5"),
        5 => t!("kit-weekday-6"),
        _ => t!("kit-weekday-7"),
    }
}

fn weekday_full(day: u32) -> &'static str {
    match day {
        0 => t!("kit-weekday-full-1"),
        1 => t!("kit-weekday-full-2"),
        2 => t!("kit-weekday-full-3"),
        3 => t!("kit-weekday-full-4"),
        4 => t!("kit-weekday-full-5"),
        5 => t!("kit-weekday-full-6"),
        _ => t!("kit-weekday-full-7"),
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Pick {
    Day(Date),
    Month(i32),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DateBox {
    pub chosen: Option<Date>,
    pub shown: (i32, u32),
    pub weeks: bool,
}

impl DateBox {
    pub fn new(chosen: Option<Date>, today: Date) -> DateBox {
        let at = chosen.unwrap_or(today);
        DateBox { chosen, shown: (at.year.clamp(1, MOST_YEAR), at.month.clamp(1, 12)), weeks: false }
    }

    pub fn update(&mut self, pick: Pick) {
        match pick {
            Pick::Day(d) => self.chosen = Some(d),
            Pick::Month(by) => {
                let n = (self.shown.0 * 12 + self.shown.1 as i32 - 1).saturating_add(by).clamp(12, MOST_YEAR * 12 + 11);
                self.shown = (n.div_euclid(12), n.rem_euclid(12) as u32 + 1);
            }
        }
    }

    pub fn view<M: 'static>(&self, wrap: impl Fn(Pick) -> M + Clone + 'static) -> Node<M> {
        let chosen = self.chosen;
        month_grid(self.shown, self.weeks, move |d| chosen == Some(d), wrap)
    }
}

fn month_grid<M: 'static>(shown: (i32, u32), weeks: bool, on: impl Fn(Date) -> bool, wrap: impl Fn(Pick) -> M + Clone + 'static) -> Node<M> {
    let (year, month) = shown;
    let first = first_weekday();
    let wide = if weeks { CELL * 7.0 / 8.0 } else { CELL };
    let cell = |n: Node<M>| n.width(Length::Fixed(wide));
    let (back, forward) = (wrap.clone(), wrap.clone());
    let mut rows = vec![
        row(vec![
            word_button(t!("kit-date-previous")).on_press(back(Pick::Month(-1))),
            label(t!("kit-date-month-year", month = month_name(month), year = format::digits(&year.to_string()))).width(Length::Fill),
            word_button(t!("kit-date-next")).on_press(forward(Pick::Month(1))),
        ]).gap(crate::theme::gaps().medium),
        row(weeks.then(|| cell(form_label(t!("kit-date-week-short")))).into_iter()
            .chain((0..7).map(|d| cell(form_label(weekday_name((first + d) % 7))))).collect()).gap(0.0),
    ];
    let one = Date { year, month, day: 1 };
    let lead = (one.weekday() + 7 - first) % 7;
    let thursday = one.days() - lead as i32 + ((10 - first) % 7) as i32;
    let mut days = std::iter::repeat_n(None, lead as usize).collect::<Vec<_>>();
    days.extend((1..=days_in_month(year, month)).map(Some));
    for (k, week) in days.chunks(7).enumerate() {
        let mut cells: Vec<Node<M>> = Vec::new();
        if weeks {
            let n = iso_week(thursday + 7 * k as i32, year).1;
            cells.push(cell(note(format::digits(&n.to_string())).called(t!("kit-date-week", week = n as i64))));
        }
        cells.extend(week.iter().map(|day| match day.map(|day| Date { year, month, day }) {
            Some(date) => cell(word_button(format::digits(&date.day.to_string()))
                .on(on(date))
                .spoken(t!("kit-date-full", weekday = weekday_full(date.weekday()), day = date.day as i64, month = month_in_date(month), year = year as i64))
                .on_press(wrap(Pick::Day(date)))),
            None => cell(label("")),
        }));
        cells.extend((week.len()..7).map(|_| cell(label(""))));
        rows.push(row(cells).gap(0.0));
    }
    column(rows).gap(crate::theme::gaps().small).width(Length::Shrink)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RangeBox {
    pub from: Option<Date>,
    pub to: Option<Date>,
    pub shown: (i32, u32),
    pub weeks: bool,
}

impl RangeBox {
    pub fn new(today: Date) -> RangeBox {
        RangeBox { from: None, to: None, shown: DateBox::new(None, today).shown, weeks: false }
    }

    pub fn update(&mut self, pick: Pick) {
        match (pick, self.from, self.to) {
            (Pick::Day(d), Some(from), None) if d >= from => self.to = Some(d),
            (Pick::Day(d), _, _) => (self.from, self.to) = (Some(d), None),
            (Pick::Month(by), _, _) => {
                let mut b = DateBox { chosen: None, shown: self.shown, weeks: false };
                b.update(Pick::Month(by));
                self.shown = b.shown;
            }
        }
    }

    pub fn view<M: 'static>(&self, wrap: impl Fn(Pick) -> M + Clone + 'static) -> Node<M> {
        let (from, to) = (self.from, self.to);
        month_grid(self.shown, self.weeks, move |d| from.is_some_and(|f| d >= f && d <= to.unwrap_or(f)), wrap)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Time {
    pub hour: u32,
    pub minute: u32,
}

impl Time {
    pub fn new(hour: u32, minute: u32) -> Option<Time> {
        (hour < 24 && minute < 60).then_some(Time { hour, minute })
    }

    pub fn text(self) -> String {
        format!("{:02}:{:02}", self.hour, self.minute)
    }

    pub fn written(self) -> String {
        let m = Moment { year: 2000, month: 1, day: 1, weekday: 6, hour: self.hour, minute: self.minute, second: 0 };
        format::digits(&format::date(&m, Pattern::Clock))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Typed {
    Hours(String),
    Minutes(String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct TimeBox {
    pub hours: String,
    pub minutes: String,
}

impl TimeBox {
    pub fn new(time: Time) -> TimeBox {
        TimeBox { hours: format!("{:02}", time.hour), minutes: format!("{:02}", time.minute) }
    }

    pub fn update(&mut self, typed: Typed) {
        let figures = |s: String| latin_digits(&s).chars().filter(char::is_ascii_digit).take(2).collect::<String>();
        match typed {
            Typed::Hours(s) => self.hours = figures(s),
            Typed::Minutes(s) => self.minutes = figures(s),
        }
    }

    pub fn time(&self) -> Option<Time> {
        Time::new(self.hours.parse().ok()?, self.minutes.parse().ok()?)
    }

    pub fn view<M: 'static>(&self, wrap: impl Fn(Typed) -> M + Clone + 'static) -> Node<M> {
        let (h, m) = (wrap.clone(), wrap);
        let box_width = Length::Fixed(64.0);
        row(vec![
            entry(format::digits(&self.hours), t!("kit-time-hours"), move |s| h(Typed::Hours(s))).width(box_width),
            label(":"),
            entry(format::digits(&self.minutes), t!("kit-time-minutes"), move |s| m(Typed::Minutes(s))).width(box_width),
        ]).gap(crate::theme::gaps().medium).width(Length::Shrink)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weekdays_months_and_leap_years_are_right() {
        assert_eq!(Date::new(2026, 10, 25).unwrap().weekday(), 6);
        assert_eq!(Date::new(2024, 2, 29).unwrap().weekday(), 3);
        assert_eq!(Date::new(1970, 1, 1).unwrap().weekday(), 3);
        assert!(Date::new(2023, 2, 29).is_none() && Date::new(2000, 2, 29).is_some() && Date::new(1900, 2, 29).is_none());
        assert_eq!(Date::parse("2026-10-25").map(Date::iso).as_deref(), Some("2026-10-25"));
        assert!(Date::parse("2026-13-01").is_none());
        for loose in ["+2026-+1-+5", "2026-1-5", "2026-10-+5", "2026/10/25", " 2026-10-2 "] {
            assert!(Date::parse(loose).is_none(), "{loose}");
        }
        assert_eq!((days_in_month(2026, 0), days_in_month(2026, 13), days_in_month(2026, 12)), (0, 0, 31));
        assert!(Date::parse("99999999-01-01").is_none() && Date::new(0, 1, 1).is_none() && Date::new(9999, 12, 31).is_some());
        let mut b = DateBox::new(None, Date::new(9999, 12, 1).unwrap());
        b.update(Pick::Month(i32::MAX));
        assert_eq!(b.shown, (9999, 12));
        b.update(Pick::Month(i32::MIN));
        assert_eq!(b.shown, (1, 1));
    }

    #[test]
    fn a_whole_date_takes_the_month_as_it_reads_in_a_date() {
        use crate::lang::{with_catalogue, Bundle, Catalogue, KIT_ENGLISH};
        let russian = Catalogue::new(Bundle::from_text("xx", "kit-month-11 = ноябрь\nkit-month-in-date-11 = ноября"), Bundle::from_files("en", KIT_ENGLISH));
        with_catalogue(russian, || {
            assert_eq!(month_name(11), "ноябрь");
            assert_eq!(month_in_date(11), "ноября");
        });
        with_catalogue(Catalogue::new(Bundle::default(), Bundle::from_files("en", KIT_ENGLISH)), || {
            assert_eq!(month_in_date(11), month_name(11));
        });
    }

    #[test]
    fn months_step_across_years_and_a_day_is_chosen() {
        let mut b = DateBox::new(None, Date::new(2026, 12, 5).unwrap());
        b.update(Pick::Month(1));
        assert_eq!(b.shown, (2027, 1));
        b.update(Pick::Month(-2));
        assert_eq!(b.shown, (2026, 11));
        b.update(Pick::Day(Date::new(2026, 11, 3).unwrap()));
        assert_eq!(b.chosen.map(Date::iso).as_deref(), Some("2026-11-03"));
    }

    #[test]
    fn the_grid_starts_the_week_and_writes_figures_as_the_language_does() {
        use crate::lang::{with_catalogue, Bundle, Catalogue, KIT_ENGLISH};
        let english = || Catalogue::new(Bundle::default(), Bundle::from_files("en", KIT_ENGLISH));
        let sunday_first = || Catalogue::new(Bundle::from_text("xx", "kit-date-first-weekday = 7\nkit-number-digits = ٠١٢٣٤٥٦٧٨٩"), Bundle::from_files("en", KIT_ENGLISH));
        let grid = || DateBox::new(None, Date::new(2026, 11, 1).unwrap()).view(|p| p).said();
        with_catalogue(english(), || {
            let said = grid();
            let at = said.iter().position(|w| w == "Mon").unwrap();
            assert_eq!(said[at + 6], "Sun");
            assert_eq!(said[at + 7..at + 13].iter().filter(|w| w.is_empty()).count(), 6);
            assert_eq!(Date::new(2026, 11, 1).unwrap().written(Pattern::Full), "01 November 2026");
            assert_eq!(Time::new(7, 5).unwrap().written(), "07:05");
        });
        with_catalogue(sunday_first(), || {
            assert_eq!(first_weekday(), 6);
            let said = grid();
            let at = said.iter().position(|w| w == "Sun").unwrap();
            assert_eq!(said[at + 6], "Sat");
            assert_eq!(said[at + 7], "button: ١", "the first falls on the first column, in the language's figures");
            assert!(said.iter().any(|w| w.contains("٢٠٢٦")));
            assert_eq!(Time::new(7, 5).unwrap().written(), "٠٧:٠٥");
        });
    }

    #[test]
    fn a_range_takes_two_ends_in_order_and_starts_again() {
        let day = |d| Date::new(2026, 10, d).unwrap();
        let mut r = RangeBox::new(day(1));
        r.update(Pick::Day(day(10)));
        assert_eq!((r.from, r.to), (Some(day(10)), None));
        r.update(Pick::Day(day(4)));
        assert_eq!((r.from, r.to), (Some(day(4)), None));
        r.update(Pick::Day(day(12)));
        assert_eq!((r.from, r.to), (Some(day(4)), Some(day(12))));
        r.update(Pick::Month(1));
        assert_eq!(r.shown, (2026, 11));
        r.update(Pick::Day(day(20)));
        assert_eq!((r.from, r.to), (Some(day(20)), None));
        r.update(Pick::Day(day(20)));
        assert_eq!((r.from, r.to), (Some(day(20)), Some(day(20))));
    }

    #[test]
    fn iso_weeks_cross_years_as_the_standard_says() {
        let week = |y, m, d| Date::new(y, m, d).unwrap().week();
        assert_eq!(week(2026, 10, 30), (2026, 44));
        assert_eq!(week(2021, 1, 3), (2020, 53), "a Sunday early in January can belong to the year before");
        assert_eq!(week(2024, 12, 30), (2025, 1), "a Monday late in December can belong to the year after");
        assert_eq!(week(2026, 1, 1), (2026, 1));
        assert_eq!(week(2027, 1, 1), (2026, 53));
        assert_eq!(week(9999, 12, 31), (9999, 52));
        assert_eq!(week(1, 1, 1), (1, 1));
    }

    #[test]
    fn week_numbers_run_down_the_grid_and_are_read_as_weeks() {
        use crate::lang::{with_catalogue, Bundle, Catalogue, KIT_ENGLISH};
        let english = || Catalogue::new(Bundle::default(), Bundle::from_files("en", KIT_ENGLISH));
        let sunday_first = || Catalogue::new(Bundle::from_text("xx", "kit-date-first-weekday = 7"), Bundle::from_files("en", KIT_ENGLISH));
        let weeks = |n: Node<Pick>| {
            let mut ui = crate::ui::Ui::new(n, crate::theme::Theme::new(false, "#fbc711"));
            ui.lay_at(600, 600);
            ui.access("T").nodes.iter().filter_map(|(_, p)| p.label().filter(|l| l.starts_with("Week")).map(String::from)).collect::<Vec<_>>()
        };
        let mut b = DateBox::new(None, Date::new(2026, 12, 1).unwrap());
        with_catalogue(english(), || {
            assert!(weeks(b.view(|p| p)).is_empty(), "off unless asked for");
            b.weeks = true;
            assert!(b.view(|p| p).said().iter().any(|w| w == "Wk"));
            assert_eq!(weeks(b.view(|p| p)), ["Week 49", "Week 50", "Week 51", "Week 52", "Week 53"]);
        });
        with_catalogue(sunday_first(), || {
            assert_eq!(weeks(b.view(|p| p))[0], "Week 49");
        });
        let mut r = RangeBox::new(Date::new(2027, 1, 1).unwrap());
        r.weeks = true;
        with_catalogue(english(), || assert_eq!(weeks(r.view(|p| p))[0], "Week 53"));
    }

    #[test]
    fn the_time_box_keeps_two_figures_and_checks_the_range() {
        let mut t = TimeBox::new(Time::new(7, 5).unwrap());
        assert_eq!(t.time().map(Time::text).as_deref(), Some("07:05"));
        t.update(Typed::Hours("2a45".into()));
        assert_eq!(t.hours, "24");
        assert!(t.time().is_none());
        t.update(Typed::Minutes(String::new()));
        assert!(t.time().is_none());
    }
}
