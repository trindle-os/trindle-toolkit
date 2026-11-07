//! Tests of files.

use super::*;

#[test]
fn a_place_is_its_own_only_where_every_step_to_it_is_on_this_computer() {
    for own in ["/usr/share/icons/hicolor/48x48/apps/foot.png", "/opt/app/icon.png", "/var/lib/flatpak/exports/share/icons/a.svg", "/usr/bin/env"] {
        assert!(own_place(Path::new(own)), "{own}");
    }
    let beside = beside_the_program().get(1).expect("a share folder beside the build").clone();
    let real = beside.join(format!("own-place-{}", std::process::id()));
    let _ = fs::remove_dir_all(&real);
    fs::create_dir_all(&real).unwrap();
    fs::write(real.join("a.png"), b"made up").unwrap();
    assert!(own_place(&real.join("a.png")) && own_place(&real.join("gone.png")));
    std::os::unix::fs::symlink("/proc/self", real.join("elsewhere")).unwrap();
    assert!(!own_place(&real.join("elsewhere/status")), "a link out of an own place was followed");
    assert!(!own_place(&real.join("elsewhere")));
    fs::remove_dir_all(&real).unwrap();
    for far in [PathBuf::from("/home/sam/Pictures/a.png"), PathBuf::from("/home/sam/.appimage/a.AppImage"), PathBuf::from("/home/sam/Applications"),
                PathBuf::from("/mnt/share/a.png"), PathBuf::from("/media/stick/a.png"), PathBuf::from("/run/media/sam/stick/a.png"),
                PathBuf::from("/run/user/1000/gvfs/smb-share:server=nas,share=films/a.png"), PathBuf::from("/tmp/note-1.png"),
                PathBuf::from("/dev/zero"), PathBuf::from("/usrx/a.png"), PathBuf::from("icons/a.png"), PathBuf::from("/usr/../mnt/share/a.png"),
                PathBuf::from("/")] {
        assert!(!own_place(&far), "{}", far.display());
    }
}

#[test]
fn a_look_that_may_be_on_a_share_is_never_waited_for_past_its_time() {
    use std::time::{Duration, Instant};
    static G: Gentle = Gentle::new(2);
    let me = std::thread::current().id();
    assert_eq!(G.run(true, Duration::ZERO, move || std::thread::current().id() == me), Some(true), "a look here went to a thread");
    assert_eq!(G.run(false, Duration::from_secs(10), || 7), Some(7), "a share that answers was not heard");
    let (hold, held) = std::sync::mpsc::channel::<()>();
    let began = Instant::now();
    assert_eq!(G.run(false, Duration::from_millis(100), move || held.recv().map_or(1, |()| 1)), None);
    assert!(began.elapsed() < Duration::from_secs(2), "a look that did not answer was waited for");
    let ran = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let mark = ran.clone();
    assert_eq!(G.run(false, Duration::from_secs(10), move || mark.store(true, std::sync::atomic::Ordering::SeqCst)), None);
    assert!(!ran.load(std::sync::atomic::Ordering::SeqCst), "a look was started just after one was late");
    assert_eq!(G.run(true, Duration::ZERO, || 3), Some(3), "a look here waited on the late one");
    drop(hold);
    assert!(here(Path::new("/usr/bin")) && here(Path::new("/usr/no-such-thing/at-all")));
    assert!(!here(Path::new("/proc/self/status")) && !here(Path::new("relative/name")));
}

#[test]
fn the_walk_to_an_own_place_stops_at_a_link_or_a_mount_that_leads_off_this_computer() {
    let room = private_temp("trindle-own-walk").unwrap();
    let room = { fs::create_dir_all(&room).unwrap(); fs::canonicalize(&room).unwrap() };
    let (own, share, disk) = (room.join("own"), room.join("share"), room.join("disk"));
    for dir in [own.join("icons"), own.join("mounted"), share.clone(), disk.clone()] {
        fs::create_dir_all(dir).unwrap();
    }
    for file in [own.join("icons/a.png"), own.join("mounted/a.png"), share.join("a.png"), disk.join("a.png")] {
        fs::write(file, b"made up").unwrap();
    }
    let link = |to: &str, at: &str| std::os::unix::fs::symlink(to, own.join(at)).unwrap();
    link(share.to_str().unwrap(), "to-share");
    link("../share/a.png", "up-to-share.png");
    link(disk.to_str().unwrap(), "to-disk");
    link("icons/../icons/a.png", "round-about.png");
    link("to-share", "twice");
    link("loop-b", "loop-a");
    link("loop-a", "loop-b");
    link("nowhere/at/all", "dangling");
    let roots = [own.clone()];
    let mounts = vec![(PathBuf::from("/"), true), (share.clone(), false), (own.join("mounted"), false)];
    let is = |name: &str| own_among(&own.join(name), &roots, &mounts);
    for name in ["icons/a.png", "icons/gone.png", "to-disk/a.png", "round-about.png", "dangling", "dangling/a.png"] {
        assert!(is(name), "{name}");
    }
    for name in ["to-share/a.png", "to-share", "up-to-share.png", "twice/a.png", "mounted", "mounted/a.png", "loop-a", "loop-a/a.png"] {
        assert!(!is(name), "{name}");
    }
    assert!(!own_among(&disk.join("a.png"), &roots, &mounts));
    assert!(!own_among(&own.join("icons/a.png"), &roots, &[(PathBuf::from("/"), false)]));
    assert!(!own_among(&own.join("icons/a.png"), &roots, &[]));
    let mut over = mounts.clone();
    over.push((own.join("mounted"), true));
    assert!(own_among(&own.join("mounted/a.png"), &roots, &over));
    fs::remove_dir_all(&room).unwrap();
    let list = "36 1 0:31 / / rw,noatime shared:1 - btrfs /dev/dm-0 rw\n\
                90 36 0:55 / /run/user/1000/gvfs rw,nosuid shared:500 - fuse.gvfsd-fuse gvfsd-fuse rw\n\
                91 36 0:56 / /media/sam/My\\040Stick rw - vfat /dev/sda1 rw\n\
                92 36 0:57 / /mnt/nas rw - nfs4 nas:/films rw\nnot a line\n";
    let odd = [&b"93 36 0:58 / /mnt/"[..], &[0xff, 0xfe], b" rw - nfs4 x rw\n", list.as_bytes()].concat();
    assert_eq!(mounts_from(&odd).len(), 5);
    assert_eq!(mounts_from(list.as_bytes()), [(PathBuf::from("/"), true), (PathBuf::from("/run/user/1000/gvfs"), false),
                                   (PathBuf::from("/media/sam/My Stick"), false), (PathBuf::from("/mnt/nas"), false)]);
    for _ in 0..2 {
        let mounts = mounted().expect("the kernel's list of what is mounted");
        assert!(mounts.iter().any(|(at, _)| at == Path::new("/")));
    }
}

#[test]
fn two_programs_changing_different_keys_lose_neither() {
    let dir = private_temp("trindle-test-lock").unwrap();
    let p = dir.join("env");
    let threads: Vec<_> = ["A", "B"].into_iter().map(|key| {
        let p = p.clone();
        std::thread::spawn(move || {
            for n in 0..20 {
                update_env(&p, &[(key, Some(&n.to_string()))]).unwrap();
            }
        })
    }).collect();
    for t in threads {
        t.join().unwrap();
    }
    assert_eq!(read_env(&p).len(), 2);
    assert_eq!(read_env(&p).get("A").map(String::as_str), Some("19"));
    fs::remove_dir_all(&dir).ok();
}

#[test]
fn update_env_keeps_comments_and_order() {
    let dir = private_temp("trindle-test-env").unwrap();
    let p = dir.join("env");
    fs::write(&p, "# a comment\nA=1\nB=2\n").unwrap();
    update_env(&p, &[("B", Some("3")), ("C", Some("4")), ("A", None)]).unwrap();
    assert_eq!(fs::read_to_string(&p).unwrap(), "# a comment\nB=3\nC=4\n");
    fs::remove_dir_all(&dir).ok();
}

#[test]
fn a_folders_note_of_a_change_is_out_unless_the_drive_refuses() {
    assert!(folder_out(&std::env::temp_dir()).is_ok());
    assert!(folder_out(Path::new("/nowhere/at/all")).is_ok(), "a folder that cannot be opened has nothing to wait for");
    for refused in [libc::EIO, libc::ENOSPC, libc::EDQUOT] {
        assert!(drive_refused(&std::io::Error::from_raw_os_error(refused)));
    }
    for cannot_be_asked in [libc::EINVAL, libc::ENOSYS, libc::EROFS, libc::EOPNOTSUPP, libc::EBADF] {
        assert!(!drive_refused(&std::io::Error::from_raw_os_error(cannot_be_asked)));
    }
}

#[test]
fn a_file_that_cannot_be_read_is_an_error_and_is_never_rewritten() {
    let dir = private_temp("trindle-test-honest").unwrap();
    fs::create_dir_all(&dir).unwrap();
    let fresh = dir.join("fresh.conf");
    assert_eq!(read_small(&fresh).unwrap(), "");
    update_env(&fresh, &[("A", Some("1"))]).unwrap();
    assert_eq!(read_small(&fresh).unwrap(), "A=1\n");
    let bad = dir.join("bad.conf");
    let was = b"density=roomy\nleft=~/Work\n\xff\xfe\nstart=chosen\n".to_vec();
    fs::write(&bad, &was).unwrap();
    let e = read_small(&bad).unwrap_err();
    assert_eq!(e.to_string(), t!("kit-files-not-text"));
    assert!(update_env(&bad, &[("ticks", Some("yes"))]).is_err());
    assert!(update_env(&bad, &[("left", None)]).is_err());
    assert_eq!(fs::read(&bad).unwrap(), was, "every line is still there");
    assert_eq!(fs::read_dir(&dir).unwrap().filter(|e| !e.as_ref().unwrap().file_name().to_string_lossy().ends_with(".lock")).count(), 2, "nothing was left half written");
    let folder = dir.join("folder.conf");
    fs::create_dir_all(&folder).unwrap();
    assert!(read_small(&folder).is_err() && update_env(&folder, &[("A", Some("1"))]).is_err());
    assert!(folder.is_dir());
    assert!(read_small(&fresh.join("deeper.conf")).is_err());
    let big = dir.join("big.conf");
    fs::write(&big, vec![b'a'; SMALL_MOST as usize + 1]).unwrap();
    assert_eq!(read_small(&big).unwrap_err().to_string(), t!("kit-files-too-big"));
    assert!(update_env(&big, &[("A", Some("1"))]).is_err());
    assert_eq!(fs::metadata(&big).unwrap().len(), SMALL_MOST + 1);
    fs::write(&big, vec![b'a'; SMALL_MOST as usize]).unwrap();
    assert!(read_small(&big).is_ok());
    assert_eq!(read_text(&bad), "");
    assert!(read_env(&bad).is_empty());
    fs::remove_dir_all(&dir).ok();
}

#[test]
fn a_settings_file_is_written_as_a_new_file_and_through_no_link() {
    use std::os::unix::fs::{MetadataExt, PermissionsExt};
    let dir = private_temp("trindle-test-whole").unwrap();
    fs::create_dir_all(&dir).unwrap();
    let conf = dir.join("trindle.conf");
    fs::write(&conf, "theme=dark\n").unwrap();
    fs::set_permissions(&conf, fs::Permissions::from_mode(0o600)).unwrap();
    fs::write(dir.join("precious"), "not to be written over").unwrap();
    std::os::unix::fs::symlink(dir.join("precious"), dir.join("trindle.trindle-tmp")).unwrap();
    const NEXT: u64 = 4_000_000_007;
    let planted = dir.join(spare_name(std::ffi::OsStr::new("trindle.conf"), NEXT));
    std::os::unix::fs::symlink(dir.join("precious"), &planted).unwrap();
    SPARE_NEXT.set(Some(NEXT));
    fs::hard_link(&conf, dir.join("before")).unwrap();
    update_env(&conf, &[("density", Some("roomy"))]).expect("a name left in the way does not stop a settings file being written");
    assert!(SPARE_NEXT.get().is_none(), "the spare file was made under the name the link has");
    assert_eq!(fs::read_to_string(dir.join("precious")).unwrap(), "not to be written over", "the new words were written through a link");
    assert!(fs::symlink_metadata(&planted).unwrap().file_type().is_symlink(), "the link left in the way was removed or replaced");
    fs::remove_file(&planted).unwrap();
    assert_eq!(fs::read_to_string(&conf).unwrap(), "theme=dark\ndensity=roomy\n");
    assert!(fs::symlink_metadata(&conf).unwrap().file_type().is_file(), "the settings file became a link");
    assert_eq!(fs::read_to_string(dir.join("before")).unwrap(), "theme=dark\n", "the old file was written over where it lay");
    assert_eq!(fs::metadata(&conf).unwrap().mode() & 0o777, 0o600, "it keeps who may read it");
    write_text(&dir.join("a.conf"), "one\n").unwrap();
    write_text(&dir.join("a.txt"), "two\n").unwrap();
    assert_eq!((read_text(dir.join("a.conf")).as_str(), read_text(dir.join("a.txt")).as_str()), ("one\n", "two\n"));
    let left: Vec<String> = fs::read_dir(&dir).unwrap().flatten().map(|e| e.file_name().to_string_lossy().into_owned()).filter(|n| n.contains("tmp") && n != "trindle.trindle-tmp").collect();
    assert!(left.is_empty(), "{left:?}");
    fs::create_dir_all(dir.join("elsewhere")).unwrap();
    fs::write(dir.join("elsewhere/real.conf"), "a=1\n").unwrap();
    std::os::unix::fs::symlink(dir.join("elsewhere/real.conf"), dir.join("linked.conf")).unwrap();
    write_text(&dir.join("linked.conf"), "a=2\n").unwrap();
    assert!(fs::symlink_metadata(dir.join("linked.conf")).unwrap().file_type().is_symlink());
    assert_eq!(fs::read_to_string(dir.join("elsewhere/real.conf")).unwrap(), "a=2\n");
    let count = || fs::read_dir(&dir).unwrap().count();
    let before = count();
    FULL.set(true);
    let got = update_env(&conf, &[("ticks", Some("yes"))]);
    FULL.set(false);
    assert_eq!(got.map_err(|e| e.raw_os_error()), Err(Some(libc::ENOSPC)));
    assert_eq!(fs::read_to_string(&conf).unwrap(), "theme=dark\ndensity=roomy\n");
    assert_eq!(count(), before, "a half-written file was left");
    fs::set_permissions(&dir, fs::Permissions::from_mode(0o500)).unwrap();
    let got = write_text(&conf, "lost=everything\n");
    fs::set_permissions(&dir, fs::Permissions::from_mode(0o700)).unwrap();
    assert!(got.is_err(), "{NOT_AS_ROOT}");
    assert_eq!(fs::read_to_string(&conf).unwrap(), "theme=dark\ndensity=roomy\n");
    fs::remove_dir_all(&dir).ok();
}

#[test]
fn a_rename_never_takes_a_name_from_what_has_it() {
    let dir = private_temp("trindle-test-rename-new").unwrap();
    fs::create_dir_all(dir.join("folder")).unwrap();
    fs::write(dir.join("mine"), "mine").unwrap();
    fs::write(dir.join("theirs"), "theirs").unwrap();
    std::os::unix::fs::symlink("/no/such/place", dir.join("link")).unwrap();
    for taken in ["theirs", "folder", "link"] {
        let e = rename_new(&dir.join("mine"), &dir.join(taken)).expect_err("a taken name was given away");
        assert_eq!(e.kind(), std::io::ErrorKind::AlreadyExists, "{taken}: {e}");
    }
    assert_eq!((read_text(dir.join("mine")).as_str(), read_text(dir.join("theirs")).as_str()), ("mine", "theirs"));
    assert!(dir.join("folder").is_dir() && fs::symlink_metadata(dir.join("link")).unwrap().file_type().is_symlink());
    rename_new(&dir.join("mine"), &dir.join("free")).unwrap();
    assert_eq!(read_text(dir.join("free")), "mine");
    assert!(!dir.join("mine").exists());
    assert!(rename_new(&dir.join("gone"), &dir.join("anywhere")).is_err());
    fs::remove_dir_all(&dir).ok();
}

#[test]
fn where_the_kernel_cannot_refuse_a_name_taken_meanwhile_is_still_not_written_over() {
    let dir = private_temp("trindle-test-rename-gap").unwrap();
    fs::create_dir_all(dir.join("album")).unwrap();
    fs::write(dir.join("album/one.jpg"), "one").unwrap();
    fs::write(dir.join("mine"), "mine").unwrap();
    std::os::unix::fs::symlink("/no/such/place", dir.join("link")).unwrap();
    let unrefusing = |from: &str, to: &str, then: Option<Box<dyn Fn()>>| {
        NO_REFUSE.set(true);
        MEANWHILE.set(then);
        let got = rename_new(&dir.join(from), &dir.join(to));
        NO_REFUSE.set(false);
        MEANWHILE.set(None);
        got
    };
    let theirs = dir.join("report.txt");
    let made = theirs.clone();
    let e = unrefusing("mine", "report.txt", Some(Box::new(move || fs::write(&made, "theirs").unwrap()))).expect_err("a name taken meanwhile was given away");
    assert_eq!(e.kind(), std::io::ErrorKind::AlreadyExists, "{e}");
    assert_eq!((read_text(&theirs).as_str(), read_text(dir.join("mine")).as_str()), ("theirs", "mine"), "the file made meanwhile was written over");
    for full in [true, false] {
        let made = dir.join(format!("holiday-{full}"));
        let m = made.clone();
        let got = unrefusing("album", &format!("holiday-{full}"), Some(Box::new(move || {
            fs::create_dir(&m).unwrap();
            if full {
                fs::write(m.join("theirs.jpg"), "theirs").unwrap();
            }
        })));
        assert!(got.is_err(), "a folder made meanwhile was taken for free");
        assert!(made.is_dir() && dir.join("album/one.jpg").exists() && (!full || made.join("theirs.jpg").exists()));
    }
    assert_eq!(unrefusing("mine", "album", None).map_err(|e| e.kind()), Err(std::io::ErrorKind::AlreadyExists));
    unrefusing("mine", "moved", None).unwrap();
    unrefusing("link", "moved-link", None).unwrap();
    unrefusing("album", "moved-album", None).unwrap();
    assert_eq!(read_text(dir.join("moved")), "mine");
    assert!(!dir.join("mine").exists() && !dir.join("album").exists() && fs::symlink_metadata(dir.join("link")).is_err());
    assert_eq!(fs::read_link(dir.join("moved-link")).unwrap(), Path::new("/no/such/place"));
    assert_eq!(read_text(dir.join("moved-album/one.jpg")), "one");
    fs::remove_dir_all(&dir).ok();
}

#[test]
fn where_the_kernel_cannot_refuse_what_takes_the_old_name_meanwhile_is_not_removed() {
    let dir = private_temp("trindle-test-rename-old-name").unwrap();
    fs::create_dir_all(&dir).unwrap();
    let (mine, moved) = (dir.join("mine.txt"), dir.join("moved.txt"));
    fs::write(&mine, "mine").unwrap();
    let (again, spare) = (mine.clone(), dir.join("theirs.tmp"));
    NO_REFUSE.set(true);
    LINKED.set(Some(Box::new(move || {
        fs::write(&spare, "theirs").unwrap();
        fs::rename(&spare, &again).unwrap();
    })));
    let got = rename_new(&mine, &moved);
    NO_REFUSE.set(false);
    LINKED.set(None);
    assert!(got.is_ok(), "the thing has its new name, so it moved: {got:?}");
    assert_eq!(read_text(&moved), "mine");
    assert_eq!(fs::read_to_string(&mine).ok().as_deref(), Some("theirs"), "the file put under the old name meanwhile was removed");
    let left: Vec<_> = fs::read_dir(&dir).unwrap().flatten().map(|e| e.file_name()).collect();
    assert_eq!(left.len(), 2, "nothing is left under a name of the move's own: {left:?}");
    NO_REFUSE.set(true);
    let got = rename_new(&moved, &dir.join("again.txt"));
    NO_REFUSE.set(false);
    assert!(got.is_ok() && !moved.exists() && read_text(dir.join("again.txt")) == "mine", "{got:?}");
    fs::remove_dir_all(&dir).ok();
}

#[test]
fn a_settings_file_that_is_a_pipe_is_never_waited_on() {
    let dir = private_temp("trindle-test-settings-pipe").unwrap();
    fs::create_dir_all(&dir).unwrap();
    let (pipe, link, seen) = (dir.join("trindle.conf"), dir.join("linked.conf"), dir.join("seen"));
    let c = std::ffi::CString::new(pipe.as_os_str().as_bytes()).unwrap();
    assert_eq!(crate::sys::os::mkfifo(&c, 0o600), 0);
    std::os::unix::fs::symlink(&pipe, &link).unwrap();
    let (tx, rx) = std::sync::mpsc::channel();
    let (one, two) = (pipe.clone(), link.clone());
    std::thread::spawn(move || {
        let small = [read_small(&one).map_err(|e| e.to_string()), read_small(&two).map_err(|e| e.to_string())];
        let bytes = read_bytes_within(&one, 64).is_err();
        let shown = [read_text(&one), read_text(&two)];
        let lines = unread_lines(&one, &seen, 1 << 20, 10);
        let _ = tx.send((small, bytes, shown, lines));
    });
    let got = rx.recv_timeout(std::time::Duration::from_secs(10)).expect("a settings file that is a pipe was waited on");
    let refused = Err(t!("kit-files-not-ordinary").to_string());
    assert_eq!(got, ([refused.clone(), refused], true, [String::new(), String::new()], Vec::new()));
    fs::remove_file(&pipe).unwrap();
    fs::write(&pipe, "theme=dark\n").unwrap();
    assert_eq!((read_small(&link).unwrap().as_str(), read_text(&link).as_str()), ("theme=dark\n", "theme=dark\n"));
    fs::remove_dir_all(&dir).ok();
}

#[test]
fn a_thing_that_was_named_is_opened_as_what_it_is_and_never_waited_on() {
    use std::io::Read;
    let dir = private_temp("trindle-test-named").unwrap();
    fs::create_dir_all(dir.join("album")).unwrap();
    fs::write(dir.join("album/one.jpg"), "one").unwrap();
    fs::write(dir.join("secret"), "a made-up secret").unwrap();
    std::os::unix::fs::symlink(dir.join("secret"), dir.join("album/way")).unwrap();
    std::os::unix::fs::symlink(dir.join("album"), dir.join("alias")).unwrap();
    let pipe = dir.join("album/pipe");
    let c = std::ffi::CString::new(pipe.as_os_str().as_bytes()).unwrap();
    assert_eq!(crate::sys::os::mkfifo(&c, 0o600), 0);
    let Named::File(named, meta) = open_named(&dir.join("album/one.jpg"), None, None).unwrap() else { panic!("not a file") };
    fs::rename(dir.join("album/one.jpg"), dir.join("album/moved.jpg")).unwrap();
    fs::write(dir.join("album/one.jpg"), "another").unwrap();
    let mut read = String::new();
    readable(&named).unwrap().read_to_string(&mut read).unwrap();
    assert_eq!((read.as_str(), meta.len()), ("one", 3), "what was opened is what is read, whatever has the name now");
    assert!(is_changed(&open_named(&dir.join("album/one.jpg"), None, Some(Which::of(&meta))).unwrap_err()));
    assert!(matches!(open_named(&dir.join("album/moved.jpg"), None, Some(Which::of(&meta))), Ok(Named::File(..))));
    let Named::Folder(album, _) = open_named(&dir.join("album"), None, None).unwrap() else { panic!("not a folder") };
    assert!(matches!(open_named(Path::new("moved.jpg"), Some(&album), None), Ok(Named::File(..))));
    assert!(open_named(Path::new("../secret"), Some(&album), None).is_err(), "only its last name is read, inside that folder");
    let Named::Link(way) = open_named(Path::new("way"), Some(&album), None).unwrap() else { panic!("the link was followed") };
    assert_eq!(link_of(&way).unwrap(), dir.join("secret"));
    assert!(matches!(open_named(&dir.join("alias"), None, None), Ok(Named::Link(..))));
    let (tx, rx) = std::sync::mpsc::channel();
    let at = pipe.clone();
    std::thread::spawn(move || {
        let _ = tx.send(open_named(&at, None, None).map(drop).map_err(|e| is_not_a_file(&e)));
    });
    let got = rx.recv_timeout(std::time::Duration::from_secs(2));
    let _ = fs::OpenOptions::new().write(true).custom_flags(libc::O_NONBLOCK).open(&pipe);
    assert_eq!(got, Ok(Err(true)), "a pipe is not a file, and opening it must not wait");
    assert!(open_named(Path::new("/dev/null"), None, None).is_err_and(|e| is_not_a_file(&e)), "nor is a device");
    assert_eq!(open_named(&dir.join("nothing"), None, None).unwrap_err().kind(), std::io::ErrorKind::NotFound);
    fs::remove_dir_all(&dir).ok();
}

#[test]
fn a_settings_link_that_leads_to_nothing_yet_stays_a_link() {
    let dir = private_temp("trindle-test-dangling").unwrap();
    fs::create_dir_all(dir.join("dotfiles")).unwrap();
    let is_link = |p: &Path| fs::symlink_metadata(p).is_ok_and(|m| m.file_type().is_symlink());
    let conf = dir.join("trindle.conf");
    std::os::unix::fs::symlink(dir.join("dotfiles/trindle.conf"), &conf).unwrap();
    write_text(&conf, "theme=dark\n").unwrap();
    assert!(is_link(&conf), "the link was replaced by a plain file");
    assert_eq!(fs::read_to_string(dir.join("dotfiles/trindle.conf")).unwrap(), "theme=dark\n");
    std::os::unix::fs::symlink("dotfiles/beside.conf", dir.join("beside.conf")).unwrap();
    std::os::unix::fs::symlink(dir.join("beside.conf"), dir.join("twice.conf")).unwrap();
    write_text(&dir.join("twice.conf"), "a=1\n").unwrap();
    assert!(is_link(&dir.join("twice.conf")) && is_link(&dir.join("beside.conf")));
    assert_eq!(fs::read_to_string(dir.join("dotfiles/beside.conf")).unwrap(), "a=1\n");
    let lost = dir.join("lost.conf");
    std::os::unix::fs::symlink(dir.join("unplugged/lost.conf"), &lost).unwrap();
    let e = write_text(&lost, "a=1\n").expect_err("a link to nowhere was written");
    assert_eq!(e.to_string(), t!("kit-files-dangling"));
    assert!(is_link(&lost) && !dir.join("unplugged").exists(), "the link was replaced, or its place made up");
    std::os::unix::fs::symlink(dir.join("round.conf"), dir.join("round.conf")).unwrap();
    assert!(write_text(&dir.join("round.conf"), "a=1\n").is_err() && is_link(&dir.join("round.conf")));
    fs::remove_dir_all(&dir).ok();
}

#[test]
fn the_spare_file_is_private_from_the_moment_it_is_made() {
    use std::os::unix::fs::{MetadataExt, PermissionsExt};
    let dir = private_temp("trindle-test-spare-mode").unwrap();
    fs::create_dir_all(&dir).unwrap();
    let mode = |p: &Path| fs::metadata(p).unwrap().mode() & 0o777;
    fs::write(dir.join("probe"), "").unwrap();
    let usual = mode(&dir.join("probe"));
    for (name, old) in [("private.conf", Some(0o600)), ("shared.conf", Some(0o644)), ("new.conf", None)] {
        let conf = dir.join(name);
        if let Some(old) = old {
            fs::write(&conf, "a=1\n").unwrap();
            fs::set_permissions(&conf, fs::Permissions::from_mode(old)).unwrap();
        }
        SPARE_MODE.set(None);
        write_text(&conf, "a=2\n").unwrap();
        assert_eq!(SPARE_MODE.get(), Some(0o600), "{name}: who could read the spare file as it was made");
        assert_eq!(mode(&conf), old.unwrap_or(usual), "{name}: it ends as the old file was, or as any new file is");
    }
    fs::remove_dir_all(&dir).ok();
}

#[test]
fn a_private_file_is_put_in_place_whole_and_for_this_person_alone() {
    use std::os::unix::fs::{MetadataExt, PermissionsExt};
    let dir = private_temp("trindle-test-private").unwrap();
    fs::create_dir_all(&dir).unwrap();
    let mode = |p: &Path| fs::metadata(p).unwrap().mode() & 0o777;
    for (name, old) in [("open", Some(0o644)), ("shut", Some(0o600)), ("new", None)] {
        let file = dir.join(name);
        if let Some(old) = old {
            fs::write(&file, "old\n").unwrap();
            fs::set_permissions(&file, fs::Permissions::from_mode(old)).unwrap();
        }
        SPARE_MODE.set(None);
        write_whole_private(&file, b"new\n").unwrap();
        assert_eq!((SPARE_MODE.get(), mode(&file), fs::read(&file).unwrap()), (Some(0o600), 0o600, b"new\n".to_vec()), "{name}");
        FULL.set(true);
        let failed = write_whole_private(&file, b"newer, and longer\n");
        FULL.set(false);
        assert!(failed.is_err() && fs::read(&file).unwrap() == b"new\n", "{name}: whole as it was");
    }
    assert_eq!(fs::read_dir(&dir).unwrap().count(), 3, "no spare file is left");
    fs::remove_dir_all(&dir).ok();
}

pub fn write_only(path: &Path, bytes: &[u8]) {
    use std::os::unix::fs::PermissionsExt;
    fs::write(path, bytes).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o200)).unwrap();
    assert!(fs::read(path).is_err(), "{}", NOT_AS_ROOT);
}

pub const NOT_AS_ROOT: &str = "This test needs a file its account cannot read, and no file refuses root: run as root it would check nothing. \
                               Run the tests as an ordinary account.";

#[test]
fn a_file_that_may_be_written_and_not_read_is_not_written_over() {
    let dir = private_temp("trindle-test-rewrite").unwrap();
    fs::create_dir_all(&dir).unwrap();
    let p = dir.join("kept");
    assert!(may_rewrite(&p).is_ok(), "not there yet");
    fs::write(&p, b"one\n\xff\n").unwrap();
    assert!(may_rewrite(&p).is_ok() && read_bytes_within(&p, 16).unwrap() == b"one\n\xff\n");
    assert!(read_bytes_within(&p, 3).is_err() && read_within(&p, 16).is_err());
    write_only(&p, b"A=1\nB=2\n");
    let e = may_rewrite(&p).unwrap_err();
    assert_eq!(e.to_string(), t!("kit-files-not-allowed"));
    assert!(update_env(&p, &[("C", Some("3"))]).is_err());
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(&p, fs::Permissions::from_mode(0o600)).unwrap();
    assert_eq!(fs::read_to_string(&p).unwrap(), "A=1\nB=2\n");
    fs::remove_dir_all(&dir).ok();
}

#[test]
fn a_value_that_would_not_stay_on_its_line_is_refused() {
    let dir = private_temp("trindle-test-lines").unwrap();
    fs::create_dir_all(&dir).unwrap();
    let p = dir.join("env");
    fs::write(&p, "A=1\n").unwrap();
    for bad in ["x\nB=2", "x\rB=2", "x\u{0}", "x\u{1b}[0m", "tab\there", "x\u{85}y"] {
        assert!(!fits_a_line(bad), "{bad:?}");
        assert!(update_env(&p, &[("A", Some(bad))]).is_err(), "{bad:?}");
        assert!(update_env(&p, &[("C", Some("fine")), ("A", Some(bad))]).is_err(), "one bad value and nothing is written");
    }
    for bad in ["", "B=2\nC", "a b", "1st", "A\n"] {
        assert!(update_env(&p, &[(bad, Some("x"))]).is_err(), "{bad:?}");
        assert!(update_env(&p, &[(bad, None)]).is_err(), "{bad:?}");
    }
    assert_eq!(fs::read_to_string(&p).unwrap(), "A=1\n");
    assert!(fits_a_line("~/Work and play/caf\u{e9} #1 = fine"));
    update_env(&p, &[("A", Some("~/Work and play/caf\u{e9} #1 = fine"))]).unwrap();
    assert_eq!(read_env(&p).get("A").map(String::as_str), Some("~/Work and play/caf\u{e9} #1 = fine"));
    fs::remove_dir_all(&dir).ok();
}

#[test]
fn lines_that_cannot_be_marked_as_seen_are_not_announced_again_and_again() {
    use std::os::unix::fs::PermissionsExt;
    let dir = private_temp("trindle-test-log-seen").unwrap();
    let (log, seen) = (dir.join("log"), dir.join("seen"));
    fs::write(&log, "one\ntwo\n").unwrap();
    fs::write(&seen, "0").unwrap();
    assert_eq!(unread_lines(&log, &seen, 1 << 20, 10), ["one", "two"], "marked, so announced once");
    fs::write(&seen, "0").unwrap();
    fs::set_permissions(&dir, fs::Permissions::from_mode(0o500)).unwrap();
    let again = unread_lines(&log, &seen, 1 << 20, 10);
    fs::set_permissions(&dir, fs::Permissions::from_mode(0o700)).unwrap();
    assert!(again.is_empty(), "{again:?}");
    fs::remove_dir_all(&dir).ok();
}

#[test]
fn a_log_that_cannot_be_read_is_left_alone() {
    let dir = private_temp("trindle-test-log").unwrap();
    fs::create_dir_all(&dir).unwrap();
    let (log, seen) = (dir.join("log"), dir.join("seen"));
    fs::create_dir_all(&log).unwrap();
    assert!(unread_lines(&log, &seen, 1, 1).is_empty());
    assert!(log.is_dir() && !seen.exists());
    fs::remove_dir(&log).unwrap();
    fs::write(&log, b"one\ntwo \xff\nthree\n").unwrap();
    fs::write(&seen, "1").unwrap();
    assert_eq!(unread_lines(&log, &seen, 4, 2).len(), 2);
    assert_eq!(fs::read_to_string(&log).unwrap().lines().last(), Some("three"));
    fs::remove_dir_all(&dir).ok();
}
