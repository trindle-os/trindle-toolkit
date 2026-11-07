//! One page of every control the toolkit draws; run `cargo test -- --ignored shoot_the_gallery` with TRINDLE_SHOTS, TRINDLE_SHOT_ACCENT, TRINDLE_SHOT_TAIL.

use std::time::{Duration, Instant};

use super::input::{Button, Input};
use super::theme::Theme;
use super::ui::Ui;
use super::widget::{button, card, choice, column, dropdown, entry, even_grid, form_label, heading, headline, item, label, list, main_action, note,
                    page_title, remove, ring, row, slider, space, spinner, switch, Length, Node, Tone};

const SIZE: (u32, u32) = (1280, 800);

fn line(name: &str, about: &str, control: Node<()>) -> Node<()> {
    row(vec![column(vec![form_label(name), note(about)]).gap(crate::theme::gaps().tiny).width(Length::Fill), control]).gap(crate::theme::gaps().roomy).width(Length::Fill)
}

fn tile(name: &str, rows: Vec<Node<()>>) -> Node<()> {
    let mut all = vec![heading(name)];
    all.extend(rows);
    card(column(all).gap(14.0)).width(Length::Fill)
}

fn tiles(saved: u32) -> Vec<Node<()>> {
    let sizes = || vec!["Small".to_string(), "Medium".to_string(), "Large".to_string()];
    let towns = || ["Mildenhall", "Ely", "Thetford", "Brandon", "Newmarket"].iter().map(|t| t.to_string()).collect::<Vec<_>>();
    let buttons = tile("Buttons", vec![
        line("Round buttons", "At rest, on, and one that cannot be used.",
             row(vec![button("list-add-symbolic", "Add"), button("document-open-symbolic", "Chosen").on(true).on_press(()),
                      button("edit-copy-symbolic", "Copy").on_press(()).key("gallery:pointed"), button("edit-paste-symbolic", "Paste").off(true)]).gap(crate::theme::gaps().medium).width(Length::Shrink)),
        line("The main action", "One on a screen, in the highlight.", main_action("document-open-symbolic", "Install").on_press(())),
        line("Not ready", "The same button while it cannot be used.", main_action("document-open-symbolic", "Install")),
        line("Remove", "Pressed once: it asks for a second press.", remove("user-trash-symbolic", "Remove").on_press(()).key("gallery:remove")),
        line("A tag", "A small word that says a state.", card(note("Waiting")).chip().width(Length::Shrink)),
    ]);
    let choices = tile("Choices", vec![
        line("Switch, on", "Anything that is on or off.", switch(true, "On", |_| ())),
        line("Switch, off", "The knob shows on its track.", switch(false, "Off", |_| ())),
        line("A few answers", "Two to four are a strip.", choice(sizes(), 1, "Size", |_| ())),
        line("Many answers", "A dropdown, shut.", dropdown(towns(), Some(2), "Town", |_| ()).key("gallery:shut")),
        line("Slider", "With the highlight up to its knob.", slider(62.0, 0.0, 100.0, 1.0, "Volume", |_| ()).width(Length::Fixed(200.0))),
    ]);
    let typing = tile("Typing", vec![
        line("Name", "A box with words in it.", entry("John Smith", "Name", |_| ()).width(Length::Fixed(220.0))),
        line("Email", "Empty, with its hint.", entry("", "john@example.com", |_| ()).width(Length::Fixed(220.0))),
        line("Password", "Hidden, with its eye.", entry("correct horse", "Password", |_| ()).secret().key("gallery:secret").width(Length::Fixed(220.0))),
        line("Just saved", "The box glows in the highlight.", entry("07700 900123", "Phone", |_| ()).glow(saved).key("gallery:saved").width(Length::Fixed(220.0))),
        column(vec![line("Port", "A value that was refused.", entry("99999", "Port", |_| ()).width(Length::Fixed(220.0))),
                    note("That port is not free. Choose one from 1024 to 65535.").tone(Tone::Problem)]).gap(6.0),
    ]);
    let rows = vec![item("Documents").icon("folder-symbolic").folder().detail("12 items"), item("Pictures").icon("folder-symbolic").folder().detail("340 items"),
                    item("holiday.mp4").icon("video-x-generic-symbolic").detail("1.2 GB"), item("notes.txt").icon("text-x-generic-symbolic").detail("4 KB")];
    let lists = tile("Lists and progress", vec![
        list(rows, Some(1), |_| ()).called("Files").width(Length::Fill),
        row(vec![ring(0.64, 56.0).tone(Tone::Accent), ring(0.3, 56.0), ring(1.0, 56.0), ring(0.9, 56.0).tone(Tone::Problem), spinner().key("gallery:spinner"),
                 column(vec![label("Downloading"), note("64 per cent, a minute left")]).gap(crate::theme::gaps().tiny)]).gap(crate::theme::gaps().large),
        card(column(vec![form_label("A box inside a card"), note("One step on from the card, with quiet words in it.")]).gap(crate::theme::gaps().small)).width(Length::Fill),
    ]);
    vec![buttons, choices, typing, lists]
}

fn tile_of(i: usize) -> Node<()> {
    tiles(0).swap_remove(i)
}

pub(crate) fn gallery(saved: u32) -> Node<()> {
    let [buttons, choices, typing, lists]: [Node<()>; 4] = tiles(saved).try_into().ok().unwrap();
    let top = column(vec![page_title("Every control"), headline("Each part the toolkit draws, as this theme draws it.")]).gap(6.0);
    let sides = even_grid(480.0, vec![column(vec![buttons, lists]).gap(crate::theme::gaps().medium), column(vec![choices, typing]).gap(crate::theme::gaps().medium)]);
    column(vec![top, sides, space()]).gap(crate::theme::gaps().large).padding(crate::theme::gaps().huge)
}

fn middle(ui: &Ui<()>, key: &str) -> (f32, f32) {
    let (_, r, _) = ui.boxes().into_iter().find(|(id, _, _)| id.contains(key)).unwrap_or_else(|| panic!("{key}"));
    (r.x + r.w / 2.0, r.y + r.h / 2.0)
}

fn save(ui: &mut Ui<()>, name: &str) {
    let out = std::path::PathBuf::from(std::env::var("TRINDLE_SHOTS").unwrap_or_else(|_| "target/style-check".into()));
    std::fs::create_dir_all(&out).unwrap();
    let mut pix = tiny_skia::Pixmap::new(SIZE.0, SIZE.1).unwrap();
    ui.paint(&mut pix.as_mut(), 1.0);
    pix.save_png(out.join(format!("{name}.png"))).unwrap();
}

#[test]
#[ignore = "writes pictures to look at; it checks nothing"]
fn shoot_the_gallery() {
    let accent = std::env::var("TRINDLE_SHOT_ACCENT").unwrap_or_else(|_| "#fbc711".into());
    let tail = std::env::var("TRINDLE_SHOT_TAIL").unwrap_or_default();
    let now = Instant::now();
    let mut ui = Ui::new(gallery(0), Theme::new(false, &accent));
    ui.lay_at(SIZE.0, SIZE.1);
    let (x, y) = middle(&ui, "gallery:remove");
    ui.input(&Input::Press { button: Button::Left, x, y }, now);
    ui.input(&Input::Release { button: Button::Left, x, y }, now);
    ui.set_view(gallery(1));
    ui.lay_at(SIZE.0, SIZE.1);
    let (x, y) = middle(&ui, "gallery:pointed");
    ui.input(&Input::PointerMove { x, y }, now);
    ui.tick(now + Duration::from_millis(600));
    save(&mut ui, &format!("controls{tail}"));
    let mut ui = Ui::new(gallery(0), Theme::new(false, &accent));
    ui.lay_at(SIZE.0, SIZE.1);
    let (x, y) = middle(&ui, "gallery:shut");
    ui.input(&Input::Press { button: Button::Left, x, y }, now);
    ui.input(&Input::Release { button: Button::Left, x, y }, now);
    save(&mut ui, &format!("dropdown-open{tail}"));
}

fn painted_gallery() {
    let now = Instant::now();
    let mut pix = tiny_skia::Pixmap::new(SIZE.0, SIZE.1).unwrap();
    let mut ui = Ui::new(gallery(0), Theme::chosen());
    ui.lay_at(SIZE.0, SIZE.1);
    let (x, y) = middle(&ui, "gallery:remove");
    ui.input(&Input::Press { button: Button::Left, x, y }, now);
    ui.input(&Input::Release { button: Button::Left, x, y }, now);
    ui.set_view(gallery(1));
    ui.lay_at(SIZE.0, SIZE.1);
    let (x, y) = middle(&ui, "gallery:pointed");
    ui.input(&Input::PointerMove { x, y }, now);
    ui.tick(now + Duration::from_millis(600));
    ui.paint(&mut pix.as_mut(), 1.0);
    let mut ui = Ui::new(gallery(0), Theme::chosen());
    ui.lay_at(SIZE.0, SIZE.1);
    let (x, y) = middle(&ui, "gallery:shut");
    ui.input(&Input::Press { button: Button::Left, x, y }, now);
    ui.input(&Input::Release { button: Button::Left, x, y }, now);
    ui.paint(&mut pix.as_mut(), 1.0);
    let greyed = column(vec![
        row(vec![button("document-open-symbolic", "Chosen").on(true).on_press(()).off(true), button("list-add-symbolic", "Add").on_press(()).off(true),
                 main_action("document-open-symbolic", "Install").on_press(()).off(true), remove("user-trash-symbolic", "Remove").on_press(()).off(true)]).gap(crate::theme::gaps().medium),
        row(vec![switch(true, "On", |_| ()).off(true), switch(false, "Off", |_| ()).off(true),
                 choice(vec!["Small".to_string(), "Medium".to_string()], 1, "Size", |_| ()).off(true),
                 slider(62.0, 0.0, 100.0, 1.0, "Volume", |_| ()).width(Length::Fixed(200.0)).off(true)]).gap(crate::theme::gaps().medium),
        entry("John Smith", "Name", |_| ()).width(Length::Fixed(220.0)).off(true),
        card(column(vec![label("Words that cannot be used"), note("And quiet ones."), row(vec![button("document-open-symbolic", "Chosen").on(true).on_press(())]).off(true)])).off(true),
    ]).gap(crate::theme::gaps().roomy).padding(crate::theme::gaps().huge);
    Ui::new(greyed, Theme::chosen()).paint(&mut pix.as_mut(), 1.0);
}

#[test]
fn the_gallery_draws_only_pairs_the_table_holds() {
    use super::theme::drawn;
    drawn::none(drawn::in_every_look("every control", 80, painted_gallery));
}

pub(crate) mod golden;
mod states;
mod scales;
mod themes;
mod names;
mod every;

#[cfg(test)]
mod edge_ring {
    use super::*;
    use crate::theme::{luminance, tests::every_look};
    use crate::widget::scroll;

    fn pale(pix: &tiny_skia::Pixmap, bg: crate::colour::Colour, what: &str, bottom: bool) -> Vec<String> {
        let (w, h) = (pix.width(), pix.height());
        let reference = luminance(bg) + 0.004;
        let mut out = Vec::new();
        for y in 0..h {
            for x in 0..w {
                if x >= 2 && y >= 2 && x + 2 < w && (y + 2 < h || !bottom) {
                    continue;
                }
                let p = pix.pixel(x, y).unwrap().demultiply();
                let c = crate::colour::Colour { r: p.red() as f32 / 255.0, g: p.green() as f32 / 255.0, b: p.blue() as f32 / 255.0, a: 1.0 };
                if p.alpha() < 255 || luminance(c) > reference {
                    out.push(format!("{what}: pixel {x},{y} is {} (alpha {}) on a page of {}", c.css(), p.alpha(), bg.css()));
                    if out.len() > 3 {
                        return out;
                    }
                }
            }
        }
        out
    }

    fn fits(ui: &Ui<()>, wide: u32, scale: f32) -> u32 {
        ((ui.natural_height(wide as f32 / scale) + 48.0) * scale).ceil() as u32
    }

    fn long_page() -> Node<()> {
        scroll("page", column((0..60).map(|i| label(format!("Row {i}"))).collect()).gap(6.0).padding(24.0))
    }

    fn views() -> Vec<(&'static str, fn() -> Node<()>, bool)> {
        vec![("the gallery", || gallery(0), false), ("a long scrolled page", long_page, true), ("a scrolled gallery", || scroll("gallery", gallery(0)), true)]
    }

    #[test]
    fn the_bells_flash_is_never_whiter_than_the_highlight_and_leaves_nothing() {
        let mut faults = Vec::new();
        for (look, theme) in every_look() {
            for scale in [1.0, 1.25, 1.5, 2.0] {
                let size = ((700.0 * scale) as u32, (200.0 * scale) as u32);
                let mut ui = Ui::new(gallery(0), theme.clone());
                let high = fits(&ui, size.0, scale);
                let mut pix = tiny_skia::Pixmap::new(size.0, high).unwrap();
                ui.ring_bell(Instant::now() - crate::theme::sheet().bell / 5);
                ui.paint(&mut pix.as_mut(), scale);
                let limit = luminance(theme.accent).max(luminance(theme.bg)) + 0.004;
                for (i, p) in pix.pixels().iter().enumerate() {
                    let (x, y) = (i as u32 % size.0, i as u32 / size.0);
                    if x >= 2 && y >= 2 && x + 2 < size.0 && y + 2 < high {
                        continue;
                    }
                    let p = p.demultiply();
                    let c = crate::colour::Colour { r: p.red() as f32 / 255.0, g: p.green() as f32 / 255.0, b: p.blue() as f32 / 255.0, a: 1.0 };
                    if luminance(c) > limit {
                        faults.push(format!("{look}, scale {scale}: flash pixel {x},{y} is {}, whiter than the highlight {}", c.css(), theme.accent.css()));
                        break;
                    }
                }
                ui.tick(Instant::now() + crate::theme::sheet().bell * 2);
                ui.paint(&mut pix.as_mut(), scale);
                faults.extend(pale(&pix, theme.bg, &format!("{look}, scale {scale}, after the flash"), true));
            }
        }
        assert!(faults.is_empty(), "{}", faults.iter().take(12).cloned().collect::<Vec<_>>().join("\n"));
    }

    #[test]
    fn nothing_paler_than_the_page_lies_on_a_windows_edge() {
        let now = Instant::now();
        let mut faults = Vec::new();
        for (look, theme) in every_look() {
            for (name, view, scrolls) in views() {
                for (scale, size) in [(1.0, (640u32, 480u32)), (1.25, (800, 600)), (1.5, (961, 721)), (1.5, (960, 720)), (2.0, (1280, 960)), (1.25, (801, 603))] {
                    for pointed in [false, true] {
                        let mut ui = Ui::new(view(), theme.clone());
                        let high = if scrolls { size.1 } else { fits(&ui, size.0, scale) };
                        let mut pix = tiny_skia::Pixmap::new(size.0, high).unwrap();
                        ui.paint(&mut pix.as_mut(), scale);
                        if pointed {
                            ui.input(&Input::PointerMove { x: size.0 as f32 - 4.0, y: 40.0 }, now);
                            ui.paint(&mut pix.as_mut(), scale);
                        }
                        faults.extend(pale(&pix, theme.bg, &format!("{look}, {name}, scale {scale}, {}x{}, pointer on the bar {pointed}", size.0, size.1), !scrolls));
                    }
                }
            }
        }
        assert!(faults.is_empty(), "{} pale pixels on a window's edge:\n{}", faults.len(), faults.iter().take(12).cloned().collect::<Vec<_>>().join("\n"));
    }
}

fn median(mut v: Vec<f64>) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[v.len() / 2]
}

fn rss_kb() -> u64 {
    let s = std::fs::read_to_string("/proc/self/status").unwrap_or_default();
    s.lines().find(|l| l.starts_with("VmRSS")).and_then(|l| l.split_whitespace().nth(1)).and_then(|n| n.parse().ok()).unwrap_or(0)
}

#[test]
#[ignore = "measures time and memory; it checks nothing"]
fn measure_the_gallery() {
    let before = rss_kb();
    let start = Instant::now();
    let mut ui = Ui::new(gallery(0), Theme::chosen());
    ui.lay_at(SIZE.0, SIZE.1);
    let mut pix = tiny_skia::Pixmap::new(SIZE.0, SIZE.1).unwrap();
    ui.paint(&mut pix.as_mut(), 1.0);
    println!("first page (build, lay out, paint, fonts cold): {:.1} ms", start.elapsed().as_secs_f64() * 1000.0);
    let (mut full, mut hover, mut rebuild, mut patch) = (vec![], vec![], vec![], vec![]);
    for i in 0..200 {
        let t = Instant::now();
        ui.paint(&mut pix.as_mut(), 1.0);
        full.push(t.elapsed().as_secs_f64() * 1000.0);
        let (x, y) = middle(&ui, if i % 2 == 0 { "gallery:pointed" } else { "gallery:remove" });
        let t = Instant::now();
        ui.input(&Input::PointerMove { x, y }, Instant::now());
        ui.paint(&mut pix.as_mut(), 1.0);
        hover.push(t.elapsed().as_secs_f64() * 1000.0);
        let (x, y) = middle(&ui, if i % 2 == 0 { "gallery:remove" } else { "gallery:pointed" });
        let t = Instant::now();
        let reply = ui.input(&Input::PointerMove { x, y }, Instant::now());
        assert!(reply.patch);
        ui.paint_live(&mut pix.as_mut(), 1.0);
        patch.push(t.elapsed().as_secs_f64() * 1000.0);
        let t = Instant::now();
        ui.set_view(gallery(i % 3));
        ui.lay_at(SIZE.0, SIZE.1);
        ui.paint(&mut pix.as_mut(), 1.0);
        rebuild.push(t.elapsed().as_secs_f64() * 1000.0);
    }
    let mut bare: Ui<()> = Ui::new(column(vec![label("x")]), Theme::chosen());
    bare.lay_at(SIZE.0, SIZE.1);
    let floor: Vec<f64> = (0..200).map(|_| { let t = Instant::now(); bare.paint(&mut pix.as_mut(), 1.0); t.elapsed().as_secs_f64() * 1000.0 }).collect();
    println!("paint, one label only (the floor): median {:.2} ms", median(floor));
    for (name, node) in [("buttons", 0), ("choices", 1), ("typing", 2), ("lists", 3)].map(|(n, i)| (n, tile_of(i))) {
        let mut one: Ui<()> = Ui::new(node, Theme::chosen());
        one.lay_at(SIZE.0, SIZE.1);
        let v: Vec<f64> = (0..100).map(|_| { let t = Instant::now(); one.paint(&mut pix.as_mut(), 1.0); t.elapsed().as_secs_f64() * 1000.0 }).collect();
        println!("  just {name}: median {:.2} ms", median(v));
    }
    let faded = Ui::new(column(vec![gallery(0).opacity(0.5)]), Theme::chosen());
    let mut faded: Ui<()> = faded;
    faded.lay_at(SIZE.0, SIZE.1);
    let v: Vec<f64> = (0..100).map(|_| { let t = Instant::now(); faded.paint(&mut pix.as_mut(), 1.0); t.elapsed().as_secs_f64() * 1000.0 }).collect();
    println!("whole page at half opacity (a layer, then laid over): median {:.2} ms", median(v));
    let copies: Vec<f64> = (0..100).map(|_| { let t = Instant::now(); let mut b = pix.data().to_vec(); b.chunks_exact_mut(4).for_each(|p| p.swap(0, 2)); std::hint::black_box(&b); t.elapsed().as_secs_f64() * 1000.0 }).collect();
    println!("copy and swap the 4 MB frame to the compositor's order: median {:.2} ms", median(copies));
    println!("paint, nothing changed: median {:.2} ms", median(full));
    println!("pointer move to another widget, painted whole: median {:.2} ms", median(hover));
    println!("pointer move to another widget, only the two painted: median {:.2} ms", median(patch));
    println!("new view, lay out and paint: median {:.2} ms", median(rebuild));
    println!("memory taken by the page: {} kB (process {} kB)", rss_kb().saturating_sub(before), rss_kb());
}

#[test]
fn the_gallery_paints_within_its_budget() {
    let cpu = crate::sys::os::thread_time;
    let start = cpu();
    let mut ui = Ui::new(gallery(0), Theme::chosen());
    ui.lay_at(SIZE.0, SIZE.1);
    let mut pix = tiny_skia::Pixmap::new(SIZE.0, SIZE.1).unwrap();
    ui.paint(&mut pix.as_mut(), 1.0);
    assert!(cpu() - start < Duration::from_secs(3), "the first page took {:?}", cpu() - start);
    let before = rss_kb();
    let mut times: Vec<Duration> = (0..30).map(|i| {
        let t = cpu();
        ui.set_view(gallery(i % 3));
        ui.lay_at(SIZE.0, SIZE.1);
        ui.paint(&mut pix.as_mut(), 1.0);
        cpu() - t
    }).collect();
    times.sort();
    assert!(times[15] < Duration::from_millis(150), "a new view, laid out and painted, took {:?} at the median", times[15]);
    assert!(rss_kb().saturating_sub(before) < 200_000, "30 repaints grew the process by {} kB", rss_kb().saturating_sub(before));
}

fn on_part_edge(parts: &[super::widget::Rect], i: usize) -> bool {
    let (x, y) = ((i % SIZE.0 as usize) as f32, (i / SIZE.0 as usize) as f32);
    parts.iter().any(|p| (y == p.y || y == p.y + p.h - 1.0 || x == p.x || x == p.x + p.w - 1.0) && x >= p.x && x < p.x + p.w && y >= p.y && y < p.y + p.h)
}

#[test]
fn pointing_at_one_widget_after_another_paints_only_them_and_changes_nothing_else() {
    fn whole(ui: &mut Ui<()>) -> Vec<u8> {
        let mut pix = tiny_skia::Pixmap::new(SIZE.0, SIZE.1).unwrap();
        ui.paint(&mut pix.as_mut(), 1.0);
        let mut bytes = pix.data().to_vec();
        bytes.chunks_exact_mut(4).for_each(|p| p.swap(0, 2));
        bytes
    }
    let now = Instant::now();
    let mut ui: Ui<()> = Ui::new(gallery(0), Theme::new(false, "#fbc711"));
    ui.lay_at(SIZE.0, SIZE.1);
    let mut shown = whole(&mut ui);
    let mut steps = 0;
    let mut every_part: Vec<super::widget::Rect> = Vec::new();
    for key in ["gallery:pointed", "gallery:remove", "gallery:shut", "gallery:saved", "gallery:pointed"] {
        let (x, y) = middle(&ui, key);
        let reply = ui.input(&Input::PointerMove { x, y }, now);
        assert!(reply.patch && !reply.redraw, "pointing at {key} should patch");
        let mut canvas = tiny_skia::Pixmap::new(SIZE.0, SIZE.1).unwrap();
        canvas.data_mut().copy_from_slice(&shown);
        let parts = ui.paint_live(&mut canvas.as_mut(), 1.0).expect("a part is painted");
        assert!(!parts.is_empty());
        every_part.extend(parts.iter().cloned());
        shown = canvas.data().to_vec();
        let mut again: Ui<()> = Ui::new(gallery(0), Theme::new(false, "#fbc711"));
        again.lay_at(SIZE.0, SIZE.1);
        again.input(&Input::PointerMove { x, y }, now);
        let expected = whole(&mut again);
        let wrong = shown.chunks_exact(4).zip(expected.chunks_exact(4)).enumerate()
            .filter(|(i, (a, b))| !(330..366).contains(&(i % SIZE.0 as usize)) && !on_part_edge(&every_part, *i) && a.iter().zip(*b).any(|(p, q)| p.abs_diff(*q) > 6)).count();
        assert_eq!(wrong, 0, "{key}: {wrong} pixels differ from a whole painting by more than a shade");
        steps += 1;
    }
    assert_eq!(steps, 5);
    let (x, y) = middle(&ui, "gallery:pointed");
    let reply = ui.input(&Input::Press { button: Button::Left, x, y }, now);
    assert!(reply.patch && !reply.redraw);
    let mut canvas = tiny_skia::Pixmap::new(SIZE.0, SIZE.1).unwrap();
    canvas.data_mut().copy_from_slice(&shown);
    every_part.extend(ui.paint_live(&mut canvas.as_mut(), 1.0).expect("a part is painted"));
    let mut again: Ui<()> = Ui::new(gallery(0), Theme::new(false, "#fbc711"));
    again.lay_at(SIZE.0, SIZE.1);
    again.input(&Input::PointerMove { x, y }, now);
    again.input(&Input::Press { button: Button::Left, x, y }, now);
    let expected = whole(&mut again);
    let wrong = canvas.data().chunks_exact(4).zip(expected.chunks_exact(4)).enumerate()
        .filter(|(i, (a, b))| !(330..366).contains(&(i % SIZE.0 as usize)) && !on_part_edge(&every_part, *i) && a.iter().zip(*b).any(|(p, q)| p.abs_diff(*q) > 6)).count();
    assert_eq!(wrong, 0, "{wrong} pixels differ after a press");
}

#[test]
#[ignore = "measures time; it checks nothing"]
fn measure_the_first_page() {
    let t = Instant::now();
    let fonts = crate::text::Fonts::new();
    println!("fonts found: {:.1} ms ({} families)", t.elapsed().as_secs_f64() * 1000.0, fonts.families().len());
    let t = Instant::now();
    let theme = Theme::chosen();
    println!("theme and style sheet: {:.1} ms", t.elapsed().as_secs_f64() * 1000.0);
    let t = Instant::now();
    let mut ui: Ui<()> = Ui::new(gallery(0), theme);
    println!("Ui::new: {:.1} ms", t.elapsed().as_secs_f64() * 1000.0);
    let t = Instant::now();
    ui.lay_at(SIZE.0, SIZE.1);
    println!("first lay out (fonts loaded on use): {:.1} ms", t.elapsed().as_secs_f64() * 1000.0);
    let mut pix = tiny_skia::Pixmap::new(SIZE.0, SIZE.1).unwrap();
    let t = Instant::now();
    ui.paint(&mut pix.as_mut(), 1.0);
    println!("first paint (glyph shapes made): {:.1} ms", t.elapsed().as_secs_f64() * 1000.0);
    let t = Instant::now();
    ui.paint(&mut pix.as_mut(), 1.0);
    println!("second paint: {:.1} ms", t.elapsed().as_secs_f64() * 1000.0);
}
