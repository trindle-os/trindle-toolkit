//! Runs a risky check in a forked child, so a crash or hang in a C library costs the child and not the program.

use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Passed,
    Refused,
    Crashed(i32),
    TimedOut,
    NoChild,
}

pub fn in_child(limit: Duration, check: impl FnOnce() -> bool) -> Outcome {
    // SAFETY: fork has no arguments; the child below only calls `check`, then `_exit`.
    let pid = unsafe { libc::fork() };
    if pid < 0 {
        return Outcome::NoChild;
    }
    if pid == 0 {
        let code = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(check)) {
            Ok(true) => 0,
            _ => 1,
        };
        // SAFETY: ends this child at once, with a number alone.
        unsafe { libc::_exit(code) }
    }
    let start = Instant::now();
    loop {
        let mut status = 0;
        // SAFETY: a child of this process and an int owned here; WNOHANG, so it does not wait.
        let done = unsafe { libc::waitpid(pid, &mut status, libc::WNOHANG) };
        if done == pid {
            return if libc::WIFSIGNALED(status) {
                Outcome::Crashed(libc::WTERMSIG(status))
            } else if libc::WIFEXITED(status) && libc::WEXITSTATUS(status) == 0 {
                Outcome::Passed
            } else {
                Outcome::Refused
            };
        }
        if done < 0 {
            return Outcome::NoChild;
        }
        if start.elapsed() >= limit {
            // SAFETY: the child just waited for, still ours; then reaped so it leaves no zombie.
            unsafe {
                libc::kill(pid, libc::SIGKILL);
                libc::waitpid(pid, &mut status, 0);
            }
            return Outcome::TimedOut;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LIMIT: Duration = Duration::from_secs(5);

    #[test]
    fn a_child_that_says_yes_passes() {
        assert_eq!(in_child(LIMIT, || true), Outcome::Passed);
    }

    #[test]
    fn a_child_that_says_no_or_panics_is_refused() {
        assert_eq!(in_child(LIMIT, || false), Outcome::Refused);
        assert_eq!(in_child(LIMIT, || panic!("no")), Outcome::Refused);
    }

    #[test]
    fn a_child_that_aborts_is_a_crash_and_the_parent_lives() {
        assert_eq!(in_child(LIMIT, || std::process::abort()), Outcome::Crashed(libc::SIGABRT));
    }

    #[test]
    fn a_child_that_hangs_is_killed_at_the_limit() {
        let t = Instant::now();
        assert_eq!(in_child(Duration::from_millis(200), || loop { std::thread::sleep(Duration::from_secs(1)) }), Outcome::TimedOut);
        assert!(t.elapsed() < Duration::from_secs(3));
    }
}
