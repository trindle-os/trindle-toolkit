//! Pictures kept in tests/golden and compared pixel by pixel; TRINDLE_GOLDEN=refresh rewrites them. All but `words` are text-free, so no installed font changes them; `words` is drawn with `text::built_in_fonts_only`.

use super::*;
use crate::theme::tests::every_look;
use crate::widget::tick;

const SMALL: (u32, u32) = (300, 150);

pub(crate) fn parts() -> Node<()> {
    column(vec![
        row(vec![switch(true, "On", |_| ()), switch(false, "Off", |_| ()), tick(true, "Agree", |_| ()), tick(false, "Agree", |_| ()), ring(0.64, 40.0).tone(Tone::Accent)]).gap(crate::theme::gaps().large),
        slider(62.0, 0.0, 100.0, 1.0, "Volume", |_| ()).width(Length::Fixed(240.0)),
        row(vec![card(space()).width(Length::Fixed(120.0)).height(Length::Fixed(40.0)), ring(0.3, 40.0)]).gap(crate::theme::gaps().large),
    ]).gap(14.0).padding(crate::theme::gaps().large)
}

fn path(name: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden").join(format!("{name}.png"))
}

fn look(name: &str) -> crate::theme::Theme {
    every_look().into_iter().find(|(n, _)| n == name).unwrap_or_else(|| panic!("{name}")).1
}

fn painted(name: &str) -> tiny_skia::Pixmap {
    let mut ui = Ui::new(parts(), look(name));
    ui.lay_at(SMALL.0, SMALL.1);
    let mut pix = tiny_skia::Pixmap::new(SMALL.0, SMALL.1).unwrap();
    ui.paint(&mut pix.as_mut(), 1.0);
    pix
}

fn table(name: &str) -> tiny_skia::Pixmap {
    use crate::table::{paint_column_drop, paint_table, Column, Table};
    let theme = look(name);
    let (fonts, icons) = (crate::text::Fonts::new(), crate::icon::Icons::new());
    let pen = crate::widget::Pen { theme: &theme, fonts: &fonts, icons: &icons, scale: 1.0 };
    let mut t = Table::new(vec![Column::new("", 120.0).sortable(), Column::new("", 80.0), Column::new("", 80.0)]);
    t.set_rows(40);
    t.set_groups(vec![(0, String::new()), (2, String::new())]);
    t.sort = Some((0, true));
    t.chosen = Some(1);
    t.step_column(1);
    t.step_column(1);
    let mut pix = tiny_skia::Pixmap::new(SMALL.0, SMALL.1).unwrap();
    paint_table(&mut pix.as_mut(), &pen, &t, Some(2), |_, _| String::new());
    paint_column_drop(&mut pix.as_mut(), &pen, &t, Some((2, 0)));
    pix
}

#[test]
fn parts_look_as_their_stored_pictures_do() {
    let refresh = std::env::var("TRINDLE_GOLDEN").is_ok_and(|v| v == "refresh");
    let mut faults = Vec::new();
    let pictures = [("dark", "Trindle, dark", painted as fn(&str) -> tiny_skia::Pixmap), ("light", "Trindle, light", painted), ("bold", "the bold theme, dark", painted), ("table-dark", "Trindle, dark", table), ("table-light", "Trindle, light", table)];
    for (name, look, paint) in pictures {
        let pix = paint(look);
        let file = path(name);
        if refresh {
            std::fs::create_dir_all(file.parent().unwrap()).unwrap();
            pix.save_png(&file).unwrap();
            continue;
        }
        let Ok(kept) = tiny_skia::Pixmap::load_png(&file) else {
            faults.push(format!("{name}: no stored picture; run with TRINDLE_GOLDEN=refresh"));
            continue;
        };
        if (kept.width(), kept.height()) != SMALL {
            faults.push(format!("{name}: stored picture is the wrong size"));
            continue;
        }
        let wrong = pix.data().iter().zip(kept.data()).filter(|(a, b)| a.abs_diff(**b) > 6).count();
        if wrong > pix.data().len() / 400 {
            faults.push(format!("{name}: {wrong} of {} values differ by more than a shade; TRINDLE_GOLDEN=refresh keeps the new picture", pix.data().len()));
        }
    }
    assert!(faults.is_empty(), "{}", faults.join("\n"));
}

fn with_words() -> Node<()> {
    use crate::widget::word_button;
    card(column(vec![
        heading("Install Trindle"),
        note("Quiet words under a heading, in the built-in font."),
        entry("John Smith", "Name", |_| ()).width(Length::Fixed(220.0)),
        choice(vec!["Small".to_string(), "Medium".to_string(), "Large".to_string()], 1, "Size", |_| ()),
        word_button("Dark").on_press(()),
    ]).gap(crate::theme::gaps().large)).width(Length::Fixed(300.0))
}

const WORDS: (u32, u32) = (360, 260);

fn words_picture(name: &str) -> tiny_skia::Pixmap {
    let mut ui = Ui::new(column(vec![with_words()]).padding(crate::theme::gaps().large), look(name));
    ui.lay_at(WORDS.0, WORDS.1);
    ui.still();
    let mut pix = tiny_skia::Pixmap::new(WORDS.0, WORDS.1).unwrap();
    ui.paint(&mut pix.as_mut(), 1.0);
    pix
}

const WORDS_INNER: &str = "TRINDLE_GOLDEN_WORDS";

#[test]
fn a_picture_with_words_looks_as_its_stored_one_does_on_every_computer() {
    if std::env::var_os(WORDS_INNER).is_none() {
        let out = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["gallery::golden::a_picture_with_words_looks_as_its_stored_one_does_on_every_computer", "--exact", "--test-threads=1"]).env(WORDS_INNER, "1").output().unwrap();
        let said = String::from_utf8_lossy(&out.stdout);
        assert!(out.status.success() && said.contains("1 passed"), "{}", said.lines().filter(|l| !l.ends_with("... ok")).take(30).collect::<Vec<_>>().join("\n"));
        return;
    }
    crate::text::built_in_fonts_only();
    let first = words_picture("Trindle, dark");
    assert_eq!(first.data(), words_picture("Trindle, dark").data(), "the same picture twice");
    assert!(first.pixels().iter().filter(|p| **p != first.pixels()[0]).count() > 2000, "words were drawn");
    let file = path("words");
    if std::env::var("TRINDLE_GOLDEN").is_ok_and(|v| v == "refresh") {
        first.save_png(&file).unwrap();
        return;
    }
    let kept = tiny_skia::Pixmap::load_png(&file).expect("no stored picture; run with TRINDLE_GOLDEN=refresh");
    assert_eq!((kept.width(), kept.height()), (WORDS.0, WORDS.1), "stored picture is the wrong size");
    let wrong = first.data().iter().zip(kept.data()).filter(|(a, b)| a.abs_diff(**b) > 6).count();
    assert!(wrong <= first.data().len() / 400, "{wrong} of {} values differ by more than a shade; TRINDLE_GOLDEN=refresh keeps the new picture", first.data().len());
}
