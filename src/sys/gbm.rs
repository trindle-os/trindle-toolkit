//! Buffers the GPU draws into and the compositor reads: render nodes, libgbm opened at run time, dmabuf export.
#![cfg_attr(not(test), allow(dead_code))]

use super::gl::{table, Library};
use std::ffi::{c_char, c_void, CStr};
use std::os::fd::{AsRawFd, BorrowedFd, FromRawFd, OwnedFd};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::OnceLock;

table!(Gbm {
    gbm_create_device(i32) -> *mut c_void;
    gbm_device_destroy(*mut c_void);
    gbm_device_get_backend_name(*mut c_void) -> *const c_char;
    gbm_device_is_format_supported(*mut c_void, u32, u32) -> i32;
    gbm_bo_create(*mut c_void, u32, u32, u32, u32) -> *mut c_void;
    gbm_bo_create_with_modifiers(*mut c_void, u32, u32, u32, *const u64, u32) -> *mut c_void;
    gbm_bo_destroy(*mut c_void);
    gbm_bo_get_plane_count(*mut c_void) -> i32;
    gbm_bo_get_fd_for_plane(*mut c_void, i32) -> i32;
    gbm_bo_get_stride_for_plane(*mut c_void, i32) -> u32;
    gbm_bo_get_offset(*mut c_void, i32) -> u32;
    gbm_bo_get_modifier(*mut c_void) -> u64;
    gbm_bo_get_format(*mut c_void) -> u32;
});

pub const ARGB8888: u32 = u32::from_le_bytes(*b"AR24");
pub const MOD_LINEAR: u64 = 0;
pub const USE_RENDERING: u32 = 1 << 2;

fn gbm() -> Option<&'static Gbm> {
    static T: OnceLock<Option<Gbm>> = OnceLock::new();
    T.get_or_init(|| Gbm::load(&Library::open(c"libgbm.so.1")?)).as_ref()
}

pub fn loaded() -> bool {
    gbm().is_some()
}

pub fn render_nodes() -> Vec<PathBuf> {
    let mut nodes: Vec<PathBuf> = std::fs::read_dir("/dev/dri").into_iter().flatten().flatten()
        .map(|e| e.path()).filter(|p| p.file_name().is_some_and(|n| n.to_string_lossy().starts_with("renderD"))).collect();
    nodes.sort();
    nodes
}

pub fn render_node_of(dev: libc::dev_t) -> Option<PathBuf> {
    let dir = format!("/sys/dev/char/{}:{}/device/drm", libc::major(dev), libc::minor(dev));
    std::fs::read_dir(dir).ok()?.flatten().map(|e| e.file_name())
        .find(|n| n.to_string_lossy().starts_with("renderD")).map(|n| Path::new("/dev/dri").join(n))
}

pub fn open_node(path: &Path) -> Option<OwnedFd> {
    std::fs::OpenOptions::new().read(true).write(true).open(path).ok().map(Into::into)
}

pub struct Device {
    gbm: &'static Gbm,
    device: *mut c_void,
    _node: Option<OwnedFd>,
    shown: std::cell::Cell<bool>,
}

impl Device {
    pub fn new(node: OwnedFd) -> Option<Rc<Device>> {
        let gbm = gbm()?;
        // SAFETY: an open descriptor, kept with the device for as long as it lives; GBM does not close it.
        let device = unsafe { (gbm.gbm_create_device)(node.as_raw_fd()) };
        (!device.is_null()).then(|| Rc::new(Device { gbm, device, _node: Some(node), shown: std::cell::Cell::new(false) }))
    }

    pub(super) fn raw(&self) -> *mut c_void {
        self.shown.set(true);
        self.device
    }

    pub fn backend(&self) -> String {
        // SAFETY: this value's own device; GBM answers null or a NUL-terminated string it keeps, copied out at once.
        unsafe {
            let at = (self.gbm.gbm_device_get_backend_name)(self.device);
            if at.is_null() { String::new() } else { CStr::from_ptr(at).to_string_lossy().into_owned() }
        }
    }

    pub fn supports(&self, format: u32, usage: u32) -> bool {
        // SAFETY: this value's own device and plain numbers.
        unsafe { (self.gbm.gbm_device_is_format_supported)(self.device, format, usage) != 0 }
    }

    pub fn buffer(self: &Rc<Self>, w: u32, h: u32, format: u32, modifiers: &[u64]) -> Option<Buffer> {
        let g = self.gbm;
        // SAFETY: this value's own device, plain numbers, and a slice whose length goes with it.
        let bo = unsafe {
            if modifiers.is_empty() {
                (g.gbm_bo_create)(self.device, w, h, format, USE_RENDERING)
            } else {
                (g.gbm_bo_create_with_modifiers)(self.device, w, h, format, modifiers.as_ptr(), modifiers.len().try_into().ok()?)
            }
        };
        (!bo.is_null()).then(|| Buffer { device: self.clone(), bo, width: w, height: h })
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        if self.shown.get() {
            std::mem::forget(self._node.take());
            return;
        }
        // SAFETY: this value's own device; every buffer holds the device, so none is left.
        unsafe { (self.gbm.gbm_device_destroy)(self.device) };
    }
}

pub struct Buffer {
    device: Rc<Device>,
    bo: *mut c_void,
    pub width: u32,
    pub height: u32,
}

pub struct Plane {
    pub fd: OwnedFd,
    pub stride: u32,
    pub offset: u32,
}

pub struct Dmabuf {
    pub width: u32,
    pub height: u32,
    pub format: u32,
    pub modifier: u64,
    pub planes: Vec<Plane>,
}

impl Buffer {
    pub fn export(&self) -> Option<Dmabuf> {
        let g = self.device.gbm;
        // SAFETY: this value's own live buffer.
        let (count, format, modifier) = unsafe { ((g.gbm_bo_get_plane_count)(self.bo), (g.gbm_bo_get_format)(self.bo), (g.gbm_bo_get_modifier)(self.bo)) };
        let planes = (0..count).map(|i| {
            // SAFETY: this buffer and a plane below its count.
            let (fd, stride, offset) = unsafe { ((g.gbm_bo_get_fd_for_plane)(self.bo, i), (g.gbm_bo_get_stride_for_plane)(self.bo, i), (g.gbm_bo_get_offset)(self.bo, i)) };
            // SAFETY: a fresh descriptor GBM hands over to the caller, checked before it is owned.
            (fd >= 0).then(|| Plane { fd: unsafe { OwnedFd::from_raw_fd(fd) }, stride, offset })
        }).collect::<Option<Vec<_>>>()?;
        (!planes.is_empty()).then_some(Dmabuf { width: self.width, height: self.height, format, modifier, planes })
    }
}

pub fn wait_for(buffer: BorrowedFd, fence: OwnedFd) -> bool {
    const IMPORT: u64 = 0x4008_6203;
    #[repr(C)]
    struct Import {
        flags: u32,
        fd: i32,
    }
    let mut arg = Import { flags: 2, fd: fence.as_raw_fd() };
    // SAFETY: an open descriptor and a struct of the size and layout the request names, which the kernel only
    // reads; the fence stays open across the call and is closed after.
    unsafe { libc::ioctl(buffer.as_raw_fd(), IMPORT as _, &mut arg as *mut Import) == 0 }
}

#[cfg(test)]
pub fn read_linear(d: &Dmabuf) -> Option<Vec<u8>> {
    const SYNC: u64 = 0x4008_6200;
    let (mut start, mut end) = (1u64, 5u64);
    let p = d.planes.first()?;
    let (fd, len) = (p.fd.as_raw_fd(), (p.offset + p.stride * d.height) as usize);
    // SAFETY: an open dmabuf and an 8-byte flags word for each sync; `len` bytes of it mapped for reading only,
    // copied out and unmapped before the sync is ended.
    unsafe {
        if libc::ioctl(fd, SYNC as _, &mut start as *mut u64) != 0 {
            return None;
        }
        let map = libc::mmap(std::ptr::null_mut(), len, libc::PROT_READ, libc::MAP_SHARED, fd, 0);
        let bytes = (map != libc::MAP_FAILED).then(|| std::slice::from_raw_parts(map as *const u8, len).to_vec());
        if bytes.is_some() {
            libc::munmap(map, len);
        }
        libc::ioctl(fd, SYNC as _, &mut end as *mut u64);
        bytes
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        // SAFETY: this value's own buffer, on a device it still holds.
        unsafe { (self.device.gbm.gbm_bo_destroy)(self.bo) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::MetadataExt;

    #[test]
    fn a_missing_library_or_node_is_none() {
        assert!(Gbm::load(&Library::open(c"libc.so.6").unwrap()).is_none());
        assert!(open_node(Path::new("/dev/dri/renderD999")).is_none());
        assert!(render_node_of(libc::makedev(999, 999)).is_none());
        assert_eq!(ARGB8888, 0x3432_5241);
    }

    #[test]
    #[ignore = "needs a GPU and Mesa"]
    fn a_buffer_is_made_and_exported() {
        let card = std::fs::metadata("/dev/dri/card0").unwrap().rdev();
        let node = render_node_of(card).expect("no render node for card0");
        assert_eq!(render_nodes().first(), Some(&node));
        let dev = Device::new(open_node(&node).unwrap()).expect("no GBM device");
        println!("node {node:?}, backend {:?}, ARGB8888 renderable {}", dev.backend(), dev.supports(ARGB8888, USE_RENDERING));
        for mods in [&[][..], &[MOD_LINEAR]] {
            let d = dev.buffer(256, 128, ARGB8888, mods).expect("no buffer").export().expect("no export");
            let p: Vec<_> = d.planes.iter().map(|p| (p.fd.as_raw_fd(), p.stride, p.offset)).collect();
            println!("asked {mods:x?}: {}x{} format {:#x} modifier {:#x}, planes (fd, stride, offset) {p:?}", d.width, d.height, d.format, d.modifier);
            assert_eq!(d.format, ARGB8888);
            assert!(d.planes[0].stride >= 256 * 4);
            assert!(mods.is_empty() || d.modifier == MOD_LINEAR);
        }
    }
}
