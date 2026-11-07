//! The window's own sums, and the stand-in mouse and keyboard tests elsewhere act through.

use super::*;
use super::display::takes_keys;
use super::painting::{catch_up, changed_parts, pixels_for};
use super::surface_events::{fit, is_suspended};
use super::typer::around;
use crate::event::Flow;

use wayland_client::protocol::wl_registry;
use wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_surface_v1::{Anchor, KeyboardInteractivity};
use wayland_protocols_misc::zwp_virtual_keyboard_v1::client::{
    zwp_virtual_keyboard_manager_v1::ZwpVirtualKeyboardManagerV1, zwp_virtual_keyboard_v1::ZwpVirtualKeyboardV1,
};
use wayland_protocols_wlr::virtual_pointer::v1::client::{
    zwlr_virtual_pointer_manager_v1::ZwlrVirtualPointerManagerV1, zwlr_virtual_pointer_v1::ZwlrVirtualPointerV1,
};

#[test]
fn every_panel_that_asks_for_the_keyboard_and_is_out_holds_it_and_nothing_else_does() {
    for keys in [Keys::Always, Keys::OnClick] {
        assert!(takes_keys(true, keys, false), "{keys:?}");
        assert!(!takes_keys(true, keys, true), "a panel put away still held the keyboard: {keys:?}");
        assert!(!takes_keys(false, keys, false), "a window was counted as a panel");
    }
    assert!(!takes_keys(true, Keys::Never, false));
    let told = |keys| keyboard_interactivity(keys) != KeyboardInteractivity::None;
    for keys in [Keys::Never, Keys::OnClick, Keys::Always] {
        assert_eq!(takes_keys(true, keys, false), told(keys), "{keys:?}");
    }
}

#[test]
fn a_window_too_big_for_the_screen_starts_at_what_fits() {
    assert_eq!(fit((1200, 760), (1278, 625)), Some((1200, 625)));
    assert_eq!(fit((1200, 760), (1000, 600)), Some((1000, 600)));
    assert_eq!(fit((800, 600), (1920, 1040)), None);
    assert_eq!(fit((800, 600), (0, 0)), None);
}

pub(crate) struct Hands {
    pub conn: Connection,
    pub queue: wayland_client::EventQueue<Lent>,
    pub mouse: Option<ZwlrVirtualPointerV1>,
    pub keys: Option<ZwpVirtualKeyboardV1>,
}

pub(crate) struct Lent;
macro_rules! lent_ignore {
    ($($t:ty),*) => {$(
        impl Dispatch<$t, ()> for Lent {
            fn event(_: &mut Self, _: &$t, _: <$t as Proxy>::Event, _: &(), _: &Connection, _: &QueueHandle<Self>) {}
        }
    )*};
}
lent_ignore!(wl_seat::WlSeat, ZwpVirtualKeyboardManagerV1, ZwpVirtualKeyboardV1, ZwlrVirtualPointerManagerV1,
             ZwlrVirtualPointerV1);
impl Dispatch<wl_registry::WlRegistry, wayland_client::globals::GlobalListContents> for Lent {
    fn event(_: &mut Self, _: &wl_registry::WlRegistry, _: wl_registry::Event,
             _: &wayland_client::globals::GlobalListContents, _: &Connection, _: &QueueHandle<Self>) {}
}

impl Hands {
    pub fn lend() -> Hands {
        use std::io::Write;
        use std::os::fd::AsFd;
        use xkbcommon::xkb;
        let conn = Connection::connect_to_env().expect("a second connection");
        let (globals, mut queue) = wayland_client::globals::registry_queue_init::<Lent>(&conn).expect("globals");
        let qh = queue.handle();
        let seat: wl_seat::WlSeat = globals.bind(&qh, 1..=1, ()).expect("a seat");
        let mouse = globals.bind::<ZwlrVirtualPointerManagerV1, _, _>(&qh, 1..=1, ()).ok()
            .map(|m| m.create_virtual_pointer(Some(&seat), &qh, ()));
        let keys = globals.bind::<ZwpVirtualKeyboardManagerV1, _, _>(&qh, 1..=1, ()).ok()
            .map(|m| m.create_virtual_keyboard(&seat, &qh, ()));
        if let Some(k) = &keys {
            let context = xkb::Context::new(xkb::CONTEXT_NO_FLAGS);
            let map = xkb::Keymap::new_from_names(&context, "", "pc105", "gb", "", None, xkb::KEYMAP_COMPILE_NO_FLAGS)
                .expect("the gb layout");
            let text = map.get_as_string(xkb::KEYMAP_FORMAT_TEXT_V1);
            let file = std::env::temp_dir().join(format!("trindle-keymap-{}", std::process::id()));
            let mut f = std::fs::File::options().read(true).write(true).create(true).truncate(true).open(&file).unwrap();
            f.write_all(text.as_bytes()).unwrap();
            f.write_all(&[0]).unwrap();
            k.keymap(1, f.as_fd(), text.len() as u32 + 1);
            std::fs::remove_file(&file).unwrap();
        }
        queue.roundtrip(&mut Lent).unwrap();
        Hands { conn, queue, mouse, keys }
    }
}

#[test]
fn edges_match_the_layer_shell() {
    let all = Edges::TOP | Edges::BOTTOM | Edges::LEFT | Edges::RIGHT;
    assert_eq!(all, Edges::ALL);
    assert_eq!(Anchor::from_bits_truncate(all.0), Anchor::Top | Anchor::Bottom | Anchor::Left | Anchor::Right);
    assert_eq!(Anchor::from_bits_truncate(Edges::LEFT.0), Anchor::Left);
}

#[test]
fn a_window_takes_the_finer_scale_else_the_largest_screens() {
    for (fine, want) in [(120, 1.0), (150, 1.25), (180, 1.5), (210, 1.75), (240, 2.0)] {
        let (scale, size) = pixels_for((801, 603), Some(fine), [1, 2].into_iter());
        assert_eq!(scale, want);
        assert_eq!(size, ((801.0 * want as f64).ceil() as u32, (603.0 * want as f64).ceil() as u32), "{want}");
    }
    assert_eq!(pixels_for((400, 300), None, [1, 2].into_iter()), (2.0, (800, 600)), "across a plain and a sharp screen");
    assert_eq!(pixels_for((400, 300), None, [].into_iter()), (1.0, (400, 300)), "on no screen yet");
    assert_eq!(pixels_for((400, 300), None, [0, -3].into_iter()), (1.0, (400, 300)), "a screen that says nonsense");
    assert_eq!(pixels_for((MOST_SIDE, 10), Some(240), [].into_iter()).1, (MOST_SIDE, 20), "never past the largest side");
}

#[test]
fn a_suspended_window_is_told_apart() {
    let states = |v: &[u32]| v.iter().flat_map(|n| n.to_ne_bytes()).collect::<Vec<u8>>();
    assert!(!is_suspended(&states(&[])));
    assert!(!is_suspended(&states(&[1, 4])), "maximised and active is seen");
    assert!(is_suspended(&states(&[4, 9])));
    assert!(has_state(&states(&[1, 4]), 1), "maximised is seen");
    assert!(!has_state(&states(&[4, 9]), 1));
}

#[test]
fn red_and_blue_change_places_for_the_compositor() {
    let mut px = [1, 2, 3, 4, 5, 6, 7, 8];
    to_compositor(&mut px);
    assert_eq!(px, [3, 2, 1, 4, 7, 6, 5, 8]);
}

#[test]
fn a_nul_byte_is_taken_out_of_what_the_protocol_is_given() {
    assert_eq!(no_nul("a\0b\0"), "ab");
}

#[test]
fn the_input_method_sees_the_words_round_the_caret_within_its_limit() {
    assert_eq!(around("hello", 2, 4), ("hello".to_string(), 2, 4));
    let text = "é".repeat(5000);
    let (part, caret, anchor) = around(&text, 6000, 6000);
    assert!(part.len() < 4000 && part.chars().all(|c| c == 'é'));
    assert_eq!(caret, anchor);
    assert!(caret > 1900 && caret < 2100 && part.is_char_boundary(caret));
    let (part, caret, anchor) = around(&text, 6100, 6000);
    assert_eq!((caret - anchor, part.len() < 4000), (100, true));
    let (part, caret, anchor) = around(&text, 9000, 10);
    assert!(part.len() < 4000 && anchor == 0 && caret > 1900);
    let wide = "\u{1F600}".repeat(1500);
    for caret in (0..6000).step_by(4) {
        assert!(around(&wide, caret, caret).0.len() < 4000, "at {caret}");
    }
    let uneven = format!("{}\u{1F600}{}", "a".repeat(3998), "a".repeat(6000));
    let (part, caret, _) = around(&uneven, 6000, 6000);
    assert!(part.len() < 4000 && part.starts_with('a') && caret < part.len(), "{} bytes", part.len());
    let (part, caret, _) = around(&text, text.len(), text.len());
    assert_eq!((caret, text.ends_with(&part)), (part.len(), true));
}

#[test]
#[ignore = "needs a real screen and compositor: run by hand"]
fn windows_show_on_a_real_compositor() {
    use tiny_skia::{Color, Paint, Rect, Transform};
    let events = Loop::new();
    let display = Display::connect(&events, || panic!("compositor gone")).expect("connected");
    let blue = Rc::new(Cell::new(false));
    let seen = blue.clone();
    let window = display.open(Kind::Window { title: "Trindle Toolkit".into(), app_id: "org.trindle.Test".into() },
                              (400, 300), move |canvas, scale| {
        canvas.fill(Color::from_rgba8(9, 9, 11, 255));
        let mut paint = Paint::default();
        paint.set_color_rgba8(70, 130, 220, 255);
        let rect = Rect::from_xywh(20.0, 20.0, 100.0, 60.0).unwrap();
        canvas.fill_rect(rect, &paint, Transform::from_scale(scale, scale), None);
        let at = (30.0 * scale) as usize;
        let i = (at * canvas.width() as usize + at) * 4;
        seen.set(canvas.data_mut()[i..i + 4] == [70, 130, 220, 255]);
    }).expect("a window");
    let panel = display.open(Kind::Panel(Panel {
        layer: Layer::Top, edges: Edges::TOP | Edges::LEFT | Edges::RIGHT, keep: 32, margins: [0; 4],
        keys: Keys::Never, name: "test-panel".into(), screen: None,
    }), (0, 32), |canvas, _| canvas.fill(Color::from_rgba8(60, 60, 64, 255))).expect("a panel");
    let quit = events.clone();
    events.after(std::time::Duration::from_secs(4), move || quit.quit());
    events.run();
    assert!(blue.get(), "the window's last paint did not put the blue square where it was drawn");
    assert!(panel.size().0 > 0 && window.size().0 > 0);
}

#[test]
fn only_the_boxes_that_changed_are_handed_over_as_damaged() {
    let (w, h) = (64usize, 80usize);
    let len = w * h * 4;
    let mut memory = vec![0u8; len * 2];
    assert_eq!(changed_parts(&memory, (64, 80), 0, 1), [[0, 0, 1, 1]]);
    for (x, y) in [(10usize, 3usize), (12, 4), (50, 40)] {
        memory[(y * w + x) * 4] = 9;
    }
    assert_eq!(changed_parts(&memory, (64, 80), 0, 1), [[10, 0, 3, 16], [50, 32, 1, 16]]);
    memory[(20 * w + 11) * 4] = 9;
    assert_eq!(changed_parts(&memory, (64, 80), 0, 1)[0], [10, 0, 3, 32]);
    let h = 300usize;
    let mut memory = vec![0u8; w * h * 4 * 2];
    for band in 0..19 {
        memory[((band * 16) * w + if band % 2 == 0 { 2 } else { 50 }) * 4] = 7;
    }
    assert_eq!(changed_parts(&memory, (64, 300), 0, 1), [[2, 0, 49, 300]]);
}

#[test]
fn a_buffer_that_is_behind_is_given_only_the_parts_it_missed() {
    let mut memory = [vec![1u8; 24], vec![2u8; 24]].concat();
    catch_up(&mut memory, (3, 2), 0, 1, [1, 0, 1, 2]);
    let second: Vec<u8> = memory[24..].chunks_exact(4).map(|p| p[0]).collect();
    assert_eq!(second, [2, 1, 2, 2, 1, 2], "the middle column alone");
    assert!(memory[..24].iter().all(|b| *b == 1), "the one copied from is untouched");
    catch_up(&mut memory, (3, 2), 0, 1, [-5, -5, 6, 6]);
    let second: Vec<u8> = memory[24..].chunks_exact(4).map(|p| p[0]).collect();
    assert_eq!(second, [1, 1, 2, 2, 1, 2]);
    let before = memory.clone();
    catch_up(&mut memory, (3, 2), 0, 1, [9, 9, 4, 4]);
    catch_up(&mut memory, (3, 2), 0, 1, [0, 0, 0, 0]);
    assert_eq!(memory, before);
}

fn hands_at(now: u32, mouse: &Option<ZwlrVirtualPointerV1>, keys: &Option<ZwpVirtualKeyboardV1>) {
    let ms = now * 300;
    if let Some(m) = mouse {
        match now {
            3 => { m.motion_absolute(ms, 1, 1, 2, 2); m.frame(); }
            4 => { m.motion(ms, 10.0, 5.0); m.frame(); }
            5 => { m.button(ms, 0x110, wl_pointer::ButtonState::Pressed); m.frame(); }
            6 => { m.button(ms, 0x110, wl_pointer::ButtonState::Released); m.frame(); }
            7 => {
                m.axis_source(wl_pointer::AxisSource::Wheel);
                m.axis_discrete(ms, wl_pointer::Axis::VerticalScroll, 15.0, 1);
                m.frame();
            }
            _ => {}
        }
    }
    if let Some(k) = keys {
        match now {
            8 => k.key(ms, 30, 1),
            9 => k.key(ms, 30, 0),
            10 => k.key(ms, 48, 1),
            15 => k.key(ms, 48, 0),
            16 => { k.key(ms, 28, 1); k.key(ms, 28, 0); }
            _ => {}
        }
    }
}

fn what_was_heard(heard: &[Input], has_keys: bool, has_mouse: bool) {
    let typed: String = heard.iter().filter_map(|i| match i {
        Input::Key { text: Some(t), .. } => Some(t.as_str()),
        _ => None,
    }).collect();
    eprintln!("typed {typed:?}");
    if has_keys {
        assert!(heard.contains(&Input::Focus(true)));
        assert!(typed.starts_with("abb"), "the held key repeats");
        assert!(heard.iter().any(|i| matches!(i, Input::Key { sym: super::super::input::Keysym::Return, text: None, .. })));
    }
    if has_mouse {
        assert!(heard.iter().any(|i| matches!(i, Input::PointerIn { .. })));
        assert!(heard.iter().any(|i| matches!(i, Input::Press { button: Button::Left, .. })));
        assert!(heard.iter().any(|i| matches!(i, Input::Release { button: Button::Left, .. })));
        assert!(heard.iter().any(|i| matches!(i, Input::Scroll { y, steps: true, .. } if *y == 1.0)));
    }
}

#[test]
#[ignore = "needs a real screen and compositor: run by hand"]
fn windows_hear_the_mouse_and_keyboard() {
    let events = Loop::new();
    let display = Display::connect(&events, || panic!("compositor gone")).expect("connected");
    let window = display.open(Kind::Window { title: "Input".into(), app_id: "org.trindle.Test".into() },
                              (1200, 800), |canvas, _| canvas.fill(tiny_skia::Color::from_rgba8(9, 9, 11, 255)))
        .expect("a window");
    let heard = Rc::new(RefCell::new(Vec::new()));
    let h = heard.clone();
    window.on_input(move |input| h.borrow_mut().push(input));
    window.set_cursor(Cursor::Hand);

    let Hands { conn, mouse, keys, queue: _queue } = Hands::lend();
    let t = Rc::new(Cell::new(0u32));
    let steps = events.clone();
    let (has_keys, has_mouse) = (keys.is_some(), mouse.is_some());
    events.every(std::time::Duration::from_millis(300), move || {
        let now = t.get();
        t.set(now + 1);
        hands_at(now, &mouse, &keys);
        let _ = conn.flush();
        if now == 18 {
            steps.quit();
            return Flow::Stop;
        }
        Flow::Continue
    });
    events.run();
    let heard = heard.borrow();
    for i in heard.iter() {
        eprintln!("heard {i:?}");
    }
    what_was_heard(&heard, has_keys, has_mouse);
}
