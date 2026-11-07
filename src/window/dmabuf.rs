//! Windows the GPU draws: GBM buffers handed to the compositor with linux-dmabuf, beside the shared-memory pair.

use std::os::fd::AsFd;
use std::os::unix::fs::FileExt;
use std::path::PathBuf;
use std::rc::Rc;

use wayland_client::protocol::wl_buffer;
use wayland_client::{event_created_child, Connection, Dispatch, Proxy, QueueHandle};
use wayland_protocols::wp::linux_dmabuf::zv1::client::{
    zwp_linux_buffer_params_v1::{self, ZwpLinuxBufferParamsV1},
    zwp_linux_dmabuf_feedback_v1::{self, ZwpLinuxDmabufFeedbackV1},
    zwp_linux_dmabuf_v1::{self, ZwpLinuxDmabufV1},
};

use super::{frame, pixels, show, Display, State, Window, BEHIND_MOST};
use crate::paint::remote::Remote;
use crate::sys::gbm;

const MOD_INVALID: u64 = 0x00ff_ffff_ffff_ffff;

pub(super) struct Offer {
    global: ZwpLinuxDmabufV1,
    table: Vec<(u32, u64)>,
    main: Option<u64>,
    tranche: (Option<u64>, Vec<u64>),
    tranches: Vec<(Option<u64>, Vec<u64>)>,
    pub(super) modifiers: Vec<u64>,
}

impl Offer {
    pub(super) fn new(global: ZwpLinuxDmabufV1, qh: &QueueHandle<State>) -> Offer {
        if global.version() >= 4 {
            global.get_default_feedback(qh, ());
        }
        Offer { global, table: Vec::new(), main: None, tranche: (None, Vec::new()), tranches: Vec::new(), modifiers: Vec::new() }
    }

    fn layouts(&self) -> Vec<u64> {
        self.modifiers.iter().copied().filter(|&m| m != MOD_INVALID).collect()
    }
}

pub(crate) enum Alloc {
    Gbm(Rc<gbm::Device>),
    Service(Rc<Remote>),
    #[cfg(test)]
    Memory,
}

impl Alloc {
    #[cfg(test)]
    pub(crate) fn on(node: &std::path::Path) -> Option<Alloc> {
        Some(Alloc::Gbm(gbm::Device::new(gbm::open_node(node)?)?))
    }

    fn make(&self, offer: &Offer, qh: &QueueHandle<State>, id: u64, (w, h): (u32, u32), slot: usize) -> Option<Slot> {
        let layouts = offer.layouts();
        let buffer_id = crate::paint::new_id();
        let (gpu, d, service) = match self {
            Alloc::Gbm(dev) => {
                let b = dev.buffer(w, h, gbm::ARGB8888, &layouts)?;
                let d = b.export()?;
                (Some(b), d, None)
            }
            Alloc::Service(r) => (None, r.buffer(buffer_id, (w, h), &layouts)?, Some(r.clone())),
            #[cfg(test)]
            Alloc::Memory => {
                let m = crate::common::shm::Memory::new(c"trindle-dmabuf", w as usize * h as usize * 4)?;
                let plane = gbm::Plane { fd: m.fd().try_clone_to_owned().ok()?, stride: w * 4, offset: 0 };
                (None, gbm::Dmabuf { width: w, height: h, format: gbm::ARGB8888, modifier: gbm::MOD_LINEAR, planes: vec![plane] }, None)
            }
        };
        let modifier = if layouts.is_empty() { MOD_INVALID } else { d.modifier };
        let params = offer.global.create_params(qh, id);
        for (i, p) in d.planes.iter().enumerate() {
            params.add(p.fd.as_fd(), i as u32, p.offset, p.stride, (modifier >> 32) as u32, modifier as u32);
        }
        let wl = params.create_immed(w as i32, h as i32, d.format, zwp_linux_buffer_params_v1::Flags::empty(), qh, (id, slot));
        let modifier = (modifier != MOD_INVALID).then_some(modifier);
        Some(Slot { wl, params, gpu: gpu.map(|b| (b, d)), service, modifier, id: buffer_id })
    }
}

pub(super) struct Slot {
    wl: wl_buffer::WlBuffer,
    params: ZwpLinuxBufferParamsV1,
    gpu: Option<(gbm::Buffer, gbm::Dmabuf)>,
    service: Option<Rc<Remote>>,
    modifier: Option<u64>,
    id: u64,
}

impl Drop for Slot {
    fn drop(&mut self) {
        self.wl.destroy();
        self.params.destroy();
        if let Some(r) = &self.service {
            r.free(self.id);
        }
    }
}

pub(crate) struct Frame<'a> {
    pub dmabuf: Option<&'a gbm::Dmabuf>,
    pub modifier: Option<u64>,
    pub id: u64,
    pub size: (u32, u32),
    pub scale: f32,
    pub age: u32,
    pub behind: &'a [[i32; 4]],
}

pub(crate) enum Drawn {
    Whole,
    Parts(Vec<[i32; 4]>),
    Failed,
    Sent { seq: u64, parts: Option<Vec<[i32; 4]>> },
    Gone,
    Again,
    Moved(Alloc),
}

pub(crate) type GpuPaint = Box<dyn FnMut(&Frame) -> Drawn>;

pub(super) struct Pool {
    alloc: Alloc,
    paint: GpuPaint,
    sleep: Option<Box<dyn FnMut(bool)>>,
    asleep: bool,
    size: (u32, u32),
    pub(super) slots: Vec<Slot>,
    ages: Ages,
    sent: Option<(u64, usize, bool)>,
}

#[derive(Default)]
pub(crate) struct Ages {
    drawn: [u64; 2],
    turn: u64,
    pub(crate) behind: [Vec<[i32; 4]>; 2],
}

impl Ages {
    pub(crate) fn age(&self, slot: usize) -> u32 {
        if self.drawn[slot] == 0 { 0 } else { (self.turn - self.drawn[slot] + 1) as u32 }
    }

    pub(crate) fn fresh(&self) -> bool {
        self.drawn == [0; 2]
    }

    pub(crate) fn shown(&mut self, slot: usize, parts: Option<&[[i32; 4]]>) {
        self.turn += 1;
        self.drawn[slot] = self.turn;
        self.behind[slot].clear();
        let other = &mut self.behind[1 - slot];
        match parts {
            Some(p) if other.len() + p.len() <= BEHIND_MOST => other.extend(p),
            _ => {
                other.clear();
                self.drawn[1 - slot] = 0;
            }
        }
    }
}

impl Display {
    pub(super) fn trim_unseen(&self) {
        let mut told = Vec::new();
        {
            let mut state = self.state.borrow_mut();
            for win in state.windows.iter_mut() {
                let unseen = win.hidden || win.suspended;
                let Some(pool) = win.gpu.as_mut().filter(|p| p.asleep != unseen) else { continue };
                pool.asleep = unseen;
                if unseen {
                    pool.sent = None;
                    pool.slots.clear();
                    pool.size = (0, 0);
                    pool.ages = Ages::default();
                    win.busy = [false; 2];
                } else {
                    win.dirty = true;
                }
                told.push((win.id, unseen));
            }
        }
        for (id, unseen) in told {
            let call = self.state.borrow_mut().win(id).and_then(|w| w.gpu.as_mut()?.sleep.take());
            let Some(mut call) = call else { continue };
            call(unseen);
            if let Some(pool) = self.state.borrow_mut().win(id).and_then(|w| w.gpu.as_mut()) {
                pool.sleep.get_or_insert(call);
            }
        }
    }

    pub(crate) fn takes_dmabufs(&self) -> bool {
        self.state.borrow().dmabuf.as_ref().is_some_and(|o| !o.modifiers.is_empty())
    }

    pub(crate) fn dmabuf_node(&self) -> Option<PathBuf> {
        let state = self.state.borrow();
        let offer = state.dmabuf.as_ref().filter(|o| !o.modifiers.is_empty())?;
        match offer.main {
            Some(dev) => gbm::render_node_of(dev),
            None => gbm::render_nodes().into_iter().next(),
        }
    }

    pub(super) fn paint_gpu(&self, qh: &QueueHandle<State>) {
        let mut done = Vec::new();
        while let Some((id, slot, mut pool, scale)) = self.next_for_gpu(qh, &done) {
            done.push(id);
            let first = pool.ages.fresh();
            let frame = match pool.slots.get(slot) {
                Some(s) => Frame { dmabuf: s.gpu.as_ref().map(|g| &g.1), modifier: s.modifier, id: s.id, size: pool.size, scale, age: pool.ages.age(slot), behind: &pool.ages.behind[slot] },
                None => Frame { dmabuf: None, modifier: None, id: 0, size: pool.size, scale, age: 0, behind: &[] },
            };
            let parts = match (pool.paint)(&frame) {
                Drawn::Failed | Drawn::Gone | Drawn::Again => {
                    if let Some(win) = self.state.borrow_mut().win(id) {
                        win.busy = [false; 2];
                        win.dirty = true;
                    }
                    continue;
                }
                Drawn::Sent { seq, .. } => {
                    pool.sent = Some((seq, slot, first));
                    if let Some(win) = self.state.borrow_mut().win(id) {
                        win.gpu = Some(pool);
                    }
                    continue;
                }
                Drawn::Moved(alloc) => {
                    done.pop();
                    if let Some(win) = self.state.borrow_mut().win(id) {
                        pool.moved_to(alloc);
                        win.busy = [false; 2];
                        win.dirty = true;
                        win.gpu = Some(pool);
                    }
                    continue;
                }
                Drawn::Whole => None,
                Drawn::Parts(p) => (!first).then_some(p),
            };
            pool.ages.shown(slot, parts.as_deref());
            let mut state = self.state.borrow_mut();
            let compositor = state.compositor.clone();
            if let Some(win) = state.win(id) {
                frame::tell_geometry(win, &compositor, qh);
                frame::tell_opaque(win, &compositor, qh);
                show(win, &pool.slots[slot].wl, pool.size, slot, qh, parts.as_deref());
                win.gpu = Some(pool);
            }
        }
    }

    fn next_for_gpu(&self, qh: &QueueHandle<State>, done: &[u64]) -> Option<(u64, usize, Box<Pool>, f32)> {
        let mut state = self.state.borrow_mut();
        let State { screens, windows, dmabuf, .. } = &mut *state;
        let offer = dmabuf.as_ref()?;
        for win in windows.iter_mut() {
            if win.gpu.as_ref().is_none_or(|p| p.sent.is_some()) || !win.dirty || !win.configured || win.hidden || win.waiting || done.contains(&win.id) {
                continue;
            }
            let (scale, px) = pixels(win, screens);
            if px.0 == 0 || px.1 == 0 {
                continue;
            }
            let pool = win.gpu.as_mut()?;
            if pool.size != px || pool.slots.is_empty() {
                pool.slots.clear();
                pool.slots = (0..2).map(|slot| pool.alloc.make(offer, qh, win.id, px, slot)).collect::<Option<_>>().unwrap_or_default();
                pool.size = px;
                pool.ages = Ages::default();
                win.busy = [false; 2];
                if pool.slots.is_empty() {
                    if matches!(pool.alloc, Alloc::Service(_)) {
                        win.dirty = false;
                        return Some((win.id, 0, win.gpu.take()?, scale));
                    }
                    win.gpu = None;
                    continue;
                }
            }
            let Some(slot) = (0..2).find(|&i| !win.busy[i]) else { continue };
            win.dirty = false;
            return Some((win.id, slot, win.gpu.take()?, scale));
        }
        None
    }
}

impl Pool {
    fn moved_to(&mut self, alloc: Alloc) {
        self.alloc = alloc;
        self.sent = None;
        self.slots.clear();
        self.size = (0, 0);
        self.ages = Ages::default();
    }
}

impl Window {
    pub(crate) fn answered(&self, seq: u64, drawn: Drawn) {
        let qh = self.display.queue.borrow().handle();
        {
            let mut state = self.display.state.borrow_mut();
            let compositor = state.compositor.clone();
            let Some(win) = state.win(self.id) else { return };
            let Some(pool) = win.gpu.as_mut() else { return };
            let Some((sent, slot, first)) = pool.sent else { return };
            if sent != seq {
                return;
            }
            pool.sent = None;
            let parts = match drawn {
                Drawn::Whole => Some(None),
                Drawn::Parts(p) => Some((!first).then_some(p)),
                Drawn::Moved(alloc) => {
                    pool.moved_to(alloc);
                    None
                }
                Drawn::Again => None,
                Drawn::Failed | Drawn::Gone | Drawn::Sent { .. } => {
                    win.gpu = None;
                    None
                }
            };
            match parts {
                Some(parts) => {
                    pool_shown(win, slot, parts, &compositor, &qh);
                }
                None => {
                    win.busy = [false; 2];
                    win.dirty = true;
                }
            }
        }
        self.display.paint();
    }

    pub(crate) fn draw_by_gpu(&self, alloc: Alloc, sleep: impl FnMut(bool) + 'static, paint: impl FnMut(&Frame) -> Drawn + 'static) -> bool {
        {
            let mut state = self.display.state.borrow_mut();
            if state.dmabuf.as_ref().is_none_or(|o| o.modifiers.is_empty()) {
                return false;
            }
            let Some(win) = state.win(self.id) else { return false };
            if let Some(old) = win.buffers.take() {
                old.buffers.iter().for_each(|b| b.destroy());
            }
            win.busy = [false; 2];
            win.drawn = [0; 2];
            win.dirty = true;
            win.gpu = Some(Box::new(Pool { alloc, paint: Box::new(paint), sleep: Some(Box::new(sleep)), asleep: false, size: (0, 0), slots: Vec::new(), ages: Ages::default(), sent: None }));
        }
        self.display.paint();
        true
    }
}

fn pool_shown(win: &mut super::Win, slot: usize, parts: Option<Vec<[i32; 4]>>, compositor: &wayland_client::protocol::wl_compositor::WlCompositor, qh: &QueueHandle<State>) {
    let Some(mut pool) = win.gpu.take() else { return };
    pool.ages.shown(slot, parts.as_deref());
    frame::tell_geometry(win, compositor, qh);
    frame::tell_opaque(win, compositor, qh);
    show(win, &pool.slots[slot].wl, pool.size, slot, qh, parts.as_deref());
    win.gpu = Some(pool);
}

impl Dispatch<ZwpLinuxDmabufV1, ()> for State {
    fn event(state: &mut Self, _: &ZwpLinuxDmabufV1, event: zwp_linux_dmabuf_v1::Event, _: &(), _: &Connection, _: &QueueHandle<Self>) {
        if let (zwp_linux_dmabuf_v1::Event::Modifier { format: gbm::ARGB8888, modifier_hi, modifier_lo }, Some(o)) = (event, &mut state.dmabuf) {
            o.modifiers.push((u64::from(modifier_hi) << 32) | u64::from(modifier_lo));
        }
    }
}

fn device(bytes: &[u8]) -> Option<u64> {
    Some(u64::from_ne_bytes(bytes.try_into().ok()?))
}

const MOST_TABLE: u32 = 1 << 20;

impl Dispatch<ZwpLinuxDmabufFeedbackV1, ()> for State {
    fn event(state: &mut Self, _: &ZwpLinuxDmabufFeedbackV1, event: zwp_linux_dmabuf_feedback_v1::Event, _: &(), _: &Connection, _: &QueueHandle<Self>) {
        use zwp_linux_dmabuf_feedback_v1::Event as E;
        let Some(o) = &mut state.dmabuf else { return };
        match event {
            E::FormatTable { fd, size } => {
                let mut bytes = vec![0; size.min(MOST_TABLE) as usize];
                o.table = std::fs::File::from(fd).read_exact_at(&mut bytes, 0).map(|()| bytes.chunks_exact(16)
                    .map(|e| (u32::from_ne_bytes([e[0], e[1], e[2], e[3]]), u64::from_ne_bytes(e[8..16].try_into().unwrap_or_default()))).collect())
                    .unwrap_or_default();
            }
            E::MainDevice { device: d } => o.main = device(&d),
            E::TrancheTargetDevice { device: d } => o.tranche.0 = device(&d),
            E::TrancheFormats { indices } => {
                let found = indices.chunks_exact(2).filter_map(|i| o.table.get(u16::from_ne_bytes([i[0], i[1]]) as usize))
                    .filter(|e| e.0 == gbm::ARGB8888).map(|e| e.1);
                o.tranche.1.extend(found);
            }
            E::TrancheDone => o.tranches.push(std::mem::take(&mut o.tranche)),
            E::Done => {
                let tranches = std::mem::take(&mut o.tranches);
                let mut on_main: Vec<u64> = Vec::new();
                for m in tranches.iter().filter(|t| t.0.is_none() || t.0 == o.main).flat_map(|t| &t.1) {
                    if !on_main.contains(m) {
                        on_main.push(*m);
                    }
                }
                o.modifiers = on_main;
            }
            _ => {}
        }
    }
}

impl Dispatch<ZwpLinuxBufferParamsV1, u64> for State {
    fn event(state: &mut Self, _: &ZwpLinuxBufferParamsV1, event: zwp_linux_buffer_params_v1::Event, id: &u64, _: &Connection, _: &QueueHandle<Self>) {
        if let (zwp_linux_buffer_params_v1::Event::Failed, Some(win)) = (event, state.win(*id)) {
            if win.gpu.take().is_some() {
                win.busy = [false; 2];
                win.dirty = true;
            }
        }
    }

    event_created_child!(State, ZwpLinuxBufferParamsV1, [
        zwp_linux_buffer_params_v1::EVT_CREATED_OPCODE => (wl_buffer::WlBuffer, (u64::MAX, 0)),
    ]);
}
