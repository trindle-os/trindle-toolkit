//! trindle-lang-check says its help and its one error in words from a language file, not from the code.

use std::process::Command;

#[test]
fn the_checker_gives_its_help_in_words_from_a_language_file() {
    let bin = env!("CARGO_BIN_EXE_trindle-lang-check");
    let help = Command::new(bin).arg("--help").output().unwrap();
    let said = String::from_utf8_lossy(&help.stdout);
    assert!(help.status.success() && said.contains("Usage: trindle-lang-check") && said.contains("--missing lists every word"), "{said}");
    let source = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/bin/trindle-lang-check.rs")).unwrap();
    assert!(!source.contains("Usage:") && !source.contains("No share/lang"), "the words are in a language file");
}
