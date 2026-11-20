//! For tests: runs a check once in each language the toolkit ships, with that language's own words.

use super::{english, folder_files, with_catalogue, Bundle, Catalogue};
use std::path::Path;

pub(crate) fn tags() -> Vec<String> {
    let mut tags: Vec<String> = std::fs::read_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("share/lang"))
        .unwrap().flatten().filter(|e| e.path().is_dir()).filter_map(|e| e.file_name().into_string().ok()).collect();
    tags.sort();
    tags
}

fn long_lines(tag: &str) -> Vec<String> {
    let mut lines: Vec<String> = folder_files(&Path::new(env!("CARGO_MANIFEST_DIR")).join("share/lang").join(tag)).iter()
        .flat_map(|(_, text)| text.lines().filter_map(|l| l.split_once(" = ").map(|(_, v)| v.trim().to_string())).collect::<Vec<_>>())
        .filter(|v| !v.contains(['{', '}', '#', '[', '*']) && v.chars().count() > 18)
        .collect();
    if tag != "en" && lines.len() < 8 {
        lines.extend(long_lines("en"));
    }
    lines.sort_by_key(|l| std::cmp::Reverse(l.chars().count()));
    lines.dedup();
    lines
}

pub(crate) fn each_language(mut check: impl FnMut(&str, &[String])) {
    for tag in tags() {
        let files = folder_files(&Path::new(env!("CARGO_MANIFEST_DIR")).join("share/lang").join(&tag));
        let files: Vec<(&str, &str)> = files.iter().map(|(a, b)| (a.as_str(), b.as_str())).collect();
        let long = long_lines(&tag);
        assert!(long.len() >= 4, "{tag}: too few long lines to lay out ({})", long.len());
        with_catalogue(Catalogue::new(Bundle::from_files(&tag, &files), english()), || check(&tag, &long));
    }
}
