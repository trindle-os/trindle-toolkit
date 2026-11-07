//! Painting a window into its two buffers and handing the frame to the compositor.

use super::*;

impl Display {
    pub(super) fn paint(&self) {
        if self.painting.replace(true) {
            return;
        }
        let qh = self.queue.borrow().handle();
        self.paint_gpu(&qh);
        let mut done = Vec::new();
        self.patch_windows(&qh, &mut done);
        done.clear();
        self.paint_windows(&qh, &mut done);
        self.painting.set(false);
        let _ = self.conn.flush();
    }

    fn patch_windows(&self, qh: &QueueHandle<State>, done: &mut Vec<u64>) {
        while let Some((id, slot, from, behind, mut buffers, mut patcher, scale)) = self.next_to_patch(done) {
            let (w, h) = buffers.size;
            let len = (w * h * 4) as usize;
            let memory = buffers.memory.bytes_mut();
            if let Some(from) = from {
                memory.copy_within(from * len..(from + 1) * len, slot * len);
            }
            for part in &behind {
                catch_up(memory, (w, h), 1 - slot, slot, *part);
            }
            let parts = PixmapMut::from_bytes(&mut memory[slot * len..(slot + 1) * len], w, h).and_then(|mut c| patcher(&mut c, scale));
            let mut state = self.state.borrow_mut();
            if let Some(win) = state.win(id) {
                win.patcher.get_or_insert(patcher);
                win.drawn[slot] = win.turn;
                if let (true, Some(parts)) = (win.moves, &parts) {
                    let other = &mut win.behind[1 - slot];
                    other.extend(parts);
                    if other.len() > BEHIND_MOST {
                        other.clear();
                        win.drawn[1 - slot] = 0;
                    }
                }
                match parts {
                    Some(parts) if !parts.is_empty() => show(win, &buffers.buffers[slot], buffers.size, slot, qh, Some(&parts)),
                    Some(_) => {}
                    None => win.dirty = true,
                }
                win.buffers = Some(buffers);
            } else {
                buffers.buffers.iter().for_each(|b| b.destroy());
            }
            done.push(id);
        }
    }

    fn paint_windows(&self, qh: &QueueHandle<State>, done: &mut Vec<u64>) {
        while let Some((id, slot, mut buffers, mut paint, scale)) = self.next_to_paint(qh, done) {
            let (w, h) = buffers.size;
            let len = (w * h * 4) as usize;
            let bytes = &mut buffers.memory.bytes_mut()[slot * len..(slot + 1) * len];
            bytes.fill(0);
            if let Some(mut canvas) = PixmapMut::from_bytes(bytes, w, h) {
                paint(&mut canvas, scale);
            }
            to_compositor(bytes);
            let opaque = bytes.chunks_exact(4).all(|p| p[3] == 255);
            let mut state = self.state.borrow_mut();
            let compositor = state.compositor.clone();
            if let Some(win) = state.win(id) {
                win.paint.get_or_insert(paint);
                let parts = (win.turn > 0 && win.drawn[1 - slot] == win.turn && win.drawn[slot] != win.turn).then(|| changed_parts(buffers.memory.bytes(), buffers.size, slot, 1 - slot));
                win.turn += 1;
                win.drawn[slot] = win.turn;
                win.patch = false;
                win.behind = [Vec::new(), Vec::new()];
                if win.opaque != opaque {
                    win.opaque = opaque;
                    if opaque {
                        let region = compositor.create_region(qh, ());
                        let (w, h) = frame::whole(win);
                        region.add(0, 0, w as i32, h as i32);
                        win.surface.set_opaque_region(Some(&region));
                        region.destroy();
                    } else {
                        win.surface.set_opaque_region(None);
                    }
                }
                frame::tell_geometry(win, &compositor, qh);
                show(win, &buffers.buffers[slot], buffers.size, slot, qh, parts.as_deref());
                win.buffers = Some(buffers);
            } else {
                buffers.buffers.iter().for_each(|b| b.destroy());
            }
            done.push(id);
        }
    }

    #[allow(clippy::type_complexity)]
    fn next_to_patch(&self, done: &[u64]) -> Option<(u64, usize, Option<usize>, Vec<[i32; 4]>, Buffers, Patcher, f32)> {
        let mut state = self.state.borrow_mut();
        let State { screens, windows, .. } = &mut *state;
        for win in windows.iter_mut() {
            if !win.patch || win.dirty || win.gpu.is_some() || !win.configured || win.hidden || win.waiting || win.patcher.is_none() || done.contains(&win.id) {
                continue;
            }
            let (scale, px) = pixels(win, screens);
            if win.buffers.as_ref().is_none_or(|b| b.size != px) {
                win.dirty = true;
                continue;
            }
            let Some(slot) = (0..2).find(|&i| !win.busy[i]) else { continue };
            let from = if win.drawn[slot] == win.turn {
                None
            } else if win.drawn[1 - slot] == win.turn {
                Some(1 - slot)
            } else {
                win.dirty = true;
                continue;
            };
            win.patch = false;
            let behind = std::mem::take(&mut win.behind[slot]);
            let behind = if from.is_some() { Vec::new() } else { behind };
            let Some(buffers) = win.buffers.take() else { continue };
            return Some((win.id, slot, from, behind, buffers, win.patcher.take()?, scale));
        }
        None
    }

    fn next_to_paint(&self, qh: &QueueHandle<State>, done: &[u64]) -> Option<(u64, usize, Buffers, Painter, f32)> {
        let mut state = self.state.borrow_mut();
        let State { shm, screens, windows, .. } = &mut *state;
        for win in windows.iter_mut() {
            if !win.dirty || win.gpu.is_some() || !win.configured || win.hidden || win.waiting || win.paint.is_none() || done.contains(&win.id) {
                continue;
            }
            let (scale, px) = pixels(win, screens);
            if px.0 == 0 || px.1 == 0 {
                continue;
            }
            if win.buffers.as_ref().is_none_or(|b| b.size != px) {
                if let Some(old) = win.buffers.take() {
                    old.buffers.iter().for_each(|b| b.destroy());
                }
                win.busy = [false; 2];
                win.buffers = new_buffers(shm, qh, win.id, px);
                win.drawn = [0; 2];
            }
            let Some(slot) = (0..2).find(|&i| !win.busy[i]) else { continue };
            let Some(buffers) = win.buffers.take() else { continue };
            win.dirty = false;
            return Some((win.id, slot, buffers, win.paint.take()?, scale));
        }
        None
    }
}

pub(super) fn pixels(win: &Win, screens: &[Screen]) -> (f32, (u32, u32)) {
    let fine = (win.viewport.is_some() && win.scale120 > 0).then_some(win.scale120);
    pixels_for(frame::whole(win), fine, screens.iter().filter(|s| win.on.contains(&s.id)).map(|s| s.scale))
}

pub(super) fn pixels_for((w, h): (u32, u32), fine: Option<u32>, on: impl Iterator<Item = i32>) -> (f32, (u32, u32)) {
    let cut = |v: u64| v.min(u64::from(MOST_SIDE)) as u32;
    if let Some(s) = fine.map(u64::from) {
        return (s as f32 / 120.0, (cut((u64::from(w) * s).div_ceil(120)), cut((u64::from(h) * s).div_ceil(120))));
    }
    let s = on.max().unwrap_or(1).max(1) as u64;
    (s as f32, (cut(u64::from(w) * s), cut(u64::from(h) * s)))
}

fn new_buffers(shm: &wl_shm::WlShm, qh: &QueueHandle<State>, id: u64, (w, h): (u32, u32)) -> Option<Buffers> {
    let len = w as usize * h as usize * 4;
    let both = i32::try_from(len * 2).ok()?;
    let memory = shm::Memory::new(c"trindle-window", len * 2)?;
    let pool = shm.create_pool(memory.fd(), both, qh, ());
    let one = |slot: usize| pool.create_buffer((slot * len) as i32, w as i32, h as i32, w as i32 * 4,
                                               wl_shm::Format::Argb8888, qh, (id, slot));
    let buffers = [one(0), one(1)];
    pool.destroy();
    Some(Buffers { memory, buffers, size: (w, h) })
}

pub(super) fn changed_parts(memory: &[u8], (w, h): (u32, u32), a: usize, b: usize) -> Vec<[i32; 4]> {
    const BAND: usize = 16;
    const MOST_PARTS: usize = 12;
    let (w, h) = (w as usize, h as usize);
    let len = w * h * 4;
    if memory.len() < len * 2 || w == 0 || h == 0 {
        return vec![[0, 0, w as i32, h as i32]];
    }
    let (now, was) = (&memory[a * len..(a + 1) * len], &memory[b * len..(b + 1) * len]);
    let mut parts: Vec<[i32; 4]> = Vec::new();
    for top in (0..h).step_by(BAND) {
        let bottom = (top + BAND).min(h);
        let (mut left, mut right) = (usize::MAX, 0);
        for row in top..bottom {
            let (x, y) = (&now[row * w * 4..(row + 1) * w * 4], &was[row * w * 4..(row + 1) * w * 4]);
            if x == y {
                continue;
            }
            let first = x.iter().zip(y).position(|(p, q)| p != q).unwrap_or(0) / 4;
            let last = x.iter().zip(y).rposition(|(p, q)| p != q).unwrap_or(0) / 4;
            left = left.min(first);
            right = right.max(last + 1);
        }
        if left < right {
            parts.push([left as i32, top as i32, (right - left) as i32, (bottom - top) as i32]);
        }
    }
    let mut joined: Vec<[i32; 4]> = Vec::new();
    for part in parts {
        match joined.last_mut() {
            Some(last) if last[1] + last[3] == part[1] && part[0] < last[0] + last[2] && last[0] < part[0] + part[2] => {
                let (l, r) = (last[0].min(part[0]), (last[0] + last[2]).max(part[0] + part[2]));
                *last = [l, last[1], r - l, part[1] + part[3] - last[1]];
            }
            _ => joined.push(part),
        }
    }
    if joined.len() > MOST_PARTS {
        let (l, t) = (joined.iter().map(|p| p[0]).min().unwrap_or(0), joined[0][1]);
        let (r, b) = (joined.iter().map(|p| p[0] + p[2]).max().unwrap_or(0), joined.last().map_or(0, |p| p[1] + p[3]));
        joined = vec![[l, t, r - l, b - t]];
    }
    if joined.is_empty() {
        joined.push([0, 0, 1, 1]);
    }
    joined
}

pub(super) const MOST_SIDE: u32 = 16384;

pub(super) const BEHIND_MOST: usize = 16;

pub(super) fn catch_up(memory: &mut [u8], size: (u32, u32), from: usize, to: usize, part: [i32; 4]) {
    let (w, h) = (size.0 as i64, size.1 as i64);
    let len = (w * h * 4) as usize;
    let (left, right) = (i64::from(part[0]).clamp(0, w), (i64::from(part[0]) + i64::from(part[2])).clamp(0, w));
    let (top, bottom) = (i64::from(part[1]).clamp(0, h), (i64::from(part[1]) + i64::from(part[3])).clamp(0, h));
    if memory.len() < len * 2 || left >= right {
        return;
    }
    for row in top..bottom {
        let start = ((row * w + left) * 4) as usize;
        let end = ((row * w + right) * 4) as usize;
        memory.copy_within(from * len + start..from * len + end, to * len + start);
    }
}

pub(super) fn to_compositor(bytes: &mut [u8]) {
    for px in bytes.chunks_exact_mut(4) {
        px.swap(0, 2);
    }
}

pub(super) fn show(win: &mut Win, buffer: &wl_buffer::WlBuffer, (pw, ph): (u32, u32), slot: usize, qh: &QueueHandle<State>, parts: Option<&[[i32; 4]]>) {
    let fine = win.viewport.is_some() && win.scale120 > 0;
    let (sw, sh) = frame::whole(win);
    match &win.viewport {
        Some(v) if fine => {
            win.surface.set_buffer_scale(1);
            v.set_destination(sw as i32, sh as i32);
        }
        v => {
            if let Some(v) = v {
                v.set_destination(-1, -1);
            }
            win.surface.set_buffer_scale((pw / sw.max(1)).max(1) as i32);
        }
    }
    win.surface.attach(Some(buffer), 0, 0);
    match parts {
        Some(parts) => parts.iter().for_each(|p| win.surface.damage_buffer(p[0], p[1], p[2], p[3])),
        None => win.surface.damage_buffer(0, 0, pw as i32, ph as i32),
    }
    win.surface.frame(qh, win.id);
    win.surface.commit();
    win.mapped = true;
    win.busy[slot] = true;
    win.waiting = true;
    win.sent = Some(std::time::Instant::now());
}
