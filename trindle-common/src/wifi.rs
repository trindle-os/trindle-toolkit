//! Wi-Fi through NetworkManager's nmcli: whether it is on, the networks near by, and joining one.

use std::collections::HashSet;
use std::io::Write;
use std::process::{Command, Stdio};

/// Wi-Fi on or off, how many Wi-Fi adapters there are, the saved networks
/// by the name each broadcasts (not by the connection's own name, which can
/// differ), then those near by.
pub const LOOK: &str = "nmcli radio wifi; echo '#adapters'; nmcli -t -f TYPE dev | grep -cx wifi; echo '#saved'; \
    nmcli -t -f UUID,TYPE con show | sed -n 's/:802-11-wireless$//p' | while IFS= read -r u; do \
    nmcli -g 802-11-wireless.ssid con show uuid \"$u\"; done; echo '#near'; \
    nmcli -t -f IN-USE,SIGNAL,SECURITY,SSID dev wifi list --rescan no";

/// The longest name a network can broadcast, in bytes.
pub const SSID_MOST: usize = 32;

/// At most this many networks, and saved names, are kept.
const NEAR_MOST: usize = 64;
const SAVED_MOST: usize = 256;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Net {
    pub ssid: String,
    pub signal: u32,
    pub inuse: bool,
    pub secured: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Seen {
    pub on: bool,
    pub adapters: u32,
    pub saved: HashSet<String>,
    /// The one in use first, then strongest first.
    pub near: Vec<Net>,
}

impl Seen {
    /// Whether joining this network needs its password typed.
    pub fn asks(&self, net: &Net) -> bool {
        net.secured && !self.saved.contains(&net.ssid)
    }
}

/// How joining went.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Joined {
    Yes,
    WrongPassword,
    /// Why, as nmcli said it.
    Failed(String),
}

/// The icon name's part for a signal of 0 to 100.
pub fn signal_level(signal: u32) -> &'static str {
    match signal {
        76.. => "excellent",
        51..=75 => "good",
        26..=50 => "ok",
        _ => "weak",
    }
}

/// nmcli's terse form puts a backslash before a colon or a backslash in a value.
fn unescape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        out.push(if c == '\\' { chars.next().unwrap_or('\\') } else { c });
    }
    out
}

/// A name that is safe to show and to hand to nmcli: not empty, no longer
/// than a network may broadcast, no control or direction-changing marks
/// that could disguise it, and not one nmcli would read as an option.
pub fn fit_name(ssid: &str) -> bool {
    !ssid.is_empty() && ssid.len() <= SSID_MOST && !ssid.starts_with('-')
        && !ssid.chars().any(|c| c.is_control() || matches!(c, '\u{200e}' | '\u{200f}' | '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}'))
}

/// What `LOOK` printed.
pub fn parse(out: &str) -> Seen {
    let mut seen = Seen { on: out.lines().next() == Some("enabled"), ..Seen::default() };
    let mut section = "";
    for l in out.lines().skip(1) {
        if let Some(s) = l.strip_prefix('#') {
            section = match s { "adapters" | "saved" => s, _ => "near" };
            continue;
        }
        match section {
            "adapters" => seen.adapters = l.trim().parse().unwrap_or(0),
            "saved" if seen.saved.len() < SAVED_MOST => {
                seen.saved.insert(unescape(l));
            }
            "near" => near_line(&mut seen.near, l),
            _ => {}
        }
    }
    seen.near.sort_by_key(|n| (!n.inuse, std::cmp::Reverse(n.signal)));
    seen
}

/// One line of `nmcli dev wifi list`; a network heard on more than one
/// band is one entry, at its strongest.
fn near_line(near: &mut Vec<Net>, l: &str) {
    // The name comes last so splitn keeps its colons.
    let mut parts = l.splitn(4, ':');
    let (Some(inuse), Some(sig), Some(sec), Some(ssid)) = (parts.next(), parts.next(), parts.next(), parts.next()) else { return };
    let ssid = unescape(ssid);
    if !fit_name(&ssid) {
        return;
    }
    let net = Net { inuse: inuse == "*", signal: sig.parse::<u32>().unwrap_or(0).min(100), secured: !sec.is_empty() && sec != "--", ssid };
    if let Some(n) = near.iter_mut().find(|n| n.ssid == net.ssid) {
        n.inuse |= net.inuse;
        n.signal = n.signal.max(net.signal);
    } else if near.len() < NEAR_MOST {
        near.push(net);
    }
}

/// Reads Wi-Fi as it is now; waits on nmcli, so not on a window's thread.
pub fn look() -> Seen {
    parse(&run(&["sh", "-c", LOOK], None).1)
}

/// Asks the adapters to look for networks again.
pub fn rescan() {
    run(&["nmcli", "dev", "wifi", "rescan"], None);
}

/// Switches Wi-Fi on or off; true if it went through.
pub fn power(on: bool) -> bool {
    run(&["nmcli", "radio", "wifi", if on { "on" } else { "off" }], None).0
}

/// Seconds nmcli may wait for a join (its own wait is 90), so a wrong password is told soon.
const JOIN_WAIT: &str = "30";

/// Joins `ssid`. A password goes to nmcli on its input, never on its
/// command line, and is not copied on the way.
pub fn join(ssid: &str, password: Option<&str>) -> Joined {
    if !fit_name(ssid) {
        return Joined::Failed(String::new());
    }
    // nmcli reads the password up to the first line break, so one with a
    // control character would be cut short; no Wi-Fi password holds one.
    if password.is_some_and(|pw| pw.chars().any(char::is_control)) {
        return Joined::WrongPassword;
    }
    let (ok, _, err) = match password {
        Some(pw) => run(&["nmcli", "--wait", JOIN_WAIT, "--ask", "dev", "wifi", "connect", ssid], Some(pw)),
        None => run(&["nmcli", "--wait", JOIN_WAIT, "dev", "wifi", "connect", ssid], None),
    };
    joined(ok, &err, password.is_some())
}

/// Reasons nmcli gives, as "(7)", when the adapter could not get secrets or the key exchange with the router failed.
const KEY_REASONS: [&str; 5] = ["(7)", "(8)", "(9)", "(10)", "(11)"];

/// What nmcli's answer means. `asked` is whether a password was given, for a secured network: then running out of time, or
/// a key exchange that fails, is read as a wrong password, which is what it nearly always is.
pub fn joined(ok: bool, err: &str, asked: bool) -> Joined {
    let said = |w: &str| err.to_ascii_lowercase().contains(&w.to_ascii_lowercase());
    let key_trouble = said("Secrets were required") || said("property is invalid");
    let after_asking = said("Timeout expired") || said("supplicant") || KEY_REASONS.iter().any(|r| err.contains(&format!("failed: {r}")));
    match ok {
        true => Joined::Yes,
        false if key_trouble || (asked && after_asking) => Joined::WrongPassword,
        false => Joined::Failed(err.trim().to_string()),
    }
}

/// Runs it to the end in the C locale: whether it succeeded, what it printed and what it complained.
fn run(argv: &[&str], secret: Option<&str>) -> (bool, String, String) {
    let Some((program, args)) = argv.split_first() else { return (false, String::new(), String::new()) };
    let child = Command::new(program).args(args).env("LC_ALL", "C")
        .stdin(if secret.is_some() { Stdio::piped() } else { Stdio::null() })
        .stdout(Stdio::piped()).stderr(Stdio::piped()).spawn();
    let mut child = match child {
        Ok(c) => c,
        Err(e) => return (false, String::new(), e.to_string()),
    };
    let pipe = child.stdin.take();
    std::thread::scope(|s| {
        if let (Some(secret), Some(mut pipe)) = (secret, pipe) {
            // Written alongside, so a program that answers before it has read
            // everything cannot leave both sides waiting; written straight
            // from the caller's copy, then the line's end, so no copy is left.
            s.spawn(move || {
                let _ = pipe.write_all(secret.as_bytes()).and_then(|_| pipe.write_all(b"\n"));
            });
        }
        match child.wait_with_output() {
            Ok(o) => (o.status.success(), String::from_utf8_lossy(&o.stdout).into_owned(), String::from_utf8_lossy(&o.stderr).into_owned()),
            Err(e) => (false, String::new(), e.to_string()),
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const OUT: &str = "enabled\n#adapters\n1\n#saved\nHome\\:Net\n#near\n\
        :40:WPA2:Example cafe\n*:62:WPA2:Home\\:Net\n:81:WPA2:John Smith's Wi-Fi\n:90::Open\\\\Net\n:20:WPA2:Example cafe\n:70:--:\n";

    #[test]
    fn the_list_is_read_in_use_first_then_strongest_first() {
        let seen = parse(OUT);
        assert!(seen.on);
        assert_eq!(seen.adapters, 1);
        assert!(seen.saved.contains("Home:Net"));
        let names: Vec<&str> = seen.near.iter().map(|n| n.ssid.as_str()).collect();
        assert_eq!(names, ["Home:Net", "Open\\Net", "John Smith's Wi-Fi", "Example cafe"]);
        assert!(seen.near[0].inuse && seen.near[0].secured);
        assert!(!seen.near[1].secured, "no security is an open network");
        assert_eq!(seen.near[3].signal, 40, "a network heard twice keeps its strongest");
        assert!(!seen.asks(&seen.near[0]), "a saved network joins without its password");
        assert!(seen.asks(&seen.near[2]));
    }

    #[test]
    fn hostile_names_are_left_out() {
        let long = "x".repeat(SSID_MOST + 1);
        let out = format!("enabled\n#near\n:50:WPA2:{long}\n:50:WPA2:Bell\x07\n:50:WPA2:abc\u{202e}gpj\n:50:WPA2:--ask\n:50:WPA2:{}\n:999:WPA2:Loud\n",
                          "y".repeat(SSID_MOST));
        let seen = parse(&out);
        let names: Vec<&str> = seen.near.iter().map(|n| n.ssid.as_str()).collect();
        assert_eq!(names, ["Loud".to_string(), "y".repeat(SSID_MOST)]);
        assert_eq!(seen.near[0].signal, 100);
        assert_eq!(join("--ask", Some("pw")), Joined::Failed(String::new()), "never handed to nmcli");
        let many = (0..200).map(|i| format!(":50:WPA2:net{i}\n")).collect::<Vec<_>>().concat();
        assert_eq!(parse(&format!("enabled\n#near\n{many}")).near.len(), NEAR_MOST);
    }

    #[test]
    fn no_adapter_and_wifi_off_are_told_apart() {
        let none = parse("enabled\n#adapters\n0\n#saved\n#near\n");
        assert!(none.on && none.adapters == 0 && none.near.is_empty());
        let off = parse("disabled\n#adapters\n1\n#saved\n#near\n");
        assert!(!off.on && off.adapters == 1);
        assert_eq!(parse(""), Seen::default(), "no nmcli at all");
    }

    #[test]
    fn nmcli_answers_are_told_apart() {
        assert_eq!(joined(true, "", true), Joined::Yes);
        for asked in [true, false] {
            assert_eq!(joined(false, "Error: Connection activation failed: Secrets were required, but not provided.\n", asked), Joined::WrongPassword);
            assert_eq!(joined(false, "Error: 802-11-wireless-security.psk: property is invalid.\n", asked), Joined::WrongPassword);
            assert_eq!(joined(false, " Error: No network with SSID 'x' found.\n", asked), Joined::Failed("Error: No network with SSID 'x' found.".into()));
        }
    }

    #[test]
    fn running_out_of_time_or_a_failed_key_exchange_after_a_password_is_a_wrong_password() {
        for said in ["Error: Timeout expired (30 seconds)\n",
                     "Error: Connection activation failed: (7) Secrets were required, but not provided.\n",
                     "Error: Connection activation failed: (8) 802.1X supplicant disconnected\n",
                     "Error: Connection activation failed: (9) 802.1X supplicant configuration failed\n",
                     "Error: Connection activation failed: (10) 802.1X supplicant failed\n",
                     "Error: Connection activation failed: (11) 802.1X supplicant took too long to authenticate\n"] {
            assert_eq!(joined(false, said, true), Joined::WrongPassword, "{said}");
        }
        assert_eq!(joined(false, "Error: Timeout expired (30 seconds)\n", false), Joined::Failed("Error: Timeout expired (30 seconds)".into()), "an open network that times out is not a password");
        assert!(matches!(joined(false, "Error: Connection activation failed: (53) The Wi-Fi network could not be found\n", true), Joined::Failed(_)));
        assert!(matches!(joined(false, "Error: Connection activation failed: (70) something\n", true), Joined::Failed(_)), "(70) is not (7)");
    }

    #[test]
    fn a_join_is_given_a_time_limit_under_nmclis_own() {
        assert!(JOIN_WAIT.parse::<u32>().is_ok_and(|n| (5..90).contains(&n)));
    }

    #[test]
    fn a_secret_goes_in_on_the_input_as_one_line() {
        let (ok, out, _) = run(&["sh", "-c", "read -r l; printf '[%s]' \"$l\"; echo \"$#\""], Some("p a:ss"));
        assert!(ok);
        assert_eq!(out, "[p a:ss]0\n");
        assert!(!run(&["/nonexistent/program"], None).0);
    }

    #[test]
    fn a_password_with_a_line_break_is_refused_before_nmcli_runs() {
        for pw in ["first\nsecond", "tab\there", "end\r"] {
            assert_eq!(join("Example cafe", Some(pw)), Joined::WrongPassword, "{pw:?}");
        }
    }
}
