//! Tests of bus.

use super::testing::{own_daemon, until};
use super::*;

#[test]
fn a_test_never_reaches_the_logins_own_session_bus() {
    for own in [None, Some(""), Some("unix:path=/run/user/1000/bus"), Some("unix:path=/run/user/1000/bus,guid=ab12"), Some("unix:path=/tmp/made;unix:path=/run/user/1000/bus")] {
        let refused = refused_for_tests(own, 1000).expect("refused");
        assert!(refused.text.contains("dbus-run-session"), "{own:?}");
    }
    for made in ["unix:path=/tmp/dbus-AbC", "unix:abstract=/tmp/dbus-AbC,guid=ab12", "unix:path=/run/user/1001/bus", "unix:path=/run/user/1000/bus2"] {
        assert!(refused_for_tests(Some(made), 1000).is_none(), "{made}");
    }
}

#[test]
fn a_wait_on_a_stalled_bus_ends_at_the_limit() {
    let start = std::time::Instant::now();
    let r: zbus::Result<()> = within(std::time::Duration::from_millis(50), std::future::pending());
    assert!(matches!(r, Err(zbus::Error::InputOutput(_))));
    assert!(start.elapsed() < std::time::Duration::from_secs(2));
    assert_eq!(within(std::time::Duration::from_secs(5), async { Ok(3) }).ok(), Some(3));
}

fn slot<T>() -> Rc<RefCell<Option<T>>> {
    Rc::new(RefCell::new(None))
}

const XML: &str = "  <interface name=\"org.trindle.Test\">\n    <method name=\"Add\"><arg type=\"i\" direction=\"in\"/><arg type=\"i\" direction=\"in\"/><arg type=\"i\" direction=\"out\"/></method>\n    <property name=\"Name\" type=\"s\" access=\"readwrite\"/>\n  </interface>";

fn offer(name: Rc<RefCell<String>>, kept: Rc<RefCell<Vec<Asked>>>) -> Offer {
    let n = name.clone();
    Offer {
        name: "org.trindle.Test".into(),
        xml: XML.into(),
        call: Box::new(move |a: Asked| match a.member().as_str() {
            "Add" => match a.message().body().deserialize::<(i32, i32)>() {
                Ok((x, y)) => a.reply(&(x + y)),
                Err(_) => a.fail("org.freedesktop.DBus.Error.InvalidArgs", "Two numbers."),
            },
            "Never" => kept.borrow_mut().push(a),
            "Hold" => {
                use std::io::Write;
                let (ours, mut theirs) = std::os::unix::net::UnixStream::pair().expect("pair");
                theirs.write_all(b"held").expect("write");
                a.reply(&zbus::zvariant::OwnedFd::from(std::os::fd::OwnedFd::from(ours)));
            }
            _ => a.fail("org.freedesktop.DBus.Error.UnknownMethod", "No such method."),
        }),
        props: Box::new(move || {
            let mut p = Props::new();
            p.insert("Name".into(), Value::from(n.borrow().as_str()).try_into().expect("plain value"));
            p
        }),
        set: Some(Box::new(move |prop, v| match (prop, <&str>::try_from(&v)) {
            ("Name", Ok(s)) => {
                *name.borrow_mut() = s.to_string();
                Ok(())
            }
            _ => Err(Failed::new("org.freedesktop.DBus.Error.InvalidArgs", "Only Name, as words.")),
        })),
    }
}

fn wait_for<T: 'static>(lp: &Loop, s: &Rc<RefCell<Option<T>>>) {
    let s = s.clone();
    assert!(until(lp, move || s.borrow().is_some()));
}

#[test]
fn calls_properties_and_introspection_reach_an_object_we_offer() {
    let d = own_daemon();
    let lp = Loop::new();
    let (a, b) = (Bus::at(&lp, &d.1).expect("bus a"), Bus::at(&lp, &d.1).expect("bus b"));
    let kept = Rc::new(RefCell::new(Vec::new()));
    let _served = a.serve("/org/trindle/Test", offer(Rc::new(RefCell::new("Ann".into())), kept)).expect("served");
    let who = a.unique_name();
    let t = b.call(&who, "/org/trindle/Test", "org.trindle.Test");

    let sum = slot();
    let s = sum.clone();
    t.method("Add", &(2i32, 3i32), false, None, &lp, move |r| *s.borrow_mut() = Some(r.map(|m| m.body().deserialize::<i32>().ok())));
    wait_for(&lp, &sum);
    assert_eq!(sum.borrow_mut().take(), Some(Ok(Some(5))));

    let all = slot();
    let s = all.clone();
    t.get_all(&lp, move |r| *s.borrow_mut() = Some(r.map(|p| get_str(&p, "Name"))));
    wait_for(&lp, &all);
    assert_eq!(all.borrow_mut().take(), Some(Ok("Ann".to_string())));

    let set = slot();
    let s = set.clone();
    t.set("Name", Value::from("Bob"), false, &lp, move |r| *s.borrow_mut() = Some(r));
    wait_for(&lp, &set);
    assert_eq!(set.borrow_mut().take(), Some(Ok(())));
    let s = set.clone();
    t.set("Name", Value::from(7u32), false, &lp, move |r| *s.borrow_mut() = Some(r));
    wait_for(&lp, &set);
    assert_eq!(set.borrow_mut().take().and_then(|r| r.err()).map(|f| f.name), Some("org.freedesktop.DBus.Error.InvalidArgs".into()));

    let one = slot();
    let s = one.clone();
    t.get("Name", &lp, move |r| *s.borrow_mut() = Some(r.ok().and_then(|v| String::try_from(v).ok())));
    wait_for(&lp, &one);
    assert_eq!(one.borrow_mut().take(), Some(Some("Bob".to_string())));

    let xml = slot();
    for path in ["/", "/org/trindle/Test"] {
        let s = xml.clone();
        b.call(&who, path, INTROSPECTABLE).method("Introspect", &(), false, None, &lp, move |r| {
            *s.borrow_mut() = r.ok().and_then(|m| m.body().deserialize::<String>().ok())
        });
        wait_for(&lp, &xml);
        let got = xml.borrow_mut().take().unwrap_or_default();
        if path == "/" {
            assert!(got.contains("<node name=\"org\"/>"), "{got}");
        } else {
            assert!(got.contains("org.trindle.Test") && got.contains("org.freedesktop.DBus.Properties"), "{got}");
        }
    }

    let gone = slot();
    let s = gone.clone();
    b.call(&who, "/nowhere", "org.trindle.Test").method("Add", &(1i32, 1i32), false, None, &lp, move |r| {
        *s.borrow_mut() = Some(r.err().map(|f| f.name))
    });
    wait_for(&lp, &gone);
    assert_eq!(gone.borrow_mut().take(), Some(Some("org.freedesktop.DBus.Error.UnknownObject".into())));
}

#[test]
fn a_file_handed_over_in_an_answer_arrives_open() {
    use std::io::Read;
    let d = own_daemon();
    let lp = Loop::new();
    let (a, b) = (Bus::at(&lp, &d.1).expect("bus a"), Bus::at(&lp, &d.1).expect("bus b"));
    let _served = a.serve("/t", offer(Rc::new(RefCell::new(String::new())), Rc::default())).expect("served");
    let got = slot();
    let s = got.clone();
    b.call(&a.unique_name(), "/t", "org.trindle.Test").method("Hold", &(), false, None, &lp, move |r| {
        *s.borrow_mut() = r.ok().and_then(|m| m.body().deserialize::<zbus::zvariant::OwnedFd>().ok())
    });
    let s = got.clone();
    assert!(until(&lp, move || s.borrow().is_some()));
    let fd = std::os::fd::OwnedFd::from(got.borrow_mut().take().expect("a file"));
    let mut word = [0u8; 4];
    std::os::unix::net::UnixStream::from(fd).read_exact(&mut word).expect("read");
    assert_eq!(&word, b"held");
}

#[test]
fn an_unanswered_call_fails_once_its_wait_is_up_and_a_dropped_ask_fails() {
    let d = own_daemon();
    let lp = Loop::new();
    let (a, b) = (Bus::at(&lp, &d.1).expect("bus a"), Bus::at(&lp, &d.1).expect("bus b"));
    let kept = Rc::new(RefCell::new(Vec::new()));
    let _served = a.serve("/t", offer(Rc::new(RefCell::new(String::new())), kept.clone())).expect("served");
    let who = a.unique_name();
    let got = slot();
    let s = got.clone();
    b.call(&who, "/t", "org.trindle.Test").method("Never", &(), false, Some(Duration::from_millis(100)), &lp, move |r| {
        *s.borrow_mut() = Some(r.err().map(|f| f.name))
    });
    let s = got.clone();
    assert!(until(&lp, move || s.borrow().is_some()));
    assert_eq!(got.borrow_mut().take(), Some(Some("org.freedesktop.DBus.Error.NoReply".into())));

    let s = got.clone();
    b.call(&who, "/t", "org.trindle.Test").method("Never", &(), false, None, &lp, move |r| *s.borrow_mut() = Some(r.err().map(|f| f.name)));
    let k = kept.clone();
    assert!(until(&lp, move || k.borrow().len() == 2));
    kept.borrow_mut().clear();
    let s = got.clone();
    assert!(until(&lp, move || s.borrow().is_some()));
    assert_eq!(got.borrow_mut().take(), Some(Some("org.freedesktop.DBus.Error.Failed".into())));
}

#[test]
fn an_answer_to_a_bus_that_has_stopped_does_not_hold_the_caller_and_arrives_when_it_goes_on() {
    let d = own_daemon();
    let lp = Loop::new();
    let (a, b) = (Bus::at(&lp, &d.1).expect("bus a"), Bus::at(&lp, &d.1).expect("bus b"));
    let kept = Rc::new(RefCell::new(Vec::new()));
    let _served = a.serve("/t", offer(Rc::new(RefCell::new(String::new())), kept.clone())).expect("served");
    let got = slot();
    let s = got.clone();
    b.call(&a.unique_name(), "/t", "org.trindle.Test").method("Never", &(), false, Some(Duration::from_secs(60)), &lp, move |r| {
        *s.borrow_mut() = Some(r.ok().and_then(|m| m.body().deserialize::<String>().ok()))
    });
    let k = kept.clone();
    assert!(until(&lp, move || k.borrow().len() == 1));
    let asked = kept.borrow_mut().pop().expect("the call");

    d.signal("-STOP");
    let (words, began) = ("a long answer ".repeat(1 << 19), std::time::Instant::now());
    asked.reply(&words);
    let took = began.elapsed();
    d.signal("-CONT");
    assert!(took < Duration::from_secs(1), "an answer of {} bytes held the caller for {took:?}", words.len());

    let s = got.clone();
    assert!(until(&lp, move || s.borrow().is_some()));
    assert_eq!(got.borrow_mut().take(), Some(Some(words)));
}

#[test]
fn signals_are_heard_until_the_listen_is_dropped() {
    let d = own_daemon();
    let lp = Loop::new();
    let (a, b) = (Bus::at(&lp, &d.1).expect("bus a"), Bus::at(&lp, &d.1).expect("bus b"));
    let heard = Rc::new(RefCell::new(Vec::new()));
    let h = heard.clone();
    let listen = b
        .listen(Some(&a.unique_name()), "org.trindle.Test", Some("Rang"), None, move |m| {
            h.borrow_mut().push(m.body().deserialize::<String>().unwrap_or_default())
        })
        .expect("listening");
    a.emit("/t", "org.trindle.Test", "Rang", &("one",)).expect("sent");
    a.emit("/t", "org.trindle.Test", "Other", &("no",)).expect("sent");
    a.emit("/t", "org.trindle.Test", "Rang", &("two",)).expect("sent");
    let h = heard.clone();
    assert!(until(&lp, move || h.borrow().len() == 2));
    assert_eq!(*heard.borrow(), ["one", "two"]);
    drop(listen);
    a.emit("/t", "org.trindle.Test", "Rang", &("three",)).expect("sent");
    let w = Rc::new(Cell::new(false));
    let w2 = w.clone();
    lp.after(Duration::from_millis(300), move || w2.set(true));
    until(&lp, move || w.get());
    assert_eq!(heard.borrow().len(), 2);
}

#[test]
fn a_signal_that_comes_while_the_listener_runs_is_kept_for_after() {
    let d = own_daemon();
    let lp = Loop::new();
    let (a, b) = (Bus::at(&lp, &d.1).expect("bus a"), Bus::at(&lp, &d.1).expect("bus b"));
    let heard = Rc::new(RefCell::new(Vec::new()));
    let (h, l) = (heard.clone(), lp.clone());
    let _listen = b
        .listen(Some(&a.unique_name()), "org.trindle.Test", Some("Rang"), None, move |m| {
            let first = h.borrow().is_empty();
            h.borrow_mut().push(m.body().deserialize::<String>().unwrap_or_default());
            if first {
                let flag = Rc::new(Cell::new(false));
                let set = flag.clone();
                l.after(Duration::from_millis(200), move || set.set(true));
                until(&l, move || flag.get());
                assert_eq!(h.borrow().len(), 1, "the second was not run inside the first");
            }
        })
        .expect("listening");
    a.emit("/t", "org.trindle.Test", "Rang", &("one",)).expect("sent");
    a.emit("/t", "org.trindle.Test", "Rang", &("two",)).expect("sent");
    let h = heard.clone();
    until(&lp, move || h.borrow().len() == 2);
    assert_eq!(*heard.borrow(), ["one", "two"]);
}

#[test]
fn a_watched_name_is_seen_to_come_and_go() {
    let d = own_daemon();
    let lp = Loop::new();
    let b = Bus::at(&lp, &d.1).expect("bus b");
    let seen = Rc::new(RefCell::new(Vec::new()));
    let (s1, s2) = (seen.clone(), seen.clone());
    let _w = b
        .watch("org.example.Name", &lp, move |o| s1.borrow_mut().push(format!("in {o}")), move || s2.borrow_mut().push("out".into()))
        .expect("watching");
    let s = seen.clone();
    assert!(until(&lp, move || s.borrow().len() == 1));
    let a = Bus::at(&lp, &d.1).expect("bus a");
    let owned = slot();
    let o = owned.clone();
    a.own("org.example.Name", false, &lp, move |ok| *o.borrow_mut() = Some(ok));
    let (s, o) = (seen.clone(), owned.clone());
    assert!(until(&lp, move || s.borrow().len() == 2 && o.borrow().is_some()));
    assert_eq!(*owned.borrow(), Some(true));
    let who = a.unique_name();
    drop(a);
    let s = seen.clone();
    assert!(until(&lp, move || s.borrow().len() == 3));
    assert_eq!(*seen.borrow(), ["out".to_string(), format!("in {who}"), "out".to_string()]);
}

#[test]
fn a_firmly_owned_name_is_not_replaced() {
    let d = own_daemon();
    let lp = Loop::new();
    let a = Bus::at(&lp, &d.1).expect("bus a");
    let b = Bus::at(&lp, &d.1).expect("bus b");
    let (first, second) = (slot(), slot());
    let o = first.clone();
    a.own_firmly("org.example.Firm", &lp, move |ok| *o.borrow_mut() = Some(ok));
    let o = first.clone();
    assert!(until(&lp, move || o.borrow().is_some()));
    let o = second.clone();
    b.own("org.example.Firm", true, &lp, move |ok| *o.borrow_mut() = Some(ok));
    let o = second.clone();
    assert!(until(&lp, move || o.borrow().is_some()));
    assert_eq!((*first.borrow(), *second.borrow()), (Some(true), Some(false)));
}

#[test]
fn the_same_interface_cannot_be_offered_twice_at_one_path() {
    let d = own_daemon();
    let lp = Loop::new();
    let a = Bus::at(&lp, &d.1).expect("bus a");
    let _first = a.serve("/t", offer(Rc::new(RefCell::new(String::new())), Rc::default())).expect("served");
    assert!(a.serve("/t", offer(Rc::new(RefCell::new(String::new())), Rc::default())).is_err());
}

#[test]
fn only_a_real_object_path_is_offered() {
    for good in ["/", "/t", "/org/mpris/MediaPlayer2", "/org/freedesktop/login1/session/_31"] {
        assert!(object_path(good), "{good}");
    }
    for bad in ["", "t", "/t/", "//t", "/a b", "/a\"b", "/a<b>", "/a-b", "/a.b"] {
        assert!(!object_path(bad), "{bad}");
    }
    let d = own_daemon();
    let lp = Loop::new();
    let a = Bus::at(&lp, &d.1).expect("bus a");
    assert!(a.serve("/a\"b/>", offer(Rc::new(RefCell::new(String::new())), Rc::default())).is_err());
}
