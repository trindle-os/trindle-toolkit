//! Drag and drop with other programs, as the compositor speaks of it: what another program holds over one of this program's windows, and what this program holds over another's. The rules, with their tests, are in `toolkit::dnd`; this file only passes them to and from the compositor.

use std::os::fd::{AsFd, OwnedFd};
use std::path::PathBuf;
use std::sync::{mpsc, Arc, Mutex};
use std::time::Instant;

use wayland_client::backend::ObjectId;
use wayland_client::protocol::wl_data_device_manager::DndAction;
use wayland_client::WEnum;

use super::*;
use crate::dnd::{self, End, Given, Incoming, Outgoing, Pull, Wanted, Way, Ways};

const PICTURE_AWAY: i32 = 12;
static SENDING: crate::common::run::Slots = crate::common::run::Slots::new(dnd::MOST_SENDING);
static PULLING: crate::common::run::Slots = crate::common::run::Slots::new(dnd::MOST_SENDING);

fn portal_wanted() -> bool {
    !(cfg!(any(test, feature = "testing")) && std::env::var_os("TRINDLE_TEST_NO_PORTAL").is_some())
}

pub struct Dragged {
    pub files: Vec<PathBuf>,
    pub text: Option<String>,
    pub typed: Vec<(String, Vec<u8>)>,
    pub right: bool,
    pub loops: bool,
    pub picture: Option<tiny_skia::Pixmap>,
    pub scale: u32,
}

pub(super) struct HeldIn {
    win: u64,
    serial: u32,
    offer: wl_data_offer::WlDataOffer,
    at: (f32, f32),
}

pub(super) struct Pulling {
    held: HeldIn,
    pull: Pull,
}

pub(super) struct Taking {
    id: u64,
    win: u64,
    at: (f32, f32),
    offer: wl_data_offer::WlDataOffer,
    way: Way,
    ways: Ways,
}

pub(super) enum Back {
    Arrived { id: u64, given: Given },
    Keyed { source: ObjectId, key: String, bus: zbus::blocking::Connection },
}

struct Picture {
    surface: wl_surface::WlSurface,
    buffer: wl_buffer::WlBuffer,
    _memory: shm::Memory,
}

pub(super) struct HeldOut {
    win: u64,
    source: wl_data_source::WlDataSource,
    picture: Option<Picture>,
    out: Outgoing,
    loops: bool,
    button: Button,
    touch: bool,
    over: Option<(u64, (f32, f32))>,
    own: Option<wl_data_offer::WlDataOffer>,
    modern: bool,
}

impl HeldOut {
    fn clear(self) {
        if let Some(o) = self.own {
            o.destroy();
        }
        self.source.destroy();
        if let Some(p) = self.picture {
            p.surface.destroy();
            p.buffer.destroy();
        }
    }
}

pub(super) struct Sent {
    uri: String,
    text: String,
    typed: Vec<(String, Vec<u8>)>,
    key: Mutex<Option<(String, zbus::blocking::Connection)>>,
    sends: Arc<dnd::Sends>,
}

fn bits(action: WEnum<DndAction>) -> u32 {
    match action {
        WEnum::Value(v) => v.bits(),
        WEnum::Unknown(u) => u,
    }
}

fn action(ways: Ways) -> DndAction {
    DndAction::from_bits_truncate(ways.bits())
}

fn with<R>(offer: &wl_data_offer::WlDataOffer, f: impl FnOnce(&mut Incoming) -> R) -> Option<R> {
    offer.data::<Mutex<Incoming>>().and_then(|i| i.lock().ok().map(|mut i| f(&mut i)))
}

pub(super) fn kinds(offer: &wl_data_offer::WlDataOffer) -> Vec<String> {
    with(offer, |i| i.kinds().to_vec()).unwrap_or_default()
}

impl Dispatch<wl_data_offer::WlDataOffer, Mutex<Incoming>> for State {
    fn event(_: &mut Self, _: &wl_data_offer::WlDataOffer, event: wl_data_offer::Event, about: &Mutex<Incoming>,
             _: &Connection, _: &QueueHandle<Self>) {
        let Ok(mut about) = about.lock() else { return };
        match event {
            wl_data_offer::Event::Offer { mime_type } => about.kind(mime_type),
            wl_data_offer::Event::SourceActions { source_actions } => about.source_ways(bits(source_actions)),
            wl_data_offer::Event::Action { dnd_action } => about.chosen(bits(dnd_action)),
            _ => {}
        }
    }
}

pub(super) fn device_event(state: &mut State, event: wl_data_device::Event) {
    use wl_data_device::Event as E;
    match event {
        E::Enter { serial, surface, x, y, id } => drag_enter(state, serial, &surface, (x as f32, y as f32), id),
        E::Motion { x, y, .. } => drag_motion(state, x as f32, y as f32),
        E::Leave => drag_leave(state),
        E::Drop => drag_drop(state),
        _ => {}
    }
}

fn drag_enter(state: &mut State, serial: u32, surface: &wl_surface::WlSurface, (x, y): (f32, f32), id: Option<wl_data_offer::WlDataOffer>) {
    if let Some(old) = state.held_in.take() {
        old.offer.destroy();
    }
    let Some(win) = win_of(surface).filter(|w| state.windows.iter().any(|v| v.id == *w)) else {
        if let Some(o) = id {
            o.destroy();
        }
        return;
    };
    if let Some(out) = state.held_out.as_mut().filter(|o| o.loops) {
        if let Some(o) = out.own.take() {
            o.destroy();
        }
        out.over = Some((win, (x, y)));
        if let Some(offer) = id {
            let first = with(&offer, |i| i.kinds().first().cloned()).flatten();
            offer.accept(serial, first);
            if offer.version() >= 3 {
                offer.set_actions(action(Ways::ALL), DndAction::Copy);
            }
            out.own = Some(offer);
        }
        state.inbox.push((win, Input::PointerMove { x, y }));
        return;
    }
    let Some(offer) = id else { return };
    let (offered, kinds) = with(&offer, |i| (i.offered(), i.kinds().to_vec())).unwrap_or_default();
    state.held_in = Some(HeldIn { win, serial, offer, at: (x, y) });
    state.inbox.push((win, Input::DragIn { x, y, offered, kinds }));
}

fn drag_motion(state: &mut State, x: f32, y: f32) {
    if let Some(out) = state.held_out.as_mut().filter(|o| o.loops) {
        if let Some((win, at)) = &mut out.over {
            *at = (x, y);
            state.inbox.push((*win, Input::PointerMove { x, y }));
        }
    } else if let Some(h) = &mut state.held_in {
        h.at = (x, y);
        state.inbox.push((h.win, Input::DragOver { x, y }));
    }
}

fn drag_leave(state: &mut State) {
    if let Some(out) = state.held_out.as_mut().filter(|o| o.loops) {
        if let Some(o) = out.own.take() {
            o.destroy();
        }
        if let Some((win, _)) = out.over.take() {
            state.inbox.push((win, Input::PointerOut));
        }
    } else if let Some(h) = state.held_in.take() {
        h.offer.destroy();
        state.inbox.push((h.win, Input::DragGone));
    }
}

fn drag_drop(state: &mut State) {
    if let Some(out) = state.held_out.take_if(|o| o.loops) {
        if let Some((win, (x, y))) = out.over {
            state.inbox.push((win, Input::Release { button: out.button, x, y }));
            if out.touch && win == out.win {
                state.inbox.push((win, Input::TouchCancel));
            }
        }
        if out.over.is_none_or(|(win, _)| win != out.win) {
            state.inbox.push((out.win, Input::DragEnded { took: None, short: None }));
        }
        out.clear();
        return;
    }
    let Some(h) = state.held_in.take() else { return };
    let modern = h.offer.version() >= 3;
    match with(&h.offer, |i| i.let_go(modern)).flatten() {
        Some(pull) => state.pulls.push(Pulling { held: h, pull }),
        None => {
            h.offer.destroy();
            state.inbox.push((h.win, Input::DragGone));
        }
    }
}

pub(super) fn forget(state: &mut State, win: u64) {
    if let Some(h) = state.held_in.take_if(|h| h.win == win) {
        h.offer.destroy();
    }
    if let Some(out) = state.held_out.take_if(|o| o.win == win) {
        out.clear();
    }
    if let Some(out) = &mut state.held_out {
        out.over.take_if(|(w, _)| *w == win);
    }
    state.pulls.retain(|p| {
        if p.held.win == win {
            p.held.offer.destroy();
        }
        p.held.win != win
    });
    state.taking.retain(|t| {
        if t.win == win {
            t.offer.destroy();
        }
        t.win != win
    });
}

pub(super) fn pointer_back(state: &mut State) {
    if let Some(out) = state.held_out.take_if(|o| !o.modern) {
        let win = out.win;
        out.clear();
        state.inbox.push((win, Input::DragEnded { took: None, short: None }));
    }
}

impl Dispatch<wl_data_source::WlDataSource, Sent> for State {
    fn event(state: &mut Self, source: &wl_data_source::WlDataSource, event: wl_data_source::Event, sent: &Sent,
             _: &Connection, _: &QueueHandle<Self>) {
        use wl_data_source::Event as E;
        fn mine<'a>(state: &'a mut State, source: &wl_data_source::WlDataSource) -> Option<&'a mut Outgoing> {
            state.held_out.as_mut().filter(|o| &o.source == source).map(|o| &mut o.out)
        }
        match event {
            E::Send { mime_type, fd } => {
                let bytes = if let Some((_, b)) = sent.typed.iter().find(|(k, _)| *k == mime_type) {
                    b.clone()
                } else if mime_type == dnd::URI_LIST {
                    sent.uri.clone().into_bytes()
                } else if dnd::PORTAL_KINDS.contains(&mime_type.as_str()) {
                    let key = sent.key.lock().ok().and_then(|k| k.as_ref().map(|k| k.0.clone())).unwrap_or_default();
                    [key.as_bytes(), &[0]].concat()
                } else {
                    sent.text.clone().into_bytes()
                };
                let late = sent.sends.begin_form(&mime_type);
                let sends = sent.sends.clone();
                let form = mime_type;
                write_on_thread(&SENDING, fd, bytes, move |got| sends.ended_form(late, &form, got));
            }
            E::Action { dnd_action } => {
                if let Some(out) = mine(state, source) {
                    out.action(bits(dnd_action));
                }
            }
            E::DndDropPerformed => {
                if let Some(out) = mine(state, source) {
                    out.performed();
                    sent.sends.let_go();
                }
            }
            E::DndFinished | E::Cancelled => {
                let Some(mut out) = state.held_out.take_if(|o| &o.source == source) else { return };
                let end = if matches!(event, E::DndFinished) { out.out.finished(&sent.sends) } else { out.out.cancelled() };
                let win = out.win;
                out.clear();
                let (took, short) = match end {
                    End::Took(taken) => (Some(taken), None),
                    End::Nothing => (None, None),
                    End::Short(way) => (None, Some(dnd::Short::Cut(way))),
                    End::Unshown(way) => (None, Some(dnd::Short::Unshown(way))),
                };
                state.inbox.push((win, Input::DragEnded { took, short }));
            }
            _ => {}
        }
    }
}

fn pipe() -> Option<(OwnedFd, OwnedFd)> {
    crate::sys::fd::pipe(libc::O_CLOEXEC)
}

fn picture_size(w: u32, h: u32) -> Option<(usize, i32, i32, i32)> {
    let (wide, high) = (i32::try_from(w).ok()?, i32::try_from(h).ok()?);
    let stride = wide.checked_mul(4)?;
    let len = usize::try_from(stride).ok()?.checked_mul(usize::try_from(h).ok()?)?;
    i32::try_from(len).ok()?;
    Some((len, wide, high, stride))
}

fn picture(state: &State, qh: &QueueHandle<State>, pixels: &tiny_skia::Pixmap) -> Option<Picture> {
    let (len, w, h, stride) = picture_size(pixels.width(), pixels.height())?;
    let mut memory = shm::Memory::new(c"trindle-drag", len)?;
    let bytes = memory.bytes_mut().get_mut(..len)?;
    bytes.copy_from_slice(pixels.data());
    to_compositor(bytes);
    let pool = state.shm.create_pool(memory.fd(), i32::try_from(len).ok()?, qh, ());
    let buffer = pool.create_buffer(0, w, h, stride, wl_shm::Format::Argb8888, qh, (0u64, 0usize));
    pool.destroy();
    let surface = state.compositor.create_surface(qh, 0u64);
    Some(Picture { surface, buffer, _memory: memory })
}

fn files_by_key(read: OwnedFd, end: Instant) -> mpsc::Receiver<Option<Vec<PathBuf>>> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let files = dnd::read_all(read, dnd::MOST_BYTES, end).ok()
            .and_then(|bytes| dnd::portal_key(&bytes))
            .and_then(|key| dnd::portal::retrieve(&key))
            .and_then(|paths| dnd::checked_paths(&paths).ok());
        let _ = tx.send(files);
    });
    rx
}

fn read_given(read: OwnedFd, side: Option<OwnedFd>, words: Option<OwnedFd>, keyed: bool, kind: dnd::Kind) -> Given {
    let end = Instant::now() + dnd::MOST_TIME;
    if keyed {
        let files = files_by_key(read, end).recv_timeout(dnd::MOST_TIME).ok().flatten();
        files.map_or(Given::Nothing(dnd::Why::NotFiles), Given::Files)
    } else {
        let side = side.map(|read| files_by_key(read, end));
        let words = words.map(|read| {
            let (tx, rx) = mpsc::channel();
            std::thread::spawn(move || {
                let _ = tx.send(dnd::read_all(read, dnd::MOST_BYTES, end).ok());
            });
            rx
        });
        let main = dnd::read_all(read, dnd::MOST_BYTES, end);
        let left = || end.saturating_duration_since(Instant::now());
        let files = side.and_then(|rx| rx.recv_timeout(left()).ok().flatten());
        let words = words.and_then(|rx| rx.recv_timeout(left()).ok().flatten());
        dnd::given(kind, main, files, words)
    }
}

fn press_to_drag_by(state: &mut State) -> Option<(u32, Button, bool)> {
    Some(match (state.pressed, state.touched.last().copied()) {
        (Some((p, button)), Some((_, t))) if !dnd::newer(t, p) => (p, button, false),
        (_, Some((id, t))) => {
            state.touches.retain(|(i, _)| *i != id);
            state.touched.retain(|(i, _)| *i != id);
            (t, Button::Left, true)
        }
        (Some((p, button)), None) => (p, button, false),
        (None, None) => return None,
    })
}

impl Picture {
    fn show(&self, pixels: &tiny_skia::Pixmap, scale: u32) {
        let scale = scale.max(1) as i32;
        self.surface.set_buffer_scale(scale);
        if self.surface.version() >= 5 {
            self.surface.attach(Some(&self.buffer), 0, 0);
            self.surface.offset(PICTURE_AWAY, PICTURE_AWAY);
        } else {
            self.surface.attach(Some(&self.buffer), PICTURE_AWAY, PICTURE_AWAY);
        }
        self.surface.damage_buffer(0, 0, pixels.width() as i32, pixels.height() as i32);
        self.surface.commit();
    }
}

impl Display {
    pub(super) fn start_pulls(&self) {
        let pulls = std::mem::take(&mut self.state.borrow_mut().pulls);
        for Pulling { held, pull } in pulls {
            let id = self.next.replace(self.next.get() + 1);
            let ways = with(&held.offer, |i| i.ways()).unwrap_or_default();
            let slot = PULLING.take();
            let (Some(slot), Some((read, write)), side) = (slot, pipe(), pull.portal.as_ref().and_then(|_| pipe())) else {
                held.offer.destroy();
                self.state.borrow_mut().inbox.push((held.win, Input::DragGone));
                continue;
            };
            held.offer.receive(pull.main.clone(), write.as_fd());
            let side = side.zip(pull.portal.clone()).map(|((read, write), kind)| {
                held.offer.receive(kind, write.as_fd());
                read
            });
            let words = pull.text.clone().and_then(|kind| pipe().map(|ends| (kind, ends))).map(|(kind, (read, write))| {
                held.offer.receive(kind, write.as_fd());
                read
            });
            let _ = self.conn.flush();
            drop(write);
            self.state.borrow_mut().taking.push(Taking { id, win: held.win, at: held.at, offer: held.offer, way: pull.way, ways });
            let back = self.back.clone();
            let kind = pull.kind;
            let keyed = dnd::PORTAL_KINDS.contains(&pull.main.as_str());
            std::thread::spawn(move || {
                let _slot = slot;
                let given = read_given(read, side, words, keyed, kind);
                back.send(Back::Arrived { id, given });
            });
        }
    }

    pub(super) fn back(&self, back: Back) {
        match back {
            Back::Arrived { id, given } => {
                let mut state = self.state.borrow_mut();
                let Some(t) = state.taking.iter().find(|t| t.id == id) else { return };
                let (win, (x, y), way) = (t.win, t.at, t.way);
                state.inbox.push((win, Input::Dropped { x, y, given, way, id }));
                drop(state);
                self.deliver();
                self.paint();
                let _ = self.conn.flush();
            }
            Back::Keyed { source, key, bus } => {
                let state = self.state.borrow();
                let Some(out) = state.held_out.as_ref().filter(|o| o.source.id() == source) else {
                    drop(state);
                    std::thread::spawn(move || dnd::portal::stop_on(&bus, &key));
                    return;
                };
                let Some(sent) = out.source.data::<Sent>() else { return };
                if let Ok(mut k) = sent.key.lock() {
                    *k = Some((key, bus));
                }
                for kind in dnd::PORTAL_KINDS {
                    out.source.offer(kind.into());
                }
                drop(state);
                let _ = self.conn.flush();
            }
        }
    }
}

impl Window {
    pub fn drop_answer(&self, wanted: Wanted) {
        let state = self.display.state.borrow();
        let Some(h) = state.held_in.as_ref().filter(|h| h.win == self.id) else { return };
        let Some(answer) = with(&h.offer, |i| i.answer(wanted)).flatten() else { return };
        h.offer.accept(h.serial, answer.accept);
        if h.offer.version() >= 3 {
            let rather = answer.rather.map_or(DndAction::None, |w| action(Ways::from_bits(w.bits())));
            h.offer.set_actions(action(answer.ways), rather);
        }
        drop(state);
        let _ = self.display.conn.flush();
    }

    pub fn drop_done(&self, id: u64, done: bool) {
        let mut state = self.display.state.borrow_mut();
        let Some(at) = state.taking.iter().position(|t| t.id == id && t.win == self.id) else { return };
        let t = state.taking.remove(at);
        drop(state);
        if done && t.offer.version() >= 3 {
            match t.way {
                Way::Ask if t.ways.has(Way::Copy) => {
                    t.offer.set_actions(DndAction::Copy, DndAction::Copy);
                    t.offer.finish();
                }
                Way::Ask => {}
                _ => t.offer.finish(),
            }
        }
        t.offer.destroy();
        let _ = self.display.conn.flush();
    }

    pub fn drag_out(&self, what: Dragged) -> bool {
        let qh = self.display.queue.borrow().handle();
        let mut state = self.display.state.borrow_mut();
        if state.held_out.is_some() || (what.files.is_empty() && what.text.is_none() && what.typed.is_empty()) {
            return false;
        }
        let (Some(manager), Some(device)) = (state.clipboard.clone(), state.device.clone()) else { return false };
        let Some((serial, button, touch)) = press_to_drag_by(&mut state) else { return false };
        let Some(surface) = state.win(self.id).map(|w| w.surface.clone()) else { return false };
        let words = what.text.is_some() || !what.files.is_empty();
        let text = what.text.clone().unwrap_or_else(|| dnd::path_lines(&what.files));
        let kinds: Vec<String> = what.typed.iter().map(|(k, _)| k.clone()).collect();
        let sends = Arc::new(if what.files.is_empty() { dnd::Sends::default() } else { dnd::Sends::for_files() });
        let source = manager.create_data_source(&qh, Sent { uri: dnd::uri_list(&what.files), text, typed: what.typed, key: Mutex::new(None), sends });
        for kind in kinds {
            source.offer(kind);
        }
        if !what.files.is_empty() {
            source.offer(dnd::URI_LIST.into());
        }
        for kind in dnd::TEXT_OUT.into_iter().filter(|_| words) {
            source.offer(kind.into());
        }
        let modern = manager.version() >= 3;
        let shift = state.layout.as_ref().is_some_and(|k| k.mods().shift);
        let (ways, writable) = dnd::out_plan(modern, !what.files.is_empty(), what.right, shift);
        if modern {
            source.set_actions(action(ways));
        }
        let picture = what.picture.as_ref().and_then(|p| picture(&state, &qh, p));
        device.start_drag(Some(&source), &surface, picture.as_ref().map(|p| &p.surface), serial);
        if let (Some(p), Some(pixels)) = (&picture, &what.picture) {
            p.show(pixels, what.scale);
        }
        let id = source.id();
        state.held_out = Some(HeldOut { win: self.id, source, picture, out: Outgoing::new(), loops: what.loops, button, touch, over: None, own: None, modern });
        drop(state);
        let _ = self.display.conn.flush();
        if !what.files.is_empty() && portal_wanted() {
            let (back, files) = (self.display.back.clone(), what.files);
            std::thread::spawn(move || {
                if let Some((key, bus)) = dnd::portal::start(&files, writable) {
                    back.send(Back::Keyed { source: id, key, bus });
                }
            });
        }
        true
    }

    pub fn dragging_out(&self) -> bool {
        self.display.state.borrow().held_out.as_ref().is_some_and(|o| o.win == self.id)
    }

    pub fn drag_stop(&self) {
        if let Some(out) = self.display.state.borrow_mut().held_out.take_if(|o| o.win == self.id) {
            out.clear();
        }
        let _ = self.display.conn.flush();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ways_become_the_compositors_numbers_and_back() {
        assert_eq!(action(Ways::ALL), DndAction::Copy | DndAction::Move | DndAction::Ask);
        assert_eq!(action(Ways::NONE), DndAction::None);
        assert_eq!(action(dnd::ways_out(true, false)), DndAction::Copy | DndAction::Ask);
        assert_eq!(bits(WEnum::Value(DndAction::Move)), Way::Move.bits());
        assert_eq!(bits(WEnum::Unknown(64)), 64);
    }

    #[test]
    fn a_picture_too_big_for_the_compositors_numbers_is_refused() {
        assert_eq!(picture_size(2, 3), Some((24, 2, 3, 8)));
        assert_eq!(picture_size(0, 5), Some((0, 0, 5, 0)));
        assert_eq!(picture_size(u32::MAX, 1), None);
        assert_eq!(picture_size(30_000, 30_000), None);
    }
}
