//! Small jobs for programs beside Trindle: whole-file writes, settings files, a line for the ring, a folder shown, a picture decoded off the loop.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

#[cfg(feature = "ui")]
use image::DynamicImage;

pub use trindle_common::ini::Ini;
pub use trindle_common::notify::{notify_line, status_line};

pub fn file_uri(path: &Path) -> Option<String> {
    use std::os::unix::ffi::OsStrExt;
    if !path.is_absolute() {
        return None;
    }
    let mut out = String::from("file://");
    for &b in path.as_os_str().as_bytes() {
        if b.is_ascii_alphanumeric() || b"/-_.~".contains(&b) {
            out.push(b as char);
        } else {
            let _ = write!(out, "%{b:02X}");
        }
    }
    Some(out)
}

pub fn open_folder(bus: &super::super::bus::Bus, lp: &super::super::event::Loop, folder: &Path, done: impl FnOnce(bool) + 'static) {
    let uri = file_uri(folder);
    let folder: PathBuf = folder.to_path_buf();
    let found = uri.is_some();
    let asked: Vec<&str> = uri.iter().map(String::as_str).collect();
    bus.call("org.freedesktop.FileManager1", "/org/freedesktop/FileManager1", "org.freedesktop.FileManager1")
        .method("ShowFolders", &(asked, ""), false, None, lp, move |r| {
            if r.is_ok() && found {
                return done(true);
            }
            let started = xdg_open(&folder)
                .stdin(std::process::Stdio::null()).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null()).spawn();
            done(started.map(|mut child| {
                std::thread::spawn(move || child.wait());
            }).is_ok());
        });
}

fn xdg_open(folder: &Path) -> std::process::Command {
    let mut command = std::process::Command::new("xdg-open");
    command.arg(Path::new(".").join(folder));
    command
}

#[cfg(feature = "ui")]
pub fn decode_in_background(path: PathBuf, done: impl FnOnce(Option<DynamicImage>) + 'static) {
    super::run::on_thread(move || super::super::picture::load(&path), done);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    use super::super::files::write_whole_private;

    fn folder() -> PathBuf {
        super::super::files::private_temp("plumbing").expect("a private folder")
    }

    #[test]
    fn xdg_open_is_given_the_folder_alone_and_never_an_option() {
        let args = |name: &str| xdg_open(Path::new(name)).get_args().map(|a| a.to_os_string()).collect::<Vec<_>>();
        assert_eq!(args("/tmp/My Files"), ["/tmp/My Files"]);
        assert_eq!(args("-x"), ["./-x"]);
    }

    #[test]
    fn a_file_is_replaced_whole_privately_and_leaves_nothing_behind() {
        use std::os::unix::fs::PermissionsExt;
        let dir = folder();
        let path = dir.join("state.txt");
        write_whole_private(&path, b"one").unwrap();
        write_whole_private(&path, b"two two").unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"two two");
        assert_eq!(fs::metadata(&path).unwrap().permissions().mode() & 0o777, 0o600);
        assert_eq!(fs::read_dir(&dir).unwrap().count(), 1, "no spare file is left");
        write_whole_private(&dir.join("made").join("x"), b"x").unwrap();
        assert_eq!(fs::read(dir.join("made").join("x")).unwrap(), b"x");
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn an_address_keeps_plain_characters_and_escapes_the_rest() {
        assert_eq!(file_uri(Path::new("/home/ann/My Pictures/é#1")).as_deref(), Some("file:///home/ann/My%20Pictures/%C3%A9%231"));
        assert_eq!(file_uri(Path::new("relative")), None);
    }

    #[test]
    fn a_folder_is_shown_through_the_file_manager_service() {
        use super::super::super::bus::testing::{own_daemon, until};
        use super::super::super::bus::{Asked, Bus, Offer, Props};
        use super::super::super::event::Loop;
        use std::{cell::{Cell, RefCell}, rc::Rc};
        let d = own_daemon();
        let lp = Loop::new();
        let (service, us) = (Bus::at(&lp, &d.1).expect("service"), Bus::at(&lp, &d.1).expect("us"));
        let asked = Rc::new(RefCell::new(Vec::new()));
        let a = asked.clone();
        let xml = r#"  <interface name="org.freedesktop.FileManager1"><method name="ShowFolders"/></interface>"#;
        let _served = service.serve("/org/freedesktop/FileManager1", Offer {
            name: "org.freedesktop.FileManager1".into(),
            xml: xml.into(),
            call: Box::new(move |q: Asked| {
                if let Ok((uris, _)) = q.message().body().deserialize::<(Vec<String>, String)>() {
                    a.borrow_mut().extend(uris);
                }
                q.reply(&());
            }),
            props: Box::new(Props::new),
            set: None,
        }).expect("served");
        let owned = Rc::new(Cell::new(false));
        let o = owned.clone();
        service.own("org.freedesktop.FileManager1", false, &lp, move |ok| o.set(ok));
        assert!(until(&lp, move || owned.get()));
        let said = Rc::new(Cell::new(None));
        let s = said.clone();
        open_folder(&us, &lp, Path::new("/tmp/My Files"), move |ok| s.set(Some(ok)));
        let s = said.clone();
        assert!(until(&lp, move || s.get().is_some()));
        assert_eq!((said.get(), asked.borrow().as_slice()), (Some(true), ["file:///tmp/My%20Files".to_string()].as_slice()));
    }
}
