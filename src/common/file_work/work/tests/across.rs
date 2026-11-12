//! Tests of a move across drives: nothing is taken away that was not proved copied.

use super::*;

#[test]
fn a_move_to_another_drive_takes_everything_and_leaves_nothing() {
    let (from, to) = two_drives("across");
    let over = moved(&from, &to, &quiet);
    assert!(over.failed.is_empty(), "{:?}", over.failed);
    assert!(!from.join("proj").exists(), "the first is still there");
    nothing_lost(&from, &to);
    assert_eq!(fs::read(to.join("proj/inner/two.txt")).unwrap(), vec![7u8; 3 * CHUNK + 5]);
    let back = run(Plan { kind: Kind::Undo(over.undo), items: Vec::new(), to: from.clone(), key: None }, &Ctl::default(), &quiet);
    assert!(back.failed.is_empty(), "{:?}", back.failed);
    assert!(from.join("proj/more/four.txt").exists() && !to.join("proj").exists());
    tidy([&from, &to]);
}

#[test]
fn a_folder_that_cannot_be_listed_is_never_removed_by_a_move() {
    let (from, to) = two_drives("nolist");
    faults::set(faults::Fault::List, &from.join("proj/more"));
    let over = moved(&from, &to, &quiet);
    assert_eq!(fs::read(from.join("proj/more/three.txt")).ok().as_deref(), Some(&b"three"[..]), "a file that was never copied was removed");
    assert_eq!(fs::read(from.join("proj/more/four.txt")).unwrap(), b"four");
    nothing_lost(&from, &to);
    assert!(from.join("proj/one.txt").exists() && !to.join("proj").exists(), "part of a move was left");
    assert!(!over.failed.is_empty(), "{:?}", over.failed);
    let (from2, to2) = two_drives("nolist-real");
    fs::set_permissions(from2.join("proj/more"), fs::Permissions::from_mode(0o000)).unwrap();
    if fs::read_dir(from2.join("proj/more")).is_ok() {
        fs::set_permissions(from2.join("proj/more"), fs::Permissions::from_mode(0o700)).unwrap();
        tidy([&from, &to]);
        tidy([&from2, &to2]);
        panic!("{}", crate::common::files::tests::NOT_AS_ROOT);
    }
    let over = moved(&from2, &to2, &quiet);
    for dir in [&from2, &to2] {
        let _ = fs::set_permissions(dir.join("proj/more"), fs::Permissions::from_mode(0o700));
    }
    assert!(!over.failed.is_empty() && !to2.join("proj").exists(), "{:?}", over.failed);
    nothing_lost(&from2, &to2);
    tidy([&from, &to]);
    tidy([&from2, &to2]);
}

#[test]
fn a_list_cut_short_by_an_error_leaves_the_folder_where_it_was() {
    let (from, to) = two_drives("cut");
    faults::set(faults::Fault::ListCut, &from.join("proj/more"));
    let over = moved(&from, &to, &quiet);
    nothing_lost(&from, &to);
    assert!(from.join("proj/more/three.txt").exists() && from.join("proj/more/four.txt").exists(), "nothing of a folder not read whole is taken away");
    assert!(!over.failed.is_empty() && !to.join("proj").exists(), "{:?}", over.failed);
    tidy([&from, &to]);
}

#[test]
fn a_file_that_cannot_be_read_to_its_end_stays_where_it_was() {
    let (from, to) = two_drives("readerr");
    faults::set(faults::Fault::Read, &from.join("proj/inner/two.txt"));
    let over = moved(&from, &to, &quiet);
    assert_eq!(fs::metadata(from.join("proj/inner/two.txt")).unwrap().len() as usize, 3 * CHUNK + 5);
    assert!(!to.join("proj").exists(), "part of a move was left");
    nothing_lost(&from, &to);
    assert!(from.join("proj/one.txt").exists(), "the rest was taken though the whole did not come");
    assert!(over.failed.iter().any(|f| f == "two.txt"), "{:?}", over.failed);
    tidy([&from, &to]);
}

#[test]
fn what_is_put_in_a_folder_while_it_is_moved_is_not_removed() {
    let (from, to) = two_drives("late");
    let late = from.join("proj/more/late.txt");
    let over = moved(&from, &to, &|news| {
        if let News::Total(_) = news {
            fs::write(&late, b"written meanwhile").unwrap();
        }
    });
    assert_eq!(fs::read(&late).ok().as_deref(), Some(&b"written meanwhile"[..]), "a file that was never copied was removed");
    nothing_lost(&from, &to);
    assert!(!from.join("proj/more/three.txt").exists() && !from.join("proj/inner").exists(), "what was copied was not moved");
    assert!(over.failed.iter().any(|f| f.starts_with("proj (couldn't be moved: 1 item")), "{:?}", over.failed);
    tidy([&from, &to]);
}

#[test]
fn a_file_changed_after_it_was_copied_is_not_removed() {
    let (from, to) = two_drives("changed");
    let one = from.join("proj/one.txt");
    let once = AtomicBool::new(false);
    let over = moved(&from, &to, &|news| {
        if matches!(news, News::Files(done, all) if done == all) && !once.swap(true, Ordering::Relaxed) {
            fs::write(&one, b"one, and more since").unwrap();
        }
    });
    assert_eq!(fs::read(&one).ok().as_deref(), Some(&b"one, and more since"[..]), "what was written since the copy was lost");
    assert!(over.failed.iter().any(|f| f.starts_with("proj (couldn't be moved: 1 item")), "{:?}", over.failed);
    assert!(!from.join("proj/more").exists());
    tidy([&from, &to]);
}

#[test]
fn what_takes_the_name_in_the_moment_before_it_is_taken_away_is_not_removed() {
    let (from, to) = two_drives("swapped");
    let one = from.join("proj/one.txt");
    faults::set(faults::Fault::Swapped, &one);
    let over = moved(&from, &to, &quiet);
    assert_eq!(fs::read(&one).ok().as_deref(), Some(faults::SWAPPED), "a file that was never copied was removed");
    assert_eq!(fs::read(to.join("proj/one.txt")).unwrap(), b"one", "what was copied is what was there before");
    assert!(over.failed.iter().any(|f| f.starts_with("proj (couldn't be moved: 1 item")), "{:?}", over.failed);
    let left: Vec<_> = fs::read_dir(from.join("proj")).unwrap().flatten().map(|e| e.file_name()).collect();
    assert_eq!(left, ["one.txt"], "{left:?}");
    nothing_lost(&to, &to);
    tidy([&from, &to]);
}

#[test]
fn a_small_file_the_drive_refuses_is_a_failed_copy_and_no_other_is() {
    let dir = place("refused");
    for n in 0..40 {
        fs::write(dir.join(format!("a/small{n}.txt")), vec![n as u8; 2_000_000]).unwrap();
    }
    gentle::faults::set(&dir.join("b/a/one.txt"));
    gentle::faults::set(&dir.join("b/a/small7.txt"));
    let (bytes, across) = (AtomicU64::new(0), AtomicU64::new(0));
    let tell = |n: News| match n {
        News::Done(n) => bytes.store(n, Ordering::SeqCst),
        News::Files(n, _) => across.store(n, Ordering::SeqCst),
        _ => {}
    };
    let over = run(Plan { kind: Kind::Copy, items: vec![dir.join("a")], to: dir.join("b"), key: None }, &Ctl::default(), &tell);
    let mut failed = over.failed.clone();
    failed.sort();
    assert_eq!(failed, ["one.txt", "small7.txt"], "the files the drive refused, each by its own name, and no other");
    assert!(!dir.join("b/a").exists(), "a folder with a copy the drive refused was left looking whole");
    assert_eq!(fs::read(dir.join("a/one.txt")).unwrap(), b"one", "the first is as it was");
    assert_eq!(across.load(Ordering::SeqCst), 40, "a file the drive refused was counted as across");
    assert_eq!(bytes.load(Ordering::SeqCst), 39 * 2_000_000 + 3 * CHUNK as u64 + 5, "bytes the drive refused were counted as done");
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn a_drive_that_will_not_keep_what_was_written_is_not_a_finished_copy() {
    let dir = place("not-kept");
    gentle::faults::set(&dir.join("b"));
    let over = run(Plan { kind: Kind::Copy, items: vec![dir.join("a")], to: dir.join("b"), key: None }, &Ctl::default(), &quiet);
    assert_eq!(over.not_kept, ["b"]);
    assert!(over.failed.is_empty(), "{:?}", over.failed);
    assert!(over.troubled(), "work the drive would not keep was taken for work done");
    assert_eq!(over.troubles(), [trouble("b", not_kept())]);
    assert!(!not_kept().contains('\u{2014}') && not_kept().chars().next().is_some_and(char::is_lowercase));
    assert_eq!(fs::read(dir.join("b/a/one.txt")).unwrap(), b"one");
    let other = place("kept");
    let over = run(Plan { kind: Kind::Copy, items: vec![other.join("a")], to: other.join("b"), key: None }, &Ctl::default(), &quiet);
    assert!(over.failed.is_empty(), "{:?}", over.failed);
    fs::remove_dir_all(&dir).unwrap();
    fs::remove_dir_all(&other).unwrap();
}

#[test]
fn a_full_drive_leaves_the_first_where_it_was() {
    let (from, to) = two_drives("full");
    faults::set(faults::Fault::Full, &to.join("proj/inner/two.txt"));
    let over = moved(&from, &to, &quiet);
    assert_eq!(fs::metadata(from.join("proj/inner/two.txt")).unwrap().len() as usize, 3 * CHUNK + 5);
    assert!(!to.join("proj/inner/two.txt").exists(), "half a copy was left");
    nothing_lost(&from, &to);
    assert!(from.join("proj/one.txt").exists() && !to.join("proj").exists(), "part of a move was left");
    assert!(!over.failed.is_empty(), "{:?}", over.failed);
    tidy([&from, &to]);
}
