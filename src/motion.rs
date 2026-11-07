//! Movement over time: a share from 0 to 1 handed out on the loop's clock, eased so things start quickly and settle gently.

use std::cell::Cell;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use super::event::{Flow, Loop};

const TICK: Duration = Duration::from_millis(16);

static REDUCED: AtomicBool = AtomicBool::new(false);

pub fn reduce(on: bool) {
    #[cfg(test)]
    return LOCAL.with(|l| l.set(Some(on)));
    #[cfg(not(test))]
    REDUCED.store(on, Ordering::Relaxed);
}

#[cfg(test)]
thread_local! {
    static LOCAL: Cell<Option<bool>> = const { Cell::new(None) };
}

pub fn reduced() -> bool {
    #[cfg(test)]
    if let Some(on) = LOCAL.with(Cell::get) {
        return on;
    }
    REDUCED.load(Ordering::Relaxed) || super::theme::sheet().reduce_motion
}

pub fn slide(lp: &Loop, step: impl FnMut(f32) + 'static, done: impl FnOnce() + 'static) -> Tween {
    tween(lp, super::theme::sheet().slide_time.as_millis() as u64, Ease::Out, step, done)
}

pub fn fade(lp: &Loop, step: impl FnMut(f32) + 'static, done: impl FnOnce() + 'static) -> Tween {
    tween(lp, super::theme::sheet().fade_time.as_millis() as u64, Ease::Cubic, step, done)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ease {
    Out,
    Cubic,
}

impl Ease {
    pub fn at(self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Ease::Out => 1.0 - (1.0 - t).powi(3),
            Ease::Cubic if t < 0.5 => 4.0 * t * t * t,
            Ease::Cubic => 1.0 - (-2.0 * t + 2.0).powi(3) / 2.0,
        }
    }
}

pub struct Tween {
    stopped: Rc<Cell<bool>>,
}

impl Tween {
    pub fn stop(&self) {
        self.stopped.set(true);
    }
}

pub fn tween(lp: &Loop, ms: u64, ease: Ease, mut step: impl FnMut(f32) + 'static,
             done: impl FnOnce() + 'static) -> Tween {
    let stopped = Rc::new(Cell::new(false));
    let flag = stopped.clone();
    let began = Instant::now();
    let length = Duration::from_millis(ms);
    if reduced() || length.is_zero() {
        lp.after(Duration::ZERO, move || {
            if !flag.get() {
                step(1.0);
                done();
            }
        });
        return Tween { stopped };
    }
    let mut done = Some(done);
    lp.every(TICK, move || {
        if flag.get() {
            return Flow::Stop;
        }
        let t = began.elapsed().as_secs_f32() / length.as_secs_f32();
        if t < 1.0 {
            step(ease.at(t));
            return Flow::Continue;
        }
        step(1.0);
        if let Some(d) = done.take() {
            d();
        }
        Flow::Stop
    });
    Tween { stopped }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    #[test]
    fn reduced_motion_ends_a_tween_on_its_first_step() {
        struct Back;
        impl Drop for Back {
            fn drop(&mut self) {
                reduce(false);
            }
        }
        let _back = Back;
        let lp = Loop::new();
        reduce(true);
        let steps = Rc::new(RefCell::new(Vec::new()));
        let s = steps.clone();
        let l = lp.clone();
        tween(&lp, 5000, Ease::Out, move |t| s.borrow_mut().push(t), move || l.quit());
        let began = Instant::now();
        lp.run();
        assert_eq!(*steps.borrow(), [1.0]);
        assert!(began.elapsed() < TICK, "no timer is waited for");
    }

    #[test]
    fn a_slide_takes_the_sheets_time_on_the_loops_clock() {
        reduce(false);
        let lp = Loop::new();
        let steps = Rc::new(RefCell::new(Vec::new()));
        let (s, l) = (steps.clone(), lp.clone());
        let began = Instant::now();
        slide(&lp, move |t| s.borrow_mut().push(t), move || l.quit());
        lp.run();
        let took = began.elapsed();
        let time = super::super::theme::sheet().slide_time;
        assert!(took >= time, "{took:?}");
        let steps = steps.borrow();
        assert!(steps.len() > 2 && steps.windows(2).all(|w| w[0] <= w[1]), "{steps:?}");
        assert_eq!(steps.last(), Some(&1.0));
    }

    #[test]
    fn a_fade_under_reduced_motion_is_instant() {
        reduce(true);
        let lp = Loop::new();
        let steps = Rc::new(RefCell::new(Vec::new()));
        let (s, l) = (steps.clone(), lp.clone());
        fade(&lp, move |t| s.borrow_mut().push(t), move || l.quit());
        lp.run();
        reduce(false);
        assert_eq!(*steps.borrow(), [1.0]);
    }

    #[test]
    fn eases_start_at_nought_and_end_at_one() {
        for e in [Ease::Out, Ease::Cubic] {
            assert_eq!(e.at(0.0), 0.0);
            assert_eq!(e.at(1.0), 1.0);
            assert!(e.at(0.5) > 0.0 && e.at(0.5) < 1.0);
            assert!((0..10).all(|i| e.at(i as f32 / 10.0) <= e.at((i + 1) as f32 / 10.0)), "never goes back");
        }
        assert!(Ease::Out.at(0.5) > 0.5, "out starts fast");
        assert!((Ease::Cubic.at(0.5) - 0.5).abs() < 1e-6, "cubic is halfway at half time");
    }
}
