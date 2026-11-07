//! Tests of the gentle rule.

use super::*;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

#[derive(Clone, Default)]
struct Slow {
    handed: Arc<AtomicU64>,
    out: Arc<AtomicU64>,
    forgotten: Arc<AtomicU64>,
    worst: Arc<AtomicU64>,
}

impl Write for Slow {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let handed = self.handed.fetch_add(buf.len() as u64, Ordering::SeqCst) + buf.len() as u64;
        self.worst.fetch_max(handed - self.out.load(Ordering::SeqCst), Ordering::SeqCst);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Sink for Slow {
    fn start(&self, _: u64, _: u64) {}

    fn wait(&self, _: u64, len: u64) -> io::Result<()> {
        self.out.fetch_add(len, Ordering::SeqCst);
        Ok(())
    }

    fn forget(&self, _: u64, len: u64) {
        self.forgotten.fetch_add(len, Ordering::SeqCst);
    }
}

#[test]
fn what_is_said_done_has_reached_a_slow_drive_and_little_is_ever_waiting() {
    let drive = Slow::default();
    let said = AtomicU64::new(0);
    let reached = |n: u64| {
        said.fetch_add(n, Ordering::SeqCst);
    };
    let size = 100 * CHUNK;
    let mut from = io::repeat(7).take(size as u64);
    let mut out = Out::new(drive.clone(), &reached);
    let mut most_ahead = 0;
    let n = pour(&mut from, &mut out, &mut || {
        let (told, is_out, handed) = (said.load(Ordering::SeqCst), drive.out.load(Ordering::SeqCst), drive.handed.load(Ordering::SeqCst));
        assert!(told <= is_out, "{told} said done when {is_out} had reached the drive");
        most_ahead = most_ahead.max(handed - told);
        true
    }).unwrap();
    assert_eq!(n, size as u64);
    assert!(said.load(Ordering::SeqCst) < size as u64, "all of it was said done before the end was waited for");
    assert!(most_ahead > 0 && most_ahead <= 2 * STEP + CHUNK as u64, "{most_ahead} bytes were handed over and not yet said");
    assert!(drive.worst.load(Ordering::SeqCst) <= 2 * STEP + CHUNK as u64, "more than two steps were waiting to be written");
    out.finish_now().unwrap();
    assert_eq!(said.load(Ordering::SeqCst), size as u64);
    assert_eq!(drive.out.load(Ordering::SeqCst), size as u64);
    assert_eq!(drive.forgotten.load(Ordering::SeqCst), size as u64, "what was written out is not kept in memory");
}

#[test]
fn small_files_wait_together_and_never_more_than_the_most() {
    let drive = Slow::default();
    let said = AtomicU64::new(0);
    let reached = |n: u64| {
        said.fetch_add(n, Ordering::SeqCst);
    };
    let held = Held::default();
    let each = 1_000_000u64;
    for _ in 0..200 {
        let mut out = Out::new(drive.clone(), &reached);
        out.write_all(&vec![1u8; each as usize]).unwrap();
        out.finish(&held, ()).unwrap();
        let waiting = drive.handed.load(Ordering::SeqCst) - drive.out.load(Ordering::SeqCst);
        assert!(waiting <= HELD, "{waiting} bytes of small files were waiting");
        assert_eq!(said.load(Ordering::SeqCst), drive.out.load(Ordering::SeqCst));
    }
    assert!(said.load(Ordering::SeqCst) < 200 * each, "the last small files are not out until they are waited for");
    assert!(held.settle(&reached).is_empty());
    assert_eq!(said.load(Ordering::SeqCst), 200 * each);
    for _ in 0..(HELD_FILES * 3) {
        Out::new(drive.clone(), &reached).finish(&held, ()).unwrap();
        assert!(held.waiting.lock().unwrap().0.len() <= HELD_FILES);
    }
}

struct Picky {
    bad: bool,
    drive: Slow,
}

impl Write for Picky {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.drive.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Sink for Picky {
    fn start(&self, _: u64, _: u64) {}

    fn wait(&self, from: u64, len: u64) -> io::Result<()> {
        if self.bad { Err(io::Error::from_raw_os_error(libc::EIO)) } else { self.drive.wait(from, len) }
    }

    fn forget(&self, from: u64, len: u64) {
        self.drive.forget(from, len);
    }
}

#[test]
fn a_small_file_the_drive_refuses_is_answered_as_itself_and_never_as_another() {
    let drive = Slow::default();
    let said = AtomicU64::new(0);
    let reached = |n: u64| {
        said.fetch_add(n, Ordering::SeqCst);
    };
    let held: Held<Picky, usize> = Held::default();
    let each = 1_000_000u64;
    for n in 0..100usize {
        let mut out = Out::new(Picky { bad: n == 3 || n == 70, drive: drive.clone() }, &reached);
        out.write_all(&vec![1u8; each as usize]).unwrap();
        out.finish(&held, n).unwrap_or_else(|e| panic!("file {n} was blamed for another: {e}"));
    }
    let lost = held.settle(&reached);
    let mut whose: Vec<usize> = lost.iter().map(|l| l.whose).collect();
    whose.sort();
    assert_eq!(whose, [3, 70], "the files the drive refused, and only those");
    assert!(lost.iter().all(|l| l.why.raw_os_error() == Some(libc::EIO) && l.file.bad));
    assert_eq!(said.load(Ordering::SeqCst), 98 * each, "a file the drive refused was counted as reached");
    assert!(held.settle(&reached).is_empty(), "each is answered once");
}

struct Marked {
    at: usize,
    unwritten: std::cell::RefCell<Vec<bool>>,
}

impl Write for Marked {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut marks = self.unwritten.borrow_mut();
        if marks.len() < self.at + buf.len() {
            marks.resize(self.at + buf.len(), false);
        }
        marks[self.at..self.at + buf.len()].fill(true);
        self.at += buf.len();
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl io::Seek for Marked {
    fn seek(&mut self, to: io::SeekFrom) -> io::Result<u64> {
        self.at = match to {
            io::SeekFrom::Start(n) => n as usize,
            io::SeekFrom::End(n) => (self.unwritten.borrow().len() as i64 + n) as usize,
            io::SeekFrom::Current(n) => (self.at as i64 + n) as usize,
        };
        Ok(self.at as u64)
    }
}

impl Sink for Marked {
    fn start(&self, _: u64, _: u64) {}

    fn wait(&self, from: u64, len: u64) -> io::Result<()> {
        let mut marks = self.unwritten.borrow_mut();
        let end = ((from + len) as usize).min(marks.len());
        marks[(from as usize).min(end)..end].fill(false);
        Ok(())
    }

    fn forget(&self, _: u64, _: u64) {}
}

#[test]
fn what_an_archive_writes_again_lower_down_is_out_before_it_is_done() {
    use std::io::Seek;
    let mut tide = Tide::new(Marked { at: 0, unwritten: Default::default() });
    let chunk = vec![5u8; CHUNK];
    for _ in 0..(3 * STEP as usize / CHUNK) {
        tide.write_all(&chunk).unwrap();
    }
    for at in [30u64, STEP + 30] {
        tide.seek(io::SeekFrom::Start(at)).unwrap();
        tide.write_all(&[9u8; 26]).unwrap();
    }
    tide.seek(io::SeekFrom::End(0)).unwrap();
    tide.write_all(&[1u8; 4000]).unwrap();
    let file = tide.finish().unwrap();
    let left = file.unwritten.borrow().iter().filter(|m| **m).count();
    assert_eq!(left, 0, "{left} bytes were written and never waited for");
    assert_eq!(file.unwritten.borrow().len(), 3 * STEP as usize + 4000);
}

#[test]
fn the_share_is_never_full_until_the_work_is_over_and_tiny_files_move_it() {
    assert_eq!(share(0, 0, None, false), 0.0);
    assert_eq!(share(50, 100, None, false), 0.5);
    assert_eq!(share(100, 100, None, false), MOST_BEFORE_OVER);
    assert_eq!(share(100, 100, None, true), 1.0);
    assert_eq!(share(3, 100, None, true), 1.0);
    assert_eq!(share(0, 0, Some((500, 1000)), false), 0.5);
    let big = 100 * FILE_COUNTS_AS;
    assert!(share(big / 2, big, Some((0, 1)), false) < 0.5);
    assert!(share(big, big, Some((1, 1)), false) == MOST_BEFORE_OVER);
    assert_eq!(most_held(AT_ONCE), 4 * (1 << 20) + 4 * 16 * (1 << 20) + (32 << 20));
}

#[test]
fn a_gentle_thread_is_last_in_line_and_the_thread_that_started_it_is_not() {
    let before = disk::priorities();
    let gentle = spawn("gentle-test", disk::priorities).unwrap().join().unwrap();
    assert_eq!(gentle, (disk::LOWEST, disk::IOPRIO_CLASS_IDLE as i64));
    let scoped_one = std::thread::scope(|s| scoped(s, disk::priorities).join().unwrap());
    assert_eq!(scoped_one, (disk::LOWEST, disk::IOPRIO_CLASS_IDLE as i64));
    assert_eq!(disk::priorities(), before, "the thread that asked is as it was");
    assert!(before.0 < disk::LOWEST);
    let said = program(std::process::Command::new("sh").args(["-c", "nice"])).output().unwrap();
    assert_eq!(String::from_utf8_lossy(&said.stdout).trim(), "19");
}

fn read_through(mut from: In) -> u64 {
    let mut buf = vec![0u8; CHUNK];
    let mut n = 0u64;
    loop {
        match from.read(&mut buf).unwrap() {
            0 => return n,
            k => n += k as u64,
        }
    }
}

#[test]
fn every_step_read_gently_is_given_back_but_what_was_in_memory_before() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("target").join(format!("trindle-gentle-steps-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let a = dir.join("a");
    const STEPS: u64 = 6;
    let len = STEPS * STEP + 777;
    let data: Vec<u8> = (0..len as usize).map(|i| (i % 239) as u8).collect();
    std::fs::write(&a, &data).unwrap();
    let file = File::open(&a).unwrap();
    file.sync_all().unwrap();
    let none_kept = |file: &File| {
        file.forget(0, 0);
        disk::in_memory(file, 0, len) == Some(false)
    };
    let step_kept = |file: &File, n: u64| disk::in_memory(file, n * STEP, STEP).unwrap();
    if !none_kept(&file) {
        let _ = std::fs::remove_dir_all(&dir);
        return;
    }
    let given_back = in_a_few_tries(|| {
        assert!(none_kept(&file));
        assert_eq!(read_through(In::new(File::open(&a).unwrap())), len);
        (0..=STEPS).all(|n| !step_kept(&file, n))
    });
    assert!(given_back, "steps of the {STEPS} were left in memory after a gentle read, every time");
    let given_back = in_a_few_tries(|| {
        assert!(none_kept(&file));
        let mut from = In::new(File::open(&a).unwrap()).take(len);
        let mut buf = vec![0u8; 100_003];
        while from.read(&mut buf).unwrap() > 0 {}
        drop(from);
        disk::in_memory(&file, 0, len) == Some(false)
    });
    assert!(given_back, "what was read in odd pieces, or the last of it, was left in memory, every time");
    let other = File::open(&a).unwrap();
    let kept_as_it_was = in_a_few_tries(|| {
        assert!(none_kept(&file));
        let mut some = vec![0u8; 4096];
        std::os::unix::fs::FileExt::read_exact_at(&other, &mut some, 2 * STEP + 4096).unwrap();
        let before: Vec<u64> = (0..=STEPS).filter(|n| step_kept(&file, *n)).collect();
        assert!(before.contains(&2) && before.len() < STEPS as usize, "{before:?}");
        let from = In::new(File::open(&a).unwrap());
        assert_eq!(from.kept, before);
        assert_eq!(read_through(from), len);
        (0..=STEPS).filter(|n| step_kept(&file, *n)).collect::<Vec<u64>>() == before
    });
    assert!(kept_as_it_was, "what another program had in memory was taken, or more was left than it had, every time");
    let _ = std::fs::remove_dir_all(&dir);
}

fn in_a_few_tries(mut go: impl FnMut() -> bool) -> bool {
    (0..5).any(|_| go())
}

#[test]
#[ignore]
fn measure_what_a_copy_does_to_the_computer() {
    let room = std::path::PathBuf::from(std::env::var("TRINDLE_MEASURE").unwrap());
    let (src, probe) = (room.join("src"), room.join("probe"));
    for d in [&src, &probe] {
        let _ = std::fs::remove_dir_all(d);
        std::fs::create_dir_all(d).unwrap();
    }
    made_up_files(&src);
    all_out(&room).unwrap();
    let names: Vec<String> = std::fs::read_dir(&src).unwrap().flatten().map(|e| e.file_name().to_string_lossy().into_owned()).collect();
    let total: u64 = names.iter().map(|n| std::fs::metadata(src.join(n)).unwrap().len()).sum();
    let bench = Bench { room: room.clone(), src, probe, names, total };
    let next = std::sync::atomic::AtomicUsize::new(0);
    let old = |dest: &std::path::Path| bench.the_old_way(dest, &next);
    let new = |dest: &std::path::Path| bench.the_gentle_way(dest, &next);
    for round in 0..2 {
        println!("MEASURE round {round}, {} MB in {} files", bench.total >> 20, bench.names.len());
        bench.run("nothing copying", None);
        bench.run("the old way", Some(&old));
        bench.run("the gentle way", Some(&new));
    }
    let _ = std::fs::remove_dir_all(&room);
}

fn made_up_files(src: &std::path::Path) {
    let big: usize = std::env::var("TRINDLE_MEASURE_MB").ok().and_then(|n| n.parse().ok()).unwrap_or(80);
    let mut noise = vec![0u8; 1 << 20];
    for n in 0..4 {
        let mut f = File::create(src.join(format!("big{n}"))).unwrap();
        for _ in 0..big {
            disk::random(&mut noise);
            f.write_all(&noise).unwrap();
        }
    }
    for n in 0..2000 {
        disk::random(&mut noise[..16384]);
        std::fs::write(src.join(format!("small{n}")), &noise[..16384]).unwrap();
    }
}

fn waiting() -> u64 {
    let text = std::fs::read_to_string("/proc/meminfo").unwrap();
    let kb = |key: &str| text.lines().find(|l| l.starts_with(key)).and_then(|l| l.split_whitespace().nth(1)).and_then(|n| n.parse::<u64>().ok()).unwrap_or(0);
    kb("Dirty:") + kb("Writeback:")
}

struct Bench {
    room: std::path::PathBuf,
    src: std::path::PathBuf,
    probe: std::path::PathBuf,
    names: Vec<String>,
    total: u64,
}

impl Bench {
    fn small(&self, n: usize) -> (std::time::Duration, std::time::Duration) {
        use std::os::unix::fs::OpenOptionsExt;
        use std::time::Instant;
        let at = self.probe.join(format!("p{}", n % 50));
        let t = Instant::now();
        let mut f = File::create(&at).unwrap();
        f.write_all(&[7u8; 4096]).unwrap();
        f.sync_all().unwrap();
        let wrote = t.elapsed();
        f.forget(0, 4096);
        drop(f);
        let t = Instant::now();
        let mut f = std::fs::OpenOptions::new().read(true).custom_flags(libc::O_DIRECT).open(&at).unwrap();
        disk::read_a_page_round_the_store(&f);
        let _ = f.flush();
        (wrote, t.elapsed())
    }

    fn run(&self, label: &str, copy: Option<&(dyn Fn(&std::path::Path) + Sync)>) {
        use std::sync::atomic::AtomicBool;
        use std::time::{Duration, Instant};
        let dest = self.room.join(format!("dest-{}", label.replace(' ', "-")));
        let _ = std::fs::remove_dir_all(&dest);
        std::fs::create_dir_all(&dest).unwrap();
        for n in &self.names {
            let f = File::open(self.src.join(n)).unwrap();
            f.forget(0, 0);
        }
        all_out(&self.room).unwrap();
        let before = waiting();
        let over = AtomicBool::new(false);
        let (mut writes, mut reads, mut most) = (Vec::new(), Vec::new(), 0u64);
        let (took, after) = std::thread::scope(|s| {
            let job = s.spawn(|| {
                let t = Instant::now();
                match copy {
                    Some(copy) => copy(&dest),
                    None => std::thread::sleep(Duration::from_secs(2)),
                }
                let took = t.elapsed();
                over.store(true, std::sync::atomic::Ordering::SeqCst);
                let t = Instant::now();
                all_out(&dest).unwrap();
                (took, t.elapsed())
            });
            let mut n = 0;
            while !over.load(std::sync::atomic::Ordering::SeqCst) {
                let (w, r) = self.small(n);
                writes.push(w);
                reads.push(r);
                most = most.max(waiting());
                n += 1;
            }
            job.join().unwrap()
        });
        writes.sort();
        reads.sort();
        let ms = |d: &Duration| d.as_secs_f64() * 1000.0;
        let mid = |v: &Vec<Duration>| v.get(v.len() / 2).map_or(0.0, ms);
        let worst = |v: &Vec<Duration>| v.last().map_or(0.0, ms);
        println!("MEASURE {label}: copy {:.2}s ({:.0} MB/s), then {:.2}s more before all was on the drive; unwritten data at most {} MB (was {} MB); small write typical {:.2} ms worst {:.1} ms; small read typical {:.2} ms worst {:.1} ms; {} probes",
                 took.as_secs_f64(), if copy.is_some() { self.total as f64 / 1e6 / took.as_secs_f64() } else { 0.0 }, after.as_secs_f64(), most / 1024, before / 1024,
                 mid(&writes), worst(&writes), mid(&reads), worst(&reads), writes.len());
        let _ = std::fs::remove_dir_all(&dest);
    }

    fn the_old_way(&self, dest: &std::path::Path, next: &std::sync::atomic::AtomicUsize) {
        next.store(0, std::sync::atomic::Ordering::SeqCst);
        std::thread::scope(|s| {
            for _ in 0..AT_ONCE {
                s.spawn(|| loop {
                    let i = next.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    let Some(name) = self.names.get(i) else { break };
                    let (mut from, mut to) = (File::open(self.src.join(name)).unwrap(), File::create_new(dest.join(name)).unwrap());
                    let mut buf = vec![0u8; CHUNK];
                    loop {
                        let n = from.read(&mut buf).unwrap();
                        if n == 0 {
                            break;
                        }
                        to.write_all(&buf[..n]).unwrap();
                    }
                });
            }
        });
    }

    fn the_gentle_way(&self, dest: &std::path::Path, next: &std::sync::atomic::AtomicUsize) {
        next.store(0, std::sync::atomic::Ordering::SeqCst);
        let held = Held::default();
        std::thread::scope(|s| {
            for _ in 0..AT_ONCE {
                scoped(s, || loop {
                    let i = next.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    let Some(name) = self.names.get(i) else { break };
                    let mut out = Out::new(File::create_new(dest.join(name)).unwrap(), &|_| {});
                    pour(&mut In::new(File::open(self.src.join(name)).unwrap()), &mut out, &mut || true).unwrap();
                    out.finish(&held, ()).unwrap();
                });
            }
        });
        assert!(held.settle(&|_| {}).is_empty());
        all_out(dest).unwrap();
    }
}

#[test]
fn a_real_file_copied_gently_is_whole_and_out() {
    let dir = std::env::temp_dir().join(format!("trindle-gentle-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let (a, b) = (dir.join("a"), dir.join("b"));
    let data: Vec<u8> = (0..(3 * STEP as usize + 12345)).map(|i| (i % 251) as u8).collect();
    std::fs::write(&a, &data).unwrap();
    let n = copy_file_whole(File::open(&a).unwrap(), File::create_new(&b).unwrap()).unwrap();
    assert_eq!(n, data.len() as u64);
    assert_eq!(std::fs::read(&b).unwrap(), data);
    all_out(&dir).unwrap();
    if let Some(kept) = disk::in_memory(&File::open(&b).unwrap(), 0, STEP) {
        assert!(!kept || std::fs::read(&b).is_ok());
    }
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_copy_by_name_has_the_firsts_mode_and_the_rest_of_a_file_is_read_gently() {
    use std::io::Seek;
    use std::os::unix::fs::PermissionsExt;
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("target").join(format!("trindle-gentle-rest-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let (a, b, c) = (dir.join("a"), dir.join("b"), dir.join("c"));
    let data: Vec<u8> = (0..(3 * STEP as usize + 777)).map(|i| (i % 241) as u8).collect();
    std::fs::write(&a, &data).unwrap();
    std::fs::set_permissions(&a, std::fs::Permissions::from_mode(0o754)).unwrap();
    std::fs::write(&b, b"older").unwrap();
    std::fs::set_permissions(&b, std::fs::Permissions::from_mode(0o600)).unwrap();
    assert_eq!(copy_named(&a, &b).unwrap(), data.len() as u64);
    assert_eq!(std::fs::read(&b).unwrap(), data);
    std::fs::hard_link(&a, dir.join("hard")).unwrap();
    std::os::unix::fs::symlink(&a, dir.join("soft")).unwrap();
    for same in [a.clone(), dir.join("hard"), dir.join("soft")] {
        assert!(copy_named(&a, &same).is_err(), "{} is the same file", same.display());
        assert_eq!(std::fs::read(&a).unwrap(), data);
    }
    assert_eq!(std::fs::metadata(&b).unwrap().permissions().mode() & 0o7777, 0o754, "a copy over a file that was there kept that file's mode");
    let mut from = File::open(&a).unwrap();
    from.sync_all().unwrap();
    let given_back = in_a_few_tries(|| {
        from.forget(0, 0);
        let told = disk::in_memory(&from, 0, data.len() as u64) == Some(false);
        from.seek(io::SeekFrom::Start(8192)).unwrap();
        let _ = std::fs::remove_file(&c);
        copy_into(&mut from, File::create_new(&c).unwrap()).unwrap();
        let left = told && disk::in_memory(&from, 8192, data.len() as u64 - 8192) != Some(false);
        assert_eq!(std::fs::read(&c).unwrap(), data[8192..]);
        assert_eq!(from.stream_position().unwrap(), data.len() as u64, "the file is left at its end");
        !left
    });
    assert!(given_back, "what was read was left in memory, every time");
    let _ = std::fs::remove_dir_all(&dir);
}
