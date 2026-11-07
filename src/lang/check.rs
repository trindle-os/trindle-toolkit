//! The missing-words check. A language is a folder (`share/lang/<code>/`)

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use super::parse::{parse, Parsed};

pub struct Domain {
    pub name: String,
    pub lang_dir: PathBuf,
    pub sources: Vec<PathBuf>,
    pub skip: Vec<String>,
}

#[derive(Default, Debug)]
pub struct Report {
    pub problems: Vec<String>,
    pub notes: Vec<String>,
    pub english_keys: usize,
    pub coverage: BTreeMap<String, (usize, usize)>,
    pub lacking: BTreeMap<String, Vec<String>>,
}

impl Report {
    pub fn text(&self) -> String {
        let mut out = String::new();
        for p in &self.problems {
            let _ = writeln!(out, "PROBLEM {p}");
        }
        for n in &self.notes {
            let _ = writeln!(out, "note    {n}");
        }
        let _ = writeln!(out, "{} English messages, {} problems, {} notes", self.english_keys, self.problems.len(), self.notes.len());
        out
    }

    pub fn missing_text(&self, only: Option<&str>) -> String {
        let mut out = String::new();
        for (lang, (done, total)) in &self.coverage {
            let _ = writeln!(out, "{lang}: {done} of {total} words ({}%)", done * 100 / (*total).max(1));
        }
        for (lang, keys) in self.lacking.iter().filter(|(l, _)| only.is_none_or(|o| o == l.as_str())) {
            let _ = writeln!(out, "\n{lang} lacks:");
            for k in keys {
                let _ = writeln!(out, "  {k}");
            }
        }
        out
    }
}

struct Use {
    key: String,
    args: Option<BTreeSet<String>>,
    at: String,
}

fn rs_files(dir: &Path, into: &mut Vec<PathBuf>) {
    let Ok(read) = std::fs::read_dir(dir) else { return };
    let mut entries: Vec<_> = read.flatten().map(|e| e.path()).collect();
    entries.sort();
    for p in entries {
        let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if p.is_dir() {
            if name != "target" && !name.starts_with('.') {
                rs_files(&p, into);
            }
        } else if name.ends_with(".rs") {
            into.push(p);
        }
    }
}

fn boundary(before: Option<char>) -> bool {
    before.is_none_or(|c| !(c.is_alphanumeric() || c == '_'))
}

fn uses_in(text: &str, file: &str) -> Vec<Use> {
    let code: String = text.lines().map(|l| if l.trim_start().starts_with("//") { "" } else { l }).collect::<Vec<_>>().join("\n");
    let mut out = Vec::new();
    for (pat, macro_call) in [("t!(", true), ("lang::t(", false), ("t_with(", false)] {
        let mut from = 0;
        while let Some(found) = code[from..].find(pat) {
            let start = from + found;
            from = start + pat.len();
            if !boundary(code[..start].chars().next_back()) {
                continue;
            }
            let Some(rest) = code[from..].trim_start().strip_prefix('"') else { continue };
            let Some(end) = rest.find('"') else { continue };
            let key = &rest[..end];
            if key.is_empty() || !key.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
                continue;
            }
            let args = macro_call.then(|| rest[end + 1..].trim_start().strip_prefix(',').map(macro_args).unwrap_or_default());
            let line = code[..start].matches('\n').count() + 1;
            out.push(Use { key: key.to_string(), args, at: format!("{file}:{line}") });
        }
    }
    out
}

fn macro_args(list: &str) -> BTreeSet<String> {
    let (mut depth, mut quoted, mut escaped) = (0i32, false, false);
    let (mut seg, mut segs) = (String::new(), Vec::new());
    for c in list.chars() {
        if quoted {
            quoted = c != '"' || escaped;
            escaped = c == '\\' && !escaped;
        } else {
            match c {
                '"' => quoted = true,
                '(' | '[' | '{' => depth += 1,
                ')' | ']' | '}' => {
                    depth -= 1;
                    if depth < 0 {
                        break;
                    }
                }
                ',' if depth == 0 => {
                    segs.push(std::mem::take(&mut seg));
                    continue;
                }
                _ => {}
            }
        }
        seg.push(c);
    }
    segs.push(seg);
    let mut names = BTreeSet::new();
    for s in segs {
        let s = s.trim();
        let name: String = s.chars().take_while(|c| c.is_ascii_alphanumeric() || *c == '_').collect();
        let tail = s[name.len()..].trim_start();
        if !name.is_empty() && tail.starts_with('=') && !tail.starts_with("==") {
            names.insert(name);
        }
    }
    names
}

struct Language {
    parsed: Parsed,
    file_of: BTreeMap<String, String>,
    faults: Vec<String>,
}

fn read_dir_ftl(dir: &Path, problems: &mut Vec<String>) -> BTreeMap<String, Language> {
    let mut all = BTreeMap::new();
    let Ok(read) = std::fs::read_dir(dir) else { return all };
    let mut entries: Vec<PathBuf> = read.flatten().map(|e| e.path()).collect();
    entries.sort();
    for p in entries {
        let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
        if p.is_dir() {
            let mut language = Language { parsed: Parsed::default(), file_of: BTreeMap::new(), faults: Vec::new() };
            for (file, text) in super::folder_files(&p) {
                let one = parse(&text);
                for f in &one.faults {
                    language.faults.push(format!("{name}/{file} line {}: {}", f.line, f.why));
                }
                for key in one.order {
                    if let Some(first) = language.file_of.get(&key) {
                        language.faults.push(format!("{key} is in both {name}/{first} and {name}/{file}"));
                        continue;
                    }
                    language.file_of.insert(key.clone(), file.clone());
                    if let Some(m) = one.messages.get(&key) {
                        language.parsed.messages.insert(key.clone(), m.clone());
                    }
                    language.parsed.order.push(key);
                }
            }
            all.insert(name, language);
        } else if name.ends_with(".ftl") {
            problems.push(format!("{} is not in a language folder (use <code>/{name})", p.display()));
        }
    }
    all
}

type English = BTreeMap<String, (BTreeSet<String>, String)>;

pub fn check(domains: &[Domain]) -> Report {
    let mut report = Report::default();
    let mut english: English = BTreeMap::new();
    let mut named: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut uses: Vec<Use> = Vec::new();

    for d in domains {
        let files = read_dir_ftl(&d.lang_dir, &mut report.problems);
        let Some(en_language) = files.get("en") else {
            report.problems.push(format!("{}: {} has no en folder of .ftl files", d.name, d.lang_dir.display()));
            continue;
        };
        check_english(d, en_language, &mut report, &mut english, &mut named);
        for (lang, language) in &files {
            if lang != "en" {
                check_translation(d, lang, language, en_language, &mut report);
            }
        }
        collect_uses(d, &mut uses);
    }
    report.english_keys = english.len();
    for (key, refs) in &named {
        for r in refs.iter().filter(|r| !english.contains_key(*r)) {
            report.problems.push(format!("{key} names {r}, which no English file has"));
        }
    }
    report.problems.extend(cycles(&named));
    check_uses(&uses, &english, &mut report);
    report
}

fn check_english(d: &Domain, en_language: &Language, report: &mut Report, english: &mut English, named: &mut BTreeMap<String, BTreeSet<String>>) {
    let en = &en_language.parsed;
    for f in &en_language.faults {
        report.problems.push(format!("{}: {f}", d.name));
    }
    for key in &en.order {
        if let Some(why) = en.messages[key].stray_plural("en") {
            report.problems.push(format!("{}: en/{} {key} is wrong: {why}", d.name, en_language.file_of[key]));
        }
        let vars = en.messages[key].variables();
        named.insert(key.clone(), en.messages[key].references());
        if let Some((_, other)) = english.insert(key.clone(), (vars, format!("{}/{}", d.name, en_language.file_of[key]))) {
            report.problems.push(format!("{key} is in the English of both {other} and {}/{}", d.name, en_language.file_of[key]));
        }
    }
}

fn check_translation(d: &Domain, lang: &str, language: &Language, en_language: &Language, report: &mut Report) {
    let en = &en_language.parsed;
    let parsed = &language.parsed;
    for f in &language.faults {
        report.problems.push(format!("{}: {f}", d.name));
    }
    let missing: Vec<&String> = en.order.iter().filter(|k| !parsed.messages.contains_key(*k)).collect();
    let (done, total) = report.coverage.entry(lang.to_string()).or_default();
    *done += en.order.len() - missing.len();
    *total += en.order.len();
    report.lacking.entry(lang.to_string()).or_default().extend(missing.iter().map(|k| format!("{}: {k}", en_language.file_of[*k])));
    if !missing.is_empty() {
        let shown: Vec<&str> = missing.iter().take(12).map(|k| k.as_str()).collect();
        let more = if missing.len() > 12 { format!(", and {} more", missing.len() - 12) } else { String::new() };
        report.notes.push(format!("{}: {lang} lacks {} of {} keys: {}{more}", d.name, missing.len(), en.order.len(), shown.join(", ")));
    }
    for key in &parsed.order {
        match en.messages.get(key) {
            None => report.problems.push(format!("{}: {lang}/{} has {key}, which English does not", d.name, language.file_of[key])),
            Some(english_message) => {
                let (theirs, ours) = (parsed.messages[key].variables(), english_message.variables());
                if theirs != ours {
                    let list = |s: &BTreeSet<String>| s.iter().map(|v| format!("${v}")).collect::<Vec<_>>().join(" ");
                    report.problems.push(format!("{}: {lang}/{} {key} uses [{}] but English uses [{}]", d.name, language.file_of[key], list(&theirs), list(&ours)));
                } else if let Some(why) = parsed.messages[key].mismatch(english_message, lang) {
                    report.problems.push(format!("{}: {lang}/{} {key} is not shown: {why}", d.name, language.file_of[key]));
                } else if let Some(why) = parsed.messages[key].stray_plural(lang) {
                    report.problems.push(format!("{}: {lang}/{} {key} is wrong: {why}", d.name, language.file_of[key]));
                }
            }
        }
    }
}

fn collect_uses(d: &Domain, uses: &mut Vec<Use>) {
    let mut found = Vec::new();
    for s in &d.sources {
        if s.is_file() {
            found.push(s.clone());
        } else {
            rs_files(s, &mut found);
        }
    }
    for path in found {
        let shown = path.display().to_string();
        if d.skip.iter().any(|s| shown.ends_with(s.as_str())) {
            continue;
        }
        if let Ok(text) = std::fs::read_to_string(&path) {
            uses.extend(uses_in(&text, &shown));
        }
    }
}

fn check_uses(uses: &[Use], english: &English, report: &mut Report) {
    let asked: BTreeSet<&str> = uses.iter().map(|u| u.key.as_str()).collect();
    for u in uses {
        match english.get(&u.key) {
            None => report.problems.push(format!("{} asks for {}, which no English file has", u.at, u.key)),
            Some((vars, _)) => {
                if let Some(given) = &u.args {
                    if given != vars {
                        let list = |s: &BTreeSet<String>| s.iter().cloned().collect::<Vec<_>>().join(" ");
                        report.problems.push(format!("{} gives [{}] to {}, which takes [{}]", u.at, list(given), u.key, list(vars)));
                    }
                }
            }
        }
    }
    for (key, (_, domain)) in english {
        if !asked.contains(key.as_str()) {
            report.problems.push(format!("{domain}: English has {key}, which no code asks for"));
        }
    }
}

fn cycles(named: &BTreeMap<String, BTreeSet<String>>) -> Vec<String> {
    fn walk<'a>(key: &'a str, named: &'a BTreeMap<String, BTreeSet<String>>, path: &mut Vec<&'a str>, done: &mut BTreeSet<&'a str>, out: &mut Vec<String>) {
        if let Some(at) = path.iter().position(|p| *p == key) {
            out.push(format!("{} names itself again: {}", key, [&path[at..], &[key]].concat().join(" names ")));
            return;
        }
        if !done.insert(key) {
            return;
        }
        path.push(key);
        for next in named.get(key).into_iter().flatten().filter(|n| named.contains_key(*n)) {
            walk(next, named, path, done, out);
        }
        path.pop();
    }
    let (mut out, mut done) = (Vec::new(), BTreeSet::new());
    for key in named.keys() {
        walk(key, named, &mut Vec::new(), &mut done, &mut out);
    }
    out
}

pub fn domains_at(root: &Path) -> Vec<Domain> {
    let beside = root.join("..").join("trindle-toolkit");
    #[cfg(not(trindle_shipped))]
    let fetched = Path::new(env!("CARGO_MANIFEST_DIR"));
    #[cfg(trindle_shipped)]
    let fetched = Path::new("");
    let kit = if beside.join("share/lang/en").is_dir() && beside.join("src/lang/check.rs").is_file() {
        beside
    } else if root.join("src/lang/check.rs").is_file() {
        root.to_path_buf()
    } else if fetched.join("src/lang/check.rs").is_file() {
        fetched.to_path_buf()
    } else {
        PathBuf::new()
    };
    let mut all = Vec::new();
    if !kit.as_os_str().is_empty() {
        all.push(Domain {
            name: "toolkit".into(),
            lang_dir: kit.join("share/lang"),
            sources: vec![kit.join("src"), kit.join("examples")],
            skip: vec!["lang/mod.rs".into(), "lang/check.rs".into(), "lang/tests.rs".into()],
        });
    }
    let same = |a: &Path, b: &Path| a.canonicalize().ok().is_some_and(|a| b.canonicalize().ok() == Some(a));
    if !same(&kit, root) && root.join("share/lang/en").is_dir() {
        all.push(Domain { name: "trindle".into(), lang_dir: root.join("share/lang"), sources: vec![root.join("src")], skip: Vec::new() });
    }
    all
}
