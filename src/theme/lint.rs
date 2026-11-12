//! The style sheet checker: every number within the limits written beside it in Trindle's sheet, every colour pair at its WCAG 2.2 AA floor.

use crate::common::sheet;
use crate::common::style::{self, Layers};

use super::pairs;

pub fn limits() -> Vec<(String, f32, f32)> {
    style::CSS.lines().filter_map(|l| {
        let (name, rest) = l.trim_start().strip_prefix("--")?.split_once(':')?;
        let inside = rest.split_once("/* (")?.1.split_once(')')?.0;
        let (min, max) = inside.split_once(" to ")?;
        Some((name.trim().to_string(), min.trim().parse().ok()?, max.trim().parse().ok()?))
    }).collect()
}

pub fn lint(css: &str) -> Vec<String> {
    let mut out = Vec::new();
    if let Err(f) = super::check(css) {
        out.push(f.words());
    }
    let layers = Layers::new(style::CSS, css);
    let limits = limits();
    for d in style::declarations(css) {
        if let Some(kind) = sheet::step_kind(&d.name).filter(|&k| css != style::CSS && !sheet::is_step(k, &d.value)) {
            let said = crate::t!("kit-theme-line", line = d.line, why = &crate::t!("kit-theme-not-step", name = &d.name, steps = sheet::steps(kind), guide = sheet::GUIDE));
            if !out.contains(&said) {
                out.push(said);
            }
            continue;
        }
        let Some((_, min, max)) = limits.iter().find(|l| l.0 == d.name) else { continue };
        let v = match d.value.trim().strip_prefix("var(--").and_then(|v| v.strip_suffix(')')) {
            Some(other) => layers.number(other.trim(), d.light, f32::MIN, f32::MAX),
            None => sheet::number(&d.value),
        };
        if let Some(v) = v.filter(|v| v < min || v > max) {
            out.push(crate::t!("kit-sheet-out-of-range", line = d.line, name = &d.name, value = v.to_string(), min = min.to_string(), max = max.to_string()));
        }
    }
    let looks: &[bool] = match crate::common::sheet::one_look(css) {
        Some(false) => &[false],
        Some(true) => &[true],
        None => &[false, true],
    };
    for &light in looks {
        let look = if light { crate::t!("kit-sheet-light") } else { crate::t!("kit-sheet-dark") };
        for p in pairs::painted(&layers, light, style::DEFAULT_ACCENT) {
            if !p.short() || !pairs::theirs(&layers, light, p.ink) {
                continue;
            }
            let said = crate::t!("kit-sheet-short", look = look, ink = p.ink.name().unwrap_or("trindle-text"), on = &p.on,
                                 stands = format!("{:.2}", p.stands()), floor = format!("{:.1}", p.ink.floor()));
            if !out.contains(&said) {
                out.push(said);
            }
        }
    }
    out
}

pub fn logged(path: &std::path::Path) -> Vec<String> {
    let file = path.display().to_string();
    let css = match std::fs::read_to_string(path) {
        Ok(css) => css,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Vec::new(),
        Err(_) => return vec![crate::t!("kit-sheet-unread", file = &file)],
    };
    let problems = lint(&css);
    if problems.is_empty() {
        return vec![crate::t!("kit-sheet-fine", file = &file)];
    }
    problems.into_iter().map(|p| format!("{file}: {p}")).collect()
}

pub fn log(path: &std::path::Path) {
    for line in logged(path) {
        eprintln!("{line}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trindle_s_own_sheet_and_every_theme_it_ships_are_clean() {
        assert_eq!(lint(style::CSS), Vec::<String>::new());
        for f in std::fs::read_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/share/themes")).unwrap().flatten() {
            let css = std::fs::read_to_string(f.path()).unwrap();
            assert_eq!(lint(&css), Vec::<String>::new(), "{}", f.path().display());
        }
    }

    #[test]
    fn the_limits_are_read_from_beside_each_value() {
        let all = limits();
        assert!(all.contains(&("trindle-gap".to_string(), 6.0, 18.0)), "{all:?}");
        assert!(all.len() > 40, "{}", all.len());
    }

    #[test]
    fn a_number_past_its_limits_is_named_with_its_line() {
        let said = lint(":root {\n  --trindle-gap: 40;\n  --trindle-card-gap: var(--trindle-gap);\n}\n");
        assert_eq!(said.len(), 2, "{said:?}");
        assert!(said[0].starts_with("Line 2: --trindle-gap is 40. It must be between 6 and 18"), "{said:?}");
        assert!(said[1].starts_with("Line 3: --trindle-card-gap is 40"), "{said:?}");
    }

    #[test]
    fn words_too_faint_to_read_are_named_with_what_they_are_on() {
        let said = lint(":root { --trindle-text: #3a3a3a; }");
        assert!(!said.is_empty() && said.iter().all(|s| s.starts_with("In the dark look, --trindle-text on ")), "{said:?}");
        assert_eq!(said[0], "In the dark look, --trindle-text on the page has a contrast of 1.66 to 1. It needs at least 4.5 to 1.");
    }

    #[test]
    fn a_saved_sheet_is_logged_with_its_problems_or_as_fine() {
        let at = std::env::temp_dir().join(format!("trindle-lint-{}.css", std::process::id()));
        std::fs::write(&at, ":root { --trindle-gap: 2; }").unwrap();
        let bad = logged(&at);
        std::fs::write(&at, ":root { --trindle-gap: 8; }").unwrap();
        let good = logged(&at);
        std::fs::remove_file(&at).unwrap();
        assert_eq!(bad, vec![format!("{}: Line 1: --trindle-gap is 2. It must be between 6 and 18.", at.display())]);
        assert_eq!(good, vec![format!("{}: no problems found.", at.display())]);
        assert!(logged(&at).is_empty(), "a sheet taken away is no problem");
        std::fs::create_dir(&at).unwrap();
        let unread = logged(&at);
        std::fs::remove_dir(&at).unwrap();
        assert_eq!(unread, vec![format!("{} couldn't be read.", at.display())], "a sheet that is there and cannot be read is said");
    }

    #[test]
    fn a_number_where_a_step_is_asked_is_named_once_with_its_steps_and_the_guide() {
        let said = lint(":root {\n  --trindle-gap-small: 5;\n  --trindle-board-lane: 75px;\n  --trindle-gap-large: huge;\n}\n");
        assert_eq!(said, vec![
            "Line 2: --trindle-gap-small can only be one of these: none, tiny, small, medium, roomy, large, huge. Trindle ignores any other value. See docs/style-sheet-for-ai.md.",
            "Line 3: --trindle-board-lane can only be one of these: smaller, small, normal, large, larger. Trindle ignores any other value. See docs/style-sheet-for-ai.md.",
        ]);
        assert!(super::super::check(":root { --trindle-gap-small: 5; }").unwrap_err().why.contains("docs/style-sheet-for-ai.md"));
        assert!(super::super::check(":root { --trindle-gap-small: tiny; --trindle-board-lane: larger; }").is_ok());
    }

    const GUIDE: &str = include_str!("../../docs/style-sheet-for-ai.md");

    #[test]
    fn the_guide_lists_every_key_with_its_kind_and_its_example_is_clean() {
        for d in style::declarations(style::CSS) {
            let row = GUIDE.lines().find(|l| l.starts_with(&format!("| `--{}` |", d.name)));
            let Some(row) = row else { panic!("--{} is not in the guide", d.name) };
            if let Some(kind) = sheet::step_kind(&d.name) {
                assert!(row.contains(if kind == sheet::Step::Gap { "a gap step" } else { "a size step" }), "{row}");
            }
        }
        let example = GUIDE.split("```css\n").nth(2).and_then(|s| s.split("```").next()).unwrap();
        assert!(example.contains("--trindle-board-lane: larger;"), "{example}");
        assert_eq!(lint(example), Vec::<String>::new());
        assert!(super::super::check(example).is_ok());
        let l = Layers::new(style::CSS, example);
        assert_eq!((l.step("trindle-board-lane", false), l.step("trindle-colour-swatch", false), l.step("trindle-gap-small", false)),
                   (Some(390.0), Some(46.0), Some(8.0)), "as the guide says");
    }

    #[test]
    fn what_check_refuses_is_said_first() {
        let said = lint(":root { --trindle-gap: url(x); }");
        assert!(said[0].contains("another file"), "{said:?}");
    }
}
