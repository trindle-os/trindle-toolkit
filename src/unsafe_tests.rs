//! Counts every `unsafe` in the source against the audited ledger below, so a new one cannot come in unseen.

const LEDGER: [(&str, usize); 10] = [
    ("src/sys/disk.rs", 17),
    ("src/sys/fd.rs", 13),
    ("src/sys/gbm.rs", 11),
    ("src/sys/gl.rs", 31),
    ("src/sys/lean.rs", 2),
    ("src/sys/mem.rs", 10),
    ("src/sys/os.rs", 14),
    ("src/sys/signal.rs", 7),
    ("src/sys/sock.rs", 32),
    ("trindle-common/src/sys.rs", 1),
];

fn count(text: &str) -> usize {
    text.lines().map(|l| l.split("//").next().unwrap_or(""))
        .map(|l| ["unsafe {", "unsafe fn", "unsafe impl", "unsafe extern"].iter().map(|k| l.matches(k).count()).sum::<usize>()).sum()
}

fn walk(dir: &std::path::Path, into: &mut Vec<std::path::PathBuf>) {
    for e in std::fs::read_dir(dir).unwrap().flatten() {
        let p = e.path();
        if p.is_dir() {
            walk(&p, into);
        } else if p.extension().is_some_and(|x| x == "rs") {
            into.push(p);
        }
    }
}

#[test]
fn every_unsafe_is_in_the_audited_ledger() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    for dir in ["src", "trindle-common/src", "examples", "tests"] {
        walk(&root.join(dir), &mut files);
    }
    let mut wrong = Vec::new();
    for f in files {
        let name = f.strip_prefix(root).unwrap().display().to_string();
        let n = count(&std::fs::read_to_string(&f).unwrap());
        let allowed = LEDGER.iter().find(|l| l.0 == name).map_or(0, |l| l.1);
        if n > 0 && allowed == 0 && name != "src/unsafe_tests.rs" {
            wrong.push(format!("{name} has {n} unsafe, but unsafe belongs in the sys modules alone"));
        } else if n != allowed && name != "src/unsafe_tests.rs" {
            wrong.push(format!("{name} has {n} unsafe, the ledger says {allowed}"));
        }
    }
    assert!(wrong.is_empty(), "{wrong:#?}");
    assert!(LEDGER.iter().all(|l| l.0.starts_with("src/sys/") || l.0 == "trindle-common/src/sys.rs"), "the ledger names a file outside the sys modules");
}

#[test]
fn the_count_passes_over_comments() {
    assert_eq!(count("let a = unsafe { f() }; // unsafe { g() }\n// unsafe fn h()\nunsafe impl Send for X {}"), 2);
}
