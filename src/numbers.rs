//! Numbers as a language writes and reads them, beside `lang::format`: a share as a percentage, and typed numbers read back.

use crate::lang::format::{self, figures};
use crate::lang::numerals::digit_value;
use crate::t;

pub fn percent(share: f64, places: usize) -> String {
    t!("kit-number-percent", n = format::number(share * 100.0, places))
}

pub fn read(text: &str) -> Option<f64> {
    let own = figures();
    let mark = t!("kit-number-decimal");
    let group = t!("kit-number-group");
    let mut plain = String::new();
    let mut rest = text.trim();
    if let Some(r) = rest.strip_prefix(['-', '\u{2212}']) {
        plain.push('-');
        rest = r.trim_start();
    }
    let marks: Vec<&str> = [mark, ".", "\u{066B}"].into_iter().filter(|m| !m.is_empty() && (*m == mark || group != "." || *m != ".")).collect();
    let mut seen_mark = false;
    let mut i = 0;
    while i < rest.len() {
        let tail = &rest[i..];
        let c = tail.chars().next()?;
        if let Some(d) = own.iter().position(|&o| o == c).or_else(|| digit_value(c).map(usize::from)) {
            plain.push(char::from(b'0' + d as u8));
            i += c.len_utf8();
        } else if let Some(m) = marks.iter().find(|m| tail.starts_with(**m)).filter(|_| !seen_mark) {
            seen_mark = true;
            plain.push('.');
            i += m.len();
        } else if !seen_mark && !group.is_empty() && tail.starts_with(group) {
            i += group.len();
        } else if !seen_mark && tail.starts_with('\u{066C}') {
            i += '\u{066C}'.len_utf8();
        } else if !seen_mark && c.is_whitespace() {
            i += c.len_utf8();
        } else {
            return None;
        }
    }
    plain.parse().ok().filter(|n: &f64| n.is_finite())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lang::{with_catalogue, Bundle, Catalogue, KIT_ENGLISH};

    fn language(text: &str) -> Catalogue {
        Catalogue::new(Bundle::from_text("xx", text), Bundle::from_files("en", KIT_ENGLISH))
    }

    #[test]
    fn english_writes_and_reads_plain_figures() {
        with_catalogue(language(""), || {
            assert_eq!(format::number(1234.5, 1), "1,234.5");
            assert_eq!(percent(0.455, 1), "45.5%");
            assert_eq!(read("1,234.5"), Some(1234.5));
            assert_eq!(read(" -12 "), Some(-12.0));
            assert_eq!(read("12a"), None);
            assert_eq!(read("1.2.3"), None);
        });
    }

    #[test]
    fn a_language_with_its_own_figures_and_marks_writes_and_reads_them() {
        let arabic = "kit-number-digits = ٠١٢٣٤٥٦٧٨٩\nkit-number-decimal = ٫\nkit-number-group = ٬\nkit-number-percent = { $n }٪";
        with_catalogue(language(arabic), || {
            assert_eq!(format::number(1234.5, 1), "١٬٢٣٤٫٥");
            assert_eq!(format::decimal(2.5, 1), "٢٫٥");
            assert_eq!(percent(0.5, 0), "٥٠٪");
            assert_eq!(read("١٬٢٣٤٫٥"), Some(1234.5));
            assert_eq!(read("1234.5"), Some(1234.5), "plain figures and a full stop are read too");
        });
        let german = "kit-number-decimal = ,\nkit-number-group = .\nkit-number-percent = { $n } %";
        with_catalogue(language(german), || {
            assert_eq!(percent(0.25, 0), "25 %");
            assert_eq!(read("1.234,5"), Some(1234.5), "a full stop groups, it is not the mark");
            assert_eq!(read("3,75"), Some(3.75));
        });
    }
}
