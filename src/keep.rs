//! What a program keeps between runs in its own state folder: its window's size and whether it was maximised, and what it had open.

use std::io;
use std::os::unix::fs::DirBuilderExt;
use std::path::{Path, PathBuf};

use crate::window::Window;

const MOST: u32 = 16384;
const MOST_OPEN: usize = 256;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Shape {
    pub size: (u32, u32),
    pub maximised: bool,
}

impl Shape {
    pub fn new(size: (u32, u32)) -> Shape {
        Shape { size, maximised: false }
    }

    pub fn load(app_id: &str) -> Option<Shape> {
        Shape::load_in(&folder(app_id)?)
    }

    pub fn save(&self, app_id: &str) -> io::Result<()> {
        self.save_in(&folder(app_id).ok_or(io::ErrorKind::InvalidInput)?)
    }

    fn load_in(dir: &Path) -> Option<Shape> {
        Shape::parse(&trindle_common::files::read_small(&dir.join("window")).ok()?)
    }

    fn save_in(&self, dir: &Path) -> io::Result<()> {
        made(dir)?;
        trindle_common::files::write_private(&dir.join("window"), self.text().as_bytes())
    }

    pub fn note(&mut self, window: &Window) {
        self.maximised = window.maximised();
        if !self.maximised {
            self.size = window.size();
        }
    }

    pub fn apply(&self, window: &Window) {
        if self.maximised {
            window.set_maximised(true);
        }
    }

    fn text(&self) -> String {
        format!("{} {} {}\n", self.size.0, self.size.1, if self.maximised { "maximised" } else { "normal" })
    }

    fn parse(text: &str) -> Option<Shape> {
        let mut words = text.split_whitespace();
        let side = |w: Option<&str>| w?.parse::<u32>().ok().filter(|v| (1..=MOST).contains(v));
        let size = (side(words.next())?, side(words.next())?);
        let maximised = match words.next()? {
            "maximised" => true,
            "normal" => false,
            _ => return None,
        };
        words.next().is_none().then_some(Shape { size, maximised })
    }
}

pub fn keep_open(app_id: &str, open: &[String]) -> io::Result<()> {
    keep_open_in(&folder(app_id).ok_or(io::ErrorKind::InvalidInput)?, open)
}

pub fn opened_last(app_id: &str) -> Vec<String> {
    folder(app_id).map(|d| opened_last_in(&d)).unwrap_or_default()
}

fn keep_open_in(dir: &Path, open: &[String]) -> io::Result<()> {
    if open.len() > MOST_OPEN || open.iter().any(|o| o.is_empty() || !trindle_common::files::fits_a_line(o)) {
        return Err(io::ErrorKind::InvalidInput.into());
    }
    made(dir)?;
    let text: String = open.iter().flat_map(|o| [o.as_str(), "\n"]).collect();
    trindle_common::files::write_private(&dir.join("open"), text.as_bytes())
}

fn opened_last_in(dir: &Path) -> Vec<String> {
    let text = trindle_common::files::read_small(&dir.join("open")).unwrap_or_default();
    text.lines().filter(|l| !l.is_empty()).take(MOST_OPEN).map(str::to_string).collect()
}

fn made(dir: &Path) -> io::Result<()> {
    std::fs::DirBuilder::new().recursive(true).mode(0o700).create(dir)
}

pub fn folder(app_id: &str) -> Option<PathBuf> {
    plain_id(app_id).then(|| state_home().map(|s| s.join(app_id)))?
}

fn state_home() -> Option<PathBuf> {
    std::env::var_os("XDG_STATE_HOME").map(PathBuf::from).filter(|p| p.is_absolute())
        .or_else(|| trindle_common::files::home().map(|h| h.join(".local/state")))
}

pub(crate) fn plain_id(id: &str) -> bool {
    (1..=255).contains(&id.len()) && !id.starts_with('.') && !id.contains("..")
        && id.bytes().all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("trindle-keep-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn a_shape_comes_back_as_it_was_kept() {
        let dir = scratch("shape");
        assert_eq!(Shape::load_in(&dir), None, "nothing kept yet");
        let shape = Shape { size: (900, 640), maximised: true };
        shape.save_in(&dir).unwrap();
        assert_eq!(Shape::load_in(&dir), Some(shape));
        Shape::new((300, 200)).save_in(&dir).unwrap();
        assert_eq!(Shape::load_in(&dir), Some(Shape::new((300, 200))));
        let mode = std::fs::metadata(dir.join("window")).unwrap().permissions();
        assert_eq!(std::os::unix::fs::PermissionsExt::mode(&mode) & 0o777, 0o600);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_file_this_did_not_write_is_not_believed() {
        for bad in ["", "900", "900 640", "0 640 normal", "900 99999 normal", "-1 2 normal", "900 640 huge", "900 640 normal more"] {
            assert_eq!(Shape::parse(bad), None, "{bad:?}");
        }
        assert_eq!(Shape::parse(" 900  640 normal\n"), Some(Shape::new((900, 640))));
    }

    #[test]
    fn what_was_open_comes_back_in_order() {
        let dir = scratch("open");
        assert!(opened_last_in(&dir).is_empty());
        let open = vec!["/home/john/notes.txt".to_string(), "https://example.com/".to_string()];
        keep_open_in(&dir, &open).unwrap();
        assert_eq!(opened_last_in(&dir), open);
        keep_open_in(&dir, &[]).unwrap();
        assert!(opened_last_in(&dir).is_empty());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn an_item_with_a_line_break_is_refused() {
        let dir = scratch("break");
        assert!(keep_open_in(&dir, &["one\ntwo".to_string()]).is_err());
        assert!(keep_open_in(&dir, &[String::new()]).is_err());
        assert!(!dir.join("open").exists());
    }

    #[test]
    fn only_a_plain_app_id_has_a_folder() {
        for bad in ["", ".", "..", "../x", "a/b", "org.example..x", ".hidden", "a b", "é"] {
            assert!(!plain_id(bad), "{bad:?}");
        }
        assert!(plain_id("org.example.Notes"));
        assert!(plain_id("com.your-name.names_2"));
    }
}
