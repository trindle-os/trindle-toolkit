//! A player's controls offered to the desktop over MPRIS (org.mpris.MediaPlayer2), so the panel's media applet and the media keys reach it.

use crate::t;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use zbus::zvariant::{ObjectPath, OwnedValue, Value};

use super::bus::{Asked, Bus, Failed, Offer, Props, Served};
use super::event::Loop;

const PATH: &str = "/org/mpris/MediaPlayer2";
const ROOT: &str = "org.mpris.MediaPlayer2";
const PLAYER: &str = "org.mpris.MediaPlayer2.Player";
const NO_TRACK: &str = "/org/mpris/MediaPlayer2/TrackList/NoTrack";

const ROOT_XML: &str = r#"  <interface name="org.mpris.MediaPlayer2">
    <method name="Raise"/>
    <method name="Quit"/>
    <property name="CanQuit" type="b" access="read"/>
    <property name="CanRaise" type="b" access="read"/>
    <property name="HasTrackList" type="b" access="read"/>
    <property name="Identity" type="s" access="read"/>
    <property name="DesktopEntry" type="s" access="read"/>
    <property name="SupportedUriSchemes" type="as" access="read"/>
    <property name="SupportedMimeTypes" type="as" access="read"/>
  </interface>"#;

const PLAYER_XML: &str = r#"  <interface name="org.mpris.MediaPlayer2.Player">
    <method name="Next"/>
    <method name="Previous"/>
    <method name="Pause"/>
    <method name="PlayPause"/>
    <method name="Stop"/>
    <method name="Play"/>
    <method name="Seek"><arg type="x" direction="in"/></method>
    <method name="SetPosition"><arg type="o" direction="in"/><arg type="x" direction="in"/></method>
    <method name="OpenUri"><arg type="s" direction="in"/></method>
    <signal name="Seeked"><arg type="x"/></signal>
    <property name="PlaybackStatus" type="s" access="read"/>
    <property name="Rate" type="d" access="read"/>
    <property name="MinimumRate" type="d" access="read"/>
    <property name="MaximumRate" type="d" access="read"/>
    <property name="Volume" type="d" access="read"/>
    <property name="Metadata" type="a{sv}" access="read"/>
    <property name="Position" type="x" access="read"/>
    <property name="CanGoNext" type="b" access="read"/>
    <property name="CanGoPrevious" type="b" access="read"/>
    <property name="CanPlay" type="b" access="read"/>
    <property name="CanPause" type="b" access="read"/>
    <property name="CanSeek" type="b" access="read"/>
    <property name="CanControl" type="b" access="read"/>
  </interface>"#;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Control {
    Play,
    Pause,
    PlayPause,
    Stop,
    Next,
    Previous,
    Seek(i64),
    SetPosition(i64),
    Raise,
    Quit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Playing {
    Playing,
    Paused,
    Stopped,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Track {
    pub title: String,
    pub artists: Vec<String>,
    pub album: String,
    pub length: i64,
    pub art: String,
}

struct Shown {
    identity: String,
    entry: String,
    playing: Playing,
    track: Option<(u64, Track)>,
    position: i64,
    next: bool,
    previous: bool,
}

pub struct Media {
    bus: Bus,
    lp: Loop,
    name: String,
    shown: Rc<RefCell<Shown>>,
    tracks: std::cell::Cell<u64>,
    _served: [Served; 2],
}

fn value(v: Value<'_>) -> OwnedValue {
    v.try_into().expect("a plain value")
}

fn track_path(id: u64) -> String {
    format!("/org/trindle/track/{id}")
}

fn metadata(s: &Shown) -> OwnedValue {
    let mut m: HashMap<&str, Value<'_>> = HashMap::new();
    match &s.track {
        None => {
            m.insert("mpris:trackid", Value::from(ObjectPath::try_from(NO_TRACK).expect("a path")));
        }
        Some((id, t)) => {
            m.insert("mpris:trackid", Value::from(ObjectPath::try_from(track_path(*id)).expect("a path")));
            m.insert("xesam:title", Value::from(t.title.as_str()));
            m.insert("xesam:artist", Value::from(t.artists.clone()));
            if !t.album.is_empty() {
                m.insert("xesam:album", Value::from(t.album.as_str()));
            }
            if t.length > 0 {
                m.insert("mpris:length", Value::from(t.length));
            }
            if !t.art.is_empty() {
                m.insert("mpris:artUrl", Value::from(t.art.as_str()));
            }
        }
    }
    value(Value::from(m))
}

fn status_word(p: Playing) -> &'static str {
    match p {
        Playing::Playing => "Playing",
        Playing::Paused => "Paused",
        Playing::Stopped => "Stopped",
    }
}

fn player_props(s: &Shown) -> Props {
    let mut p = Props::new();
    let has = s.track.is_some();
    p.insert("PlaybackStatus".into(), value(Value::from(status_word(s.playing))));
    for k in ["Rate", "MinimumRate", "MaximumRate", "Volume"] {
        p.insert(k.into(), OwnedValue::from(1.0f64));
    }
    p.insert("Metadata".into(), metadata(s));
    p.insert("Position".into(), OwnedValue::from(s.position));
    for (k, v) in [("CanGoNext", s.next), ("CanGoPrevious", s.previous), ("CanPlay", has), ("CanPause", has), ("CanSeek", has), ("CanControl", true)] {
        p.insert(k.into(), OwnedValue::from(v));
    }
    p
}

fn root_props(s: &Shown) -> Props {
    let mut p = Props::new();
    for (k, v) in [("CanQuit", true), ("CanRaise", true), ("HasTrackList", false)] {
        p.insert(k.into(), OwnedValue::from(v));
    }
    p.insert("Identity".into(), value(Value::from(s.identity.as_str())));
    p.insert("DesktopEntry".into(), value(Value::from(s.entry.as_str())));
    for k in ["SupportedUriSchemes", "SupportedMimeTypes"] {
        p.insert(k.into(), value(Value::from(Vec::<String>::new())));
    }
    p
}

fn answer(a: Asked, shown: &RefCell<Shown>, control: &mut impl FnMut(Control)) {
    let body = a.message().body();
    let asked = match a.member().as_str() {
        "Raise" => Some(Control::Raise),
        "Quit" => Some(Control::Quit),
        "Next" => Some(Control::Next),
        "Previous" => Some(Control::Previous),
        "Pause" => Some(Control::Pause),
        "PlayPause" => Some(Control::PlayPause),
        "Stop" => Some(Control::Stop),
        "Play" => Some(Control::Play),
        "Seek" => match body.deserialize::<i64>() {
            Ok(by) => Some(Control::Seek(by)),
            Err(_) => return a.fail("org.freedesktop.DBus.Error.InvalidArgs", t!("kit-mpris-seek")),
        },
        "SetPosition" => match body.deserialize::<(ObjectPath<'_>, i64)>() {
            Ok((track, at)) => {
                let current = shown.borrow().track.as_ref().map(|(id, t)| (track_path(*id), t.length));
                current.filter(|(path, length)| track.as_str() == path && at >= 0 && (*length == 0 || at <= *length))
                    .map(|_| Control::SetPosition(at))
            }
            Err(_) => return a.fail("org.freedesktop.DBus.Error.InvalidArgs", t!("kit-mpris-set-position")),
        },
        "OpenUri" => return a.fail("org.freedesktop.DBus.Error.NotSupported", t!("kit-mpris-no-uri")),
        _ => return a.fail("org.freedesktop.DBus.Error.UnknownMethod", t!("kit-bus-no-method-here")),
    };
    if let Some(c) = asked {
        control(c);
    }
    a.reply(&());
}

impl Media {
    pub fn new(bus: &Bus, lp: &Loop, app_id: &str, identity: &str, control: impl FnMut(Control) + 'static, done: impl FnOnce(bool) + 'static) -> Result<Media, Failed> {
        if !super::keep::plain_id(app_id) {
            return Err(Failed::new("org.freedesktop.DBus.Error.InvalidArgs", t!("kit-mpris-app-id")));
        }
        let shown = Rc::new(RefCell::new(Shown { identity: identity.into(), entry: app_id.into(), playing: Playing::Stopped,
                                                 track: None, position: 0, next: false, previous: false }));
        let control = Rc::new(RefCell::new(control));
        let (s, c) = (shown.clone(), control.clone());
        let props = shown.clone();
        let root = bus.serve(PATH, Offer {
            name: ROOT.into(),
            xml: ROOT_XML.into(),
            call: Box::new(move |a: Asked| answer(a, &s, &mut *c.borrow_mut())),
            props: Box::new(move || root_props(&props.borrow())),
            set: None,
        })?;
        let (s, props) = (shown.clone(), shown.clone());
        let player = bus.serve(PATH, Offer {
            name: PLAYER.into(),
            xml: PLAYER_XML.into(),
            call: Box::new(move |a: Asked| answer(a, &s, &mut *control.borrow_mut())),
            props: Box::new(move || player_props(&props.borrow())),
            set: None,
        })?;
        let name = format!("{ROOT}.{app_id}");
        bus.own(&name, false, lp, done);
        Ok(Media { bus: bus.clone(), lp: lp.clone(), name, shown, tracks: std::cell::Cell::new(0), _served: [root, player] })
    }

    fn tell(&self, keys: &[&str]) {
        let all = player_props(&self.shown.borrow());
        let changed: HashMap<&str, Value<'_>> = keys.iter().filter_map(|k| Some((*k, Value::from(all.get(*k)?.try_clone().ok()?)))).collect();
        let _ = self.bus.changed(PATH, PLAYER, &changed);
    }

    pub fn set_playing(&self, playing: Playing) {
        self.shown.borrow_mut().playing = playing;
        self.tell(&["PlaybackStatus"]);
    }

    pub fn set_track(&self, track: Option<Track>) {
        let id = self.tracks.get() + 1;
        self.tracks.set(id);
        let mut s = self.shown.borrow_mut();
        s.track = track.map(|t| (id, t));
        s.position = 0;
        drop(s);
        self.tell(&["Metadata", "CanPlay", "CanPause", "CanSeek"]);
    }

    pub fn set_can(&self, next: bool, previous: bool) {
        let mut s = self.shown.borrow_mut();
        (s.next, s.previous) = (next, previous);
        drop(s);
        self.tell(&["CanGoNext", "CanGoPrevious"]);
    }

    pub fn set_position(&self, at: i64, jumped: bool) {
        self.shown.borrow_mut().position = at;
        if jumped {
            let _ = self.bus.emit(PATH, PLAYER, "Seeked", &at);
        }
    }
}

impl Drop for Media {
    fn drop(&mut self) {
        self.bus.release(&self.name, &self.lp);
    }
}

#[cfg(test)]
mod tests {
    use super::super::bus::testing::{own_daemon, until};
    use super::super::bus::get_str;
    use super::*;
    use std::cell::Cell;

    #[test]
    fn the_desktop_reads_the_track_and_controls_the_player() {
        let d = own_daemon();
        let lp = Loop::new();
        let (us, panel) = (Bus::at(&lp, &d.1).expect("us"), Bus::at(&lp, &d.1).expect("panel"));
        let asked = Rc::new(RefCell::new(Vec::new()));
        let a = asked.clone();
        let owned = Rc::new(Cell::new(None));
        let o = owned.clone();
        let media = Media::new(&us, &lp, "org.trindle.Test", "Test player", move |c| a.borrow_mut().push(c), move |ok| o.set(Some(ok))).expect("served");
        let o = owned.clone();
        assert!(until(&lp, move || o.get().is_some()));
        assert_eq!(owned.get(), Some(true));
        media.set_track(Some(Track { title: "First light".into(), artists: vec!["John Smith".into()], length: 180_000_000, ..Track::default() }));
        media.set_playing(Playing::Playing);

        let player = panel.call("org.mpris.MediaPlayer2.org.trindle.Test", PATH, PLAYER);
        let props = Rc::new(RefCell::new(None));
        let p = props.clone();
        player.get_all(&lp, move |r| *p.borrow_mut() = r.ok());
        let p = props.clone();
        assert!(until(&lp, move || p.borrow().is_some()));
        let all = props.borrow_mut().take().unwrap();
        assert_eq!(get_str(&all, "PlaybackStatus"), "Playing");
        let meta = HashMap::<String, OwnedValue>::try_from(all["Metadata"].try_clone().unwrap()).unwrap();
        assert_eq!(String::try_from(meta["xesam:title"].try_clone().unwrap()).unwrap(), "First light");
        assert!(bool::try_from(&all["CanPause"]).unwrap());

        player.method("PlayPause", &(), false, None, &lp, |_| {});
        player.method("Seek", &(-5_000_000i64,), false, None, &lp, |_| {});
        let track = ObjectPath::try_from(track_path(1)).unwrap();
        player.method("SetPosition", &(track.clone(), 60_000_000i64), false, None, &lp, |_| {});
        player.method("SetPosition", &(track, 900_000_000i64), false, None, &lp, |_| {});
        player.method("SetPosition", &(ObjectPath::try_from(track_path(9)).unwrap(), 1i64), false, None, &lp, |_| {});
        panel.call("org.mpris.MediaPlayer2.org.trindle.Test", PATH, ROOT).method("Raise", &(), false, None, &lp, |_| {});
        let a = asked.clone();
        assert!(until(&lp, move || a.borrow().len() == 4));
        assert_eq!(*asked.borrow(), [Control::PlayPause, Control::Seek(-5_000_000), Control::SetPosition(60_000_000), Control::Raise]);
    }

    #[test]
    fn an_app_id_that_is_not_plain_is_refused() {
        let d = own_daemon();
        let lp = Loop::new();
        let us = Bus::at(&lp, &d.1).expect("us");
        assert!(Media::new(&us, &lp, "../x", "X", |_| {}, |_| {}).is_err());
    }
}
