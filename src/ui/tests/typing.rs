//! Tests of text boxes and selectable words: typing, the clipboard and input methods.

use super::*;

#[derive(Default)]
struct Form {
    name: String,
    pass: String,
    sent: u32,
}

impl App for Form {
    type Msg = Msg;
    fn update(&mut self, msg: Msg) {
        match msg {
            Msg::Name(t) => self.name = t,
            Msg::Pass(t) => self.pass = t,
            Msg::Go => self.sent += 1,
            _ => {}
        }
    }
    fn view(&self) -> Node<Msg> {
        column(vec![
            entry(self.name.clone(), "Name", Msg::Name).key("name").on_submit(Msg::Go),
            entry(self.pass.clone(), "Password", Msg::Pass).key("pass").secret(),
        ]).padding(10.0)
    }
}

fn feed(ui: &mut Ui<Msg>, form: &mut Form, input: &Input) -> Reply<Msg> {
    let reply = ui.input(input, Instant::now());
    for m in &reply.messages {
        form.update(m.clone());
    }
    ui.set_view(form.view());
    reply
}

fn form_ui(form: &Form) -> Ui<Msg> {
    let mut ui = Ui::new(form.view(), Theme::new(false, "#fbc711"));
    ui.size = (400, 300);
    ui
}

#[test]
fn a_text_box_types_selects_copies_and_undoes() {
    let mut form = Form::default();
    let mut ui = form_ui(&form);
    feed(&mut ui, &mut form, &key(Keysym::Tab, false));
    assert_eq!(ui.focus.as_deref(), Some("name"));
    for t in ["h", "i", " ", "y", "o", "u"] {
        feed(&mut ui, &mut form, &typed(t));
    }
    assert_eq!(form.name, "hi you");
    for _ in 0..3 {
        feed(&mut ui, &mut form, &key(Keysym::Left, true));
    }
    assert_eq!(feed(&mut ui, &mut form, &ctrl(Keysym::c)).copy.as_deref(), Some("you"));
    feed(&mut ui, &mut form, &typed("me"));
    assert_eq!(form.name, "hi me");
    feed(&mut ui, &mut form, &ctrl(Keysym::z));
    assert_eq!(form.name, "hi you");
    feed(&mut ui, &mut form, &key(Keysym::End, false));
    feed(&mut ui, &mut form, &Input::Key { sym: Keysym::BackSpace, text: None, mods: Mods { ctrl: true, ..Mods::default() }, repeat: false });
    assert_eq!(form.name, "hi ");
    assert_eq!(feed(&mut ui, &mut form, &key(Keysym::Return, false)).messages, vec![Msg::Go]);
    feed(&mut ui, &mut form, &ctrl(Keysym::a));
    assert_eq!(feed(&mut ui, &mut form, &ctrl(Keysym::x)).copy.as_deref(), Some("hi "));
    assert_eq!(form.name, "");
    assert!(feed(&mut ui, &mut form, &ctrl(Keysym::v)).paste);
    feed(&mut ui, &mut form, &Input::Key { sym: Keysym::q, text: Some("q".into()), mods: Mods { ctrl: true, ..Mods::default() }, repeat: false });
    assert_eq!(form.name, "");
}

#[test]
fn a_password_never_leaves_its_box() {
    let mut form = Form::default();
    let mut ui = form_ui(&form);
    ui.focus = Some("pass".into());
    for t in ["s", "3", "c", "r", "3", "t"] {
        feed(&mut ui, &mut form, &typed(t));
    }
    feed(&mut ui, &mut form, &ctrl(Keysym::a));
    assert!(feed(&mut ui, &mut form, &ctrl(Keysym::c)).copy.is_none());
    assert!(feed(&mut ui, &mut form, &ctrl(Keysym::x)).copy.is_none());
    assert_eq!(form.pass, "s3cr3t");
    let reply = ui.paste("new\npass\n");
    for m in &reply.messages {
        form.update(m.clone());
    }
    assert_eq!(form.pass, "new pass");
}

fn composed(delete: (u32, u32), commit: &str, preedit: &str, caret: Option<(usize, usize)>) -> Input {
    Input::Composed { delete, commit: commit.into(), preedit: preedit.into(), caret }
}

#[test]
fn an_input_method_types_its_words_and_shows_the_one_being_made() {
    let mut form = Form::default();
    let mut ui = form_ui(&form);
    assert_eq!(ui.typing(), None);
    feed(&mut ui, &mut form, &key(Keysym::Tab, false));
    for t in ["h", "i", " "] {
        feed(&mut ui, &mut form, &typed(t));
    }
    let mut plain = tiny_skia::Pixmap::new(400, 300).unwrap();
    ui.paint(&mut plain.as_mut(), 1.0);
    feed(&mut ui, &mut form, &composed((0, 0), "", "にほ", Some((6, 6))));
    assert_eq!(form.name, "hi ");
    let mut making = tiny_skia::Pixmap::new(400, 300).unwrap();
    ui.paint(&mut making.as_mut(), 1.0);
    assert_ne!(plain.data(), making.data());
    let t = ui.typing().unwrap();
    assert_eq!((t.id.as_str(), t.text.as_str(), t.caret, t.anchor, t.secret), ("name", "hi ", 3, 3, false));
    let r = ui.spot("name").unwrap().rect;
    assert!(t.area.0 as f32 > r.x && (t.area.0 as f32) < r.x + r.w && t.area.1 as f32 >= r.y && t.area.3 > 0, "{:?}", t.area);
    feed(&mut ui, &mut form, &composed((0, 0), "日本", "", None));
    assert_eq!(form.name, "hi 日本");
    assert!(ui.preedit.is_none());
    let mut done = tiny_skia::Pixmap::new(400, 300).unwrap();
    ui.paint(&mut done.as_mut(), 1.0);
    assert_ne!(done.data(), making.data());
    feed(&mut ui, &mut form, &composed((6, 0), "にっぽん", "", None));
    assert_eq!(form.name, "hi にっぽん");
    feed(&mut ui, &mut form, &composed((4, 0), "", "", None));
    assert_eq!(form.name, "hi にっ");
    feed(&mut ui, &mut form, &key(Keysym::Left, false));
    feed(&mut ui, &mut form, &composed((0, 3), "", "", None));
    assert_eq!(form.name, "hi に");
    feed(&mut ui, &mut form, &ctrl(Keysym::z));
    assert_eq!(form.name, "hi にっ");
    feed(&mut ui, &mut form, &key(Keysym::Tab, false));
    assert!(ui.typing().unwrap().secret);
    feed(&mut ui, &mut form, &Input::Focus(false));
    assert_eq!(ui.typing(), None);
}

#[test]
fn clicks_place_the_caret_and_pick_words() {
    let mut form = Form { name: "hello world".into(), ..Form::default() };
    let mut ui = form_ui(&form);
    ui.settle();
    let r = ui.spot("name").unwrap().rect;
    let (x, y) = (r.x + crate::theme::sheet().entry_padding + 4.0, r.y + r.h / 2.0);
    let now = Instant::now();
    ui.input(&Input::PointerIn { x, y }, now);
    let click = |ui: &mut Ui<Msg>, at: Instant| {
        ui.input(&Input::Press { button: Button::Left, x, y }, at);
        ui.input(&Input::Release { button: Button::Left, x, y }, at);
    };
    click(&mut ui, now);
    assert_eq!(ui.focus.as_deref(), Some("name"));
    assert_eq!(ui.cursor(), Cursor::Text);
    let caret = ui.edits["name"].caret;
    assert!(caret <= 1, "caret at {caret}");
    click(&mut ui, now + Duration::from_millis(100));
    assert_eq!(ui.edits["name"].selected(), "hello");
    click(&mut ui, now + Duration::from_millis(200));
    assert_eq!(ui.edits["name"].selected(), "hello world");
    click(&mut ui, now + Duration::from_secs(2));
    assert_eq!(ui.edits["name"].selected(), "");
    ui.input(&Input::Press { button: Button::Left, x, y }, now + Duration::from_secs(4));
    ui.input(&Input::PointerMove { x: r.x + r.w - 2.0, y }, now + Duration::from_secs(4));
    assert_eq!(ui.edits["name"].selected().trim_start_matches('h'), "ello world");
    feed(&mut ui, &mut form, &Input::Release { button: Button::Left, x: r.x + r.w - 2.0, y });
}

#[test]
fn words_that_pick_select_by_clicks_and_copy() {
    let view = || column(vec![label("hello world").key("words"), label("plain words").key("plain")]).padding(10.0);
    let mut ui: Ui<Msg> = Ui::new(view(), Theme::new(false, "#fbc711"));
    ui.size = (400, 300);
    ui.settle();
    let r = ui.spot("words").unwrap().rect;
    let (x, y) = (r.x + 4.0, r.y + r.h / 2.0);
    let now = Instant::now();
    ui.input(&Input::PointerIn { x, y }, now);
    assert_eq!(ui.cursor(), Cursor::Text);
    let click = |ui: &mut Ui<Msg>, at: Instant| {
        ui.input(&Input::Press { button: Button::Left, x, y }, at);
        ui.input(&Input::Release { button: Button::Left, x, y }, at);
    };
    let copied = |ui: &mut Ui<Msg>| ui.input(&ctrl(Keysym::c), now).copy;
    click(&mut ui, now);
    assert_eq!(copied(&mut ui), None, "a single click selects nothing");
    click(&mut ui, now + Duration::from_millis(100));
    assert_eq!(copied(&mut ui).as_deref(), Some("hello"));
    click(&mut ui, now + Duration::from_millis(200));
    assert_eq!(copied(&mut ui).as_deref(), Some("hello world"));
    let later = now + Duration::from_secs(4);
    ui.input(&Input::Press { button: Button::Left, x, y }, later);
    ui.input(&Input::PointerMove { x: r.x + r.w, y }, later);
    ui.input(&Input::Release { button: Button::Left, x: r.x + r.w, y }, later);
    ui.set_view(view());
    ui.settle();
    assert_eq!(copied(&mut ui).as_deref().map(|c| c.trim_start_matches('h')), Some("ello world"));
    let mut picked = tiny_skia::Pixmap::new(400, 300).unwrap();
    ui.paint(&mut picked.as_mut(), 1.0);
    let p = ui.spot("plain").unwrap().rect;
    ui.input(&Input::Press { button: Button::Left, x: p.x + 4.0, y: p.y + 4.0 }, later);
    ui.input(&Input::Release { button: Button::Left, x: p.x + 4.0, y: p.y + 4.0 }, later);
    assert_eq!(copied(&mut ui), None);
    let mut plain = tiny_skia::Pixmap::new(400, 300).unwrap();
    ui.paint(&mut plain.as_mut(), 1.0);
    assert_ne!(picked.data(), plain.data(), "no band once let go");
}

#[test]
fn cut_words_keep_to_one_line_with_an_ellipsis() {
    let long = "a name far too long to fit on one line of this narrow window";
    let laid = |words: Node<Msg>| {
        let mut ui: Ui<Msg> = Ui::new(column(vec![words.key("n")]).padding(10.0), Theme::new(false, "#fbc711"));
        ui.size = (140, 200);
        ui.settle();
        let mut pixmap = tiny_skia::Pixmap::new(140, 200).unwrap();
        ui.paint(&mut pixmap.as_mut(), 1.0);
        (ui.spot("n").unwrap().rect, pixmap)
    };
    let (wrapped, _) = laid(label(long));
    let (cut, pixmap) = laid(label(long).cut());
    assert!(cut.h < wrapped.h / 2.0, "one line: {} against {}", cut.h, wrapped.h);
    assert!(cut.x + cut.w <= 130.0, "within the room: {cut:?}");
    let clear = pixmap.pixel(139, 199).unwrap();
    for y in cut.y as u32..(cut.y + cut.h) as u32 {
        for x in 131..140 {
            assert_eq!(pixmap.pixel(x, y).unwrap(), clear, "ink at {x},{y}");
        }
    }
    let (short, _) = laid(label("Short").cut());
    assert!(short.w < 100.0);
    let mut ui: Ui<Msg> = Ui::new(column(vec![label(long).cut().key("n"), label("Short").cut().key("s")]).padding(10.0),
                                  Theme::new(false, "#fbc711"));
    ui.size = (140, 200);
    ui.settle();
    let (n, sh) = (ui.spot("n").unwrap().rect, ui.spot("s").unwrap().rect);
    assert_eq!(ui.cut_short("n"), Some(long));
    assert_eq!(ui.cut_short("s"), None);
    assert_eq!(ui.hit(n.x + 2.0, n.y + 2.0).as_deref(), Some("n"));
    assert_eq!(ui.hit(sh.x + 2.0, sh.y + 2.0), None);
}

#[test]
fn a_long_line_slides_to_keep_the_caret_in_view() {
    let form = Form { name: "a line far too long to fit in a text box of this size at all".into(), ..Form::default() };
    let mut ui = form_ui(&form);
    ui.size = (200, 100);
    ui.focus = Some("name".into());
    let mut pixmap = tiny_skia::Pixmap::new(400, 200).unwrap();
    ui.paint(&mut pixmap.as_mut(), 2.0);
    assert!(ui.edits["name"].scroll > 0.0);
    ui.input(&key(Keysym::Home, false), Instant::now());
    ui.paint(&mut pixmap.as_mut(), 2.0);
    assert_eq!(ui.edits["name"].scroll, 0.0);
    let r = ui.spot("name").unwrap().rect;
    let t = Theme::new(false, "#fbc711");
    let px = pixmap.pixel((r.x * 2.0 + 6.0) as u32, ((r.y + r.h - 4.0) * 2.0) as u32).unwrap();
    assert_eq!(px.red(), (t.field.r * 255.0).round() as u8);
}

#[test]
#[ignore]
fn a_text_box_answers_a_real_keyboard_and_clipboard() {
    use crate::window::tests::Hands;
    let events = Loop::new();
    let display = Display::connect(&events, || panic!("compositor gone")).expect("connected");
    let shown = Shown::open(&events, &display, Kind::Window { title: "Form".into(), app_id: "org.trindle.Test".into() },
                            (420, 160), Form::default()).expect("a window");
    let Hands { conn, keys, queue: _queue, .. } = Hands::lend();
    let keys = keys.expect("a stand-in keyboard");
    let presses = [15, 35, 23, 1030, 1046, 107, 1047, 28];
    let at = Cell::new(0usize);
    let steps = events.clone();
    events.every(Duration::from_millis(250), move || {
        let i = at.get();
        at.set(i + 1);
        if let Some(&key) = i.checked_sub(4).and_then(|n| presses.get(n)) {
            let held = key > 1000;
            if held {
                keys.modifiers(4, 0, 0, 0);
            }
            keys.key(i as u32 * 250, key % 1000, 1);
            keys.key(i as u32 * 250 + 10, key % 1000, 0);
            if held {
                keys.modifiers(0, 0, 0, 0);
            }
        }
        let _ = conn.flush();
        if i == 4 + presses.len() + 3 {
            steps.quit();
            return crate::event::Flow::Stop;
        }
        crate::event::Flow::Continue
    });
    events.run();
    let (name, sent) = shown.with(|f| (f.name.clone(), f.sent));
    eprintln!("name {name:?}, sent {sent}");
    assert_eq!((name.as_str(), sent), ("hihi", 1));
    let (w, h) = (840, 320);
    let mut pixmap = tiny_skia::Pixmap::new(w, h).unwrap();
    shown.held.borrow_mut().ui.paint(&mut pixmap.as_mut(), 2.0);
    let file = std::env::temp_dir().join(format!("trindle-form-{w}x{h}.rgba"));
    std::fs::write(&file, pixmap.data()).unwrap();
    eprintln!("painted to {}", file.display());
}

#[derive(Default)]
struct Method {
    active: bool,
    done: u32,
    around: Option<String>,
    password: bool,
}

mod method {
    use wayland_client::{protocol::{wl_registry, wl_seat}, Connection, Dispatch, Proxy, QueueHandle, WEnum};
    use wayland_protocols::wp::text_input::zv3::client::zwp_text_input_v3::ContentPurpose;
    use wayland_protocols_misc::zwp_input_method_v2::client::{
        zwp_input_method_manager_v2::ZwpInputMethodManagerV2, zwp_input_method_v2::{self, ZwpInputMethodV2},
    };

    use super::Method;

    impl Dispatch<ZwpInputMethodV2, ()> for Method {
        fn event(m: &mut Self, _: &ZwpInputMethodV2, event: zwp_input_method_v2::Event, _: &(), _: &Connection,
                 _: &QueueHandle<Self>) {
            use zwp_input_method_v2::Event;
            match event {
                Event::Activate => m.active = true,
                Event::Deactivate => m.active = false,
                Event::SurroundingText { text, .. } => m.around = Some(text),
                Event::ContentType { purpose, .. } => m.password = purpose == WEnum::Value(ContentPurpose::Password),
                Event::Done => m.done += 1,
                _ => {}
            }
        }
    }

    macro_rules! quiet {
        ($($t:ty),*) => {$(
            impl Dispatch<$t, ()> for Method {
                fn event(_: &mut Self, _: &$t, _: <$t as Proxy>::Event, _: &(), _: &Connection, _: &QueueHandle<Self>) {}
            }
        )*};
    }
    quiet!(wl_seat::WlSeat, ZwpInputMethodManagerV2);

    impl Dispatch<wl_registry::WlRegistry, wayland_client::globals::GlobalListContents> for Method {
        fn event(_: &mut Self, _: &wl_registry::WlRegistry, _: wl_registry::Event,
                 _: &wayland_client::globals::GlobalListContents, _: &Connection, _: &QueueHandle<Self>) {}
    }

    pub fn take(conn: &Connection) -> (wayland_client::EventQueue<Method>, ZwpInputMethodV2) {
        let (globals, queue) = wayland_client::globals::registry_queue_init::<Method>(conn).expect("globals");
        let qh = queue.handle();
        let seat: wl_seat::WlSeat = globals.bind(&qh, 1..=1, ()).expect("a seat");
        let manager: ZwpInputMethodManagerV2 = globals.bind(&qh, 1..=1, ()).expect("input methods");
        let method = manager.get_input_method(&seat, &qh, ());
        (queue, method)
    }
}

fn open_form(events: &Loop) -> Shown<Form> {
    let display = Display::connect(events, || panic!("compositor gone")).expect("connected");
    Shown::open(events, &display, Kind::Window { title: "Form".into(), app_id: "org.trindle.Test".into() },
                (420, 160), Form::default()).expect("a window")
}

fn the_box_holds_the_words(shown: &Shown<Form>, m: &Method, made: &Cell<String>) {
    let name = shown.with(|f| f.name.clone());
    eprintln!("while making {:?}, name {name:?}, told {:?}, password {}", made.take(), m.around, m.password);
    assert_eq!(name, "日本語");
    assert_eq!(m.around.as_deref(), Some(""));
    assert!(m.password);
}

#[test]
#[ignore]
fn a_text_box_takes_words_from_a_real_input_method() {
    use crate::window::tests::Hands;
    let events = Loop::new();
    let shown = open_form(&events);
    let Hands { conn, keys, queue: _queue, .. } = Hands::lend();
    let keys = keys.expect("a stand-in keyboard");
    let (mut queue, method) = method::take(&conn);
    let state = Rc::new(RefCell::new(Method::default()));
    let (at, step) = (Cell::new(0u32), Cell::new(0u32));
    let (steps, told) = (events.clone(), state.clone());
    let tab = move |at: u32| {
        keys.key(at * 250, 15, 1);
        keys.key(at * 250 + 10, 15, 0);
    };
    let made = Rc::new(Cell::new(String::new()));
    let saw = made.clone();
    let looked = shown.held.clone();
    events.every(Duration::from_millis(250), move || {
        let i = at.get();
        at.set(i + 1);
        queue.roundtrip(&mut *told.borrow_mut()).unwrap();
        let m = told.borrow();
        match step.get() {
            0 if i >= 4 => {
                tab(i);
                step.set(1);
            }
            1 if m.active && m.done > 0 => {
                method.set_preedit_string("にほ".into(), 6, 6);
                method.commit(m.done);
                step.set(2);
            }
            2 => {
                saw.set(looked.borrow().app.name.clone());
                method.commit_string("日本語".into());
                method.commit(m.done);
                step.set(3);
            }
            3 if m.around.as_deref() == Some("日本語") => {
                tab(i);
                step.set(4);
            }
            4 if m.password || i > 60 => step.set(5),
            5 => {
                steps.quit();
                return crate::event::Flow::Stop;
            }
            _ if i > 60 => step.set(5),
            _ => {}
        }
        drop(m);
        let _ = conn.flush();
        crate::event::Flow::Continue
    });
    events.run();
    the_box_holds_the_words(&shown, &state.borrow(), &made);
}
