//! Descriptors: waiting on them, pipes, their flags, reading, locks and folder watches.

use std::ffi::CStr;
use std::os::fd::{FromRawFd, OwnedFd, RawFd};

pub fn poll(fds: &mut [libc::pollfd], timeout: libc::c_int) -> libc::c_int {
    // SAFETY: a live slice of `fds.len()` pollfds, of which poll only
    // writes the `revents`; a closed descriptor is reported, not touched.
    unsafe { libc::poll(fds.as_mut_ptr(), fds.len() as libc::nfds_t, timeout) }
}

pub fn pipe(flags: libc::c_int) -> Option<(OwnedFd, OwnedFd)> {
    let mut ends = [0; 2];
    // SAFETY: pipe2 writes two descriptors into an array of two.
    if unsafe { libc::pipe2(ends.as_mut_ptr(), flags) } != 0 {
        return None;
    }
    // SAFETY: both ends are fresh from pipe2 and nothing else holds them.
    Some(unsafe { (OwnedFd::from_raw_fd(ends[0]), OwnedFd::from_raw_fd(ends[1])) })
}

pub fn flags(fd: RawFd) -> libc::c_int {
    // SAFETY: F_GETFL takes no pointer; a descriptor not open is refused.
    unsafe { libc::fcntl(fd, libc::F_GETFL) }
}

pub fn set_flags(fd: RawFd, flags: libc::c_int) -> libc::c_int {
    // SAFETY: F_SETFL takes plain numbers; a descriptor not open is refused.
    unsafe { libc::fcntl(fd, libc::F_SETFL, flags) }
}

pub fn unread(fd: RawFd) -> Option<libc::c_int> {
    let mut left: libc::c_int = 0;
    // SAFETY: FIONREAD writes one int into `left`, owned here; on a file
    // that cannot say, it fails and writes nothing.
    (unsafe { libc::ioctl(fd, libc::FIONREAD, &mut left) } == 0).then_some(left)
}

pub fn read(fd: RawFd, buf: &mut [u8]) -> isize {
    // SAFETY: the kernel writes at most `buf.len()` bytes into `buf`.
    unsafe { libc::read(fd, buf.as_mut_ptr().cast(), buf.len()) }
}

pub fn lock(fd: RawFd) -> libc::c_int {
    // SAFETY: no pointers; the hold ends when the descriptor closes.
    unsafe { libc::flock(fd, libc::LOCK_EX) }
}

pub fn watch(flags: libc::c_int) -> std::io::Result<OwnedFd> {
    // SAFETY: no pointers; the result is checked before it is owned.
    let fd = unsafe { libc::inotify_init1(flags) };
    if fd < 0 {
        return Err(std::io::Error::last_os_error());
    }
    // SAFETY: a fresh descriptor nothing else owns.
    Ok(unsafe { OwnedFd::from_raw_fd(fd) })
}

pub fn mark(fd: RawFd, path: &CStr, mask: u32) -> libc::c_int {
    // SAFETY: a NUL-terminated path that lives through the call.
    unsafe { libc::inotify_add_watch(fd, path.as_ptr(), mask) }
}

pub fn unmark(fd: RawFd, wd: libc::c_int) {
    // SAFETY: no pointers.
    unsafe { libc::inotify_rm_watch(fd, wd) };
}

pub fn watch_event(bytes: &[u8]) -> Option<libc::inotify_event> {
    if bytes.len() < std::mem::size_of::<libc::inotify_event>() {
        return None;
    }
    // SAFETY: there are bytes enough for a whole header, and inotify_event
    // is plain numbers; it may not be aligned, so it is copied out.
    Some(unsafe { std::ptr::read_unaligned(bytes.as_ptr().cast()) })
}
