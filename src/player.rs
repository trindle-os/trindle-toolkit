//! A video and audio player's controls and what state it is in; the program decodes, through the `Decoder` it fills in, and puts each frame in `Frames`.

use std::time::Duration;

use super::input::{Keysym, Mods};
use super::widget::{button, column, label, moving, row, slider, Frames, Length, Node};
use crate::t;

const SKIP: Duration = Duration::from_secs(5);

pub trait Decoder {
    fn play(&mut self, from: Duration);
    fn pause(&mut self);
    fn seek(&mut self, to: Duration);
    fn volume(&mut self, level: f32);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Stopped,
    Playing,
    Paused,
    Ended,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Control {
    PlayPause,
    Stop,
    Seek(f32),
    Back,
    Forward,
    Volume(f32),
    Mute,
}

pub struct Player {
    pub state: State,
    pub at: Duration,
    pub length: Option<Duration>,
    pub volume: f32,
    pub muted: bool,
    pub frames: Option<Frames>,
}

impl Player {
    pub fn new(length: Option<Duration>, frames: Option<Frames>) -> Player {
        Player { state: State::Stopped, at: Duration::ZERO, length, volume: 1.0, muted: false, frames }
    }

    pub fn update(&mut self, control: Control, decoder: &mut dyn Decoder) {
        match control {
            Control::PlayPause => match self.state {
                State::Playing => {
                    decoder.pause();
                    self.state = State::Paused;
                }
                State::Ended => {
                    self.at = Duration::ZERO;
                    decoder.play(self.at);
                    self.state = State::Playing;
                }
                State::Stopped | State::Paused => {
                    decoder.play(self.at);
                    self.state = State::Playing;
                }
            },
            Control::Stop => {
                decoder.pause();
                decoder.seek(Duration::ZERO);
                self.at = Duration::ZERO;
                self.state = State::Stopped;
            }
            Control::Seek(secs) => self.seek(Duration::try_from_secs_f32(secs.max(0.0)).unwrap_or(self.at), decoder),
            Control::Back => self.seek(self.at.saturating_sub(SKIP), decoder),
            Control::Forward => self.seek(self.at + SKIP, decoder),
            Control::Volume(v) => {
                self.volume = v.clamp(0.0, 1.0);
                self.muted = false;
                decoder.volume(self.volume);
            }
            Control::Mute => {
                self.muted = !self.muted;
                decoder.volume(if self.muted { 0.0 } else { self.volume });
            }
        }
    }

    fn seek(&mut self, to: Duration, decoder: &mut dyn Decoder) {
        self.at = self.length.map_or(to, |l| to.min(l));
        decoder.seek(self.at);
        if self.state == State::Ended {
            self.state = State::Paused;
        }
    }

    pub fn reached(&mut self, at: Duration) {
        self.at = self.length.map_or(at, |l| at.min(l));
    }

    pub fn ended(&mut self) {
        self.state = State::Ended;
        if let Some(l) = self.length {
            self.at = l;
        }
    }

    pub fn key(sym: Keysym, mods: Mods) -> Option<Control> {
        if mods.ctrl || mods.alt || mods.logo {
            return None;
        }
        match sym {
            Keysym::space | Keysym::k => Some(Control::PlayPause),
            Keysym::Left | Keysym::j => Some(Control::Back),
            Keysym::Right | Keysym::l => Some(Control::Forward),
            Keysym::m => Some(Control::Mute),
            _ => None,
        }
    }

    pub fn view<M: 'static>(&self, size: (f32, f32), name: impl Into<String>, wrap: impl Fn(Control) -> M + Clone + 'static) -> Node<M> {
        let playing = self.state == State::Playing;
        let (play_icon, play_tip) = if playing { ("media-playback-pause-symbolic", t!("kit-player-pause")) } else { ("media-playback-start-symbolic", t!("kit-player-play")) };
        let (mute_icon, mute_tip) = if self.muted { ("audio-volume-muted-symbolic", t!("kit-player-unmute")) } else { ("audio-volume-high-symbolic", t!("kit-player-mute")) };
        let time = match self.length {
            Some(l) => t!("kit-player-time", at = clock(self.at), length = clock(l)),
            None => clock(self.at),
        };
        let (w1, w2, w3, w4) = (wrap.clone(), wrap.clone(), wrap.clone(), wrap.clone());
        let mut controls = vec![
            button(play_icon, play_tip).on_press(wrap(Control::PlayPause)),
            button("media-seek-backward-symbolic", t!("kit-player-back")).on_press(w1(Control::Back)),
            button("media-seek-forward-symbolic", t!("kit-player-forward")).on_press(w2(Control::Forward)),
            label(time),
        ];
        if let Some(l) = self.length {
            controls.push(slider(self.at.as_secs_f32(), 0.0, l.as_secs_f32().max(1.0), 1.0, t!("kit-player-seek"), move |v| w3(Control::Seek(v))));
        }
        controls.push(button(mute_icon, mute_tip).on(self.muted).on_press(w4(Control::Mute)));
        let w5 = wrap;
        controls.push(slider(if self.muted { 0.0 } else { self.volume }, 0.0, 1.0, 0.05, t!("kit-player-volume"), move |v| w5(Control::Volume(v))).width(Length::Fixed(crate::theme::sheet().volume)));
        let bar = row(controls).gap(crate::theme::gaps().medium);
        match &self.frames {
            Some(f) => column(vec![moving(size, name, f.clone()).width(Length::Fill).height(Length::Fill), bar]).gap(crate::theme::gaps().medium),
            None => bar,
        }
    }
}

pub fn clock(d: Duration) -> String {
    let s = d.as_secs();
    if s >= 3600 {
        format!("{}:{:02}:{:02}", s / 3600, s / 60 % 60, s % 60)
    } else {
        format!("{}:{:02}", s / 60, s % 60)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct Told(Vec<String>);

    impl Decoder for Told {
        fn play(&mut self, from: Duration) {
            self.0.push(format!("play {}", from.as_secs()));
        }
        fn pause(&mut self) {
            self.0.push("pause".into());
        }
        fn seek(&mut self, to: Duration) {
            self.0.push(format!("seek {}", to.as_secs()));
        }
        fn volume(&mut self, level: f32) {
            self.0.push(format!("volume {level}"));
        }
    }

    fn player() -> (Player, Told) {
        (Player::new(Some(Duration::from_secs(60)), None), Told::default())
    }

    #[test]
    fn play_pause_and_stop_tell_the_decoder_and_change_state() {
        let (mut p, mut d) = player();
        p.update(Control::PlayPause, &mut d);
        assert_eq!(p.state, State::Playing);
        p.reached(Duration::from_secs(12));
        p.update(Control::PlayPause, &mut d);
        assert_eq!(p.state, State::Paused);
        p.update(Control::PlayPause, &mut d);
        p.update(Control::Stop, &mut d);
        assert_eq!((p.state, p.at), (State::Stopped, Duration::ZERO));
        assert_eq!(d.0, ["play 0", "pause", "play 12", "pause", "seek 0"]);
    }

    #[test]
    fn seeking_stays_within_the_length_and_play_after_the_end_starts_again() {
        let (mut p, mut d) = player();
        p.update(Control::Seek(90.0), &mut d);
        assert_eq!(p.at, Duration::from_secs(60));
        p.update(Control::Back, &mut d);
        assert_eq!(p.at, Duration::from_secs(55));
        p.ended();
        assert_eq!((p.state, p.at), (State::Ended, Duration::from_secs(60)));
        p.update(Control::PlayPause, &mut d);
        assert_eq!((p.state, p.at), (State::Playing, Duration::ZERO));
        assert_eq!(d.0.last().unwrap(), "play 0");
    }

    #[test]
    fn mute_keeps_the_volume_to_go_back_to() {
        let (mut p, mut d) = player();
        p.update(Control::Volume(0.5), &mut d);
        p.update(Control::Mute, &mut d);
        p.update(Control::Mute, &mut d);
        assert_eq!(d.0, ["volume 0.5", "volume 0", "volume 0.5"]);
    }

    #[test]
    fn keys_are_the_usual_players_keys() {
        let none = Mods::default();
        assert_eq!(Player::key(Keysym::space, none), Some(Control::PlayPause));
        assert_eq!(Player::key(Keysym::Right, none), Some(Control::Forward));
        assert_eq!(Player::key(Keysym::m, none), Some(Control::Mute));
        assert_eq!(Player::key(Keysym::m, Mods { ctrl: true, ..none }), None);
    }

    #[test]
    fn the_controls_are_named_for_a_screen_reader_and_follow_the_state() {
        let (mut p, mut d) = player();
        p.reached(Duration::from_secs(75));
        let said = p.view((320.0, 180.0), "Clip", |c| c).said();
        assert!(said.contains(&"button: Play".to_string()), "{said:?}");
        assert!(said.contains(&"1:00 of 1:00".to_string()), "{said:?}");
        p.update(Control::PlayPause, &mut d);
        let n = p.view((320.0, 180.0), "Clip", |c| c);
        assert!(n.said().contains(&"button: Pause".to_string()));
        assert!(n.presses().iter().any(|(tip, _, m)| tip == "Mute" && **m == Control::Mute));
    }

    #[test]
    fn a_stream_with_no_end_has_no_seek_bar() {
        let p = Player::new(None, None);
        let n = p.view((0.0, 0.0), "Radio", |c| c);
        assert!(!n.said().iter().any(|s| s.contains(" of ")));
        assert_eq!(clock(Duration::from_secs(3725)), "1:02:05");
    }
}
