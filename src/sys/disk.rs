//! The system calls the file work makes: names within an opened folder, how a drive and the kernel's store are asked, and the gentle priorities.

use std::ffi::CStr;
use std::fs::File;
use std::io;
use std::os::fd::AsRawFd;

const IOPRIO_WHO_PROCESS: libc::c_int = 1;
pub const IOPRIO_CLASS_IDLE: libc::c_int = 3;
const IOPRIO_CLASS_SHIFT: libc::c_int = 13;
pub const LOWEST: libc::c_int = 19;

#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
const SYS_CACHESTAT: libc::c_long = 451;

fn said(done: libc::c_int) -> io::Result<()> {
    if done == 0 { Ok(()) } else { Err(io::Error::last_os_error()) }
}

pub fn uid() -> u32 {
    // SAFETY: getuid takes nothing and cannot fail.
    unsafe { libc::getuid() }
}

pub fn local_time() -> libc::tm {
    // SAFETY: time with a null pointer only answers; localtime_r writes
    // into the zeroed tm we own and is safe from any thread.
    unsafe {
        let t = libc::time(std::ptr::null_mut());
        let mut tm: libc::tm = std::mem::zeroed();
        libc::localtime_r(&t, &mut tm);
        tm
    }
}

pub fn random(bytes: &mut [u8]) {
    // SAFETY: the kernel writes at most `bytes.len()` bytes into `bytes`,
    // which is that long and lives until the call returns.
    unsafe { libc::getrandom(bytes.as_mut_ptr().cast(), bytes.len(), 0) };
}

pub fn rename_in(dir: &File, from: &CStr, to: &CStr) -> io::Result<()> {
    // SAFETY: both names are strings that end in a nought, kept alive
    // until the call returns, and the folder's descriptor is kept open
    // by `dir`.
    said(unsafe { libc::renameat(dir.as_raw_fd(), from.as_ptr(), dir.as_raw_fd(), to.as_ptr()) })
}

pub fn remove_in(dir: &File, name: &CStr, folder: bool) -> io::Result<()> {
    // SAFETY: the name is a string that ends in a nought, kept alive
    // until the call returns, and the folder's descriptor is kept open
    // by `dir`.
    said(unsafe { libc::unlinkat(dir.as_raw_fd(), name.as_ptr(), if folder { libc::AT_REMOVEDIR } else { 0 }) })
}

pub fn can_read(path: &CStr) -> bool {
    // SAFETY: the path is a string that ends in a nought, kept alive
    // until the call returns, and the call only asks about it.
    unsafe { libc::access(path.as_ptr(), libc::R_OK) == 0 }
}

pub fn last_in_line() {
    // SAFETY: two plain system calls that act on the calling thread
    // alone (0 names it) and are passed numbers only.
    unsafe {
        libc::setpriority(libc::PRIO_PROCESS, 0, LOWEST);
        libc::syscall(libc::SYS_ioprio_set, IOPRIO_WHO_PROCESS, 0, IOPRIO_CLASS_IDLE << IOPRIO_CLASS_SHIFT);
    }
}

pub fn start_last_in_line(cmd: &mut std::process::Command) -> &mut std::process::Command {
    use std::os::unix::process::CommandExt;
    // SAFETY: run in the new process before the program starts, it makes
    // only the two system calls of `last_in_line`, which take numbers
    // alone, touch no memory and are safe after a fork.
    unsafe {
        cmd.pre_exec(|| {
            last_in_line();
            Ok(())
        })
    }
}

pub fn start_writing_out(file: &File, from: u64, len: u64) {
    // SAFETY: a system call on a file that is open, passed numbers.
    unsafe { libc::sync_file_range(file.as_raw_fd(), from as i64, len as i64, libc::SYNC_FILE_RANGE_WRITE) };
}

pub fn wait_written_out(file: &File, from: u64, len: u64) -> io::Result<()> {
    let all = libc::SYNC_FILE_RANGE_WAIT_BEFORE | libc::SYNC_FILE_RANGE_WRITE | libc::SYNC_FILE_RANGE_WAIT_AFTER;
    // SAFETY: a system call on a file that is open, passed numbers.
    said(unsafe { libc::sync_file_range(file.as_raw_fd(), from as i64, len as i64, all) })
}

pub fn need_not_keep(file: &File, from: u64, len: u64) {
    // SAFETY: advice on a file that is open; the file is unchanged.
    unsafe { libc::posix_fadvise(file.as_raw_fd(), from as i64, len as i64, libc::POSIX_FADV_DONTNEED) };
}

pub fn will_read_through(file: &File) {
    // SAFETY: advice on a file that is open; the file is unchanged.
    unsafe { libc::posix_fadvise(file.as_raw_fd(), 0, 0, libc::POSIX_FADV_SEQUENTIAL) };
}

pub fn all_written_out(dir: &File) -> io::Result<()> {
    // SAFETY: a system call on a folder that is open.
    said(unsafe { libc::syncfs(dir.as_raw_fd()) })
}

pub fn page_size() -> Option<u64> {
    // SAFETY: asks the system one number and changes nothing.
    u64::try_from(unsafe { libc::sysconf(libc::_SC_PAGESIZE) }).ok().filter(|p| *p > 0)
}

pub fn in_memory(file: &File, from: u64, len: u64) -> Option<bool> {
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    {
        let range = [from, len];
        let mut told = [0u64; 5];
        // SAFETY: the call reads two numbers from `range` and writes five
        // into `told`, which are that long and live until it returns.
        let done = unsafe { libc::syscall(SYS_CACHESTAT, file.as_raw_fd(), range.as_ptr(), told.as_mut_ptr(), 0u32) };
        (done == 0).then_some(told[0] > 0)
    }
    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    {
        let _ = (file, from, len);
        None
    }
}

#[cfg(test)]
pub fn priorities() -> (i32, i64) {
    // SAFETY: two calls that read the calling thread's own priorities and
    // are passed numbers only.
    unsafe { (libc::getpriority(libc::PRIO_PROCESS, 0), libc::syscall(libc::SYS_ioprio_get, IOPRIO_WHO_PROCESS, 0) >> IOPRIO_CLASS_SHIFT) }
}

#[cfg(test)]
pub fn read_a_page_round_the_store(file: &File) {
    // SAFETY: a page of memory that starts on a page's edge, as a read
    // that goes round the kernel's store must be given, freed below and
    // used nowhere else.
    unsafe {
        let page = libc::mmap(std::ptr::null_mut(), 4096, libc::PROT_READ | libc::PROT_WRITE, libc::MAP_PRIVATE | libc::MAP_ANONYMOUS, -1, 0);
        let _ = libc::read(file.as_raw_fd(), page, 4096);
        libc::munmap(page, 4096);
    }
}
