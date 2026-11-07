//! Every look at a place that the thread drawing a window makes outside the reading of a folder (`reading.rs`) goes through here. A share, an NFS export or a far place that has stopped answering holds a thread that asks it anything for as long as that lasts, and the thread that draws a window is not allowed to be that thread. So a look is made on a thread of its own and waited for no more than `ANSWER_WITHIN`; a place that has not answered is left alone for `SILENT_FOR`, so a column of rows asking one after another costs one wait and not one each. Whatever did not answer is treated as not known, never as not there, and nothing is changed, moved or deleted because a place was silent: the asker goes without, or says `not_answering()` where the person is working.

use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use crate::common::run::Slots;
use crate::t;

pub fn not_answering() -> &'static str {
    t!("kit-reach-not-answering")
}

pub const ANSWER_WITHIN: Duration = Duration::from_millis(if cfg!(any(test, feature = "testing")) { 1000 } else { 400 });

pub const SILENT_FOR: Duration = Duration::from_millis(if cfg!(any(test, feature = "testing")) { 600 } else { 5000 });

static OUT: Slots = Slots::new(8);

static SILENT: Mutex<Vec<(PathBuf, Instant)>> = Mutex::new(Vec::new());

#[cfg(any(test, feature = "testing"))]
static NEVER: Mutex<Vec<PathBuf>> = Mutex::new(Vec::new());

#[cfg(any(test, feature = "testing"))]
pub fn never(place: &Path, on: bool) {
    let mut all = NEVER.lock().unwrap_or_else(|e| e.into_inner());
    all.retain(|p| p != place);
    if on {
        all.push(place.to_path_buf());
    }
    SILENT.lock().unwrap_or_else(|e| e.into_inner()).clear();
}

#[cfg(any(test, feature = "testing"))]
fn hold(at: &Path) {
    while NEVER.lock().unwrap_or_else(|e| e.into_inner()).iter().any(|p| at.starts_with(p)) {
        std::thread::sleep(Duration::from_millis(5));
    }
}

fn place_of(at: &Path) -> &Path {
    at.parent().unwrap_or(at)
}

pub fn silent(at: &Path) -> bool {
    let mut all = SILENT.lock().unwrap_or_else(|e| e.into_inner());
    all.retain(|(_, when)| when.elapsed() < SILENT_FOR);
    all.iter().any(|(place, _)| at.starts_with(place))
}

pub fn within<T: Send + 'static>(at: &Path, look: impl FnOnce() -> T + Send + 'static) -> Option<T> {
    if silent(at) {
        return None;
    }
    let slot = OUT.take()?;
    let (send, hear) = mpsc::channel();
    #[cfg(any(test, feature = "testing"))]
    let held = at.to_path_buf();
    let spawned = std::thread::Builder::new().name("file-reach".into()).spawn(move || {
        let _slot = slot;
        #[cfg(any(test, feature = "testing"))]
        hold(&held);
        let _ = send.send(look());
    });
    if spawned.is_err() {
        return None;
    }
    match hear.recv_timeout(ANSWER_WITHIN) {
        Ok(got) => Some(got),
        Err(_) => {
            SILENT.lock().unwrap_or_else(|e| e.into_inner()).push((place_of(at).to_path_buf(), Instant::now()));
            None
        }
    }
}

pub fn meta(at: &Path, follow: bool) -> Option<std::fs::Metadata> {
    let path = at.to_path_buf();
    within(at, move || if follow { std::fs::metadata(&path) } else { std::fs::symlink_metadata(&path) }).and_then(Result::ok)
}

pub fn is_dir(at: &Path) -> bool {
    meta(at, true).is_some_and(|m| m.is_dir())
}

pub fn answers(at: &Path) -> bool {
    let path = at.to_path_buf();
    within(at, move || std::fs::symlink_metadata(&path).is_ok()).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn made(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("trindle-reach-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("inner")).unwrap();
        std::fs::write(dir.join("a.txt"), "made up").unwrap();
        dir
    }

    fn soon(answered: impl Fn() -> bool) -> bool {
        let end = Instant::now() + Duration::from_secs(20);
        while Instant::now() < end {
            if answered() {
                return true;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        false
    }

    #[test]
    fn a_place_that_never_answers_costs_one_wait_and_changes_nothing() {
        let (live, dead) = (made("live"), made("dead"));
        assert!(soon(|| is_dir(&live.join("inner")) && meta(&live.join("a.txt"), true).is_some_and(|m| m.is_file())));

        never(&dead, true);
        let began = Instant::now();
        assert!(!is_dir(&dead.join("inner")), "a place that did not answer was known to be a folder");
        assert!(began.elapsed() < ANSWER_WITHIN * 4, "the first look waited much longer than the limit");

        let began = Instant::now();
        for _ in 0..50 {
            assert!(meta(&dead.join("a.txt"), false).is_none() && !answers(&dead.join("inner")));
        }
        assert!(began.elapsed() < ANSWER_WITHIN * 10, "fifty more looks at a silent place each waited");

        assert!(soon(|| is_dir(&live.join("inner")) && answers(&live)));

        never(&dead, false);
        assert!(dead.join("a.txt").is_file() && dead.join("inner").is_dir());
        assert!(soon(|| is_dir(&dead.join("inner"))), "a place that answers again was still left alone");
        assert_eq!(std::fs::read_to_string(dead.join("a.txt")).unwrap(), "made up");
    }

    #[test]
    fn a_silent_place_is_asked_again_in_time() {
        let dead = made("again");
        never(&dead, true);
        assert!(!answers(&dead.join("inner")));
        assert!(silent(&dead.join("a.txt")));
        never(&dead, false);
        SILENT.lock().unwrap().push((dead.clone(), Instant::now()));
        assert!(!is_dir(&dead.join("inner")));
        std::thread::sleep(SILENT_FOR + Duration::from_millis(50));
        assert!(soon(|| is_dir(&dead.join("inner"))));
    }
}
