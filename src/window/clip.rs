//! The clipboard: text, the mouse's selection, a picture as PNG (image/png) and a list of files (text/uri-list), put on it and read from it.

use std::os::fd::{AsFd, OwnedFd};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use wayland_protocols::wp::primary_selection::zv1::client::{
    zwp_primary_selection_device_v1, zwp_primary_selection_offer_v1, zwp_primary_selection_source_v1,
};

use super::*;
use crate::common::run;
use crate::dnd::{self, Incoming};

const PNG: &str = "image/png";
const COPIED_FILES: &str = "x-special/gnome-copied-files";
const IMAGE_MOST: usize = 64 << 20;
const IMAGE_TIME: std::time::Duration = std::time::Duration::from_secs(15);
static HANDING: run::Slots = run::Slots::new(8);
const PASTE_MOST: usize = 1 << 20;
const COPY_CHECK: std::time::Duration = std::time::Duration::from_millis(400);

type Kinds = Vec<(String, Arc<Vec<u8>>)>;

fn hand_over(kinds: Vec<(String, std::sync::Arc<Vec<u8>>)>) {
    let _ = std::thread::Builder::new().name("copy-hand-over".into()).spawn(move || {
        crate::common::selection::offer_kinds(kinds, || {});
    });
}

fn text_kinds(text: String) -> Vec<(String, std::sync::Arc<Vec<u8>>)> {
    let data = std::sync::Arc::new(text.into_bytes());
    crate::common::selection::TEXT.iter().map(|t| (t.to_string(), data.clone())).collect()
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct OnClipboard {
    pub text: bool,
    pub image: bool,
    pub files: bool,
}

fn is_png(bytes: &[u8]) -> bool {
    bytes.starts_with(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A])
}

fn has_kinds(types: &[String]) -> OnClipboard {
    let has = |k: &str| types.iter().any(|t| t == k);
    OnClipboard { text: dnd::TEXT_KINDS.into_iter().any(has), image: has(PNG), files: has(dnd::URI_LIST) }
}

fn file_kinds(paths: &[PathBuf]) -> Option<Kinds> {
    let list = dnd::uri_list(paths);
    if list.is_empty() {
        return None;
    }
    let note = format!("copy\n{}", list.replace("\r\n", "\n").trim_end());
    let plain = Arc::new(dnd::path_lines(paths).into_bytes());
    let mut kinds: Kinds = vec![(dnd::URI_LIST.into(), Arc::new(list.into_bytes())), (COPIED_FILES.into(), Arc::new(note.into_bytes()))];
    kinds.extend(dnd::TEXT_OUT.iter().map(|t| (t.to_string(), plain.clone())));
    Some(kinds)
}

pub(super) struct Clipped(Kinds);

impl Dispatch<wl_data_source::WlDataSource, Clipped> for State {
    fn event(state: &mut Self, source: &wl_data_source::WlDataSource, event: wl_data_source::Event, held: &Clipped, _: &Connection,
             _: &QueueHandle<Self>) {
        match event {
            wl_data_source::Event::Send { mime_type, fd } => {
                let Some((_, data)) = held.0.iter().find(|(k, _)| *k == mime_type) else { return };
                write_on_thread(&HANDING, fd, data.clone(), |_| {});
            }
            wl_data_source::Event::Cancelled => {
                if state.copying.copied.as_ref() == Some(source) {
                    state.copying.copied = None;
                }
                source.destroy();
            }
            _ => {}
        }
    }
}

impl Window {
    pub fn copy_image(&self, png: &[u8]) -> bool {
        if !is_png(png) || png.len() > IMAGE_MOST {
            return false;
        }
        self.put(vec![(PNG.into(), Arc::new(png.to_vec()))]);
        true
    }

    pub fn copy_files(&self, paths: &[PathBuf]) -> bool {
        let Some(kinds) = file_kinds(paths) else { return false };
        self.put(kinds);
        true
    }

    pub fn clipboard_has(&self) -> OnClipboard {
        self.display.state.borrow().copying.selection.as_ref().map(|o| has_kinds(&drag::kinds(o))).unwrap_or_default()
    }

    pub fn paste_image(&self, call: impl FnOnce(Vec<u8>) + 'static) {
        self.read_kind(PNG, IMAGE_MOST, IMAGE_TIME, move |bytes| {
            if is_png(&bytes) {
                call(bytes);
            }
        });
    }

    pub fn paste_files(&self, call: impl FnOnce(Vec<PathBuf>) + 'static) {
        self.read_kind(dnd::URI_LIST, dnd::MOST_BYTES, dnd::MOST_TIME, move |bytes| {
            if let Ok(paths) = dnd::parse_uri_list(&bytes) {
                call(paths);
            }
        });
    }

    fn put(&self, kinds: Kinds) {
        let qh = self.display.queue.borrow().handle();
        let mut state = self.display.state.borrow_mut();
        let taken = match (&state.clipboard, &state.device) {
            (Some(m), Some(device)) => {
                let source = m.create_data_source(&qh, Clipped(kinds.clone()));
                for (kind, _) in &kinds {
                    source.offer(kind.clone());
                }
                device.set_selection(Some(&source), state.serial);
                if let Some(old) = state.copying.copied.replace(source) {
                    old.destroy();
                }
                true
            }
            _ => false,
        };
        let before = state.copying.selections;
        drop(state);
        let _ = self.display.conn.flush();
        self.hand_over_unless_taken(taken, before, kinds);
    }

    fn read_kind(&self, kind: &'static str, most: usize, time: std::time::Duration, call: impl FnOnce(Vec<u8>) + 'static) {
        let state = self.display.state.borrow();
        let Some(offer) = &state.copying.selection else { return };
        if !drag::kinds(offer).iter().any(|k| k == kind) {
            return;
        }
        let offer = offer.clone();
        drop(state);
        let Some((read, write)) = run::pipe() else { return };
        let Some(slot) = HANDING.take() else { return };
        offer.receive(kind.into(), write.as_fd());
        let _ = self.display.conn.flush();
        drop(write);
        run::on_thread(move || {
            let _slot = slot;
            dnd::read_all(read, most, Instant::now() + time)
        }, move |got| {
            if let Ok(bytes) = got {
                call(bytes);
            }
        });
    }

    pub fn copy(&self, text: &str) {
        let qh = self.display.queue.borrow().handle();
        let mut state = self.display.state.borrow_mut();
        let taken = match (&state.clipboard, &state.device) {
            (Some(m), Some(device)) => {
                let source = m.create_data_source(&qh, text.to_string());
                for t in crate::dnd::TEXT_KINDS {
                    source.offer(t.into());
                }
                device.set_selection(Some(&source), state.serial);
                let old = state.copying.copied.replace(source);
                if let Some(old) = old {
                    old.destroy();
                }
                true
            }
            _ => false,
        };
        let before = state.copying.selections;
        drop(state);
        let _ = self.display.conn.flush();
        self.hand_over_unless_taken(taken, before, text_kinds(text.to_string()));
    }

    fn hand_over_unless_taken(&self, taken: bool, before: u64, kinds: Vec<(String, std::sync::Arc<Vec<u8>>)>) {
        if !taken {
            return hand_over(kinds);
        }
        let display = self.display.clone();
        self.display.events.after(COPY_CHECK, move || {
            if display.state.borrow().copying.selections == before {
                hand_over(kinds);
            }
        });
    }

    pub fn paste(&self, call: impl FnOnce(String) + 'static) {
        let state = self.display.state.borrow();
        let Some(offer) = &state.copying.selection else { return };
        let types = drag::kinds(offer);
        let Some(kind) = crate::dnd::TEXT_KINDS.into_iter().find(|k| types.iter().any(|t| t == k)) else { return };
        let offer = offer.clone();
        drop(state);
        self.read_text(move |fd| offer.receive(kind.into(), fd), call);
    }

    pub fn select(&self, text: &str) {
        let qh = self.display.queue.borrow().handle();
        let mut state = self.display.state.borrow_mut();
        let Some((manager, device)) = &state.copying.primary else { return };
        let source = manager.create_source(&qh, text.to_string());
        for t in crate::dnd::TEXT_KINDS {
            source.offer(t.into());
        }
        device.set_selection(Some(&source), state.serial);
        if let Some(old) = state.copying.primary_copied.replace(source) {
            old.destroy();
        }
        drop(state);
        let _ = self.display.conn.flush();
    }

    pub fn paste_selection(&self, call: impl FnOnce(String) + 'static) {
        let state = self.display.state.borrow();
        let Some(offer) = &state.copying.primary_offer else { return };
        let types = offer.data::<Mutex<Vec<String>>>().and_then(|k| k.lock().ok().map(|k| k.clone())).unwrap_or_default();
        let Some(kind) = crate::dnd::TEXT_KINDS.into_iter().find(|k| types.iter().any(|t| t == k)) else { return };
        let offer = offer.clone();
        drop(state);
        self.read_text(move |fd| offer.receive(kind.into(), fd), call);
    }

    fn read_text(&self, ask: impl FnOnce(std::os::fd::BorrowedFd), call: impl FnOnce(String) + 'static) {
        let Some((read, write)) = run::pipe() else { return eprintln!("trindle: no pipe to paste through") };
        let Some(slot) = HANDING.take() else { return eprintln!("trindle: too many pastes at once, so one was not read") };
        ask(write.as_fd());
        let _ = self.display.conn.flush();
        drop(write);
        run::on_thread(move || {
            let _slot = slot;
            crate::dnd::read_all(read, PASTE_MOST, Instant::now() + crate::dnd::MOST_TIME)
        }, move |got| {
            if let Some(text) = got.ok().and_then(|b| crate::dnd::clean_text(&b)) {
                call(text);
            }
        });
    }
}

impl Dispatch<wl_data_device::WlDataDevice, ()> for State {
    fn event(state: &mut Self, _: &wl_data_device::WlDataDevice, event: wl_data_device::Event, _: &(), _: &Connection,
             _: &QueueHandle<Self>) {
        match event {
            wl_data_device::Event::Selection { id } => {
                state.copying.selections += 1;
                if let Some(old) = std::mem::replace(&mut state.copying.selection, id) {
                    old.destroy();
                }
            }
            event => drag::device_event(state, event),
        }
    }

    wayland_client::event_created_child!(State, wl_data_device::WlDataDevice, [
        wl_data_device::EVT_DATA_OFFER_OPCODE => (wl_data_offer::WlDataOffer, Mutex::new(Incoming::new())),
    ]);
}

fn give_text(fd: OwnedFd, kind: &str, text: &str) {
    if !crate::dnd::TEXT_KINDS.contains(&kind) {
        return;
    }
    write_on_thread(&HANDING, fd, text.as_bytes().to_vec(), |_| {});
}

pub(super) fn write_on_thread<B>(places: &'static run::Slots, fd: OwnedFd, bytes: B,
                                 then: impl FnOnce(&Result<crate::dnd::Sent, crate::dnd::Why>) + Send + 'static)
where B: std::ops::Deref + Send + 'static, B::Target: AsRef<[u8]> {
    let Some(slot) = places.take() else { return eprintln!("trindle: too many sends at once, so one was not sent") };
    let made = std::thread::Builder::new().name("send".into()).spawn(move || {
        let _slot = slot;
        let _ = crate::dnd::write_all_then(fd, (*bytes).as_ref(), Instant::now() + crate::dnd::MOST_TIME, then);
    });
    if made.is_err() {
        eprintln!("trindle: no thread to send on, so one was not sent");
    }
}

impl Dispatch<wl_data_source::WlDataSource, String> for State {
    fn event(state: &mut Self, source: &wl_data_source::WlDataSource, event: wl_data_source::Event, text: &String,
             _: &Connection, _: &QueueHandle<Self>) {
        match event {
            wl_data_source::Event::Send { mime_type, fd } => give_text(fd, &mime_type, text),
            wl_data_source::Event::Cancelled => {
                if state.copying.copied.as_ref() == Some(source) {
                    state.copying.copied = None;
                }
                source.destroy();
            }
            _ => {}
        }
    }
}

impl Dispatch<ZwpPrimarySelectionDeviceV1, ()> for State {
    fn event(state: &mut Self, _: &ZwpPrimarySelectionDeviceV1, event: zwp_primary_selection_device_v1::Event, _: &(), _: &Connection,
             _: &QueueHandle<Self>) {
        if let zwp_primary_selection_device_v1::Event::Selection { id } = event {
            if let Some(old) = std::mem::replace(&mut state.copying.primary_offer, id) {
                old.destroy();
            }
        }
    }

    wayland_client::event_created_child!(State, ZwpPrimarySelectionDeviceV1, [
        zwp_primary_selection_device_v1::EVT_DATA_OFFER_OPCODE => (ZwpPrimarySelectionOfferV1, Mutex::new(Vec::<String>::new())),
    ]);
}

impl Dispatch<ZwpPrimarySelectionOfferV1, Mutex<Vec<String>>> for State {
    fn event(_: &mut Self, _: &ZwpPrimarySelectionOfferV1, event: zwp_primary_selection_offer_v1::Event, kinds: &Mutex<Vec<String>>,
             _: &Connection, _: &QueueHandle<Self>) {
        if let (zwp_primary_selection_offer_v1::Event::Offer { mime_type }, Ok(mut kinds)) = (event, kinds.lock()) {
            crate::dnd::keep_kind(&mut kinds, mime_type);
        }
    }
}

impl Dispatch<ZwpPrimarySelectionSourceV1, String> for State {
    fn event(state: &mut Self, source: &ZwpPrimarySelectionSourceV1, event: zwp_primary_selection_source_v1::Event, text: &String,
             _: &Connection, _: &QueueHandle<Self>) {
        match event {
            zwp_primary_selection_source_v1::Event::Send { mime_type, fd } => give_text(fd, &mime_type, text),
            zwp_primary_selection_source_v1::Event::Cancelled => {
                if state.copying.primary_copied.as_ref() == Some(source) {
                    state.copying.primary_copied = None;
                }
                source.destroy();
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn paths(p: &[&str]) -> Vec<PathBuf> {
        p.iter().map(PathBuf::from).collect()
    }

    fn text(k: &Kinds, kind: &str) -> String {
        String::from_utf8(k.iter().find(|(n, _)| n == kind).unwrap().1.to_vec()).unwrap()
    }

    #[test]
    fn only_a_real_png_is_taken_or_passed_on() {
        assert!(is_png(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 1, 2]));
        assert!(!is_png(b"GIF89a") && !is_png(b"") && !is_png(&[0x89, b'P', b'N', b'G']));
    }

    #[test]
    fn copied_files_go_by_the_address_list_the_managers_note_and_plain_paths() {
        let k = file_kinds(&paths(&["/home/john/a b.txt", "/home/john/c.png"])).unwrap();
        assert_eq!(text(&k, dnd::URI_LIST), "file:///home/john/a%20b.txt\r\nfile:///home/john/c.png\r\n");
        assert_eq!(text(&k, COPIED_FILES), "copy\nfile:///home/john/a%20b.txt\nfile:///home/john/c.png");
        assert_eq!(text(&k, "text/plain;charset=utf-8"), "/home/john/a b.txt\n/home/john/c.png");
        assert_eq!(text(&k, "text/plain"), text(&k, "text/plain;charset=utf-8"));
        assert_eq!(dnd::parse_uri_list(text(&k, dnd::URI_LIST).as_bytes()), Ok(paths(&["/home/john/a b.txt", "/home/john/c.png"])));
    }

    #[test]
    fn a_path_that_is_not_whole_is_not_copied() {
        assert!(file_kinds(&paths(&["relative/x.txt"])).is_none());
        assert!(file_kinds(&[]).is_none());
        let k = file_kinds(&paths(&["relative.txt", "/home/john/ok.txt"])).unwrap();
        assert_eq!(text(&k, dnd::URI_LIST), "file:///home/john/ok.txt\r\n", "the whole ones go");
    }

    #[test]
    fn the_clipboard_is_asked_what_it_holds_by_the_kinds_it_names() {
        let named = |k: &[&str]| has_kinds(&k.iter().map(|s| s.to_string()).collect::<Vec<_>>());
        assert_eq!(named(&[]), OnClipboard::default());
        assert_eq!(named(&["image/png", "text/html"]), OnClipboard { text: false, image: true, files: false });
        assert_eq!(named(&["text/uri-list", "text/plain"]), OnClipboard { text: true, image: false, files: true });
        assert_eq!(named(&["UTF8_STRING"]), OnClipboard { text: true, image: false, files: false });
    }
}
