//! Tests of dnd.

use super::*;

#[test]
fn a_drag_begins_from_whichever_of_a_button_and_a_finger_went_down_last() {
    assert!(newer(10, 9) && !newer(9, 10) && !newer(5, 5));
    assert!(newer(2, u32::MAX - 1) && !newer(u32::MAX - 1, 2));
}

fn portal_answers(a: crate::bus::Asked, h: &std::cell::RefCell<Vec<String>>) {
    use std::collections::HashMap;
    use zbus::zvariant::OwnedValue;
    match a.member().as_str() {
        "StartTransfer" => {
            let (how,): (HashMap<String, OwnedValue>,) = a.message().body().deserialize().expect("how");
            let writable = how.get("writable").and_then(|v| bool::try_from(v).ok());
            h.borrow_mut().push(format!("start, writable {writable:?}"));
            a.reply(&"made-up-key");
        }
        "AddFiles" => {
            let (_, fds, _): (String, Vec<zbus::zvariant::OwnedFd>, HashMap<String, OwnedValue>) = a.message().body().deserialize().expect("files");
            let how: Vec<&str> = fds.iter().map(|fd| {
                let flags = crate::sys::fd::flags(fd.as_raw_fd());
                if flags < 0 {
                    "unknown"
                } else if flags & libc::O_PATH != 0 {
                    "named"
                } else if flags & libc::O_ACCMODE == libc::O_RDONLY {
                    "read"
                } else {
                    "written"
                }
            }).collect();
            h.borrow_mut().push(format!("files {}", how.join(" ")));
            a.reply(&());
        }
        other => a.fail("org.freedesktop.DBus.Error.UnknownMethod", other),
    }
}

#[test]
fn the_portal_is_told_a_drag_out_may_only_be_read_unless_it_is_a_shift_move() {
    use crate::bus::testing::{daemon, until};
    use crate::bus::{Asked, Bus, Offer, Props};
    use crate::event::Loop;
    use std::cell::RefCell;
    use std::rc::Rc;
    use std::sync::atomic::AtomicBool;
    use std::sync::Arc;
    assert!(!writable_out(ways_out(false, false)) && !writable_out(ways_out(true, false)) && !writable_out(ways_out(true, true)));
    assert!(writable_out(ways_out(false, true)));
    assert!(!writable_out(Ways::ALL) && !writable_out(Ways::NONE) && !writable_out(Ways::MOVE | Ways::COPY));
    let d = daemon().expect("a test bus");
    assert!(d.1.starts_with("unix:"), "{}", d.1);
    assert_ne!(Some(d.1.clone()), std::env::var("DBUS_SESSION_BUS_ADDRESS").ok(), "never the real bus");
    let lp = Loop::new();
    let fake = Bus::at(&lp, &d.1).expect("the made-up portal's bus");
    let named = Rc::new(std::cell::Cell::new(false));
    let n = named.clone();
    fake.own_firmly(portal::NAME, &lp, move |ok| n.set(ok));
    assert!(until(&lp, move || named.get()), "the made-up portal owns the name");
    let heard: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let h = heard.clone();
    let _served = fake.serve(portal::PATH, Offer {
        name: portal::FACE.into(),
        xml: format!("<interface name=\"{}\"/>", portal::FACE),
        call: Box::new(move |a: Asked| portal_answers(a, &h)),
        props: Box::new(Props::new),
        set: None,
    }).expect("served");
    let dir = crate::common::files::private_temp("trindle-dnd-portal").unwrap();
    std::fs::create_dir_all(dir.join("folder")).unwrap();
    std::fs::write(dir.join("report.txt"), "the report").unwrap();
    let files = vec![dir.join("report.txt"), dir.join("folder")];
    let done = Arc::new(AtomicBool::new(false));
    let (flag, address) = (done.clone(), d.1.clone());
    let client = std::thread::spawn(move || {
        let bus = zbus::blocking::connection::Builder::address(address.as_str()).unwrap().method_timeout(Duration::from_secs(5)).build().unwrap();
        let keys: Vec<Option<String>> = [(false, false), (true, false), (false, true)].into_iter()
            .map(|(right, shift)| portal::start_on(&bus, &files, out_plan(true, true, right, shift).1)).collect();
        flag.store(true, Ordering::SeqCst);
        keys
    });
    for _ in 0..4 {
        let d = done.clone();
        if until(&lp, move || d.load(Ordering::SeqCst)) {
            break;
        }
    }
    let keys = client.join().unwrap();
    let _ = std::fs::remove_dir_all(&dir);
    assert_eq!(keys, vec![Some("made-up-key".to_string()); 3]);
    assert_eq!(*heard.borrow(), ["start, writable Some(false)", "files read read", "start, writable Some(false)", "files read read",
                                 "start, writable Some(true)", "files named named"]);
}

#[test]
fn only_a_shift_move_of_files_on_a_modern_compositor_may_be_written_to() {
    for modern in [false, true] {
        for files in [false, true] {
            for right in [false, true] {
                for shift in [false, true] {
                    let (ways, writable) = out_plan(modern, files, right, shift);
                    assert_eq!(writable, modern && files && !right && shift, "modern {modern}, files {files}, right {right}, shift {shift}");
                    let offered = if !files { Ways::COPY } else { ways_out(right, shift) };
                    assert_eq!(ways, offered);
                }
            }
        }
    }
}

#[test]
fn a_files_drag_is_not_taken_by_reading_only_its_text() {
    let took = |sends: &Sends| {
        let mut out = Outgoing::new();
        out.action(Way::Move.bits());
        out.performed();
        out.finished(sends)
    };
    let sends = Sends::for_files();
    sends.let_go();
    let late = sends.begin_form("text/plain");
    sends.ended_whole_form(late, "text/plain");
    assert_eq!(took(&sends), End::Unshown(Way::Move), "the text alone proves no file was taken");
    assert_eq!(sends.taken(Way::Move), None);
    let late = sends.begin_form(URI_LIST);
    sends.ended_whole_form(late, URI_LIST);
    assert_eq!(took(&sends), End::Took(Taken::made_up(Way::Move)));
}

#[test]
fn a_drag_is_only_taken_when_every_byte_was_handed_over() {
    sends_are_counted_whole();
    every_form_asked_for_is_read();
    what_came_to_nothing_stays_nothing();
    a_real_pipe_proves_it();
}

fn took(sends: &Sends) -> End {
    let mut out = Outgoing::new();
    out.action(Way::Move.bits());
    out.performed();
    out.finished(sends)
}

fn moved() -> End {
    End::Took(Taken::made_up(Way::Move))
}

fn sends_are_counted_whole() {
    assert_eq!(took(&Sends::default()), End::Short(Way::Move));
    let sends = Sends::default();
    sends.let_go();
    let late = sends.begin_form("");
    sends.ended_whole_form(late, "");
    assert_eq!(took(&sends), moved());
    let sends = Sends::default();
    let early = sends.begin_form("");
    sends.ended_whole_form(early, "");
    sends.let_go();
    assert_eq!(took(&sends), moved());
    let late = sends.begin_form("");
    assert_eq!(took(&sends), End::Short(Way::Move));
    assert_eq!(sends.taken(Way::Copy), None);
    sends.ended_whole_form(late, "");
    assert_eq!(took(&sends), moved());
}

fn every_form_asked_for_is_read() {
    let sends = Sends::default();
    sends.begin_form("key");
    sends.let_go();
    assert_eq!(took(&sends), End::Short(Way::Move));
    let late = sends.begin_form("list");
    sends.ended_whole_form(late, "list");
    assert_eq!(took(&sends), End::Short(Way::Move), "a form asked for and never read was passed over");
    assert_eq!(sends.taken(Way::Move), None);
    let sends = Sends::default();
    let list = sends.begin_form("list");
    sends.ended_whole_form(list, "list");
    sends.begin_form("key");
    sends.let_go();
    assert_eq!(took(&sends), End::Short(Way::Move), "one form read of two that were asked for");
    let sends = Sends::default();
    sends.begin_form("list");
    sends.begin_form("key");
    sends.let_go();
    for form in ["list", "key"] {
        let late = sends.begin_form(form);
        sends.ended_whole_form(late, form);
    }
    assert_eq!(took(&sends), moved(), "a look that was given up stopped the move");
    let sends = Sends::default();
    sends.begin_form("list");
    sends.begin_form("key");
    sends.let_go();
    let late = sends.begin_form("list");
    sends.ended_whole_form(late, "list");
    assert_eq!(took(&sends), End::Short(Way::Move), "a form never read after it was let go was passed over");
    let late = sends.begin_form("key");
    assert_eq!(took(&sends), End::Short(Way::Move), "a form being written was passed over");
    sends.ended_whole_form(late, "key");
    assert_eq!(took(&sends), moved());
    let sends = Sends::default();
    sends.begin_form("text");
    sends.let_go();
    let late = sends.begin_form("list");
    sends.ended_unshown_form(late, "list");
    assert_eq!(took(&sends), End::Short(Way::Move));
}

fn what_came_to_nothing_stays_nothing() {
    assert!(crate::common::source::code(include_str!("../window/drag.rs")).contains("begin_form(&mime_type)"));
    let whole = Sends::default();
    whole.ended_whole_form(whole.begin_form(""), "");
    assert_eq!(Outgoing::new().finished(&whole), End::Nothing);
    let mut out = Outgoing::new();
    out.performed();
    assert_eq!(out.cancelled(), End::Nothing);
    assert_eq!(out.finished(&whole), End::Nothing, "finished is heard once");
    assert_eq!(whole.taken(Way::Move).map(Taken::way), Some(Way::Move));
    assert_eq!(Sends::default().taken(Way::Move), None);
}

fn a_real_pipe_proves_it() {
    let big = vec![b'x'; 4 << 20];
    let sends = std::sync::Arc::new(Sends::default());
    sends.let_go();
    let (keep, write) = pipe();
    let (late, noted) = (sends.begin_form(""), sends.clone());
    let got = write_all_then(write.into(), &big, Instant::now() + Duration::from_millis(200), |got| {
        if proven(got) {
            noted.ended_whole_form(late, "");
        }
    });
    assert_eq!(got, Err(Why::TooSlow));
    drop(keep);
    assert_eq!(took(&sends), End::Short(Way::Move));
    let sends = std::sync::Arc::new(Sends::default());
    sends.let_go();
    let (read, write) = pipe();
    let (late, noted) = (sends.begin_form(""), sends.clone());
    let seen = sends.clone();
    let reader = std::thread::spawn(move || {
        let all = read_all(read, 8 << 20, Instant::now() + Duration::from_secs(10)).map(|got| got.len());
        (all, seen.whole())
    });
    let got = write_all_then(write.into(), &big, Instant::now() + Duration::from_secs(10), |got| {
        if proven(got) {
            noted.ended_whole_form(late, "");
        }
    });
    assert_eq!(got, Ok(Sent::Read));
    assert_eq!(reader.join().unwrap(), (Ok(big.len()), true));
    assert_eq!(took(&sends), moved());
}

#[test]
fn a_reader_that_never_reads_is_given_up_on_and_its_place_is_free_again() {
    let big = vec![b'x'; 4 << 20];
    let (keep, write) = pipe();
    let began = Instant::now();
    assert_eq!(write_all(write.into(), &big, began + Duration::from_millis(200)), Err(Why::TooSlow));
    assert!(began.elapsed() >= Duration::from_millis(150) && began.elapsed() < Duration::from_secs(10), "{:?}", began.elapsed());
    drop(keep);
    let (read, write) = pipe();
    let reader = std::thread::spawn(move || read_all(read, 8 << 20, Instant::now() + Duration::from_secs(10)));
    assert_eq!(write_all(write.into(), &big, Instant::now() + Duration::from_secs(10)), Ok(Sent::Read));
    assert_eq!(reader.join().unwrap().map(|got| got.len()), Ok(big.len()));
    let (read, write) = pipe();
    drop(read);
    assert_eq!(write_all(write.into(), &big, Instant::now() + Duration::from_secs(10)), Err(Why::Empty));
    static PLACES: crate::common::run::Slots = crate::common::run::Slots::new(MOST_SENDING);
    let go = std::sync::Arc::new(std::sync::Barrier::new(MOST_SENDING + 1));
    let mut stuck = Vec::new();
    for _ in 0..MOST_SENDING {
        let (keep, write) = pipe();
        let slot = PLACES.take().expect("a place");
        let bytes = big.clone();
        let go = go.clone();
        stuck.push((keep, std::thread::spawn(move || {
            let _slot = slot;
            go.wait();
            write_all(write.into(), &bytes, Instant::now() + Duration::from_millis(300))
        })));
    }
    assert!(PLACES.take().is_none(), "a ninth is not given a thread");
    go.wait();
    for (keep, thread) in stuck {
        assert_eq!(thread.join().unwrap(), Err(Why::TooSlow));
        drop(keep);
    }
    let again: Vec<_> = (0..MOST_SENDING).map(|_| PLACES.take()).collect();
    assert!(again.iter().all(Option::is_some), "every place is free again");
    assert!(PLACES.take().is_none());
}

#[test]
fn a_pipe_dragged_out_is_turned_away_and_holds_nothing_up() {
    use std::os::unix::fs::OpenOptionsExt;
    let dir = crate::common::files::private_temp("trindle-dnd-pipe").unwrap();
    std::fs::create_dir_all(&dir).unwrap();
    let fifo = dir.join("pipe");
    let name = std::ffi::CString::new(fifo.as_os_str().as_bytes()).unwrap();
    assert_eq!(crate::sys::os::mkfifo(&name, 0o600), 0);
    let (tx, rx) = std::sync::mpsc::channel();
    let at = fifo.clone();
    std::thread::spawn(move || {
        let _ = tx.send(portal::opened(&at, false).is_some());
    });
    let got = rx.recv_timeout(Duration::from_secs(2));
    let _ = std::fs::OpenOptions::new().write(true).custom_flags(libc::O_NONBLOCK).open(&fifo);
    assert_eq!(got, Ok(false), "a pipe is not a file to hand over, and opening it must not wait");
    assert!(portal::opened(&fifo, true).is_none(), "nor is it named for a move");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_link_dragged_out_is_never_handed_over_as_what_it_leads_to() {
    let dir = crate::common::files::private_temp("trindle-dnd-link").unwrap();
    std::fs::create_dir_all(dir.join("private")).unwrap();
    std::fs::write(dir.join("private/key"), "a made-up key").unwrap();
    std::os::unix::fs::symlink(dir.join("private/key"), dir.join("invoice.pdf")).unwrap();
    std::os::unix::fs::symlink(dir.join("private"), dir.join("holiday")).unwrap();
    std::os::unix::fs::symlink("/no/such/place", dir.join("broken")).unwrap();
    for link in ["invoice.pdf", "holiday", "broken"] {
        for writable in [false, true] {
            assert!(portal::opened(&dir.join(link), writable).is_none(), "the link {link} was handed over (writable {writable})");
        }
    }
    assert!(portal::opened(&dir.join("private/key"), false).is_some() && portal::opened(&dir.join("private"), true).is_some());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_send_is_whole_only_when_the_other_program_has_read_all_of_it() {
    use std::io::Read;
    let list = b"file:///home/john/report.txt\r\nfile:///home/john/notes.txt\r\n";
    let soon = || Instant::now() + Duration::from_secs(5);
    let (read, write) = pipe();
    let closer = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(100));
        drop(read);
    });
    assert_eq!(write_all(write.into(), list, soon()), Err(Why::Empty), "left unread and closed");
    closer.join().unwrap();
    let (read, write) = pipe();
    let half = std::thread::spawn(move || {
        let mut some = [0u8; 20];
        std::fs::File::from(read).read_exact(&mut some).unwrap();
    });
    assert_eq!(write_all(write.into(), list, soon()), Err(Why::Empty), "half read and closed");
    half.join().unwrap();
    let (keep, write) = pipe();
    let began = Instant::now();
    assert_eq!(write_all(write.into(), list, began + Duration::from_millis(200)), Err(Why::TooSlow), "never read");
    assert!(began.elapsed() >= Duration::from_millis(150) && began.elapsed() < Duration::from_secs(10), "{:?}", began.elapsed());
    drop(keep);
    let (read, write) = pipe();
    let reader = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(100));
        read_all(read, 1 << 20, Instant::now() + Duration::from_secs(5))
    });
    assert_eq!(write_all(write.into(), list, soon()), Ok(Sent::Read));
    assert_eq!(reader.join().unwrap(), Ok(list.to_vec()));
}

#[test]
fn a_send_down_a_file_or_a_socket_is_never_proof_that_it_was_read() {
    let list = b"file:///home/john/report.txt\r\nfile:///home/john/notes.txt\r\n";
    let soon = || Instant::now() + Duration::from_secs(5);
    let taken = |to: OwnedFd| {
        let sends = Sends::default();
        sends.let_go();
        let late = sends.begin_form("");
        let _ = write_all_then(to, list, soon(), |got| sends.ended_form(late, "", got));
        let mut out = Outgoing::new();
        out.action(Way::Move.bits());
        out.performed();
        out.finished(&sends)
    };
    let dir = crate::common::files::private_temp("trindle-dnd-file").unwrap();
    std::fs::create_dir_all(&dir).unwrap();
    let file = std::fs::File::create(dir.join("taken")).unwrap();
    assert_eq!(taken(file.into()), End::Unshown(Way::Move), "a file that took the bytes was proof they were read");
    assert_eq!(std::fs::read(dir.join("taken")).unwrap(), list, "it was written all the same: a copy is not held back");
    let (ours, theirs) = std::os::unix::net::UnixStream::pair().unwrap();
    let closer = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(100));
        drop(theirs);
    });
    assert_eq!(taken(ours.into()), End::Unshown(Way::Move), "a socket closed unread was proof it was read");
    closer.join().unwrap();
    let (ours, mut theirs) = std::os::unix::net::UnixStream::pair().unwrap();
    let reader = std::thread::spawn(move || {
        let mut all = Vec::new();
        std::io::Read::read_to_end(&mut theirs, &mut all).map(|_| all)
    });
    assert_eq!(taken(ours.into()), End::Unshown(Way::Move), "a socket was proof it was read");
    assert_eq!(reader.join().unwrap().ok().as_deref(), Some(&list[..]));
    let (read, write) = pipe();
    let reader = std::thread::spawn(move || read_all(read, 1 << 20, Instant::now() + Duration::from_secs(5)));
    assert_eq!(taken(write.into()), End::Took(Taken::made_up(Way::Move)));
    assert_eq!(reader.join().unwrap(), Ok(list.to_vec()));
    let copy = |sends: &Sends| {
        let mut out = Outgoing::new();
        out.action(Way::Copy.bits());
        out.performed();
        out.finished(sends)
    };
    let sends = Sends::default();
    sends.let_go();
    sends.ended_form(sends.begin_form(""), "", &Ok(Sent::Unshown));
    assert!(sends.written() && sends.copied() && !sends.whole());
    assert_eq!((copy(&sends), sends.taken(Way::Move), sends.taken(Way::Copy)), (End::Nothing, None, None));
    sends.ended_form(sends.begin_form(""), "", &Ok(Sent::Read));
    assert!(sends.taken(Way::Move).is_none() && sends.written());
    sends.ended_form(sends.begin_form(""), "", &Err(Why::TooSlow));
    assert!(!sends.written() && !sends.copied());
    assert_eq!(copy(&sends), End::Short(Way::Copy));
    let null = std::fs::OpenOptions::new().write(true).open("/dev/null").unwrap();
    assert_eq!(write_all(null.into(), list, soon()), Err(Why::Empty));
    let _ = std::fs::remove_dir_all(&dir);
}

fn files(list: &str) -> Result<Vec<PathBuf>, Why> {
    parse_uri_list(list.as_bytes())
}

fn paths(list: &[&str]) -> Vec<PathBuf> {
    list.iter().map(PathBuf::from).collect()
}

#[test]
fn a_list_of_files_is_read_as_other_programs_write_it() {
    assert_eq!(files("file:///home/john/report.pdf\r\n"), Ok(paths(&["/home/john/report.pdf"])));
    assert_eq!(files("file:///home/john/Photos/"), Ok(paths(&["/home/john/Photos"])));
    let list = "# from a test\r\nfile:///home/john/a.txt\nfile://localhost/home/john/b%20c.txt\r\nFILE:///home/john/a.txt\r\n";
    assert_eq!(files(list), Ok(paths(&["/home/john/a.txt", "/home/john/b c.txt"])));
    assert_eq!(parse_uri_list(b"file:///home/john/caf%C3%A9%23%3F.txt\r\n\0"), Ok(paths(&["/home/john/café#?.txt"])));
    let odd = files("file:///home/john/%FF%FE").unwrap();
    assert_eq!(odd[0].as_os_str().as_bytes(), b"/home/john/\xff\xfe");
}

#[test]
fn a_hostile_list_is_refused_whole() {
    let bad = [
        "http://example.com/a.txt",
        "file://example.com/home/john/a.txt",
        "file://home/john/a.txt",
        "smb://server/share/a.txt",
        "/home/john/a.txt",
        "file:/home/john/a.txt",
        "file:home/john/a.txt",
        "javascript:alert(1)",
        "file://",
        "file:///",
        "file:///home/john/../../etc/passwd",
        "file:///home/john/./a.txt",
        "file:///home//john/a.txt",
        "file:///home/john/..",
        "file:///home/%2E%2E/a.txt",
        "file:///home/john/a%00.txt",
        "file:///home/john/a%0Ab.txt",
        "file:///home/john/a%0Db.txt",
        "file:///home/john/a%1Bb.txt",
        "file:///home/john/a%7Fb.txt",
        "file:///home/john/a b.txt",
        "file:///home/john/a\tb.txt",
        "file:///home/john/caf\u{e9}.txt",
        "file:///home/john/a%2",
        "file:///home/john/a%zz.txt",
        "file:///home/john/a%",
        "file:///home/john/a.txt?x=1",
        "file:///home/john/a.txt#top",
    ];
    for line in bad {
        assert_eq!(files(line), Err(Why::NotFiles), "{line:?}");
        assert_eq!(files(&format!("file:///home/john/good.txt\r\n{line}\r\n")), Err(Why::NotFiles), "{line:?}");
    }
    assert_eq!(parse_uri_list(b"file:///home/john/a.txt\0\r\nfile:///home/john/b.txt"), Err(Why::NotFiles));
    assert_eq!(files(""), Err(Why::Empty));
    assert_eq!(files("\r\n# nothing\r\n"), Err(Why::Empty));
}

#[test]
fn a_list_too_long_or_a_name_too_long_is_refused() {
    assert_eq!(files(&format!("file:///home/{}", "a".repeat(256))), Err(Why::NotFiles));
    assert!(files(&format!("file:///home/{}", "a".repeat(255))).is_ok());
    let deep = "/abcdefgh".repeat(456);
    assert!(deep.len() > MOST_PATH);
    assert_eq!(files(&format!("file://{deep}")), Err(Why::NotFiles));
    let many = |n: usize| (0..n).fold(String::new(), |mut s, i| {
        let _ = std::fmt::Write::write_fmt(&mut s, format_args!("file:///home/john/{i}\r\n"));
        s
    });
    assert_eq!(files(&many(MOST_FILES)).map(|f| f.len()), Ok(MOST_FILES));
    assert_eq!(files(&many(MOST_FILES + 1)), Err(Why::TooMany));
    assert_eq!(parse_uri_list(&vec![b'#'; MOST_BYTES + 1]), Err(Why::TooBig));
}

#[test]
fn a_list_written_here_reads_back_the_same() {
    let odd = vec![
        PathBuf::from("/home/john/Report (final) 100%.pdf"),
        PathBuf::from("/home/john/café #1?.txt"),
        PathBuf::from(std::ffi::OsString::from_vec(b"/home/john/\xff\xfe".to_vec())),
    ];
    let list = uri_list(&odd);
    assert!(list.is_ascii() && list.ends_with("\r\n") && !list.contains(' '));
    assert_eq!(parse_uri_list(list.as_bytes()), Ok(odd));
    assert_eq!(path_lines(&paths(&["/a/b", "/c d"])), "/a/b\n/c d");
    assert_eq!(uri_list(&paths(&["relative.txt", "/a"])), "file:///a\r\n");
}

#[test]
fn the_portal_is_held_to_the_same_rules() {
    assert_eq!(portal_key(b"abc-123\0"), Some("abc-123".to_string()));
    assert_eq!(portal_key(b"abc-123"), Some("abc-123".to_string()));
    assert_eq!(portal_key(b"abc\0junk"), None);
    assert_eq!(portal_key(b"abc\0\0"), None);
    assert_eq!(portal_key(b""), None);
    assert_eq!(portal_key(b"\0"), None);
    assert_eq!(portal_key(b"two words"), None);
    assert_eq!(portal_key(b"line\nbreak"), None);
    assert_eq!(portal_key(&[b'k'; 256]), None);
    let s = |v: &[&str]| v.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    assert_eq!(checked_paths(&s(&["/run/user/1000/doc/ab12/a.txt"])), Ok(paths(&["/run/user/1000/doc/ab12/a.txt"])));
    assert_eq!(checked_paths(&s(&["/home/john/a.txt", "relative.txt"])), Err(Why::NotFiles));
    assert_eq!(checked_paths(&s(&["/home/john/../a.txt"])), Err(Why::NotFiles));
    assert_eq!(checked_paths(&s(&["/home/john/a\nb"])), Err(Why::NotFiles));
    assert_eq!(checked_paths(&[]), Err(Why::Empty));
    assert!(is_plain(Path::new("/home/john/a.txt")) && !is_plain(Path::new("a.txt")) && !is_plain(Path::new("/")));
}

#[test]
fn text_is_made_safe_to_show() {
    assert_eq!(clean_text(b"hello\nthere\tyou"), Some("hello\nthere\tyou".to_string()));
    assert_eq!(clean_text(b"a\0b\x1b[31mc\r\n"), Some("ab[31mc\n".to_string()));
    assert_eq!(clean_text(b"caf\xff"), Some("caf\u{fffd}".to_string()));
    assert_eq!(clean_text(b"\0\x07"), None);
    assert_eq!(clean_text(b""), None);
    assert_eq!(clean_text("a\u{202E}b\u{200F}c\u{200B}d\u{FEFF}".as_bytes()), Some("abcd".to_string()));
    assert_eq!(clean_text("a\u{200D}b".as_bytes()), Some("a\u{200D}b".to_string()));
}

fn offer(kinds: &[&str], ways: Ways) -> Incoming {
    let mut o = Incoming::new();
    for k in kinds {
        o.kind(k.to_string());
    }
    o.source_ways(ways.bits());
    o
}

#[test]
fn an_offer_is_answered_once_and_read_only_when_agreed() {
    let mut o = offer(&[URI_LIST, "text/plain"], Ways::ALL);
    assert_eq!(o.offered(), Offered { files: true, text: true, ways: Ways::ALL });
    assert_eq!(o.let_go(true), None);
    let yes = o.answer(Some((Way::Copy, Kind::Files))).expect("an answer");
    assert_eq!(yes, Answer { accept: Some(URI_LIST.into()), ways: Ways::COPY, rather: Some(Way::Copy) });
    assert_eq!(o.answer(Some((Way::Copy, Kind::Files))), None);
    assert_eq!(o.let_go(true), None);
    o.chosen(Way::Copy.bits());
    assert_eq!(o.let_go(true), Some(Pull { kind: Kind::Files, main: URI_LIST.into(), portal: None, text: Some("text/plain".into()), way: Way::Copy }));
    assert_eq!(o.answer(Some((Way::Copy, Kind::Text))).and_then(|a| a.accept), Some("text/plain".to_string()));
    assert_eq!(o.answer(None), Some(Answer { accept: None, ways: Ways::NONE, rather: None }));
    assert_eq!(o.answer(None), None);
    o.chosen(0);
    assert_eq!(o.let_go(true), None);
    o.answer(Some((Way::Move, Kind::Files)));
    assert_eq!(o.let_go(true), None);
    o.chosen(Way::Move.bits());
    assert_eq!(o.let_go(true).map(|p| p.way), Some(Way::Move));
}

#[test]
fn a_way_or_kind_that_is_not_offered_is_not_taken() {
    let mut o = offer(&[URI_LIST], Ways::COPY);
    assert_eq!(o.answer(Some((Way::Move, Kind::Files))).and_then(|a| a.accept), None);
    assert_eq!(o.let_go(true), None);
    let mut o = offer(&["text/plain;charset=utf-8"], Ways::ALL);
    assert_eq!(o.offered(), Offered { files: false, text: true, ways: Ways::ALL });
    assert_eq!(o.answer(Some((Way::Copy, Kind::Files))).and_then(|a| a.accept), None);
    let o = offer(&["image/png"], Ways::ALL);
    assert_eq!(o.offered(), Offered { files: false, text: false, ways: Ways::ALL });
    let mut o = Incoming::new();
    o.kind(URI_LIST.into());
    assert_eq!(o.offered().ways, Ways::COPY);
    o.answer(Some((Way::Copy, Kind::Files)));
    assert_eq!(o.let_go(false).map(|p| p.way), Some(Way::Copy));
    assert_eq!(o.let_go(true), None);
}

#[test]
fn the_portal_is_asked_as_well_when_it_is_offered() {
    let mut o = offer(&[PORTAL_KINDS[0], URI_LIST], Ways::COPY);
    o.answer(Some((Way::Copy, Kind::Files)));
    o.chosen(1);
    assert_eq!(o.let_go(true), Some(Pull { kind: Kind::Files, main: URI_LIST.into(), portal: Some(PORTAL_KINDS[0].into()), text: None, way: Way::Copy }));
    let mut o = offer(&[PORTAL_KINDS[1]], Ways::COPY);
    assert!(o.offered().files);
    o.answer(Some((Way::Copy, Kind::Files)));
    o.chosen(1);
    assert_eq!(o.let_go(true), Some(Pull { kind: Kind::Files, main: PORTAL_KINDS[1].into(), portal: None, text: None, way: Way::Copy }));
}

#[test]
fn an_offer_naming_kinds_without_end_is_cut_short() {
    let mut o = Incoming::new();
    for i in 0..100_000 {
        o.kind(format!("application/x-made-up-{i}"));
    }
    o.kind("k".repeat(100_000));
    o.kind(URI_LIST.into());
    assert_eq!(o.kinds.len(), MOST_KINDS);
    assert!(!o.offered().files, "what came after the limit was not heard");
    assert!(o.kinds.iter().all(|k| k.len() <= MOST_KIND));
}

#[test]
fn the_way_follows_the_program_it_comes_from_and_the_keys() {
    assert_eq!(usual_way(Ways::ALL, false, false), Some(Way::Copy));
    assert_eq!(usual_way(Ways::ALL, true, false), Some(Way::Move));
    assert_eq!(usual_way(Ways::ALL, false, true), Some(Way::Copy));
    assert_eq!(usual_way(Ways::MOVE, false, false), Some(Way::Move));
    assert_eq!(usual_way(Ways::COPY, true, false), Some(Way::Copy), "Shift cannot move what may only be copied");
    assert_eq!(usual_way(Ways::ASK | Ways::COPY, false, false), Some(Way::Ask));
    assert_eq!(usual_way(Ways::ASK, false, false), Some(Way::Ask));
    assert_eq!(usual_way(Ways::NONE, false, false), None);
    assert_eq!(ways_out(false, false), Ways::COPY);
    assert_eq!(ways_out(false, true), Ways::MOVE);
    assert_eq!(ways_out(true, false), Ways::ASK | Ways::COPY);
    assert_eq!(ways_out(true, true), Ways::ASK | Ways::COPY);
    assert_eq!(usual_way(ways_out(true, false), false, false), Some(Way::Ask));
    assert_eq!(usual_way(ways_out(false, true), false, false), Some(Way::Move));
    assert_eq!(usual_way(ways_out(false, false), true, false), Some(Way::Copy));
    assert_eq!(Ways::from_bits(0xffff_fff8), Ways::NONE);
    assert_eq!(Way::from_bits(3), None);
}

#[test]
fn a_move_is_only_told_once_the_other_side_has_finished() {
    let whole = Sends::default();
    whole.ended_whole_form(whole.begin_form(""), "");
    let mut out = Outgoing::new();
    out.action(Way::Move.bits());
    out.performed();
    assert!(!out.is_over());
    assert_eq!(out.finished(&whole), End::Took(Taken::made_up(Way::Move)));
    assert_eq!(out.finished(&whole), End::Nothing, "told once");
    let mut out = Outgoing::new();
    out.action(Way::Move.bits());
    out.performed();
    assert_eq!(out.cancelled(), End::Nothing);
    assert_eq!(out.finished(&whole), End::Nothing);
    let mut out = Outgoing::new();
    out.action(Way::Move.bits());
    assert_eq!(out.finished(&whole), End::Nothing);
    let mut out = Outgoing::new();
    assert_eq!(out.cancelled(), End::Nothing);
    assert!(out.is_over());
    for bits in [1, 4, 0] {
        let mut out = Outgoing::new();
        out.action(bits);
        out.performed();
        assert_eq!(out.finished(&whole), End::Took(Taken::made_up(Way::Copy)));
    }
    let mut out = Outgoing::new();
    out.action(1);
    out.performed();
    out.cancelled();
    out.action(2);
    assert_eq!(out.finished(&whole), End::Nothing);
}

fn pipe() -> (OwnedFd, std::fs::File) {
    let (read, write) = crate::common::run::pipe().unwrap();
    (read, std::fs::File::from(write))
}

#[test]
fn what_is_sent_is_read_within_its_limits() {
    let soon = || Instant::now() + Duration::from_secs(5);
    let (read, mut write) = pipe();
    write.write_all(b"file:///home/john/a.txt\r\n").unwrap();
    drop(write);
    assert_eq!(read_all(read, MOST_BYTES, soon()), Ok(b"file:///home/john/a.txt\r\n".to_vec()));
    let (read, write) = pipe();
    drop(write);
    assert_eq!(read_all(read, MOST_BYTES, soon()), Ok(Vec::new()));
    let (read, mut write) = pipe();
    let sender = std::thread::spawn(move || {
        let chunk = [b'x'; 65536];
        while write.write_all(&chunk).is_ok() {}
    });
    assert_eq!(read_all(read, MOST_BYTES, soon()), Err(Why::TooBig));
    sender.join().unwrap();
    let (read, mut write) = pipe();
    write.write_all(b"file:///home/john/").unwrap();
    let began = Instant::now();
    assert_eq!(read_all(read, MOST_BYTES, began + Duration::from_millis(200)), Err(Why::TooSlow));
    assert!(began.elapsed() < Duration::from_secs(2));
    drop(write);
    let (read, _write) = pipe();
    assert_eq!(read_all(read, MOST_BYTES, Instant::now()), Err(Why::TooSlow));
}

#[test]
fn data_of_any_kind_is_asked_for_by_its_own_name_and_handed_over_as_it_came() {
    let mut o = offer(&["application/x-card", "text/plain"], Ways::COPY);
    let yes = o.answer(Some((Way::Copy, Kind::Typed("application/x-card".into())))).expect("an answer");
    assert_eq!(yes.accept, Some("application/x-card".into()));
    assert_eq!(o.answer(Some((Way::Copy, Kind::Typed("image/png".into())))).and_then(|a| a.accept), None);
    o.answer(Some((Way::Copy, Kind::Typed("application/x-card".into()))));
    o.chosen(Way::Copy.bits());
    let card = Kind::Typed("application/x-card".into());
    assert_eq!(o.let_go(true), Some(Pull { kind: card.clone(), main: "application/x-card".into(), portal: None, text: None, way: Way::Copy }));
    assert_eq!(given(card.clone(), Ok(vec![0, 255, 7]), None, None), Given::Typed("application/x-card".into(), vec![0, 255, 7]));
    assert_eq!(given(card.clone(), Ok(Vec::new()), None, None), Given::Nothing(Why::Empty));
    assert_eq!(given(card, Err(Why::TooBig), None, None), Given::Nothing(Why::TooBig));
}

#[test]
fn what_was_read_becomes_what_a_widget_is_given() {
    let list = || Ok(b"file:///home/john/a.txt\r\n".to_vec());
    assert_eq!(given(Kind::Files, list(), None, None), Given::Files(paths(&["/home/john/a.txt"])));
    assert_eq!(given(Kind::Files, list(), Some(paths(&["/run/user/1000/doc/1/a.txt"])), None), Given::Files(paths(&["/run/user/1000/doc/1/a.txt"])));
    assert_eq!(given(Kind::Files, Ok(b"hello".to_vec()), None, None), Given::Nothing(Why::NotFiles));
    assert_eq!(given(Kind::Files, Err(Why::TooBig), None, None), Given::Nothing(Why::TooBig));
    assert_eq!(given(Kind::Text, Ok(b"hello".to_vec()), None, None), Given::Text("hello".into()));
    assert_eq!(given(Kind::Text, Ok(Vec::new()), None, None), Given::Nothing(Why::Empty));
    assert_eq!(given(Kind::Text, Err(Why::TooSlow), None, None), Given::Nothing(Why::TooSlow));
    let link = || Ok(b"https://example.com/page\r\n".to_vec());
    assert_eq!(given(Kind::Files, link(), None, Some(b"https://example.com/page".to_vec())), Given::Text("https://example.com/page".into()));
    assert_eq!(given(Kind::Files, link(), None, Some(Vec::new())), Given::Nothing(Why::NotFiles));
    assert_eq!(given(Kind::Files, list(), None, Some(b"other".to_vec())), Given::Files(paths(&["/home/john/a.txt"])));
    assert_eq!(given(Kind::Files, Err(Why::TooSlow), None, Some(b"other".to_vec())), Given::Nothing(Why::TooSlow));
    for why in [Why::Empty, Why::TooBig, Why::TooSlow, Why::TooMany, Why::NotFiles] {
        assert!(why.words().ends_with('.'));
    }
}
