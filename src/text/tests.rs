//! Tests of text.

use super::*;
use tiny_skia::Pixmap;

fn fonts() -> &'static Fonts {
    thread_local! {
        static FONTS: &'static Fonts = Box::leak(Box::new(Fonts::new()));
    }
    FONTS.with(|f| *f)
}

fn family_of(f: &Fonts, c: char) -> Option<String> {
    let id = f.stand_in(c, &Style::new(14.0))?;
    f.db.borrow().face(id).and_then(|face| face.families.first().map(|(n, _)| n.clone()))
}

#[test]
fn a_font_in_the_home_folder_is_read_into_memory_not_mapped() {
    let system = Fonts::new();
    let Some(path) = system.db.borrow().faces().find_map(|f| match &f.source {
        fontdb::Source::File(p) if p.extension().is_some_and(|e| e == "ttf") => Some(p.clone()),
        _ => None,
    }) else {
        eprintln!("SKIPPED: no .ttf font on this computer to copy");
        return;
    };
    let home = std::env::temp_dir().join(format!("trindle-home-fonts-{}", std::process::id()));
    std::fs::create_dir_all(&home).unwrap();
    std::fs::copy(&path, home.join("own.ttf")).unwrap();
    let mut db = Database::new();
    db.load_fonts_dir(&home);
    let before = db.len();
    assert!(before > 0 && db.faces().all(|f| matches!(f.source, fontdb::Source::File(_))));
    read_fonts_under(&mut db, &home);
    assert_eq!(db.len(), before);
    assert!(db.faces().all(|f| matches!(f.source, fontdb::Source::Binary(_))));
    let _ = std::fs::remove_dir_all(&home);
}

#[test]
fn a_language_without_a_script_font_wants_none_and_letters_fall_back_as_before() {
    let tags = |s: &str| vec![s.to_string()];
    for code in ["en", "zz-made-up", "fr"] {
        assert!(Fonts::for_language(&tags(code)).wanted.borrow().is_empty(), "{code}");
    }
    let (plain, made_up) = (Fonts::for_language(&[]), Fonts::for_language(&tags("zz")));
    for c in ['a', 'é', '中', 'न', 'ب'] {
        assert_eq!(family_of(&plain, c), family_of(&made_up, c), "{c}");
    }
    let l = made_up.layout("hello", &Style::new(14.0), None);
    assert_eq!(l.lines.len(), 1);
}

#[test]
#[ignore = "needs Noto Sans Devanagari installed"]
fn the_chosen_languages_installed_script_font_is_tried_first() {
    let hindi = Fonts::for_language(&["hi".to_string()]);
    assert_eq!(hindi.wanted.borrow().first().map(String::as_str), Some("Noto Sans Devanagari"));
    assert_eq!(family_of(&hindi, 'न').as_deref(), Some("Noto Sans Devanagari"));
}

#[test]
fn the_script_font_order_follows_the_scripts_list_whatever_is_installed() {
    use crate::lang::script::{installed, script_of};
    let hindi = script_of(&["hi".to_string()]).expect("hi has a script");
    let have = ["Some Other", "noto serif devanagari", "Noto Sans Devanagari"];
    assert_eq!(installed(hindi, &have), ["Noto Sans Devanagari", "Noto Serif Devanagari"]);
    assert!(installed(hindi, &["Noto Sans Arabic"]).is_empty());
}

#[test]
fn marks_are_tidied_to_whole_letters_and_joined() {
    let t = "aé中b";
    assert_eq!(clean_marks(t, &[2..3]), vec![1..3]);
    assert_eq!(clean_marks(t, &[0..1, 1..3, 2..99, 7..9]), vec![0..t.len()]);
    assert_eq!(clean_marks(t, &[5..5, 40..50]), Vec::<Range<usize>>::new());
    assert_eq!(clean_marks(t, &[4..6, 0..1]), vec![0..1, 3..6]);
}

#[test]
fn marked_letters_have_bands_where_they_are_shown() {
    let l = fonts().layout("hello world", &Style::new(14.0), None);
    let all = l.mark_bands(&[0..11]);
    assert_eq!(all.len(), 1);
    let two = l.mark_bands(&[0..2, 6..8]);
    assert_eq!(two.len(), 2);
    assert!(two[0].0 < two[1].0 && two[0].2 < all[0].2);
    assert!(l.mark_bands(&[]).is_empty());
    let a = fonts().layout("سلام", &Style::new(14.0), None);
    let half = a.mark_bands(&[0..2]);
    let whole = a.mark_bands(&[0..8]);
    assert_eq!(half.len(), 1);
    assert!(half[0].2 < whole[0].2 + 0.01);
    assert!(half[0].0 + half[0].2 >= whole[0].0 + whole[0].2 - 0.6);
}

#[test]
fn spaced_letters_move_apart_by_the_gap() {
    let plain = fonts().layout("SOUND", &Style::new(12.0), None);
    let wide = plain.spaced(2.0);
    assert!((wide.width - plain.width - 8.0).abs() < 0.01);
    let (a, b) = (&plain.lines[0].glyphs, &wide.lines[0].glyphs);
    assert_eq!(a[0].x, b[0].x);
    assert!((b[4].x - a[4].x - 8.0).abs() < 0.01);
}

#[test]
fn words_on_an_arc_sit_on_the_circle() {
    let layout = fonts().layout("Settings", &Style::new(14.0), None);
    let mut px = Pixmap::new(200, 200).unwrap();
    fonts().draw_on_arc(&mut px.as_mut(), &layout, (100.0, 100.0), 80.0, -std::f32::consts::FRAC_PI_2,
                        Colour::rgb(1.0, 1.0, 1.0));
    let ink = |x0: u32, y0: u32, x1: u32, y1: u32| (y0..y1).flat_map(|y| (x0..x1).map(move |x| (x, y)))
        .any(|(x, y)| px.data()[((y * 200 + x) * 4 + 3) as usize] > 0);
    assert!(ink(60, 5, 140, 35), "the words are at the top of the circle");
    assert!(!ink(0, 60, 200, 200), "and nowhere else");
}

#[test]
fn a_right_to_left_line_loses_its_left_end_and_takes_the_dots_there() {
    let style = Style::new(12.0);
    let mut layout = fonts().layout("abcdefgh", &style, None);
    for g in &mut layout.lines[0].glyphs {
        g.rtl = true;
    }
    let dots = fonts().layout("...", &style, None);
    let room = layout.width / 2.0;
    let out = layout.cut(room, &dots);
    assert_eq!(out.lines[0].x, 0.0, "the line begins where it did");
    assert!((out.width - out.lines[0].width).abs() < 0.01 && out.width <= room + 0.5, "and ends inside the room: {}", out.width);
    assert!(out.lines[0].glyphs.iter().all(|g| g.x >= 0.0 && g.x < room), "every letter is inside the room");
    assert!(out.lines[0].glyphs.len() < layout.lines[0].glyphs.len() + 3, "and the left end is gone");
    assert!(out.lines[0].width <= room + 0.5, "{} of {room}", out.lines[0].width);
}

#[test]
fn words_are_cut_after_a_number_of_lines_with_dots_on_the_last() {
    let style = Style::new(16.0);
    let dots = fonts().layout("\u{2026}", &style, None);
    let long = "Use one computer's mouse and keyboard on another computer beside it, over your own network, all day long";
    let room = fonts().layout("Use one computer's mouse and", &style, None).width;
    let wrapped = fonts().layout(long, &style, Some(room));
    assert!(wrapped.lines.len() > 2, "{} lines", wrapped.lines.len());
    let cut = wrapped.cut_after(2, room, &dots);
    assert_eq!(cut.lines.len(), 2);
    assert_eq!(cut.lines[0].glyphs.len(), wrapped.lines[0].glyphs.len(), "the first line is whole");
    assert!(cut.lines[1].width <= room + 0.01, "the second ends inside the room: {} in {room}", cut.lines[1].width);
    let last = cut.lines[1].glyphs.last().unwrap();
    assert_eq!(last.glyph, dots.lines[0].glyphs[0].glyph, "and ends with the dots");
    assert!((last.y - cut.lines[1].baseline).abs() < 0.01, "which sit on the second line");
    assert!((cut.height - (wrapped.lines[1].top + wrapped.lines[1].height)).abs() < 0.01);
    let two = fonts().layout("Use one computer's mouse and keyboard on another", &style, Some(room));
    assert_eq!(two.lines.len(), 2);
    assert_eq!(two.cut_after(2, room, &dots).lines[1].glyphs.len(), two.lines[1].glyphs.len());
    assert_eq!(wrapped.cut_after(0, room, &dots).lines.len(), wrapped.lines.len());
}

#[test]
fn lines_break_between_words_to_fit_and_at_new_lines() {
    let style = Style::new(16.0);
    let one = fonts().layout("one two three four", &style, None);
    assert_eq!(one.lines.len(), 1);
    let narrow = fonts().layout("one two three four", &style, Some(one.width / 2.0));
    assert!(narrow.lines.len() >= 2);
    for line in &narrow.lines {
        assert!(line.width <= one.width / 2.0 + 0.01, "{line:?}");
        assert!(!line.glyphs.is_empty());
    }
    assert!(narrow.height > one.height);
    assert_eq!(fonts().layout("a\nb\n", &style, None).lines.len(), 3);
}

#[test]
fn a_word_wider_than_its_box_is_broken_between_letters() {
    let style = Style::new(16.0);
    let word = fonts().layout("abcdefghij", &style, None);
    let cut = fonts().layout("abcdefghij", &style, Some(word.width / 3.0));
    assert!(cut.lines.len() >= 3);
    assert_eq!(cut.lines.iter().map(|l| l.glyphs.len()).sum::<usize>(), 10);
}

#[test]
fn right_to_left_writing_reads_from_the_right() {
    let style = Style::new(16.0);
    let hebrew = fonts().layout("שלום", &style, Some(400.0));
    let line = &hebrew.lines[0];
    assert!(line.x > 0.0, "a right-to-left paragraph starts at the right");
    let first = line.glyphs.iter().find(|g| g.at == 0).unwrap();
    let last = line.glyphs.iter().max_by_key(|g| g.at).unwrap();
    assert!(first.x > last.x);
    let mixed = fonts().layout("go שלום now", &style, None);
    let go = mixed.lines[0].glyphs.iter().find(|g| g.at == 0).unwrap();
    let now = mixed.lines[0].glyphs.iter().max_by_key(|g| g.at).unwrap();
    assert!(go.x < now.x, "left-to-right words keep their order around it");
}

#[test]
fn the_computers_fonts_join_later_and_built_in_ids_still_hold() {
    let f = Fonts::among_with(Database::new(), &[], Sharp::default(), None);
    f.pending.replace(Some(Vec::new()));
    let built: Vec<(ID, String)> = f.db.borrow().faces().map(|x| (x.id, x.post_script_name.clone())).collect();
    let system = scanned(true).unwrap();
    assert!(f.families().len() >= built.len());
    assert!(f.pending.borrow().is_none());
    assert_eq!(*f.sans.borrow(), sans_of(&system), "Trindle Sans is taken up if installed");
    let db = f.db.borrow();
    for (id, name) in built {
        assert_eq!(db.face(id).map(|x| x.post_script_name.clone()), Some(name));
    }
}

#[test]
fn the_first_window_does_not_wait_for_a_slow_scan_and_the_fonts_join_when_it_ends() {
    let system: &'static std::sync::OnceLock<Database> = Box::leak(Box::default());
    let (release, held) = std::sync::mpsc::channel::<()>();
    let warm = std::sync::Mutex::new(Some(std::thread::spawn(move || {
        let _ = held.recv();
        let _ = system.set(scan());
    })));
    let started = std::time::Instant::now();
    assert!(scanned_from(&warm, system, false).is_none(), "the scan is still going");
    assert!(started.elapsed() < std::time::Duration::from_millis(100), "and nothing waited for it");
    let first = Fonts::among_with(Database::new(), &[], Sharp::default(), None);
    assert!(first.chosen(&Style::new(16.0)).is_some(), "the first frame is set in the built-in font");
    release.send(()).unwrap();
    let ready = scanned_from(&warm, system, true).unwrap();
    assert!(system.get().is_some() && ready.faces().count() == system.get().unwrap().faces().count(), "the scan's answer is kept for the repaint");
}

#[test]
fn the_built_in_fonts_measure_alike_whatever_is_installed() {
    let alone = Fonts::among_with(Database::new(), &[], Sharp::default(), None);
    assert_eq!(*alone.sans.borrow(), BUILT_IN);
    let sans = |weight, italic| Style { family: Some(BUILT_IN.into()), weight, italic, ..Style::new(20.0) };
    let width = |f: &Fonts, s: &Style| {
        let boxes = if s.family.as_deref() == Some(MONOSPACE) { " ┌─┐" } else { "" };
        f.layout(&format!("Hamburgefonts 0123 سلام{boxes}"), s, None).width
    };
    let known = [(sans(400, false), 242.68), (sans(700, false), 257.8), (sans(400, true), 231.24), (Style::monospace(20.0), 313.34)];
    for (style, wide) in known {
        assert_eq!(width(&alone, &style), width(fonts(), &style), "{style:?}");
        assert!((width(&alone, &style) - wide).abs() < 0.01, "{style:?}: {}", width(&alone, &style));
    }
}

#[test]
fn the_persons_own_mono_font_is_used_when_installed_else_the_built_in() {
    let db = scanned(true).unwrap();
    let own = db.faces().find(|f| f.monospaced && f.families.iter().all(|(n, _)| !BUILT_IN_FAMILIES.contains(&n.as_str()))).and_then(|f| f.families.first().map(|(n, _)| n.clone()));
    let tags: &[String] = &[];
    for not_usable in [None, Some("No Such Family 123".to_string()), Some(BUILT_IN.to_string())] {
        let f = Fonts::among_with(scanned(true).unwrap(), tags, Sharp::default(), not_usable);
        assert_eq!(f.monospace(), BUILT_IN_MONO);
    }
    let Some(own) = own else {
        eprintln!("SKIPPED: no installed fixed-width font on this computer");
        return;
    };
    let f = Fonts::among_with(db, tags, Sharp::default(), Some(own.clone()));
    assert_eq!(f.monospace(), own);
    let id = f.chosen(&Style::monospace(14.0)).unwrap();
    assert_eq!(f.db.borrow().face(id).and_then(|face| face.families.first().map(|(n, _)| n.clone())), Some(own));
    assert_eq!(Fonts::new().monospace(), BUILT_IN_MONO, "tests and the harness stay on the built-in font");
}

#[test]
fn letters_the_font_lacks_come_from_another() {
    let style = Style::monospace(16.0);
    let arabic = fonts().layout("سلام", &style, None);
    assert!(arabic.lines[0].glyphs.iter().all(|g| g.glyph != 0), "no empty boxes");
}

#[test]
fn text_is_painted_inside_its_box() {
    let style = Style::new(20.0);
    let layout = fonts().layout("Hello", &style, None);
    let mut canvas = Pixmap::new(200, 60).unwrap();
    fonts().draw(&mut canvas.as_mut(), &layout, 10.0, 10.0, Colour::rgb(1.0, 1.0, 1.0));
    let inked: Vec<(u32, u32)> = (0..60).flat_map(|y| (0..200).map(move |x| (x, y)))
        .filter(|&(x, y)| canvas.pixel(x, y).unwrap().alpha() > 0).collect();
    assert!(inked.len() > 50);
    for (x, y) in inked {
        assert!(x >= 10 && (x as f32) <= 11.0 + layout.width && y >= 10 && (y as f32) <= 11.0 + layout.height, "{x},{y}");
    }
}

#[test]
fn upright_words_are_recorded_as_one_run_of_letters() {
    let layout = fonts().layout("Hello", &Style::new(20.0), None);
    let mut painter = Painter::default();
    fonts().paint(&mut painter, &layout, 10.3, 10.6, Colour::rgb(1.0, 1.0, 1.0));
    let runs: Vec<usize> = painter.ops.iter().filter_map(|op| match op {
        crate::paint::Op::Glyphs { run, .. } => Some(run.len()),
        _ => None,
    }).collect();
    assert_eq!(runs, [5]);
}

#[test]
fn lines_sit_where_they_are_aligned() {
    let at = |align| fonts().layout("hi", &Style { align, ..Style::new(16.0) }, Some(300.0)).lines[0].x;
    let (start, centre, end) = (at(Align::Start), at(Align::Centre), at(Align::End));
    assert_eq!(start, 0.0);
    assert!((centre * 2.0 - end).abs() < 0.01 && end > 250.0);
}

#[test]
fn a_click_finds_its_place_and_the_caret_stands_there() {
    let style = Style::new(16.0);
    let text = "one two\nthree";
    let layout = fonts().layout(text, &style, None);
    assert_eq!(layout.hit(-5.0, 1.0), 0);
    assert_eq!(layout.hit(999.0, 1.0), 7);
    assert_eq!(layout.hit(999.0, layout.height - 1.0), text.len());
    for at in [0, 3, 4, 7, 8, 13] {
        let (x, top, height) = layout.caret(at);
        assert_eq!(layout.hit(x, top + height / 2.0), at, "{at}");
    }
    assert!(layout.caret(8).1 > layout.caret(7).1, "a new line starts lower");
    let hebrew = fonts().layout("שלום", &style, None);
    assert!(hebrew.caret(0).0 > hebrew.caret("שלום".len()).0, "right to left starts at the right");
}

#[test]
fn stretches_are_turned_round_deepest_first() {
    let l = |n| Level::new(n).unwrap();
    let mut parts = [(l(0), 'a'), (l(1), 'b'), (l(2), 'c'), (l(1), 'd'), (l(0), 'e')];
    reorder(&mut parts);
    assert_eq!(parts.map(|p| p.1), ['a', 'd', 'c', 'b', 'e']);
}

#[test]
fn emoji_are_drawn_in_colour_beside_plain_letters() {
    let Some(data) = emoji::tests::test_font(emoji::tests::NOTO) else { return };
    let fonts = Fonts::new();
    fonts.db.borrow_mut().load_font_data(data);
    let layout = fonts.layout("a\u{1F600}", &Style::new(32.0), None);
    let mut pixmap = Pixmap::new(120, 60).unwrap();
    fonts.draw(&mut pixmap.as_mut(), &layout, 4.0, 4.0, Colour::rgb(1.0, 1.0, 1.0));
    assert!(emoji::tests::coloured(&pixmap) > 300);
    let grey = pixmap.pixels().iter().filter(|c| c.alpha() > 200 && c.red() == c.blue() && c.red() > 200).count();
    assert!(grey > 20);
}

#[test]
fn a_letter_asked_for_as_an_emoji_is_coloured_though_the_main_font_has_it() {
    let Some(emoji_font) = emoji::tests::test_font(emoji::tests::NOTO) else { return };
    let Ok(own) = std::fs::read("/usr/share/fonts/truetype/trindle-sans/TrindleSans-Variable.ttf") else { return };
    let fonts = Fonts::new();
    fonts.db.borrow_mut().load_font_data(emoji_font);
    fonts.db.borrow_mut().load_font_data(own);
    *fonts.sans.borrow_mut() = FAMILY.to_string();
    let drawn = |words: &str| {
        let layout = fonts.layout(words, &Style::new(32.0), None);
        let mut pixmap = Pixmap::new(120, 60).unwrap();
        fonts.draw(&mut pixmap.as_mut(), &layout, 4.0, 4.0, Colour::rgb(1.0, 1.0, 1.0));
        emoji::tests::coloured(&pixmap)
    };
    assert_eq!(drawn("\u{2764}"), 0, "alone it is a letter of the main font");
    assert!(drawn("\u{2764}\u{fe0f}") > 300);
}
