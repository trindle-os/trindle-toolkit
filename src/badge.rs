//! A count and a progress bar on the program's launcher icon in the dock, sent as com.canonical.Unity.LauncherEntry signals.

use std::collections::HashMap;

use zbus::zvariant::Value;

use super::bus::{Bus, Failed};

const IFACE: &str = "com.canonical.Unity.LauncherEntry";

pub struct Badge {
    bus: Bus,
    path: String,
    uri: String,
}

impl Badge {
    pub fn new(bus: &Bus, app_id: &str) -> Option<Badge> {
        super::keep::plain_id(app_id).then(|| Badge {
            bus: bus.clone(),
            path: format!("/org/trindle/launcher/{}", app_id.replace(['.', '-'], "_")),
            uri: format!("application://{app_id}.desktop"),
        })
    }

    fn send(&self, props: HashMap<&str, Value<'_>>) -> Result<(), Failed> {
        self.bus.emit(&self.path, IFACE, "Update", &(self.uri.as_str(), props))
    }

    pub fn count(&self, count: Option<u32>) -> Result<(), Failed> {
        self.send(HashMap::from([("count", Value::from(i64::from(count.unwrap_or(0)))), ("count-visible", Value::from(count.is_some()))]))
    }

    pub fn progress(&self, done: Option<f64>) -> Result<(), Failed> {
        let at = done.unwrap_or(0.0).clamp(0.0, 1.0);
        self.send(HashMap::from([("progress", Value::from(at)), ("progress-visible", Value::from(done.is_some()))]))
    }

    pub fn urgent(&self, urgent: bool) -> Result<(), Failed> {
        self.send(HashMap::from([("urgent", Value::from(urgent))]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bus::testing::{own_daemon, until};
    use crate::event::Loop;
    use std::cell::RefCell;
    use std::rc::Rc;
    use zbus::zvariant::OwnedValue;

    #[test]
    fn the_dock_hears_the_count_and_progress_for_the_program() {
        let d = own_daemon();
        let lp = Loop::new();
        let (us, dock) = (Bus::at(&lp, &d.1).expect("us"), Bus::at(&lp, &d.1).expect("dock"));
        let heard = Rc::new(RefCell::new(Vec::new()));
        let h = heard.clone();
        let _l = dock.listen(None, IFACE, Some("Update"), None, move |m| {
            let (uri, props) = m.body().deserialize::<(String, HashMap<String, OwnedValue>)>().expect("an update");
            let mut keys: Vec<(String, String)> = props.iter().map(|(k, v)| (k.clone(), format!("{:?}", Value::from(v.try_clone().unwrap())))).collect();
            keys.sort();
            h.borrow_mut().push((uri, keys));
        }).expect("listening");
        assert!(Badge::new(&us, "a/b").is_none());
        let badge = Badge::new(&us, "org.example.Mail").expect("a badge");
        badge.count(Some(3)).unwrap();
        badge.progress(Some(1.5)).unwrap();
        badge.count(None).unwrap();
        let h = heard.clone();
        assert!(until(&lp, move || h.borrow().len() == 3));
        let heard = heard.borrow();
        assert!(heard.iter().all(|(uri, _)| uri == "application://org.example.Mail.desktop"));
        let show = |v: Value<'_>| format!("{v:?}");
        assert_eq!(heard[0].1, [("count".to_string(), show(Value::from(3i64))), ("count-visible".to_string(), show(Value::from(true)))]);
        assert_eq!(heard[1].1, [("progress".to_string(), show(Value::from(1.0f64))), ("progress-visible".to_string(), show(Value::from(true)))]);
        assert_eq!(heard[2].1[1], ("count-visible".to_string(), show(Value::from(false))));
    }
}
