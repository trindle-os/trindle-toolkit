//! Trindle's link to D-Bus: calls, properties, signals, owned names and offered objects; a callback is taken out of its table before it runs.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::{Rc, Weak};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use futures_lite::StreamExt;
use serde::Serialize;
use zbus::message::{Flags, Type};
use zbus::zvariant::{DynamicType, OwnedObjectPath, OwnedValue, Value};
use zbus::{Connection, MatchRule, Message, MessageStream};

use super::event::{Loop, Post};
use crate::t;

pub type Props = HashMap<String, OwnedValue>;

type Task = zbus::Task<()>;

const DBUS: &str = "org.freedesktop.DBus";
const PROPERTIES: &str = "org.freedesktop.DBus.Properties";
const INTROSPECTABLE: &str = "org.freedesktop.DBus.Introspectable";
const BUSY: &str = "org.freedesktop.DBus.Error.LimitsExceeded";
const PEER: &str = "org.freedesktop.DBus.Peer";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Failed {
    pub name: String,
    pub text: String,
}

impl Failed {
    pub fn new(name: &str, text: &str) -> Failed {
        Failed { name: name.into(), text: text.into() }
    }
}

impl std::fmt::Display for Failed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(if self.text.is_empty() { &self.name } else { &self.text })
    }
}

impl From<zbus::Error> for Failed {
    fn from(e: zbus::Error) -> Failed {
        match e {
            zbus::Error::MethodError(name, text, _) => Failed { name: name.to_string(), text: text.unwrap_or_default() },
            e => Failed { name: String::new(), text: e.to_string() },
        }
    }
}

enum Back {
    Answered(u64, Result<Message, Failed>),
    Heard(u64, Message),
    Asked(Message),
    ServeEnded,
}

struct Waiting {
    done: Box<dyn FnOnce(Result<Message, Failed>)>,
    _task: Task,
}

struct Hearing {
    call: Option<Box<dyn FnMut(&Message)>>,
    queued: Vec<Message>,
    _task: Task,
}

pub struct Offer {
    pub name: String,
    pub xml: String,
    pub call: Box<dyn FnMut(Asked)>,
    pub props: Box<dyn Fn() -> Props>,
    pub set: Option<Box<dyn FnMut(&str, OwnedValue) -> Result<(), Failed>>>,
}

struct Inner {
    conn: Connection,
    post: Post<Back>,
    next: Cell<u64>,
    waiting: RefCell<HashMap<u64, Waiting>>,
    hearing: RefCell<HashMap<u64, Hearing>>,
    offered: RefCell<Vec<(String, Vec<(String, Option<Offer>)>)>>,
    serving: RefCell<Option<Task>>,
    queued: Arc<AtomicUsize>,
}

#[derive(Clone)]
pub struct Bus(Rc<Inner>);

struct Watched<A, V> {
    owner: Option<String>,
    heard: bool,
    appeared: A,
    vanished: V,
}
impl<A: FnMut(&str), V: FnMut()> Watched<A, V> {
    fn now(&mut self, owner: Option<String>) {
        if owner == self.owner {
            return;
        }
        self.owner = owner;
        match &self.owner {
            Some(o) => (self.appeared)(o),
            None => (self.vanished)(),
        }
    }
}

#[cfg(any(test, feature = "testing"))]
fn refused_for_tests(address: Option<&str>, uid: u32) -> Option<Failed> {
    let own = format!("unix:path=/run/user/{uid}/bus");
    let is_own = address.is_none_or(|a| a.is_empty() || a.split(';').any(|one| one == own || one.starts_with(&format!("{own},"))));
    is_own.then(|| Failed::new("org.trindle.Toolkit.Testing", "a test may not use the login's own session bus: run it under dbus-run-session"))
}

impl Bus {
    pub fn session(lp: &Loop) -> Result<Bus, Failed> {
        #[cfg(any(test, feature = "testing"))]
        if let Some(refused) = refused_for_tests(std::env::var("DBUS_SESSION_BUS_ADDRESS").ok().as_deref(), crate::sys::sock::own_uid()) {
            return Err(refused);
        }
        Self::open(lp, zbus::conn::Builder::session())
    }

    pub fn system(lp: &Loop) -> Result<Bus, Failed> {
        Self::open(lp, zbus::conn::Builder::system())
    }

    #[cfg(any(test, feature = "testing"))]
    pub fn at(lp: &Loop, address: &str) -> Result<Bus, Failed> {
        Self::open(lp, zbus::conn::Builder::address(address))
    }

    fn open(lp: &Loop, builder: zbus::Result<zbus::conn::Builder<'static>>) -> Result<Bus, Failed> {
        let conn = within(BUS_WAIT, builder?.build())?;
        let inner = Rc::new_cyclic(|me: &Weak<Inner>| {
            let me = me.clone();
            Inner {
                conn,
                post: lp.post(move |back| {
                    if let Some(inner) = me.upgrade() {
                        Bus(inner).arrived(back);
                    }
                }),
                next: Cell::new(1),
                waiting: RefCell::new(HashMap::new()),
                hearing: RefCell::new(HashMap::new()),
                offered: RefCell::new(Vec::new()),
                serving: RefCell::new(None),
                queued: Arc::new(AtomicUsize::new(0)),
            }
        });
        Ok(Bus(inner))
    }

    pub fn unique_name(&self) -> String {
        self.0.conn.unique_name().map(|n| n.to_string()).unwrap_or_default()
    }

    fn id(&self) -> u64 {
        let id = self.0.next.get();
        self.0.next.set(id + 1);
        id
    }

    fn arrived(&self, back: Back) {
        match back {
            Back::Answered(id, result) => {
                let done = self.0.waiting.borrow_mut().remove(&id).map(|w| w.done);
                if let Some(done) = done {
                    done(result);
                }
            }
            Back::Heard(id, msg) => {
                let call = self.0.hearing.borrow_mut().get_mut(&id).and_then(|h| h.call.take());
                let Some(mut call) = call else {
                    if let Some(h) = self.0.hearing.borrow_mut().get_mut(&id) {
                        h.queued.push(msg);
                    }
                    return;
                };
                call(&msg);
                loop {
                    let next = self.0.hearing.borrow_mut().get_mut(&id).and_then(|h| (!h.queued.is_empty()).then(|| h.queued.remove(0)));
                    match next {
                        Some(m) => call(&m),
                        None => break,
                    }
                }
                if let Some(h) = self.0.hearing.borrow_mut().get_mut(&id) {
                    h.call = Some(call);
                }
            }
            Back::Asked(msg) => {
                self.0.queued.fetch_sub(1, Ordering::Relaxed);
                self.asked(msg)
            }
            Back::ServeEnded => *self.0.serving.borrow_mut() = None,
        }
    }

    fn send_call(&self, msg: zbus::Result<Message>, wait: Option<Duration>, lp: Option<&Loop>, done: Box<dyn FnOnce(Result<Message, Failed>)>) {
        let msg = match msg {
            Ok(m) => m,
            Err(e) => return done(Err(e.into())),
        };
        let id = self.id();
        let (conn, post) = (self.0.conn.clone(), self.0.post.clone());
        let task = self.0.conn.executor().spawn(
            async move {
                let result = async {
                    let mut stream = MessageStream::from(&conn);
                    let serial = msg.primary_header().serial_num();
                    conn.send(&msg).await?;
                    while let Some(m) = stream.next().await {
                        let m = m?;
                        if m.header().reply_serial() == Some(serial) {
                            return Ok(m);
                        }
                    }
                    Err(zbus::Error::Failure(t!("kit-bus-closed").into()))
                }
                .await;
                post.send(Back::Answered(id, result.map_err(Failed::from).and_then(answer)));
            },
            "trindle bus call",
        );
        self.0.waiting.borrow_mut().insert(id, Waiting { done, _task: task });
        if let (Some(wait), Some(lp)) = (wait, lp) {
            let me = Rc::downgrade(&self.0);
            lp.after(wait, move || {
                let Some(inner) = me.upgrade() else { return };
                let waiting = inner.waiting.borrow_mut().remove(&id);
                if let Some(w) = waiting {
                    (w.done)(Err(Failed::new("org.freedesktop.DBus.Error.NoReply", t!("kit-bus-no-answer-in-time"))));
                }
            });
        }
    }

    fn listen_rule(&self, rule: zbus::Result<MatchRule<'_>>, call: Box<dyn FnMut(&Message)>) -> Result<Listen, Failed> {
        let rule = rule?;
        let mut stream = within(BUS_WAIT, MessageStream::for_match_rule(rule, &self.0.conn, Some(64)))?;
        let id = self.id();
        let post = self.0.post.clone();
        let task = self.0.conn.executor().spawn(
            async move {
                while let Some(m) = stream.next().await {
                    if let Ok(m) = m {
                        post.send(Back::Heard(id, m));
                    }
                }
            },
            "trindle bus listen",
        );
        self.0.hearing.borrow_mut().insert(id, Hearing { call: Some(call), queued: Vec::new(), _task: task });
        Ok(Listen { bus: Rc::downgrade(&self.0), id })
    }

    pub fn listen(
        &self,
        sender: Option<&str>,
        iface: &str,
        member: Option<&str>,
        path: Option<&str>,
        call: impl FnMut(&Message) + 'static,
    ) -> Result<Listen, Failed> {
        let rule = (|| {
            let mut b = MatchRule::builder().msg_type(Type::Signal).interface(iface)?;
            if let Some(s) = sender {
                b = b.sender(s)?;
            }
            if let Some(m) = member {
                b = b.member(m)?;
            }
            if let Some(p) = path {
                b = b.path(p)?;
            }
            Ok(b.build())
        })();
        self.listen_rule(rule, Box::new(call))
    }

    pub fn watch(
        &self,
        name: &str,
        lp: &Loop,
        appeared: impl FnMut(&str) + 'static,
        vanished: impl FnMut() + 'static,
    ) -> Result<Listen, Failed> {
        let state = Rc::new(RefCell::new(Watched { owner: None, heard: false, appeared, vanished }));
        let rule = (|| {
            Ok(MatchRule::builder()
                .msg_type(Type::Signal)
                .sender(DBUS)?
                .interface(DBUS)?
                .member("NameOwnerChanged")?
                .arg(0, name)?
                .build())
        })();
        let st = state.clone();
        let listen = self.listen_rule(
            rule,
            Box::new(move |msg| {
                if let Ok((_, _, new)) = msg.body().deserialize::<(String, String, String)>() {
                    let mut st = st.borrow_mut();
                    st.heard = true;
                    st.now(if new.is_empty() { None } else { Some(new) });
                }
            }),
        )?;
        let first = Rc::downgrade(&state);
        self.call(DBUS, "/org/freedesktop/DBus", DBUS).method("GetNameOwner", &(name,), false, None, lp, move |r| {
            let Some(state) = first.upgrade() else { return };
            let mut st = state.borrow_mut();
            if !st.heard {
                st.heard = true;
                let owner = r.ok().and_then(|m| m.body().deserialize::<String>().ok());
                if owner.is_none() {
                    (st.vanished)();
                } else {
                    st.now(owner);
                }
            }
        });
        Ok(listen)
    }

    pub fn own(&self, name: &str, replace: bool, lp: &Loop, done: impl FnOnce(bool) + 'static) {
        self.request(name, 1 | if replace { 2 } else { 0 } | 4, lp, done);
    }

    pub fn own_firmly(&self, name: &str, lp: &Loop, done: impl FnOnce(bool) + 'static) {
        self.request(name, 4, lp, done);
    }

    pub fn take_firmly(&self, name: &str, lp: &Loop, done: impl FnOnce(bool) + 'static) {
        self.request(name, 2 | 4, lp, done);
    }

    pub fn release(&self, name: &str, lp: &Loop) {
        self.call(DBUS, "/org/freedesktop/DBus", DBUS).method("ReleaseName", &(name,), false, None, lp, |_| {});
    }

    fn request(&self, name: &str, flags: u32, lp: &Loop, done: impl FnOnce(bool) + 'static) {
        self.call(DBUS, "/org/freedesktop/DBus", DBUS).method("RequestName", &(name, flags), false, None, lp, move |r| {
            done(matches!(r.ok().and_then(|m| m.body().deserialize::<u32>().ok()), Some(1) | Some(4)))
        });
    }

    pub fn call<'a>(&'a self, name: &'a str, path: &'a str, iface: &'a str) -> Call<'a> {
        Call { bus: self, name, path, iface, start: true }
    }

    pub fn emit<B: Serialize + DynamicType>(&self, path: &str, iface: &str, member: &str, body: &B) -> Result<(), Failed> {
        let msg = Message::signal(path, iface, member)?.build(body)?;
        Ok(within(BUS_WAIT, self.0.conn.send(&msg))?)
    }

    pub fn changed(&self, path: &str, iface: &str, props: &HashMap<&str, Value<'_>>) -> Result<(), Failed> {
        self.emit(path, PROPERTIES, "PropertiesChanged", &(iface, props, Vec::<&str>::new()))
    }

    pub fn serve(&self, path: &str, offer: Offer) -> Result<Served, Failed> {
        if !object_path(path) {
            return Err(Failed::new("org.freedesktop.DBus.Error.InvalidArgs", t!("kit-bus-bad-path")));
        }
        if self.0.serving.borrow().is_none() {
            let rule = MatchRule::builder().msg_type(Type::MethodCall).build();
            let mut stream = within(BUS_WAIT, MessageStream::for_match_rule(rule, &self.0.conn, Some(64)))?;
            let (post, queued, conn) = (self.0.post.clone(), self.0.queued.clone(), self.0.conn.clone());
            let task = self.0.conn.executor().spawn(
                async move {
                    while let Some(m) = stream.next().await {
                        let Ok(m) = m else { continue };
                        if queued.fetch_add(1, Ordering::Relaxed) >= MOST_QUEUED {
                            queued.fetch_sub(1, Ordering::Relaxed);
                            if !m.primary_header().flags().contains(Flags::NoReplyExpected) {
                                if let Ok(e) = Message::error(&m.header(), BUSY).and_then(|b| b.build(&t!("kit-bus-too-many-waiting"))) {
                                    let _ = conn.send(&e).await;
                                }
                            }
                            continue;
                        }
                        post.send(Back::Asked(m));
                    }
                    post.send(Back::ServeEnded);
                },
                "trindle bus serve",
            );
            *self.0.serving.borrow_mut() = Some(task);
        }
        let iface = offer.name.clone();
        let mut offered = self.0.offered.borrow_mut();
        if offered.iter().any(|(p, offers)| p == path && offers.iter().any(|(n, _)| *n == iface)) {
            return Err(Failed::new("org.freedesktop.DBus.Error.ObjectPathInUse", t!("kit-bus-already-offered")));
        }
        match offered.iter_mut().find(|(p, _)| p == path) {
            Some((_, offers)) => offers.push((iface.clone(), Some(offer))),
            None => offered.push((path.to_string(), vec![(iface.clone(), Some(offer))])),
        }
        Ok(Served { bus: Rc::downgrade(&self.0), path: path.to_string(), iface })
    }

    fn with_offer(&self, path: &str, iface: Option<&str>, f: impl FnOnce(&mut Offer)) -> bool {
        let taken = {
            let mut offered = self.0.offered.borrow_mut();
            offered.iter_mut().find(|(p, _)| p == path).and_then(|(_, offers)| {
                offers.iter_mut().find(|(n, o)| o.is_some() && iface.is_none_or(|i| n == i))?.1.take()
            })
        };
        let Some(mut offer) = taken else { return false };
        f(&mut offer);
        let back = {
            let mut offered = self.0.offered.borrow_mut();
            let slot = offered.iter_mut().find(|(p, _)| p == path)
                .and_then(|(_, offers)| offers.iter_mut().find(|(n, o)| o.is_none() && *n == offer.name));
            match slot {
                Some((_, slot)) => {
                    *slot = Some(offer);
                    None
                }
                None => Some(offer),
            }
        };
        drop(back);
        true
    }

    fn busy(&self, path: &str, iface: Option<&str>) -> bool {
        let offered = self.0.offered.borrow();
        offered.iter().find(|(p, _)| p == path).is_some_and(|(_, offers)| offers.iter().any(|(n, o)| o.is_none() && iface.is_none_or(|i| n == i)))
    }

    fn asked(&self, msg: Message) {
        let hdr = msg.header();
        let path = hdr.path().map(|p| p.to_string()).unwrap_or_default();
        let iface = hdr.interface().map(|i| i.to_string());
        let member = hdr.member().map(|m| m.to_string()).unwrap_or_default();
        let asked = Asked { conn: self.0.conn.clone(), msg: msg.clone(), answered: false };
        match (iface.as_deref(), member.as_str()) {
            (Some(PEER), "Ping") => asked.reply(&()),
            (Some(PEER), "GetMachineId") => match std::fs::read_to_string("/etc/machine-id") {
                Ok(id) => asked.reply(&id.trim()),
                Err(_) => asked.fail("org.freedesktop.DBus.Error.Failed", t!("kit-bus-no-machine-id")),
            },
            (Some(INTROSPECTABLE), "Introspect") => asked.reply(&self.introspect(&path)),
            (Some(PROPERTIES), "Get" | "GetAll" | "Set") => self.properties(&path, &member, asked),
            (iface, _) => {
                let mut asked = Some(asked);
                let known = self.0.offered.borrow().iter().any(|(p, _)| *p == path);
                if !self.with_offer(&path, iface, |o| (o.call)(asked.take().expect("asked once"))) {
                    let asked = asked.take().expect("asked once");
                    if self.busy(&path, iface) {
                        asked.fail(BUSY, t!("kit-bus-busy"));
                    } else if known {
                        asked.fail("org.freedesktop.DBus.Error.UnknownInterface", t!("kit-bus-no-interface"));
                    } else {
                        asked.fail("org.freedesktop.DBus.Error.UnknownObject", t!("kit-bus-no-object"));
                    }
                }
            }
        }
    }

    fn properties(&self, path: &str, member: &str, asked: Asked) {
        const NO_IFACE: &str = "org.freedesktop.DBus.Error.UnknownInterface";
        const NO_PROP: &str = "org.freedesktop.DBus.Error.UnknownProperty";
        let mut asked = Some(asked);
        let found = match member {
            "GetAll" => {
                let Ok(iface) = asked.as_ref().expect("asked").msg.body().deserialize::<String>() else {
                    return asked.take().expect("asked").fail("org.freedesktop.DBus.Error.InvalidArgs", t!("kit-bus-give-interface"));
                };
                self.with_offer(path, Some(&iface), |o| asked.take().expect("asked").reply(&(o.props)()))
            }
            "Get" => {
                let Ok((iface, prop)) = asked.as_ref().expect("asked").msg.body().deserialize::<(String, String)>() else {
                    return asked.take().expect("asked").fail("org.freedesktop.DBus.Error.InvalidArgs", t!("kit-bus-give-property"));
                };
                self.with_offer(path, Some(&iface), |o| {
                    let a = asked.take().expect("asked");
                    match (o.props)().remove(&prop) {
                        Some(v) => a.reply(&Value::from(v)),
                        None => a.fail(NO_PROP, t!("kit-bus-no-property")),
                    }
                })
            }
            _ => {
                let Ok((iface, prop, value)) = asked.as_ref().expect("asked").msg.body().deserialize::<(String, String, OwnedValue)>() else {
                    return asked.take().expect("asked").fail("org.freedesktop.DBus.Error.InvalidArgs", t!("kit-bus-give-value"));
                };
                self.with_offer(path, Some(&iface), |o| {
                    let a = asked.take().expect("asked");
                    match o.set.as_mut() {
                        Some(set) => match set(&prop, value) {
                            Ok(()) => a.reply(&()),
                            Err(f) => a.fail(&f.name, &f.text),
                        },
                        None => a.fail("org.freedesktop.DBus.Error.PropertyReadOnly", t!("kit-bus-read-only")),
                    }
                })
            }
        };
        if !found {
            if let Some(a) = asked.take() {
                if self.busy(path, None) {
                    a.fail(BUSY, t!("kit-bus-busy"));
                } else {
                    a.fail(NO_IFACE, t!("kit-bus-no-interface"));
                }
            }
        }
    }

    fn introspect(&self, path: &str) -> String {
        let offered = self.0.offered.borrow();
        let mut xml = String::from(
            "<!DOCTYPE node PUBLIC \"-//freedesktop//DTD D-BUS Object Introspection 1.0//EN\" \
             \"http://www.freedesktop.org/standards/dbus/1.0/introspect.dtd\">\n<node>\n",
        );
        if let Some((_, offers)) = offered.iter().find(|(p, _)| p == path) {
            xml.push_str(STANDARD);
            for o in offers.iter().filter_map(|(_, o)| o.as_ref()) {
                xml.push_str(&o.xml);
                xml.push('\n');
            }
        }
        let prefix = if path == "/" { "/".to_string() } else { format!("{path}/") };
        let mut children: Vec<&str> = offered
            .iter()
            .filter_map(|(p, _)| p.strip_prefix(&prefix))
            .filter_map(|rest| rest.split('/').next())
            .filter(|c| !c.is_empty())
            .collect();
        children.sort_unstable();
        children.dedup();
        for c in children {
            xml.push_str(&format!("  <node name=\"{c}\"/>\n"));
        }
        xml.push_str("</node>\n");
        xml
    }
}

const STANDARD: &str = "  <interface name=\"org.freedesktop.DBus.Peer\">\n    <method name=\"Ping\"/>\n    \
<method name=\"GetMachineId\"><arg type=\"s\" name=\"machine_uuid\" direction=\"out\"/></method>\n  </interface>\n  \
<interface name=\"org.freedesktop.DBus.Introspectable\">\n    <method name=\"Introspect\"><arg type=\"s\" name=\"xml_data\" direction=\"out\"/></method>\n  </interface>\n  \
<interface name=\"org.freedesktop.DBus.Properties\">\n    \
<method name=\"Get\"><arg type=\"s\" name=\"interface_name\" direction=\"in\"/><arg type=\"s\" name=\"property_name\" direction=\"in\"/><arg type=\"v\" name=\"value\" direction=\"out\"/></method>\n    \
<method name=\"GetAll\"><arg type=\"s\" name=\"interface_name\" direction=\"in\"/><arg type=\"a{sv}\" name=\"properties\" direction=\"out\"/></method>\n    \
<method name=\"Set\"><arg type=\"s\" name=\"interface_name\" direction=\"in\"/><arg type=\"s\" name=\"property_name\" direction=\"in\"/><arg type=\"v\" name=\"value\" direction=\"in\"/></method>\n    \
<signal name=\"PropertiesChanged\"><arg type=\"s\" name=\"interface_name\"/><arg type=\"a{sv}\" name=\"changed_properties\"/><arg type=\"as\" name=\"invalidated_properties\"/></signal>\n  </interface>\n";

fn answer(m: Message) -> Result<Message, Failed> {
    if m.message_type() != Type::Error {
        return Ok(m);
    }
    let name = m.header().error_name().map(|n| n.to_string()).unwrap_or_default();
    let text = m.body().deserialize::<String>().unwrap_or_default();
    Err(Failed { name, text })
}

pub struct Call<'a> {
    bus: &'a Bus,
    name: &'a str,
    path: &'a str,
    iface: &'a str,
    start: bool,
}

impl Call<'_> {
    pub fn never_start(self) -> Self {
        Call { start: false, ..self }
    }

    pub fn method<B: Serialize + DynamicType>(
        &self,
        method: &str,
        body: &B,
        interactive: bool,
        wait: Option<Duration>,
        lp: &Loop,
        done: impl FnOnce(Result<Message, Failed>) + 'static,
    ) {
        let msg = (|| {
            let mut b = Message::method_call(self.path, method)?.destination(self.name)?.interface(self.iface)?;
            if interactive {
                b = b.with_flags(Flags::AllowInteractiveAuth)?;
            }
            if !self.start {
                b = b.with_flags(Flags::NoAutoStart)?;
            }
            b.build(body)
        })();
        self.bus.send_call(msg, wait, Some(lp), Box::new(done));
    }

    pub fn get_all(&self, lp: &Loop, done: impl FnOnce(Result<Props, Failed>) + 'static) {
        let props = Call { iface: PROPERTIES, ..*self };
        props.method("GetAll", &(self.iface,), false, None, lp, move |r| {
            done(r.and_then(|m| m.body().deserialize::<Props>().map_err(Failed::from)))
        });
    }

    pub fn get(&self, prop: &str, lp: &Loop, done: impl FnOnce(Result<OwnedValue, Failed>) + 'static) {
        let props = Call { iface: PROPERTIES, ..*self };
        props.method("Get", &(self.iface, prop), false, None, lp, move |r| {
            done(r.and_then(|m| m.body().deserialize::<OwnedValue>().map_err(Failed::from)))
        });
    }

    pub fn set(&self, prop: &str, value: Value<'_>, interactive: bool, lp: &Loop, done: impl FnOnce(Result<(), Failed>) + 'static) {
        let props = Call { iface: PROPERTIES, ..*self };
        props.method("Set", &(self.iface, prop, value), interactive, None, lp, move |r| done(r.map(|_| ())));
    }
}

pub struct Listen {
    bus: Weak<Inner>,
    id: u64,
}

impl Drop for Listen {
    fn drop(&mut self) {
        if let Some(inner) = self.bus.upgrade() {
            let gone = inner.hearing.borrow_mut().remove(&self.id);
            drop(gone);
        }
    }
}

pub struct Served {
    bus: Weak<Inner>,
    path: String,
    iface: String,
}

impl Drop for Served {
    fn drop(&mut self) {
        let Some(inner) = self.bus.upgrade() else { return };
        let gone = {
            let mut offered = inner.offered.borrow_mut();
            let Some(at) = offered.iter().position(|(p, _)| *p == self.path) else { return };
            let offers = &mut offered[at].1;
            let gone = offers.iter().position(|(n, _)| *n == self.iface).and_then(|i| offers.remove(i).1);
            if offers.is_empty() {
                offered.remove(at);
            }
            gone
        };
        drop(gone);
    }
}

pub struct Asked {
    conn: Connection,
    msg: Message,
    answered: bool,
}

impl Asked {
    pub fn message(&self) -> &Message {
        &self.msg
    }

    pub fn member(&self) -> String {
        self.msg.header().member().map(|m| m.to_string()).unwrap_or_default()
    }

    pub fn sender(&self) -> String {
        self.msg.header().sender().map(|s| s.to_string()).unwrap_or_default()
    }

    fn wants_answer(&self) -> bool {
        !self.msg.primary_header().flags().contains(Flags::NoReplyExpected)
    }

    pub fn reply<B: Serialize + DynamicType>(mut self, body: &B) {
        self.answered = true;
        if self.wants_answer() {
            let built = Message::method_return(&self.msg.header()).and_then(|b| match self.conn.unique_name() {
                Some(me) => b.sender(me),
                None => Ok(b),
            }).and_then(|b| b.build(body));
            if let Ok(m) = built {
                send_soon(&self.conn, m);
            }
        }
    }

    pub fn fail(mut self, name: &str, text: &str) {
        self.answered = true;
        if self.wants_answer() {
            if let Ok(m) = Message::error(&self.msg.header(), name).and_then(|b| b.build(&text)) {
                send_soon(&self.conn, m);
            }
        }
    }
}

fn object_path(path: &str) -> bool {
    path == "/" || path.strip_prefix('/').is_some_and(|rest| rest.split('/').all(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')))
}

const MOST_QUEUED: usize = 256;

const BUS_WAIT: std::time::Duration = std::time::Duration::from_secs(5);

fn within<T>(limit: std::time::Duration, work: impl std::future::Future<Output = zbus::Result<T>>) -> zbus::Result<T> {
    zbus::block_on(futures_lite::future::or(work, async {
        async_io::Timer::after(limit).await;
        Err(zbus::Error::InputOutput(std::sync::Arc::new(std::io::ErrorKind::TimedOut.into())))
    }))
}

fn send_soon(conn: &Connection, m: Message) {
    let c = conn.clone();
    let mut sending = Box::pin(async move {
        let _ = c.send(&m).await;
    });
    if zbus::block_on(futures_lite::future::poll_once(&mut sending)).is_none() {
        conn.executor().spawn(sending, "trindle bus answer").detach();
    }
}

impl Drop for Asked {
    fn drop(&mut self) {
        if !self.answered && self.wants_answer() {
            if let Ok(m) = Message::error(&self.msg.header(), "org.freedesktop.DBus.Error.Failed").and_then(|b| b.build(&t!("kit-bus-unanswered"))) {
                send_soon(&self.conn, m);
            }
        }
    }
}

pub fn get_str(p: &Props, k: &str) -> String {
    p.get(k).and_then(|v| <&str>::try_from(v).ok()).unwrap_or_default().to_string()
}

pub fn get_path(p: &Props, k: &str) -> String {
    p.get(k)
        .and_then(|v| v.try_clone().ok())
        .and_then(|v| OwnedObjectPath::try_from(v).ok())
        .map(|p| p.to_string())
        .unwrap_or_default()
}

fn get<T: for<'a> TryFrom<&'a OwnedValue> + Default>(p: &Props, k: &str) -> T {
    p.get(k).and_then(|v| T::try_from(v).ok()).unwrap_or_default()
}

pub fn get_bool(p: &Props, k: &str) -> bool {
    get(p, k)
}

pub fn get_u32(p: &Props, k: &str) -> u32 {
    get(p, k)
}

pub fn get_f64(p: &Props, k: &str) -> f64 {
    get(p, k)
}

pub fn get_i64(p: &Props, k: &str) -> i64 {
    get(p, k)
}

pub fn get_paths(p: &Props, k: &str) -> Vec<String> {
    p.get(k)
        .and_then(|v| v.try_clone().ok())
        .and_then(|v| Vec::<OwnedObjectPath>::try_from(v).ok())
        .map(|ps| ps.into_iter().map(|p| p.to_string()).collect())
        .unwrap_or_default()
}

pub fn get_bytes(p: &Props, k: &str) -> Vec<u8> {
    p.get(k).and_then(|v| v.try_clone().ok()).and_then(|v| Vec::<u8>::try_from(v).ok()).unwrap_or_default()
}

#[cfg(any(test, feature = "testing"))]
pub mod testing {
    use super::super::event::{Flow, Loop};
    use std::cell::Cell;
    use std::io::{BufRead, BufReader};
    use std::process::{Child, Command, Stdio};
    use std::rc::Rc;
    use std::time::Duration;

    pub struct Daemon(Child, pub String);

    impl Daemon {
        pub fn signal(&self, name: &str) {
            assert!(Command::new("kill").args([name, &self.0.id().to_string()]).status().is_ok_and(|s| s.success()), "kill {name} did not reach the bus");
        }
    }

    impl Drop for Daemon {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }

    pub fn daemon() -> Option<Daemon> {
        let mut child = Command::new("dbus-daemon")
            .args(["--session", "--print-address=1", "--nofork", "--nopidfile"])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .ok()?;
        let mut line = String::new();
        BufReader::new(child.stdout.take()?).read_line(&mut line).ok()?;
        Some(Daemon(child, line.trim().to_string()))
    }

    pub fn own_daemon() -> Daemon {
        let d = daemon().expect("dbus-daemon could not be started, and this test checks nothing without a bus of its own: install dbus-daemon");
        assert!(!d.1.is_empty(), "dbus-daemon started and gave no address, so this test has no bus of its own");
        d
    }

    pub fn until(lp: &Loop, done: impl Fn() -> bool + 'static) -> bool {
        let ok = Rc::new(Cell::new(false));
        let (o, l, mut ticks) = (ok.clone(), lp.clone(), 0);
        lp.every(Duration::from_millis(5), move || {
            ticks += 1;
            if done() {
                o.set(true);
            }
            if o.get() || ticks > 1000 {
                l.quit();
                return Flow::Stop;
            }
            Flow::Continue
        });
        lp.run();
        ok.get()
    }
}

#[cfg(test)]
mod tests;
