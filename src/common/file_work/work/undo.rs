//! Taking a step back: each kind of step, and what does it again.

use std::fs;
use std::path::{Path, PathBuf};

use super::plan::Step;
use super::side::Side;
use super::walk::{Again, Walk};
use crate::common::file_work::trash;

pub(super) fn step_made<S: Side>(step: &Step<S>) -> Option<&Path> {
    match step {
        Step::Remove(p) | Step::Back { now: p, .. } | Step::Retrash { now: p, .. } => Some(p),
        Step::Restore(_) | Step::Other(_) => None,
    }
}

impl<S: Side> Walk<'_, S> {
    pub(super) fn undo_step(&mut self, step: &Step<S>) {
        match step {
            Step::Remove(p) => self.undo_made(p),
            Step::Other(done) => match S::undo(done) {
                Ok(back) => self.over.undo.push(Step::Other(back)),
                Err(failed) => {
                    if let Some(now) = failed {
                        self.fail(&now);
                    }
                }
            },
            Step::Back { now, was } => self.undo_moved(now, was),
            Step::Retrash { now, was, note } => self.undo_put_back(now, was, note),
            Step::Restore(t) => self.undo_trashed(t),
        }
    }

    fn undo_made(&mut self, p: &Path) {
        if fs::symlink_metadata(p).is_err() {
            return;
        }
        match self.ctl.put(p) {
            Ok(t) => self.over.undo.push(Step::Restore(t)),
            Err(_) => self.fail(p),
        }
    }

    fn undo_moved(&mut self, now: &Path, was: &Path) {
        if fs::symlink_metadata(now).is_err() {
            return;
        }
        if let Some(up) = was.parent() {
            let _ = fs::create_dir_all(up);
        }
        self.again = Again::Move;
        self.place(now, was);
    }

    fn undo_put_back(&mut self, now: &Path, was: &PathBuf, note: &str) {
        if fs::symlink_metadata(now).is_err() {
            return;
        }
        let free = fs::symlink_metadata(was).is_err() && trash::note(was).is_none();
        if free && crate::common::files::rename_new(now, was).is_ok() {
            match trash::renote(was, note) {
                Ok(()) => self.over.undo.push(Step::Restore(was.clone())),
                Err(_) => self.fail(was),
            }
        } else {
            match self.ctl.put(now) {
                Ok(t) => self.over.undo.push(Step::Restore(t)),
                Err(_) => self.fail(now),
            }
        }
    }

    fn undo_trashed(&mut self, t: &PathBuf) {
        if fs::symlink_metadata(t).is_err() {
            return;
        }
        match trash::origin(t) {
            Some(to) => {
                self.untrashed.push(t.clone());
                self.again = Again::Trash;
                self.place(t, &to);
            }
            None => self.fail(t),
        }
    }
}
