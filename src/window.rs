//! Windows on screen: ordinary windows the compositor gives a title bar, and panels and overlays on the layer shell, all painted by Trindle itself.

use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};

use tiny_skia::PixmapMut;
use wayland_client::protocol::{
    wl_buffer, wl_compositor, wl_data_device, wl_subcompositor, wl_subsurface, wl_data_device_manager, wl_data_offer, wl_data_source, wl_region,
    wl_keyboard, wl_output, wl_pointer, wl_seat, wl_shm, wl_shm_pool, wl_surface, wl_touch,
};
use wayland_client::{Connection, Dispatch, EventQueue, Proxy, QueueHandle};
use wayland_protocols::wp::cursor_shape::v1::client::{
    wp_cursor_shape_device_v1::WpCursorShapeDeviceV1,
    wp_cursor_shape_manager_v1::WpCursorShapeManagerV1,
};
use wayland_protocols::wp::fractional_scale::v1::client::{
    wp_fractional_scale_manager_v1::WpFractionalScaleManagerV1,
    wp_fractional_scale_v1::WpFractionalScaleV1,
};
use wayland_protocols::wp::primary_selection::zv1::client::{
    zwp_primary_selection_device_manager_v1::ZwpPrimarySelectionDeviceManagerV1,
    zwp_primary_selection_device_v1::ZwpPrimarySelectionDeviceV1,
    zwp_primary_selection_offer_v1::ZwpPrimarySelectionOfferV1,
    zwp_primary_selection_source_v1::ZwpPrimarySelectionSourceV1,
};
use wayland_protocols::wp::text_input::zv3::client::{
    zwp_text_input_manager_v3::ZwpTextInputManagerV3,
    zwp_text_input_v3::ZwpTextInputV3,
};
use wayland_protocols::wp::viewporter::client::{wp_viewport::WpViewport, wp_viewporter::WpViewporter};
use wayland_protocols::xdg::activation::v1::client::xdg_activation_v1::XdgActivationV1;
use wayland_protocols::xdg::decoration::zv1::client::{
    zxdg_decoration_manager_v1::ZxdgDecorationManagerV1,
    zxdg_toplevel_decoration_v1::ZxdgToplevelDecorationV1,
};
use wayland_protocols::xdg::dialog::v1::client::{xdg_dialog_v1::XdgDialogV1, xdg_wm_dialog_v1::XdgWmDialogV1};
use wayland_protocols::wp::idle_inhibit::zv1::client::{zwp_idle_inhibit_manager_v1::ZwpIdleInhibitManagerV1, zwp_idle_inhibitor_v1::ZwpIdleInhibitorV1};
use wayland_protocols::xdg::toplevel_icon::v1::client::{xdg_toplevel_icon_manager_v1::XdgToplevelIconManagerV1, xdg_toplevel_icon_v1::XdgToplevelIconV1};
use wayland_protocols::xdg::foreign::zv2::client::{zxdg_imported_v2::{self, ZxdgImportedV2}, zxdg_importer_v2::ZxdgImporterV2};
use wayland_protocols::xdg::shell::client::{xdg_surface, xdg_toplevel, xdg_wm_base};
use wayland_protocols::xdg::xdg_output::zv1::client::{
    zxdg_output_manager_v1::ZxdgOutputManagerV1,
    zxdg_output_v1::ZxdgOutputV1,
};
use wayland_protocols::ext::session_lock::v1::client::{
    ext_session_lock_manager_v1::ExtSessionLockManagerV1, ext_session_lock_surface_v1::ExtSessionLockSurfaceV1,
};
use wayland_protocols_wlr::layer_shell::v1::client::{
    zwlr_layer_shell_v1::ZwlrLayerShellV1,
    zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
};

mod clip;
mod display;
pub(crate) mod dmabuf;
mod drag;
mod frame;
mod gesture;
mod gpu;
mod handle;
mod inbox;
mod layers;
mod lock;
mod painting;
mod pen;
mod screens;
mod seat;
mod surface_events;
mod typer;
pub use gpu::GpuSurface;
pub use clip::OnClipboard;
pub use lock::{Phase as LockPhase, SessionLock};
#[cfg(test)]
mod stock_tests;
#[cfg(test)]
mod press_tests;
#[cfg(test)]
pub(crate) mod tests;
pub use drag::Dragged;

use clip::write_on_thread;
use display::keyboard_interactivity;
use inbox::Inbox;
use painting::{pixels, show, to_compositor, BEHIND_MOST, MOST_SIDE};
use screens::add_screen;
use surface_events::{has_state, Placed, Unvoid};

use super::event::{Hold, Loop, Post};
use super::input::{Button, Cursor, Input, Keyboard, Typing};
use crate::common::shm;

thread_local! {
    static PRESS: Cell<Option<u32>> = const { Cell::new(None) };
    static CODES: RefCell<std::collections::VecDeque<(u32, u32)>> = const { RefCell::new(std::collections::VecDeque::new()) };
    static WAS: RefCell<std::collections::VecDeque<(u32, u32)>> = const { RefCell::new(std::collections::VecDeque::new()) };
}

const CODES_KEPT: usize = 64;

fn note_code(list: &'static std::thread::LocalKey<RefCell<std::collections::VecDeque<(u32, u32)>>>, serial: u32, code: u32) {
    list.with(|l| {
        let mut l = l.borrow_mut();
        if l.len() >= CODES_KEPT {
            l.pop_front();
        }
        l.push_back((serial, code));
    });
}

fn code_in(list: &'static std::thread::LocalKey<RefCell<std::collections::VecDeque<(u32, u32)>>>, serial: u32) -> Option<u32> {
    list.with(|l| l.borrow().iter().rev().find(|(s, _)| *s == serial).map(|(_, c)| *c))
}

pub fn code_now() -> Option<u32> {
    let serial = PRESS.get()?;
    code_in(&WAS, serial).or_else(|| code_in(&CODES, serial))
}

pub fn press_now() -> Option<u32> {
    PRESS.get()
}

const FIRST_BUTTON: u32 = 0x100;

pub fn key_now() -> bool {
    code_now().is_some_and(|code| code < FIRST_BUTTON)
}

fn press_seen() -> bool {
    OWN.with(|own| match own.borrow().as_ref() {
        None => true,
        Some(seen) => PRESS.get().is_some_and(|serial| seen.has(serial)),
    })
}

pub fn own_key_now() -> bool {
    key_now() && press_seen()
}

#[cfg(any(test, feature = "testing"))]
pub fn note_press_code(serial: u32, code: u32) {
    note_code(&CODES, serial, code);
}

thread_local! {
    static OWN: std::cell::RefCell<Option<crate::common::own_press::Seen>> = const { std::cell::RefCell::new(None) };
    static BY_READER: Cell<bool> = const { Cell::new(false) };
}

pub fn persons_own(act: &str) -> bool {
    use crate::host::{host, Act};
    let listed = host().act(act);
    if listed == Act::Waiting {
        return true;
    }
    let own = listed == Act::PersonsOwn
        && (BY_READER.get() || press_seen());
    if !own {
        REFUSED_ACT.set(true);
    }
    own
}

thread_local! {
    static REFUSED_ACT: Cell<bool> = const { Cell::new(false) };
}

pub fn refused_act() -> bool {
    REFUSED_ACT.replace(false)
}

pub fn refuse() {
    REFUSED_ACT.set(true);
}

pub fn only_your_own() -> &'static str {
    crate::t!("kit-only-your-own")
}

pub fn asked_by_reader() -> bool {
    BY_READER.get()
}

pub fn by_reader<R>(during: impl FnOnce() -> R) -> R {
    let before = BY_READER.replace(crate::host::host().reader_switched_on());
    let out = during();
    BY_READER.set(before);
    out
}

pub fn reader_may_ask(name: &str) -> bool {
    let host = crate::host::host();
    !host.asks_only_the_person(name) || OWN.with(|own| own.borrow().is_none()) || host.reader_switched_on()
}

#[cfg(any(test, feature = "testing"))]
pub fn with_named<R>(told: bool, serial: Option<u32>, during: impl FnOnce() -> R) -> R {
    let mut seen = crate::common::own_press::Seen::default();
    if let Some(serial) = serial {
        seen.note(serial);
    }
    let before = OWN.with(|own| own.replace(told.then_some(seen)));
    let out = during();
    OWN.with(|own| *own.borrow_mut() = before);
    out
}

#[allow(dead_code, non_camel_case_types, unused_unsafe, unused_variables, non_upper_case_globals, non_snake_case,
        unused_imports, missing_docs, clippy::all)]
mod named {
    use wayland_client;
    use wayland_client::protocol::*;

    pub mod __interfaces {
        use wayland_client::backend as wayland_backend;
        use wayland_client::protocol::__interfaces::*;
        wayland_scanner::generate_interfaces!("protocols/own_press.xml");
    }
    use self::__interfaces::*;

    wayland_scanner::generate_client_code!("protocols/own_press.xml");
}
use named::trindle_own_press_v1::{self, TrindleOwnPressV1};

impl Dispatch<TrindleOwnPressV1, ()> for State {
    fn event(state: &mut Self, _: &TrindleOwnPressV1, event: trindle_own_press_v1::Event, _: &(), _: &Connection, _: &QueueHandle<Self>) {
        match event {
            trindle_own_press_v1::Event::Own { serial } => OWN.with(|own| {
                if let Some(seen) = own.borrow_mut().as_mut() {
                    seen.note(serial);
                }
            }),
            trindle_own_press_v1::Event::Refused { surface } => {
                if let Some(id) = win_of(&surface) {
                    state.inbox.push((id, Input::Refused));
                }
            }
            trindle_own_press_v1::Event::Was { serial, code } => note_code(&WAS, serial, code),
        }
    }
}

#[cfg(any(test, feature = "testing"))]
pub fn with_press<R>(serial: Option<u32>, during: impl FnOnce() -> R) -> R {
    let before = PRESS.replace(serial);
    let out = during();
    PRESS.set(before);
    out
}

fn no_nul(text: &str) -> String {
    text.replace('\0', "")
}

pub enum Kind {
    Window { title: String, app_id: String },
    Panel(Panel),
    Lock { screen: String },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Layer {
    Background,
    Bottom,
    Top,
    Overlay,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Keys {
    Never,
    OnClick,
    Always,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Edges(u32);

impl Edges {
    pub const NONE: Edges = Edges(0);
    pub const TOP: Edges = Edges(1);
    pub const BOTTOM: Edges = Edges(2);
    pub const LEFT: Edges = Edges(4);
    pub const RIGHT: Edges = Edges(8);
    pub const ALL: Edges = Edges(15);
}

impl std::ops::BitOr for Edges {
    type Output = Edges;
    fn bitor(self, other: Edges) -> Edges {
        Edges(self.0 | other.0)
    }
}

pub struct Panel {
    pub layer: Layer,
    pub edges: Edges,
    pub keep: i32,
    pub margins: [i32; 4],
    pub keys: Keys,
    pub name: String,
    pub screen: Option<String>,
}

pub type Painter = Box<dyn FnMut(&mut PixmapMut, f32)>;
pub type Patcher = Box<dyn FnMut(&mut PixmapMut, f32) -> Option<Vec<[i32; 4]>>>;

enum Role {
    Window { xdg: xdg_surface::XdgSurface, top: xdg_toplevel::XdgToplevel, title_bar: Option<ZxdgToplevelDecorationV1>,
             names: (String, String) },
    Panel(ZwlrLayerSurfaceV1),
    Lock(ExtSessionLockSurfaceV1, String),
}

struct Buffers {
    memory: shm::Memory,
    buffers: [wl_buffer::WlBuffer; 2],
    size: (u32, u32),
}

struct Win {
    id: u64,
    surface: wl_surface::WlSurface,
    role: Role,
    viewport: Option<WpViewport>,
    fractional: Option<WpFractionalScaleV1>,
    size: (u32, u32),
    proposed: Option<(u32, u32)>,
    on: Vec<u32>,
    scale120: u32,
    configured: bool,
    ack: Option<u32>,
    mapped: bool,
    void: u32,
    hidden: bool,
    dirty: bool,
    waiting: bool,
    sent: Option<std::time::Instant>,
    suspended: bool,
    maximised: bool,
    on_suspended: Option<Box<dyn FnMut(bool)>>,
    told_suspended: bool,
    busy: [bool; 2],
    buffers: Option<Buffers>,
    paint: Option<Painter>,
    on_close: Option<Box<dyn FnMut()>>,
    closing: bool,
    keys: Keys,
    patcher: Option<Patcher>,
    patch: bool,
    opaque: bool,
    drawn: [u64; 2],
    turn: u64,
    moves: bool,
    behind: [Vec<[i32; 4]>; 2],
    on_input: Option<Box<dyn FnMut(Input)>>,
    cursor: Cursor,
    typing: Option<Typing>,
    parent: Option<ZxdgImportedV2>,
    dialog: Option<XdgDialogV1>,
    awake: Option<ZwpIdleInhibitorV1>,
    frame_asked: bool,
    shadow: u32,
    corner: u32,
    solid: bool,
    opaque_for: Option<[u32; 5]>,
    snug: bool,
    geometry_told: Option<(u32, (u32, u32))>,
    compositor_menu: bool,
    gpu: Option<Box<dmabuf::Pool>>,
}

impl Win {
    fn new(id: u64, surface: wl_surface::WlSurface, role: Role, viewport: Option<WpViewport>, fractional: Option<WpFractionalScaleV1>, size: (u32, u32), paint: Painter, keys: Keys) -> Self {
        Win {
            id, surface, role, viewport, fractional, size, proposed: None, on: Vec::new(), scale120: 0, configured: false, ack: None, mapped: false, void: 0, hidden: false,
            dirty: true, waiting: false, sent: None, suspended: false, maximised: false, on_suspended: None, told_suspended: false, busy: [false; 2], buffers: None, paint: Some(paint), on_close: None,
            patcher: None, patch: false, opaque: false, drawn: [0; 2], turn: 0, moves: false, behind: [Vec::new(), Vec::new()],
            closing: false, keys, on_input: None, cursor: Cursor::Arrow, typing: None, parent: None, dialog: None, awake: None, frame_asked: false,
            shadow: 0, corner: 0, solid: false, opaque_for: None, snug: false, geometry_told: None, compositor_menu: true, gpu: None,
        }
    }
}

struct Screen {
    id: u32,
    output: wl_output::WlOutput,
    name: String,
    scale: i32,
    place: [i32; 4],
    placer: Option<ZxdgOutputV1>,
    fresh: bool,
}

struct State {
    compositor: wl_compositor::WlCompositor,
    shm: wl_shm::WlShm,
    dmabuf: Option<dmabuf::Offer>,
    wm: Option<xdg_wm_base::XdgWmBase>,
    layers: Option<ZwlrLayerShellV1>,
    locker: Option<ExtSessionLockManagerV1>,
    lock: Option<lock::Held>,
    title_bars: Option<ZxdgDecorationManagerV1>,
    viewporter: Option<WpViewporter>,
    fractions: Option<WpFractionalScaleManagerV1>,
    places: Option<ZxdgOutputManagerV1>,
    subsurfaces: Option<wl_subcompositor::WlSubcompositor>,
    screens: Vec<Screen>,
    windows: Vec<Win>,
    seat: Option<wl_seat::WlSeat>,
    pointer: Option<wl_pointer::WlPointer>,
    keyboard: Option<wl_keyboard::WlKeyboard>,
    touch: Option<wl_touch::WlTouch>,
    shapes: Option<WpCursorShapeManagerV1>,
    shape: Option<WpCursorShapeDeviceV1>,
    pens: pen::Pens,
    gestures: gesture::Gestures,
    pointer_on: Option<(u64, u32)>,
    pointer_at: (f32, f32),
    scrolled: [f32; 2],
    steps: [f32; 2],
    stopped: bool,
    named: Option<TrindleOwnPressV1>,
    keys_on: Option<u64>,
    layout: Option<Keyboard>,
    held: Option<u32>,
    held_serial: u32,
    held_count: u64,
    start_repeat: bool,
    touches: Vec<(i32, u64)>,
    touched: Vec<(i32, u32)>,
    inbox: Inbox,
    serial: u32,
    clipboard: Option<wl_data_device_manager::WlDataDeviceManager>,
    device: Option<wl_data_device::WlDataDevice>,
    copying: Copying,
    pressed: Option<(u32, Button)>,
    held_in: Option<drag::HeldIn>,
    held_out: Option<drag::HeldOut>,
    pulls: Vec<drag::Pulling>,
    taking: Vec<drag::Taking>,
    input_method: InputMethod,
    activation: Option<XdgActivationV1>,
    vouched: Vec<(u64, String)>,
    pass: Option<String>,
    importer: Option<ZxdgImporterV2>,
    dialogs: Option<XdgWmDialogV1>,
    inhibits: Option<ZwpIdleInhibitManagerV1>,
    icons: Option<XdgToplevelIconManagerV1>,
    screens_changed: bool,
}

#[derive(Default)]
struct Copying {
    selection: Option<wl_data_offer::WlDataOffer>,
    selections: u64,
    copied: Option<wl_data_source::WlDataSource>,
    primary: Option<(ZwpPrimarySelectionDeviceManagerV1, ZwpPrimarySelectionDeviceV1)>,
    primary_offer: Option<ZwpPrimarySelectionOfferV1>,
    primary_copied: Option<ZwpPrimarySelectionSourceV1>,
}

#[derive(Default)]
struct InputMethod {
    typer: Option<ZwpTextInputV3>,
    typer_on: Option<u64>,
    typer_sent: Option<Typing>,
    composing: Composing,
    by_method: bool,
}

#[derive(Default)]
struct Composing {
    delete: (u32, u32),
    commit: String,
    preedit: String,
    caret: Option<(usize, usize)>,
}

impl State {
    fn win(&mut self, id: u64) -> Option<&mut Win> {
        self.windows.iter_mut().find(|w| w.id == id)
    }
}

pub struct Display {
    conn: Connection,
    queue: RefCell<EventQueue<State>>,
    state: RefCell<State>,
    next: Cell<u64>,
    painting: Cell<bool>,
    events: Loop,
    me: Weak<Display>,
    waiting: RefCell<Vec<(u64, Box<dyn FnOnce(Option<String>)>, Hold)>>,
    on_screens: RefCell<Vec<Rc<dyn Fn()>>>,
    on_lock: RefCell<Option<Rc<dyn Fn(lock::Phase)>>>,
    on_keys_held: RefCell<Option<Rc<dyn Fn()>>>,
    back: Post<drag::Back>,
}

pub struct Window {
    id: u64,
    display: Rc<Display>,
}

fn win_of(surface: &wl_surface::WlSurface) -> Option<u64> {
    surface.data::<u64>().copied()
}

macro_rules! ignore {
    ($($t:ty),*) => {$(
        impl Dispatch<$t, ()> for State {
            fn event(_: &mut Self, _: &$t, _: <$t as Proxy>::Event, _: &(), _: &Connection, _: &QueueHandle<Self>) {}
        }
    )*};
}
ignore!(wl_compositor::WlCompositor, wl_shm::WlShm, wl_shm_pool::WlShmPool, ZwlrLayerShellV1, WpViewporter, WpViewport,
        WpFractionalScaleManagerV1, ZxdgOutputManagerV1, ZxdgDecorationManagerV1, WpCursorShapeManagerV1,
        WpCursorShapeDeviceV1, wl_data_device_manager::WlDataDeviceManager, ZwpPrimarySelectionDeviceManagerV1, ZwpTextInputManagerV3, XdgActivationV1,
        wl_region::WlRegion, wl_subcompositor::WlSubcompositor, wl_subsurface::WlSubsurface, ZxdgImporterV2, XdgWmDialogV1, XdgDialogV1, ZwpIdleInhibitManagerV1, ZwpIdleInhibitorV1, XdgToplevelIconV1, XdgToplevelIconManagerV1);

impl Dispatch<ZxdgImportedV2, ()> for State {
    fn event(_: &mut Self, _: &ZxdgImportedV2, _: zxdg_imported_v2::Event, _: &(), _: &Connection, _: &QueueHandle<Self>) {}
}

