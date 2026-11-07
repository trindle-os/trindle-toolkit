//! The rule for taking anything of the person's away, and the proofs it asks for.

use std::ffi::{CString, OsStr, OsString};
use std::fmt::Write as _;
use std::fs::{self, File};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::{FileTypeExt, MetadataExt};
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

use crate::common::files::{through, Which};
#[cfg(feature = "ui")]
use crate::dnd;
use crate::sys::disk;
use crate::t;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Seal {
    which: Which,
    len: u64,
    changed: (i64, i64),
    touched: (i64, i64),
}

impl Seal {
    pub fn of(meta: &fs::Metadata) -> Seal {
        Seal { which: Which::of(meta), len: meta.len(), changed: (meta.mtime(), meta.mtime_nsec()), touched: (meta.ctime(), meta.ctime_nsec()) }
    }

    fn renamed(&self, now: &fs::Metadata) -> bool {
        let now = Seal::of(now);
        (now.which, now.len, now.changed) == (self.which, self.len, self.changed)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FarSeal {
    pub size: u64,
    pub modified: Option<i64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum First {
    Here(Seal),
    Far(FarSeal),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Copied {
    first: First,
}

impl Copied {
    pub fn of_file(first: &fs::Metadata, after: &fs::Metadata, read: u64, lies: Option<u64>) -> Option<Copied> {
        let seal = Seal::of(first);
        (first.is_file() && Seal::of(after) == seal && read == seal.len && lies == Some(read)).then_some(Copied { first: First::Here(seal) })
    }

    pub fn of_link(first: &fs::Metadata, leads: &Path, copy: &Path) -> Option<Copied> {
        let same = first.file_type().is_symlink() && fs::read_link(copy).is_ok_and(|to| to == leads);
        same.then_some(Copied { first: First::Here(Seal::of(first)) })
    }

    pub fn of_far(first: FarSeal, now: Option<FarSeal>, sent: u64, lies: Option<u64>) -> Option<Copied> {
        (now == Some(first) && sent == first.size && lies == Some(sent)).then_some(Copied { first: First::Far(first) })
    }

    pub fn far(&self) -> Option<FarSeal> {
        match self.first {
            First::Far(seal) => Some(seal),
            First::Here(_) => None,
        }
    }
}

const GOING: &str = ".trindle-moving";

pub fn tag() -> String {
    let mut bytes = [0u8; 8];
    disk::random(&mut bytes);
    bytes.iter().fold(String::with_capacity(bytes.len() * 2), |mut hex, b| {
        let _ = write!(hex, "{b:02x}");
        hex
    })
}

fn c_name(name: &OsStr) -> Option<CString> {
    CString::new(name.as_bytes()).ok()
}

pub fn take_away(dir: &File, name: &OsStr, proof: &Copied) -> bool {
    let First::Here(seal) = proof.first else { return false };
    let there = through(dir).join(name);
    match fs::symlink_metadata(&there) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return true,
        Ok(now) if Seal::of(&now) == seal => {}
        _ => return false,
    }
    #[cfg(any(test, feature = "testing"))]
    if let Ok(folder) = fs::read_link(through(dir)) {
        super::work::faults::meanwhile(&folder.join(name));
    }
    let own = match set_aside(dir, name) {
        Aside::Under(own) => own,
        Aside::Gone => return true,
        Aside::Stays => return false,
    };
    if fs::symlink_metadata(through(dir).join(&own)).is_ok_and(|now| seal.renamed(&now)) && remove_in(dir, &own, false).is_ok() {
        return true;
    }
    give_back(dir, &own, name);
    false
}

enum Aside {
    Under(OsString),
    Gone,
    Stays,
}

fn set_aside(dir: &File, name: &OsStr) -> Aside {
    let Some(from) = c_name(name) else { return Aside::Stays };
    let tag = tag();
    let mut long = name.to_os_string();
    long.push(format!(".{tag}{GOING}"));
    let short = OsString::from(format!("{tag}{GOING}"));
    for own in [long, short] {
        let Some(to) = c_name(&own) else { continue };
        if fs::symlink_metadata(through(dir).join(&own)).is_ok() {
            continue;
        }
        match disk::rename_in(dir, &from, &to) {
            Ok(()) => return Aside::Under(own),
            Err(e) if e.raw_os_error() == Some(libc::ENAMETOOLONG) => continue,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Aside::Gone,
            Err(_) => return Aside::Stays,
        }
    }
    Aside::Stays
}

fn give_back(dir: &File, own: &OsStr, name: &OsStr) {
    let at = through(dir);
    named_back(&at.join(own), &at, name, false);
}

pub fn named_back(aside: &Path, dir: &Path, name: &OsStr, folder: bool) -> PathBuf {
    use crate::common::files::{free_in, rename_new};
    let back = dir.join(name);
    if rename_new(aside, &back).is_ok() {
        return back;
    }
    for _ in 0..4 {
        let free = dir.join(free_in(name, folder, |n| fs::symlink_metadata(dir.join(n)).is_ok()));
        if rename_new(aside, &free).is_ok() {
            return free;
        }
    }
    aside.to_path_buf()
}

const ASKED_HELD: usize = 256;

#[derive(Clone, Debug)]
pub struct Asked {
    path: PathBuf,
    which: Which,
    _held: Option<Arc<File>>,
}

pub fn asked(paths: &[PathBuf]) -> Vec<Asked> {
    let look = |(n, path): (usize, &PathBuf)| {
        let (held, meta) = crate::common::files::open_same(path, None).ok()?;
        Some(Asked { path: path.clone(), which: Which::of(&meta), _held: (n < ASKED_HELD).then(|| Arc::new(held)) })
    };
    paths.iter().enumerate().filter_map(look).collect()
}

fn remove_in(dir: &File, name: &OsStr, folder: bool) -> std::io::Result<()> {
    let name = c_name(name).ok_or_else(|| std::io::Error::from(std::io::ErrorKind::InvalidInput))?;
    disk::remove_in(dir, &name, folder)
}

fn empty(dir: &File) -> std::io::Result<()> {
    let names: Vec<OsString> = fs::read_dir(through(dir))?.map(|e| e.map(|e| e.file_name())).collect::<Result<_, _>>()?;
    for name in names {
        let (inner, meta) = crate::common::files::open_in(dir, &name, None)?;
        if meta.is_dir() {
            empty(&inner)?;
        }
        remove_in(dir, &name, meta.is_dir())?;
    }
    Ok(())
}

impl Asked {
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn which(&self) -> Which {
        self.which
    }

    pub fn delete(&self) -> std::io::Result<()> {
        use crate::common::files::{changed, open_in};
        use std::os::unix::fs::OpenOptionsExt;
        let (Some(up), Some(name)) = (self.path.parent(), self.path.file_name()) else { return Err(changed()) };
        let dir = fs::OpenOptions::new().read(true).custom_flags(libc::O_PATH | libc::O_DIRECTORY).open(up)?;
        let (named, meta) = open_in(&dir, name, Some(self.which))?;
        #[cfg(any(test, feature = "testing"))]
        super::work::faults::meanwhile(&self.path);
        if meta.is_dir() {
            empty(&named)?;
            return remove_in(&dir, name, true);
        }
        let own = match set_aside(&dir, name) {
            Aside::Under(own) => own,
            Aside::Gone => return Ok(()),
            Aside::Stays => return Err(std::io::Error::other(t!("kit-proof-not-taken-away"))),
        };
        let same = fs::symlink_metadata(through(&dir).join(&own)).is_ok_and(|now| Which::of(&now) == self.which);
        let gone = if same { remove_in(&dir, &own, false) } else { Err(changed()) };
        if gone.is_err() {
            give_back(&dir, &own, name);
        }
        gone
    }
}

pub fn unmake(made: &File, at: &Path) {
    let ours = made.metadata().ok().map(|m| Which::of(&m));
    if ours.is_some() && fs::symlink_metadata(at).ok().map(|m| Which::of(&m)) == ours {
        let _ = fs::remove_file(at);
    }
}

pub fn not_as_picked() -> &'static str {
    t!("kit-proof-not-as-picked")
}
pub fn not_looked_through() -> &'static str {
    t!("kit-proof-not-looked-through")
}

#[derive(Clone, Debug)]
struct Within {
    at: PathBuf,
    seal: Seal,
    folder: bool,
}

impl Within {
    fn is(&self, now: &Within) -> bool {
        self.at == now.at && self.folder == now.folder && if self.folder { self.seal.which == now.seal.which } else { self.seal == now.seal }
    }
}

fn can_read(path: &Path) -> bool {
    c_name(path.as_os_str()).is_some_and(|c| disk::can_read(&c))
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Trouble {
    pub at: String,
    pub system: bool,
    pub why: String,
}

impl Trouble {
    pub fn line(&self) -> String {
        format!("{}: {}", self.at, self.why)
    }
}

const TROUBLES_NAMED: usize = 200;

#[derive(Clone, Debug)]
enum Look {
    Whole(Vec<Within>),
    Refused(Vec<Trouble>, Counts),
    Unsure,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Counts {
    pub unreadable: usize,
    pub system: usize,
    pub other: usize,
}

impl Counts {
    pub fn add(&mut self, other: Counts) {
        self.unreadable += other.unreadable;
        self.system += other.system;
        self.other += other.other;
    }

    pub fn words(&self) -> Option<String> {
        let one = |n: usize, f: fn(usize, String) -> String| (n > 0).then(|| f(n, crate::lang::format::number(n as f64, 0)));
        let mut parts: Vec<String> = [
            one(self.unreadable, |n, count| t!("kit-proof-count-unreadable", n = n, count = count)),
            one(self.system, |n, count| t!("kit-proof-count-system", n = n, count = count)),
            one(self.other, |n, count| t!("kit-proof-count-other", n = n, count = count)),
        ]
        .into_iter()
        .flatten()
        .collect();
        let last = parts.pop()?;
        if parts.is_empty() { Some(last) } else { Some(t!("kit-proof-count-both", first = parts.join(", "), second = last)) }
    }

    pub fn said(&self) -> Option<String> {
        self.words().map(|w| t!("kit-proof-not-moved-said", why = w))
    }

    pub fn reason(&self) -> String {
        t!("kit-proof-not-moved-reason", why = self.words().unwrap_or_else(|| t!("kit-proof-count-some").to_string()))
    }
}

fn looked_through(top: &Path) -> Look {
    let (mut out, mut bad, mut counts) = (Vec::new(), Vec::new(), Counts::default());
    let name = top.file_name().map(PathBuf::from).unwrap_or_default();
    let mut todo = vec![PathBuf::new()];
    while let Some(dir) = todo.pop() {
        let read = match fs::read_dir(top.join(&dir)) {
            Ok(read) => read,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Look::Unsure,
            Err(e) => {
                counts.unreadable += 1;
                if bad.len() < TROUBLES_NAMED {
                    bad.push(Trouble { at: name.join(&dir).to_string_lossy().into_owned(), system: false, why: why_not_read(&e) });
                }
                continue;
            }
        };
        for entry in read {
            let Ok(entry) = entry else { return Look::Unsure };
            let Ok(meta) = entry.metadata() else { return Look::Unsure };
            let (at, kind) = (dir.join(entry.file_name()), meta.file_type());
            let mut trouble = |system: bool, why: &str| {
                if system { counts.system += 1 } else { counts.unreadable += 1 }
                if bad.len() < TROUBLES_NAMED {
                    bad.push(Trouble { at: name.join(&at).to_string_lossy().into_owned(), system, why: why.into() });
                }
            };
            if !(kind.is_symlink() || kind.is_file() || kind.is_dir()) {
                trouble(true, if kind.is_fifo() { t!("kit-proof-pipe") } else if kind.is_socket() { t!("kit-proof-socket") } else { t!("kit-proof-device") });
            } else if !kind.is_symlink() && !can_read(&entry.path()) {
                trouble(false, t!("kit-proof-no-permission"));
            }
            if kind.is_dir() {
                todo.push(at.clone());
            }
            out.push(Within { at, seal: Seal::of(&meta), folder: kind.is_dir() });
        }
    }
    if counts != Counts::default() {
        return Look::Refused(bad, counts);
    }
    out.sort_by(|a, b| a.at.cmp(&b.at));
    Look::Whole(out)
}

fn why_not_read(e: &std::io::Error) -> String {
    if e.kind() == std::io::ErrorKind::PermissionDenied { t!("kit-proof-no-permission").into() } else { t!("kit-proof-unreadable").into() }
}

#[derive(Clone, Debug)]
pub struct Picked {
    seal: Seal,
    inside: Option<Arc<OnceLock<Look>>>,
}

pub fn picked_up(path: &Path, aside: bool) -> Option<Picked> {
    let meta = fs::symlink_metadata(path).ok()?;
    let seal = Seal::of(&meta);
    if !meta.is_dir() {
        return Some(Picked { seal, inside: None });
    }
    let inside = Arc::new(OnceLock::new());
    let (slot, top) = (inside.clone(), path.to_path_buf());
    let look = move || drop(slot.set(looked_through(&top)));
    if !aside || std::thread::Builder::new().name("file-picked-up".into()).spawn(look.clone()).is_err() {
        look();
    }
    Some(Picked { seal, inside: Some(inside) })
}

#[cfg_attr(not(feature = "ui"), allow(dead_code))]
enum Now {
    Same,
    Gone,
    Changed,
    Unsure,
    Refused(Vec<Trouble>, Counts),
}

impl Picked {
    fn now(&self, path: &Path) -> Now {
        let now = match fs::symlink_metadata(path) {
            Ok(now) => now,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Now::Gone,
            Err(_) => return Now::Changed,
        };
        let Some(inside) = &self.inside else {
            return if !now.is_dir() && self.seal == Seal::of(&now) { Now::Same } else { Now::Changed };
        };
        if !now.is_dir() || self.seal.which != Which::of(&now) {
            return Now::Changed;
        }
        let was = match inside.get() {
            None | Some(Look::Unsure) => return Now::Unsure,
            Some(Look::Refused(bad, counts)) => return Now::Refused(bad.clone(), *counts),
            Some(Look::Whole(was)) => was,
        };
        match looked_through(path) {
            Look::Whole(held) if held.len() == was.len() && was.iter().zip(&held).all(|(was, now)| was.is(now)) => Now::Same,
            Look::Whole(_) => Now::Changed,
            Look::Refused(bad, counts) => Now::Refused(bad, counts),
            Look::Unsure => Now::Unsure,
        }
    }

    fn lies(&self, at: &Path) -> bool {
        fs::symlink_metadata(at).is_ok_and(|now| match self.inside {
            Some(_) => now.is_dir() && self.seal.which == Which::of(&now),
            None => self.seal.renamed(&now),
        })
    }
}

#[derive(Clone, Debug)]
pub struct Handed {
    path: PathBuf,
    was: Picked,
}

#[derive(Clone, Debug)]
pub enum Left {
    Same(Handed),
    Gone,
    Changed(PathBuf),
    Unsure(PathBuf),
    Refused(PathBuf, Vec<Trouble>, Counts),
}

impl Handed {
    #[cfg(feature = "ui")]
    pub fn of(path: PathBuf, was: Option<Picked>, taken: &dnd::Taken) -> Left {
        if taken.way() != dnd::Way::Move {
            return Left::Changed(path);
        }
        let Some(was) = was else { return Left::Changed(path) };
        match was.now(&path) {
            Now::Same => Left::Same(Handed { path, was }),
            Now::Gone => Left::Gone,
            Now::Changed => Left::Changed(path),
            Now::Unsure => Left::Unsure(path),
            Now::Refused(bad, counts) => Left::Refused(path, bad, counts),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn into_trash(&self, put: impl FnOnce(&Path) -> std::io::Result<PathBuf>) -> Result<PathBuf, (&'static str, Option<PathBuf>)> {
        match self.was.now(&self.path) {
            Now::Same => {}
            Now::Unsure | Now::Refused(..) => return Err((not_looked_through(), None)),
            Now::Gone | Now::Changed => return Err((not_as_picked(), None)),
        }
        #[cfg(any(test, feature = "testing"))]
        super::work::faults::meanwhile(&self.path);
        let lies = put(&self.path).map_err(|_| (t!("kit-proof-not-to-trash"), None))?;
        if self.was.lies(&lies) {
            return Ok(lies);
        }
        match crate::common::files::rename_new(&lies, &self.path) {
            Ok(()) => {
                super::trash::forget(&lies);
                Err((not_as_picked(), None))
            }
            Err(_) => Err((not_as_picked(), Some(lies))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn place(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("trindle-files-proof-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn folder(dir: &Path) -> File {
        use std::os::unix::fs::OpenOptionsExt;
        fs::OpenOptions::new().read(true).custom_flags(libc::O_PATH | libc::O_DIRECTORY).open(dir).unwrap()
    }

    #[test]
    fn a_thing_set_aside_has_its_name_back_or_the_nearest_free_one_and_says_where() {
        let dir = place("named-back");
        let aside = dir.join("0123456789abcdef");
        let set_aside = |holds: &[u8]| fs::write(&aside, holds).unwrap();
        set_aside(b"first");
        assert_eq!(named_back(&aside, &dir, OsStr::new("notes.txt"), false), dir.join("notes.txt"));
        set_aside(b"second");
        assert_eq!(named_back(&aside, &dir, OsStr::new("notes.txt"), false), dir.join("notes (2).txt"));
        set_aside(b"third");
        assert_eq!(named_back(&aside, &dir, OsStr::new("notes.txt"), false), dir.join("notes (3).txt"));
        let read = |name: &str| fs::read(dir.join(name)).unwrap();
        assert_eq!((read("notes.txt"), read("notes (2).txt"), read("notes (3).txt")), (b"first".to_vec(), b"second".to_vec(), b"third".to_vec()));
        assert!(!aside.exists());
        fs::create_dir(dir.join("v1.2")).unwrap();
        fs::create_dir(&aside).unwrap();
        assert_eq!(named_back(&aside, &dir, OsStr::new("v1.2"), true), dir.join("v1.2 (2)"));
        assert_eq!(named_back(&aside, &dir, OsStr::new("gone.txt"), false), aside);
        #[cfg(feature = "ui")]
        {
            let text = crate::common::source::code(include_str!("proof.rs"));
            let code = text.split_once("#[cfg(test)]\nmod tests").map_or(text, |(code, _)| code);
            assert_eq!(code.matches("named_back(").count(), 2, "the making of it counts as one");
        }
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn what_stayed_is_said_by_one_function_with_the_real_counts() {
        let c = Counts { unreadable: 2, ..Counts::default() };
        assert_eq!(c.said().as_deref(), Some("Not moved: 2 unreadable files."));
        assert_eq!(Counts { system: 1, ..Counts::default() }.said().as_deref(), Some("Not moved: 1 system file."));
        assert_eq!(Counts { other: 63_210, ..Counts::default() }.said().as_deref(), Some("Not moved: 63,210 items."));
        assert_eq!(Counts { unreadable: 1, system: 2, other: 3 }.words().as_deref(), Some("1 unreadable file, 2 system files and 3 items"));
        assert_eq!(Counts::default().said(), None);
        assert_eq!(Counts { other: 1, ..Counts::default() }.reason(), "not moved: 1 item");
        assert_eq!(Counts::default().reason(), "not moved: some items");
    }

    #[test]
    fn a_folder_of_any_size_is_looked_through_and_every_thing_that_stays_it_is_named() {
        use std::os::unix::fs::PermissionsExt;
        let dir = place("troubles");
        let album = dir.join("Photos");
        fs::create_dir_all(album.join("a")).unwrap();
        for n in 0..50_001u32 {
            fs::write(album.join(format!("{n}")), b"").unwrap();
        }
        assert!(matches!(looked_through(&album), Look::Whole(all) if all.len() == 50_002), "more than 50,000 things was refused");
        for name in ["pipe-1", "a/pipe-2"] {
            let pipe = c_name(album.join(name).as_os_str()).unwrap();
            assert_eq!(crate::sys::os::mkfifo(&pipe, 0o600), 0);
        }
        let _socket = std::os::unix::net::UnixListener::bind(album.join("a/sock")).unwrap();
        for name in ["old.jpg", "a/new.jpg"] {
            fs::write(album.join(name), b"x").unwrap();
            fs::set_permissions(album.join(name), fs::Permissions::from_mode(0o200)).unwrap();
        }
        assert!(fs::read(album.join("old.jpg")).is_err(), "{}", crate::common::files::tests::NOT_AS_ROOT);
        let Look::Refused(bad, counts) = looked_through(&album) else { panic!("a folder with such things was vouched for") };
        assert_eq!(counts, Counts { unreadable: 2, system: 3, other: 0 });
        assert_eq!(counts.words().as_deref(), Some("2 unreadable files and 3 system files"));
        assert_eq!(Counts { unreadable: 63_210, system: 0, other: 0 }.words().as_deref(), Some("63,210 unreadable files"));
        assert_eq!(Counts { unreadable: 0, system: 1, other: 0 }.words().as_deref(), Some("1 system file"));
        assert_eq!(Counts::default().words(), None);
        let mut lines: Vec<String> = bad.iter().map(Trouble::line).collect();
        lines.sort();
        assert_eq!(lines, ["Photos/a/new.jpg: no permission", "Photos/a/pipe-2: a pipe", "Photos/a/sock: a socket", "Photos/old.jpg: no permission", "Photos/pipe-1: a pipe"]);
        for n in 0..TROUBLES_NAMED + 5 {
            let pipe = c_name(album.join(format!("more-{n}")).as_os_str()).unwrap();
            assert_eq!(crate::sys::os::mkfifo(&pipe, 0o600), 0);
        }
        let Look::Refused(bad, counts) = looked_through(&album) else { panic!() };
        assert_eq!((bad.len(), counts.system), (TROUBLES_NAMED, 3 + TROUBLES_NAMED + 5));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_proof_is_made_only_for_a_whole_copy_and_takes_away_only_that_very_thing() {
        let dir = place("copied");
        let one = dir.join("one.txt");
        fs::write(&one, b"one").unwrap();
        let meta = fs::symlink_metadata(&one).unwrap();
        assert!(Copied::of_file(&meta, &meta, 2, Some(2)).is_none(), "not all of it was read");
        assert!(Copied::of_file(&meta, &meta, 3, Some(2)).is_none(), "the copy is short");
        assert!(Copied::of_file(&meta, &meta, 3, None).is_none(), "the copy could not be looked at");
        fs::write(dir.join("other.txt"), b"two").unwrap();
        assert!(Copied::of_file(&meta, &fs::symlink_metadata(dir.join("other.txt")).unwrap(), 3, Some(3)).is_none(), "another file by the end");
        assert!(Copied::of_file(&fs::symlink_metadata(&dir).unwrap(), &fs::symlink_metadata(&dir).unwrap(), 0, Some(0)).is_none(), "a folder is not a file");
        let proof = Copied::of_file(&meta, &meta, 3, Some(3)).expect("a whole copy");
        assert!(!take_away(&folder(&dir), OsStr::new("other.txt"), &proof));
        assert_eq!(fs::read(dir.join("other.txt")).unwrap(), b"two");
        fs::rename(&one, dir.join("kept.txt")).unwrap();
        fs::write(&one, b"new").unwrap();
        assert!(!take_away(&folder(&dir), OsStr::new("one.txt"), &proof));
        assert_eq!(fs::read(&one).unwrap(), b"new");
        fs::remove_file(&one).unwrap();
        fs::rename(dir.join("kept.txt"), &one).unwrap();
        let again = Copied::of_file(&fs::symlink_metadata(&one).unwrap(), &fs::symlink_metadata(&one).unwrap(), 3, Some(3)).unwrap();
        assert!(take_away(&folder(&dir), OsStr::new("one.txt"), &again));
        let left: Vec<_> = fs::read_dir(&dir).unwrap().flatten().map(|e| e.file_name()).collect();
        assert_eq!(left, ["other.txt"], "{left:?}");
        assert!(take_away(&folder(&dir), OsStr::new("one.txt"), &again));
        let far = Copied::of_far(FarSeal { size: 3, modified: Some(7) }, Some(FarSeal { size: 3, modified: Some(7) }), 3, Some(3)).unwrap();
        assert!(!take_away(&folder(&dir), OsStr::new("other.txt"), &far) && dir.join("other.txt").exists());
        assert_eq!(far.far(), Some(FarSeal { size: 3, modified: Some(7) }));
        let first = FarSeal { size: 3, modified: Some(7) };
        assert!(Copied::of_far(first, Some(FarSeal { size: 3, modified: Some(8) }), 3, Some(3)).is_none());
        assert!(Copied::of_far(first, Some(first), 2, Some(2)).is_none() && Copied::of_far(first, Some(first), 3, Some(2)).is_none());
        assert!(Copied::of_far(first, None, 3, Some(3)).is_none());
        std::os::unix::fs::symlink("/no/such/place", dir.join("link")).unwrap();
        std::os::unix::fs::symlink("/no/such/place", dir.join("copy")).unwrap();
        std::os::unix::fs::symlink("/another/place", dir.join("wrong")).unwrap();
        let link = fs::symlink_metadata(dir.join("link")).unwrap();
        assert!(Copied::of_link(&link, Path::new("/no/such/place"), &dir.join("wrong")).is_none());
        assert!(Copied::of_link(&meta, Path::new("/no/such/place"), &dir.join("copy")).is_none(), "a file is not a link");
        let proof = Copied::of_link(&link, Path::new("/no/such/place"), &dir.join("copy")).unwrap();
        assert!(take_away(&folder(&dir), OsStr::new("link"), &proof) && fs::symlink_metadata(dir.join("link")).is_err());
        fs::remove_dir_all(&dir).unwrap();
    }

    #[cfg(feature = "ui")]
    #[test]
    fn what_was_dragged_out_is_handed_over_only_as_the_thing_that_was_picked_up() {
        let dir = place("handed");
        let (file, sub) = (dir.join("report.txt"), dir.join("album"));
        fs::write(&file, b"the report").unwrap();
        fs::create_dir(&sub).unwrap();
        let moved = dnd::Taken::made_up(dnd::Way::Move);
        let (was, was_sub) = (picked_up(&file, false), picked_up(&sub, false));
        assert!(matches!(Handed::of(file.clone(), was.clone(), &moved), Left::Same(h) if h.path() == file));
        assert!(matches!(Handed::of(file.clone(), was.clone(), &dnd::Taken::made_up(dnd::Way::Copy)), Left::Changed(_)), "a copy removes nothing");
        assert!(matches!(Handed::of(file.clone(), None, &moved), Left::Changed(_)), "nothing was known of it");
        assert!(matches!(Handed::of(sub.clone(), was_sub.clone(), &moved), Left::Same(_)));
        fs::write(sub.join("new.jpg"), b"new").unwrap();
        assert!(matches!(Handed::of(sub.clone(), was_sub.clone(), &moved), Left::Changed(_)), "a file put in it since was never handed over");
        fs::remove_file(sub.join("new.jpg")).unwrap();
        assert!(matches!(Handed::of(sub.clone(), was_sub.clone(), &moved), Left::Same(_)));
        let aside = picked_up(&sub, true).unwrap();
        let until = std::time::Instant::now() + std::time::Duration::from_secs(10);
        while aside.inside.as_ref().unwrap().get().is_none() && std::time::Instant::now() < until {
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        assert!(matches!(Handed::of(sub.clone(), Some(aside), &moved), Left::Same(_)));
        let unlooked = Picked { seal: Seal::of(&fs::symlink_metadata(&sub).unwrap()), inside: Some(Arc::new(OnceLock::new())) };
        assert!(matches!(Handed::of(sub.clone(), Some(unlooked), &moved), Left::Unsure(_)));
        let pipe = c_name(sub.join("pipe").as_os_str()).unwrap();
        assert_eq!(crate::sys::os::mkfifo(&pipe, 0o600), 0);
        assert!(matches!(Handed::of(sub.clone(), picked_up(&sub, false), &moved), Left::Refused(..)));
        fs::write(&file, b"the report, and more").unwrap();
        assert!(matches!(Handed::of(file.clone(), was.clone(), &moved), Left::Changed(p) if p == file));
        fs::remove_file(&file).unwrap();
        assert!(matches!(Handed::of(file.clone(), was, &moved), Left::Gone));
        fs::remove_dir_all(&dir).unwrap();
    }
}
