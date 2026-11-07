//! A touchpad pinch told by a stand-in compositor reaches the window as `Input::PadPinch`.

use std::cell::RefCell;

use wayland_protocols::wp::pointer_gestures::zv1::server::{zwp_pointer_gesture_pinch_v1, zwp_pointer_gestures_v1};
use wayland_server::protocol::wl_pointer;

use super::*;
use crate::input::Input;

thread_local! {
    static POINTER: RefCell<Option<wl_pointer::WlPointer>> = const { RefCell::new(None) };
    static PINCH: RefCell<Option<zwp_pointer_gesture_pinch_v1::ZwpPointerGesturePinchV1>> = const { RefCell::new(None) };
}

pub(super) fn pointer(p: wl_pointer::WlPointer) {
    POINTER.with(|s| *s.borrow_mut() = Some(p));
}

impl Dispatch<wl_pointer::WlPointer, ()> for Stock {
    fn request(_: &mut Self, _: &Client, _: &wl_pointer::WlPointer, _: wl_pointer::Request, _: &(), _: &DisplayHandle, _: &mut DataInit<'_, Self>) {}
}

impl GlobalDispatch<zwp_pointer_gestures_v1::ZwpPointerGesturesV1, ()> for Stock {
    fn bind(_: &mut Self, _: &DisplayHandle, _: &Client, new: New<zwp_pointer_gestures_v1::ZwpPointerGesturesV1>, _: &(), init: &mut DataInit<'_, Self>) {
        init.init(new, ());
    }
}

impl Dispatch<zwp_pointer_gestures_v1::ZwpPointerGesturesV1, ()> for Stock {
    fn request(_: &mut Self, _: &Client, _: &zwp_pointer_gestures_v1::ZwpPointerGesturesV1, request: zwp_pointer_gestures_v1::Request, _: &(), _: &DisplayHandle, init: &mut DataInit<'_, Self>) {
        if let zwp_pointer_gestures_v1::Request::GetPinchGesture { id, .. } = request {
            let pinch = init.init(id, ());
            PINCH.with(|s| *s.borrow_mut() = Some(pinch));
        }
    }
}

impl Dispatch<zwp_pointer_gesture_pinch_v1::ZwpPointerGesturePinchV1, ()> for Stock {
    fn request(_: &mut Self, _: &Client, _: &zwp_pointer_gesture_pinch_v1::ZwpPointerGesturePinchV1, _: zwp_pointer_gesture_pinch_v1::Request, _: &(), _: &DisplayHandle, _: &mut DataInit<'_, Self>) {}
}

fn serve_pinch(client: UnixStream, heard: Arc<AtomicBool>, most: Duration) {
    let painted = Arc::new(AtomicBool::new(false));
    let (mut display, mut state) = stand_in(client, painted.clone());
    (state.pointing, state.releases) = (true, true);
    display.handle().create_global::<Stock, zwp_pointer_gestures_v1::ZwpPointerGesturesV1, ()>(3, ());
    let (end, mut sent) = (Instant::now() + most, false);
    while Instant::now() < end && !heard.load(Ordering::SeqCst) {
        if display.dispatch_clients(&mut state).is_err() {
            break;
        }
        for frame in state.frames.drain(..) {
            frame.done(0);
        }
        let surface = state.shown.keys().next().and_then(|id| wl_surface::WlSurface::from_id(&display.handle(), id.clone()).ok());
        let (pointer, pinch) = (POINTER.with(|s| s.borrow().clone()), PINCH.with(|s| s.borrow().clone()));
        if let (false, true, Some(surface), Some(pointer), Some(pinch)) = (sent, painted.load(Ordering::SeqCst), surface, pointer, pinch) {
            pointer.enter(1, &surface, 50.0, 40.0);
            pointer.frame();
            pinch.begin(2, 0, &surface, 2);
            pinch.update(1, 3.0, -1.0, 1.5, 45.0);
            pinch.update(2, 0.0, 0.0, 2.0, 45.0);
            pinch.end(3, 3, 0);
            sent = true;
        }
        let _ = display.flush_clients();
        std::thread::sleep(Duration::from_millis(5));
    }
    let _ = display.flush_clients();
}

#[test]
fn a_touchpad_pinch_reaches_the_window_under_the_pointer() {
    let (ours, theirs) = UnixStream::pair().expect("a pair of sockets");
    let heard = Arc::new(AtomicBool::new(false));
    let told = heard.clone();
    let server = std::thread::spawn(move || serve_pinch(theirs, told, Duration::from_secs(20)));
    let conn = wayland_client::Connection::from_socket(ours).expect("a Wayland connection");
    let events = Loop::new();
    let display = Display::connect_on(conn, &events, || ()).expect("the toolkit connected");
    let kind = Kind::Window { title: "Pinch".into(), app_id: "org.example.Pinch".into() };
    let window = display.open(kind, (200, 100), |canvas, _| canvas.fill(tiny_skia::Color::from_rgba8(40, 40, 40, 255)))
        .expect("a window opened");
    let got = Rc::new(std::cell::RefCell::new(Vec::new()));
    let (quit, seen, done) = (events.clone(), got.clone(), heard.clone());
    window.on_input(move |input| {
        if matches!(input, Input::PadPinch(_) | Input::PadPinchEnd { .. }) {
            seen.borrow_mut().push(input.clone());
        }
        if matches!(input, Input::PadPinchEnd { .. }) {
            done.store(true, Ordering::SeqCst);
            quit.quit();
        }
    });
    let quit = events.clone();
    events.after(Duration::from_secs(15), move || quit.quit());
    events.run();
    drop(window);
    heard.store(true, Ordering::SeqCst);
    server.join().expect("the stand-in compositor");
    let got = got.borrow();
    let twists: Vec<_> = got.iter().filter_map(|i| if let Input::PadPinch(t) = i { Some(*t) } else { None }).collect();
    assert_eq!(twists.len(), 2, "both updates were heard: {got:?}");
    assert!((twists[0].scale - 1.5).abs() < 0.01 && (twists[1].scale - 2.0 / 1.5).abs() < 0.01, "{twists:?}");
    assert!((twists[0].turn - std::f32::consts::FRAC_PI_4).abs() < 0.01);
    assert_eq!((twists[0].centre, twists[0].moved), ((50.0, 40.0), (3.0, -1.0)), "about the pointer, and moved by the fingers");
    assert_eq!(got.last(), Some(&Input::PadPinchEnd { cancelled: false }));
}
