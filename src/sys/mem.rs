//! Memory shared with another program: an anonymous file mapped into this one.

use std::ffi::{c_void, CStr};
use std::os::fd::{AsFd, AsRawFd, BorrowedFd, FromRawFd, OwnedFd};

pub struct Memory {
    fd: OwnedFd,
    map: *mut c_void,
    len: usize,
}

// SAFETY: the mapping belongs to this value alone and is only reached through
// it, so moving it to another thread moves the only way to touch it.
unsafe impl Send for Memory {}

impl Memory {
    pub fn new(name: &CStr, len: usize) -> Option<Memory> {
        if len == 0 {
            return None;
        }
        // SAFETY: a plain syscall with a NUL-terminated name and no memory of
        // ours involved; it answers with a file descriptor or -1.
        let fd = unsafe { libc::memfd_create(name.as_ptr(), libc::MFD_CLOEXEC | libc::MFD_ALLOW_SEALING) };
        if fd < 0 {
            return None;
        }
        // SAFETY: memfd_create just returned this descriptor and nothing else
        // owns it, so OwnedFd may take it over and close it.
        let fd = unsafe { OwnedFd::from_raw_fd(fd) };
        // SAFETY: acts on that descriptor alone, sizing the file to `len`.
        if unsafe { libc::ftruncate(fd.as_raw_fd(), len as libc::off_t) } != 0 {
            return None;
        }
        // SAFETY: acts on that descriptor alone, adding a seal.
        if unsafe { libc::fcntl(fd.as_raw_fd(), libc::F_ADD_SEALS, libc::F_SEAL_SHRINK) } != 0 {
            return None;
        }
        // SAFETY: `len` bytes of the file just sized to `len`, through a
        // descriptor this value owns. Unmapped once, in Drop.
        let map = unsafe { libc::mmap(std::ptr::null_mut(), len, libc::PROT_READ | libc::PROT_WRITE, libc::MAP_SHARED, fd.as_raw_fd(), 0) };
        if map == libc::MAP_FAILED {
            return None;
        }
        Some(Memory { fd, map, len })
    }

    pub fn fd(&self) -> BorrowedFd<'_> {
        self.fd.as_fd()
    }

    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn bytes(&self) -> &[u8] {
        // SAFETY: `len` bytes mapped readable and writable in `new`, alive
        // until Drop, and only reached through this value, so the borrow
        // checker keeps this slice from outliving or aliasing a mutable one.
        unsafe { std::slice::from_raw_parts(self.map as *const u8, self.len) }
    }

    /// # Safety
    /// Nothing may use `bytes` or `bytes_mut` while the pointer is written through.
    pub unsafe fn as_mut_ptr(&self) -> *mut u8 {
        self.map as *mut u8
    }

    pub fn bytes_mut(&mut self) -> &mut [u8] {
        // SAFETY: as in `bytes`; `&mut self` makes this the only slice.
        unsafe { std::slice::from_raw_parts_mut(self.map as *mut u8, self.len) }
    }
}

impl Drop for Memory {
    fn drop(&mut self) {
        // SAFETY: the address and length mapped in `new`, freed exactly once.
        unsafe { libc::munmap(self.map, self.len) };
    }
}
