//! Hears when files change: the kernel says which file in a watched folder was written, made, moved in or removed. Folders rather than files, because Trindle writes a file by moving a finished copy over it.

use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::ffi::{CString, OsStr};
use std::os::fd::{AsRawFd, OwnedFd, RawFd};
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use super::event::{Flow, Loop};

const CHANGES: u32 = libc::IN_CLOSE_WRITE | libc::IN_MOVED_TO | libc::IN_MOVED_FROM | libc::IN_CREATE | libc::IN_DELETE
    | libc::IN_DELETE_SELF | libc::IN_MOVE_SELF;

pub struct Watch {
    fd: OwnedFd,
    folders: RefCell<HashMap<i32, Vec<PathBuf>>>,
    unplaced: Cell<bool>,
    subfolders: bool,
}

impl Watch {
    pub fn new() -> std::io::Result<Self> {
        let fd = crate::sys::fd::watch(libc::IN_NONBLOCK | libc::IN_CLOEXEC)?;
        Ok(Self { fd, folders: RefCell::default(), unplaced: Cell::new(false), subfolders: false })
    }

    pub fn with_subfolders(mut self) -> Self {
        self.subfolders = true;
        self
    }

    pub fn folder(&mut self, folder: &Path) -> std::io::Result<()> {
        let mark = add(&self.fd, folder)?;
        self.known(mark, folder);
        Ok(())
    }

    pub fn forget(&mut self, folder: &Path) {
        let folders = self.folders.get_mut();
        let marks: Vec<i32> = folders.iter().filter(|(_, f)| f.iter().any(|p| p == folder)).map(|(m, _)| *m).collect();
        for mark in marks {
            let paths = folders.entry(mark).or_default();
            paths.retain(|p| p != folder);
            if paths.is_empty() {
                crate::sys::fd::unmark(self.fd.as_raw_fd(), mark);
                folders.remove(&mark);
            }
        }
    }

    pub fn lend(&self) -> Option<OwnedFd> {
        self.fd.try_clone().ok()
    }

    pub fn known(&mut self, mark: i32, folder: &Path) {
        let paths = self.folders.get_mut().entry(mark).or_default();
        if !paths.iter().any(|p| p == folder) {
            paths.push(folder.to_path_buf());
        }
    }

    pub fn fd(&self) -> RawFd {
        self.fd.as_raw_fd()
    }

    pub fn unplaced(&self) -> bool {
        self.unplaced.take()
    }

    pub fn changed(&self) -> Vec<PathBuf> {
        let (mut found, mut seen) = (Vec::new(), HashSet::new());
        let mut buf = [0u8; 4096];
        loop {
            let n = crate::sys::fd::read(self.fd.as_raw_fd(), &mut buf);
            if n <= 0 {
                break;
            }
            let mut at = 0;
            let head = std::mem::size_of::<libc::inotify_event>();
            while at + head <= n as usize {
                let Some(event) = crate::sys::fd::watch_event(&buf[at..]) else { break };
                let name = &buf[at + head..at + head + event.len as usize];
                let name = &name[..name.iter().position(|b| *b == 0).unwrap_or(name.len())];
                if event.mask & libc::IN_Q_OVERFLOW != 0 {
                    self.unplaced.set(true);
                } else if event.mask & (libc::IN_DELETE_SELF | libc::IN_MOVE_SELF) != 0 {
                    self.unplaced.set(true);
                    if event.mask & libc::IN_MOVE_SELF != 0 {
                        crate::sys::fd::unmark(self.fd.as_raw_fd(), event.wd);
                        self.folders.borrow_mut().remove(&event.wd);
                    }
                } else if event.mask & libc::IN_IGNORED != 0 {
                    self.folders.borrow_mut().remove(&event.wd);
                } else if event.mask & libc::IN_ISDIR != 0 && !self.subfolders {
                } else if let Some(folders) = self.folders.borrow().get(&event.wd) {
                    for folder in folders {
                        let path = folder.join(OsStr::from_bytes(name));
                        if seen.insert(path.clone()) {
                            found.push(path);
                        }
                    }
                } else if event.wd >= 0 {
                    self.unplaced.set(true);
                }
                at += head + event.len as usize;
            }
        }
        found
    }
}

pub struct Live {
    watch: Rc<RefCell<Watch>>,
    folder: PathBuf,
    stopped: Rc<Cell<bool>>,
}

impl Drop for Live {
    fn drop(&mut self) {
        self.stopped.set(true);
        let mut watch = self.watch.borrow_mut();
        let marked = watch.folders.get_mut().values().any(|f| f.contains(&self.folder));
        watch.forget(&self.folder);
        if !marked {
            let mark = crate::sys::fd::mark(watch.fd(), c"/", libc::IN_ATTRIB | libc::IN_MASK_CREATE);
            if mark >= 0 {
                crate::sys::fd::unmark(watch.fd(), mark);
            }
        }
    }
}

pub fn live(events: &Loop, folder: &Path, mut changed: impl FnMut(Vec<PathBuf>, bool) + 'static) -> std::io::Result<Live> {
    let mut watch = Watch::new()?.with_subfolders();
    watch.folder(folder)?;
    let fd = watch.fd();
    let (watch, stopped) = (Rc::new(RefCell::new(watch)), Rc::new(Cell::new(false)));
    let (heard, done) = (watch.clone(), stopped.clone());
    events.watch(fd, move |_| {
        let (files, again) = {
            let w = heard.borrow();
            (w.changed(), w.unplaced())
        };
        if done.get() {
            return Flow::Stop;
        }
        if !files.is_empty() || again {
            changed(files, again);
        }
        Flow::Continue
    });
    Ok(Live { watch, folder: folder.to_path_buf(), stopped })
}

pub fn add(ear: &OwnedFd, folder: &Path) -> std::io::Result<i32> {
    let name = CString::new(folder.as_os_str().as_bytes())?;
    let mark = crate::sys::fd::mark(ear.as_raw_fd(), &name, CHANGES | libc::IN_ONLYDIR);
    if mark < 0 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(mark)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Dir(PathBuf);

    impl Drop for Dir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn dir(name: &str) -> Dir {
        static COUNT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let n = COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!("trindle-watch-{}-{n}-{name}", std::process::id()));
        std::fs::create_dir(&path).unwrap();
        Dir(path)
    }

    #[test]
    fn hears_a_file_written_and_one_moved_in() {
        let guard = dir("files");
        let dir = guard.0.clone();
        let mut watch = Watch::new().unwrap();
        watch.folder(&dir).unwrap();
        assert!(watch.changed().is_empty());
        std::fs::write(dir.join("a.conf"), "x").unwrap();
        std::fs::write(dir.join("b.tmp"), "y").unwrap();
        std::fs::rename(dir.join("b.tmp"), dir.join("b.conf")).unwrap();
        let heard = watch.changed();
        assert!(heard.contains(&dir.join("a.conf")));
        assert!(heard.contains(&dir.join("b.conf")));
        assert_eq!(heard.iter().filter(|p| **p == dir.join("a.conf")).count(), 1);
        assert!(watch.changed().is_empty());
        assert!(watch.folder(&dir.join("missing")).is_err());
        let inner = dir.join("inner");
        std::fs::create_dir_all(&inner).unwrap();
        let (ear, at) = (watch.lend().expect("a second hold"), inner.clone());
        let mark = std::thread::spawn(move || add(&ear, &at)).join().unwrap().unwrap();
        watch.known(mark, &inner);
        std::fs::write(inner.join("c.conf"), "z").unwrap();
        assert!(watch.changed().contains(&inner.join("c.conf")));
        let late = dir.join("late");
        std::fs::create_dir_all(&late).unwrap();
        let ear = watch.lend().expect("a hold");
        let mark = add(&ear, &late).unwrap();
        let _ = watch.changed();
        std::fs::write(late.join("d.conf"), "w").unwrap();
        assert!(watch.changed().is_empty());
        assert!(watch.unplaced(), "news of an unnoted folder was dropped silently");
        assert!(!watch.unplaced(), "said twice");
        watch.known(mark, &late);
        let _ = watch.changed();
        std::fs::rename(dir.join("a.conf"), inner.join("a.conf")).unwrap();
        assert!(watch.changed().contains(&dir.join("a.conf")));
    }

    #[test]
    fn a_sub_folder_is_no_changed_file_and_a_file_is_no_folder_to_watch() {
        let guard = dir("sub");
        let mut watch = Watch::new().unwrap();
        watch.folder(&guard.0).unwrap();
        std::fs::create_dir(guard.0.join("inner")).unwrap();
        std::fs::write(guard.0.join("f"), "x").unwrap();
        assert_eq!(watch.changed(), [guard.0.join("f")]);
        assert!(watch.folder(&guard.0.join("f")).is_err());
    }

    #[test]
    fn two_paths_to_one_folder_are_both_heard_and_forgotten_apart() {
        let guard = dir("alias");
        let real = guard.0.join("real");
        std::fs::create_dir(&real).unwrap();
        let link = guard.0.join("link");
        std::os::unix::fs::symlink(&real, &link).unwrap();
        let mut watch = Watch::new().unwrap();
        watch.folder(&real).unwrap();
        watch.folder(&link).unwrap();
        std::fs::write(real.join("a"), "x").unwrap();
        let mut heard = watch.changed();
        heard.sort();
        assert_eq!(heard, [link.join("a"), real.join("a")]);
        watch.forget(&real);
        std::fs::write(real.join("b"), "x").unwrap();
        assert_eq!(watch.changed(), [link.join("b")]);
    }

    #[test]
    fn a_folder_that_goes_is_forgotten_and_said_to_need_reading_again() {
        let guard = dir("gone");
        let folder = guard.0.join("watched");
        std::fs::create_dir_all(&folder).unwrap();
        let mut watch = Watch::new().unwrap();
        watch.folder(&folder).unwrap();
        std::fs::remove_dir(&folder).unwrap();
        let heard = watch.changed();
        assert!(!heard.contains(&folder) && !heard.contains(&folder.join("")), "the folder itself is not a changed file");
        assert!(watch.unplaced());
        assert!(watch.folders.borrow().is_empty());
        std::fs::create_dir_all(&folder).unwrap();
        watch.folder(&folder).unwrap();
        std::fs::write(folder.join("a"), "x").unwrap();
        assert!(watch.changed().contains(&folder.join("a")));
        watch.forget(&folder);
        std::fs::write(folder.join("b"), "x").unwrap();
        assert!(watch.changed().is_empty());
    }

    #[test]
    fn a_live_folder_hears_files_and_sub_folders_on_the_loop_until_dropped() {
        let guard = dir("live");
        let folder = guard.0.clone();
        let lp = Loop::new();
        let heard = Rc::new(RefCell::new(Vec::new()));
        let (into, quit) = (heard.clone(), lp.clone());
        let kept = live(&lp, &folder, move |files, _| {
            into.borrow_mut().extend(files);
            if into.borrow().len() >= 2 {
                quit.quit();
            }
        }).unwrap();
        let give_up = lp.clone();
        lp.after(std::time::Duration::from_secs(5), move || give_up.quit());
        std::fs::write(folder.join("a"), "x").unwrap();
        std::fs::create_dir(folder.join("sub")).unwrap();
        lp.run();
        assert!(heard.borrow().contains(&folder.join("a")));
        assert!(heard.borrow().contains(&folder.join("sub")), "a view of a folder hears of a sub-folder");
        drop(kept);
        let before = heard.borrow().len();
        std::fs::write(folder.join("b"), "x").unwrap();
        let soon = lp.clone();
        lp.after(std::time::Duration::from_millis(50), move || soon.quit());
        lp.run();
        assert_eq!(heard.borrow().len(), before, "nothing is heard once it is dropped");
    }

    #[test]
    fn a_live_whose_folder_went_first_still_lets_go_of_the_loop() {
        let guard = dir("live-gone");
        let folder = guard.0.join("watched");
        std::fs::create_dir(&folder).unwrap();
        let lp = Loop::new();
        let quit = lp.clone();
        let kept = live(&lp, &folder, move |_, _| quit.quit()).unwrap();
        let held = Rc::downgrade(&kept.watch);
        let give_up = lp.clone();
        lp.after(std::time::Duration::from_secs(5), move || give_up.quit());
        std::fs::remove_dir(&folder).unwrap();
        lp.run();
        assert!(kept.watch.borrow().folders.borrow().is_empty(), "the kernel dropped the mark");
        drop(kept);
        let soon = lp.clone();
        lp.after(std::time::Duration::from_millis(200), move || soon.quit());
        lp.run();
        assert!(held.upgrade().is_none(), "the loop still holds the watch");
    }

    #[test]
    fn a_callback_may_drop_the_live_that_called_it() {
        let guard = dir("live-drop");
        let folder = guard.0.clone();
        let lp = Loop::new();
        let slot: Rc<RefCell<Option<Live>>> = Rc::new(RefCell::new(None));
        let (mine, quit) = (slot.clone(), lp.clone());
        *slot.borrow_mut() = Some(live(&lp, &folder, move |_, _| {
            mine.borrow_mut().take();
            quit.quit();
        }).unwrap());
        let give_up = lp.clone();
        lp.after(std::time::Duration::from_secs(5), move || give_up.quit());
        std::fs::write(folder.join("a"), "x").unwrap();
        lp.run();
        assert!(slot.borrow().is_none());
    }
}
