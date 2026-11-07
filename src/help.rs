//! Help pages from simple markdown or HTML: headings, paragraphs, lists, links, code and pictures' words, shown with the toolkit's own widgets.

use crate::code::{self, Lang};
use crate::rich::{self, Span};
use crate::widget::{column, drawing, heading, note, page_title, rich, row, section, Node};

const INDENT: f32 = 20.0;

#[derive(Clone, Debug, PartialEq)]
pub enum Block {
    Heading(u8, String),
    Para(Vec<Span>),
    Item(String, usize, Vec<Span>),
    Code(String, Lang),
    Picture(String),
}

pub fn markdown(text: &str) -> Vec<Block> {
    let mut out = Vec::new();
    let mut para: Vec<&str> = Vec::new();
    let flush = |out: &mut Vec<Block>, para: &mut Vec<&str>| {
        if !para.is_empty() {
            out.push(Block::Para(rich::parse(&para.join(" "))));
            para.clear();
        }
    };
    let mut lines = text.lines();
    while let Some(line) = lines.next() {
        let trimmed = line.trim();
        let indent = line.len() - line.trim_start().len();
        if let Some(fence) = trimmed.strip_prefix("```") {
            flush(&mut out, &mut para);
            let body: Vec<&str> = lines.by_ref().take_while(|l| !l.trim_start().starts_with("```")).collect();
            let lang = match fence.trim() {
                "rust" | "rs" => Lang::Rust,
                "sh" | "bash" | "shell" | "zsh" => Lang::Shell,
                _ => Lang::guess("", body.first().copied().unwrap_or("")),
            };
            out.push(Block::Code(body.join("\n"), lang));
        } else if trimmed.is_empty() {
            flush(&mut out, &mut para);
        } else if let Some((level, words)) = heading_of(trimmed) {
            flush(&mut out, &mut para);
            out.push(Block::Heading(level, words.to_string()));
        } else if let Some(alt) = trimmed.strip_prefix("![").and_then(|r| r.split_once("](")).filter(|_| trimmed.ends_with(')')).map(|(a, _)| a) {
            flush(&mut out, &mut para);
            out.push(Block::Picture(alt.to_string()));
        } else if let Some((mark, words)) = item_of(trimmed) {
            flush(&mut out, &mut para);
            out.push(Block::Item(mark, indent / 2, rich::parse(words)));
        } else {
            para.push(trimmed);
        }
    }
    flush(&mut out, &mut para);
    out
}

fn heading_of(line: &str) -> Option<(u8, &str)> {
    let level = line.bytes().take_while(|&b| b == b'#').count();
    let words = line[level..].strip_prefix(' ')?;
    (1..=6).contains(&level).then(|| (level as u8, words.trim_end_matches('#').trim()))
}

fn item_of(line: &str) -> Option<(String, &str)> {
    if let Some(w) = line.strip_prefix("- ").or_else(|| line.strip_prefix("* ")) {
        return Some(("\u{2022}".into(), w));
    }
    let digits = line.bytes().take_while(u8::is_ascii_digit).count();
    let w = line.get(digits..)?.strip_prefix(". ")?;
    (digits > 0).then(|| (format!("{}.", &line[..digits]), w))
}

#[derive(Default)]
struct Gathered {
    out: Vec<Block>,
    spans: Vec<Span>,
    now: Span,
    into: Option<(u8, String)>,
    lists: Vec<Option<usize>>,
    pre: Option<String>,
}

impl Gathered {
    fn end_block(&mut self) {
        let words: Vec<Span> = tidy(std::mem::take(&mut self.spans));
        let target = self.into.take();
        if words.is_empty() {
            return;
        }
        match target {
            Some((level @ 1..=6, _)) => self.out.push(Block::Heading(level, words.iter().map(|s| s.text.as_str()).collect())),
            Some((_, mark)) => self.out.push(Block::Item(mark, self.lists.len().saturating_sub(1), words)),
            None => self.out.push(Block::Para(words)),
        }
    }

    fn add(&mut self, words: &str) {
        add_words(&mut self.spans, &self.now, &mut self.pre, words);
    }

    fn tag<'t>(&mut self, tag: &str, name: &str, shut: bool, rest: &mut &'t str) {
    match (name, shut) {
        ("script" | "style", false) => {
            let end = format!("</{name}");
            let left: &'t str = rest;
            *rest = left.as_bytes().windows(end.len()).position(|w| w.eq_ignore_ascii_case(end.as_bytes())).map_or("", |at| &left[at..]);
        }
        ("h1" | "h2" | "h3" | "h4" | "h5" | "h6", false) => {
            self.end_block();
            self.into = Some((name.as_bytes()[1] - b'0', String::new()));
        }
        ("p" | "div" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "li" | "ul" | "ol", true) | ("p" | "div", false) => {
            self.end_block();
            if matches!(name, "ul" | "ol") && shut {
                self.lists.pop();
            }
        }
        ("ul", false) => {
            self.end_block();
            self.lists.push(None);
        }
        ("ol", false) => {
            self.end_block();
            self.lists.push(Some(0));
        }
        ("li", false) => {
            self.end_block();
            let mark = match self.lists.last_mut() {
                Some(Some(n)) => {
                    *n += 1;
                    format!("{n}.")
                }
                _ => "\u{2022}".into(),
            };
            self.into = Some((0, mark));
        }
        ("br", _) => add_words(&mut self.spans, &self.now, &mut self.pre, "\n"),
        ("b" | "strong", s) => self.now.bold = !s,
        ("i" | "em", s) => self.now.italic = !s,
        ("code", s) if self.pre.is_none() => self.now.code = !s,
        ("a", false) => self.now.link = attr(tag, "href"),
        ("a", true) => self.now.link = None,
        ("pre", false) => {
            self.end_block();
            self.pre = Some(String::new());
        }
        ("pre", true) => {
            let body = self.pre.take().unwrap_or_default();
            let lang = if attr(tag, "class").is_some_and(|c| c.contains("rust")) { Lang::Rust } else { Lang::guess("", body.lines().next().unwrap_or("")) };
            self.out.push(Block::Code(body.trim_matches('\n').to_string(), lang));
        }
        ("img", _) => {
            self.end_block();
            self.out.push(Block::Picture(attr(tag, "alt").unwrap_or_default()));
        }
        _ => {}
    }
    }
}

pub fn html(text: &str) -> Vec<Block> {
    let mut g = Gathered::default();
    let mut rest = text;
    while !rest.is_empty() {
        let Some(open) = rest.find('<') else {
            g.add(rest);
            break;
        };
        g.add(&rest[..open]);
        let Some(close) = rest[open..].find('>') else {
            g.add(&rest[open..]);
            break;
        };
        let tag = &rest[open + 1..open + close];
        rest = &rest[open + close + 1..];
        let shut = tag.starts_with('/');
        let name = tag.trim_start_matches('/').split(|c: char| c.is_whitespace() || c == '/').next().unwrap_or("").to_ascii_lowercase();
        g.tag(tag, &name, shut, &mut rest);
    }
    g.end_block();
    g.out
}

fn attr(tag: &str, name: &str) -> Option<String> {
    let low = tag.to_ascii_lowercase();
    let at = low.match_indices(name).find(|(i, _)| low[i + name.len()..].trim_start().starts_with('=') && (*i == 0 || low.as_bytes()[i - 1].is_ascii_whitespace()))?.0;
    let after = tag[at + name.len()..].trim_start().strip_prefix('=')?.trim_start();
    let value = match after.chars().next()? {
        q @ ('"' | '\'') => after[1..].split(q).next()?,
        _ => after.split(|c: char| c.is_whitespace() || c == '>').next()?,
    };
    Some(entities(value))
}

fn add_words(spans: &mut Vec<Span>, now: &Span, pre: &mut Option<String>, raw: &str) {
    if let Some(code) = pre {
        code.push_str(&entities(raw));
        return;
    }
    let words = entities(raw);
    let squeezed: String = if raw == "\n" { "\n".into() } else {
        let mut s = String::new();
        for c in words.chars() {
            if c.is_whitespace() {
                if !s.ends_with(' ') {
                    s.push(' ');
                }
            } else {
                s.push(c);
            }
        }
        s
    };
    if squeezed.is_empty() {
        return;
    }
    match spans.last_mut() {
        Some(last) if (last.bold, last.italic, last.code, &last.link) == (now.bold, now.italic, now.code, &now.link) => last.text.push_str(&squeezed),
        _ => spans.push(Span { text: squeezed, ..now.clone() }),
    }
}

fn tidy(mut spans: Vec<Span>) -> Vec<Span> {
    if let Some(first) = spans.first_mut() {
        first.text = first.text.trim_start_matches(' ').to_string();
    }
    if let Some(last) = spans.last_mut() {
        last.text = last.text.trim_end_matches(' ').to_string();
    }
    spans.retain(|s| !s.text.is_empty());
    spans
}

fn entities(s: &str) -> String {
    let mut out = String::new();
    let mut rest = s;
    while let Some(at) = rest.find('&') {
        out.push_str(&rest[..at]);
        let tail = &rest[at..];
        let end = tail.find(';').filter(|&e| e <= 10);
        let named = end.map(|e| &tail[1..e]);
        let c = named.and_then(|n| match n {
            "amp" => Some('&'),
            "lt" => Some('<'),
            "gt" => Some('>'),
            "quot" => Some('"'),
            "apos" => Some('\''),
            "nbsp" => Some('\u{a0}'),
            _ => n.strip_prefix('#').and_then(|d| match d.strip_prefix(['x', 'X']) {
                Some(h) => u32::from_str_radix(h, 16).ok(),
                None => d.parse().ok(),
            }).and_then(char::from_u32),
        });
        match (c, end) {
            (Some(c), Some(e)) => {
                out.push(c);
                rest = &tail[e + 1..];
            }
            _ => {
                out.push('&');
                rest = &tail[1..];
            }
        }
    }
    out.push_str(rest);
    out
}

pub fn view<M: 'static>(blocks: &[Block], follow: impl Fn(String) -> M + Clone + 'static) -> Node<M> {
    let items = blocks.iter().map(|b| match b {
        Block::Heading(1, w) => page_title(w.clone()),
        Block::Heading(2, w) => section(w.clone()),
        Block::Heading(_, w) => heading(w.clone()),
        Block::Para(spans) => rich(spans.clone()).on_link(follow.clone()),
        Block::Item(mark, depth, spans) => {
            let mut parts = vec![note(mark.clone()), rich(spans.clone()).on_link(follow.clone())];
            if *depth > 0 {
                parts.insert(0, drawing((*depth as f32 * INDENT, 1.0), "", |_, _| {}));
            }
            row(parts).gap(crate::theme::gaps().medium)
        }
        Block::Code(text, lang) => code::view(text, *lang, crate::t!("kit-help-code")),
        Block::Picture(words) => note(words.clone()),
    }).collect();
    column(items).gap(crate::theme::gaps().roomy)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markdown_is_read_into_blocks() {
        let page = "# Help\n\nPress **Save** to\nkeep it. See [more](more.md).\n\n## Steps\n- one\n  - inner\n2. two\n\n```rust\nfn main() {}\n```\n![A window](w.png)";
        assert_eq!(markdown(page), vec![
            Block::Heading(1, "Help".into()),
            Block::Para(vec![Span::plain("Press "), Span::plain("Save").bold(), Span::plain(" to keep it. See "), Span::link("more", "more.md"), Span::plain(".")]),
            Block::Heading(2, "Steps".into()),
            Block::Item("\u{2022}".into(), 0, vec![Span::plain("one")]),
            Block::Item("\u{2022}".into(), 1, vec![Span::plain("inner")]),
            Block::Item("2.".into(), 0, vec![Span::plain("two")]),
            Block::Code("fn main() {}".into(), Lang::Rust),
            Block::Picture("A window".into()),
        ]);
    }

    #[test]
    fn html_is_read_into_the_same_blocks() {
        let page = "<h1>Help</h1><script>alert('no')</script><p>Press <b>Save</b>   to\nkeep it &amp; see <a href=\"more.html\">more</a>.</p>\
                    <ol><li>one</li><li><em>two</em></li></ol><pre>a &lt; b</pre><img src=x.png alt='A window'>";
        assert_eq!(html(page), vec![
            Block::Heading(1, "Help".into()),
            Block::Para(vec![Span::plain("Press "), Span::plain("Save").bold(), Span::plain(" to keep it & see "), Span::link("more", "more.html"), Span::plain(".")]),
            Block::Item("1.".into(), 0, vec![Span::plain("one")]),
            Block::Item("2.".into(), 0, vec![Span::plain("two").italic()]),
            Block::Code("a < b".into(), Lang::Plain),
            Block::Picture("A window".into()),
        ]);
        assert_eq!(entities("&#65;&#x42;&bogus; &"), "AB&bogus; &");
    }
}
