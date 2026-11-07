//! One window of the common controls, each changing something you can see; src/gallery.rs is the test that draws every control for the screenshots. Built, not run on screen, by `cargo build --release --examples`.

use std::time::Instant;

use trindle_toolkit::dialog::confirm;
use trindle_toolkit::input::{Input, Keysym, Mods};
use trindle_toolkit::table::{paint_band, paint_table, Branch, Column, Gesture, Table, Touch};
use trindle_toolkit::bars::progress_bar;
use trindle_toolkit::range::{Range, RangeMsg};
use trindle_toolkit::spin::{Spin, SpinMsg};
use trindle_toolkit::t;
use trindle_toolkit::ui::{App, Shown};
use trindle_toolkit::widget::{drawing, button, card, choice, column, dropdown, entry, form_label, heading, label, main_action, note, page_title, remove, ring, row, scroll, scroll_x, slider, split, switch, word_button, Length, MenuRow, Node, Pointer};
use trindle_toolkit::window::Kind;

const TOWNS: [&str; 4] = ["Mildenhall", "Ely", "Thetford", "Brandon"];
const FILES: (f32, f32) = (360.0, 220.0);

fn entry_of(id: usize) -> (String, bool) {
    match id {
        0 => (t!("kit-example-gallery-columns-documents").to_string(), true),
        1 => (t!("kit-example-gallery-columns-pictures").to_string(), true),
        2 => (t!("kit-example-gallery-columns-notes").to_string(), false),
        10 => (t!("kit-example-gallery-columns-letters").to_string(), false),
        11 => (t!("kit-example-gallery-columns-reports").to_string(), false),
        _ => (t!("kit-example-gallery-columns-holiday").to_string(), false),
    }
}

fn inside(id: usize) -> Vec<Branch> {
    match id {
        0 => vec![Branch::leaf(10), Branch::leaf(11)],
        _ => vec![Branch::leaf(20)],
    }
}

fn file_words(table: &Table, row: usize, column: usize) -> String {
    let Some(b) = table.branch(row) else { return String::new() };
    let (name, folder) = entry_of(b.id);
    match column {
        0 => name,
        _ if folder => t!("kit-example-gallery-columns-folder").to_string(),
        _ => t!("kit-example-gallery-columns-file").to_string(),
    }
}

struct Gallery {
    presses: u32,
    notices: bool,
    size: usize,
    town: Option<usize>,
    volume: f32,
    copies: Spin,
    price: Range,
    asking: bool,
    turn: u32,
    name: String,
    files: Table,
    touch: Gesture,
}

#[derive(Clone)]
enum Msg {
    Add,
    Reset,
    Notices(bool),
    Size(usize),
    Town(usize),
    Volume(f32),
    Copies(SpinMsg),
    Price(RangeMsg),
    Ask,
    Answer(bool),
    Name(String),
    FilesPointer(Pointer, Mods),
    FilesKey(Keysym, Mods),
}

impl Gallery {
    fn files_touched(&mut self, touched: Option<Touch>) {
        if let Some(Touch::Open(row)) = touched {
            if let Some(id) = self.files.branch(row).map(|b| b.id) {
                self.files.open_row(row, inside(id));
            }
        }
    }
}

impl App for Gallery {
    type Msg = Msg;

    fn update(&mut self, msg: Msg) {
        match msg {
            Msg::Add => self.presses += 1,
            Msg::Reset => self.presses = 0,
            Msg::Notices(on) => self.notices = on,
            Msg::Size(i) => self.size = i,
            Msg::Town(i) => self.town = Some(i),
            Msg::Volume(v) => self.volume = v,
            Msg::Copies(m) => {
                self.copies.update(m);
            }
            Msg::Price(m) => {
                self.price.update(m);
            }
            Msg::Ask => {
                self.asking = true;
                self.turn += 1;
            }
            Msg::Answer(yes) => {
                self.asking = false;
                if yes {
                    self.presses = 0;
                }
            }
            Msg::Name(text) => self.name = text,
            Msg::FilesPointer(p, mods) => {
                let touched = self.touch.pointer_mods(&mut self.files, p, Instant::now(), mods, FILES);
                self.files_touched(touched);
            }
            Msg::FilesKey(sym, mods) => {
                let touched = self.files.key(sym, mods, FILES.1);
                self.files_touched(touched);
            }
        }
    }

    fn view(&self) -> Node<Msg> {
        if self.asking {
            return confirm(t!("kit-example-gallery-ask-title"), t!("kit-example-gallery-ask-message"), t!("kit-example-gallery-ask-yes"), Msg::Answer(true), Msg::Answer(false), self.turn);
        }
        let sizes = vec![t!("kit-example-gallery-size-small").to_string(), t!("kit-example-gallery-size-medium").to_string(), t!("kit-example-gallery-size-large").to_string()];
        let menu_sizes = sizes.clone();
        let (notices, size) = (self.notices, self.size);
        let towns = TOWNS.iter().map(|town| town.to_string()).collect();
        let buttons = card(column(vec![
            heading(t!("kit-example-gallery-buttons")),
            row(vec![
                button("list-add-symbolic", t!("kit-example-gallery-add")).on_press(Msg::Add),
                main_action("document-open-symbolic", t!("kit-example-gallery-main")).on_press(Msg::Add),
                remove("user-trash-symbolic", t!("kit-example-gallery-remove")).on_press(Msg::Reset),
            ]).gap(8.0),
            note(t!("kit-example-gallery-count", n = self.presses)),
        ]).gap(12.0)).width(Length::Fill);
        let choices = card(column(vec![
            heading(t!("kit-example-gallery-choices")),
            switch(self.notices, t!("kit-example-gallery-switch-name"), Msg::Notices),
            form_label(t!("kit-example-gallery-size-name")),
            choice(sizes, self.size, t!("kit-example-gallery-size-name"), Msg::Size),
            form_label(t!("kit-example-gallery-town-name")),
            dropdown(towns, self.town, t!("kit-example-gallery-town-name"), Msg::Town),
            slider(self.volume, 0.0, 100.0, 1.0, t!("kit-example-gallery-volume-name"), Msg::Volume),
            note(t!("kit-example-gallery-volume", n = self.volume.round() as u32)),
        ]).gap(12.0)).width(Length::Fill);
        let typing = card(column(vec![
            heading(t!("kit-example-gallery-typing")),
            entry(self.name.clone(), t!("kit-example-gallery-entry-hint"), Msg::Name),
            label(if self.name.is_empty() { String::new() } else { t!("kit-example-gallery-hello", name = self.name.as_str()) }),
        ]).gap(12.0)).width(Length::Fill);
        let progress = card(column(vec![heading(t!("kit-example-gallery-progress")), ring(self.volume / 100.0, 48.0)]).gap(12.0)).width(Length::Fill);
        let spans = card(column(vec![
            heading(t!("kit-example-gallery-span")),
            progress_bar(Some(self.volume / 100.0), t!("kit-example-gallery-span-progress")),
            self.price.node(t!("kit-example-gallery-span-name"), Msg::Price),
            note(t!("kit-example-gallery-span-now", low = self.price.low.round() as u32, high = self.price.high.round() as u32)),
        ]).gap(12.0)).width(Length::Fill);
        let (table, touch) = (self.files.clone(), self.touch.clone());
        let said = table.said(FILES.1, |r, c| file_words(&table, r, c));
        let columns = card(column(vec![
            heading(t!("kit-example-gallery-columns")),
            note(t!("kit-example-gallery-columns-how")),
            drawing(FILES, t!("kit-example-gallery-columns-name"), move |canvas, pen| {
                let hover = touch.hovering(&table, FILES.1);
                paint_table(canvas, pen, &table, hover, |r, c| file_words(&table, r, c));
                paint_band(canvas, pen, touch.band(&table, FILES));
            }).key("gallery-columns").on_pointer_mods(Msg::FilesPointer).table_rows(said).on_keys(|input| match input {
                Input::Key { sym, mods, .. } => Some(Msg::FilesKey(*sym, *mods)),
                _ => None,
            }),
        ]).gap(12.0)).width(Length::Fill);
        let numbers = card(column(vec![heading(t!("kit-example-gallery-copies")), self.copies.node(t!("kit-example-gallery-copies-name"), Msg::Copies)]).gap(12.0)).width(Length::Fill);
        let dialogs = card(column(vec![
            heading(t!("kit-example-gallery-dialog")),
            note(t!("kit-example-gallery-dialog-how")),
            button("edit-clear-symbolic", t!("kit-example-gallery-dialog-open")).on_press(Msg::Ask),
        ]).gap(12.0)).width(Length::Fill);
        let along = card(column(vec![
            heading(t!("kit-example-gallery-along")),
            note(t!("kit-example-gallery-along-how")),
            scroll_x("gallery-along", row((1..=12).map(|n| word_button(t!("kit-example-gallery-along-item", n = n)).on_press(Msg::Add).width(Length::Fixed(96.0))).collect()).gap(8.0)).height(Length::Fixed(48.0)),
        ]).gap(12.0)).width(Length::Fill);
        let panes = card(column(vec![
            heading(t!("kit-example-gallery-split")),
            split("gallery-split", card(note(t!("kit-example-gallery-split-first"))).width(Length::Fill), card(note(t!("kit-example-gallery-split-second"))).width(Length::Fill)).height(Length::Fixed(120.0)),
        ]).gap(12.0)).width(Length::Fill);
        let menu = card(column(vec![heading(t!("kit-example-gallery-menu")), note(t!("kit-example-gallery-menu-how"))]).gap(12.0)).width(Length::Fill).context_menu(move || {
            let mut rows = vec![MenuRow::send("", t!("kit-example-gallery-switch-name"), Msg::Notices(!notices)).checked(notices)];
            for (i, words) in menu_sizes.iter().enumerate() {
                let row = MenuRow::send("", words.clone(), Msg::Size(i)).radio(i == size);
                rows.push(if i == 0 { row.rule() } else { row });
            }
            rows
        });
        scroll("gallery", column(vec![page_title(t!("kit-example-gallery-heading")), buttons, choices, typing, progress, numbers, spans, columns, dialogs, along, panes, menu]).padding(24.0).gap(16.0))
    }
}

fn main() {
    let kind = Kind::Window { title: t!("kit-example-gallery-title").into(), app_id: "org.example.Gallery".into() };
    if let Err(why) = Shown::run(kind, (480, 640), Gallery { presses: 0, notices: true, size: 1, town: None, volume: 40.0, copies: Spin::new(1.0, 1.0, 99.0, 1.0).unit(t!("kit-example-gallery-copies-unit")), price: Range::new(20.0, 70.0, 0.0, 100.0, 5.0), asking: false, turn: 0, name: String::new(), files: files(), touch: Gesture::new() }) {
        eprintln!("{why}");
        std::process::exit(1);
    }
}

fn files() -> Table {
    let mut table = Table::new(vec![Column::new(t!("kit-example-gallery-columns-head-name"), 220.0), Column::new(t!("kit-example-gallery-columns-head-kind"), 120.0)]);
    table.set_tree(vec![Branch::closed(0), Branch::closed(1), Branch::leaf(2)]);
    table.multiple = true;
    table
}
