//! The Trash and deleting for good, an item at a time.

use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;

use super::plan::{Ctl, Kind, News, Over, Plan, Step, NAP};
use super::said::{not_looked, trouble};
use super::side::Side;
use crate::common::file_work::{proof, trash};
use crate::common::files::{changed, is_changed};

impl<S: Side> Ctl<S> {
    pub(super) fn put(&self, path: &Path) -> io::Result<PathBuf> {
        to_trash(self.bin.as_ref(), path)
    }
}

pub fn to_trash(bin: Option<&trash::Bin>, path: &Path) -> io::Result<PathBuf> {
    match bin {
        Some(b) => trash::put_in(b, path, &trash::now_text()),
        None => trash::put(path),
    }
}

pub(super) fn bin<S: Side>(plan: &Plan<S>, ctl: &Ctl<S>, tell: &dyn Fn(News<S>)) -> Over<S> {
    let mut over = Over::default();
    tell(News::Total(plan.items.len() as u64));
    for (i, item) in plan.items.iter().enumerate() {
        while ctl.pause.load(Ordering::Relaxed) && !ctl.stop.load(Ordering::Relaxed) {
            std::thread::sleep(NAP);
        }
        if ctl.stop.load(Ordering::Relaxed) {
            over.stopped = true;
            break;
        }
        if let Some(handed) = &ctl.handed {
            let went = match (&plan.kind, handed.iter().find(|h| h.path() == item)) {
                (Kind::Trash, Some(proof)) => proof.into_trash(|path| ctl.put(path)),
                _ => Err((proof::not_as_picked(), None)),
            };
            match went {
                Ok(t) => over.undo.push(Step::Restore(t)),
                Err((why, lies)) => {
                    over.failed.push(trouble(&S::name_of(item), why));
                    over.undo.extend(lies.map(Step::Restore));
                }
            }
        } else if let Kind::Trash = plan.kind {
            match ctl.put(item) {
                Ok(t) => over.undo.push(Step::Restore(t)),
                Err(_) => over.failed.push(S::name_of(item)),
            }
        } else {
            let gone = match ctl.asked.iter().find(|a| a.path() == item) {
                Some(asked) => asked.delete(),
                None => Err(changed()),
            };
            match gone {
                Ok(()) => trash::forget(item),
                Err(e) if is_changed(&e) => over.failed.push(trouble(&S::name_of(item), not_looked())),
                Err(_) => over.failed.push(S::name_of(item)),
            }
        }
        tell(News::Done(i as u64 + 1));
    }
    over
}
