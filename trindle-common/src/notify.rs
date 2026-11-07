//! The `notify.jsonl` and `status.txt` a plugin keeps in its home, which the ring shows under the plugin's name.

use std::io::{Read, Seek, SeekFrom, Write};
use std::os::fd::AsRawFd;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;

use crate::files::{write_private, SMALL_MOST};

const TRIM_AT: usize = 50;
const KEEP: usize = 20;

pub fn notify_line(path: &Path, summary: &str, body: &str) -> std::io::Result<()> {
    let mut file = std::fs::OpenOptions::new().read(true).append(true).create(true).mode(0o600)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC).open(path)?;
    if crate::sys::lock(file.as_raw_fd()) != 0 {
        return Err(std::io::Error::last_os_error());
    }
    let size = file.metadata()?.len();
    let from = size.saturating_sub(SMALL_MOST);
    let mut tail = Vec::new();
    file.seek(SeekFrom::Start(from))?;
    (&file).take(SMALL_MOST).read_to_end(&mut tail)?;
    let tail = String::from_utf8_lossy(&tail);
    let lines: Vec<&str> = tail.lines().skip(usize::from(from > 0)).collect();
    if from > 0 || lines.len() > TRIM_AT {
        file.set_len(0)?;
        let keep = &lines[lines.len().saturating_sub(KEEP)..];
        if !keep.is_empty() {
            file.write_all((keep.join("\n") + "\n").as_bytes())?;
        }
    }
    let line = format!("{{\"summary\":{},\"body\":{}}}\n", json_text(summary), json_text(body));
    file.write_all(line.as_bytes())
}

pub fn status_line(path: &Path, line: &str) -> std::io::Result<()> {
    write_private(path, format!("{line}\n").as_bytes())
}

fn json_text(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn folder(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("trindle-common-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn a_line_escapes_what_would_break_it() {
        assert_eq!(json_text("a\"b\\c\nd\u{1}"), "\"a\\\"b\\\\c\\nd\\u0001\"");
    }

    #[test]
    fn a_long_file_is_cut_back_to_its_last_lines() {
        let dir = folder("notify");
        let path = dir.join("notify.jsonl");
        for n in 0..60 {
            notify_line(&path, "s", &n.to_string()).unwrap();
        }
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.lines().count() <= TRIM_AT, "{} lines", text.lines().count());
        assert!(text.lines().last().unwrap().contains("\"59\""));
        status_line(&dir.join("status.txt"), "ready").unwrap();
        assert_eq!(std::fs::read_to_string(dir.join("status.txt")).unwrap(), "ready\n");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn writers_at_once_each_add_whole_lines() {
        let dir = folder("notify-many");
        let path = dir.join("notify.jsonl");
        let all: Vec<_> = (0..4).map(|w| {
            let path = path.clone();
            std::thread::spawn(move || {
                for n in 0..40 {
                    notify_line(&path, &format!("w{w}"), &"x".repeat(200 + n)).unwrap();
                }
            })
        }).collect();
        for t in all {
            t.join().unwrap();
        }
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.lines().count() <= TRIM_AT + 1, "{} lines", text.lines().count());
        assert!(text.lines().all(|l| l.starts_with("{\"summary\":\"w") && l.ends_with("\"}")), "a line was broken");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
