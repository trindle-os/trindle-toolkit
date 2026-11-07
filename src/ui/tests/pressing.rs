//! Tests of the keyboard and the pointer: what a key or a click does to the widgets.

use super::*;

#[test]
fn tab_moves_the_keyboard_and_space_presses() {
    let mut ui = ui(false, 0.5);
    let now = Instant::now();
    ui.input(&key(Keysym::Tab, false), now);
    assert_eq!(ui.focus.as_deref(), Some("0.0"));
    assert_eq!(ui.input(&key(Keysym::space, false), now).messages, vec![Msg::Save]);
    ui.input(&key(Keysym::Tab, false), now);
    assert_eq!(ui.focus.as_deref(), Some("wifi"));
    assert_eq!(ui.input(&key(Keysym::Return, false), now).messages, vec![Msg::Wifi(true)]);
    ui.input(&key(Keysym::Tab, false), now);
    assert_eq!(ui.input(&key(Keysym::Right, false), now).messages, vec![Msg::Volume(0.6)]);
    assert_eq!(ui.input(&key(Keysym::End, false), now).messages, vec![Msg::Volume(1.0)]);
    ui.input(&key(Keysym::Tab, false), now);
    assert_eq!(ui.focus.as_deref(), Some("0.0"));
    ui.input(&key(Keysym::ISO_Left_Tab, true), now);
    assert_eq!(ui.focus.as_deref(), Some("2"));
    ui.focus = Some("wifi".into());
    ui.set_view(column(vec![button("a", "A").on_press(Msg::Save), switch(true, "Wi-Fi", Msg::Wifi).key("wifi")]));
    ui.input(&key(Keysym::Tab, false), now);
    assert_eq!(ui.focus.as_deref(), Some("0"));
}

#[test]
fn tab_to_a_named_button_shows_its_name_and_the_next_key_takes_it_away() {
    let mut ui = ui(false, 0.5);
    let now = Instant::now();
    ui.input(&key(Keysym::Tab, false), now);
    let id = ui.focus.clone().unwrap();
    let shown = |ui: &Ui<Msg>| ui.tip.as_ref().map(|t| (t.id.clone(), matches!(t.state, TipState::Shown(_))));
    assert_eq!(shown(&ui), Some((id, true)));
    let mut ui = ui_with_label_up();
    assert!(ui.input(&key(Keysym::Left, false), now).redraw);
    assert!(ui.tip.is_none());
}

fn ui_with_label_up() -> Ui<Msg> {
    let mut ui = ui(false, 0.5);
    ui.tip = Some(Tip { id: "wifi".into(), state: TipState::Shown(Instant::now()) });
    ui
}

#[test]
fn a_step_of_nothing_leaves_the_value_where_it_is() {
    assert_eq!(snap(0.4, 0.0, 1.0, 0.0), 0.4);
    assert_eq!(snap(2.0, 0.0, 1.0, -1.0), 1.0);
    assert_eq!(snap(0.4, 0.0, 1.0, f32::NAN), 0.4);
    assert_eq!(snap(0.37, 0.0, 1.0, 0.1), 0.4);
}

#[test]
fn a_row_is_picked_or_dropped_in_order() {
    assert_eq!(toggled(&[1, 4], 2), vec![1, 2, 4]);
    assert_eq!(toggled(&[1, 2, 4], 2), vec![1, 4]);
    assert_eq!(toggled(&[], 0), vec![0]);
}

#[test]
fn greyed_controls_are_passed_over_and_cannot_be_pressed() {
    let now = Instant::now();
    let mut ui = Ui::new(column(vec![
        row(vec![button("a", "A").on_press(Msg::Save), switch(false, "Wi-Fi", Msg::Wifi).key("wifi")]).off(true),
        button("b", "B").on_press(Msg::Go).key("go"),
    ]), Theme::new(false, "#fbc711"));
    ui.size = (400, 300);
    ui.input(&key(Keysym::Tab, false), now);
    assert_eq!(ui.focus.as_deref(), Some("go"));
    let mut reply = Reply::new();
    ui.activate("wifi", now, &mut reply);
    assert!(reply.messages.is_empty());
    assert!(ui.node("wifi").is_some_and(|n| n.tip().is_some()));
}

#[test]
fn escape_says_no_and_the_safer_answer_starts_with_the_keyboard() {
    let now = Instant::now();
    let tree = || column(vec![
        entry("", "Name", Msg::Name).key("name"),
        row(vec![button("no", "No").on_press(Msg::Pick(0)).first().key("no"), button("yes", "Yes").on_press(Msg::Go)])
            .on_escape(Msg::Open(1)),
    ]).on_escape(Msg::Open(0));
    let mut ui = Ui::new(tree(), Theme::new(false, "#fbc711"));
    ui.size = (400, 300);
    ui.settle();
    assert_eq!(ui.focus.as_deref(), Some("no"));
    assert_eq!(ui.input(&key(Keysym::Escape, false), now).messages, vec![Msg::Open(1)]);
    ui.focus = Some("name".into());
    assert_eq!(ui.input(&key(Keysym::Escape, false), now).messages, vec![Msg::Open(0)]);
    ui.focus = None;
    ui.set_view(tree());
    ui.settle();
    assert_eq!(ui.focus, None);
    assert_eq!(ui.input(&key(Keysym::Escape, false), now).messages, vec![Msg::Open(0)]);
}

#[test]
fn a_grab_moves_the_keyboard_once_per_turn() {
    let tree = |turn: u32| column(vec![
        entry("", "Name", Msg::Name).key("name"),
        entry("", "Password", Msg::Pass).key("pass").secret().grab(turn),
    ]);
    let mut ui = Ui::new(tree(1), Theme::new(false, "#fbc711"));
    ui.size = (400, 300);
    ui.settle();
    assert_eq!(ui.focus.as_deref(), Some("pass"));
    ui.focus = Some("name".into());
    ui.set_view(tree(1));
    ui.settle();
    assert_eq!(ui.focus.as_deref(), Some("name"));
    ui.set_view(tree(2));
    ui.settle();
    assert_eq!(ui.focus.as_deref(), Some("pass"));
}

#[test]
fn a_pressable_card_takes_a_click_but_its_buttons_come_first() {
    let view = card(row(vec![label("Maps"), button("document-open-symbolic", "Open").on_press(Msg::Save).key("save")]))
        .on_press(Msg::Go).key("card");
    let mut ui = Ui::new(column(vec![view]).padding(10.0), Theme::new(false, "#fbc711"));
    ui.size = (400, 300);
    ui.settle();
    let now = Instant::now();
    let click = |ui: &mut Ui<Msg>, x: f32, y: f32| {
        ui.input(&Input::Press { button: Button::Left, x, y }, now);
        ui.input(&Input::Release { button: Button::Left, x, y }, now).messages
    };
    let r = ui.spot("card").unwrap().rect;
    assert_eq!(click(&mut ui, r.x + 4.0, r.y + 4.0), vec![Msg::Go]);
    let b = ui.spot("save").unwrap().rect;
    assert_eq!(click(&mut ui, b.x + b.w / 2.0, b.y + b.h / 2.0), vec![Msg::Save]);
    ui.input(&key(Keysym::Tab, false), now);
    assert_eq!(ui.focus.as_deref(), Some("card"));
    assert_eq!(ui.input(&key(Keysym::Return, false), now).messages, vec![Msg::Go]);
}

#[test]
fn a_click_presses_only_when_it_ends_where_it_began() {
    let mut ui = ui(false, 0.5);
    let now = Instant::now();
    let at = |ui: &mut Ui<Msg>, id: &str| {
        ui.settle();
        let r = ui.spot(id).unwrap().rect;
        (r.x + r.w / 2.0, r.y + r.h / 2.0)
    };
    let (x, y) = at(&mut ui, "0.0");
    ui.input(&Input::Press { button: Button::Left, x, y }, now);
    assert_eq!(ui.input(&Input::Release { button: Button::Left, x, y }, now).messages, vec![Msg::Save]);
    ui.input(&Input::Press { button: Button::Left, x, y }, now);
    assert!(ui.input(&Input::Release { button: Button::Left, x: 399.0, y: 299.0 }, now).messages.is_empty());
    let (x, y) = at(&mut ui, "wifi");
    ui.input(&Input::TouchDown { id: 3, x, y }, now);
    assert_eq!(ui.input(&Input::TouchUp { id: 3 }, now).messages, vec![Msg::Wifi(true)]);
    let r = ui.spot("2").unwrap().rect;
    ui.input(&Input::Press { button: Button::Left, x: r.x, y: r.y + 5.0 }, now);
    let m = ui.input(&Input::PointerMove { x: r.x + r.w * 0.74, y: r.y }, now).messages;
    assert_eq!(m, vec![Msg::Volume(0.8)]);
    ui.input(&Input::Release { button: Button::Left, x: r.x + r.w, y: r.y }, now);
    assert!(ui.dragging.is_none());
}

#[test]
fn a_name_shows_after_a_rest_and_goes_again() {
    let mut ui = ui(false, 0.5);
    let start = Instant::now();
    ui.settle();
    let r = ui.spot("0.1").unwrap().rect;
    ui.input(&Input::PointerIn { x: r.x + 4.0, y: r.y + 18.0 }, start);
    assert_eq!(ui.hover.as_deref(), Some("0.1"));
    assert!(!ui.tick(start + Duration::from_millis(100)));
    assert_eq!(ui.next_wake(start + Duration::from_millis(100)), Some(tip_rest() - Duration::from_millis(100)));
    assert!(ui.tick(start + tip_rest()));
    assert!(ui.tick(start + tip_rest() + tip_stays()));
    assert_eq!(ui.next_wake(start + tip_rest() + tip_stays()), None);
    ui.input(&Input::PointerOut, start);
    assert!(ui.tip.is_none() && ui.hover.is_none());
}

#[test]
fn a_refused_act_says_why_in_the_window() {
    use crate::window::{persons_own, refused_act, with_named, with_press};
    struct Guarded;
    const STOP: &str = "made-up-stop";
    impl crate::host::Host for Guarded {
        fn act(&self, act: &str) -> crate::host::Act {
            if act == STOP { crate::host::Act::PersonsOwn } else { crate::host::Act::Unknown }
        }
    }
    let stop = || crate::host::with(&Guarded, || persons_own(STOP));
    let _ = refused_act();
    assert!(with_named(true, Some(7), || with_press(Some(7), stop)));
    assert!(!refused_act(), "a press that was the person's own was taken for a refusal");
    assert!(!with_named(true, Some(7), || with_press(Some(8), stop)));
    assert!(refused_act(), "a refused act asked for no words");
    assert!(!refused_act(), "one refusal was said twice");
    let mut ui = ui(false, 0.5);
    let start = Instant::now();
    ui.settle();
    let r = ui.spot("0.1").unwrap().rect;
    ui.input(&Input::PointerIn { x: r.x + 4.0, y: r.y + 18.0 }, start);
    assert!(ui.input(&Input::Refused, start).redraw);
    assert!(ui.saying_refused());
    assert_eq!(ui.refused.as_ref().and_then(|(at, _)| at.clone()).as_deref(), Some("0.1"));
    let mut pix = Pixmap::new(ui.size.0.max(400), ui.size.1.max(300)).unwrap();
    let before = pix.data().to_vec();
    assert!(ui.paint_refused(&mut pix.as_mut()));
    assert_ne!(pix.data(), &before[..], "the refusal's label was not painted");
    assert!(ui.next_wake(start).is_some_and(|d| d <= tip_stays()), "nothing wakes the window to take the refusal's label away");
    assert!(ui.tick(start + tip_stays()));
    assert!(!ui.saying_refused(), "the refusal's label did not go");
    ui.input(&Input::PointerOut, start);
    ui.say_refused(start);
    assert_eq!(ui.refused.as_ref().map(|(at, _)| at.clone()), Some(None));
    assert!(ui.input(&Input::NoLayout, start).redraw && ui.saying_refused());
    assert_eq!(ui.refused_words, crate::t!("kit-no-layout"));
    assert!(crate::window::only_your_own().len() < 40 && !crate::window::only_your_own().contains('\u{2014}'));
}

#[test]
fn a_choice_picks_by_pointer_and_arrows() {
    let kinds = || vec!["ext4".to_string(), "exFAT".to_string(), "NTFS".to_string()];
    let mut ui = Ui::new(column(vec![choice(kinds(), 1, "Kind", Msg::Pick).key("kind")]).padding(10.0),
                         Theme::new(false, "#fbc711"));
    ui.size = (400, 300);
    let now = Instant::now();
    ui.settle();
    let r = ui.spot("kind").unwrap().rect;
    let (x, y) = (r.x + theme::sheet().tabs_padding + 4.0, r.y + r.h / 2.0);
    ui.input(&Input::Press { button: Button::Left, x, y }, now);
    assert_eq!(ui.input(&Input::Release { button: Button::Left, x, y }, now).messages, vec![Msg::Pick(0)]);
    let mid = r.x + r.w / 2.0;
    ui.input(&Input::Press { button: Button::Left, x: mid, y }, now);
    assert!(ui.input(&Input::Release { button: Button::Left, x: mid, y }, now).messages.is_empty());
    assert_eq!(ui.input(&key(Keysym::Right, false), now).messages, vec![Msg::Pick(2)]);
    assert_eq!(ui.input(&key(Keysym::Home, false), now).messages, vec![Msg::Pick(0)]);
    let mut pixmap = tiny_skia::Pixmap::new(400, 300).unwrap();
    ui.paint(&mut pixmap.as_mut(), 1.0);
    let at = |x: f32, y: f32| {
        let p = pixmap.pixel(x as u32, y as u32).unwrap();
        (p.red(), p.green(), p.blue())
    };
    let hex = |c: Colour| ((c.r * 255.0).round() as u8, (c.g * 255.0).round() as u8, (c.b * 255.0).round() as u8);
    assert_eq!(at(mid, r.y + theme::sheet().tabs_padding + 4.0), hex(ui.theme.card(0)));
    let (strip, control) = (at(r.x + r.h / 2.0, r.y + 1.0), hex(ui.theme.card(1)));
    assert!(control.0.abs_diff(strip.0) <= 4 && control.2.abs_diff(strip.2) <= 4, "{strip:?} {control:?}");
}

struct Settings {
    saved: u32,
    wifi: bool,
    volume: f32,
}

impl App for Settings {
    type Msg = Msg;
    fn update(&mut self, msg: Msg) {
        match msg {
            Msg::Save => self.saved += 1,
            Msg::Wifi(on) => self.wifi = on,
            Msg::Volume(v) => self.volume = v,
            _ => {}
        }
    }
    fn view(&self) -> Node<Msg> {
        view(self.wifi, self.volume)
    }
}

#[test]
#[ignore]
fn widgets_answer_a_real_keyboard() {
    use crate::window::tests::Hands;
    let events = Loop::new();
    let display = Display::connect(&events, || panic!("compositor gone")).expect("connected");
    let shown = Shown::open(&events, &display, Kind::Window { title: "Widgets".into(), app_id: "org.trindle.Test".into() },
                            (420, 220), Settings { saved: 0, wifi: false, volume: 0.5 }).expect("a window");
    let Hands { conn, keys, queue: _queue, .. } = Hands::lend();
    let keys = keys.expect("a stand-in keyboard");
    let presses = [15, 57, 15, 28, 15, 106];
    let at = Cell::new(0usize);
    let steps = events.clone();
    events.every(Duration::from_millis(250), move || {
        let i = at.get();
        at.set(i + 1);
        if let Some(&key) = i.checked_sub(4).and_then(|n| presses.get(n)) {
            keys.key(i as u32 * 250, key, 1);
            keys.key(i as u32 * 250 + 10, key, 0);
        }
        let _ = conn.flush();
        if i == 4 + presses.len() + 3 {
            steps.quit();
            return crate::event::Flow::Stop;
        }
        crate::event::Flow::Continue
    });
    events.run();
    let (saved, wifi, volume) = shown.with(|s| (s.saved, s.wifi, s.volume));
    eprintln!("saved {saved}, wifi {wifi}, volume {volume}");
    assert_eq!((saved, wifi, volume), (1, true, 0.6));
    let (w, h) = (840, 440);
    let mut pixmap = tiny_skia::Pixmap::new(w, h).unwrap();
    shown.held.borrow_mut().ui.paint(&mut pixmap.as_mut(), 2.0);
    let file = std::env::temp_dir().join(format!("trindle-widgets-{w}x{h}.rgba"));
    std::fs::write(&file, pixmap.data()).unwrap();
    eprintln!("painted to {}", file.display());
}

#[derive(Default)]
struct Pointed {
    over: Option<u8>,
    updated: u32,
    shown: Cell<u32>,
}

#[derive(Clone, Debug, PartialEq)]
enum Aim {
    Over(Option<u8>),
    Idle,
}

impl App for Pointed {
    type Msg = Aim;
    fn update(&mut self, msg: Aim) {
        self.updated += 1;
        if let Aim::Over(o) = msg {
            self.over = o;
        }
    }
    fn unchanged(&self, msg: &Aim) -> bool {
        match msg {
            Aim::Over(o) => *o == self.over,
            Aim::Idle => true,
        }
    }
    fn view(&self) -> Node<Aim> {
        self.shown.set(self.shown.get() + 1);
        let part = |n: u8| drawing((100.0, 100.0), format!("part {n}"), |_, _| {}).key(format!("p{n}")).on_hover().on_pointer(move |p| match p {
            crate::widget::Pointer::Hover(..) => Aim::Over(Some(n)),
            crate::widget::Pointer::Leave => Aim::Over(None),
            _ => Aim::Idle,
        });
        row(vec![part(1), part(2)])
    }
}

#[test]
fn pointer_moves_within_one_item_rebuild_and_repaint_nothing() {
    let mut app = Pointed::default();
    let mut ui: Ui<Aim> = Ui::new(app.view(), Theme::new(false, "#fbc711"));
    ui.size = (400, 300);
    ui.settle();
    let (a, b) = (ui.spot("p1").unwrap().rect, ui.spot("p2").unwrap().rect);
    let now = Instant::now();
    let step = |ui: &mut Ui<Aim>, app: &mut Pointed, input: Input| -> (bool, bool) {
        let reply = ui.input(&input, now);
        let redraw = reply.redraw;
        let changed = super::apply_messages(app, reply.messages, || false).unwrap();
        if changed {
            ui.set_view(app.view());
            ui.settle();
        }
        (changed, redraw)
    };
    let (changed, _) = step(&mut ui, &mut app, Input::PointerIn { x: a.x + 5.0, y: a.y + 5.0 });
    assert!(changed && app.over == Some(1), "arriving on a part is a change");
    let (updated, shown) = (app.updated, app.shown.get());
    for i in 0..40 {
        let (changed, redraw) = step(&mut ui, &mut app, Input::PointerMove { x: a.x + 5.0 + i as f32, y: a.y + 5.0 + (i % 7) as f32 });
        assert!(!changed && !redraw, "move {i} inside the same part rebuilt or repainted");
    }
    assert_eq!((app.updated, app.shown.get()), (updated, shown), "a move inside one part updated or showed the program");
    let (changed, _) = step(&mut ui, &mut app, Input::PointerMove { x: b.x + 5.0, y: b.y + 5.0 });
    assert!(changed && app.over == Some(2), "going to the next part is a change");
    let (changed, _) = step(&mut ui, &mut app, Input::PointerOut);
    assert!(changed && app.over.is_none(), "leaving is a change");
}

#[test]
fn the_pointer_goes_through_a_picture_laid_over_a_part() {
    let seen = |passes: bool| {
        let cover = drawing((100.0, 100.0), "cover", |_, _| {}).key("cover");
        let cover = if passes { cover.passes() } else { cover };
        let part = drawing((100.0, 100.0), "part", |_, _| {}).key("part").on_hover().on_pointer(|p| match p {
            crate::widget::Pointer::Hover(..) => Aim::Over(Some(1)),
            crate::widget::Pointer::Leave => Aim::Over(None),
            _ => Aim::Idle,
        });
        let mut ui: Ui<Aim> = Ui::new(crate::widget::layers(vec![part, cover]), Theme::new(false, "#fbc711"));
        ui.size = (400, 300);
        ui.settle();
        let r = ui.spot("part").unwrap().rect;
        let now = Instant::now();
        let mut got = ui.input(&Input::PointerIn { x: r.x + 5.0, y: r.y + 5.0 }, now).messages;
        for i in 0..10 {
            got.extend(ui.input(&Input::PointerMove { x: r.x + 10.0 + i as f32, y: r.y + 10.0 }, now).messages);
        }
        got
    };
    let through = seen(true);
    assert!(!through.is_empty() && through.iter().all(|m| *m == Aim::Over(Some(1))), "the pointer was lost crossing a picture that passes it: {through:?}");
    assert!(!seen(false).contains(&Aim::Over(Some(1))), "a drawing that does not pass the pointer should cover what is under it");
}
