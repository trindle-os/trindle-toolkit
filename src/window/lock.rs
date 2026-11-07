//! Locking the session (ext-session-lock-v1): the order the requests must come in, and the lock itself.

use std::rc::Rc;

use wayland_client::protocol::wl_surface;
use wayland_client::{Connection, Dispatch, Proxy, QueueHandle};
use wayland_protocols::ext::session_lock::v1::client::{
    ext_session_lock_manager_v1::ExtSessionLockManagerV1,
    ext_session_lock_surface_v1::{self, ExtSessionLockSurfaceV1},
    ext_session_lock_v1::{self, ExtSessionLockV1},
};

use super::{Display, State, MOST_SIDE};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Asked,
    Locked,
    Finished,
    Unlocked,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Request {
    Lock,
    Surface(String),
    UnlockAndDestroy,
    Destroy,
    Sync,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refused {
    NotLocked,
    NotFinished,
    Covered,
    Over,
}

#[derive(Debug)]
pub struct Sequence {
    phase: Phase,
    screens: Vec<String>,
}

impl Sequence {
    pub fn start() -> (Sequence, Request) {
        (Sequence { phase: Phase::Asked, screens: Vec::new() }, Request::Lock)
    }

    pub fn phase(&self) -> Phase {
        self.phase
    }

    pub fn locked(&mut self) -> bool {
        let fits = self.phase == Phase::Asked;
        if fits {
            self.phase = Phase::Locked;
        }
        fits
    }

    pub fn finished(&mut self) {
        if matches!(self.phase, Phase::Asked | Phase::Locked) {
            self.phase = Phase::Finished;
        }
    }

    pub fn surface(&mut self, screen: &str) -> Result<Request, Refused> {
        if matches!(self.phase, Phase::Finished | Phase::Unlocked) {
            return Err(Refused::Over);
        }
        if self.screens.iter().any(|s| s == screen) {
            return Err(Refused::Covered);
        }
        self.screens.push(screen.to_string());
        Ok(Request::Surface(screen.to_string()))
    }

    pub fn uncovered(&mut self, screen: &str) {
        self.screens.retain(|s| s != screen);
    }

    pub fn unlock(&mut self) -> Result<[Request; 2], Refused> {
        match self.phase {
            Phase::Locked => {
                self.phase = Phase::Unlocked;
                Ok([Request::UnlockAndDestroy, Request::Sync])
            }
            Phase::Asked => Err(Refused::NotLocked),
            Phase::Finished | Phase::Unlocked => Err(Refused::Over),
        }
    }

    pub fn destroy(&mut self) -> Result<Request, Refused> {
        match self.phase {
            Phase::Finished => {
                self.phase = Phase::Unlocked;
                Ok(Request::Destroy)
            }
            Phase::Unlocked => Err(Refused::Over),
            _ => Err(Refused::NotFinished),
        }
    }
}

pub(super) struct Held {
    pub(super) object: ExtSessionLockV1,
    pub(super) seq: Sequence,
    changed: bool,
}

impl Display {
    pub fn has_session_lock(&self) -> bool {
        self.state.borrow().locker.is_some()
    }

    pub fn lock_session(self: &Rc<Self>) -> Option<SessionLock> {
        let qh = self.queue.borrow().handle();
        let mut state = self.state.borrow_mut();
        if state.lock.is_some() {
            return None;
        }
        let (seq, _) = Sequence::start();
        let object = state.locker.as_ref()?.lock(&qh, ());
        state.lock = Some(Held { object, seq, changed: false });
        drop(state);
        if self.conn.flush().is_err() {
            return None;
        }
        Some(SessionLock { display: self.clone() })
    }

    pub fn on_lock(&self, call: impl Fn(Phase) + 'static) {
        *self.on_lock.borrow_mut() = Some(Rc::new(call));
    }

    pub(super) fn lock_changed(&self, f: impl FnOnce(Phase)) {
        let mut state = self.state.borrow_mut();
        let Some(held) = state.lock.as_mut().filter(|h| h.changed) else { return };
        held.changed = false;
        let phase = held.seq.phase();
        drop(state);
        f(phase);
    }
}

pub struct SessionLock {
    display: Rc<Display>,
}

impl SessionLock {
    pub fn phase(&self) -> Phase {
        self.display.state.borrow().lock.as_ref().map_or(Phase::Unlocked, |h| h.seq.phase())
    }

    pub fn unlock(&self) -> bool {
        self.finish(|seq| seq.unlock().map(|r| r.to_vec()))
    }

    pub fn destroy(&self) -> bool {
        self.finish(|seq| seq.destroy().map(|r| vec![r]))
    }

    fn finish(&self, step: impl FnOnce(&mut Sequence) -> Result<Vec<Request>, Refused>) -> bool {
        let mut state = self.display.state.borrow_mut();
        let Some(mut held) = state.lock.take() else { return false };
        let requests = match step(&mut held.seq) {
            Ok(r) => r,
            Err(_) => {
                state.lock = Some(held);
                return false;
            }
        };
        let mut wait = false;
        for request in requests {
            match request {
                Request::UnlockAndDestroy => held.object.unlock_and_destroy(),
                Request::Destroy => held.object.destroy(),
                Request::Sync => wait = true,
                _ => {}
            }
        }
        drop(state);
        let _ = self.display.conn.flush();
        if wait {
            return self.display.queue.borrow_mut().roundtrip(&mut self.display.state.borrow_mut()).is_ok();
        }
        true
    }
}

impl State {
    pub(super) fn lock_surface(&mut self, screen: &str, surface: &wl_surface::WlSurface, qh: &QueueHandle<State>, id: u64) -> Option<ExtSessionLockSurfaceV1> {
        let output = self.screens.iter().find(|s| s.name == screen)?.output.clone();
        let held = self.lock.as_mut()?;
        held.seq.surface(screen).ok()?;
        Some(held.object.get_lock_surface(surface, &output, qh, id))
    }
}

impl Dispatch<ExtSessionLockManagerV1, ()> for State {
    fn event(_: &mut Self, _: &ExtSessionLockManagerV1, _: <ExtSessionLockManagerV1 as Proxy>::Event, _: &(), _: &Connection, _: &QueueHandle<Self>) {}
}

impl Dispatch<ExtSessionLockV1, ()> for State {
    fn event(state: &mut Self, _: &ExtSessionLockV1, event: ext_session_lock_v1::Event, _: &(), _: &Connection, _: &QueueHandle<Self>) {
        let Some(held) = state.lock.as_mut() else { return };
        match event {
            ext_session_lock_v1::Event::Locked => held.changed |= held.seq.locked(),
            ext_session_lock_v1::Event::Finished => {
                held.seq.finished();
                held.changed = true;
            }
            _ => {}
        }
    }
}

impl Dispatch<ExtSessionLockSurfaceV1, u64> for State {
    fn event(state: &mut Self, _: &ExtSessionLockSurfaceV1, event: ext_session_lock_surface_v1::Event, id: &u64, _: &Connection, _: &QueueHandle<Self>) {
        let Some(w) = state.win(*id) else { return };
        if let ext_session_lock_surface_v1::Event::Configure { serial, width, height } = event {
            w.ack = Some(serial);
            let size = (width.min(MOST_SIDE), height.min(MOST_SIDE));
            if !(w.configured && w.size == size && !w.dirty) {
                w.size = size;
                w.configured = true;
                w.dirty = true;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_lock_is_asked_for_then_surfaces_then_unlocked_in_that_order() {
        let (mut seq, first) = Sequence::start();
        assert_eq!(first, Request::Lock);
        assert_eq!(seq.surface("A"), Ok(Request::Surface("A".into())));
        assert_eq!(seq.surface("B"), Ok(Request::Surface("B".into())));
        assert_eq!(seq.unlock(), Err(Refused::NotLocked), "unlocking before the compositor says locked is an error");
        assert!(seq.locked());
        assert_eq!(seq.phase(), Phase::Locked);
        assert_eq!(seq.unlock(), Ok([Request::UnlockAndDestroy, Request::Sync]), "the compositor is waited for after unlocking");
        assert_eq!(seq.phase(), Phase::Unlocked);
        assert_eq!(seq.unlock(), Err(Refused::Over));
    }

    #[test]
    fn a_screen_is_covered_once_until_its_surface_is_gone() {
        let (mut seq, _) = Sequence::start();
        seq.surface("A").unwrap();
        assert_eq!(seq.surface("A"), Err(Refused::Covered));
        seq.uncovered("A");
        assert!(seq.surface("A").is_ok());
    }

    #[test]
    fn a_lock_the_compositor_ended_is_only_destroyed_and_never_unlocked() {
        let (mut seq, _) = Sequence::start();
        assert_eq!(seq.destroy(), Err(Refused::NotFinished), "destroying a live lock would leave the session locked for good");
        seq.finished();
        assert_eq!(seq.phase(), Phase::Finished);
        assert!(!seq.locked(), "a locked word after finished means nothing");
        assert_eq!(seq.unlock(), Err(Refused::Over));
        assert_eq!(seq.surface("A"), Err(Refused::Over));
        assert_eq!(seq.destroy(), Ok(Request::Destroy));
    }

    #[test]
    fn a_locked_session_ended_by_the_compositor_cannot_be_unlocked() {
        let (mut seq, _) = Sequence::start();
        assert!(seq.locked());
        seq.finished();
        assert_eq!(seq.unlock(), Err(Refused::Over));
        assert_eq!(seq.destroy(), Ok(Request::Destroy));
        assert!(!seq.locked());
    }
}
