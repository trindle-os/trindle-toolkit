//! Makes a new program from the template in share/template: a window, a style sheet, a language file and a test. `trindle-new <name> [--toolkit <path>]`.

use std::path::{Path, PathBuf};

use trindle_toolkit::t;

const FILES: [(&str, &str); 4] = [
    ("src/lib.rs", include_str!(concat!("../../share/template/src/lib", ".rs"))),
    ("share/lang/en/app.ftl", include_str!("../../share/template/share/lang/en/app.ftl")),
    ("share/style.css", include_str!("../../share/template/share/style.css")),
    (".gitignore", "/target\n"),
];

fn fail(said: String) -> ! {
    eprintln!("{said}");
    std::process::exit(1);
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (name, toolkit) = match args.as_slice() {
        [n] if !n.starts_with('-') => (n.clone(), None),
        [n, flag, p] if flag == "--toolkit" => (n.clone(), Some(p.clone())),
        _ => {
            println!("{}", t!("kit-new-usage"));
            return;
        }
    };
    let good = name.starts_with(|c: char| c.is_ascii_lowercase()) && name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
    if !good {
        fail(t!("kit-new-bad-name", name = &name));
    }
    let root = PathBuf::from(&name);
    if root.exists() {
        fail(t!("kit-new-exists", name = &name));
    }
    let version = env!("CARGO_PKG_VERSION");
    let toolkit = match toolkit {
        Some(p) => format!("{{ path = {:?}, version = \"{version}\"", Path::new(&p).canonicalize().unwrap_or_else(|_| fail(t!("kit-new-no-toolkit", path = &p))).display().to_string()),
        None => format!("{{ version = \"{version}\""),
    };
    let crate_name = name.replace('-', "_");
    let cargo = format!("[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\ntrindle-toolkit = {toolkit} }}\n\n[dev-dependencies]\ntrindle-toolkit = {toolkit}, features = [\"testing\"] }}\n");
    let main = format!("fn main() {{\n    if let Err(why) = {crate_name}::run(\"org.example.{crate_name}\") {{\n        eprintln!(\"{{why}}\");\n        std::process::exit(1);\n    }}\n}}\n");
    let all = FILES.iter().map(|(p, text)| (*p, text.to_string())).chain([("Cargo.toml", cargo), ("src/main.rs", main)]);
    for (path, text) in all {
        let at = root.join(path);
        let written = at.parent().map_or(Ok(()), std::fs::create_dir_all).and_then(|()| trindle_toolkit::common::files::write_whole(&at, text.as_bytes()));
        if written.is_err() {
            fail(t!("kit-new-unwritten", path = at.display().to_string()));
        }
    }
    println!("{}", t!("kit-new-made", name = &name));
}
