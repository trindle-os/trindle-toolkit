//! Tests that what is brought is the very thing that was named, never through a link.

use super::*;

#[test]
fn a_folder_sent_into_itself_through_a_link_is_refused() {
    let dir = place("loop");
    std::os::unix::fs::symlink(dir.join("a"), dir.join("b/way")).unwrap();
    let over = run(Plan { kind: Kind::Copy, items: vec![dir.join("a")], to: dir.join("b/way"), key: None }, &Ctl::default(), &quiet);
    assert_eq!(over.failed, ["a"], "{:?}", over.failed);
    assert!(!dir.join("a/a").exists(), "it was copied into itself");
    assert!(over.undo.is_empty());
    fs::create_dir_all(dir.join("a/inner/deep")).unwrap();
    std::os::unix::fs::symlink(dir.join("a/inner"), dir.join("b/inside")).unwrap();
    for kind in [Kind::Copy, Kind::Move] {
        let over = run(Plan { kind, items: vec![dir.join("a")], to: dir.join("b/inside/deep"), key: None }, &Ctl::default(), &quiet);
        assert_eq!(over.failed, ["a"]);
    }
    assert!(dir.join("a/one.txt").exists() && fs::read_dir(dir.join("a/inner/deep")).unwrap().next().is_none());
    assert!(goes_into(&dir.join("b/way"), &dir.join("a")) && goes_into(&dir.join("b/way/inner"), &dir.join("a")) && goes_into(&dir.join("a"), &dir.join("a")));
    assert!(!goes_into(&dir.join("b"), &dir.join("a")) && !goes_into(&dir.join("a"), &dir.join("a/inner")));
    assert!(within(&dir.join("b/way/inner"), &dir.join("a")) && within(&dir.join("a/inner"), &dir.join("a")));
    assert!(!within(&dir.join("a"), &dir.join("a/inner")) && !within(&dir.join("b"), &dir.join("a")));
    assert!(!within(&dir.join("a"), &dir.join("b/way")) && !goes_into(&dir.join("a"), &dir.join("b/way")));
    let over = run(Plan { kind: Kind::Copy, items: vec![dir.join("b/way")], to: dir.join("a"), key: None }, &Ctl::default(), &quiet);
    assert!(over.failed.is_empty(), "{:?}", over.failed);
    assert_eq!(fs::read_link(dir.join("a/way")).unwrap(), dir.join("a"));
    assert_eq!(real_of(&dir.join("b/way")), fs::canonicalize(dir.join("b")).unwrap().join("way"));
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn odd_names_links_and_a_place_that_went_are_left_as_expected() {
    let dir = place("hostile");
    let odd: Vec<std::ffi::OsString> = vec!["line\nbreak.txt".into(), "it's \"quoted\".txt".into(), "-rf".into(),
                                            std::ffi::OsString::from_vec(vec![b'b', 0xff, 0xfe, b'.', b't'])];
    fs::create_dir_all(dir.join("odd")).unwrap();
    for name in &odd {
        fs::write(dir.join("odd").join(name), b"odd").unwrap();
    }
    fs::write(dir.join("secret.txt"), b"secret").unwrap();
    std::os::unix::fs::symlink(dir.join("secret.txt"), dir.join("odd/way out")).unwrap();
    std::os::unix::fs::symlink("/no/such/place", dir.join("odd/broken")).unwrap();
    let over = run(Plan { kind: Kind::Copy, items: vec![dir.join("odd")], to: dir.join("b"), key: None }, &Ctl::default(), &quiet);
    assert!(over.failed.is_empty(), "{:?}", over.failed);
    for name in &odd {
        assert_eq!(fs::read(dir.join("b/odd").join(name)).unwrap(), b"odd");
    }
    assert!(fs::symlink_metadata(dir.join("b/odd/way out")).unwrap().file_type().is_symlink());
    assert_eq!(fs::read_link(dir.join("b/odd/broken")).unwrap(), Path::new("/no/such/place"));
    fs::create_dir_all(dir.join("c")).unwrap();
    std::os::unix::fs::symlink(dir.join("b"), dir.join("c/a")).unwrap();
    let bin = own_bin(&dir);
    let plan = Plan { kind: Kind::Copy, items: vec![dir.join("a")], to: dir.join("c"), key: None };
    let (clashes, over) = asked(plan, Some(bin), |_| (Answer::Replace, false));
    assert_eq!(clashes.iter().map(|c| c.folders).collect::<Vec<_>>(), [false]);
    assert!(over.failed.is_empty(), "{:?}", over.failed);
    assert!(dir.join("c/a").is_dir() && !fs::symlink_metadata(dir.join("c/a")).unwrap().file_type().is_symlink());
    assert!(dir.join("b/odd").is_dir(), "what the link pointed at is as it was");
    let over = run(Plan { kind: Kind::Move, items: vec![dir.join("a/one.txt")], to: dir.join("gone/away"), key: None }, &Ctl::default(), &quiet);
    assert_eq!(over.failed, ["one.txt"]);
    assert_eq!(fs::read(dir.join("a/one.txt")).unwrap(), b"one");
    let over = run(Plan { kind: Kind::Copy, items: vec![dir.join("a")], to: dir.join("gone/away"), key: None }, &Ctl::default(), &quiet);
    assert_eq!(over.failed, ["a"]);
    assert!(over.undo.is_empty() && !dir.join("gone").exists());
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn a_file_is_copied_from_what_was_opened_and_never_through_a_link() {
    let dir = place("nofollow");
    fs::write(dir.join("secret"), b"a made-up key").unwrap();
    std::os::unix::fs::symlink(dir.join("secret"), dir.join("a/way")).unwrap();
    let was = Which::of(&fs::symlink_metadata(dir.join("a/one.txt")).unwrap());
    let got = copy_one(&dir.join("a/way"), &dir.join("b/way"), was);
    assert!(got.is_err(), "a link was followed");
    assert!(fs::symlink_metadata(dir.join("b/way")).is_err(), "nothing is left behind");
    let link = Which::of(&fs::symlink_metadata(dir.join("a/way")).unwrap());
    assert!(copy_one(&dir.join("a/way"), &dir.join("b/way"), link).is_err());
    let got = copy_one(&dir.join("secret"), &dir.join("b/one.txt"), was);
    assert!(got.as_ref().is_err_and(is_changed), "{got:?}");
    assert!(fs::symlink_metadata(dir.join("b/one.txt")).is_err());
    copy_one(&dir.join("a/one.txt"), &dir.join("b/one.txt"), was).unwrap();
    assert_eq!(fs::read(dir.join("b/one.txt")).unwrap(), b"one");
    let pipe = std::ffi::CString::new(dir.join("a/pipe").as_os_str().as_bytes()).unwrap();
    assert_eq!(crate::sys::os::mkfifo(&pipe, 0o600), 0);
    let it = Which::of(&fs::symlink_metadata(dir.join("a/pipe")).unwrap());
    assert!(copy_one(&dir.join("a/pipe"), &dir.join("b/pipe"), it).is_err());
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn a_folder_swapped_for_a_link_while_it_is_copied_is_not_gone_into() {
    let dir = place("swapdir");
    fs::create_dir_all(dir.join("private")).unwrap();
    fs::write(dir.join("private/key"), b"a made-up key").unwrap();
    let was = fs::symlink_metadata(dir.join("a/inner")).unwrap();
    fs::rename(dir.join("a/inner"), dir.join("a/moved")).unwrap();
    std::os::unix::fs::symlink(dir.join("private"), dir.join("a/inner")).unwrap();
    let e = open_folder(&dir.join("a/inner"), &was).expect_err("a link was gone into");
    assert!(is_changed(&e) || e.raw_os_error() == Some(libc::ELOOP), "{e}");
    fs::remove_file(dir.join("a/inner")).unwrap();
    fs::create_dir(dir.join("a/inner")).unwrap();
    assert!(open_folder(&dir.join("a/inner"), &was).is_err_and(|e| is_changed(&e)));
    let held = open_folder(&dir.join("a/moved"), &was).unwrap();
    fs::rename(dir.join("a/moved"), dir.join("a/elsewhere")).unwrap();
    let names: Vec<OsString> = listed(&held).unwrap().things.into_iter().map(|(name, _)| name).collect();
    assert_eq!(names, ["two.txt"]);
    std::os::unix::fs::symlink(dir.join("private"), dir.join("a/elsewhere/way")).unwrap();
    let over = run(Plan { kind: Kind::Copy, items: vec![dir.join("a")], to: dir.join("b"), key: None }, &Ctl::default(), &quiet);
    assert!(over.failed.is_empty(), "{:?}", over.failed);
    assert!(fs::symlink_metadata(dir.join("b/a/elsewhere/way")).unwrap().file_type().is_symlink());
    assert_eq!(fs::read(dir.join("b/a/elsewhere/two.txt")).unwrap().len(), 3 * CHUNK + 5);
    let sure = vec![Sure { path: dir.join("a/one.txt"), which: Which::of(&fs::symlink_metadata(dir.join("private/key")).unwrap()), held: None, known: None }];
    let over = run(Plan { kind: Kind::Copy, items: vec![dir.join("a/one.txt")], to: dir.join("private"), key: None }, &Ctl { sure, ..Ctl::default() }, &quiet);
    assert_eq!(over.failed, [trouble("one.txt", changed_since())]);
    assert!(!dir.join("private/one.txt").exists());
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn a_name_in_an_opened_folder_is_opened_there_and_through_no_link() {
    let dir = place("openin");
    fs::write(dir.join("secret"), b"a made-up key").unwrap();
    std::os::unix::fs::symlink(dir.join("secret"), dir.join("a/way")).unwrap();
    std::os::unix::fs::symlink(dir.join("b"), dir.join("a/out")).unwrap();
    let (held, _) = open_same(&dir.join("a"), None).unwrap();
    let (one, meta) = open_in(&held, OsStr::new("one.txt"), None).unwrap();
    assert!(meta.is_file() && Which::of(&meta) == Which::of(&fs::symlink_metadata(dir.join("a/one.txt")).unwrap()));
    let mut read = String::new();
    readable(&one).unwrap().read_to_string(&mut read).unwrap();
    assert_eq!(read, "one");
    assert!(open_in(&held, OsStr::new("inner"), None).unwrap().1.is_dir());
    let (way, meta) = open_in(&held, OsStr::new("way"), None).unwrap();
    assert!(meta.file_type().is_symlink());
    assert_eq!(link_of(&way).unwrap(), dir.join("secret"));
    assert!(readable(&way).is_err() && matches!(open_named(Path::new("way"), Some(&held), None), Ok(Named::Link(_))), "a link is never read through");
    let other = Which::of(&fs::symlink_metadata(dir.join("secret")).unwrap());
    assert!(open_in(&held, OsStr::new("one.txt"), Some(other)).is_err_and(|e| is_changed(&e)));
    for bad in ["..", ".", "", "inner/two.txt", "../b", "/etc/hostname", "out/anything"] {
        assert!(open_in(&held, OsStr::new(bad), None).is_err(), "{bad:?} was opened");
    }
    fs::rename(dir.join("a"), dir.join("elsewhere")).unwrap();
    fs::create_dir_all(dir.join("a")).unwrap();
    fs::write(dir.join("a/one.txt"), b"another").unwrap();
    let (_, meta) = open_in(&held, OsStr::new("one.txt"), None).unwrap();
    assert_eq!(meta.len(), 3, "the name was looked up by the folder's old path");
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn a_folder_the_copy_made_itself_is_said_not_to_be_copied() {
    let dir = place("made");
    let over = run(Plan { kind: Kind::Copy, items: vec![dir.join("a"), dir.join("b/a")], to: dir.join("b"), key: None }, &Ctl::default(), &quiet);
    assert!(dir.join("b/a/one.txt").exists());
    assert_eq!(over.failed.len(), 1, "{:?}", over.failed);
    assert!(over.failed[0].starts_with('a'), "{:?}", over.failed);
    fs::remove_dir_all(&dir).unwrap();
}
