//! Lists the words each language still lacks, and fails (exit 1) when English and the code disagree. Run it from a checkout, or give the folder: `trindle-lang-check [folder]`. `--missing [language]` also lists every word each language (or one) lacks. See share/lang/README.md.

use std::path::PathBuf;

use trindle_toolkit::lang::check::{check, domains_at};
use trindle_toolkit::t;

fn main() {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    let missing = args.first().is_some_and(|a| a == "--missing");
    if missing {
        args.remove(0);
    }
    if args.first().is_some_and(|a| a.starts_with('-')) {
        println!("{}", t!("kit-lang-check-usage"));
        return;
    }
    let only = (missing && args.first().is_some_and(|a| !std::path::Path::new(a).is_dir())).then(|| args.remove(0));
    let root: PathBuf = args.first().map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."));
    let domains = domains_at(&root);
    if domains.is_empty() {
        eprintln!("{}", t!("kit-lang-check-no-english", folder = root.display().to_string()));
        std::process::exit(2);
    }
    let report = check(&domains);
    if missing {
        print!("{}", report.missing_text(only.as_deref()));
    }
    print!("{}", report.text());
    if !report.problems.is_empty() {
        std::process::exit(1);
    }
}
