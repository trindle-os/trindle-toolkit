//! The drawing service: one GPU and one GL context drawing for every program of a user, so the driver's memory is paid once.

use std::collections::HashMap;
use std::os::fd::{AsFd, AsRawFd, BorrowedFd, OwnedFd};
use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::rc::Rc;
use std::time::{Duration, Instant};

use super::gpu::{self, Gpu, Store, Target};
use super::link::{read_ask, write_tell, Ask, Tell, LOCK, MOST_FILE, MOST_MESSAGE, SOCKET, VERSION};
use super::list::Op;
use super::log::{say, Kind};
use super::wire::{Decoder, MAX_FRAME_SHADOW_PIXELS};
use crate::shadow::Shadows;
use crate::sys::fd::poll;
use crate::sys::gbm::{self, Dmabuf};
use crate::sys::gl;
use crate::sys::sock::{self, Got, Sealed, MOST_FDS};

const MOST_PROGRAMS: usize = 32;
const MOST_BUFFERS: usize = 32;
const MOST_BUFFER_PIXELS: u64 = 1 << 26;
const MOST_SIDE: u32 = 16384;
const FDS_KEPT: u64 = 128;
const ACCEPT_REST: Duration = Duration::from_millis(100);
const SLICE: Duration = Duration::from_millis(8);
const FRAME_MOST: Duration = Duration::from_secs(2);
const SHADOWS_MOST: usize = 8 << 20;
const IDLE_MS: i32 = 1000;
const SETTLE_AFTER: Duration = Duration::from_secs(2);
const GL_CONTEXT_LOST: u32 = 0x0507;

#[derive(Clone, Copy)]
pub(crate) struct Sizes {
    pub(crate) start: u32,
    pub(crate) most: u32,
    pub(crate) letters: u32,
}

impl Default for Sizes {
    fn default() -> Sizes {
        Sizes { start: gpu::START, most: gpu::SIDE, letters: gpu::SIDE }
    }
}

pub(crate) fn runtime_file(name: &str) -> Option<PathBuf> {
    let dir = PathBuf::from(std::env::var_os("XDG_RUNTIME_DIR")?);
    sock::private_folder(&dir).then(|| dir.join(name))
}

struct Buffer {
    target: Target,
    dmabuf: Dmabuf,
    _bo: gbm::Buffer,
}

struct Playing {
    ops: std::vec::IntoIter<Op<'static>>,
    seq: u64,
    id: u64,
    spent: Duration,
    wait: Option<OwnedFd>,
}

struct Client {
    fd: OwnedFd,
    decoder: Decoder,
    store: Store,
    shadows: Shadows,
    buffers: HashMap<u64, Buffer>,
    pixels: u64,
    fds: usize,
    playing: Option<Playing>,
    gone: bool,
}

enum Stop {
    Program(&'static str),
    Driver,
}

struct Service {
    gpu: Gpu,
    device: Rc<gbm::Device>,
    listener: OwnedFd,
    clients: Vec<Client>,
    rdev: u64,
    sizes: Sizes,
    first: usize,
    fds_each: usize,
    rest_until: Option<Instant>,
    last_busy: Instant,
    settled: bool,
    buf: Vec<u8>,
}

pub fn main() -> ExitCode {
    let (Some(socket), Some(lock)) = (runtime_file(SOCKET), runtime_file(LOCK)) else {
        eprintln!("trindle-draw: no private runtime folder");
        return ExitCode::FAILURE;
    };
    serve(&socket, &lock, Sizes::default())
}

pub(crate) fn serve(socket: &Path, lock: &Path, sizes: Sizes) -> ExitCode {
    let held = std::fs::OpenOptions::new().create(true).truncate(false).write(true).mode(0o600).custom_flags(libc::O_CLOEXEC).open(lock);
    let Ok(held) = held else {
        eprintln!("trindle-draw: the lock file cannot be opened");
        return ExitCode::FAILURE;
    };
    if !sock::try_lock(held.as_raw_fd()) {
        return ExitCode::SUCCESS;
    }
    if crate::theme::drawn_by_processor() {
        eprintln!("trindle-draw: {}", super::Why::Asked.words());
        return ExitCode::FAILURE;
    }
    let started = start(socket, sizes);
    let mut service = match started {
        Ok(s) => s,
        Err(why) => {
            eprintln!("trindle-draw: {}", why.words());
            return ExitCode::FAILURE;
        }
    };
    service.run();
    let _ = std::fs::remove_file(socket);
    eprintln!("trindle-draw: {}", super::Why::Failed.words());
    ExitCode::FAILURE
}

fn start(socket: &Path, sizes: Sizes) -> Result<Service, super::Why> {
    use super::Why;
    if !gl::libraries() {
        return Err(Why::NoLibraries);
    }
    let node = std::env::var_os("TRINDLE_DRAW_NODE").map(PathBuf::from).or_else(|| gbm::render_nodes().into_iter().next()).ok_or(Why::NoNode)?;
    let rdev = std::fs::metadata(&node).map_err(|_| Why::NoNode)?.rdev();
    let device = gbm::Device::new(gbm::open_node(&node).ok_or(Why::NoNode)?).ok_or(Why::NoNode)?;
    let mut gpu = Gpu::on(device.clone()).ok_or(Why::NoContext)?;
    if gl::is_software(&gpu.renderer()) {
        return Err(Why::Software);
    }
    if !gpu::agrees(&mut gpu) {
        return Err(Why::SelfTest);
    }
    gpu.serve(sizes.letters);
    let _ = std::fs::remove_file(socket);
    let listener = sock::listen(socket).map_err(|_| Why::NoService)?;
    let fds_each = (sock::open_files_up().saturating_sub(FDS_KEPT) / MOST_PROGRAMS as u64) as usize;
    Ok(Service { gpu, device, listener, clients: Vec::new(), rdev, sizes, first: 0, fds_each, rest_until: None, last_busy: Instant::now(), settled: true, buf: vec![0; MOST_MESSAGE + 1] })
}

impl Service {
    fn run(&mut self) {
        loop {
            let n = self.clients.len();
            let running = self.clients.iter().any(|c| c.playing.as_ref().is_some_and(|p| p.wait.is_none()));
            self.rest_until = self.rest_until.filter(|until| Instant::now() < *until);
            let listening = if self.rest_until.is_some() { -1 } else { self.listener.as_raw_fd() };
            let mut fds = vec![libc::pollfd { fd: listening, events: libc::POLLIN, revents: 0 }];
            fds.extend(self.clients.iter().map(|c| libc::pollfd { fd: c.fd.as_raw_fd(), events: if c.playing.is_some() { 0 } else { libc::POLLIN }, revents: 0 }));
            fds.extend(self.clients.iter().map(|c| libc::pollfd { fd: c.playing.as_ref().and_then(|p| p.wait.as_ref()).map_or(-1, |w| w.as_raw_fd()), events: libc::POLLIN, revents: 0 }));
            poll(&mut fds, if running { 0 } else if self.rest_until.is_some() { ACCEPT_REST.as_millis() as i32 } else { IDLE_MS });
            if fds[0].revents & libc::POLLIN != 0 {
                self.accept();
            }
            for k in 0..n {
                let at = (self.first + k) % n;
                let ready = fds[at + 1].revents;
                let c = &mut self.clients[at];
                if c.playing.is_some() && ready & (libc::POLLHUP | libc::POLLERR | libc::POLLNVAL) != 0 {
                    c.gone = true;
                    continue;
                }
                match c.playing.as_mut() {
                    Some(p) if p.wait.is_some() && fds[at + 1 + n].revents == 0 => continue,
                    Some(p) => p.wait = None,
                    None if ready == 0 => continue,
                    None => {}
                }
                self.last_busy = Instant::now();
                self.settled = false;
                match self.turn(at) {
                    Ok(()) => {}
                    Err(Stop::Program(why)) => {
                        say(Kind::Whole, || format!("a program was let go: {why}"));
                        self.clients[at].gone = true;
                    }
                    Err(Stop::Driver) => return,
                }
            }
            self.first = self.first.wrapping_add(1);
            self.let_go();
            if !self.settled && self.last_busy.elapsed() >= SETTLE_AFTER {
                self.settled = true;
                crate::sys::lean::settle();
            }
        }
    }

    fn accept(&mut self) {
        for _ in 0..MOST_PROGRAMS {
            let fd = match sock::accept(self.listener.as_raw_fd()) {
                Ok(fd) => fd,
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => return,
                Err(_) => {
                    self.rest_until = Some(Instant::now() + ACCEPT_REST);
                    return;
                }
            };
            if self.clients.len() >= MOST_PROGRAMS || sock::peer_uid(fd.as_raw_fd()) != Some(sock::own_uid()) {
                continue;
            }
            let hello = write_tell(&Tell::Hello { version: VERSION, rdev: self.rdev });
            if sock::send(fd.as_raw_fd(), &hello, &[]).is_err() {
                continue;
            }
            self.clients.push(Client {
                fd, decoder: Decoder::default(), store: self.gpu.store(self.sizes.start, self.sizes.most), shadows: Shadows::default(),
                buffers: HashMap::new(), pixels: 0, fds: 0, playing: None, gone: false,
            });
        }
    }

    fn let_go(&mut self) {
        let mut at = 0;
        while at < self.clients.len() {
            if !self.clients[at].gone {
                at += 1;
                continue;
            }
            let c = self.clients.swap_remove(at);
            for (_, b) in c.buffers {
                self.gpu.free(b.target);
            }
            self.gpu.free_store(c.store);
            if self.clients.is_empty() {
                self.gpu.all_gone();
            }
        }
    }

    fn turn(&mut self, at: usize) -> Result<(), Stop> {
        let Service { gpu, device, clients, buf, fds_each, .. } = self;
        let c = &mut clients[at];
        gpu.swap_store(&mut c.store);
        let r = catch_unwind(AssertUnwindSafe(|| {
            if c.playing.is_some() {
                gpu.resume();
                play(gpu, c)
            } else {
                read_one(gpu, device, c, *fds_each, buf)
            }
        }));
        if r.is_err() {
            gpu.abandon();
            c.playing = None;
        }
        gpu.swap_store(&mut c.store);
        let fault = gl_fault(gpu);
        r.unwrap_or(Err(Stop::Program("drawing what it sent panicked"))).and(fault)
    }
}

fn gl_fault(gpu: &Gpu) -> Result<(), Stop> {
    match gpu.error() {
        GL_CONTEXT_LOST => Err(Stop::Driver),
        _ if !gpu.current() => Err(Stop::Driver),
        0 => Ok(()),
        _ => Err(Stop::Program("what it sent made a GL error")),
    }
}

fn tell(c: &Client, what: &Tell, fds: &[BorrowedFd]) -> Result<(), Stop> {
    sock::send(c.fd.as_raw_fd(), &write_tell(what), fds).map_err(|_| Stop::Program("it does not read what it is told"))
}

fn read_one(gpu: &mut Gpu, device: &Rc<gbm::Device>, c: &mut Client, fds_each: usize, buf: &mut [u8]) -> Result<(), Stop> {
    let mut fds: Vec<OwnedFd> = Vec::new();
    let len = match sock::recv(c.fd.as_raw_fd(), buf, &mut fds) {
        Got::Nothing => return Ok(()),
        Got::Ended => {
            c.gone = true;
            return Ok(());
        }
        Got::Message { cut: true, .. } => return Err(Stop::Program("a message too long")),
        Got::Message { len, .. } => len,
    };
    let ask = read_ask(&buf[..len]).map_err(|_| Stop::Program("not a message"))?;
    if fds.len() > usize::from(matches!(ask, Ask::Frame { .. })) {
        return Err(Stop::Program("descriptors it had no reason to send"));
    }
    match ask {
        Ask::Buffer { id, w, h, layouts } => make_buffer(gpu, device, c, fds_each, id, (w, h), &layouts),
        Ask::Free { id } => {
            let b = c.buffers.remove(&id).ok_or(Stop::Program("a buffer it does not have"))?;
            c.pixels -= u64::from(b.target.w) * u64::from(b.target.h);
            c.fds -= b.dmabuf.planes.len();
            gpu.free(b.target);
            Ok(())
        }
        Ask::Sleep => {
            gpu.empty_store();
            gpu.forget_shades();
            c.shadows.clear();
            Ok(())
        }
        Ask::Frame { seq, id, part, control, pixels, inline } => {
            let map;
            let bytes = match fds.first() {
                Some(fd) if inline.is_empty() => {
                    map = Sealed::map(fd.as_fd(), MOST_FILE).ok_or(Stop::Program("a frame's file not sealed"))?;
                    map.bytes()
                }
                Some(_) => return Err(Stop::Program("a frame both sent and in a file")),
                None => inline,
            };
            if bytes.len() != control as usize + pixels as usize {
                return Err(Stop::Program("a frame of the wrong length"));
            }
            let (control, pixels) = bytes.split_at(control as usize);
            start_frame(gpu, c, seq, id, part, control, pixels)
        }
    }
}

fn make_buffer(gpu: &mut Gpu, device: &Rc<gbm::Device>, c: &mut Client, fds_each: usize, id: u64, (w, h): (u32, u32), layouts: &[u64]) -> Result<(), Stop> {
    let pixels = u64::from(w) * u64::from(h);
    if w == 0 || h == 0 || w > MOST_SIDE || h > MOST_SIDE || c.buffers.contains_key(&id) {
        return Err(Stop::Program("a buffer that cannot be"));
    }
    let made = (c.buffers.len() < MOST_BUFFERS && c.pixels + pixels <= MOST_BUFFER_PIXELS).then(|| {
        let bo = device.buffer(w, h, gbm::ARGB8888, layouts)?;
        let dmabuf = bo.export()?;
        if dmabuf.planes.len() > MOST_FDS || c.fds + dmabuf.planes.len() > fds_each {
            return None;
        }
        gpu.error();
        let target = gpu.target_of(&dmabuf, (!layouts.is_empty()).then_some(dmabuf.modifier))?;
        gpu.clear_target(&target);
        gpu.finish();
        Some(Buffer { target, dmabuf, _bo: bo })
    }).flatten();
    let Some(b) = made else { return tell(c, &Tell::NotMade { id }, &[]) };
    let planes = b.dmabuf.planes.iter().map(|p| (p.stride, p.offset)).collect();
    let fds: Vec<BorrowedFd> = b.dmabuf.planes.iter().map(|p| p.fd.as_fd()).collect();
    let told = tell(c, &Tell::Made { id, format: b.dmabuf.format, modifier: b.dmabuf.modifier, planes }, &fds);
    if told.is_err() {
        gpu.free(b.target);
        return told;
    }
    c.pixels += pixels;
    c.fds += b.dmabuf.planes.len();
    c.buffers.insert(id, b);
    Ok(())
}

fn start_frame(gpu: &mut Gpu, c: &mut Client, seq: u64, id: u64, part: [i32; 4], control: &[u8], pixels: &[u8]) -> Result<(), Stop> {
    let target = c.buffers.get(&id).ok_or(Stop::Program("a frame for a buffer it does not have"))?;
    c.decoder.hold_to(target.target.w, target.target.h);
    let ops = c.decoder.frame(control, pixels).map_err(|_| Stop::Program("a frame that is not one"))?;
    let news = c.decoder.news().ok_or(Stop::Program("no memory"))?;
    if news.reset {
        gpu.empty_store();
    }
    let mut placed = true;
    for (key, mask, hollow) in news.masks {
        placed &= gpu.letter(key, &mask, hollow);
    }
    for (name, picture) in news.images {
        gpu.picture(name, &picture);
    }
    if !placed {
        return tell(c, &Tell::Again { seq }, &[]);
    }
    c.shadows.allow(MAX_FRAME_SHADOW_PIXELS);
    gpu.begin(&target.target, Some(part));
    c.playing = Some(Playing { ops: ops.into_iter(), seq, id, spent: Duration::ZERO, wait: None });
    play(gpu, c)
}

fn play(gpu: &mut Gpu, c: &mut Client) -> Result<(), Stop> {
    let began = Instant::now();
    let Some(p) = c.playing.as_mut() else { return Ok(()) };
    let finished = gpu.run(&mut p.ops, &c.shadows, Some(began + SLICE));
    p.spent += began.elapsed();
    let (seq, id, spent) = (p.seq, p.id, p.spent);
    if gpu.lost() {
        gpu.abandon();
        c.playing = None;
        return tell(c, &Tell::Again { seq }, &[]);
    }
    if !finished {
        p.wait = gpu.fence();
        return if spent > FRAME_MOST { Err(Stop::Program("a frame that takes too long")) } else { Ok(()) };
    }
    gpu.end();
    c.playing = None;
    let b = c.buffers.get(&id).ok_or(Stop::Program("a frame for a buffer it does not have"))?;
    if let Some(fence) = gpu.fence() {
        gbm::wait_for(b.dmabuf.planes[0].fd.as_fd(), fence);
    }
    gl_fault(gpu)?;
    if c.shadows.bytes() > SHADOWS_MOST {
        c.shadows.clear();
        gpu.forget_shades();
    }
    tell(c, &Tell::Done { seq }, &[])
}

#[cfg(test)]
pub(crate) mod rig {
    use std::path::PathBuf;
    use std::process::{Child, Command, Stdio};
    use std::time::{Duration, Instant};

    pub(crate) struct Running {
        pub(crate) child: Child,
        pub(crate) socket: PathBuf,
        dir: PathBuf,
    }

    impl Running {
        pub(crate) fn start(name: &str, side: u32, data_mb: u64) -> Running {
            let dir = std::env::temp_dir().join(format!("trindle-draw-test-{}-{name}", std::process::id()));
            let _ = std::fs::remove_dir_all(&dir);
            std::fs::create_dir_all(&dir).unwrap();
            let socket = dir.join("s");
            let child = Command::new(std::env::current_exe().unwrap())
                .args(["--exact", "paint::service::child::a_service_for_the_other_tests", "--include-ignored", "--nocapture"])
                .env("TRINDLE_DRAW_TEST_DIR", &dir).env("TRINDLE_DRAW_TEST_SIDE", side.to_string()).env("TRINDLE_DRAW_TEST_DATA", data_mb.to_string())
                .stdout(Stdio::null()).spawn().unwrap();
            let began = Instant::now();
            while !socket.exists() {
                assert!(began.elapsed() < Duration::from_secs(30), "the service did not start");
                std::thread::sleep(Duration::from_millis(5));
            }
            Running { child, socket, dir }
        }

        pub(crate) fn alive(&mut self) -> bool {
            self.child.try_wait().unwrap().is_none()
        }

        pub(crate) fn peak_mb(&self) -> f64 {
            let text = std::fs::read_to_string(format!("/proc/{}/status", self.child.id())).unwrap_or_default();
            let kb = text.lines().find_map(|l| l.strip_prefix("VmHWM:")?.split_whitespace().next()?.parse::<f64>().ok());
            kb.unwrap_or(0.0) / 1024.0
        }
    }

    impl Drop for Running {
        fn drop(&mut self) {
            let _ = self.child.kill();
            let _ = self.child.wait();
            let _ = std::fs::remove_dir_all(&self.dir);
        }
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod child {
    use super::*;

    #[test]
    #[ignore = "started by the other tests of the service, which name it"]
    fn a_service_for_the_other_tests() {
        let Some(dir) = std::env::var_os("TRINDLE_DRAW_TEST_DIR").map(PathBuf::from) else { return };
        let number = |key: &str| std::env::var(key).ok().and_then(|v| v.parse::<u64>().ok()).unwrap_or(0);
        let data = number("TRINDLE_DRAW_TEST_DATA");
        if data > 0 {
            let limit = libc::rlimit { rlim_cur: data << 20, rlim_max: data << 20 };
            assert!(crate::sys::sock::limit_data(&limit));
        }
        let side = number("TRINDLE_DRAW_TEST_SIDE") as u32;
        let sizes = if side == 0 { Sizes::default() } else { Sizes { start: side, most: side, letters: side } };
        serve(&dir.join("s"), &dir.join("l"), sizes);
    }
}
