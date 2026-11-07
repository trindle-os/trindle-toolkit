//! Tests of the file work, and what they share.

use super::*;
use super::copy::copy_one;
use super::looks::open_folder;
use super::plan::rated_by;
use crate::common::file_work::{gentle, proof, trash};
use crate::common::files::{is_changed, link_of, open_named, readable, Named};
use crate::sys::disk;
use std::ffi::{OsStr, OsString};
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::io::Read;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::mpsc;
use std::sync::Mutex;
use std::time::Duration;

mod across;
mod clash;
mod named;

fn place(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("trindle-files-work-{tag}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("a/inner")).unwrap();
    fs::create_dir_all(dir.join("b")).unwrap();
    fs::write(dir.join("a/one.txt"), b"one").unwrap();
    fs::write(dir.join("a/inner/two.txt"), vec![7u8; 3 * CHUNK + 5]).unwrap();
    dir
}

fn quiet(_: News) {}

fn asked(plan: Plan, bin: Option<trash::Bin>, answer: impl Fn(&Clash) -> (Answer, bool)) -> (Vec<Clash>, Over) {
    let (tx, rx) = mpsc::channel();
    let (seen_tx, seen) = mpsc::channel();
    let ctl = Ctl { answers: Some(rx), bin, ..Ctl::default() };
    let mut clashes = Vec::new();
    let over = std::thread::scope(|s| {
        let job = s.spawn(move || {
            let over = run(plan, &ctl, &move |n| if let News::Clash(c) = n { let _ = seen_tx.send(c); });
            drop(ctl);
            over
        });
        while let Ok(clash) = seen.recv() {
            tx.send(answer(&clash)).unwrap();
            clashes.push(clash);
        }
        job.join().unwrap()
    });
    (clashes, over)
}

fn own_bin(dir: &Path) -> trash::Bin {
    let bin = trash::Bin::at(&dir.join("Trash"), None);
    bin.ready().unwrap();
    bin
}

fn two_drives(tag: &str) -> (PathBuf, PathBuf) {
    let name = format!("trindle-files-work-{tag}-{}", std::process::id());
    let spots = [std::env::temp_dir(), PathBuf::from("/dev/shm"), PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target"), PathBuf::from("/var/tmp")];
    let drive = |p: &PathBuf| fs::metadata(p).map(|m| m.dev()).ok();
    let from = spots[0].clone();
    let to = spots.iter().find(|p| drive(p).is_some() && drive(p) != drive(&from)).expect("this test needs two drives to move between");
    let (from, to) = (from.join(&name), to.join(&name));
    for dir in [&from, &to] {
        let _ = fs::remove_dir_all(dir);
        fs::create_dir_all(dir).unwrap();
    }
    fs::create_dir_all(from.join("proj/inner")).unwrap();
    fs::create_dir_all(from.join("proj/more")).unwrap();
    fs::write(from.join("proj/one.txt"), b"one").unwrap();
    fs::write(from.join("proj/inner/two.txt"), vec![7u8; 3 * CHUNK + 5]).unwrap();
    fs::write(from.join("proj/more/three.txt"), b"three").unwrap();
    fs::write(from.join("proj/more/four.txt"), b"four").unwrap();
    (from, to)
}

fn moved(from: &Path, to: &Path, tell: &dyn Fn(News)) -> Over {
    run(Plan { kind: Kind::Move, items: vec![from.join("proj")], to: to.to_path_buf(), key: None }, &Ctl::default(), tell)
}

fn nothing_lost(from: &Path, to: &Path) {
    for (name, len) in [("proj/one.txt", 3), ("proj/inner/two.txt", 3 * CHUNK + 5), ("proj/more/three.txt", 5), ("proj/more/four.txt", 4)] {
        let has = |dir: &Path| fs::metadata(dir.join(name)).is_ok_and(|m| m.len() as usize == len);
        assert!(has(from) || has(to), "{name} is whole in neither place");
    }
}

fn tidy(dirs: [&Path; 2]) {
    for dir in dirs {
        fs::remove_dir_all(dir).unwrap();
    }
}

#[test]
fn free_names_keep_the_ending() {
    let dir = place("names");
    assert_eq!(free_name(&dir.join("a"), OsStr::new("one.txt"), false), dir.join("a/one (2).txt"));
    fs::write(dir.join("a/one (2).txt"), b"").unwrap();
    assert_eq!(free_name(&dir.join("a"), OsStr::new("one.txt"), false), dir.join("a/one (3).txt"));
    assert_eq!(free_name(&dir, OsStr::new("a"), true), dir.join("a (2)"));
    assert_eq!(free_name(&dir, OsStr::new(".hidden"), false), dir.join(".hidden (2)"));
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn progress_is_rate_limited_and_the_rest_is_not() {
    let heard = std::cell::RefCell::new(Vec::new());
    let tell = rated_by(|n| heard.borrow_mut().push(n), Duration::from_millis(200));
    tell(News::Total(10));
    for i in 0..1000 {
        tell(News::Done(i));
        tell(News::Files(i, 1000));
    }
    tell(News::Over(Over::default()));
    let count = |f: fn(&News) -> bool| heard.borrow().iter().filter(|n| f(n)).count();
    assert_eq!(count(|n| matches!(n, News::Done(_))), 1);
    assert_eq!(count(|n| matches!(n, News::Files(..))), 1);
    assert_eq!(count(|n| matches!(n, News::Total(_) | News::Over(_))), 2);
    std::thread::sleep(Duration::from_millis(220));
    tell(News::Done(5));
    assert_eq!(count(|n| matches!(n, News::Done(_))), 2);
}

#[test]
fn the_work_thread_runs_at_low_priority() {
    let asker = disk::priorities();
    let (nice, class) = gentle::spawn("low-t", disk::priorities).unwrap().join().unwrap();
    assert!(nice >= 10, "nice {nice}");
    assert_eq!(class, 3, "idle disk class");
    assert_eq!(disk::priorities(), asker, "the one that asked is left as it was");
}

#[test]
#[ignore]
fn measure_a_copy_beside_a_window() {
    let base = PathBuf::from(std::env::var("TRINDLE_MEASURE_DIR").unwrap());
    let (src, to) = (base.join("src"), base.join("to"));
    let _ = fs::remove_dir_all(&base);
    fs::create_dir_all(&src).unwrap();
    fs::create_dir_all(&to).unwrap();
    let block = vec![7u8; 1 << 20];
    let files: usize = std::env::var("TRINDLE_MEASURE_MB").ok().and_then(|v| v.parse().ok()).unwrap_or(3000);
    for i in 0..files {
        fs::write(src.join(format!("f{i:05}")), &block).unwrap();
    }
for limited in [false, true] {
    let _ = fs::remove_dir_all(&to);
    fs::create_dir_all(&to).unwrap();
    let told = std::sync::atomic::AtomicU64::new(0);
    let stop = AtomicBool::new(false);
    let late = Mutex::new(Vec::<u64>::new());
    let started = std::time::Instant::now();
    std::thread::scope(|s| {
        s.spawn(|| {
            while !stop.load(Ordering::Relaxed) {
                let t = std::time::Instant::now();
                std::thread::sleep(Duration::from_millis(5));
                late.lock().unwrap().push(t.elapsed().saturating_sub(Duration::from_millis(5)).as_micros() as u64);
            }
        });
        let told = &told;
        let count = move |_: News| {
            told.fetch_add(1, Ordering::Relaxed);
        };
        let plan = Plan { kind: Kind::Copy, items: vec![src.clone()], to: to.clone(), key: None };
        gentle::scoped(s, move || {
            if limited {
                run(plan, &Ctl::default(), &rated(count))
            } else {
                run(plan, &Ctl::default(), &count)
            }
        })
        .join()
        .unwrap();
        stop.store(true, Ordering::Relaxed);
    });
    let secs = started.elapsed().as_secs_f64();
    let mut late = late.into_inner().unwrap();
    late.sort_unstable();
    let at = |p: f64| late[((late.len() - 1) as f64 * p) as usize];
    eprintln!("{files} MB copy, rated={limited}, {secs:.1}s, reports heard {} ({:.1}/s), wake lateness us: p50 {} p99 {} max {}",
              told.load(Ordering::Relaxed), told.load(Ordering::Relaxed) as f64 / secs, at(0.5), at(0.99), at(1.0));
    }
    let _ = fs::remove_dir_all(&base);
}

#[test]
fn what_is_told_of_a_copy_never_goes_back_and_ends_at_the_whole() {
    let dir = place("told");
    fs::create_dir_all(dir.join("a/many")).unwrap();
    fs::create_dir_all(dir.join("b")).unwrap();
    for n in 0..300 {
        fs::write(dir.join(format!("a/many/f{n}")), vec![n as u8; n]).unwrap();
    }
    fs::write(dir.join("a/many/big"), vec![9u8; 20 << 20]).unwrap();
    let total: u64 = (0..300u64).sum::<u64>() + (20 << 20);
    let told = std::cell::RefCell::new(Vec::new());
    let over = run(Plan { kind: Kind::Copy, items: vec![dir.join("a/many")], to: dir.join("b"), key: None }, &Ctl::default(), &|n: News| told.borrow_mut().push(n));
    assert!(over.failed.is_empty());
    let told = told.into_inner();
    assert!(matches!(told.first(), Some(News::Total(t)) if *t == total));
    let done: Vec<u64> = told.iter().filter_map(|n| if let News::Done(d) = n { Some(*d) } else { None }).collect();
    assert!(done.windows(2).all(|w| w[0] <= w[1]) && done.last() == Some(&total), "{:?}", done.last());
    assert!(told.iter().any(|n| matches!(n, News::Files(301, 301))));
    assert_eq!(fs::read(dir.join("b/many/big")).unwrap().len(), 20 << 20);
    assert!((0..300).all(|n| fs::read(dir.join(format!("b/many/f{n}"))).unwrap() == vec![n as u8; n]));
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn the_crew_does_every_job_and_no_more_at_once_than_it_has_threads() {
    let (now, most, sum) = (AtomicUsize::new(0), AtomicUsize::new(0), AtomicU64::new(0));
    let each = |n: u64| {
        most.fetch_max(now.fetch_add(1, Ordering::SeqCst) + 1, Ordering::SeqCst);
        std::thread::sleep(Duration::from_millis(2));
        sum.fetch_add(n, Ordering::SeqCst);
        now.fetch_sub(1, Ordering::SeqCst);
    };
    crew(4, &each, |bench| {
        for n in 1..=100u64 {
            let mut job = n;
            while let Err(back) = bench.give_wait(job, 4, Duration::from_millis(50)) {
                job = back;
            }
        }
        while !bench.idle() {
            std::thread::sleep(Duration::from_millis(1));
        }
    });
    assert_eq!(sum.load(Ordering::SeqCst), 5050);
    assert!((2..=4).contains(&most.load(Ordering::SeqCst)), "{}", most.load(Ordering::SeqCst));
}

#[test]
fn spinning_disks_get_one_thread() {
    assert_eq!(threads(&[Path::new("/no/such/place/at/all")]), threads(&[Path::new("/")]));
    let tmp = std::env::temp_dir();
    assert!(matches!(threads(&[tmp.as_path()]), 1 | 4));
}

#[test]
fn copies_keep_contents_and_undo_takes_them_away() {
    let dir = place("copy");
    let ctl = Ctl::default();
    let over = run(Plan { kind: Kind::Copy, items: vec![dir.join("a")], to: dir.join("b"), key: None }, &ctl, &quiet);
    assert!(over.failed.is_empty(), "{:?}", over.failed);
    assert_eq!(fs::read(dir.join("b/a/inner/two.txt")).unwrap().len(), 3 * CHUNK + 5);
    assert_eq!(fs::read(dir.join("b/a/one.txt")).unwrap(), b"one");
    assert_eq!(over.undo, [Step::Remove(dir.join("b/a"))]);
    let same = run(Plan { kind: Kind::Copy, items: vec![dir.join("a/one.txt")], to: dir.join("a"), key: None }, &ctl, &quiet);
    assert_eq!(same.undo, [Step::Remove(dir.join("a/one (2).txt"))]);
    let inside = run(Plan { kind: Kind::Copy, items: vec![dir.join("a")], to: dir.join("a/inner"), key: None }, &ctl, &quiet);
    assert_eq!(inside.failed, ["a"]);
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn undoing_a_put_back_keeps_the_trash_note() {
    let dir = place("retrash");
    let bin = trash::Bin::at(&dir.join("Trash"), None);
    bin.ready().unwrap();
    let note = format!("[Trash Info]\nPath={}\nDeletionDate=2020-01-02T03:04:05\n", dir.join("a/gone.txt").display());
    fs::write(bin.files.join("gone.txt"), b"gone").unwrap();
    fs::write(bin.info.join("gone.txt.trashinfo"), &note).unwrap();
    let ctl = Ctl { bin: Some(bin.clone()), ..Ctl::default() };
    let back = run(Plan { kind: Kind::Restore, items: vec![bin.files.join("gone.txt")], to: PathBuf::new(), key: None }, &ctl, &quiet);
    assert!(back.failed.is_empty(), "{:?}", back.failed);
    assert!(dir.join("a/gone.txt").exists() && !bin.info.join("gone.txt.trashinfo").exists());
    let undo = run(Plan { kind: Kind::Undo(back.undo), items: Vec::new(), to: dir.clone(), key: None }, &ctl, &quiet);
    assert!(undo.failed.is_empty(), "{:?}", undo.failed);
    assert!(!dir.join("a/gone.txt").exists());
    assert_eq!(fs::read(bin.files.join("gone.txt")).unwrap(), b"gone");
    assert_eq!(fs::read_to_string(bin.info.join("gone.txt.trashinfo")).unwrap(), note);
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn undoing_answers_how_to_do_it_again() {
    let dir = place("redo");
    let bin = own_bin(&dir);
    let ctl = Ctl { bin: Some(bin.clone()), ..Ctl::default() };
    let undo = |steps: Vec<Step>| {
        let over = run(Plan { kind: Kind::Undo(steps), items: Vec::new(), to: dir.clone(), key: None }, &ctl, &quiet);
        assert!(over.failed.is_empty(), "{:?}", over.failed);
        over.undo
    };
    let moved = run(Plan { kind: Kind::Move, items: vec![dir.join("a/one.txt")], to: dir.join("b"), key: None }, &ctl, &quiet);
    let again = undo(moved.undo.clone());
    assert_eq!(again, [Step::Back { now: dir.join("a/one.txt"), was: dir.join("b/one.txt") }]);
    assert!(dir.join("a/one.txt").exists() && !dir.join("b/one.txt").exists());
    let back = undo(again);
    assert!(dir.join("b/one.txt").exists() && !dir.join("a/one.txt").exists());
    assert_eq!(back, moved.undo);
    let copied = run(Plan { kind: Kind::Copy, items: vec![dir.join("a/inner")], to: dir.join("b"), key: None }, &ctl, &quiet);
    let again = undo(copied.undo);
    assert!(!dir.join("b/inner").exists());
    assert!(matches!(again.as_slice(), [Step::Restore(_)]), "{again:?}");
    let back = undo(again);
    assert_eq!(fs::read(dir.join("b/inner/two.txt")).unwrap().len(), 3 * CHUNK + 5);
    let again = undo(back);
    assert!(!dir.join("b/inner").exists());
    undo(again);
    assert!(dir.join("b/inner/two.txt").exists());
    let trashed = run(Plan { kind: Kind::Trash, items: vec![dir.join("b/one.txt")], to: PathBuf::new(), key: None }, &ctl, &quiet);
    let again = undo(trashed.undo);
    assert!(dir.join("b/one.txt").exists());
    let back = undo(again);
    assert!(!dir.join("b/one.txt").exists());
    assert_eq!(fs::read(bin.files.join("one.txt")).unwrap(), b"one");
    assert!(matches!(back.as_slice(), [Step::Restore(_)]), "{back:?}");
    undo(back);
    assert_eq!(fs::read(dir.join("b/one.txt")).unwrap(), b"one");
    let (two, three) = (dir.join("a/inner/two.txt"), dir.join("a/inner/three.txt"));
    assert!(undo(vec![Step::Back { now: three.clone(), was: two.clone() }, Step::Remove(dir.join("no such thing"))]).is_empty());
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn deleting_for_good_takes_only_the_thing_that_was_asked_about() {
    use std::os::unix::fs::symlink;
    let dir = place("delete-asked");
    let delete = |items: &[PathBuf], asked: Vec<proof::Asked>| {
        run(Plan { kind: Kind::Delete, items: items.to_vec(), to: PathBuf::new(), key: None }, &Ctl { asked, ..Ctl::default() }, &quiet)
    };
    let not_it = |name: &str| vec![trouble(name, not_looked())];
    fs::write(dir.join("b/keep.txt"), b"keep").unwrap();
    symlink(dir.join("b"), dir.join("a/inner/link")).unwrap();
    symlink(dir.join("b/keep.txt"), dir.join("link.txt")).unwrap();
    let pipe = std::ffi::CString::new(dir.join("pipe").into_os_string().into_vec()).unwrap();
    assert_eq!(crate::sys::os::mkfifo(&pipe, 0o600), 0);
    let items = [dir.join("a"), dir.join("link.txt"), dir.join("pipe")];
    let over = delete(&items, proof::asked(&items));
    assert!(over.failed.is_empty(), "{:?}", over.failed);
    assert!(items.iter().all(|i| fs::symlink_metadata(i).is_err()), "what was asked about is still there");
    assert_eq!(fs::read(dir.join("b/keep.txt")).unwrap(), b"keep", "a link was gone through");
    fs::create_dir_all(dir.join("mine/deep")).unwrap();
    fs::write(dir.join("mine/deep/x"), b"x").unwrap();
    let mine = [dir.join("mine")];
    assert_eq!(delete(&mine, Vec::new()).failed, not_it("mine"));
    assert!(dir.join("mine/deep/x").exists(), "a thing nobody was asked about was deleted");
    let asked = proof::asked(&mine);
    fs::rename(dir.join("mine"), dir.join("moved")).unwrap();
    fs::create_dir_all(dir.join("mine/precious")).unwrap();
    fs::write(dir.join("mine/precious/thesis.txt"), b"years of work").unwrap();
    assert_eq!(delete(&mine, asked).failed, not_it("mine"));
    assert_eq!(fs::read(dir.join("mine/precious/thesis.txt")).ok().as_deref(), Some(&b"years of work"[..]), "a folder the person was never asked about was deleted");
    assert!(dir.join("moved/deep/x").exists(), "the name is all that was asked about: what has another name stays");
    fs::write(dir.join("f.txt"), b"asked about").unwrap();
    let file = [dir.join("f.txt")];
    let asked = proof::asked(&file);
    fs::rename(dir.join("f.txt"), dir.join("f.old")).unwrap();
    fs::write(dir.join("f.txt"), b"another").unwrap();
    assert_eq!(delete(&file, asked).failed, not_it("f.txt"));
    assert_eq!(fs::read(dir.join("f.txt")).unwrap(), b"another");
    fs::write(dir.join("g.txt"), b"asked about").unwrap();
    let file = [dir.join("g.txt")];
    let asked = proof::asked(&file);
    faults::set(faults::Fault::Swapped, &dir.join("g.txt"));
    assert_eq!(delete(&file, asked).failed, not_it("g.txt"));
    assert_eq!(fs::read(dir.join("g.txt")).unwrap(), faults::SWAPPED);
    fs::create_dir_all(dir.join("h/in")).unwrap();
    fs::write(dir.join("h/in/x"), b"x").unwrap();
    let folder = [dir.join("h")];
    let asked = proof::asked(&folder);
    faults::set(faults::Fault::Swapped, &dir.join("h"));
    assert_eq!(delete(&folder, asked).failed, ["h"]);
    assert_eq!(fs::read(dir.join("h/kept.txt")).unwrap(), faults::SWAPPED, "what another program put under the name was deleted");
    assert_eq!(fs::read_dir(dir.join("h.away")).unwrap().count(), 0, "the folder asked about was emptied, by what was opened");
    let mut names: Vec<String> = fs::read_dir(&dir).unwrap().flatten().map(|e| e.file_name().to_string_lossy().into_owned()).collect();
    names.sort();
    assert_eq!(names, ["b", "f.old", "f.txt", "g.txt", "h", "h.away", "mine", "moved"], "nothing is left under a name of the work's own");
    fs::remove_dir_all(&dir).unwrap();
}
