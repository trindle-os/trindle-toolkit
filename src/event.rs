//! The loop every Trindle program waits in: files, timers and signals, run one at a time on the main thread; signals arrive through a pipe, one per loop.

use std::cell::{Cell, RefCell};
use std::io::{Read, Write};
use std::os::fd::{AsRawFd, RawFd};
use std::os::unix::net::UnixStream;
use std::panic::Location;
use std::rc::Rc;
use std::sync::{mpsc, Arc};
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Flow {
    Continue,
    Stop,
}

enum Kind {
    Fd(RawFd),
    Timer { due: Instant, every: Option<Duration> },
    Signal(libc::c_int),
}

type Callback = Box<dyn FnMut(bool) -> Flow>;

struct Entry {
    id: u64,
    kind: Kind,
    call: Option<Callback>,
    from: &'static Location<'static>,
    runs: u32,
}

const SHORTEST: Duration = Duration::from_millis(1);

const SPINNING: u32 = 4000;

struct Turns {
    since: Cell<Option<Instant>>,
    count: Cell<u32>,
    hot: Cell<bool>,
    said: Cell<bool>,
}

struct Inner {
    entries: RefCell<Vec<Entry>>,
    next: Cell<u64>,
    quit: Cell<bool>,
    spent: Cell<bool>,
    held: Cell<usize>,
    pipe: Cell<Option<(RawFd, usize)>>,
    turns: Turns,
}

impl Drop for Inner {
    fn drop(&mut self) {
        for e in self.entries.get_mut() {
            if let Kind::Signal(signal) = e.kind {
                release(signal);
            }
        }
        let Some((read, slot)) = self.pipe.get() else { return };
        crate::sys::signal::close_pipe(read, slot);
    }
}

fn due_in(wait: Duration) -> Instant {
    let now = Instant::now();
    now.checked_add(wait).or_else(|| now.checked_add(Duration::from_secs(30 * 365 * 86_400))).unwrap_or(now)
}

pub struct Hold(Rc<Inner>);

impl Drop for Hold {
    fn drop(&mut self) {
        self.0.held.set(self.0.held.get() - 1);
    }
}

#[derive(Clone)]
pub struct Loop(Rc<Inner>);

impl Default for Loop {
    fn default() -> Self {
        Self::new()
    }
}

impl Loop {
    pub fn new() -> Loop {
        Loop(Rc::new(Inner { entries: RefCell::new(Vec::new()), next: Cell::new(1), quit: Cell::new(false), spent: Cell::new(false), held: Cell::new(0), pipe: Cell::new(None),
            turns: Turns { since: Cell::new(None), count: Cell::new(0), hot: Cell::new(false), said: Cell::new(false) } }))
    }

    #[track_caller]
    fn add(&self, kind: Kind, call: Callback) {
        let id = self.0.next.get();
        self.0.next.set(id + 1);
        self.0.entries.borrow_mut().push(Entry { id, kind, call: Some(call), from: Location::caller(), runs: 0 });
    }

    #[track_caller]
    pub fn watch(&self, fd: RawFd, call: impl FnMut(bool) -> Flow + 'static) {
        self.add(Kind::Fd(fd), Box::new(call))
    }

    #[track_caller]
    pub fn every(&self, every: Duration, mut call: impl FnMut() -> Flow + 'static) {
        let every = every.max(SHORTEST);
        self.add(Kind::Timer { due: due_in(every), every: Some(every) }, Box::new(move |_| call()))
    }

    #[track_caller]
    pub fn after(&self, wait: Duration, call: impl FnOnce() + 'static) {
        let mut call = Some(call);
        self.add(Kind::Timer { due: due_in(wait), every: None }, Box::new(move |_| {
            if let Some(c) = call.take() {
                c();
            }
            Flow::Stop
        }))
    }

    #[track_caller]
    pub fn on_signal(&self, signal: libc::c_int, mut call: impl FnMut() -> Flow + 'static) -> bool {
        if !self.catch(signal) {
            return false;
        }
        self.add(Kind::Signal(signal), Box::new(move |_| call()));
        true
    }

    #[track_caller]
    pub fn post<T: 'static>(&self, mut call: impl FnMut(T) + 'static) -> Post<T> {
        let (tx, rx) = mpsc::channel();
        let Ok((mut ear, bell)) = UnixStream::pair() else {
            return Post { tx, bell: None };
        };
        if ear.set_nonblocking(true).and(bell.set_nonblocking(true)).is_err() {
            return Post { tx, bell: None };
        }
        self.watch(ear.as_raw_fd(), move |hung_up| {
            let mut rung = [0u8; 64];
            let mut ended = false;
            loop {
                match ear.read(&mut rung) {
                    Ok(0) => {
                        ended = true;
                        break;
                    }
                    Ok(_) => {}
                    Err(_) => break,
                }
            }
            for t in rx.try_iter() {
                call(t);
            }
            if hung_up || ended { Flow::Stop } else { Flow::Continue }
        });
        Post { tx, bell: Some(Arc::new(bell)) }
    }

    pub fn quit(&self) {
        self.0.quit.set(true);
        self.0.spent.set(false);
    }

    pub fn quitting(&self) -> bool {
        self.0.quit.get()
    }

    pub fn hold(&self) -> Hold {
        self.0.held.set(self.0.held.get() + 1);
        Hold(self.0.clone())
    }

    fn done(&self) -> bool {
        self.0.quit.get() && self.0.held.get() == 0
    }

    pub fn run(&self) {
        if self.0.spent.replace(false) {
            self.0.quit.set(false);
        }
        while !self.done() {
            self.turn();
        }
        self.0.spent.set(true);
    }

    fn turn(&self) {
        for (id, hung_up) in self.wait() {
            if self.done() {
                break;
            }
            self.fire(id, hung_up);
        }
        self.count_turn(Instant::now());
    }

    fn count_turn(&self, now: Instant) {
        let t = &self.0.turns;
        if t.said.get() {
            return;
        }
        t.count.set(t.count.get() + 1);
        let since = t.since.get().unwrap_or(now);
        t.since.set(Some(since));
        if now.duration_since(since) < Duration::from_secs(1) {
            return;
        }
        let spinning = t.count.get() > SPINNING;
        let mut entries = self.0.entries.borrow_mut();
        if spinning && t.hot.get() {
            t.said.set(true);
            let most = entries.iter().max_by_key(|e| e.runs).map(|e| (e.runs, describe(e)));
            let what = match most {
                Some((runs, what)) if runs > 0 => format!("mostly for {what}, run {runs} times"),
                _ => "for nothing it was watching".into(),
            };
            eprintln!("{}: the event loop woke {} times in a second, {what}; it is spinning", program(), t.count.get());
        }
        t.hot.set(spinning);
        t.count.set(0);
        t.since.set(Some(now));
        for e in entries.iter_mut() {
            e.runs = 0;
        }
    }

    fn wait(&self) -> Vec<(u64, bool)> {
        let pipe = self.0.pipe.get().map(|(r, _)| r);
        let (mut polls, ids, timeout) = {
            let entries = self.0.entries.borrow();
            let mut polls = Vec::new();
            let mut ids = Vec::new();
            if let Some(r) = pipe {
                polls.push(libc::pollfd { fd: r, events: libc::POLLIN, revents: 0 });
                ids.push(0);
            }
            let mut soonest: Option<Instant> = None;
            for e in entries.iter() {
                if e.call.is_none() {
                    continue;
                }
                match e.kind {
                    Kind::Fd(fd) => {
                        polls.push(libc::pollfd { fd, events: libc::POLLIN | libc::POLLRDHUP, revents: 0 });
                        ids.push(e.id);
                    }
                    Kind::Timer { due, .. } => soonest = Some(soonest.map_or(due, |s| s.min(due))),
                    Kind::Signal(_) => {}
                }
            }
            (polls, ids, soonest.map_or(-1, |s| millis_until(s, Instant::now())))
        };
        let n = crate::sys::fd::poll(&mut polls, timeout);
        let mut ready = Vec::new();
        if n > 0 {
            for (p, &id) in polls.iter().zip(&ids) {
                if p.revents == 0 {
                    continue;
                }
                if id == 0 {
                    ready.extend(self.signalled(p.fd));
                } else {
                    ready.push((id, ended(p.fd, p.revents)));
                }
            }
        }
        let now = Instant::now();
        ready.extend(self.0.entries.borrow().iter()
            .filter(|e| e.call.is_some() && matches!(e.kind, Kind::Timer { due, .. } if due <= now))
            .map(|e| (e.id, false)));
        ready
    }

    fn signalled(&self, fd: RawFd) -> Vec<(u64, bool)> {
        let mut got = [0u8; 64];
        let mut signals = Vec::new();
        loop {
            let n = crate::sys::fd::read(fd, &mut got);
            if n <= 0 {
                break;
            }
            for &s in &got[..n as usize] {
                if !signals.contains(&s) {
                    signals.push(s);
                }
            }
        }
        self.0.entries.borrow().iter()
            .filter(|e| matches!(e.kind, Kind::Signal(s) if signals.contains(&(s as u8))))
            .map(|e| (e.id, false))
            .collect()
    }

    fn fire(&self, id: u64, hung_up: bool) {
        let call = self.0.entries.borrow_mut().iter_mut().find(|e| e.id == id).and_then(|e| {
            e.runs += 1;
            e.call.take()
        });
        let Some(mut call) = call else { return };
        let flow = call(hung_up);
        let mut entries = self.0.entries.borrow_mut();
        let Some(at) = entries.iter().position(|e| e.id == id) else { return };
        let over = flow == Flow::Continue && hung_up && matches!(entries[at].kind, Kind::Fd(fd) if still_ended(fd));
        if over {
            eprintln!("{}: no longer watching {}: it has hung up", program(), describe(&entries[at]));
        }
        let now = Instant::now();
        let keep = flow == Flow::Continue && !over && match &mut entries[at].kind {
            Kind::Timer { due, every: Some(every) } => {
                *due += *every;
                if *due <= now {
                    *due = now + *every;
                }
                true
            }
            Kind::Timer { every: None, .. } => false,
            _ => true,
        };
        if keep {
            entries[at].call = Some(call);
        } else if let Kind::Signal(signal) = entries.remove(at).kind {
            release(signal);
        }
    }
}

fn ended(fd: RawFd, revents: libc::c_short) -> bool {
    if revents & (libc::POLLHUP | libc::POLLERR | libc::POLLNVAL | libc::POLLRDHUP) == 0 {
        return false;
    }
    if revents & libc::POLLIN == 0 {
        return true;
    }
    crate::sys::fd::unread(fd) == Some(0)
}

fn still_ended(fd: RawFd) -> bool {
    let mut p = libc::pollfd { fd, events: libc::POLLIN | libc::POLLRDHUP, revents: 0 };
    crate::sys::fd::poll(std::slice::from_mut(&mut p), 0) > 0 && ended(fd, p.revents)
}

fn program() -> String {
    std::env::args().next().unwrap_or_default()
}

fn describe(e: &Entry) -> String {
    let what = match e.kind {
        Kind::Fd(fd) => {
            let name = std::fs::read_link(format!("/proc/self/fd/{fd}")).map(|p| p.display().to_string()).unwrap_or_default();
            format!("file {fd} ({name})")
        }
        Kind::Timer { every: Some(every), .. } => format!("the timer every {every:?}"),
        Kind::Timer { every: None, .. } => "a timer".into(),
        Kind::Signal(s) => format!("signal {s}"),
    };
    format!("{what} at {}:{}", e.from.file(), e.from.line())
}

pub struct Post<T> {
    tx: mpsc::Sender<T>,
    bell: Option<Arc<UnixStream>>,
}

impl<T> Clone for Post<T> {
    fn clone(&self) -> Self {
        Post { tx: self.tx.clone(), bell: self.bell.clone() }
    }
}

impl<T> Post<T> {
    pub fn send(&self, t: T) -> bool {
        let Some(bell) = &self.bell else { return false };
        if self.tx.send(t).is_err() {
            return false;
        }
        let _ = (&**bell).write(&[1]);
        true
    }
}

fn millis_until(due: Instant, now: Instant) -> libc::c_int {
    let left = due.saturating_duration_since(now);
    left.as_micros().div_ceil(1000).min(libc::c_int::MAX as u128) as libc::c_int
}

static OLD: std::sync::Mutex<Vec<(libc::c_int, usize, libc::sigaction)>> = std::sync::Mutex::new(Vec::new());

fn release(signal: libc::c_int) {
    let Ok(mut all) = OLD.lock() else { return };
    let Some(at) = all.iter().position(|o| o.0 == signal) else { return };
    all[at].1 -= 1;
    if all[at].1 == 0 {
        let (_, _, old) = all.remove(at);
        crate::sys::signal::restore(signal, &old);
    }
}

impl Loop {
    fn catch(&self, signal: libc::c_int) -> bool {
        if self.0.pipe.get().is_none() {
            let Some(pipe) = crate::sys::signal::open_pipe() else { return false };
            self.0.pipe.set(Some(pipe));
        }
        let Ok(mut all) = OLD.lock() else { return false };
        if let Some(o) = all.iter_mut().find(|o| o.0 == signal) {
            o.1 += 1;
            return true;
        }
        let Some(old) = crate::sys::signal::catch(signal) else { return false };
        all.push((signal, 1, old));
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timers_run_in_order_and_once_ones_only_once() {
        let lp = Loop::new();
        let seen = Rc::new(RefCell::new(Vec::new()));
        let s = seen.clone();
        lp.after(Duration::from_millis(150), move || s.borrow_mut().push("late"));
        let s = seen.clone();
        lp.after(Duration::from_millis(5), move || s.borrow_mut().push("soon"));
        let (s, count) = (seen.clone(), Rc::new(Cell::new(0)));
        let c = count.clone();
        lp.every(Duration::from_millis(10), move || {
            c.set(c.get() + 1);
            if c.get() == 2 {
                s.borrow_mut().push("twice");
                return Flow::Stop;
            }
            Flow::Continue
        });
        let quit = lp.clone();
        lp.after(Duration::from_millis(250), move || quit.quit());
        lp.run();
        assert_eq!(*seen.borrow(), vec!["soon", "twice", "late"]);
        assert_eq!(count.get(), 2);
    }

    #[test]
    fn a_held_errand_finishes_after_quit() {
        let lp = Loop::new();
        let hold = lp.hold();
        let done = Rc::new(Cell::new(false));
        let d = done.clone();
        lp.after(Duration::from_millis(20), move || {
            d.set(true);
            drop(hold);
        });
        let quit = lp.clone();
        lp.after(Duration::from_millis(1), move || quit.quit());
        lp.run();
        assert!(done.get());
    }

    #[test]
    fn a_file_with_something_to_read_wakes_the_loop() {
        let lp = Loop::new();
        let (read, write) = crate::sys::fd::pipe(0).unwrap();
        std::fs::File::from(write).write_all(b"x").unwrap();
        let quit = lp.clone();
        let got = Rc::new(Cell::new(false));
        let g = got.clone();
        lp.watch(read.as_raw_fd(), move |hung_up| {
            g.set(!hung_up);
            quit.quit();
            Flow::Stop
        });
        lp.run();
        assert!(got.get());
    }

    #[test]
    fn things_posted_from_another_thread_arrive_in_order() {
        let lp = Loop::new();
        let got = Rc::new(RefCell::new(Vec::new()));
        let (g, quit) = (got.clone(), lp.clone());
        let post = lp.post(move |n: u32| {
            g.borrow_mut().push(n);
            if n == 99 {
                quit.quit();
            }
        });
        let away = std::thread::spawn(move || {
            for n in 0..100 {
                assert!(post.send(n));
            }
        });
        lp.run();
        away.join().unwrap();
        assert_eq!(*got.borrow(), (0..100).collect::<Vec<_>>());
    }

    #[test]
    fn a_quit_asked_before_run_is_kept_and_a_later_run_is_fresh() {
        let lp = Loop::new();
        lp.quit();
        let began = Instant::now();
        let net = lp.clone();
        lp.after(Duration::from_secs(5), move || net.quit());
        lp.run();
        assert!(began.elapsed() < Duration::from_secs(2), "a quit asked before run was forgotten");
        assert!(lp.quitting());
        let quit = lp.clone();
        lp.after(Duration::from_millis(100), move || quit.quit());
        let again = Instant::now();
        lp.run();
        assert!(again.elapsed() >= Duration::from_millis(90), "the second run ended on the first one's quit");
    }

    #[test]
    fn a_post_all_gone_is_let_go_and_the_loop_rests() {
        let lp = Loop::new();
        let got = Rc::new(Cell::new(0u32));
        let g = got.clone();
        let post = lp.post(move |n: u32| g.set(g.get() + n));
        assert!(post.send(5));
        drop(post);
        assert_eq!(lp.0.entries.borrow().len(), 1);
        let quit = lp.clone();
        lp.after(Duration::from_millis(300), move || quit.quit());
        let began = Instant::now();
        let mut turns = 0;
        lp.0.quit.set(false);
        while !lp.done() {
            for (id, hung_up) in lp.wait() {
                lp.fire(id, hung_up);
            }
            turns += 1;
        }
        assert_eq!(got.get(), 5, "what was sent before the last copy went still arrives");
        assert!(began.elapsed() >= Duration::from_millis(290));
        assert!(turns <= 4, "the loop woke {turns} times in 300 ms with nothing to do");
        assert!(lp.0.entries.borrow().is_empty(), "nothing is still listened for");
    }

    fn turns_until_quit(lp: &Loop) -> u32 {
        let mut turns = 0;
        lp.0.quit.set(false);
        while !lp.done() {
            lp.turn();
            turns += 1;
        }
        turns
    }

    #[test]
    fn a_file_that_has_hung_up_is_let_go_whatever_its_callback_says() {
        use std::os::fd::OwnedFd;
        let pipe = || {
            crate::sys::fd::pipe(libc::O_CLOEXEC | libc::O_NONBLOCK).unwrap()
        };
        let socket = |shut_only: bool| {
            let (a, b) = UnixStream::pair().unwrap();
            a.set_nonblocking(true).unwrap();
            if shut_only {
                b.shutdown(std::net::Shutdown::Write).unwrap();
                (OwnedFd::from(a), Some(OwnedFd::from(b)))
            } else {
                (OwnedFd::from(a), None)
            }
        };
        let (pipe_read, pipe_write) = pipe();
        drop(pipe_write);
        let (closed, _) = socket(false);
        let (half_shut, _kept) = socket(true);
        for (name, fd) in [("a pipe", pipe_read), ("a socket", closed), ("a socket half shut", half_shut)] {
            let lp = Loop::new();
            let told = Rc::new(RefCell::new(Vec::new()));
            let t = told.clone();
            lp.watch(fd.as_raw_fd(), move |hung_up| {
                t.borrow_mut().push(hung_up);
                Flow::Continue
            });
            let quit = lp.clone();
            lp.after(Duration::from_millis(150), move || quit.quit());
            let turns = turns_until_quit(&lp);
            assert_eq!(*told.borrow(), vec![true], "{name}: told once that it hung up");
            assert!(turns <= 4, "{name}: the loop woke {turns} times in 150 ms with nothing to do");
            assert!(lp.0.entries.borrow().is_empty(), "{name}: nothing is still watched");
        }
    }

    #[test]
    fn what_is_left_in_a_closed_socket_is_read_before_it_counts_as_hung_up() {
        let (mut ear, mut mouth) = UnixStream::pair().unwrap();
        ear.set_nonblocking(true).unwrap();
        mouth.write_all(b"last words").unwrap();
        drop(mouth);
        let lp = Loop::new();
        let seen = Rc::new(RefCell::new(Vec::new()));
        let (s, quit) = (seen.clone(), lp.clone());
        lp.watch(ear.as_raw_fd(), move |hung_up| {
            let mut got = [0u8; 64];
            let n = if hung_up { 0 } else { ear.read(&mut got).unwrap_or(0) };
            s.borrow_mut().push((hung_up, String::from_utf8_lossy(&got[..n]).into_owned()));
            if hung_up {
                quit.quit();
                return Flow::Stop;
            }
            Flow::Continue
        });
        let late = lp.clone();
        lp.after(Duration::from_secs(2), move || late.quit());
        lp.run();
        assert_eq!(*seen.borrow(), vec![(false, "last words".to_string()), (true, String::new())]);
    }

    #[test]
    fn a_loop_run_inside_a_callback_rests() {
        let lp = Loop::new();
        let (mut ear, mut mouth) = UnixStream::pair().unwrap();
        ear.set_nonblocking(true).unwrap();
        mouth.write_all(b"x").unwrap();
        let inner_turns = Rc::new(Cell::new(0));
        for from_timer in [false, true] {
            let (inside, turns, outer) = (lp.clone(), inner_turns.clone(), lp.clone());
            let call = move || {
                let quit = inside.clone();
                inside.after(Duration::from_millis(100), move || quit.quit());
                turns.set(turns.get() + turns_until_quit(&inside));
                outer.quit();
            };
            if from_timer {
                lp.every(Duration::from_millis(1), move || {
                    call();
                    Flow::Stop
                });
            } else {
                lp.watch(ear.as_raw_fd(), move |_| {
                    call();
                    Flow::Stop
                });
            }
            lp.run();
            assert!(inner_turns.get() <= 4, "the loop inside woke {} times in 100 ms", inner_turns.get());
            inner_turns.set(0);
        }
        let mut got = [0u8; 1];
        assert_eq!(ear.read(&mut got).unwrap(), 1, "the byte was left for the callback to read");
    }

    #[test]
    fn a_timer_of_no_length_does_not_spin() {
        let lp = Loop::new();
        let count = Rc::new(Cell::new(0u32));
        let c = count.clone();
        lp.every(Duration::ZERO, move || {
            c.set(c.get() + 1);
            Flow::Continue
        });
        let quit = lp.clone();
        lp.after(Duration::from_millis(50), move || quit.quit());
        lp.run();
        assert!(count.get() <= 60, "ran {} times in 50 ms", count.get());
    }

    #[test]
    fn a_spinning_loop_is_noticed_and_a_busy_one_is_not() {
        let began = Instant::now();
        let turn_for = |lp: &Loop, seconds: u32, a_second: u32| {
            for s in 0..seconds {
                for i in 0..a_second {
                    lp.count_turn(began + Duration::from_secs(s.into()) + Duration::from_secs(1) * i / a_second);
                }
            }
            lp.count_turn(began + Duration::from_secs(seconds.into()));
        };
        let busy = Loop::new();
        turn_for(&busy, 5, SPINNING - 100);
        assert!(!busy.0.turns.said.get(), "a fast mouse or a file copy is not a fault");
        let burst = Loop::new();
        turn_for(&burst, 1, SPINNING * 2);
        turn_for(&burst, 3, 10);
        assert!(!burst.0.turns.said.get(), "one hot second is not a fault");
        let spinning = Loop::new();
        spinning.every(Duration::from_secs(60), || Flow::Continue);
        spinning.0.entries.borrow_mut()[0].runs = 9;
        let line = describe(&spinning.0.entries.borrow()[0]);
        assert!(line.starts_with("the timer every 60s at ") && line.contains("event.rs:"), "{line}");
        turn_for(&spinning, 2, SPINNING * 2);
        assert!(spinning.0.turns.said.get());
        assert_eq!(spinning.0.entries.borrow()[0].runs, 0, "each second is counted afresh");
    }

    #[test]
    fn a_signal_reaches_its_callback() {
        let lp = Loop::new();
        let quit = lp.clone();
        lp.on_signal(libc::SIGUSR2, move || {
            quit.quit();
            Flow::Continue
        });
        crate::sys::signal::raise(libc::SIGUSR2);
        let lp2 = lp.clone();
        lp.after(Duration::from_secs(2), move || lp2.quit());
        let start = Instant::now();
        lp.run();
        assert!(start.elapsed() < Duration::from_secs(1));
    }

    #[test]
    fn a_signal_gets_its_old_action_back_once_its_entry_stops() {
        let action = || crate::sys::signal::action(libc::SIGURG);
        let lp = Loop::new();
        let quit = lp.clone();
        lp.on_signal(libc::SIGURG, move || {
            quit.quit();
            Flow::Stop
        });
        assert_eq!(action(), crate::sys::signal::handler());
        crate::sys::signal::raise(libc::SIGURG);
        let lp2 = lp.clone();
        lp.after(Duration::from_secs(2), move || lp2.quit());
        lp.run();
        assert_eq!(action(), libc::SIG_DFL, "the loop lives on, but nothing wants the signal now");
    }

    #[test]
    fn poll_never_wakes_before_a_timer_is_due() {
        let now = Instant::now();
        assert_eq!(millis_until(now + Duration::from_micros(1500), now), 2);
        assert_eq!(millis_until(now, now + Duration::from_secs(1)), 0);
    }
}
