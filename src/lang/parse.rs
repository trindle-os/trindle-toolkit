//! A small reader for the part of Project Fluent that Trindle uses:

use std::collections::{BTreeMap, BTreeSet, HashMap};

use super::plural;

#[derive(Clone, Debug, PartialEq)]
pub enum Arg {
    Text(String),
    Num(f64),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gender {
    Male,
    Female,
    Other,
}

impl From<Gender> for Arg {
    fn from(g: Gender) -> Arg {
        Arg::Text(match g {
            Gender::Male => "male",
            Gender::Female => "female",
            Gender::Other => "other",
        }.to_string())
    }
}

impl From<&str> for Arg {
    fn from(s: &str) -> Arg {
        Arg::Text(s.to_string())
    }
}
impl From<String> for Arg {
    fn from(s: String) -> Arg {
        Arg::Text(s)
    }
}
impl From<&String> for Arg {
    fn from(s: &String) -> Arg {
        Arg::Text(s.clone())
    }
}
impl From<f64> for Arg {
    fn from(n: f64) -> Arg {
        Arg::Num(n)
    }
}
impl From<f32> for Arg {
    fn from(n: f32) -> Arg {
        Arg::Num(n as f64)
    }
}
macro_rules! numbers {
    ($($t:ty),*) => {$(impl From<$t> for Arg { fn from(n: $t) -> Arg { Arg::Num(n as f64) } })*};
}
numbers!(i8, i16, i32, i64, isize, u8, u16, u32, u64, usize);

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Piece {
    Text(String),
    Var(String),
    Ref(String),
    Select { var: String, variants: Vec<Variant>, default: usize },
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Key {
    Num(f64),
    Name(String),
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Variant {
    pub key: Key,
    pub pieces: Vec<Piece>,
}

#[derive(Clone, Debug)]
pub struct Message {
    pub(crate) pieces: Vec<Piece>,
    pub(crate) plain: Option<&'static str>,
}

impl Message {
    fn new(pieces: Vec<Piece>) -> Message {
        let plain = match pieces.as_slice() {
            [] => Some(""),
            _ => None,
        };
        Message { pieces, plain }
    }

    pub(crate) fn keep(&mut self) {
        if let [Piece::Text(t)] = self.pieces.as_slice() {
            self.plain = Some(Box::leak(t.clone().into_boxed_str()));
        }
    }

    pub(crate) fn mismatch(&self, english: &Message, lang: &str) -> Option<String> {
        let (mine, theirs) = (Shape::of(&self.pieces), Shape::of(&english.pieces));
        let (mine_vars, their_vars) = (mine.without_genders(), theirs.without_genders());
        if mine_vars != their_vars {
            return Some(format!("its variables {mine_vars:?} are not English's {their_vars:?}"));
        }
        if mine.refs != theirs.refs {
            return Some("its references to other messages differ from English".into());
        }
        if theirs.selects.is_empty() {
            return None;
        }
        let needed = plural::categories(lang);
        for var in theirs.selects.keys() {
            let plural_lines = theirs.selects[var].iter().all(|n| matches!(n.as_str(), "zero" | "one" | "two" | "few" | "many" | "other"));
            if needed.len() < 2 || !plural_lines {
                continue;
            }
            let Some(lines) = mine.selects.get(var) else {
                return Some(format!("it has no plural choice on {var}"));
            };
            if let Some(missing) = needed.iter().find(|c| **c != "other" && !lines.iter().any(|l| l == *c)) {
                return Some(format!("its plural choice on {var} has no {missing} line"));
            }
        }
        None
    }

    pub(crate) fn stray_plural(&self, lang: &str) -> Option<String> {
        let selects = Shape::of(&self.pieces).selects;
        let plural = |lines: &Vec<String>| lines.iter().all(|n| matches!(n.as_str(), "zero" | "one" | "two" | "few" | "many" | "other"));
        if !selects.values().any(plural) {
            return None;
        }
        let mut forms = plural::categories(lang);
        forms.extend([1e6, 1e9].map(|n| plural::category(lang, n)));
        selects.iter().filter(|(_, lines)| plural(lines)).find_map(|(var, lines)| {
            lines.iter().find(|l| !forms.contains(l.as_str())).map(|l| format!("its plural choice on {var} has a {l} line, which {lang} never uses"))
        })
    }

    pub fn variables(&self) -> BTreeSet<String> {
        Shape::of(&self.pieces).vars
    }

    pub fn references(&self) -> BTreeSet<String> {
        Shape::of(&self.pieces).refs
    }
}

struct Shape {
    vars: BTreeSet<String>,
    read: BTreeSet<String>,
    refs: BTreeSet<String>,
    selects: BTreeMap<String, Vec<String>>,
}

impl Shape {
    fn of(pieces: &[Piece]) -> Shape {
        let mut s = Shape { vars: BTreeSet::new(), read: BTreeSet::new(), refs: BTreeSet::new(), selects: BTreeMap::new() };
        s.walk(pieces);
        s
    }

    fn without_genders(&self) -> BTreeSet<String> {
        let gender = |v: &String| !self.read.contains(v) && self.selects[v].iter().any(|n| n == "male" || n == "female");
        self.vars.iter().filter(|v| !self.selects.contains_key(*v) || !gender(v)).cloned().collect()
    }

    fn walk(&mut self, pieces: &[Piece]) {
        for p in pieces {
            match p {
                Piece::Text(_) => {}
                Piece::Var(v) => {
                    self.vars.insert(v.clone());
                    self.read.insert(v.clone());
                }
                Piece::Ref(r) => {
                    self.refs.insert(r.clone());
                }
                Piece::Select { var, variants, .. } => {
                    self.vars.insert(var.clone());
                    let names = self.selects.entry(var.clone()).or_default();
                    names.extend(variants.iter().filter_map(|v| match &v.key {
                        Key::Name(n) => Some(n.clone()),
                        Key::Num(_) => None,
                    }));
                    for v in variants {
                        self.walk(&v.pieces);
                    }
                }
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Fault {
    pub line: usize,
    pub why: String,
}

#[derive(Clone, Debug, Default)]
pub struct Parsed {
    pub messages: HashMap<String, Message>,
    pub order: Vec<String>,
    pub faults: Vec<Fault>,
}

fn is_ident(s: &str) -> bool {
    let s = s.strip_prefix('-').unwrap_or(s);
    let mut chars = s.chars();
    chars.next().is_some_and(|c| c.is_ascii_alphabetic()) && chars.all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

fn body_of<'a>(lines: &[&'a str], at: &mut usize, first: &'a str) -> Vec<&'a str> {
    let mut body = vec![first];
    while *at < lines.len() {
        let next = lines[*at];
        if next.starts_with([' ', '\t']) {
            body.push(next);
            *at += 1;
        } else if next.trim().is_empty() {
            let end = lines[*at..].iter().position(|l| !l.trim().is_empty()).map_or(lines.len(), |p| *at + p);
            if end < lines.len() && lines[end].starts_with([' ', '\t']) {
                body.extend_from_slice(&lines[*at..end]);
                *at = end;
            } else {
                break;
            }
        } else {
            break;
        }
    }
    body
}

fn entries(text: &str) -> Vec<(usize, &str, Vec<&str>)> {
    let lines: Vec<&str> = text.trim_start_matches('\u{feff}').lines().collect();
    let mut out = Vec::new();
    let mut at = 0;
    while at < lines.len() {
        let line = lines[at];
        let number = at + 1;
        at += 1;
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        let body = body_of(&lines, &mut at, line);
        out.push((number, line, body[1..].to_vec()));
    }
    out
}

pub fn parse(text: &str) -> Parsed {
    let mut out = Parsed::default();
    for (number, line, rest) in entries(text) {
        if line.starts_with([' ', '\t']) {
            out.faults.push(Fault { line: number, why: "a line that is indented but belongs to no message".into() });
            continue;
        }
        let Some((key, first)) = line.split_once('=') else {
            out.faults.push(Fault { line: number, why: "a message is written key = words".into() });
            continue;
        };
        let key = key.trim();
        if !is_ident(key) {
            out.faults.push(Fault { line: number, why: format!("{key:?} is not a key (letters, numbers, - and _, starting with a letter)") });
            continue;
        }
        match read_pattern(&dedent(first, &rest)) {
            Ok(pieces) => {
                if out.messages.contains_key(key) {
                    out.faults.push(Fault { line: number, why: format!("{key} is written twice; the first is kept") });
                } else {
                    out.order.push(key.to_string());
                    out.messages.insert(key.to_string(), Message::new(pieces));
                }
            }
            Err(why) => out.faults.push(Fault { line: number, why: format!("{key}: {why}") }),
        }
    }
    out
}

fn dedent(first: &str, rest: &[&str]) -> Vec<char> {
    let indent = rest.iter().filter(|l| !l.trim().is_empty()).map(|l| l.len() - l.trim_start().len()).min().unwrap_or(0);
    let mut out = String::new();
    out.push_str(first.trim_start());
    for l in rest {
        out.push('\n');
        if l.len() >= indent && l.is_char_boundary(indent) {
            out.push_str(&l[indent..]);
        } else {
            out.push_str(l.trim_start());
        }
    }
    out.trim_end().chars().collect()
}

pub fn raw_messages(text: &str) -> Vec<(String, String)> {
    entries(text).into_iter()
        .filter(|(_, line, _)| !line.starts_with([' ', '\t']))
        .filter_map(|(_, line, rest)| {
            let (key, first) = line.split_once('=')?;
            let key = key.trim();
            (is_ident(key) && !key.starts_with('-')).then(|| (key.to_string(), dedent(first, &rest).into_iter().collect()))
        })
        .collect()
}

pub fn placeholders(words: &str) -> Option<Vec<String>> {
    let chars: Vec<char> = words.trim_end().chars().collect();
    let shape = Shape::of(&read_pattern(&chars).ok()?);
    let mut all = BTreeSet::new();
    all.extend(shape.read.iter().map(|v| format!("${v}")));
    all.extend(shape.refs.iter().map(|r| format!("&{r}")));
    all.extend(shape.selects.keys().map(|v| format!("?{v}")));
    Some(all.into_iter().collect())
}

const DEEPEST: u8 = 8;

struct Reader<'a> {
    c: &'a [char],
    i: usize,
    depth: u8,
}

type R<T> = Result<T, String>;

fn read_pattern(chars: &[char]) -> R<Vec<Piece>> {
    let mut r = Reader { c: chars, i: 0, depth: 0 };
    while r.c.get(r.i).is_some_and(|c| *c == '\n') {
        r.i += 1;
    }
    if r.i > 0 && r.c.get(r.i) == Some(&'.') {
        return Err("attributes (.name = ...) are not supported".into());
    }
    let pieces = r.pattern(false)?;
    if r.i < r.c.len() {
        return Err("words after the end of the message".into());
    }
    Ok(pieces)
}

impl Reader<'_> {
    fn peek(&self) -> Option<char> {
        self.c.get(self.i).copied()
    }

    fn skip_space(&mut self) {
        while self.peek().is_some_and(char::is_whitespace) {
            self.i += 1;
        }
    }

    fn eat(&mut self, s: &str) -> bool {
        let want: Vec<char> = s.chars().collect();
        if self.c.get(self.i..self.i + want.len()) == Some(&want[..]) {
            self.i += want.len();
            true
        } else {
            false
        }
    }

    fn word(&mut self) -> String {
        let start = self.i;
        while self.peek().is_some_and(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
            self.i += 1;
        }
        self.c[start..self.i].iter().collect()
    }

    fn pattern(&mut self, in_variant: bool) -> R<Vec<Piece>> {
        let mut pieces = Vec::new();
        let mut text = String::new();
        while let Some(c) = self.peek() {
            match c {
                '{' => {
                    self.i += 1;
                    if !text.is_empty() {
                        pieces.push(Piece::Text(std::mem::take(&mut text)));
                    }
                    pieces.push(self.placeable()?);
                }
                '\n' => {
                    if in_variant {
                        let mut j = self.i + 1;
                        while self.c.get(j).is_some_and(|c| c.is_whitespace()) {
                            j += 1;
                        }
                        if matches!(self.c.get(j), Some('[' | '*' | '}') | None) {
                            break;
                        }
                        self.i = j;
                        if !(text.is_empty() && pieces.is_empty()) {
                            text.push('\n');
                        }
                    } else {
                        text.push('\n');
                        self.i += 1;
                        let mut j = self.i;
                        while self.c.get(j).is_some_and(|c| *c == ' ' || *c == '\t') {
                            j += 1;
                        }
                        if self.c.get(j) == Some(&'.') {
                            return Err("attributes (.name = ...) are not supported".into());
                        }
                    }
                }
                '}' if in_variant => break,
                _ => {
                    text.push(c);
                    self.i += 1;
                }
            }
        }
        if !text.is_empty() {
            pieces.push(Piece::Text(text.trim_end().to_string()));
        }
        pieces.retain(|p| !matches!(p, Piece::Text(t) if t.is_empty()));
        Ok(pieces)
    }

    fn placeable(&mut self) -> R<Piece> {
        self.skip_space();
        let (piece, closed) = match self.peek() {
            Some('$') => {
                self.i += 1;
                let var = self.word();
                if var.is_empty() {
                    return Err("a $ with no variable name".into());
                }
                self.after_var(var)?
            }
            Some('"') => {
                self.i += 1;
                (Piece::Text(self.quoted()?), false)
            }
            Some('{') => return Err("a placeable inside a placeable is not supported".into()),
            Some(c) if c.is_ascii_alphabetic() || c == '-' => self.named()?,
            _ => return Err("a { with nothing readable in it".into()),
        };
        if closed {
            return Ok(piece);
        }
        self.skip_space();
        if !self.eat("}") {
            return Err("a { is never closed with }".into());
        }
        Ok(piece)
    }

    fn after_var(&mut self, var: String) -> R<(Piece, bool)> {
        self.skip_space();
        if self.eat("->") {
            return Ok((self.select(var)?, true));
        }
        Ok((Piece::Var(var), false))
    }

    fn quoted(&mut self) -> R<String> {
        let mut s = String::new();
        loop {
            match self.peek() {
                Some('"') => {
                    self.i += 1;
                    return Ok(s);
                }
                Some('\\') => {
                    self.i += 1;
                    match self.peek() {
                        Some(c @ ('"' | '\\' | '{')) => {
                            s.push(c);
                            self.i += 1;
                        }
                        _ => return Err("unknown escape in quoted text".into()),
                    }
                }
                Some('\n') | None => return Err("quoted text is not closed".into()),
                Some(c) => {
                    s.push(c);
                    self.i += 1;
                }
            }
        }
    }

    fn named(&mut self) -> R<(Piece, bool)> {
        let name = self.word();
        if name == "NUMBER" && self.peek() == Some('(') {
            self.i += 1;
            self.skip_space();
            if !self.eat("$") {
                return Err("NUMBER needs a variable".into());
            }
            let var = self.word();
            self.skip_space();
            if !self.eat(")") {
                return Err("NUMBER( is not closed".into());
            }
            self.after_var(var)
        } else if self.peek() == Some('.') || self.peek() == Some('(') {
            Err("attributes and functions are not supported".into())
        } else {
            Ok((Piece::Ref(name), false))
        }
    }

    fn select(&mut self, var: String) -> R<Piece> {
        if self.depth >= DEEPEST {
            return Err("selectors inside selectors go too deep".into());
        }
        self.depth += 1;
        let made = self.variants(var);
        self.depth -= 1;
        made
    }

    fn variants(&mut self, var: String) -> R<Piece> {
        let mut variants = Vec::new();
        let mut default = None;
        loop {
            self.skip_space();
            if self.eat("}") {
                break;
            }
            let star = self.eat("*");
            if !self.eat("[") {
                return Err("a selector line must start [category] or *[category]".into());
            }
            self.skip_space();
            let start = self.i;
            while self.peek().is_some_and(|c| c != ']' && c != '\n') {
                self.i += 1;
            }
            let raw: String = self.c[start..self.i].iter().collect();
            if !self.eat("]") {
                return Err("a [category] is not closed".into());
            }
            let raw = raw.trim();
            let key = match raw.parse::<f64>() {
                Ok(n) if raw.chars().next().is_some_and(|c| c.is_ascii_digit() || c == '-') => Key::Num(n),
                _ if is_ident(raw) => Key::Name(raw.to_string()),
                _ => return Err(format!("{raw:?} is not a category or a number")),
            };
            while self.peek().is_some_and(|c| c == ' ' || c == '\t') {
                self.i += 1;
            }
            let pieces = self.pattern(true)?;
            if star {
                if default.is_some() {
                    return Err("two * defaults in one selector".into());
                }
                default = Some(variants.len());
            }
            variants.push(Variant { key, pieces });
        }
        let Some(default) = default else { return Err("a selector needs one *[other] line".into()) };
        Ok(Piece::Select { var, variants, default })
    }
}

pub(crate) fn number(n: f64) -> String {
    if n.fract() == 0.0 && n.abs() < 1e15 {
        format!("{}", n as i64)
    } else {
        format!("{n}")
    }
}

pub(crate) const MOST_WRITTEN: usize = 1 << 16;

pub(crate) trait Lookup {
    fn find(&self, key: &str) -> Option<(&Message, &str)>;
}

pub(crate) fn render(pieces: &[Piece], lang: &str, args: &[(&str, Arg)], book: &dyn Lookup, chain: &[&str], out: &mut String) {
    let arg = |name: &str| args.iter().find(|(n, _)| *n == name).map(|(_, a)| a);
    for p in pieces {
        match p {
            Piece::Text(t) => out.push_str(t),
            Piece::Var(v) => match arg(v) {
                Some(Arg::Text(t)) => out.push_str(t),
                Some(Arg::Num(n)) => out.push_str(&number(*n)),
                None => {}
            },
            Piece::Ref(key) => {
                if chain.len() < 12 && out.len() < MOST_WRITTEN && !chain.contains(&key.as_str()) {
                    if let Some((m, from)) = book.find(key) {
                        render(&m.pieces, from, args, book, &[chain, &[key.as_str()]].concat(), out);
                    }
                }
            }
            Piece::Select { var, variants, default } => {
                let chosen = match arg(var) {
                    Some(Arg::Num(n)) => {
                        let category = plural::category(lang, *n);
                        variants.iter().find(|v| matches!(v.key, Key::Num(k) if k == *n))
                            .or_else(|| variants.iter().find(|v| matches!(&v.key, Key::Name(k) if k == category)))
                    }
                    Some(Arg::Text(s)) => variants.iter().find(|v| matches!(&v.key, Key::Name(k) if k == s)),
                    None => None,
                };
                let variant = chosen.or_else(|| variants.get(*default));
                if let Some(v) = variant {
                    render(&v.pieces, lang, args, book, chain, out);
                }
            }
        }
    }
}
