//! A program's own desktop entry: its launcher, the kinds of file and address it opens, the default for them, and starting at login.

use std::io;
use std::path::{Path, PathBuf};

use trindle_common::files::{config_home, data_home, fits_a_line};
use trindle_common::ini::Ini;

const SECTION: &str = "Desktop Entry";
const DEFAULTS: &str = "Default Applications";

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Entry {
    pub app_id: String,
    pub name: String,
    pub comment: String,
    pub icon: String,
    pub exec: Vec<String>,
    pub kinds: Vec<String>,
    pub schemes: Vec<String>,
    pub unlisted: bool,
}

impl Entry {
    pub fn new(app_id: &str, name: &str, exec: &[&str]) -> Entry {
        Entry { app_id: app_id.into(), name: name.into(), exec: exec.iter().map(|a| a.to_string()).collect(), ..Entry::default() }
    }

    pub fn text(&self) -> Option<String> {
        if !crate::keep::plain_id(&self.app_id) || self.name.is_empty() || self.exec.is_empty() {
            return None;
        }
        let kinds: Vec<String> = self.kinds.iter().cloned().chain(self.schemes.iter().map(|s| scheme_kind(s))).collect();
        if !self.kinds.iter().all(|k| is_kind(k)) || !self.schemes.iter().all(|s| is_scheme(s)) {
            return None;
        }
        let mut exec = self.exec.iter().map(|a| quoted(a)).collect::<Option<Vec<_>>>()?.join(" ");
        if !self.schemes.is_empty() {
            exec.push_str(" %U");
        } else if !kinds.is_empty() {
            exec.push_str(" %F");
        }
        let mut ini = Ini::default();
        let mut ok = ini.set(SECTION, "Type", "Application") && ini.set(SECTION, "Name", &escaped(&self.name)) && ini.set(SECTION, "Exec", &exec);
        for (k, v) in [("Comment", &self.comment), ("Icon", &self.icon)] {
            ok &= v.is_empty() || ini.set(SECTION, k, &escaped(v));
        }
        if !kinds.is_empty() {
            ok &= ini.set(SECTION, "MimeType", &format!("{};", kinds.join(";")));
        }
        if self.unlisted {
            ok &= ini.set(SECTION, "NoDisplay", "true");
        }
        ok.then(|| ini.text())
    }

    pub fn install(&self) -> io::Result<PathBuf> {
        self.write_in(&data_home().ok_or(io::ErrorKind::NotFound)?.join("applications"))
    }

    pub fn autostart(&self, on: bool) -> io::Result<()> {
        let dir = config_home().ok_or(io::ErrorKind::NotFound)?.join("autostart");
        if on {
            return self.write_in(&dir).map(drop);
        }
        match std::fs::remove_file(dir.join(self.file_name())) {
            Err(e) if e.kind() != io::ErrorKind::NotFound => Err(e),
            _ => Ok(()),
        }
    }

    fn file_name(&self) -> String {
        format!("{}.desktop", self.app_id)
    }

    fn write_in(&self, dir: &Path) -> io::Result<PathBuf> {
        let text = self.text().ok_or(io::ErrorKind::InvalidInput)?;
        std::fs::create_dir_all(dir)?;
        let path = dir.join(self.file_name());
        trindle_common::files::write_private(&path, text.as_bytes())?;
        Ok(path)
    }
}

pub fn scheme_kind(scheme: &str) -> String {
    format!("x-scheme-handler/{scheme}")
}

pub fn make_default(app_id: &str, kinds: &[&str]) -> io::Result<()> {
    make_default_in(&config_home().ok_or(io::ErrorKind::NotFound)?.join("mimeapps.list"), app_id, kinds)
}

pub fn default_for(kind: &str) -> Option<String> {
    default_in(&config_home()?.join("mimeapps.list"), kind)
}

pub fn openers(kind: &str) -> Vec<String> {
    let system = std::env::var("XDG_DATA_DIRS").ok().filter(|v| !v.is_empty()).unwrap_or_else(|| "/usr/local/share:/usr/share".into());
    let dirs = data_home().into_iter().chain(system.split(':').map(PathBuf::from).filter(|p| p.is_absolute()));
    openers_in(dirs.map(|d| d.join("applications")), kind)
}

fn make_default_in(path: &Path, app_id: &str, kinds: &[&str]) -> io::Result<()> {
    if !crate::keep::plain_id(app_id) || !kinds.iter().all(|k| is_kind(k)) {
        return Err(io::ErrorKind::InvalidInput.into());
    }
    let mut ini = Ini::read(path)?;
    for k in kinds {
        if !ini.set(DEFAULTS, k, &format!("{app_id}.desktop;")) {
            return Err(io::ErrorKind::InvalidInput.into());
        }
    }
    ini.write(path)
}

fn default_in(path: &Path, kind: &str) -> Option<String> {
    Ini::read(path).ok()?.get_list(DEFAULTS, kind).first().map(|d| d.to_string())
}

fn openers_in(dirs: impl Iterator<Item = PathBuf>, kind: &str) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    for dir in dirs {
        let Ok(names) = std::fs::read_dir(&dir) else { continue };
        let mut here: Vec<(String, PathBuf)> = names.flatten().filter_map(|e| {
            let name = e.file_name().into_string().ok()?;
            name.ends_with(".desktop").then(|| (name, e.path()))
        }).collect();
        here.sort();
        for (name, path) in here {
            if found.contains(&name) {
                continue;
            }
            let Ok(ini) = Ini::read(&path) else { continue };
            if ini.get_list(SECTION, "MimeType").contains(&kind) && ini.get_bool(SECTION, "Hidden") != Some(true) {
                found.push(name);
            }
        }
    }
    found
}

fn is_kind(k: &str) -> bool {
    let Some((a, b)) = k.split_once('/') else { return false };
    let part = |p: &str| !p.is_empty() && p.bytes().all(|c| c.is_ascii_alphanumeric() || b"!#$&^_.+-".contains(&c));
    part(a) && part(b) && k.len() <= 255
}

fn is_scheme(s: &str) -> bool {
    s.bytes().next().is_some_and(|c| c.is_ascii_alphabetic()) && s.bytes().all(|c| c.is_ascii_alphanumeric() || b"+.-".contains(&c))
}

fn escaped(v: &str) -> String {
    v.replace('\\', "\\\\")
}

fn quoted(arg: &str) -> Option<String> {
    if arg.is_empty() || !fits_a_line(arg) {
        return None;
    }
    let arg = arg.replace('%', "%%");
    let plain = arg.chars().all(|c| !" \t\"'\\><~|&;$*?#()`=".contains(c));
    if plain {
        return Some(arg);
    }
    let mut out = String::from("\"");
    for c in arg.chars() {
        if "\"`$\\".contains(c) {
            out.push('\\');
        }
        out.push(c);
    }
    out.push('"');
    Some(escaped(&out))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("trindle-entry-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn notes() -> Entry {
        let mut e = Entry::new("org.example.Notes", "Notes", &["/usr/bin/notes"]);
        e.icon = "accessories-text-editor".into();
        e.kinds = vec!["text/plain".into(), "text/markdown".into()];
        e
    }

    #[test]
    fn a_launcher_opens_files_and_lists_its_kinds() {
        let ini = Ini::parse(&notes().text().unwrap());
        assert_eq!(ini.get(SECTION, "Type"), Some("Application"));
        assert_eq!(ini.get(SECTION, "Exec"), Some("/usr/bin/notes %F"));
        assert_eq!(ini.get_list(SECTION, "MimeType"), ["text/plain", "text/markdown"]);
        assert_eq!(ini.get(SECTION, "NoDisplay"), None);
    }

    #[test]
    fn an_address_handler_is_handed_addresses() {
        let mut e = Entry::new("org.example.Mail", "Mail", &["mail"]);
        e.schemes = vec!["mailto".into()];
        e.unlisted = true;
        let ini = Ini::parse(&e.text().unwrap());
        assert_eq!(ini.get(SECTION, "Exec"), Some("mail %U"));
        assert_eq!(ini.get_list(SECTION, "MimeType"), ["x-scheme-handler/mailto"]);
        assert_eq!(ini.get_bool(SECTION, "NoDisplay"), Some(true));
    }

    #[test]
    fn arguments_are_quoted_as_the_format_asks() {
        let e = Entry::new("org.example.Notes", "Notes", &["/opt/My Notes/notes", "--restore", "100%", "a\"b$c"]);
        let ini = Ini::parse(&e.text().unwrap());
        assert_eq!(ini.get(SECTION, "Exec"), Some(r#""/opt/My Notes/notes" --restore 100%% "a\\"b\\$c""#));
    }

    #[test]
    fn a_value_that_would_add_a_line_or_a_kind_is_refused() {
        for bad in [Entry::new("org.example.Notes", "Notes\nExec=evil", &["notes"]), Entry::new("../x", "Notes", &["notes"]),
                    Entry::new("org.example.Notes", "Notes", &["notes\n"]), Entry::new("org.example.Notes", "", &["notes"]),
                    Entry::new("org.example.Notes", "Notes", &[])] {
            assert_eq!(bad.text(), None, "{bad:?}");
        }
        let mut e = notes();
        e.kinds = vec!["text/plain;evil/x".into()];
        assert_eq!(e.text(), None);
        let mut e = notes();
        e.schemes = vec!["1http".into()];
        assert_eq!(e.text(), None);
    }

    #[test]
    fn the_default_is_kept_beside_the_others() {
        let dir = scratch("defaults");
        let list = dir.join("mimeapps.list");
        std::fs::write(&list, "[Default Applications]\nimage/png=org.example.Pictures.desktop;\n[Added Associations]\ntext/plain=a.desktop;\n").unwrap();
        make_default_in(&list, "org.example.Notes", &["text/plain", &scheme_kind("notes")]).unwrap();
        assert_eq!(default_in(&list, "text/plain").as_deref(), Some("org.example.Notes.desktop"));
        assert_eq!(default_in(&list, "x-scheme-handler/notes").as_deref(), Some("org.example.Notes.desktop"));
        assert_eq!(default_in(&list, "image/png").as_deref(), Some("org.example.Pictures.desktop"));
        assert_eq!(Ini::read(&list).unwrap().get("Added Associations", "text/plain"), Some("a.desktop;"));
        assert!(make_default_in(&list, "org.example.Notes", &["text/plain\n"]).is_err());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn openers_come_from_the_persons_folder_first_and_each_once() {
        let (mine, system) = (scratch("mine"), scratch("system"));
        notes().write_in(&mine).unwrap();
        notes().write_in(&system).unwrap();
        let mut other = Entry::new("org.example.Other", "Other", &["other"]);
        other.kinds = vec!["text/plain".into()];
        other.write_in(&system).unwrap();
        let mut gone = Entry::new("org.example.Gone", "Gone", &["gone"]);
        gone.kinds = vec!["image/png".into()];
        gone.write_in(&system).unwrap();
        assert_eq!(openers_in([mine.clone(), system.clone()].into_iter(), "text/plain"), ["org.example.Notes.desktop", "org.example.Other.desktop"]);
        assert_eq!(openers_in([mine.clone(), system.clone()].into_iter(), "video/mp4"), Vec::<String>::new());
        std::fs::remove_dir_all(&mine).unwrap();
        std::fs::remove_dir_all(&system).unwrap();
    }

    #[test]
    fn an_entry_written_reads_back_the_same() {
        let dir = scratch("write");
        let path = notes().write_in(&dir).unwrap();
        assert_eq!(path, dir.join("org.example.Notes.desktop"));
        assert_eq!(std::fs::read_to_string(&path).unwrap(), notes().text().unwrap());
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
