//! Files on the clipboard, as other programs expect them: a file copied here can be pasted anywhere outside this program. A copy is offered by every name a program might ask for: the address list browsers, mail programs and Qt programs read, GNOME's own with its copy-or-cut line that GTK programs read, KDE's cut note, and the plain paths, so a text box that is pasted into gets the paths. A paste reads the same ways, so files copied in another program come here too.

use std::path::PathBuf;
use std::sync::Arc;

use super::address;

pub const URIS: &str = "text/uri-list";
pub const GNOME: &str = "x-special/gnome-copied-files";
pub const KDE_CUT: &str = "application/x-kde-cutselection";
pub const READ: [&str; 3] = [GNOME, URIS, KDE_CUT];

#[derive(Clone, Debug, PartialEq)]
pub struct Pasted {
    pub paths: Vec<PathBuf>,
    pub cut: bool,
}

pub fn kinds_for(paths: &[PathBuf], cut: bool) -> Vec<(String, Arc<Vec<u8>>)> {
    let uris: Vec<String> = paths.iter().map(|p| address::file_uri(p)).collect();
    let mut list = uris.join("\r\n");
    list.push_str("\r\n");
    let gnome = format!("{}\n{}", if cut { "cut" } else { "copy" }, uris.join("\n"));
    let plain = paths.iter().map(|p| p.display().to_string()).collect::<Vec<_>>().join("\n");
    let mut out: Vec<(String, Arc<Vec<u8>>)> = vec![
        (URIS.into(), Arc::new(list.into_bytes())),
        (GNOME.into(), Arc::new(gnome.into_bytes())),
        (KDE_CUT.into(), Arc::new(if cut { b"1".to_vec() } else { b"0".to_vec() })),
    ];
    let plain = Arc::new(plain.into_bytes());
    out.extend(crate::common::selection::TEXT.iter().map(|t| (t.to_string(), plain.clone())));
    out
}

pub fn paths_in(text: &str) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for line in text.lines().map(str::trim) {
        if line.is_empty() || line.starts_with('#') || line == "copy" || line == "cut" {
            continue;
        }
        let path = if line.starts_with("file://") { address::uri_path(line) } else if line.starts_with('/') { Some(PathBuf::from(line)) } else { None };
        match path.filter(|p| super::reach::meta(p, false).is_some()) {
            Some(p) => out.push(p),
            None => return Vec::new(),
        }
    }
    out
}

pub fn parse(got: &[(String, Vec<u8>)]) -> Pasted {
    let text = |name: &str| got.iter().find(|(k, _)| k == name).map(|(_, d)| String::from_utf8_lossy(d).into_owned());
    if let Some(g) = text(GNOME) {
        let cut = g.lines().next().is_some_and(|l| l.trim() == "cut");
        return Pasted { paths: paths_in(&g), cut };
    }
    let cut = text(KDE_CUT).is_some_and(|c| c.trim() == "1");
    if let Some(u) = text(URIS) {
        return Pasted { paths: paths_in(&u), cut };
    }
    let first = got.iter().find(|(k, _)| k.starts_with("text/") || matches!(k.as_str(), "UTF8_STRING" | "STRING" | "TEXT"));
    Pasted { paths: first.map(|(_, d)| paths_in(&String::from_utf8_lossy(d))).unwrap_or_default(), cut: false }
}

fn noted(paths: &[PathBuf]) -> String {
    paths.iter().map(|p| address::file_uri(p)).collect::<Vec<_>>().join("\n")
}

pub fn note_cut(at: &std::path::Path, paths: &[PathBuf]) {
    if let Err(e) = crate::common::files::write_whole_private(at, noted(paths).as_bytes()) {
        eprintln!("trindle: files: the note of what was cut: {e}");
    }
}

pub fn forget_cut(at: &std::path::Path, paths: Option<&[PathBuf]>) {
    let still = paths.is_none_or(|p| crate::common::files::read_small(at).is_ok_and(|text| text == noted(p)));
    if still {
        let _ = std::fs::remove_file(at);
    }
}

pub fn moves(named: &[PathBuf], said_cut: bool, own: Option<(&[PathBuf], bool)>, note: Option<&std::path::Path>) -> bool {
    match own {
        Some((paths, cut)) if paths == named => cut,
        _ => said_cut && note.is_some_and(|at| crate::common::files::read_small(at).is_ok_and(|text| text == noted(named))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bytes(k: &[(String, Arc<Vec<u8>>)], name: &str) -> String {
        String::from_utf8(k.iter().find(|(n, _)| n == name).unwrap_or_else(|| panic!("no {name}")).1.to_vec()).unwrap()
    }

    #[test]
    fn a_copy_is_offered_by_every_name_other_programs_ask_for() {
        let paths = vec![PathBuf::from("/tmp/a b.txt"), PathBuf::from("/tmp/c")];
        let k = kinds_for(&paths, false);
        assert_eq!(bytes(&k, URIS), "file:///tmp/a%20b.txt\r\nfile:///tmp/c\r\n");
        assert_eq!(bytes(&k, GNOME), "copy\nfile:///tmp/a%20b.txt\nfile:///tmp/c");
        assert_eq!(bytes(&k, KDE_CUT), "0");
        for t in crate::common::selection::TEXT {
            assert_eq!(bytes(&k, t), "/tmp/a b.txt\n/tmp/c", "{t} is the paths");
        }
    }

    #[test]
    fn a_cut_says_cut_in_both_notes() {
        let k = kinds_for(&[PathBuf::from("/tmp/x")], true);
        assert_eq!(bytes(&k, GNOME), "cut\nfile:///tmp/x");
        assert_eq!(bytes(&k, KDE_CUT), "1");
    }

    #[test]
    fn what_was_offered_is_read_back_in_every_way() {
        let dir = std::env::temp_dir().join(format!("trindle-clip-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("one two.txt");
        std::fs::write(&file, b"x").unwrap();
        for cut in [false, true] {
            let offered: Vec<(String, Vec<u8>)> = kinds_for(std::slice::from_ref(&file), cut).into_iter().map(|(k, d)| (k, d.to_vec())).collect();
            let named = |names: &[&str]| -> Vec<(String, Vec<u8>)> { offered.iter().filter(|(k, _)| names.contains(&k.as_str())).cloned().collect() };
            let want = Pasted { paths: vec![file.clone()], cut };
            assert_eq!(parse(&named(&[GNOME])), want, "GNOME's list");
            assert_eq!(parse(&named(&[URIS, KDE_CUT])), want, "the address list with KDE's note");
            let plain = parse(&named(&["text/plain"]));
            assert_eq!((plain.paths, plain.cut), (vec![file.clone()], false), "plain paths are a copy");
        }
        let uri = address::file_uri(&file);
        let theirs = vec![(URIS.to_string(), format!("{uri}\r\n").into_bytes())];
        assert_eq!(parse(&theirs), Pasted { paths: vec![file.clone()], cut: false });
        let words = vec![("text/plain".to_string(), b"just some words".to_vec())];
        assert!(parse(&words).paths.is_empty());
        assert!(parse(&[]).paths.is_empty());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[cfg(feature = "ui")]
    #[test]
    fn places_are_looked_at_only_through_reach() {
        let text = crate::common::source::code(include_str!("clip.rs"));
        let code = &text[..text.find("#[cfg(test)]").unwrap_or(text.len())];
        for look in [".is_dir()", ".exists()", ".is_file()", "fs::metadata(", "symlink_metadata(", "read_dir("] {
            assert!(!code.contains(look), "it looks at a place with {look}, not through `reach`");
        }
    }
}
