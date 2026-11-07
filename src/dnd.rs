//! Drag and drop with no compositor in it: nothing a stranger sends is trusted, so every rule is bounded and tested.

use std::collections::HashSet;
use std::io::{Read, Write};
use std::os::fd::{AsRawFd, OwnedFd};
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::path::Path;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

pub const URI_LIST: &str = "text/uri-list";
pub const TEXT_KINDS: [&str; 5] = ["text/plain;charset=utf-8", "UTF8_STRING", "text/plain", "TEXT", "STRING"];
pub const TEXT_OUT: [&str; 2] = ["text/plain;charset=utf-8", "text/plain"];
pub const PORTAL_KINDS: [&str; 2] = ["application/vnd.portal.filetransfer", "application/vnd.portal.files"];

pub const MOST_BYTES: usize = 1 << 20;
pub const MOST_TIME: Duration = Duration::from_secs(5);
pub const MOST_FILES: usize = 4096;
pub fn keep_kind(kinds: &mut Vec<String>, kind: String) {
    if kinds.len() < MOST_KINDS && kind.len() <= MOST_KIND && !kinds.contains(&kind) {
        kinds.push(kind);
    }
}

pub const MOST_PATH: usize = 4095;
const MOST_PART: usize = 255;
pub const MOST_KINDS: usize = 64;
const MOST_KIND: usize = 255;
const MOST_KEY: usize = 255;
pub const MOST_SENDING: usize = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Way {
    Copy,
    Move,
    Ask,
}

impl Way {
    pub fn bits(self) -> u32 {
        match self {
            Way::Copy => 1,
            Way::Move => 2,
            Way::Ask => 4,
        }
    }

    fn from_bits(bits: u32) -> Option<Way> {
        match bits {
            1 => Some(Way::Copy),
            2 => Some(Way::Move),
            4 => Some(Way::Ask),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Ways(u32);

impl Ways {
    pub const NONE: Ways = Ways(0);
    pub const COPY: Ways = Ways(1);
    pub const MOVE: Ways = Ways(2);
    pub const ASK: Ways = Ways(4);
    pub const ALL: Ways = Ways(7);

    pub fn from_bits(bits: u32) -> Ways {
        Ways(bits & 7)
    }

    pub fn bits(self) -> u32 {
        self.0
    }

    pub fn has(self, way: Way) -> bool {
        self.0 & way.bits() != 0
    }

}

impl std::ops::BitOr for Ways {
    type Output = Ways;
    fn bitor(self, other: Ways) -> Ways {
        Ways(self.0 | other.0)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Offered {
    pub files: bool,
    pub text: bool,
    pub ways: Ways,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Kind {
    Files,
    Text,
    Typed(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Given {
    Files(Vec<PathBuf>),
    Text(String),
    Typed(String, Vec<u8>),
    Nothing(Why),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Why {
    Empty,
    TooBig,
    TooSlow,
    TooMany,
    NotFiles,
}

impl Why {
    pub fn words(self) -> &'static str {
        match self {
            Why::Empty => crate::t!("kit-dropped-empty"),
            Why::TooBig => crate::t!("kit-dropped-too-big"),
            Why::TooSlow => crate::t!("kit-dropped-too-slow"),
            Why::TooMany => crate::t!("kit-dropped-too-many"),
            Why::NotFiles => crate::t!("kit-dropped-not-files"),
        }
    }
}

pub type Wanted = Option<(Way, Kind)>;

pub fn usual_way(ways: Ways, shift: bool, ctrl: bool) -> Option<Way> {
    let first = |order: [Way; 3]| order.into_iter().find(|w| ways.has(*w));
    if ctrl {
        return first([Way::Copy, Way::Ask, Way::Move]);
    }
    if shift {
        return first([Way::Move, Way::Copy, Way::Ask]);
    }
    if ways.has(Way::Ask) && !ways.has(Way::Move) {
        return Some(Way::Ask);
    }
    first([Way::Copy, Way::Move, Way::Ask])
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Answer {
    pub accept: Option<String>,
    pub ways: Ways,
    pub rather: Option<Way>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pull {
    pub kind: Kind,
    pub main: String,
    pub portal: Option<String>,
    pub text: Option<String>,
    pub way: Way,
}

#[derive(Debug, Default)]
pub struct Incoming {
    kinds: Vec<String>,
    ways: Option<Ways>,
    wanted: Wanted,
    told: Option<Answer>,
    chosen: Option<Way>,
}

impl Incoming {
    pub fn new() -> Incoming {
        Incoming::default()
    }

    pub fn kind(&mut self, kind: String) {
        keep_kind(&mut self.kinds, kind);
    }

    pub fn source_ways(&mut self, bits: u32) {
        self.ways = Some(Ways::from_bits(bits));
    }

    pub fn chosen(&mut self, bits: u32) {
        self.chosen = Way::from_bits(bits);
    }

    pub fn kinds(&self) -> &[String] {
        &self.kinds
    }

    pub fn ways(&self) -> Ways {
        self.offered().ways
    }

    fn has(&self, kind: &str) -> bool {
        self.kinds.iter().any(|k| k == kind)
    }

    fn text_kind(&self) -> Option<&'static str> {
        TEXT_KINDS.into_iter().find(|k| self.has(k))
    }

    fn portal_kind(&self) -> Option<&'static str> {
        PORTAL_KINDS.into_iter().find(|k| self.has(k))
    }

    pub fn offered(&self) -> Offered {
        Offered {
            files: self.has(URI_LIST) || self.portal_kind().is_some(),
            text: self.text_kind().is_some(),
            ways: self.ways.unwrap_or(Ways::COPY),
        }
    }

    fn kind_for(&self, kind: &Kind) -> Option<String> {
        match kind {
            Kind::Files if self.has(URI_LIST) => Some(URI_LIST.to_string()),
            Kind::Files => self.portal_kind().map(str::to_string),
            Kind::Text => self.text_kind().map(str::to_string),
            Kind::Typed(k) => self.has(k).then(|| k.clone()),
        }
    }

    pub fn answer(&mut self, wanted: Wanted) -> Option<Answer> {
        let offered = self.offered();
        let wanted = wanted.filter(|(way, kind)| offered.ways.has(*way) && self.kind_for(kind).is_some());
        self.wanted = wanted.clone();
        let answer = match wanted {
            Some((way, kind)) => Answer { accept: self.kind_for(&kind), ways: Ways::from_bits(way.bits()), rather: Some(way) },
            None => Answer { accept: None, ways: Ways::NONE, rather: None },
        };
        if self.told.as_ref() == Some(&answer) {
            return None;
        }
        self.told = Some(answer.clone());
        Some(answer)
    }

    pub fn let_go(&self, modern: bool) -> Option<Pull> {
        let (wanted, kind) = self.wanted.clone()?;
        let way = if modern { self.chosen? } else { wanted };
        let main = self.kind_for(&kind)?;
        let (portal, text) = match kind {
            Kind::Files => (self.portal_kind().map(str::to_string).filter(|p| *p != main), self.text_kind().map(str::to_string)),
            Kind::Text | Kind::Typed(_) => (None, None),
        };
        Some(Pull { kind, main, portal, text, way })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum End {
    Took(Taken),
    Nothing,
    Short(Way),
    Unshown(Way),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Short {
    Cut(Way),
    Unshown(Way),
}

#[derive(Debug, Default)]
pub struct Outgoing {
    chosen: Option<Way>,
    let_go: bool,
    over: bool,
}

impl Outgoing {
    pub fn new() -> Outgoing {
        Outgoing::default()
    }

    pub fn action(&mut self, bits: u32) {
        if !self.over {
            self.chosen = Way::from_bits(bits);
        }
    }

    pub fn performed(&mut self) {
        self.let_go = true;
    }

    pub fn finished(&mut self, sends: &Sends) -> End {
        if std::mem::replace(&mut self.over, true) || !self.let_go {
            return End::Nothing;
        }
        let way = match self.chosen {
            Some(Way::Ask) | None => Way::Copy,
            Some(way) => way,
        };
        match (sends.taken(way), way) {
            (Some(taken), _) => End::Took(taken),
            (None, Way::Move) if sends.written() => End::Unshown(way),
            (None, Way::Move) => End::Short(way),
            (None, _) if sends.copied() => End::Nothing,
            (None, _) => End::Short(way),
        }
    }

    pub fn cancelled(&mut self) -> End {
        self.over = true;
        End::Nothing
    }

    #[cfg(any(test, feature = "testing"))]
    pub fn is_over(&self) -> bool {
        self.over
    }
}

#[derive(Debug, Default)]
pub struct Sends {
    read: AtomicUsize,
    unshown: AtomicUsize,
    late: AtomicUsize,
    late_ended: AtomicUsize,
    late_read: AtomicUsize,
    let_go: std::sync::atomic::AtomicBool,
    files: std::sync::atomic::AtomicBool,
    forms: std::sync::Mutex<std::collections::HashMap<String, Form>>,
}

#[derive(Debug, Default)]
struct Form {
    looking: usize,
    ended: bool,
    read: bool,
    any: bool,
}

impl Sends {
    pub fn for_files() -> Sends {
        let sends = Sends::default();
        sends.files.store(true, Ordering::SeqCst);
        sends
    }

    pub fn let_go(&self) {
        self.let_go.store(true, Ordering::SeqCst);
    }

    fn form<T>(&self, form: &str, then: impl FnOnce(&mut Form) -> T) -> T {
        then(self.forms.lock().unwrap_or_else(|e| e.into_inner()).entry(form.to_string()).or_default())
    }

    pub fn begin_form(&self, form: &str) -> bool {
        let late = self.let_go.load(Ordering::SeqCst);
        if late {
            self.late.fetch_add(1, Ordering::SeqCst);
        } else {
            self.form(form, |f| f.looking += 1);
        }
        late
    }

    pub fn ended_whole_form(&self, late: bool, form: &str) {
        self.read.fetch_add(1, Ordering::SeqCst);
        self.form(form, |f| f.any = true);
        if late {
            self.late_ended.fetch_add(1, Ordering::SeqCst);
            self.late_read.fetch_add(1, Ordering::SeqCst);
            self.form(form, |f| (f.ended, f.read) = (true, true));
        } else {
            self.form(form, |f| f.looking = f.looking.saturating_sub(1));
        }
    }

    pub fn ended_unshown_form(&self, late: bool, form: &str) {
        self.unshown.fetch_add(1, Ordering::SeqCst);
        if late {
            self.late_ended.fetch_add(1, Ordering::SeqCst);
            self.form(form, |f| f.ended = true);
        } else {
            self.form(form, |f| f.looking = f.looking.saturating_sub(1));
        }
    }

    pub fn ended_form(&self, late: bool, form: &str, got: &Result<Sent, Why>) {
        match got {
            Ok(Sent::Read) => self.ended_whole_form(late, form),
            Ok(Sent::Unshown) => self.ended_unshown_form(late, form),
            Err(_) => {}
        }
    }

    fn looks_settled(&self, ended: impl Fn(&Form) -> bool) -> bool {
        self.forms.lock().unwrap_or_else(|e| e.into_inner()).values().all(|f| f.looking == 0 || ended(f))
    }

    pub fn whole(&self) -> bool {
        self.read.load(Ordering::SeqCst) > 0 && self.late_read.load(Ordering::SeqCst) == self.late.load(Ordering::SeqCst)
            && self.looks_settled(|f| f.read) && self.files_read()
    }

    fn files_read(&self) -> bool {
        !self.files.load(Ordering::SeqCst)
            || self.forms.lock().unwrap_or_else(|e| e.into_inner()).iter().any(|(k, f)| f.any && (k == URI_LIST || PORTAL_KINDS.contains(&k.as_str())))
    }

    pub fn written(&self) -> bool {
        self.read.load(Ordering::SeqCst) + self.unshown.load(Ordering::SeqCst) > 0
            && self.late_ended.load(Ordering::SeqCst) == self.late.load(Ordering::SeqCst) && self.looks_settled(|f| f.ended)
    }

    pub fn copied(&self) -> bool {
        self.read.load(Ordering::SeqCst) + self.unshown.load(Ordering::SeqCst) > 0 && self.late.load(Ordering::SeqCst) == self.late_ended.load(Ordering::SeqCst)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Taken {
    way: Way,
}

impl Taken {
    pub fn way(self) -> Way {
        self.way
    }

    #[cfg(any(test, feature = "testing"))]
    pub fn made_up(way: Way) -> Taken {
        Taken { way }
    }
}

impl Sends {
    pub fn taken(&self, way: Way) -> Option<Taken> {
        self.whole().then_some(Taken { way })
    }
}

pub fn ways_out(right: bool, shift: bool) -> Ways {
    if right {
        Ways::ASK | Ways::COPY
    } else if shift {
        Ways::MOVE
    } else {
        Ways::COPY
    }
}

pub fn writable_out(ways: Ways) -> bool {
    ways == Ways::MOVE
}

pub fn out_plan(modern: bool, files: bool, right: bool, shift: bool) -> (Ways, bool) {
    let ways = if files { ways_out(right, shift) } else { Ways::COPY };
    (ways, modern && files && writable_out(ways))
}

pub fn newer(a: u32, b: u32) -> bool {
    a != b && a.wrapping_sub(b) < u32::MAX / 2
}

#[cfg(any(test, feature = "testing"))]
pub fn write_all(to: OwnedFd, bytes: &[u8], end: Instant) -> Result<Sent, Why> {
    write_all_then(to, bytes, end, |_| ())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sent {
    Read,
    Unshown,
}

#[cfg(any(test, feature = "testing"))]
pub fn proven(got: &Result<Sent, Why>) -> bool {
    matches!(got, Ok(Sent::Read))
}

pub fn write_all_then(to: OwnedFd, bytes: &[u8], end: Instant, then: impl FnOnce(&Result<Sent, Why>)) -> Result<Sent, Why> {
    let mut file = std::fs::File::from(to);
    let got = written(&mut file, bytes, end);
    then(&got);
    drop(file);
    got
}

fn written(file: &mut std::fs::File, bytes: &[u8], end: Instant) -> Result<Sent, Why> {
    let flags = crate::sys::fd::flags(file.as_raw_fd());
    if flags < 0 || crate::sys::fd::set_flags(file.as_raw_fd(), flags | libc::O_NONBLOCK) < 0 {
        return Err(Why::Empty);
    }
    let mut sent = 0;
    while sent < bytes.len() {
        let left = end.saturating_duration_since(Instant::now());
        if left.is_zero() {
            return Err(Why::TooSlow);
        }
        let mut fd = libc::pollfd { fd: file.as_raw_fd(), events: libc::POLLOUT, revents: 0 };
        let ready = crate::sys::fd::poll(std::slice::from_mut(&mut fd), left.as_millis().min(i32::MAX as u128) as i32);
        if ready == 0 {
            return Err(Why::TooSlow);
        }
        if ready < 0 {
            if std::io::Error::last_os_error().kind() == std::io::ErrorKind::Interrupted {
                continue;
            }
            return Err(Why::Empty);
        }
        match file.write(&bytes[sent..]) {
            Ok(0) => return Err(Why::Empty),
            Ok(n) => sent += n,
            Err(e) if matches!(e.kind(), std::io::ErrorKind::WouldBlock | std::io::ErrorKind::Interrupted) => {}
            Err(_) => return Err(Why::Empty),
        }
    }
    read_out(file, end)
}

fn read_out(file: &std::fs::File, end: Instant) -> Result<Sent, Why> {
    use std::os::unix::fs::FileTypeExt;
    let kind = file.metadata().map_err(|_| Why::Empty)?.file_type();
    if !kind.is_fifo() {
        return if kind.is_file() || kind.is_socket() { Ok(Sent::Unshown) } else { Err(Why::Empty) };
    }
    let unread = || crate::sys::fd::unread(file.as_raw_fd());
    loop {
        match unread() {
            Some(0) => return Ok(Sent::Read),
            Some(_) => {}
            None => return Err(Why::Empty),
        }
        let left = end.saturating_duration_since(Instant::now());
        if left.is_zero() {
            return Err(Why::TooSlow);
        }
        let mut fd = libc::pollfd { fd: file.as_raw_fd(), events: 0, revents: 0 };
        let woke = crate::sys::fd::poll(std::slice::from_mut(&mut fd), left.as_millis().min(10) as i32);
        if woke > 0 && fd.revents & (libc::POLLERR | libc::POLLHUP | libc::POLLNVAL) != 0 {
            return if unread() == Some(0) { Ok(Sent::Read) } else { Err(Why::Empty) };
        }
    }
}

fn checked(path: &[u8]) -> Option<PathBuf> {
    if path.first() != Some(&b'/') || path.len() > MOST_PATH || path.iter().any(|b| *b < 0x20 || *b == 0x7f) {
        return None;
    }
    let inner = path[1..].strip_suffix(b"/").unwrap_or(&path[1..]);
    if inner.is_empty() {
        return None;
    }
    for part in inner.split(|b| *b == b'/') {
        if part.is_empty() || part == b"." || part == b".." || part.len() > MOST_PART {
            return None;
        }
    }
    Some(PathBuf::from(std::ffi::OsString::from_vec(path[..=inner.len()].to_vec())))
}

fn hex(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

fn file_of(line: &[u8]) -> Option<PathBuf> {
    let rest = line.get(..7).filter(|s| s.eq_ignore_ascii_case(b"file://")).and(line.get(7..))?;
    let slash = rest.iter().position(|b| *b == b'/')?;
    let (host, path) = rest.split_at(slash);
    if !host.is_empty() && !host.eq_ignore_ascii_case(b"localhost") {
        return None;
    }
    let mut out = Vec::with_capacity(path.len());
    let mut bytes = path.iter().copied();
    while let Some(b) = bytes.next() {
        match b {
            b'%' => out.push(hex(bytes.next()?)? * 16 + hex(bytes.next()?)?),
            b'?' | b'#' => return None,
            b if b.is_ascii_graphic() => out.push(b),
            _ => return None,
        }
    }
    checked(&out)
}

pub fn parse_uri_list(bytes: &[u8]) -> Result<Vec<PathBuf>, Why> {
    if bytes.len() > MOST_BYTES {
        return Err(Why::TooBig);
    }
    let mut out: Vec<PathBuf> = Vec::new();
    let mut seen = HashSet::new();
    let bytes = bytes.strip_suffix(&[0]).unwrap_or(bytes);
    for line in bytes.split(|b| *b == b'\n') {
        let line = line.strip_suffix(b"\r").unwrap_or(line);
        if line.is_empty() || line.first() == Some(&b'#') {
            continue;
        }
        let path = file_of(line).ok_or(Why::NotFiles)?;
        if seen.insert(path.clone()) {
            if out.len() == MOST_FILES {
                return Err(Why::TooMany);
            }
            out.push(path);
        }
    }
    if out.is_empty() { Err(Why::Empty) } else { Ok(out) }
}

pub fn checked_paths(paths: &[String]) -> Result<Vec<PathBuf>, Why> {
    if paths.len() > MOST_FILES {
        return Err(Why::TooMany);
    }
    let mut out: Vec<PathBuf> = Vec::new();
    let mut seen = HashSet::new();
    for p in paths {
        let path = checked(p.as_bytes()).ok_or(Why::NotFiles)?;
        if seen.insert(path.clone()) {
            out.push(path);
        }
    }
    if out.is_empty() { Err(Why::Empty) } else { Ok(out) }
}

pub fn portal_key(bytes: &[u8]) -> Option<String> {
    let key = bytes.strip_suffix(&[0]).unwrap_or(bytes);
    (!key.is_empty() && key.len() <= MOST_KEY && key.iter().all(u8::is_ascii_graphic))
        .then(|| String::from_utf8_lossy(key).into_owned())
}

pub fn clean_text(bytes: &[u8]) -> Option<String> {
    let text: String = String::from_utf8_lossy(bytes).chars()
        .filter(|c| *c == '\n' || *c == '\t' || !(c.is_control() || is_hidden_mark(*c)))
        .collect();
    (!text.is_empty()).then_some(text)
}

fn is_hidden_mark(c: char) -> bool {
    matches!(c, '\u{61C}' | '\u{200B}' | '\u{200E}' | '\u{200F}' | '\u{2028}' | '\u{2029}' | '\u{202A}'..='\u{202E}' | '\u{2060}'..='\u{2069}' | '\u{FEFF}')
}

pub fn uri_list(paths: &[PathBuf]) -> String {
    let mut out = String::new();
    for path in paths.iter().filter(|p| p.is_absolute()) {
        out.push_str("file://");
        for b in path.as_os_str().as_bytes() {
            match b {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'/' | b'-' | b'.' | b'_' | b'~' => out.push(*b as char),
                b => out.push_str(&format!("%{b:02X}")),
            }
        }
        out.push_str("\r\n");
    }
    out
}

pub fn path_lines(paths: &[PathBuf]) -> String {
    paths.iter().map(|p| p.to_string_lossy().into_owned()).collect::<Vec<_>>().join("\n")
}

pub fn read_all(from: OwnedFd, most: usize, end: Instant) -> Result<Vec<u8>, Why> {
    let mut file = std::fs::File::from(from);
    let mut got = Vec::new();
    let mut chunk = [0u8; 16384];
    loop {
        let left = end.saturating_duration_since(Instant::now());
        if left.is_zero() {
            return Err(Why::TooSlow);
        }
        let mut fd = libc::pollfd { fd: file.as_raw_fd(), events: libc::POLLIN, revents: 0 };
        let ready = crate::sys::fd::poll(std::slice::from_mut(&mut fd), left.as_millis().min(i32::MAX as u128) as i32);
        if ready == 0 {
            return Err(Why::TooSlow);
        }
        if ready < 0 {
            if std::io::Error::last_os_error().kind() == std::io::ErrorKind::Interrupted {
                continue;
            }
            return Err(Why::Empty);
        }
        match file.read(&mut chunk) {
            Ok(0) => return Ok(got),
            Ok(n) if got.len() + n > most => return Err(Why::TooBig),
            Ok(n) => got.extend_from_slice(&chunk[..n]),
            Err(e) if matches!(e.kind(), std::io::ErrorKind::WouldBlock | std::io::ErrorKind::Interrupted) => {}
            Err(_) => return Err(Why::Empty),
        }
    }
}

pub fn given(kind: Kind, main: Result<Vec<u8>, Why>, portal: Option<Vec<PathBuf>>, text: Option<Vec<u8>>) -> Given {
    if let (Kind::Files, Some(files)) = (&kind, portal) {
        return Given::Files(files);
    }
    let bytes = match main {
        Ok(b) => b,
        Err(why) => return Given::Nothing(why),
    };
    match kind {
        Kind::Files => match parse_uri_list(&bytes) {
            Ok(files) => Given::Files(files),
            Err(Why::NotFiles) => text.and_then(|t| clean_text(&t)).map_or(Given::Nothing(Why::NotFiles), Given::Text),
            Err(why) => Given::Nothing(why),
        },
        Kind::Text => clean_text(&bytes).map_or(Given::Nothing(Why::Empty), Given::Text),
        Kind::Typed(_) if bytes.is_empty() => Given::Nothing(Why::Empty),
        Kind::Typed(k) => Given::Typed(k, bytes),
    }
}

pub fn is_plain(path: &Path) -> bool {
    checked(path.as_os_str().as_bytes()).is_some()
}

pub mod portal {
    use std::collections::HashMap;
    use std::os::fd::AsFd;
    use std::path::{Path, PathBuf};

    use zbus::blocking::Connection;
    use zbus::zvariant::{Fd, Value};

    pub(super) const NAME: &str = "org.freedesktop.portal.Documents";
    pub(super) const PATH: &str = "/org/freedesktop/portal/documents";
    pub(super) const FACE: &str = "org.freedesktop.portal.FileTransfer";
    const AT_ONCE: usize = 16;

    pub fn retrieve(key: &str) -> Option<Vec<String>> {
        let bus = session()?;
        let none: HashMap<&str, Value<'_>> = HashMap::new();
        bus.call_method(Some(NAME), PATH, Some(FACE), "RetrieveFiles", &(key, none)).ok()?.body().deserialize().ok()
    }

    pub fn stop_on(bus: &Connection, key: &str) {
        let _ = bus.call_method(Some(NAME), PATH, Some(FACE), "StopTransfer", &(key,));
    }

    pub fn start(files: &[PathBuf], writable: bool) -> Option<(String, Connection)> {
        let bus = session()?;
        let key = start_on(&bus, files, writable)?;
        Some((key, bus))
    }

    fn session() -> Option<Connection> {
        zbus::blocking::connection::Builder::session().ok()?.method_timeout(super::MOST_TIME).build().ok()
    }

    pub(super) fn opened(file: &Path, writable: bool) -> Option<std::fs::File> {
        use crate::common::files::{open_named, readable, Named};
        match open_named(file, None, None).ok()? {
            Named::File(named, _) | Named::Folder(named, _) if writable => Some(named),
            Named::File(named, _) | Named::Folder(named, _) => readable(&named).ok(),
            Named::Link(..) => None,
        }
    }

    pub fn start_on(bus: &Connection, files: &[PathBuf], writable: bool) -> Option<String> {
        let mut how: HashMap<&str, Value<'_>> = HashMap::new();
        how.insert("writable", Value::Bool(writable));
        how.insert("autostop", Value::Bool(true));
        let key: String = bus.call_method(Some(NAME), PATH, Some(FACE), "StartTransfer", &(how,)).ok()?.body().deserialize().ok()?;
        let added = || -> Option<()> {
            for some in files.chunks(AT_ONCE) {
                let open: Vec<std::fs::File> = some.iter().filter_map(|f| opened(f, writable)).collect();
                if open.len() != some.len() {
                    return None;
                }
                let fds: Vec<Fd<'_>> = open.iter().map(|f| Fd::from(f.as_fd())).collect();
                let none: HashMap<&str, Value<'_>> = HashMap::new();
                bus.call_method(Some(NAME), PATH, Some(FACE), "AddFiles", &(key.as_str(), fds, none)).ok()?;
            }
            Some(())
        };
        if added().is_none() {
            let _ = bus.call_method(Some(NAME), PATH, Some(FACE), "StopTransfer", &(key.as_str(),));
            return None;
        }
        Some(key)
    }
}

#[cfg(test)]
mod tests;
