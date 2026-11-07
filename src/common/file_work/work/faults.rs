//! Faults a test can ask for, since a drive cannot be made to fail or be filled without being root: each is set for one path in a test's own folder, and is met at the very call that would fail.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Fault {
    List,
    ListCut,
    Read,
    Full,
    Swapped,
}

pub const SWAPPED: &[u8] = b"written by another program since";

static SET: Mutex<Vec<(Fault, PathBuf)>> = Mutex::new(Vec::new());

pub fn set(fault: Fault, path: &Path) {
    SET.lock().unwrap().push((fault, path.to_path_buf()));
}

pub fn at(path: &Path) -> Option<Fault> {
    let mut set = SET.lock().unwrap();
    let path = crate::common::files::unspare(path);
    let i = set.iter().position(|(_, p)| *p == path)?;
    let fault = set[i].0;
    if matches!(fault, Fault::List | Fault::ListCut) {
        set.remove(i);
    }
    Some(fault)
}

pub fn meanwhile(path: &Path) {
    let mut set = SET.lock().unwrap();
    if let Some(i) = set.iter().position(|(f, p)| *f == Fault::Swapped && p == path) {
        set.remove(i);
        if std::fs::symlink_metadata(path).is_ok_and(|m| m.is_dir()) {
            let mut away = path.as_os_str().to_owned();
            away.push(".away");
            std::fs::rename(path, &away).unwrap();
            std::fs::create_dir(path).unwrap();
            std::fs::write(path.join("kept.txt"), SWAPPED).unwrap();
            return;
        }
        std::fs::remove_file(path).unwrap();
        std::fs::write(path, SWAPPED).unwrap();
    }
}

pub fn of(folder: &std::fs::File) -> Option<Fault> {
    at(&std::fs::read_link(crate::common::files::through(folder)).ok()?)
}
