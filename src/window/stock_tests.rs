//! A window opens on a stand-in compositor that offers only the core protocols every Wayland desktop has.

use std::collections::{HashMap, HashSet};
use std::os::unix::net::UnixStream;
use std::sync::atomic::{AtomicBool, Ordering};
use std::rc::Rc;
use std::sync::Arc;
use std::time::{Duration, Instant};

use wayland_protocols::wp::linux_dmabuf::zv1::server::{zwp_linux_buffer_params_v1, zwp_linux_dmabuf_feedback_v1, zwp_linux_dmabuf_v1};
use wayland_protocols::ext::session_lock::v1::server::{ext_session_lock_manager_v1, ext_session_lock_surface_v1, ext_session_lock_v1};
use wayland_protocols::xdg::shell::server::{xdg_positioner, xdg_surface, xdg_toplevel, xdg_wm_base};
use wayland_server::backend::{ClientData, ClientId, DisconnectReason, ObjectId};
use wayland_server::protocol::{wl_buffer, wl_callback, wl_compositor, wl_output, wl_region, wl_seat, wl_shm, wl_shm_pool, wl_subcompositor, wl_subsurface, wl_surface};
use wayland_server::{Client, DataInit, Dispatch, DisplayHandle, GlobalDispatch, New, Resource};

use crate::event::{Flow, Loop};
use crate::window::{Display, Kind};

#[path = "gesture_tests.rs"]
#[cfg(test)]
mod gesture_tests;

struct Stock {
    painted: Arc<AtomicBool>,
    attached: HashSet<ObjectId>,
    serial: u32,
    titles: Vec<String>,
    frames: Vec<wl_callback::WlCallback>,
    lock_log: Vec<&'static str>,
    sub_log: Vec<String>,
    frame_log: Vec<String>,
    releases: bool,
    pending: HashMap<ObjectId, wl_buffer::WlBuffer>,
    shown: HashMap<ObjectId, wl_buffer::WlBuffer>,
    dmabuf_log: Vec<String>,
    pointing: bool,
}

struct Quiet;

impl ClientData for Quiet {
    fn initialized(&self, _: ClientId) {}
    fn disconnected(&self, _: ClientId, _: DisconnectReason) {}
}

impl GlobalDispatch<wl_compositor::WlCompositor, ()> for Stock {
    fn bind(_: &mut Self, _: &DisplayHandle, _: &Client, new: New<wl_compositor::WlCompositor>, _: &(), init: &mut DataInit<'_, Self>) {
        init.init(new, ());
    }
}

impl GlobalDispatch<xdg_wm_base::XdgWmBase, ()> for Stock {
    fn bind(_: &mut Self, _: &DisplayHandle, _: &Client, new: New<xdg_wm_base::XdgWmBase>, _: &(), init: &mut DataInit<'_, Self>) {
        init.init(new, ());
    }
}

impl GlobalDispatch<wl_shm::WlShm, ()> for Stock {
    fn bind(_: &mut Self, _: &DisplayHandle, _: &Client, new: New<wl_shm::WlShm>, _: &(), init: &mut DataInit<'_, Self>) {
        let shm = init.init(new, ());
        shm.format(wl_shm::Format::Argb8888);
        shm.format(wl_shm::Format::Xrgb8888);
    }
}

impl GlobalDispatch<wl_seat::WlSeat, ()> for Stock {
    fn bind(state: &mut Self, _: &DisplayHandle, _: &Client, new: New<wl_seat::WlSeat>, _: &(), init: &mut DataInit<'_, Self>) {
        let seat = init.init(new, ());
        seat.capabilities(if state.pointing { wl_seat::Capability::Pointer } else { wl_seat::Capability::empty() });
        if seat.version() >= 2 {
            seat.name("seat0".into());
        }
    }
}

impl GlobalDispatch<wl_output::WlOutput, ()> for Stock {
    fn bind(_: &mut Self, _: &DisplayHandle, _: &Client, new: New<wl_output::WlOutput>, _: &(), init: &mut DataInit<'_, Self>) {
        let output = init.init(new, ());
        output.geometry(0, 0, 300, 190, wl_output::Subpixel::Unknown, "Made up".into(), "Screen".into(), wl_output::Transform::Normal);
        output.mode(wl_output::Mode::Current, 1280, 800, 60_000);
        if output.version() >= 4 {
            output.name("STOCK-1".into());
        }
        if output.version() >= 2 {
            output.scale(1);
            output.done();
        }
    }
}

impl Dispatch<wl_compositor::WlCompositor, ()> for Stock {
    fn request(_: &mut Self, _: &Client, _: &wl_compositor::WlCompositor, request: wl_compositor::Request, _: &(), _: &DisplayHandle, init: &mut DataInit<'_, Self>) {
        match request {
            wl_compositor::Request::CreateSurface { id } => {
                init.init(id, ());
            }
            wl_compositor::Request::CreateRegion { id } => {
                init.init(id, ());
            }
            _ => {}
        }
    }
}

impl Dispatch<wl_surface::WlSurface, ()> for Stock {
    fn request(state: &mut Self, _: &Client, surface: &wl_surface::WlSurface, request: wl_surface::Request, _: &(), _: &DisplayHandle, init: &mut DataInit<'_, Self>) {
        match request {
            wl_surface::Request::Attach { buffer: Some(b), .. } => {
                state.attached.insert(surface.id());
                if state.releases {
                    state.pending.insert(surface.id(), b);
                }
            }
            wl_surface::Request::Attach { buffer: None, .. } => {
                state.attached.remove(&surface.id());
            }
            wl_surface::Request::Commit => {
                if state.attached.remove(&surface.id()) {
                    state.painted.store(true, Ordering::SeqCst);
                }
                if let Some(b) = state.pending.remove(&surface.id()) {
                    match state.shown.insert(surface.id(), b.clone()) {
                        Some(old) if old != b => old.release(),
                        _ => {}
                    }
                }
            }
            wl_surface::Request::Frame { callback } => {
                state.frames.push(init.init(callback, ()));
            }
            wl_surface::Request::DamageBuffer { x, y, width, height } => state.dmabuf_log.push(format!("damage {x} {y} {width} {height}")),
            wl_surface::Request::SetOpaqueRegion { region: None } => state.frame_log.push("region none".into()),
            wl_surface::Request::SetInputRegion { region } => state.frame_log.push(format!("input {}", if region.is_some() { "part" } else { "all" })),
            _ => {}
        }
    }
}

impl Dispatch<wl_region::WlRegion, ()> for Stock {
    fn request(state: &mut Self, _: &Client, _: &wl_region::WlRegion, request: wl_region::Request, _: &(), _: &DisplayHandle, _: &mut DataInit<'_, Self>) {
        if let wl_region::Request::Add { x, y, width, height } = request {
            state.frame_log.push(format!("region {x} {y} {width} {height}"));
        }
    }
}

impl Dispatch<wl_callback::WlCallback, ()> for Stock {
    fn request(_: &mut Self, _: &Client, _: &wl_callback::WlCallback, _: wl_callback::Request, _: &(), _: &DisplayHandle, _: &mut DataInit<'_, Self>) {}
}

impl Dispatch<wl_shm::WlShm, ()> for Stock {
    fn request(_: &mut Self, _: &Client, _: &wl_shm::WlShm, request: wl_shm::Request, _: &(), _: &DisplayHandle, init: &mut DataInit<'_, Self>) {
        if let wl_shm::Request::CreatePool { id, .. } = request {
            init.init(id, ());
        }
    }
}

impl Dispatch<wl_shm_pool::WlShmPool, ()> for Stock {
    fn request(state: &mut Self, _: &Client, _: &wl_shm_pool::WlShmPool, request: wl_shm_pool::Request, _: &(), _: &DisplayHandle, init: &mut DataInit<'_, Self>) {
        if let wl_shm_pool::Request::CreateBuffer { id, offset, width, height, .. } = request {
            if offset == 0 {
                state.frame_log.push(format!("buffer {width} {height}"));
            }
            init.init(id, ());
        }
    }
}

impl Dispatch<wl_buffer::WlBuffer, ()> for Stock {
    fn request(_: &mut Self, _: &Client, _: &wl_buffer::WlBuffer, _: wl_buffer::Request, _: &(), _: &DisplayHandle, _: &mut DataInit<'_, Self>) {}
}

impl Dispatch<wl_seat::WlSeat, ()> for Stock {
    fn request(_: &mut Self, _: &Client, _: &wl_seat::WlSeat, request: wl_seat::Request, _: &(), _: &DisplayHandle, init: &mut DataInit<'_, Self>) {
        if let wl_seat::Request::GetPointer { id } = request {
            gesture_tests::pointer(init.init(id, ()));
        }
    }
}

impl Dispatch<wl_output::WlOutput, ()> for Stock {
    fn request(_: &mut Self, _: &Client, _: &wl_output::WlOutput, _: wl_output::Request, _: &(), _: &DisplayHandle, _: &mut DataInit<'_, Self>) {}
}

impl Dispatch<xdg_wm_base::XdgWmBase, ()> for Stock {
    fn request(_: &mut Self, _: &Client, _: &xdg_wm_base::XdgWmBase, request: xdg_wm_base::Request, _: &(), _: &DisplayHandle, init: &mut DataInit<'_, Self>) {
        match request {
            xdg_wm_base::Request::GetXdgSurface { id, .. } => {
                init.init(id, ());
            }
            xdg_wm_base::Request::CreatePositioner { id } => {
                init.init(id, ());
            }
            _ => {}
        }
    }
}

impl Dispatch<xdg_positioner::XdgPositioner, ()> for Stock {
    fn request(_: &mut Self, _: &Client, _: &xdg_positioner::XdgPositioner, _: xdg_positioner::Request, _: &(), _: &DisplayHandle, _: &mut DataInit<'_, Self>) {}
}

impl Dispatch<xdg_surface::XdgSurface, ()> for Stock {
    fn request(state: &mut Self, _: &Client, xdg: &xdg_surface::XdgSurface, request: xdg_surface::Request, _: &(), _: &DisplayHandle, init: &mut DataInit<'_, Self>) {
        match request {
            xdg_surface::Request::GetToplevel { id } => {
                init.init(id, xdg.clone()).configure(0, 0, Vec::new());
                state.serial += 1;
                xdg.configure(state.serial);
            }
            xdg_surface::Request::SetWindowGeometry { x, y, width, height } => state.frame_log.push(format!("geometry {x} {y} {width} {height}")),
            _ => {}
        }
    }
}

impl Dispatch<xdg_toplevel::XdgToplevel, xdg_surface::XdgSurface> for Stock {
    fn request(state: &mut Self, _: &Client, top: &xdg_toplevel::XdgToplevel, request: xdg_toplevel::Request, xdg: &xdg_surface::XdgSurface, _: &DisplayHandle, _: &mut DataInit<'_, Self>) {
        match request {
            xdg_toplevel::Request::SetTitle { title } => state.titles.push(title),
            xdg_toplevel::Request::SetMaximized => {
                state.frame_log.push("maximise".into());
                top.configure(400, 300, 1u32.to_ne_bytes().to_vec());
                state.serial += 1;
                xdg.configure(state.serial);
            }
            xdg_toplevel::Request::ShowWindowMenu { x, y, .. } => state.frame_log.push(format!("menu {x} {y}")),
            _ => {}
        }
    }
}

fn stand_in(client: UnixStream, painted: Arc<AtomicBool>) -> (wayland_server::Display<Stock>, Stock) {
    let display = wayland_server::Display::<Stock>::new().expect("a stand-in compositor");
    let handle = display.handle();
    handle.create_global::<Stock, wl_compositor::WlCompositor, ()>(4, ());
    handle.create_global::<Stock, wl_shm::WlShm, ()>(1, ());
    handle.create_global::<Stock, xdg_wm_base::XdgWmBase, ()>(5, ());
    handle.create_global::<Stock, wl_seat::WlSeat, ()>(7, ());
    handle.create_global::<Stock, wl_output::WlOutput, ()>(4, ());
    handle.create_global::<Stock, ext_session_lock_manager_v1::ExtSessionLockManagerV1, ()>(1, ());
    handle.create_global::<Stock, wl_subcompositor::WlSubcompositor, ()>(1, ());
    display.handle().insert_client(client, Arc::new(Quiet)).expect("the toolkit's connection");
    let state = Stock { painted, attached: HashSet::new(), serial: 0, titles: Vec::new(), frames: Vec::new(), lock_log: Vec::new(), sub_log: Vec::new(), frame_log: Vec::new(),
                       releases: false, pending: HashMap::new(), shown: HashMap::new(), dmabuf_log: Vec::new(), pointing: false };
    (display, state)
}

fn serve(client: UnixStream, painted: Arc<AtomicBool>, titles: usize, most: Duration) -> Vec<String> {
    let (mut display, mut state) = stand_in(client, painted.clone());
    let end = Instant::now() + most;
    while Instant::now() < end && !(painted.load(Ordering::SeqCst) && state.titles.len() >= titles) {
        if display.dispatch_clients(&mut state).is_err() {
            break;
        }
        for frame in state.frames.drain(..) {
            frame.done(0);
        }
        let _ = display.flush_clients();
        std::thread::sleep(Duration::from_millis(5));
    }
    let _ = display.flush_clients();
    state.titles
}

#[test]
fn a_window_opens_on_a_compositor_with_only_the_core_protocols() {
    let (ours, theirs) = UnixStream::pair().expect("a pair of sockets");
    let painted = Arc::new(AtomicBool::new(false));
    let seen = painted.clone();
    let server = std::thread::spawn(move || serve(theirs, seen, 2, Duration::from_secs(20)));
    let conn = wayland_client::Connection::from_socket(ours).expect("a Wayland connection");
    let events = Loop::new();
    let display = Display::connect_on(conn, &events, || ()).expect("the toolkit connected with only the core protocols");
    assert!(!display.has_layers(), "the stand-in offers no layer shell");
    assert_eq!(display.screens(), ["STOCK-1"], "the screen was not read");
    let kind = Kind::Window { title: "Stock".into(), app_id: "org.example.Stock".into() };
    let window = display.open(kind, (200, 100), |canvas, _| canvas.fill(tiny_skia::Color::from_rgba8(40, 40, 40, 255)))
        .expect("an ordinary window opened");
    window.set_title("Renamed");
    let (quit, done) = (events.clone(), painted.clone());
    events.every(Duration::from_millis(10), move || {
        if done.load(Ordering::SeqCst) {
            quit.quit();
            return Flow::Stop;
        }
        Flow::Continue
    });
    let quit = events.clone();
    events.after(Duration::from_secs(15), move || quit.quit());
    events.run();
    drop(window);
    let titles = server.join().expect("the stand-in compositor");
    assert!(painted.load(Ordering::SeqCst), "the window was never painted on a compositor with only the core protocols");
    assert_eq!(titles, ["Stock", "Renamed"], "the window's title was not given, or not changed");
}

impl GlobalDispatch<ext_session_lock_manager_v1::ExtSessionLockManagerV1, ()> for Stock {
    fn bind(_: &mut Self, _: &DisplayHandle, _: &Client, new: New<ext_session_lock_manager_v1::ExtSessionLockManagerV1>, _: &(), init: &mut DataInit<'_, Self>) {
        init.init(new, ());
    }
}

impl Dispatch<ext_session_lock_manager_v1::ExtSessionLockManagerV1, ()> for Stock {
    fn request(state: &mut Self, _: &Client, _: &ext_session_lock_manager_v1::ExtSessionLockManagerV1, request: ext_session_lock_manager_v1::Request, _: &(), _: &DisplayHandle, init: &mut DataInit<'_, Self>) {
        if let ext_session_lock_manager_v1::Request::Lock { id } = request {
            state.lock_log.push("lock");
            init.init(id, ());
        }
    }
}

impl Dispatch<ext_session_lock_v1::ExtSessionLockV1, ()> for Stock {
    fn request(state: &mut Self, _: &Client, lock: &ext_session_lock_v1::ExtSessionLockV1, request: ext_session_lock_v1::Request, _: &(), _: &DisplayHandle, init: &mut DataInit<'_, Self>) {
        match request {
            ext_session_lock_v1::Request::GetLockSurface { id, .. } => {
                state.lock_log.push("surface");
                state.serial += 1;
                init.init(id, ()).configure(state.serial, 640, 480);
                lock.locked();
            }
            ext_session_lock_v1::Request::UnlockAndDestroy => state.lock_log.push("unlock"),
            ext_session_lock_v1::Request::Destroy => state.lock_log.push("destroy"),
            _ => {}
        }
    }
}

impl Dispatch<ext_session_lock_surface_v1::ExtSessionLockSurfaceV1, ()> for Stock {
    fn request(state: &mut Self, _: &Client, _: &ext_session_lock_surface_v1::ExtSessionLockSurfaceV1, request: ext_session_lock_surface_v1::Request, _: &(), _: &DisplayHandle, _: &mut DataInit<'_, Self>) {
        if let ext_session_lock_surface_v1::Request::AckConfigure { .. } = request {
            state.lock_log.push("ack");
        }
    }
}

fn serve_lock(client: UnixStream, most: Duration) -> Vec<&'static str> {
    let (mut display, mut state) = stand_in(client, Arc::new(AtomicBool::new(false)));
    let end = Instant::now() + most;
    while Instant::now() < end && !state.lock_log.contains(&"unlock") {
        if display.dispatch_clients(&mut state).is_err() {
            break;
        }
        for frame in state.frames.drain(..) {
            frame.done(0);
        }
        let _ = display.flush_clients();
        std::thread::sleep(Duration::from_millis(5));
    }
    let _ = display.flush_clients();
    state.lock_log
}

#[test]
fn a_session_lock_is_asked_covered_and_undone_in_order() {
    use super::LockPhase;
    let (ours, theirs) = UnixStream::pair().expect("a pair of sockets");
    let server = std::thread::spawn(move || serve_lock(theirs, Duration::from_secs(20)));
    let conn = wayland_client::Connection::from_socket(ours).expect("a Wayland connection");
    let events = Loop::new();
    let display = Display::connect_on(conn, &events, || ()).expect("the toolkit connected");
    assert!(display.has_session_lock());
    let on = Kind::Lock { screen: "STOCK-1".into() };
    assert!(display.open(Kind::Lock { screen: "STOCK-1".into() }, (1, 1), |_, _| ()).is_none(), "no surface before the lock is asked for");
    let told = Rc::new(std::cell::Cell::new(None));
    let seen = told.clone();
    display.on_lock(move |phase| seen.set(Some(phase)));
    let lock = display.lock_session().expect("the lock was asked for");
    assert!(display.lock_session().is_none(), "one lock at a time");
    assert!(!lock.unlock(), "unlocking before the session is locked is refused");
    let cover = display.open(on, (1, 1), |canvas, _| canvas.fill(tiny_skia::Color::BLACK)).expect("a surface covers the screen");
    assert!(display.open(Kind::Lock { screen: "STOCK-1".into() }, (1, 1), |_, _| ()).is_none(), "one surface a screen");
    assert!(display.open(Kind::Lock { screen: "NOWHERE".into() }, (1, 1), |_, _| ()).is_none(), "no surface on a screen that is not there");
    let quit = events.clone();
    let seen = told.clone();
    events.every(Duration::from_millis(10), move || {
        if seen.get() == Some(LockPhase::Locked) {
            quit.quit();
            return Flow::Stop;
        }
        Flow::Continue
    });
    let quit = events.clone();
    events.after(Duration::from_secs(15), move || quit.quit());
    events.run();
    assert_eq!((lock.phase(), told.get()), (LockPhase::Locked, Some(LockPhase::Locked)));
    assert!(lock.unlock(), "a locked session is unlocked");
    assert!(!lock.unlock(), "and only once");
    drop(cover);
    assert_eq!(server.join().expect("the stand-in compositor"), ["lock", "surface", "ack", "unlock"]);
}

impl GlobalDispatch<wl_subcompositor::WlSubcompositor, ()> for Stock {
    fn bind(_: &mut Self, _: &DisplayHandle, _: &Client, new: New<wl_subcompositor::WlSubcompositor>, _: &(), init: &mut DataInit<'_, Self>) {
        init.init(new, ());
    }
}

impl Dispatch<wl_subcompositor::WlSubcompositor, ()> for Stock {
    fn request(state: &mut Self, _: &Client, _: &wl_subcompositor::WlSubcompositor, request: wl_subcompositor::Request, _: &(), _: &DisplayHandle, init: &mut DataInit<'_, Self>) {
        if let wl_subcompositor::Request::GetSubsurface { id, .. } = request {
            init.init(id, ());
            state.sub_log.push("made".into());
        }
    }
}

impl Dispatch<wl_subsurface::WlSubsurface, ()> for Stock {
    fn request(state: &mut Self, _: &Client, _: &wl_subsurface::WlSubsurface, request: wl_subsurface::Request, _: &(), _: &DisplayHandle, _: &mut DataInit<'_, Self>) {
        state.sub_log.push(match request {
            wl_subsurface::Request::SetPosition { x, y } => format!("at {x} {y}"),
            wl_subsurface::Request::SetDesync => "desync".into(),
            wl_subsurface::Request::PlaceBelow { .. } => "below".into(),
            wl_subsurface::Request::Destroy => "ended".into(),
            _ => "other".into(),
        });
    }
}

fn serve_sub(client: UnixStream, most: Duration) -> Vec<String> {
    let (mut display, mut state) = stand_in(client, Arc::new(AtomicBool::new(false)));
    let end = Instant::now() + most;
    while Instant::now() < end && !state.sub_log.iter().any(|l| l == "ended") {
        if display.dispatch_clients(&mut state).is_err() {
            break;
        }
        for frame in state.frames.drain(..) {
            frame.done(0);
        }
        let _ = display.flush_clients();
        std::thread::sleep(Duration::from_millis(5));
    }
    state.sub_log
}

#[test]
fn a_gpu_surface_is_a_subsurface_placed_in_points() {
    let (ours, theirs) = UnixStream::pair().expect("a pair of sockets");
    let server = std::thread::spawn(move || serve_sub(theirs, Duration::from_secs(20)));
    let conn = wayland_client::Connection::from_socket(ours).expect("a Wayland connection");
    let events = Loop::new();
    let display = Display::connect_on(conn, &events, || ()).expect("connected");
    let kind = Kind::Window { title: "Gpu".into(), app_id: "org.example.Gpu".into() };
    let window = display.open(kind, (200, 100), |_, _| {}).expect("a window");
    let gpu = window.gpu_surface((10.4, 20.0)).expect("a subsurface on a compositor that has them");
    gpu.place((30.0, 40.0));
    gpu.below();
    #[cfg(feature = "gpu")]
    assert_ne!(gpu.display_ptr(), gpu.surface_ptr(), "a display and a surface pointer");
    drop(gpu);
    let log = server.join().expect("the stand-in compositor");
    assert_eq!(log, ["made", "desync", "at 10 20", "at 30 40", "below", "ended"]);
}

fn serve_frame(client: UnixStream, done: Arc<AtomicBool>, most: Duration) -> Vec<String> {
    let (mut display, mut state) = stand_in(client, Arc::new(AtomicBool::new(false)));
    let end = Instant::now() + most;
    while Instant::now() < end && !done.load(Ordering::SeqCst) {
        if display.dispatch_clients(&mut state).is_err() {
            break;
        }
        for frame in state.frames.drain(..) {
            frame.done(0);
        }
        if state.frame_log.iter().any(|l| l == "input all") {
            done.store(true, Ordering::SeqCst);
        }
        let _ = display.flush_clients();
        std::thread::sleep(Duration::from_millis(5));
    }
    let _ = display.flush_clients();
    state.frame_log
}

#[test]
fn a_shadowed_window_tells_its_geometry_and_drops_the_shadow_when_maximised() {
    let (ours, theirs) = UnixStream::pair().expect("a pair of sockets");
    let done = Arc::new(AtomicBool::new(false));
    let seen = done.clone();
    let server = std::thread::spawn(move || serve_frame(theirs, seen, Duration::from_secs(20)));
    let conn = wayland_client::Connection::from_socket(ours).expect("a Wayland connection");
    let events = Loop::new();
    let display = Display::connect_on(conn, &events, || ()).expect("connected");
    let kind = Kind::Window { title: "Framed".into(), app_id: "org.example.Framed".into() };
    let window = Rc::new(display.open(kind, (200, 100), |_, _| {}).expect("a window"));
    window.set_shadow(10);
    assert_eq!(window.size(), (200, 100), "the size is the window's own, without the room");
    let (w, quit, asked) = (window.clone(), events.clone(), std::cell::Cell::new(false));
    events.every(Duration::from_millis(10), move || {
        if done.load(Ordering::SeqCst) {
            quit.quit();
            return Flow::Stop;
        }
        if !asked.get() && w.frame().shadow == 10.0 && w.seen() {
            asked.set(true);
            w.show_window_menu(30.0, 12.0);
            w.set_maximised(true);
        }
        Flow::Continue
    });
    let quit = events.clone();
    events.after(Duration::from_secs(15), move || quit.quit());
    events.run();
    assert!(window.maximised() && window.frame().shadow == 0.0, "no shadow when maximised");
    assert_eq!(window.size(), (400, 300));
    drop(window);
    let log = server.join().expect("the stand-in compositor");
    let at = |l: &str| log.iter().position(|x| x == l).unwrap_or_else(|| panic!("{l} missing from {log:?}"));
    assert!(at("buffer 220 120") < at("geometry 10 10 200 100"), "{log:?}");
    assert!(at("region 10 10 200 100") < at("input part"), "{log:?}");
    assert!(at("menu 30 12") < at("maximise") && at("maximise") < at("buffer 400 300"), "{log:?}");
    assert!(at("geometry 0 0 400 300") < at("input all"), "{log:?}");
}

impl GlobalDispatch<zwp_linux_dmabuf_v1::ZwpLinuxDmabufV1, ()> for Stock {
    fn bind(_: &mut Self, _: &DisplayHandle, _: &Client, new: New<zwp_linux_dmabuf_v1::ZwpLinuxDmabufV1>, _: &(), init: &mut DataInit<'_, Self>) {
        init.init(new, ());
    }
}

const ARGB: u32 = u32::from_le_bytes(*b"AR24");
const XRGB: u32 = u32::from_le_bytes(*b"XR24");

impl Dispatch<zwp_linux_dmabuf_v1::ZwpLinuxDmabufV1, ()> for Stock {
    fn request(_: &mut Self, _: &Client, _: &zwp_linux_dmabuf_v1::ZwpLinuxDmabufV1, request: zwp_linux_dmabuf_v1::Request, _: &(), _: &DisplayHandle, init: &mut DataInit<'_, Self>) {
        match request {
            zwp_linux_dmabuf_v1::Request::CreateParams { params_id } => {
                init.init(params_id, ());
            }
            zwp_linux_dmabuf_v1::Request::GetDefaultFeedback { id } => {
                let f = init.init(id, ());
                let entries: [(u32, u64); 4] = [(XRGB, 0), (ARGB, 0), (ARGB, 0x0100_0000_0000_0001), (ARGB, 0x0200_0000_0000_0002)];
                let mut table = crate::common::shm::Memory::new(c"stock-table", 64).expect("a format table");
                for (e, (format, modifier)) in table.bytes_mut().chunks_exact_mut(16).zip(entries) {
                    e[..4].copy_from_slice(&format.to_ne_bytes());
                    e[8..].copy_from_slice(&modifier.to_ne_bytes());
                }
                f.format_table(table.fd(), 64);
                let (main, other) = (0x1234u64.to_ne_bytes().to_vec(), 0x9999u64.to_ne_bytes().to_vec());
                f.main_device(main.clone());
                for (device, indices) in [(main, [1u16, 2, 0]), (other, [3, 1, 0])] {
                    f.tranche_target_device(device);
                    f.tranche_formats(indices.iter().flat_map(|i| i.to_ne_bytes()).collect());
                    f.tranche_flags(zwp_linux_dmabuf_feedback_v1::TrancheFlags::empty());
                    f.tranche_done();
                }
                f.done();
            }
            _ => {}
        }
    }
}

impl Dispatch<zwp_linux_dmabuf_feedback_v1::ZwpLinuxDmabufFeedbackV1, ()> for Stock {
    fn request(_: &mut Self, _: &Client, _: &zwp_linux_dmabuf_feedback_v1::ZwpLinuxDmabufFeedbackV1, _: zwp_linux_dmabuf_feedback_v1::Request, _: &(), _: &DisplayHandle, _: &mut DataInit<'_, Self>) {}
}

impl Dispatch<zwp_linux_buffer_params_v1::ZwpLinuxBufferParamsV1, ()> for Stock {
    fn request(state: &mut Self, _: &Client, _: &zwp_linux_buffer_params_v1::ZwpLinuxBufferParamsV1, request: zwp_linux_buffer_params_v1::Request, _: &(), _: &DisplayHandle, init: &mut DataInit<'_, Self>) {
        match request {
            zwp_linux_buffer_params_v1::Request::Add { fd, plane_idx, offset, stride, modifier_hi, modifier_lo } => {
                let len = std::fs::File::from(fd).metadata().map_or(0, |m| m.len());
                state.dmabuf_log.push(format!("plane {plane_idx} {offset} {stride} {:#x} {len}", (u64::from(modifier_hi) << 32) | u64::from(modifier_lo)));
            }
            zwp_linux_buffer_params_v1::Request::CreateImmed { buffer_id, width, height, format, .. } => {
                init.init(buffer_id, ());
                state.dmabuf_log.push(format!("dmabuf {width} {height} {format:#x}"));
            }
            _ => {}
        }
    }
}

fn serve_dmabuf(client: UnixStream, done: Arc<AtomicBool>, most: Duration) -> Vec<String> {
    let (mut display, mut state) = stand_in(client, Arc::new(AtomicBool::new(false)));
    display.handle().create_global::<Stock, zwp_linux_dmabuf_v1::ZwpLinuxDmabufV1, ()>(4, ());
    state.releases = true;
    let end = Instant::now() + most;
    while Instant::now() < end && !done.load(Ordering::SeqCst) {
        if display.dispatch_clients(&mut state).is_err() {
            break;
        }
        for frame in state.frames.drain(..) {
            frame.done(0);
        }
        let _ = display.flush_clients();
        std::thread::sleep(Duration::from_millis(5));
    }
    let _ = display.dispatch_clients(&mut state);
    let regions = state.frame_log.iter().filter(|l| l.starts_with("region"));
    state.dmabuf_log.iter().cloned().chain(regions.cloned()).collect()
}

#[test]
fn a_window_drawn_by_the_gpu_hands_over_dmabufs_and_their_damage() {
    let (frames, log) = draw_by_gpu(super::dmabuf::Alloc::Memory, true);
    let part = vec![[10, 10, 20, 20]];
    assert_eq!(frames, [((200, 100), 0, vec![], false), ((200, 100), 0, vec![], false), ((200, 100), 2, part.clone(), false), ((200, 100), 2, part, false)]);
    let from = log.iter().position(|l| l.starts_with("plane")).unwrap_or_else(|| panic!("no dmabuf in {log:?}"));
    let (plane, made, part) = ("plane 0 0 800 0x0 80000", "dmabuf 200 100 0x34325241", "damage 10 10 20 20");
    assert_eq!(log[from..from + 8], [plane, made, plane, made, "damage 0 0 200 100", part, part, part], "{log:?}");
    assert_eq!(log[from + 8..], ["region 6 0 188 6", "region 6 94 188 6", "region 0 6 200 88"], "{log:?}");
}

#[test]
fn a_see_through_window_drawn_by_the_gpu_is_not_opaque() {
    let (_, log) = draw_by_gpu(super::dmabuf::Alloc::Memory, false);
    let regions: Vec<_> = log.iter().filter(|l| l.starts_with("region")).collect();
    assert_eq!(regions, ["region none"], "{log:?}");
}

#[test]
fn without_dmabufs_or_the_gpu_they_are_for_the_processor_draws() {
    std::env::set_var("TRINDLE_DRAWN_BY_GPU", "1");
    for dmabuf in [false, true] {
        let (ours, theirs) = UnixStream::pair().expect("a pair of sockets");
        let done = Arc::new(AtomicBool::new(false));
        let seen = done.clone();
        let server = std::thread::spawn(move || if dmabuf { serve_dmabuf(theirs, seen, Duration::from_secs(20)); } else { serve(theirs, seen, 0, Duration::from_secs(20)); });
        let conn = wayland_client::Connection::from_socket(ours).expect("a Wayland connection");
        let display = Display::connect_on(conn, &Loop::new(), || ()).expect("connected");
        let why = if dmabuf { crate::paint::Why::NoNode } else { crate::paint::Why::NoDmabuf };
        assert_eq!(crate::paint::gpu_for(&display).err(), Some(why));
        done.store(true, Ordering::SeqCst);
        drop(display);
        server.join().expect("the stand-in compositor");
    }
}

#[test]
fn a_window_whose_gpu_fails_goes_back_to_the_processor() {
    let (ours, theirs) = UnixStream::pair().expect("a pair of sockets");
    let done = Arc::new(AtomicBool::new(false));
    let seen = done.clone();
    let server = std::thread::spawn(move || serve_dmabuf(theirs, seen, Duration::from_secs(20)));
    let conn = wayland_client::Connection::from_socket(ours).expect("a Wayland connection");
    let events = Loop::new();
    let display = Display::connect_on(conn, &events, || ()).expect("connected");
    let (drawn, tries) = (Rc::new(std::cell::Cell::new(0)), Rc::new(std::cell::Cell::new(0)));
    let (d, t) = (drawn.clone(), tries.clone());
    let kind = Kind::Window { title: "Failing".into(), app_id: "org.example.Failing".into() };
    let window = Rc::new(display.open(kind, (200, 100), move |canvas, _| {
        d.set(d.get() + 1);
        canvas.fill(tiny_skia::Color::from_rgba8(40, 40, 40, 255));
    }).expect("a window"));
    assert!(window.draw_by_gpu(super::dmabuf::Alloc::Memory, |_| (), move |_| {
        t.set(t.get() + 1);
        super::dmabuf::Drawn::Failed
    }));
    let (w, quit, d) = (window.clone(), events.clone(), drawn.clone());
    events.every(Duration::from_millis(10), move || {
        if d.get() >= 3 {
            quit.quit();
            return Flow::Stop;
        }
        w.redraw();
        Flow::Continue
    });
    let quit = events.clone();
    events.after(Duration::from_secs(15), move || quit.quit());
    events.run();
    done.store(true, Ordering::SeqCst);
    drop(window);
    let log = server.join().expect("the stand-in compositor");
    assert_eq!(tries.get(), 1, "{log:?}");
    assert!(drawn.get() >= 3, "{log:?}");
    assert_eq!(log.iter().find(|l| l.starts_with("damage")).map(String::as_str), Some("damage 0 0 200 100"), "drawn whole first: {log:?}");
}

#[test]
fn a_minimised_or_hidden_window_the_gpu_draws_frees_its_buffers_and_makes_them_again_on_show() {
    let (ours, theirs) = UnixStream::pair().expect("a pair of sockets");
    let done = Arc::new(AtomicBool::new(false));
    let seen = done.clone();
    let server = std::thread::spawn(move || serve_dmabuf(theirs, seen, Duration::from_secs(20)));
    let conn = wayland_client::Connection::from_socket(ours).expect("a Wayland connection");
    let events = Loop::new();
    let display = Display::connect_on(conn, &events, || ()).expect("connected");
    let kind = Kind::Window { title: "Hidden".into(), app_id: "org.example.Hidden".into() };
    let window = Rc::new(display.open(kind, (200, 100), |_, _| {}).expect("a window"));
    let (frames, slept) = (Rc::new(std::cell::Cell::new(0)), Rc::new(std::cell::RefCell::new(Vec::new())));
    let (f, s) = (frames.clone(), slept.clone());
    assert!(window.draw_by_gpu(super::dmabuf::Alloc::Memory, move |yes| s.borrow_mut().push(yes), move |_| {
        f.set(f.get() + 1);
        super::dmabuf::Drawn::Whole
    }));
    let (w, quit, f, s, d) = (window.clone(), events.clone(), frames.clone(), slept.clone(), display.clone());
    let (minimised_at, held) = (std::cell::Cell::new(0), |d: &Rc<Display>, id| d.state.borrow_mut().win(id).and_then(|x| x.gpu.as_ref().map(|p| p.slots.len())).unwrap_or(99));
    events.every(Duration::from_millis(10), move || {
        let n = f.get();
        if n >= 2 && minimised_at.get() == 0 {
            minimised_at.set(n);
            if let Some(x) = d.state.borrow_mut().win(w.id) { x.suspended = true; }
            d.trim_unseen();
            assert_eq!(held(&d, w.id), 0, "the buffers are freed while it is minimised");
            assert_eq!(*s.borrow(), [true]);
            if let Some(x) = d.state.borrow_mut().win(w.id) { x.suspended = false; }
            d.trim_unseen();
            assert_eq!(*s.borrow(), [true, false]);
        }
        if minimised_at.get() > 0 && n > minimised_at.get() {
            assert_eq!(held(&d, w.id), 2, "made again for the next frame");
            w.set_visible(false);
            assert_eq!(held(&d, w.id), 0, "and freed again when it is hidden");
            assert_eq!(*s.borrow(), [true, false, true]);
            quit.quit();
            return Flow::Stop;
        }
        w.redraw();
        Flow::Continue
    });
    let quit = events.clone();
    events.after(Duration::from_secs(15), move || quit.quit());
    events.run();
    done.store(true, Ordering::SeqCst);
    drop(window);
    let log = server.join().expect("the stand-in compositor");
    assert_eq!(*slept.borrow(), [true, false, true]);
    assert_eq!(log.iter().filter(|l| *l == "dmabuf 200 100 0x34325241").count(), 4, "two buffers made, freed and made again: {log:?}");
}

#[test]
#[ignore = "needs a GPU and Mesa"]
fn a_window_drawn_by_the_gpu_hands_over_gbm_buffers() {
    let node = crate::sys::gbm::render_nodes().into_iter().next().expect("a render node");
    let (frames, log) = draw_by_gpu(super::dmabuf::Alloc::on(&node).expect("GBM on the render node"), true);
    println!("{log:#?}");
    assert_eq!(frames.iter().take(4).map(|f| (f.1, f.3)).collect::<Vec<_>>(), [(0, true), (0, true), (2, true), (2, true)]);
    assert_eq!(log.iter().filter(|l| *l == "dmabuf 200 100 0x34325241").count(), 2, "{log:?}");
}

#[test]
#[ignore = "needs a GPU and Mesa"]
fn a_window_on_the_drawing_service_moves_on_when_the_service_is_killed() {
    use crate::paint::remote::{Remote, Sending};
    use crate::paint::service::rig::Running;
    use super::dmabuf::Alloc;
    let mut service = Running::start("window", 0, 0);
    let remote = Remote::at(&service.socket).expect("the service answers");
    let (conn, done, server) = stand_in_with_dmabuf(Duration::from_secs(30));
    let events = Loop::new();
    remote.hear_by(&events);
    let display = Display::connect_on(conn, &events, || ()).expect("connected");
    let kind = Kind::Window { title: "Served".into(), app_id: "org.example.Served".into() };
    let window = Rc::new(display.open(kind, (200, 100), |_, _| {}).expect("a window"));
    let counts = Rc::new(std::cell::Cell::new((0usize, 0usize, false)));
    let (c, w, mut sending, r) = (counts.clone(), Rc::downgrade(&window), Sending::new(remote.clone()), remote.clone());
    assert!(window.draw_by_gpu(Alloc::Service(remote.clone()), |_| (), move |frame| {
        drawn_on_service(frame, &mut sending, &r, &w, &c)
    }));
    let (w, quit, c) = (window.clone(), events.clone(), counts.clone());
    let mut child = Some(service.child.id());
    events.every(Duration::from_millis(4), move || {
        let (on_service, after, _) = c.get();
        if on_service >= 10 {
            if let Some(pid) = child.take() {
                std::process::Command::new("kill").args(["-9", &pid.to_string()]).status().expect("kill");
            }
        }
        if after >= 5 {
            quit.quit();
            return Flow::Stop;
        }
        w.redraw();
        Flow::Continue
    });
    let quit = events.clone();
    events.after(Duration::from_secs(20), move || quit.quit());
    let began = Instant::now();
    events.run();
    done.store(true, Ordering::SeqCst);
    drop(window);
    server.join().expect("the stand-in compositor");
    let (on_service, after, told) = counts.get();
    println!("{on_service} frames by the service, {after} after it, in {:?}", began.elapsed());
    assert!(on_service >= 10, "the service drew {on_service}");
    assert!(told, "the window was never told");
    assert!(after >= 5, "only {after} frames after the service went");
    assert!(remote.gone());
    assert!(!service.alive());
}

#[test]
#[ignore = "needs a GPU and Mesa"]
fn a_window_drawn_by_the_gpu_for_hundreds_of_frames_stays_what_the_processor_draws() {
    let shared = crate::paint::here();
    let (conn, done, server) = stand_in_with_dmabuf(Duration::from_secs(60));
    let events = Loop::new();
    let display = Display::connect_on(conn, &events, || ()).expect("connected");
    let kind = Kind::Window { title: "Soak".into(), app_id: "org.example.Soak".into() };
    let window = Rc::new(display.open(kind, (480, 320), |_, _| {}).expect("a window"));
    let (wrong, ages) = (Rc::new(std::cell::RefCell::new(Vec::<String>::new())), Rc::new(std::cell::RefCell::new(std::collections::BTreeSet::new())));
    let (bad, old, count) = (wrong.clone(), ages.clone(), Rc::new(std::cell::Cell::new(0usize)));
    let c = count.clone();
    let mut drawing = crate::paint::Drawing::new(shared.clone());
    let mut ui = crate::ui::Ui::new(crate::gallery::gallery(0), crate::theme::Theme::new(false, "#fbc711"));
    let mut rng = 0x2545_F491_4F6C_DD1Du64;
    let start = Instant::now();
    assert!(window.draw_by_gpu(super::dmabuf::Alloc::Gbm(shared.device.clone()), |_| (), move |fr| {
        let n = c.get();
        c.set(n + 1);
        play_on(&mut ui, &mut rng, n, fr.size, start);
        let mut want = tiny_skia::Pixmap::new(fr.size.0, fr.size.1).unwrap();
        ui.paint(&mut want.as_mut(), fr.scale);
        let drawn = ui.paint_by_gpu(fr, &mut drawing);
        old.borrow_mut().insert((fr.age, fr.behind.len().min(1)));
        let Some(d) = fr.dmabuf else { return drawn };
        if matches!(drawn, super::dmabuf::Drawn::Failed) {
            bad.borrow_mut().push(format!("frame {n}: the GPU failed"));
            return drawn;
        }
        let got = crate::sys::gbm::read_linear(d).expect("the buffer read");
        let first = first_difference(want.data(), &got, d);
        if let Some((x, y, a, g)) = first {
            bad.borrow_mut().push(format!("frame {n}, age {}, {} parts behind: pixel ({x}, {y}) processor {a:?}, GPU {g:?}", fr.age, fr.behind.len()));
        }
        drawn
    }));
    let (w, quit, c) = (window.clone(), events.clone(), count.clone());
    events.every(Duration::from_millis(8), move || {
        if c.get() >= 400 {
            quit.quit();
            return Flow::Stop;
        }
        w.redraw();
        Flow::Continue
    });
    let quit = events.clone();
    events.after(Duration::from_secs(50), move || quit.quit());
    events.run();
    done.store(true, Ordering::SeqCst);
    drop(window);
    server.join().expect("the stand-in compositor");
    println!("{} frames; (age, behind at all) seen {:?}", count.get(), ages.borrow());
    assert!(count.get() >= 100, "only {} frames", count.get());
    assert!(wrong.borrow().is_empty(), "{} frames wrong, the first {:?}", wrong.borrow().len(), &wrong.borrow()[..wrong.borrow().len().min(5)]);
}

fn stand_in_with_dmabuf(most: Duration) -> (wayland_client::Connection, Arc<AtomicBool>, std::thread::JoinHandle<Vec<String>>) {
    let (ours, theirs) = UnixStream::pair().expect("a pair of sockets");
    let done = Arc::new(AtomicBool::new(false));
    let seen = done.clone();
    let server = std::thread::spawn(move || serve_dmabuf(theirs, seen, most));
    let conn = wayland_client::Connection::from_socket(ours).expect("a Wayland connection");
    (conn, done, server)
}

fn drawn_on_service(frame: &super::dmabuf::Frame, sending: &mut crate::paint::remote::Sending, r: &crate::paint::remote::Remote, w: &std::rc::Weak<crate::window::Window>,
                    c: &Rc<std::cell::Cell<(usize, usize, bool)>>) -> super::dmabuf::Drawn {
    use crate::paint::remote::Answer;
    use super::dmabuf::{Alloc, Drawn};
    if frame.dmabuf.is_some() || frame.id == 0 || r.gone() && frame.age == 0 && frame.behind.is_empty() && frame.dmabuf.is_none() && c.get().2 {
        c.set((c.get().0, c.get().1 + 1, c.get().2));
        return Drawn::Whole;
    }
    let mut p = crate::paint::Painter::default();
    p.fill(crate::widget::Rect { x: 0.0, y: 0.0, w: 200.0, h: 100.0 }, crate::colour::Colour { r: 1.0, g: 0.0, b: 0.0, a: 1.0 });
    match sending.draw(frame, p, &crate::shadow::Shadows::default()) {
        Drawn::Sent { seq, parts } => {
            let (w, c) = (w.clone(), c.clone());
            sending.remote().when_answered(seq, move |answer| {
                let Some(window) = w.upgrade() else { return };
                window.answered(seq, match answer {
                    Answer::Done => {
                        c.set((c.get().0 + 1, c.get().1, c.get().2));
                        parts.map_or(Drawn::Whole, Drawn::Parts)
                    }
                    _ => {
                        c.set((c.get().0, c.get().1, true));
                        Drawn::Moved(Alloc::Memory)
                    }
                });
            });
            Drawn::Sent { seq, parts: None }
        }
        Drawn::Gone => {
            c.set((c.get().0, c.get().1, true));
            Drawn::Moved(Alloc::Memory)
        }
        drawn => drawn,
    }
}

fn play_on(ui: &mut crate::ui::Ui<()>, rng: &mut u64, n: usize, size: (u32, u32), start: Instant) {
    use crate::input::{Button, Input};
    let mut next = || { *rng ^= *rng << 13; *rng ^= *rng >> 7; *rng ^= *rng << 17; (*rng >> 11) as f32 / (1u64 << 53) as f32 };
    let (x, y) = (next() * size.0 as f32, next() * size.1 as f32);
    let now = start + Duration::from_millis(16 * n as u64);
    ui.lay_at(size.0, size.1);
    for _ in 0..(next() * 3.0) as usize {
        ui.input(&Input::PointerMove { x: next() * size.0 as f32, y: next() * size.1 as f32 }, now);
    }
    if n % 9 == 8 {
        ui.input(&Input::Press { button: Button::Left, x, y }, now);
        ui.input(&Input::Release { button: Button::Left, x, y }, now);
    }
    if n % 40 == 39 {
        ui.set_view(crate::gallery::gallery((n / 40) as u32));
        ui.lay_at(size.0, size.1);
    }
    ui.tick(now);
}

fn first_difference(want: &[u8], got: &[u8], d: &crate::sys::gbm::Dmabuf) -> Option<(usize, usize, [u8; 4], [u8; 4])> {
    let (w, p) = (d.width as usize, &d.planes[0]);
    want.chunks_exact(4).enumerate().find_map(|(i, px)| {
        let at = p.offset as usize + (i / w) * p.stride as usize + (i % w) * 4;
        let g = [got[at + 2], got[at + 1], got[at], got[at + 3]];
        px.iter().zip(g).any(|(a, b)| a.abs_diff(b) > 2).then(|| (i % w, i / w, [px[0], px[1], px[2], px[3]], g))
    })
}

fn draw_by_gpu(alloc: super::dmabuf::Alloc, solid: bool) -> (Vec<((u32, u32), u32, Vec<[i32; 4]>, bool)>, Vec<String>) {
    let (ours, theirs) = UnixStream::pair().expect("a pair of sockets");
    let done = Arc::new(AtomicBool::new(false));
    let seen = done.clone();
    let server = std::thread::spawn(move || serve_dmabuf(theirs, seen, Duration::from_secs(20)));
    let conn = wayland_client::Connection::from_socket(ours).expect("a Wayland connection");
    let events = Loop::new();
    let display = Display::connect_on(conn, &events, || ()).expect("connected");
    assert_eq!(display.state.borrow().dmabuf.as_ref().map(|o| o.modifiers.clone()), Some(vec![0, 0x0100_0000_0000_0001]),
               "the main device's Argb8888 layouts, in the compositor's order, without the other GPU's");
    let kind = Kind::Window { title: "Drawn".into(), app_id: "org.example.Drawn".into() };
    let window = Rc::new(display.open(kind, (200, 100), |_, _| {}).expect("a window"));
    let frames = Rc::new(std::cell::RefCell::new(Vec::new()));
    let f = frames.clone();
    window.set_corner(6);
    window.set_solid(solid);
    assert!(window.draw_by_gpu(alloc, |_| (), move |fr| {
        f.borrow_mut().push((fr.size, fr.age, fr.behind.to_vec(), fr.dmabuf.is_some()));
        super::dmabuf::Drawn::Parts(vec![[10, 10, 20, 20]])
    }));
    let (w, quit, f) = (window.clone(), events.clone(), frames.clone());
    events.every(Duration::from_millis(10), move || {
        if f.borrow().len() >= 4 {
            quit.quit();
            return Flow::Stop;
        }
        w.redraw();
        Flow::Continue
    });
    let quit = events.clone();
    events.after(Duration::from_secs(15), move || quit.quit());
    events.run();
    done.store(true, Ordering::SeqCst);
    drop(window);
    let log = server.join().expect("the stand-in compositor");
    let frames = frames.borrow().clone();
    (frames, log)
}
