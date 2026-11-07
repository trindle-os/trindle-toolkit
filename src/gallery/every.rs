//! Every public widget, made by its own constructor and laid out alone, has a name a screen reader speaks on each part the keyboard reaches or that means something.

use std::rc::Rc;

use accesskit::Action;

use crate::colour::Colour;
use crate::theme::{Palette, Theme};
use crate::ui::Ui;
use crate::widget::*;
use crate::{badges, bars, buttons, chips, code, combo, complete, crash, dial, duration, emoji_picker, find, frames, info, nav, open_with, outline, pages,
            dialog, panel, picker, range, rating, rows, settings, spin, steps, tiles, toolbox, validate, when};

fn every() -> Vec<(&'static str, Node<()>)> {
    let words = || vec!["Small".to_string(), "Medium".to_string(), "Large".to_string()];
    let page = || column(vec![label("A page")]);
    let picture = || Rc::new(image::DynamicImage::new_rgba8(8, 8));
    let today = when::Date::new(2026, 10, 30).unwrap();
    let apps = [open_with::App { id: "org.example.Viewer.desktop".into(), name: "Viewer".into(), icon: "image-x-generic".into(), exec: "viewer %f".into(), mimes: vec![] }];
    let about = pages::About { name: "Notes".into(), icon: "text-x-generic".into(), version: "1.0".into(), summary: "Notes kept as you type.".into(), ..Default::default() };
    let places = [outline::Entry::new("Start", 0, "start"), outline::Entry::new("More", 1, "more")];
    let tools = [toolbox::Tool { icon: "edit-copy-symbolic", name: "Copy" }, toolbox::Tool { icon: "edit-paste-symbolic", name: "Paste" }];
    let chips = chips::Chips { chips: vec!["Ely".into()], draft: String::new() };
    vec![
        ("spinner", spinner()),
        ("ring", ring(0.4, 40.0)),
        ("icon_circle", icon_circle("folder-symbolic")),
        ("icon_image", icon_image("folder-symbolic", 24.0, Tone::Plain)),
        ("avatar", avatar(None, "John Smith", 40.0)),
        ("image_view", image_view(picture(), "A picture", (80.0, 80.0))),
        ("drawing", drawing((40.0, 40.0), "A drawing", |_, _| {})),
        ("remove", remove("user-trash-symbolic", "Remove").on_press(())),
        ("remove_by_hand", remove_by_hand("user-trash-symbolic", "Remove", "remove").on_press(())),
        ("button", button("list-add-symbolic", "Add").on_press(())),
        ("link", link("https://example.com")),
        ("link_named", link_named("Example", "https://example.com")),
        ("main_action", main_action("document-open-symbolic", "Install").on_press(())),
        ("quiet_action", quiet_action("document-open-symbolic", "Open").on_press(())),
        ("main_word", main_word("Save").on_press(())),
        ("word_button", word_button("Cancel").on_press(())),
        ("tick", tick(true, "Agree", |_| ())),
        ("switch", switch(false, "Wi-Fi", |_| ())),
        ("entry", entry("", "Name", |_| ())),
        ("entry secret", entry("", "Password", |_| ()).secret().key("password")),
        ("choice", choice(words(), 0, "Size", |_| ())),
        ("dropdown", dropdown(words(), None, "Size", |_| ())),
        ("one_of", one_of(words(), Some(1), "Size", |_| ())),
        ("slider", slider(5.0, 0.0, 10.0, 1.0, "Volume", |_| ())),
        ("list", list(vec![item("One"), item("Two")], Some(0), |_| ()).called("Numbers")),
        ("tree", tree(vec![tree_row(item("Home"), 0), tree_row(item("Notes"), 1)], None, |_| (), |_| (), |_| ()).called("Folders")),
        ("frames::dots", frames::dots(3, 1, |_| ())),
        ("frames::pager", frames::pager(vec![page(), page()], 200.0, 0.0, |_| ())),
        ("rating::rating", rating::rating(3, 5, "Stars", |_| ())),
        ("nav::stack", nav::stack(words(), 0, |_| (), page())),
        ("nav::stack_beside", nav::stack_beside(vec![item("One"), item("Two")], 0, |_| (), page())),
        ("nav::nav_view", nav::nav_view("Details", Some(()), page())),
        ("nav::split_view", nav::split_view(200.0, page(), page(), "Details", true, ())),
        ("nav::drawer", nav::drawer(page(), page(), true, nav::Edge::Start, 200.0, ())),
        ("nav::bottom_sheet", nav::bottom_sheet(page(), page(), true, ())),
        ("steps::wizard", steps::wizard(1, &words(), page(), (), ())),
        ("tiles::sketch", tiles::sketch("A sketch", (10.0, 10.0), 40.0, |_| {})),
        ("tiles::bars", tiles::bars(Palette::Indigo, vec![1.0, 3.0, 2.0])),
        ("tiles::fill_bar", tiles::fill_bar(Some(0.5), "Disc")),
        ("tiles::standard_ring", tiles::standard_ring(Some(0.5), 60.0)),
        ("tiles::time_pie", tiles::time_pie(0.3, 60.0, |t: &Theme| (t.accent, t.panel))),
        ("tiles::tick_box", tiles::tick_box(true)),
        ("tiles::radio_group", tiles::radio_group(vec![("Day".into(), "Each day".into()), ("Week".into(), "Each week".into())], 0, |_| ())),
        ("tiles::empty_state", tiles::empty_state("Nothing yet", Some(word_button("Add").on_press(())))),
        ("tiles::loading_state", tiles::loading_state(80.0)),
        ("tiles::skeleton", tiles::skeleton(3)),
        ("tiles::error_state", tiles::error_state("Could not read it", word_button("Try again").on_press(()))),
        ("settings::setting", settings::setting("Wi-Fi", "Join known networks", switch(true, "Wi-Fi", |_| ()))),
        ("settings::look_page", settings::look_page(&settings::LookChoice::default(), &picker::ColourBox::new(Colour::rgb(0.98, 0.78, 0.07)), |_| ())),
        ("crash::page", crash::page("Notes", "A report", ())),
        ("bars::progress_bar", bars::progress_bar(Some(0.4), "Copying")),
        ("dialog::confirm", dialog::confirm("Delete?", "It goes to the bin.", "Delete", (), (), 0)),
        ("dialog::choose", dialog::choose("Name taken", "Pick one.", vec![("Keep both".to_string(), ()), ("Replace".to_string(), ())], (), 0)),
        ("dialog::ask", dialog::ask("Rename", "A new name.", "", "New name", |_| (), "Rename", (), (), 0)),
        ("bars::path_bar", bars::path_bar(&["home".to_string(), "notes".to_string()], 2, |_| ())),
        ("pages::preferences", pages::preferences(600.0, vec![item("General")], 0, |_| (), true, (), page())),
        ("pages::status_page", pages::status_page("folder-symbolic", "Nothing here", "Add a file to start.", None)),
        ("pages::welcome_page", pages::welcome_page("folder-symbolic", "Notes", "Notes kept as you type.", "Start", ())),
        ("pages::about_page", pages::about_page("about", &about)),
        ("emoji_picker::emoji_page", emoji_picker::emoji_page("", |_| (), |_| ())),
        ("info::info_panel", info::info_panel(info::Kind::Warning, "Disc nearly full", vec![word_button("Look").on_press(())])),
        ("panel::side_panel", panel::side_panel(page(), page(), 200.0, 1.0, panel::Side::End)),
        ("toolbox::toolbox", toolbox::toolbox(&tools, 0, |_| ())),
        ("toolbox::tab_pages", toolbox::tab_pages(words(), 0, "Sizes", |_| (), page())),
        ("open_with::chooser", open_with::chooser(&apps, Some(0), |_| ())),
        ("outline::outline", outline::outline("places", &places, Some(0), |_| ())),
        ("spin::spin", spin::spin(3.0, 0.0, 10.0, 1.0, "Copies", |_| ())),
        ("spin::Spin::node", spin::Spin::new(3.0, 0.0, 10.0, 1.0).unit("px").node("Copies", |_| ())),
        ("spin::time_spin", spin::time_spin(when::Time::new(9, 30).unwrap(), |_| ())),
        ("spin::month_year", spin::month_year(2026, 10, |_, _| ())),
        ("code::view", code::view("fn main() {}", code::Lang::Rust, "main.rs")),
        ("rows::toggle_row", rows::toggle_row("Wi-Fi", "Join known networks", true, |_| ())),
        ("rows::combo_row", rows::combo_row("Size", "How large", words(), Some(0), |_| ())),
        ("rows::entry_row", rows::entry_row("Name", "Who you are", "", |_| ())),
        ("rows::spin_row", rows::spin_row("Copies", "How many", 1.0, (1.0, 9.0, 1.0), |_| ())),
        ("buttons::colour_button", buttons::colour_button(Colour::rgb(0.2, 0.4, 0.8), "Colour", ())),
        ("buttons::font_button", buttons::font_button("Trindle Sans", 12.0, "Font", ())),
        ("buttons::file_button", buttons::file_button(None, "File", ())),
        ("buttons::lock_button", buttons::lock_button(true, ())),
        ("picker::swatch_grid", picker::swatch_grid(&[Colour::rgb(1.0, 0.0, 0.0), Colour::rgb(0.0, 0.0, 1.0)], Some(0), 2, |_| ())),
        ("picker::ColourBox", picker::ColourBox::new(Colour::rgb(0.2, 0.4, 0.8)).view(|_| ())),
        ("dial::dial", dial::dial(3.0, 0.0, 10.0, 1.0, "Bass", |_| ())),
        ("dial::level_bar", dial::level_bar(3.0, 0.0, 10.0, "Battery")),
        ("dial::scale", dial::scale(3.0, 0.0, 10.0, 1.0, &[(0.0, "Low"), (10.0, "High")], "Speed", |_| ())),
        ("dial::volume", dial::volume(0.5, false, (), |_| ())),
        ("badges::badge_count", badges::badge_count(4, "Unread")),
        ("badges::avatar_group", badges::avatar_group(vec![(None, "John Smith".into()), (None, "Jane Smith".into())], 32.0, 1)),
        ("combo::combo", combo::combo("", &words(), "Size", |_| ())),
        ("complete::Complete", complete::Complete::new("").view("town", "Town", &words(), |_| ())),
        ("chips::Chips", chips.view("Towns", |_| ())),
        ("find::Find", find::Find::default().view(|_| ())),
        ("validate::CheckedBox", validate::CheckedBox::new(validate::Mask::Phone, "07700 900123").view("Phone", |_| ())),
        ("range::Range", range::Range::new(2.0, 8.0, 0.0, 10.0, 1.0).node("Price", |_| ())),
        ("when::DateBox", when::DateBox::new(Some(today), today).view(|_| ())),
        ("when::RangeBox", when::RangeBox::new(today).view(|_| ())),
        ("when::TimeBox", when::TimeBox::new(when::Time::new(9, 30).unwrap()).view(|_| ())),
        ("duration::DurationBox", duration::DurationBox::new(90).view(|_| ())),
    ]
}

#[test]
fn every_public_widget_names_each_part_a_reader_reaches() {
    let mut wrong = Vec::new();
    for (name, node) in every() {
        let mut ui = Ui::new(node, Theme::new(true, "#fbc711"));
        ui.lay_at(800, 600);
        let tree = ui.access("Every widget");
        for (id, p) in &tree.nodes {
            let named = [p.label(), p.value()].into_iter().flatten().any(|n| !n.trim().is_empty());
            if p.supports_action(Action::Focus) && !p.is_hidden() && matches!(p.role(), accesskit::Role::Unknown | accesskit::Role::GenericContainer) {
                wrong.push(format!("{name}: {id:?} can be reached but has no role"));
            }
            if p.supports_action(Action::Focus) && !named && !p.is_hidden() {
                wrong.push(format!("{name}: {id:?} {:?} can be reached but has no name", p.role()));
            }
        }
        wrong.extend(ui.unnamed_controls().into_iter().map(|w| format!("{name}: {w}")));
        wrong.extend(ui.unnamed_lists().into_iter().map(|w| format!("{name}: a list with no name{w}")));
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}
