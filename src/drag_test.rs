//! Kept whole on purpose: Trindle's own tests (common/xwm_test.rs) use the parts the toolkit's tests do not.

use std::collections::HashMap;
use std::io::{Read, Write};
use std::os::fd::{AsFd, AsRawFd};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use wayland_client::backend::ObjectId;
use wayland_client::globals::{registry_queue_init, GlobalList, GlobalListContents};
use wayland_client::protocol::wl_data_device_manager::DndAction;
use wayland_client::protocol::{
    wl_buffer, wl_compositor, wl_data_device, wl_data_device_manager, wl_data_offer, wl_data_source, wl_output,
    wl_pointer, wl_registry, wl_seat, wl_shm, wl_shm_pool, wl_surface,
};
use wayland_client::{delegate_noop, event_created_child, Connection, Dispatch, EventQueue, Proxy, QueueHandle, WEnum};
use wayland_protocols::wp::primary_selection::zv1::client::{
    zwp_primary_selection_device_manager_v1::ZwpPrimarySelectionDeviceManagerV1,
    zwp_primary_selection_device_v1::{self, ZwpPrimarySelectionDeviceV1},
    zwp_primary_selection_offer_v1::ZwpPrimarySelectionOfferV1,
    zwp_primary_selection_source_v1::{self, ZwpPrimarySelectionSourceV1},
};
use wayland_protocols::xdg::shell::client::{xdg_surface, xdg_toplevel, xdg_wm_base};
use wayland_protocols_misc::zwp_virtual_keyboard_v1::client::{
    zwp_virtual_keyboard_manager_v1::ZwpVirtualKeyboardManagerV1, zwp_virtual_keyboard_v1::ZwpVirtualKeyboardV1,
};
use wayland_protocols_wlr::virtual_pointer::v1::client::{
    zwlr_virtual_pointer_manager_v1::ZwlrVirtualPointerManagerV1, zwlr_virtual_pointer_v1::ZwlrVirtualPointerV1,
};

use super::event::{Loop, Post};
use super::window::Display;

pub const URI_LIST: &str = "text/uri-list";
pub const TEXT: &str = "text/plain;charset=utf-8";
pub const LEFT_BUTTON: u32 = 0x110;
pub const SHIFT: u32 = 42;

pub fn harness() -> Option<PathBuf> {
    let run = std::env::var_os("TRINDLE_DND_TEST").map(PathBuf::from)?;
    std::thread::sleep(Duration::from_millis(700));
    Some(run)
}

fn now_ms() -> u32 {
    static START: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();
    START.get_or_init(Instant::now).elapsed().as_millis() as u32 + 1000
}

fn bits(action: WEnum<DndAction>) -> u32 {
    match action {
        WEnum::Value(v) => v.bits(),
        WEnum::Unknown(u) => u,
    }
}

#[derive(Default)]
pub struct Told {
    configured: bool,
    pub screen: (u32, u32),
    pub pointer: Option<(f64, f64)>,
    pub pressed: Option<u32>,
    device: Option<wl_data_device::WlDataDevice>,
    kinds: HashMap<ObjectId, Vec<String>>,
    ways: HashMap<ObjectId, u32>,
    pub answer: Option<(String, u32, u32)>,
    pub held: Option<wl_data_offer::WlDataOffer>,
    pub action: u32,
    pub dropped: Option<wl_data_offer::WlDataOffer>,
    pub serving: HashMap<String, Vec<u8>>,
    pub never_ends: bool,
    kept_open: Vec<std::fs::File>,
    pub asked: Vec<String>,
    pub source_action: u32,
    pub performed: bool,
    pub finished: usize,
    pub cancelled: usize,
}

pub struct Plain {
    pub conn: Connection,
    queue: EventQueue<Told>,
    pub told: Told,
    globals: GlobalList,
    surface: wl_surface::WlSurface,
    manager: wl_data_device_manager::WlDataDeviceManager,
    source: Option<wl_data_source::WlDataSource>,
    copied: Option<wl_data_source::WlDataSource>,
    primary: Option<(ZwpPrimarySelectionDeviceManagerV1, ZwpPrimarySelectionDeviceV1)>,
    primary_copied: Option<ZwpPrimarySelectionSourceV1>,
    _keep: (xdg_surface::XdgSurface, xdg_toplevel::XdgToplevel, wl_buffer::WlBuffer, crate::common::shm::Memory),
}

impl Plain {
    pub fn open(socket: &Path, title: &str, size: (usize, usize)) -> Plain {
        let stream = std::os::unix::net::UnixStream::connect(socket).expect("the harness's socket");
        let conn = Connection::from_socket(stream).unwrap();
        let (globals, mut queue) = registry_queue_init::<Told>(&conn).unwrap();
        let qh = queue.handle();
        let compositor: wl_compositor::WlCompositor = globals.bind(&qh, 1..=4, ()).unwrap();
        let shm: wl_shm::WlShm = globals.bind(&qh, 1..=1, ()).unwrap();
        let base: xdg_wm_base::XdgWmBase = globals.bind(&qh, 1..=1, ()).unwrap();
        let seat: wl_seat::WlSeat = globals.bind(&qh, 1..=5, ()).unwrap();
        let _pointer = seat.get_pointer(&qh, ());
        let _screen: wl_output::WlOutput = globals.bind(&qh, 1..=2, ()).unwrap();
        let manager: wl_data_device_manager::WlDataDeviceManager = globals.bind(&qh, 3..=3, ()).unwrap();
        let mut told = Told { device: Some(manager.get_data_device(&seat, &qh, ())), ..Told::default() };
        let surface = compositor.create_surface(&qh, ());
        let xdg = base.get_xdg_surface(&surface, &qh, ());
        let toplevel = xdg.get_toplevel(&qh, ());
        toplevel.set_title(title.into());
        toplevel.set_app_id("org.example.Plain".into());
        surface.commit();
        let end = Instant::now() + Duration::from_secs(10);
        while !told.configured || told.screen == (0, 0) {
            assert!(Instant::now() < end, "the plain window was never given a size");
            queue.blocking_dispatch(&mut told).unwrap();
        }
        let (w, h) = size;
        let mut memory = crate::common::shm::Memory::new(c"plain-window", w * h * 4).unwrap();
        memory.bytes_mut().fill(0x60);
        let pool = shm.create_pool(memory.fd(), (w * h * 4) as i32, &qh, ());
        let buffer = pool.create_buffer(0, w as i32, h as i32, (w * 4) as i32, wl_shm::Format::Xrgb8888, &qh, ());
        surface.attach(Some(&buffer), 0, 0);
        surface.commit();
        let primary = globals.bind::<ZwpPrimarySelectionDeviceManagerV1, _, _>(&qh, 1..=1, ()).ok()
            .map(|m| { let d = m.get_device(&seat, &qh, ()); (m, d) });
        let mut plain = Plain { conn, queue, told, globals, surface, manager, source: None, copied: None, primary,
                                primary_copied: None, _keep: (xdg, toplevel, buffer, memory) };
        plain.settle(300);
        plain
    }

    pub fn turn(&mut self) {
        self.queue.dispatch_pending(&mut self.told).unwrap();
        self.conn.flush().unwrap();
        let Some(guard) = self.conn.prepare_read() else { return };
        let mut fd = libc::pollfd { fd: guard.connection_fd().as_raw_fd(), events: libc::POLLIN, revents: 0 };
        if crate::sys::fd::poll(std::slice::from_mut(&mut fd), 5) > 0 {
            guard.read().unwrap();
        }
        self.queue.dispatch_pending(&mut self.told).unwrap();
        self.conn.flush().unwrap();
    }

    pub fn settle(&mut self, ms: u64) {
        let end = Instant::now() + Duration::from_millis(ms);
        while Instant::now() < end {
            self.turn();
        }
    }

    pub fn until(&mut self, what: &str, done: impl Fn(&Told) -> bool) {
        let end = Instant::now() + Duration::from_secs(10);
        while !done(&self.told) {
            assert!(Instant::now() < end, "never happened: {what}");
            self.turn();
        }
    }

    pub fn mouse(&self) -> Mouse {
        let qh = self.queue.handle();
        let pointer = self.globals.bind::<ZwlrVirtualPointerManagerV1, _, _>(&qh, 1..=2, ()).expect("a stand-in mouse")
            .create_virtual_pointer(None, &qh, ());
        Mouse { pointer, conn: self.conn.clone(), at: (0.0, 0.0), screen: self.told.screen }
    }

    pub fn drag(&mut self, ways: u32) {
        let qh = self.queue.handle();
        let serial = self.told.pressed.expect("a button is held on the plain window");
        let source = self.manager.create_data_source(&qh, ());
        let mut kinds: Vec<&String> = self.told.serving.keys().collect();
        kinds.sort();
        for kind in kinds {
            source.offer(kind.clone());
        }
        source.set_actions(DndAction::from_bits_truncate(ways));
        self.told.device.as_ref().unwrap().start_drag(Some(&source), &self.surface, None, serial);
        self.told.source_action = 0;
        self.told.performed = false;
        self.told.asked.clear();
        self.source = Some(source);
        self.conn.flush().unwrap();
    }

    pub fn copy(&mut self, on: bool) {
        let qh = self.queue.handle();
        let device = self.told.device.clone().unwrap();
        if let Some(old) = self.copied.take() {
            old.destroy();
        }
        if on {
            let source = self.manager.create_data_source(&qh, ());
            let mut kinds: Vec<&String> = self.told.serving.keys().collect();
            kinds.sort();
            for kind in kinds {
                source.offer(kind.clone());
            }
            device.set_selection(Some(&source), 1);
            self.copied = Some(source);
        } else {
            device.set_selection(None, 1);
        }
        self.conn.flush().unwrap();
    }

    pub fn copy_primary(&mut self, on: bool) {
        let qh = self.queue.handle();
        let (manager, device) = self.primary.clone().expect("the compositor offers a primary selection");
        if let Some(old) = self.primary_copied.take() {
            old.destroy();
        }
        if on {
            let source = manager.create_source(&qh, ());
            let mut kinds: Vec<&String> = self.told.serving.keys().collect();
            kinds.sort();
            for kind in kinds {
                source.offer(kind.clone());
            }
            device.set_selection(Some(&source), 1);
            self.primary_copied = Some(source);
        } else {
            device.set_selection(None, 1);
        }
        self.conn.flush().unwrap();
    }

    pub fn kinds(&self) -> Vec<String> {
        let offer = self.told.held.as_ref().or(self.told.dropped.as_ref());
        offer.and_then(|o| self.told.kinds.get(&o.id())).cloned().unwrap_or_default()
    }

    pub fn ways(&self) -> u32 {
        let offer = self.told.held.as_ref().or(self.told.dropped.as_ref());
        offer.and_then(|o| self.told.ways.get(&o.id())).copied().unwrap_or(0)
    }

    pub fn read(&mut self, kind: &str) -> Vec<u8> {
        self.read_while(kind, || {})
    }

    pub fn read_while(&mut self, kind: &str, also: impl FnMut()) -> Vec<u8> {
        let offer = self.told.dropped.clone().expect("something was let go on the plain window");
        let rx = self.start_read(&offer, kind);
        self.wait_for(&rx, also)
    }

    pub fn ask_early(&mut self, kind: &str) -> mpsc::Receiver<Vec<u8>> {
        let offer = self.told.held.clone().expect("something is held over the plain window");
        self.start_read(&offer, kind)
    }

    fn start_read(&mut self, offer: &wl_data_offer::WlDataOffer, kind: &str) -> mpsc::Receiver<Vec<u8>> {
        let (read, write) = crate::common::run::pipe().unwrap();
        offer.receive(kind.into(), write.as_fd());
        self.conn.flush().unwrap();
        drop(write);
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let mut data = Vec::new();
            let _ = std::fs::File::from(read).read_to_end(&mut data);
            let _ = tx.send(data);
        });
        rx
    }

    pub fn wait_for(&mut self, rx: &mpsc::Receiver<Vec<u8>>, mut also: impl FnMut()) -> Vec<u8> {
        let end = Instant::now() + Duration::from_secs(10);
        loop {
            if let Ok(data) = rx.try_recv() {
                return data;
            }
            assert!(Instant::now() < end, "what was let go never came");
            also();
            self.turn();
        }
    }

    pub fn finish(&mut self) {
        let offer = self.told.dropped.take().expect("something was let go on the plain window");
        offer.finish();
        offer.destroy();
        self.conn.flush().unwrap();
    }
}

pub struct Mouse {
    pointer: ZwlrVirtualPointerV1,
    conn: Connection,
    pub at: (f32, f32),
    pub screen: (u32, u32),
}

impl Mouse {
    pub fn place(&mut self, x: f32, y: f32) {
        self.at = (x, y);
        self.pointer.motion_absolute(now_ms(), x.round().max(0.0) as u32, y.round().max(0.0) as u32, self.screen.0, self.screen.1);
        self.pointer.frame();
        let _ = self.conn.flush();
    }

    pub fn button(&self, button: u32, down: bool) {
        let state = if down { wl_pointer::ButtonState::Pressed } else { wl_pointer::ButtonState::Released };
        self.pointer.button(now_ms(), button, state);
        self.pointer.frame();
        let _ = self.conn.flush();
    }

    pub fn glide(&mut self, to: (f32, f32), plain: &mut Plain) {
        let from = self.at;
        let steps = (((to.0 - from.0).hypot(to.1 - from.1)) / 12.0).ceil().max(1.0) as usize;
        for i in 1..=steps {
            let t = i as f32 / steps as f32;
            self.place(from.0 + (to.0 - from.0) * t, from.1 + (to.1 - from.1) * t);
            plain.settle(12);
        }
        plain.settle(150);
    }
}

pub struct Keys {
    keys: ZwpVirtualKeyboardV1,
    conn: Connection,
    _queue: EventQueue<Told>,
}

impl Keys {
    pub fn outer(run: &Path) -> Option<Keys> {
        use xkbcommon::xkb;
        let outer = std::env::var_os("TRINDLE_OUTER_DISPLAY")?;
        let conn = Connection::from_socket(std::os::unix::net::UnixStream::connect(run.join(outer)).ok()?).ok()?;
        let (globals, mut queue) = registry_queue_init::<Told>(&conn).ok()?;
        let qh = queue.handle();
        let seat: wl_seat::WlSeat = globals.bind(&qh, 1..=1, ()).ok()?;
        let keys = globals.bind::<ZwpVirtualKeyboardManagerV1, _, _>(&qh, 1..=1, ()).ok()?.create_virtual_keyboard(&seat, &qh, ());
        let context = xkb::Context::new(xkb::CONTEXT_NO_FLAGS);
        let map = xkb::Keymap::new_from_names(&context, "", "pc105", "gb", "", None, xkb::KEYMAP_COMPILE_NO_FLAGS)?;
        let text = map.get_as_string(xkb::KEYMAP_FORMAT_TEXT_V1);
        let memory = crate::common::shm::Memory::new(c"test-keymap", text.len() + 1)?;
        std::fs::File::from(memory.fd().try_clone_to_owned().ok()?).write_all(text.as_bytes()).ok()?;
        keys.keymap(1, memory.fd(), text.len() as u32 + 1);
        queue.roundtrip(&mut Told::default()).ok()?;
        Some(Keys { keys, conn, _queue: queue })
    }

    pub fn key(&self, key: u32, down: bool) {
        self.keys.key(now_ms(), key, u32::from(down));
        let _ = self.conn.flush();
    }
}

impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for Told {
    fn event(_: &mut Self, _: &wl_registry::WlRegistry, _: wl_registry::Event, _: &GlobalListContents, _: &Connection,
             _: &QueueHandle<Self>) {}
}
delegate_noop!(Told: ignore wl_compositor::WlCompositor);
delegate_noop!(Told: ignore wl_shm::WlShm);
delegate_noop!(Told: ignore wl_shm_pool::WlShmPool);
delegate_noop!(Told: ignore wl_buffer::WlBuffer);
delegate_noop!(Told: ignore wl_surface::WlSurface);
delegate_noop!(Told: ignore wl_seat::WlSeat);

impl Dispatch<wl_output::WlOutput, ()> for Told {
    fn event(told: &mut Self, _: &wl_output::WlOutput, event: wl_output::Event, _: &(), _: &Connection, _: &QueueHandle<Self>) {
        if let wl_output::Event::Mode { flags: WEnum::Value(flags), width, height, .. } = event {
            if flags.contains(wl_output::Mode::Current) {
                told.screen = (width.max(1) as u32, height.max(1) as u32);
            }
        }
    }
}
delegate_noop!(Told: ignore xdg_toplevel::XdgToplevel);
delegate_noop!(Told: ignore wl_data_device_manager::WlDataDeviceManager);
delegate_noop!(Told: ignore ZwpVirtualKeyboardManagerV1);
delegate_noop!(Told: ignore ZwpVirtualKeyboardV1);
delegate_noop!(Told: ignore ZwlrVirtualPointerManagerV1);
delegate_noop!(Told: ignore ZwlrVirtualPointerV1);

impl Dispatch<xdg_wm_base::XdgWmBase, ()> for Told {
    fn event(_: &mut Self, base: &xdg_wm_base::XdgWmBase, event: xdg_wm_base::Event, _: &(), _: &Connection,
             _: &QueueHandle<Self>) {
        if let xdg_wm_base::Event::Ping { serial } = event {
            base.pong(serial);
        }
    }
}

impl Dispatch<xdg_surface::XdgSurface, ()> for Told {
    fn event(told: &mut Self, surface: &xdg_surface::XdgSurface, event: xdg_surface::Event, _: &(), _: &Connection,
             _: &QueueHandle<Self>) {
        if let xdg_surface::Event::Configure { serial } = event {
            surface.ack_configure(serial);
            told.configured = true;
        }
    }
}

impl Dispatch<wl_pointer::WlPointer, ()> for Told {
    fn event(told: &mut Self, _: &wl_pointer::WlPointer, event: wl_pointer::Event, _: &(), _: &Connection,
             _: &QueueHandle<Self>) {
        match event {
            wl_pointer::Event::Enter { surface_x, surface_y, .. } | wl_pointer::Event::Motion { surface_x, surface_y, .. } => {
                told.pointer = Some((surface_x, surface_y));
            }
            wl_pointer::Event::Leave { .. } => told.pointer = None,
            wl_pointer::Event::Button { serial, state, .. } => {
                told.pressed = (state == WEnum::Value(wl_pointer::ButtonState::Pressed)).then_some(serial);
            }
            _ => {}
        }
    }
}

impl Dispatch<wl_data_device::WlDataDevice, ()> for Told {
    fn event(told: &mut Self, _: &wl_data_device::WlDataDevice, event: wl_data_device::Event, _: &(), _: &Connection,
             _: &QueueHandle<Self>) {
        use wl_data_device::Event as E;
        match event {
            E::DataOffer { id } => {
                told.kinds.insert(id.id(), Vec::new());
            }
            E::Enter { serial, id, .. } => {
                told.action = 0;
                if let (Some(offer), Some((kind, ways, rather))) = (&id, &told.answer) {
                    offer.accept(serial, Some(kind.clone()));
                    offer.set_actions(DndAction::from_bits_truncate(*ways), DndAction::from_bits_truncate(*rather));
                }
                told.held = id;
            }
            E::Leave => {
                if let Some(o) = told.held.take() {
                    o.destroy();
                }
            }
            E::Drop => told.dropped = told.held.take(),
            _ => {}
        }
    }

    event_created_child!(Told, wl_data_device::WlDataDevice, [
        wl_data_device::EVT_DATA_OFFER_OPCODE => (wl_data_offer::WlDataOffer, ())
    ]);
}

impl Dispatch<wl_data_offer::WlDataOffer, ()> for Told {
    fn event(told: &mut Self, offer: &wl_data_offer::WlDataOffer, event: wl_data_offer::Event, _: &(), _: &Connection,
             _: &QueueHandle<Self>) {
        match event {
            wl_data_offer::Event::Offer { mime_type } => told.kinds.entry(offer.id()).or_default().push(mime_type),
            wl_data_offer::Event::SourceActions { source_actions } => {
                told.ways.insert(offer.id(), bits(source_actions));
            }
            wl_data_offer::Event::Action { dnd_action } => told.action = bits(dnd_action),
            _ => {}
        }
    }
}

impl Dispatch<wl_data_source::WlDataSource, ()> for Told {
    fn event(told: &mut Self, _: &wl_data_source::WlDataSource, event: wl_data_source::Event, _: &(), _: &Connection,
             _: &QueueHandle<Self>) {
        use wl_data_source::Event as E;
        match event {
            E::Send { mime_type, fd } => {
                let bytes = told.serving.get(&mime_type).cloned().unwrap_or_default();
                told.asked.push(mime_type);
                let (tx, rx) = mpsc::channel();
                let never_ends = told.never_ends;
                std::thread::spawn(move || {
                    let mut file = std::fs::File::from(fd);
                    hand_over(&mut file, &bytes);
                    if never_ends {
                        let _ = tx.send(file);
                    }
                });
                if never_ends {
                    told.kept_open.extend(rx.recv_timeout(Duration::from_secs(2)));
                }
            }
            E::Action { dnd_action } => told.source_action = bits(dnd_action),
            E::DndDropPerformed => told.performed = true,
            E::DndFinished => told.finished += 1,
            E::Cancelled => told.cancelled += 1,
            _ => {}
        }
    }
}

delegate_noop!(Told: ignore ZwpPrimarySelectionDeviceManagerV1);
delegate_noop!(Told: ignore ZwpPrimarySelectionOfferV1);

impl Dispatch<ZwpPrimarySelectionDeviceV1, ()> for Told {
    fn event(_: &mut Self, _: &ZwpPrimarySelectionDeviceV1, _: zwp_primary_selection_device_v1::Event, _: &(), _: &Connection,
             _: &QueueHandle<Self>) {}

    event_created_child!(Told, ZwpPrimarySelectionDeviceV1, [
        zwp_primary_selection_device_v1::EVT_DATA_OFFER_OPCODE => (ZwpPrimarySelectionOfferV1, ())
    ]);
}

impl Dispatch<ZwpPrimarySelectionSourceV1, ()> for Told {
    fn event(told: &mut Self, _: &ZwpPrimarySelectionSourceV1, event: zwp_primary_selection_source_v1::Event, _: &(),
             _: &Connection, _: &QueueHandle<Self>) {
        if let zwp_primary_selection_source_v1::Event::Send { mime_type, fd } = event {
            let bytes = told.serving.get(&mime_type).cloned().unwrap_or_default();
            told.asked.push(mime_type);
            std::thread::spawn(move || {
                hand_over(&mut std::fs::File::from(fd), &bytes);
            });
        }
    }
}

enum Ask<T> {
    Do(Box<dyn FnOnce(&T) + Send>),
    Quit,
}

pub struct Stage<T: 'static> {
    post: Post<Ask<T>>,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl<T: 'static> Stage<T> {
    pub fn start(make: impl FnOnce(&Loop, &Rc<Display>) -> T + Send + 'static) -> Stage<T> {
        let (tx, rx) = mpsc::channel();
        let thread = std::thread::spawn(move || {
            let events = Loop::new();
            let display = Display::connect(&events, || eprintln!("the compositor went away")).expect("connected");
            let mut made = Some(make(&events, &display));
            let lp = events.clone();
            let post = events.post(move |ask: Ask<T>| match (ask, &made) {
                (Ask::Do(f), Some(made)) => f(made),
                (Ask::Do(_), None) => {}
                (Ask::Quit, _) => {
                    made = None;
                    lp.quit();
                }
            });
            tx.send(post).unwrap();
            events.run();
        });
        let post = rx.recv_timeout(Duration::from_secs(20)).expect("the windows opened");
        Stage { post, thread: Some(thread) }
    }

    pub fn ask<R: Send + 'static>(&self, f: impl FnOnce(&T) -> R + Send + 'static) -> R {
        let (tx, rx) = mpsc::channel();
        self.post.send(Ask::Do(Box::new(move |made: &T| {
            let _ = tx.send(f(made));
        })));
        rx.recv_timeout(Duration::from_secs(20)).expect("the windows answered")
    }

    pub fn until(&self, what: &str, f: impl Fn(&T) -> bool + Send + Clone + 'static) {
        let end = Instant::now() + Duration::from_secs(10);
        while !self.ask(f.clone()) {
            assert!(Instant::now() < end, "never happened: {what}");
            std::thread::sleep(Duration::from_millis(30));
        }
    }
}

impl<T: 'static> Drop for Stage<T> {
    fn drop(&mut self) {
        self.post.send(Ask::Quit);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

fn hand_over(file: &mut std::fs::File, bytes: &[u8]) {
    if let Err(e) = file.write_all(bytes) {
        if e.kind() != std::io::ErrorKind::BrokenPipe {
            eprintln!("drag_test: a send failed: {e}");
        }
    }
}

pub fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("trindle-dnd-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}
