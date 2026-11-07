//! Looking at places: what is in an opened folder, what lies inside what, and which name is free.

use std::collections::HashSet;
use std::ffi::{OsStr, OsString};
use std::fs::{self, File};
use std::io;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;

#[cfg(any(test, feature = "testing"))]
use super::faults;
use super::plan::{Answer, Clash, Ctl, News, Same};
use super::side::Side;
use super::{free_in, through, Which};
use crate::common::files::{changed, open_named, Named, SPARE_END};

pub const NAME_MOST: usize = 255;

#[cfg(test)]
pub(super) fn open_folder(path: &Path, meta: &fs::Metadata) -> io::Result<File> {
    open_folder_from(path, None, meta)
}

pub(super) fn open_folder_from(path: &Path, up: Option<&File>, meta: &fs::Metadata) -> io::Result<File> {
    match open_named(path, up, Some(Which::of(meta)))? {
        Named::Folder(held, _) => Ok(held),
        _ => Err(changed()),
    }
}

pub fn listed(folder: &File) -> io::Result<Listed> {
    #[cfg(any(test, feature = "testing"))]
    let cut = faults::of(folder);
    #[cfg(any(test, feature = "testing"))]
    if cut == Some(faults::Fault::List) {
        return Err(io::Error::from_raw_os_error(libc::EIO));
    }
    let mut out = Listed { things: Vec::new(), whole: true };
    for child in fs::read_dir(through(folder))? {
        let Ok(child) = child else {
            out.whole = false;
            break;
        };
        out.things.push((child.file_name(), child.metadata()));
        #[cfg(any(test, feature = "testing"))]
        if cut == Some(faults::Fault::ListCut) {
            out.whole = false;
            break;
        }
    }
    Ok(out)
}

pub struct Listed {
    pub things: Vec<(OsString, io::Result<fs::Metadata>)>,
    pub whole: bool,
}
pub fn came_since(name: &OsStr, meta: &fs::Metadata, known: Option<&HashSet<Which>>) -> bool {
    let Some(known) = known else { return false };
    (name.as_bytes().first() == Some(&b'.') || meta.file_type().is_symlink()) && !known.contains(&Which::of(meta))
}

pub fn real_of(path: &Path) -> PathBuf {
    let linked = fs::symlink_metadata(path).is_ok_and(|m| m.file_type().is_symlink());
    match (linked, path.parent(), path.file_name()) {
        (true, Some(up), Some(name)) => fs::canonicalize(up).map(|up| up.join(name)).unwrap_or_else(|_| path.to_path_buf()),
        _ => fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf()),
    }
}

pub fn within(inner: &Path, outer: &Path) -> bool {
    inner.starts_with(outer) || real_of(inner).starts_with(real_of(outer))
}

pub fn goes_into(dir: &Path, thing: &Path) -> bool {
    dir.starts_with(thing) || fs::canonicalize(dir).is_ok_and(|real| real.starts_with(real_of(thing)))
}
pub fn once_each(items: Vec<PathBuf>) -> Vec<PathBuf> {
    let kept: Vec<PathBuf> = items.iter().enumerate()
        .filter(|(i, item)| !items.iter().enumerate().any(|(j, other)| j != *i && item.starts_with(other) && (*item != other || j < *i)))
        .map(|(_, item)| item.clone())
        .collect();
    kept
}
pub fn free_name(dir: &Path, name: &OsStr, folder: bool) -> PathBuf {
    dir.join(free_in(name, folder, |n| fs::symlink_metadata(dir.join(n)).is_ok()))
}

pub fn ask<S: Side>(ctl: &Ctl<S>, tell: &dyn Fn(News<S>), same: &mut Same, clash: Clash) -> Option<Answer> {
    let all = if clash.folders { &mut same.folders } else { &mut same.files };
    if let Some(a) = *all {
        return Some(a);
    }
    let Some(answers) = &ctl.answers else {
        return Some(match (clash.folders, ctl.fixed) {
            (true, None | Some(Answer::Replace)) => Answer::Merge,
            (_, fixed) => fixed.unwrap_or(Answer::Skip),
        });
    };
    tell(News::Clash(clash));
    let (answer, rest) = answers.recv().ok()?;
    if ctl.stop.load(Ordering::Relaxed) {
        return None;
    }
    if rest {
        *all = Some(answer);
    }
    Some(answer)
}

pub fn spare_name(name: &str) -> String {
    let tag = crate::common::file_work::proof::tag();
    if name.len() + tag.len() + SPARE_END.len() + 1 > NAME_MOST { format!("{tag}{SPARE_END}") } else { format!("{name}.{tag}{SPARE_END}") }
}

pub fn name_on_disk(path: &Path) -> String {
    path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| path.to_string_lossy().into_owned())
}
