//! The one-window rule: a program that should have one window claims a name on the session bus; a second copy hears that the name is taken, asks the first to come forward with the pass it was started with, and ends.

use crate::t;
use crate::bus::{Asked, Bus, Failed, Offer, Props, Served};
use crate::event::Loop;
use crate::window::Window;
use std::rc::Rc;
use std::time::{Duration, Instant};

pub const IFACE: &str = "org.trindle.OneWindow";
const XML: &str = r#"<interface name="org.trindle.OneWindow"><method name="Raise"><arg type="s" direction="in"/></method></interface>"#;
const WAIT: Duration = Duration::from_secs(2);
const TRIES: u32 = 3;
const RETRY: Duration = Duration::from_millis(300);

pub fn pass_from_env() -> String {
    let pass = pass_from(|k| std::env::var(k).ok());
    for k in ["XDG_ACTIVATION_TOKEN", "DESKTOP_STARTUP_ID"] {
        std::env::remove_var(k);
    }
    pass
}

fn pass_from(get: impl Fn(&str) -> Option<String>) -> String {
    ["XDG_ACTIVATION_TOKEN", "DESKTOP_STARTUP_ID"].iter().find_map(|k| get(k).filter(|v| !v.is_empty())).unwrap_or_default()
}

pub fn claim(bus: &Bus, lp: &Loop, name: &'static str, path: &'static str, first: impl FnOnce() + 'static, later: impl FnOnce() + 'static) {
    claim_with(bus, lp, name, path, pass_from_env(), first, later);
}

fn claim_with(bus: &Bus, lp: &Loop, name: &'static str, path: &'static str, pass: String, first: impl FnOnce() + 'static, later: impl FnOnce() + 'static) {
    let (b, l) = (bus.clone(), lp.clone());
    bus.own_firmly(name, lp, move |ok| {
        if ok {
            return first();
        }
        raise(b, l, name, path, pass, TRIES, Instant::now() + WAIT, Box::new(first), Box::new(later));
    });
}

#[allow(clippy::too_many_arguments)]
fn raise(bus: Bus, lp: Loop, name: &'static str, path: &'static str, pass: String, left: u32, end: Instant, first: Box<dyn FnOnce()>, later: Box<dyn FnOnce()>) {
    let (b, l) = (bus.clone(), lp.clone());
    let wait = end.saturating_duration_since(Instant::now());
    bus.call(name, path, IFACE).method("Raise", &(pass.clone(),), false, Some(wait), &lp, move |r| match r {
        Ok(_) => later(),
        Err(_) if left > 1 && Instant::now() + RETRY < end => {
            let again = l.clone();
            l.after(RETRY, move || raise(b, again, name, path, pass, left - 1, end, first, later));
        }
        Err(_) => first(),
    });
}

pub fn serve(bus: &Bus, path: &str, on_raise: impl Fn(Option<String>) + 'static) -> Result<Served, Failed> {
    bus.serve(path, Offer {
        name: IFACE.into(),
        xml: XML.into(),
        call: Box::new(move |asked: Asked| {
            if asked.member() != "Raise" {
                return asked.fail("org.freedesktop.DBus.Error.UnknownMethod", t!("kit-once-raise-only"));
            }
            let pass = asked.message().body().deserialize::<(String,)>().map(|p| p.0).unwrap_or_default();
            on_raise((!pass.is_empty()).then_some(pass));
            asked.reply(&());
        }),
        props: Box::new(Props::new),
        set: None,
    })
}

pub fn serve_window(bus: &Bus, path: &str, window: &Rc<Window>) -> Result<Served, Failed> {
    let w = Rc::downgrade(window);
    serve(bus, path, move |pass| {
        if let (Some(w), Some(pass)) = (w.upgrade(), pass) {
            w.expect_pass(&pass);
        }
    })
}

#[cfg(test)]
mod tests {
    use crate::bus::testing::{own_daemon, until};
    use crate::bus::Bus;
    use crate::event::Loop;
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;

    const NAME: &str = "org.trindle.OnceTest";
    const PATH: &str = "/org/trindle/OnceTest";

    #[test]
    fn a_second_copy_hands_its_pass_to_the_first_and_does_not_start() {
        let d = own_daemon();
        let lp = Loop::new();
        let (a, b) = (Bus::at(&lp, &d.1).expect("bus a"), Bus::at(&lp, &d.1).expect("bus b"));
        let started = Rc::new(Cell::new(0));
        let heard = Rc::new(RefCell::new(Vec::new()));
        let held = Rc::new(RefCell::new(None));
        let (s, h, k, bus) = (started.clone(), heard.clone(), held.clone(), a.clone());
        super::claim(&a, &lp, NAME, PATH, move || {
            s.set(s.get() + 1);
            *k.borrow_mut() = super::serve(&bus, PATH, move |p| h.borrow_mut().push(p)).ok();
        }, || {});
        let s = started.clone();
        assert!(until(&lp, move || s.get() == 1));
        let (s, later) = (started.clone(), Rc::new(Cell::new(false)));
        let l = later.clone();
        super::claim_with(&b, &lp, NAME, PATH, "pass-1".into(), move || s.set(s.get() + 10), move || l.set(true));
        let l = later.clone();
        assert!(until(&lp, move || l.get()));
        assert_eq!(started.get(), 1, "the second copy started a window of its own");
        assert_eq!(*heard.borrow(), vec![Some("pass-1".to_string())]);
    }

    #[test]
    fn a_member_other_than_raise_is_refused() {
        let d = own_daemon();
        let lp = Loop::new();
        let (a, b) = (Bus::at(&lp, &d.1).expect("bus a"), Bus::at(&lp, &d.1).expect("bus b"));
        let name = "org.trindle.OnceRefuse";
        let owned = Rc::new(Cell::new(false));
        let o = owned.clone();
        a.own_firmly(name, &lp, move |ok| o.set(ok));
        let o = owned.clone();
        assert!(until(&lp, move || o.get()));
        let _held = super::serve(&a, PATH, |_| panic!("only Raise reaches here")).unwrap();
        let answer = Rc::new(RefCell::new(None));
        let got = answer.clone();
        b.call(name, PATH, super::IFACE).method("Other", &(), false, Some(std::time::Duration::from_secs(2)), &lp, move |r| *got.borrow_mut() = Some(r.is_err()));
        let got = answer.clone();
        assert!(until(&lp, move || got.borrow().is_some()));
        assert_eq!(*answer.borrow(), Some(true));
    }

    #[test]
    fn the_pass_is_the_first_one_set_and_never_empty() {
        let env = |pairs: &'static [(&'static str, &'static str)]| move |k: &str| pairs.iter().find(|p| p.0 == k).map(|p| p.1.to_string());
        assert_eq!(super::pass_from(env(&[("XDG_ACTIVATION_TOKEN", "a"), ("DESKTOP_STARTUP_ID", "b")])), "a");
        assert_eq!(super::pass_from(env(&[("XDG_ACTIVATION_TOKEN", ""), ("DESKTOP_STARTUP_ID", "b")])), "b");
        assert_eq!(super::pass_from(env(&[])), "");
    }

    #[test]
    fn a_second_copy_starts_itself_when_the_first_never_answers() {
        let d = own_daemon();
        let lp = Loop::new();
        let (a, b) = (Bus::at(&lp, &d.1).expect("bus a"), Bus::at(&lp, &d.1).expect("bus b"));
        let owned = Rc::new(Cell::new(false));
        let o = owned.clone();
        a.own("org.trindle.OnceHung", false, &lp, move |ok| o.set(ok));
        assert!(until(&lp, move || owned.get()), "the first copy did not get the name");
        let _other = super::serve(&a, "/org/trindle/Other", |_| {}).expect("other object");
        let (started, later) = (Rc::new(Cell::new(false)), Rc::new(Cell::new(false)));
        let (s, l) = (started.clone(), later.clone());
        super::claim(&b, &lp, "org.trindle.OnceHung", PATH, move || s.set(true), move || l.set(true));
        let s = started.clone();
        assert!(until(&lp, move || s.get()), "the second copy gave up without starting");
        assert!(!later.get(), "the second copy said it had handed over when nothing heard it");
    }

    #[test]
    fn the_name_is_held_firmly_and_a_holder_that_does_not_answer_does_not_end_the_second_copy() {
        let d = own_daemon();
        let lp = Loop::new();
        let (a, b, c) = (Bus::at(&lp, &d.1).expect("bus a"), Bus::at(&lp, &d.1).expect("bus b"), Bus::at(&lp, &d.1).expect("bus c"));
        let started = Rc::new(Cell::new(0));
        let s = started.clone();
        super::claim(&a, &lp, NAME, PATH, move || s.set(s.get() + 1), || {});
        let s = started.clone();
        assert!(until(&lp, move || s.get() == 1));
        let took = Rc::new(Cell::new(None));
        let t = took.clone();
        c.take_firmly(NAME, &lp, move |ok| t.set(Some(ok)));
        let t = took.clone();
        assert!(until(&lp, move || t.get().is_some()));
        assert_eq!(took.get(), Some(false), "the name was taken from the copy that held it");
        let (s, later) = (started.clone(), Rc::new(Cell::new(false)));
        let l = later.clone();
        super::claim(&b, &lp, NAME, PATH, move || s.set(s.get() + 10), move || l.set(true));
        let s = started.clone();
        assert!(until(&lp, move || s.get() == 11));
        assert!(!later.get(), "it ended as if the first had come forward");
    }
}
