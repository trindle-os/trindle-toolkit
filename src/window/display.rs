//! The connection to the compositor: joining it, opening windows, and answering what it sends.

use std::os::fd::AsRawFd;

use wayland_client::globals::{registry_queue_init, GlobalList};
use wayland_protocols::xdg::activation::v1::client::xdg_activation_token_v1::{self, XdgActivationTokenV1};
use wayland_protocols::xdg::decoration::zv1::client::zxdg_toplevel_decoration_v1::Mode;
use wayland_protocols_wlr::layer_shell::v1::client::{
    zwlr_layer_shell_v1,
    zwlr_layer_surface_v1::{Anchor, KeyboardInteractivity},
};

use super::*;
use crate::event::Flow;

impl Display {
    pub fn connect(events: &Loop, gone: impl Fn() + 'static) -> Option<Rc<Display>> {
        crate::text::warm_fonts();
        let conn = Connection::connect_to_env().map_err(|e| eprintln!("trindle: no compositor: {e}")).ok()?;
        Display::connect_on(conn, events, gone)
    }

    pub fn connect_on(conn: Connection, events: &Loop, gone: impl Fn() + 'static) -> Option<Rc<Display>> {
        let (globals, mut queue) = registry_queue_init::<State>(&conn).ok()?;
        let qh = queue.handle();
        let (Ok(compositor), Ok(shm)) = (globals.bind(&qh, 4..=6, ()), globals.bind(&qh, 1..=1, ())) else {
            eprintln!("trindle: the compositor cannot show windows");
            return None;
        };
        let mut state = State::bound(&globals, &qh, compositor, shm);
        state.follow_seat(&globals, &qh);
        let outputs: Vec<(u32, u32)> = globals.contents().clone_list().into_iter()
            .filter(|g| g.interface == "wl_output")
            .map(|g| (g.name, g.version))
            .collect();
        for (id, version) in outputs {
            add_screen(&mut state, &qh, globals.registry(), id, version);
        }
        queue.roundtrip(&mut state).ok()?;
        state.screens_changed = false;
        for s in &mut state.screens {
            s.fresh = false;
        }
        let me = Display::made(conn, queue, state, events);
        Display::listen(&me, events, gone);
        Some(me)
    }

    fn made(conn: Connection, queue: EventQueue<State>, state: State, events: &Loop) -> Rc<Display> {
        Rc::new_cyclic(|me: &Weak<Display>| {
            let weak = me.clone();
            let back = events.post(move |back| {
                if let Some(me) = weak.upgrade() {
                    me.back(back);
                }
            });
            Display { conn, queue: RefCell::new(queue), state: RefCell::new(state),
                next: Cell::new(1), painting: Cell::new(false), events: events.clone(), me: me.clone(),
                waiting: RefCell::new(Vec::new()), on_screens: RefCell::new(Vec::new()), on_lock: RefCell::new(None), on_keys_held: RefCell::new(None), back }
        })
    }

    fn listen(me: &Rc<Display>, events: &Loop, gone: impl Fn() + 'static) {
        crate::common::run::hand_back_to(events);
        let weak = Rc::downgrade(me);
        let drawn_again = events.post(move |()| {
            if let Some(me) = weak.upgrade() {
                me.state.borrow_mut().windows.iter_mut().for_each(|w| w.dirty = true);
                me.paint();
            }
        });
        crate::text::on_fonts_ready(move || drawn_again.send(()));
        let weak = Rc::downgrade(me);
        crate::common::run::vouch_with(Rc::new(move |start| match weak.upgrade() {
            Some(me) => me.vouch(start),
            None => start(None),
        }));
        let fd = me.conn.backend().poll_fd().as_raw_fd();
        let weak = Rc::downgrade(me);
        events.watch(fd, move |hung_up| {
            let Some(me) = weak.upgrade() else { return Flow::Stop };
            if hung_up || !me.pump() {
                gone();
                return Flow::Stop;
            }
            Flow::Continue
        });
    }

    pub fn screens(&self) -> Vec<String> {
        self.state.borrow().screens.iter().map(|s| s.name.clone()).collect()
    }

    pub fn places(&self) -> Vec<[i32; 4]> {
        self.state.borrow().screens.iter().map(|s| s.place).filter(|p| p[2] > 0 && p[3] > 0).collect()
    }

    pub fn on_screens(&self, f: impl Fn() + 'static) {
        self.on_screens.borrow_mut().push(Rc::new(f));
    }

    pub fn holds_keys(&self) -> bool {
        self.state.borrow().windows.iter().any(|w| takes_keys(matches!(w.role, Role::Panel(_)), w.keys, w.hidden))
    }

    pub fn on_keys_held(&self, f: impl Fn() + 'static) {
        *self.on_keys_held.borrow_mut() = Some(Rc::new(f));
    }

    pub(super) fn keys_held(&self) {
        if !self.holds_keys() {
            return;
        }
        let told = self.on_keys_held.borrow().clone();
        if let Some(told) = told {
            told();
        }
    }

    pub fn has_layers(&self) -> bool {
        self.state.borrow().layers.is_some()
    }

    pub fn has_touch(&self) -> bool {
        self.state.borrow().touch.is_some()
    }

    pub fn open(self: &Rc<Self>, kind: Kind, size: (u32, u32), paint: impl FnMut(&mut PixmapMut, f32) + 'static) -> Option<Window> {
        let qh = self.queue.borrow().handle();
        let id = self.next.replace(self.next.get() + 1);
        let mut state = self.state.borrow_mut();
        if match &kind {
            Kind::Window { .. } => state.wm.is_none(),
            Kind::Panel(_) => state.layers.is_none(),
            Kind::Lock { .. } => state.lock.is_none(),
        } {
            return None;
        }
        let surface = state.compositor.create_surface(&qh, id);
        let mut keys = Keys::Never;
        let role = match kind {
            Kind::Window { title, app_id } => {
                let (title, app_id) = (no_nul(&title), no_nul(&app_id));
                let wm = state.wm.as_ref()?;
                let xdg = wm.get_xdg_surface(&surface, &qh, id);
                let top = xdg.get_toplevel(&qh, id);
                top.set_title(title.clone());
                top.set_app_id(app_id.clone());
                let title_bar = state.title_bars.as_ref().map(|m| m.get_toplevel_decoration(&top, &qh, ()));
                if let Some(t) = &title_bar {
                    t.set_mode(Mode::ServerSide);
                }
                Role::Window { xdg, top, title_bar, names: (title, app_id) }
            }
            Kind::Panel(p) => {
                let layer = panel_surface(&state, &p, size, &surface, &qh, id)?;
                keys = p.keys;
                Role::Panel(layer)
            }
            Kind::Lock { screen } => {
                let Some(lock) = state.lock_surface(&screen, &surface, &qh, id) else {
                    surface.destroy();
                    return None;
                };
                Role::Lock(lock, screen)
            }
        };
        let (viewport, fractional) = match (&state.viewporter, &state.fractions) {
            (Some(v), Some(f)) => (Some(v.get_viewport(&surface, &qh, ())), Some(f.get_fractional_scale(&surface, &qh, id))),
            _ => (None, None),
        };
        surface.commit();
        state.windows.push(Win::new(id, surface, role, viewport, fractional, size, Box::new(paint), keys));
        drop(state);
        self.keys_held();
        let _ = self.conn.flush();
        Some(Window { id, display: self.clone() })
    }

    fn pump(&self) -> bool {
        if let Some(guard) = self.conn.prepare_read() {
            if guard.read().is_err() && self.conn.flush().is_err() {
                return false;
            }
        }
        if self.queue.borrow_mut().dispatch_pending(&mut self.state.borrow_mut()).is_err() {
            return false;
        }
        self.answer_layers();
        self.lock_changed(|phase| {
            let call = self.on_lock.borrow().clone();
            if let Some(call) = call {
                call(phase);
            }
        });
        self.closes();
        self.suspends();
        self.trim_unseen();
        self.start_pulls();
        self.deliver();
        let vouched = std::mem::take(&mut self.state.borrow_mut().vouched);
        for (key, token) in vouched {
            self.vouched(key, Some(token));
        }
        if std::mem::take(&mut self.state.borrow_mut().start_repeat) {
            self.repeat_later(true);
        }
        if std::mem::take(&mut self.state.borrow_mut().screens_changed) {
            let calls = self.on_screens.borrow().clone();
            for call in calls {
                call();
            }
        }
        self.paint();
        self.conn.flush().is_ok()
    }

    pub fn vouch(&self, call: impl FnOnce(Option<String>) + 'static) {
        let key = self.next.get();
        self.next.set(key + 1);
        let qh = self.queue.borrow().handle();
        let state = self.state.borrow();
        let Some(m) = &state.activation else {
            drop(state);
            return call(None);
        };
        let token = m.get_activation_token(&qh, key);
        if let Some(seat) = &state.seat {
            token.set_serial(state.serial, seat);
        }
        if let Some(w) = state.keys_on.and_then(|id| state.windows.iter().find(|w| w.id == id)) {
            token.set_surface(&w.surface);
        }
        token.commit();
        drop(state);
        self.waiting.borrow_mut().push((key, Box::new(call), self.events.hold()));
        let _ = self.conn.flush();
        let me = self.me.clone();
        self.events.after(std::time::Duration::from_secs(1), move || {
            if let Some(me) = me.upgrade() {
                me.vouched(key, None);
            }
        });
    }

    fn vouched(&self, key: u64, token: Option<String>) {
        let call = {
            let mut waiting = self.waiting.borrow_mut();
            waiting.iter().position(|(k, ..)| *k == key).map(|i| waiting.remove(i).1)
        };
        if let Some(call) = call {
            call(token);
        }
    }

    pub(super) fn deliver(&self) {
        loop {
            let mut state = self.state.borrow_mut();
            let Some((id, input, serial)) = state.inbox.next() else { return };
            let call = state.win(id).and_then(|w| w.on_input.take());
            drop(state);
            let Some(mut call) = call else { continue };
            let before = PRESS.replace(serial);
            call(input);
            PRESS.set(before);
            if let Some(w) = self.state.borrow_mut().win(id) {
                w.on_input.get_or_insert(call);
            }
        }
    }

    fn repeat_later(&self, first: bool) {
        let state = self.state.borrow();
        let Some((wait, every)) = state.layout.as_ref().and_then(|k| k.repeat) else { return };
        let count = state.held_count;
        drop(state);
        let me = self.me.clone();
        self.events.after(if first { wait } else { every }, move || {
            let Some(me) = me.upgrade() else { return };
            let mut state = me.state.borrow_mut();
            let (Some(key), Some(id)) = (state.held, state.keys_on) else { return };
            if state.held_count != count {
                return;
            }
            if let Some(input) = state.layout.as_mut().and_then(|k| k.press(key, true)) {
                let serial = state.held_serial;
                state.inbox.push_pressed(id, input, serial);
            }
            drop(state);
            me.deliver();
            me.paint();
            me.repeat_later(false);
        });
    }

    fn closes(&self) {
        loop {
            let next = self.state.borrow_mut().windows.iter_mut().find(|w| w.closing && w.on_close.is_some())
                .map(|w| { w.closing = false; (w.id, w.on_close.take()) });
            let Some((id, Some(mut call))) = next else { return };
            call();
            if let Some(w) = self.state.borrow_mut().win(id) {
                w.on_close.get_or_insert(call);
            }
        }
    }

    fn suspends(&self) {
        loop {
            let next = self.state.borrow_mut().windows.iter_mut()
                .find(|w| w.suspended != w.told_suspended && w.on_suspended.is_some())
                .map(|w| { w.told_suspended = w.suspended; (w.id, w.suspended, w.on_suspended.take()) });
            let Some((id, now, Some(mut call))) = next else { return };
            call(now);
            if let Some(w) = self.state.borrow_mut().win(id) {
                w.on_suspended.get_or_insert(call);
            }
        }
    }

    fn answer_layers(&self) {
        let mut state = self.state.borrow_mut();
        for w in &mut state.windows {
            if !matches!(w.role, Role::Panel(_) | Role::Lock(..)) {
                continue;
            }
            let Some(serial) = w.ack.take() else { continue };
            if w.void > 0 {
                continue;
            }
            match &w.role {
                Role::Panel(layer) => layer.ack_configure(serial),
                Role::Lock(lock, _) => lock.ack_configure(serial),
                Role::Window { .. } => {}
            }
            if !w.dirty {
                w.surface.commit();
            }
        }
    }
}

impl State {
    fn bound(globals: &GlobalList, qh: &QueueHandle<State>, compositor: wl_compositor::WlCompositor, shm: wl_shm::WlShm) -> State {
        State {
            compositor,
            shm,
            dmabuf: globals.bind(qh, 3..=4, ()).ok().map(|g| dmabuf::Offer::new(g, qh)),
            wm: globals.bind(qh, 1..=6, ()).ok(),
            layers: globals.bind(qh, 1..=4, ()).ok(),
            locker: globals.bind(qh, 1..=1, ()).ok(),
            lock: None,
            title_bars: globals.bind(qh, 1..=1, ()).ok(),
            viewporter: globals.bind(qh, 1..=1, ()).ok(),
            fractions: globals.bind(qh, 1..=1, ()).ok(),
            places: globals.bind(qh, 1..=3, ()).ok(),
            subsurfaces: globals.bind(qh, 1..=1, ()).ok(),
            screens: Vec::new(),
            windows: Vec::new(),
            seat: globals.bind(qh, 1..=9, ()).ok(),
            pointer: None,
            keyboard: None,
            touch: None,
            shapes: globals.bind(qh, 1..=1, ()).ok(),
            shape: None,
            pens: pen::Pens::default(),
            gestures: gesture::Gestures::default(),
            pointer_on: None,
            pointer_at: (0.0, 0.0),
            scrolled: [0.0; 2],
            steps: [0.0; 2],
            stopped: false,
            keys_on: None,
            layout: None,
            held: None,
            held_serial: 0,
            held_count: 0,
            start_repeat: false,
            touches: Vec::new(),
            touched: Vec::new(),
            inbox: Inbox::default(),
            serial: 0,
            clipboard: globals.bind(qh, 1..=3, ()).ok(),
            device: None,
            copying: Copying::default(),
            pressed: None,
            held_in: None,
            held_out: None,
            pulls: Vec::new(),
            taking: Vec::new(),
            input_method: InputMethod::default(),
            activation: globals.bind(qh, 1..=1, ()).ok(),
            vouched: Vec::new(),
            pass: None,
            importer: globals.bind(qh, 1..=1, ()).ok(),
            dialogs: globals.bind(qh, 1..=1, ()).ok(),
            inhibits: globals.bind(qh, 1..=1, ()).ok(),
            icons: globals.bind(qh, 1..=1, ()).ok(),
            screens_changed: false,
            named: globals.bind(qh, 1..=3, ()).ok(),
        }
    }

    fn follow_seat(&mut self, globals: &GlobalList, qh: &QueueHandle<State>) {
        if self.named.is_some() {
            OWN.with(|own| *own.borrow_mut() = Some(crate::common::own_press::Seen::default()));
        }
        if let (Some(m), Some(seat)) = (&self.clipboard, &self.seat) {
            self.device = Some(m.get_data_device(seat, qh, ()));
        }
        self.pens = pen::Pens::bind(globals, qh, self.seat.as_ref());
        self.gestures = gesture::Gestures::bind(globals, qh);
        let selecting: Option<ZwpPrimarySelectionDeviceManagerV1> = globals.bind(qh, 1..=1, ()).ok();
        if let (Some(m), Some(seat)) = (selecting, &self.seat) {
            let device = m.get_device(seat, qh, ());
            self.copying.primary = Some((m, device));
        }
        let typers: Option<ZwpTextInputManagerV3> = globals.bind(qh, 1..=1, ()).ok();
        if let (Some(m), Some(seat)) = (&typers, &self.seat) {
            self.input_method.typer = Some(m.get_text_input(seat, qh, ()));
        }
    }
}

fn panel_surface(state: &State, p: &Panel, size: (u32, u32), surface: &wl_surface::WlSurface, qh: &QueueHandle<State>, id: u64) -> Option<ZwlrLayerSurfaceV1> {
    let output = p.screen.as_ref().and_then(|n| state.screens.iter().find(|s| &s.name == n)).map(|s| &s.output);
    if let (Some(name), None) = (&p.screen, output) {
        eprintln!("trindle: no screen called {name}, so the compositor places panel {}", p.name);
    }
    let layer = state.layers.as_ref()?.get_layer_surface(surface, output, to_layer(p.layer), no_nul(&p.name), qh, id);
    layer.set_anchor(Anchor::from_bits_truncate(p.edges.0));
    layer.set_size(size.0, size.1);
    layer.set_exclusive_zone(p.keep);
    let [top, right, bottom, left] = p.margins;
    layer.set_margin(top, right, bottom, left);
    layer.set_keyboard_interactivity(keyboard_interactivity(p.keys));
    Some(layer)
}

fn to_layer(layer: Layer) -> zwlr_layer_shell_v1::Layer {
    match layer {
        Layer::Background => zwlr_layer_shell_v1::Layer::Background,
        Layer::Bottom => zwlr_layer_shell_v1::Layer::Bottom,
        Layer::Top => zwlr_layer_shell_v1::Layer::Top,
        Layer::Overlay => zwlr_layer_shell_v1::Layer::Overlay,
    }
}

pub(super) fn takes_keys(panel: bool, keys: Keys, hidden: bool) -> bool {
    panel && keys != Keys::Never && !hidden
}

pub(super) fn keyboard_interactivity(keys: Keys) -> KeyboardInteractivity {
    match keys {
        Keys::Never => KeyboardInteractivity::None,
        Keys::OnClick => KeyboardInteractivity::OnDemand,
        Keys::Always => KeyboardInteractivity::Exclusive,
    }
}

impl Dispatch<XdgActivationTokenV1, u64> for State {
    fn event(state: &mut Self, token: &XdgActivationTokenV1, event: xdg_activation_token_v1::Event, key: &u64,
             _: &Connection, _: &QueueHandle<Self>) {
        if let xdg_activation_token_v1::Event::Done { token: pass } = event {
            state.vouched.push((*key, pass));
            token.destroy();
        }
    }
}
