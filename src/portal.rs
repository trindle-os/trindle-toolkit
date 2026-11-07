//! The desktop portal's asks beside choosing files (`chooser`): a screenshot, the camera, printing, and opening an address or a file in the person's chosen program.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::os::fd::OwnedFd;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::Duration;

use zbus::zvariant::{OwnedValue, Value};

use super::bus::{Bus, Listen};
use super::event::Loop;

const NAME: &str = "org.freedesktop.portal.Desktop";
const PATH: &str = "/org/freedesktop/portal/desktop";
const TAKEN: Duration = Duration::from_secs(25);

pub type Results = HashMap<String, OwnedValue>;

#[derive(Debug, PartialEq, Eq)]
pub enum Answer<T> {
    Given(T),
    Cancelled,
    Failed,
}

thread_local! {
    static NEXT: Cell<u32> = const { Cell::new(0) };
}

pub(crate) fn token(bus: &Bus) -> (String, String) {
    let n = NEXT.with(|c| {
        c.set(c.get() + 1);
        c.get()
    });
    let token = format!("trindle_{}_{n}", std::process::id());
    let sender = bus.unique_name().trim_start_matches(':').replace('.', "_");
    let path = format!("{PATH}/request/{sender}/{token}");
    (token, path)
}

pub(crate) fn request<B: serde::Serialize + zbus::zvariant::DynamicType>(bus: &Bus, events: &Loop, iface: &str, method: &str, body: &B, path: &str,
                                                                        done: impl FnOnce(Answer<Results>) + 'static) -> Option<Listen> {
    let done: Rc<RefCell<Option<Box<dyn FnOnce(Answer<Results>)>>>> = Rc::new(RefCell::new(Some(Box::new(done))));
    let end = move |d: &Rc<RefCell<Option<Box<dyn FnOnce(Answer<Results>)>>>>, a| {
        let f = d.borrow_mut().take();
        if let Some(f) = f {
            f(a);
        }
    };
    let d = done.clone();
    let listen = bus.listen(Some(NAME), "org.freedesktop.portal.Request", Some("Response"), Some(path), move |msg| {
        end(&d, match msg.body().deserialize::<(u32, Results)>() {
            Ok((0, r)) => Answer::Given(r),
            Ok((1, _)) => Answer::Cancelled,
            _ => Answer::Failed,
        });
    });
    let Ok(listen) = listen else {
        end(&done, Answer::Failed);
        return None;
    };
    bus.call(NAME, PATH, iface).method(method, body, true, Some(TAKEN), events, move |r| {
        if r.is_err() {
            end(&done, Answer::Failed);
        }
    });
    Some(listen)
}

fn options(token: &str) -> HashMap<&str, Value<'_>> {
    HashMap::from([("handle_token", Value::from(token))])
}

pub fn screenshot(bus: &Bus, events: &Loop, interactive: bool, done: impl FnOnce(Answer<PathBuf>) + 'static) -> Option<Listen> {
    let (token, path) = token(bus);
    let mut o = options(&token);
    o.insert("interactive", Value::from(interactive));
    request(bus, events, "org.freedesktop.portal.Screenshot", "Screenshot", &("", o), &path, move |a| done(given(a, |r| {
        let uri = String::try_from(r.get("uri")?.try_clone().ok()?).ok()?;
        crate::chooser::from_uri(&uri)
    })))
}

pub fn open_uri(bus: &Bus, events: &Loop, uri: &str, ask: bool, done: impl FnOnce(Answer<()>) + 'static) -> Option<Listen> {
    if !trindle_common::files::fits_a_line(uri) || !uri.contains(':') {
        done(Answer::Failed);
        return None;
    }
    let (token, path) = token(bus);
    let mut o = options(&token);
    o.insert("ask", Value::from(ask));
    request(bus, events, "org.freedesktop.portal.OpenURI", "OpenURI", &("", uri, o), &path, move |a| done(given(a, |_| Some(()))))
}

pub fn open_file(bus: &Bus, events: &Loop, file: &Path, ask: bool, done: impl FnOnce(Answer<()>) + 'static) -> Option<Listen> {
    let Ok(f) = std::fs::File::open(file) else {
        done(Answer::Failed);
        return None;
    };
    let (token, path) = token(bus);
    let mut o = options(&token);
    o.insert("ask", Value::from(ask));
    let fd = zbus::zvariant::Fd::from(&f);
    request(bus, events, "org.freedesktop.portal.OpenURI", "OpenFile", &("", fd, o), &path, move |a| done(given(a, |_| Some(()))))
}

pub fn print_pdf(bus: &Bus, events: &Loop, title: &str, pdf: &[u8], done: impl FnOnce(Answer<()>) + 'static) -> Option<Listen> {
    use std::io::{Seek, SeekFrom};
    let file = crate::sys::sock::sealed(c"trindle-print", &[pdf]).map(std::fs::File::from)
        .and_then(|mut f| f.seek(SeekFrom::Start(0)).map(|_| f));
    let Ok(file) = file else {
        done(Answer::Failed);
        return None;
    };
    let (token, path) = token(bus);
    let fd = zbus::zvariant::Fd::from(&file);
    request(bus, events, "org.freedesktop.portal.Print", "Print", &("", title, fd, options(&token)), &path, move |a| done(given(a, |_| Some(()))))
}

pub fn camera(bus: &Bus, events: &Loop, done: impl FnOnce(Answer<OwnedFd>) + 'static) -> Option<Listen> {
    const IFACE: &str = "org.freedesktop.portal.Camera";
    let (token, path) = token(bus);
    let (b, l) = (bus.clone(), events.clone());
    request(bus, events, IFACE, "AccessCamera", &(options(&token),), &path, move |a| match a {
        Answer::Given(_) => {
            let none: HashMap<&str, Value> = HashMap::new();
            b.call(NAME, PATH, IFACE).method("OpenPipeWireRemote", &(none,), false, Some(TAKEN), &l, move |r| {
                let fd = r.ok().and_then(|m| m.body().deserialize::<zbus::zvariant::OwnedFd>().ok());
                done(fd.map_or(Answer::Failed, |fd| Answer::Given(fd.into())));
            });
        }
        Answer::Cancelled => done(Answer::Cancelled),
        Answer::Failed => done(Answer::Failed),
    })
}

fn given<T>(a: Answer<Results>, read: impl FnOnce(&Results) -> Option<T>) -> Answer<T> {
    match a {
        Answer::Given(r) => read(&r).map_or(Answer::Failed, Answer::Given),
        Answer::Cancelled => Answer::Cancelled,
        Answer::Failed => Answer::Failed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bus::testing::{own_daemon, until};
    use crate::bus::{Asked, Offer, Props, Served};

    type Heard = Rc<RefCell<Vec<(String, Results)>>>;

    fn pretend(bus: &Bus, iface: &str, code: u32, heard: Heard) -> Served {
        let b = bus.clone();
        bus.serve(PATH, Offer {
            name: iface.into(),
            xml: String::new(),
            call: Box::new(move |a: Asked| {
                let member = a.member();
                if member == "OpenPipeWireRemote" {
                    let (ours, _theirs) = std::os::unix::net::UnixStream::pair().expect("pair");
                    heard.borrow_mut().push((member, Results::new()));
                    return a.reply(&zbus::zvariant::Fd::from(OwnedFd::from(ours)));
                }
                let body = a.message().body();
                let options = match member.as_str() {
                    "Screenshot" => body.deserialize::<(String, Results)>().map(|b| b.1),
                    "OpenURI" => body.deserialize::<(String, String, Results)>().map(|b| b.2),
                    "OpenFile" => body.deserialize::<(String, zbus::zvariant::OwnedFd, Results)>().map(|b| b.2),
                    "Print" => body.deserialize::<(String, String, zbus::zvariant::OwnedFd, Results)>().map(|(_, title, fd, mut o)| {
                        use std::io::Read;
                        let mut bytes = String::new();
                        std::fs::File::from(OwnedFd::from(fd)).read_to_string(&mut bytes).expect("the file readable from its start");
                        o.insert("heard title".into(), Value::from(title).try_to_owned().unwrap());
                        o.insert("heard bytes".into(), Value::from(bytes).try_to_owned().unwrap());
                        o
                    }),
                    _ => body.deserialize::<(Results,)>().map(|b| b.0),
                }.expect("the ask's shape");
                let token = String::try_from(options["handle_token"].try_clone().unwrap()).unwrap();
                let path = format!("{PATH}/request/{}/{token}", a.sender().trim_start_matches(':').replace('.', "_"));
                a.reply(&zbus::zvariant::ObjectPath::try_from(path.as_str()).unwrap());
                let r: HashMap<&str, Value> = HashMap::from([("uri", Value::from("file:///tmp/Screenshot%201.png"))]);
                let _ = b.emit(&path, "org.freedesktop.portal.Request", "Response", &(code, r));
                heard.borrow_mut().push((member, options));
            }),
            props: Box::new(Props::new),
            set: None,
        }).expect("served")
    }

    fn setup() -> (crate::bus::testing::Daemon, Loop, Bus, Bus) {
        let d = own_daemon();
        let lp = Loop::new();
        let (p, us) = (Bus::at(&lp, &d.1).expect("portal"), Bus::at(&lp, &d.1).expect("us"));
        let owned = Rc::new(Cell::new(false));
        let o = owned.clone();
        p.own(NAME, false, &lp, move |ok| o.set(ok));
        assert!(until(&lp, move || owned.get()));
        (d, lp, p, us)
    }

    fn wait<T: 'static>(lp: &Loop, got: &Rc<RefCell<Option<T>>>) -> T {
        let g = got.clone();
        assert!(until(lp, move || g.borrow().is_some()));
        got.borrow_mut().take().unwrap()
    }

    fn flag(r: &Results, k: &str) -> Option<bool> {
        bool::try_from(r.get(k)?.try_clone().ok()?).ok()
    }

    #[test]
    fn a_print_hands_the_portal_the_whole_file_from_its_start() {
        let (_d, lp, p, us) = setup();
        let heard = Heard::default();
        let _s = pretend(&p, "org.freedesktop.portal.Print", 0, heard.clone());
        let got = Rc::new(RefCell::new(None));
        let g = got.clone();
        let _l = print_pdf(&us, &lp, "A letter", b"%PDF-1.4 words", move |a| *g.borrow_mut() = Some(a)).expect("asked");
        assert_eq!(wait(&lp, &got), Answer::Given(()));
        let text = |k: &str| String::try_from(heard.borrow()[0].1[k].try_clone().unwrap()).unwrap();
        assert_eq!(text("heard title"), "A letter");
        assert_eq!(text("heard bytes"), "%PDF-1.4 words");
    }

    #[test]
    fn a_print_the_person_called_off_is_told_as_cancelled() {
        let (_d, lp, p, us) = setup();
        let _s = pretend(&p, "org.freedesktop.portal.Print", 1, Heard::default());
        let got = Rc::new(RefCell::new(None));
        let g = got.clone();
        let _l = print_pdf(&us, &lp, "No", b"x", move |a| *g.borrow_mut() = Some(a)).expect("asked");
        assert_eq!(wait(&lp, &got), Answer::Cancelled);
    }

    #[test]
    fn a_screenshot_comes_back_as_a_path() {
        let (_d, lp, p, us) = setup();
        let heard = Heard::default();
        let _s = pretend(&p, "org.freedesktop.portal.Screenshot", 0, heard.clone());
        let got = Rc::new(RefCell::new(None));
        let g = got.clone();
        let _l = screenshot(&us, &lp, true, move |a| *g.borrow_mut() = Some(a)).expect("asked");
        assert_eq!(wait(&lp, &got), Answer::Given(PathBuf::from("/tmp/Screenshot 1.png")));
        assert_eq!(flag(&heard.borrow()[0].1, "interactive"), Some(true));
    }

    #[test]
    fn a_cancel_and_a_failure_are_told_apart() {
        for (code, want) in [(1, Answer::Cancelled), (2, Answer::Failed)] {
            let (_d, lp, p, us) = setup();
            let _s = pretend(&p, "org.freedesktop.portal.Screenshot", code, Heard::default());
            let got = Rc::new(RefCell::new(None));
            let g = got.clone();
            let _l = screenshot(&us, &lp, false, move |a| *g.borrow_mut() = Some(a)).expect("asked");
            assert_eq!(wait(&lp, &got), want, "code {code}");
        }
    }

    #[test]
    fn with_no_portal_the_ask_fails() {
        let (_d, lp, p, us) = setup();
        let _s = p.serve(PATH, Offer {
            name: "org.freedesktop.portal.OpenURI".into(),
            xml: String::new(),
            call: Box::new(|a: Asked| a.fail("org.freedesktop.DBus.Error.UnknownMethod", "none here")),
            props: Box::new(Props::new),
            set: None,
        }).expect("served");
        let got = Rc::new(RefCell::new(None));
        let g = got.clone();
        let _l = open_uri(&us, &lp, "https://example.com/", false, move |a| *g.borrow_mut() = Some(a));
        assert_eq!(wait(&lp, &got), Answer::Failed);
    }

    #[test]
    fn an_address_and_a_file_are_opened_through_the_portal() {
        let (_d, lp, p, us) = setup();
        let heard = Heard::default();
        let _s = pretend(&p, "org.freedesktop.portal.OpenURI", 0, heard.clone());
        let got = Rc::new(RefCell::new(None));
        let g = got.clone();
        let _l = open_uri(&us, &lp, "https://example.com/", true, move |a| *g.borrow_mut() = Some(a)).expect("asked");
        assert_eq!(wait(&lp, &got), Answer::Given(()));
        let g = got.clone();
        let _l = open_file(&us, &lp, Path::new("Cargo.toml"), false, move |a| *g.borrow_mut() = Some(a)).expect("asked");
        assert_eq!(wait(&lp, &got), Answer::Given(()));
        let h = heard.borrow();
        assert_eq!((h[0].0.as_str(), flag(&h[0].1, "ask")), ("OpenURI", Some(true)));
        assert_eq!((h[1].0.as_str(), flag(&h[1].1, "ask")), ("OpenFile", Some(false)));
    }

    #[test]
    fn an_address_that_breaks_a_line_or_a_missing_file_is_refused_unasked() {
        let lp = Loop::new();
        let d = own_daemon();
        let us = Bus::at(&lp, &d.1).expect("us");
        let got = Rc::new(RefCell::new(None));
        for uri in ["https://example.com/\nX-Evil: 1", "no address", ""] {
            let g = got.clone();
            assert!(open_uri(&us, &lp, uri, false, move |a| *g.borrow_mut() = Some(a)).is_none());
            assert_eq!(got.borrow_mut().take(), Some(Answer::Failed), "{uri:?}");
        }
        let g = got.clone();
        assert!(open_file(&us, &lp, Path::new("/no/such/file"), false, move |a| *g.borrow_mut() = Some(a)).is_none());
        assert_eq!(got.borrow_mut().take(), Some(Answer::Failed));
    }

    #[test]
    fn the_camera_is_asked_for_then_its_pipewire_handed_over() {
        let (_d, lp, p, us) = setup();
        let heard = Heard::default();
        let _s = pretend(&p, "org.freedesktop.portal.Camera", 0, heard.clone());
        let got = Rc::new(RefCell::new(None));
        let g = got.clone();
        let _l = camera(&us, &lp, move |a| *g.borrow_mut() = Some(a)).expect("asked");
        assert!(matches!(wait(&lp, &got), Answer::Given(_)));
        let members: Vec<String> = heard.borrow().iter().map(|h| h.0.clone()).collect();
        assert_eq!(members, ["AccessCamera", "OpenPipeWireRemote"]);
    }

    #[test]
    fn a_camera_refused_is_a_cancel_and_pipewire_is_never_asked() {
        let (_d, lp, p, us) = setup();
        let heard = Heard::default();
        let _s = pretend(&p, "org.freedesktop.portal.Camera", 1, heard.clone());
        let got = Rc::new(RefCell::new(None));
        let g = got.clone();
        let _l = camera(&us, &lp, move |a| *g.borrow_mut() = Some(a)).expect("asked");
        assert!(matches!(wait(&lp, &got), Answer::Cancelled));
        assert_eq!(heard.borrow().len(), 1);
    }
}
