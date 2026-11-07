//! Which ten figures a person sees, by the Unicode CLDR's default for the language and region, the person's own choice over it, and 0 to 9 asked for on one field.

use std::cell::Cell;

use crate::t;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Numerals {
    #[default]
    Auto,
    Latin,
    Native,
}

impl Numerals {
    pub const ALL: [Numerals; 3] = [Numerals::Auto, Numerals::Latin, Numerals::Native];

    pub const KEY: &'static str = "numerals";

    pub fn parse(text: &str) -> Numerals {
        match text.trim().to_ascii_lowercase().as_str() {
            "latin" => Numerals::Latin,
            "native" => Numerals::Native,
            _ => Numerals::Auto,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Numerals::Auto => "auto",
            Numerals::Latin => "latin",
            Numerals::Native => "native",
        }
    }

    #[cfg(feature = "ui")]
    pub fn read(config: &crate::config::Config) -> Numerals {
        config.get(Self::KEY).map_or(Numerals::Auto, |v| Numerals::parse(&v))
    }

    #[cfg(feature = "ui")]
    pub fn save(self, config: &crate::config::Config) -> std::io::Result<()> {
        config.set(Self::KEY, self.name())
    }

    pub fn label(self) -> &'static str {
        match self {
            Numerals::Auto => t!("kit-numerals-auto"),
            Numerals::Latin => t!("kit-numerals-latin"),
            Numerals::Native => t!("kit-numerals-native"),
        }
    }
}

const ZEROS: [char; 17] = [
    '\u{0660}', '\u{06F0}', '\u{0966}', '\u{09E6}', '\u{0A66}', '\u{0AE6}', '\u{0BE6}', '\u{0C66}', '\u{0CE6}',
    '\u{0D66}', '\u{0E50}', '\u{0ED0}', '\u{0F20}', '\u{1040}', '\u{17E0}', '\u{1810}', '\u{FF10}',
];

pub fn digit_value(c: char) -> Option<u8> {
    if c.is_ascii_digit() {
        return Some(c as u8 - b'0');
    }
    ZEROS.iter().find(|&&z| (z as u32..z as u32 + 10).contains(&(c as u32))).map(|&z| (c as u32 - z as u32) as u8)
}

pub fn latin_digits(text: &str) -> String {
    text.chars().map(|c| digit_value(c).map_or(c, |d| char::from(b'0' + d))).collect()
}

fn rule(tag: &str) -> Option<(char, bool)> {
    let mut parts = tag.split(['-', '_']);
    let base = parts.next()?.to_ascii_lowercase();
    let region = parts.find(|p| p.len() == 2 && p.bytes().all(|b| b.is_ascii_alphabetic())).map(str::to_ascii_uppercase);
    let region = region.as_deref();
    Some(match base.as_str() {
        "ar" => ('\u{0660}', !matches!(region, Some("AE" | "DZ" | "EH" | "LY" | "MA" | "TN"))),
        "ckb" | "sd" => ('\u{0660}', true),
        "fa" | "ps" => ('\u{06F0}', true),
        "ur" => ('\u{06F0}', region == Some("IN")),
        "bn" | "as" => ('\u{09E6}', true),
        "mr" | "ne" => ('\u{0966}', true),
        "hi" => ('\u{0966}', false),
        "my" => ('\u{1040}', true),
        "dz" => ('\u{0F20}', true),
        _ => return None,
    })
}

fn ten(zero: char) -> Vec<char> {
    (0..10).filter_map(|i| char::from_u32(zero as u32 + i)).collect()
}

thread_local! {
    static LATIN: Cell<bool> = const { Cell::new(false) };
}

pub fn latin<R>(during: impl FnOnce() -> R) -> R {
    struct Restore(bool);
    impl Drop for Restore {
        fn drop(&mut self) {
            LATIN.set(self.0);
        }
    }
    let _restore = Restore(LATIN.replace(true));
    during()
}

pub fn figures() -> Vec<char> {
    let plain: Vec<char> = ('0'..='9').collect();
    if LATIN.get() {
        return plain;
    }
    let choice = Numerals::parse(&crate::host::host().numerals().unwrap_or_default());
    if choice == Numerals::Latin {
        return plain;
    }
    match rule(&super::chosen_tag()) {
        Some((zero, by_default)) if choice == Numerals::Native || by_default => ten(zero),
        Some(_) => plain,
        None => {
            let own: Vec<char> = t!("kit-number-digits").chars().filter(|c| !c.is_whitespace()).collect();
            if own.len() == 10 { own } else { plain }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lang::{format, with_catalogue, Bundle, Catalogue, KIT_ENGLISH};

    struct Chose(&'static str);
    impl crate::host::Host for Chose {
        fn numerals(&self) -> Option<String> {
            Some(self.0.to_string())
        }
    }

    fn in_tag<R>(tag: &str, choice: &'static str, during: impl FnOnce() -> R) -> R {
        let host: &'static Chose = Box::leak(Box::new(Chose(choice)));
        let catalogue = Catalogue::new(Bundle::from_text(tag, ""), Bundle::from_files("en", KIT_ENGLISH));
        crate::host::with(host, || with_catalogue(catalogue, during))
    }

    fn seen(tag: &str, choice: &'static str) -> String {
        in_tag(tag, choice, || format::digits("0123456789"))
    }

    const EASTERN: &str = "٠١٢٣٤٥٦٧٨٩";
    const PERSIAN: &str = "۰۱۲۳۴۵۶۷۸۹";
    const LATIN: &str = "0123456789";

    #[test]
    fn the_region_decides_the_figures_as_cldr_does() {
        for tag in ["ar", "ar-SA", "ar-EG", "ar-IQ", "ar-SY", "ar-JO", "ar-LB", "ar-SD", "ar-YE", "ar_KW"] {
            assert_eq!(seen(tag, "auto"), EASTERN, "{tag}");
        }
        for tag in ["ar-MA", "ar-DZ", "ar-TN", "ar-LY", "ar-AE", "ar-EH", "en", "de-DE", "fr", "ur", "ur-PK", "hi", "tr", "ja"] {
            assert_eq!(seen(tag, "auto"), LATIN, "{tag}");
        }
        assert_eq!((seen("fa", "auto"), seen("fa-IR", ""), seen("ur-IN", "auto")), (PERSIAN.into(), PERSIAN.into(), PERSIAN.into()));
        assert_eq!(seen("bn", "auto"), "০১২৩৪৫৬৭৮৯");
    }

    #[test]
    fn a_person_can_ask_for_0_to_9_or_the_languages_own() {
        assert_eq!(seen("ar-EG", "latin"), LATIN);
        assert_eq!(seen("ar-MA", "native"), EASTERN);
        assert_eq!(seen("hi", "native"), "०१२३४५६७८९");
        assert_eq!(seen("ur", "native"), PERSIAN);
        assert_eq!(seen("en", "native"), LATIN, "a language with none of its own");
        assert_eq!(seen("ar-EG", "rubbish"), EASTERN, "unknown is auto");
        assert_eq!(Numerals::ALL.map(|n| Numerals::parse(n.name())), Numerals::ALL);
    }

    #[test]
    fn a_language_file_unknown_here_still_gives_its_own_figures() {
        let catalogue = Catalogue::new(Bundle::from_text("xx", "kit-number-digits = ٠١٢٣٤٥٦٧٨٩"), Bundle::from_files("en", KIT_ENGLISH));
        with_catalogue(catalogue, || assert_eq!(format::digits("5"), "٥"));
    }

    #[test]
    fn one_field_can_ask_for_0_to_9() {
        in_tag("ar-EG", "auto", || {
            assert_eq!(format::number(1234.5, 1), "١,٢٣٤.٥");
            assert_eq!(latin(|| format::digits("2026")), "2026");
            assert_eq!(format::digits("2026"), "٢٠٢٦", "and only for the call");
        });
    }

    #[test]
    fn every_script_reads_back_as_0_to_9() {
        assert_eq!(latin_digits("٢٠٢٦-۱۰-١٢ ४५ ০৯ 77"), "2026-10-12 45 09 77");
        assert_eq!(digit_value('٩'), Some(9));
        assert_eq!(digit_value('a'), None);
    }

    #[test]
    fn the_choice_is_kept_in_the_programs_settings() {
        let folder = std::env::temp_dir().join(format!("trindle-numerals-{}", std::process::id()));
        let config = crate::config::Config::at(&folder);
        assert_eq!(Numerals::read(&config), Numerals::Auto);
        Numerals::Native.save(&config).unwrap();
        assert_eq!(Numerals::read(&config), Numerals::Native);
        std::fs::remove_dir_all(&folder).unwrap();
    }

    #[test]
    fn typed_and_written_numbers_round_trip_in_both_sets_and_mixed() {
        use crate::{duration, numbers, spin::{Spin, SpinMsg}, validate::Mask, when::{Date, Time, TimeBox, Typed}};
        in_tag("ar-EG", "auto", || {
            assert_eq!(numbers::read(&format::number(1234.5, 1)), Some(1234.5));
            assert_eq!(numbers::read("1234.5"), Some(1234.5));
            assert_eq!(numbers::read("١٢3٤.٥"), Some(1234.5), "mixed sets");
            assert_eq!(numbers::read("۱۲"), Some(12.0), "Persian figures too");
            let mut spin = Spin::new(4.5, 0.0, 10.0, 0.5);
            assert_eq!(spin.text(), "٤.٥");
            spin.update(SpinMsg::Typed("٧٫٥".into()));
            assert!(spin.update(SpinMsg::Done));
            assert_eq!(spin.value, 7.5);
            spin.update(SpinMsg::Typed("2.5".into()));
            spin.update(SpinMsg::Done);
            assert_eq!(spin.value, 2.5);
            assert_eq!(Spin::new(4.5, 0.0, 10.0, 0.5).latin().text(), "4.5");
            let mut d = duration::DurationBox::new(3725);
            d.update(duration::Typed::Minutes("٣٠".into()));
            assert_eq!((d.minutes.as_str(), d.total()), ("30", Some(3600 + 30 * 60 + 5)));
            let mut b = TimeBox::new(Time::new(7, 5).unwrap());
            b.update(Typed::Hours("٢٣".into()));
            b.update(Typed::Minutes("5٩".into()));
            assert_eq!(b.time(), Time::new(23, 59));
            assert_eq!(Time::new(7, 5).unwrap().written(), "٠٧:٠٥");
            assert_eq!(Date::parse("٢٠٢٦-١٠-٣٠").map(Date::iso).as_deref(), Some("2026-10-30"));
            assert_eq!(Mask::Phone.shape("٠٧٧٠٠ ٩٠٠١٢٣"), "07700 900123", "a phone box keeps 0 to 9");
            assert_eq!(Mask::Phone.problem("٠٧٧٠٠٩٠٠١٢٣"), None);
            assert_eq!(Mask::Date.shape("٢٠٢٦-١٠-٣٠"), "2026-10-30");
        });
        in_tag("en", "latin", || assert_eq!(crate::numbers::read("٣.٥"), Some(3.5), "an English reader takes Arabic figures"));
    }

    #[test]
    fn text_layout_and_the_screen_reader_get_the_chosen_figures() {
        use crate::{spin::Spin, text::{Fonts, Style}};
        let spin = Spin::new(4.5, 0.0, 10.0, 0.5);
        in_tag("ar-EG", "auto", || {
            let laid = Fonts::new().layout(&spin.text(), &Style::new(16.0), None);
            assert_eq!(&**laid.whole(), "٤.٥");
        });
        in_tag("ar-EG", "latin", || {
            assert_eq!(Fonts::new().layout(&spin.text(), &Style::new(16.0), None).whole().as_ref(), "4.5");
        });
    }
}
