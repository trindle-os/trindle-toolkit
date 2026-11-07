//! A surface inside a window that a program draws into itself, with the GPU (EGL, Vulkan) or a video decoder: a Wayland subsurface.

use std::rc::Rc;

use wayland_client::protocol::{wl_subsurface, wl_surface};
use wayland_client::Connection;
#[cfg(feature = "gpu")]
use {std::ffi::c_void, std::ptr::NonNull, wayland_client::Proxy};

use super::{Display, Window};

pub struct GpuSurface {
    surface: wl_surface::WlSurface,
    sub: wl_subsurface::WlSubsurface,
    parent: wl_surface::WlSurface,
    display: Rc<Display>,
}

impl Window {
    pub fn gpu_surface(&self, at: (f32, f32)) -> Option<GpuSurface> {
        let d = &self.display;
        let qh = d.queue.borrow().handle();
        let id = d.next.replace(d.next.get() + 1);
        let mut state = d.state.borrow_mut();
        let parent = state.win(self.id)?.surface.clone();
        let subsurfaces = state.subsurfaces.clone()?;
        let surface = state.compositor.create_surface(&qh, id);
        let sub = subsurfaces.get_subsurface(&surface, &parent, &qh, ());
        let nothing = state.compositor.create_region(&qh, ());
        surface.set_input_region(Some(&nothing));
        nothing.destroy();
        sub.set_desync();
        drop(state);
        let g = GpuSurface { surface, sub, parent, display: d.clone() };
        g.place(at);
        Some(g)
    }
}

impl GpuSurface {
    pub fn surface(&self) -> &wl_surface::WlSurface {
        &self.surface
    }

    pub fn connection(&self) -> &Connection {
        &self.display.conn
    }

    pub fn place(&self, at: (f32, f32)) {
        self.sub.set_position(at.0.round() as i32, at.1.round() as i32);
        self.parent.commit();
        let _ = self.display.conn.flush();
    }

    pub fn below(&self) {
        self.sub.place_below(&self.parent);
        self.parent.commit();
        let _ = self.display.conn.flush();
    }
}

#[cfg(feature = "gpu")]
impl GpuSurface {
    pub fn display_ptr(&self) -> NonNull<c_void> {
        NonNull::new(self.display.conn.backend().display_ptr().cast()).expect("libwayland's display is never null")
    }

    pub fn surface_ptr(&self) -> NonNull<c_void> {
        NonNull::new(self.surface.id().as_ptr().cast()).expect("the surface lives as long as self")
    }
}

impl Drop for GpuSurface {
    fn drop(&mut self) {
        self.sub.destroy();
        self.surface.destroy();
        let _ = self.display.conn.flush();
    }
}
