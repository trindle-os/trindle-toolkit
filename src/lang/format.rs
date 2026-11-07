//! Dates, times, numbers and sizes written the way a language writes them.

use crate::t;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Moment {
    pub year: i32,
    pub month: u32,
    pub day: u32,
    pub weekday: u32,
    pub hour: u32,
    pub minute: u32,
    pub second: u32,
}

pub fn weekday_name(weekday: u32) -> &'static str {
    match weekday % 7 {
        0 => t!("kit-date-weekday-sun"),
        1 => t!("kit-date-weekday-mon"),
        2 => t!("kit-date-weekday-tue"),
        3 => t!("kit-date-weekday-wed"),
        4 => t!("kit-date-weekday-thu"),
        5 => t!("kit-date-weekday-fri"),
        _ => t!("kit-date-weekday-sat"),
    }
}

pub fn weekday_short(weekday: u32) -> &'static str {
    match weekday % 7 {
        0 => t!("kit-date-weekday-short-sun"),
        1 => t!("kit-date-weekday-short-mon"),
        2 => t!("kit-date-weekday-short-tue"),
        3 => t!("kit-date-weekday-short-wed"),
        4 => t!("kit-date-weekday-short-thu"),
        5 => t!("kit-date-weekday-short-fri"),
        _ => t!("kit-date-weekday-short-sat"),
    }
}

pub fn month_name(month: u32) -> &'static str {
    match (month % 12 + 11) % 12 {
        0 => t!("kit-date-month-jan"),
        1 => t!("kit-date-month-feb"),
        2 => t!("kit-date-month-mar"),
        3 => t!("kit-date-month-apr"),
        4 => t!("kit-date-month-may"),
        5 => t!("kit-date-month-jun"),
        6 => t!("kit-date-month-jul"),
        7 => t!("kit-date-month-aug"),
        8 => t!("kit-date-month-sep"),
        9 => t!("kit-date-month-oct"),
        10 => t!("kit-date-month-nov"),
        _ => t!("kit-date-month-dec"),
    }
}

pub fn month_short(month: u32) -> &'static str {
    match (month % 12 + 11) % 12 {
        0 => t!("kit-date-month-short-jan"),
        1 => t!("kit-date-month-short-feb"),
        2 => t!("kit-date-month-short-mar"),
        3 => t!("kit-date-month-short-apr"),
        4 => t!("kit-date-month-short-may"),
        5 => t!("kit-date-month-short-jun"),
        6 => t!("kit-date-month-short-jul"),
        7 => t!("kit-date-month-short-aug"),
        8 => t!("kit-date-month-short-sep"),
        9 => t!("kit-date-month-short-oct"),
        10 => t!("kit-date-month-short-nov"),
        _ => t!("kit-date-month-short-dec"),
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Pattern {
    Clock,
    DayMonth,
    DayMonthPadded,
    Full,
    WeekdayClock,
    ShortDay,
    ShortDayClock,
    DayShort,
    DayShortYear,
    DayShortClock,
    DayLong,
    DayLongYear,
    Stamp,
}

impl Pattern {
    pub fn text(self) -> &'static str {
        match self {
            Pattern::Clock => t!("kit-date-pattern-clock"),
            Pattern::DayMonth => t!("kit-date-pattern-day-month"),
            Pattern::DayMonthPadded => t!("kit-date-pattern-day-month-padded"),
            Pattern::Full => t!("kit-date-pattern-full"),
            Pattern::WeekdayClock => t!("kit-date-pattern-weekday-clock"),
            Pattern::ShortDay => t!("kit-date-pattern-short-day"),
            Pattern::ShortDayClock => t!("kit-date-pattern-short-day-clock"),
            Pattern::DayShort => t!("kit-date-pattern-day-short"),
            Pattern::DayShortYear => t!("kit-date-pattern-day-short-year"),
            Pattern::DayShortClock => t!("kit-date-pattern-day-short-clock"),
            Pattern::DayLong => t!("kit-date-pattern-day-long"),
            Pattern::DayLongYear => t!("kit-date-pattern-day-long-year"),
            Pattern::Stamp => t!("kit-date-pattern-stamp"),
        }
    }
}

pub fn date(m: &Moment, kind: Pattern) -> String {
    format(m, kind.text())
}

pub fn format(m: &Moment, pattern: &str) -> String {
    let mut out = String::new();
    let mut chars = pattern.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '%' {
            out.push(c);
            continue;
        }
        let bare = chars.peek() == Some(&'-');
        if bare {
            chars.next();
        }
        let Some(code) = chars.next() else {
            out.push('%');
            if bare {
                out.push('-');
            }
            break;
        };
        let two = |n: u32| if bare { n.to_string() } else { format!("{n:02}") };
        let hour12 = if m.hour % 12 == 0 { 12 } else { m.hour % 12 };
        match code {
            'A' => out.push_str(weekday_name(m.weekday)),
            'a' => out.push_str(weekday_short(m.weekday)),
            'B' => out.push_str(month_name(m.month)),
            'b' | 'h' => out.push_str(month_short(m.month)),
            'd' => out.push_str(&two(m.day)),
            'e' => out.push_str(&if bare { m.day.to_string() } else { format!("{:>2}", m.day) }),
            'm' => out.push_str(&two(m.month)),
            'Y' => out.push_str(&m.year.to_string()),
            'y' => out.push_str(&format!("{:02}", m.year.rem_euclid(100))),
            'H' => out.push_str(&two(m.hour)),
            'I' => out.push_str(&two(hour12)),
            'M' => out.push_str(&two(m.minute)),
            'S' => out.push_str(&two(m.second)),
            'p' => out.push_str(if m.hour < 12 { t!("kit-date-am") } else { t!("kit-date-pm") }),
            'P' => out.push_str(&(if m.hour < 12 { t!("kit-date-am") } else { t!("kit-date-pm") }).to_lowercase()),
            'F' => out.push_str(&format!("{}-{:02}-{:02}", m.year, m.month, m.day)),
            'T' => out.push_str(&format!("{:02}:{:02}:{:02}", m.hour, m.minute, m.second)),
            'R' => out.push_str(&format!("{:02}:{:02}", m.hour, m.minute)),
            '%' => out.push('%'),
            other => {
                out.push('%');
                if bare {
                    out.push('-');
                }
                out.push(other);
            }
        }
    }
    out
}

pub fn decimal(n: f64, places: usize) -> String {
    digits(&marked(n, places))
}

pub(crate) fn marked(n: f64, places: usize) -> String {
    let plain = format!("{n:.places$}");
    let mark = t!("kit-number-decimal");
    if mark == "." || mark.is_empty() {
        plain
    } else {
        plain.replacen('.', mark, 1)
    }
}

pub fn figures() -> Vec<char> {
    super::numerals::figures()
}

pub fn digits(text: &str) -> String {
    let own = figures();
    text.chars().map(|c| if c.is_ascii_digit() { own[(c as u8 - b'0') as usize] } else { c }).collect()
}

pub fn number(n: f64, places: usize) -> String {
    let plain = marked(n.abs(), places);
    let mark = t!("kit-number-decimal");
    let (whole, rest) = match plain.find(|c: char| !c.is_ascii_digit()) {
        Some(at) if !mark.is_empty() && plain[at..].starts_with(mark) => plain.split_at(at),
        _ => (plain.as_str(), ""),
    };
    let group = t!("kit-number-group");
    let whole: Vec<char> = whole.chars().collect();
    let mut grouped = String::new();
    for (i, d) in whole.iter().enumerate() {
        if i > 0 && (whole.len() - i) % 3 == 0 {
            grouped.push_str(group);
        }
        grouped.push(*d);
    }
    let sign = if n < 0.0 && plain.chars().any(|c| c.is_ascii_digit() && c != '0') { "-" } else { "" };
    digits(&format!("{sign}{grouped}{rest}"))
}

fn sized(step: usize, text: String) -> String {
    match step {
        0 => t!("kit-size-kb", size = text),
        1 => t!("kit-size-mb", size = text),
        2 => t!("kit-size-gb", size = text),
        _ => t!("kit-size-tb", size = text),
    }
}

fn reaches_thousand(n: f64, places: usize) -> bool {
    format!("{n:.places$}").parse::<f64>().is_ok_and(|r| r >= 1000.0)
}

pub fn size(bytes: u64) -> String {
    let n = bytes as f64;
    let places = |step: usize| usize::from(step >= 2);
    let value = |step: usize| if step == 0 { (n / 1e3).max(1.0) } else { n / 10f64.powi(3 * (step as i32 + 1)) };
    let mut step = [(1e12, 3), (1e9, 2), (1e6, 1)].iter().find(|(from, _)| n >= *from).map_or(0, |&(_, s)| s);
    if step < 3 && reaches_thousand(value(step), places(step)) {
        step += 1;
    }
    sized(step, decimal(value(step), places(step)))
}

pub fn size_from_megabytes(megabytes: f64) -> String {
    if megabytes >= 1000.0 {
        t!("kit-size-gb", size = decimal(megabytes / 1000.0, 1))
    } else {
        t!("kit-size-mb", size = decimal(megabytes, 0))
    }
}

pub fn size_exact(bytes: u64) -> String {
    if bytes < 1000 {
        return t!("kit-size-bytes", n = bytes);
    }
    let mut n = bytes as f64;
    for step in 0..3 {
        n /= 1000.0;
        if !reaches_thousand(n, 1) {
            return sized(step, decimal(n, 1));
        }
    }
    sized(3, decimal(n / 1000.0, 1))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lang::{with_catalogue, Bundle, Catalogue, KIT_ENGLISH};

    const MOMENT: Moment = Moment { year: 2026, month: 10, day: 10, weekday: 6, hour: 21, minute: 5, second: 9 };
    const MORNING: Moment = Moment { year: 2026, month: 1, day: 3, weekday: 0, hour: 0, minute: 7, second: 0 };

    fn english() -> Catalogue {
        Catalogue::new(Bundle::default(), Bundle::from_files("en", KIT_ENGLISH))
    }

    fn made_up() -> Catalogue {
        let text = "
kit-date-weekday-sat = Zorgdag
kit-date-weekday-short-sat = Zor
kit-date-month-oct = Blumar
kit-date-month-short-oct = Blu
kit-date-pattern-day-month = %A, %B %-d
kit-date-pattern-full = %Y/%B/%d
kit-number-decimal = ,
kit-number-group = _
kit-size-mb = MB { $size }
";
        Catalogue::new(Bundle::from_text("xx", text), Bundle::from_files("en", KIT_ENGLISH))
    }

    #[test]
    fn english_is_written_as_it_always_was() {
        with_catalogue(english(), || {
            assert_eq!(date(&MOMENT, Pattern::DayMonth), "Saturday 10 October");
            assert_eq!(date(&MOMENT, Pattern::DayMonthPadded), "Saturday 10 October");
            assert_eq!(date(&MORNING, Pattern::DayMonth), "Sunday 3 January");
            assert_eq!(date(&MORNING, Pattern::DayMonthPadded), "Sunday 03 January");
            assert_eq!(date(&MOMENT, Pattern::Full), "10 October 2026");
            assert_eq!(date(&MOMENT, Pattern::Clock), "21:05");
            assert_eq!(date(&MOMENT, Pattern::WeekdayClock), "Sat 21:05");
            assert_eq!(date(&MOMENT, Pattern::ShortDay), "Sat 10 Oct");
            assert_eq!(date(&MOMENT, Pattern::ShortDayClock), "Sat 10 Oct, 21:05");
            assert_eq!(date(&MORNING, Pattern::DayShort), "3 Jan");
            assert_eq!(date(&MORNING, Pattern::DayShortYear), "3 Jan 2026");
            assert_eq!(date(&MOMENT, Pattern::DayShortClock), "10 Oct, 21:05");
            assert_eq!(date(&MORNING, Pattern::DayLong), "3 January");
            assert_eq!(date(&MORNING, Pattern::DayLongYear), "3 January 2026");
            assert_eq!(date(&MORNING, Pattern::Stamp), "Sun  3 Jan 2026, 00:07");
            assert_eq!(format(&MOMENT, "%I:%M %p, %-I %P, %e|%%|%y|%F|%T|%R"), "09:05 PM, 9 pm, 10|%|26|2026-10-10|21:05:09|21:05");
            assert_eq!(format(&MORNING, "%I %p %-H %-m %e"), "12 AM 0 1  3");
            assert_eq!(format(&MOMENT, "%Q %-Q % %"), "%Q %-Q % %");
            assert_eq!(size(512), "1 KB");
            assert_eq!(size(512_000_000), "512 MB");
            assert_eq!(size(14_400_000_000), "14.4 GB");
            assert_eq!(size(2_000_000_000_000), "2.0 TB");
            assert_eq!(size_from_megabytes(45.0), "45 MB");
            assert_eq!(size_from_megabytes(1300.0), "1.3 GB");
            assert_eq!(size(999_500), "1 MB");
            assert_eq!(size(999_499), "999 KB");
            assert_eq!(size(999_950_000_000), "1.0 TB");
            assert_eq!(size_exact(999_950), "1.0 MB");
            assert_eq!(size_exact(999_949), "999.9 KB");
            assert_eq!(size_exact(1), "1 byte");
            assert_eq!(size_exact(812), "812 bytes");
            assert_eq!(size_exact(4_500), "4.5 KB");
            assert_eq!(size_exact(3_000_000_000_000), "3.0 TB");
            assert_eq!(number(1234567.5, 1), "1,234,567.5");
            assert_eq!(number(123.0, 0), "123");
            assert_eq!(number(-1000.0, 0), "-1,000");
            assert_eq!(decimal(3.456, 2), "3.46");
        });
    }

    #[test]
    fn a_made_up_language_writes_dates_numbers_and_sizes_its_own_way() {
        with_catalogue(made_up(), || {
            assert_eq!(date(&MOMENT, Pattern::DayMonth), "Zorgdag, Blumar 10");
            assert_eq!(date(&MOMENT, Pattern::Full), "2026/Blumar/10");
            assert_eq!(format(&MOMENT, "%a %b"), "Zor Blu");
            assert_eq!(number(1234567.5, 1), "1_234_567,5");
            assert_eq!(decimal(3.5, 1), "3,5");
            assert_eq!(size(512_000_000), "MB 512");
            assert_eq!(size(14_400_000_000), "14,4 GB");
        });
    }

    #[test]
    fn what_a_language_lacks_is_written_in_english_word_by_word() {
        with_catalogue(made_up(), || {
            assert_eq!(weekday_name(6), "Zorgdag");
            assert_eq!(weekday_name(0), "Sunday");
            assert_eq!(month_name(1), "January");
            assert_eq!(month_short(12), "Dec");
            assert_eq!(date(&MOMENT, Pattern::Clock), "21:05");
            assert_eq!(date(&MOMENT, Pattern::WeekdayClock), "Zor 21:05");
            assert_eq!(size_exact(812), "812 bytes");
        });
    }

    #[test]
    fn counting_around_keeps_every_name_and_never_panics() {
        with_catalogue(english(), || {
            assert_eq!(weekday_name(7), "Sunday");
            assert_eq!(month_name(0), "December");
            assert_eq!(month_name(13), "January");
            for n in 0..14 {
                assert!(!weekday_short(n).is_empty() && !month_short(n).is_empty());
            }
        });
    }
}
