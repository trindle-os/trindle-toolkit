//! What a program and the drawing service say to each other, one message to a packet, read trusting nothing.

use super::wire::{MAX_CONTROL, MAX_PIXEL_BYTES};

pub(crate) const VERSION: u32 = 1;
pub(crate) const SOCKET: &str = "trindle-draw.socket";
pub(crate) const LOCK: &str = "trindle-draw.lock";
pub(crate) const MOST_MESSAGE: usize = 48 << 10;
pub(crate) const MOST_LAYOUTS: usize = 256;
pub(crate) const MOST_FILE: usize = MAX_CONTROL + MAX_PIXEL_BYTES;

const BUFFER: u8 = 1;
const FREE: u8 = 2;
const FRAME: u8 = 3;
const SLEEP: u8 = 4;

const HELLO: u8 = 128;
const MADE: u8 = 129;
const NOT_MADE: u8 = 130;
const DONE: u8 = 131;
const AGAIN: u8 = 132;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Ask<'a> {
    Buffer { id: u64, w: u32, h: u32, layouts: Vec<u64> },
    Free { id: u64 },
    Frame { seq: u64, id: u64, part: [i32; 4], control: u32, pixels: u32, inline: &'a [u8] },
    Sleep,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Tell {
    Hello { version: u32, rdev: u64 },
    Made { id: u64, format: u32, modifier: u64, planes: Vec<(u32, u32)> },
    NotMade { id: u64 },
    Done { seq: u64 },
    Again { seq: u64 },
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Bad;

struct Rd<'a>(&'a [u8]);

impl<'a> Rd<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], Bad> {
        if n > self.0.len() {
            return Err(Bad);
        }
        let (head, rest) = self.0.split_at(n);
        self.0 = rest;
        Ok(head)
    }

    fn u16(&mut self) -> Result<u16, Bad> {
        self.take(2)?.try_into().map(u16::from_le_bytes).map_err(|_| Bad)
    }

    fn u32(&mut self) -> Result<u32, Bad> {
        self.take(4)?.try_into().map(u32::from_le_bytes).map_err(|_| Bad)
    }

    fn u64(&mut self) -> Result<u64, Bad> {
        self.take(8)?.try_into().map(u64::from_le_bytes).map_err(|_| Bad)
    }

    fn i32(&mut self) -> Result<i32, Bad> {
        self.take(4)?.try_into().map(i32::from_le_bytes).map_err(|_| Bad)
    }

    fn end<T>(&self, v: T) -> Result<T, Bad> {
        if self.0.is_empty() { Ok(v) } else { Err(Bad) }
    }
}

pub(crate) fn read_ask(bytes: &[u8]) -> Result<Ask<'_>, Bad> {
    let mut r = Rd(bytes);
    match r.take(1)?[0] {
        BUFFER => {
            let (id, w, h, n) = (r.u64()?, r.u32()?, r.u32()?, r.u16()? as usize);
            if n > MOST_LAYOUTS || n * 8 != r.0.len() {
                return Err(Bad);
            }
            let layouts = (0..n).map(|_| r.u64()).collect::<Result<_, _>>()?;
            r.end(Ask::Buffer { id, w, h, layouts })
        }
        FREE => {
            let id = r.u64()?;
            r.end(Ask::Free { id })
        }
        FRAME => {
            let (seq, id) = (r.u64()?, r.u64()?);
            let part = [r.i32()?, r.i32()?, r.i32()?, r.i32()?];
            let (control, pixels) = (r.u32()?, r.u32()?);
            if control as usize > MAX_CONTROL || pixels as usize > MAX_PIXEL_BYTES {
                return Err(Bad);
            }
            Ok(Ask::Frame { seq, id, part, control, pixels, inline: r.0 })
        }
        SLEEP => r.end(Ask::Sleep),
        _ => Err(Bad),
    }
}

pub(crate) fn read_tell(bytes: &[u8]) -> Result<Tell, Bad> {
    let mut r = Rd(bytes);
    match r.take(1)?[0] {
        HELLO => {
            let (version, rdev) = (r.u32()?, r.u64()?);
            r.end(Tell::Hello { version, rdev })
        }
        MADE => {
            let (id, format, modifier, n) = (r.u64()?, r.u32()?, r.u64()?, r.u16()? as usize);
            if n == 0 || n > crate::sys::sock::MOST_FDS {
                return Err(Bad);
            }
            let planes = (0..n).map(|_| Ok((r.u32()?, r.u32()?))).collect::<Result<_, _>>()?;
            r.end(Tell::Made { id, format, modifier, planes })
        }
        NOT_MADE => {
            let id = r.u64()?;
            r.end(Tell::NotMade { id })
        }
        DONE => {
            let seq = r.u64()?;
            r.end(Tell::Done { seq })
        }
        AGAIN => {
            let seq = r.u64()?;
            r.end(Tell::Again { seq })
        }
        _ => Err(Bad),
    }
}

pub(crate) fn write_ask(ask: &Ask) -> Vec<u8> {
    let mut w = Vec::new();
    match ask {
        Ask::Buffer { id, w: wide, h, layouts } => {
            w.push(BUFFER);
            w.extend_from_slice(&id.to_le_bytes());
            w.extend_from_slice(&wide.to_le_bytes());
            w.extend_from_slice(&h.to_le_bytes());
            w.extend_from_slice(&(layouts.len() as u16).to_le_bytes());
            layouts.iter().for_each(|m| w.extend_from_slice(&m.to_le_bytes()));
        }
        Ask::Free { id } => {
            w.push(FREE);
            w.extend_from_slice(&id.to_le_bytes());
        }
        Ask::Frame { seq, id, part, control, pixels, inline } => {
            w.push(FRAME);
            w.extend_from_slice(&seq.to_le_bytes());
            w.extend_from_slice(&id.to_le_bytes());
            part.iter().for_each(|v| w.extend_from_slice(&v.to_le_bytes()));
            w.extend_from_slice(&control.to_le_bytes());
            w.extend_from_slice(&pixels.to_le_bytes());
            w.extend_from_slice(inline);
        }
        Ask::Sleep => w.push(SLEEP),
    }
    w
}

pub(crate) fn write_tell(tell: &Tell) -> Vec<u8> {
    let mut w = Vec::new();
    match tell {
        Tell::Hello { version, rdev } => {
            w.push(HELLO);
            w.extend_from_slice(&version.to_le_bytes());
            w.extend_from_slice(&rdev.to_le_bytes());
        }
        Tell::Made { id, format, modifier, planes } => {
            w.push(MADE);
            w.extend_from_slice(&id.to_le_bytes());
            w.extend_from_slice(&format.to_le_bytes());
            w.extend_from_slice(&modifier.to_le_bytes());
            w.extend_from_slice(&(planes.len() as u16).to_le_bytes());
            for (stride, offset) in planes {
                w.extend_from_slice(&stride.to_le_bytes());
                w.extend_from_slice(&offset.to_le_bytes());
            }
        }
        Tell::NotMade { id } => {
            w.push(NOT_MADE);
            w.extend_from_slice(&id.to_le_bytes());
        }
        Tell::Done { seq } => {
            w.push(DONE);
            w.extend_from_slice(&seq.to_le_bytes());
        }
        Tell::Again { seq } => {
            w.push(AGAIN);
            w.extend_from_slice(&seq.to_le_bytes());
        }
    }
    w
}

#[cfg(test)]
mod tests {
    use super::*;

    fn asks() -> Vec<Ask<'static>> {
        vec![
            Ask::Buffer { id: 7, w: 640, h: 480, layouts: vec![0, 1 << 56] },
            Ask::Buffer { id: 8, w: 1, h: 1, layouts: Vec::new() },
            Ask::Free { id: 7 },
            Ask::Frame { seq: 3, id: 7, part: [0, -1, 640, 480], control: 3, pixels: 2, inline: b"abcde" },
            Ask::Frame { seq: 4, id: 7, part: [0; 4], control: 1 << 20, pixels: 1 << 24, inline: b"" },
            Ask::Sleep,
        ]
    }

    fn tells() -> Vec<Tell> {
        vec![
            Tell::Hello { version: VERSION, rdev: 57_984 },
            Tell::Made { id: 7, format: 0x3432_5241, modifier: 1 << 56, planes: vec![(2560, 0), (640, 4096)] },
            Tell::NotMade { id: 7 },
            Tell::Done { seq: 9 },
            Tell::Again { seq: 9 },
        ]
    }

    #[test]
    fn every_message_comes_back_as_it_went() {
        for a in asks() {
            assert_eq!(read_ask(&write_ask(&a)), Ok(a));
        }
        for t in tells() {
            assert_eq!(read_tell(&write_tell(&t)), Ok(t));
        }
    }

    #[test]
    fn a_message_cut_short_or_with_more_is_not_one() {
        for a in asks() {
            let bytes = write_ask(&a);
            for n in 0..bytes.len() {
                if !matches!(a, Ask::Frame { .. }) || n < 41 {
                    assert_eq!(read_ask(&bytes[..n]), Err(Bad), "{a:?} cut at {n}");
                }
            }
            if !matches!(a, Ask::Frame { .. }) {
                let mut more = bytes.clone();
                more.push(0);
                assert_eq!(read_ask(&more), Err(Bad), "{a:?} with more");
            }
        }
        for t in tells() {
            let bytes = write_tell(&t);
            for n in 0..bytes.len() {
                assert_eq!(read_tell(&bytes[..n]), Err(Bad), "{t:?} cut at {n}");
            }
            let mut more = bytes.clone();
            more.push(0);
            assert_eq!(read_tell(&more), Err(Bad), "{t:?} with more");
        }
    }

    #[test]
    fn counts_and_lengths_past_the_limits_are_not_a_message() {
        let huge = Ask::Frame { seq: 1, id: 1, part: [0; 4], control: u32::MAX, pixels: 0, inline: b"" };
        assert_eq!(read_ask(&write_ask(&huge)), Err(Bad));
        let huge = Ask::Frame { seq: 1, id: 1, part: [0; 4], control: 0, pixels: u32::MAX, inline: b"" };
        assert_eq!(read_ask(&write_ask(&huge)), Err(Bad));
        let many = Ask::Buffer { id: 1, w: 1, h: 1, layouts: vec![0; MOST_LAYOUTS + 1] };
        assert_eq!(read_ask(&write_ask(&many)), Err(Bad));
        let mut says_many = write_ask(&Ask::Buffer { id: 1, w: 1, h: 1, layouts: vec![0] });
        says_many[17] = 200;
        assert_eq!(read_ask(&says_many), Err(Bad));
        let planes = Tell::Made { id: 1, format: 0, modifier: 0, planes: vec![(0, 0); 5] };
        assert_eq!(read_tell(&write_tell(&planes)), Err(Bad));
    }

    #[test]
    fn random_bytes_never_panic() {
        let mut x = 0x9e37_79b9_7f4a_7c15u64;
        let mut next = move || {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            x
        };
        for n in 0..100_000 {
            let len = next() as usize % 80;
            let mut bytes: Vec<u8> = (0..len).map(|_| next() as u8).collect();
            if n % 2 == 0 && !bytes.is_empty() {
                bytes[0] = [BUFFER, FREE, FRAME, SLEEP, HELLO, MADE, NOT_MADE, DONE, AGAIN][next() as usize % 9];
            }
            let _ = read_ask(&bytes);
            let _ = read_tell(&bytes);
        }
    }
}
