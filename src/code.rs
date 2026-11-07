//! A code view: lines of Rust, shell or plain text with numbers down the side and words coloured by kind from the sheet.

use std::ops::Range;

use crate::text::Style;
use crate::theme;
use crate::widget::{drawing, Node, Reading};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lang {
    Rust,
    Shell,
    Plain,
}

impl Lang {
    pub fn guess(name: &str, first_line: &str) -> Lang {
        let ext = name.rsplit_once('.').map_or("", |(_, e)| e);
        match ext {
            "rs" => Lang::Rust,
            "sh" | "bash" | "zsh" => Lang::Shell,
            _ if first_line.starts_with("#!") && ["sh", "bash", "zsh", "dash"].iter().any(|s| first_line.rsplit(['/', ' ']).next() == Some(*s)) => Lang::Shell,
            _ => Lang::Plain,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Keyword,
    Type,
    Text,
    Number,
    Comment,
}

const RUST_WORDS: &[&str] = &[
    "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern", "false", "fn", "for", "if", "impl", "in",
    "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return", "self", "Self", "static", "struct", "super", "trait", "true", "type",
    "unsafe", "use", "where", "while",
];
const RUST_TYPES: &[&str] = &[
    "bool", "char", "str", "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64", "i128", "isize", "f32", "f64",
];
const SHELL_WORDS: &[&str] = &[
    "if", "then", "else", "elif", "fi", "for", "while", "until", "do", "done", "case", "esac", "in", "function", "return", "local", "export",
    "select", "time", "break", "continue", "exit",
];

pub fn tokens(text: &str, lang: Lang) -> Vec<(Range<usize>, Kind)> {
    match lang {
        Lang::Rust => rust(text),
        Lang::Shell => shell(text),
        Lang::Plain => Vec::new(),
    }
}

fn quoted(b: &[u8], from: usize, quote: u8, escapes: bool) -> usize {
    let mut i = from + 1;
    while i < b.len() {
        match b[i] {
            b'\\' if escapes => i += 2,
            c if c == quote => return i + 1,
            _ => i += 1,
        }
    }
    b.len()
}

fn word_end(b: &[u8], from: usize) -> usize {
    (from..b.len()).find(|&i| !(b[i].is_ascii_alphanumeric() || b[i] == b'_' || b[i] >= 0x80)).unwrap_or(b.len())
}

fn block_comment_end(b: &[u8], from: usize) -> usize {
    let (mut depth, mut j) = (0, from);
    while j < b.len() {
        if b[j..].starts_with(b"/*") {
            depth += 1;
            j += 2;
        } else if b[j..].starts_with(b"*/") {
            depth -= 1;
            j += 2;
            if depth == 0 {
                break;
            }
        } else {
            j += 1;
        }
    }
    j.min(b.len())
}

fn rust(text: &str) -> Vec<(Range<usize>, Kind)> {
    let b = text.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < b.len() {
        let c = b[i];
        let next = b.get(i + 1).copied();
        let (end, kind) = if c == b'/' && next == Some(b'/') {
            (text[i..].find('\n').map_or(b.len(), |n| i + n), Some(Kind::Comment))
        } else if c == b'/' && next == Some(b'*') {
            (block_comment_end(b, i), Some(Kind::Comment))
        } else if c == b'r' && matches!(next, Some(b'"' | b'#')) && b[i + 1..].iter().position(|&x| x != b'#').is_some_and(|p| b[i + 1 + p] == b'"') {
            let hashes = b[i + 1..].iter().take_while(|&&x| x == b'#').count();
            let close = format!("\"{}", "#".repeat(hashes));
            let body = i + 2 + hashes;
            (text[body..].find(&close).map_or(b.len(), |n| body + n + close.len()), Some(Kind::Text))
        } else if c == b'"' {
            (quoted(b, i, b'"', true), Some(Kind::Text))
        } else if c == b'\'' {
            let close = if next == Some(b'\\') { b[i + 2..].iter().position(|&x| x == b'\'').map(|p| i + 2 + p) } else {
                text[i + 1..].chars().next().map(|ch| i + 1 + ch.len_utf8()).filter(|&j| b.get(j) == Some(&b'\''))
            };
            match close {
                Some(j) => (j + 1, Some(Kind::Text)),
                None => (word_end(b, i + 1).max(i + 1), None),
            }
        } else if c.is_ascii_digit() {
            let mut j = i;
            while j < b.len() && (b[j].is_ascii_alphanumeric() || b[j] == b'_' || (b[j] == b'.' && b.get(j + 1).is_some_and(u8::is_ascii_digit))) {
                j += 1;
            }
            (j, Some(Kind::Number))
        } else if c.is_ascii_alphabetic() || c == b'_' {
            let j = word_end(b, i);
            let w = &text[i..j];
            let kind = if RUST_WORDS.contains(&w) {
                Some(Kind::Keyword)
            } else if RUST_TYPES.contains(&w) || w.starts_with(|c: char| c.is_ascii_uppercase()) {
                Some(Kind::Type)
            } else {
                None
            };
            (j, kind)
        } else {
            (i + text[i..].chars().next().map_or(1, char::len_utf8), None)
        };
        if let Some(k) = kind {
            out.push((i..end, k));
        }
        i = end;
    }
    out
}

fn shell(text: &str) -> Vec<(Range<usize>, Kind)> {
    let b = text.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < b.len() {
        let c = b[i];
        let after_space = i == 0 || b[i - 1].is_ascii_whitespace() || b[i - 1] == b';';
        let (end, kind) = if c == b'#' && after_space {
            (text[i..].find('\n').map_or(b.len(), |n| i + n), Some(Kind::Comment))
        } else if c == b'\'' {
            (quoted(b, i, b'\'', false), Some(Kind::Text))
        } else if c == b'"' {
            (quoted(b, i, b'"', true), Some(Kind::Text))
        } else if c == b'$' && b.get(i + 1) == Some(&b'{') {
            (text[i..].find('}').map_or(b.len(), |n| i + n + 1), Some(Kind::Type))
        } else if c == b'$' {
            let j = word_end(b, i + 1);
            (if j > i + 1 { j } else { (i + 2).min(b.len()) }, Some(Kind::Type))
        } else if c.is_ascii_alphanumeric() || c == b'_' {
            let j = (i..b.len()).find(|&k| !(b[k].is_ascii_alphanumeric() || matches!(b[k], b'_' | b'-' | b'.'))).unwrap_or(b.len());
            let w = &text[i..j];
            let kind = if after_space && SHELL_WORDS.contains(&w) {
                Some(Kind::Keyword)
            } else if w.bytes().all(|x| x.is_ascii_digit()) {
                Some(Kind::Number)
            } else {
                None
            };
            (j, kind)
        } else {
            (i + text[i..].chars().next().map_or(1, char::len_utf8), None)
        };
        if let Some(k) = kind {
            out.push((i..end, k));
        }
        i = end;
    }
    out
}

const PAD: f32 = 8.0;
const LINE: f32 = 1.4;
const LETTER: f32 = 0.6;

pub fn view<M>(text: &str, lang: Lang, name: &str) -> Node<M> {
    let text = text.replace('\t', "    ");
    let lines: Vec<String> = text.split('\n').map(|l| l.trim_end_matches('\r').to_string()).collect();
    let size = theme::sheet().text.0;
    let gutter = lines.len().to_string().len();
    let longest = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0);
    let want = (2.0 * PAD + (gutter + 2 + longest) as f32 * size * LETTER, 2.0 * PAD + lines.len() as f32 * size * LINE);
    let reading = Reading { lines: lines.clone(), caret: None, selection: None, editable: false };
    let toks = tokens(&text, lang);
    drawing(want, name, move |canvas, pen| {
        let t = pen.theme;
        let px = |p: f32| p * pen.scale;
        let style = Style::monospace(px(size));
        let letter = pen.fonts.layout("0", &style, None).width.max(1.0);
        let under = [t.panel];
        let (plain, quiet) = (t.words_on(&under), t.quiet_on(&under));
        let inks = theme::sheet().code.map(|c| t.ink_on(c, &under));
        let left = px(PAD) + (gutter + 2) as f32 * letter;
        let mut start = 0;
        let mut k = 0;
        for (n, line) in text.split('\n').enumerate() {
            let y = px(PAD) + n as f32 * px(size * LINE);
            let number = (n + 1).to_string();
            let nl = pen.fonts.layout(&number, &style, None);
            pen.fonts.draw(canvas, &nl, px(PAD) + (gutter - number.len()) as f32 * letter, y, quiet);
            let end = start + line.len();
            while k < toks.len() && toks[k].0.end <= start {
                k += 1;
            }
            let mut at = start;
            let mut j = k;
            while at < end {
                let (to, ink) = match toks.get(j) {
                    Some((r, kind)) if r.start <= at => (r.end.min(end), inks[*kind as usize]),
                    Some((r, _)) if r.start < end => (r.start, plain),
                    _ => (end, plain),
                };
                let piece = text[at..to].trim_end_matches('\r');
                if !piece.trim().is_empty() {
                    let col = text[start..at].chars().count() as f32;
                    pen.fonts.draw(canvas, &pen.fonts.layout(piece, &style, None), left + col * letter, y, ink);
                }
                if toks.get(j).is_some_and(|(r, _)| r.end <= to) {
                    j += 1;
                }
                at = to;
            }
            start = end + 1;
        }
    }).reads(reading)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(text: &str, lang: Lang) -> Vec<(&str, Kind)> {
        tokens(text, lang).into_iter().map(|(r, k)| (&text[r], k)).collect()
    }

    #[test]
    fn rust_words_strings_numbers_and_comments_are_found() {
        let src = "pub fn main() -> Option<u8> { let s = \"a \\\" b\"; let n = 1_000.5; 'x'; /* one /* two */ */ // done\n}";
        assert_eq!(kinds(src, Lang::Rust), vec![
            ("pub", Kind::Keyword), ("fn", Kind::Keyword), ("Option", Kind::Type), ("u8", Kind::Type), ("let", Kind::Keyword),
            ("\"a \\\" b\"", Kind::Text), ("let", Kind::Keyword), ("1_000.5", Kind::Number), ("'x'", Kind::Text),
            ("/* one /* two */ */", Kind::Comment), ("// done", Kind::Comment),
        ]);
        assert_eq!(kinds("fn f<'a>(x: &'a str) { r#\"raw \"q\"\"#; 1..2 }", Lang::Rust), vec![
            ("fn", Kind::Keyword), ("str", Kind::Type), ("r#\"raw \"q\"\"#", Kind::Text), ("1", Kind::Number), ("2", Kind::Number),
        ]);
    }

    #[test]
    fn shell_words_variables_and_comments_are_found() {
        let src = "if [ \"$HOME\" ]; then echo 'it''s' ${PATH} $1 # note\nfi";
        assert_eq!(kinds(src, Lang::Shell), vec![
            ("if", Kind::Keyword), ("\"$HOME\"", Kind::Text), ("then", Kind::Keyword), ("'it'", Kind::Text), ("'s'", Kind::Text),
            ("${PATH}", Kind::Type), ("$1", Kind::Type), ("# note", Kind::Comment), ("fi", Kind::Keyword),
        ]);
        assert!(kinds("a#b", Lang::Shell).is_empty(), "a hash inside a word is no comment");
        assert!(tokens("fn main", Lang::Plain).is_empty());
    }

    #[test]
    fn the_language_is_guessed_from_the_name_or_the_first_line() {
        assert_eq!(Lang::guess("main.rs", ""), Lang::Rust);
        assert_eq!(Lang::guess("run", "#!/bin/bash"), Lang::Shell);
        assert_eq!(Lang::guess("notes.txt", "hello"), Lang::Plain);
    }
}
