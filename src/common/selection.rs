//! The clipboard as the desktop holds it, over ext-data-control or wlr-data-control (the same protocol under two names).

use std::collections::HashMap;
use std::io::{Read, Write};
use std::os::fd::{AsFd, AsRawFd, BorrowedFd, OwnedFd};
use std::sync::{mpsc, Arc};
use std::time::{Duration, Instant};

use wayland_client::backend::ObjectId;
use wayland_client::globals::GlobalListContents;
use wayland_client::protocol::{wl_registry, wl_seat};
use super::windowless::Windowless;
use wayland_client::{event_created_child, Connection, Dispatch, Proxy, QueueHandle};
use wayland_protocols::ext::data_control::v1::client::{
    ext_data_control_device_v1::{self, ExtDataControlDeviceV1},
    ext_data_control_manager_v1::ExtDataControlManagerV1,
    ext_data_control_offer_v1::{self, ExtDataControlOfferV1},
    ext_data_control_source_v1::{self, ExtDataControlSourceV1},
};
use wayland_protocols_wlr::data_control::v1::client::{
    zwlr_data_control_device_v1::{self, ZwlrDataControlDeviceV1},
    zwlr_data_control_manager_v1::ZwlrDataControlManagerV1,
    zwlr_data_control_offer_v1::{self, ZwlrDataControlOfferV1},
    zwlr_data_control_source_v1::{self, ZwlrDataControlSourceV1},
};

pub const TEXT: &[&str] = &["text/plain;charset=utf-8", "text/plain", "UTF8_STRING", "STRING", "TEXT"];
pub const PNG: &str = "image/png";
const SECRET: &str = "x-kde-passwordManagerHint";
const PATIENCE: Duration = Duration::from_secs(5);
static SENDERS: super::run::Slots = super::run::Slots::new(8);
pub const SECRET_FOR: Duration = Duration::from_secs(30);

fn choose(types: &[String]) -> Option<&str> {
    let has = |t: &str| types.iter().any(|o| o == t);
    TEXT.iter().copied().find(|t| has(t))
        .or_else(|| has(PNG).then_some(PNG))
        .or_else(|| types.iter().map(String::as_str).find(|t| t.starts_with("text/")))
}

enum Manager {
    Ext(ExtDataControlManagerV1),
    Wlr(ZwlrDataControlManagerV1),
}

enum Device {
    Ext(ExtDataControlDeviceV1),
    Wlr(ZwlrDataControlDeviceV1),
}

enum Source {
    Ext(ExtDataControlSourceV1),
    Wlr(ZwlrDataControlSourceV1),
}

#[derive(Clone)]
enum Offer {
    Ext(ExtDataControlOfferV1),
    Wlr(ZwlrDataControlOfferV1),
}

impl Offer {
    fn id(&self) -> ObjectId {
        match self {
            Offer::Ext(o) => o.id(),
            Offer::Wlr(o) => o.id(),
        }
    }

    fn receive(&self, mime: &str, fd: BorrowedFd) {
        match self {
            Offer::Ext(o) => o.receive(mime.into(), fd),
            Offer::Wlr(o) => o.receive(mime.into(), fd),
        }
    }

    fn destroy(&self) {
        match self {
            Offer::Ext(o) => o.destroy(),
            Offer::Wlr(o) => o.destroy(),
        }
    }
}

impl Source {
    fn offer(&self, mime: &str) {
        match self {
            Source::Ext(s) => s.offer(mime.into()),
            Source::Wlr(s) => s.offer(mime.into()),
        }
    }

    fn destroy(&self) {
        match self {
            Source::Ext(s) => s.destroy(),
            Source::Wlr(s) => s.destroy(),
        }
    }
}

#[derive(Default)]
struct State {
    types: HashMap<ObjectId, Vec<String>>,
    selection: Option<Offer>,
    fresh: bool,
    serving: HashMap<String, Arc<Vec<u8>>>,
    over: bool,
}

impl State {
    fn selected(&mut self, offer: Option<Offer>) {
        if let Some(old) = self.selection.take() {
            self.forget(&old);
        }
        self.selection = offer;
        self.fresh = true;
    }

    fn forget(&mut self, offer: &Offer) {
        self.types.remove(&offer.id());
        offer.destroy();
    }

    fn send(&self, mime: &str, fd: OwnedFd) {
        let Some(data) = self.serving.get(mime).cloned() else { return };
        let Some(slot) = SENDERS.take() else { return };
        let _ = std::thread::Builder::new().spawn(move || {
            let _slot = slot;
            write_by(fd, &data, Instant::now() + PATIENCE);
        });
    }
}

struct Desk {
    wl: Windowless<State>,
    state: State,
    manager: Manager,
    device: Device,
}

impl Desk {
    fn open() -> Result<Desk, &'static str> {
        let wl = Windowless::<State>::open()?;
        let qh = wl.queue.handle();
        let (globals, seat) = (&wl.globals, &wl.seat);
        let manager = match globals.bind::<ExtDataControlManagerV1, _, _>(&qh, 1..=1, ()) {
            Ok(m) => Manager::Ext(m),
            Err(_) => Manager::Wlr(globals.bind::<ZwlrDataControlManagerV1, _, _>(&qh, 1..=2, ())
                .map_err(|_| "this desktop does not share its clipboard")?),
        };
        let device = match &manager {
            Manager::Ext(m) => Device::Ext(m.get_data_device(seat, &qh, ())),
            Manager::Wlr(m) => Device::Wlr(m.get_data_device(seat, &qh, ())),
        };
        let mut desk = Desk { wl, state: State::default(), manager, device };
        desk.wl.queue.roundtrip(&mut desk.state).map_err(|_| "the desktop did not answer")?;
        Ok(desk)
    }

    fn secret(&self) -> Option<ObjectId> {
        let offer = self.state.selection.as_ref()?;
        is_secret(self.state.types.get(&offer.id())?).then(|| offer.id())
    }

    fn clear(&mut self) {
        match &self.device {
            Device::Ext(d) => d.set_selection(None),
            Device::Wlr(d) => d.set_selection(None),
        }
        let _ = self.wl.conn.flush();
    }

    fn dispatch_for(&mut self, wait: Option<Duration>) -> bool {
        self.wl.dispatch_for(&mut self.state, wait)
    }

    fn read(&mut self, limit: u64, secrets: bool) -> Option<Vec<u8>> {
        let offer = self.state.selection.clone()?;
        let types = self.state.types.get(&offer.id())?.clone();
        if !secrets && is_secret(&types) {
            return None;
        }
        self.fetch(&offer, choose(&types)?, limit)
    }

    fn read_kinds(&mut self, limit: u64, kinds: &[&str], fallback: bool, secrets: bool) -> Vec<(String, Vec<u8>)> {
        let Some(offer) = self.state.selection.clone() else { return Vec::new() };
        let Some(types) = self.state.types.get(&offer.id()).cloned() else { return Vec::new() };
        if !secrets && is_secret(&types) {
            return Vec::new();
        }
        let mut got: Vec<(String, Vec<u8>)> = kinds.iter().filter(|k| types.iter().any(|t| t == *k))
            .filter_map(|k| Some((k.to_string(), self.fetch(&offer, k, limit)?))).collect();
        if got.is_empty() && fallback {
            if let Some(kind) = choose(&types) {
                got.extend(self.fetch(&offer, kind, limit).map(|d| (kind.to_string(), d)));
            }
        }
        got
    }

    fn fetch(&mut self, offer: &Offer, kind: &str, limit: u64) -> Option<Vec<u8>> {
        let (read, write) = super::run::pipe()?;
        offer.receive(kind, write.as_fd());
        self.wl.conn.flush().ok()?;
        drop(write);
        read_by(read, limit.saturating_add(1), Instant::now() + PATIENCE).filter(|d| d.len() as u64 <= limit)
    }
}

fn ready(fd: &impl AsRawFd, events: i16, deadline: Instant) -> bool {
    loop {
        let left = deadline.saturating_duration_since(Instant::now());
        let mut p = libc::pollfd { fd: fd.as_raw_fd(), events, revents: 0 };
        let n = crate::sys::fd::poll(std::slice::from_mut(&mut p), left.as_millis().min(i32::MAX as u128) as i32);
        if n < 0 && std::io::Error::last_os_error().kind() == std::io::ErrorKind::Interrupted {
            continue;
        }
        return n > 0;
    }
}

fn nonblocking(fd: &OwnedFd) {
    let flags = crate::sys::fd::flags(fd.as_raw_fd());
    if flags >= 0 {
        crate::sys::fd::set_flags(fd.as_raw_fd(), flags | libc::O_NONBLOCK);
    }
}

fn read_by(fd: OwnedFd, limit: u64, deadline: Instant) -> Option<Vec<u8>> {
    nonblocking(&fd);
    let mut data = Vec::new();
    let mut buf = [0u8; 16384];
    let mut file = std::fs::File::from(fd);
    while (data.len() as u64) < limit {
        if !ready(&file, libc::POLLIN, deadline) {
            return None;
        }
        match file.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => data.extend_from_slice(&buf[..n.min((limit - data.len() as u64) as usize)]),
            Err(e) if matches!(e.kind(), std::io::ErrorKind::WouldBlock | std::io::ErrorKind::Interrupted) => {}
            Err(_) => return None,
        }
    }
    (!data.is_empty()).then_some(data)
}

fn write_by(fd: OwnedFd, data: &[u8], deadline: Instant) {
    nonblocking(&fd);
    let mut file = std::fs::File::from(fd);
    let mut done = 0;
    while done < data.len() {
        if !ready(&file, libc::POLLOUT, deadline) {
            return;
        }
        match file.write(&data[done..]) {
            Ok(n) => done += n,
            Err(e) if matches!(e.kind(), std::io::ErrorKind::WouldBlock | std::io::ErrorKind::Interrupted) => {}
            Err(_) => return,
        }
    }
}

fn is_secret(types: &[String]) -> bool {
    types.iter().any(|t| t == SECRET)
}

pub fn watch(limit: u64, keep: impl Fn(Vec<u8>), resting_by: impl FnOnce(Box<dyn Fn() -> bool + Send>)) -> i32 {
    let mut desk = match Desk::open() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("trindle: {e}");
            return 1;
        }
    };
    let mut wipe: Option<(ObjectId, Instant)> = None;
    let busy = Arc::new(std::sync::atomic::AtomicBool::new(true));
    let resting = busy.clone();
    resting_by(Box::new(move || !resting.load(std::sync::atomic::Ordering::SeqCst)));
    while !desk.state.over {
        busy.store(true, std::sync::atomic::Ordering::SeqCst);
        if std::mem::take(&mut desk.state.fresh) {
            wipe = desk.secret().map(|id| (id, Instant::now() + SECRET_FOR));
            if let Some(data) = desk.read(limit, false) {
                keep(data);
            }
        }
        if let Some((id, at)) = wipe.clone() {
            if Instant::now() >= at {
                wipe = None;
                if desk.secret() == Some(id) {
                    desk.clear();
                }
                continue;
            }
        }
        let wait = wipe.as_ref().map(|(_, at)| at.saturating_duration_since(Instant::now()));
        busy.store(wipe.is_some(), std::sync::atomic::Ordering::SeqCst);
        if !desk.dispatch_for(wait) {
            break;
        }
    }
    0
}

pub fn paste(limit: u64) -> Option<Vec<u8>> {
    Desk::open().ok()?.read(limit, true)
}

pub fn paste_kinds(limit: u64, kinds: &[&str]) -> Vec<(String, Vec<u8>)> {
    match Desk::open() {
        Ok(mut desk) => desk.read_kinds(limit, kinds, true, true),
        Err(_) => Vec::new(),
    }
}

pub struct Held {
    pub kinds: Vec<(String, Vec<u8>)>,
    pub copied_at: Option<Instant>,
}

struct Job {
    kinds: &'static [&'static str],
    limit: u64,
    back: mpsc::Sender<Held>,
}

pub struct Reader {
    jobs: mpsc::Sender<Job>,
}

impl Reader {
    pub fn start() -> Reader {
        let (jobs, rx) = mpsc::channel::<Job>();
        std::thread::spawn(move || {
            let Ok(mut desk) = Desk::open() else {
                while rx.recv().is_ok() {}
                return;
            };
            desk.state.fresh = false;
            let mut copied_at: Option<Instant> = None;
            loop {
                if desk.state.over {
                    while rx.recv().is_ok() {}
                    return;
                }
                if std::mem::take(&mut desk.state.fresh) {
                    copied_at = Some(Instant::now());
                }
                loop {
                    match rx.try_recv() {
                        Ok(job) => {
                            let kinds = desk.read_kinds(job.limit, job.kinds, false, false);
                            let _ = job.back.send(Held { kinds, copied_at });
                        }
                        Err(mpsc::TryRecvError::Empty) => break,
                        Err(mpsc::TryRecvError::Disconnected) => return,
                    }
                }
                if !desk.dispatch_for(Some(Duration::from_millis(200))) {
                    while rx.recv().is_ok() {}
                    return;
                }
            }
        });
        Reader { jobs }
    }

    pub fn ask(&self, kinds: &'static [&'static str], limit: u64) -> Option<Held> {
        let (back, rx) = mpsc::channel();
        self.jobs.send(Job { kinds, limit, back }).ok()?;
        rx.recv_timeout(PATIENCE * 2).ok()
    }
}

pub fn offer(data: Vec<u8>, types: &[&str], ready: impl FnOnce()) -> i32 {
    let data = Arc::new(data);
    offer_kinds(types.iter().map(|t| (t.to_string(), data.clone())).collect(), ready)
}

pub fn offer_kinds(kinds: Vec<(String, Arc<Vec<u8>>)>, ready: impl FnOnce()) -> i32 {
    let mut desk = match Desk::open() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("trindle: {e}");
            return 1;
        }
    };
    let qh = desk.wl.queue.handle();
    let source = match &desk.manager {
        Manager::Ext(m) => Source::Ext(m.create_data_source(&qh, ())),
        Manager::Wlr(m) => Source::Wlr(m.create_data_source(&qh, ())),
    };
    for (t, _) in &kinds {
        source.offer(t);
    }
    match (&desk.device, &source) {
        (Device::Ext(d), Source::Ext(s)) => d.set_selection(Some(s)),
        (Device::Wlr(d), Source::Wlr(s)) => d.set_selection(Some(s)),
        _ => unreachable!("a source is always made by the device's own manager"),
    }
    desk.state.serving = kinds.into_iter().collect();
    if desk.wl.queue.roundtrip(&mut desk.state).is_err() || desk.state.over {
        eprintln!("trindle: the desktop would not take it");
        return 1;
    }
    ready();
    while !desk.state.over {
        if desk.wl.queue.blocking_dispatch(&mut desk.state).is_err() {
            break;
        }
    }
    source.destroy();
    let _ = desk.wl.conn.flush();
    0
}

macro_rules! data_control {
    ($kind:ident, $manager:ty, $device:ident, $Device:ty, $offer:ident, $Offer:ty, $source:ident, $Source:ty) => {
        impl Dispatch<$Device, ()> for State {
            fn event(state: &mut Self, _: &$Device, event: $device::Event, _: &(), _: &Connection,
                     _: &QueueHandle<Self>) {
                match event {
                    $device::Event::DataOffer { id } => {
                        state.types.insert(id.id(), Vec::new());
                    }
                    $device::Event::Selection { id } => state.selected(id.map(Offer::$kind)),
                    $device::Event::PrimarySelection { id: Some(id) } => state.forget(&Offer::$kind(id)),
                    $device::Event::Finished => state.over = true,
                    _ => {}
                }
            }

            event_created_child!(State, $Device, [$device::EVT_DATA_OFFER_OPCODE => ($Offer, ())]);
        }

        impl Dispatch<$Offer, ()> for State {
            fn event(state: &mut Self, offer: &$Offer, event: $offer::Event, _: &(), _: &Connection,
                     _: &QueueHandle<Self>) {
                if let $offer::Event::Offer { mime_type } = event {
                    state.types.entry(offer.id()).or_default().push(mime_type);
                }
            }
        }

        impl Dispatch<$Source, ()> for State {
            fn event(state: &mut Self, _: &$Source, event: $source::Event, _: &(), _: &Connection,
                     _: &QueueHandle<Self>) {
                match event {
                    $source::Event::Send { mime_type, fd } => state.send(&mime_type, fd),
                    $source::Event::Cancelled => state.over = true,
                    _ => {}
                }
            }
        }

        impl Dispatch<$manager, ()> for State {
            fn event(_: &mut Self, _: &$manager, _: <$manager as Proxy>::Event, _: &(), _: &Connection,
                     _: &QueueHandle<Self>) {}
        }
    };
}

data_control!(Ext, ExtDataControlManagerV1, ext_data_control_device_v1, ExtDataControlDeviceV1,
              ext_data_control_offer_v1, ExtDataControlOfferV1, ext_data_control_source_v1, ExtDataControlSourceV1);
data_control!(Wlr, ZwlrDataControlManagerV1, zwlr_data_control_device_v1, ZwlrDataControlDeviceV1,
              zwlr_data_control_offer_v1, ZwlrDataControlOfferV1, zwlr_data_control_source_v1, ZwlrDataControlSourceV1);

impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for State {
    fn event(_: &mut Self, _: &wl_registry::WlRegistry, _: wl_registry::Event, _: &GlobalListContents,
             _: &Connection, _: &QueueHandle<Self>) {}
}

impl Dispatch<wl_seat::WlSeat, ()> for State {
    fn event(_: &mut Self, _: &wl_seat::WlSeat, _: wl_seat::Event, _: &(), _: &Connection, _: &QueueHandle<Self>) {}
}

#[cfg(test)]
mod tests {
    use super::{choose, is_secret};

    #[test]
    fn a_password_managers_mark_is_seen() {
        assert!(is_secret(&types(&["text/plain", "x-kde-passwordManagerHint"])));
        assert!(!is_secret(&types(&["text/plain", "UTF8_STRING"])));
    }

    fn types(list: &[&str]) -> Vec<String> {
        list.iter().map(|t| t.to_string()).collect()
    }

    #[test]
    fn prefers_text_then_pictures() {
        assert_eq!(choose(&types(&["TEXT", "text/plain;charset=utf-8", "image/png"])), Some("text/plain;charset=utf-8"));
        assert_eq!(choose(&types(&["UTF8_STRING", "STRING"])), Some("UTF8_STRING"));
        assert_eq!(choose(&types(&["text/html", "image/png"])), Some("image/png"));
        assert_eq!(choose(&types(&["text/uri-list", "x-special/gnome-copied-files"])), Some("text/uri-list"));
        assert_eq!(choose(&types(&["application/x-qt-image"])), None);
        assert_eq!(choose(&[]), None);
    }
}
