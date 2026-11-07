//! Work asked for in bursts, such as the sheets read again at every save: one thread does it, and whoever asks while it works is served by one more go.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

pub(super) struct Coalesced<W> {
    running: AtomicBool,
    waiting: Mutex<Vec<W>>,
}

impl<W: Send + 'static> Coalesced<W> {
    pub(super) const fn new() -> Self {
        Coalesced { running: AtomicBool::new(false), waiting: Mutex::new(Vec::new()) }
    }

    pub(super) fn ask(&'static self, who: W, work: fn(), done: fn(Vec<W>)) {
        self.waiting.lock().unwrap_or_else(|e| e.into_inner()).push(who);
        if self.running.swap(true, Ordering::AcqRel) {
            return;
        }
        if std::thread::Builder::new().name("sheets".into()).spawn(move || self.go(work, done)).is_err() {
            self.go(work, done);
        }
    }

    fn go(&self, work: fn(), done: fn(Vec<W>)) {
        loop {
            let batch = std::mem::take(&mut *self.waiting.lock().unwrap_or_else(|e| e.into_inner()));
            if batch.is_empty() {
                self.running.store(false, Ordering::Release);
                if self.waiting.lock().unwrap_or_else(|e| e.into_inner()).is_empty() || self.running.swap(true, Ordering::AcqRel) {
                    return;
                }
                continue;
            }
            work();
            done(batch);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;

    static BURST: Coalesced<usize> = Coalesced::new();
    static STARTED: AtomicUsize = AtomicUsize::new(0);
    static NOW: AtomicUsize = AtomicUsize::new(0);
    static MOST_AT_ONCE: AtomicUsize = AtomicUsize::new(0);
    static SERVED: AtomicUsize = AtomicUsize::new(0);
    static STALE: AtomicUsize = AtomicUsize::new(0);

    fn work() {
        STARTED.fetch_add(1, Ordering::SeqCst);
        let now = NOW.fetch_add(1, Ordering::SeqCst) + 1;
        MOST_AT_ONCE.fetch_max(now, Ordering::SeqCst);
        std::thread::sleep(std::time::Duration::from_millis(15));
        NOW.fetch_sub(1, Ordering::SeqCst);
    }

    fn done(batch: Vec<usize>) {
        let began = STARTED.load(Ordering::SeqCst);
        STALE.fetch_add(batch.iter().filter(|&&asked_at| asked_at >= began).count(), Ordering::SeqCst);
        SERVED.fetch_add(batch.len(), Ordering::SeqCst);
    }

    #[test]
    fn a_burst_of_askers_is_served_by_a_few_goes_that_never_overlap_and_each_began_after_its_asker() {
        let askers: Vec<_> = (0..4).map(|_| std::thread::spawn(|| {
            for _ in 0..50 {
                BURST.ask(STARTED.load(Ordering::SeqCst), work, done);
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
        })).collect();
        for a in askers {
            a.join().unwrap();
        }
        let until = std::time::Instant::now() + std::time::Duration::from_secs(20);
        while SERVED.load(Ordering::SeqCst) < 200 && std::time::Instant::now() < until {
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        assert_eq!(SERVED.load(Ordering::SeqCst), 200, "an asker was never served");
        assert_eq!(MOST_AT_ONCE.load(Ordering::SeqCst), 1, "two goes ran at once");
        assert_eq!(STALE.load(Ordering::SeqCst), 0, "an asker was served by a go that began before it asked");
        assert!(STARTED.load(Ordering::SeqCst) < 200, "every ask had a go of its own");
    }
}
