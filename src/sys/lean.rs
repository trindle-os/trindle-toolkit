//! Keeping the GPU drawer's memory down: what the GL stack needed only to start is handed back once it is running.

use std::collections::HashSet;
use std::sync::OnceLock;

static BEFORE: OnceLock<HashSet<String>> = OnceLock::new();

pub(super) fn note_libraries() {
    BEFORE.get_or_init(|| files(&std::fs::read_to_string("/proc/self/maps").unwrap_or_default()));
}

fn files(maps: &str) -> HashSet<String> {
    maps.lines().filter_map(|l| l.split_whitespace().nth(5)).filter(|p| p.starts_with('/')).map(String::from).collect()
}

fn untouched_new(smaps: &str, before: &HashSet<String>) -> Vec<(usize, usize)> {
    let mut found = Vec::new();
    let mut now: Option<(usize, usize)> = None;
    for l in smaps.lines() {
        let mut f = l.split_whitespace();
        let first = f.next().unwrap_or("");
        if let Some(key) = first.strip_suffix(':') {
            if key == "Anonymous" && f.next() != Some("0") {
                now = None;
            }
            continue;
        }
        if let Some(r) = now.take() {
            found.push(r);
        }
        let perms = f.next().unwrap_or("");
        let path = f.nth(3).unwrap_or("");
        if !path.starts_with('/') || before.contains(path) || perms.as_bytes().get(1) != Some(&b'-') {
            continue;
        }
        if let Some((a, b)) = first.split_once('-') {
            if let (Ok(a), Ok(b)) = (usize::from_str_radix(a, 16), usize::from_str_radix(b, 16)) {
                now = Some((a, b - a));
            }
        }
    }
    found.extend(now);
    found
}

pub(crate) fn settle() {
    // SAFETY: a plain call that returns unused heap to the system.
    #[cfg(target_env = "gnu")]
    unsafe {
        libc::malloc_trim(0)
    };
    let Some(before) = BEFORE.get() else { return };
    let smaps = std::fs::read_to_string("/proc/self/smaps").unwrap_or_default();
    for (at, len) in untouched_new(&smaps, before) {
        // SAFETY: a private file mapping that cannot be written and has no changed page (Anonymous is 0), so
        // MADV_DONTNEED only drops pages that are read back from the file, with the same contents, on the next use.
        unsafe { libc::madvise(at as *mut libc::c_void, len, libc::MADV_DONTNEED) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_new_read_only_unchanged_file_mappings_are_chosen() {
        let smaps = "\
1000-3000 r-xp 00000000 08:01 1 /usr/lib/libc.so.6
Anonymous:             0 kB
4000-6000 r-xp 00000000 08:01 2 /usr/lib/libLLVM.so.19.1
Rss:                  40 kB
Anonymous:             0 kB
6000-7000 rw-p 00002000 08:01 2 /usr/lib/libLLVM.so.19.1
Anonymous:             4 kB
7000-8000 r--p 00003000 08:01 2 /usr/lib/libLLVM.so.19.1
Anonymous:             4 kB
8000-9000 r--p 00004000 08:01 2 /usr/lib/libLLVM.so.19.1
Anonymous:             0 kB
9000-a000 r--p 00000000 00:00 0 [vdso]
Anonymous:             0 kB
b000-c000 r-xp 00000000 00:00 0
Anonymous:             0 kB
";
        let before = files("1000-3000 r-xp 00000000 08:01 1 /usr/lib/libc.so.6");
        assert_eq!(untouched_new(smaps, &before), vec![(0x4000, 0x2000), (0x8000, 0x1000)]);
    }

    #[test]
    fn settling_leaves_the_program_working() {
        note_libraries();
        settle();
        let v: Vec<u8> = (0..100_000).map(|i| i as u8).collect();
        assert_eq!(v[99_999], 99_999u32 as u8);
    }
}
