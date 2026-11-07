//! Kinetic scrolling: a swipe or a touchpad flick let go keeps its scrolling area moving, slowing until it stops.

use std::time::{Duration, Instant};

const RECENT: Duration = Duration::from_millis(100);
const LEAST: f32 = 60.0;

#[derive(Default)]
pub(super) struct Fling {
    seen: Option<(String, Vec<(Instant, f32)>)>,
    coasting: Option<(String, f32, Instant)>,
}

impl Fling {
    pub(super) fn note(&mut self, area: &str, at: Instant, offset: f32) {
        self.coasting = None;
        match &mut self.seen {
            Some((was, moves)) if was == area => {
                moves.retain(|(t, _)| at.saturating_duration_since(*t) <= RECENT);
                moves.push((at, offset));
            }
            _ => self.seen = Some((area.to_string(), vec![(at, offset)])),
        }
    }

    pub(super) fn let_go(&mut self, now: Instant, scale: f32) {
        let Some((area, moves)) = self.seen.take() else { return };
        let fresh: Vec<_> = moves.into_iter().filter(|(t, _)| now.saturating_duration_since(*t) <= RECENT).collect();
        let (Some(a), Some(b)) = (fresh.first(), fresh.last()) else { return };
        let dt = b.0.saturating_duration_since(a.0).as_secs_f32();
        let speed = if dt > 0.0 { (b.1 - a.1) / dt } else { 0.0 };
        if speed.abs() >= LEAST * scale && !crate::motion::reduced() {
            self.coasting = Some((area, speed, now));
        }
    }

    pub(super) fn stop(&mut self) {
        self.seen = None;
        self.coasting = None;
    }

    pub(super) fn coasting(&self) -> bool {
        self.coasting.is_some()
    }

    pub(super) fn step(&mut self, now: Instant, friction: f32, scale: f32) -> Option<(String, f32)> {
        let (area, speed, last) = self.coasting.as_mut()?;
        let dt = now.saturating_duration_since(*last).as_secs_f32();
        *last = now;
        let by = *speed * dt;
        *speed *= (-friction * dt).exp();
        let moved = (area.clone(), by);
        if speed.abs() < LEAST * scale {
            self.coasting = None;
        }
        Some(moved)
    }
}
