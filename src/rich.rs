//! Words in several looks in one paragraph (bold, italic, code and links), flowed and wrapped as one, and a small markup for them.

use std::ops::Range;

use tiny_skia::PixmapMut;
use unicode_bidi::{BidiInfo, Level};
use unicode_linebreak::BreakOpportunity;
use unicode_segmentation::UnicodeSegmentation;

use crate::colour::Colour;
use crate::draw::words_on;
use crate::paint::Painter;
use crate::text::{Fonts, Layout, Style, MONOSPACE};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Span {
    pub text: String,
    pub bold: bool,
    pub italic: bool,
    pub code: bool,
    pub link: Option<String>,
}

impl Span {
    pub fn plain(text: impl Into<String>) -> Span {
        Span { text: text.into(), ..Span::default() }
    }

    pub fn link(text: impl Into<String>, address: impl Into<String>) -> Span {
        Span { text: text.into(), link: Some(address.into()), ..Span::default() }
    }

    pub fn bold(mut self) -> Span {
        self.bold = true;
        self
    }

    pub fn italic(mut self) -> Span {
        self.italic = true;
        self
    }

    pub fn code(mut self) -> Span {
        self.code = true;
        self
    }

    fn with(&self, text: String) -> Span {
        Span { text, ..self.clone() }
    }

    fn same_look(&self, o: &Span) -> bool {
        (self.bold, self.italic, self.code, &self.link) == (o.bold, o.italic, o.code, &o.link)
    }
}

pub fn style_of(span: &Span, base: &Style) -> Style {
    Style {
        family: if span.code { Some(MONOSPACE.to_string()) } else { base.family.clone() },
        weight: if span.bold { 700 } else { base.weight },
        italic: span.italic || base.italic,
        ..base.clone()
    }
}

fn flush(out: &mut Vec<Span>, word: &mut String, now: &Span) {
    if !word.is_empty() {
        push(out, now.with(std::mem::take(word)));
    }
}

pub fn parse(markup: &str) -> Vec<Span> {
    let chars: Vec<(usize, char)> = markup.char_indices().collect();
    let mut out: Vec<Span> = Vec::new();
    let mut now = Span::default();
    let mut word = String::new();
    let rest = |i: usize| &markup[chars.get(i).map_or(markup.len(), |c| c.0)..];
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i].1;
        let next = chars.get(i + 1).map(|c| c.1);
        let before = i.checked_sub(1).map(|j| chars[j].1);
        match c {
            '\\' if next.is_some() => {
                word.extend(next);
                i += 2;
                continue;
            }
            '`' => {
                if let Some((body, used)) = code_at(rest(i + 1)) {
                    flush(&mut out, &mut word, &now);
                    push(&mut out, Span { code: true, ..now.with(body) });
                    i += 1 + rest(i + 1)[..used].chars().count();
                    continue;
                }
                word.push(c);
            }
            '*' if next == Some('*') && (now.bold || rest(i + 2).contains("**")) => {
                flush(&mut out, &mut word, &now);
                now.bold = !now.bold;
                i += 2;
                continue;
            }
            '*' if next != Some('*') && (now.italic || rest(i + 1).contains('*')) => {
                flush(&mut out, &mut word, &now);
                now.italic = !now.italic;
            }
            '_' if (now.italic && !next.is_some_and(char::is_alphanumeric))
                || (!now.italic && !before.is_some_and(char::is_alphanumeric) && rest(i + 1).contains('_')) => {
                flush(&mut out, &mut word, &now);
                now.italic = !now.italic;
            }
            '[' if now.link.is_none() => {
                if let Some((words, address, used)) = link_at(rest(i + 1)) {
                    flush(&mut out, &mut word, &now);
                    let inner = Span { link: Some(address), ..now.clone() };
                    for s in parse(words) {
                        push(&mut out, Span { link: inner.link.clone(), bold: s.bold || inner.bold, italic: s.italic || inner.italic, ..s });
                    }
                    i += 1 + rest(i + 1)[..used].chars().count();
                    continue;
                }
                word.push(c);
            }
            _ => word.push(c),
        }
        i += 1;
    }
    flush(&mut out, &mut word, &now);
    out
}

fn code_at(s: &str) -> Option<(String, usize)> {
    let (mut body, mut it) = (String::new(), s.char_indices().peekable());
    while let Some((i, c)) = it.next() {
        match c {
            '`' => return Some((body, i + 1)),
            '\\' if matches!(it.peek(), Some((_, '`' | '\\'))) => body.extend(it.next().map(|(_, c)| c)),
            _ => body.push(c),
        }
    }
    None
}

fn link_at(s: &str) -> Option<(&str, String, usize)> {
    let (mut close, mut it) = (None, s.char_indices().peekable());
    while let Some((i, c)) = it.next() {
        match c {
            '\\' => {
                it.next();
            }
            '`' => {
                if let Some((_, used)) = code_at(&s[i + 1..]) {
                    while it.next_if(|(j, _)| *j < i + 1 + used).is_some() {}
                }
            }
            ']' if s[i + 1..].starts_with('(') => {
                close = Some(i);
                break;
            }
            _ => {}
        }
    }
    let close = close?;
    let (mut address, mut it) = (String::new(), s[close + 2..].char_indices().peekable());
    while let Some((i, c)) = it.next() {
        match c {
            ')' => return (!s[..close].contains('\n')).then_some((&s[..close], address, close + 2 + i + 1)),
            '\\' if matches!(it.peek(), Some((_, ')' | '\\'))) => address.extend(it.next().map(|(_, c)| c)),
            _ => address.push(c),
        }
    }
    None
}

fn push(out: &mut Vec<Span>, s: Span) {
    match out.last_mut() {
        Some(last) if last.same_look(&s) => last.text.push_str(&s.text),
        _ if !s.text.is_empty() => out.push(s),
        _ => {}
    }
}

pub fn markup(spans: &[Span]) -> String {
    let mut out = String::new();
    for s in spans {
        let mut body: String = if s.code {
            s.text.chars().flat_map(|c| if matches!(c, '`' | '\\') { vec!['\\', c] } else { vec![c] }).collect()
        } else {
            s.text.chars().flat_map(|c| if matches!(c, '*' | '_' | '`' | '[' | ']' | '\\') { vec!['\\', c] } else { vec![c] }).collect()
        };
        if s.code {
            body = format!("`{body}`");
        }
        if s.italic {
            body = format!("*{body}*");
        }
        if s.bold {
            body = format!("**{body}**");
        }
        if let Some(a) = &s.link {
            let a: String = a.chars().flat_map(|c| if matches!(c, ')' | '\\') { vec!['\\', c] } else { vec![c] }).collect();
            body = format!("[{body}]({a})");
        }
        out.push_str(&body);
    }
    out
}

#[derive(Clone, Debug)]
pub struct Piece {
    pub layout: Layout,
    pub x: f32,
    pub y: f32,
    pub at: usize,
    pub len: usize,
    pub span: usize,
}

#[derive(Clone, Debug)]
pub struct Row {
    pub top: f32,
    pub height: f32,
    pub range: Range<usize>,
    pub rtl: bool,
}

#[derive(Clone, Debug)]
pub struct Flow {
    pub pieces: Vec<Piece>,
    pub rows: Vec<Row>,
    pub width: f32,
    pub height: f32,
    room: f32,
}

pub fn flow(lay: &dyn Fn(&str, &Style) -> Layout, spans: &[Span], base: &Style, width: Option<f32>) -> Flow {
    let whole: String = spans.iter().map(|s| s.text.as_str()).collect();
    let chunks = stretches(&whole, spans);
    let lines = break_lines(lay, &whole, spans, base, width, chunks);
    place_rows(&whole, lines, &lay("", base), width)
}

fn stretches(whole: &str, spans: &[Span]) -> Vec<(Range<usize>, usize, bool)> {
    let starts: Vec<usize> = spans.iter().scan(0, |at, s| {
        let here = *at;
        *at += s.text.len();
        Some(here)
    }).collect();
    let span_of = |b: usize| (0..spans.len()).rev().find(|&i| starts[i] <= b && !spans[i].text.is_empty()).unwrap_or(0);
    let mut cuts: Vec<(usize, bool)> = unicode_linebreak::linebreaks(whole).map(|(i, k)| (i, k == BreakOpportunity::Mandatory && i < whole.len())).collect();
    cuts.extend(starts.iter().map(|&s| (s, false)));
    cuts.sort_by_key(|c| (c.0, !c.1));
    cuts.dedup_by_key(|c| c.0);
    let ws = |r: &Range<usize>| whole[r.clone()].len() - whole[r.clone()].trim_end_matches([' ', '\t']).len();
    let mut chunks: Vec<(Range<usize>, usize, bool)> = Vec::new();
    let mut from = 0;
    for (cut, hard) in cuts.into_iter().filter(|c| c.0 > 0) {
        let mut r = from..cut;
        from = cut;
        r.end -= whole[r.clone()].len() - whole[r.clone()].trim_end_matches(['\n', '\r']).len();
        chunks.push((r.clone(), span_of(r.start), hard));
    }
    for i in 0..chunks.len().saturating_sub(1) {
        if !chunks[i].2 {
            let n = ws(&chunks[i].0);
            chunks[i].0.end -= n;
            chunks[i + 1].0.start = chunks[i].0.end;
        }
    }
    chunks
}

type FlowLine = (Vec<(Range<usize>, usize, Layout)>, Range<usize>);

fn break_lines(lay: &dyn Fn(&str, &Style) -> Layout, whole: &str, spans: &[Span], base: &Style, width: Option<f32>, mut chunks: Vec<(Range<usize>, usize, bool)>) -> Vec<FlowLine> {
    let mut lines: Vec<FlowLine> = Vec::new();
    let mut line: Vec<(Range<usize>, usize, Layout)> = Vec::new();
    let mut line_from = 0;
    let mut x = 0.0;
    let mut i = 0;
    while i < chunks.len() {
        let (mut r, span, hard) = chunks[i].clone();
        if line.is_empty() {
            r.start = r.end - whole[r.clone()].trim_start_matches([' ', '\t']).len();
        }
        if !r.is_empty() {
            let mut l = lay(&whole[r.clone()], &style_of(&spans[span], base));
            if width.is_some_and(|w| x + l.width > w) && !line.is_empty() {
                lines.push((std::mem::take(&mut line), line_from..r.start));
                line_from = r.start;
                x = 0.0;
                r.start = r.end - whole[r.clone()].trim_start_matches([' ', '\t']).len();
                l = lay(&whole[r.clone()], &style_of(&spans[span], base));
            }
            if let Some(w) = width.filter(|&w| line.is_empty() && l.width > w) {
                let style = style_of(&spans[span], base);
                let cuts: Vec<usize> = whole[r.clone()].grapheme_indices(true).skip(1).map(|(k, _)| r.start + k).collect();
                let fit = cuts.partition_point(|&k| lay(&whole[r.start..k], &style).width <= w);
                if let Some(&k) = cuts.get(fit.saturating_sub(1)) {
                    line.push((r.start..k, span, lay(&whole[r.start..k], &style)));
                    lines.push((std::mem::take(&mut line), line_from..k));
                    (line_from, x) = (k, 0.0);
                    chunks[i].0.start = k;
                    continue;
                }
            }
            x += l.width;
            line.push((r.clone(), span, l));
        }
        if hard {
            let end = chunks.get(i + 1).map_or(whole.len(), |c| c.0.start);
            lines.push((std::mem::take(&mut line), line_from..end));
            line_from = end;
            x = 0.0;
        }
        i += 1;
    }
    if !line.is_empty() || lines.is_empty() {
        lines.push((line, line_from..whole.len()));
    }
    if whole.ends_with('\n') {
        lines.push((Vec::new(), whole.len()..whole.len()));
    }
    lines
}

fn place_rows(whole: &str, lines: Vec<FlowLine>, plain: &Layout, width: Option<f32>) -> Flow {
    let bidi = BidiInfo::new(whole, None);
    let level_at = |b: usize| bidi.levels.get(b).copied().unwrap_or_else(Level::ltr);
    let rtl_at = |b: usize| bidi.paragraphs.iter().find(|p| p.range.contains(&b)).or(bidi.paragraphs.last()).is_some_and(|p| p.level.is_rtl());
    let widest = lines.iter().map(|(l, _)| l.iter().map(|p| p.2.width).sum::<f32>()).fold(0.0, f32::max);
    let room = width.unwrap_or(widest);
    let (mut pieces, mut rows, mut top) = (Vec::new(), Vec::new(), 0.0);
    for (items, range) in lines {
        let rtl = rtl_at(range.start);
        let rise = |l: &Layout| l.lines.first().map_or(0.0, |f| f.baseline);
        let ascent = items.iter().map(|p| rise(&p.2)).fold(rise(plain), f32::max);
        let below = items.iter().map(|p| p.2.height - rise(&p.2)).fold(plain.height - rise(plain), f32::max);
        let mut order: Vec<(Level, usize)> = items.iter().enumerate().map(|(k, p)| {
            let first = whole[p.0.clone()].find(|c: char| !c.is_whitespace()).map_or(p.0.start, |o| p.0.start + o);
            (level_at(first), k)
        }).collect();
        if rtl || order.iter().any(|o| o.0.is_rtl()) {
            crate::text::reorder(&mut order);
        }
        let used: f32 = items.iter().map(|p| p.2.width).sum();
        let mut x = if rtl { room - used } else { 0.0 };
        for (_, k) in order {
            let (r, span, l) = &items[k];
            pieces.push(Piece { layout: l.clone(), x, y: top + ascent - rise(l), at: r.start, len: r.len(), span: *span });
            x += l.width;
        }
        rows.push(Row { top, height: ascent + below, range, rtl });
        top += ascent + below;
    }
    Flow { pieces, rows, width: widest, height: top, room }
}

impl Flow {
    pub fn span_at(&self, x: f32, y: f32) -> Option<usize> {
        self.pieces.iter().find(|p| x >= p.x && x < p.x + p.layout.width && y >= p.y && y < p.y + p.layout.height).map(|p| p.span)
    }

    pub fn bands(&self, span: usize) -> Vec<(f32, f32, f32, f32)> {
        self.pieces.iter().filter(|p| p.span == span).map(|p| (p.x, p.y, p.layout.width, p.layout.height)).collect()
    }

    fn row_of(&self, at: usize) -> usize {
        self.rows.iter().rposition(|r| r.range.start <= at).unwrap_or(0)
    }

    pub fn caret(&self, at: usize) -> (f32, f32, f32) {
        let row = self.row_of(at);
        let Some(r) = self.rows.get(row) else { return (0.0, 0.0, 0.0) };
        let here: Vec<&Piece> = self.pieces.iter().filter(|p| r.range.contains(&p.at) || (r.range.is_empty() && p.at == r.range.start)).collect();
        let inside = here.iter().find(|p| at >= p.at && at <= p.at + p.len);
        let x = match inside {
            Some(p) => p.x + p.layout.caret(at - p.at).0,
            None => match here.iter().filter(|p| p.at < at).max_by_key(|p| p.at) {
                Some(p) => p.x + p.layout.caret(p.len).0,
                None if r.rtl => self.room,
                None => 0.0,
            },
        };
        (x, r.top, r.height)
    }

    pub fn hit(&self, x: f32, y: f32) -> usize {
        let Some(r) = self.rows.iter().find(|r| y < r.top + r.height).or(self.rows.last()) else { return 0 };
        let here: Vec<&Piece> = self.pieces.iter().filter(|p| r.range.contains(&p.at)).collect();
        let near = here.iter().min_by(|a, b| {
            let d = |p: &Piece| if x < p.x { p.x - x } else { (x - p.x - p.layout.width).max(0.0) };
            d(a).total_cmp(&d(b))
        });
        match near {
            Some(p) => p.at + p.layout.hit(x - p.x, y - p.y),
            None => r.range.start,
        }
    }

    pub fn draw(&self, fonts: &Fonts, canvas: &mut PixmapMut, x: f32, y: f32, ink: &dyn Fn(usize) -> Colour) {
        let mut painter = Painter::default();
        self.draw_on(fonts, &mut painter, x, y, ink);
        painter.play(canvas);
    }

    pub fn draw_on<'a>(&self, fonts: &'a Fonts, painter: &mut Painter<'a>, x: f32, y: f32, ink: &dyn Fn(usize) -> Colour) {
        for p in &self.pieces {
            words_on(painter, fonts, p.layout.clone(), x + p.x, y + p.y, ink(p.span));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fonts() -> &'static Fonts {
        thread_local! { static F: &'static Fonts = Box::leak(Box::new(Fonts::new())); }
        F.with(|f| *f)
    }

    fn laid(spans: &[Span], width: Option<f32>) -> Flow {
        flow(&|t, s| fonts().layout(t, s, None), spans, &Style::new(16.0), width)
    }

    #[test]
    fn two_stars_with_no_partner_stay_as_letters() {
        let text: String = parse("2 ** 3").iter().map(|s| s.text.as_str()).collect();
        assert_eq!(text, "2 ** 3");
    }

    #[test]
    fn markup_is_read_into_looks_and_written_back() {
        let s = parse("Press **Save** or *undo*, see `cargo test` and [the guide](https://trindle.org/guide).");
        assert_eq!(s, vec![
            Span::plain("Press "), Span::plain("Save").bold(), Span::plain(" or "), Span::plain("undo").italic(), Span::plain(", see "),
            Span::plain("cargo test").code(), Span::plain(" and "), Span::link("the guide", "https://trindle.org/guide"), Span::plain("."),
        ]);
        assert_eq!(parse(&markup(&s)), s);
        assert_eq!(parse("snake_case_name and 2 * 3"), vec![Span::plain("snake_case_name and 2 * 3")], "lone marks are letters");
        assert_eq!(parse(r"\*kept\*"), vec![Span::plain("*kept*")]);
        assert_eq!(parse(&markup(&[Span::plain("a*b_[c]")])), vec![Span::plain("a*b_[c]")]);
    }

    #[test]
    fn code_with_a_backtick_and_a_link_with_a_bracket_are_written_so_they_read_back() {
        let spans = vec![
            Span::plain("run "), Span::plain("echo `date`").code(), Span::plain(" or "), Span::plain(r"C:\dir\").code(), Span::plain(" or "),
            Span::plain("a`").code().bold(), Span::plain(" then "), Span::link("the (old) page", "https://example.com/a_(b)"),
            Span::plain(" "), Span::link("odd ](x) [y]", r"https://example.com/\)"), Span::plain(" "), Span::link("`](`", "z").bold(),
        ];
        let written = markup(&spans);
        assert!(written.contains(r"`echo \`date\``") && written.contains(r"(https://example.com/a_(b\))"), "{written}");
        assert_eq!(parse(&written), spans);
        assert_eq!(parse(r"`a\`b` and `c\d`"), vec![Span::plain("a`b").code(), Span::plain(" and "), Span::plain(r"c\d").code()]);
        assert_eq!(parse("[`a](b`](u)"), vec![Span { code: true, ..Span::link("a](b", "u") }], "a ]( in code does not end the words");
        assert_eq!(parse(r"[x](a\)b) and `un\`closed"), vec![Span::link("x", "a)b"), Span::plain(" and `un`closed")]);
    }

    #[test]
    fn spans_flow_and_wrap_as_one_paragraph() {
        let spans = parse("one **two** three *four* five six seven");
        let one = laid(&spans, None);
        assert_eq!(one.rows.len(), 1);
        let bold = one.pieces.iter().find(|p| p.span == 1).unwrap();
        let before = one.pieces.iter().find(|p| p.span == 0).unwrap();
        assert!(bold.x >= before.x + before.layout.width - 0.5, "pieces follow on");
        let narrow = laid(&spans, Some(one.width / 2.0));
        assert!(narrow.rows.len() >= 2 && narrow.width <= one.width / 2.0 + 0.5);
        assert!(narrow.height > one.height);
        for r in &narrow.rows {
            let first = narrow.pieces.iter().filter(|p| r.range.contains(&p.at)).map(|p| p.x).fold(f32::MAX, f32::min);
            assert!(first.abs() < 0.5);
        }
        assert_eq!(laid(&parse("a\n\nb"), None).rows.len(), 3, "an empty line keeps its room");
        let ended = laid(&parse("a\n"), None);
        assert_eq!(ended.rows.len(), 2);
        assert!(ended.caret(2).1 > ended.caret(1).1, "after the last break the caret is on a new line");
    }

    #[test]
    fn a_word_wider_than_the_line_is_broken_between_letters() {
        let spans = parse("go abcdefghijklmnopqrstuvwxyzabcdefghijklmnop");
        let one = laid(&spans, None);
        let narrow = laid(&spans, Some(one.width / 4.0));
        assert!(narrow.rows.len() >= 4 && narrow.width <= one.width / 4.0 + 0.5, "{} rows, {} wide", narrow.rows.len(), narrow.width);
        let text: String = narrow.rows.iter().map(|r| &"go abcdefghijklmnopqrstuvwxyzabcdefghijklmnop"[r.range.clone()]).collect();
        assert_eq!(text.replace(' ', ""), "goabcdefghijklmnopqrstuvwxyzabcdefghijklmnop", "no letter is lost");
    }

    #[test]
    fn a_link_is_found_where_it_is_drawn() {
        let spans = parse("Read [the guide](g) now");
        let f = laid(&spans, None);
        let (x, y, w, h) = f.bands(1)[0];
        assert_eq!(f.span_at(x + w / 2.0, y + h / 2.0), Some(1));
        assert_eq!(f.span_at(1.0, y + h / 2.0), Some(0));
    }

    #[test]
    fn right_to_left_words_stand_right_to_left_and_lines_start_at_the_right() {
        let spans = vec![Span::plain("שלום "), Span::plain("עולם").bold()];
        let f = laid(&spans, Some(400.0));
        let (a, b) = (f.pieces.iter().find(|p| p.span == 0).unwrap(), f.pieces.iter().find(|p| p.span == 1).unwrap());
        assert!(a.x > b.x, "the first word read is on the right");
        assert!((a.x + a.layout.width - 400.0).abs() < 0.5, "the line ends at the right edge");
        assert!(f.caret(0).0 > f.caret(f.rows[0].range.end).0);
    }

    #[test]
    fn the_caret_and_a_click_agree() {
        let spans = parse("one **two** three");
        let f = laid(&spans, None);
        for at in [0, 4, 7, 9] {
            let (x, y, h) = f.caret(at);
            assert_eq!(f.hit(x + 0.1, y + h / 2.0), at, "byte {at}");
        }
    }
}
