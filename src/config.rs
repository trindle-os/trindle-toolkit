//! Settings kept as one file per key in a config folder, as cosmic-config does, written whole and watched for changes made by other programs.

use std::io;
use std::os::unix::fs::DirBuilderExt;
use std::path::{Path, PathBuf};

use super::event::Loop;
use super::keep::plain_id;
use super::watch::{self, Live};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Config {
    folder: PathBuf,
}

impl Config {
    pub fn new(app_id: &str, version: u32) -> Option<Config> {
        plain_id(app_id).then(|| trindle_common::files::config_home().map(|c| Config::at(c.join(app_id).join(format!("v{version}")))))?
    }

    pub fn at(folder: impl Into<PathBuf>) -> Config {
        Config { folder: folder.into() }
    }

    pub fn folder(&self) -> &Path {
        &self.folder
    }

    pub fn get(&self, key: &str) -> Option<String> {
        self.read(key).ok()?
    }

    pub fn read(&self, key: &str) -> io::Result<Option<String>> {
        if !plain_id(key) {
            return Err(io::ErrorKind::InvalidInput.into());
        }
        match std::fs::read_to_string(self.folder.join(key)) {
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
            other => other.map(Some),
        }
    }

    pub fn set(&self, key: &str, value: &str) -> io::Result<()> {
        if !plain_id(key) {
            return Err(io::ErrorKind::InvalidInput.into());
        }
        self.made()?;
        trindle_common::files::write_private(&self.folder.join(key), value.as_bytes())
    }

    pub fn unset(&self, key: &str) -> io::Result<()> {
        if !plain_id(key) {
            return Err(io::ErrorKind::InvalidInput.into());
        }
        match std::fs::remove_file(self.folder.join(key)) {
            Err(e) if e.kind() != io::ErrorKind::NotFound => Err(e),
            _ => Ok(()),
        }
    }

    pub fn keys(&self) -> Vec<String> {
        let mut keys: Vec<String> = std::fs::read_dir(&self.folder).into_iter().flatten().flatten()
            .filter(|e| e.file_type().is_ok_and(|t| t.is_file()))
            .filter_map(|e| e.file_name().into_string().ok()).filter(|k| plain_id(k)).collect();
        keys.sort();
        keys
    }

    pub fn watch(&self, events: &Loop, mut changed: impl FnMut(Vec<String>) + 'static) -> io::Result<Live> {
        self.made()?;
        let me = self.clone();
        watch::live(events, &self.folder, move |files, again| {
            let mut keys: Vec<String> = files.iter().filter_map(|f| f.file_name()?.to_str().map(str::to_string))
                .filter(|k| plain_id(k)).collect();
            if again {
                keys = me.keys();
            }
            if !keys.is_empty() {
                changed(keys);
            }
        })
    }

    fn made(&self) -> io::Result<()> {
        std::fs::DirBuilder::new().recursive(true).mode(0o700).create(&self.folder)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;
    use super::super::bus::testing::until;

    fn folder(name: &str) -> PathBuf {
        let p = std::env::temp_dir().join(format!("trindle-config-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&p);
        p
    }

    #[test]
    fn a_key_is_set_read_listed_and_forgotten() {
        let dir = folder("keys");
        let c = Config::at(dir.join("v1"));
        assert_eq!(c.get("accent"), None);
        c.set("accent", "#3366cc").unwrap();
        c.set("is_dark", "true").unwrap();
        assert_eq!(c.get("accent").as_deref(), Some("#3366cc"));
        assert_eq!(c.keys(), ["accent", "is_dark"]);
        c.unset("accent").unwrap();
        c.unset("accent").unwrap();
        assert_eq!(c.keys(), ["is_dark"]);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn read_tells_a_missing_key_from_one_that_cannot_be_read() {
        let dir = folder("read");
        let c = Config::at(&dir);
        assert_eq!(c.read("accent").unwrap(), None);
        c.set("accent", "#3366cc").unwrap();
        assert_eq!(c.read("accent").unwrap().as_deref(), Some("#3366cc"));
        std::fs::write(dir.join("not_text"), [0xff, 0xfe, 0x00]).unwrap();
        std::fs::create_dir(dir.join("a_folder")).unwrap();
        for key in ["not_text", "a_folder"] {
            assert!(c.read(key).is_err(), "{key}");
            assert_eq!(c.get(key), None);
        }
        assert_eq!(c.read("../x").unwrap_err().kind(), io::ErrorKind::InvalidInput);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn a_key_that_could_reach_out_of_the_folder_is_refused() {
        let c = Config::at(folder("refused"));
        for key in ["../x", "a/b", ".hidden", ""] {
            assert!(c.set(key, "1").is_err(), "{key}");
            assert_eq!(c.get(key), None);
            assert!(c.unset(key).is_err(), "{key}");
        }
        assert_eq!(Config::new("../evil", 1), None);
        let _ = std::fs::remove_dir_all(c.folder());
    }

    #[test]
    fn a_change_by_another_program_is_heard_once_per_key() {
        let dir = folder("watch");
        let c = Config::at(&dir);
        let lp = Loop::new();
        let heard = Rc::new(RefCell::new(Vec::new()));
        let h = heard.clone();
        let _live = c.watch(&lp, move |keys| h.borrow_mut().extend(keys)).unwrap();
        Config::at(&dir).set("is_dark", "false").unwrap();
        let h = heard.clone();
        assert!(until(&lp, move || !h.borrow().is_empty()));
        assert_eq!(*heard.borrow(), ["is_dark"]);
        std::fs::remove_dir_all(dir).unwrap();
    }
}
