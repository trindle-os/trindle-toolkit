//! For tests that read source as text: the code alone, with its comments taken out.

pub fn code(text: &'static str) -> &'static str {
    use std::collections::HashMap;
    use std::sync::Mutex;
    static DONE: Mutex<Option<HashMap<usize, &'static str>>> = Mutex::new(None);
    let key = text.as_ptr() as usize;
    if let Some(done) = DONE.lock().ok().and_then(|d| d.as_ref()?.get(&key).copied()) {
        return done;
    }
    let mut out = String::with_capacity(text.len());
    for line in text.lines() {
        let cut = line.match_indices("//").map(|(at, _)| at).find(|at| *at == 0 || line[..*at].ends_with([' ', '\t']));
        out.push_str(&line[..cut.unwrap_or(line.len())]);
        out.push('\n');
    }
    let out: &'static str = Box::leak(out.into_boxed_str());
    if let Ok(mut done) = DONE.lock() {
        done.get_or_insert_with(HashMap::new).insert(key, out);
    }
    out
}

pub fn root() -> &'static std::path::Path {
    static ROOT: std::sync::OnceLock<std::path::PathBuf> = std::sync::OnceLock::new();
    ROOT.get_or_init(|| {
        if let Some(named) = std::env::var_os("TRINDLE_SOURCE_ROOT").filter(|v| !v.is_empty()) {
            return named.into();
        }
        match (std::env::var_os("CARGO_PKG_NAME"), std::env::var_os("CARGO_MANIFEST_DIR")) {
            (Some(name), Some(dir)) if name == "trindle" => dir.into(),
            _ => toolkit_root().to_path_buf(),
        }
    })
}

pub fn toolkit_root() -> &'static std::path::Path {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
}

pub fn trees() -> Vec<(std::path::PathBuf, &'static str)> {
    let toolkit = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    if root() == toolkit {
        vec![(toolkit.join("src"), "toolkit/")]
    } else {
        vec![(root().join("src"), ""), (toolkit.join("src"), "toolkit/")]
    }
}

pub fn path_of(name: &str) -> std::path::PathBuf {
    match name.strip_prefix("toolkit/") {
        Some(rest) => std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join(rest),
        None => root().join("src").join(name),
    }
}

pub fn every_file() -> Vec<(String, String)> {
    let mut all = Vec::new();
    for (src, named) in trees().into_iter().filter(|(src, _)| src.is_dir()) {
        let mut folders = vec![src.clone()];
        while let Some(folder) = folders.pop() {
            for e in std::fs::read_dir(&folder).unwrap_or_else(|e| panic!("{}: {e}", folder.display())) {
                let path = e.unwrap_or_else(|e| panic!("{}: {e}", folder.display())).path();
                if path.is_dir() {
                    folders.push(path);
                } else if path.extension().is_some_and(|x| x == "rs") {
                    let name = path.strip_prefix(&src).unwrap_or(&path).to_string_lossy().into_owned();
                    all.push((format!("{named}{name}"), std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))));
                }
            }
        }
    }
    all.sort();
    all
}

#[cfg(test)]
mod tests {
    use super::code;

    fn read_whole(text: &str) -> usize {
        let reads = concat!("include_", "str!(\"");
        text.match_indices(reads)
            .filter(|(at, _)| {
                let named = text[at + reads.len()..].split('"').next().unwrap_or_default();
                named.ends_with(".rs") && !text[..*at].ends_with("code(")
            })
            .count()
    }

    fn scan(dir: &std::path::Path, src: &std::path::Path, found: &mut Vec<(String, usize)>) {
        let Ok(list) = std::fs::read_dir(dir) else { return };
        for entry in list.flatten() {
            let path = entry.path();
            if path.is_dir() {
                scan(&path, src, found);
                continue;
            }
            let Ok(text) = std::fs::read_to_string(&path) else { continue };
            let count = read_whole(&text);
            if count > 0 {
                found.push((path.strip_prefix(src).unwrap_or(&path).to_string_lossy().into_owned(), count));
            }
        }
    }

    #[test]
    fn every_test_that_reads_source_reads_it_without_its_comments() {
        let whole = concat!("let s = include_", "str!(\"mod.rs\");");
        let through = concat!("let s = crate::common::source::code(include_", "str!(\"mod.rs\"));");
        let other = concat!("let s = include_", "str!(\"../../share/style.css\");");
        assert_eq!((read_whole(whole), read_whole(through), read_whole(other)), (1, 0, 0));
        let mut found = Vec::new();
        for (src, _) in super::trees() {
            scan(&src, &src, &mut found);
            assert!(std::fs::read_dir(&src).is_ok(), "the source was not found at {}", src.display());
        }
        assert!(found.is_empty(), "these read a source file with its comments, each so many times: {found:?}. Read it through common::source::code");
    }

    #[test]
    fn only_the_code_is_left_and_a_comment_can_neither_pass_nor_fail_a_check() {
        let text = "//! About `watch(a)`.\nfn f() {\n    // call watch(b) here one day\n    let at = \"https://example.com\"; // watch(c)\n    /// watch(d)\n    watch(e);\n}\n";
        let code = code(text);
        assert_eq!(code, "\nfn f() {\n    \n    let at = \"https://example.com\"; \n    \n    watch(e);\n}\n");
        for said_only in ["watch(a)", "watch(b)", "watch(c)", "watch(d)"] {
            assert!(text.contains(said_only) && !code.contains(said_only), "{said_only}");
        }
        assert!(code.contains("watch(e)") && code.contains("https://example.com"));
    }
}
