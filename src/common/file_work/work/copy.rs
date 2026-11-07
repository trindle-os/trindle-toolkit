//! Copying the queued files, on a crew of gentle threads: a chunk at a time, told once written out, and checked where a first is to go.

use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Mutex;

#[cfg(any(test, feature = "testing"))]
use super::faults;
use super::crew::crew;
use super::plan::{Ctl, News, NAP, TELL_GAP};
use super::side::Side;
use super::Which;
use crate::common::file_work::gentle;
use crate::common::file_work::proof::{self, Copied};
use crate::common::files::{changed, is_changed, open_named, readable, Named};
use crate::t;

pub const CHUNK: usize = gentle::CHUNK;

pub(super) type Queued = (PathBuf, PathBuf, u64, Which);

type Waiting = gentle::Held<File, (PathBuf, PathBuf)>;

struct Pouring<'a> {
    pause: &'a AtomicBool,
    stop: &'a AtomicBool,
    check: bool,
    across: AtomicU64,
    copied: AtomicU64,
    failed: Mutex<Vec<(PathBuf, bool)>>,
    sealed: Mutex<HashMap<PathBuf, Copied>>,
    held: Waiting,
}

pub(super) fn pool<S: Side>(queue: Vec<Queued>, n: usize, ctl: &Ctl<S>, tell: &dyn Fn(News<S>), check: bool) -> (Vec<(PathBuf, bool)>, HashMap<PathBuf, Copied>) {
    if queue.is_empty() {
        return (Vec::new(), HashMap::new());
    }
    let pouring = Pouring { pause: &ctl.pause, stop: &ctl.stop, check, across: AtomicU64::new(0), copied: AtomicU64::new(0),
                            failed: Mutex::new(Vec::new()), sealed: Mutex::new(HashMap::new()), held: gentle::Held::default() };
    let mut queue = queue;
    queue.sort_by_key(|q| std::cmp::Reverse(q.2));
    let files = queue.len() as u64;
    crew(n, &|job: Queued| pouring.one(job), |bench| {
        for job in queue {
            let _ = bench.give(job, usize::MAX);
        }
        while !bench.idle() {
            std::thread::sleep(TELL_GAP);
            pouring.say(tell, files);
        }
    });
    let failed = pouring.settled();
    pouring.say(tell, files);
    (failed, pouring.sealed.into_inner().unwrap_or_default())
}

impl Pouring<'_> {
    fn say<S: Side>(&self, tell: &dyn Fn(News<S>), files: u64) {
        tell(News::Files(self.across.load(Ordering::Relaxed), files));
        tell(News::Done(self.copied.load(Ordering::Relaxed)));
    }

    fn one(&self, (src, dest, _, which): Queued) {
        match self.copy_file(&src, &dest, which) {
            Ok(proof) => {
                self.across.fetch_add(1, Ordering::Relaxed);
                if let (Some(proof), Ok(mut s)) = (proof, self.sealed.lock()) {
                    s.insert(src, proof);
                }
            }
            Err(e) if e.kind() == io::ErrorKind::Interrupted => {}
            Err(e) => {
                if let Ok(mut f) = self.failed.lock() {
                    f.push((src, is_changed(&e)));
                }
            }
        }
    }

    fn settled(&self) -> Vec<(PathBuf, bool)> {
        let mut failed = self.failed.lock().map(|mut f| std::mem::take(&mut *f)).unwrap_or_default();
        let lost = self.held.settle(&|n| {
            self.copied.fetch_add(n, Ordering::Relaxed);
        });
        for gentle::Lost { file, whose: (src, dest), .. } in lost {
            proof::unmake(&file, &dest);
            self.across.fetch_sub(1, Ordering::Relaxed);
            failed.push((src, false));
        }
        failed
    }

    fn copy_file(&self, src: &Path, dest: &Path, which: Which) -> io::Result<Option<Copied>> {
        let Named::File(named, meta) = open_named(src, None, Some(which))? else { return Err(changed()) };
        let mut from = gentle::In::new(readable(&named)?);
        let to = OpenOptions::new().read(self.check).write(true).create_new(true).mode(meta.mode() & 0o7777).open(dest)?;
        let made = match to.try_clone() {
            Ok(made) => made,
            Err(e) => {
                proof::unmake(&to, dest);
                return Err(e);
            }
        };
        let reached = |n: u64| {
            self.copied.fetch_add(n, Ordering::Relaxed);
        };
        let mut out = gentle::Out::new(to, &reached);
        let poured = self.pour(&mut from, &mut out, (src, dest), &meta);
        let done = poured.and_then(|(across, small)| {
            if !self.check {
                return out.finish(&self.held, (src.to_path_buf(), dest.to_path_buf())).map(|()| None);
            }
            proved(out.finish_now()?, &from, &meta, across, small).map(Some)
        });
        if done.is_err() {
            proof::unmake(&made, dest);
        }
        done
    }

    fn pour(&self, from: &mut gentle::In, out: &mut gentle::Out<'_, File>, (_src, _dest): (&Path, &Path), meta: &fs::Metadata) -> io::Result<(u64, Option<Vec<u8>>)> {
        let mut buf = vec![0u8; CHUNK];
        let mut across = 0u64;
        let mut small = (self.check && meta.len() <= CHUNK as u64).then(Vec::new);
        loop {
            while self.pause.load(Ordering::Relaxed) && !self.stop.load(Ordering::Relaxed) {
                std::thread::sleep(NAP);
            }
            if self.stop.load(Ordering::Relaxed) {
                return Err(io::Error::from(io::ErrorKind::Interrupted));
            }
            let n = from.read(&mut buf)?;
            if n == 0 {
                break;
            }
            #[cfg(any(test, feature = "testing"))]
            if faults::at(_src) == Some(faults::Fault::Read) && across > 0 {
                return Err(io::Error::from_raw_os_error(libc::EIO));
            }
            #[cfg(any(test, feature = "testing"))]
            if faults::at(_dest) == Some(faults::Fault::Full) {
                return Err(io::Error::from_raw_os_error(libc::ENOSPC));
            }
            out.write_all(&buf[..n])?;
            across += n as u64;
            if small.as_ref().is_some_and(|s| s.len() + n > CHUNK) {
                small = None;
            }
            if let Some(s) = &mut small {
                s.extend_from_slice(&buf[..n]);
            }
        }
        out.file().set_modified(meta.modified()?)?;
        Ok((across, small))
    }
}

#[cfg(test)]
pub(super) fn copy_one(src: &Path, dest: &Path, which: Which) -> io::Result<Option<Copied>> {
    let (pause, stop) = (AtomicBool::new(false), AtomicBool::new(false));
    let pouring = Pouring { pause: &pause, stop: &stop, check: false, across: AtomicU64::new(0), copied: AtomicU64::new(0),
                            failed: Mutex::new(Vec::new()), sealed: Mutex::new(HashMap::new()), held: gentle::Held::default() };
    pouring.copy_file(src, dest, which)
}

fn proved(mut to: File, from: &gentle::In, meta: &fs::Metadata, across: u64, small: Option<Vec<u8>>) -> io::Result<Copied> {
    use std::io::Seek;
    let now = from.file().metadata()?;
    if proof::Seal::of(&now) != proof::Seal::of(meta) || across != meta.len() {
        return Err(changed());
    }
    to.sync_all()?;
    let wrong = || io::Error::other(t!("kit-work-copy-wrong"));
    let proof = Copied::of_file(meta, &now, across, to.metadata().ok().map(|m| m.len())).ok_or_else(wrong)?;
    if let Some(read) = small {
        let mut back = Vec::with_capacity(read.len());
        to.rewind()?;
        to.read_to_end(&mut back)?;
        if back != read {
            return Err(wrong());
        }
    }
    Ok(proof)
}

pub fn threads(ends: &[&Path]) -> usize {
    if ends.iter().any(|p| spinning(p)) { 1 } else { gentle::AT_ONCE }
}

fn spinning(path: &Path) -> bool {
    rotational(path) == Some(true)
}

pub fn rotational(path: &Path) -> Option<bool> {
    let dev = path.ancestors().find_map(|p| fs::metadata(p).ok())?.dev();
    let major = ((dev >> 8) & 0xfff) | ((dev >> 32) & !0xfff);
    let minor = (dev & 0xff) | ((dev >> 12) & !0xff);
    let sys = fs::canonicalize(format!("/sys/dev/block/{major}:{minor}")).ok()?;
    let found = [Some(sys.as_path()), sys.parent()].into_iter().flatten()
        .find_map(|d| crate::common::files::read_string(d.join("queue/rotational")).ok())?;
    Some(found.trim() == "1")
}

