//! OpenGL ES on the GPU: libEGL and libGLESv2 opened at run time, and a context on EGL's surfaceless platform.
#![cfg_attr(not(test), allow(dead_code))]

use super::gbm::{Device, Dmabuf};
use std::ffi::{c_char, c_void, CStr, CString};
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
use std::rc::Rc;
use std::ptr::null_mut;
use std::sync::OnceLock;

pub(super) struct Library(*mut c_void);

impl Library {
    pub(super) fn open(name: &CStr) -> Option<Library> {
        // SAFETY: a NUL-terminated name; the handle is never passed to dlclose.
        let handle = unsafe { libc::dlopen(name.as_ptr(), libc::RTLD_NOW | libc::RTLD_LOCAL) };
        (!handle.is_null()).then_some(Library(handle))
    }

    pub(super) fn symbol(&self, name: &str) -> Option<*mut c_void> {
        let name = CString::new(name).ok()?;
        // SAFETY: a live handle from dlopen and a NUL-terminated name.
        let at = unsafe { libc::dlsym(self.0, name.as_ptr()) };
        (!at.is_null()).then_some(at)
    }
}

macro_rules! table {
    ($name:ident { $($f:ident($($t:ty),*) $(-> $r:ty)?;)* }) => {
        #[allow(non_snake_case)]
        struct $name { $($f: unsafe extern "C" fn($($t),*) $(-> $r)?,)* }

        impl $name {
            #[allow(dead_code)]
            fn load(lib: &super::gl::Library) -> Option<$name> {
                Self::load_with(|n| lib.symbol(n))
            }

            fn load_with(find: impl Fn(&str) -> Option<*mut std::ffi::c_void>) -> Option<$name> {
                Some($name { $($f: {
                    let at = find(stringify!($f))?;
                    // SAFETY: the symbol has this C type, as the Khronos and GBM headers declare it.
                    unsafe { std::mem::transmute::<*mut std::ffi::c_void, unsafe extern "C" fn($($t),*) $(-> $r)?>(at) }
                },)* })
            }
        }
    };
}
pub(super) use table;

table!(Egl {
    eglGetPlatformDisplay(u32, *mut c_void, *const isize) -> *mut c_void;
    eglInitialize(*mut c_void, *mut i32, *mut i32) -> u32;
    eglTerminate(*mut c_void) -> u32;
    eglQueryString(*mut c_void, i32) -> *const c_char;
    eglBindAPI(u32) -> u32;
    eglCreateContext(*mut c_void, *mut c_void, *mut c_void, *const i32) -> *mut c_void;
    eglDestroyContext(*mut c_void, *mut c_void) -> u32;
    eglMakeCurrent(*mut c_void, *mut c_void, *mut c_void, *mut c_void) -> u32;
    eglGetCurrentContext() -> *mut c_void;
    eglGetProcAddress(*const c_char) -> *mut c_void;
});

table!(Images {
    eglCreateImageKHR(*mut c_void, *mut c_void, u32, *mut c_void, *const i32) -> *mut c_void;
    eglDestroyImageKHR(*mut c_void, *mut c_void) -> u32;
    glEGLImageTargetTexture2DOES(u32, *mut c_void);
});

table!(Fences {
    eglCreateSyncKHR(*mut c_void, u32, *const i32) -> *mut c_void;
    eglDestroySyncKHR(*mut c_void, *mut c_void) -> u32;
    eglDupNativeFenceFDANDROID(*mut c_void, *mut c_void) -> i32;
});

table!(Gles {
    glGetString(u32) -> *const c_char;
    glGetError() -> u32;
    glClearColor(f32, f32, f32, f32);
    glClear(u32);
    glFinish();
    glFlush();
    glReadPixels(i32, i32, i32, i32, u32, u32, *mut c_void);
    glGenFramebuffers(i32, *mut u32);
    glDeleteFramebuffers(i32, *const u32);
    glBindFramebuffer(u32, u32);
    glCheckFramebufferStatus(u32) -> u32;
    glGenRenderbuffers(i32, *mut u32);
    glDeleteRenderbuffers(i32, *const u32);
    glBindRenderbuffer(u32, u32);
    glRenderbufferStorage(u32, u32, i32, i32);
    glFramebufferRenderbuffer(u32, u32, u32, u32);
    glViewport(i32, i32, i32, i32);
    glScissor(i32, i32, i32, i32);
    glEnable(u32);
    glDisable(u32);
    glBlendFunc(u32, u32);
    glActiveTexture(u32);
    glBindTexture(u32, u32);
    glTexParameteri(u32, u32, i32);
    glGenTextures(i32, *mut u32);
    glDeleteTextures(i32, *const u32);
    glTexImage2D(u32, i32, i32, i32, i32, i32, u32, u32, *const c_void);
    glTexSubImage2D(u32, i32, i32, i32, i32, i32, u32, u32, *const c_void);
    glCopyTexSubImage2D(u32, i32, i32, i32, i32, i32, i32, i32);
    glPixelStorei(u32, i32);
    glFramebufferTexture2D(u32, u32, u32, u32, i32);
    glCreateShader(u32) -> u32;
    glShaderSource(u32, i32, *const *const c_char, *const i32);
    glCompileShader(u32);
    glGetShaderiv(u32, u32, *mut i32);
    glGetShaderInfoLog(u32, i32, *mut i32, *mut c_char);
    glDeleteShader(u32);
    glCreateProgram() -> u32;
    glAttachShader(u32, u32);
    glLinkProgram(u32);
    glGetProgramiv(u32, u32, *mut i32);
    glGetProgramInfoLog(u32, i32, *mut i32, *mut c_char);
    glDeleteProgram(u32);
    glUseProgram(u32);
    glGetUniformLocation(u32, *const c_char) -> i32;
    glUniform1i(i32, i32);
    glUniform2f(i32, f32, f32);
    glGenBuffers(i32, *mut u32);
    glBindBuffer(u32, u32);
    glBufferData(u32, isize, *const c_void, u32);
    glGenVertexArrays(i32, *mut u32);
    glBindVertexArray(u32);
    glEnableVertexAttribArray(u32);
    glVertexAttribPointer(u32, i32, u32, u8, i32, *const c_void);
    glVertexAttribDivisor(u32, u32);
    glDrawArraysInstanced(u32, i32, i32, i32);
});

const EGL_TRUE: u32 = 1;
const EGL_NONE: i32 = 0x3038;
const EGL_EXTENSIONS: i32 = 0x3055;
const EGL_PLATFORM_SURFACELESS_MESA: u32 = 0x31DD;
const EGL_TRACK_REFERENCES_KHR: isize = 0x3352;
const EGL_OPENGL_ES_API: u32 = 0x30A0;
const EGL_CONTEXT_MAJOR_VERSION: i32 = 0x3098;
const EGL_CONTEXT_MINOR_VERSION: i32 = 0x30FB;
const EGL_PLATFORM_GBM_KHR: u32 = 0x31D7;
const EGL_WIDTH: i32 = 0x3057;
const EGL_HEIGHT: i32 = 0x3056;
const EGL_LINUX_DMA_BUF_EXT: u32 = 0x3270;
const EGL_LINUX_DRM_FOURCC_EXT: i32 = 0x3271;
const EGL_PLANES: [[i32; 3]; 4] = [[0x3272, 0x3273, 0x3274], [0x3275, 0x3276, 0x3277], [0x3278, 0x3279, 0x327A], [0x3440, 0x3441, 0x3442]];
const EGL_MODIFIERS: [[i32; 2]; 4] = [[0x3443, 0x3444], [0x3445, 0x3446], [0x3447, 0x3448], [0x3449, 0x344A]];
const EGL_SYNC_NATIVE_FENCE_ANDROID: u32 = 0x3144;

pub const GL_RENDERER: u32 = 0x1F01;
pub const GL_VERSION: u32 = 0x1F02;
const GL_EXTENSIONS: u32 = 0x1F03;
pub const GL_COLOR_BUFFER_BIT: u32 = 0x4000;
pub const GL_FRAMEBUFFER: u32 = 0x8D40;
pub const GL_RENDERBUFFER: u32 = 0x8D41;
pub const GL_FRAMEBUFFER_COMPLETE: u32 = 0x8CD5;
pub const GL_COLOR_ATTACHMENT0: u32 = 0x8CE0;
pub const GL_RGBA8: u32 = 0x8058;
const GL_RGBA: u32 = 0x1908;
const GL_UNSIGNED_BYTE: u32 = 0x1401;
pub const GL_TEXTURE_2D: u32 = 0x0DE1;
pub const GL_TEXTURE0: u32 = 0x84C0;
pub const GL_BLEND: u32 = 0x0BE2;
pub const GL_SCISSOR_TEST: u32 = 0x0C11;
pub const GL_ONE: u32 = 1;
pub const GL_ONE_MINUS_SRC_ALPHA: u32 = 0x0303;
const GL_R8: u32 = 0x8229;
const GL_RED: u32 = 0x1903;
const GL_TEXTURE_MAG_FILTER: u32 = 0x2800;
const GL_TEXTURE_MIN_FILTER: u32 = 0x2801;
const GL_TEXTURE_WRAP_S: u32 = 0x2802;
const GL_TEXTURE_WRAP_T: u32 = 0x2803;
const GL_NEAREST: i32 = 0x2600;
const GL_CLAMP_TO_EDGE: i32 = 0x812F;
const GL_UNPACK_ROW_LENGTH: u32 = 0x0CF2;
const GL_UNPACK_ALIGNMENT: u32 = 0x0CF5;
const GL_VERTEX_SHADER: u32 = 0x8B31;
const GL_FRAGMENT_SHADER: u32 = 0x8B30;
const GL_COMPILE_STATUS: u32 = 0x8B81;
const GL_LINK_STATUS: u32 = 0x8B82;
const GL_ARRAY_BUFFER: u32 = 0x8892;
const GL_STREAM_DRAW: u32 = 0x88E0;
const GL_FLOAT: u32 = 0x1406;
const GL_TRIANGLES: u32 = 0x0004;
const UPLOAD_UNIT: u32 = 3;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Texel {
    R8,
    Rgba8,
}

impl Texel {
    fn bytes(self) -> usize {
        match self { Texel::R8 => 1, Texel::Rgba8 => 4 }
    }

    fn formats(self) -> (u32, u32) {
        match self { Texel::R8 => (GL_R8, GL_RED), Texel::Rgba8 => (GL_RGBA8, GL_RGBA) }
    }
}

pub struct Instances {
    vao: u32,
    vbo: u32,
    groups: usize,
}

fn egl() -> Option<&'static Egl> {
    static T: OnceLock<Option<Egl>> = OnceLock::new();
    T.get_or_init(|| Egl::load(&Library::open(c"libEGL.so.1")?)).as_ref()
}

fn gles() -> Option<&'static Gles> {
    static T: OnceLock<Option<Gles>> = OnceLock::new();
    T.get_or_init(|| Gles::load(&Library::open(c"libGLESv2.so.2")?)).as_ref()
}

fn by_name(egl: &'static Egl) -> impl Fn(&str) -> Option<*mut c_void> {
    move |name| {
        let name = CString::new(name).ok()?;
        // SAFETY: a NUL-terminated name EGL only reads.
        let at = unsafe { (egl.eglGetProcAddress)(name.as_ptr()) };
        (!at.is_null()).then_some(at)
    }
}

fn has(list: &str, name: &str) -> bool {
    list.split_ascii_whitespace().any(|e| e == name)
}

pub fn libraries() -> bool {
    egl().is_some() && gles().is_some() && super::gbm::loaded()
}

pub fn is_software(renderer: &str) -> bool {
    renderer.contains("llvmpipe") || renderer.contains("softpipe")
}

thread_local! {
    static DEPTH: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

static OWNER: std::sync::Mutex<bool> = std::sync::Mutex::new(false);
static FREE: std::sync::Condvar = std::sync::Condvar::new();

pub(crate) struct Exclusive;

impl Exclusive {
    pub(crate) fn take() -> Exclusive {
        if DEPTH.get() == 0 {
            let mut held = OWNER.lock().unwrap_or_else(|e| e.into_inner());
            while *held {
                held = FREE.wait(held).unwrap_or_else(|e| e.into_inner());
            }
            *held = true;
        }
        DEPTH.set(DEPTH.get() + 1);
        Exclusive
    }
}

impl Drop for Exclusive {
    fn drop(&mut self) {
        DEPTH.set(DEPTH.get() - 1);
        if DEPTH.get() == 0 {
            *OWNER.lock().unwrap_or_else(|e| e.into_inner()) = false;
            FREE.notify_one();
        }
    }
}

pub struct Context {
    egl: &'static Egl,
    gl: &'static Gles,
    display: *mut c_void,
    context: *mut c_void,
    terminate: bool,
    images: Option<Images>,
    fences: Option<Fences>,
    _device: Option<Rc<Device>>,
    #[cfg(test)]
    _alone: Exclusive,
}

impl Context {
    pub fn new() -> Option<Context> {
        Context::on(EGL_PLATFORM_SURFACELESS_MESA, "EGL_MESA_platform_surfaceless", None)
    }

    pub fn on_gbm(device: Rc<Device>) -> Option<Context> {
        let c = Context::on(EGL_PLATFORM_GBM_KHR, "EGL_KHR_platform_gbm", Some(device))?;
        c.images.is_some().then_some(c)
    }

    fn on(platform: u32, extension: &str, device: Option<Rc<Device>>) -> Option<Context> {
        let _one = Exclusive::take();
        let (egl, gl) = (egl()?, gles()?);
        let client = query(egl, null_mut(), EGL_EXTENSIONS)?;
        if !has(&client, extension) {
            return None;
        }
        let terminate = has(&client, "EGL_KHR_display_reference");
        let attribs = if terminate { [EGL_TRACK_REFERENCES_KHR, EGL_TRUE as isize, EGL_NONE as isize] } else { [EGL_NONE as isize; 3] };
        let native = device.as_ref().map_or(null_mut(), |d| d.raw());
        // SAFETY: no native display for the surfaceless platform, or a live GBM device kept with the context
        // for the GBM one; the attribute list ends in EGL_NONE.
        let display = unsafe { (egl.eglGetPlatformDisplay)(platform, native, attribs.as_ptr()) };
        let (mut major, mut minor) = (0, 0);
        // SAFETY: a display EGL just gave, and two ints owned here for the version.
        if display.is_null() || unsafe { (egl.eglInitialize)(display, &mut major, &mut minor) } != EGL_TRUE {
            return None;
        }
        let ext = query(egl, display, EGL_EXTENSIONS)?;
        let images = (has(&ext, "EGL_EXT_image_dma_buf_import_modifiers") && has(&ext, "EGL_KHR_image_base")).then(|| Images::load_with(by_name(egl))).flatten();
        let fences = (has(&ext, "EGL_ANDROID_native_fence_sync") && has(&ext, "EGL_KHR_fence_sync")).then(|| Fences::load_with(by_name(egl))).flatten();
        let mut c = Context { egl, gl, display, context: null_mut(), terminate, images, fences, _device: device,
                       #[cfg(test)]
                       _alone: _one };
        if !has(&ext, "EGL_KHR_surfaceless_context") || !has(&ext, "EGL_KHR_no_config_context") {
            return None;
        }
        let attribs = [EGL_CONTEXT_MAJOR_VERSION, 3, EGL_CONTEXT_MINOR_VERSION, 0, EGL_NONE];
        // SAFETY: plain numbers, then this display, no config (EGL_KHR_no_config_context), no sharing and
        // an attribute list ending in EGL_NONE.
        c.context = unsafe {
            if (egl.eglBindAPI)(EGL_OPENGL_ES_API) != EGL_TRUE {
                return None;
            }
            (egl.eglCreateContext)(display, null_mut(), null_mut(), attribs.as_ptr())
        };
        if c.context.is_null() || !c.make_current() {
            return None;
        }
        if !has(&c.string(GL_EXTENSIONS), "GL_OES_EGL_image") {
            c.images = None;
        }
        Some(c)
    }

    pub fn import(&self, d: &Dmabuf, modifier: Option<u64>) -> Option<u32> {
        let ext = self.images.as_ref()?;
        if d.planes.len() > 4 {
            return None;
        }
        let mut attribs = vec![EGL_WIDTH, d.width as i32, EGL_HEIGHT, d.height as i32, EGL_LINUX_DRM_FOURCC_EXT, d.format as i32];
        for (i, p) in d.planes.iter().enumerate() {
            attribs.extend([EGL_PLANES[i][0], p.fd.as_raw_fd(), EGL_PLANES[i][1], p.offset as i32, EGL_PLANES[i][2], p.stride as i32]);
            if let Some(m) = modifier {
                attribs.extend([EGL_MODIFIERS[i][0], m as u32 as i32, EGL_MODIFIERS[i][1], (m >> 32) as u32 as i32]);
            }
        }
        attribs.push(EGL_NONE);
        // SAFETY: this value's display, no context and no client buffer, as EGL_LINUX_DMA_BUF_EXT asks, and an
        // attribute list ending in EGL_NONE whose descriptors `d` keeps open; EGL takes its own hold on them.
        let image = unsafe { (ext.eglCreateImageKHR)(self.display, null_mut(), EGL_LINUX_DMA_BUF_EXT, null_mut(), attribs.as_ptr()) };
        if image.is_null() {
            return None;
        }
        let mut n = 0;
        // SAFETY: GL writes one name into `n`; the image is live until destroyed here, after the texture has
        // taken its own hold on the buffer.
        unsafe {
            (self.gl.glGenTextures)(1, &mut n);
            (self.gl.glActiveTexture)(GL_TEXTURE0 + UPLOAD_UNIT);
            (self.gl.glBindTexture)(GL_TEXTURE_2D, n);
            (ext.glEGLImageTargetTexture2DOES)(GL_TEXTURE_2D, image);
            (ext.eglDestroyImageKHR)(self.display, image);
        }
        if self.error() != 0 {
            self.delete_texture(n);
            return None;
        }
        Some(n)
    }

    pub fn fence(&self) -> Option<OwnedFd> {
        let Some(f) = &self.fences else {
            self.flush();
            return None;
        };
        let attribs = [EGL_NONE];
        // SAFETY: this value's display and an attribute list ending in EGL_NONE; the fence asked for is made at
        // the flush, then duplicated (a descriptor handed to the caller, checked before it is owned) and let go.
        unsafe {
            let sync = (f.eglCreateSyncKHR)(self.display, EGL_SYNC_NATIVE_FENCE_ANDROID, attribs.as_ptr());
            self.flush();
            if sync.is_null() {
                return None;
            }
            let fd = (f.eglDupNativeFenceFDANDROID)(self.display, sync);
            (f.eglDestroySyncKHR)(self.display, sync);
            (fd >= 0).then(|| OwnedFd::from_raw_fd(fd))
        }
    }

    pub fn make_current(&self) -> bool {
        // SAFETY: this value's own display and context; no surfaces.
        unsafe { (self.egl.eglMakeCurrent)(self.display, null_mut(), null_mut(), self.context) == EGL_TRUE }
    }

    pub fn string(&self, name: u32) -> String {
        // SAFETY: GL answers null or a NUL-terminated string it keeps; it is copied out at once.
        unsafe {
            let at = (self.gl.glGetString)(name);
            if at.is_null() { String::new() } else { CStr::from_ptr(at).to_string_lossy().into_owned() }
        }
    }

    pub fn software(&self) -> bool {
        is_software(&self.string(GL_RENDERER))
    }

    pub fn read_rgba(&self, x: i32, y: i32, w: u16, h: u16) -> Vec<u8> {
        let mut px = vec![0u8; usize::from(w) * usize::from(h) * 4];
        // SAFETY: RGBA bytes under the default pack state fill exactly w * h * 4 bytes. Nothing in this
        // module sets the pack state or binds a pack buffer; adding glPixelStorei or a GL_PIXEL_PACK_BUFFER
        // bind would break this.
        unsafe { (self.gl.glReadPixels)(x, y, w.into(), h.into(), GL_RGBA, GL_UNSIGNED_BYTE, px.as_mut_ptr().cast()) };
        px
    }

    pub fn framebuffer(&self) -> u32 {
        let mut n = 0;
        // SAFETY: GL writes one name into `n`.
        unsafe { (self.gl.glGenFramebuffers)(1, &mut n) };
        n
    }

    pub fn delete_framebuffer(&self, n: u32) {
        // SAFETY: GL reads one name from `n`.
        unsafe { (self.gl.glDeleteFramebuffers)(1, &n) };
    }

    pub fn renderbuffer(&self) -> u32 {
        let mut n = 0;
        // SAFETY: GL writes one name into `n`.
        unsafe { (self.gl.glGenRenderbuffers)(1, &mut n) };
        n
    }

    pub fn delete_renderbuffer(&self, n: u32) {
        // SAFETY: GL reads one name from `n`.
        unsafe { (self.gl.glDeleteRenderbuffers)(1, &n) };
    }

    pub fn texture(&self, texel: Texel, w: u32, h: u32) -> u32 {
        let (inner, format) = texel.formats();
        let mut n = 0;
        // SAFETY: GL writes one name into `n`; a null pointer for the data asks for storage only, nothing is read.
        unsafe {
            (self.gl.glGenTextures)(1, &mut n);
            (self.gl.glActiveTexture)(GL_TEXTURE0 + UPLOAD_UNIT);
            (self.gl.glBindTexture)(GL_TEXTURE_2D, n);
            (self.gl.glTexImage2D)(GL_TEXTURE_2D, 0, inner as i32, w as i32, h as i32, 0, format, GL_UNSIGNED_BYTE, std::ptr::null());
        }
        for (name, v) in [(GL_TEXTURE_MIN_FILTER, GL_NEAREST), (GL_TEXTURE_MAG_FILTER, GL_NEAREST), (GL_TEXTURE_WRAP_S, GL_CLAMP_TO_EDGE), (GL_TEXTURE_WRAP_T, GL_CLAMP_TO_EDGE)] {
            self.tex_parameter(GL_TEXTURE_2D, name, v);
        }
        n
    }

    pub fn delete_texture(&self, n: u32) {
        // SAFETY: GL reads one name from `n`.
        unsafe { (self.gl.glDeleteTextures)(1, &n) };
    }

    pub fn upload(&self, n: u32, texel: Texel, (x, y, w, h): (u32, u32, u32, u32), data: &[u8], stride: usize) {
        let (w8, h8) = (w as usize, h as usize);
        if w == 0 || h == 0 || stride < w8 || data.len() < ((h8 - 1) * stride + w8) * texel.bytes() {
            return;
        }
        let (_, format) = texel.formats();
        // SAFETY: with alignment 1 and the row length `stride`, GL reads h - 1 strides and w more texels,
        // which the check above keeps inside `data`; both are put back to GL's defaults at once, so no
        // other call (glReadPixels among them) ever sees them.
        unsafe {
            (self.gl.glActiveTexture)(GL_TEXTURE0 + UPLOAD_UNIT);
            (self.gl.glBindTexture)(GL_TEXTURE_2D, n);
            (self.gl.glPixelStorei)(GL_UNPACK_ALIGNMENT, 1);
            (self.gl.glPixelStorei)(GL_UNPACK_ROW_LENGTH, stride as i32);
            (self.gl.glTexSubImage2D)(GL_TEXTURE_2D, 0, x as i32, y as i32, w as i32, h as i32, format, GL_UNSIGNED_BYTE, data.as_ptr().cast());
            (self.gl.glPixelStorei)(GL_UNPACK_ROW_LENGTH, 0);
            (self.gl.glPixelStorei)(GL_UNPACK_ALIGNMENT, 4);
        }
    }

    pub fn copy_texture(&self, from: u32, to: u32, (w, h): (u32, u32)) -> bool {
        let fbo = self.framebuffer();
        self.bind_framebuffer(GL_FRAMEBUFFER, fbo);
        self.framebuffer_texture(GL_FRAMEBUFFER, GL_COLOR_ATTACHMENT0, GL_TEXTURE_2D, from, 0);
        let ok = self.framebuffer_status(GL_FRAMEBUFFER) == GL_FRAMEBUFFER_COMPLETE;
        if ok {
            // SAFETY: numbers only; the copy reads the bound framebuffer and writes the texture bound on the upload unit.
            unsafe {
                (self.gl.glActiveTexture)(GL_TEXTURE0 + UPLOAD_UNIT);
                (self.gl.glBindTexture)(GL_TEXTURE_2D, to);
                (self.gl.glCopyTexSubImage2D)(GL_TEXTURE_2D, 0, 0, 0, 0, 0, w as i32, h as i32);
            }
        }
        self.bind_framebuffer(GL_FRAMEBUFFER, 0);
        self.delete_framebuffer(fbo);
        ok && self.error() == 0
    }

    pub fn program(&self, vertex: &str, fragment: &str) -> Result<u32, String> {
        let p = self.create_program();
        for (kind, text) in [(GL_VERTEX_SHADER, vertex), (GL_FRAGMENT_SHADER, fragment)] {
            let s = self.create_shader(kind);
            let (at, len) = (text.as_ptr().cast::<c_char>(), text.len() as i32);
            // SAFETY: one string given with its length, so no NUL is needed; GL copies it before returning.
            unsafe { (self.gl.glShaderSource)(s, 1, &at, &len) };
            self.compile_shader(s);
            let ok = self.status(s, false);
            self.attach_shader(p, s);
            self.delete_shader(s);
            ok?;
        }
        self.link_program(p);
        self.status(p, true).inspect_err(|_| self.delete_program(p))?;
        Ok(p)
    }

    fn status(&self, n: u32, program: bool) -> Result<(), String> {
        let (mut ok, mut log, mut len) = (0, vec![0u8; 4096], 0);
        // SAFETY: GL writes one int into `ok`, and at most log.len() bytes into `log` with their count in `len`.
        unsafe {
            if program {
                (self.gl.glGetProgramiv)(n, GL_LINK_STATUS, &mut ok);
                (self.gl.glGetProgramInfoLog)(n, log.len() as i32, &mut len, log.as_mut_ptr().cast());
            } else {
                (self.gl.glGetShaderiv)(n, GL_COMPILE_STATUS, &mut ok);
                (self.gl.glGetShaderInfoLog)(n, log.len() as i32, &mut len, log.as_mut_ptr().cast());
            }
        }
        log.truncate(len.clamp(0, 4096) as usize);
        if ok == 1 { Ok(()) } else { Err(String::from_utf8_lossy(&log).into_owned()) }
    }

    pub fn uniform_location(&self, p: u32, name: &str) -> i32 {
        let Ok(name) = CString::new(name) else { return -1 };
        // SAFETY: a NUL-terminated name GL only reads.
        unsafe { (self.gl.glGetUniformLocation)(p, name.as_ptr()) }
    }

    pub fn instances(&self, groups: usize) -> Instances {
        let (mut vao, mut vbo) = (0, 0);
        // SAFETY: GL writes one name into each. The attribute pointers are offsets into `vbo`, bound to
        // GL_ARRAY_BUFFER while they are set, so they never name client memory.
        unsafe {
            (self.gl.glGenVertexArrays)(1, &mut vao);
            (self.gl.glGenBuffers)(1, &mut vbo);
            (self.gl.glBindVertexArray)(vao);
            (self.gl.glBindBuffer)(GL_ARRAY_BUFFER, vbo);
            for g in 0..groups as u32 {
                (self.gl.glEnableVertexAttribArray)(g);
                (self.gl.glVertexAttribPointer)(g, 4, GL_FLOAT, 0, (groups * 16) as i32, (g as usize * 16) as *const c_void);
                (self.gl.glVertexAttribDivisor)(g, 1);
            }
            (self.gl.glBindVertexArray)(0);
        }
        Instances { vao, vbo, groups }
    }

    pub fn draw_instances(&self, i: &Instances, data: &[f32]) {
        let n = data.len() / (i.groups * 4);
        if n == 0 {
            return;
        }
        // SAFETY: GL copies data.len() floats out of `data`; the draw reads only `i`'s own buffer, which
        // holds the n whole instances drawn, as every attribute of `i.vao` points into it.
        unsafe {
            (self.gl.glBindVertexArray)(i.vao);
            (self.gl.glBindBuffer)(GL_ARRAY_BUFFER, i.vbo);
            (self.gl.glBufferData)(GL_ARRAY_BUFFER, std::mem::size_of_val(data) as isize, data.as_ptr().cast(), GL_STREAM_DRAW);
            (self.gl.glDrawArraysInstanced)(GL_TRIANGLES, 0, 6, n as i32);
            (self.gl.glBindVertexArray)(0);
        }
    }
}

macro_rules! plain {
    ($($m:ident = $f:ident($($a:ident: $t:ty),*) $(-> $r:ty)?;)*) => {
        impl Context {
            $(pub fn $m(&self, $($a: $t),*) $(-> $r)? {
                // SAFETY: numbers only; GL checks every name and enum and reports a bad one through
                // glGetError, and with no context current the call does nothing.
                unsafe { (self.gl.$f)($($a),*) }
            })*
        }
    };
}

plain! {
    error = glGetError() -> u32;
    clear_colour = glClearColor(r: f32, g: f32, b: f32, a: f32);
    clear = glClear(mask: u32);
    finish = glFinish();
    flush = glFlush();
    bind_framebuffer = glBindFramebuffer(target: u32, n: u32);
    framebuffer_status = glCheckFramebufferStatus(target: u32) -> u32;
    bind_renderbuffer = glBindRenderbuffer(target: u32, n: u32);
    renderbuffer_storage = glRenderbufferStorage(target: u32, format: u32, w: i32, h: i32);
    framebuffer_renderbuffer = glFramebufferRenderbuffer(target: u32, attachment: u32, rtarget: u32, n: u32);
    framebuffer_texture = glFramebufferTexture2D(target: u32, attachment: u32, ttarget: u32, n: u32, level: i32);
    viewport = glViewport(x: i32, y: i32, w: i32, h: i32);
    scissor = glScissor(x: i32, y: i32, w: i32, h: i32);
    enable = glEnable(cap: u32);
    disable = glDisable(cap: u32);
    blend_func = glBlendFunc(src: u32, dst: u32);
    active_texture = glActiveTexture(unit: u32);
    bind_texture = glBindTexture(target: u32, n: u32);
    tex_parameter = glTexParameteri(target: u32, name: u32, v: i32);
    create_shader = glCreateShader(kind: u32) -> u32;
    compile_shader = glCompileShader(n: u32);
    delete_shader = glDeleteShader(n: u32);
    create_program = glCreateProgram() -> u32;
    attach_shader = glAttachShader(p: u32, s: u32);
    link_program = glLinkProgram(p: u32);
    delete_program = glDeleteProgram(p: u32);
    use_program = glUseProgram(p: u32);
    uniform_int = glUniform1i(at: i32, v: i32);
    uniform_2f = glUniform2f(at: i32, x: f32, y: f32);
}

impl Drop for Context {
    fn drop(&mut self) {
        let _one = Exclusive::take();
        // SAFETY: this value's own display and context; the context is let go first when it is current here.
        unsafe {
            if !self.context.is_null() {
                if (self.egl.eglGetCurrentContext)() == self.context {
                    (self.egl.eglMakeCurrent)(self.display, null_mut(), null_mut(), null_mut());
                }
                (self.egl.eglDestroyContext)(self.display, self.context);
            }
            if self.terminate {
                (self.egl.eglTerminate)(self.display);
            }
        }
    }
}

fn query(egl: &Egl, display: *mut c_void, name: i32) -> Option<String> {
    // SAFETY: called only with null or an initialised display of this module; EGL answers null or a
    // NUL-terminated string it keeps, copied out at once.
    unsafe {
        let at = (egl.eglQueryString)(display, name);
        (!at.is_null()).then(|| CStr::from_ptr(at).to_string_lossy().into_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_missing_library_or_symbol_is_none() {
        assert!(Library::open(c"libtrindle-not-here.so.0").is_none());
        let libc = Library::open(c"libc.so.6").unwrap();
        assert!(libc.symbol("eglInitialize").is_none());
        assert!(Egl::load(&libc).is_none());
        assert!(Gles::load(&libc).is_none());
    }

    #[test]
    fn software_renderers_count_as_no_gpu() {
        assert!(is_software("llvmpipe (LLVM 19.1.7, 256 bits)"));
        assert!(is_software("softpipe"));
        assert!(!is_software("AMD Radeon 760M (radeonsi, gfx1103_r1, LLVM 19.1.7, DRM 3.61)"));
    }

    #[test]
    fn extensions_match_whole_names() {
        assert!(has("EGL_KHR_a EGL_KHR_b", "EGL_KHR_b"));
        assert!(!has("EGL_KHR_ab", "EGL_KHR_a"));
    }

    #[test]
    #[ignore = "needs a GPU and Mesa"]
    fn a_context_clears_and_reads_back() {
        let c = Context::new().expect("no EGL surfaceless context");
        println!("renderer {:?}, version {:?}, software {}", c.string(GL_RENDERER), c.string(GL_VERSION), c.software());
        let (fb, rb) = (c.framebuffer(), c.renderbuffer());
        c.bind_renderbuffer(GL_RENDERBUFFER, rb);
        c.renderbuffer_storage(GL_RENDERBUFFER, GL_RGBA8, 16, 16);
        c.bind_framebuffer(GL_FRAMEBUFFER, fb);
        c.framebuffer_renderbuffer(GL_FRAMEBUFFER, GL_COLOR_ATTACHMENT0, GL_RENDERBUFFER, rb);
        assert_eq!(c.framebuffer_status(GL_FRAMEBUFFER), GL_FRAMEBUFFER_COMPLETE);
        c.clear_colour(1.0, 0.5, 0.0, 1.0);
        c.clear(GL_COLOR_BUFFER_BIT);
        c.finish();
        let px = c.read_rgba(0, 0, 16, 16);
        println!("first pixel {:?}", &px[..4]);
        assert!(px.chunks(4).all(|p| p == [255, 128, 0, 255] || p == [255, 127, 0, 255]), "{:?}", &px[..4]);
        assert_eq!(c.error(), 0);
        c.delete_framebuffer(fb);
        c.delete_renderbuffer(rb);
    }

    #[test]
    fn lock_taken_in_a_thread_local_drop() {
        struct Late;
        impl Drop for Late {
            fn drop(&mut self) {
                drop(Exclusive::take());
            }
        }
        thread_local! { static LATE: Late = const { Late }; }
        let ended = std::thread::spawn(|| {
            LATE.with(|_| ());
            drop(Exclusive::take());
        })
        .join();
        assert!(ended.is_ok());
    }
}
