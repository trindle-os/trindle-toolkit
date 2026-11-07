//! A desktop notification and a tray icon over D-Bus; with no panel or daemon to show them they do nothing.

use crate::t;
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet, VecDeque};
use std::rc::Rc;
use std::sync::atomic::{AtomicU32, Ordering};

use zbus::zvariant::{OwnedValue, Value};

use super::bus::{Asked, Bus, Failed, Listen, Offer, Props, Served};
use super::event::Loop;

const NOTES: &str = "org.freedesktop.Notifications";
const NOTES_PATH: &str = "/org/freedesktop/Notifications";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Urgency {
    Low,
    #[default]
    Normal,
    Critical,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Note {
    pub app: String,
    pub icon: String,
    pub summary: String,
    pub body: String,
    pub actions: Vec<(String, String)>,
    pub urgency: Urgency,
    pub timeout: i32,
    pub reply: Option<String>,
}

impl Note {
    pub fn new(app: impl Into<String>, summary: impl Into<String>, body: impl Into<String>) -> Note {
        let body = body.into().replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;");
        Note { app: app.into(), icon: String::new(), summary: summary.into(), body, actions: Vec::new(), urgency: Urgency::Normal, timeout: -1, reply: None }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Why {
    Expired,
    Dismissed,
    Closed,
    Other,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Heard {
    Action { id: u32, key: String },
    Pass { id: u32, pass: String },
    Replied { id: u32, text: String },
    Gone { id: u32, why: Why },
}

struct Hear {
    call: RefCell<Box<dyn FnMut(Heard)>>,
    queued: RefCell<VecDeque<Heard>>,
}

impl Hear {
    fn tell(&self, event: Heard) {
        self.queued.borrow_mut().push_back(event);
        let Ok(mut call) = self.call.try_borrow_mut() else { return };
        loop {
            let next = self.queued.borrow_mut().pop_front();
            let Some(event) = next else { break };
            call(event);
        }
    }
}

pub struct Notifier {
    bus: Bus,
    mine: Rc<RefCell<HashSet<u32>>>,
    _listens: [Listen; 4],
}

impl Notifier {
    pub fn new(bus: &Bus, heard: impl FnMut(Heard) + 'static) -> Result<Notifier, Failed> {
        let heard = Rc::new(Hear { call: RefCell::new(Box::new(heard)), queued: RefCell::new(VecDeque::new()) });
        let mine = Rc::new(RefCell::new(HashSet::new()));
        let (h, m) = (heard.clone(), mine.clone());
        let action = bus.listen(Some(NOTES), NOTES, Some("ActionInvoked"), Some(NOTES_PATH), move |msg| {
            if let Ok((id, key)) = msg.body().deserialize::<(u32, String)>() {
                if m.borrow().contains(&id) {
                    h.tell(Heard::Action { id, key });
                }
            }
        })?;
        let (h, m) = (heard.clone(), mine.clone());
        let pass = bus.listen(Some(NOTES), NOTES, Some("ActivationToken"), Some(NOTES_PATH), move |msg| {
            if let Ok((id, pass)) = msg.body().deserialize::<(u32, String)>() {
                if m.borrow().contains(&id) {
                    h.tell(Heard::Pass { id, pass });
                }
            }
        })?;
        let (h, m) = (heard.clone(), mine.clone());
        let replied = bus.listen(Some(NOTES), NOTES, Some("NotificationReplied"), Some(NOTES_PATH), move |msg| {
            if let Ok((id, text)) = msg.body().deserialize::<(u32, String)>() {
                if m.borrow().contains(&id) {
                    h.tell(Heard::Replied { id, text });
                }
            }
        })?;
        let m = mine.clone();
        let gone = bus.listen(Some(NOTES), NOTES, Some("NotificationClosed"), Some(NOTES_PATH), move |msg| {
            if let Ok((id, why)) = msg.body().deserialize::<(u32, u32)>() {
                if !m.borrow_mut().remove(&id) {
                    return;
                }
                let why = match why {
                    1 => Why::Expired,
                    2 => Why::Dismissed,
                    3 => Why::Closed,
                    _ => Why::Other,
                };
                heard.tell(Heard::Gone { id, why });
            }
        })?;
        Ok(Notifier { bus: bus.clone(), mine, _listens: [action, pass, replied, gone] })
    }

    pub fn send(&self, lp: &Loop, note: &Note, replaces: u32, done: impl FnOnce(Result<u32, Failed>) + 'static) {
        let mut actions: Vec<&str> = note.actions.iter().flat_map(|(k, l)| [k.as_str(), l.as_str()]).collect();
        if note.reply.is_some() {
            actions.extend(["inline-reply", ""]);
        }
        let urgency: u8 = match note.urgency {
            Urgency::Low => 0,
            Urgency::Normal => 1,
            Urgency::Critical => 2,
        };
        let mut hints: HashMap<&str, Value<'_>> = HashMap::from([("urgency", Value::U8(urgency))]);
        if let Some(r) = &note.reply {
            hints.insert("x-kde-reply-placeholder-text", Value::from(r.as_str()));
        }
        let body = (note.app.as_str(), replaces, note.icon.as_str(), note.summary.as_str(), note.body.as_str(), actions, hints, note.timeout);
        let mine = self.mine.clone();
        self.bus.call(NOTES, NOTES_PATH, NOTES).method("Notify", &body, false, None, lp, move |r| {
            let id = r.and_then(|m| m.body().deserialize::<u32>().map_err(|e| Failed::new("", &e.to_string())));
            if let Ok(id) = id {
                mine.borrow_mut().insert(id);
            }
            done(id)
        });
    }

    pub fn close(&self, lp: &Loop, id: u32) {
        self.bus.call(NOTES, NOTES_PATH, NOTES).method("CloseNotification", &id, false, None, lp, |_| {});
    }
}

const ITEM_XML: &str = r#"  <interface name="org.kde.StatusNotifierItem">
    <method name="Activate"><arg type="i" direction="in"/><arg type="i" direction="in"/></method>
    <method name="SecondaryActivate"><arg type="i" direction="in"/><arg type="i" direction="in"/></method>
    <method name="ContextMenu"><arg type="i" direction="in"/><arg type="i" direction="in"/></method>
    <method name="Scroll"><arg type="i" direction="in"/><arg type="s" direction="in"/></method>
    <property name="Category" type="s" access="read"/>
    <property name="Id" type="s" access="read"/>
    <property name="Title" type="s" access="read"/>
    <property name="Status" type="s" access="read"/>
    <property name="IconName" type="s" access="read"/>
    <property name="ItemIsMenu" type="b" access="read"/>
    <property name="Menu" type="o" access="read"/>
    <signal name="NewIcon"/>
    <signal name="NewStatus"><arg type="s"/></signal>
    <signal name="NewTitle"/>
  </interface>"#;
const ITEM: &str = "org.kde.StatusNotifierItem";
const ITEM_PATH: &str = "/StatusNotifierItem";

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Click {
    Activate(i32, i32),
    Secondary(i32, i32),
    Menu(i32, i32),
    Scroll(i32, String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Passive,
    Active,
    Attention,
}

struct Shown {
    id: String,
    title: String,
    icon: String,
    status: Status,
}

pub struct Tray {
    bus: Bus,
    shown: Rc<RefCell<Shown>>,
    name: String,
    lp: Loop,
    _watch: Option<Listen>,
    _served: Served,
}

fn register(bus: &Bus, lp: &Loop, name: &str, then: impl FnOnce(bool) + 'static) {
    bus.call("org.kde.StatusNotifierWatcher", "/StatusNotifierWatcher", "org.kde.StatusNotifierWatcher")
        .method("RegisterStatusNotifierItem", &name, false, None, lp, move |r| then(r.is_ok()));
}

fn answer_click(a: Asked, click: &mut impl FnMut(Click)) {
    let two = a.message().body().deserialize::<(i32, i32)>().ok();
    let member = a.member();
    let heard = match (member.as_str(), two) {
        ("Activate", Some((x, y))) => Some(Click::Activate(x, y)),
        ("SecondaryActivate", Some((x, y))) => Some(Click::Secondary(x, y)),
        ("ContextMenu", Some((x, y))) => Some(Click::Menu(x, y)),
        ("Scroll", _) => a.message().body().deserialize::<(i32, String)>().ok().map(|(by, how)| Click::Scroll(by, how)),
        _ => None,
    };
    match heard {
        Some(c) => {
            click(c);
            a.reply(&());
        }
        None if matches!(member.as_str(), "Activate" | "SecondaryActivate" | "ContextMenu" | "Scroll") => {
            a.fail("org.freedesktop.DBus.Error.InvalidArgs", t!("kit-notify-bad-arguments"))
        }
        None => a.fail("org.freedesktop.DBus.Error.UnknownMethod", t!("kit-bus-no-method-here")),
    }
}

fn item_props(s: &Shown) -> Props {
    let text = |v: &str| Value::from(v).try_into().expect("plain value");
    let mut p = Props::new();
    p.insert("Category".into(), text("ApplicationStatus"));
    p.insert("Id".into(), text(&s.id));
    p.insert("Title".into(), text(&s.title));
    p.insert("IconName".into(), text(&s.icon));
    p.insert("Status".into(), text(status_word(s.status)));
    p.insert("ItemIsMenu".into(), OwnedValue::from(false));
    p.insert("Menu".into(), Value::from(zbus::zvariant::ObjectPath::try_from("/NO_DMENU").expect("a path")).try_into().expect("plain value"));
    p
}

static TRAYS: AtomicU32 = AtomicU32::new(1);

impl Tray {
    pub fn new(bus: &Bus, lp: &Loop, id: &str, title: &str, icon: &str, mut click: impl FnMut(Click) + 'static, done: impl FnOnce(bool) + 'static) -> Result<Tray, Failed> {
        let shown = Rc::new(RefCell::new(Shown { id: id.into(), title: title.into(), icon: icon.into(), status: Status::Active }));
        let props = shown.clone();
        let served = bus.serve(ITEM_PATH, Offer {
            name: ITEM.into(),
            xml: ITEM_XML.into(),
            call: Box::new(move |a: Asked| answer_click(a, &mut click)),
            props: Box::new(move || item_props(&props.borrow())),
            set: None,
        })?;
        let name = format!("org.kde.StatusNotifierItem-{}-{}", std::process::id(), TRAYS.fetch_add(1, Ordering::Relaxed));
        let lost = Rc::new(Cell::new(false));
        let (b, n, ll, gone) = (bus.clone(), name.clone(), lp.clone(), lost.clone());
        bus.own(&name, false, lp, move |owned| {
            if !owned {
                return done(false);
            }
            register(&b, &ll, &n, move |ok| {
                gone.set(!ok);
                done(ok);
            });
        });
        let (b, n, ll, gone) = (bus.clone(), name.clone(), lp.clone(), lost.clone());
        let vanished = lost.clone();
        let watch = bus.watch("org.kde.StatusNotifierWatcher", lp, move |_| {
            if gone.replace(false) {
                register(&b, &ll, &n, |_| {});
            }
        }, move || vanished.set(true)).ok();
        Ok(Tray { bus: bus.clone(), shown, name, lp: lp.clone(), _watch: watch, _served: served })
    }

    pub fn set_icon(&self, icon: &str) {
        self.shown.borrow_mut().icon = icon.into();
        let _ = self.bus.emit(ITEM_PATH, ITEM, "NewIcon", &());
    }

    pub fn set_title(&self, title: &str) {
        self.shown.borrow_mut().title = title.into();
        let _ = self.bus.emit(ITEM_PATH, ITEM, "NewTitle", &());
    }

    pub fn set_status(&self, status: Status) {
        self.shown.borrow_mut().status = status;
        let _ = self.bus.emit(ITEM_PATH, ITEM, "NewStatus", &status_word(status));
    }
}

impl Drop for Tray {
    fn drop(&mut self) {
        self.bus.release(&self.name, &self.lp);
    }
}

fn status_word(s: Status) -> &'static str {
    match s {
        Status::Passive => "Passive",
        Status::Active => "Active",
        Status::Attention => "NeedsAttention",
    }
}

#[cfg(test)]
mod tests {
    use super::super::bus::testing::{own_daemon, until};
    use super::super::bus::get_str;
    use super::*;
    use std::cell::Cell;

    fn service(bus: &Bus, lp: &Loop, got: Rc<RefCell<Vec<(u32, String, Vec<String>, i32)>>>) -> Served {
        let xml = r#"  <interface name="org.freedesktop.Notifications"><method name="Notify"/><method name="CloseNotification"/></interface>"#;
        let served = bus.serve(NOTES_PATH, Offer {
            name: NOTES.into(),
            xml: xml.into(),
            call: Box::new(move |a: Asked| match a.member().as_str() {
                "Notify" => {
                    let (_, replaces, _, summary, _, actions, _, timeout) =
                        a.message().body().deserialize::<(String, u32, String, String, String, Vec<String>, HashMap<String, OwnedValue>, i32)>().expect("a Notify body");
                    got.borrow_mut().push((replaces, summary, actions, timeout));
                    a.reply(&42u32);
                }
                _ => a.reply(&()),
            }),
            props: Box::new(Props::new),
            set: None,
        }).expect("served");
        let owned = Rc::new(Cell::new(false));
        let o = owned.clone();
        bus.own(NOTES, false, lp, move |ok| o.set(ok));
        assert!(until(lp, move || owned.get()));
        served
    }

    #[test]
    fn a_notification_is_sent_and_its_action_and_closing_are_heard() {
        let d = own_daemon();
        let lp = Loop::new();
        let (svc, us) = (Bus::at(&lp, &d.1).expect("service"), Bus::at(&lp, &d.1).expect("us"));
        let got = Rc::new(RefCell::new(Vec::new()));
        let _served = service(&svc, &lp, got.clone());
        let heard = Rc::new(RefCell::new(Vec::new()));
        let h = heard.clone();
        let notifier = Notifier::new(&us, move |e| h.borrow_mut().push(e)).expect("listening");
        let mut note = Note::new("Test", "Done", "It finished.");
        note.actions.push(("open".into(), "Open".into()));
        note.timeout = 5000;
        let id = Rc::new(Cell::new(0));
        let i = id.clone();
        notifier.send(&lp, &note, 0, move |r| i.set(r.expect("numbered")));
        let i = id.clone();
        assert!(until(&lp, move || i.get() != 0));
        assert_eq!(id.get(), 42);
        assert_eq!(got.borrow().as_slice(), [(0, "Done".to_string(), vec!["open".to_string(), "Open".to_string()], 5000)]);

        svc.emit(NOTES_PATH, NOTES, "ActionInvoked", &(7u32, "open")).expect("signal");
        svc.emit(NOTES_PATH, NOTES, "ActionInvoked", &(42u32, "open")).expect("signal");
        svc.emit(NOTES_PATH, NOTES, "NotificationClosed", &(42u32, 2u32)).expect("signal");
        let h = heard.clone();
        assert!(until(&lp, move || h.borrow().len() == 2));
        assert_eq!(*heard.borrow(), [Heard::Action { id: 42, key: "open".into() }, Heard::Gone { id: 42, why: Why::Dismissed }]);
    }

    #[test]
    fn a_reply_box_is_offered_and_the_pass_and_reply_are_heard() {
        let d = own_daemon();
        let lp = Loop::new();
        let (svc, us) = (Bus::at(&lp, &d.1).expect("service"), Bus::at(&lp, &d.1).expect("us"));
        let got = Rc::new(RefCell::new(Vec::new()));
        let _served = service(&svc, &lp, got.clone());
        let heard = Rc::new(RefCell::new(Vec::new()));
        let h = heard.clone();
        let notifier = Notifier::new(&us, move |e| h.borrow_mut().push(e)).expect("listening");
        let mut note = Note::new("Test", "John Smith", "Are you coming?");
        note.reply = Some("Reply".into());
        let id = Rc::new(Cell::new(0));
        let i = id.clone();
        notifier.send(&lp, &note, 0, move |r| i.set(r.expect("numbered")));
        let i = id.clone();
        assert!(until(&lp, move || i.get() != 0));
        assert_eq!(got.borrow()[0].2, ["inline-reply", ""]);

        svc.emit(NOTES_PATH, NOTES, "ActivationToken", &(7u32, "not-ours")).expect("signal");
        svc.emit(NOTES_PATH, NOTES, "ActivationToken", &(42u32, "pass-1")).expect("signal");
        svc.emit(NOTES_PATH, NOTES, "NotificationReplied", &(42u32, "On my way")).expect("signal");
        let h = heard.clone();
        assert!(until(&lp, move || h.borrow().len() == 2));
        assert_eq!(*heard.borrow(), [Heard::Pass { id: 42, pass: "pass-1".into() }, Heard::Replied { id: 42, text: "On my way".into() }]);
    }

    #[test]
    fn the_body_of_a_note_is_plain_text_until_it_is_set_as_markup() {
        assert_eq!(Note::new("Test", "Done", "a <b>&</b>").body, "a &lt;b&gt;&amp;&lt;/b&gt;");
    }

    #[test]
    fn a_tray_icon_registers_with_the_watcher_and_hears_clicks() {
        let d = own_daemon();
        let lp = Loop::new();
        let (watcher, us, panel) = (Bus::at(&lp, &d.1).expect("watcher"), Bus::at(&lp, &d.1).expect("us"), Bus::at(&lp, &d.1).expect("panel"));
        let told = Rc::new(RefCell::new(Vec::new()));
        let t = told.clone();
        let xml = r#"  <interface name="org.kde.StatusNotifierWatcher"><method name="RegisterStatusNotifierItem"/></interface>"#;
        let _served = watcher.serve("/StatusNotifierWatcher", Offer {
            name: "org.kde.StatusNotifierWatcher".into(),
            xml: xml.into(),
            call: Box::new(move |a: Asked| {
                if let Ok(who) = a.message().body().deserialize::<String>() {
                    t.borrow_mut().push(who);
                }
                a.reply(&());
            }),
            props: Box::new(Props::new),
            set: None,
        }).expect("served");
        let owned = Rc::new(Cell::new(false));
        let o = owned.clone();
        watcher.own("org.kde.StatusNotifierWatcher", false, &lp, move |ok| o.set(ok));
        assert!(until(&lp, move || owned.get()));

        let clicks = Rc::new(RefCell::new(Vec::new()));
        let c = clicks.clone();
        let registered = Rc::new(Cell::new(None));
        let r = registered.clone();
        let tray = Tray::new(&us, &lp, "org.trindle.Test", "Test", "mail-unread", move |e| c.borrow_mut().push(e), move |ok| r.set(Some(ok))).expect("tray");
        let r = registered.clone();
        assert!(until(&lp, move || r.get().is_some()));
        assert_eq!(registered.get(), Some(true));
        let name = told.borrow()[0].clone();
        assert!(name.starts_with("org.kde.StatusNotifierItem-"));

        let item = panel.call(&name, ITEM_PATH, ITEM);
        let props = Rc::new(RefCell::new(None));
        let p = props.clone();
        item.get_all(&lp, move |r| *p.borrow_mut() = r.ok());
        let p = props.clone();
        assert!(until(&lp, move || p.borrow().is_some()));
        let all = props.borrow_mut().take().unwrap();
        assert_eq!((get_str(&all, "Id").as_str(), get_str(&all, "IconName").as_str(), get_str(&all, "Status").as_str()), ("org.trindle.Test", "mail-unread", "Active"));
        tray.set_status(Status::Attention);
        let again = Rc::new(RefCell::new(None));
        let p = again.clone();
        item.get_all(&lp, move |r| *p.borrow_mut() = r.ok());
        let p = again.clone();
        assert!(until(&lp, move || p.borrow().is_some()));
        assert_eq!(get_str(again.borrow().as_ref().unwrap(), "Status"), "NeedsAttention");

        for (member, body) in [("Activate", (3i32, 4i32)), ("ContextMenu", (5, 6))] {
            item.method(member, &body, false, None, &lp, |_| {});
        }
        let c = clicks.clone();
        assert!(until(&lp, move || c.borrow().len() == 2));
        assert_eq!(*clicks.borrow(), [Click::Activate(3, 4), Click::Menu(5, 6)]);
    }
}
