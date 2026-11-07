//! The one crew of gentle threads that copies files, handing jobs out from the thread that asked.

use std::sync::Mutex;
use std::time::Duration;

use crate::common::file_work::gentle;

pub fn crew<J: Send, R>(n: usize, each: &(dyn Fn(J) + Sync), lead: impl FnOnce(&Bench<J>) -> R) -> R {
    let bench = Bench { jobs: Mutex::new((std::collections::VecDeque::new(), 0, false)), more: std::sync::Condvar::new(), room: std::sync::Condvar::new() };
    std::thread::scope(|s| {
        for _ in 0..n.max(1) {
            gentle::scoped(s, || bench.work(each));
        }
        let _shut = Shut(&bench);
        lead(&bench)
    })
}

pub struct Bench<J> {
    jobs: Mutex<(std::collections::VecDeque<J>, usize, bool)>,
    more: std::sync::Condvar,
    room: std::sync::Condvar,
}

struct Shut<'a, J>(&'a Bench<J>);

impl<J> Drop for Shut<'_, J> {
    fn drop(&mut self) {
        self.0.jobs.lock().unwrap_or_else(|e| e.into_inner()).2 = true;
        self.0.more.notify_all();
    }
}

impl<J> Bench<J> {
    fn work(&self, each: &(dyn Fn(J) + Sync)) {
        loop {
            let job = {
                let mut jobs = self.jobs.lock().unwrap_or_else(|e| e.into_inner());
                loop {
                    if let Some(job) = jobs.0.pop_front() {
                        jobs.1 += 1;
                        self.room.notify_all();
                        break job;
                    }
                    if jobs.2 {
                        return;
                    }
                    jobs = self.more.wait(jobs).unwrap_or_else(|e| e.into_inner());
                }
            };
            each(job);
            self.jobs.lock().unwrap_or_else(|e| e.into_inner()).1 -= 1;
        }
    }

    pub fn give(&self, job: J, most: usize) -> Result<(), J> {
        let mut jobs = self.jobs.lock().unwrap_or_else(|e| e.into_inner());
        if jobs.0.len() >= most {
            return Err(job);
        }
        jobs.0.push_back(job);
        self.more.notify_one();
        Ok(())
    }

    pub fn give_wait(&self, job: J, most: usize, long: Duration) -> Result<(), J> {
        let mut jobs = self.jobs.lock().unwrap_or_else(|e| e.into_inner());
        if jobs.0.len() >= most {
            jobs = self.room.wait_timeout(jobs, long).unwrap_or_else(|e| e.into_inner()).0;
        }
        if jobs.0.len() >= most {
            return Err(job);
        }
        jobs.0.push_back(job);
        self.more.notify_one();
        Ok(())
    }

    pub fn idle(&self) -> bool {
        let jobs = self.jobs.lock().unwrap_or_else(|e| e.into_inner());
        jobs.0.is_empty() && jobs.1 == 0
    }
}
