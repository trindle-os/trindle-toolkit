//! A crash or fault written down as a plain report the person can copy, and the page that shows it.

use std::path::{Path, PathBuf};

use super::widget::{column, lead, main_action, page_title, scroll, label, Node};
use crate::t;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Report {
    pub program: String,
    pub version: String,
    pub what: String,
    pub place: String,
    pub when: String,
    pub system: String,
    pub trace: String,
}

impl Report {
    pub fn new(program: &str, version: &str, what: &str) -> Report {
        Report { program: program.into(), version: version.into(), what: what.into(), place: String::new(), when: now(), system: system(), trace: String::new() }
    }

    pub fn text(&self) -> String {
        let mut out = format!("{} {}\n{}\n{}\n{}\n", self.program, self.version, self.when, self.system, self.what);
        for extra in [&self.place, &self.trace] {
            if !extra.is_empty() {
                out.push_str(extra);
                out.push('\n');
            }
        }
        out
    }
}

pub fn install(folder: PathBuf, program: &'static str, version: &'static str) {
    let before = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let what = info.payload().downcast_ref::<&str>().map(|s| s.to_string())
            .or_else(|| info.payload().downcast_ref::<String>().cloned())
            .unwrap_or_default();
        let home = crate::common::files::home();
        let place = hide_home(&info.location().map(|l| format!("{}:{}", l.file(), l.line())).unwrap_or_default(), &home);
        let trace = hide_home(&std::backtrace::Backtrace::force_capture().to_string(), &home);
        let report = Report { place, trace, ..Report::new(program, version, &what) };
        let _ = std::fs::create_dir_all(&folder);
        let _ = trindle_common::files::write_private(&folder.join("crash.txt"), report.text().as_bytes());
        before(info);
    }));
}

fn hide_home(text: &str, home: &Path) -> String {
    match home.to_str() {
        Some(h) if h.len() > 1 => text.replace(&format!("{h}/"), "~/"),
        _ => text.to_string(),
    }
}

pub fn take(folder: &Path) -> Option<String> {
    let path = folder.join("crash.txt");
    let text = trindle_common::files::read_small(&path).ok()?;
    let _ = std::fs::remove_file(&path);
    (!text.is_empty()).then_some(text)
}

pub fn page<M>(program: &str, report: &str, copy: M) -> Node<M> {
    column(vec![
        page_title(t!("kit-crash-title")),
        lead(t!("kit-crash-lead", program = program.to_string())),
        scroll("crash-report", label(report.to_string())),
        main_action("edit-copy-symbolic", t!("kit-crash-copy")).on_press(copy),
    ]).gap(crate::theme::gaps().large).padding(crate::theme::gaps().huge)
}

fn now() -> String {
    let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs()) as i64;
    let (days, rest) = (secs.div_euclid(86_400), secs.rem_euclid(86_400));
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!("{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z", rest / 3600, rest / 60 % 60, rest % 60)
}

fn system() -> String {
    let release = std::fs::read_to_string("/proc/sys/kernel/osrelease").unwrap_or_default();
    let name = std::fs::read_to_string("/etc/os-release").unwrap_or_default().lines()
        .find_map(|l| l.strip_prefix("PRETTY_NAME=")).map(|n| n.trim_matches('"').to_string()).unwrap_or_default();
    format!("{name} Linux {}", release.trim()).trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_report_reads_as_plain_lines_with_no_empty_parts() {
        let r = Report { program: "Viewer".into(), version: "1.2".into(), what: "The file could not be read".into(), place: String::new(),
                         when: "2026-10-30T12:00:00Z".into(), system: "Linux 6.12".into(), trace: String::new() };
        assert_eq!(r.text(), "Viewer 1.2\n2026-10-30T12:00:00Z\nLinux 6.12\nThe file could not be read\n");
        let now = Report::new("V", "1", "x").when;
        assert!(now.len() == 20 && now.starts_with("20") && now.ends_with('Z'), "{now}");
    }

    #[test]
    fn a_report_does_not_carry_the_home_folder() {
        let trace = "at /home/john/.cargo/registry/x.rs:3\nat /home/john/work/app/src/main.rs:9";
        assert_eq!(hide_home(trace, Path::new("/home/john")), "at ~/.cargo/registry/x.rs:3\nat ~/work/app/src/main.rs:9");
        assert_eq!(hide_home("at /src/main.rs:9", Path::new("/")), "at /src/main.rs:9", "a home of / is no name to hide");
        assert_eq!(hide_home("at /home/johnny/x.rs", Path::new("/home/john")), "at /home/johnny/x.rs");
    }

    #[test]
    fn a_panic_leaves_a_report_that_is_taken_once() {
        let folder = std::env::temp_dir().join(format!("trindle-crash-{}", std::process::id()));
        install(folder.clone(), "Viewer", "1.2");
        let _ = std::panic::catch_unwind(|| panic!("the crash test's own panic"));
        let _ = std::panic::take_hook();
        let text = take(&folder).expect("a report");
        assert!(text.starts_with("Viewer 1.2\n") && text.contains("the crash test's own panic") && text.contains("crash.rs:"), "{text}");
        assert!(take(&folder).is_none(), "shown once");
        let _ = std::fs::remove_dir_all(&folder);
    }

    #[test]
    fn the_page_says_what_happened_and_offers_to_copy_the_report() {
        let n = page("Viewer", "Viewer 1.2\nboom", 7);
        let said = n.said();
        assert!(said.iter().any(|s| s.contains("Viewer")) && said.contains(&"Viewer 1.2\nboom".to_string()), "{said:?}");
        assert!(n.presses().iter().any(|(tip, _, m)| tip == "Copy report" && **m == 7));
    }
}
