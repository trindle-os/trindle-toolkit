//! The eye in a password box: what every one of them shares, whoever draws it.

use std::time::{Duration, Instant};

pub const REST: Duration = Duration::from_secs(30);

pub fn rested(last: Instant, now: Instant) -> bool {
    now.saturating_duration_since(last) >= REST
}

pub const SHOW_ICON: &str = "view-reveal-symbolic";
pub const HIDE_ICON: &str = "view-conceal-symbolic";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_password_left_showing_hides_after_half_a_minute() {
        let at = Instant::now();
        assert!(!rested(at, at + Duration::from_secs(29)));
        assert!(rested(at, at + REST));
        assert!(!rested(at + Duration::from_secs(5), at));
    }
}
