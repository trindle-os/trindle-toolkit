//! Hearing when this person's session is locked and unlocked, from logind's LockedHint on the system bus.

use std::collections::HashMap;

use zbus::zvariant::OwnedValue;

use super::bus::{Bus, Listen};
use super::event::Loop;

const LOGIN: &str = "org.freedesktop.login1";
const SESSION: &str = "org.freedesktop.login1.Session";

pub fn on_locked(bus: &Bus, lp: &Loop, told: impl FnMut(bool) + 'static, done: impl FnOnce(Option<Listen>) + 'static) {
    let (b, l) = (bus.clone(), lp.clone());
    let pid = std::process::id();
    bus.call(LOGIN, "/org/freedesktop/login1", "org.freedesktop.login1.Manager").method("GetSessionByPID", &(pid,), false, None, lp, move |r| {
        let Some(path) = r.ok().and_then(|m| m.body().deserialize::<zbus::zvariant::OwnedObjectPath>().ok()) else { return done(None) };
        done(watch(&b, &l, path.as_str(), told));
    });
}

fn watch(bus: &Bus, lp: &Loop, path: &str, told: impl FnMut(bool) + 'static) -> Option<Listen> {
    let told = std::rc::Rc::new(std::cell::RefCell::new(told));
    let t = told.clone();
    let listen = bus.listen(Some(LOGIN), "org.freedesktop.DBus.Properties", Some("PropertiesChanged"), Some(path), move |m| {
        let Ok((iface, changed, _)) = m.body().deserialize::<(String, HashMap<String, OwnedValue>, Vec<String>)>() else { return };
        if let Some(locked) = (iface == SESSION).then(|| changed.get("LockedHint").and_then(|v| bool::try_from(v).ok())).flatten() {
            (t.borrow_mut())(locked);
        }
    }).ok()?;
    bus.call(LOGIN, path, SESSION).get("LockedHint", lp, move |r| {
        if let Some(locked) = r.ok().and_then(|v| bool::try_from(v).ok()) {
            (told.borrow_mut())(locked);
        }
    });
    Some(listen)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bus::testing::{own_daemon, until};
    use crate::bus::{Asked, Offer, Props};
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;
    use zbus::zvariant::Value;

    const PATH: &str = "/org/freedesktop/login1/session/_31";

    #[test]
    fn a_lock_and_an_unlock_are_heard_after_how_it_is_now() {
        let d = own_daemon();
        let lp = Loop::new();
        let (login, us) = (Bus::at(&lp, &d.1).expect("login"), Bus::at(&lp, &d.1).expect("us"));
        let owned = Rc::new(Cell::new(false));
        let o = owned.clone();
        login.own(LOGIN, false, &lp, move |ok| o.set(ok));
        assert!(until(&lp, move || owned.get()));
        let _manager = login.serve("/org/freedesktop/login1", Offer {
            name: "org.freedesktop.login1.Manager".into(),
            xml: String::new(),
            call: Box::new(|a: Asked| a.reply(&zbus::zvariant::ObjectPath::try_from(PATH).unwrap())),
            props: Box::new(Props::new),
            set: None,
        }).unwrap();
        let _session = login.serve(PATH, Offer {
            name: SESSION.into(),
            xml: String::new(),
            call: Box::new(|a: Asked| a.fail("org.freedesktop.DBus.Error.UnknownMethod", "none")),
            props: Box::new(|| Props::from([("LockedHint".to_string(), OwnedValue::from(false))])),
            set: None,
        }).unwrap();
        let heard = Rc::new(RefCell::new(Vec::new()));
        let held: Rc<RefCell<Option<Option<Listen>>>> = Rc::new(RefCell::new(None));
        let (h, k) = (heard.clone(), held.clone());
        on_locked(&us, &lp, move |l| h.borrow_mut().push(l), move |l| *k.borrow_mut() = Some(l));
        let h = heard.clone();
        assert!(until(&lp, move || h.borrow().len() == 1));
        assert!(held.borrow().as_ref().is_some_and(Option::is_some), "a Listen to keep");
        for locked in [true, false] {
            let changed: HashMap<&str, Value> = HashMap::from([("LockedHint", Value::from(locked))]);
            login.emit(PATH, "org.freedesktop.DBus.Properties", "PropertiesChanged", &(SESSION, changed, Vec::<String>::new())).unwrap();
        }
        let h = heard.clone();
        assert!(until(&lp, move || h.borrow().len() == 3));
        assert_eq!(*heard.borrow(), [false, true, false]);
    }

    #[test]
    fn with_no_logind_nothing_is_heard() {
        let d = own_daemon();
        let lp = Loop::new();
        let us = Bus::at(&lp, &d.1).expect("us");
        let held = Rc::new(RefCell::new(None));
        let k = held.clone();
        on_locked(&us, &lp, |_| panic!("nothing to hear"), move |l| *k.borrow_mut() = Some(l.is_some()));
        let h = held.clone();
        assert!(until(&lp, move || h.borrow().is_some()));
        assert_eq!(*held.borrow(), Some(false));
    }
}
