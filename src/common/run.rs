//! Work done off the main thread. The work happens on threads of its own; what it finds is handed back to the thread that asked through the toolkit's loop, held until a window has connected if it finishes sooner.

use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::os::fd::OwnedFd;
use std::rc::Rc;
use std::sync::Mutex;

type Errand = Box<dyn FnOnce() + Send>;
type Wake = Box<dyn Fn(Errand) + Send>;

static WAKE: Mutex<Option<Wake>> = Mutex::new(None);
static EARLY: Mutex<Vec<Errand>> = Mutex::new(Vec::new());

const MOST_EARLY: usize = 4096;

pub fn hand_back_to(events: &crate::event::Loop) {
    let post = events.post(|errand: Errand| errand());
    if let Ok(mut wake) = WAKE.lock() {
        let early = EARLY.lock().map(|mut e| std::mem::take(&mut *e)).unwrap_or_default();
        for errand in early {
            post.send(errand);
        }
        *wake = Some(Box::new(move |errand| {
            post.send(errand);
        }));
    }
}

fn wake(errand: Errand) {
    let wake = WAKE.lock().ok();
    match wake.as_ref().and_then(|w| w.as_ref()) {
        Some(w) => w(errand),
        None => {
            if let Ok(mut early) = EARLY.lock() {
                if early.len() < MOST_EARLY {
                    early.push(errand);
                }
            }
        }
    }
}

pub type Waiting = Box<dyn FnMut(Box<dyn Any + Send>) -> bool>;

thread_local! {
    static WAITING: RefCell<HashMap<u64, Waiting>> = RefCell::new(HashMap::new());
    static NEXT: std::cell::Cell<u64> = const { std::cell::Cell::new(1) };
}

pub fn wait_for(call: Waiting) -> u64 {
    let id = NEXT.with(|n| {
        let id = n.get();
        n.set(id + 1);
        id
    });
    WAITING.with(|w| w.borrow_mut().insert(id, call));
    id
}

pub fn answer(id: u64, value: Box<dyn Any + Send>) {
    wake(Box::new(move || {
        let call = WAITING.with(|w| w.borrow_mut().remove(&id));
        if let Some(mut call) = call {
            if call(value) {
                WAITING.with(|w| w.borrow_mut().insert(id, call));
            }
        }
    }));
}

pub fn on_thread<T: Send + 'static>(work: impl FnOnce() -> T + Send + 'static, done: impl FnOnce(T) + 'static) {
    let mut done = Some(done);
    let id = wait_for(Box::new(move |v| {
        if let (Some(done), Ok(v)) = (done.take(), v.downcast::<T>()) {
            done(*v);
        }
        false
    }));
    std::thread::spawn(move || match std::panic::catch_unwind(std::panic::AssertUnwindSafe(work)) {
        Ok(v) => answer(id, Box::new(v)),
        Err(_) => answer(id, Box::new(Panicked)),
    });
}

struct Panicked;

pub struct Slots {
    out: std::sync::atomic::AtomicUsize,
    most: usize,
}

pub struct Slot(&'static Slots);

impl Slots {
    pub const fn new(most: usize) -> Slots {
        Slots { out: std::sync::atomic::AtomicUsize::new(0), most }
    }

    pub fn take(&'static self) -> Option<Slot> {
        use std::sync::atomic::Ordering::SeqCst;
        self.out.fetch_update(SeqCst, SeqCst, |out| (out < self.most).then_some(out + 1)).ok().map(|_| Slot(self))
    }
}

impl Drop for Slot {
    fn drop(&mut self) {
        self.0.out.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
    }
}

pub fn on_slot<T: Send + 'static>(slot: Slot, name: &str, wait: std::time::Duration, work: impl FnOnce() -> T + Send + 'static) -> Option<T> {
    let (tell, heard) = std::sync::mpsc::channel();
    std::thread::Builder::new().name(name.into()).spawn(move || {
        let _slot = slot;
        let _ = tell.send(work());
    }).ok()?;
    heard.recv_timeout(wait).ok()
}

pub fn soon(then: impl FnOnce() + 'static) {
    let mut then = Some(then);
    let id = wait_for(Box::new(move |_| {
        if let Some(then) = then.take() {
            then();
        }
        false
    }));
    answer(id, Box::new(()));
}

pub fn every(gap: std::time::Duration, mut tick: impl FnMut() -> bool + 'static) {
    let (stop, stopped) = std::sync::mpsc::channel::<()>();
    let id = wait_for(Box::new(move |_| {
        let more = tick();
        if !more {
            let _ = stop.send(());
        }
        more
    }));
    std::thread::spawn(move || {
        while let Err(std::sync::mpsc::RecvTimeoutError::Timeout) = stopped.recv_timeout(gap) {
            answer(id, Box::new(()));
        }
    });
}

pub fn vouched(start: impl FnOnce(Option<String>) + 'static) {
    let vouch = VOUCH.with(|v| v.borrow().clone());
    match vouch {
        Some(vouch) => vouch(Box::new(start)),
        None => start(None),
    }
}

pub type Vouch = Rc<dyn Fn(Box<dyn FnOnce(Option<String>)>)>;

thread_local! {
    static VOUCH: RefCell<Option<Vouch>> = const { RefCell::new(None) };
}

pub fn vouch_with(vouch: Vouch) {
    VOUCH.with(|v| *v.borrow_mut() = Some(vouch));
}

pub fn pipe() -> Option<(OwnedFd, OwnedFd)> {
    crate::sys::fd::pipe(libc::O_CLOEXEC)
}
