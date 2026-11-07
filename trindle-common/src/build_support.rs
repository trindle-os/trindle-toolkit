//! For a program's build.rs: lists the English word files into the program, so a new file needs no line of code.

use std::path::Path;

pub fn english_list(dir: &Path) -> String {
    let read = std::fs::read_dir(dir).unwrap_or_else(|e| panic!("cannot read {}: {e}", dir.display()));
    let mut names: Vec<String> = read.flatten().filter_map(|e| e.file_name().into_string().ok()).filter(|n| n.ends_with(".ftl")).collect();
    assert!(!names.is_empty(), "{} holds no .ftl file, so the program would have no words", dir.display());
    names.sort();
    let mut out = String::from("&[\n");
    for n in &names {
        out.push_str(&format!("    ({n:?}, include_str!({:?})),\n", dir.join(n).display().to_string()));
    }
    out.push_str("]\n");
    out
}

pub fn write_english_list() {
    let root = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    println!("cargo:rerun-if-changed=share/lang/en");
    let text = english_list(&Path::new(&root).join("share/lang/en"));
    std::fs::write(Path::new(&std::env::var("OUT_DIR").unwrap()).join("english.rs"), text).unwrap();
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_list_names_each_file_in_order() {
        let dir = std::env::temp_dir().join(format!("trindle-common-ftl-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("b.ftl"), "x = 1").unwrap();
        std::fs::write(dir.join("a.ftl"), "x = 1").unwrap();
        std::fs::write(dir.join("notes.txt"), "no").unwrap();
        let text = super::english_list(&dir);
        let (a, b) = (text.find("\"a.ftl\"").unwrap(), text.find("\"b.ftl\"").unwrap());
        assert!(a < b && !text.contains("notes"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
