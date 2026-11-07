//! A part can say what the whole of it is, and cut words show whole on pointing.

use super::*;
use super::super::theme::Theme;
use super::super::widget::{self as ww, Length};

const LONG: &str = "A line of words that is a good deal longer than the room its row has for it";
const UNDER: &str = "And a second line under it that is no shorter than the first one was";
const LINES: &str = "First line\nSecond line\n  Third, set in";

fn line(text: &str) -> Node<()> {
    let text = text.to_string();
    ww::drawing((1.0, 24.0), text.clone(), move |canvas, pen| {
        let style = Style::new(14.0 * pen.scale);
        let dots = pen.fonts.layout("\u{2026}", &style, None);
        let laid = pen.fonts.layout(&text, &style, None).cut(canvas.width() as f32, &dots);
        pen.fonts.draw(canvas, &laid, 0.0, 2.0, pen.theme.fg);
    }).width(Length::Fill)
}

fn two_lines() -> Node<()> {
    ww::drawing((1.0, 48.0), format!("{LONG}\n{UNDER}"), move |canvas, pen| {
        let style = Style::new(14.0 * pen.scale);
        let dots = pen.fonts.layout("\u{2026}", &style, None);
        for (i, text) in [LONG, UNDER].into_iter().enumerate() {
            let laid = pen.fonts.layout(text, &style, None).cut(canvas.width() as f32, &dots);
            pen.fonts.draw(canvas, &laid, 0.0, 2.0 + 24.0 * i as f32, pen.theme.fg);
        }
    }).width(Length::Fill)
}

fn squeezed(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn page() -> Node<()> {
    let narrow = Length::Fixed(160.0);
    ww::column(vec![
        ww::card(line(&squeezed(LINES))).flat().on_press(()).whole(LINES).key("lines").width(Length::Fixed(300.0)),
        ww::card(line("Short")).flat().on_press(()).whole("Short").key("same").width(Length::Fixed(300.0)),
        ww::card(line(LONG)).flat().on_press(()).whole(LONG).key("long").width(narrow),
        line(LONG).key("bare").width(narrow),
        line("Short").key("fits").width(narrow),
        ww::card(line(LONG)).flat().on_press(()).key("carded").width(narrow),
        two_lines().key("two").width(narrow),
        line(LONG).own_labels().key("own").width(narrow),
        ww::button("document-open-symbolic", "Open").on_press(()).key("round"),
    ]).padding(10.0).gap(10.0)
}

fn painted() -> Ui<()> {
    let mut ui = Ui::new(page(), Theme::new(false, "#fbc711"));
    let mut pix = tiny_skia::Pixmap::new(600, 700).unwrap();
    ui.paint(&mut pix.as_mut(), 1.0);
    ui
}

fn pointed(ui: &mut Ui<()>, key: &str, share: (f32, f32)) -> (Option<String>, Option<String>) {
    let r = ui.spot(key).unwrap_or_else(|| panic!("{key}")).rect;
    let (x, y) = (r.x + r.w * share.0, r.y + r.h * share.1);
    ui.pointer = Some((x, y));
    let hit = ui.hit(x, y);
    let label = hit.as_ref().and_then(|id| ui.label_of(id));
    (hit, label)
}

#[test]
fn a_part_that_says_its_whole_shows_it_on_pointing_when_it_shows_less() {
    let mut ui = painted();
    assert_eq!(pointed(&mut ui, "lines", (0.5, 0.5)), (Some("lines".into()), Some(LINES.into())));
    assert_eq!(pointed(&mut ui, "lines", (0.99, 0.04)).1.as_deref(), Some(LINES));
    assert_eq!(pointed(&mut ui, "same", (0.5, 0.5)), (Some("same".into()), None));
    assert_eq!(pointed(&mut ui, "long", (0.5, 0.5)).1.as_deref(), Some(LONG));
    assert_eq!(pointed(&mut ui, "long", (0.99, 0.04)).1.as_deref(), Some(LONG));
}

#[test]
fn words_a_drawing_cuts_by_hand_show_whole_on_pointing_and_only_then() {
    let mut ui = painted();
    assert_eq!(pointed(&mut ui, "bare", (0.5, 0.5)), (Some("bare".into()), Some(LONG.into())));
    assert_eq!(pointed(&mut ui, "fits", (0.5, 0.5)), (None, None));
    assert_eq!(pointed(&mut ui, "carded", (0.5, 0.5)), (Some("carded".into()), Some(LONG.into())));
    assert_eq!(pointed(&mut ui, "two", (0.5, 0.25)).1.as_deref(), Some(LONG));
    assert_eq!(pointed(&mut ui, "two", (0.5, 0.75)).1.as_deref(), Some(UNDER));
    assert_eq!(pointed(&mut ui, "own", (0.5, 0.5)), (None, None));
    let (all, unsaid) = ui.cut_unsaid();
    assert!(all >= 5, "only {all} pieces were cut");
    assert!(unsaid.is_empty(), "{unsaid:?}");
}

#[test]
fn a_drawings_label_is_for_the_line_under_the_pointer() {
    let mut ui = painted();
    let r = ui.spot("two").unwrap().rect;
    let now = Instant::now();
    let mut reply = Reply::new();
    ui.pointer_to(r.x + 20.0, r.y + r.h * 0.25, now, &mut reply);
    ui.tick(now + tip_rest() + Duration::from_millis(10));
    assert!(matches!(ui.tip.as_ref().map(|t| t.state), Some(TipState::Shown(_))));
    ui.pointer_to(r.x + 20.0, r.y + r.h * 0.75, now + Duration::from_secs(1), &mut reply);
    assert!(matches!(ui.tip.as_ref().map(|t| t.state), Some(TipState::Waiting(_))), "a new line starts the label again");
}

#[test]
fn a_label_that_tells_the_whole_stays_while_the_pointer_rests() {
    for (key, stays) in [("lines", true), ("long", true), ("bare", true), ("round", false)] {
        let mut ui = painted();
        ui.set_labels(Labels::Brief);
        let mut pix = tiny_skia::Pixmap::new(600, 700).unwrap();
        ui.paint(&mut pix.as_mut(), 1.0);
        let r = ui.spot(key).unwrap().rect;
        let now = Instant::now();
        let mut reply = Reply::new();
        ui.pointer_to(r.x + r.w / 2.0, r.y + r.h / 2.0, now, &mut reply);
        ui.tick(now + tip_rest());
        assert!(matches!(ui.tip.as_ref().map(|t| t.state), Some(TipState::Shown(_))), "{key} shows");
        let later = now + tip_rest() + tip_stays() + Duration::from_secs(60);
        ui.tick(later);
        assert_eq!(matches!(ui.tip.as_ref().map(|t| t.state), Some(TipState::Shown(_))), stays, "{key}");
        if stays {
            assert_eq!(ui.next_wake(now + tip_rest() + Duration::from_millis(1)), None, "{key} has nothing to wake for");
            ui.pointer_to(590.0, 690.0, later, &mut reply);
            assert!(ui.tip.is_none(), "{key} goes when the pointer leaves");
        }
    }
}

fn last_line(l: &Layout) -> String {
    let line = l.lines.last().unwrap();
    l.whole()[line.range.clone()].trim().to_string()
}

#[test]
fn a_long_label_is_kept_to_its_most_width_and_lines_and_says_how_much_more() {
    let fonts = Fonts::new();
    let k = theme::sheet();
    let most = k.label_lines_most;
    for scale in [1.0, 2.0] {
        let wide = |l: &Layout| l.widest();
        let l = label_layout(&fonts, &format!("{LONG} {UNDER} {LONG}"), scale, 3000.0);
        assert!(l.lines.len() > 1 && wide(&l) <= k.label_width_most * scale + 0.5, "{} lines, {} wide at {scale}", l.lines.len(), wide(&l));
        let l = label_layout(&fonts, "Open", scale, 3000.0);
        assert_eq!((l.lines.len(), &**l.whole()), (1, "Open"));
        let fits: String = (1..=most).map(|i| format!("Line {i}")).collect::<Vec<_>>().join("\n");
        let l = label_layout(&fonts, &fits, scale, 3000.0);
        assert_eq!((l.lines.len(), last_line(&l)), (most, format!("Line {most}")));
        let many: String = (1..=400).map(|i| format!("Line {i}")).collect::<Vec<_>>().join("\n");
        let l = label_layout(&fonts, &many, scale, 3000.0);
        assert_eq!(l.lines.len(), most);
        assert_eq!(last_line(&l), format!("+{} lines", 400 - (most - 1)));
        assert!(l.whole().contains(&format!("Line {}\n", most - 1)) && !l.whole().contains(&format!("Line {most}\n")));
        let one_more: String = (1..=most + 1).map(|i| format!("Line {i}")).collect::<Vec<_>>().join("\n");
        let l = label_layout(&fonts, &one_more, scale, 3000.0);
        assert_eq!((l.lines.len(), last_line(&l)), (most, "+2 lines".to_string()));
        let l = label_layout(&fonts, "\n\nOne\nTwo\n\n\n", scale, 3000.0);
        assert_eq!((l.lines.len(), last_line(&l)), (2, "Two".to_string()));
    }
    let huge = "word ".repeat(250_000);
    let l = label_layout(&fonts, &huge, 1.0, 3000.0);
    assert_eq!((l.lines.len(), last_line(&l)), (most, "+ more".to_string()));
    assert!(l.whole().len() < 20_000, "{} bytes were laid out", l.whole().len());
    let l = label_layout(&fonts, LONG, 1.0, 200.0);
    assert!(l.lines.iter().all(|line| line.width <= 200.0));
}

#[test]
fn a_label_of_any_height_has_the_one_corner() {
    let fonts = Fonts::new();
    for scale in [1.0, 1.75] {
        let corner = theme::sheet().corner * scale;
        for text in ["Open", "Open\nthe folder", LINES, &(1..=30).map(|i| format!("Line {i}")).collect::<Vec<_>>().join("\n")] {
            let laid = label_layout(&fonts, text, scale, 3000.0);
            assert_eq!(label_corner(&laid, scale), corner, "{} lines at {scale}", laid.lines.len());
        }
    }
}

#[test]
fn a_label_with_no_room_under_or_over() {
    let mut ui = painted();
    let shown = |ui: &mut Ui<()>, key: &str, tall: u32| {
        let r = ui.spot(key).unwrap().rect;
        ui.pointer = Some((r.x + r.w / 2.0, r.y + r.h / 2.0));
        ui.tip = Some(Tip { id: key.into(), state: TipState::Shown(Instant::now()) });
        let mut pix = tiny_skia::Pixmap::new(600, tall).unwrap();
        ui.tip_drawn(&mut pix.as_mut())
    };
    for key in ["lines", "bare"] {
        let r = ui.spot(key).unwrap().rect;
        let drawn = shown(&mut ui, key, 700).expect("the label is drawn");
        assert!(drawn.y > r.y + r.h / 2.0 && drawn.y + drawn.h <= 700.0, "{key}: {drawn:?} for {r:?}");
    }
    let r = ui.spot("lines").unwrap().rect;
    let tall = (r.y + r.h + 30.0) as u32;
    let drawn = shown(&mut ui, "lines", tall).expect("the whole is still shown");
    assert!(drawn.y >= 0.0 && drawn.y + drawn.h <= tall as f32 && drawn.x >= 0.0 && drawn.x + drawn.w <= 600.0, "{drawn:?} on a canvas {tall} tall");
    assert_eq!(shown(&mut ui, "bare", 20), None);
}

#[test]
fn a_window_that_sends_its_labels_out_draws_none_and_says_where_each_goes() {
    let mut ui = painted();
    ui.out_labels = true;
    let r = ui.spot("lines").unwrap().rect;
    ui.pointer = Some((r.x + r.w / 2.0, r.y + r.h / 2.0));
    ui.tip = Some(Tip { id: "lines".into(), state: TipState::Shown(Instant::now()) });
    let mut pix = tiny_skia::Pixmap::new(600, 700).unwrap();
    ui.paint(&mut pix.as_mut(), 1.0);
    let out = ui.sent_out.borrow().clone().expect("the label is sent out");
    assert_eq!(out.words, LINES);
    assert!(out.at.w > 0.0 && out.at.h > 0.0 && out.size.0 > 0.0 && out.size.1 > 0.0, "{out:?}");
    ui.tip = None;
    let mut bare = tiny_skia::Pixmap::new(600, 700).unwrap();
    ui.paint(&mut bare.as_mut(), 1.0);
    assert_eq!(pix.data(), bare.data(), "a label was drawn in the window");
    assert_eq!(*ui.sent_out.borrow(), None, "a label stays after the pointer has gone");
    ui.out_labels = false;
    ui.tip = Some(Tip { id: "lines".into(), state: TipState::Shown(Instant::now()) });
    ui.paint(&mut pix.as_mut(), 1.0);
    assert_eq!(*ui.sent_out.borrow(), None);
    assert_ne!(pix.data(), bare.data());
}

#[test]
fn the_spoken_name_is_the_rows_own_words() {
    let mut ui = painted();
    let names: Vec<String> = ui.access("Test").nodes.iter().filter_map(|(_, part)| part.label().map(str::to_string)).collect();
    assert!(names.iter().any(|n| n.contains(&squeezed(LINES))), "{names:?}");
    assert!(!names.iter().any(|n| n == LINES), "{names:?}");
}
