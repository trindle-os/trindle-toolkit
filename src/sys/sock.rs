//! Unix packet sockets that carry descriptors, and sealed memory files, for the drawing service.

use std::ffi::CStr;
use std::io::{Error, ErrorKind, Result};
use std::mem::{size_of, zeroed};
use std::os::fd::{AsRawFd, BorrowedFd, FromRawFd, OwnedFd, RawFd};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

pub const MOST_FDS: usize = 4;

fn checked(r: libc::c_int) -> Result<libc::c_int> {
    if r < 0 { Err(Error::last_os_error()) } else { Ok(r) }
}

fn address(path: &Path) -> Result<(libc::sockaddr_un, libc::socklen_t)> {
    // SAFETY: sockaddr_un is plain numbers, so all zeroes is a valid empty one.
    let mut a: libc::sockaddr_un = unsafe { zeroed() };
    a.sun_family = libc::AF_UNIX as libc::sa_family_t;
    let bytes = path.as_os_str().as_bytes();
    if bytes.len() >= a.sun_path.len() || bytes.contains(&0) {
        return Err(ErrorKind::InvalidInput.into());
    }
    for (to, from) in a.sun_path.iter_mut().zip(bytes) {
        *to = *from as libc::c_char;
    }
    Ok((a, size_of::<libc::sockaddr_un>() as libc::socklen_t))
}

fn packet_socket() -> Result<OwnedFd> {
    // SAFETY: plain numbers; the result is checked before it is owned.
    let fd = checked(unsafe { libc::socket(libc::AF_UNIX, libc::SOCK_SEQPACKET | libc::SOCK_CLOEXEC | libc::SOCK_NONBLOCK, 0) })?;
    // SAFETY: a fresh descriptor nothing else holds.
    Ok(unsafe { OwnedFd::from_raw_fd(fd) })
}

pub fn listen(path: &Path) -> Result<OwnedFd> {
    let s = packet_socket()?;
    let (a, len) = address(path)?;
    // SAFETY: plain numbers, and an address that lives through the call, whose length goes with it. The mask is put back at once.
    // A driver thread making a file in that moment only makes it more private; no other thread here sets the mask.
    let bound = unsafe {
        let was = libc::umask(0o177);
        let bound = libc::bind(s.as_raw_fd(), (&a as *const libc::sockaddr_un).cast(), len);
        libc::umask(was);
        bound
    };
    checked(bound)?;
    // SAFETY: plain numbers on a descriptor held here.
    checked(unsafe { libc::listen(s.as_raw_fd(), 32) })?;
    Ok(s)
}

pub fn connect(path: &Path) -> Result<OwnedFd> {
    let s = packet_socket()?;
    let (a, len) = address(path)?;
    // SAFETY: an address that lives through the call, whose length goes with it.
    checked(unsafe { libc::connect(s.as_raw_fd(), (&a as *const libc::sockaddr_un).cast(), len) })?;
    Ok(s)
}

pub fn accept(fd: RawFd) -> Result<OwnedFd> {
    // SAFETY: no address is asked for; the result is checked before it is owned.
    let got = checked(unsafe { libc::accept4(fd, std::ptr::null_mut(), std::ptr::null_mut(), libc::SOCK_CLOEXEC | libc::SOCK_NONBLOCK) })?;
    // SAFETY: a fresh descriptor nothing else holds.
    Ok(unsafe { OwnedFd::from_raw_fd(got) })
}

pub fn open_files_up() -> u64 {
    // SAFETY: rlimit is plain numbers, so all zeroes is valid; the kernel fills it, and reads it back, through a live reference.
    unsafe {
        let mut limit: libc::rlimit = zeroed();
        if libc::getrlimit(libc::RLIMIT_NOFILE, &mut limit) != 0 {
            return 0;
        }
        limit.rlim_cur = limit.rlim_max;
        if libc::setrlimit(libc::RLIMIT_NOFILE, &limit) != 0 {
            libc::getrlimit(libc::RLIMIT_NOFILE, &mut limit);
        }
        limit.rlim_cur
    }
}

pub fn peer_uid(fd: RawFd) -> Option<u32> {
    // SAFETY: ucred is plain numbers, so all zeroes is valid; the kernel fills it, its length told.
    unsafe {
        let mut who: libc::ucred = zeroed();
        let mut len = size_of::<libc::ucred>() as libc::socklen_t;
        let r = libc::getsockopt(fd, libc::SOL_SOCKET, libc::SO_PEERCRED, (&mut who as *mut libc::ucred).cast(), &mut len);
        (r == 0 && len as usize == size_of::<libc::ucred>()).then_some(who.uid)
    }
}

pub fn shut(fd: RawFd) {
    // SAFETY: plain numbers; a descriptor not a socket is refused.
    unsafe { libc::shutdown(fd, libc::SHUT_RDWR) };
}

#[derive(Debug, PartialEq, Eq)]
pub enum Got {
    Nothing,
    Ended,
    Message { len: usize, cut: bool },
}

pub fn recv(fd: RawFd, buf: &mut [u8], fds: &mut Vec<OwnedFd>) -> Got {
    #[repr(align(8))]
    struct Room([u8; 128]);
    let mut room = Room([0; 128]);
    let mut io = libc::iovec { iov_base: buf.as_mut_ptr().cast(), iov_len: buf.len() };
    // SAFETY: msghdr is plain numbers and pointers, so all zeroes is valid; the kernel writes only into the buffer and the
    // room named, each with its length. Each descriptor it hands over is new and owned from here on; walking the headers
    // through the libc macros stays inside `room` for the length the kernel left in msg_controllen.
    unsafe {
        let mut m: libc::msghdr = zeroed();
        m.msg_iov = &mut io;
        m.msg_iovlen = 1;
        m.msg_control = room.0.as_mut_ptr().cast();
        m.msg_controllen = room.0.len() as _;
        let n = libc::recvmsg(fd, &mut m, libc::MSG_DONTWAIT | libc::MSG_CMSG_CLOEXEC);
        if n < 0 {
            let e = Error::last_os_error();
            return if matches!(e.kind(), ErrorKind::WouldBlock | ErrorKind::Interrupted) { Got::Nothing } else { Got::Ended };
        }
        let mut c = libc::CMSG_FIRSTHDR(&m);
        while !c.is_null() {
            if (*c).cmsg_level == libc::SOL_SOCKET && (*c).cmsg_type == libc::SCM_RIGHTS {
                let bytes = ((*c).cmsg_len as usize).saturating_sub(libc::CMSG_LEN(0) as usize);
                let at = libc::CMSG_DATA(c).cast::<RawFd>();
                for i in 0..bytes / size_of::<RawFd>() {
                    fds.push(OwnedFd::from_raw_fd(at.add(i).read_unaligned()));
                }
            }
            c = libc::CMSG_NXTHDR(&m, c);
        }
        let cut = m.msg_flags & (libc::MSG_TRUNC | libc::MSG_CTRUNC) != 0;
        if n == 0 && fds.is_empty() {
            return Got::Ended;
        }
        Got::Message { len: n as usize, cut }
    }
}

pub fn send(fd: RawFd, bytes: &[u8], fds: &[BorrowedFd]) -> Result<()> {
    if fds.len() > MOST_FDS {
        return Err(ErrorKind::InvalidInput.into());
    }
    #[repr(align(8))]
    struct Room([u8; 128]);
    let mut room = Room([0; 128]);
    let mut io = libc::iovec { iov_base: bytes.as_ptr().cast_mut().cast(), iov_len: bytes.len() };
    // SAFETY: msghdr is plain numbers and pointers, so all zeroes is valid. The kernel only reads the bytes through the iovec, and
    // the control message is written inside `room`, which holds MOST_FDS descriptors; the descriptors stay open through the call.
    let sent = unsafe {
        let mut m: libc::msghdr = zeroed();
        m.msg_iov = &mut io;
        m.msg_iovlen = 1;
        if !fds.is_empty() {
            let len = fds.len() * size_of::<RawFd>();
            m.msg_control = room.0.as_mut_ptr().cast();
            m.msg_controllen = libc::CMSG_SPACE(len as u32) as _;
            let c = libc::CMSG_FIRSTHDR(&m);
            (*c).cmsg_level = libc::SOL_SOCKET;
            (*c).cmsg_type = libc::SCM_RIGHTS;
            (*c).cmsg_len = libc::CMSG_LEN(len as u32) as _;
            let at = libc::CMSG_DATA(c).cast::<RawFd>();
            for (i, f) in fds.iter().enumerate() {
                at.add(i).write_unaligned(f.as_raw_fd());
            }
        }
        libc::sendmsg(fd, &m, libc::MSG_DONTWAIT | libc::MSG_NOSIGNAL)
    };
    if sent < 0 {
        return Err(Error::last_os_error());
    }
    if sent as usize != bytes.len() {
        return Err(ErrorKind::WriteZero.into());
    }
    Ok(())
}

pub fn sealed(name: &CStr, parts: &[&[u8]]) -> Result<OwnedFd> {
    // SAFETY: a NUL-terminated name and plain flags; the result is checked before it is owned.
    let fd = checked(unsafe { libc::memfd_create(name.as_ptr(), libc::MFD_CLOEXEC | libc::MFD_ALLOW_SEALING) })?;
    // SAFETY: a fresh descriptor nothing else holds.
    let fd = unsafe { OwnedFd::from_raw_fd(fd) };
    for part in parts {
        let mut left = *part;
        while !left.is_empty() {
            // SAFETY: the kernel reads at most `left.len()` bytes of a live slice.
            let n = unsafe { libc::write(fd.as_raw_fd(), left.as_ptr().cast(), left.len()) };
            if n <= 0 {
                let e = Error::last_os_error();
                if e.kind() == ErrorKind::Interrupted {
                    continue;
                }
                return Err(e);
            }
            left = &left[n as usize..];
        }
    }
    // SAFETY: plain numbers on the descriptor owned here. No mapping of it exists, so the write seal is granted.
    checked(unsafe { libc::fcntl(fd.as_raw_fd(), libc::F_ADD_SEALS, SEALS | libc::F_SEAL_SEAL) })?;
    Ok(fd)
}

const SEALS: libc::c_int = libc::F_SEAL_WRITE | libc::F_SEAL_SHRINK | libc::F_SEAL_GROW;

pub struct Sealed {
    at: *mut libc::c_void,
    len: usize,
}

impl Sealed {
    pub fn map(fd: BorrowedFd, most: usize) -> Option<Sealed> {
        // SAFETY: plain numbers; a descriptor that is not a memory file is refused.
        let seals = unsafe { libc::fcntl(fd.as_raw_fd(), libc::F_GET_SEALS) };
        if seals < 0 || seals & SEALS != SEALS {
            return None;
        }
        // SAFETY: stat is plain numbers, so all zeroes is valid; the kernel fills it.
        let st = unsafe {
            let mut st: libc::stat = zeroed();
            (libc::fstat(fd.as_raw_fd(), &mut st) == 0).then_some(st)
        }?;
        let len = usize::try_from(st.st_size).ok().filter(|l| *l <= most)?;
        if len == 0 {
            return Some(Sealed { at: std::ptr::null_mut(), len: 0 });
        }
        // SAFETY: `len` bytes of a file sealed at that length, read only and private; unmapped once, in Drop.
        let at = unsafe { libc::mmap(std::ptr::null_mut(), len, libc::PROT_READ, libc::MAP_PRIVATE, fd.as_raw_fd(), 0) };
        (at != libc::MAP_FAILED).then_some(Sealed { at, len })
    }

    pub fn bytes(&self) -> &[u8] {
        if self.len == 0 {
            return &[];
        }
        // SAFETY: `len` bytes mapped for this value's life, and the seals checked in `map` mean they never change.
        unsafe { std::slice::from_raw_parts(self.at.cast::<u8>(), self.len) }
    }
}

impl Drop for Sealed {
    fn drop(&mut self) {
        if self.len > 0 {
            // SAFETY: this value's own mapping; no slice of it outlives the borrow of the value.
            unsafe { libc::munmap(self.at, self.len) };
        }
    }
}

pub fn try_lock(fd: RawFd) -> bool {
    // SAFETY: no pointers.
    unsafe { libc::flock(fd, libc::LOCK_EX | libc::LOCK_NB) == 0 }
}

pub fn private_folder(path: &Path) -> bool {
    use std::os::unix::fs::MetadataExt;
    // SAFETY: no pointers; it cannot fail.
    let me = unsafe { libc::getuid() };
    std::fs::metadata(path).is_ok_and(|m| m.is_dir() && m.uid() == me && m.mode() & 0o077 == 0)
}

#[cfg(test)]
pub fn limit_data(limit: &libc::rlimit) -> bool {
    // SAFETY: the kernel only reads the struct.
    unsafe { libc::setrlimit(libc::RLIMIT_DATA, limit) == 0 }
}

pub fn own_uid() -> u32 {
    // SAFETY: no pointers; it cannot fail.
    unsafe { libc::getuid() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::fd::AsFd;

    fn pair() -> (OwnedFd, OwnedFd) {
        let mut ends = [0; 2];
        // SAFETY: socketpair writes two descriptors into an array of two.
        assert_eq!(unsafe { libc::socketpair(libc::AF_UNIX, libc::SOCK_SEQPACKET | libc::SOCK_CLOEXEC | libc::SOCK_NONBLOCK, 0, ends.as_mut_ptr()) }, 0);
        // SAFETY: both fresh, and nothing else holds them.
        unsafe { (OwnedFd::from_raw_fd(ends[0]), OwnedFd::from_raw_fd(ends[1])) }
    }

    #[test]
    fn a_message_and_its_descriptors_go_across_whole() {
        let (a, b) = pair();
        let file = sealed(c"test", &[b"hello ", b"there"]).unwrap();
        send(a.as_raw_fd(), b"one", &[file.as_fd()]).unwrap();
        let (mut buf, mut fds) = ([0; 16], Vec::new());
        assert_eq!(recv(b.as_raw_fd(), &mut buf, &mut fds), Got::Message { len: 3, cut: false });
        assert_eq!(&buf[..3], b"one");
        assert_eq!(fds.len(), 1);
        let map = Sealed::map(fds[0].as_fd(), 100).unwrap();
        assert_eq!(map.bytes(), b"hello there");
        assert_eq!(recv(b.as_raw_fd(), &mut buf, &mut fds), Got::Nothing);
    }

    #[test]
    fn a_message_too_long_is_told_as_cut_and_a_gone_peer_as_ended() {
        let (a, b) = pair();
        send(a.as_raw_fd(), &[7; 40], &[]).unwrap();
        let (mut buf, mut fds) = ([0; 16], Vec::new());
        assert_eq!(recv(b.as_raw_fd(), &mut buf, &mut fds), Got::Message { len: 16, cut: true });
        drop(a);
        assert_eq!(recv(b.as_raw_fd(), &mut buf, &mut fds), Got::Ended);
        assert!(send(b.as_raw_fd(), b"x", &[]).is_err(), "no signal, an error");
    }

    #[test]
    fn a_file_not_sealed_against_change_is_never_mapped() {
        // SAFETY: a NUL-terminated name and plain flags.
        let plain = unsafe { libc::memfd_create(c"plain".as_ptr(), libc::MFD_CLOEXEC | libc::MFD_ALLOW_SEALING) };
        // SAFETY: fresh, and nothing else holds it.
        let plain = unsafe { OwnedFd::from_raw_fd(plain) };
        assert!(Sealed::map(plain.as_fd(), 100).is_none(), "no seals");
        // SAFETY: plain numbers.
        assert_eq!(unsafe { libc::fcntl(plain.as_raw_fd(), libc::F_ADD_SEALS, libc::F_SEAL_SHRINK | libc::F_SEAL_GROW) }, 0);
        assert!(Sealed::map(plain.as_fd(), 100).is_none(), "still writable");
        let not_memory = std::fs::File::open("/proc/self/stat").unwrap();
        assert!(Sealed::map(not_memory.as_fd(), 100).is_none(), "not a memory file");
        let (a, _b) = pair();
        assert!(Sealed::map(a.as_fd(), 100).is_none(), "a socket");
        let big = sealed(c"big", &[&[0; 200]]).unwrap();
        assert!(Sealed::map(big.as_fd(), 100).is_none(), "too long");
        // SAFETY: the kernel reads one byte of a live slice; the seal must refuse it.
        let wrote = unsafe { libc::write(big.as_raw_fd(), b"x".as_ptr().cast(), 1) };
        assert!(wrote < 0, "a sealed file takes no writing");
    }

    #[test]
    fn the_socket_file_is_made_private() {
        use std::os::unix::fs::PermissionsExt;
        let dir = std::env::temp_dir().join(format!("trindle-sock-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("s");
        let _ = std::fs::remove_file(&path);
        let l = listen(&path).unwrap();
        assert_eq!(std::fs::metadata(&path).unwrap().permissions().mode() & 0o777, 0o600);
        let c = connect(&path).unwrap();
        let got = accept(l.as_raw_fd()).unwrap();
        assert_eq!(peer_uid(got.as_raw_fd()), Some(own_uid()));
        assert_eq!(peer_uid(c.as_raw_fd()), Some(own_uid()));
        assert!(accept(l.as_raw_fd()).is_err_and(|e| e.kind() == ErrorKind::WouldBlock));
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
