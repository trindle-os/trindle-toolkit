//! Which presses were the person's own: the compositor writes each real press's serial to a pipe before passing it on.

use std::collections::VecDeque;
use std::io::{ErrorKind, Read, Write as _};

pub const NAME: &str = "TRINDLE_RING_HAND";

pub const NONE: &str = "none";

const KEPT: usize = 64;

const MOST_READ: usize = 128 * 1024;

pub fn write(pipe: &mut std::fs::File, serial: u32) -> bool {
    loop {
        match pipe.write(&serial.to_le_bytes()) {
            Ok(_) => return true,
            Err(e) if e.kind() == ErrorKind::Interrupted => {}
            Err(e) if e.kind() == ErrorKind::WouldBlock => return true,
            Err(_) => return false,
        }
    }
}

#[derive(Default)]
pub struct Seen {
    newest: VecDeque<u32>,
    part: Vec<u8>,
}

impl Seen {
    pub fn read(&mut self, pipe: &mut impl Read) {
        let mut bytes = [0u8; 4096];
        let mut read = 0;
        while read < MOST_READ {
            match pipe.read(&mut bytes) {
                Ok(0) => break,
                Ok(n) => {
                    read += n;
                    self.part.extend_from_slice(&bytes[..n]);
                    let whole = self.part.len() / 4 * 4;
                    for four in self.part[..whole].chunks_exact(4) {
                        self.newest.push_back(u32::from_le_bytes(four.try_into().expect("chunks of four")));
                    }
                    self.part.drain(..whole);
                    self.trim();
                }
                Err(e) if e.kind() == ErrorKind::Interrupted => {}
                Err(_) => break,
            }
        }
    }

    pub fn note(&mut self, serial: u32) {
        self.newest.push_back(serial);
        self.trim();
    }

    fn trim(&mut self) {
        let over = self.newest.len().saturating_sub(KEPT);
        self.newest.drain(..over);
    }

    pub fn has(&self, serial: u32) -> bool {
        self.newest.contains(&serial)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pipe() -> (std::fs::File, std::fs::File) {
        let (read, write) = crate::sys::fd::pipe(libc::O_CLOEXEC | libc::O_NONBLOCK).unwrap();
        (read.into(), write.into())
    }

    #[test]
    fn only_a_number_the_compositor_wrote_is_the_persons_own() {
        let (mut read, mut wrote) = pipe();
        let mut seen = Seen::default();
        seen.read(&mut read);
        assert!(!seen.has(7), "a press was the person's with nothing written");
        assert!(write(&mut wrote, 7) && write(&mut wrote, 9));
        assert!(!seen.has(7), "a number counted before it was read");
        seen.read(&mut read);
        assert!(seen.has(7) && seen.has(9));
        assert!(!seen.has(8) && !seen.has(0), "a number never written counted");
    }

    #[test]
    fn a_number_that_arrives_in_two_parts_is_read_whole() {
        let mut seen = Seen::default();
        let bytes = 0x0102_0304u32.to_le_bytes();
        seen.read(&mut &bytes[..3]);
        assert!(!seen.has(0x0102_0304));
        seen.read(&mut &bytes[3..]);
        assert!(seen.has(0x0102_0304));
    }

    #[test]
    fn only_the_newest_are_kept_and_a_full_pipe_is_never_waited_on() {
        let (mut read, mut wrote) = pipe();
        for serial in 0..40_000u32 {
            assert!(write(&mut wrote, serial), "a full pipe was taken for one nobody reads");
        }
        let mut seen = Seen::default();
        seen.read(&mut read);
        assert!(seen.newest.len() <= KEPT);
        assert!(!seen.has(39_999), "a number refused by a full pipe counted");
        assert!(write(&mut wrote, 50_000));
        seen.read(&mut read);
        assert!(seen.has(50_000));
        drop(read);
        assert!(!write(&mut wrote, 1), "a pipe nobody reads was written to");
    }
}
