//! Checks style sheets: every number within its limits and every colour pair at its WCAG 2.2 AA floor; exit 1 on any problem. `trindle-sheet-check [file.css ...]`, by default the person's own sheet and themes.

use std::path::PathBuf;

use trindle_toolkit::common::sheet::{own_sheet, own_themes};
use trindle_toolkit::t;
use trindle_toolkit::theme::lint::lint;

fn main() {
    let mut files: Vec<PathBuf> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    if files.iter().any(|f| f.to_string_lossy().starts_with('-')) {
        println!("{}", t!("kit-sheet-usage"));
        return;
    }
    if files.is_empty() {
        files.push(own_sheet());
        if let Ok(read) = std::fs::read_dir(own_themes()) {
            files.extend(read.flatten().map(|e| e.path()).filter(|p| p.extension().is_some_and(|e| e == "css")));
        }
    }
    let mut failed = false;
    for f in files {
        let shown = f.display().to_string();
        let Ok(css) = std::fs::read_to_string(&f) else {
            println!("{}", t!("kit-sheet-unread", file = &shown));
            failed = true;
            continue;
        };
        let problems = lint(&css);
        if problems.is_empty() {
            println!("{}", t!("kit-sheet-fine", file = &shown));
        }
        for p in &problems {
            println!("{shown}: {p}");
        }
        failed |= !problems.is_empty();
    }
    if failed {
        std::process::exit(1);
    }
}
