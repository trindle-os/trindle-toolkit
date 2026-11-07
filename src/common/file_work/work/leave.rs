//! The one way a move takes away what it copied.

use std::ffi::{OsStr, OsString};
use std::fs::{self, File, OpenOptions};
use std::io;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};

use super::looks::listed;
use super::{open_in, through, Which};
use crate::common::file_work::proof::{self, Copied};

pub(super) enum Moved {
    File(OsString),
    Link(OsString, Copied),
    Folder { name: OsString, which: Which, all: bool, inside: Vec<Moved> },
}

impl Moved {
    fn name(&self) -> &OsStr {
        match self {
            Moved::File(name) | Moved::Link(name, _) | Moved::Folder { name, .. } => name,
        }
    }
}

fn leave(dir: &File, at: &Path, node: &Moved, copied: &std::collections::HashMap<PathBuf, Copied>) -> usize {
    let there = through(dir).join(node.name());
    let gone = |e: &io::Error| e.kind() == io::ErrorKind::NotFound;
    let one = |proof: Option<&Copied>| match proof {
        Some(proof) => usize::from(!proof::take_away(dir, node.name(), proof)),
        None => usize::from(!fs::symlink_metadata(&there).is_err_and(|e| gone(&e))),
    };
    match node {
        Moved::File(_) => one(copied.get(at)),
        Moved::Link(_, proof) => one(Some(proof)),
        Moved::Folder { all: false, .. } => 1,
        Moved::Folder { which, inside, .. } => {
            let held = match open_in(dir, node.name(), Some(*which)) {
                Ok((held, meta)) if meta.is_dir() => held,
                Err(e) if gone(&e) => return 0,
                _ => return 1,
            };
            let now = match listed(&held) {
                Ok(now) if now.whole => now,
                _ => return 1,
            };
            let mut stayed = 0;
            for (name, _) in &now.things {
                stayed += match inside.iter().find(|child| child.name() == name) {
                    Some(child) => leave(&held, &at.join(name), child, copied),
                    None => 1,
                };
            }
            if stayed == 0 && fs::remove_dir(&there).is_err() {
                stayed = 1;
            }
            stayed
        }
    }
}

pub(super) fn leave_top(src: &Path, node: &Moved, sealed: &std::collections::HashMap<PathBuf, Copied>) -> usize {
    let Some(up) = src.parent() else { return 1 };
    match OpenOptions::new().read(true).custom_flags(libc::O_PATH | libc::O_DIRECTORY).open(up) {
        Ok(dir) => leave(&dir, src, node, sealed),
        Err(_) => 1,
    }
}
