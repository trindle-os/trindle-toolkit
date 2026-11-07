//! Typing to jump: letters typed over a list take the choice to the first row whose name starts with them, whatever their case and accents.

use std::time::{Duration, Instant};

pub const PAUSE: Duration = Duration::from_millis(1000);

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Typed {
    pub letters: String,
    pub at: Option<Instant>,
}

impl Typed {
    pub fn fresh(&self, now: Instant) -> bool {
        self.at.is_none_or(|at| now.saturating_duration_since(at) >= PAUSE)
    }

    pub fn takes(&self, text: &str, now: Instant) -> bool {
        !text.is_empty() && !text.chars().any(char::is_control) && (!self.fresh(now) || !text.trim().is_empty())
    }

    pub fn add(&mut self, text: &str, now: Instant) -> bool {
        let fresh = self.fresh(now);
        if fresh {
            self.letters.clear();
        }
        self.letters.push_str(&fold(text));
        self.at = Some(now);
        fresh
    }

    pub fn jump(&mut self, text: &str, now: Instant, chosen: Option<usize>, n: usize, name: impl Fn(usize) -> String) -> Option<usize> {
        let fresh = self.add(text, now);
        let names: Vec<String> = (0..n).map(|i| fold(&name(i))).collect();
        let starts = |from: usize, with: &str| (0..n).map(|k| (from + k) % n).find(|&i| names[i].starts_with(with));
        let from = chosen.map_or(0, |c| if fresh { c + 1 } else { c });
        let mut chars = self.letters.chars();
        let first = chars.next();
        starts(from, &self.letters).or_else(|| first.filter(|f| chars.all(|c| c == *f))
            .and_then(|f| starts(chosen.map_or(0, |c| c + 1), &f.to_string())))
    }
}

pub fn fold(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        fold_into(c, &mut out);
    }
    out
}

const EXTENDED_A: &[u8; 128] = b"aaaaaaccccccccddddeeeeeeeeeegggggggghhhhiiiiiiiiii..jjkkkllllllllllnnnnnnnnnoooooo..rrrrrrssssssssttttttuuuuuuuuuuuuwwyyyzzzzzzs";

pub fn fold_into(c: char, out: &mut String) {
    if c.is_ascii() {
        out.push(c.to_ascii_lowercase());
        return;
    }
    for c in c.to_lowercase() {
        let plain = match c {
            '\u{300}'..='\u{36f}' => continue,
            'à'..='å' => 'a',
            'ç' => 'c',
            'è'..='ë' => 'e',
            'ì'..='ï' => 'i',
            'ð' => 'd',
            'ñ' => 'n',
            'ò'..='ö' | 'ø' => 'o',
            'ù'..='ü' => 'u',
            'ý' | 'ÿ' => 'y',
            'æ' => {
                out.push_str("ae");
                continue;
            }
            'ß' => {
                out.push_str("ss");
                continue;
            }
            'þ' => {
                out.push_str("th");
                continue;
            }
            'ĳ' => {
                out.push_str("ij");
                continue;
            }
            'œ' => {
                out.push_str("oe");
                continue;
            }
            '\u{100}'..='\u{17f}' => EXTENDED_A[c as usize - 0x100] as char,
            c => c,
        };
        out.push(plain);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const NAMES: [&str; 6] = ["Apple", "banana", "Éclair", "eggs", "Elderflower", "fig"];

    fn go(t: &mut Typed, text: &str, now: Instant, chosen: Option<usize>) -> Option<usize> {
        t.jump(text, now, chosen, NAMES.len(), |i| NAMES[i].to_string())
    }

    #[test]
    fn letters_jump_narrow_and_step_whatever_their_case_and_accents() {
        let start = Instant::now();
        let mut t = Typed::default();
        assert_eq!(go(&mut t, "e", start, None), Some(2));
        assert_eq!(go(&mut t, "g", start + Duration::from_millis(300), Some(2)), Some(3));
        assert_eq!(t.letters, "eg");
        let later = start + Duration::from_secs(3);
        assert_eq!(go(&mut t, "E", later, Some(3)), Some(4));
        assert_eq!(go(&mut t, "e", later + Duration::from_millis(200), Some(4)), Some(2));
        assert_eq!(go(&mut t, "e", later + Duration::from_millis(400), Some(2)), Some(3));
        let mut t = Typed::default();
        assert_eq!(go(&mut t, "é", start, Some(5)), Some(2));
        let mut t = Typed::default();
        assert_eq!(go(&mut t, "z", start, Some(1)), None);
        assert_eq!(Typed::default().jump("a", start, None, 0, |_| String::new()), None);
    }

    #[test]
    fn a_space_only_counts_inside_a_name_and_keys_that_type_nothing_never_do() {
        let now = Instant::now();
        let mut t = Typed::default();
        assert!(!t.takes(" ", now) && !t.takes("\r", now) && !t.takes("\u{8}", now) && !t.takes("", now));
        assert!(t.takes("a", now));
        t.jump("a", now, None, 0, |_| String::new());
        assert!(t.takes(" ", now + Duration::from_millis(100)));
        assert!(!t.takes(" ", now + Duration::from_secs(2)));
    }
}
