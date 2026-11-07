//! Every part in each of its states, in every look, again under high contrast and under large text: each is drawn, none is cut off, none runs past a narrow window, and the keyboard's ring shows on each part it can reach.

use super::*;
use crate::theme::tests::{every_look, every_look_high_contrast};
use crate::widget::{icon_image, link_named, quiet_action, scroll, scroll_x, section, section_rule, tick, title, word_button};

fn parts() -> Vec<(&'static str, Node<()>)> {
    let two = || vec!["Small".to_string(), "Medium".to_string()];
    let big = || card(column(vec![label("A long line inside a scrolling area that is wider than the window it sits in, to be scrolled sideways")])).width(Length::Fixed(900.0));
    vec![
        ("button-rest", button("list-add-symbolic", "Add")),
        ("button-on", button("document-open-symbolic", "Chosen").on(true).on_press(())),
        ("button-off", button("edit-paste-symbolic", "Paste").off(true)),
        ("main-rest", main_action("document-open-symbolic", "Install").on_press(())),
        ("main-off", main_action("document-open-symbolic", "Install").off(true)),
        ("remove-rest", remove("user-trash-symbolic", "Remove").on_press(())),
        ("quiet-rest", quiet_action("list-add-symbolic", "New").on_press(())),
        ("word-rest", word_button("Dark").on_press(())),
        ("word-on", word_button("Light").on(true).on_press(())),
        ("link-rest", link_named("Trindle", "https://trindle.org")),
        ("chip", card(note("Waiting")).chip().width(Length::Shrink)),
        ("switch-on", switch(true, "On", |_| ())),
        ("switch-off", switch(false, "Off", |_| ())),
        ("switch-grey", switch(true, "On", |_| ()).off(true)),
        ("tick-on", tick(true, "Agree", |_| ())),
        ("tick-off", tick(false, "Agree", |_| ())),
        ("tick-grey", tick(true, "Agree", |_| ()).off(true)),
        ("choice-rest", choice(two(), 1, "Size", |_| ())),
        ("choice-grey", choice(two(), 0, "Size", |_| ()).off(true)),
        ("dropdown-rest", dropdown(vec!["Ely".to_string(), "Brandon".to_string()], Some(1), "Town", |_| ())),
        ("slider-rest", slider(62.0, 0.0, 100.0, 1.0, "Volume", |_| ()).width(Length::Fixed(200.0))),
        ("slider-grey", slider(62.0, 0.0, 100.0, 1.0, "Volume", |_| ()).width(Length::Fixed(200.0)).off(true)),
        ("entry-filled", entry("John Smith", "Name", |_| ()).width(Length::Fixed(220.0))),
        ("entry-empty", entry("", "john@example.com", |_| ()).width(Length::Fixed(220.0))),
        ("entry-secret", entry("correct horse", "Password", |_| ()).secret().width(Length::Fixed(220.0))),
        ("entry-saved", entry("07700 900123", "Phone", |_| ()).glow(1).width(Length::Fixed(220.0))),
        ("entry-grey", entry("John Smith", "Name", |_| ()).width(Length::Fixed(220.0)).off(true)),
        ("problem", note("That port is not free.").tone(Tone::Problem)),
        ("list-rest", list(vec![item("Documents").icon("folder-symbolic").folder().detail("12 items"), item("notes.txt").icon("text-x-generic-symbolic").detail("4 KB")], Some(0), |_| ())),
        ("ring-part", ring(0.64, 56.0).tone(Tone::Accent)),
        ("ring-done", ring(1.0, 56.0)),
        ("ring-failed", ring(0.9, 56.0).tone(Tone::Problem)),
        ("spinner-part", spinner()),
        ("icon", icon_image("folder-symbolic", 24.0, Tone::Calm).called("Folder")),
        ("card-part", card(column(vec![heading("A card"), note("Quiet words.")]))),
        ("page-title", page_title("Page")),
        ("title", title("Window")),
        ("section", section("Section")),
        ("form-label", form_label("Setting")),
        ("headline", headline("A headline")),
        ("rule", section_rule()),
        ("date-weeks", card(crate::when::DateBox { weeks: true, ..crate::when::DateBox::new(None, crate::when::Date::new(2026, 11, 1).unwrap()) }.view(|_| ()))),
        ("scroll-down", scroll("down", column((0..40).map(|i| label(format!("Row {i}"))).collect())).height(Length::Fixed(120.0))),
        ("scroll-across", scroll_x("across", big()).height(Length::Fixed(80.0))),
    ]
}

pub(super) fn page() -> Node<()> {
    column(parts().into_iter().map(|(name, part)| row(vec![part.key(format!("part:{name}"))]).width(Length::Fill)).collect()).gap(8.0).padding(12.0)
}

fn looks() -> Vec<(String, Theme)> {
    let mut all = every_look();
    all.extend(every_look_high_contrast());
    all
}

const LARGE: &str = "TRINDLE_STATES_LARGE";

#[test]
fn every_part_in_every_state_is_drawn_and_none_overflows() {
    let mut faults = Vec::new();
    if std::env::var_os(LARGE).is_some() {
        let plain = crate::common::style::Layers::new(crate::common::style::CSS, "").number("trindle-text-size", false, 1.0, 100.0).unwrap();
        assert!(crate::theme::sheet().text.0 >= 1.9 * plain, "large text is not on");
    }
    for (look, theme) in looks() {
        for width in [1280u32, 320] {
            let mut ui = Ui::new(page(), theme.clone());
            let high = ui.natural_height(width as f32).ceil() as u32 + 8;
            ui.lay_at(width, high);
            faults.extend(ui.squeezed_or_touching().into_iter().map(|f| format!("{look}, {width}: {f}")));
            let boxes = ui.boxes();
            for (name, _) in parts() {
                let Some((_, r, _)) = boxes.iter().find(|(id, _, _)| id.ends_with(&format!("part:{name}"))) else {
                    faults.push(format!("{look}, {width}: {name} is not laid out"));
                    continue;
                };
                if r.w <= 0.0 || r.h <= 0.0 || r.x < -0.5 || r.x + r.w > width as f32 + 0.5 || r.y + r.h > high as f32 + 0.5 {
                    faults.push(format!("{look}, {width}: {name} is at {},{} {}x{} outside the window", r.x, r.y, r.w, r.h));
                }
            }
            let mut pix = tiny_skia::Pixmap::new(width, high).unwrap();
            ui.paint(&mut pix.as_mut(), 1.0);
            if pix.pixels().iter().all(|p| *p == pix.pixels()[0]) {
                faults.push(format!("{look}, {width}: nothing was drawn"));
            }
        }
    }
    assert!(faults.is_empty(), "{}", faults.iter().take(12).cloned().collect::<Vec<_>>().join("\n"));
}

#[test]
fn every_part_in_every_state_is_drawn_and_none_overflows_under_large_text() {
    if std::env::var_os(LARGE).is_some() {
        return;
    }
    let sheet = std::env::temp_dir().join(format!("trindle-states-large-{}.css", std::process::id()));
    std::fs::write(&sheet, ":root { --trindle-large-text: 2; }").unwrap();
    let out = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["gallery::states::every_part_in_every_state_is_drawn_and_none_overflows", "--exact", "--test-threads=1"])
        .env("TRINDLE_SHOT_SHEET", &sheet).env(LARGE, "1").output().unwrap();
    let _ = std::fs::remove_file(&sheet);
    let said = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success() && said.contains("1 passed"), "under large text:\n{}", said.lines().filter(|l| !l.ends_with("... ok")).take(40).collect::<Vec<_>>().join("\n"));
}

fn painted(ui: &mut Ui<()>, width: u32, high: u32) -> tiny_skia::Pixmap {
    let mut pix = tiny_skia::Pixmap::new(width, high).unwrap();
    ui.paint(&mut pix.as_mut(), 1.0);
    pix
}

#[test]
fn every_part_the_keyboard_reaches_shows_its_ring_in_every_look() {
    use crate::colour::Colour;
    use crate::input::{Keysym, Mods};
    let mut faults = Vec::new();
    for (look, theme) in looks() {
        let floor = if look.ends_with("high contrast") { crate::theme::HIGH_EDGES } else { crate::theme::EDGES };
        let ink = |p: &tiny_skia::PremultipliedColorU8| Colour::rgb(f32::from(p.red()) / 255.0, f32::from(p.green()) / 255.0, f32::from(p.blue()) / 255.0);
        let near = |p: &tiny_skia::PremultipliedColorU8| [(p.red(), theme.lit_edge.r), (p.green(), theme.lit_edge.g), (p.blue(), theme.lit_edge.b)].iter().all(|(v, a)| (f32::from(*v) / 255.0 - a).abs() < 0.02);
        for (n, (name, _)) in parts().into_iter().enumerate() {
            let part = parts().swap_remove(n).1;
            let mut ui = Ui::new(column(vec![row(vec![part.key("part")]).width(Length::Fill)]).padding(12.0), theme.clone());
            let (width, high) = (360, ui.natural_height(360.0).ceil() as u32 + 24);
            ui.lay_at(width, high);
            ui.still();
            if !ui.access("Part").nodes.iter().any(|(_, p)| p.supports_action(accesskit::Action::Focus)) {
                continue;
            }
            let (before, was) = (painted(&mut ui, width, high), ui.access("Part").focus);
            ui.input(&Input::Key { sym: Keysym::Tab, text: None, mods: Mods::default(), repeat: false }, Instant::now());
            let after = painted(&mut ui, width, high);
            if ui.access("Part").focus == was {
                if !name.ends_with("grey") && !name.ends_with("-off") {
                    faults.push(format!("{look}: {name} can be focused by a screen reader but not by Tab"));
                }
                continue;
            }
            let added: Vec<_> = after.pixels().iter().zip(before.pixels()).filter(|(a, b)| near(a) && !near(b)).collect();
            if added.len() < 8 {
                faults.push(format!("{look}: {name} shows no ring when the keyboard is on it ({} pixels)", added.len()));
                continue;
            }
            let page = before.pixel(1, 1).map(|p| ink(&p)).unwrap();
            let stands = crate::theme::contrast(theme.lit_edge, page);
            if stands < floor {
                faults.push(format!("{look}: {name}'s ring stands only {stands:.2} to one from the page, not {floor}"));
            }
        }
    }
    assert!(faults.is_empty(), "{} faults:\n{}", faults.len(), faults.iter().take(25).cloned().collect::<Vec<_>>().join("\n"));
}

#[test]
#[ignore = "writes pictures to look at; it checks nothing"]
fn shoot_the_states() {
    let out = std::path::PathBuf::from(std::env::var("TRINDLE_SHOTS").unwrap_or_else(|_| "target/style-check".into()));
    std::fs::create_dir_all(&out).unwrap();
    for (look, theme) in looks().into_iter().filter(|(look, _)| look.starts_with("Trindle")) {
        let mut ui = Ui::new(page(), theme);
        let high = ui.natural_height(1280.0).ceil() as u32 + 8;
        ui.lay_at(1280, high);
        ui.still();
        painted(&mut ui, 1280, high).save_png(out.join(format!("states-{}.png", look.to_lowercase().replace([',', ' '], "-").replace("--", "-")))).unwrap();
    }
}
