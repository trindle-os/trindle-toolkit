//! Going through what was asked for: each thing placed, asked about when its name is taken, and brought by a rename or queued to be copied.

use std::collections::HashSet;
use std::ffi::{OsStr, OsString};
use std::fs::{self, DirBuilder, File};
use std::io;
use std::os::unix::fs::{DirBuilderExt, MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;

use super::copy::Queued;
use super::leave::Moved;
use super::looks::{ask, came_since, free_name, goes_into, listed, open_folder_from, spare_name, within};
use super::plan::{Answer, Clash, Ctl, Kind, News, Over, Plan, Same, Step, NAP};
use super::said::{changed_since, changed_why, not_all, not_looked, not_there_why, no_safe_restore, taken_meanwhile, too_big_to_check, trouble};
use super::side::Side;
use super::undo::step_made;
use super::Which;
use crate::common::file_work::proof::Copied;
use crate::common::file_work::trash;
use crate::common::files::{changed, is_changed, link_of, open_named, rename_new, Named};
use crate::t;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Mode {
    Copy,
    Move,
    Restore,
    Undo,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Again {
    Move,
    Trash,
}

pub(super) struct Walk<'a, S: Side> {
    pub(super) ctl: &'a Ctl<S>,
    pub(super) tell: &'a dyn Fn(News<S>),
    pub(super) mode: Mode,
    pub(super) again: Again,
    same: Same,
    pub(super) over: Over<S>,
    pub(super) queue: Vec<Queued>,
    pub(super) after: Vec<(PathBuf, Moved)>,
    broke: usize,
    pub(super) empties: Vec<PathBuf>,
    pub(super) modes: Vec<(PathBuf, u32)>,
    made: HashSet<(u64, u64)>,
    pub(super) untrashed: Vec<PathBuf>,
    pub(super) ways: Vec<Way>,
}

pub(super) struct Way {
    pub(super) src: PathBuf,
    pub(super) landing: PathBuf,
    pub(super) dest: PathBuf,
    pub(super) old: Option<Which>,
    pub(super) whole: bool,
}

type From<'a> = (Option<&'a File>, Option<&'a HashSet<Which>>);

pub(super) fn taken_or_not(e: &io::Error) -> Option<&'static str> {
    (e.kind() == io::ErrorKind::AlreadyExists).then_some(taken_meanwhile())
}

impl<'a, S: Side> Walk<'a, S> {
    pub(super) fn new(ctl: &'a Ctl<S>, tell: &'a dyn Fn(News<S>), mode: Mode) -> Walk<'a, S> {
        Walk { ctl, tell, mode, again: Again::Move, same: Same::default(), over: Over::default(), queue: Vec::new(), after: Vec::new(), broke: 0,
               empties: Vec::new(), modes: Vec::new(), made: HashSet::new(), untrashed: Vec::new(), ways: Vec::new() }
    }

    pub(super) fn go_through(&mut self, plan: &Plan<S>) {
        match &plan.kind {
            Kind::Undo(steps) => {
                self.same = Same { files: Some(Answer::Both), folders: Some(Answer::Merge) };
                for step in steps.iter().rev() {
                    if self.stopped() {
                        break;
                    }
                    self.undo_step(step);
                }
            }
            Kind::Restore => self.restore(&plan.items),
            _ => {
                for item in &plan.items {
                    if self.stopped() {
                        break;
                    }
                    self.item(item, &plan.to);
                }
            }
        }
    }

    pub(super) fn restore_beneath(&mut self, item: &Path, top: &Path, to: &Path) {
        match trash::move_into_drive(top, item, to) {
            Ok(landed) => self.record(item, &landed),
            Err(e) => match beneath_reason(&e) {
                Some(why) => self.fail_why(item, why),
                None => self.fail(item),
            },
        }
    }

    fn restore(&mut self, items: &[PathBuf]) {
        for item in items {
            if self.stopped() {
                break;
            }
            match trash::origin(item) {
                Some(to) => {
                    self.untrashed.push(item.clone());
                    match trash::drive_top(item) {
                        Some(top) => self.restore_beneath(item, &top, &to),
                        None => {
                            if let Some(up) = to.parent() {
                                let _ = fs::create_dir_all(up);
                            }
                            self.place(item, &to);
                        }
                    }
                }
                None => self.fail(item),
            }
        }
    }

    fn stopped(&self) -> bool {
        while self.ctl.pause.load(Ordering::Relaxed) && !self.ctl.stop.load(Ordering::Relaxed) {
            std::thread::sleep(NAP);
        }
        self.ctl.stop.load(Ordering::Relaxed)
    }

    pub(super) fn fail(&mut self, path: &Path) {
        self.broke += 1;
        self.over.failed.push(S::name_of(path));
    }

    pub(super) fn fail_why(&mut self, path: &Path, why: &str) {
        if ![not_looked(), too_big_to_check(), changed_since()].contains(&why) {
            self.broke += 1;
        }
        self.over.failed.push(trouble(&S::name_of(path), why));
    }

    pub(super) fn fail_maybe(&mut self, path: &Path, why: Option<&str>) {
        match why {
            Some(why) => self.fail_why(path, why),
            None => self.fail(path),
        }
    }

    fn record(&mut self, from: &Path, to: &Path) {
        let moved = Step::Back { now: to.to_path_buf(), was: from.to_path_buf() };
        let untrashed = |from: &Path| match trash::note(from) {
            Some(note) => Step::Retrash { now: to.to_path_buf(), was: from.to_path_buf(), note },
            None => Step::Remove(to.to_path_buf()),
        };
        self.over.undo.push(match (self.mode, self.again) {
            (Mode::Copy, _) => Step::Remove(to.to_path_buf()),
            (Mode::Restore, _) | (Mode::Undo, Again::Trash) => untrashed(from),
            (Mode::Move, _) | (Mode::Undo, Again::Move) => moved,
        });
    }

    fn item(&mut self, src: &Path, dir: &Path) {
        let Some(name) = src.file_name() else { return self.fail(src) };
        if goes_into(dir, src) {
            return self.fail(src);
        }
        let mut dest = dir.join(name);
        if src.parent() == Some(dir) {
            if self.mode != Mode::Copy {
                self.over.skipped += 1;
                return;
            }
            dest = free_name(dir, name, src.is_dir());
        }
        self.place(src, &dest);
    }

    pub(super) fn place(&mut self, src: &Path, dest: &Path) {
        self.place_seen(src, dest, None, None, None);
    }

    fn place_seen(&mut self, src: &Path, dest: &Path, seen: Option<fs::Metadata>, up: Option<&File>, strict: Option<&HashSet<Which>>) {
        if self.stopped() {
            return;
        }
        let Some(meta) = seen.or_else(|| fs::symlink_metadata(src).ok()) else { return self.fail(src) };
        let ctl = self.ctl;
        let sure = ctl.sure.iter().find(|s| s.path == src);
        if sure.is_some_and(|s| s.which != Which::of(&meta)) {
            return self.fail_why(src, changed_since());
        }
        let strict = strict.or(sure.and_then(|s| s.known.as_deref()));
        let Some((dest, way)) = self.clear_name(src, dest, &meta, (up, strict)) else { return };
        let spare = match (dest.parent(), dest.file_name()) {
            (Some(dir), Some(name)) => dir.join(spare_name(&name.to_string_lossy())),
            _ => return self.fail(src),
        };
        if self.mode == Mode::Copy {
            return self.bring(src, &dest, &spare, way, &meta, (up, strict), false);
        }
        self.move_to(src, &dest, &spare, way, &meta, (up, strict));
    }

    fn clear_name(&mut self, src: &Path, dest: &Path, meta: &fs::Metadata, from: From) -> Option<(PathBuf, Option<Which>)> {
        let Ok(there) = fs::symlink_metadata(dest) else { return Some((dest.to_path_buf(), None)) };
        let folders = there.is_dir() && meta.is_dir();
        let clash = Clash { name: S::name_of(dest), folders, sides: Some((src.to_path_buf(), dest.to_path_buf())), replace: true };
        match self.ask(clash)? {
            Answer::Merge if folders => {
                self.merge(src, dest, meta, from.0, from.1);
                None
            }
            Answer::Skip | Answer::Merge => {
                self.over.skipped += 1;
                None
            }
            Answer::Both => {
                let (Some(dir), Some(name)) = (dest.parent(), dest.file_name()) else {
                    self.fail(src);
                    return None;
                };
                Some((free_name(dir, name, meta.is_dir()), None))
            }
            Answer::Replace => {
                if within(src, dest) {
                    self.fail_why(src, t!("kit-work-inside-replaced"));
                    return None;
                }
                Some((dest.to_path_buf(), Some(Which::of(&there))))
            }
        }
    }

    fn move_to(&mut self, src: &Path, dest: &Path, spare: &Path, way: Option<Which>, meta: &fs::Metadata, from: From) {
        let landing = if way.is_some() { spare } else { dest };
        match rename_new(src, landing) {
            Ok(()) => {
                if self.moved_in_place(src, dest, landing, way, meta) {
                    self.record(src, dest);
                }
            }
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => self.fail_why(src, taken_meanwhile()),
            Err(e) if e.raw_os_error() == Some(libc::EXDEV) => self.bring(src, dest, spare, way, meta, from, true),
            Err(_) => self.fail(src),
        }
    }

    fn moved_in_place(&mut self, src: &Path, dest: &Path, landing: &Path, way: Option<Which>, meta: &fs::Metadata) -> bool {
        let Some(old) = way else { return true };
        let (bin, failed) = match self.make_way(dest, Some(old)) {
            Ok(bin) => (bin, rename_new(landing, dest).err().map(|e| taken_or_not(&e))),
            Err(why) => (None, Some(Some(why))),
        };
        self.trashed(bin, None);
        let Some(why) = failed else { return true };
        if rename_new(landing, src).is_err() {
            if let (Some(dir), Some(name)) = (dest.parent(), dest.file_name()) {
                let _ = rename_new(landing, &free_name(dir, name, meta.is_dir()));
            }
        }
        self.fail_maybe(src, why);
        false
    }

    fn bring(&mut self, src: &Path, dest: &Path, landing: &Path, way: Option<Which>, meta: &fs::Metadata, from: From, moving: bool) {
        let said = self.broke;
        let copied = self.copy_tree(src, landing, meta, from);
        let whole = copied.is_some() && self.broke == said && !self.ctl.stop.load(Ordering::Relaxed);
        self.ways.push(Way { src: src.to_path_buf(), landing: landing.to_path_buf(), dest: dest.to_path_buf(), old: way, whole });
        if let Some(copied) = copied {
            if moving {
                self.after.push((src.to_path_buf(), copied));
            }
            self.record(src, dest);
        }
    }

    pub(super) fn trashed(&mut self, bin: Option<PathBuf>, new: Option<&Path>) {
        let Some(bin) = bin else { return };
        if self.mode == Mode::Undo {
            return;
        }
        let at = new.and_then(|new| self.over.undo.iter().rposition(|s| step_made(s) == Some(new))).unwrap_or(self.over.undo.len());
        self.over.undo.insert(at, Step::Restore(bin));
    }

    fn merge(&mut self, src: &Path, dest: &Path, meta: &fs::Metadata, up: Option<&File>, strict: Option<&HashSet<Which>>) {
        let held = match open_folder_from(src, up, meta) {
            Ok(held) => held,
            Err(e) if is_changed(&e) => return self.fail_why(src, changed_why()),
            Err(_) => return self.fail(src),
        };
        let Ok(children) = listed(&held) else { return self.fail(src) };
        for (name, seen) in children.things {
            match seen {
                Ok(seen) if came_since(&name, &seen, strict) => self.fail_why(&src.join(&name), not_there_why(strict)),
                Ok(seen) => self.place_seen(&src.join(&name), &dest.join(&name), Some(seen), Some(&held), strict),
                Err(_) => self.fail(&src.join(&name)),
            }
        }
        if !children.whole {
            return self.fail_why(src, not_all());
        }
        if self.mode != Mode::Copy {
            self.empties.push(src.to_path_buf());
        }
    }

    fn ask(&mut self, clash: Clash) -> Option<Answer> {
        ask(self.ctl, self.tell, &mut self.same, clash)
    }

    fn copy_tree(&mut self, src: &Path, dest: &Path, meta: &fs::Metadata, from: From) -> Option<Moved> {
        let kind = meta.file_type();
        let name = src.file_name().map(OsStr::to_os_string).unwrap_or_default();
        if kind.is_symlink() {
            return self.copy_link(src, dest, meta, from.0, name);
        }
        if kind.is_file() {
            self.queue.push((src.to_path_buf(), dest.to_path_buf(), meta.len(), Which::of(meta)));
            return Some(Moved::File(name));
        }
        if !kind.is_dir() {
            self.fail(src);
            return None;
        }
        self.copy_folder(src, dest, meta, from, name)
    }

    fn copy_link(&mut self, src: &Path, dest: &Path, meta: &fs::Metadata, up: Option<&File>, name: OsString) -> Option<Moved> {
        let made = match open_named(src, up, Some(Which::of(meta))) {
            Ok(Named::Link(link)) => link_of(&link).and_then(|to| std::os::unix::fs::symlink(&to, dest).map(|()| to)),
            Ok(_) => Err(changed()),
            Err(e) => Err(e),
        };
        let Some(proof) = made.ok().and_then(|to| Copied::of_link(meta, &to, dest)) else {
            self.fail(src);
            return None;
        };
        Some(Moved::Link(name, proof))
    }

    fn copy_folder(&mut self, src: &Path, dest: &Path, meta: &fs::Metadata, (up, strict): From, name: OsString) -> Option<Moved> {
        if self.made.contains(&(meta.dev(), meta.ino())) {
            self.fail_why(src, t!("kit-work-copy-made-it"));
            return None;
        }
        let held = match open_folder_from(src, up, meta) {
            Ok(held) => held,
            Err(e) => {
                if is_changed(&e) { self.fail_why(src, changed_why()) } else { self.fail(src) }
                return None;
            }
        };
        if DirBuilder::new().mode(0o700).create(dest).is_err() {
            self.fail(src);
            return None;
        }
        if let Ok(m) = fs::symlink_metadata(dest) {
            self.made.insert((m.dev(), m.ino()));
        }
        self.modes.push((dest.to_path_buf(), meta.permissions().mode()));
        let which = Which::of(meta);
        let Ok(children) = listed(&held) else {
            self.fail(src);
            return Some(Moved::Folder { name, which, all: false, inside: Vec::new() });
        };
        let (mut all, mut inside) = (children.whole, Vec::new());
        if !all {
            self.fail_why(src, not_all());
        }
        for (child, seen) in children.things {
            if self.stopped() {
                all = false;
                break;
            }
            match seen {
                Ok(m) if came_since(&child, &m, strict) => self.fail_why(&src.join(&child), not_there_why(strict)),
                Ok(m) => inside.extend(self.copy_tree(&src.join(&child), &dest.join(&child), &m, (Some(&held), strict))),
                Err(_) => self.fail(&src.join(&child)),
            }
        }
        Some(Moved::Folder { name, which, all, inside })
    }
}

fn beneath_reason(e: &io::Error) -> Option<&'static str> {
    match e.raw_os_error() {
        _ if e.kind() == io::ErrorKind::AlreadyExists => Some(taken_meanwhile()),
        Some(libc::EINVAL) => Some(no_safe_restore()),
        _ => None,
    }
}

#[cfg(test)]
mod beneath_reason_tests {
    use super::*;

    #[test]
    fn a_drive_that_cannot_keep_a_name_says_so_and_a_taken_name_says_that() {
        assert_eq!(beneath_reason(&io::Error::from_raw_os_error(libc::EINVAL)), Some(no_safe_restore()));
        assert!(no_safe_restore().contains("copy the file out of the Trash"));
        assert_eq!(beneath_reason(&io::Error::from(io::ErrorKind::AlreadyExists)), Some(taken_meanwhile()));
        assert_eq!(beneath_reason(&io::Error::from(io::ErrorKind::PermissionDenied)), None);
    }
}
