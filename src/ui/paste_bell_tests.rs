//! A grid a program paints itself: the clipboard is asked for only by the person's own key press, and the bell is limited.

use super::*;
use super::super::host::{with, Host};
use super::super::theme::Theme;
use super::super::widget::{column, drawing};
use super::super::window::{note_press_code, own_key_now, refused_act, with_named, with_press};

struct Shell {
    paste: bool,
    bell: bool,
}

impl App for Shell {
    type Msg = ();
    fn update(&mut self, _: ()) {}
    fn view(&self) -> Node<()> {
        drawing((100.0, 100.0), "Terminal", |_, _| {})
    }
    fn wants_paste(&mut self) -> bool {
        std::mem::take(&mut self.paste)
    }
    fn wants_bell(&mut self) -> bool {
        std::mem::take(&mut self.bell)
    }
}

const KEY: u32 = 30;
const CLICK: u32 = 0x110;

fn plain() -> Ui<()> {
    Ui::<()>::new(column(vec![]), Theme::new(false, "#fbc711"))
}

#[test]
fn the_clipboard_is_read_only_for_the_persons_own_key_press() {
    note_press_code(7001, KEY);
    note_press_code(7002, CLICK);
    note_press_code(7003, KEY);
    let asks = |named: bool, press: Option<u32>| {
        let mut app = Shell { paste: true, bell: false };
        let granted = with_named(named, Some(7001), || with_press(press, || paste_asked(&mut app)));
        assert!(!app.paste, "the wish was not taken out");
        granted
    };
    assert!(asks(true, Some(7001)), "the person's own key was refused");
    assert!(!refused_act(), "a granted wish was said to be refused");
    assert!(!asks(true, None), "the clipboard was read with no key press behind it");
    assert!(refused_act(), "the refusal was not said");
    assert!(!asks(true, Some(7002)), "a click could ask for the clipboard");
    refused_act();
    assert!(!asks(true, Some(7003)), "a key the compositor never named as the person's own asked");
    refused_act();
    assert!(asks(false, Some(7003)), "where no presses are named, a key was refused");
    assert!(!asks(false, None), "a wish with no key press at all was granted");
    refused_act();
    let mut quiet = Shell { paste: false, bell: false };
    assert!(!with_press(None, || paste_asked(&mut quiet)));
    assert!(!refused_act());
}

#[test]
fn only_a_key_press_counts_as_the_persons_own_key() {
    note_press_code(7011, KEY);
    assert!(!own_key_now(), "no press and still own");
    assert!(with_named(true, Some(7011), || with_press(Some(7011), own_key_now)));
    assert!(!with_named(true, Some(1), || with_press(Some(7011), own_key_now)));
}

#[test]
fn a_bell_flashes_the_edge_and_is_let_go_after() {
    let mut ui = plain();
    let t0 = Instant::now();
    assert!(ui.ring_bell(t0));
    let time = theme::sheet().bell;
    assert!(ui.bell_at(t0 + time / 2) > 0.0);
    assert_eq!(ui.bell_at(t0 + time), 0.0);
    assert!(ui.next_wake(t0).is_some(), "the flash would not be painted again");
    assert!(ui.tick(t0 + time), "the last frame was not asked for");
    assert!(ui.next_wake(t0 + time).is_none(), "the flash never ended");
}

#[test]
fn bells_come_at_most_three_a_second() {
    let mut ui = plain();
    let t0 = Instant::now();
    assert!(ui.ring_bell(t0));
    let rung = (1..100).filter(|i| ui.ring_bell(t0 + Duration::from_millis(i * 10))).count();
    assert!(rung <= 2, "{rung} more flashes in a second, the limit is three");
    assert!(ui.ring_bell(t0 + Duration::from_secs(2)));
}

#[test]
fn the_bell_is_taken_out_so_it_rings_once() {
    let mut app = Shell { paste: false, bell: true };
    assert!(app.wants_bell());
    assert!(!app.wants_bell());
}

#[test]
fn where_nothing_moves_the_bell_does_not_fade() {
    struct Still;
    impl Host for Still {
        fn moving(&self) -> bool {
            false
        }
    }
    static STILL: Still = Still;
    with(&STILL, || {
        let mut ui = plain();
        let t0 = Instant::now();
        ui.ring_bell(t0);
        let time = theme::sheet().bell;
        for share in [0.01, 0.1, 0.5, 0.9] {
            assert_eq!(ui.bell_at(t0 + time.mul_f32(share)), 1.0, "it faded at {share}");
        }
        assert_eq!(ui.bell_at(t0 + time), 0.0);
    });
    let mut ui = plain();
    let t0 = Instant::now();
    ui.ring_bell(t0);
    let time = theme::sheet().bell;
    assert!(ui.bell_at(t0 + time.mul_f32(0.05)) < ui.bell_at(t0 + time.mul_f32(0.2)));
    assert!(ui.bell_at(t0 + time.mul_f32(0.9)) < 1.0);
}

#[test]
fn the_flash_is_painted_on_the_window_edge_and_only_there() {
    let mut ui = plain();
    let mut calm = tiny_skia::Pixmap::new(200, 100).unwrap();
    ui.paint(&mut calm.as_mut(), 1.0);
    ui.ring_bell(super::tests::ago(theme::sheet().bell / 5));
    let mut flash = tiny_skia::Pixmap::new(200, 100).unwrap();
    ui.paint(&mut flash.as_mut(), 1.0);
    assert_ne!(calm.pixel(100, 0), flash.pixel(100, 0), "the top edge did not flash");
    assert_ne!(calm.pixel(0, 50), flash.pixel(0, 50), "the left edge did not flash");
    assert_eq!(calm.pixel(100, 50), flash.pixel(100, 50), "the middle changed");
}

#[test]
fn the_clipboard_is_read_from_guarded_places_only() {
    use crate::common::source::code;
    let src = [code(include_str!("../ui.rs")), code(include_str!("delivering.rs")), code(include_str!("following.rs")),
               code(include_str!("settling.rs")), code(include_str!("shown.rs")), code(include_str!("timing.rs"))].concat();
    assert_eq!(src.matches(".wants_paste()").count(), 1, "the clipboard is asked for in more than one place");
    assert_eq!(src.matches("paste_asked(&mut").count(), 2, "a way round paste_asked");
    assert_eq!(src.matches("read_clipboard(").count(), 2, "the clipboard is read from somewhere unguarded");
}

#[test]
fn a_programs_announcement_is_kept_once_for_the_reader() {
    struct Scan(Option<String>);
    impl App for Scan {
        type Msg = ();
        fn update(&mut self, _: ()) {}
        fn view(&self) -> Node<()> {
            drawing((10.0, 10.0), "Scan", |_, _| {})
        }
        fn announcement(&mut self) -> Option<String> {
            self.0.take()
        }
    }
    let mut ui = plain();
    let mut app = Scan(Some("Scanning page 4".into()));
    assert!(announced(&mut app, &mut ui));
    assert_eq!(ui.news, "Scanning page 4");
    assert!(!announced(&mut app, &mut ui), "taken out, so said once");
    app.0 = Some("Scanning page 4".into());
    assert!(!announced(&mut app, &mut ui), "the same words twice run are said once");
    app.0 = Some("Scanning page 5".into());
    assert!(announced(&mut app, &mut ui));
}
