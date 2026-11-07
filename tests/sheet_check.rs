//! The trindle-sheet-check command passes the shipped themes and fails a sheet with a problem.

use std::process::Command;

#[test]
fn the_command_passes_the_themes_and_fails_a_bad_sheet() {
    let bin = env!("CARGO_BIN_EXE_trindle-sheet-check");
    let themes = concat!(env!("CARGO_MANIFEST_DIR"), "/share/themes");
    let all: Vec<_> = std::fs::read_dir(themes).unwrap().flatten().map(|e| e.path()).collect();
    let out = Command::new(bin).args(&all).output().unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stdout));
    let bad = std::env::temp_dir().join(format!("trindle-sheet-check-{}.css", std::process::id()));
    std::fs::write(&bad, ":root { --trindle-page-padding: 90; }").unwrap();
    let out = Command::new(bin).arg(&bad).output().unwrap();
    let _ = std::fs::remove_file(&bad);
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stdout).contains("--trindle-page-padding is 90"));
}
