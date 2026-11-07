//! Signals: a handler that writes each one caught to every loop's pipe, and putting back what was there before.

use std::os::fd::{IntoRawFd, RawFd};
use std::sync::atomic::{AtomicI32, AtomicUsize, Ordering};

static HANDLING: AtomicUsize = AtomicUsize::new(0);
static WRITERS: [AtomicI32; 8] = [const { AtomicI32::new(-1) }; 8];

extern "C" fn on_signal(signal: libc::c_int) {
    // SAFETY: errno and write are safe in a signal handler; errno is put
    // back so the code the signal interrupted sees its own.
    unsafe {
        let errno = *libc::__errno_location();
        let byte = signal as u8;
        HANDLING.fetch_add(1, Ordering::SeqCst);
        for w in &WRITERS {
            let fd = w.load(Ordering::SeqCst);
            if fd >= 0 {
                libc::write(fd, (&byte as *const u8).cast(), 1);
            }
        }
        HANDLING.fetch_sub(1, Ordering::SeqCst);
        *libc::__errno_location() = errno;
    }
}

pub fn open_pipe() -> Option<(RawFd, usize)> {
    let (read, write) = super::fd::pipe(libc::O_CLOEXEC | libc::O_NONBLOCK)?;
    let write = write.into_raw_fd();
    let Some(slot) = WRITERS.iter().position(|w| w.compare_exchange(-1, write, Ordering::SeqCst, Ordering::SeqCst).is_ok()) else {
        // SAFETY: the write end was just made here and is not shared; the
        // read end closes as it drops.
        unsafe { libc::close(write) };
        return None;
    };
    Some((read.into_raw_fd(), slot))
}

pub fn close_pipe(read: RawFd, slot: usize) {
    let write = WRITERS[slot].swap(-1, Ordering::SeqCst);
    while HANDLING.load(Ordering::SeqCst) > 0 {
        std::thread::yield_now();
    }
    // SAFETY: both ends were opened by `open_pipe` for one loop alone, and
    // the handler no longer sees the write end.
    unsafe {
        libc::close(read);
        libc::close(write);
    }
}

pub fn catch(signal: libc::c_int) -> Option<libc::sigaction> {
    // SAFETY: a zeroed sigaction is a valid empty one; the handler only
    // writes one byte to each pipe, which is safe in a signal handler.
    unsafe {
        let mut act: libc::sigaction = std::mem::zeroed();
        act.sa_sigaction = on_signal as extern "C" fn(libc::c_int) as usize;
        act.sa_flags = libc::SA_RESTART;
        libc::sigemptyset(&mut act.sa_mask);
        let mut old: libc::sigaction = std::mem::zeroed();
        (libc::sigaction(signal, &act, &mut old) == 0).then_some(old)
    }
}

pub fn restore(signal: libc::c_int, old: &libc::sigaction) {
    // SAFETY: `old` is the action sigaction gave back when this signal was first caught.
    unsafe { libc::sigaction(signal, old, std::ptr::null_mut()) };
}

#[cfg(test)]
pub fn handler() -> usize {
    on_signal as extern "C" fn(libc::c_int) as usize
}

#[cfg(test)]
pub fn action(signal: libc::c_int) -> usize {
    // SAFETY: a null new action only reads the one in place into `now`.
    unsafe {
        let mut now: libc::sigaction = std::mem::zeroed();
        libc::sigaction(signal, std::ptr::null(), &mut now);
        now.sa_sigaction
    }
}

#[cfg(test)]
pub fn raise(signal: libc::c_int) {
    // SAFETY: no pointers; the caller has the signal caught.
    unsafe { libc::raise(signal) };
}
