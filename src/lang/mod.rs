//! Words in the person's language.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

pub mod check;
pub mod format;
pub mod numerals;
mod parse;
pub mod plural;
pub mod script;

pub use parse::{parse, placeholders, raw_messages, Arg, Fault, Gender, Message, Parsed};

pub const KIT_ENGLISH: &[(&str, &str)] = include!(concat!(env!("OUT_DIR"), "/english.rs"));

const FILE_MOST: u64 = 1 << 20;

#[macro_export]
macro_rules! t {
    ($key:literal) => {
        $crate::lang::t($key)
    };
    ($key:literal, $($name:ident = $value:expr),+ $(,)?) => {
        $crate::lang::t_with($key, &[$((stringify!($name), $crate::lang::Arg::from($value))),+])
    };
}

#[derive(Clone, Debug, Default)]
pub struct Bundle {
    pub lang: String,
    messages: HashMap<String, Message>,
}

impl Bundle {
    pub fn from_text(lang: &str, text: &str) -> Bundle {
        let parsed = parse(text);
        for f in &parsed.faults {
            note_dropped(&format!("{lang}: line {}: {}", f.line, f.why));
        }
        Bundle { lang: lang.to_string(), messages: parsed.messages }
    }

    pub fn from_files(lang: &str, files: &[(&str, &str)]) -> Bundle {
        let mut bundle = Bundle { lang: lang.to_string(), ..Bundle::default() };
        for (_, text) in files {
            bundle.fill_from(Bundle::from_text(lang, text));
        }
        bundle
    }

    pub fn fill_from(&mut self, other: Bundle) {
        for (key, message) in other.messages {
            self.messages.entry(key).or_insert(message);
        }
    }

    pub fn get(&self, key: &str) -> Option<&Message> {
        self.messages.get(key)
    }

    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    pub fn len(&self) -> usize {
        self.messages.len()
    }

    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.messages.keys().map(String::as_str)
    }
}

#[derive(Debug, Default)]
pub struct Catalogue {
    chosen: Bundle,
    english: Bundle,
}

impl parse::Lookup for Catalogue {
    fn find(&self, key: &str) -> Option<(&Message, &str)> {
        self.find(key)
    }
}

impl Catalogue {
    pub fn new(mut chosen: Bundle, mut english: Bundle) -> Catalogue {
        let lang = chosen.lang.clone();
        chosen.messages.retain(|key, m| match english.get(key).and_then(|e| m.mismatch(e, &lang)) {
            Some(why) => {
                note_dropped(&format!("{lang}: {key}: {why}; English shows"));
                false
            }
            None => true,
        });
        chosen.messages.values_mut().chain(english.messages.values_mut()).for_each(Message::keep);
        Catalogue { chosen, english }
    }

    fn find(&self, key: &str) -> Option<(&Message, &str)> {
        if let Some(m) = self.chosen.get(key) {
            return Some((m, &self.chosen.lang));
        }
        self.english.get(key).map(|m| (m, "en"))
    }

    pub fn plain(&self, key: &str) -> &'static str {
        match self.find(key) {
            Some((m, from)) => match m.plain {
                Some(text) => text,
                None if from != "en" => self.english.get(key).and_then(|e| e.plain).unwrap_or(""),
                None => "",
            },
            None => {
                note_unknown(key);
                ""
            }
        }
    }

    pub fn with(&self, key: &str, args: &[(&str, Arg)]) -> String {
        let Some((m, from)) = self.find(key) else {
            note_unknown(key);
            return String::new();
        };
        if let Some(text) = m.plain {
            return text.to_string();
        }
        let mut out = String::new();
        parse::render(&m.pieces, from, args, self, &[key], &mut out);
        out
    }
}

static DROPPED: Mutex<Vec<String>> = Mutex::new(Vec::new());
const DROPPED_MOST: usize = 1024;

#[derive(Debug, PartialEq)]
enum Admit {
    Listed,
    Known,
    FullNow,
    Full,
}

fn admit(all: &mut Vec<String>, what: &str, most: usize) -> Admit {
    if all.iter().any(|w| w == what) {
        Admit::Known
    } else if all.len() < most {
        all.push(what.to_string());
        Admit::Listed
    } else if all.len() == most {
        all.push("(more left out, not listed)".into());
        Admit::FullNow
    } else {
        Admit::Full
    }
}

fn note_dropped(what: &str) {
    let mut all = DROPPED.lock().unwrap_or_else(|e| e.into_inner());
    match admit(&mut all, what, DROPPED_MOST) {
        Admit::Listed => eprintln!("trindle: translation left out: {what}"),
        Admit::FullNow => eprintln!("trindle: more translations left out, not listed (the list is full)"),
        Admit::Known | Admit::Full => {}
    }
}

pub fn dropped_translations() -> Vec<String> {
    DROPPED.lock().unwrap_or_else(|e| e.into_inner()).clone()
}

static UNKNOWN: Mutex<Vec<String>> = Mutex::new(Vec::new());

fn note_unknown(key: &str) {
    let mut all = UNKNOWN.lock().unwrap_or_else(|e| e.into_inner());
    if all.len() < 256 && !all.iter().any(|k| k == key) {
        all.push(key.to_string());
    }
}

pub fn unknown_keys() -> Vec<String> {
    UNKNOWN.lock().unwrap_or_else(|e| e.into_inner()).clone()
}

static CATALOGUE: OnceLock<Catalogue> = OnceLock::new();

#[cfg(any(test, feature = "testing"))]
thread_local! {
    static TRIAL: std::cell::RefCell<Option<std::rc::Rc<Catalogue>>> = const { std::cell::RefCell::new(None) };
}

#[cfg(any(test, feature = "testing"))]
pub fn with_catalogue<R>(catalogue: Catalogue, during: impl FnOnce() -> R) -> R {
    struct Restore(Option<std::rc::Rc<Catalogue>>);
    impl Drop for Restore {
        fn drop(&mut self) {
            TRIAL.with(|t| *t.borrow_mut() = self.0.take());
        }
    }
    let _restore = Restore(TRIAL.with(|t| t.replace(Some(std::rc::Rc::new(catalogue)))));
    during()
}

pub fn english_files() -> Vec<(&'static str, &'static str)> {
    KIT_ENGLISH.iter().chain(crate::host::host().words().iter()).copied().collect()
}

pub fn english() -> Bundle {
    let mut bundle = Bundle::from_files("en", KIT_ENGLISH);
    bundle.fill_from(Bundle::from_files("en", crate::host::host().words()));
    bundle
}

fn catalogue() -> &'static Catalogue {
    CATALOGUE.get_or_init(|| {
        Catalogue::new(load(&choice(), &folders()), english())
    })
}

#[inline]
fn with_current<R>(ask: impl FnOnce(&Catalogue) -> R) -> R {
    #[cfg(any(test, feature = "testing"))]
    {
        let trial = TRIAL.with(|t| t.borrow().clone());
        if let Some(trial) = trial {
            return ask(&trial);
        }
    }
    ask(catalogue())
}

pub fn chosen_tag() -> String {
    with_current(|c| c.chosen.lang.clone())
}

pub fn t(key: &str) -> &'static str {
    with_current(|c| c.plain(key))
}

pub fn t_with(key: &str, args: &[(&str, Arg)]) -> String {
    with_current(|c| c.with(key, args))
}

pub fn choice() -> Vec<String> {
    let env = |k: &str| std::env::var(k).ok();
    choice_from(crate::host::host().language(), env("LC_ALL"), env("LC_MESSAGES"), env("LANG"))
}

pub fn choice_from(own: Option<String>, all: Option<String>, messages: Option<String>, lang: Option<String>) -> Vec<String> {
    let own = own.filter(|v| v.trim() != "auto");
    let named = [own, all, messages, lang].into_iter().flatten().find(|v| !v.trim().is_empty());
    tags_of(&named.unwrap_or_default())
}

pub fn load(tags: &[String], dirs: &[PathBuf]) -> Bundle {
    let mut chosen = Bundle { lang: tags.first().cloned().unwrap_or_else(|| "en".into()), ..Bundle::default() };
    for tag in tags.iter().filter(|t| !t.contains(['/', '\\']) && !t.contains("..")) {
        for dir in dirs {
            for (_, text) in folder_files(&dir.join(tag)) {
                chosen.fill_from(Bundle::from_text(&chosen.lang, &text));
            }
        }
    }
    chosen
}

fn is_region(part: &str) -> bool {
    part.len() == 2 && part.bytes().all(|b| b.is_ascii_alphabetic()) || part.len() == 3 && part.bytes().all(|b| b.is_ascii_digit())
}

pub fn tags_of(name: &str) -> Vec<String> {
    let base = name.split(['.', '@']).next().unwrap_or("").trim().replace('_', "-");
    if base.is_empty() || base == "C" || base == "POSIX" {
        return Vec::new();
    }
    let mut parts = base.split('-');
    let primary = parts.next().unwrap_or("").to_ascii_lowercase();
    if primary.is_empty() || !primary.chars().all(|c| c.is_ascii_alphabetic()) {
        return Vec::new();
    }
    let (mut script, mut region, mut variants) = (None, None, Vec::new());
    for (n, part) in parts.enumerate() {
        if part.is_empty() || !part.chars().all(|c| c.is_ascii_alphanumeric()) {
            return Vec::new();
        }
        if n == 0 && part.len() == 4 && part.chars().all(|c| c.is_ascii_alphabetic()) {
            let lower = part.to_ascii_lowercase();
            script = Some(format!("{}{}", lower[..1].to_ascii_uppercase(), &lower[1..]));
        } else if region.is_none() && variants.is_empty() && is_region(part) {
            region = Some(part.to_ascii_uppercase());
        } else {
            variants.push(part.to_ascii_lowercase());
        }
    }
    let mut tags = Vec::new();
    let mut stem = primary.clone();
    let mut steps = Vec::new();
    if let Some(s) = &script {
        stem = format!("{stem}-{s}");
        steps.push(stem.clone());
    }
    if let Some(r) = &region {
        stem = format!("{stem}-{r}");
        steps.push(stem.clone());
    }
    for v in &variants {
        stem = format!("{stem}-{v}");
        steps.push(stem.clone());
    }
    steps.reverse();
    tags.extend(steps);
    match (primary.as_str(), region.as_deref(), script.is_some()) {
        ("zh", Some("CN" | "SG"), false) => tags.push("zh-Hans".into()),
        ("zh", Some("TW" | "HK" | "MO"), false) => tags.push("zh-Hant".into()),
        _ => {}
    }
    tags.push(primary);
    tags
}

pub fn folders() -> Vec<PathBuf> {
    let mut dirs = vec![crate::common::files::config_home().join("trindle").join("lang")];
    let data = std::env::var("XDG_DATA_DIRS").ok().filter(|v| !v.is_empty()).unwrap_or_else(|| "/usr/local/share:/usr/share".into());
    for d in data.split(':').filter(|d| d.starts_with('/')) {
        dirs.push(Path::new(d).join("trindle").join("lang"));
        dirs.push(Path::new(d).join("trindle-toolkit").join("lang"));
    }
    dirs
}

pub fn folder_files(dir: &Path) -> Vec<(String, String)> {
    let Ok(read) = std::fs::read_dir(dir) else { return Vec::new() };
    let mut paths: Vec<PathBuf> = read.flatten().map(|e| e.path()).filter(|p| p.extension().is_some_and(|x| x == "ftl")).collect();
    paths.sort();
    paths
        .into_iter()
        .filter_map(|p| {
            let name = p.file_name()?.to_str()?.to_string();
            read_file(&p).map(|text| (name, text))
        })
        .collect()
}

pub fn read_file(path: &Path) -> Option<String> {
    use std::io::Read as _;
    let file = std::fs::File::open(path).ok()?;
    if !file.metadata().ok()?.is_file() {
        return None;
    }
    let mut text = String::new();
    file.take(FILE_MOST + 1).read_to_string(&mut text).ok()?;
    (text.len() as u64 <= FILE_MOST).then_some(text)
}

#[cfg(test)]
mod tests;
