//! A settings file in the `[Section]` and `Key=value` form Trindle's programs keep.

use std::fmt::Write as _;
use std::path::Path;

use crate::files::{fits_a_line, read_small, write_private};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Ini {
    sections: Vec<(String, Vec<(String, String)>)>,
}

impl Ini {
    pub fn parse(text: &str) -> Ini {
        let mut ini = Ini::default();
        let mut at: Option<usize> = None;
        for line in text.strip_prefix('\u{feff}').unwrap_or(text).lines().map(str::trim) {
            if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
                continue;
            }
            if let Some(name) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
                at = ini.sections.iter().position(|(s, _)| s == name).or_else(|| {
                    ini.sections.push((name.to_string(), Vec::new()));
                    Some(ini.sections.len() - 1)
                });
            } else if let (Some(i), Some((k, v))) = (at, line.split_once('=')) {
                ini.sections[i].1.push((k.trim().to_string(), v.trim().to_string()));
            }
        }
        ini
    }

    pub fn read(path: &Path) -> std::io::Result<Ini> {
        read_small(path).map(|t| Ini::parse(&t))
    }

    pub fn get(&self, section: &str, key: &str) -> Option<&str> {
        self.sections.iter().find(|(s, _)| s == section)?.1.iter().rev().find(|(k, _)| k == key).map(|(_, v)| v.as_str())
    }

    pub fn get_bool(&self, section: &str, key: &str) -> Option<bool> {
        self.get(section, key).and_then(|v| v.parse().ok())
    }

    pub fn get_list(&self, section: &str, key: &str) -> Vec<&str> {
        self.get(section, key).map(|v| v.split(';').filter(|p| !p.is_empty()).collect()).unwrap_or_default()
    }

    pub fn set(&mut self, section: &str, key: &str, value: &str) -> bool {
        if ![section, key, value].iter().all(|v| fits_a_line(v)) || key.contains('=') || key.starts_with(['#', ';', '[']) || section.contains(']') {
            return false;
        }
        let at = match self.sections.iter().position(|(s, _)| s == section) {
            Some(i) => i,
            None => {
                self.sections.push((section.to_string(), Vec::new()));
                self.sections.len() - 1
            }
        };
        match self.sections[at].1.iter_mut().rev().find(|(k, _)| k == key) {
            Some(kv) => kv.1 = value.to_string(),
            None => self.sections[at].1.push((key.to_string(), value.to_string())),
        }
        true
    }

    pub fn text(&self) -> String {
        let mut out = String::new();
        for (name, pairs) in &self.sections {
            let _ = writeln!(out, "[{name}]");
            for (k, v) in pairs {
                let _ = writeln!(out, "{k}={v}");
            }
            out.push('\n');
        }
        out
    }

    pub fn write(&self, path: &Path) -> std::io::Result<()> {
        write_private(path, self.text().as_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_setting_is_read_changed_and_written_back() {
        let mut ini = Ini::parse("\u{feff}# note\n[Settings]\nname = Ann\nlist=a;b;;c\nOn=true\n");
        assert_eq!(ini.get("Settings", "name"), Some("Ann"));
        assert_eq!(ini.get_list("Settings", "list"), vec!["a", "b", "c"]);
        assert_eq!(ini.get_bool("Settings", "On"), Some(true));
        assert!(ini.set("Settings", "name", "Bo"));
        assert!(ini.set("Other", "k", "v"));
        assert_eq!(Ini::parse(&ini.text()), ini);
    }

    #[test]
    fn a_leading_byte_order_mark_is_ignored_and_a_repeated_section_is_merged() {
        let ini = Ini::parse("\u{feff}[A]\nx=1\n[B]\ny=2\n[A]\nz=3\n");
        assert_eq!((ini.get("A", "x"), ini.get("A", "z"), ini.get("B", "y")), (Some("1"), Some("3"), Some("2")));
    }

    #[test]
    fn a_value_that_would_write_a_second_setting_is_refused() {
        let mut ini = Ini::default();
        assert!(!ini.set("S", "k", "a\nb=c"));
        assert!(!ini.set("S", "k=", "v"));
        assert!(!ini.set("S]", "k", "v"));
        assert!(!ini.set("S", "#k", "v") && !ini.set("S", ";k", "v") && !ini.set("S", "[x", "y]"));
        assert_eq!(ini.text(), "");
    }

    #[test]
    fn set_changes_the_value_get_reads_when_a_key_is_repeated() {
        let mut ini = Ini::parse("[S]\nk=1\nk=2\n");
        assert!(ini.set("S", "k", "3"));
        assert_eq!(ini.get("S", "k"), Some("3"));
    }

    #[test]
    fn a_file_is_written_private_and_read_back() {
        let dir = std::env::temp_dir().join(format!("trindle-common-ini-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("a.ini");
        assert_eq!(Ini::read(&path).unwrap(), Ini::default());
        let mut ini = Ini::default();
        ini.set("S", "k", "v");
        ini.write(&path).unwrap();
        assert_eq!(Ini::read(&path).unwrap().get("S", "k"), Some("v"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
