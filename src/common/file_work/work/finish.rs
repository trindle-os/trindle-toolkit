//! The end of a piece of work: the copies made, each put in its place, what was moved taken away, and all of it out to its drive.

use std::collections::HashMap;
use std::fs::{self, File};
use std::io;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;

use super::copy::{pool, threads};
use super::leave::{leave_top, Moved};
use super::plan::{News, Over, Step};
use super::said::{changed_why, no_trash, stayed, taken_meanwhile, trouble};
use super::side::Side;
use super::undo::step_made;
use super::walk::{taken_or_not, Walk};
use super::Which;
use crate::common::file_work::proof::{Copied, Counts};
use crate::common::file_work::{gentle, trash};
use crate::common::files::rename_new;

fn unmake_spare(landing: &Path) {
    let _ = match fs::symlink_metadata(landing) {
        Ok(m) if m.is_dir() => fs::remove_dir_all(landing),
        Ok(_) => fs::remove_file(landing),
        Err(_) => Ok(()),
    };
}

impl<S: Side> Walk<'_, S> {
    pub(super) fn finish(mut self, ends: &[&Path]) -> Over<S> {
        let total: u64 = self.queue.iter().map(|q| q.2).sum();
        (self.tell)(News::Total(total));
        let drives = self.drives_written_to();
        let (failed, sealed) = pool(std::mem::take(&mut self.queue), threads(ends), self.ctl, self.tell, !self.after.is_empty() || !self.ways.is_empty());
        self.over.failed.extend(failed.iter().map(|(p, changed)| if *changed { trouble(&S::name_of(p), changed_why()) } else { S::name_of(p) }));
        let stopped = self.ctl.stop.load(Ordering::Relaxed);
        let stay = self.settle(&failed, stopped);
        self.after.retain(|(src, _)| !stay.contains(src));
        if !stopped {
            self.take_away_firsts(&sealed);
        }
        for dir in &self.empties {
            let _ = fs::remove_dir(dir);
        }
        for (dir, mode) in self.modes.iter().rev() {
            let _ = fs::set_permissions(dir, fs::Permissions::from_mode(*mode));
        }
        for t in &self.untrashed {
            if fs::symlink_metadata(t).is_err() {
                trash::forget(t);
            }
        }
        if !stopped {
            self.last_writes(&drives, ends);
        }
        self.over.stopped = stopped;
        self.over.failed.dedup();
        self.over.not_kept.dedup();
        self.over
    }

    fn drives_written_to(&self) -> Vec<(u64, PathBuf, File)> {
        let mut drives: Vec<(u64, PathBuf, File)> = Vec::new();
        for up in self.queue.iter().filter_map(|q| q.1.parent()) {
            if let Ok(m) = fs::metadata(up) {
                if !drives.iter().any(|(dev, ..)| *dev == m.dev()) {
                    if let Ok(dir) = gentle::drive_of(up) {
                        drives.push((m.dev(), up.to_path_buf(), dir));
                    }
                }
            }
        }
        drives
    }

    fn take_away_firsts(&mut self, sealed: &HashMap<PathBuf, Copied>) {
        for (src, copied) in &self.after {
            let left = leave_top(src, copied, sealed);
            let said = match (left, copied) {
                (0, _) => continue,
                (n, Moved::Folder { .. }) => trouble(&S::name_of(src), &Counts { other: n, ..Default::default() }.reason()),
                _ if !sealed.contains_key(src) && matches!(copied, Moved::File(_)) => continue,
                _ => trouble(&S::name_of(src), stayed()),
            };
            self.over.failed.push(said);
        }
    }

    fn last_writes(&mut self, drives: &[(u64, PathBuf, File)], ends: &[&Path]) {
        for (dev, folder, dir) in drives {
            if gentle::all_out_of(dir).is_err() {
                let sent_to = ends.first().filter(|end| fs::metadata(end).is_ok_and(|m| m.dev() == *dev)).map_or(folder.as_path(), |end| *end);
                self.over.not_kept.push(S::name_of(sent_to));
            }
        }
    }

    pub(super) fn make_way(&mut self, dest: &Path, old: Option<Which>) -> Result<Option<PathBuf>, &'static str> {
        let Some(old) = old else { return Ok(None) };
        match fs::symlink_metadata(dest) {
            Ok(now) if Which::of(&now) != old => Err(taken_meanwhile()),
            Ok(_) => self.ctl.put(dest).map(Some).map_err(|_| no_trash()),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(_) => Err(no_trash()),
        }
    }

    fn settle(&mut self, failed: &[(PathBuf, bool)], stopped: bool) -> Vec<PathBuf> {
        let mut stay = Vec::new();
        for way in std::mem::take(&mut self.ways) {
            let whole = way.whole && !stopped && !failed.iter().any(|(p, _)| p.starts_with(&way.src));
            let done = whole && match self.make_way(&way.dest, way.old) {
                Ok(bin) => {
                    let named = rename_new(&way.landing, &way.dest);
                    self.trashed(bin, Some(&way.dest));
                    if let Err(e) = &named {
                        self.fail_maybe(&way.src, taken_or_not(e));
                    }
                    named.is_ok()
                }
                Err(why) => {
                    self.fail_why(&way.src, why);
                    false
                }
            };
            if done {
                for (dir, _) in &mut self.modes {
                    if let Ok(rest) = dir.strip_prefix(&way.landing) {
                        *dir = if rest.as_os_str().is_empty() { way.dest.clone() } else { way.dest.join(rest) };
                    }
                }
                continue;
            }
            if let Some(i) = self.over.undo.iter().rposition(|s: &Step<S>| step_made(s) == Some(way.dest.as_path())) {
                self.over.undo.remove(i);
            }
            self.modes.retain(|(dir, _)| !dir.starts_with(&way.landing));
            unmake_spare(&way.landing);
            stay.push(way.src);
        }
        stay
    }
}
