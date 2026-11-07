//! Every window on a thread following the person's style sheet and theme as they change.

use super::*;

thread_local! {
    static RESTYLE: RefCell<Vec<Box<dyn Fn() -> bool>>> = RefCell::new(Vec::new());
    static WATCHING: Cell<bool> = const { Cell::new(false) };
}

fn watch_sheet(events: &Loop) {
    if WATCHING.with(Cell::get) {
        return;
    }
    let sheet = crate::common::style::own_sheet();
    let Some(folder) = sheet.parent() else { return };
    let watched = crate::watch::Watch::new()
        .and_then(|w| std::fs::create_dir_all(folder).map(|()| w.with_subfolders()))
        .and_then(|mut w| w.folder(folder).map(|()| w));
    let mut watch = match watched {
        Ok(w) => w,
        Err(e) => {
            eprintln!("{}", crate::t!("kit-sheet-not-watched", folder = folder.display().to_string(), why = e.to_string()));
            return;
        }
    };
    WATCHING.with(|w| w.set(true));
    let conf = crate::common::files::trindle_conf();
    let themes = crate::common::sheet::own_themes();
    let _ = watch.folder(&themes);
    SEEN.with(|s| s.set(theme::reading()));
    end_trial_when_due(events);
    let again = events.clone();
    let read = events.post(move |()| {
        end_trial_when_due(&again);
        restyle_if_new();
    });
    events.watch(watch.fd(), move |hung_up| {
        if hung_up {
            return crate::event::Flow::Stop;
        }
        let changed = watch.changed();
        if changed.contains(&themes) {
            let _ = watch.folder(&themes);
        }
        if changed.iter().any(|p| *p == sheet || *p == conf || p.starts_with(&themes)) {
            reread_off(&read, changed.contains(&sheet).then(|| sheet.clone()));
        }
        restyle_if_new();
        crate::event::Flow::Continue
    });
}

fn reread_off(read: &crate::event::Post<()>, own: Option<std::path::PathBuf>) {
    static ASKED: rereading::Coalesced<(crate::event::Post<()>, Option<std::path::PathBuf>)> = rereading::Coalesced::new();
    ASKED.ask((read.clone(), own), theme::reread, |batch| {
        let mut logged: Vec<std::path::PathBuf> = Vec::new();
        for (told, own) in batch {
            told.send(());
            if let Some(p) = own.filter(|p| !logged.contains(p)) {
                theme::lint::log(&p);
                logged.push(p);
            }
        }
    });
}

thread_local! {
    static SEEN: Cell<u64> = const { Cell::new(0) };
    static TRIAL_ENDS: Cell<u64> = const { Cell::new(0) };
}

fn restyle_if_new() {
    let now = theme::reading();
    if SEEN.with(|s| s.replace(now)) == now {
        return;
    }
    let all = RESTYLE.with(|all| std::mem::take(&mut *all.borrow_mut()));
    let kept: Vec<_> = all.into_iter().filter(|f| f()).collect();
    RESTYLE.with(|all| {
        let mut all = all.borrow_mut();
        let newer = std::mem::replace(&mut *all, kept);
        all.extend(newer);
    });
}

fn end_trial_when_due(events: &Loop) {
    let Some((_, until)) = crate::common::sheet::trial() else { return };
    if TRIAL_ENDS.with(|t| t.replace(until)) == until {
        return;
    }
    let left = crate::common::sheet::trial_left().unwrap_or_default();
    let again = events.clone();
    events.after(left + std::time::Duration::from_millis(300), move || {
        let next = again.clone();
        let read = again.post(move |()| {
            restyle_if_new();
            end_trial_when_due(&next);
        });
        let started = std::thread::Builder::new().name("sheets".into()).spawn(move || {
            crate::host::host().end_trial_if_over();
            theme::reread();
            read.send(());
        });
        if started.is_err() {
            crate::host::host().end_trial_if_over();
            theme::reread();
            restyle_if_new();
            end_trial_when_due(&again);
        }
    });
}

impl<A: App + 'static> Shown<A> where A::Msg: Clone + 'static {
    pub(super) fn follow_sheet(&self) {
        watch_sheet(&self.events);
        let (h, w, ev) = (Rc::downgrade(&self.held), Rc::downgrade(&self.window), self.events.clone());
        RESTYLE.with(|all| all.borrow_mut().push(Box::new(move || {
            let (Some(held), Some(window)) = (h.upgrade(), w.upgrade()) else { return false };
            if ev.quitting() {
                return false;
            }
            {
                let Ok(mut held) = held.try_borrow_mut() else { return true };
                let theme = (held.look)();
                held.ui.theme = theme;
                held.ui.set_labels(Labels::chosen());
                let view = held.app.view();
                held.ui.set_view(view);
            }
            window.redraw();
            true
        })));
    }
}
