//! Plural categories: which of zero, one, two, few, many and other a number falls in, for the common languages (the CLDR cardinal rules, whole numbers and the plain fractions). A language not listed here counts like English (among them da, fil, tl, si, gd, mt, br, dsb and hsb, whose rules are not written here), and a message written for a category a language never uses is simply not chosen, so a wrong guess shows the `other` line rather than nothing.

pub fn category(lang: &str, n: f64) -> &'static str {
    let mut tag = lang.split(['-', '_']);
    let first = tag.next().unwrap_or("");
    let portugal = tag.next().is_some_and(|region| region.eq_ignore_ascii_case("pt"));
    let mut small = [0u8; 8];
    let primary = match small.get_mut(..first.len()) {
        Some(room) => {
            room.copy_from_slice(first.as_bytes());
            room.make_ascii_lowercase();
            std::str::from_utf8(room).unwrap_or("")
        }
        None => "",
    };
    let n = n.abs();
    let whole = n.fract() == 0.0;
    let i = n.trunc() as u64;
    let (m10, m100) = (i % 10, i % 100);
    let few_tail = (2..=4).contains(&m10) && !(12..=14).contains(&m100);
    if whole && i != 0 && i % 1_000_000 == 0 && matches!(primary, "es" | "fr" | "pt" | "it" | "ca") {
        return "many";
    }
    let x = Num { n, whole, i, m10, m100, few_tail };
    early(primary, portugal, &x)
}

struct Num {
    n: f64,
    whole: bool,
    i: u64,
    m10: u64,
    m100: u64,
    few_tail: bool,
}

fn early(primary: &str, portugal: bool, x: &Num) -> &'static str {
    let Num { n, whole, i, m10, m100, few_tail } = *x;
    match primary {
        "ja" | "zh" | "ko" | "vi" | "th" | "id" | "ms" | "lo" | "my" | "km" | "jv" | "yue" => "other",
        "pt" if portugal => {
            if whole && i == 1 { "one" } else { "other" }
        }
        "fr" | "pt" | "hy" | "kab" | "ff" => {
            if i <= 1 { "one" } else { "other" }
        }
        "hi" | "bn" | "fa" | "gu" | "kn" | "zu" | "am" | "as" => {
            if i == 0 || n == 1.0 { "one" } else { "other" }
        }
        "mr" => {
            if n == 1.0 { "one" } else { "other" }
        }
        "is" => {
            if !whole || (m10 == 1 && m100 != 11) { "one" } else { "other" }
        }
        "mk" => {
            if whole && m10 == 1 && m100 != 11 { "one" } else { "other" }
        }
        "ln" | "ti" | "mg" | "pa" => {
            if whole && i <= 1 { "one" } else { "other" }
        }
        "ru" | "uk" | "be" => {
            if !whole { "other" }
            else if m10 == 1 && m100 != 11 { "one" }
            else if few_tail { "few" }
            else { "many" }
        }
        "pl" => {
            if !whole { "other" }
            else if i == 1 { "one" }
            else if few_tail { "few" }
            else { "many" }
        }
        "cs" | "sk" => {
            if !whole { "many" }
            else if i == 1 { "one" }
            else if (2..=4).contains(&i) { "few" }
            else { "other" }
        }
        _ => late(primary, x),
    }
}

fn late(primary: &str, x: &Num) -> &'static str {
    let Num { n: _, whole, i, m10, m100, few_tail } = *x;
    match primary {
        "sl" => {
            if !whole { "few" } else {
                match m100 {
                    1 => "one",
                    2 => "two",
                    3 | 4 => "few",
                    _ => "other",
                }
            }
        }
        "hr" | "sr" | "bs" => {
            if !whole { "other" }
            else if m10 == 1 && m100 != 11 { "one" }
            else if few_tail { "few" }
            else { "other" }
        }
        "lt" => {
            let teens = (11..=19).contains(&m100);
            if !whole { "many" }
            else if m10 == 1 && !teens { "one" }
            else if (2..=9).contains(&m10) && !teens { "few" }
            else { "other" }
        }
        "lv" => {
            if whole && (m10 == 0 || (11..=19).contains(&m100)) { "zero" }
            else if whole && m10 == 1 && m100 != 11 { "one" }
            else { "other" }
        }
        "ro" => {
            if whole && i == 1 { "one" }
            else if !whole || i == 0 || (i != 1 && (2..=19).contains(&m100)) { "few" }
            else { "other" }
        }
        _ => last(primary, x),
    }
}

fn last(primary: &str, x: &Num) -> &'static str {
    let Num { n, whole, i, m100, .. } = *x;
    match primary {
        "ar" => {
            if !whole { "other" }
            else if i == 0 { "zero" }
            else if i == 1 { "one" }
            else if i == 2 { "two" }
            else if (3..=10).contains(&m100) { "few" }
            else if (11..=99).contains(&m100) { "many" }
            else { "other" }
        }
        "he" | "iw" => {
            if whole && i == 1 { "one" }
            else if whole && i == 2 { "two" }
            else { "other" }
        }
        "cy" => {
            if !whole { "other" } else {
                match i {
                    0 => "zero",
                    1 => "one",
                    2 => "two",
                    3 => "few",
                    6 => "many",
                    _ => "other",
                }
            }
        }
        "ga" => {
            if !whole { "other" } else {
                match i {
                    1 => "one",
                    2 => "two",
                    3..=6 => "few",
                    7..=10 => "many",
                    _ => "other",
                }
            }
        }
        _ => {
            if n == 1.0 { "one" } else { "other" }
        }
    }
}

#[cfg(test)]
thread_local!(pub(crate) static CALLS: std::cell::Cell<u32> = const { std::cell::Cell::new(0) });

pub fn categories(lang: &str) -> std::collections::BTreeSet<&'static str> {
    #[cfg(test)]
    CALLS.with(|c| c.set(c.get() + 1));
    (0..=1000).map(f64::from).chain([0.5, 1.5, 2.5]).map(|n| category(lang, n)).collect()
}

#[cfg(test)]
mod romanian {
    #[test]
    fn few_is_two_to_nineteen_of_each_hundred() {
        assert_eq!(super::category("ro", 101.0), "other");
        assert_eq!(super::category("ro", 102.0), "few");
        assert_eq!(super::category("ro", 0.0), "few");
    }
}
