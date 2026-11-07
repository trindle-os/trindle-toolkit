//! Asking the file chooser portal which file to open or where to save, so a window needs no file chooser of its own.

use std::cell::Cell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use zbus::zvariant::Value;

use super::bus::{Bus, Listen};
use super::event::Loop;
use super::portal::Answer;

#[derive(Default)]
pub struct Ask {
    pub title: String,
    pub folder: Option<PathBuf>,
    pub name: Option<String>,
    pub filters: Vec<(String, Vec<String>)>,
}

thread_local! {
    static FAILED: Cell<bool> = const { Cell::new(false) };
}

pub fn did_not_answer() -> &'static str {
    crate::t!("kit-chooser-no-answer")
}

pub fn failed() -> bool {
    FAILED.with(Cell::get)
}

pub fn open(bus: &Bus, events: &Loop, ask: Ask, done: impl FnOnce(Option<PathBuf>) + 'static) -> Option<Listen> {
    choose(bus, events, "OpenFile", Kind::One, ask, move |all| done(all.into_iter().next()))
}

pub fn open_folder(bus: &Bus, events: &Loop, ask: Ask, done: impl FnOnce(Option<PathBuf>) + 'static) -> Option<Listen> {
    choose(bus, events, "OpenFile", Kind::Folder, ask, move |all| done(all.into_iter().next()))
}

pub fn open_many(bus: &Bus, events: &Loop, ask: Ask, done: impl FnOnce(Vec<PathBuf>) + 'static) -> Option<Listen> {
    choose(bus, events, "OpenFile", Kind::Many, ask, done)
}

pub fn save(bus: &Bus, events: &Loop, ask: Ask, done: impl FnOnce(Option<PathBuf>) + 'static) -> Option<Listen> {
    choose(bus, events, "SaveFile", Kind::One, ask, move |all| done(all.into_iter().next()))
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    One,
    Many,
    Folder,
}

fn choose(bus: &Bus, events: &Loop, method: &str, kind: Kind, ask: Ask, done: impl FnOnce(Vec<PathBuf>) + 'static) -> Option<Listen> {
    FAILED.with(|x| x.set(false));
    let (token, path) = super::portal::token(bus);
    let options = request_options(kind, &ask, &token);
    super::portal::request(bus, events, "org.freedesktop.portal.FileChooser", method, &("", ask.title.as_str(), options), &path, move |a| {
        FAILED.with(|x| x.set(matches!(a, Answer::Failed)));
        let chosen = match a {
            Answer::Given(r) => r.get("uris").and_then(|v| v.try_clone().ok()).and_then(|v| Vec::<String>::try_from(v).ok())
                .map(|u| u.iter().filter_map(|u| from_uri(u)).collect()).unwrap_or_default(),
            _ => Vec::new(),
        };
        done(chosen);
    })
}

fn request_options<'a>(kind: Kind, ask: &'a Ask, token: &'a str) -> HashMap<&'a str, Value<'a>> {
    let mut options: HashMap<&'a str, Value<'a>> = HashMap::new();
    options.insert("handle_token", Value::from(token));
    if kind == Kind::Many {
        options.insert("multiple", Value::from(true));
    }
    if kind == Kind::Folder {
        options.insert("directory", Value::from(true));
    }
    if let Some(name) = &ask.name {
        options.insert("current_name", Value::from(name.as_str()));
    }
    if let Some(folder) = &ask.folder {
        options.insert("current_folder", Value::from(nul_ended(folder)));
    }
    if !ask.filters.is_empty() {
        let filters: Vec<(String, Vec<(u32, String)>)> = ask.filters.iter()
            .map(|(label, globs)| (label.clone(), globs.iter().map(|g| (0u32, g.clone())).collect()))
            .collect();
        options.insert("filters", Value::from(filters));
    }
    options
}

fn nul_ended(p: &Path) -> Vec<u8> {
    use std::os::unix::ffi::OsStrExt;
    let mut bytes = p.as_os_str().as_bytes().to_vec();
    bytes.push(0);
    bytes
}

pub fn from_uri(uri: &str) -> Option<PathBuf> {
    use std::os::unix::ffi::OsStringExt;
    let rest = uri.strip_prefix("file://")?;
    let at = rest.find('/')?;
    if !rest[..at].is_empty() && !rest[..at].eq_ignore_ascii_case("localhost") || rest.contains(['?', '#']) {
        return None;
    }
    let rest = &rest[at..];
    let b = rest.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        let hex = |c: u8| (c as char).to_digit(16);
        if b[i] == b'%' && i + 2 < b.len() {
            if let (Some(h), Some(l)) = (hex(b[i + 1]), hex(b[i + 2])) {
                out.push((h * 16 + l) as u8);
                i += 3;
                continue;
            }
        }
        out.push(b[i]);
        i += 1;
    }
    let path = PathBuf::from(std::ffi::OsString::from_vec(out));
    (!path.as_os_str().as_encoded_bytes().contains(&0) && !path.components().any(|c| c == std::path::Component::ParentDir)).then_some(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::bus::testing::{own_daemon, until};
    use super::super::bus::{Asked, Offer, Props};
    use std::cell::RefCell;
    use std::rc::Rc;
    use zbus::zvariant::OwnedValue;

    type Options = HashMap<String, OwnedValue>;

    fn portal(bus: &Bus, heard: Rc<RefCell<Vec<(String, String, Options)>>>) -> super::super::bus::Served {
        let b = bus.clone();
        bus.serve("/org/freedesktop/portal/desktop", Offer {
            name: "org.freedesktop.portal.FileChooser".into(),
            xml: String::new(),
            call: Box::new(move |a: Asked| {
                let Ok((_, title, options)) = a.message().body().deserialize::<(String, String, Options)>() else { return };
                let token = options.get("handle_token").and_then(|v| String::try_from(v.try_clone().ok()?).ok()).unwrap_or_default();
                let sender = a.sender().trim_start_matches(':').replace('.', "_");
                let path = format!("/org/freedesktop/portal/desktop/request/{sender}/{token}");
                let member = a.member();
                a.reply(&zbus::zvariant::ObjectPath::try_from(path.as_str()).expect("path"));
                let mut r: HashMap<&str, Value> = HashMap::new();
                r.insert("uris", Value::from(vec!["file:///tmp/My%20File.deb"]));
                let code = match title.as_str() {
                    "Cancel" => 1u32,
                    "Fail" => 2,
                    _ => 0,
                };
                let _ = b.emit(&path, "org.freedesktop.portal.Request", "Response", &(code, r));
                heard.borrow_mut().push((member, title, options));
            }),
            props: Box::new(Props::new),
            set: None,
        }).expect("served")
    }

    #[test]
    fn the_portal_is_asked_and_its_answer_heard() {
        let d = own_daemon();
        let lp = Loop::new();
        let (p, us) = (Bus::at(&lp, &d.1).expect("portal"), Bus::at(&lp, &d.1).expect("us"));
        let owned = Rc::new(Cell::new(false));
        let o = owned.clone();
        p.own("org.freedesktop.portal.Desktop", false, &lp, move |ok| o.set(ok));
        let o = owned.clone();
        assert!(until(&lp, move || o.get()));
        let heard = Rc::new(RefCell::new(Vec::new()));
        let _served = portal(&p, heard.clone());

        let got: Rc<RefCell<Option<Option<PathBuf>>>> = Rc::new(RefCell::new(None));
        let g = got.clone();
        let ask = Ask { title: "Open a package".into(), filters: vec![("Packages".into(), vec!["*.deb".into()])], ..Ask::default() };
        let _listen = open(&us, &lp, ask, move |p| *g.borrow_mut() = Some(p)).expect("asked");
        let g = got.clone();
        assert!(until(&lp, move || g.borrow().is_some()));
        assert_eq!(got.borrow_mut().take(), Some(Some(PathBuf::from("/tmp/My File.deb"))));
        {
            let h = heard.borrow();
            let (member, title, options) = &h[0];
            assert_eq!((member.as_str(), title.as_str()), ("OpenFile", "Open a package"));
            let filters = options.get("filters").and_then(|v| v.try_clone().ok())
                .and_then(|v| Vec::<(String, Vec<(u32, String)>)>::try_from(v).ok());
            assert_eq!(filters, Some(vec![("Packages".to_string(), vec![(0, "*.deb".to_string())])]));
        }

        let g = got.clone();
        let ask = Ask { title: "Cancel".into(), folder: Some(PathBuf::from("/tmp")), name: Some("a.png".into()), ..Ask::default() };
        let _listen = save(&us, &lp, ask, move |p| *g.borrow_mut() = Some(p)).expect("asked");
        let g = got.clone();
        assert!(until(&lp, move || g.borrow().is_some()));
        assert_eq!(got.borrow_mut().take(), Some(None));
        let h = heard.borrow();
        assert_eq!(h[1].0, "SaveFile");
        assert_eq!(h[1].2.get("current_folder").and_then(|v| Vec::<u8>::try_from(v.try_clone().ok()?).ok()), Some(b"/tmp\0".to_vec()));
        drop(h);

        let g = got.clone();
        let _listen = open_folder(&us, &lp, Ask { title: "Pick a folder".into(), ..Ask::default() }, move |p| *g.borrow_mut() = Some(p)).expect("asked");
        let g = got.clone();
        assert!(until(&lp, move || g.borrow().is_some()));
        let h = heard.borrow();
        assert_eq!(h[2].0, "OpenFile");
        assert_eq!(h[2].2.get("directory").and_then(|v| bool::try_from(v.try_clone().ok()?).ok()), Some(true));
        assert!(!h[0].2.contains_key("directory"));
    }

    #[test]
    fn a_file_address_back_to_its_path() {
        assert_eq!(from_uri("file:///home/a/My%20Shot.png"), Some(PathBuf::from("/home/a/My Shot.png")));
        assert_eq!(from_uri("file://localhost/tmp/x%"), Some(PathBuf::from("/tmp/x%")));
        assert_eq!(from_uri("file:///tmp/a%41"), Some(PathBuf::from("/tmp/aA")));
        assert_eq!(from_uri("https://example.com/a"), None);
        assert_eq!(from_uri("file://otherhost/tmp/x"), None);
        assert_eq!(from_uri("file:///tmp/a?x=1"), None);
        assert_eq!(from_uri("file:///tmp/a%00b"), None);
        assert_eq!(from_uri("file:///tmp/../etc/x"), None);
    }

    #[test]
    fn a_window_that_failed_is_told_apart_from_a_cancel() {
        let d = own_daemon();
        let lp = Loop::new();
        let (p, us) = (Bus::at(&lp, &d.1).expect("portal"), Bus::at(&lp, &d.1).expect("us"));
        let owned = Rc::new(Cell::new(false));
        let o = owned.clone();
        p.own("org.freedesktop.portal.Desktop", false, &lp, move |ok| o.set(ok));
        let o = owned.clone();
        assert!(until(&lp, move || o.get()));
        let _served = portal(&p, Rc::new(RefCell::new(Vec::new())));
        let got: Rc<RefCell<Option<Option<PathBuf>>>> = Rc::new(RefCell::new(None));
        for (title, failed_after) in [("Cancel", false), ("Fail", true)] {
            let g = got.clone();
            let ask = Ask { title: title.into(), ..Ask::default() };
            let _listen = open(&us, &lp, ask, move |p| *g.borrow_mut() = Some(p)).expect("asked");
            let g = got.clone();
            assert!(until(&lp, move || g.borrow().is_some()));
            assert_eq!(got.borrow_mut().take(), Some(None));
            assert_eq!(failed(), failed_after, "{title}");
        }
    }
}
