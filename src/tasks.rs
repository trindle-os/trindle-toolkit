//! Work on other threads and timers that hand their results back to the window's own thread as messages, as Qt's threads and timers do.

use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

use super::event::{Flow, Loop, Post};

pub struct Tasks<M> {
    post: Post<M>,
    events: Loop,
}

impl<M> Clone for Tasks<M> {
    fn clone(&self) -> Self {
        Tasks { post: self.post.clone(), events: self.events.clone() }
    }
}

#[derive(Clone, Default)]
pub struct Timer(Rc<Cell<bool>>);

impl Timer {
    pub fn stop(&self) {
        self.0.set(true);
    }
}

fn panic_words(why: &(dyn std::any::Any + Send)) -> String {
    match (why.downcast_ref::<&str>(), why.downcast_ref::<String>()) {
        (Some(s), _) => s.to_string(),
        (_, Some(s)) => s.clone(),
        _ => String::new(),
    }
}

impl<M: Send + 'static> Tasks<M> {
    pub fn new(events: &Loop, deliver: impl FnMut(M) + 'static) -> Tasks<M> {
        Tasks { post: events.post(deliver), events: events.clone() }
    }

    pub fn spawn(&self, work: impl FnOnce() -> M + Send + 'static) {
        let post = self.post.clone();
        std::thread::spawn(move || post.send(work()));
    }

    pub fn spawn_catching(&self, work: impl FnOnce() -> M + Send + 'static, failed: impl FnOnce(String) -> M + Send + 'static) {
        self.spawn_sending_catching(move |post| { post.send(work()); }, failed);
    }

    pub fn spawn_sending(&self, work: impl FnOnce(&Post<M>) + Send + 'static) {
        let post = self.post.clone();
        std::thread::spawn(move || work(&post));
    }

    pub fn spawn_sending_catching(&self, work: impl FnOnce(&Post<M>) + Send + 'static, failed: impl FnOnce(String) -> M + Send + 'static) {
        let post = self.post.clone();
        std::thread::spawn(move || {
            if let Err(why) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| work(&post))) {
                post.send(failed(panic_words(why.as_ref())));
            }
        });
    }

    pub fn after(&self, wait: Duration, make: impl FnOnce() -> M + 'static) {
        let post = self.post.clone();
        self.events.after(wait, move || {
            post.send(make());
        });
    }

    pub fn every(&self, every: Duration, mut make: impl FnMut() -> Option<M> + 'static) -> Timer {
        let (post, timer) = (self.post.clone(), Timer::default());
        let stopped = timer.0.clone();
        self.events.every(every, move || {
            if stopped.get() {
                return Flow::Stop;
            }
            if make().is_some_and(|m| post.send(m)) { Flow::Continue } else { Flow::Stop }
        });
        timer
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    fn gathered(n: usize) -> (Loop, Rc<RefCell<Vec<String>>>, Tasks<String>) {
        let lp = Loop::new();
        let got = Rc::new(RefCell::new(Vec::new()));
        let (into, quit) = (got.clone(), lp.clone());
        let tasks = Tasks::new(&lp, move |m| {
            into.borrow_mut().push(m);
            if into.borrow().len() == n {
                quit.quit();
            }
        });
        (lp, got, tasks)
    }

    #[test]
    fn work_on_a_thread_comes_back_on_the_loops_thread() {
        let (lp, got, tasks) = gathered(3);
        let here = std::thread::current().id();
        tasks.spawn(move || format!("{}", std::thread::current().id() != here));
        tasks.spawn_sending(|post| {
            post.send("half".into());
            post.send("done".into());
        });
        lp.run();
        let mut got = got.borrow().clone();
        got.sort();
        assert_eq!(got, ["done", "half", "true"]);
    }

    #[test]
    fn a_panic_in_the_work_reaches_the_program() {
        let (lp, got, tasks) = gathered(4);
        tasks.spawn_catching(|| panic!("no disc"), |why| format!("failed: {why}"));
        tasks.spawn_catching(|| "fine".to_string(), |why| why);
        tasks.spawn_sending_catching(
            |post| {
                post.send("half".into());
                panic!("{}", 7);
            },
            |why| format!("failed: {why}"),
        );
        lp.run();
        let mut got = got.borrow().clone();
        got.sort();
        assert_eq!(got, ["failed: 7", "failed: no disc", "fine", "half"]);
    }

    #[test]
    fn timers_deliver_once_or_until_stopped() {
        let (lp, got, tasks) = gathered(4);
        tasks.after(Duration::from_millis(5), || "once".into());
        let count = Rc::new(Cell::new(0));
        let (c, stop_after) = (count.clone(), Rc::new(RefCell::new(None::<Timer>)));
        let me = stop_after.clone();
        let timer = tasks.every(Duration::from_millis(5), move || {
            c.set(c.get() + 1);
            if c.get() == 3 {
                if let Some(t) = me.borrow().as_ref() {
                    t.stop();
                }
            }
            Some(format!("tick {}", c.get()))
        });
        *stop_after.borrow_mut() = Some(timer);
        lp.run();
        let got = got.borrow();
        assert!(got.contains(&"once".to_string()));
        assert_eq!(got.iter().filter(|m| m.starts_with("tick")).count(), 3);
    }

    #[test]
    fn a_timer_whose_maker_gives_nothing_stops() {
        let (lp, got, tasks) = gathered(2);
        let mut n = 0;
        tasks.every(Duration::from_millis(1), move || {
            n += 1;
            (n <= 2).then(|| n.to_string())
        });
        lp.run();
        assert_eq!(*got.borrow(), ["1", "2"]);
    }
}
