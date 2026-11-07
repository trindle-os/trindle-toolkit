//! Home and XDG folders, and a private file written whole and put in place in one step.

use std::io::Write;
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};

pub fn home() -> Option<PathBuf> {
    absolute(std::env::var_os("HOME"))
}

fn absolute(value: Option<std::ffi::OsString>) -> Option<PathBuf> {
    value.map(PathBuf::from).filter(|p| p.is_absolute())
}

fn xdg(var: &str, under_home: &str) -> Option<PathBuf> {
    absolute(std::env::var_os(var)).or_else(|| home().map(|h| h.join(under_home)))
}

pub fn config_home() -> Option<PathBuf> {
    xdg("XDG_CONFIG_HOME", ".config")
}

pub fn data_home() -> Option<PathBuf> {
    xdg("XDG_DATA_HOME", ".local/share")
}

pub fn cache_home() -> Option<PathBuf> {
    xdg("XDG_CACHE_HOME", ".cache")
}

pub fn runtime_dir() -> Option<PathBuf> {
    absolute(std::env::var_os("XDG_RUNTIME_DIR"))
}

pub fn private_runtime_dir() -> std::io::Result<PathBuf> {
    use std::io::{Error, ErrorKind};
    use std::os::unix::fs::MetadataExt;
    let dir = runtime_dir().ok_or_else(|| Error::new(ErrorKind::NotFound, "XDG_RUNTIME_DIR is not set to a full path"))?;
    let (was, me) = (std::fs::metadata(&dir)?, std::fs::metadata("/proc/self")?);
    if !was.is_dir() || was.uid() != me.uid() || was.mode() & 0o077 != 0 {
        return Err(Error::new(ErrorKind::PermissionDenied, "XDG_RUNTIME_DIR is not a folder for this person alone"));
    }
    Ok(dir)
}

pub fn fits_a_line(value: &str) -> bool {
    !value.chars().any(char::is_control) && value.trim() == value
}

pub const SMALL_MOST: u64 = 1 << 20;

pub fn read_small(path: &Path) -> std::io::Result<String> {
    use std::io::{Error, ErrorKind, Read};
    let file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(String::new()),
        Err(e) => return Err(e),
    };
    let mut bytes = Vec::new();
    file.take(SMALL_MOST + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > SMALL_MOST {
        return Err(Error::new(ErrorKind::InvalidData, "the file is too big to be a settings file"));
    }
    String::from_utf8(bytes).map_err(|_| Error::new(ErrorKind::InvalidData, "the file is not text"))
}

pub fn write_private(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let dir = path.parent().filter(|d| !d.as_os_str().is_empty()).unwrap_or(Path::new("."));
    let name = path.file_name().ok_or_else(|| std::io::Error::from(std::io::ErrorKind::InvalidInput))?;
    clear_stale(dir, name);
    let (spare, mut file) = open_spare(dir, name)?;
    let done = file.write_all(bytes).and_then(|()| file.set_permissions(std::fs::Permissions::from_mode(0o600)))
        .and_then(|()| file.sync_all()).and_then(|()| std::fs::rename(&spare, path));
    if let Err(e) = done {
        let _ = std::fs::remove_file(&spare);
        return Err(e);
    }
    let _ = std::fs::File::open(dir).and_then(|d| d.sync_all());
    Ok(())
}

const STALE: std::time::Duration = std::time::Duration::from_secs(600);

fn clear_stale(dir: &Path, name: &std::ffi::OsStr) {
    let mut start = std::ffi::OsString::from(".");
    start.push(name);
    start.push(".");
    let start = start.as_encoded_bytes();
    let Ok(read) = std::fs::read_dir(dir) else { return };
    for entry in read.flatten() {
        let found = entry.file_name();
        let found = found.as_encoded_bytes();
        let old = entry.metadata().and_then(|m| m.modified()).ok().and_then(|t| t.elapsed().ok()).is_some_and(|age| age > STALE);
        if found.starts_with(start) && found.ends_with(b".part") && old {
            let _ = std::fs::remove_file(entry.path());
        }
    }
}

fn open_spare(dir: &Path, name: &std::ffi::OsStr) -> std::io::Result<(PathBuf, std::fs::File)> {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNT: AtomicU64 = AtomicU64::new(0);
    let mut last = std::io::Error::from(std::io::ErrorKind::AlreadyExists);
    for _ in 0..16 {
        let mut part = std::ffi::OsString::from(".");
        part.push(name);
        part.push(format!(".{}-{}.part", std::process::id(), COUNT.fetch_add(1, Ordering::Relaxed)));
        let spare = dir.join(part);
        match std::fs::OpenOptions::new().write(true).create_new(true).mode(0o600).custom_flags(libc::O_NOFOLLOW).open(&spare) {
            Ok(file) => return Ok((spare, file)),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => last = e,
            Err(e) => return Err(e),
        }
    }
    Err(last)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::MetadataExt;

    fn folder(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("trindle-common-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn a_spare_file_a_crash_left_is_removed_at_the_next_write() {
        let dir = folder("stale");
        let path = dir.join("settings.ini");
        let old = dir.join(".settings.ini.1-0.part");
        let fresh = dir.join(".settings.ini.2-0.part");
        let other = dir.join(".other.ini.1-0.part");
        for f in [&old, &fresh, &other] {
            std::fs::write(f, "x").unwrap();
        }
        let long_ago = std::time::SystemTime::now() - 2 * STALE;
        for f in [&old, &other] {
            std::fs::File::options().write(true).open(f).unwrap().set_modified(long_ago).unwrap();
        }
        write_private(&path, b"new").unwrap();
        assert!(!old.exists(), "a stale spare file was kept");
        assert!(fresh.exists(), "a spare file that may be in use was removed");
        assert!(other.exists(), "another file's spare was removed");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_runtime_folder_open_to_others_is_refused() {
        let dir = folder("runtime");
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o755)).unwrap();
        std::env::set_var("XDG_RUNTIME_DIR", &dir);
        assert!(private_runtime_dir().is_err());
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700)).unwrap();
        assert_eq!(private_runtime_dir().unwrap(), dir);
        std::env::set_var("XDG_RUNTIME_DIR", "relative");
        assert!(private_runtime_dir().is_err());
        std::env::remove_var("XDG_RUNTIME_DIR");
        assert!(private_runtime_dir().is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_relative_folder_is_ignored() {
        assert_eq!(absolute(Some("config".into())), None);
        assert_eq!(absolute(Some("".into())), None);
        assert_eq!(absolute(Some("/data".into())), Some(PathBuf::from("/data")));
        assert_eq!(absolute(None), None);
    }

    #[test]
    fn a_private_file_is_put_in_place_whole_and_for_this_person_alone() {
        let dir = folder("write");
        let path = dir.join("settings.ini");
        std::fs::write(&path, "old old old").unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();
        write_private(&path, b"new").unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), b"new");
        assert_eq!(std::fs::metadata(&path).unwrap().mode() & 0o777, 0o600);
        let left: Vec<_> = std::fs::read_dir(&dir).unwrap().flatten().collect();
        assert_eq!(left.len(), 1, "a spare file was left behind");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_link_is_replaced_and_never_written_through() {
        let dir = folder("link");
        let target = dir.join("target");
        std::fs::write(&target, "keep").unwrap();
        let path = dir.join("settings.ini");
        std::os::unix::fs::symlink(&target, &path).unwrap();
        write_private(&path, b"new").unwrap();
        assert_eq!(std::fs::read(&target).unwrap(), b"keep");
        assert_eq!(std::fs::read(&path).unwrap(), b"new");
        assert!(!std::fs::symlink_metadata(&path).unwrap().file_type().is_symlink());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_failed_write_leaves_the_old_file_and_no_spare() {
        let dir = folder("fail");
        let path = dir.join("taken");
        std::fs::create_dir(&path).unwrap();
        assert!(write_private(&path, b"x").is_err());
        assert_eq!(std::fs::read_dir(&dir).unwrap().flatten().count(), 1);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_missing_file_reads_as_empty_and_an_unreadable_one_is_an_error() {
        let dir = folder("read");
        assert_eq!(read_small(&dir.join("none")).unwrap(), "");
        std::fs::write(dir.join("binary"), [0xff, 0xfe]).unwrap();
        assert!(read_small(&dir.join("binary")).is_err());
        assert!(read_small(&dir).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_value_that_would_not_stay_on_its_line_is_refused() {
        assert!(fits_a_line("plain value"));
        assert!(!fits_a_line("two\nlines"));
        assert!(!fits_a_line(" padded"));
    }
}
