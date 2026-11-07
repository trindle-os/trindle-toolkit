//! trindle-new writes a whole program from the template, and refuses a bad name or a folder already there.

use std::process::Command;

#[test]
fn trindle_new_writes_the_template_and_refuses_what_it_cannot_make() {
    let bin = env!("CARGO_BIN_EXE_trindle-new");
    let kit = env!("CARGO_MANIFEST_DIR");
    let dir = std::env::temp_dir().join(format!("trindle-new-test-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let made = Command::new(bin).args(["my-notes", "--toolkit", kit]).current_dir(&dir).output().unwrap();
    assert!(made.status.success(), "{}", String::from_utf8_lossy(&made.stderr));
    let root = dir.join("my-notes");
    let read = |p: &str| std::fs::read_to_string(root.join(p)).unwrap();
    assert_eq!(read("src/lib.rs"), std::fs::read_to_string(format!("{kit}/share/template/src/lib.rs")).unwrap());
    assert_eq!(read("share/lang/en/app.ftl"), std::fs::read_to_string(format!("{kit}/share/template/share/lang/en/app.ftl")).unwrap());
    assert!(read("share/style.css").contains("--trindle-card-corner"));
    assert!(read("src/main.rs").contains("my_notes::run(\"org.example.my_notes\")"));
    let cargo = read("Cargo.toml");
    assert!(cargo.contains("name = \"my-notes\"") && cargo.contains("features = [\"testing\"]") && cargo.contains("path = "), "{cargo}");
    let again = Command::new(bin).arg("my-notes").current_dir(&dir).output().unwrap();
    assert_eq!(again.status.code(), Some(1));
    let bad = Command::new(bin).arg("My Notes").current_dir(&dir).output().unwrap();
    assert_eq!(bad.status.code(), Some(1));
    assert!(!dir.join("My Notes").exists());
    std::fs::remove_dir_all(&dir).unwrap();
}
