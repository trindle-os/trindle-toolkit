//! What a link's button says of where it leads, read from the address itself; `read` is the one rule for every link in every program.

pub const ICON: &str = "web-browser-symbolic";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Link {
    pub label: String,
    pub spoken: String,
    pub web: bool,
}

const DOTS: char = '\u{2026}';

fn hidden(c: char) -> bool {
    c.is_control()
        || matches!(c,
            '\u{ad}' | '\u{34f}' | '\u{61c}' | '\u{115f}' | '\u{1160}' | '\u{17b4}' | '\u{17b5}' | '\u{180b}'..='\u{180f}'
            | '\u{200b}'..='\u{200f}' | '\u{2028}'..='\u{202f}' | '\u{205f}'..='\u{206f}' | '\u{3000}' | '\u{3164}'
            | '\u{fe00}'..='\u{fe0f}' | '\u{feff}' | '\u{ffa0}' | '\u{fff0}'..='\u{fffb}' | '\u{1d173}'..='\u{1d17a}'
            | '\u{e0000}'..='\u{e0fff}')
}

pub fn is_web(address: &str) -> bool {
    read(address, usize::MAX).web
}

pub fn read(address: &str, most: usize) -> Link {
    let not = |label: &str| Link { label: label.to_string(), spoken: label.to_string(), web: false };
    let whole: String = address.trim_matches(|c: char| c.is_whitespace() || c.is_control()).chars().filter(|c| !matches!(c, '\t' | '\n' | '\r')).collect();
    let Some((scheme, rest)) = whole.split_once(':') else { return not(crate::t!("kit-link-not-web")) };
    let named = scheme.chars().next().is_some_and(|c| c.is_ascii_alphabetic()) && scheme.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'));
    if !named {
        return not(crate::t!("kit-link-not-web"));
    }
    let usual = match scheme.to_ascii_lowercase().as_str() {
        "http" => "80",
        "https" => "443",
        "mailto" => return not(crate::t!("kit-link-mailto")),
        "file" => return not(crate::t!("kit-link-file")),
        _ => return not(crate::t!("kit-link-not-web")),
    };
    let rest = rest.replace('\\', "/");
    let Some(rest) = rest.strip_prefix("//") else { return not(crate::t!("kit-link-not-web")) };
    let rest = rest.trim_start_matches('/');
    let end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let (first, after) = rest.split_at(end);
    let host = first.rsplit('@').next().unwrap_or(first);
    let Some(host) = host_shown(host, usual) else { return not(crate::t!("kit-link-not-web")) };
    let path = after.split(['?', '#']).next().unwrap_or("");
    let path: String = path.chars().filter(|c| !hidden(*c) && !c.is_whitespace()).collect();
    let path = path.trim_end_matches('/');
    let label = fitted(&host, path, most);
    Link { spoken: crate::t!("kit-link-to", label = &label), label, web: true }
}

fn host_shown(host: &str, usual: &str) -> Option<String> {
    let (name, port) = match host.strip_prefix('[') {
        Some(inside) => {
            let (numbers, after) = inside.split_once(']')?;
            let ok = !numbers.is_empty() && numbers.chars().all(|c| c.is_ascii_hexdigit() || matches!(c, ':' | '.'));
            (ok.then(|| format!("[{}]", numbers.to_ascii_lowercase()))?, after.strip_prefix(':').unwrap_or(after))
        }
        None => {
            let (name, port) = host.rsplit_once(':').unwrap_or((host, ""));
            (name_shown(name)?, port)
        }
    };
    if !port.chars().all(|c| c.is_ascii_digit()) || port.len() > 5 || port.parse::<u32>().is_ok_and(|p| p > 65535) {
        return None;
    }
    let port = port.trim_start_matches('0');
    Some(if port.is_empty() || port == usual { name } else { format!("{name}:{port}") })
}

fn name_shown(name: &str) -> Option<String> {
    let name: String = percent_read(name)?.chars().map(|c| if ('\u{ff01}'..='\u{ff5e}').contains(&c) { char::from_u32(c as u32 - 0xfee0).unwrap_or(c) } else { c }).collect();
    let name = name.to_lowercase();
    let name = name.replace(['\u{3002}', '\u{ff61}'], ".");
    let name = name.strip_prefix("www.").filter(|rest| rest.contains('.')).unwrap_or(&name);
    if name.is_empty() || name.split('.').filter(|part| !part.is_empty()).count() == 0 {
        return None;
    }
    let mut out = Vec::new();
    for part in name.split('.') {
        if part.chars().any(|c| c.is_whitespace() || c.is_control() || (c.is_ascii() && !(c.is_ascii_alphanumeric() || matches!(c, '-' | '_')))) {
            return None;
        }
        if part.is_ascii() {
            out.push(part.to_string());
        } else {
            out.push(format!("xn--{}", punycode(part)?));
        }
    }
    let name = out.join(".");
    match ipv4(&name) {
        Some(address) => address,
        None => Some(name),
    }
}

fn ipv4(host: &str) -> Option<Option<String>> {
    let mut parts: Vec<&str> = host.split('.').collect();
    if parts.len() > 1 && parts.last() == Some(&"") {
        parts.pop();
    }
    let hex = |p: &str| p.get(..2).is_some_and(|x| x.eq_ignore_ascii_case("0x"));
    let last = parts.last()?;
    if !(!last.is_empty() && last.bytes().all(|b| b.is_ascii_digit()) || hex(last) && last[2..].bytes().all(|b| b.is_ascii_hexdigit())) {
        return None;
    }
    let number = |p: &str| -> Option<u64> {
        let (digits, radix) = if hex(p) { (&p[2..], 16) } else if p.len() > 1 && p.starts_with('0') { (&p[1..], 8) } else { (p, 10) };
        if digits.is_empty() { Some(0) } else { u64::from_str_radix(digits, radix).ok() }
    };
    let numbers: Option<Vec<u64>> = parts.iter().map(|p| number(p)).collect();
    Some(numbers.filter(|n| n.len() <= 4).and_then(|n| {
        let (head, tail) = n.split_at(n.len() - 1);
        if head.iter().any(|&h| h > 255) || tail[0] >= 256u64.pow(5 - n.len() as u32) {
            return None;
        }
        let whole = head.iter().enumerate().fold(0u64, |acc, (i, &h)| acc + (h << (8 * (3 - i)))) + tail[0];
        Some(format!("{}.{}.{}.{}", whole >> 24, (whole >> 16) & 255, (whole >> 8) & 255, whole & 255))
    }))
}

fn percent_read(text: &str) -> Option<String> {
    let bytes = text.as_bytes();
    let (mut out, mut i) = (Vec::with_capacity(bytes.len()), 0);
    while i < bytes.len() {
        let hex = |b: u8| (b as char).to_digit(16);
        match (bytes[i], bytes.get(i + 1).copied().and_then(hex), bytes.get(i + 2).copied().and_then(hex)) {
            (b'%', Some(a), Some(b)) => {
                out.push((a * 16 + b) as u8);
                i += 3;
            }
            (b, _, _) => {
                out.push(b);
                i += 1;
            }
        }
    }
    String::from_utf8(out).ok()
}

fn punycode(part: &str) -> Option<String> {
    const BASE: u32 = 36;
    const T_MIN: u32 = 1;
    const T_MAX: u32 = 26;
    const SKEW: u32 = 38;
    const DAMP: u32 = 700;
    let letters: Vec<u32> = part.chars().map(|c| c as u32).collect();
    if letters.len() > 63 {
        return None;
    }
    let digit = |d: u32| (if d < 26 { b'a' + d as u8 } else { b'0' + (d - 26) as u8 }) as char;
    let mut out: String = part.chars().filter(char::is_ascii).collect();
    let basic = out.len() as u32;
    if basic > 0 {
        out.push('-');
    }
    let (mut n, mut delta, mut bias, mut done) = (128u32, 0u32, 72u32, basic);
    while (done as usize) < letters.len() {
        let next = letters.iter().copied().filter(|c| *c >= n).min()?;
        delta = delta.checked_add((next - n).checked_mul(done + 1)?)?;
        n = next;
        for c in &letters {
            if *c < n {
                delta = delta.checked_add(1)?;
            }
            if *c == n {
                let (mut q, mut k) = (delta, BASE);
                loop {
                    let t = if k <= bias { T_MIN } else if k >= bias + T_MAX { T_MAX } else { k - bias };
                    if q < t {
                        break;
                    }
                    out.push(digit(t + (q - t) % (BASE - t)));
                    q = (q - t) / (BASE - t);
                    k += BASE;
                }
                out.push(digit(q));
                let mut d = if done == basic { delta / DAMP } else { delta / 2 };
                d += d / (done + 1);
                let mut k = 0;
                while d > ((BASE - T_MIN) * T_MAX) / 2 {
                    d /= BASE - T_MIN;
                    k += BASE;
                }
                bias = k + (BASE - T_MIN + 1) * d / (d + SKEW);
                delta = 0;
                done += 1;
            }
        }
        delta = delta.checked_add(1)?;
        n = n.checked_add(1)?;
    }
    Some(out)
}

fn fitted(host: &str, path: &str, most: usize) -> String {
    let count = |s: &str| s.chars().count();
    if path.is_empty() || count(host) + count(path) <= most {
        return format!("{host}{path}");
    }
    let mut shown = host.to_string();
    for step in path.split('/').filter(|s| !s.is_empty()) {
        if count(&shown) + 1 + count(step) + 2 > most {
            if shown == host {
                let room = most.saturating_sub(count(host) + 2);
                let start: String = step.chars().take(room).collect();
                return format!("{host}/{start}{DOTS}");
            }
            break;
        }
        shown.push('/');
        shown.push_str(step);
    }
    format!("{shown}/{DOTS}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn label(address: &str) -> String {
        read(address, 48).label
    }

    #[test]
    fn a_host_written_as_a_number_is_shown_as_the_address_a_browser_goes_to() {
        for (typed, shown) in [("0x7f.1", "127.0.0.1"), ("2130706433", "127.0.0.1"), ("0177.0.0.1", "127.0.0.1"), ("10.1.2.3", "10.1.2.3"), ("192.168.1", "192.168.0.1"), ("1.2.3.4.", "1.2.3.4")] {
            assert_eq!(label(&format!("http://{typed}/x")), format!("{shown}/x"), "{typed}");
        }
        for bad in ["1.2.3.4.5", "256.1.1.1", "999999999999", "1.2.3.0x1ff"] {
            assert!(!read(&format!("http://{bad}/"), 48).web, "{bad}");
        }
        assert_eq!(label("http://a1.example.com/"), "a1.example.com");
    }

    #[test]
    fn a_link_says_the_host_and_as_much_of_the_path_as_fits() {
        assert_eq!(label("https://www.example.com/"), "example.com");
        assert_eq!(label("https://example.com"), "example.com");
        assert_eq!(label("HTTP://WWW.Example.COM/Docs/Start"), "example.com/Docs/Start");
        assert_eq!(label("https://example.com/docs/start?utm_source=x&id=7#top"), "example.com/docs/start");
        assert_eq!(label("https://example.com:8443/admin"), "example.com:8443/admin");
        assert_eq!(label("https://example.com:443/admin"), "example.com/admin");
        assert_eq!(label("http://example.com:443/admin"), "example.com:443/admin");
        assert_eq!(label("https://example.com:80/admin"), "example.com:80/admin");
        assert_eq!(label("https://[2001:DB8::1]:8080/x"), "[2001:db8::1]:8080/x");
        assert_eq!(label("https://www.com/"), "www.com");
        let long = label("https://docs.example.com/guides/getting-started/installing/on-a-computer-with-no-network/step-three.html");
        assert_eq!(long, "docs.example.com/guides/getting-started/\u{2026}");
        assert!(long.chars().count() <= 48);
        let one = label("https://example.com/a-single-step-of-the-path-that-is-far-too-long-to-fit-in-any-label-at-all");
        assert_eq!(one.chars().count(), 48);
        assert!(one.starts_with("example.com/a-single-step") && one.ends_with('\u{2026}'));
        let host = format!("{}.example.com", "a".repeat(60));
        assert_eq!(label(&format!("https://{host}/path/to/a/page")), format!("{host}/\u{2026}"));
        assert_eq!(read("https://example.com/docs", 48).spoken, "Link to example.com/docs");
        assert!(is_web("https://example.com/docs"));
    }

    #[test]
    fn a_hostile_address_cannot_pass_as_another_site() {
        assert_eq!(label("https://example.com@evil.example/login"), "evil.example/login");
        assert_eq!(label("https://example.com:secret@evil.example/"), "evil.example");
        assert_eq!(label("https://example.com%2F@evil.example/"), "evil.example");
        assert_eq!(label("https://evil.example\\@example.com/"), "evil.example/@example.com");
        assert_eq!(label("https:\\\\evil.example/example.com"), "evil.example/example.com");
        assert_eq!(label("https://evil.example/?https://example.com/"), "evil.example");
        assert_eq!(label("https://evil.example#@example.com/"), "evil.example");
        assert_eq!(label("https://example.com%2eevil.example/"), "example.com.evil.example");
        let alike = label("https://ex\u{430}mple.com/");
        assert!(alike.starts_with("xn--exmple-") && alike.ends_with(".com") && alike.is_ascii(), "{alike:?}");
        assert!(label("https://\u{435}xample.com/").starts_with("xn--xample-"));
        assert_eq!(label("https://b\u{fc}cher.example/"), "xn--bcher-kva.example");
        for bad in ["https://b\u{fc}%20cher.example/", "https://b\u{fc}%0Acher.example/", "https://b\u{fc}%2Fcher.example/"] {
            assert!(!read(bad, 48).web, "{bad}");
        }
        assert_eq!(label("https://m\u{fc}nchen.example/"), "xn--mnchen-3ya.example");
        assert_eq!(label("https://example\u{3002}com\u{ff0e}evil\u{ff61}example/"), "example.com.evil.example");
        assert_eq!(label("https://\u{ff25}\u{ff58}\u{ff41}mple\u{ff0e}com/\u{ff41}"), "example.com/\u{ff41}");
        for bad in ["https://example.com\u{ff20}evil.example/", "https://evil.example\u{ff0f}example.com/", "https://exa\u{ff05}mple.com/"] {
            assert!(!read(bad, 48).web, "{bad}");
        }
        for unseen in ['\u{200b}', '\u{200d}', '\u{202e}', '\u{2066}', '\u{feff}', '\u{ad}'] {
            let shown = label(&format!("https://exam{unseen}ple.com/"));
            assert!(shown != "example.com" && !shown.contains(unseen), "{shown:?}");
        }
        let turned = label("https://evil.example/\u{202e}moc.elpmaxe//:sptth");
        assert!(turned.starts_with("evil.example/") && turned.chars().all(|c| !hidden(c)), "{turned:?}");
        assert_eq!(label("https://evil.example/a\u{0}b\u{7}c\u{1b}[31m"), "evil.example/abc[31m");
        assert_eq!(label(" \n https://evil\t.exa\nmple/pa\rth "), "evil.example/path");
        for bad in ["https://exa mple.com/", "https://example.com<script>/", "https://", "https:///", "https://@/", "https://:80/",
                    "https://example.com:port/", "https://example.com:123456/", "https://[::1/", "https://exa%00mple.com/"] {
            let read = read(bad, 48);
            assert!(!read.web && read.label == "Not a web address", "{bad:?} read as {read:?}");
        }
        let huge = format!("https://evil.example/{}?{}", "a/".repeat(5000), "b".repeat(5000));
        assert!(label(&huge).chars().count() <= 48);
    }

    #[test]
    fn a_port_above_65535_is_not_web() {
        assert!(!is_web("http://example.com:99999"));
        assert!(is_web("http://example.com:8080"));
    }

    #[test]
    fn what_is_not_a_web_address_says_what_it_is() {
        for (address, says) in [("mailto:john.smith@example.com", "Email address"), ("MAILTO:john.smith@example.com", "Email address"),
                                ("file:///etc/passwd", "File on this computer"), ("javascript:alert(1)", "Not a web address"),
                                ("data:text/html,<b>hello</b>", "Not a web address"), ("ftp://example.com/", "Not a web address"),
                                ("example.com/page", "Not a web address"), ("/usr/bin/true", "Not a web address"), ("", "Not a web address"),
                                ("ht tp://example.com", "Not a web address"), ("https:example.com", "Not a web address")] {
            let read = read(address, 48);
            assert_eq!((read.label.as_str(), read.spoken.as_str(), read.web), (says, says, false), "{address:?}");
            assert!(!is_web(address));
        }
    }

    #[test]
    fn a_links_button_is_labelled_with_where_it_leads() {
        use crate::theme::Theme;
        use crate::ui::Ui;
        use crate::widget::{column, link, link_named, Node};
        let page: Node<()> = column(vec![
            link("https://www.example.com/docs/start?from=catalogue#top").on_press(()).key("plain"),
            link_named("Make a key", "https://user@example.com/settings/keys").on_press(()).key("named"),
            link("javascript:alert(1)").on_press(()).key("not"),
        ]);
        assert_eq!(page.said(), ["button: example.com/docs/start", "button: Make a key", "button: Not a web address"]);
        let mut ui = Ui::new(page, Theme::new(false, ""));
        assert_eq!(ui.label_at("plain").as_deref(), Some("example.com/docs/start"));
        assert_eq!(ui.label_at("named").as_deref(), Some("Make a key\nexample.com/settings/keys"));
        assert_eq!(ui.label_at("not").as_deref(), Some("Not a web address"));
    }

    #[test]
    fn punycode_is_as_the_standard_has_it() {
        assert_eq!(punycode("b\u{fc}cher").as_deref(), Some("bcher-kva"));
        assert_eq!(punycode("\u{4f8b}\u{3048}").as_deref(), Some("r8jz45g"));
        assert_eq!(punycode("\u{43f}\u{440}\u{438}\u{43c}\u{435}\u{440}").as_deref(), Some("e1afmkfd"));
        assert_eq!(punycode(&"\u{fc}".repeat(64)), None);
    }
}
