//! Config paths and file writes; nothing is written over a name made meanwhile, and a name a stranger gave is opened only by `open_named` and joined only if `is_one_name`.

use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::os::fd::AsRawFd;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::{DirBuilderExt, MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use crate::t;

pub use trindle_common::files::fits_a_line;

pub fn home() -> PathBuf {
    trindle_common::files::home().or_else(crate::sys::os::home_dir).unwrap_or_else(|| PathBuf::from("/"))
}

fn under_home(found: Option<PathBuf>, under: &str) -> PathBuf {
    found.unwrap_or_else(|| home().join(under))
}

pub fn config_home() -> PathBuf {
    under_home(trindle_common::files::config_home(), ".config")
}

pub fn data_home() -> PathBuf {
    under_home(trindle_common::files::data_home(), ".local/share")
}

pub fn cache_home() -> PathBuf {
    under_home(trindle_common::files::cache_home(), ".cache")
}

pub fn user_folder(value: &str, home: &Path) -> Option<PathBuf> {
    let value = value.trim().trim_matches('"');
    match value.strip_prefix("$HOME") {
        Some("") => Some(home.to_path_buf()),
        Some(rest) => rest.strip_prefix('/').map(|rest| home.join(rest.trim_start_matches('/'))),
        None => value.starts_with('/').then(|| PathBuf::from(value)),
    }
}

pub fn user_folder_for(listed: &str, key: &str, home: &Path) -> Option<PathBuf> {
    let name = format!("XDG_{key}_DIR=");
    listed.lines().find_map(|l| l.trim_start().strip_prefix(&name)).and_then(|v| user_folder(v, home))
}

pub fn user_folders(listed: &str, home: &Path) -> Vec<PathBuf> {
    listed.lines().map(str::trim_start).filter(|l| l.starts_with("XDG_"))
        .filter_map(|l| user_folder(l.split_once('=')?.1, home)).collect()
}

const THE_COMPUTERS: [&str; 10] = ["/usr", "/etc", "/opt", "/bin", "/sbin", "/lib", "/lib64", "/var/lib", "/var/cache", "/snap"];

const HERE: [&str; 17] = ["ext4", "ext3", "ext2", "btrfs", "xfs", "f2fs", "zfs", "bcachefs", "jfs", "reiserfs", "nilfs2", "tmpfs", "ramfs",
                          "overlay", "squashfs", "erofs", "ecryptfs"];

const MOST_LINKS: usize = 40;

pub fn own_place(path: &Path) -> bool {
    let mut roots: Vec<PathBuf> = THE_COMPUTERS.iter().map(PathBuf::from).collect();
    roots.extend([config_home(), data_home(), cache_home()]);
    roots.extend(beside_the_program().iter().cloned());
    mounted().is_some_and(|mounts| own_among(path, &roots, &mounts))
}

fn own_among(path: &Path, roots: &[PathBuf], mounts: &[(PathBuf, bool)]) -> bool {
    use std::path::Component;
    if !path.is_absolute() || path.components().any(|c| matches!(c, Component::ParentDir)) || !roots.iter().any(|own| path.starts_with(own)) {
        return false;
    }
    let here = |at: &Path| mounts.iter().rev().find(|(mount, _)| mount == at).map(|(_, here)| *here);
    if here(Path::new("/")) != Some(true) {
        return false;
    }
    let mut left: Vec<std::ffi::OsString> = path.components().rev().filter_map(|c| match c {
        Component::Normal(name) => Some(name.to_os_string()),
        _ => None,
    }).collect();
    let mut at = PathBuf::from("/");
    let mut links = 0;
    while let Some(name) = left.pop() {
        if name == ".." {
            at.pop();
            continue;
        }
        at.push(&name);
        if here(&at) == Some(false) {
            return false;
        }
        match fs::symlink_metadata(&at) {
            Err(_) => return true,
            Ok(meta) if meta.file_type().is_symlink() => {
                links += 1;
                let Ok(to) = fs::read_link(&at) else { return true };
                if links > MOST_LINKS {
                    return false;
                }
                at.pop();
                if to.is_absolute() {
                    at = PathBuf::from("/");
                }
                left.extend(to.components().rev().filter_map(|c| match c {
                    Component::Normal(name) => Some(name.to_os_string()),
                    Component::ParentDir => Some("..".into()),
                    _ => None,
                }));
            }
            Ok(_) => {}
        }
    }
    true
}

fn mounts_from(text: &[u8]) -> Vec<(PathBuf, bool)> {
    use std::os::unix::ffi::OsStringExt;
    text.split(|b| *b == b'\n').filter_map(|line| {
        let dash = line.windows(3).position(|w| w == b" - ")?;
        let (before, after) = (&line[..dash], &line[dash + 3..]);
        let (at, kind) = (before.split(|b| *b == b' ').nth(4)?, std::str::from_utf8(after.split(|b| *b == b' ').next()?).ok()?);
        let (mut name, mut bytes) = (Vec::new(), at.iter().copied());
        while let Some(b) = bytes.next() {
            if b != b'\\' {
                name.push(b);
                continue;
            }
            let digits: Vec<u8> = bytes.by_ref().take(3).collect();
            name.push(std::str::from_utf8(&digits).ok().and_then(|d| u8::from_str_radix(d, 8).ok())?);
        }
        Some((PathBuf::from(std::ffi::OsString::from_vec(name)), HERE.contains(&kind)))
    }).collect()
}

fn mounted() -> Option<std::sync::Arc<Vec<(PathBuf, bool)>>> {
    use std::io::{Read, Seek};
    use std::sync::{Arc, Mutex};
    type Kept = (fs::File, Arc<Vec<(PathBuf, bool)>>);
    static KEPT: Mutex<Option<Kept>> = Mutex::new(None);
    let read = |mut list: &fs::File| -> Option<Arc<Vec<(PathBuf, bool)>>> {
        let mut text = Vec::new();
        list.rewind().ok()?;
        list.read_to_end(&mut text).ok()?;
        Some(Arc::new(mounts_from(&text)))
    };
    let mut kept = KEPT.lock().ok()?;
    if let Some((list, mounts)) = kept.as_mut() {
        let mut asked = libc::pollfd { fd: list.as_raw_fd(), events: libc::POLLPRI, revents: 0 };
        let changed = crate::sys::fd::poll(std::slice::from_mut(&mut asked), 0) != 0;
        if changed {
            *mounts = read(list)?;
            MOUNTS_READ.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        }
        return Some(mounts.clone());
    }
    let list = fs::File::open("/proc/self/mountinfo").ok()?;
    let mounts = read(&list)?;
    *kept = Some((list, mounts.clone()));
    MOUNTS_READ.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    Some(mounts)
}

static MOUNTS_READ: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

pub fn mounts_seen() -> usize {
    let _ = mounted();
    MOUNTS_READ.load(std::sync::atomic::Ordering::SeqCst)
}

pub fn here(path: &Path) -> bool {
    mounted().is_some_and(|mounts| own_among(path, &[PathBuf::from("/")], &mounts))
}

pub const START_WAIT: std::time::Duration = std::time::Duration::from_millis(400);

const SHY_AFTER_LATE: std::time::Duration = std::time::Duration::from_secs(30);

struct Gentle {
    slots: super::run::Slots,
    late: std::sync::Mutex<Option<std::time::Instant>>,
}

static START: Gentle = Gentle::new(4);

pub fn gently<T: Send + 'static>(path: &Path, wait: std::time::Duration, look: impl FnOnce(&Path) -> T + Send + 'static) -> Option<T> {
    let at = path.to_path_buf();
    START.run(here(&at), wait, move || look(&at))
}

pub fn gently_over<T: Send + 'static>(paths: &[PathBuf], wait: std::time::Duration, work: impl FnOnce() -> T + Send + 'static) -> Option<T> {
    START.run(paths.iter().all(|p| here(p)), wait, work)
}

impl Gentle {
    const fn new(most: usize) -> Gentle {
        Gentle { slots: super::run::Slots::new(most), late: std::sync::Mutex::new(None) }
    }

    fn run<T: Send + 'static>(&'static self, here: bool, wait: std::time::Duration, work: impl FnOnce() -> T + Send + 'static) -> Option<T> {
        if here {
            return Some(work());
        }
        if self.late.lock().ok()?.is_some_and(|late| late.elapsed() < SHY_AFTER_LATE) {
            return None;
        }
        let slot = self.slots.take()?;
        let got = super::run::on_slot(slot, "gently", wait, work);
        if got.is_none() {
            if let Ok(mut late) = self.late.lock() {
                *late = Some(std::time::Instant::now());
            }
        }
        got
    }
}

fn beside_the_program() -> &'static [PathBuf] {
    static BESIDE: std::sync::OnceLock<Vec<PathBuf>> = std::sync::OnceLock::new();
    BESIDE.get_or_init(|| {
        std::env::current_exe().map(|exe| exe.ancestors().skip(1).take(4).map(|dir| dir.join("share")).collect()).unwrap_or_default()
    })
}

pub fn trindle_config() -> PathBuf {
    config_home().join("trindle")
}

pub fn config_path(name: &str) -> PathBuf {
    trindle_config().join(name)
}

fn disk_says(path: &Path) -> Option<libc::statvfs> {
    let c = std::ffi::CString::new(path.as_os_str().as_bytes()).ok()?;
    crate::sys::os::statvfs(&c)
}

pub fn room_left(path: &Path) -> Option<u64> {
    disk_says(path).map(|st| st.f_bavail.saturating_mul(st.f_frsize))
}

pub fn how_full(path: &Path) -> Option<(u64, u64)> {
    let st = disk_says(path)?;
    let used = st.f_blocks.saturating_sub(st.f_bfree).saturating_mul(st.f_frsize);
    let free = st.f_bavail.saturating_mul(st.f_frsize);
    (used.saturating_add(free) > 0).then_some((used, free))
}

pub fn share(name: &str) -> PathBuf {
    let mut places = vec![PathBuf::from("/usr/lib/trindle/share").join(name),
                          PathBuf::from("/usr/share/trindle").join(name)];
    if let Ok(exe) = std::env::current_exe() {
        for dir in exe.ancestors().skip(1).take(4) {
            places.push(dir.join("share").join(name));
        }
    }
    places.iter().find(|p| p.is_dir()).cloned().unwrap_or_else(|| places[0].clone())
}

pub fn uid_of(name: &str) -> Option<u32> {
    crate::sys::os::uid_of(&std::ffi::CString::new(name).ok()?)
}

pub fn user() -> String {
    crate::sys::os::user()
}

pub fn scratch() -> PathBuf {
    trindle_common::files::runtime_dir().unwrap_or_else(|| home().join(".cache/trindle"))
}

pub fn private_temp(name: &str) -> std::io::Result<PathBuf> {
    private_in(&std::env::temp_dir(), name)
}

pub fn private_in(dir: &Path, name: &str) -> std::io::Result<PathBuf> {
    use std::io::Read;
    use std::os::unix::fs::DirBuilderExt;
    let mut bytes = [0u8; 12];
    fs::File::open("/dev/urandom")?.read_exact(&mut bytes)?;
    let tag = bytes.iter().fold(String::new(), |mut s, b| {
        let _ = std::fmt::Write::write_fmt(&mut s, format_args!("{b:02x}"));
        s
    });
    let dir = dir.join(format!("{name}-{tag}"));
    fs::DirBuilder::new().mode(0o700).create(&dir)?;
    Ok(dir)
}

pub fn read_text(path: impl AsRef<Path>) -> String {
    use std::io::Read;
    let mut text = String::new();
    open_settings(path.as_ref()).and_then(|mut file| file.read_to_string(&mut text)).map_or_else(|_| String::new(), |_| text)
}

pub fn open_settings(path: &Path) -> std::io::Result<fs::File> {
    let file = fs::OpenOptions::new().read(true).custom_flags(libc::O_NONBLOCK).open(path)?;
    if !file.metadata()?.is_file() {
        return Err(not_a_file());
    }
    Ok(file)
}

pub fn open_file(path: impl AsRef<Path>) -> std::io::Result<fs::File> {
    let named = fs::OpenOptions::new().read(true).custom_flags(libc::O_PATH | libc::O_CLOEXEC).open(path.as_ref())?;
    if !named.metadata()?.is_file() {
        return Err(not_a_file());
    }
    readable(&named)
}

pub fn read_file(path: impl AsRef<Path>) -> std::io::Result<Vec<u8>> {
    use std::io::Read;
    let mut bytes = Vec::new();
    open_file(path.as_ref())?.read_to_end(&mut bytes)?;
    Ok(bytes)
}

pub fn read_string(path: impl AsRef<Path>) -> std::io::Result<String> {
    use std::io::Read;
    let mut text = String::new();
    open_file(path.as_ref())?.read_to_string(&mut text)?;
    Ok(text)
}

pub fn free_in(name: &std::ffi::OsStr, folder: bool, mut taken: impl FnMut(&std::ffi::OsStr) -> bool) -> std::ffi::OsString {
    use std::os::unix::ffi::OsStringExt;
    let b = name.as_bytes();
    let dot = if folder { None } else { b.iter().rposition(|&c| c == b'.').filter(|&i| i > 0) };
    let (stem, end) = match dot {
        Some(i) => (&b[..i], &b[i..]),
        None => (b, &b[b.len()..]),
    };
    let mut n = 2;
    loop {
        let mut out = stem.to_vec();
        out.extend_from_slice(format!(" ({n})").as_bytes());
        out.extend_from_slice(end);
        let p = std::ffi::OsString::from_vec(out);
        if !taken(&p) {
            return p;
        }
        n += 1;
    }
}

pub const SMALL_MOST: u64 = 1 << 20;

pub fn read_small(path: &Path) -> std::io::Result<String> {
    read_within(path, SMALL_MOST)
}

pub fn read_within(path: &Path, most: u64) -> std::io::Result<String> {
    use std::io::{Error, ErrorKind};
    String::from_utf8(read_bytes_within(path, most)?)
        .map_err(|_| Error::new(ErrorKind::InvalidData, t!("kit-files-not-text")))
}

fn open_kept(path: &Path) -> std::io::Result<Option<fs::File>> {
    match open_settings(path) {
        Ok(f) => Ok(Some(f)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(kept_why(e)),
    }
}

fn kept_why(e: std::io::Error) -> std::io::Error {
    use std::io::{Error, ErrorKind};
    match e.kind() {
        ErrorKind::PermissionDenied => Error::new(e.kind(), t!("kit-files-not-allowed")),
        _ if is_not_a_file(&e) => Error::new(e.kind(), t!("kit-files-not-ordinary")),
        kind => Error::new(kind, t!("kit-files-unreadable")),
    }
}

pub fn read_bytes_within(path: &Path, most: u64) -> std::io::Result<Vec<u8>> {
    use std::io::{Error, ErrorKind, Read};
    let Some(file) = open_kept(path)? else { return Ok(Vec::new()) };
    let mut bytes = Vec::new();
    file.take(most.saturating_add(1)).read_to_end(&mut bytes).map_err(kept_why)?;
    if bytes.len() as u64 > most {
        return Err(Error::new(ErrorKind::InvalidData, t!("kit-files-too-big")));
    }
    Ok(bytes)
}

pub fn may_rewrite(path: &Path) -> std::io::Result<()> {
    open_kept(path).map(drop)
}

pub fn write_text(path: &Path, text: &str) -> std::io::Result<()> {
    write_whole(path, text.as_bytes())
}

pub fn write_whole(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    write_new(path, bytes, Kept::Mode)
}

pub fn write_whole_private(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    write_new(path, bytes, Kept::Private)
}

pub fn write_whole_owned(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    write_new(path, bytes, Kept::Owner)
}

#[derive(Clone, Copy, PartialEq)]
enum Kept {
    Mode,
    Owner,
    Private,
}

fn write_new(path: &Path, bytes: &[u8], kept: Kept) -> std::io::Result<()> {
    write_new_by(path, kept, &mut |file| file.write_all(bytes))
}

pub fn write_whole_by(path: &Path, write: &mut dyn FnMut(&mut fs::File) -> std::io::Result<()>) -> std::io::Result<()> {
    write_new_by(path, Kept::Mode, write)
}

fn write_new_by(path: &Path, kept: Kept, write: &mut dyn FnMut(&mut fs::File) -> std::io::Result<()>) -> std::io::Result<()> {
    use std::io::{Error, ErrorKind};
    let real = led_to(path)?;
    let (Some(dir), Some(name)) = (real.parent(), real.file_name()) else {
        return Err(Error::new(ErrorKind::InvalidInput, t!("kit-files-no-name-kept")));
    };
    let dir = if dir.as_os_str().is_empty() { Path::new(".") } else { dir };
    if kept == Kept::Private {
        fs::DirBuilder::new().recursive(true).mode(0o700).create(dir)?;
    } else {
        fs::create_dir_all(dir)?;
    }
    let (spare, mut file) = open_spare(dir, name)?;
    let written = fill_spare(&mut file, &spare, &real, kept, write);
    if written.is_err() {
        let _ = fs::remove_file(&spare);
        return written;
    }
    folder_out(dir)
}

fn open_spare(dir: &Path, name: &std::ffi::OsStr) -> std::io::Result<(PathBuf, fs::File)> {
    use std::io::ErrorKind;
    use std::sync::atomic::{AtomicU64, Ordering};
    static SPARES: AtomicU64 = AtomicU64::new(0);
    const TRIES: usize = 16;
    let mut tried = 0;
    loop {
        let number = SPARES.fetch_add(1, Ordering::Relaxed);
        #[cfg(any(test, feature = "testing"))]
        let number = SPARE_NEXT.take().unwrap_or(number);
        let spare = dir.join(spare_name(name, number));
        tried += 1;
        match fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(&spare) {
            Ok(file) => return Ok((spare, file)),
            Err(e) if e.kind() == ErrorKind::AlreadyExists && tried < TRIES => continue,
            Err(e) => return Err(e),
        }
    }
}

fn fill_spare(file: &mut fs::File, spare: &Path, real: &Path, kept: Kept, write: &mut dyn FnMut(&mut fs::File) -> std::io::Result<()>) -> std::io::Result<()> {
    #[cfg(any(test, feature = "testing"))]
    SPARE_MODE.set(file.metadata().ok().map(|m| m.mode() & 0o777));
    #[cfg(any(test, feature = "testing"))]
    if FULL.get() {
        write(file)?;
        file.set_len(file.metadata()?.len() / 2)?;
        return Err(std::io::Error::from_raw_os_error(libc::ENOSPC));
    }
    write(file)?;
    if kept == Kept::Owner {
        if let Ok(old) = fs::metadata(real) {
            use std::os::unix::fs::MetadataExt;
            let new = file.metadata()?;
            if (new.uid(), new.gid()) != (old.uid(), old.gid()) {
                std::os::unix::fs::fchown(&*file, Some(old.uid()), Some(old.gid()))?;
            }
        }
    }
    let mode = match fs::metadata(real) {
        _ if kept == Kept::Private => 0o600,
        Ok(old) => old.permissions().mode() & 0o7777,
        Err(_) => 0o666 & !umask(),
    };
    file.set_permissions(fs::Permissions::from_mode(mode))?;
    file.sync_all()?;
    fs::rename(spare, real)
}

pub fn folder_out(dir: &Path) -> std::io::Result<()> {
    let Ok(folder) = fs::File::open(dir) else { return Ok(()) };
    match folder.sync_all() {
        Err(e) if drive_refused(&e) => Err(e),
        _ => Ok(()),
    }
}

fn drive_refused(e: &std::io::Error) -> bool {
    matches!(e.raw_os_error(), Some(libc::EIO | libc::ENOSPC | libc::EDQUOT))
}

fn led_to(path: &Path) -> std::io::Result<PathBuf> {
    use std::io::{Error, ErrorKind};
    let nowhere = || Error::new(ErrorKind::NotFound, t!("kit-files-dangling"));
    let mut at = path.to_path_buf();
    for _ in 0..MOST_LINKS {
        if !fs::symlink_metadata(&at).is_ok_and(|m| m.file_type().is_symlink()) {
            if at == path {
                return Ok(at);
            }
            let (Some(up), Some(name)) = (at.parent(), at.file_name()) else { return Err(nowhere()) };
            let up = if up.as_os_str().is_empty() { Path::new(".") } else { up };
            return fs::canonicalize(up).map(|up| up.join(name)).map_err(|_| nowhere());
        }
        let to = fs::read_link(&at)?;
        at = match at.parent() {
            Some(up) => up.join(to),
            None => to,
        };
    }
    Err(nowhere())
}

fn umask() -> u32 {
    fs::read_to_string("/proc/self/status").ok()
        .and_then(|status| status.lines().find_map(|l| l.strip_prefix("Umask:").and_then(|m| u32::from_str_radix(m.trim(), 8).ok())))
        .unwrap_or(0o077)
}

pub const SPARE_END: &str = ".trindle-part";

pub fn spare_name(name: &std::ffi::OsStr, number: u64) -> std::ffi::OsString {
    let mut spare = std::ffi::OsString::from(".");
    spare.push(name);
    spare.push(format!(".{}-{number}.trindle-tmp", std::process::id()));
    spare
}

#[cfg(any(test, feature = "testing"))]
pub fn unspare(path: &std::path::Path) -> std::path::PathBuf {
    path.components()
        .map(|c| {
            let name = c.as_os_str().to_string_lossy();
            match name.strip_suffix(SPARE_END) {
                Some(rest) => match rest.rsplit_once('.') {
                    Some((real, tag)) if tag.len() == 16 && tag.bytes().all(|b| b.is_ascii_hexdigit()) => std::ffi::OsString::from(real),
                    _ => c.as_os_str().to_owned(),
                },
                None => c.as_os_str().to_owned(),
            }
        })
        .collect()
}

pub fn rename_new(from: &Path, to: &Path) -> std::io::Result<()> {
    use std::io::{Error, ErrorKind};
    let c = |p: &Path| std::ffi::CString::new(p.as_os_str().as_bytes()).map_err(|_| Error::new(ErrorKind::InvalidInput, t!("kit-files-nought")));
    let (a, b) = (c(from)?, c(to)?);
    #[cfg(any(test, feature = "testing"))]
    let asked = !NO_REFUSE.get();
    #[cfg(not(any(test, feature = "testing")))]
    let asked = true;
    if asked && crate::sys::os::rename_new(&a, &b) == 0 {
        return Ok(());
    }
    let e = if asked { Error::last_os_error() } else { Error::from_raw_os_error(libc::EINVAL) };
    match e.raw_os_error() {
        Some(libc::ENOSYS | libc::EINVAL) => take_unrefused(from, to),
        _ => Err(e),
    }
}

fn take_unrefused(from: &Path, to: &Path) -> std::io::Result<()> {
    use std::io::{Error, ErrorKind};
    let taken = |e: Error| match e.kind() {
        ErrorKind::AlreadyExists => Error::new(ErrorKind::AlreadyExists, t!("kit-files-taken")),
        _ => e,
    };
    let meta = fs::symlink_metadata(from)?;
    #[cfg(any(test, feature = "testing"))]
    meanwhile();
    if meta.is_dir() {
        fs::DirBuilder::new().mode(0o700).create(to).map_err(taken)?;
        return fs::rename(from, to).inspect_err(|_| {
            let _ = fs::remove_dir(to);
        });
    }
    let linked = fs::hard_link(from, to);
    #[cfg(any(test, feature = "testing"))]
    if linked.is_ok() {
        LINKED.with_borrow(|then| {
            if let Some(then) = then {
                then();
            }
        });
    }
    match linked {
        Ok(()) => drop_name(from, Which::of(&meta)).inspect_err(|_| {
            if fs::symlink_metadata(to).is_ok_and(|now| Which::of(&now) == Which::of(&meta)) {
                let _ = fs::remove_file(to);
            }
        }),
        Err(e) if e.kind() == ErrorKind::AlreadyExists => Err(taken(e)),
        Err(_) => {
            let place = fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(to).map_err(taken)?;
            let ours = place.metadata().map(|m| Which::of(&m))?;
            let still = || fs::symlink_metadata(to).is_ok_and(|now| Which::of(&now) == ours);
            if !still() {
                return Err(Error::new(ErrorKind::AlreadyExists, t!("kit-files-taken")));
            }
            fs::rename(from, to).inspect_err(|_| {
                if still() {
                    let _ = fs::remove_file(to);
                }
            })
        }
    }
}

fn drop_name(at: &Path, was: Which) -> std::io::Result<()> {
    let (Some(folder), Some(name)) = (at.parent(), at.file_name()) else {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, t!("kit-files-not-a-name")));
    };
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let mut own = std::ffi::OsString::from(".");
    own.push(name);
    own.push(format!(".{}-{}.trindle-moved", std::process::id(), NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)));
    let aside = folder.join(own);
    if fs::symlink_metadata(&aside).is_ok() {
        return Err(std::io::Error::new(std::io::ErrorKind::AlreadyExists, t!("kit-files-old-name-stays")));
    }
    match fs::rename(at, &aside) {
        Ok(()) => {}
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(e),
    }
    let now = fs::symlink_metadata(&aside);
    if now.as_ref().is_ok_and(|now| Which::of(now) == was) {
        return fs::remove_file(&aside);
    }
    if fs::hard_link(&aside, at).is_ok() {
        let _ = fs::remove_file(&aside);
    } else if now.is_ok_and(|now| now.is_dir()) {
        let _ = fs::rename(&aside, at);
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Which {
    dev: u64,
    ino: u64,
}

impl Which {
    pub const TOO_BIG: Which = Which { dev: u64::MAX, ino: u64::MAX };

    pub fn of(meta: &fs::Metadata) -> Which {
        Which { dev: meta.dev(), ino: meta.ino() }
    }
}

fn changed_why() -> &'static str {
    t!("kit-files-changed")
}
fn not_a_file_why() -> &'static str {
    t!("kit-files-not-a-file")
}

#[derive(Debug)]
struct Reason(&'static str);

impl std::fmt::Display for Reason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}

impl std::error::Error for Reason {}

fn has_reason(e: &std::io::Error, kind: std::io::ErrorKind, reason: &str) -> bool {
    e.kind() == kind && e.get_ref().and_then(|why| why.downcast_ref::<Reason>()).is_some_and(|r| r.0 == reason)
}

pub fn changed() -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, Reason(changed_why()))
}

pub fn is_changed(e: &std::io::Error) -> bool {
    has_reason(e, std::io::ErrorKind::InvalidData, changed_why())
}

fn not_a_file() -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::Unsupported, Reason(not_a_file_why()))
}

pub fn is_not_a_file(e: &std::io::Error) -> bool {
    has_reason(e, std::io::ErrorKind::Unsupported, not_a_file_why())
}

#[derive(Debug)]
pub enum Named {
    File(fs::File, fs::Metadata),
    Folder(fs::File, fs::Metadata),
    Link(fs::File),
}

pub fn open_named(path: &Path, within: Option<&fs::File>, want: Option<Which>) -> std::io::Result<Named> {
    let (named, meta) = match (within, path.file_name()) {
        (Some(up), Some(name)) => open_in(up, name, want)?,
        (Some(_), None) => return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, t!("kit-files-no-name-opened"))),
        _ => open_same(path, want)?,
    };
    let kind = meta.file_type();
    if kind.is_file() {
        Ok(Named::File(named, meta))
    } else if kind.is_dir() {
        Ok(Named::Folder(named, meta))
    } else if kind.is_symlink() {
        Ok(Named::Link(named))
    } else {
        Err(not_a_file())
    }
}

pub fn open_same(path: &Path, want: Option<Which>) -> std::io::Result<(fs::File, fs::Metadata)> {
    let named = fs::OpenOptions::new().read(true).custom_flags(libc::O_PATH | libc::O_NOFOLLOW).open(path)?;
    let meta = named.metadata()?;
    if want.is_some_and(|w| w != Which::of(&meta)) {
        return Err(changed());
    }
    Ok((named, meta))
}

pub fn is_one_name(name: &[u8]) -> bool {
    !(name.is_empty() || name == b"." || name == b".." || name.contains(&b'/') || name.contains(&0))
}

pub fn open_in(dir: &fs::File, name: &std::ffi::OsStr, want: Option<Which>) -> std::io::Result<(fs::File, fs::Metadata)> {
    use std::io::{Error, ErrorKind};
    let bytes = name.as_bytes();
    if !is_one_name(bytes) {
        return Err(Error::new(ErrorKind::InvalidInput, t!("kit-files-not-plain")));
    }
    let c = std::ffi::CString::new(bytes).map_err(|_| Error::new(ErrorKind::InvalidInput, t!("kit-files-nought")))?;
    let flags = (libc::O_PATH | libc::O_NOFOLLOW | libc::O_CLOEXEC) as u64;
    let resolve = libc::RESOLVE_BENEATH | libc::RESOLVE_NO_SYMLINKS | libc::RESOLVE_NO_MAGICLINKS;
    let named = match crate::sys::os::open_beneath(dir.as_raw_fd(), &c, flags, resolve) {
        Ok(named) => named,
        Err(e) => {
            return match e.raw_os_error() {
                Some(libc::ENOSYS | libc::EPERM | libc::EAGAIN) => open_same(&through(dir).join(name), want),
                _ => Err(e),
            }
        }
    };
    let meta = named.metadata()?;
    if want.is_some_and(|w| w != Which::of(&meta)) {
        return Err(changed());
    }
    Ok((named, meta))
}

pub fn through(named: &fs::File) -> PathBuf {
    PathBuf::from(format!("/proc/self/fd/{}", named.as_raw_fd()))
}

pub fn readable(named: &fs::File) -> std::io::Result<fs::File> {
    fs::File::open(through(named))
}

pub fn link_of(named: &fs::File) -> std::io::Result<PathBuf> {
    use std::os::unix::ffi::OsStringExt;
    Ok(PathBuf::from(std::ffi::OsString::from_vec(crate::sys::os::link_of(named.as_raw_fd())?)))
}

#[cfg(any(test, feature = "testing"))]
thread_local! {
    pub static FULL: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    pub static NO_REFUSE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    pub static MEANWHILE: std::cell::RefCell<Option<Box<dyn Fn()>>> = const { std::cell::RefCell::new(None) };
    pub static LINKED: std::cell::RefCell<Option<Box<dyn Fn()>>> = const { std::cell::RefCell::new(None) };
    pub static SPARE_MODE: std::cell::Cell<Option<u32>> = const { std::cell::Cell::new(None) };
    pub static SPARE_NEXT: std::cell::Cell<Option<u64>> = const { std::cell::Cell::new(None) };
}

#[cfg(any(test, feature = "testing"))]
fn meanwhile() {
    MEANWHILE.with_borrow(|then| {
        if let Some(then) = then {
            then();
        }
    });
}

pub fn unread_lines(path: &Path, seen: &Path, trim_at: u64, keep: usize) -> Vec<String> {
    let read = open_settings(path).and_then(|mut file| {
        let mut bytes = Vec::new();
        std::io::Read::read_to_end(&mut file, &mut bytes).map(|_| bytes)
    });
    let bytes = match read {
        Ok(bytes) => bytes,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Vec::new(),
        Err(_) => return Vec::new(),
    };
    let text = String::from_utf8_lossy(&bytes);
    let lines: Vec<&str> = text.lines().collect();
    let from = match read_text(seen).trim().parse::<usize>().ok() {
        Some(n) if n <= lines.len() => n,
        Some(_) => 0,
        None => lines.len(),
    };
    let new = lines[from..].iter().map(|l| l.to_string()).collect();
    let mut kept = lines.len();
    if fs::metadata(path).is_ok_and(|m| m.len() > trim_at) {
        let last = &lines[lines.len().saturating_sub(keep)..];
        if !last.is_empty() && write_text(path, &(last.join("\n") + "\n")).is_ok() {
            kept = last.len();
        }
    }
    if write_whole(seen, kept.to_string().as_bytes()).is_err() {
        return Vec::new();
    }
    new
}

fn parse_line(line: &str) -> Option<(&str, &str)> {
    let t = line.trim_start();
    if t.starts_with('#') {
        return None;
    }
    let (k, v) = t.split_once('=')?;
    let k = k.trim();
    if k.is_empty() || !k.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
        || k.chars().next().is_some_and(|c| c.is_ascii_digit())
    {
        return None;
    }
    Some((k, v.trim()))
}

pub fn read_keys(text: &str) -> BTreeMap<String, String> {
    text.lines().filter_map(parse_line).map(|(k, v)| (k.to_string(), v.to_string())).collect()
}

pub fn read_env(path: &Path) -> BTreeMap<String, String> {
    read_keys(&read_text(path))
}

pub fn update_env(path: &Path, changes: &[(&str, Option<&str>)]) -> std::io::Result<()> {
    use std::io::{Error, ErrorKind};
    for (key, value) in changes {
        if parse_line(&format!("{key}=")).is_none_or(|(k, _)| k != *key) {
            return Err(Error::new(ErrorKind::InvalidInput, t!("kit-files-name-unkeepable")));
        }
        if value.is_some_and(|v| !fits_a_line(v)) {
            return Err(Error::new(ErrorKind::InvalidInput, t!("kit-files-char-unkeepable")));
        }
    }
    locked(path, || {
        let text = read_small(path)?;
        let mut done: Vec<&str> = Vec::new();
        let mut out: Vec<String> = Vec::new();
        for line in text.lines() {
            if let Some((k, _)) = parse_line(line) {
                if let Some((key, value)) = changes.iter().find(|(key, _)| *key == k) {
                    if done.contains(key) {
                        continue;
                    }
                    done.push(key);
                    if let Some(v) = value {
                        out.push(format!("{key}={v}"));
                    }
                    continue;
                }
            }
            out.push(line.to_string());
        }
        for (key, value) in changes {
            if !done.contains(key) {
                if let Some(v) = value {
                    out.push(format!("{key}={v}"));
                }
            }
        }
        let mut joined = out.join("\n").trim_end_matches('\n').to_string();
        joined.push('\n');
        write_text(path, &joined)
    })
}

fn locked<T>(path: &Path, then: impl FnOnce() -> T) -> T {
    let dir = path.parent().filter(|d| !d.as_os_str().is_empty()).unwrap_or(Path::new("."));
    let name = path.file_name().unwrap_or_default().to_string_lossy();
    let lock = fs::create_dir_all(dir)
        .and_then(|_| fs::OpenOptions::new().read(true).mode(0o600).custom_flags(libc::O_CREAT).open(dir.join(format!(".{name}.lock"))));
    if let Ok(lock) = &lock {
        crate::sys::fd::lock(lock.as_raw_fd());
    }
    then()
}

pub fn trindle_conf() -> PathBuf {
    config_path("trindle.conf")
}

pub fn saved(key: &str) -> String {
    read_env(&trindle_conf()).get(key).cloned().unwrap_or_default()
}

#[cfg(any(test, feature = "testing"))]
#[cfg_attr(not(test), allow(dead_code, unused_imports))]
pub mod tests;
