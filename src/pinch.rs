//! Two fingers on a touch screen or touchpad as a pinch (zoom) and a twist (rotate), read from the plain touch events and the touchpad's own.

use std::f32::consts::{PI, TAU};

use super::input::Input;

const NEAR: f32 = 8.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Twist {
    pub scale: f32,
    pub turn: f32,
    pub centre: (f32, f32),
    pub moved: (f32, f32),
}

#[derive(Clone, Debug, Default)]
pub struct Pinch {
    fingers: Vec<(i32, (f32, f32))>,
    pad: bool,
}

impl Pinch {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn active(&self) -> bool {
        self.fingers.len() == 2 || self.pad
    }

    pub fn input(&mut self, input: &Input) -> Option<Twist> {
        match *input {
            Input::TouchDown { id, x, y } if self.fingers.len() < 2 => {
                self.fingers.retain(|f| f.0 != id);
                self.fingers.push((id, (x, y)));
                None
            }
            Input::TouchMove { id, x, y } => {
                let before = self.pair()?;
                let at = self.fingers.iter().position(|f| f.0 == id)?;
                self.fingers[at].1 = (x, y);
                let now = self.pair()?;
                twist(before, now)
            }
            Input::TouchUp { id } => {
                self.fingers.retain(|f| f.0 != id);
                None
            }
            Input::TouchCancel => {
                self.fingers.clear();
                None
            }
            Input::PadPinch(t) => {
                self.pad = true;
                Some(t)
            }
            Input::PadPinchEnd { .. } => {
                self.pad = false;
                None
            }
            _ => None,
        }
    }

    fn pair(&self) -> Option<((f32, f32), (f32, f32))> {
        match self.fingers.as_slice() {
            [a, b] => Some((a.1, b.1)),
            _ => None,
        }
    }
}

fn twist((a0, b0): ((f32, f32), (f32, f32)), (a1, b1): ((f32, f32), (f32, f32))) -> Option<Twist> {
    let span = |a: (f32, f32), b: (f32, f32)| (b.0 - a.0, b.1 - a.1);
    let (s0, s1) = (span(a0, b0), span(a1, b1));
    let (d0, d1) = (s0.0.hypot(s0.1), s1.0.hypot(s1.1));
    if d0 < NEAR || d1 < NEAR {
        return None;
    }
    let mut turn = s1.1.atan2(s1.0) - s0.1.atan2(s0.0);
    if turn > PI {
        turn -= TAU;
    } else if turn < -PI {
        turn += TAU;
    }
    let mid = |a: (f32, f32), b: (f32, f32)| ((a.0 + b.0) / 2.0, (a.1 + b.1) / 2.0);
    let (c0, c1) = (mid(a0, b0), mid(a1, b1));
    Some(Twist { scale: d1 / d0, turn, centre: c1, moved: (c1.0 - c0.0, c1.1 - c0.1) })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn down(id: i32, x: f32, y: f32) -> Input {
        Input::TouchDown { id, x, y }
    }

    fn to(id: i32, x: f32, y: f32) -> Input {
        Input::TouchMove { id, x, y }
    }

    fn near(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-4
    }

    #[test]
    fn one_finger_is_no_gesture() {
        let mut p = Pinch::new();
        assert_eq!(p.input(&down(1, 10.0, 10.0)), None);
        assert_eq!(p.input(&to(1, 50.0, 10.0)), None);
        assert!(!p.active());
    }

    #[test]
    fn fingers_spreading_apart_zoom_about_the_point_between_them() {
        let mut p = Pinch::new();
        p.input(&down(1, 100.0, 100.0));
        p.input(&down(2, 200.0, 100.0));
        assert!(p.active());
        let t = p.input(&to(2, 300.0, 100.0)).unwrap();
        assert!(near(t.scale, 2.0) && near(t.turn, 0.0));
        assert_eq!((t.centre, t.moved), ((200.0, 100.0), (50.0, 0.0)));
        let t = p.input(&to(1, 200.0, 100.0)).unwrap();
        assert!(near(t.scale, 0.5));
    }

    #[test]
    fn fingers_turning_round_each_other_rotate() {
        let mut p = Pinch::new();
        p.input(&down(1, 100.0, 100.0));
        p.input(&down(2, 200.0, 100.0));
        let t = p.input(&to(2, 100.0, 200.0)).unwrap();
        assert!(near(t.turn, PI / 2.0) && near(t.scale, 1.0));
        p.input(&to(2, 0.0, 100.0 + 1.0));
        let t = p.input(&to(2, 0.0, 100.0 - 1.0)).unwrap();
        assert!(t.turn.abs() < 0.1);
    }

    #[test]
    fn a_lifted_finger_ends_it_and_a_third_is_ignored() {
        let mut p = Pinch::new();
        p.input(&down(1, 100.0, 100.0));
        p.input(&down(2, 200.0, 100.0));
        p.input(&down(3, 0.0, 0.0));
        assert_eq!(p.input(&to(3, 5.0, 5.0)), None);
        assert!(p.input(&to(2, 250.0, 100.0)).is_some());
        p.input(&Input::TouchUp { id: 1 });
        assert_eq!(p.input(&to(2, 300.0, 100.0)), None);
        p.input(&down(4, 100.0, 100.0));
        assert!(p.active());
        p.input(&Input::TouchCancel);
        assert!(!p.active());
    }

    #[test]
    fn a_touchpad_pinch_is_taken_as_it_comes() {
        let mut p = Pinch::new();
        let t = Twist { scale: 1.5, turn: 0.1, centre: (5.0, 6.0), moved: (1.0, 0.0) };
        assert_eq!(p.input(&Input::PadPinch(t)), Some(t));
        assert!(p.active());
        p.input(&Input::PadPinchEnd { cancelled: false });
        assert!(!p.active());
    }

    #[test]
    fn fingers_on_top_of_each_other_give_nothing() {
        let mut p = Pinch::new();
        p.input(&down(1, 100.0, 100.0));
        p.input(&down(2, 102.0, 100.0));
        assert_eq!(p.input(&to(2, 150.0, 100.0)), None);
    }
}
