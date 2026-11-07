//! The eye in a secret box: shown and hidden, when it hides by itself, and how the keyboard reaches it.

use std::time::{Duration, Instant};

use super::super::input::{Button, Input, Keysym, Mods};
use super::super::theme::Theme;
use super::super::widget::{button, card, column, entry, label, row, Length, Node, Rect, Secret, Tone};
use super::shared_tests::{key, middle, typed};
use super::{Reply, Ui};
use crate::common::eye::REST;

#[derive(Clone, Debug, PartialEq)]
enum Msg {
    Name(String),
    Pass(String),
    Pin(String),
    Go,
}

#[derive(Default)]
struct Form {
    name: String,
    pass: String,
    pin: String,
    wrong: bool,
    tries: u32,
    away: bool,
    guarded: bool,
    busy: bool,
    suggested: u32,
}

impl Form {
    fn update(&mut self, m: Msg) {
        match m {
            Msg::Name(t) => self.name = t,
            Msg::Pass(t) => self.pass = t,
            Msg::Pin(t) => self.pin = t,
            Msg::Go => self.tries += 1,
        }
    }

    fn view(&self) -> Node<Msg> {
        let mut items = vec![entry(self.name.clone(), "Name", Msg::Name).key("name")];
        if !self.away {
            let pass = entry(self.pass.clone(), "Password", Msg::Pass).key("pass").secret().on_submit(Msg::Go)
                .shown(self.suggested).off(self.busy);
            items.push(if self.guarded { pass.guarded() } else { pass });
            items.push(label(if self.wrong { "That password is not right." } else { "" }).tone(Tone::Problem).width(Length::Fill).key("why"));
        }
        items.push(row(vec![
            entry(self.pin.clone(), "PIN", Msg::Pin).key("pin").secret_of(Secret::Pin).width(Length::Fixed(110.0)),
            button("go-next", "Log in").on_press(Msg::Go).key("go"),
        ]).gap(8.0));
        column(vec![card(column(items).gap(8.0)).key("card")]).padding(10.0)
    }
}

fn ui_of(form: &Form) -> Ui<Msg> {
    let mut ui = Ui::new(form.view(), Theme::new(false, "#fbc711"));
    ui.size = (400, 300);
    ui.settle();
    ui
}

fn feed_at(ui: &mut Ui<Msg>, form: &mut Form, input: &Input, at: Instant) -> Reply<Msg> {
    let reply = ui.input(input, at);
    for m in &reply.messages {
        form.update(m.clone());
    }
    ui.set_view(form.view());
    ui.settle();
    reply
}

fn feed(ui: &mut Ui<Msg>, form: &mut Form, input: &Input) -> Reply<Msg> {
    feed_at(ui, form, input, Instant::now())
}

fn back_tab() -> Input {
    Input::Key { sym: Keysym::Tab, text: None, mods: Mods { shift: true, ..Mods::default() }, repeat: false }
}

fn click(ui: &mut Ui<Msg>, form: &mut Form, (x, y): (f32, f32)) {
    feed(ui, form, &Input::PointerMove { x, y });
    feed(ui, form, &Input::Press { button: Button::Left, x, y });
    feed(ui, form, &Input::Release { button: Button::Left, x, y });
}

fn click_on(ui: &mut Ui<Msg>, form: &mut Form, id: &str) {
    let at = middle(ui.spot(id).expect("no such widget").rect);
    click(ui, form, at);
}

fn click_eye(ui: &mut Ui<Msg>, form: &mut Form, id: &str) {
    let at = middle(ui.eye_place(id).expect("the box has an eye"));
    click(ui, form, at);
}

fn places(ui: &Ui<Msg>) -> Vec<(String, Rect)> {
    ui.spots.iter().map(|s| (s.id.clone(), s.rect)).collect()
}

#[test]
fn the_eye_shows_what_is_typed_and_hides_it_when_pressed_again() {
    let mut form = Form { pass: "hunter2".into(), ..Form::default() };
    let mut ui = ui_of(&form);
    assert!(!ui.shows_secret("pass") && ui.entry_dots("pass"));
    assert_eq!(ui.eye_name("pass"), Some("Show the password"));
    click_eye(&mut ui, &mut form, "pass");
    assert!(ui.shows_secret("pass") && !ui.entry_dots("pass"));
    assert_eq!(ui.eye_name("pass"), Some("Hide the password"));
    assert_eq!(ui.news, "Password shown");
    assert!(!ui.shows_secret("pin"));
    click_eye(&mut ui, &mut form, "pass");
    assert!(!ui.shows_secret("pass") && ui.entry_dots("pass"));
    assert_eq!(ui.news, "Password hidden");
    assert_eq!(ui.eye_name("pin"), Some("Show the PIN"));
    click_eye(&mut ui, &mut form, "pin");
    assert_eq!((ui.eye_name("pin"), ui.news.as_str()), (Some("Hide the PIN"), "PIN shown"));
    assert!(ui.eye_place("name").is_none() && ui.eye_name("name").is_none());
    assert_eq!(form.tries, 0);
}

#[test]
fn what_shows_is_drawn_as_letters_in_place_of_the_dots() {
    let form = Form { pass: "hunter2".into(), ..Form::default() };
    let mut ui = ui_of(&form);
    let paint = |ui: &mut Ui<Msg>| {
        let mut pix = tiny_skia::Pixmap::new(400, 300).unwrap();
        ui.paint(&mut pix.as_mut(), 1.0);
        pix
    };
    let hidden = paint(&mut ui);
    ui.press_eye("pass");
    let shown = paint(&mut ui);
    let r = ui.spot("pass").unwrap().rect;
    let eye = ui.eye_place("pass").unwrap();
    let differ = |a: Rect| (a.y as u32..(a.y + a.h) as u32).any(|y| (a.x as u32..(a.x + a.w) as u32)
        .any(|x| hidden.pixel(x, y) != shown.pixel(x, y)));
    assert!(differ(Rect { w: eye.x - r.x, ..r }), "the words did not change");
    let accent = ui.theme.picked;
    let lit = |pix: &tiny_skia::Pixmap| {
        let p = pix.pixel(eye.x as u32 + 2, (eye.y + eye.h / 2.0) as u32).unwrap();
        (i32::from(p.red()) - (accent.r * 255.0) as i32).abs() <= 2 && (i32::from(p.green()) - (accent.g * 255.0) as i32).abs() <= 2
    };
    assert!(!lit(&hidden) && lit(&shown));
    for y in 0..300u32 {
        for x in 0..400u32 {
            if !r.inset(-4.0).contains(x as f32, y as f32) {
                assert_eq!(hidden.pixel(x, y), shown.pixel(x, y), "at {x}, {y}");
            }
        }
    }
}

#[test]
fn nothing_moves_when_the_eye_is_pressed() {
    let mut form = Form { pass: "hunter2".into(), wrong: true, ..Form::default() };
    let mut ui = ui_of(&form);
    let before = places(&ui);
    let tall = ui.natural_height(400.0);
    click_eye(&mut ui, &mut form, "pass");
    assert!(ui.shows_secret("pass"));
    assert_eq!(places(&ui), before, "something moved when the password was shown");
    assert_eq!(ui.natural_height(400.0), tall);
    click_eye(&mut ui, &mut form, "pass");
    assert_eq!(places(&ui), before, "something moved when the password was hidden");
    assert!(before.iter().any(|(id, _)| id == "why"));
    for id in ["pass", "pin"] {
        let (r, eye) = (ui.spot(id).unwrap().rect, ui.eye_place(id).unwrap());
        assert!(eye.x > r.x && eye.x + eye.w < r.x + r.w && eye.y > r.y && eye.y + eye.h < r.y + r.h, "{id}: {eye:?} in {r:?}");
        assert_eq!(eye.w, eye.h);
    }
    let (name, pass) = (ui.spot("name").unwrap().rect, ui.spot("pass").unwrap().rect);
    assert_eq!((name.w, name.h, name.x), (pass.w, pass.h, pass.x));
}

#[test]
fn the_words_keep_clear_of_the_eye_however_long_they_are() {
    let mut form = Form { pass: "a very long password that runs past the end of its box 0123456789".into(), ..Form::default() };
    let mut ui = ui_of(&form);
    ui.focus = Some("pass".into());
    for shown in [false, true] {
        if shown {
            click_eye(&mut ui, &mut form, "pass");
        }
        feed(&mut ui, &mut form, &key(Keysym::End));
        let mut pix = tiny_skia::Pixmap::new(400, 300).unwrap();
        ui.paint(&mut pix.as_mut(), 1.0);
        let t = ui.typing().unwrap();
        let (r, eye) = (ui.spot("pass").unwrap().rect, ui.eye_place("pass").unwrap());
        assert!((t.area.0 as f32) < eye.x && (t.area.0 as f32) > r.x, "shown {shown}: the caret is at {} beside an eye at {}", t.area.0, eye.x);
        let ink = ui.theme.fg;
        let gap = (eye.x - 6.0) as u32..eye.x as u32;
        for y in r.y as u32 + 4..(r.y + r.h) as u32 - 4 {
            for x in gap.clone() {
                let p = pix.pixel(x, y).unwrap();
                assert!((i32::from(p.red()) - (ink.r * 255.0) as i32).abs() > 40, "shown {shown}: words under the eye at {x}, {y}");
            }
        }
    }
}

#[test]
fn tab_reaches_the_eye_straight_after_its_box_and_space_or_enter_presses_it() {
    let mut form = Form { pass: "hunter2".into(), ..Form::default() };
    let mut ui = ui_of(&form);
    ui.focus = Some("name".into());
    let at = |ui: &Ui<Msg>| (ui.focus.clone().unwrap_or_default(), ui.focus.as_deref().is_some_and(|id| ui.on_eye(id)));
    let mut order = Vec::new();
    for _ in 0..6 {
        feed(&mut ui, &mut form, &key(Keysym::Tab));
        order.push(at(&ui));
    }
    let stop = |id: &str, eye: bool| (id.to_string(), eye);
    assert_eq!(order, [stop("pass", false), stop("pass", true), stop("pin", false), stop("pin", true), stop("go", false), stop("name", false)]);
    let mut back = Vec::new();
    for _ in 0..6 {
        feed(&mut ui, &mut form, &back_tab());
        back.push(at(&ui));
    }
    assert_eq!(back, [stop("go", false), stop("pin", true), stop("pin", false), stop("pass", true), stop("pass", false), stop("name", false)]);
    feed(&mut ui, &mut form, &key(Keysym::Tab));
    feed(&mut ui, &mut form, &key(Keysym::Tab));
    assert_eq!(at(&ui), stop("pass", true));
    feed(&mut ui, &mut form, &key(Keysym::space));
    assert!(ui.shows_secret("pass"));
    feed(&mut ui, &mut form, &key(Keysym::Return));
    assert!(!ui.shows_secret("pass"));
    feed(&mut ui, &mut form, &key(Keysym::KP_Enter));
    assert!(ui.shows_secret("pass"));
    assert_eq!((form.tries, form.pass.as_str()), (0, "hunter2"), "a key on the eye went to the box or the program");
    assert_eq!(at(&ui), stop("pass", true));
    feed(&mut ui, &mut form, &key(Keysym::space));
    assert!(!ui.shows_secret("pass"));
    let mut pix = tiny_skia::Pixmap::new(400, 300).unwrap();
    ui.paint(&mut pix.as_mut(), 1.0);
    let (r, eye) = (ui.spot("pass").unwrap().rect, ui.eye_place("pass").unwrap());
    let accent = ui.theme.accent;
    let is_accent = |x: f32, y: f32| {
        let p = pix.pixel(x as u32, y as u32).unwrap();
        (i32::from(p.red()) - (accent.r * 255.0) as i32).abs() <= 3 && (i32::from(p.blue()) - (accent.b * 255.0) as i32).abs() <= 3
    };
    assert!(!is_accent(r.x + 30.0, r.y), "the box is ringed while the keyboard is on its eye");
    assert!((0..6).any(|d| is_accent(eye.x - d as f32, eye.y + eye.h / 2.0)), "no ring round the eye");
}

#[test]
fn typing_carries_on_in_the_box_after_the_eye_is_pressed() {
    let mut form = Form::default();
    let mut ui = ui_of(&form);
    click_on(&mut ui, &mut form, "name");
    click_eye(&mut ui, &mut form, "pass");
    assert_eq!(ui.focus.as_deref(), Some("pass"));
    assert!(!ui.on_eye("pass") && ui.shows_secret("pass"));
    for t in ["h", "u", "n"] {
        feed(&mut ui, &mut form, &typed(t));
    }
    assert_eq!(form.pass, "hun");
    click_eye(&mut ui, &mut form, "pass");
    feed(&mut ui, &mut form, &typed("t"));
    assert_eq!(form.pass, "hunt");
    feed(&mut ui, &mut form, &key(Keysym::Tab));
    assert!(ui.on_eye("pass"));
    feed(&mut ui, &mut form, &typed("e"));
    assert_eq!(form.pass, "hunte");
    assert!(!ui.on_eye("pass") && ui.focus.as_deref() == Some("pass"));
    feed(&mut ui, &mut form, &key(Keysym::Tab));
    feed(&mut ui, &mut form, &key(Keysym::BackSpace));
    assert_eq!(form.pass, "hunt");
    assert!(!ui.on_eye("pass"));
    feed(&mut ui, &mut form, &key(Keysym::Tab));
    feed(&mut ui, &mut form, &key(Keysym::Shift_L));
    assert!(ui.on_eye("pass"));
}

#[test]
fn a_press_that_ends_off_the_eye_does_nothing_and_a_finger_finds_it() {
    let mut form = Form { pass: "hunter2".into(), ..Form::default() };
    let mut ui = ui_of(&form);
    let (ex, ey) = middle(ui.eye_place("pass").unwrap());
    feed(&mut ui, &mut form, &Input::PointerMove { x: ex, y: ey });
    feed(&mut ui, &mut form, &Input::Press { button: Button::Left, x: ex, y: ey });
    feed(&mut ui, &mut form, &Input::PointerMove { x: ex - 120.0, y: ey });
    feed(&mut ui, &mut form, &Input::Release { button: Button::Left, x: ex - 120.0, y: ey });
    assert!(!ui.shows_secret("pass"));
    assert!(ui.edits.get("pass").is_none_or(|e| e.caret == e.anchor));
    let r = ui.spot("pass").unwrap().rect;
    let (tx, ty) = (r.x + r.w - 3.0, r.y + 3.0);
    feed(&mut ui, &mut form, &Input::TouchDown { id: 1, x: tx, y: ty });
    feed(&mut ui, &mut form, &Input::TouchUp { id: 1 });
    assert!(ui.shows_secret("pass"));
    click(&mut ui, &mut form, (r.x + 20.0, r.y + r.h / 2.0));
    assert!(ui.shows_secret("pass"));
}

#[test]
fn the_eyes_name_shows_on_pointing_under_the_eye() {
    let mut form = Form { pass: "hunter2".into(), ..Form::default() };
    let mut ui = ui_of(&form);
    let r = ui.spot("pass").unwrap().rect;
    let t0 = Instant::now();
    feed_at(&mut ui, &mut form, &Input::PointerMove { x: r.x + 20.0, y: r.y + r.h / 2.0 }, t0);
    assert_eq!(ui.label_of("pass"), None);
    assert_eq!(ui.cursor(), super::Cursor::Text);
    let (ex, ey) = middle(ui.eye_place("pass").unwrap());
    feed_at(&mut ui, &mut form, &Input::PointerMove { x: ex, y: ey }, t0);
    assert_eq!(ui.label_of("pass").as_deref(), Some("Show the password"));
    assert_eq!(ui.cursor(), super::Cursor::Arrow);
    ui.tick(t0 + super::tip_rest() + Duration::from_millis(10));
    let mut pix = tiny_skia::Pixmap::new(400, 300).unwrap();
    let drawn = ui.tip_drawn(&mut pix.as_mut()).expect("the eye's name is drawn");
    let eye = ui.eye_place("pass").unwrap();
    assert!(drawn.y >= r.y + r.h, "the name covers its box");
    assert!((drawn.x + drawn.w / 2.0 - (eye.x + eye.w / 2.0)).abs() < drawn.w / 2.0 + 1.0, "the name is not under the eye");
    feed_at(&mut ui, &mut form, &Input::PointerMove { x: r.x + 20.0, y: r.y + r.h / 2.0 }, t0);
    assert_eq!(ui.label_of("pass"), None);
}

#[test]
fn what_shows_hides_again_when_its_box_leaves_the_screen() {
    let mut form = Form { pass: "hunter2".into(), ..Form::default() };
    let mut ui = ui_of(&form);
    ui.press_eye("pass");
    ui.press_eye("pin");
    assert!(ui.shows_secret("pass") && ui.shows_secret("pin"));
    form.away = true;
    ui.set_view(form.view());
    ui.settle();
    assert!(!ui.shows_secret("pass"));
    form.away = false;
    ui.set_view(form.view());
    ui.settle();
    assert!(!ui.shows_secret("pass"), "a box that came back shows what it hid");
    assert!(ui.shows_secret("pin"));
    ui.press_eye("pass");
    form.busy = true;
    ui.set_view(form.view());
    ui.settle();
    assert!(!ui.shows_secret("pass"));
}

#[test]
fn what_shows_hides_again_when_the_window_loses_the_keyboard() {
    let mut form = Form { pass: "hunter2".into(), ..Form::default() };
    let mut ui = ui_of(&form);
    ui.press_eye("pass");
    let reply = feed(&mut ui, &mut form, &Input::Focus(false));
    assert!(reply.redraw && !ui.shows_secret("pass"));
    feed(&mut ui, &mut form, &Input::Focus(true));
    assert!(!ui.shows_secret("pass"));
    ui.press_eye("pass");
    assert!(ui.hide_secrets() && !ui.shows_secret("pass") && !ui.hide_secrets());
}

#[test]
fn what_shows_hides_again_when_the_try_is_made() {
    let mut form = Form { pass: "hunter2".into(), ..Form::default() };
    let mut ui = ui_of(&form);
    ui.focus = Some("pass".into());
    ui.press_eye("pass");
    feed(&mut ui, &mut form, &key(Keysym::Return));
    assert_eq!(form.tries, 1);
    assert!(!ui.shows_secret("pass"));
    ui.press_eye("pass");
    click_on(&mut ui, &mut form, "go");
    assert_eq!(form.tries, 2);
    assert!(!ui.shows_secret("pass"));
    ui.press_eye("pass");
    click_on(&mut ui, &mut form, "name");
    feed(&mut ui, &mut form, &typed("j"));
    assert!(ui.shows_secret("pass") && form.name == "j");
}

#[test]
fn what_shows_hides_again_when_the_program_empties_the_box_after_a_failed_try() {
    let mut form = Form::default();
    let mut ui = ui_of(&form);
    ui.focus = Some("pass".into());
    for t in ["n", "o"] {
        feed(&mut ui, &mut form, &typed(t));
    }
    ui.press_eye("pass");
    form.pass.clear();
    form.wrong = true;
    let before = places(&ui);
    ui.set_view(form.view());
    ui.settle();
    assert!(!ui.shows_secret("pass"));
    assert_eq!(places(&ui), before, "the wrong-password words moved something");
    feed(&mut ui, &mut form, &typed("x"));
    ui.press_eye("pass");
    feed(&mut ui, &mut form, &key(Keysym::BackSpace));
    assert!(form.pass.is_empty() && ui.shows_secret("pass"));
    feed(&mut ui, &mut form, &typed("y"));
    assert!(ui.shows_secret("pass"));
}

#[test]
fn a_guarded_box_hides_after_half_a_minute_with_no_key_pressed() {
    let mut form = Form { pass: "hunter2".into(), guarded: true, ..Form::default() };
    let mut ui = ui_of(&form);
    ui.focus = Some("pass".into());
    let t0 = Instant::now();
    let (ex, ey) = middle(ui.eye_place("pass").unwrap());
    feed_at(&mut ui, &mut form, &Input::PointerMove { x: ex, y: ey }, t0);
    feed_at(&mut ui, &mut form, &Input::Press { button: Button::Left, x: ex, y: ey }, t0);
    feed_at(&mut ui, &mut form, &Input::Release { button: Button::Left, x: ex, y: ey }, t0);
    assert!(ui.shows_secret("pass"));
    let wake = ui.eyes_wake(t0 + Duration::from_secs(10)).expect("no wake asked for");
    assert!(wake <= Duration::from_secs(20) && wake > Duration::from_secs(19));
    ui.tick(t0 + REST - Duration::from_secs(1));
    assert!(ui.shows_secret("pass"));
    feed_at(&mut ui, &mut form, &typed("x"), t0 + Duration::from_secs(25));
    ui.tick(t0 + Duration::from_secs(40));
    assert!(ui.shows_secret("pass"));
    ui.news.clear();
    assert!(ui.tick(t0 + Duration::from_secs(25) + REST));
    assert!(!ui.shows_secret("pass"));
    assert_eq!(ui.news, "Password hidden");
    assert!(ui.eyes_wake(t0 + Duration::from_secs(60)).is_none());
    ui.press_eye("pin");
    ui.tick(Instant::now() + REST + REST);
    assert!(ui.shows_secret("pin"));
}

#[test]
fn a_program_can_only_show_a_secret_it_made_and_only_once() {
    let mut form = Form::default();
    let mut ui = ui_of(&form);
    assert!(!ui.shows_secret("pass"));
    form.pass = "correct horse battery".into();
    form.suggested = 1;
    ui.set_view(form.view());
    ui.settle();
    assert!(ui.shows_secret("pass"));
    ui.press_eye("pass");
    ui.set_view(form.view());
    ui.settle();
    assert!(!ui.shows_secret("pass"));
    form.away = true;
    ui.set_view(form.view());
    ui.settle();
    form.away = false;
    ui.set_view(form.view());
    ui.settle();
    assert!(!ui.shows_secret("pass"));
}

#[test]
fn what_shows_is_still_never_copied_or_given_to_the_input_method() {
    let mut form = Form { pass: "s3cr3t".into(), ..Form::default() };
    let mut ui = ui_of(&form);
    ui.focus = Some("pass".into());
    ui.press_eye("pass");
    let ctrl = |sym| Input::Key { sym, text: None, mods: Mods { ctrl: true, ..Mods::default() }, repeat: false };
    feed(&mut ui, &mut form, &ctrl(Keysym::a));
    assert!(feed(&mut ui, &mut form, &ctrl(Keysym::c)).copy.is_none());
    assert!(feed(&mut ui, &mut form, &ctrl(Keysym::x)).copy.is_none());
    assert_eq!(form.pass, "s3cr3t");
    assert!(ui.shows_secret("pass"));
    assert!(ui.typing().unwrap().secret);
    let (ex, ey) = middle(ui.eye_place("pass").unwrap());
    feed(&mut ui, &mut form, &Input::PointerMove { x: ex, y: ey });
    let down = feed(&mut ui, &mut form, &Input::Press { button: Button::Left, x: ex, y: ey });
    let up = feed(&mut ui, &mut form, &Input::Release { button: Button::Left, x: ex, y: ey });
    assert!(down.copy.is_none() && up.copy.is_none() && down.messages.is_empty() && up.messages.is_empty());
}

#[test]
fn a_button_pressed_straight_after_typing_works_first_time() {
    let mut form = Form::default();
    let mut ui = ui_of(&form);
    let t0 = Instant::now();
    let r = ui.spot("pass").unwrap().rect;
    let (x, y) = (r.x + 10.0, r.y + r.h / 2.0);
    feed_at(&mut ui, &mut form, &Input::PointerIn { x, y }, t0);
    feed_at(&mut ui, &mut form, &Input::Press { button: Button::Left, x, y }, t0 + Duration::from_millis(10));
    feed_at(&mut ui, &mut form, &Input::Release { button: Button::Left, x, y }, t0 + Duration::from_millis(60));
    let mut ms = 100;
    for c in "plainmadeup11".chars() {
        feed_at(&mut ui, &mut form, &typed(&c.to_string()), t0 + Duration::from_millis(ms));
        ms += 30;
    }
    assert_eq!(form.pass, "plainmadeup11");
    let (ex, ey) = middle(ui.eye_place("pass").unwrap());
    let at = t0 + Duration::from_millis(ms);
    feed_at(&mut ui, &mut form, &Input::PointerMove { x: ex, y: ey }, at);
    feed_at(&mut ui, &mut form, &Input::Press { button: Button::Left, x: ex, y: ey }, at);
    feed_at(&mut ui, &mut form, &Input::Release { button: Button::Left, x: ex, y: ey }, at + Duration::from_millis(5));
    assert!(ui.shows_secret("pass"), "the first press after typing did nothing");
    feed_at(&mut ui, &mut form, &typed("G"), at + Duration::from_millis(200));
    feed_at(&mut ui, &mut form, &Input::Press { button: Button::Left, x: ex, y: ey }, at + Duration::from_millis(200));
    feed_at(&mut ui, &mut form, &Input::Release { button: Button::Left, x: ex, y: ey }, at + Duration::from_millis(205));
    assert!(!ui.shows_secret("pass"), "the second press after typing did nothing");
    assert_eq!(form.pass.len(), "plainmadeup11".len() + 1, "the second typing went in");
}

#[test]
fn the_eyes_size_comes_from_the_style_sheet_and_stays_inside_the_box() {
    let sheet = super::theme::sheet();
    assert!(sheet.entry_eye >= 24.0, "smaller than a pointer can fairly hit");
    assert!(sheet.entry_eye <= sheet.entry_height - 4.0);
    let form = Form::default();
    for scale in [1.0f32, 1.5, 2.0] {
        let mut ui = Ui::new(form.view(), Theme::new(false, "#fbc711"));
        ui.size = ((400.0 * scale) as u32, (300.0 * scale) as u32);
        ui.scale = scale;
        ui.settle();
        let (r, eye) = (ui.spot("pass").unwrap().rect, ui.eye_place("pass").unwrap());
        assert_eq!(eye.w, (sheet.entry_eye * scale).round(), "at {scale}");
        assert!(eye.x + eye.w < r.x + r.w && eye.y > r.y && eye.y + eye.h < r.y + r.h, "at {scale}");
    }
}
