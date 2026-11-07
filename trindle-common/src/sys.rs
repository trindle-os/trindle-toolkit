//! This crate's one call into the kernel that Rust cannot check; no other file holds `unsafe`.

use std::os::fd::RawFd;

pub fn lock(fd: RawFd) -> libc::c_int {
    // SAFETY: no pointers; the hold ends when the descriptor closes.
    unsafe { libc::flock(fd, libc::LOCK_EX) }
}
