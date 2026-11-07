//! With TRINDLE_GPU_LOG=1, says on stderr what the GPU drawer does that costs time or hides a fault.

use std::cell::{Cell, RefCell};
use std::time::{Duration, Instant};

#[derive(Clone, Copy)]
pub(crate) enum Kind {
    Masks,
    Pictures,
    Whole,
    Processor,
}

const EVERY: Duration = Duration::from_secs(1);

thread_local! {
    static ON: Cell<Option<bool>> = const { Cell::new(None) };
    static LAST: RefCell<[(Option<Instant>, u32); 4]> = const { RefCell::new([(None, 0); 4]) };
}

fn due(last: &mut (Option<Instant>, u32), now: Instant) -> Option<u32> {
    if last.0.is_some_and(|t| now.duration_since(t) < EVERY) {
        last.1 += 1;
        return None;
    }
    let held = last.1;
    *last = (Some(now), 0);
    Some(held)
}

pub(crate) fn say(kind: Kind, what: impl FnOnce() -> String) {
    let on = ON.with(|o| *o.get().get_or_insert_with(|| std::env::var_os("TRINDLE_GPU_LOG").is_some_and(|v| v == "1")));
    if !on {
        return;
    }
    if let Some(held) = LAST.with_borrow_mut(|l| due(&mut l[kind as usize], Instant::now())) {
        let more = if held > 0 { format!(" (and {held} more since the last)") } else { String::new() };
        eprintln!("trindle gpu: {}{more}", what());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_kind_is_said_at_once_then_once_a_second_with_the_count_between() {
        let (mut last, t) = ((None, 0), Instant::now());
        assert_eq!(due(&mut last, t), Some(0));
        assert_eq!(due(&mut last, t + Duration::from_millis(10)), None);
        assert_eq!(due(&mut last, t + Duration::from_millis(900)), None);
        assert_eq!(due(&mut last, t + Duration::from_millis(1100)), Some(2));
        assert_eq!(due(&mut last, t + Duration::from_millis(2200)), Some(0));
    }
}
