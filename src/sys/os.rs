//! Asking the system about disks, accounts, names and time.

use std::ffi::CStr;
use std::os::fd::{FromRawFd, RawFd};
use std::os::unix::ffi::OsStrExt;

pub fn statvfs(path: &CStr) -> Option<libc::statvfs> {
    // SAFETY: statvfs is plain numbers, so all-zeroes is a valid starting
    // value, and the kernel fills it in.
    let mut st: libc::statvfs = unsafe { std::mem::zeroed() };
    // SAFETY: a NUL-terminated path that outlives the call, and a local
    // struct the kernel fills in.
    (unsafe { libc::statvfs(path.as_ptr(), &mut st) } == 0).then_some(st)
}

pub fn uid_of(name: &CStr) -> Option<u32> {
    // SAFETY: passwd is plain numbers and pointers; all zero is a valid empty one.
    let mut entry: libc::passwd = unsafe { std::mem::zeroed() };
    let mut buf = vec![0u8; 4096];
    let mut found: *mut libc::passwd = std::ptr::null_mut();
    loop {
        // SAFETY: a NUL-terminated name in; `entry`, the buffer and `found`
        // are this call's own and live through it.
        let code = unsafe { libc::getpwnam_r(name.as_ptr(), &mut entry, buf.as_mut_ptr().cast(), buf.len(), &mut found) };
        if code == libc::ERANGE && buf.len() < 1 << 20 {
            buf.resize(buf.len() * 4, 0);
            continue;
        }
        return (code == 0 && !found.is_null()).then_some(entry.pw_uid);
    }
}

fn own_entry<R>(pick: impl FnOnce(&libc::passwd) -> R) -> Option<R> {
    // SAFETY: passwd is plain numbers and pointers; all zero is a valid empty one.
    let mut entry: libc::passwd = unsafe { std::mem::zeroed() };
    let mut buf = vec![0u8; 4096];
    let mut found: *mut libc::passwd = std::ptr::null_mut();
    loop {
        // SAFETY: `entry`, the buffer and `found` are this call's own and live through it.
        let code = unsafe { libc::getpwuid_r(libc::getuid(), &mut entry, buf.as_mut_ptr().cast(), buf.len(), &mut found) };
        if code == libc::ERANGE && buf.len() < 1 << 20 {
            buf.resize(buf.len() * 4, 0);
            continue;
        }
        return (code == 0 && !found.is_null()).then(|| pick(&entry));
    }
}

fn field(p: *const libc::c_char) -> Option<Vec<u8>> {
    // SAFETY: a field of an entry `own_entry` found points at a NUL-ended string inside its buffer, which is alive here.
    (!p.is_null()).then(|| unsafe { CStr::from_ptr(p) }.to_bytes().to_vec())
}

pub fn user() -> String {
    own_entry(|e| field(e.pw_name)).flatten().map(|b| String::from_utf8_lossy(&b).into_owned()).unwrap_or_default()
}

pub fn home_dir() -> Option<std::path::PathBuf> {
    let dir = std::path::PathBuf::from(std::ffi::OsStr::from_bytes(&own_entry(|e| field(e.pw_dir))??));
    dir.is_absolute().then_some(dir)
}

pub fn rename_new(from: &CStr, to: &CStr) -> libc::c_int {
    // SAFETY: both names are strings that end in a nought, kept alive
    // until the call returns; AT_FDCWD means they are read as written.
    unsafe { libc::renameat2(libc::AT_FDCWD, from.as_ptr(), libc::AT_FDCWD, to.as_ptr(), libc::RENAME_NOREPLACE) }
}

pub fn open_beneath(dir: RawFd, name: &CStr, flags: u64, resolve: u64) -> std::io::Result<std::fs::File> {
    // SAFETY: open_how is plain numbers, for which all noughts is a
    // sound value; every field that matters is set below.
    let mut how: libc::open_how = unsafe { std::mem::zeroed() };
    how.flags = flags;
    how.resolve = resolve;
    // SAFETY: the name is a string that ends in a nought and lives until
    // the call returns, and `how` is given with its own size, as the call asks.
    let fd = unsafe { libc::syscall(libc::SYS_openat2, dir, name.as_ptr(), &how as *const libc::open_how, std::mem::size_of::<libc::open_how>()) };
    if fd < 0 {
        return Err(std::io::Error::last_os_error());
    }
    // SAFETY: a descriptor the kernel has just given, which nothing else holds.
    Ok(unsafe { std::fs::File::from_raw_fd(fd as i32) })
}

pub fn link_of(fd: RawFd) -> std::io::Result<Vec<u8>> {
    let mut buf = vec![0u8; libc::PATH_MAX as usize];
    // SAFETY: an empty name asks about the opened thing itself; the
    // buffer is owned here and its true length is given.
    let n = unsafe { libc::readlinkat(fd, c"".as_ptr(), buf.as_mut_ptr().cast(), buf.len()) };
    if n < 0 {
        return Err(std::io::Error::last_os_error());
    }
    buf.truncate(n as usize);
    Ok(buf)
}

#[cfg(test)]
pub fn mkfifo(path: &CStr, mode: libc::mode_t) -> libc::c_int {
    // SAFETY: mkfifo only reads the path it is handed, which ends in a nought.
    unsafe { libc::mkfifo(path.as_ptr(), mode) }
}

#[cfg(all(any(test, feature = "testing"), feature = "ui"))]
#[cfg_attr(not(test), allow(dead_code))]
pub fn thread_time() -> std::time::Duration {
    let mut now = libc::timespec { tv_sec: 0, tv_nsec: 0 };
    // SAFETY: `now` is a valid timespec for the call to fill in.
    unsafe { libc::clock_gettime(libc::CLOCK_THREAD_CPUTIME_ID, &mut now) };
    std::time::Duration::new(now.tv_sec as u64, now.tv_nsec as u32)
}
