//! Long disk work goes gently: a copy, however big, never makes the computer it runs on slow, and what it says of itself is true.

use std::collections::VecDeque;
use std::fs::File;
use std::io::{self, Read, Write};
use std::sync::Mutex;

use crate::sys::disk;

pub const CHUNK: usize = 1 << 20;

pub const STEP: u64 = 8 << 20;
const FOLIO: u64 = 2 << 20;

pub const HELD: u64 = 32 << 20;

pub const HELD_FILES: usize = 64;

pub const AT_ONCE: usize = 4;

pub const FILE_COUNTS_AS: u64 = 64 << 10;

pub const MOST_BEFORE_OVER: f32 = 0.99;

pub const fn most_held(files: usize) -> u64 {
    files as u64 * (CHUNK as u64 + 2 * STEP) + HELD
}

pub fn share(done: u64, total: u64, files: Option<(u64, u64)>, over: bool) -> f32 {
    if over {
        return 1.0;
    }
    let (across, of) = files.unwrap_or((0, 0));
    let (done, total) = (done as f64 + (across.min(of) * FILE_COUNTS_AS) as f64, total as f64 + (of * FILE_COUNTS_AS) as f64);
    if total <= 0.0 {
        return 0.0;
    }
    ((done / total) as f32).clamp(0.0, MOST_BEFORE_OVER)
}

pub fn thread() {
    disk::last_in_line();
}

pub fn spawn<T: Send + 'static>(name: &str, work: impl FnOnce() -> T + Send + 'static) -> io::Result<std::thread::JoinHandle<T>> {
    std::thread::Builder::new().name(name.into()).spawn(move || {
        thread();
        work()
    })
}

pub fn scoped<'scope, T: Send + 'scope>(scope: &'scope std::thread::Scope<'scope, '_>, work: impl FnOnce() -> T + Send + 'scope)
        -> std::thread::ScopedJoinHandle<'scope, T> {
    scope.spawn(move || {
        thread();
        work()
    })
}

pub fn program(cmd: &mut std::process::Command) -> &mut std::process::Command {
    disk::start_last_in_line(cmd)
}

pub fn argv<'a>(argv: &[&'a str]) -> Vec<&'a str> {
    let mut out = vec!["ionice", "-c", "3", "nice", "-n", "19"];
    out.extend_from_slice(argv);
    out
}

pub trait Sink: Write {
    fn start(&self, from: u64, len: u64);
    fn wait(&self, from: u64, len: u64) -> io::Result<()>;
    fn forget(&self, from: u64, len: u64);
}

impl Sink for File {
    fn start(&self, from: u64, len: u64) {
        disk::start_writing_out(self, from, len);
    }

    fn wait(&self, from: u64, len: u64) -> io::Result<()> {
        #[cfg(any(test, feature = "testing"))]
        if faults::file(self) {
            return Err(io::Error::from_raw_os_error(libc::EIO));
        }
        let Err(e) = disk::wait_written_out(self, from, len) else { return Ok(()) };
        match e.raw_os_error() {
            Some(libc::EINVAL | libc::ENOSYS | libc::ESPIPE | libc::EOPNOTSUPP) => match self.sync_data() {
                Err(e) if !matches!(e.raw_os_error(), Some(libc::EINVAL | libc::ENOSYS | libc::EROFS | libc::EOPNOTSUPP)) => Err(e),
                _ => Ok(()),
            },
            _ => Err(e),
        }
    }

    fn forget(&self, from: u64, len: u64) {
        disk::need_not_keep(self, from, len);
    }
}

pub struct Out<'a, S: Sink> {
    to: S,
    at: u64,
    begun: u64,
    out: u64,
    reached: &'a (dyn Fn(u64) + Sync),
}

impl<'a, S: Sink> Out<'a, S> {
    pub fn new(to: S, reached: &'a (dyn Fn(u64) + Sync)) -> Out<'a, S> {
        Out { to, at: 0, begun: 0, out: 0, reached }
    }

    fn step(&mut self) -> io::Result<()> {
        if self.begun > self.out {
            self.to.wait(self.out, self.begun - self.out)?;
            self.to.forget(self.out, self.begun - self.out);
            (self.reached)(self.begun - self.out);
            self.out = self.begun;
        }
        self.to.start(self.begun, self.at - self.begun);
        self.begun = self.at;
        Ok(())
    }

    pub fn finish_now(mut self) -> io::Result<S> {
        self.to.flush()?;
        if self.at > self.out {
            self.to.wait(self.out, self.at - self.out)?;
            self.to.forget(self.out, self.at - self.out);
            (self.reached)(self.at - self.out);
        }
        Ok(self.to)
    }

    pub fn finish<T>(self, held: &Held<S, T>, whose: T) -> io::Result<()> {
        if self.begun > 0 {
            return self.finish_now().map(drop);
        }
        let Out { mut to, at, reached, .. } = self;
        to.flush()?;
        held.keep(to, at, whose, reached);
        Ok(())
    }

    pub fn file(&self) -> &S {
        &self.to
    }
}

impl<S: Sink> Write for Out<'_, S> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let n = self.to.write(buf)?;
        self.at += n as u64;
        if self.at - self.begun >= STEP {
            self.step()?;
        }
        Ok(n)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.to.flush()
    }
}

pub struct Lost<S, T> {
    pub file: S,
    pub whose: T,
    pub why: io::Error,
}

type Waiting<S, T> = (S, u64, T);

pub struct Held<S: Sink, T = ()> {
    waiting: Mutex<(VecDeque<Waiting<S, T>>, u64)>,
    lost: Mutex<Vec<Lost<S, T>>>,
}

impl<S: Sink, T> Default for Held<S, T> {
    fn default() -> Held<S, T> {
        Held { waiting: Mutex::new((VecDeque::new(), 0)), lost: Mutex::new(Vec::new()) }
    }
}

impl<S: Sink, T> Held<S, T> {
    fn keep(&self, file: S, len: u64, whose: T, reached: &(dyn Fn(u64) + Sync)) {
        let mut due = Vec::new();
        {
            let mut w = self.waiting.lock().unwrap_or_else(|e| e.into_inner());
            w.0.push_back((file, len, whose));
            w.1 += len;
            while w.1 > HELD || w.0.len() > HELD_FILES {
                let Some(first) = w.0.pop_front() else { break };
                w.1 -= first.1;
                due.push(first);
            }
        }
        let lost = out_now(due, reached);
        if !lost.is_empty() {
            self.lost.lock().unwrap_or_else(|e| e.into_inner()).extend(lost);
        }
    }

    #[must_use = "a small file that could not be written out is not done"]
    pub fn settle(&self, reached: &(dyn Fn(u64) + Sync)) -> Vec<Lost<S, T>> {
        let due: Vec<Waiting<S, T>> = {
            let mut w = self.waiting.lock().unwrap_or_else(|e| e.into_inner());
            w.1 = 0;
            w.0.drain(..).collect()
        };
        let mut lost = std::mem::take(&mut *self.lost.lock().unwrap_or_else(|e| e.into_inner()));
        lost.extend(out_now(due, reached));
        lost
    }
}

fn out_now<S: Sink, T>(due: Vec<Waiting<S, T>>, reached: &(dyn Fn(u64) + Sync)) -> Vec<Lost<S, T>> {
    for (file, len, _) in &due {
        file.start(0, *len);
    }
    let mut lost = Vec::new();
    for (file, len, whose) in due {
        match file.wait(0, len) {
            Ok(()) => {
                file.forget(0, len);
                reached(len);
            }
            Err(why) => lost.push(Lost { file, whose, why }),
        }
    }
    lost
}

pub fn all_out(folder: &std::path::Path) -> io::Result<()> {
    drive_of(folder).map_or(Ok(()), |dir| all_out_of(&dir))
}

pub fn drive_of(folder: &std::path::Path) -> io::Result<File> {
    use std::os::unix::fs::OpenOptionsExt;
    std::fs::OpenOptions::new().read(true).custom_flags(libc::O_DIRECTORY | libc::O_CLOEXEC).open(folder)
}

pub fn all_out_of(dir: &File) -> io::Result<()> {
    #[cfg(any(test, feature = "testing"))]
    if faults::folder_open(dir) {
        return Err(io::Error::from_raw_os_error(libc::EIO));
    }
    disk::all_written_out(dir)
}

pub fn written<S: Sink>(file: &S, from: u64, len: u64) -> io::Result<()> {
    file.wait(from, len)?;
    file.forget(from, len);
    Ok(())
}

pub struct Tide<S: Sink + io::Seek = File> {
    file: S,
    at: u64,
    top: u64,
    begun: u64,
    out: u64,
    again: Option<(u64, u64)>,
}

impl<S: Sink + io::Seek> Tide<S> {
    pub fn new(file: S) -> Tide<S> {
        Tide { file, at: 0, top: 0, begun: 0, out: 0, again: None }
    }

    fn again_out(&mut self) -> io::Result<()> {
        if let Some((from, to)) = self.again {
            written(&self.file, from, to - from)?;
            self.again = None;
        }
        Ok(())
    }

    pub fn finish(mut self) -> io::Result<S> {
        self.file.flush()?;
        if self.top > self.out {
            written(&self.file, self.out, self.top - self.out)?;
        }
        self.again_out()?;
        Ok(self.file)
    }
}

impl<S: Sink + io::Seek> Write for Tide<S> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let from = self.at;
        let n = self.file.write(buf)?;
        self.at += n as u64;
        if from < self.out && n > 0 {
            let (lo, hi) = self.again.unwrap_or((from, from));
            self.again = Some((lo.min(from), hi.max(self.at.min(self.out))));
        }
        self.top = self.top.max(self.at);
        if self.top - self.begun >= STEP {
            if self.begun > self.out {
                written(&self.file, self.out, self.begun - self.out)?;
                self.out = self.begun;
            }
            self.again_out()?;
            self.file.start(self.begun, self.top - self.begun);
            self.begun = self.top;
        }
        Ok(n)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.file.flush()
    }
}

impl<S: Sink + io::Seek> io::Seek for Tide<S> {
    fn seek(&mut self, to: io::SeekFrom) -> io::Result<u64> {
        self.at = self.file.seek(to)?;
        Ok(self.at)
    }
}

pub fn copy_into(from: &mut File, to: File) -> io::Result<File> {
    let mut out = Out::new(to, &|_| {});
    pour(&mut In::rest(from.try_clone()?)?, &mut out, &mut || true)?;
    out.finish_now()
}

pub fn copy_named(from: &std::path::Path, to: &std::path::Path) -> io::Result<u64> {
    use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};
    let first = crate::common::files::open_file(from)?;
    let about = first.metadata()?;
    if std::fs::metadata(to).is_ok_and(|m| m.dev() == about.dev() && m.ino() == about.ino()) {
        return Err(io::Error::from_raw_os_error(libc::EINVAL));
    }
    let mode = about.permissions().mode() & 0o7777;
    let made = std::fs::OpenOptions::new().write(true).create(true).truncate(true).mode(mode).open(to)?;
    made.set_permissions(std::fs::Permissions::from_mode(mode))?;
    copy_file_whole(first, made)
}

pub fn copy_file_whole(from: File, to: File) -> io::Result<u64> {
    let mut out = Out::new(to, &|_| {});
    let n = pour(&mut In::new(from), &mut out, &mut || true)?;
    out.finish_now()?;
    Ok(n)
}

fn kept_before(file: &File, first: u64) -> Vec<u64> {
    let len = file.metadata().map_or(0, |m| m.len());
    if len <= first || disk::in_memory(file, first, len - first) != Some(true) {
        return Vec::new();
    }
    (0..(len - first).div_ceil(STEP)).filter(|n| disk::in_memory(file, first + n * STEP, STEP) == Some(true)).collect()
}

pub struct In {
    from: File,
    at: u64,
    edge: u64,
    first: u64,
    kept: Vec<u64>,
}

impl In {
    pub fn new(from: File) -> In {
        In::begun_at(from, 0)
    }

    pub fn rest(mut from: File) -> io::Result<In> {
        use io::Seek;
        let at = from.stream_position()?;
        Ok(In::begun_at(from, at))
    }

    fn begun_at(from: File, at: u64) -> In {
        let kept = kept_before(&from, at);
        disk::will_read_through(&from);
        In { from, at, edge: at, first: at, kept }
    }

    fn pass(&mut self) {
        if self.at > self.edge {
            let steps = (self.edge - self.first) / STEP..=(self.at - 1 - self.first) / STEP;
            let ours = !self.kept.iter().any(|kept| steps.contains(kept));
            if ours {
                let page = disk::page_size().unwrap_or(4096);
                let back = self.edge - self.edge % FOLIO;
                let from = if back >= self.first && !self.kept.contains(&((back - self.first) / STEP)) { back } else { self.edge - self.edge % page };
                let ended = self.from.metadata().is_ok_and(|m| self.at >= m.len());
                let len = if ended { 0 } else { self.at - from };
                disk::need_not_keep(&self.from, from, len);
            }
        }
        self.edge = self.at;
    }

    pub fn file(&self) -> &File {
        &self.from
    }
}

impl Read for In {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = self.from.read(buf)?;
        self.at += n as u64;
        if n == 0 || self.at - self.edge >= STEP {
            self.pass();
        }
        Ok(n)
    }
}

impl Drop for In {
    fn drop(&mut self) {
        self.pass();
    }
}

pub fn pour<S: Sink>(from: &mut dyn Read, to: &mut Out<S>, go_on: &mut dyn FnMut() -> bool) -> io::Result<u64> {
    let mut buf = vec![0u8; CHUNK];
    let mut across = 0u64;
    loop {
        if !go_on() {
            return Err(io::Error::from(io::ErrorKind::Interrupted));
        }
        let n = match from.read(&mut buf) {
            Ok(0) => return Ok(across),
            Ok(n) => n,
            Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(e),
        };
        to.write_all(&buf[..n])?;
        across += n as u64;
    }
}

#[cfg(any(test, feature = "testing"))]
pub mod faults {
    use std::os::fd::AsRawFd;
    use std::path::{Path, PathBuf};
    use std::sync::Mutex;

    static BAD: Mutex<Vec<PathBuf>> = Mutex::new(Vec::new());

    pub fn set(path: &Path) {
        BAD.lock().unwrap_or_else(|e| e.into_inner()).push(real(path));
    }

    fn real(path: &Path) -> PathBuf {
        for up in path.ancestors() {
            if let (Ok(real), Ok(rest)) = (std::fs::canonicalize(up), path.strip_prefix(up)) {
                return if rest.as_os_str().is_empty() { real } else { real.join(rest) };
            }
        }
        path.to_path_buf()
    }

    fn bad() -> Vec<PathBuf> {
        BAD.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }

    pub(super) fn file(file: &std::fs::File) -> bool {
        let bad = bad();
        !bad.is_empty() && std::fs::read_link(format!("/proc/self/fd/{}", file.as_raw_fd())).is_ok_and(|at| bad.contains(&crate::common::files::unspare(&at)))
    }

    pub(super) fn folder_open(dir: &std::fs::File) -> bool {
        let bad = bad();
        !bad.is_empty() && std::fs::read_link(format!("/proc/self/fd/{}", dir.as_raw_fd())).is_ok_and(|at| bad.iter().any(|b| crate::common::files::unspare(&at).starts_with(b)))
    }
}

#[cfg(test)]
mod tests;
