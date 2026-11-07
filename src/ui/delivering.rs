//! What the widgets said handed to the program, and all that comes back from it.

use super::*;

pub(super) fn show<A: App>(events: &Loop, held: &mut Held<A>) -> bool where A::Msg: Clone + 'static {
    if events.quitting() {
        return false;
    }
    let (view, cue) = (held.app.view(), held.app.blend());
    held.ui.blend_to(cue);
    held.ui.set_view(view);
    true
}

pub(super) fn apply_messages<A: App>(app: &mut A, messages: Vec<A::Msg>, gone: impl Fn() -> bool) -> Option<bool> {
    let mut changed = false;
    for m in messages {
        if app.unchanged(&m) {
            continue;
        }
        changed = true;
        app.update(m);
        if gone() {
            return None;
        }
    }
    Some(changed)
}

pub(super) fn deliver<A: App + 'static>(events: &Loop, held: &Rc<RefCell<Held<A>>>, window: &Rc<Window>, reply: Reply<A::Msg>) where A::Msg: Clone {
    let patch = reply.patch;
    let Some((redraw, paste)) = hand_to_program(events, held, window, reply.messages, reply.redraw) else { return };
    if let Some(text) = reply.copy {
        window.copy(&text);
    }
    let rang = hand_over(held, window);
    let redraw = redraw || rang;
    if paste {
        read_clipboard(events, held, window);
    }
    if let Some(wanted) = reply.wanted {
        window.drop_answer(wanted);
    }
    if let Some(dropped) = reply.dropped {
        end_drop(held, window, dropped);
    }
    drag_out(held, window, reply.lift);
    if let Some(n) = reply.hold {
        hold_later(events, held, window, n);
    }
    if reply.paste {
        paste_into_widgets(events, held, window);
    }
    wake_later(events, held, window);
    if redraw {
        window.redraw();
    } else if patch {
        window.patch();
    }
}

fn hand_to_program<A: App + 'static>(events: &Loop, held: &Rc<RefCell<Held<A>>>, window: &Rc<Window>, messages: Vec<A::Msg>, redraw: bool) -> Option<(bool, bool)> where A::Msg: Clone {
    let mut held = held.borrow_mut();
    if let Some(place) = held.ui.take_pressed() {
        held.app.pressed(place);
    }
    let changed = apply_messages(&mut held.app, messages, || events.quitting())?;
    let paste = paste_asked(&mut held.app);
    if crate::window::refused_act() {
        held.ui.say_refused(Instant::now());
    }
    if changed && !show(events, &mut held) {
        return None;
    }
    window.set_cursor(held.ui.cursor());
    window.set_typing(held.ui.typing().or_else(|| held.app.typing()));
    if redraw || changed {
        held.tell();
    }
    Some((redraw || changed, paste))
}

fn end_drop<A: App>(held: &Rc<RefCell<Held<A>>>, window: &Window, (id, way, taken): (u64, Way, bool)) {
    let end = if taken { held.borrow_mut().app.drop_end(id, way) } else { DropEnd::Failed };
    if end != DropEnd::Later {
        window.drop_done(id, end == DropEnd::Done);
    }
}

fn hold_later<A: App + 'static>(events: &Loop, held: &Rc<RefCell<Held<A>>>, window: &Rc<Window>, n: u64) where A::Msg: Clone {
    let (h, w, ev) = (Rc::downgrade(held), Rc::downgrade(window), events.clone());
    events.after(HOLD, move || {
        let (Some(held), Some(window)) = (h.upgrade(), w.upgrade()) else { return };
        let reply = held.borrow_mut().ui.held(n);
        deliver(&ev, &held, &window, reply);
    });
}

fn paste_into_widgets<A: App + 'static>(events: &Loop, held: &Rc<RefCell<Held<A>>>, window: &Rc<Window>) where A::Msg: Clone {
    let (h, w, ev) = (Rc::downgrade(held), Rc::downgrade(window), events.clone());
    window.paste(move |text| {
        let (Some(held), Some(window)) = (h.upgrade(), w.upgrade()) else { return };
        let reply = held.borrow_mut().ui.paste(&text);
        deliver(&ev, &held, &window, reply);
    });
}

pub(super) fn outgoing<A: App>(app: &mut A) -> (Option<String>, Option<String>) {
    (app.to_clipboard(), app.window_title())
}

pub(super) fn hand_over<A: App>(held: &Rc<RefCell<Held<A>>>, window: &Window) -> bool where A::Msg: Clone {
    let (words, title) = outgoing(&mut held.borrow_mut().app);
    if let Some(words) = words {
        window.copy(&words);
    }
    if let Some(title) = title {
        window.set_title(&title);
    }
    let mut held = held.borrow_mut();
    let held = &mut *held;
    if announced(&mut held.app, &mut held.ui) {
        held.tell();
    }
    held.app.wants_bell() && held.ui.ring_bell(Instant::now())
}

pub(super) fn announced<A: App>(app: &mut A, ui: &mut Ui<A::Msg>) -> bool {
    match app.announcement() {
        Some(words) if ui.news != words => {
            ui.news = words;
            true
        }
        _ => false,
    }
}

pub(super) fn paste_asked<A: App>(app: &mut A) -> bool {
    if !app.wants_paste() {
        return false;
    }
    let own = crate::window::own_key_now();
    if !own {
        crate::window::refuse();
    }
    own
}

pub(super) fn read_clipboard<A: App + 'static>(events: &Loop, held: &Rc<RefCell<Held<A>>>, window: &Rc<Window>) where A::Msg: Clone {
    let (h, w, ev) = (Rc::downgrade(held), Rc::downgrade(window), events.clone());
    window.paste(move |text| {
        let (Some(held), Some(window)) = (h.upgrade(), w.upgrade()) else { return };
        let message = held.borrow().app.pasted(text);
        let mut reply = Reply::new();
        reply.messages.extend(message);
        deliver(&ev, &held, &window, reply);
    });
}

fn drag_out<A: App + 'static>(held: &Rc<RefCell<Held<A>>>, window: &Rc<Window>, words: Option<String>) where A::Msg: Clone {
    let mut h = held.borrow_mut();
    let over = |h: &mut Held<A>| {
        h.ui.input(&Input::DragEnded { took: None, short: None }, Instant::now());
    };
    if let Some(text) = words {
        let (picture, scale) = h.ui.drag_picture(None, false, &text).unzip();
        let what = Dragged { files: Vec::new(), text: Some(text), typed: Vec::new(), right: false, loops: false, picture, scale: scale.unwrap_or(1) };
        if !window.drag_out(what) {
            over(&mut h);
        }
        return;
    }
    match h.app.dragging() {
        Some(l) if !h.out => {
            h.out = true;
            let (picture, scale) = h.ui.drag_picture(l.icon.as_deref(), l.folder, &l.label).unzip();
            window.drag_out(Dragged { files: l.files, text: None, typed: l.typed, right: l.right, loops: true, picture, scale: scale.unwrap_or(1) });
        }
        None if h.out => {
            h.out = false;
            if window.dragging_out() && !h.ui.lifting() {
                window.drag_stop();
                over(&mut h);
            }
        }
        _ => {}
    }
}

pub(super) fn wake_later<A: App + 'static>(events: &Loop, held: &Rc<RefCell<Held<A>>>, window: &Rc<Window>) where A::Msg: Clone {
    let now = Instant::now();
    let mut h = held.borrow_mut();
    let Some(wait) = h.ui.next_wake(now) else { return };
    let wait = if window.seen() { wait } else { wait.max(UNSEEN_STEP) };
    let due = now + wait;
    if h.wake.is_some_and(|w| w <= due && w > now) {
        return;
    }
    h.wake = Some(due);
    drop(h);
    let (held, window, ev) = (Rc::downgrade(held), Rc::downgrade(window), events.clone());
    events.after(wait, move || {
        let (Some(held), Some(window)) = (held.upgrade(), window.upgrade()) else { return };
        let (redraw, part) = {
            let mut h = held.borrow_mut();
            h.wake = None;
            let (turning, before) = (h.ui.only_turns(), h.ui.stands());
            let redraw = h.ui.tick(Instant::now());
            (redraw, turning && h.ui.only_turns() && h.ui.stands() == before)
        };
        wake_later(&ev, &held, &window);
        if part {
            window.patch();
        } else if redraw {
            window.redraw();
        }
    });
}
