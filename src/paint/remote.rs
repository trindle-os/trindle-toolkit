//! A program's side of the drawing service: frames encoded and sent, answers heard from the event loop, nothing ever waited for but a buffer, and that only briefly.

use std::cell::{Cell, RefCell};
use std::collections::{HashMap, VecDeque};
use std::os::fd::{AsFd, AsRawFd, BorrowedFd, OwnedFd};
use std::path::Path;
use std::rc::{Rc, Weak};
use std::time::{Duration, Instant};

use tiny_skia::Pixmap;

use super::damage::{changes, Seen};
use super::link::{read_tell, write_ask, Ask, Tell, MOST_LAYOUTS, MOST_MESSAGE, SOCKET, VERSION};
use super::list::Op;
use super::service::runtime_file;
use super::wire::{self, Encoder};
use super::{Painter, Why};
use crate::event::{Flow, Loop};
use crate::shadow::Shadows;
use crate::sys::fd::poll;
use crate::sys::gbm::{Dmabuf, Plane};
use crate::sys::sock::{self, Got};
use crate::window::dmabuf::{Drawn, Frame};

const FRAME_WAIT: Duration = Duration::from_millis(500);
const SHORT_WAIT: Duration = Duration::from_millis(250);
const FRAME_HEAD: usize = 41;
const POST_AGAIN: Duration = Duration::from_millis(2);

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Answer {
    Done,
    Again,
    Gone,
}

type Waiter = Box<dyn FnOnce(Answer)>;

type Made = Option<(u32, u64, Vec<Plane>)>;

pub(crate) struct Remote {
    fd: OwnedFd,
    rdev: u64,
    encoder: RefCell<Encoder>,
    next: Cell<u64>,
    waiting: RefCell<HashMap<u64, (Instant, Waiter)>>,
    told: RefCell<Vec<(u64, Answer)>>,
    made: RefCell<HashMap<u64, Made>>,
    outbox: RefCell<VecDeque<(Vec<u8>, Option<OwnedFd>)>>,
    posting: Cell<bool>,
    gone: Cell<bool>,
    armed: Cell<bool>,
    events: RefCell<Option<Loop>>,
    windows: Cell<usize>,
    asleep: Cell<usize>,
    me: Weak<Remote>,
}

thread_local! {
    static HERE: RefCell<Option<Rc<Remote>>> = const { RefCell::new(None) };
}

pub(crate) fn connected(node: &Path, events: &Loop) -> Result<Rc<Remote>, Why> {
    use std::os::unix::fs::MetadataExt;
    let want = std::fs::metadata(node).map_err(|_| Why::NoNode)?.rdev();
    if let Some(r) = HERE.with_borrow(|h| h.clone()) {
        return if r.gone.get() {
            Err(Why::ServiceGone)
        } else if r.rdev != want {
            Err(Why::ServiceOtherGpu)
        } else {
            Ok(r)
        };
    }
    let r = Remote::at(&runtime_file(SOCKET).ok_or(Why::NoService)?).ok_or(Why::NoService)?;
    if r.rdev != want {
        return Err(Why::ServiceOtherGpu);
    }
    r.hear_by(events);
    HERE.set(Some(r.clone()));
    Ok(r)
}

impl Remote {
    pub(crate) fn at(path: &Path) -> Option<Rc<Remote>> {
        let fd = sock::connect(path).ok()?;
        if sock::peer_uid(fd.as_raw_fd()) != Some(sock::own_uid()) {
            return None;
        }
        let began = Instant::now();
        let mut buf = [0; 64];
        let hello = loop {
            let left = SHORT_WAIT.saturating_sub(began.elapsed()).as_millis() as i32;
            let mut wait = [libc::pollfd { fd: fd.as_raw_fd(), events: libc::POLLIN, revents: 0 }];
            poll(&mut wait, left);
            match sock::recv(fd.as_raw_fd(), &mut buf, &mut Vec::new()) {
                Got::Message { len, cut: false } => break read_tell(&buf[..len]).ok()?,
                Got::Nothing if left > 0 => {}
                _ => return None,
            }
        };
        let Tell::Hello { version: VERSION, rdev } = hello else { return None };
        Some(Rc::new_cyclic(|me| Remote {
            fd, rdev, encoder: RefCell::default(), next: Cell::new(1), waiting: RefCell::default(), told: RefCell::default(), made: RefCell::default(),
            outbox: RefCell::default(), posting: Cell::new(false), gone: Cell::new(false), armed: Cell::new(false), events: RefCell::new(None), windows: Cell::new(0), asleep: Cell::new(0), me: me.clone(),
        }))
    }

    pub(crate) fn hear_by(&self, events: &Loop) {
        *self.events.borrow_mut() = Some(events.clone());
        let me = self.me.clone();
        events.watch(self.fd.as_raw_fd(), move |hung| {
            let Some(me) = me.upgrade() else { return Flow::Stop };
            me.pump();
            me.hand_on();
            if hung {
                me.die();
            }
            if me.gone.get() { Flow::Stop } else { Flow::Continue }
        });
    }

    pub(crate) fn gone(&self) -> bool {
        self.gone.get()
    }

    fn pump(&self) {
        let mut buf = [0; 512];
        while !self.gone.get() {
            let mut fds = Vec::new();
            match sock::recv(self.fd.as_raw_fd(), &mut buf, &mut fds) {
                Got::Nothing => return,
                Got::Ended | Got::Message { cut: true, .. } => {
                    self.lost_contact();
                    return;
                }
                Got::Message { len, .. } => match read_tell(&buf[..len]) {
                    Ok(Tell::Done { seq }) => self.told.borrow_mut().push((seq, Answer::Done)),
                    Ok(Tell::Again { seq }) => self.told.borrow_mut().push((seq, Answer::Again)),
                    Ok(Tell::NotMade { id }) => {
                        self.made.borrow_mut().insert(id, None);
                    }
                    Ok(Tell::Made { id, format, modifier, planes }) if planes.len() == fds.len() => {
                        let planes = planes.into_iter().zip(fds).map(|((stride, offset), fd)| Plane { fd, stride, offset }).collect();
                        self.made.borrow_mut().insert(id, Some((format, modifier, planes)));
                    }
                    _ => {
                        self.lost_contact();
                        return;
                    }
                },
            }
        }
    }

    fn post(&self, bytes: Vec<u8>, file: Option<OwnedFd>) {
        self.outbox.borrow_mut().push_back((bytes, file));
        self.post_on();
    }

    fn post_on(&self) {
        let mut outbox = self.outbox.borrow_mut();
        while let Some((bytes, file)) = outbox.front() {
            if self.gone.get() {
                outbox.clear();
                return;
            }
            let fds: Vec<BorrowedFd> = file.iter().map(|f| f.as_fd()).collect();
            match sock::send(self.fd.as_raw_fd(), bytes, &fds) {
                Ok(()) => {
                    outbox.pop_front();
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    drop(outbox);
                    self.post_later();
                    return;
                }
                Err(_) => {
                    outbox.clear();
                    drop(outbox);
                    self.lost_contact();
                    return;
                }
            }
        }
    }

    fn post_later(&self) {
        let events = self.events.borrow();
        let Some(events) = events.as_ref().filter(|_| !self.posting.replace(true)) else { return };
        let me = self.me.clone();
        events.after(POST_AGAIN, move || {
            let Some(me) = me.upgrade() else { return };
            me.posting.set(false);
            me.post_on();
        });
    }

    fn lost_contact(&self) {
        self.gone.set(true);
        sock::shut(self.fd.as_raw_fd());
        let me = self.me.clone();
        if let Some(events) = &*self.events.borrow() {
            events.after(Duration::ZERO, move || {
                if let Some(me) = me.upgrade() {
                    me.die();
                }
            });
        }
    }

    fn hand_on(&self) {
        let told = std::mem::take(&mut *self.told.borrow_mut());
        for (seq, answer) in told {
            let waiter = self.waiting.borrow_mut().remove(&seq);
            if let Some((_, call)) = waiter {
                call(answer);
            }
        }
    }

    fn die(&self) {
        self.gone.set(true);
        sock::shut(self.fd.as_raw_fd());
        let waiting: Vec<_> = self.waiting.borrow_mut().drain().collect();
        for (_, (_, call)) in waiting {
            call(Answer::Gone);
        }
    }

    pub(crate) fn when_answered(&self, seq: u64, call: impl FnOnce(Answer) + 'static) {
        self.waiting.borrow_mut().insert(seq, (Instant::now(), Box::new(call)));
        self.arm();
    }

    fn arm(&self) {
        let events = self.events.borrow();
        let Some(events) = events.as_ref().filter(|_| !self.armed.replace(true)) else { return };
        let me = self.me.clone();
        events.after(FRAME_WAIT, move || {
            let Some(me) = me.upgrade() else { return };
            me.armed.set(false);
            me.pump();
            me.hand_on();
            let late = me.waiting.borrow().values().any(|(sent, _)| sent.elapsed() >= FRAME_WAIT);
            if late {
                me.die();
            } else if !me.waiting.borrow().is_empty() {
                me.arm();
            }
        });
    }

    pub(crate) fn buffer(&self, id: u64, (w, h): (u32, u32), layouts: &[u64]) -> Option<Dmabuf> {
        let layouts = layouts[..layouts.len().min(MOST_LAYOUTS)].to_vec();
        self.post(write_ask(&Ask::Buffer { id, w, h, layouts }), None);
        if self.gone.get() {
            return None;
        }
        let began = Instant::now();
        let made = loop {
            self.post_on();
            self.pump();
            if let Some(made) = self.made.borrow_mut().remove(&id) {
                break made;
            }
            let left = SHORT_WAIT.saturating_sub(began.elapsed()).as_millis() as i32;
            if left == 0 || self.gone.get() {
                self.lost_contact();
                return None;
            }
            let events = if self.outbox.borrow().is_empty() { libc::POLLIN } else { libc::POLLIN | libc::POLLOUT };
            poll(&mut [libc::pollfd { fd: self.fd.as_raw_fd(), events, revents: 0 }], left.min(POST_AGAIN.as_millis() as i32));
        };
        self.wake_after_reading();
        let (format, modifier, planes) = made?;
        Some(Dmabuf { width: w, height: h, format, modifier, planes })
    }

    fn wake_after_reading(&self) {
        if self.told.borrow().is_empty() {
            return;
        }
        let me = self.me.clone();
        if let Some(events) = &*self.events.borrow() {
            events.after(Duration::ZERO, move || {
                if let Some(me) = me.upgrade() {
                    me.hand_on();
                }
            });
        }
    }

    pub(crate) fn free(&self, id: u64) {
        if !self.gone.get() {
            self.post(write_ask(&Ask::Free { id }), None);
        }
    }

    pub(crate) fn send(&self, id: u64, part: [i32; 4], ops: &[Op]) -> Result<u64, wire::Error> {
        let frame = self.encoder.borrow_mut().frame(ops)?;
        let seq = self.next.replace(self.next.get() + 1);
        let (control, pixels) = (frame.control.len() as u32, frame.pixels.len() as u32);
        if FRAME_HEAD + frame.control.len() + frame.pixels.len() <= MOST_MESSAGE {
            let bytes = [frame.control.as_slice(), frame.pixels.as_slice()].concat();
            self.post(write_ask(&Ask::Frame { seq, id, part, control, pixels, inline: &bytes }), None);
        } else {
            match sock::sealed(c"trindle-frame", &[&frame.control, &frame.pixels]) {
                Ok(file) => self.post(write_ask(&Ask::Frame { seq, id, part, control, pixels, inline: &[] }), Some(file)),
                Err(_) => self.lost_contact(),
            }
        }
        Ok(seq)
    }

    fn start_again(&self) {
        self.encoder.borrow_mut().start_again();
    }

    #[cfg(test)]
    pub(crate) fn send_raw(&self, id: u64, part: [i32; 4], control: &[u8], pixels: &[u8]) -> u64 {
        let seq = self.next.replace(self.next.get() + 1);
        let (c, p) = (control.len() as u32, pixels.len() as u32);
        let sent = if FRAME_HEAD + control.len() + pixels.len() <= MOST_MESSAGE {
            sock::send(self.fd.as_raw_fd(), &write_ask(&Ask::Frame { seq, id, part, control: c, pixels: p, inline: &[control, pixels].concat() }), &[])
        } else {
            sock::sealed(c"trindle-frame", &[control, pixels]).and_then(|file| {
                sock::send(self.fd.as_raw_fd(), &write_ask(&Ask::Frame { seq, id, part, control: c, pixels: p, inline: &[] }), &[file.as_fd()])
            })
        };
        if sent.is_err() {
            self.gone.set(true);
        }
        seq
    }

    #[cfg(test)]
    pub(crate) fn send_bytes(&self, bytes: &[u8]) {
        if sock::send(self.fd.as_raw_fd(), bytes, &[]).is_err() {
            self.gone.set(true);
        }
    }

    #[cfg(test)]
    pub(crate) fn send_with_file(&self, id: u64, control: u32, file: std::os::fd::BorrowedFd) -> u64 {
        let seq = self.next.replace(self.next.get() + 1);
        if sock::send(self.fd.as_raw_fd(), &write_ask(&Ask::Frame { seq, id, part: [0; 4], control, pixels: 0, inline: &[] }), &[file]).is_err() {
            self.gone.set(true);
        }
        seq
    }

    #[cfg(test)]
    pub(crate) fn answered_yet(&self, seq: u64) -> Option<Answer> {
        self.pump();
        let at = self.told.borrow().iter().position(|(s, _)| *s == seq)?;
        Some(self.told.borrow_mut().remove(at).1)
    }

    #[cfg(test)]
    pub(crate) fn ended(&self) -> bool {
        self.pump();
        self.gone.get()
    }

    #[cfg(test)]
    pub(crate) fn block_for(&self, seq: u64) -> Answer {
        let began = Instant::now();
        loop {
            self.pump();
            let at = self.told.borrow().iter().position(|(s, _)| *s == seq);
            if let Some(at) = at {
                return self.told.borrow_mut().remove(at).1;
            }
            if self.gone.get() || began.elapsed() > Duration::from_secs(60) {
                return Answer::Gone;
            }
            poll(&mut [libc::pollfd { fd: self.fd.as_raw_fd(), events: libc::POLLIN, revents: 0 }], 100);
        }
    }
}

pub(crate) struct Sending {
    remote: Rc<Remote>,
    seen: Seen,
    scratch: Option<Pixmap>,
    asleep: bool,
    agains: u32,
}

impl Sending {
    pub(crate) fn new(remote: Rc<Remote>) -> Sending {
        remote.windows.set(remote.windows.get() + 1);
        Sending { remote, seen: Seen::new(), scratch: None, asleep: false, agains: 0 }
    }

    pub(crate) fn remote(&self) -> &Rc<Remote> {
        &self.remote
    }

    pub(crate) fn sleep(&mut self, yes: bool) {
        if yes == self.asleep {
            return;
        }
        self.asleep = yes;
        let r = &self.remote;
        if !yes {
            r.asleep.set(r.asleep.get().saturating_sub(1));
            return;
        }
        r.asleep.set(r.asleep.get() + 1);
        self.seen = Seen::new();
        self.scratch = None;
        if r.asleep.get() >= r.windows.get() && !r.gone() {
            r.start_again();
            r.post(write_ask(&Ask::Sleep), None);
        }
    }

    pub(crate) fn done(&mut self) {
        self.agains = 0;
    }

    pub(crate) fn again(&mut self) -> bool {
        self.agains += 1;
        self.seen = Seen::new();
        self.remote.start_again();
        self.agains < 3
    }

    pub(crate) fn draw(&mut self, frame: &Frame, p: Painter, _: &Shadows) -> Drawn {
        self.sleep(false);
        if self.remote.gone() || frame.id == 0 {
            return Drawn::Gone;
        }
        let (ops, changed) = changes(p, frame.size, &mut self.seen, &mut self.scratch);
        let changed: Option<Vec<[i32; 4]>> = changed.map(|c| c.iter().map(|r| [r.x(), r.y(), r.width() as i32, r.height() as i32]).collect());
        let part = match super::drawer::part_to_draw(&changed, frame) {
            Some(part) => part,
            None => return changed.map_or(Drawn::Whole, Drawn::Parts),
        };
        match self.remote.send(frame.id, part, &ops) {
            Ok(seq) => Drawn::Sent { seq, parts: changed },
            Err(why) => {
                super::log::say(super::log::Kind::Processor, || format!("a frame cannot be sent to the drawing service: {why}"));
                Drawn::Gone
            }
        }
    }
}

impl Drop for Sending {
    fn drop(&mut self) {
        let r = &self.remote;
        r.windows.set(r.windows.get().saturating_sub(1));
        if self.asleep {
            r.asleep.set(r.asleep.get().saturating_sub(1));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paint::link::{read_ask, write_tell};

    fn joined(name: &str) -> (Rc<Remote>, OwnedFd, std::path::PathBuf) {
        let dir = std::env::temp_dir().join(format!("trindle-remote-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let listener = sock::listen(&dir.join("s")).unwrap();
        let server = std::thread::spawn(move || {
            poll(&mut [libc::pollfd { fd: listener.as_raw_fd(), events: libc::POLLIN, revents: 0 }], 5000);
            let far = sock::accept(listener.as_raw_fd()).unwrap();
            sock::send(far.as_raw_fd(), &write_tell(&Tell::Hello { version: VERSION, rdev: 7 }), &[]).unwrap();
            far
        });
        let remote = Remote::at(&dir.join("s")).expect("told hello");
        (remote, server.join().unwrap(), dir)
    }

    #[test]
    fn a_call_that_reads_the_socket_again_while_answers_are_handed_on_does_not_panic() {
        let (remote, far, dir) = joined("again");
        let inner = remote.clone();
        let called = Rc::new(Cell::new(false));
        let was_called = called.clone();
        remote.when_answered(1, move |answer| {
            assert_eq!(answer, Answer::Again);
            inner.pump();
            inner.wake_after_reading();
            was_called.set(true);
        });
        sock::send(far.as_raw_fd(), &write_tell(&Tell::Again { seq: 1 }), &[]).unwrap();
        poll(&mut [libc::pollfd { fd: remote.fd.as_raw_fd(), events: libc::POLLIN, revents: 0 }], 1000);
        remote.pump();
        remote.hand_on();
        assert!(called.get());
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn a_service_busy_with_a_frame_is_not_taken_for_gone_and_nothing_is_lost_or_out_of_order() {
        let (remote, far, dir) = joined("full");
        let sent = 64u64;
        for id in 0..sent {
            let filler = vec![0u8; MOST_MESSAGE - 64];
            remote.post(write_ask(&Ask::Frame { seq: id, id, part: [0; 4], control: filler.len() as u32, pixels: 0, inline: &filler }), None);
        }
        assert!(!remote.gone(), "a full socket is not a dead service");
        assert!(!remote.outbox.borrow().is_empty(), "the test must have filled the socket");
        let mut buf = vec![0u8; MOST_MESSAGE + 1];
        let mut next = 0;
        let began = Instant::now();
        while next < sent {
            assert!(began.elapsed() < Duration::from_secs(10), "stopped at {next}");
            match sock::recv(far.as_raw_fd(), &mut buf, &mut Vec::new()) {
                Got::Message { len, cut: false } => {
                    let Ok(Ask::Frame { seq, .. }) = read_ask(&buf[..len]) else { panic!("not a frame") };
                    assert_eq!(seq, next);
                    next += 1;
                }
                Got::Nothing => remote.post_on(),
                other => panic!("{other:?}"),
            }
        }
        assert!(remote.outbox.borrow().is_empty());
        let _ = std::fs::remove_dir_all(dir);
    }
}
