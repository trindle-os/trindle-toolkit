//! A window on its way out is not asked what to show again.

use super::*;
use super::super::theme::Theme;
use super::super::widget::label;

struct Leaver {
    events: Loop,
    pages: Vec<String>,
}

#[derive(Clone)]
enum Msg {
    Add,
    Close,
}

impl App for Leaver {
    type Msg = Msg;

    fn update(&mut self, msg: Msg) {
        match msg {
            Msg::Add => self.pages.push("Notes".into()),
            Msg::Close => {
                self.pages.clear();
                self.events.quit();
            }
        }
    }

    fn view(&self) -> Node<Msg> {
        label(self.pages[0].clone())
    }
}

fn held(events: &Loop) -> Held<Leaver> {
    let app = Leaver { events: events.clone(), pages: vec!["Shopping".into()] };
    let look: Look = Rc::new(|| Theme::new(false, "#fbc711"));
    let ui = Ui::new(app.view(), look());
    Held { app, ui, look, wake: None, link: None, out: false, outside: None, told: None }
}

#[test]
fn a_program_still_open_is_asked_what_it_shows() {
    let events = Loop::new();
    let mut h = held(&events);
    h.app.update(Msg::Add);
    assert!(show(&events, &mut h));
}

#[test]
fn a_program_that_has_asked_to_stop_is_not_asked_for_a_view_again() {
    let events = Loop::new();
    let mut h = held(&events);
    h.app.update(Msg::Close);
    assert!(events.quitting());
    assert!(!show(&events, &mut h));
}
