//! Shortcuts that work while another program has the keyboard, through the global shortcuts portal: the person confirms the keys in the desktop's own settings.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use zbus::zvariant::{ObjectPath, OwnedObjectPath, OwnedValue, Value};

use super::bus::{Bus, Listen};
use super::event::Loop;
use super::portal::{request, token, Answer, Results};

const NAME: &str = "org.freedesktop.portal.Desktop";
const PATH: &str = "/org/freedesktop/portal/desktop";
const IFACE: &str = "org.freedesktop.portal.GlobalShortcuts";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Wanted {
    pub id: String,
    pub description: String,
    pub keys: String,
}

pub type Pressed = (String, bool);

pub struct GlobalKeys {
    bus: Bus,
    lp: Loop,
    session: Rc<RefCell<Option<String>>>,
    _listens: Rc<RefCell<Vec<Listen>>>,
}

fn shortcuts_body(wanted: &[Wanted]) -> Vec<(String, HashMap<&str, Value<'_>>)> {
    wanted.iter().map(|w| {
        let mut o = HashMap::from([("description", Value::from(w.description.as_str()))]);
        if !w.keys.is_empty() {
            o.insert("preferred_trigger", Value::from(w.keys.as_str()));
        }
        (w.id.clone(), o)
    }).collect()
}

fn bound(r: &Results) -> Option<Vec<(String, String)>> {
    let list = Vec::<(String, HashMap<String, OwnedValue>)>::try_from(r.get("shortcuts")?.try_clone().ok()?).ok()?;
    Some(list.into_iter().map(|(id, o)| {
        let keys = o.get("trigger_description").and_then(|v| String::try_from(v.try_clone().ok()?).ok()).unwrap_or_default();
        (id, keys)
    }).collect())
}

impl GlobalKeys {
    pub fn ask(bus: &Bus, lp: &Loop, wanted: Vec<Wanted>, pressed: impl FnMut(Pressed) + 'static,
               done: impl FnOnce(Answer<Vec<(String, String)>>) + 'static) -> GlobalKeys {
        let session: Rc<RefCell<Option<String>>> = Rc::default();
        let listens: Rc<RefCell<Vec<Listen>>> = Rc::default();
        let pressed = Rc::new(RefCell::new(pressed));
        let failed = Rc::new(std::cell::Cell::new(false));
        for (signal, down) in [("Activated", true), ("Deactivated", false)] {
            let (s, p) = (session.clone(), pressed.clone());
            let heard = bus.listen(Some(NAME), IFACE, Some(signal), Some(PATH), move |m| {
                let Ok((from, id, _, _)) = m.body().deserialize::<(OwnedObjectPath, String, u64, Results)>() else { return };
                if s.borrow().as_deref() == Some(from.as_str()) {
                    (p.borrow_mut())((id, down));
                }
            });
            match heard {
                Ok(l) => listens.borrow_mut().push(l),
                Err(_) => failed.set(true),
            }
        }
        let (asked, path) = token(bus);
        let (named, _) = token(bus);
        let options = HashMap::from([("handle_token", Value::from(asked.as_str())), ("session_handle_token", Value::from(named.as_str()))]);
        let (b, l, s, ls, bad) = (bus.clone(), lp.clone(), session.clone(), Rc::downgrade(&listens), failed.clone());
        let made = request(bus, lp, IFACE, "CreateSession", &(options,), &path, move |a| {
            let handle = match a {
                Answer::Given(r) => r.get("session_handle").and_then(|v| String::try_from(v.try_clone().ok()?).ok()),
                Answer::Cancelled => return done(Answer::Cancelled),
                Answer::Failed => None,
            };
            let (Some(handle), Some(ls)) = (handle, ls.upgrade()) else { return done(Answer::Failed) };
            if bad.get() {
                return done(Answer::Failed);
            }
            let Ok(at) = ObjectPath::try_from(handle.as_str()) else { return done(Answer::Failed) };
            *s.borrow_mut() = Some(handle.clone());
            let (asked, path) = token(&b);
            let options = HashMap::from([("handle_token", Value::from(asked.as_str()))]);
            let bind = request(&b, &l, IFACE, "BindShortcuts", &(at, shortcuts_body(&wanted), "", options), &path, move |a| done(match a {
                Answer::Given(r) => bound(&r).map_or(Answer::Failed, Answer::Given),
                Answer::Cancelled => Answer::Cancelled,
                Answer::Failed => Answer::Failed,
            }));
            ls.borrow_mut().extend(bind);
        });
        listens.borrow_mut().extend(made);
        GlobalKeys { bus: bus.clone(), lp: lp.clone(), session, _listens: listens }
    }
}

impl Drop for GlobalKeys {
    fn drop(&mut self) {
        if let Some(s) = self.session.borrow().as_deref() {
            self.bus.call(NAME, s, "org.freedesktop.portal.Session").method("Close", &(), false, None, &self.lp, |_| {});
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bus::testing::{own_daemon, until};
    use crate::bus::{Asked, Offer, Props};
    use std::cell::Cell;

    const SESSION: &str = "/org/freedesktop/portal/desktop/session/1_1/trindle";

    #[test]
    fn shortcuts_are_bound_and_their_presses_heard() {
        let d = own_daemon();
        let lp = Loop::new();
        let (p, us) = (Bus::at(&lp, &d.1).expect("portal"), Bus::at(&lp, &d.1).expect("us"));
        let asked = Rc::new(RefCell::new(Vec::new()));
        let (b, a2) = (p.clone(), asked.clone());
        let _served = p.serve(PATH, Offer {
            name: IFACE.into(),
            xml: String::new(),
            call: Box::new(move |a: Asked| {
                let member = a.member();
                let body = a.message().body();
                let options = match member.as_str() {
                    "CreateSession" => body.deserialize::<(Results,)>().map(|b| b.0),
                    _ => body.deserialize::<(OwnedObjectPath, Vec<(String, Results)>, String, Results)>().map(|b| {
                        a2.borrow_mut().extend(b.1.iter().map(|(id, o)| (id.clone(), String::try_from(o["preferred_trigger"].try_clone().unwrap()).unwrap())));
                        b.3
                    }),
                }.expect("the ask's shape");
                let token = String::try_from(options["handle_token"].try_clone().unwrap()).unwrap();
                let path = format!("{PATH}/request/{}/{token}", a.sender().trim_start_matches(':').replace('.', "_"));
                a.reply(&ObjectPath::try_from(path.as_str()).unwrap());
                let r: HashMap<&str, Value> = if member == "CreateSession" {
                    HashMap::from([("session_handle", Value::from(SESSION))])
                } else {
                    let one: Vec<(&str, HashMap<&str, Value>)> = vec![("show", HashMap::from([("trigger_description", Value::from("Ctrl+Alt+N"))]))];
                    HashMap::from([("shortcuts", Value::from(one))])
                };
                let _ = b.emit(&path, "org.freedesktop.portal.Request", "Response", &(0u32, r));
            }),
            props: Box::new(Props::new),
            set: None,
        }).expect("served");
        let owned = Rc::new(Cell::new(false));
        let o = owned.clone();
        p.own(NAME, false, &lp, move |ok| o.set(ok));
        assert!(until(&lp, move || owned.get()));

        let presses = Rc::new(RefCell::new(Vec::new()));
        let pr = presses.clone();
        let got = Rc::new(RefCell::new(None));
        let g = got.clone();
        let wanted = vec![Wanted { id: "show".into(), description: "Show the window".into(), keys: "CTRL+ALT+n".into() }];
        let _keys = GlobalKeys::ask(&us, &lp, wanted, move |p| pr.borrow_mut().push(p), move |a| *g.borrow_mut() = Some(a));
        let g = got.clone();
        assert!(until(&lp, move || g.borrow().is_some()));
        assert_eq!(got.borrow_mut().take(), Some(Answer::Given(vec![("show".to_string(), "Ctrl+Alt+N".to_string())])));
        assert_eq!(*asked.borrow(), [("show".to_string(), "CTRL+ALT+n".to_string())]);

        let none: HashMap<&str, Value> = HashMap::new();
        let other = ObjectPath::try_from("/org/freedesktop/portal/desktop/session/1_1/other").unwrap();
        p.emit(PATH, IFACE, "Activated", &(other, "show", 1u64, none.clone())).expect("signal");
        let ours = ObjectPath::try_from(SESSION).unwrap();
        p.emit(PATH, IFACE, "Activated", &(ours.clone(), "show", 2u64, none.clone())).expect("signal");
        p.emit(PATH, IFACE, "Deactivated", &(ours, "show", 3u64, none)).expect("signal");
        let pr = presses.clone();
        assert!(until(&lp, move || pr.borrow().len() == 2));
        assert_eq!(*presses.borrow(), [("show".to_string(), true), ("show".to_string(), false)]);
    }
}
