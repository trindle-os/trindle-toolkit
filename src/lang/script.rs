//! Which fonts a language's writing needs beyond Trindle Sans, found among the fonts on the computer. Only `system_families` reads the font folders.

#[derive(Debug, PartialEq, Eq)]
pub struct Script {
    pub name: &'static str,
    pub families: &'static [&'static str],
    pub package: &'static str,
}

const DEVANAGARI: Script = Script { name: "Devanagari", families: &["Noto Sans Devanagari", "Noto Serif Devanagari"], package: "fonts-noto-core" };
const ARABIC: Script = Script { name: "Arabic", families: &["Noto Sans Arabic", "Noto Naskh Arabic", "Noto Kufi Arabic"], package: "fonts-noto-core" };
const NASTALIQ: Script = Script { name: "Nastaliq", families: &["Noto Nastaliq Urdu", "Noto Naskh Arabic", "Noto Sans Arabic"], package: "fonts-noto-core" };
const BENGALI: Script = Script { name: "Bengali", families: &["Noto Sans Bengali", "Noto Serif Bengali"], package: "fonts-noto-core" };
const HEBREW: Script = Script { name: "Hebrew", families: &["Noto Sans Hebrew", "Noto Serif Hebrew"], package: "fonts-noto-core" };
const THAI: Script = Script { name: "Thai", families: &["Noto Sans Thai", "Noto Serif Thai", "Noto Looped Thai"], package: "fonts-noto-core" };
const ETHIOPIC: Script = Script { name: "Ethiopic", families: &["Noto Sans Ethiopic", "Noto Serif Ethiopic"], package: "fonts-noto-core" };
const TAMIL: Script = Script { name: "Tamil", families: &["Noto Sans Tamil", "Noto Serif Tamil"], package: "fonts-noto-core" };
const TELUGU: Script = Script { name: "Telugu", families: &["Noto Sans Telugu", "Noto Serif Telugu"], package: "fonts-noto-core" };
const MALAYALAM: Script = Script { name: "Malayalam", families: &["Noto Sans Malayalam", "Noto Serif Malayalam"], package: "fonts-noto-core" };
const KANNADA: Script = Script { name: "Kannada", families: &["Noto Sans Kannada", "Noto Serif Kannada"], package: "fonts-noto-core" };
const GUJARATI: Script = Script { name: "Gujarati", families: &["Noto Sans Gujarati", "Noto Serif Gujarati"], package: "fonts-noto-core" };
const GURMUKHI: Script = Script { name: "Gurmukhi", families: &["Noto Sans Gurmukhi", "Noto Serif Gurmukhi"], package: "fonts-noto-core" };
const SINHALA: Script = Script { name: "Sinhala", families: &["Noto Sans Sinhala", "Noto Serif Sinhala"], package: "fonts-noto-core" };
const KHMER: Script = Script { name: "Khmer", families: &["Noto Sans Khmer", "Noto Serif Khmer"], package: "fonts-noto-core" };
const LAO: Script = Script { name: "Lao", families: &["Noto Sans Lao", "Noto Serif Lao"], package: "fonts-noto-core" };
const MYANMAR: Script = Script { name: "Burmese", families: &["Noto Sans Myanmar", "Noto Serif Myanmar"], package: "fonts-noto-core" };
const GEORGIAN: Script = Script { name: "Georgian", families: &["Noto Sans Georgian", "Noto Serif Georgian"], package: "fonts-noto-core" };
const ARMENIAN: Script = Script { name: "Armenian", families: &["Noto Sans Armenian", "Noto Serif Armenian"], package: "fonts-noto-core" };
const HANS: Script = Script { name: "Simplified Chinese", families: &["Noto Sans CJK SC", "Noto Sans SC", "Noto Serif CJK SC"], package: "fonts-noto-cjk" };
const HANT: Script = Script { name: "Traditional Chinese", families: &["Noto Sans CJK TC", "Noto Sans TC", "Noto Serif CJK TC"], package: "fonts-noto-cjk" };
const JAPANESE: Script = Script { name: "Japanese", families: &["Noto Sans CJK JP", "Noto Sans JP", "Noto Serif CJK JP"], package: "fonts-noto-cjk" };
const KOREAN: Script = Script { name: "Korean", families: &["Noto Sans CJK KR", "Noto Sans KR", "Noto Serif CJK KR"], package: "fonts-noto-cjk" };

pub const SCRIPTS: &[(&str, &Script)] = &[
    ("hi", &DEVANAGARI), ("mr", &DEVANAGARI), ("ne", &DEVANAGARI), ("sa", &DEVANAGARI),
    ("ar", &ARABIC), ("fa", &ARABIC), ("ur", &NASTALIQ),
    ("ta", &TAMIL), ("te", &TELUGU), ("ml", &MALAYALAM), ("kn", &KANNADA), ("gu", &GUJARATI), ("pa", &GURMUKHI), ("si", &SINHALA),
    ("km", &KHMER), ("lo", &LAO), ("my", &MYANMAR), ("ka", &GEORGIAN), ("hy", &ARMENIAN),
    ("bn", &BENGALI), ("he", &HEBREW), ("th", &THAI), ("am", &ETHIOPIC), ("ti", &ETHIOPIC),
    ("zh-Hans", &HANS), ("zh-CN", &HANS), ("zh-SG", &HANS), ("zh-Hant", &HANT), ("zh-TW", &HANT), ("zh-HK", &HANT),
    ("zh", &HANS), ("ja", &JAPANESE), ("ko", &KOREAN),
];

pub fn script_in<'a>(table: &'a [(&str, &'a Script)], tags: &[String]) -> Option<&'a Script> {
    tags.iter().find_map(|t| table.iter().find(|(k, _)| k.eq_ignore_ascii_case(t)).map(|(_, s)| *s))
}

pub fn script_of(tags: &[String]) -> Option<&'static Script> {
    script_in(SCRIPTS, tags)
}

pub fn installed<S: AsRef<str>>(script: &Script, families: &[S]) -> Vec<String> {
    script.families.iter().filter(|w| families.iter().any(|f| f.as_ref().eq_ignore_ascii_case(w))).map(|w| w.to_string()).collect()
}

pub fn missing_in<'a, S: AsRef<str>>(table: &'a [(&str, &'a Script)], tags: &[String], families: &[S]) -> Option<&'a Script> {
    script_in(table, tags).filter(|s| installed(s, families).is_empty())
}

pub fn system_families() -> Vec<String> {
    let mut db = fontdb::Database::new();
    db.load_system_fonts();
    db.faces().filter_map(|f| f.families.first().map(|(n, _)| n.clone())).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const MADE_UP: Script = Script { name: "Zorbic", families: &["Zorb Sans", "Zorb Serif"], package: "fonts-zorb" };
    const TABLE: &[(&str, &Script)] = &[("zz", &MADE_UP), ("zz-Latn", &MADE_UP)];

    fn tags(s: &[&str]) -> Vec<String> {
        s.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn a_language_finds_its_script_by_its_most_exact_tag_and_others_find_none() {
        assert_eq!(script_in(TABLE, &tags(&["zz-XX", "zz"])), Some(&MADE_UP));
        assert_eq!(script_in(TABLE, &tags(&["ZZ"])), Some(&MADE_UP));
        assert_eq!(script_in(TABLE, &tags(&["qq"])), None);
        assert_eq!(script_in(TABLE, &[]), None);
    }

    #[test]
    fn only_the_installed_families_are_wanted_best_first() {
        assert_eq!(installed(&MADE_UP, &["Other", "zorb serif", "Zorb Sans"]), ["Zorb Sans", "Zorb Serif"]);
        assert!(installed(&MADE_UP, &["Other"]).is_empty());
    }

    #[test]
    fn without_any_of_its_fonts_the_language_reports_the_package_missing() {
        let none: [&str; 1] = ["Noto Sans"];
        assert_eq!(missing_in(TABLE, &tags(&["zz"]), &none).map(|s| s.package), Some("fonts-zorb"));
        assert_eq!(missing_in(TABLE, &tags(&["zz"]), &["Zorb Serif"]), None);
        assert_eq!(missing_in(TABLE, &tags(&["qq"]), &none), None);
    }

    #[test]
    fn the_real_table_covers_every_language_that_needs_a_font_and_names_a_package() {
        for code in ["hi", "ar", "ur", "bn", "zh-Hans", "ja", "ko", "th", "am", "ta", "te", "ml", "kn", "gu", "pa", "si", "km", "lo", "my", "ka", "hy"] {
            let s = script_of(&tags(&[code])).unwrap_or_else(|| panic!("{code}"));
            assert!(!s.families.is_empty() && s.package.starts_with("fonts-noto"), "{code}");
        }
        for code in ["en", "fr", "ru", "es", "de", "pt-BR", "vi", "id", "tr", "sw"] {
            assert!(script_of(&tags(&[code])).is_none(), "{code}");
        }
        assert_eq!(script_of(&tags(&["zh-Hant-TW", "zh-Hant", "zh"])).map(|s| s.name), Some("Traditional Chinese"));
    }
}
