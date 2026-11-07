//! Budgets that fail on a regression: a window's start, from nothing to its first painted frame, a long list of pictures, a shadow on a big canvas, and the size of the smallest program.

use std::time::Duration;

use trindle_toolkit::colour::Colour;
use trindle_toolkit::icon::Icons;
use trindle_toolkit::text::Fonts;
use trindle_toolkit::theme::Theme;
use trindle_toolkit::tiles::{Sketch, Way};
use trindle_toolkit::ui::Ui;
use trindle_toolkit::widget::{avatar, column, label, main_word, row, scroll, Length, Node, Pen};

const START_MOST: Duration = Duration::from_millis(50);
const LONG_LIST_MOST: Duration = Duration::from_millis(30);
const SHADOW_MOST: Duration = Duration::from_millis(12);
const HELLO_MOST: u64 = 18 << 20;

fn cpu() -> Duration {
    let said = std::fs::read_to_string("/proc/thread-self/schedstat").expect("the kernel counts each thread's time");
    Duration::from_nanos(said.split_whitespace().next().and_then(|n| n.parse().ok()).expect("a count of nanoseconds"))
}

#[test]
fn a_window_starts_within_its_budget() {
    let began = cpu();
    let mut ui: Ui<()> = Ui::new(column(vec![label("Not pressed yet"), main_word("Press me").on_press(())]).padding(24.0), Theme::new(false, "#fbc711"));
    let mut pix = tiny_skia::Pixmap::new(320, 160).unwrap();
    ui.paint(&mut pix.as_mut(), 1.0);
    let took = cpu() - began;
    println!("started in {took:?}");
    assert!(took < START_MOST, "the first frame took {took:?}, over the budget of {START_MOST:?}");
}

#[test]
fn the_smallest_program_is_within_its_budget() {
    let deps = std::env::current_exe().unwrap();
    let hello = deps.parent().and_then(|d| d.parent()).unwrap().join("examples").join("hello");
    let elf = std::fs::read(&hello).unwrap_or_else(|_| panic!("{} is built by cargo test with the examples", hello.display()));
    assert_eq!(elf[..6], [0x7f, b'E', b'L', b'F', 2, 1], "a 64-bit little-endian program");
    let at = |o: usize, n: usize| elf[o..o + n].iter().rev().fold(0u64, |v, &b| (v << 8) | b as u64) as usize;
    let (table, each, count) = (at(0x20, 8), at(0x36, 2), at(0x38, 2));
    let size: u64 = (0..count).map(|i| table + i * each).filter(|&h| at(h, 4) == 1).map(|h| at(h + 0x20, 8) as u64).sum();
    println!("hello loads {size} bytes");
    assert!(size < HELLO_MOST, "hello loads {size} bytes, over the budget of {HELLO_MOST}");
}

fn least(runs: usize, mut run: impl FnMut()) -> Duration {
    (0..runs).map(|_| {
        let began = cpu();
        run();
        cpu() - began
    }).min().unwrap()
}

#[test]
fn a_long_list_of_pictures_costs_what_shows_within_its_budget() {
    let page = || -> Node<()> {
        scroll("people", column((0..500).map(|i| row(vec![avatar(None, format!("John Smith {i}"), 40.0), label(format!("John Smith {i}")).width(Length::Fill)]).gap(8.0)).collect()).gap(8.0)).height(Length::Fill)
    };
    let mut ui: Ui<()> = Ui::new(page(), Theme::new(false, "#fbc711"));
    let mut pix = tiny_skia::Pixmap::new(1280, 800).unwrap();
    ui.paint(&mut pix.as_mut(), 2.0);
    let took = least(5, || {
        ui.set_view(page());
        ui.paint(&mut pix.as_mut(), 2.0);
    });
    println!("500 rows with pictures: {took:?}");
    assert!(took < LONG_LIST_MOST, "500 rows with pictures took {took:?}, over the budget of {LONG_LIST_MOST:?}");
}

#[test]
fn a_shadow_on_a_big_canvas_costs_by_the_shape_within_its_budget() {
    let (theme, fonts, icons) = (Theme::new(false, "#fbc711"), Fonts::new(), Icons::new());
    let pen = Pen { theme: &theme, fonts: &fonts, icons: &icons, scale: 1.0 };
    let mut pix = tiny_skia::Pixmap::new(1920, 1080).unwrap();
    let took = least(5, || {
        let mut canvas = pix.as_mut();
        Sketch::placed(&mut canvas, &pen, 1.0, (0.0, 0.0)).shadow(&Way::rounded((50.0, 50.0, 200.0, 120.0), 12.0), (0.0, 4.0), 16.0, Colour { r: 0.0, g: 0.0, b: 0.0, a: 0.4 });
    });
    println!("a shadow on 1920 x 1080: {took:?}");
    assert!(pix.pixels().iter().any(|p| p.alpha() > 0), "the shadow drew nothing, so its time says nothing");
    assert!(took < SHADOW_MOST, "a shadow took {took:?}, over the budget of {SHADOW_MOST:?}");
}
