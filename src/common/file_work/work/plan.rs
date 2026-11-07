//! What a piece of work is: what is asked for, how it is steered, what it says as it goes and what it comes to.

use std::collections::HashSet;
use std::fs::File;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use super::said::{not_kept, trouble};
use super::side::{Plain, Side};
use super::Which;
use crate::common::file_work::{proof, trash};

pub const TELL_GAP: Duration = Duration::from_millis(100);
pub const NAP: Duration = Duration::from_millis(50);
pub const REPORT_GAP: Duration = Duration::from_millis(250);

pub fn rated<S: Side>(tell: impl Fn(News<S>)) -> impl Fn(News<S>) {
    rated_by(tell, REPORT_GAP)
}

pub(super) fn rated_by<S: Side>(tell: impl Fn(News<S>), gap: Duration) -> impl Fn(News<S>) {
    let last: Mutex<[Option<std::time::Instant>; 2]> = Mutex::new([None; 2]);
    move |news: News<S>| {
        let k = match news {
            News::Done(_) => 0,
            News::Files(..) => 1,
            _ => return tell(news),
        };
        let now = std::time::Instant::now();
        if let Ok(mut last) = last.lock() {
            if last[k].is_some_and(|t| now.duration_since(t) < gap) {
                return;
            }
            last[k] = Some(now);
        }
        tell(news);
    }
}
#[derive(Clone, Debug)]
pub enum Kind<S: Side = Plain> {
    Copy,
    Move,
    Trash,
    Delete,
    Restore,
    Undo(Vec<Step<S>>),
    Other(S::Job),
}

pub struct Plan<S: Side = Plain> {
    pub kind: Kind<S>,
    pub items: Vec<PathBuf>,
    pub to: PathBuf,
    pub key: Option<S::Key>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Answer {
    Replace,
    Skip,
    Both,
    Merge,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Clash {
    pub name: String,
    pub folders: bool,
    pub sides: Option<(PathBuf, PathBuf)>,
    pub replace: bool,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Same {
    pub files: Option<Answer>,
    pub folders: Option<Answer>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Step<S: Side = Plain> {
    Remove(PathBuf),
    Back { now: PathBuf, was: PathBuf },
    Restore(PathBuf),
    Retrash { now: PathBuf, was: PathBuf, note: String },
    Other(S::Undo),
}

#[derive(Clone, Debug)]
pub enum News<S: Side = Plain> {
    Total(u64),
    Done(u64),
    Files(u64, u64),
    Clash(Clash),
    Over(Over<S>),
}

#[derive(Clone, Debug)]
pub struct Over<S: Side = Plain> {
    pub undo: Vec<Step<S>>,
    pub failed: Vec<String>,
    pub skipped: usize,
    pub stopped: bool,
    pub not_kept: Vec<String>,
    pub locked: Vec<(PathBuf, S::Locked)>,
}

impl<S: Side> Default for Over<S> {
    fn default() -> Over<S> {
        Over { undo: Vec::new(), failed: Vec::new(), skipped: 0, stopped: false, not_kept: Vec::new(), locked: Vec::new() }
    }
}

impl<S: Side> Over<S> {
    pub fn troubled(&self) -> bool {
        !self.failed.is_empty() || !self.not_kept.is_empty()
    }

    pub fn troubles(&self) -> Vec<String> {
        self.not_kept.iter().map(|folder| trouble(folder, not_kept())).chain(self.failed.iter().cloned()).collect()
    }
}

#[derive(Default)]
pub struct Ctl<S: Side = Plain> {
    pub pause: Arc<AtomicBool>,
    pub stop: Arc<AtomicBool>,
    pub answers: Option<Receiver<(Answer, bool)>>,
    pub fixed: Option<Answer>,
    pub bin: Option<trash::Bin>,
    pub links: S::Links,
    pub sure: Vec<Sure>,
    pub handed: Option<Vec<proof::Handed>>,
    pub asked: Vec<proof::Asked>,
}

#[derive(Clone, Debug)]
pub struct Sure {
    pub path: PathBuf,
    pub which: Which,
    pub held: Option<Arc<File>>,
    pub known: Option<Arc<HashSet<Which>>>,
}
