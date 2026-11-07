//! Tests of what the screen reader is told and what it asks done.

use super::*;
use super::super::super::theme::Theme;
use super::super::super::widget::{button, card, choice, column, entry, item, label, list, row, scroll, slider, space, switch, title, Length, Node};

#[derive(Clone, Debug, PartialEq)]
enum Msg {
    Save,
    Wifi(bool),
    Volume(f32),
    Name(String),
    Pick(usize),
    Open(usize),
    Point(w::Pointer),
    Typed(Keysym),
}

fn view(name: &str) -> Node<Msg> {
    column(vec![
        title("Network"),
        card(column(vec![
            button("document-open-symbolic", "Save").on_press(Msg::Save).key("save"),
            button("x", "Not ready").key("idle"),
            button("x", "").spoken("Tray icon").key("quiet"),
            button("x", "Show hidden files").on(false).on_press(Msg::Save).key("hidden"),
            switch(true, "Wi-Fi", Msg::Wifi).key("wifi"),
            slider(0.5, 0.0, 1.0, 0.1, "Volume", Msg::Volume).key("volume"),
            entry(name, "Name", Msg::Name).key("name"),
            entry("hush", "Password", Msg::Name).secret().key("pass"),
        ])).key("card"),
        scroll("area", list((0..30).map(|i| item(format!("Row {i}"))).collect(), Some(2), Msg::Pick).on_open(Msg::Open).key("list"))
            .height(Length::Fixed(120.0)),
    ]).padding(10.0)
}

fn ui(name: &str) -> Ui<Msg> {
    let mut ui = Ui::new(view(name), Theme::new(false, "#fbc711"));
    ui.size = (400, 600);
    ui
}

fn part<'a>(t: &'a TreeUpdate, id: &str, piece: Piece) -> &'a Part {
    let want = part_id(id, piece);
    &t.nodes.iter().find(|(n, _)| *n == want).unwrap_or_else(|| panic!("no part for {id}")).1
}

fn ask(target: NodeId, action: Action, data: Option<ActionData>) -> ActionRequest {
    ActionRequest { action, target_tree: TreeId::ROOT, target_node: target, data }
}

#[test]
fn each_widget_is_told_with_its_role_name_and_value() {
    let mut ui = ui("Ann");
    let t = ui.access("Settings");
    assert_eq!(t.nodes[0].0, WINDOW);
    assert_eq!(t.nodes[0].1.label(), Some("Settings"));
    assert_eq!(t.tree.as_ref().unwrap().toolkit_name.as_deref(), Some("Trindle Toolkit"));
    let ids: Vec<NodeId> = t.nodes.iter().map(|(n, _)| *n).collect();
    for (i, n) in ids.iter().enumerate() {
        assert!(!ids[i + 1..].contains(n));
    }
    for (_, p) in &t.nodes {
        assert!(p.children().iter().all(|c| ids.contains(c)));
    }
    let heading = t.nodes.iter().find(|(_, p)| p.role() == Role::Heading).unwrap();
    assert_eq!(heading.1.label(), Some("Network"));
    assert_eq!(part(&t, "card", Piece::Widget).role(), Role::Group);
    assert!(part(&t, "card", Piece::Widget).children().contains(&part_id("save", Piece::Widget)));
    let save = part(&t, "save", Piece::Widget);
    assert_eq!((save.role(), save.label(), save.is_disabled()), (Role::Button, Some("Save"), false));
    assert!(part(&t, "idle", Piece::Widget).is_disabled());
    assert_eq!(part(&t, "quiet", Piece::Widget).label(), Some("Tray icon"));
    let hidden = part(&t, "hidden", Piece::Widget);
    assert_eq!((hidden.label(), hidden.toggled()), (Some("Show hidden files"), Some(Toggled::False)));
    assert_eq!(save.toggled(), None);
    let wifi = part(&t, "wifi", Piece::Widget);
    assert_eq!((wifi.role(), wifi.toggled()), (Role::Switch, Some(Toggled::True)));
    let volume = part(&t, "volume", Piece::Widget);
    assert_eq!((volume.numeric_value(), volume.numeric_value_step()), (Some(0.5f32 as f64), Some(0.1f32 as f64)));
    let name = part(&t, "name", Piece::Widget);
    assert_eq!((name.role(), name.value()), (Role::TextInput, Some("Ann")));
    let pass = part(&t, "pass", Piece::Widget);
    assert_eq!((pass.role(), pass.value()), (Role::PasswordInput, Some("\u{2022}\u{2022}\u{2022}\u{2022}")));
    let area = part(&t, "area", Piece::Widget);
    assert_eq!((area.role(), area.scroll_y_min()), (Role::ScrollView, Some(0.0)));
    assert!(area.scroll_y_max().unwrap() > 0.0);
    let rows = part(&t, "list", Piece::Widget);
    assert_eq!((rows.role(), rows.children().len()), (Role::ListBox, 30));
    assert_eq!(part(&t, "list", Piece::Row(2)).is_selected(), Some(true));
    assert_eq!(part(&t, "list", Piece::Row(3)).label(), Some("Row 3"));
    assert_eq!(t.focus, WINDOW);
    ui.focus = Some("list".into());
    assert_eq!(ui.access("Settings").focus, part_id("list", Piece::Row(2)));
}

#[test]
fn what_is_scrolled_out_of_sight_is_still_the_screen_readers() {
    for (w, h) in [(480, 200), (480, 1), (480, 2000)] {
        let lines = (0..40).map(|i| row(vec![label(format!("Line {i}")), space(), switch(false, format!("Switch {i}"), Msg::Wifi)])).collect();
        let mut ui = Ui::new(column(vec![title("Many"), scroll("many", column(vec![card(column(lines))]))]), Theme::new(true, "#fbc711"));
        ui.lay_at(w, h);
        let t = ui.access("Many");
        assert_eq!(t.nodes.iter().filter(|(_, p)| p.role() == Role::Switch).count(), 40);
        let kept = ui.kept_from_a_reader();
        assert!(kept.is_empty(), "at {w} by {h} a screen reader could not reach {} parts, the first {:?}", kept.len(), kept.first());
    }
}

#[test]
fn a_secret_box_keeps_its_dots_while_its_eye_shows_it() {
    let mut ui = ui("Ann");
    let t = ui.access("Settings");
    let pass = part(&t, "pass", Piece::Widget);
    assert_eq!((pass.role(), pass.value()), (Role::PasswordInput, Some("\u{2022}\u{2022}\u{2022}\u{2022}")));
    assert!(t.nodes.iter().all(|(_, p)| !p.value().is_some_and(|v| v.contains("hush")) && !p.label().is_some_and(|l| l.contains("hush"))),
            "a hidden password reached the screen reader");
    let eye = part(&t, "pass", Piece::Eye);
    assert_eq!((eye.role(), eye.label(), eye.toggled()), (Role::Button, Some("Show the password"), Some(Toggled::False)));
    let card = part(&t, "card", Piece::Widget).children().to_vec();
    let at = card.iter().position(|c| *c == part_id("pass", Piece::Widget)).unwrap();
    assert_eq!(card.get(at + 1), Some(&part_id("pass", Piece::Eye)));
    assert!(t.nodes.iter().all(|(n, _)| *n != part_id("name", Piece::Eye)));
    let reply = ui.act(&ask(part_id("pass", Piece::Eye), Action::Click, None));
    assert!(reply.redraw && reply.messages.is_empty());
    assert_eq!(ui.news, "Password shown");
    let t = ui.access("Settings");
    let pass = part(&t, "pass", Piece::Widget);
    assert_eq!((pass.role(), pass.value()), (Role::PasswordInput, Some("\u{2022}\u{2022}\u{2022}\u{2022}")));
    assert_eq!(part(&t, "pass", Piece::Run(0)).value(), Some("\u{2022}\u{2022}\u{2022}\u{2022}"));
    assert!(t.nodes.iter().all(|(_, p)| !p.value().is_some_and(|v| v.contains("hush")) && !p.label().is_some_and(|v| v.contains("hush"))));
    let eye = part(&t, "pass", Piece::Eye);
    assert_eq!((eye.label(), eye.toggled()), (Some("Hide the password"), Some(Toggled::True)));
    assert!(t.nodes.iter().any(|(n, p)| *n == NEWS && p.label() == Some("Password shown")));
    ui.act(&ask(part_id("pass", Piece::Eye), Action::Focus, None));
    assert_eq!(ui.access("Settings").focus, part_id("pass", Piece::Eye));
    ui.act(&ask(part_id("pass", Piece::Widget), Action::Focus, None));
    assert_eq!(ui.access("Settings").focus, part_id("pass", Piece::Widget));
    ui.act(&ask(part_id("pass", Piece::Eye), Action::Click, None));
    assert_eq!(ui.news, "Password hidden");
    let t = ui.access("Settings");
    assert_eq!(part(&t, "pass", Piece::Widget).value(), Some("\u{2022}\u{2022}\u{2022}\u{2022}"));
    assert!(t.nodes.iter().all(|(_, p)| !p.value().is_some_and(|v| v.contains("hush"))));
}

#[test]
fn a_pressed_card_is_named_by_the_words_inside_it() {
    let lines = column(vec![
        card(row(vec![label("Copy"), label("this")])).on_press(Msg::Save).key("line"),
        card(space()).on_press(Msg::Save).key("blank"),
    ]);
    let mut ui = Ui::new(lines, Theme::new(false, "#fbc711"));
    ui.size = (400, 200);
    let t = ui.access("Ring");
    let line = part(&t, "line", Piece::Widget);
    assert_eq!((line.role(), line.label()), (Role::Button, Some("Copy this")));
    assert_eq!(part(&t, "blank", Piece::Widget).label(), None);
}

#[test]
fn a_text_box_is_told_letter_by_letter_with_its_selection() {
    let long = "word ".repeat(50);
    let mut ui = ui(&long);
    ui.focus = Some("name".into());
    ui.settle();
    ui.edit("name").unwrap().move_to(4, false);
    let t = ui.access("");
    let runs = part(&t, "name", Piece::Widget).children();
    assert_eq!(runs.len(), 2);
    let first = part(&t, "name", Piece::Run(0));
    assert_eq!((first.role(), first.character_lengths().len()), (Role::TextRun, RUN));
    assert_eq!(&first.word_starts()[..3], &[0, 5, 10]);
    assert_eq!(first.next_on_line(), Some(runs[1]));
    assert_eq!(part(&t, "name", Piece::Run(1)).value().map(str::len), Some(50));
    let sel = part(&t, "name", Piece::Widget).text_selection().unwrap();
    assert_eq!((sel.focus.node, sel.focus.character_index), (runs[0], 4));
    let mut ui = super::tests::ui("");
    let t = ui.access("");
    assert_eq!(part(&t, "name", Piece::Widget).children().len(), 1);
    assert_eq!(part(&t, "name", Piece::Run(0)).value(), Some(""));
}

#[test]
fn a_reader_can_put_the_caret_in_an_empty_box_and_at_the_end_of_a_full_one() {
    use accesskit_consumer::Tree;
    for text in ["", "Ann"] {
        let mut ui = super::tests::ui(text);
        let tree = Tree::new(ui.access(""), true);
        let state = tree.state();
        let mut todo = vec![state.root()];
        let mut found = None;
        while let Some(part) = todo.pop() {
            if part.role() == Role::TextInput {
                found = Some(part);
            }
            todo.extend(part.filtered_children(accesskit_consumer::common_filter));
        }
        let entry = found.expect("a text box is in the tree");
        let end = text.chars().count();
        for at in [0, end] {
            assert!(entry.text_position_from_global_usv_index(at).is_some(), "{text:?}: caret at {at}");
        }
        assert!(entry.text_position_from_global_usv_index(end + 1).is_none(), "{text:?}: past the end is refused");
    }
}

#[test]
fn what_the_screen_reader_asks_is_done() {
    let mut ui = ui("Ann");
    ui.access("");
    let at = |id: &str| part_id(id, Piece::Widget);
    assert_eq!(ui.act(&ask(at("save"), Action::Click, None)).messages, vec![Msg::Save]);
    assert!(ui.act(&ask(at("idle"), Action::Click, None)).messages.is_empty());
    assert_eq!(ui.act(&ask(at("wifi"), Action::Click, None)).messages, vec![Msg::Wifi(false)]);
    assert_eq!(ui.act(&ask(at("volume"), Action::Increment, None)).messages, vec![Msg::Volume(0.6)]);
    let m = ui.act(&ask(at("volume"), Action::SetValue, Some(ActionData::NumericValue(0.93)))).messages;
    assert_eq!(m, vec![Msg::Volume(0.9)]);
    ui.act(&ask(at("wifi"), Action::Focus, None));
    assert_eq!(ui.focus.as_deref(), Some("wifi"));
    let row = |r| part_id("list", Piece::Row(r));
    assert_eq!(ui.act(&ask(row(5), Action::Click, None)).messages, vec![Msg::Pick(5)]);
    assert_eq!(ui.act(&ask(row(2), Action::Click, None)).messages, vec![Msg::Open(2)]);
    ui.act(&ask(row(25), Action::ScrollIntoView, None));
    let s = ui.spot("area").unwrap().clone();
    assert!(ui.offset(&s) > 0.0);
    let m = ui.act(&ask(at("name"), Action::SetValue, Some(ActionData::Value("Bob".into())))).messages;
    assert_eq!(m, vec![Msg::Name("Bob".into())]);
    ui.set_view(view("Bob"));
    ui.access("");
    let run = part_id("name", Piece::Run(0));
    let sel = TextSelection { anchor: TextPosition { node: run, character_index: 0 }, focus: TextPosition { node: run, character_index: 1 } };
    ui.act(&ask(at("name"), Action::SetTextSelection, Some(ActionData::SetTextSelection(sel))));
    assert_eq!(ui.focus.as_deref(), Some("name"));
    let m = ui.act(&ask(at("name"), Action::ReplaceSelectedText, Some(ActionData::Value("R".into())))).messages;
    assert_eq!(m, vec![Msg::Name("Rob".into())]);
    assert!(!ui.act(&ask(NodeId(12345), Action::Click, None)).redraw);
}

#[test]
fn a_fill_bar_tells_how_full_it_is_without_being_busy() {
    let mut ui = Ui::new(crate::tiles::fill_bar::<Msg>(Some(0.4), "Storage").key("quota"), Theme::new(false, "#fbc711"));
    ui.size = (400, 100);
    let t = ui.access("");
    let bar = part(&t, "quota", Piece::Widget);
    assert_eq!((bar.role(), bar.label(), bar.is_busy()), (Role::ProgressIndicator, Some("Storage"), false));
    assert!((bar.numeric_value().unwrap() - 0.4).abs() < 1e-6);
}

#[test]
fn a_radio_group_row_is_told_as_a_radio_button_with_its_state() {
    let rows = vec![("Quick".to_string(), "Fast.".to_string()), ("Full".to_string(), String::new())];
    let mut ui = Ui::new(crate::tiles::radio_group(rows, 1, Msg::Pick).key("how"), Theme::new(false, "#fbc711"));
    ui.size = (400, 300);
    let t = ui.access("");
    let found: Vec<_> = t.nodes.iter().filter(|(_, n)| n.role() == Role::RadioButton).collect();
    let states: Vec<_> = found.iter().map(|(_, n)| (n.label(), n.toggled())).collect();
    assert_eq!(states, [(Some("Quick Fast."), Some(Toggled::False)), (Some("Full"), Some(Toggled::True))]);
}

#[test]
fn a_choice_is_told_as_radio_buttons_and_worked() {
    let kinds = vec!["ext4".to_string(), "exFAT".to_string(), "NTFS".to_string()];
    let mut ui = Ui::new(column(vec![choice(kinds, 1, "Kind", Msg::Pick).key("kind")]).padding(10.0),
                         Theme::new(false, "#fbc711"));
    ui.size = (400, 200);
    let t = ui.access("");
    let group = part(&t, "kind", Piece::Widget);
    assert_eq!((group.role(), group.label(), group.children().len()), (Role::RadioGroup, Some("Kind"), 3));
    let b = part(&t, "kind", Piece::Row(1));
    assert_eq!((b.role(), b.label(), b.toggled()), (Role::RadioButton, Some("exFAT"), Some(Toggled::True)));
    assert_eq!(part(&t, "kind", Piece::Row(2)).toggled(), Some(Toggled::False));
    ui.focus = Some("kind".into());
    assert_eq!(ui.access("").focus, part_id("kind", Piece::Row(1)));
    let row = |r| part_id("kind", Piece::Row(r));
    assert_eq!(ui.act(&ask(row(2), Action::Click, None)).messages, vec![Msg::Pick(2)]);
    assert!(ui.act(&ask(row(1), Action::Click, None)).messages.is_empty());
}

#[test]
#[ignore = "needs a real screen reader and a real screen: run by hand"]
fn a_real_screen_reader_reads_and_works_the_widgets() {
    use std::time::Duration;
    use super::super::super::event::{Flow, Loop};
    use std::cell::RefCell;
    use std::rc::Rc;
    let lp = Loop::new();
    let asked = Rc::new(RefCell::new(Vec::new()));
    let a = asked.clone();
    let mut link = Link::new("Trindle test", lp.post(move |ask| a.borrow_mut().push(ask)));
    let mut ui = ui("Ann");
    let heard = Rc::new(RefCell::new(Vec::new()));
    let h = heard.clone();
    lp.every(Duration::from_millis(20), move || {
        for ask in asked.take() {
            if let Ask::Act(req) = ask {
                for m in ui.act(&req).messages {
                    if let Msg::Name(n) = &m {
                        ui.set_view(view(n));
                    }
                    h.borrow_mut().push(m);
                }
            }
            link.tell(&mut ui);
        }
        Flow::Continue
    });
    let quit = lp.clone();
    lp.after(Duration::from_secs(10), move || quit.quit());
    lp.run();
    let heard = heard.borrow();
    println!("heard {heard:?}");
    assert!(heard.contains(&Msg::Save));
    assert!(heard.contains(&Msg::Name("Bob".into())));
}

fn told(tree: Node<Msg>) -> (Ui<Msg>, TreeUpdate) {
    let mut ui = Ui::new(tree, Theme::new(false, "#fbc711"));
    ui.size = (400, 300);
    let t = ui.access("Window");
    (ui, t)
}

#[test]
fn notices_alerts_and_working_things_are_told_as_such() {
    use super::super::super::widget::{ring, spinner, note, Tone};
    let (_, t) = told(column(vec![
        note("Saved").polite().key("saved"),
        note("Could not save").tone(Tone::Problem).key("bad"),
        note("Disk is full").alert().key("full"),
        spinner().key("spin"),
        ring(0.4, 40.0).key("half"),
        ring(1.0, 40.0).key("done"),
    ]));
    let saved = part(&t, "saved", Piece::Widget);
    assert_eq!(saved.live(), Some(Live::Polite));
    let bad = part(&t, "bad", Piece::Widget);
    assert_eq!((bad.role(), bad.live(), bad.label()), (Role::Alert, Some(Live::Assertive), Some("Could not save")));
    let full = part(&t, "full", Piece::Widget);
    assert_eq!((full.role(), full.live()), (Role::Alert, Some(Live::Assertive)));
    assert!(part(&t, "spin", Piece::Widget).is_busy());
    let half = part(&t, "half", Piece::Widget);
    assert!(!half.is_busy() && half.numeric_value() == Some(0.4f32 as f64) && half.label() == Some("Progress"));
    assert!(!part(&t, "done", Piece::Widget).is_busy());
}

#[test]
fn a_drawing_can_be_a_job_with_progress() {
    use super::super::super::widget::drawing;
    let (_, t) = told(column(vec![
        drawing((40.0, 40.0), "", |_, _| {}).progress(Some(0.25)).called("Copying").key("a"),
        drawing((40.0, 40.0), "", |_, _| {}).progress(None).called("Waiting").key("b"),
        drawing((40.0, 40.0), "", |_, _| {}).progress(Some(0.5)).key("c"),
    ]));
    assert_eq!(part(&t, "c", Piece::Widget).label(), Some(&*crate::t!("kit-progress")), "a job nobody named is called Progress");
    let a = part(&t, "a", Piece::Widget);
    assert_eq!((a.role(), a.label(), a.numeric_value(), a.is_busy()), (Role::ProgressIndicator, Some("Copying"), Some(0.25), false));
    let b = part(&t, "b", Piece::Widget);
    assert!(b.is_busy() && b.numeric_value().is_none());
}

#[test]
fn a_tree_view_tells_its_rows_level_and_which_are_open() {
    use super::super::super::widget::{tree, tree_row};
    let rows = vec![tree_row(item("Documents"), 0).branch(true), tree_row(item("Letters"), 1).branch(false), tree_row(item("Notes"), 1)];
    let (mut ui, t) = told(column(vec![tree(rows, Some(1), Msg::Pick, Msg::Open, Msg::Pick).key("tree")]));
    assert_eq!(part(&t, "tree", Piece::Widget).role(), Role::Tree);
    let told = |r| { let p = part(&t, "tree", Piece::Row(r)); (p.role(), p.level(), p.is_expanded()) };
    assert_eq!(told(0), (Role::TreeItem, Some(1), Some(true)));
    assert_eq!(told(1), (Role::TreeItem, Some(2), Some(false)));
    assert_eq!(told(2), (Role::TreeItem, Some(2), None));
    assert_eq!(part(&t, "tree", Piece::Row(1)).position_in_set(), Some(1));
    assert_eq!(ui.act(&ask(part_id("tree", Piece::Row(1)), Action::Expand, None)).messages, vec![Msg::Open(1)]);
    assert_eq!(ui.act(&ask(part_id("tree", Piece::Row(0)), Action::Collapse, None)).messages, vec![Msg::Open(0)]);
    assert!(ui.act(&ask(part_id("tree", Piece::Row(0)), Action::Expand, None)).messages.is_empty());
}

#[test]
fn rows_choices_and_menu_items_know_their_place_in_the_set() {
    use super::super::super::widget::{drawing, SaidRow};
    let (_, t) = told(column(vec![
        list((0..5).map(|i| item(format!("Row {i}"))).collect(), Some(1), Msg::Pick).key("list"),
        choice(vec!["One".into(), "Two".into(), "Three".into()], 2, "Pick", Msg::Pick).key("pick"),
        drawing((100.0, 60.0), "", |_, _| {}).called("Menu").key("menu").menu_rows(vec![
            SaidRow { words: "Copy".into(), top: 0.0, height: 20.0, lit: false, check: None, radio: false },
            SaidRow { words: "Paste".into(), top: 20.0, height: 20.0, lit: true, check: None, radio: false },
        ]),
    ]));
    assert_eq!(part(&t, "list", Piece::Widget).size_of_set(), Some(5));
    assert_eq!(part(&t, "list", Piece::Row(3)).position_in_set(), Some(3));
    assert_eq!(part(&t, "pick", Piece::Widget).size_of_set(), Some(3));
    assert_eq!(part(&t, "pick", Piece::Row(2)).position_in_set(), Some(2));
    let menu = part(&t, "menu", Piece::Widget);
    assert_eq!((menu.role(), menu.size_of_set()), (Role::Menu, Some(2)));
    let paste = part(&t, "menu", Piece::Item(1));
    assert_eq!((paste.role(), paste.label(), paste.position_in_set(), paste.is_selected()), (Role::MenuItem, Some("Paste"), Some(1), Some(true)));
    assert_eq!(t.focus, part_id("menu", Piece::Item(1)));
}

#[test]
fn a_painted_tab_bar_is_a_tab_list_and_a_click_reaches_the_drawing() {
    use super::super::super::tabs::{Tab, Tabs};
    use super::super::super::widget::drawing;
    let mut tabs = Tabs::new(vec![Tab::new("One"), Tab::new("Two").count(3), Tab::new("Three").fixed()]);
    tabs.chosen = 1;
    let node = || drawing((600.0, 36.0), "Tabs", |_, _| {}).key("bar").tab_list(tabs.said(600.0))
        .on_pointer(|p| match p { w::Pointer::Down(x, _) => Msg::Pick(x as usize), w::Pointer::Up(x, _) => Msg::Pick(x as usize + 1000), _ => Msg::Pick(0) });
    let (mut ui, t) = told(node());
    let bar = part(&t, "bar", Piece::Widget);
    assert_eq!((bar.role(), bar.size_of_set()), (Role::TabList, Some(3)));
    let two = part(&t, "bar", Piece::Tab(1));
    assert_eq!((two.role(), two.label(), two.is_selected(), two.position_in_set()), (Role::Tab, Some("Two, 3"), Some(true), Some(1)));
    let close = part(&t, "bar", Piece::Close(1));
    assert_eq!((close.role(), close.label()), (Role::Button, Some("Close")));
    assert!(!t.nodes.iter().any(|(id, _)| *id == part_id("bar", Piece::Close(2))));
    let reply = ui.act(&ask(part_id("bar", Piece::Tab(2)), Action::Click, None));
    assert_eq!(reply.messages, vec![Msg::Pick(410), Msg::Pick(1410)]);
}

#[test]
fn a_painted_table_is_a_table_of_rows_and_cells() {
    use super::super::super::widget::drawing;
    use super::super::super::table::{Column, Table};
    let mut table = Table::new(vec![Column::new("Name", 200.0).sortable(), Column::new("Size", 80.0).end()]);
    table.set_rows(50);
    table.chosen = Some(1);
    table.sort = Some((0, false));
    let said = table.said(32.0 + 36.0 * 3.0, |r, c| format!("r{r}c{c}"));
    let (_, t) = told(drawing((300.0, 140.0), "Files", |_, _| {}).key("files").table_rows(said));
    let whole = part(&t, "files", Piece::Widget);
    assert_eq!((whole.role(), whole.row_count(), whole.column_count()), (Role::Table, Some(51), Some(2)));
    let head = part(&t, "files", Piece::Heading(0));
    assert_eq!((head.role(), head.label(), head.sort_direction()), (Role::ColumnHeader, Some("Name"), Some(accesskit::SortDirection::Descending)));
    assert_eq!(part(&t, "files", Piece::Heading(1)).sort_direction(), None);
    let row = part(&t, "files", Piece::TableRow(1));
    assert_eq!((row.role(), row.row_index(), row.is_selected()), (Role::Row, Some(2), Some(true)));
    let cell = part(&t, "files", Piece::Cell(2, 1));
    assert_eq!((cell.role(), cell.label(), cell.column_index(), cell.row_index()), (Role::Cell, Some("r2c1"), Some(1), Some(3)));
    assert!(!t.nodes.iter().any(|(id, _)| *id == part_id("files", Piece::TableRow(10))));
    assert_eq!(whole.active_descendant(), None);
    table.step_column(1);
    let said = table.said(32.0 + 36.0 * 3.0, |r, c| format!("r{r}c{c}"));
    let (_, t) = told(drawing((300.0, 140.0), "Files", |_, _| {}).key("files").table_rows(said));
    assert_eq!(part(&t, "files", Piece::Widget).active_descendant(), Some(part_id("files", Piece::Cell(1, 0))), "the reader is on the keys' cell");
    table.chosen = None;
    let said = table.said(32.0 + 36.0 * 3.0, |r, c| format!("r{r}c{c}"));
    let (_, t) = told(drawing((300.0, 140.0), "Files", |_, _| {}).key("files").table_rows(said));
    assert_eq!(part(&t, "files", Piece::Widget).active_descendant(), Some(part_id("files", Piece::Heading(0))));
}

#[test]
fn a_tree_table_tells_levels_openness_places_and_can_be_opened_by_a_reader() {
    use super::super::super::widget::drawing;
    use super::super::super::table::{Branch, Column, Table};
    let mut table = Table::new(vec![Column::new("Name", 200.0)]);
    table.set_tree(vec![Branch::opened(0), Branch::closed(1).depth(1), Branch::leaf(2).depth(1), Branch::leaf(3)]);
    table.multiple = true;
    table.set_picked([1, 2]);
    table.chosen = Some(2);
    let said = table.said(400.0, |r, _| format!("r{r}"));
    let node = drawing((300.0, 300.0), "Files", |_, _| {}).key("files").table_rows(said)
        .on_pointer_mods(|p, _| Msg::Point(p)).on_keys(|i| match i { Input::Key { sym, .. } => Some(Msg::Typed(*sym)), _ => None });
    let (mut ui, t) = told(node);
    let whole = part(&t, "files", Piece::Widget);
    assert_eq!(whole.role(), Role::TreeGrid);
    assert!(whole.is_multiselectable());
    assert_eq!(whole.active_descendant(), None);
    let row = |i| part(&t, "files", Piece::TableRow(i));
    assert_eq!((row(0).level(), row(0).is_expanded(), row(0).position_in_set(), row(0).size_of_set()), (Some(1), Some(true), Some(0), Some(2)));
    assert_eq!((row(1).level(), row(1).is_expanded(), row(1).position_in_set(), row(1).size_of_set()), (Some(2), Some(false), Some(0), Some(2)));
    assert_eq!((row(2).level(), row(2).is_expanded(), row(2).position_in_set()), (Some(2), None, Some(1)));
    assert_eq!((row(1).is_selected(), row(2).is_selected(), row(3).is_selected()), (Some(true), Some(true), Some(false)), "every chosen row is told");
    let reply = ui.act(&ask(part_id("files", Piece::TableRow(1)), Action::Expand, None));
    assert_eq!(reply.messages.len(), 3, "a press and a release on the row, then the Right key");
    assert_eq!(reply.messages[2], Msg::Typed(Keysym::Right));
    let reply = ui.act(&ask(part_id("files", Piece::TableRow(0)), Action::Expand, None));
    assert!(reply.messages.is_empty(), "an open row is not opened again");
}

#[test]
fn a_drawing_tells_its_lines_caret_and_selection() {
    use super::super::super::widget::{drawing, Reading};
    let long = "x".repeat(RUN + 5);
    let (mut ui, t) = told(drawing((400.0, 100.0), "Terminal", |_, _| {}).key("term").reads(Reading {
        lines: vec!["$ ls".into(), String::new(), long.clone()],
        caret: Some((2, RUN + 2)),
        selection: Some(((0, 2), (2, RUN + 2))),
        editable: false,
    }));
    let term = part(&t, "term", Piece::Widget);
    assert_eq!((term.role(), term.label()), (Role::Terminal, Some("Terminal")));
    assert_eq!(part(&t, "term", Piece::Line(0, 0)).value(), Some("$ ls"));
    assert_eq!(part(&t, "term", Piece::Line(1, 0)).value(), Some(""));
    assert_eq!(part(&t, "term", Piece::Line(2, 1)).value().map(str::len), Some(5));
    assert_eq!(part(&t, "term", Piece::Line(2, 0)).next_on_line(), Some(part_id("term", Piece::Line(2, 1))));
    let sel = term.text_selection().unwrap();
    assert_eq!((sel.anchor.node, sel.anchor.character_index), (part_id("term", Piece::Line(0, 0)), 2));
    assert_eq!((sel.focus.node, sel.focus.character_index), (part_id("term", Piece::Line(2, 1)), 2));
    assert_eq!(term.children().len(), 4);
    assert!(ui.kept_from_a_reader().is_empty());
    assert!(ui.unnamed_controls().is_empty());
}

#[test]
fn a_dialog_tells_its_title_and_message() {
    let (_, t) = told(column(vec![
        label("Delete 3 files?").dialog_message().key("msg"),
        button("x", "Delete").on_press(Msg::Save),
    ]));
    let win = &t.nodes[0].1;
    assert_eq!((win.role(), win.label(), win.description()), (Role::Dialog, Some("Window"), Some("Delete 3 files?")));
    let (_, t) = told(column(vec![label("This cannot be undone").alert_message()]));
    assert_eq!(t.nodes[0].1.role(), Role::AlertDialog);
    let (_, t) = told(column(vec![label("Hello")]));
    assert_eq!(t.nodes[0].1.role(), Role::Window);
}

#[test]
fn controls_with_no_name_or_role_are_found() {
    use super::super::super::widget::drawing;
    let (mut ui, _) = told(column(vec![
        button("x", "").key("bare"),
        drawing((10.0, 10.0), "", |_, _| {}).pressable(Msg::Save).key("pic"),
        button("x", "Fine").key("fine"),
    ]));
    let wrong = ui.unnamed_controls();
    assert_eq!(wrong.len(), 2, "{wrong:?}");
    assert!(wrong.iter().all(|w| w.contains("has no name")));
    let (mut ui, _) = told(column(vec![button("x", "Fine").key("fine"), note_like()]));
    assert!(ui.unnamed_controls().is_empty());
}

#[test]
fn a_sheet_that_only_catches_input_is_not_told() {
    use super::super::super::widget::drawing;
    let (mut ui, t) = told(column(vec![
        drawing((10.0, 10.0), "", |_, _| {}).on_keys(|_| None).unseen().key("catcher"),
        button("x", "Fine").key("fine"),
    ]));
    assert!(part(&t, "catcher", Piece::Widget).is_hidden());
    assert!(ui.unnamed_controls().is_empty());
    let (mut ui, _) = told(column(vec![drawing((10.0, 10.0), "", |_, _| {}).on_keys(|_| None).key("bare")]));
    assert_eq!(ui.unnamed_controls().len(), 1);
}

fn now() -> std::time::Instant {
    std::time::Instant::now()
}

#[test]
fn the_toolkits_own_menu_is_told_with_its_lit_row_and_worked() {
    use super::super::super::input::{Button, Input};
    let mut ui = Ui::new(column(vec![label("Some words").key("w"), entry("Ann", "Name", Msg::Name).key("name")]).padding(10.0), Theme::new(false, "#fbc711"));
    ui.size = (400, 300);
    ui.settle();
    let r = ui.spot("w").unwrap().rect;
    let (x, y) = (r.x + 4.0, r.y + r.h / 2.0);
    ui.input(&Input::PointerIn { x, y }, now());
    ui.input(&Input::Press { button: Button::Right, x, y }, now());
    let t = ui.access("");
    let menu = t.nodes.iter().find(|(_, p)| p.role() == Role::Menu).expect("the menu is told").1.clone();
    let items: Vec<&Part> = t.nodes.iter().filter(|(_, p)| p.role() == Role::MenuItem).map(|(_, p)| p).collect();
    assert_eq!((menu.children().len(), items.len()), (2, 2));
    assert_eq!(items[0].label(), Some("Copy, Ctrl+C"));
    assert!(t.nodes[0].1.children().contains(&part_id(COPY_MENU, Piece::Widget)));
    assert!(ui.kept_from_a_reader().is_empty());
    ui.close_menu();
    ui.focus = Some("name".into());
    assert!(ui.menu_from_key());
    let t = ui.access("");
    assert_eq!(t.focus, part_id(COPY_MENU, Piece::Item(0)));
    let reply = ui.act(&ask(part_id(COPY_MENU, Piece::Item(1)), Action::Click, None));
    assert!(reply.redraw && !ui.menu_open());
    assert!(ui.access("").nodes.iter().all(|(_, p)| p.role() != Role::Menu));
}

#[test]
fn an_open_dropdown_list_is_told_as_options_with_the_lit_one_focused() {
    use super::super::super::widget::dropdown;
    let names: Vec<String> = (0..40).map(|i| format!("Choice {i}")).collect();
    let mut ui = Ui::new(column(vec![dropdown(names, Some(3), "Zone", Msg::Pick).key("zone")]).padding(10.0), Theme::new(false, "#fbc711"));
    ui.size = (400, 400);
    ui.settle();
    assert!(ui.access("").nodes.iter().all(|(_, p)| p.role() != Role::ListBox));
    ui.open_drop("zone");
    let t = ui.access("");
    let list = part(&t, "zone", Piece::Listing);
    assert_eq!((list.role(), list.children().len(), list.size_of_set()), (Role::ListBox, 40, Some(40)));
    assert_eq!(list.label(), Some("Zone"), "the open list is called by the dropdown's name");
    assert_eq!(part(&t, "zone", Piece::Widget).children(), &[part_id("zone", Piece::Listing)]);
    let three = part(&t, "zone", Piece::Row(3));
    assert_eq!((three.label(), three.is_selected(), three.position_in_set()), (Some("Choice 3"), Some(true), Some(3)));
    assert_eq!(t.focus, part_id("zone", Piece::Row(3)));
    assert!(ui.kept_from_a_reader().is_empty());
    let reply = ui.act(&ask(part_id("zone", Piece::Row(7)), Action::Click, None));
    assert_eq!(reply.messages, vec![Msg::Pick(7)]);
    assert!(!ui.dropdown_open());
}

#[test]
fn a_greyed_widget_refuses_what_a_screen_reader_sets() {
    let mut ui = Ui::new(column(vec![
        slider(0.5, 0.0, 1.0, 0.1, "Volume", Msg::Volume).off(true).key("volume"),
        entry("Ann", "Name", Msg::Name).off(true).key("name"),
        list((0..3).map(|i| item(format!("Row {i}"))).collect(), None, Msg::Pick).off(true).key("list"),
    ]), Theme::new(false, "#fbc711"));
    ui.size = (400, 400);
    ui.access("");
    let at = |id: &str| part_id(id, Piece::Widget);
    assert!(ui.act(&ask(at("volume"), Action::Increment, None)).messages.is_empty());
    assert!(ui.act(&ask(at("name"), Action::SetValue, Some(ActionData::Value("Bob".into())))).messages.is_empty());
    assert!(ui.act(&ask(part_id("list", Piece::Row(1)), Action::Click, None)).messages.is_empty());
}

#[test]
fn a_slider_ignores_a_value_that_is_not_a_number() {
    let mut ui = ui("Ann");
    ui.access("");
    let volume = part_id("volume", Piece::Widget);
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(ui.act(&ask(volume, Action::SetValue, Some(ActionData::NumericValue(bad)))).messages.is_empty());
    }
    for bad in ["NaN", "inf", "-inf"] {
        assert!(ui.act(&ask(volume, Action::SetValue, Some(ActionData::Value(bad.into())))).messages.is_empty());
    }
}

#[test]
fn a_chosen_row_past_the_end_is_not_the_focus_and_a_long_list_is_capped() {
    let rows: Vec<_> = (0..MOST_LINES * 4).map(|i| item(format!("Row {i}"))).collect();
    let mut ui = Ui::new(column(vec![list(rows, Some(MOST_LINES * 4 + 9), Msg::Pick).key("big")]), Theme::new(false, "#fbc711"));
    ui.size = (400, 400);
    ui.focus = Some("big".into());
    let t = ui.access("");
    assert_eq!(t.focus, part_id("big", Piece::Widget));
    assert_eq!(part(&t, "big", Piece::Widget).children().len(), MOST_LINES);
    assert_eq!(part(&t, "big", Piece::Widget).size_of_set(), Some(MOST_LINES * 4));
}

#[test]
fn a_long_scrollback_keeps_the_lines_round_the_caret() {
    use super::super::super::widget::{drawing, Reading};
    let lines: Vec<String> = (0..MOST_LINES * 2).map(|i| format!("line {i}")).collect();
    let last = lines.len() - 1;
    let (_, t) = told(drawing((400.0, 100.0), "Terminal", |_, _| {}).key("term").reads(Reading {
        lines, caret: Some((last, 3)), selection: None, editable: false,
    }));
    let term = part(&t, "term", Piece::Widget);
    assert_eq!(term.children().len(), MOST_LINES);
    let sel = term.text_selection().unwrap();
    assert_eq!(sel.focus.node, part_id("term", Piece::Line(last, 0)));
    assert_eq!(part(&t, "term", Piece::Line(last, 0)).value(), Some(format!("line {last}").as_str()));
}

#[test]
fn red_words_beside_a_box_are_linked_to_it_and_headings_have_levels() {
    use super::super::super::widget::{heading, note, section, Tone};
    let (_, t) = told(column(vec![
        title("Page"), heading("Part"), section("Bit"),
        entry("", "Name", Msg::Name).key("name"),
        note("Needs a name").tone(Tone::Problem).key("why"),
        entry("x", "Other", Msg::Name).key("other"),
    ]));
    let name = part(&t, "name", Piece::Widget);
    assert_eq!((name.invalid(), name.error_message()), (Some(Invalid::True), Some(part_id("why", Piece::Widget))));
    assert_eq!(part(&t, "other", Piece::Widget).invalid(), None);
    let levels: Vec<_> = t.nodes.iter().filter(|(_, p)| p.role() == Role::Heading).filter_map(|(_, p)| p.level()).collect();
    assert_eq!(levels, vec![2, 3, 4]);
}

#[test]
fn a_box_with_words_the_spell_checker_does_not_know_is_told_as_misspelt() {
    struct Knows;
    impl crate::spelling::Speller for Knows {
        fn known(&self, w: &str) -> bool {
            w == "fine"
        }
    }
    let (_, t) = told(column(vec![
        entry("wrod", "Note", Msg::Name).spelling(&Knows).key("note"),
        entry("fine", "Other", Msg::Name).spelling(&Knows).key("other"),
    ]));
    assert_eq!(part(&t, "note", Piece::Widget).invalid(), Some(Invalid::Spelling));
    assert_eq!(part(&t, "other", Piece::Widget).invalid(), None);
}

fn note_like() -> Node<Msg> {
    label("Plain words")
}
