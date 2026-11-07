//! What the 30 October reviews said was slow, timed on the real code: processor time of the calling thread, the median of several runs.
use crate::editing::area::TextArea;
use crate::editing::editor::Look;
use crate::icongrid::{paint_icons_on, IconGrid};
use crate::input::Mods;
use crate::model::ListModel;
use crate::paint::Painter;
use crate::spelling::Speller;
use crate::sys::os::thread_time;
use crate::text::Fonts;
use crate::theme::Theme;
use crate::tiles::{Sketch, Way};
use crate::ui::Ui;
use crate::widget::{label, layers, placed, Node, Pen, Pointer};
use std::rc::Rc;

fn median(mut v: Vec<f64>) -> f64 {
    v.sort_by(|a, b| a.total_cmp(b));
    v[v.len() / 2]
}

fn ms(n: usize, mut f: impl FnMut()) -> f64 {
    median((0..n).map(|_| {
        let t = thread_time();
        f();
        (thread_time() - t).as_secs_f64() * 1e3
    }).collect())
}

struct Knows;

impl Speller for Knows {
    fn known(&self, word: &str) -> bool {
        !word.is_empty()
    }
}

#[test]
#[ignore = "a timing run, for a machine with nothing else busy"]
fn measure_text_area() {
    let look = Look { font: None, size: 14.0, wrap: true, numbers: false };
    let line = "the quick brown fox jumps over the lazy dog, again and again\n";
    for (name, text) in [("1,000 lines", line.repeat(1000)), ("100,000 lines", line.repeat(100_000)), ("one line of 2 MB", line.replace('\n', " ").repeat(33_000))] {
        let area = TextArea::new(&text, look.clone());
        area.spelling(Some(Rc::new(Knows)));
        let took = ms(9, || { std::hint::black_box(area.node::<()>("notes", 100.0, |_| ())); });
        println!("| text area, {name}, spell checked | node() {took:.3} ms |");
    }
}

#[test]
#[ignore = "a timing run, for a machine with nothing else busy"]
fn measure_icon_grid() {
    let size = (800.0, 600.0);
    let ctrl = Mods { ctrl: true, ..Mods::default() };
    for count in [1_000, 10_000, 100_000] {
        let between = (crate::theme::sheet().icon_tile + crate::theme::sheet().gap / 2.0, 1.0);
        let mut plain = IconGrid::new(count);
        plain.pointer(Pointer::Down(between.0, between.1), Mods::default(), size);
        assert!(plain.band_rect().is_some(), "the press was on a tile");
        let mut y = 1.0;
        let drag = ms(9, || { y += 1.0; plain.pointer(Pointer::Move(700.0, y), Mods::default(), size); });
        assert!(!plain.selection.chosen.is_empty());
        let mut all = IconGrid::new(count);
        all.selection.all(count);
        all.pointer(Pointer::Down(between.0, between.1), ctrl, size);
        assert!(all.band_rect().is_some(), "the press was on a tile");
        let mut y = 1.0;
        let kept = ms(9, || { y += 1.0; all.pointer(Pointer::Move(700.0, y), ctrl, size); });
        assert_eq!(all.selection.chosen.len(), count);
        let (theme, fonts, icons) = (Theme::new(false, "#fbc711"), Fonts::new(), crate::icon::Icons::new());
        let pen = Pen { theme: &theme, fonts: &fonts, icons: &icons, scale: 1.0 };
        let mut grid = IconGrid::new(count);
        grid.top = grid.height(size.0) / 2.0;
        let paint = ms(9, || {
            let mut p = Painter::default();
            paint_icons_on(&mut p, &pen, (800, 600), &grid, None, false, |i| ("text-x-generic".into(), format!("File {i}")));
            std::hint::black_box(p);
        });
        println!("| icon grid, {count} tiles | band move {drag:.3} ms | band move, all chosen first {kept:.3} ms | record a paint {paint:.3} ms |");
    }
}

#[test]
#[ignore = "a timing run, for a machine with nothing else busy"]
fn measure_model_sort() {
    for n in [1_000usize, 100_000] {
        let items: Vec<(String, u32)> = (0..n).map(|i| (format!("File {}", (i * 7919) % n), (i % 26) as u32)).collect();
        let mut plain = ListModel::new(items.clone());
        let sorted = ms(5, || plain.sort_by(|a, b| a.0.cmp(&b.0)));
        let mut grouped = ListModel::new(items);
        grouped.sort_by(|a, b| a.0.cmp(&b.0));
        let group = ms(5, || grouped.group_by(|t| format!("Group {}", t.1)));
        println!("| list model, {n} things | sorted {sorted:.2} ms | sorted and grouped {group:.2} ms |");
    }
}

#[test]
#[ignore = "a timing run, for a machine with nothing else busy"]
fn measure_sketch_shadow() {
    let (theme, fonts, icons) = (Theme::new(false, "#fbc711"), Fonts::new(), crate::icon::Icons::new());
    let pen = Pen { theme: &theme, fonts: &fonts, icons: &icons, scale: 1.0 };
    for (w, h) in [(400u32, 300u32), (1920, 1080), (3840, 2160)] {
        let mut pix = tiny_skia::Pixmap::new(w, h).unwrap();
        let shadow = ms(5, || {
            let mut canvas = pix.as_mut();
            let mut s = Sketch::placed(&mut canvas, &pen, 1.0, (0.0, 0.0));
            s.shadow(&Way::rounded((50.0, 50.0, 200.0, 120.0), 12.0), (0.0, 4.0), 16.0, crate::colour::Colour { r: 0.0, g: 0.0, b: 0.0, a: 0.4 });
        });
        let clip = ms(5, || {
            let mut canvas = pix.as_mut();
            let mut s = Sketch::placed(&mut canvas, &pen, 1.0, (0.0, 0.0));
            s.clipped(&Way::rounded((50.0, 50.0, 200.0, 120.0), 12.0), |s| s.rect((50.0, 50.0, 200.0, 120.0), 0.0, crate::colour::Colour { r: 1.0, g: 0.0, b: 0.0, a: 1.0 }));
        });
        println!("| sketch on {w} x {h}, a 200 x 120 shape | shadow {shadow:.2} ms | clipped {clip:.2} ms |");
    }
}

#[test]
#[ignore = "a timing run, for a machine with nothing else busy"]
fn measure_small_costs() {
    let fonts = Fonts::new();
    let texts = crate::widget::Texts::default();
    let style = crate::widget::Words::Plain.style(1.0);
    for len in [20usize, 200, 5000] {
        let words = "word ".repeat(len / 5);
        texts.get(&fonts, &words, &style, Some(300.0));
        let hit = ms(9, || { for _ in 0..1000 { std::hint::black_box(texts.get(&fonts, &words, &style, Some(300.0))); } }) / 1000.0;
        println!("| text lookup that hits, {len} letters | {:.1} us |", hit * 1e3);
    }
    let weekday = ms(9, || { for _ in 0..10_000 { std::hint::black_box(crate::when::first_weekday()); } }) / 10_000.0;
    println!("| first_weekday | {:.2} us |", weekday * 1e3);
    for n in [10usize, 100, 1000] {
        let page = |n: usize| layers((0..n).map(|i| placed(label("x"), |_, _| (10.0, 10.0)).stays(format!("window-{i}"))).collect());
        let mut ui: Ui<()> = Ui::new(page(n), Theme::new(false, "#fbc711"));
        ui.lay_at(800, 600);
        let took = ms(9, || { ui.set_view(page(n)); ui.lay_at(800, 600); });
        println!("| lay out {n} places kept by name | {took:.3} ms |");
    }
}

#[test]
fn a_picture_scrolled_out_of_sight_is_not_made_ready_until_it_shows() {
    use crate::widget::{avatar, column, image_view, row, scroll, Length};
    let photos = |n: usize| -> Vec<Rc<image::DynamicImage>> { (0..n).map(|i| Rc::new(image::DynamicImage::ImageRgb8(image::RgbImage::from_pixel(64, 64, image::Rgb([i as u8, 9, 9]))))).collect() };
    let (faces, views) = (photos(300), photos(300));
    let page = || -> Node<()> {
        scroll("s", column((0..300).map(|i| row(vec![avatar(Some(faces[i].clone()), "John Smith", 40.0), image_view(views[i].clone(), "John Smith", (40.0, 40.0))]).gap(8.0)).collect()).gap(8.0)).height(Length::Fill)
    };
    let made = |of: &[Rc<image::DynamicImage>]| of.iter().filter(|p| Rc::weak_count(p) > 0).count();
    let mut ui: Ui<()> = Ui::new(page(), Theme::new(false, "#fbc711"));
    let mut pix = tiny_skia::Pixmap::new(400, 300).unwrap();
    ui.paint(&mut pix.as_mut(), 1.0);
    for of in [&faces, &views] {
        assert!((1..=20).contains(&made(of)), "{} pictures were made ready for a window showing about six", made(of));
        assert_eq!(Rc::weak_count(&of[250]), 0);
    }
    assert!(ui.scroll_area("s", 12_000.0));
    ui.paint(&mut pix.as_mut(), 1.0);
    for of in [&faces, &views] {
        assert!(Rc::weak_count(&of[250]) > 0, "a picture that came into sight was not made ready");
    }
}

#[test]
#[ignore = "a timing run, for a machine with nothing else busy"]
fn measure_off_screen_drawings() {
    use crate::widget::{avatar, column, icon_circle, icon_image, image_view, row, scroll, Length, Tone};
    let photo = |i: u32| Rc::new(image::DynamicImage::ImageRgb8(image::RgbImage::from_fn(256, 192, |x, y| image::Rgb([x as u8, y as u8, i as u8]))));
    let photos: Vec<_> = (0..500).map(photo).collect();
    let page = |what: &dyn Fn(usize) -> Node<()>, n: usize| -> Node<()> {
        scroll("s", column((0..n).map(|i| row(vec![what(i), label(format!("John Smith {i}")).width(Length::Fill)]).gap(8.0)).collect()).gap(8.0)).height(Length::Fill)
    };
    let kinds: Vec<(&str, Box<dyn Fn(usize) -> Node<()>>)> = vec![
        ("avatar, initials", Box::new(|i| avatar(None, format!("John Smith {i}"), 40.0))),
        ("avatar, photo", { let p = photos.clone(); Box::new(move |i| avatar(Some(p[i].clone()), "John Smith", 40.0)) }),
        ("image_view", { let p = photos.clone(); Box::new(move |i| image_view(p[i].clone(), "John Smith", (40.0, 40.0))) }),
        ("icon_circle", Box::new(|_| icon_circle("text-x-generic"))),
        ("icon_image", Box::new(|_| icon_image("text-x-generic", 24.0, Tone::Plain))),
    ];
    for (name, what) in &kinds {
        let mut took = Vec::new();
        for n in [20, 500] {
            let mut pix = tiny_skia::Pixmap::new(1280, 800).unwrap();
            let mut ui: Ui<()> = Ui::new(page(&**what, n), Theme::new(false, "#fbc711"));
            ui.paint(&mut pix.as_mut(), 2.0);
            took.push(ms(7, || { ui.set_view(page(&**what, n)); ui.paint(&mut pix.as_mut(), 2.0); }));
        }
        println!("| {name}, whole paint | 20 rows {:.2} ms | 500 rows {:.2} ms |", took[0], took[1]);
    }
}

#[test]
#[ignore = "a timing run, for a machine with nothing else busy"]
fn measure_fonts() {
    let fonts = Fonts::new();
    let style = crate::text::Style::new(14.0);
    let first = |text: &str| {
        let t = thread_time();
        let runs = fonts.fallbacks(text, &style);
        println!("| first look at {text:?} | {:.2} ms | {} runs, from {:?} |", (thread_time() - t).as_secs_f64() * 1e3, runs.len(), runs.last().and_then(|r| r.1.clone()));
    };
    first("hello");
    first("\u{4f60}\u{597d}");
    first("\u{10fffd}");
    first("\u{10fffc}");
    let again = ms(9, || { std::hint::black_box(fonts.fallbacks("\u{10fffd} \u{4f60}\u{597d} hello", &style)); });
    println!("| a look at letters already seen | {again:.3} ms |");
    println!("| chooser() | {:.1} ms |", ms(3, || { std::hint::black_box(fonts.chooser()); }));
    println!("| lang::script::system_families() | {:.1} ms |", ms(3, || { std::hint::black_box(crate::lang::script::system_families()); }));
}

#[test]
#[ignore = "a timing run, for a machine with nothing else busy"]
fn measure_sheet_reread() {
    crate::theme::reread();
    let one = ms(9, crate::theme::reread);
    println!("| one reading of the sheets | {one:.3} ms |");
}

#[test]
#[ignore = "a timing run, for a machine with nothing else busy"]
fn measure_mask_store() {
    use crate::sharp::{Hinting, MaskKey, Sharp};
    let data: &[u8] = include_bytes!("../share/fonts/NotoSans-Regular.ttf");
    let mut db = fontdb::Database::new();
    db.load_font_data(data.to_vec());
    let id = db.faces().next().expect("a face").id;
    let (sharp, hinting) = (Sharp::default(), Hinting::default());
    let key = |glyph: u16, size: f32| MaskKey { font: id, glyph, size: size.to_bits(), weight: 400, lean: false };
    sharp.mask(data, 0, "Noto Sans", &hinting, key(36, 13.0));
    let mut run = 0.0;
    let new = ms(7, || {
        run += 1.0;
        for g in 100..1100 { std::hint::black_box(sharp.mask(data, 0, "Noto Sans", &hinting, key(g, 14.0 + run))); }
    }) / 1000.0;
    let hit = ms(7, || { for g in 100..1100 { std::hint::black_box(sharp.mask(data, 0, "Noto Sans", &hinting, key(g, 14.0 + run))); } }) / 1000.0;
    println!("| letter not yet drawn | {:.1} us | letter kept | {:.2} us |", new * 1e3, hit * 1e3);
}
