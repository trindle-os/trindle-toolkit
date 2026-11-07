//! Pictures passed to and from the compositor through shared memory, unmapped once the compositor has finished with them.

use std::ffi::CStr;
use std::sync::Mutex;

use wayland_client::protocol::{wl_buffer, wl_shm, wl_shm_pool};
use wayland_client::{Dispatch, QueueHandle};

pub use crate::sys::mem::Memory;

pub type Held = Mutex<Option<Memory>>;

pub fn pixmap<D>(shm: &wl_shm::WlShm, qh: &QueueHandle<D>, px: &tiny_skia::Pixmap, name: &CStr) -> Option<wl_buffer::WlBuffer>
where
    D: Dispatch<wl_shm_pool::WlShmPool, ()> + Dispatch<wl_buffer::WlBuffer, Held> + 'static,
{
    let (w, h) = (px.width() as i32, px.height() as i32);
    let mut memory = Memory::new(name, px.data().len())?;
    to_compositor(px.data(), memory.bytes_mut());
    let pool = shm.create_pool(memory.fd(), memory.len() as i32, qh, ());
    let buffer = pool.create_buffer(0, w, h, w * 4, wl_shm::Format::Argb8888, qh, Mutex::new(Some(memory)));
    pool.destroy();
    Some(buffer)
}

fn to_compositor(rgba: &[u8], out: &mut [u8]) {
    for (dst, src) in out.chunks_exact_mut(4).zip(rgba.chunks_exact(4)) {
        dst.copy_from_slice(&[src[2], src[1], src[0], src[3]]);
    }
}

fn on_picture(size: (i32, i32), part: [i32; 4]) -> (usize, usize, usize, usize) {
    let edge = |v: i32, most: i32| v.clamp(0, most.max(0)) as usize;
    (edge(part[0], size.0), edge(part[1], size.1),
     edge(part[0].saturating_add(part[2]), size.0), edge(part[1].saturating_add(part[3]), size.1))
}

pub fn fill(bytes: &mut [u8], size: (i32, i32), part: [i32; 4], colour: tiny_skia::Color) {
    let c = colour.to_color_u8();
    let pixel = [c.blue(), c.green(), c.red(), 255];
    let (left, top, right, bottom) = on_picture(size, part);
    let stride = size.0.max(0) as usize * 4;
    for row in top..bottom {
        let Some(line) = bytes.get_mut(row * stride + left * 4..row * stride + right * 4) else { return };
        for px in line.chunks_exact_mut(4) {
            px.copy_from_slice(&pixel);
        }
    }
}

pub fn put(bytes: &mut [u8], size: (i32, i32), painting: &[u8], part: [i32; 4]) {
    let (left, top, right, bottom) = on_picture(size, part);
    let (stride, from_stride) = (size.0.max(0) as usize * 4, part[2].max(0) as usize * 4);
    if right <= left {
        return;
    }
    for row in top..bottom {
        let from_row = (row as i64 - i64::from(part[1])) as usize;
        let from_left = (left as i64 - i64::from(part[0])).max(0) as usize;
        let from = painting.get(from_row * from_stride + from_left * 4..from_row * from_stride + (from_left + right - left) * 4);
        let to = bytes.get_mut(row * stride + left * 4..row * stride + right * 4);
        let (Some(from), Some(to)) = (from, to) else { return };
        to_compositor(from, to);
    }
}

pub fn released(buffer: &wl_buffer::WlBuffer, frame: &Held) {
    if let Some(memory) = frame.lock().ok().and_then(|mut f| f.take()) {
        drop(memory);
        buffer.destroy();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn picture() -> Vec<u8> {
        vec![9; 4 * 3 * 4]
    }

    #[test]
    fn a_part_is_filled_blue_first_and_no_further() {
        let mut p = picture();
        fill(&mut p, (4, 3), [1, 1, 2, 1], tiny_skia::Color::from_rgba8(10, 20, 30, 255));
        assert_eq!(&p[(4 + 1) * 4..(4 + 3) * 4], &[30, 20, 10, 255, 30, 20, 10, 255]);
        assert_eq!(p.iter().filter(|b| **b == 9).count(), (12 - 2) * 4, "nothing else is touched");
    }

    #[test]
    fn a_painting_is_put_on_with_red_and_blue_swapped() {
        let mut p = picture();
        put(&mut p, (4, 3), &[255, 0, 0, 255, 0, 255, 0, 255], [2, 2, 2, 1]);
        assert_eq!(&p[(8 + 2) * 4..], &[0, 0, 255, 255, 0, 255, 0, 255]);
    }

    #[test]
    fn what_falls_off_the_picture_is_left_out() {
        let mut p = picture();
        put(&mut p, (4, 3), &[1, 2, 3, 4, 5, 6, 7, 8, 11, 12, 13, 14, 15, 16, 17, 18], [-1, -1, 2, 2]);
        assert_eq!(&p[..4], &[17, 16, 15, 18]);
        assert_eq!(p[4..].iter().filter(|b| **b != 9).count(), 0);
        let before = p.clone();
        fill(&mut p, (4, 3), [10, 10, 5, 5], tiny_skia::Color::BLACK);
        fill(&mut p, (4, 3), [-9, -9, 5, 5], tiny_skia::Color::BLACK);
        put(&mut p, (4, 3), &[], [1, 1, 0, 0]);
        put(&mut p, (4, 3), &[0; 20], [10, 0, 5, 1]);
        put(&mut p, (4, 3), &[0; 4], [2, 0, -1, 1]);
        assert_eq!(p, before);
    }
}
