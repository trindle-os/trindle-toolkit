//! The desktop's look (light or dark, accent, contrast, motion), read from the settings portal and read again on a change.

use std::collections::HashMap;
use std::rc::Rc;
use std::cell::{Cell, RefCell};
use std::path::{Path, PathBuf};

use zbus::zvariant::OwnedValue;

use super::bus::{Bus, Failed, Listen};
use super::colour::Colour;
use super::config::Config;
use super::watch;
use super::event::Loop;

const PORTAL: &str = "org.freedesktop.portal.Desktop";
const PATH: &str = "/org/freedesktop/portal/desktop";
const SETTINGS: &str = "org.freedesktop.portal.Settings";
const APPEARANCE: &str = "org.freedesktop.appearance";
const INTERFACE: &str = "org.gnome.desktop.interface";

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Look {
    pub light: Option<bool>,
    pub accent: Option<Colour>,
    pub more_contrast: bool,
    pub reduce_motion: bool,
}

type All = HashMap<String, HashMap<String, OwnedValue>>;

pub fn look_from(all: &All) -> Look {
    let get = |ns: &str, key: &str| all.get(ns).and_then(|m| m.get(key));
    let number = |v: &OwnedValue| u32::try_from(v).ok();
    Look {
        light: get(APPEARANCE, "color-scheme").and_then(number).and_then(|n| match n {
            1 => Some(false),
            2 => Some(true),
            _ => None,
        }),
        accent: get(APPEARANCE, "accent-color")
            .and_then(|v| match &**v {
                zbus::zvariant::Value::Structure(s) => match s.fields() {
                    [r, g, b] => Some((f64::try_from(r).ok()?, f64::try_from(g).ok()?, f64::try_from(b).ok()?)),
                    _ => None,
                },
                _ => None,
            })
            .filter(|(r, g, b)| [r, g, b].iter().all(|c| (0.0..=1.0).contains(*c)))
            .map(|(r, g, b)| Colour::rgb(r as f32, g as f32, b as f32)),
        more_contrast: get(APPEARANCE, "contrast").and_then(number) == Some(1),
        reduce_motion: get(INTERFACE, "enable-animations").and_then(|v| bool::try_from(v).ok()) == Some(false),
    }
}

fn cosmic_folder() -> Option<PathBuf> {
    trindle_common::files::config_home().map(|c| c.join("cosmic/com.system76.CosmicTheme.Mode/v1"))
}

pub fn look_from_cosmic(folder: &Path) -> Look {
    let light = Config::at(folder).get("is_dark").and_then(|v| match v.trim() {
        "true" => Some(false),
        "false" => Some(true),
        _ => None,
    });
    Look { light, ..Look::default() }
}

pub fn read(bus: &Bus, lp: &Loop, done: impl FnOnce(Look) + 'static) {
    let asked = (vec![APPEARANCE, INTERFACE],);
    bus.call(PORTAL, PATH, SETTINGS).method("ReadAll", &asked, false, None, lp, move |r| {
        let all = r.ok().and_then(|m| m.body().deserialize::<All>().ok());
        let look = match all {
            Some(all) => look_from(&all),
            None => cosmic_folder().map(|f| look_from_cosmic(&f)).unwrap_or_default(),
        };
        super::motion::reduce(look.reduce_motion);
        done(look);
    });
}

pub fn follow(bus: &Bus, lp: &Loop, changed: impl FnMut(Look) + 'static) -> Result<Listen, Failed> {
    let changed = Rc::new(RefCell::new(changed));
    let (asked, shown) = (Rc::new(Cell::new(0u64)), Rc::new(Cell::new(0u64)));
    let ask = Rc::new({
        let (b, l) = (bus.clone(), lp.clone());
        move || {
            let n = asked.get() + 1;
            asked.set(n);
            let (changed, shown) = (changed.clone(), shown.clone());
            read(&b, &l, move |look| {
                if n > shown.get() {
                    shown.set(n);
                    (changed.borrow_mut())(look);
                }
            });
        }
    });
    let again = ask.clone();
    let cosmic = cosmic_folder().filter(|f| f.is_dir()).and_then(|f| {
        let again = ask.clone();
        watch::live(lp, &f, move |_, _| again()).ok()
    });
    let listening = bus.listen(None, SETTINGS, Some("SettingChanged"), Some(PATH), move |m| {
        let _ = &cosmic;
        let ours = m.body().deserialize::<(String, String, OwnedValue)>().is_ok_and(|(ns, _, _)| ns == APPEARANCE || ns == INTERFACE);
        if ours {
            again();
        }
    })?;
    ask();
    Ok(listening)
}

impl Look {
    pub fn accent_css(&self) -> Option<String> {
        self.accent.map(Colour::css)
    }
}

#[cfg(test)]
mod tests {
    use super::super::bus::testing::{own_daemon, until};
    use super::super::bus::{Asked, Offer, Props};
    use super::*;
    use zbus::zvariant::Value;

    fn canned() -> All {
        let v = |v: Value<'_>| OwnedValue::try_from(v).expect("plain value");
        HashMap::from([
            (APPEARANCE.to_string(), HashMap::from([
                ("color-scheme".to_string(), v(Value::U32(1))),
                ("accent-color".to_string(), v(Value::from((0.2f64, 0.4f64, 0.8f64)))),
                ("contrast".to_string(), v(Value::U32(1))),
            ])),
            (INTERFACE.to_string(), HashMap::from([("enable-animations".to_string(), v(Value::Bool(false)))])),
        ])
    }

    #[test]
    fn the_portal_answer_becomes_light_accent_contrast_and_motion() {
        let look = look_from(&canned());
        assert_eq!(look.light, Some(false));
        assert_eq!(look.accent_css().as_deref(), Some("#3366cc"));
        assert!(look.more_contrast && look.reduce_motion);
        assert_eq!(look_from(&All::new()), Look::default());
    }

    #[test]
    fn cosmic_says_light_or_dark_when_there_is_no_portal() {
        let dir = std::env::temp_dir().join(format!("trindle-cosmic-mode-{}", std::process::id()));
        assert_eq!(look_from_cosmic(&dir), Look::default());
        let c = Config::at(&dir);
        c.set("is_dark", "true\n").unwrap();
        assert_eq!(look_from_cosmic(&dir).light, Some(false));
        c.set("is_dark", "false").unwrap();
        assert_eq!(look_from_cosmic(&dir).light, Some(true));
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn a_change_signalled_by_the_portal_is_read_again() {
        let d = own_daemon();
        let lp = Loop::new();
        let (portal, us) = (Bus::at(&lp, &d.1).expect("portal"), Bus::at(&lp, &d.1).expect("us"));
        let scheme = Rc::new(std::cell::Cell::new(1u32));
        let s = scheme.clone();
        let xml = r#"  <interface name="org.freedesktop.portal.Settings"><method name="ReadAll"/></interface>"#;
        let _served = portal.serve(PATH, Offer {
            name: SETTINGS.into(),
            xml: xml.into(),
            call: Box::new(move |a: Asked| {
                let mut all = canned();
                all.get_mut(APPEARANCE).unwrap().insert("color-scheme".into(), OwnedValue::try_from(Value::U32(s.get())).expect("plain value"));
                a.reply(&all);
            }),
            props: Box::new(Props::new),
            set: None,
        }).expect("served");
        let owned = Rc::new(std::cell::Cell::new(false));
        let o = owned.clone();
        portal.own(PORTAL, false, &lp, move |ok| o.set(ok));
        assert!(until(&lp, move || owned.get()));

        let seen = Rc::new(RefCell::new(Vec::new()));
        let sn = seen.clone();
        let _listen = follow(&us, &lp, move |look| sn.borrow_mut().push(look.light)).expect("following");
        let sn = seen.clone();
        assert!(until(&lp, move || sn.borrow().len() == 1));
        scheme.set(2);
        portal.emit(PATH, SETTINGS, "SettingChanged", &(APPEARANCE, "color-scheme", Value::U32(2))).expect("signal");
        let sn = seen.clone();
        assert!(until(&lp, move || sn.borrow().len() == 2));
        assert_eq!(*seen.borrow(), [Some(false), Some(true)]);
    }
}
