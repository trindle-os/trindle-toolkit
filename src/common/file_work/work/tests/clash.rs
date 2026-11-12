//! Tests of a name already taken: asked about, merged, kept beside or replaced only once the new one is whole.

use super::*;

#[test]
fn a_folder_meeting_a_folder_asks_first_and_merge_asks_about_the_files_inside() {
    let dir = place("move");
    fs::create_dir_all(dir.join("b/a")).unwrap();
    fs::write(dir.join("b/a/one.txt"), b"older").unwrap();
    let plan = Plan { kind: Kind::Move, items: vec![dir.join("a")], to: dir.join("b"), key: None };
    let (clashes, over) = asked(plan, None, |c| (if c.folders { Answer::Merge } else { Answer::Both }, false));
    assert_eq!(clashes, [
        Clash { name: "a".into(), folders: true, sides: Some((dir.join("a"), dir.join("b/a"))), replace: true },
        Clash { name: "one.txt".into(), folders: false, sides: Some((dir.join("a/one.txt"), dir.join("b/a/one.txt"))), replace: true },
    ]);
    assert!(over.failed.is_empty(), "{:?}", over.failed);
    assert_eq!(fs::read(dir.join("b/a/one.txt")).unwrap(), b"older");
    assert_eq!(fs::read(dir.join("b/a/one (2).txt")).unwrap(), b"one");
    assert!(dir.join("b/a/inner/two.txt").exists());
    assert!(!dir.join("a").exists());
    let undo = Plan { kind: Kind::Undo(over.undo.clone()), items: Vec::new(), to: dir.clone(), key: None };
    let back = run(undo, &Ctl::default(), &quiet);
    assert!(back.failed.is_empty(), "{:?}", back.failed);
    assert_eq!(fs::read(dir.join("a/one.txt")).unwrap(), b"one");
    assert!(dir.join("a/inner/two.txt").exists());
    assert_eq!(fs::read(dir.join("b/a/one.txt")).unwrap(), b"older");
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn a_copied_folder_merges_too_and_the_first_stays_whole() {
    let dir = place("copy-merge");
    fs::create_dir_all(dir.join("b/a")).unwrap();
    fs::write(dir.join("b/a/one.txt"), b"older").unwrap();
    fs::write(dir.join("b/a/kept.txt"), b"kept").unwrap();
    let bin = own_bin(&dir);
    let plan = Plan { kind: Kind::Copy, items: vec![dir.join("a")], to: dir.join("b"), key: None };
    let (clashes, over) = asked(plan, Some(bin.clone()), |c| (if c.folders { Answer::Merge } else { Answer::Replace }, false));
    assert_eq!(clashes.iter().map(|c| (c.name.as_str(), c.folders)).collect::<Vec<_>>(), [("a", true), ("one.txt", false)]);
    assert!(over.failed.is_empty(), "{:?}", over.failed);
    assert_eq!(fs::read(dir.join("b/a/one.txt")).unwrap(), b"one");
    assert_eq!(fs::read(dir.join("b/a/kept.txt")).unwrap(), b"kept");
    assert!(dir.join("b/a/inner/two.txt").exists());
    assert_eq!(fs::read(dir.join("a/one.txt")).unwrap(), b"one", "a copy leaves the first as it was");
    assert_eq!(fs::read(bin.files.join("one.txt")).unwrap(), b"older");
    let undo = Plan { kind: Kind::Undo(over.undo), items: Vec::new(), to: dir.clone(), key: None };
    let back = run(undo, &Ctl { bin: Some(bin), ..Ctl::default() }, &quiet);
    assert!(back.failed.is_empty(), "{:?}", back.failed);
    assert_eq!(fs::read(dir.join("b/a/one.txt")).unwrap(), b"older");
    assert!(!dir.join("b/a/inner").exists());
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn what_is_replaced_stays_where_it_is_when_the_new_one_does_not_come_whole() {
    for folder in [false, true] {
        let dir = place(if folder { "replace-fails-folder" } else { "replace-fails-file" });
        let bin = own_bin(&dir);
        let (item, old) = if folder {
            fs::create_dir_all(dir.join("b/a/old things")).unwrap();
            fs::write(dir.join("b/a/old things/diary.txt"), b"dear diary").unwrap();
            (dir.join("a"), dir.join("b/a/old things/diary.txt"))
        } else {
            fs::write(dir.join("b/two.txt"), b"dear diary").unwrap();
            (dir.join("a/inner/two.txt"), dir.join("b/two.txt"))
        };
        faults::set(faults::Fault::Read, &dir.join("a/inner/two.txt"));
        let ctl = Ctl { fixed: Some(Answer::Replace), bin: Some(bin.clone()), ..Ctl::default() };
        let over = run(Plan { kind: Kind::Copy, items: vec![item], to: dir.join("b"), key: None }, &ctl, &quiet);
        assert_eq!(over.failed.len(), 1, "{:?}", over.failed);
        assert_eq!(fs::read(&old).ok().as_deref(), Some(&b"dear diary"[..]), "the old one made way for a copy that never came whole");
        assert_eq!(fs::read_dir(dir.join("b")).unwrap().count(), 1, "nothing half made is left beside it");
        assert_eq!(fs::read_dir(&bin.files).unwrap().count(), 0, "nothing went to the Trash");
        let back = run(Plan { kind: Kind::Undo(over.undo), items: Vec::new(), to: dir.clone(), key: None }, &ctl, &quiet);
        assert!(back.failed.is_empty(), "{:?}", back.failed);
        assert_eq!(fs::read(&old).ok().as_deref(), Some(&b"dear diary"[..]), "Undo took the old one away");
        fs::remove_dir_all(&dir).unwrap();
    }
}

#[test]
fn a_copy_that_does_not_come_whole_leaves_nothing_behind() {
    for folder in [true, false] {
        let dir = place("nothing-behind");
        let item = if folder { dir.join("a") } else { dir.join("a/inner/two.txt") };
        faults::set(faults::Fault::Read, &dir.join("a/inner/two.txt"));
        let over = run(Plan { kind: Kind::Copy, items: vec![item.clone()], to: dir.join("b"), key: None }, &Ctl::default(), &quiet);
        assert!(!over.failed.is_empty());
        assert_eq!(fs::read_dir(dir.join("b")).unwrap().count(), 0, "{:?}", fs::read_dir(dir.join("b")).unwrap().collect::<Vec<_>>());
        assert_eq!(fs::read(dir.join("a/inner/two.txt")).unwrap().len(), 3 * CHUNK + 5);
        assert!(item.exists());
        fs::remove_dir_all(&dir).unwrap();
    }
    let dir = place("whole-no-spare");
    let over = run(Plan { kind: Kind::Copy, items: vec![dir.join("a")], to: dir.join("b"), key: None }, &Ctl::default(), &quiet);
    assert!(over.failed.is_empty(), "{:?}", over.failed);
    let names: Vec<String> = fs::read_dir(dir.join("b")).unwrap().map(|e| e.unwrap().file_name().to_string_lossy().into_owned()).collect();
    assert_eq!(names, ["a"]);
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn replacing_a_folder_puts_the_old_one_in_the_trash_and_undo_brings_it_back() {
    for kind in [Kind::Copy, Kind::Move] {
        let dir = place("replace-folder");
        fs::create_dir_all(dir.join("b/a/old things")).unwrap();
        fs::write(dir.join("b/a/old things/diary.txt"), b"dear diary").unwrap();
        let bin = own_bin(&dir);
        let moving = matches!(kind, Kind::Move);
        let plan = Plan { kind, items: vec![dir.join("a")], to: dir.join("b"), key: None };
        let (clashes, over) = asked(plan, Some(bin.clone()), |_| (Answer::Replace, false));
        assert_eq!(clashes.len(), 1, "one question, for the folder");
        assert!(over.failed.is_empty(), "{:?}", over.failed);
        assert_eq!(fs::read(dir.join("b/a/one.txt")).unwrap(), b"one");
        assert!(dir.join("b/a/inner/two.txt").exists());
        assert!(!dir.join("b/a/old things").exists());
        assert_eq!(dir.join("a").exists(), !moving);
        assert_eq!(fs::read(bin.files.join("a/old things/diary.txt")).unwrap(), b"dear diary");
        let undo = Plan { kind: Kind::Undo(over.undo), items: Vec::new(), to: dir.clone(), key: None };
        let back = run(undo, &Ctl { bin: Some(bin.clone()), ..Ctl::default() }, &quiet);
        assert!(back.failed.is_empty(), "{:?}", back.failed);
        assert_eq!(fs::read(dir.join("b/a/old things/diary.txt")).unwrap(), b"dear diary");
        assert!(!dir.join("b/a/one.txt").exists());
        assert_eq!(fs::read(dir.join("a/one.txt")).unwrap(), b"one");
        fs::remove_dir_all(&dir).unwrap();
    }
}

#[test]
fn a_folder_that_cannot_go_to_the_trash_is_never_replaced() {
    let dir = place("replace-no-trash");
    fs::create_dir_all(dir.join("b/a")).unwrap();
    fs::write(dir.join("b/a/diary.txt"), b"dear diary").unwrap();
    fs::write(dir.join("Trash"), b"not a folder").unwrap();
    let bin = trash::Bin::at(&dir.join("Trash"), None);
    for kind in [Kind::Copy, Kind::Move] {
        let plan = Plan { kind, items: vec![dir.join("a")], to: dir.join("b"), key: None };
        let (_, over) = asked(plan, Some(bin.clone()), |_| (Answer::Replace, false));
        assert_eq!(over.failed, ["a (the item already there couldn't be moved to the Trash)"]);
        assert!(over.undo.is_empty());
        assert_eq!(fs::read(dir.join("b/a/diary.txt")).unwrap(), b"dear diary", "the old folder is untouched");
        assert!(!dir.join("b/a/one.txt").exists());
        assert_eq!(fs::read(dir.join("a/one.txt")).unwrap(), b"one", "and so is the one being brought");
    }
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn a_folder_can_be_skipped_or_kept_beside_the_other() {
    let dir = place("folder-skip-both");
    fs::create_dir_all(dir.join("b/a")).unwrap();
    fs::write(dir.join("b/a/diary.txt"), b"dear diary").unwrap();
    let plan = || Plan { kind: Kind::Copy, items: vec![dir.join("a")], to: dir.join("b"), key: None };
    let (_, over) = asked(plan(), None, |_| (Answer::Skip, false));
    assert_eq!((over.skipped, over.undo.len(), over.failed.len()), (1, 0, 0));
    assert_eq!(fs::read_dir(dir.join("b/a")).unwrap().count(), 1);
    let (clashes, over) = asked(plan(), None, |_| (Answer::Both, false));
    assert_eq!(clashes.len(), 1);
    assert_eq!(over.undo, [Step::Remove(dir.join("b/a (2)"))]);
    assert_eq!(fs::read(dir.join("b/a (2)/one.txt")).unwrap(), b"one");
    assert_eq!(fs::read_dir(dir.join("b/a")).unwrap().count(), 1);
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn the_same_for_the_rest_covers_folders_apart_from_files() {
    let dir = place("same");
    for name in ["first", "second"] {
        fs::create_dir_all(dir.join("from").join(name)).unwrap();
        fs::create_dir_all(dir.join("b").join(name)).unwrap();
        fs::write(dir.join("from").join(name).join("in.txt"), b"new").unwrap();
        fs::write(dir.join("from").join(format!("{name}.txt")), b"new").unwrap();
        fs::write(dir.join("b").join(format!("{name}.txt")), b"old").unwrap();
    }
    let mut items: Vec<PathBuf> = fs::read_dir(dir.join("from")).unwrap().flatten().map(|e| e.path()).collect();
    items.sort();
    let plan = Plan { kind: Kind::Copy, items, to: dir.join("b"), key: None };
    let (clashes, over) = asked(plan, None, |c| (if c.folders { Answer::Merge } else { Answer::Skip }, true));
    assert_eq!(clashes.iter().map(|c| (c.name.as_str(), c.folders)).collect::<Vec<_>>(), [("first", true), ("first.txt", false)]);
    assert!(over.failed.is_empty(), "{:?}", over.failed);
    assert_eq!(over.skipped, 2);
    assert!(dir.join("b/first/in.txt").exists() && dir.join("b/second/in.txt").exists());
    assert_eq!(fs::read(dir.join("b/second.txt")).unwrap(), b"old");
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn nothing_is_replaced_by_what_is_inside_it() {
    let dir = place("inside");
    fs::create_dir_all(dir.join("a/a")).unwrap();
    fs::write(dir.join("a/a/deep.txt"), b"deep").unwrap();
    let bin = own_bin(&dir);
    let plan = Plan { kind: Kind::Move, items: vec![dir.join("a/a")], to: dir.clone(), key: None };
    let (_, over) = asked(plan, Some(bin.clone()), |_| (Answer::Replace, false));
    assert_eq!(over.failed, ["a (it's inside the folder it would replace)"]);
    assert_eq!(fs::read(dir.join("a/a/deep.txt")).unwrap(), b"deep");
    assert_eq!(fs::read(dir.join("a/one.txt")).unwrap(), b"one");
    assert_eq!(fs::read_dir(&bin.files).unwrap().count(), 0);
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn skip_and_a_stopped_copy_leave_things_as_they_were() {
    let dir = place("skip");
    fs::write(dir.join("b/one.txt"), b"keep").unwrap();
    let ctl = Ctl { fixed: Some(Answer::Skip), ..Ctl::default() };
    let over = run(Plan { kind: Kind::Copy, items: vec![dir.join("a/one.txt")], to: dir.join("b"), key: None }, &ctl, &quiet);
    assert_eq!((over.skipped, over.undo.len()), (1, 0));
    assert_eq!(fs::read(dir.join("b/one.txt")).unwrap(), b"keep");
    let stop = Ctl::default();
    stop.stop.store(true, Ordering::Relaxed);
    let over = run(Plan { kind: Kind::Copy, items: vec![dir.join("a/inner/two.txt")], to: dir.join("b"), key: None }, &stop, &quiet);
    assert!(over.stopped);
    assert!(!dir.join("b/two.txt").exists());
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn what_took_the_name_while_the_question_was_up_is_not_put_in_the_trash() {
    let dir = place("replace-swap");
    fs::write(dir.join("b/one.txt"), b"older").unwrap();
    let bin = own_bin(&dir);
    let there = dir.join("b/one.txt");
    let plan = Plan { kind: Kind::Copy, items: vec![dir.join("a/one.txt")], to: dir.join("b"), key: None };
    let (clashes, over) = asked(plan, Some(bin.clone()), |_| {
        fs::remove_file(&there).unwrap();
        fs::write(&there, b"newer, from another program").unwrap();
        (Answer::Replace, false)
    });
    assert_eq!(clashes.len(), 1);
    assert_eq!(fs::read(&there).unwrap(), b"newer, from another program", "what was never asked about was replaced");
    assert!(!bin.files.join("one.txt").exists(), "what was never asked about went to the Trash");
    assert_eq!(over.failed, ["one.txt (another item now has that name)"]);
    assert!(over.undo.is_empty());
    fs::remove_dir_all(&dir).unwrap();
}
