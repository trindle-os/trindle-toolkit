//! The Trash, as the freedesktop.org trash spec has it: each item moved to a `files` folder, with a note in `info` saying where it came from and when. Items on another drive go to that drive's own Trash, so nothing has to be copied across.

use std::ffi::OsString;
use std::fs::{self, DirBuilder, OpenOptions};
use std::io::{self, Write};
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::os::unix::fs::{DirBuilderExt, MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};

use super::address::{percent, unpercent};
use crate::common::files::data_home;
use crate::sys::disk;
use crate::t;

#[derive(Clone, Debug)]
pub struct Bin {
    pub files: PathBuf,
    pub info: PathBuf,
    pub top: Option<PathBuf>,
}

impl Bin {
    pub fn at(dir: &Path, top: Option<PathBuf>) -> Bin {
        Bin { files: dir.join("files"), info: dir.join("info"), top }
    }

    pub fn ready(&self) -> io::Result<()> {
        DirBuilder::new().recursive(true).mode(0o700).create(&self.files)?;
        DirBuilder::new().recursive(true).mode(0o700).create(&self.info)
    }
}

pub fn home_bin() -> Bin {
    Bin::at(&data_home().join("Trash"), None)
}

pub fn all_files() -> Vec<PathBuf> {
    let mounts = fs::read_to_string("/proc/self/mounts").unwrap_or_default();
    drive_files(&mounts, disk::uid(), home_bin().files)
}

pub fn mount_point(field: &str) -> PathBuf {
    PathBuf::from(field.replace("\\040", " ").replace("\\011", "\t").replace("\\134", "\\"))
}

fn drive_files(mounts: &str, id: u32, home: PathBuf) -> Vec<PathBuf> {
    let own = |d: &Path| fs::symlink_metadata(d).is_ok_and(|m| m.is_dir() && m.uid() == id);
    let sticky = |d: &Path| fs::symlink_metadata(d).is_ok_and(|m| m.is_dir() && m.permissions().mode() & 0o1000 != 0);
    let mut out = vec![home];
    for top in mounts.lines().filter_map(|l| l.split(' ').nth(1)).map(mount_point) {
        let mine = top.join(format!(".Trash-{id}"));
        let shared = top.join(".Trash");
        let found = if own(&mine) {
            Some(mine)
        } else if sticky(&shared) && own(&shared.join(id.to_string())) {
            Some(shared.join(id.to_string()))
        } else {
            None
        };
        if let Some(files) = found.map(|d| d.join("files")).filter(|f| own(f) && !out.contains(f)) {
            out.push(files);
        }
    }
    out
}

fn device(path: &Path) -> Option<u64> {
    path.ancestors().find_map(|p| fs::symlink_metadata(p).ok()).map(|m| m.dev())
}

fn mount_top(path: &Path) -> PathBuf {
    let Some(dev) = device(path) else { return PathBuf::from("/") };
    let mut top = path.to_path_buf();
    while let Some(up) = top.parent() {
        if fs::metadata(up).map(|m| m.dev()).ok() != Some(dev) {
            break;
        }
        top = up.to_path_buf();
    }
    top
}

pub fn bin_for(path: &Path) -> io::Result<Bin> {
    let home = home_bin();
    let here = fs::symlink_metadata(path)?.dev();
    if device(&home.files) == Some(here) {
        home.ready()?;
        return Ok(home);
    }
    drive_bin(mount_top(path.parent().unwrap_or(path)), disk::uid())
}

fn drive_bin(top: PathBuf, id: u32) -> io::Result<Bin> {
    let shared = top.join(".Trash");
    if let Ok(m) = fs::symlink_metadata(&shared) {
        if m.is_dir() && m.permissions().mode() & 0o1000 != 0 {
            let theirs = shared.join(id.to_string());
            if made_and_theirs(&theirs, id).unwrap_or(false) {
                let bin = Bin::at(&theirs, Some(top.clone()));
                if bin.ready().is_ok() {
                    return Ok(bin);
                }
            }
        }
    }
    let own = top.join(format!(".Trash-{id}"));
    if !made_and_theirs(&own, id)? {
        return Err(io::Error::other(t!("kit-trash-drive-unsafe")));
    }
    let bin = Bin::at(&own, Some(top));
    bin.ready()?;
    Ok(bin)
}

fn made_and_theirs(dir: &Path, id: u32) -> io::Result<bool> {
    match DirBuilder::new().mode(0o700).create(dir) {
        Err(e) if e.kind() != io::ErrorKind::AlreadyExists => return Err(e),
        _ => {}
    }
    let m = fs::symlink_metadata(dir)?;
    Ok(m.is_dir() && m.uid() == id)
}

pub fn put(path: &Path) -> io::Result<PathBuf> {
    put_in(&bin_for(path)?, path, &now_text())
}

pub fn put_in(bin: &Bin, path: &Path, when: &str) -> io::Result<PathBuf> {
    let name = path.file_name().ok_or_else(|| io::Error::other(t!("kit-trash-nothing-to-move")))?;
    let full = if path.is_absolute() { path.to_path_buf() } else { std::env::current_dir()?.join(path) };
    let written = match &bin.top {
        Some(top) => full.strip_prefix(top).map(Path::to_path_buf).unwrap_or(full.clone()),
        None => full.clone(),
    };
    let note = format!("[Trash Info]\nPath={}\nDeletionDate={when}\n", percent(&written));
    for n in 1..10_000u32 {
        let cand = if n == 1 { name.to_os_string() } else { numbered(name, n) };
        let mut info_name = cand.clone();
        info_name.push(".trashinfo");
        let info = bin.info.join(&info_name);
        let into = bin.files.join(&cand);
        let mut file = match OpenOptions::new().write(true).create_new(true).open(&info) {
            Ok(f) => f,
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(e),
        };
        if fs::symlink_metadata(&into).is_ok() {
            drop(file);
            let _ = fs::remove_file(&info);
            continue;
        }
        let done = file.write_all(note.as_bytes()).and_then(|_| crate::common::files::rename_new(&full, &into));
        if let Err(e) = done {
            let _ = fs::remove_file(&info);
            return Err(e);
        }
        return Ok(into);
    }
    Err(io::Error::other(t!("kit-trash-too-many-names")))
}

fn numbered(name: &std::ffi::OsStr, n: u32) -> OsString {
    let b = name.as_bytes();
    let dot = b.iter().rposition(|&c| c == b'.').filter(|&i| i > 0);
    let mut out = Vec::new();
    match dot {
        Some(i) => {
            out.extend_from_slice(&b[..i]);
            out.extend_from_slice(format!(".{n}").as_bytes());
            out.extend_from_slice(&b[i..]);
        }
        None => {
            out.extend_from_slice(b);
            out.extend_from_slice(format!(".{n}").as_bytes());
        }
    }
    OsString::from_vec(out)
}

pub fn now_text() -> String {
    let tm = disk::local_time();
    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}", tm.tm_year + 1900, tm.tm_mon + 1, tm.tm_mday, tm.tm_hour, tm.tm_min, tm.tm_sec)
}

fn parts(trashed: &Path) -> Option<(PathBuf, Option<PathBuf>)> {
    let files = trashed.parent()?;
    if files.file_name()? != "files" {
        return None;
    }
    let dir = files.parent()?;
    let mut info_name = trashed.file_name()?.to_os_string();
    info_name.push(".trashinfo");
    let info = dir.join("info").join(info_name);
    let name = dir.file_name()?.as_bytes();
    let top = if name.starts_with(b".Trash-") {
        dir.parent().map(Path::to_path_buf)
    } else if dir.parent().and_then(Path::file_name).is_some_and(|n| n == ".Trash") {
        dir.parent().and_then(Path::parent).map(Path::to_path_buf)
    } else {
        None
    };
    Some((info, top))
}

pub fn origin(trashed: &Path) -> Option<PathBuf> {
    let (info, top) = parts(trashed)?;
    let text = crate::common::files::read_string(info).ok()?;
    let line = text.lines().find_map(|l| l.strip_prefix("Path="))?;
    let path = unpercent(line.trim());
    match top {
        Some(t) => {
            let rel = if path.is_absolute() { path.strip_prefix(&t).ok()?.to_path_buf() } else { path };
            contained(&t, &rel)
        }
        None => path.is_absolute().then_some(path),
    }
}

fn contained(top: &Path, rel: &Path) -> Option<PathBuf> {
    let mut at = top.to_path_buf();
    for c in rel.components() {
        if !matches!(c, std::path::Component::Normal(_)) {
            return None;
        }
        at.push(c);
        if fs::symlink_metadata(&at).is_ok_and(|m| m.file_type().is_symlink()) {
            return None;
        }
    }
    Some(at)
}

pub fn note(trashed: &Path) -> Option<String> {
    crate::common::files::read_string(parts(trashed)?.0).ok()
}

pub fn renote(trashed: &Path, text: &str) -> io::Result<()> {
    let (info, _) = parts(trashed).ok_or_else(|| io::Error::from(io::ErrorKind::InvalidInput))?;
    OpenOptions::new().write(true).create_new(true).open(info)?.write_all(text.as_bytes())
}

pub fn forget(trashed: &Path) {
    if let Some((info, _)) = parts(trashed) {
        let _ = fs::remove_file(info);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn drive_with_shared_trash(name: &str) -> (PathBuf, PathBuf) {
        let top = std::env::temp_dir().join(format!("trindle-trash-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&top);
        let shared = top.join(".Trash");
        fs::create_dir_all(&shared).unwrap();
        fs::set_permissions(&shared, fs::Permissions::from_mode(0o1777)).unwrap();
        (top, shared)
    }

    #[test]
    fn a_shared_trash_gives_each_person_a_private_folder() {
        let (top, shared) = drive_with_shared_trash("private");
        let id = disk::uid();
        let bin = drive_bin(top.clone(), id).unwrap();
        let theirs = shared.join(id.to_string());
        assert_eq!(bin.files, theirs.join("files"));
        assert_eq!(fs::symlink_metadata(&theirs).unwrap().permissions().mode() & 0o777, 0o700);
        let _ = fs::remove_dir_all(&top);
    }

    #[test]
    fn a_link_laid_in_a_shared_trash_is_not_followed() {
        let (top, shared) = drive_with_shared_trash("link");
        let id = disk::uid();
        let elsewhere = top.join("someone-elses");
        fs::create_dir_all(&elsewhere).unwrap();
        std::os::unix::fs::symlink(&elsewhere, shared.join(id.to_string())).unwrap();
        let bin = drive_bin(top.clone(), id).unwrap();
        assert_eq!(bin.files, top.join(format!(".Trash-{id}")).join("files"), "the drive's other Trash is used");
        assert_eq!(fs::read_dir(&elsewhere).unwrap().count(), 0, "nothing was made through the link");
        let _ = fs::remove_dir_all(&top);
    }

    #[test]
    fn a_folder_someone_else_made_in_a_shared_trash_is_not_used() {
        let (top, shared) = drive_with_shared_trash("owner");
        let id = disk::uid() + 1;
        let laid = shared.join(id.to_string());
        fs::create_dir_all(&laid).unwrap();
        fs::create_dir_all(top.join(format!(".Trash-{id}"))).unwrap();
        assert!(drive_bin(top.clone(), id).is_err(), "neither folder is theirs");
        assert_eq!(fs::read_dir(&laid).unwrap().count(), 0, "nothing was made in it");
        let _ = fs::remove_dir_all(&top);
    }

    #[test]
    fn trashed_items_carry_a_note_and_come_back() {
        let dir = std::env::temp_dir().join(format!("trindle-files-trash-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let bin = Bin::at(&dir.join("Trash"), None);
        bin.ready().unwrap();
        let first = dir.join("my notes.txt");
        fs::write(&first, b"one").unwrap();
        let went = put_in(&bin, &first, "2026-10-03T12:00:00").unwrap();
        assert_eq!(went, bin.files.join("my notes.txt"));
        let note = fs::read_to_string(bin.info.join("my notes.txt.trashinfo")).unwrap();
        assert_eq!(note, format!("[Trash Info]\nPath={}\nDeletionDate=2026-10-03T12:00:00\n", percent(&first)));
        assert!(note.contains("my%20notes.txt"));
        fs::write(&first, b"two").unwrap();
        let again = put_in(&bin, &first, "2026-10-03T12:00:01").unwrap();
        assert_eq!(again, bin.files.join("my notes.2.txt"));
        assert_eq!(origin(&again), Some(first.clone()));
        forget(&again);
        assert!(!bin.info.join("my notes.2.txt.trashinfo").exists());
        let drive = Bin::at(&dir.join(".Trash-1000"), Some(dir.clone()));
        drive.ready().unwrap();
        let third = dir.join("inner");
        fs::create_dir_all(third.join("deep")).unwrap();
        let went = put_in(&drive, &third, "2026-10-03T12:00:02").unwrap();
        assert!(fs::read_to_string(drive.info.join("inner.trashinfo")).unwrap().contains("\nPath=inner\n"));
        assert_eq!(origin(&went), Some(third.clone()));
        let under = format!("Path={}\n", percent(&third));
        fs::write(drive.info.join("under.trashinfo"), format!("[Trash Info]\n{under}")).unwrap();
        assert_eq!(origin(&drive.files.join("under")), Some(third.clone()), "an absolute path under the drive is taken");
        let outside = dir.join("outside");
        fs::create_dir_all(&outside).unwrap();
        std::os::unix::fs::symlink(&outside, dir.join("link")).unwrap();
        fs::write(drive.info.join("linked.trashinfo"), "[Trash Info]\nPath=link/evil.desktop\n").unwrap();
        assert_eq!(origin(&drive.files.join("linked")), None, "a link on the way is refused");
        for path in ["/etc/passwd", "../outside", "inner/../../outside"] {
            fs::write(drive.info.join("bad.trashinfo"), format!("[Trash Info]\nPath={path}\n")).unwrap();
            assert_eq!(origin(&drive.files.join("bad")), None, "{path} is refused");
        }
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn every_drives_trash_is_found_from_the_mounts() {
        let dir = std::env::temp_dir().join(format!("trindle-trash-drives-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let id = disk::uid();
        fs::create_dir_all(dir.join(format!("stick/.Trash-{id}/files"))).unwrap();
        fs::create_dir_all(dir.join(format!("disk/.Trash/{id}/files"))).unwrap();
        fs::set_permissions(dir.join("disk/.Trash"), fs::Permissions::from_mode(0o1777)).unwrap();
        fs::create_dir_all(dir.join(format!("open/.Trash/{id}/files"))).unwrap();
        fs::create_dir_all(dir.join("bare")).unwrap();
        let line = |at: &str| format!("/dev/x {} ext4 rw 0 0\n", dir.join(at).display().to_string().replace(' ', "\\040"));
        let mounts: String = ["stick", "disk", "open", "bare", "stick"].iter().map(|a| line(a)).collect();
        let home = dir.join("home/Trash/files");
        assert_eq!(drive_files(&mounts, id, home.clone()), [
            home, dir.join(format!("stick/.Trash-{id}/files")), dir.join(format!("disk/.Trash/{id}/files")),
        ]);
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn the_date_is_written_as_the_spec_asks() {
        let t = now_text();
        assert_eq!(t.len(), 19);
        assert_eq!(&t[4..5], "-");
        assert_eq!(&t[10..11], "T");
    }
}
