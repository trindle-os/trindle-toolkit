//! A window showing a program's widgets: opening it, and what a program may do to it from outside.

use super::delivering::{deliver, hand_over, paste_asked, read_clipboard, show, wake_later};
use super::*;

fn painted<A: App, R>(ev: &Loop, h: &mut Held<A>, size: (u32, u32), scale: f32, paint: impl FnOnce(&mut Ui<A::Msg>) -> R) -> R where A::Msg: Clone + 'static {
    let before = (h.ui.size, h.ui.scale);
    if before != (size, scale) && !ev.quitting() {
        let k = scale.max(0.01);
        if h.app.resized(size.0 as f32 / k, size.1 as f32 / k) {
            show(ev, h);
        }
    }
    let r = paint(&mut h.ui);
    if (h.ui.size, h.ui.scale) != before {
        h.tell();
    }
    let now = h.ui.sent_out.borrow().clone();
    if let Some(call) = h.outside.clone().filter(|_| now != h.told) {
        h.told = now.clone();
        ev.after(Duration::ZERO, move || call(now));
    }
    r
}

fn move_on(window: Option<&Window>, painting: &RefCell<Painting>, name: &str) -> Drawn {
    let Some(window) = window else { return Drawn::Failed };
    match crate::paint::gpu_for(window.display()) {
        Ok(gpu) => {
            let alloc = super::window::dmabuf::Alloc::Gbm(gpu.device.clone());
            eprintln!("trindle: {name} window drawn by gpu (ServiceGone)");
            if let Ok(mut p) = painting.try_borrow_mut() {
                *p = Painting::Here(crate::paint::Drawing::new(gpu));
            }
            Drawn::Moved(alloc)
        }
        Err(why) => {
            eprintln!("trindle: {name} window drawn by processor ({why:?})");
            Drawn::Failed
        }
    }
}

fn note_painted(call: &PaintedCall, frame: Painted) {
    if let Ok(mut call) = call.try_borrow_mut() {
        if let Some(call) = call.as_mut() {
            call(&frame);
        }
    }
}

fn draw_on_gpu<A: App + 'static>(events: &Loop, held: &Rc<RefCell<Held<A>>>, window: &Rc<Window>, painting: Painting, alloc: super::window::dmabuf::Alloc, told: PaintedCall, who: String) where A::Msg: Clone + 'static {
    let (h, ev, painting) = (Rc::downgrade(held), events.clone(), Rc::new(RefCell::new(painting)));
    let asleep = painting.clone();
    let w = Rc::downgrade(window);
    window.draw_by_gpu(alloc, move |yes| {
        if let Ok(mut d) = asleep.try_borrow_mut() {
            d.sleep(yes);
        }
    }, move |frame| {
        let Some(held) = h.upgrade() else { return Drawn::Failed };
        let Ok(mut h) = held.try_borrow_mut() else { return Drawn::Failed };
        let began = Instant::now();
        let drawn = {
            let Ok(mut drawing) = painting.try_borrow_mut() else { return Drawn::Failed };
            let drawing: &mut Painting = &mut drawing;
            painted(&ev, &mut h, frame.size, frame.scale, |ui| ui.paint_by_gpu(frame, drawing))
        };
        if let Some(w) = w.upgrade() {
            w.set_solid(!h.ui.see_through && h.ui.shown >= 1.0 && h.ui.theme.bg.a >= 1.0);
        }
        match drawn {
            Drawn::Failed | Drawn::Moved(_) | Drawn::Again => drawn,
            Drawn::Gone => move_on(w.upgrade().as_deref(), &painting, &who),
            Drawn::Sent { seq, parts } => when_service_answers(seq, parts, w.clone(), painting.clone(), told.clone(), who.clone(), (frame.size, frame.scale), began),
            drawn => {
                note_painted(&told, Painted { began, took: began.elapsed(), size: frame.size, scale: frame.scale, by_gpu: true, by_service: matches!(*painting.borrow(), Painting::There(_)) });
                drawn
            }
        }
    });
}

fn when_service_answers(seq: u64, parts: Option<Vec<[i32; 4]>>, w: std::rc::Weak<Window>, painting: Rc<RefCell<Painting>>, told: PaintedCall, who: String, at: ((u32, u32), f32), began: Instant) -> Drawn {
    let remote = match &*painting.borrow() {
        Painting::There(s) => s.remote().clone(),
        Painting::Here(_) => return Drawn::Failed,
    };
    remote.when_answered(seq, move |answer| {
        let Some(window) = w.upgrade() else { return };
        let drawn = match (answer, &mut *painting.borrow_mut()) {
            (Answer::Done, Painting::There(s)) => {
                s.done();
                parts.map_or(Drawn::Whole, Drawn::Parts)
            }
            (Answer::Again, Painting::There(s)) => if s.again() { Drawn::Again } else { Drawn::Gone },
            _ => Drawn::Gone,
        };
        let shown = matches!(drawn, Drawn::Whole | Drawn::Parts(_));
        window.answered(seq, if matches!(drawn, Drawn::Gone) { move_on(Some(&window), &painting, &who) } else { drawn });
        if shown {
            note_painted(&told, Painted { began, took: began.elapsed(), size: at.0, scale: at.1, by_gpu: true, by_service: true });
        }
    });
    Drawn::Sent { seq, parts: None }
}

fn hear_input<A: App + 'static>(events: &Loop, held: &Rc<RefCell<Held<A>>>, window: &Rc<Window>) where A::Msg: Clone + 'static {
    let (h, w, ev) = (Rc::downgrade(held), Rc::downgrade(window), events.clone());
    window.on_input(move |input| {
        let (Some(held), Some(window)) = (h.upgrade(), w.upgrade()) else { return };
        if let (Input::Focus(here), Some(link)) = (&input, &mut held.borrow_mut().link) {
            link.focused(*here);
        }
        let back = matches!(input, Input::Focus(true));
        if matches!(input, Input::DragEnded { .. }) {
            held.borrow_mut().ui.input(&input, Instant::now());
        }
        let taken = {
            let h = held.borrow();
            if h.ui.menu_open() { None } else { h.app.intercept_with(&input, h.ui.keyboard()) }
        };
        if let Some(m) = taken {
            let mut reply = Reply::new();
            reply.messages.push(m);
            deliver(&ev, &held, &window, reply);
            return;
        }
        let mut reply = held.borrow_mut().ui.input(&input, Instant::now());
        if back {
            reply.messages.extend(held.borrow().app.focused());
        }
        deliver(&ev, &held, &window, reply);
    });
}

fn hear_the_reader<A: App + 'static>(events: &Loop, held: &Rc<RefCell<Held<A>>>, window: &Rc<Window>, name: String) -> crate::event::Post<access::Ask> where A::Msg: Clone + 'static {
    let (h, w, ev) = (Rc::downgrade(held), Rc::downgrade(window), events.clone());
    events.post(move |ask| {
        let (Some(held), Some(window)) = (h.upgrade(), w.upgrade()) else { return };
        match ask {
            access::Ask::Tree => held.borrow_mut().tell(),
            access::Ask::Act(req) => {
                if !super::window::reader_may_ask(&name) {
                    super::window::refuse();
                    let mut reply = Reply::new();
                    reply.redraw = true;
                    deliver(&ev, &held, &window, reply);
                    return;
                }
                super::window::by_reader(|| {
                    let reply = held.borrow_mut().ui.act(&req);
                    deliver(&ev, &held, &window, reply);
                })
            }
        }
    })
}

impl<A: App + 'static> Shown<A> where A::Msg: Clone + 'static {
    pub fn run(kind: Kind, size: (u32, u32), app: A) -> Result<(), String> {
        let events = Loop::new();
        let gone = events.clone();
        let display = Display::connect(&events, move || gone.quit()).ok_or_else(|| crate::t!("kit-run-no-desktop").to_string())?;
        let shown = Shown::open(&events, &display, kind, size, app).ok_or_else(|| crate::t!("kit-run-no-window").to_string())?;
        let quit = events.clone();
        shown.window.on_close(move || quit.quit());
        events.run();
        Ok(())
    }

    pub fn open(events: &Loop, display: &Rc<Display>, kind: Kind, size: (u32, u32), app: A) -> Option<Shown<A>> {
        let page = match &kind {
            Kind::Window { app_id, .. } => page_of(app_id),
            _ => String::new(),
        };
        crate::host::host().drawing_thread();
        let look: Look = Rc::new(move || Theme::for_page(&page));
        let mut ui = Ui::new(app.view(), look());
        ui.set_labels(Labels::chosen());
        let title = match &kind {
            Kind::Window { title, .. } => title.clone(),
            _ => String::new(),
        };
        let name = match &kind {
            Kind::Window { app_id, .. } => app_id.clone(),
            Kind::Panel(panel) => panel.name.clone(),
            Kind::Lock { .. } => "session-lock".to_string(),
        };
        let held = Rc::new(RefCell::new(Held { app, ui, look, wake: None, link: None, out: false, outside: None, told: None }));
        let (h, ev) = (held.clone(), events.clone());
        let seen: PaintedCall = Rc::new(RefCell::new(None));
        let told = seen.clone();
        let window = Rc::new(display.open(kind, size, move |canvas, scale| {
            let began = Instant::now();
            painted(&ev, &mut h.borrow_mut(), (canvas.width(), canvas.height()), scale, |ui| ui.paint(canvas, scale));
            note_painted(&told, Painted { began, took: began.elapsed(), size: (canvas.width(), canvas.height()), scale, by_gpu: false, by_service: false });
        })?);
        let drawer = match crate::paint::way_for(display, events) {
            Ok((painting, alloc)) => {
                let drawer = if matches!(painting, Painting::There(_)) { Drawer::Service } else { Drawer::Gpu };
                draw_on_gpu(events, &held, &window, painting, alloc, seen.clone(), name.clone());
                drawer
            }
            Err(why) => Drawer::Processor(why),
        };
        eprintln!("{}", drawer.line(&name));
        crate::icon::draw_again_when_one_is_back(&window);
        let h = Rc::downgrade(&held);
        window.on_patch(move |canvas, scale| {
            let held = h.upgrade()?;
            let h = held.try_borrow().ok()?;
            h.ui.paint_live(canvas, scale).map(|rects| rects.iter().map(|r| {
                let (x, y) = (r.x.round() as i32, r.y.round() as i32);
                [x, y, r.w.round() as i32, r.h.round() as i32]
            }).collect())
        });
        hear_input(events, &held, &window);
        let asks = hear_the_reader(events, &held, &window, name);
        held.borrow_mut().link = Some(access::Link::new(&title, asks));
        let shown = Shown { window, held, events: events.clone(), drawer, painted: seen };
        shown.follow_sheet();
        Some(shown)
    }

    pub fn drawer(&self) -> &Drawer {
        &self.drawer
    }

    pub fn on_painted(&self, call: impl FnMut(&Painted) + 'static) {
        *self.painted.borrow_mut() = Some(Box::new(call));
    }

    pub fn announce(&self, words: &str) {
        let mut held = self.held.borrow_mut();
        if held.ui.news == words {
            return;
        }
        held.ui.news = words.to_string();
        held.tell();
    }

    pub fn change(&self, call: impl FnOnce(&mut A)) {
        {
            let mut held = self.held.borrow_mut();
            call(&mut held.app);
            if !show(&self.events, &mut held) {
                return;
            }
            self.window.set_typing(held.ui.typing().or_else(|| held.app.typing()));
            held.tell();
        }
        let paste = {
            let mut held = self.held.borrow_mut();
            let paste = paste_asked(&mut held.app);
            if super::window::refused_act() {
                held.ui.say_refused(Instant::now());
            }
            paste
        };
        hand_over(&self.held, &self.window);
        if paste {
            read_clipboard(&self.events, &self.held, &self.window);
        }
        wake_later(&self.events, &self.held, &self.window);
        self.window.redraw();
    }

    pub fn change_if(&self, call: impl FnOnce(&mut A) -> bool) {
        let changed = call(&mut self.held.borrow_mut().app);
        if changed {
            self.change(|_| {});
        }
    }

    pub fn with<R>(&self, call: impl FnOnce(&A) -> R) -> R {
        call(&self.held.borrow().app)
    }

    pub fn busy(&self) -> bool {
        self.held.try_borrow_mut().is_err()
    }

    pub fn spoken(&self, name: &str) {
        if let Some(link) = self.held.borrow_mut().link.as_mut() {
            link.title = name.into();
        }
    }

    pub fn labels_outside(&self, call: impl Fn(Option<Outside>) + 'static) {
        let mut held = self.held.borrow_mut();
        held.ui.out_labels = true;
        held.outside = Some(Rc::new(call));
    }

    pub fn see_through(&self) {
        self.held.borrow_mut().ui.see_through = true;
        self.window.redraw();
    }

    pub fn frame_shadow(&self) {
        self.window.set_shadow(theme::sheet().frame_shadow.round() as u32);
        self.see_through();
    }

    pub fn set_shown(&self, share: f32) {
        self.held.borrow_mut().ui.shown = share;
        self.window.redraw();
    }

    pub fn reload_icons(&self) {
        crate::icon::forget_far();
        self.held.borrow_mut().ui.icons = Icons::new();
        self.window.redraw();
    }

    pub fn restyle(&self) {
        let theme = {
            let mut held = self.held.borrow_mut();
            held.ui.set_labels(Labels::chosen());
            (held.look)()
        };
        self.set_theme(theme);
    }

    pub fn set_look(&self, look: impl Fn() -> Theme + 'static) {
        self.held.borrow_mut().look = Rc::new(look);
        self.restyle();
    }

    fn set_theme(&self, theme: Theme) {
        self.held.borrow_mut().ui.theme = theme;
        self.window.redraw();
    }

    pub fn aim_at(&self, key: &str, above: f32) -> Option<(String, f32, f32)> {
        self.held.borrow_mut().ui.aim_at(key, above)
    }

    pub fn scrolled(&self, id: &str) -> Option<f32> {
        let held = self.held.borrow();
        held.ui.spot(id).map(|s| held.ui.offset(s))
    }

    pub fn scroll_area(&self, id: &str, v: f32) {
        if self.held.borrow_mut().ui.scroll_area(id, v) {
            self.window.redraw();
        }
    }

    pub fn open_beside<B: App + 'static>(&self, kind: Kind, size: (u32, u32), app: B) -> Option<Shown<B>> where B::Msg: Clone + 'static {
        Shown::open(&self.events, self.window.display(), kind, size, app)
    }

    pub fn reading(&self, keys: &[&str]) -> Option<usize> {
        self.held.borrow_mut().ui.reading(keys)
    }

    pub fn go_to(&self, key: &str) {
        if self.held.borrow_mut().ui.go_to(key) {
            self.window.redraw();
        }
    }

    pub fn fit(&self) {
        let width = self.window.size().0;
        let room = self.window.frame().shadow;
        let mut height = (self.held.borrow().ui.natural_height(width as f32 + 2.0 * room) - 2.0 * room).ceil().max(1.0) as u32;
        if let Some(room) = self.window.room_tall() {
            height = height.min(room);
        }
        self.window.resize((width, height.max(1)));
    }
}

fn page_of(app_id: &str) -> String {
    app_id.rsplit(['.', '-']).next().unwrap_or_default().to_ascii_lowercase()
}
