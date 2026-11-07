//! Open with: the installed apps that take a kind of file, the person's default first, read from the desktop's own files; and the list of files recently used.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use trindle_common::ini::Ini;

use super::common::file_work::reach;
use super::common::files::{config_home, data_home, own_place};
use super::widget::{item, list, Node};

const ENTRY: &str = "Desktop Entry";

#[derive(Clone, Debug)]
pub struct Places {
    pub config: PathBuf,
    pub data: Vec<PathBuf>,
}

impl Places {
    pub fn of_this_desktop() -> Places {
        let sys = std::env::var("XDG_DATA_DIRS").ok().filter(|d| !d.is_empty()).unwrap_or_else(|| "/usr/local/share:/usr/share".into());
        let mut data = vec![data_home()];
        data.extend(sys.split(':').filter(|d| d.starts_with('/')).map(PathBuf::from));
        Places { config: config_home(), data }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct App {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub exec: String,
    pub mimes: Vec<String>,
}

impl App {
    pub fn command(&self, files: &[&Path]) -> Vec<String> {
        let mut out = Vec::new();
        let mut used = false;
        let name = |f: &Path| f.to_string_lossy().into_owned();
        for word in exec_words(&self.exec) {
            match word.as_str() {
                "%f" | "%u" => {
                    out.extend(files.first().map(|f| name(f)));
                    used = true;
                }
                "%F" | "%U" => {
                    out.extend(files.iter().map(|f| name(f)));
                    used = true;
                }
                _ => {
                    let mut w = String::new();
                    let mut chars = word.chars();
                    while let Some(c) = chars.next() {
                        if c != '%' {
                            w.push(c);
                            continue;
                        }
                        match chars.next() {
                            Some('f' | 'u' | 'F' | 'U') => {
                                w.extend(files.first().map(|f| name(f)));
                                used = true;
                            }
                            Some('%') => w.push('%'),
                            Some('i' | 'c' | 'k') | None => {}
                            Some(o) => {
                                w.push('%');
                                w.push(o);
                            }
                        }
                    }
                    if !w.is_empty() || word.contains('"') {
                        out.push(w);
                    }
                }
            }
        }
        if !used {
            out.extend(files.iter().map(|f| f.to_string_lossy().into_owned()));
        }
        out
    }
}

fn exec_words(exec: &str) -> Vec<String> {
    let (mut out, mut cur, mut quoted, mut open) = (Vec::new(), String::new(), false, false);
    let mut chars = exec.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '"' => {
                quoted = !quoted;
                open = true;
            }
            '\\' if quoted && chars.peek().is_some_and(|n| "`$\"\\".contains(*n)) => cur.extend(chars.next()),
            c if c.is_whitespace() && !quoted => {
                if open || !cur.is_empty() {
                    out.push(std::mem::take(&mut cur));
                    open = false;
                }
            }
            c => cur.push(c),
        }
    }
    if open || !cur.is_empty() {
        out.push(cur);
    }
    out
}

pub fn apps(places: &Places) -> Vec<App> {
    let (mut out, mut seen) = (Vec::new(), HashSet::new());
    for root in &places.data {
        let root = root.join("applications");
        let mut files = Vec::new();
        gather(&root, &mut files, 0);
        files.sort();
        for path in files {
            let Some(id) = path.strip_prefix(&root).ok().map(|p| p.to_string_lossy().replace('/', "-")) else { continue };
            if !seen.insert(id.clone()) {
                continue;
            }
            let Ok(ini) = Ini::read(&path) else { continue };
            let shown = ini.get(ENTRY, "Type") == Some("Application")
                && ini.get_bool(ENTRY, "NoDisplay") != Some(true)
                && ini.get_bool(ENTRY, "Hidden") != Some(true);
            if let (true, Some(name), Some(exec)) = (shown, ini.get(ENTRY, "Name"), ini.get(ENTRY, "Exec")) {
                out.push(App {
                    icon: ini.get(ENTRY, "Icon").map_or_else(|| super::generic::name_with_who(&id, name), str::to_string),
                    id,
                    name: name.to_string(),
                    exec: exec.to_string(),
                    mimes: ini.get_list(ENTRY, "MimeType").into_iter().map(str::to_string).collect(),
                });
            }
        }
    }
    out
}

fn gather(folder: &Path, out: &mut Vec<PathBuf>, depth: u32) {
    let Ok(entries) = std::fs::read_dir(folder) else { return };
    for e in entries.flatten() {
        let path = e.path();
        if e.file_type().is_ok_and(|t| t.is_dir()) && depth < 4 {
            gather(&path, out, depth + 1);
        } else if path.extension().is_some_and(|x| x == "desktop") {
            out.push(path);
        }
    }
}

pub fn apps_for(places: &Places, mime: &str) -> Vec<App> {
    apps_for_among(&apps(places), places, mime)
}

pub fn apps_for_among(all: &[App], places: &Places, mime: &str) -> Vec<App> {
    let lists: Vec<Ini> = std::iter::once(places.config.join("mimeapps.list"))
        .chain(places.data.iter().map(|d| d.join("applications").join("mimeapps.list")))
        .filter_map(|p| Ini::read(&p).ok())
        .collect();
    let removed: HashSet<&str> = lists.iter().flat_map(|l| l.get_list("Removed Associations", mime)).collect();
    let default = lists.iter().flat_map(|l| l.get_list("Default Applications", mime)).find(|id| all.iter().any(|a| a.id == *id));
    let added: Vec<&str> = lists.iter().flat_map(|l| l.get_list("Added Associations", mime)).collect();
    let mut out: Vec<App> = all.iter()
        .filter(|a| (a.mimes.iter().any(|m| m == mime) || added.contains(&a.id.as_str())) && !removed.contains(a.id.as_str()) && Some(a.id.as_str()) != default)
        .cloned().collect();
    out.sort_by_key(|a| a.name.to_lowercase());
    if let Some(d) = default.and_then(|id| all.iter().find(|a| a.id == id)) {
        out.insert(0, d.clone());
    }
    out
}

pub fn set_default(places: &Places, mime: &str, app: &App) -> std::io::Result<()> {
    let path = places.config.join("mimeapps.list");
    let mut ini = Ini::read(&path)?;
    if !ini.set("Default Applications", mime, &format!("{};", app.id)) {
        return Err(std::io::Error::from(std::io::ErrorKind::InvalidInput));
    }
    ini.write(&path)
}

pub fn chooser<M>(apps: &[App], chosen: Option<usize>, choose: impl Fn(usize) -> M + 'static) -> Node<M> {
    list(apps.iter().map(|a| item(a.name.clone()).icon(a.icon.clone())).collect(), chosen, choose).called(crate::t!("kit-apps"))
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Recent {
    pub path: PathBuf,
    pub mime: String,
    pub modified: String,
}

pub fn recent(places: &Places) -> Vec<Recent> {
    let text = trindle_common::files::read_small(&places.data.first().map(|d| d.join("recently-used.xbel")).unwrap_or_default()).unwrap_or_default();
    let mut out: Vec<Recent> = text.split("<bookmark ").skip(1).filter_map(|b| {
        let head = &b[..b.find('>')?];
        let path = PathBuf::from(unpercent(unescape(&attr(head, "href")?).strip_prefix("file://")?));
        let mime = b.split("<mime:mime-type ").nth(1).and_then(|m| attr(m, "type")).unwrap_or_default();
        there(&path).then(|| Recent { path, mime, modified: attr(head, "modified").unwrap_or_default() })
    }).collect();
    out.sort_by(|a, b| b.modified.cmp(&a.modified));
    out
}

fn there(path: &Path) -> bool {
    if own_place(path) { path.exists() } else { reach::meta(path, true).is_some() }
}

fn attr(tag: &str, name: &str) -> Option<String> {
    let at = tag.find(&format!(" {name}=\"")).map(|i| i + name.len() + 3).or_else(|| tag.starts_with(&format!("{name}=\"")).then_some(name.len() + 2))?;
    Some(tag[at..at + tag[at..].find('"')?].to_string())
}

fn unescape(s: &str) -> String {
    s.replace("&lt;", "<").replace("&gt;", ">").replace("&quot;", "\"").replace("&apos;", "'").replace("&amp;", "&")
}

fn unpercent(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        let hex = if b[i] == b'%' && i + 2 < b.len() {
            std::str::from_utf8(&b[i + 1..i + 3]).ok().and_then(|h| u8::from_str_radix(h, 16).ok())
        } else {
            None
        };
        match hex {
            Some(v) => {
                out.push(v);
                i += 3;
            }
            None => {
                out.push(b[i]);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Scratch(PathBuf);

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn scratch(name: &str) -> (Scratch, Places) {
        let root = std::env::temp_dir().join(format!("trindle-open-with-{}-{name}", std::process::id()));
        let places = Places { config: root.join("config"), data: vec![root.join("home"), root.join("sys")] };
        for d in [&places.config, &places.data[0].join("applications"), &places.data[1].join("applications/kde")] {
            std::fs::create_dir_all(d).unwrap();
        }
        (Scratch(root), places)
    }

    fn desktop(at: &Path, name: &str, mimes: &str, extra: &str) {
        std::fs::write(at, format!("[Desktop Entry]\nType=Application\nName={name}\nExec=view %U\nIcon=viewer\nMimeType={mimes}\n{extra}")).unwrap();
    }

    #[test]
    fn exec_is_split_by_the_specification_not_on_every_space() {
        let app = |exec: &str| App { id: "a.desktop".into(), name: "A".into(), icon: String::new(), exec: exec.into(), mimes: vec![] };
        let f = Path::new("/t/a b.txt");
        assert_eq!(app(r#""/opt/My App/run" %F"#).command(&[f]), ["/opt/My App/run", "/t/a b.txt"]);
        assert_eq!(app("view --file=%u 100%%").command(&[f]), ["view", "--file=/t/a b.txt", "100%"]);
        assert_eq!(app(r#"sh -c "echo \"hi\" %i""#).command(&[f]), ["sh", "-c", r#"echo "hi" "#, "/t/a b.txt"]);
    }

    #[test]
    fn apps_for_a_kind_of_file_put_the_default_first_and_the_persons_own_copy_wins() {
        let (_s, p) = scratch("apps");
        let sys = p.data[1].join("applications");
        desktop(&sys.join("b.desktop"), "Bravo", "image/png;", "");
        desktop(&sys.join("a.desktop"), "Alpha", "image/png;text/plain;", "");
        desktop(&sys.join("kde/c.desktop"), "John Smith", "image/png;", "");
        desktop(&sys.join("hidden.desktop"), "Hidden", "image/png;", "NoDisplay=true\n");
        desktop(&sys.join("gone.desktop"), "Gone", "image/png;", "");
        desktop(&p.data[0].join("applications/b.desktop"), "Bravo mine", "image/png;", "");
        std::fs::write(p.config.join("mimeapps.list"), "[Default Applications]\nimage/png=kde-c.desktop;\n[Removed Associations]\nimage/png=gone.desktop;\n").unwrap();
        let names: Vec<String> = apps_for(&p, "image/png").into_iter().map(|a| a.name).collect();
        assert_eq!(names, ["John Smith", "Alpha", "Bravo mine"]);
        assert_eq!(apps_for(&p, "text/plain").len(), 1);
        let alpha = apps_for(&p, "text/plain").remove(0);
        set_default(&p, "image/png", &alpha).unwrap();
        assert_eq!(apps_for(&p, "image/png")[0].name, "Alpha");
    }

    #[test]
    fn the_command_fills_in_files_each_as_one_word() {
        let app = App { id: "v.desktop".into(), name: "V".into(), icon: String::new(), exec: "view --new %F %i".into(), mimes: vec![] };
        let (a, b) = (Path::new("/tmp/a b.png"), Path::new("/tmp/c.png"));
        assert_eq!(app.command(&[a, b]), ["view", "--new", "/tmp/a b.png", "/tmp/c.png"]);
        let plain = App { exec: "view".into(), ..app };
        assert_eq!(plain.command(&[a]), ["view", "/tmp/a b.png"]);
    }

    #[test]
    fn recent_files_come_newest_first_and_only_those_still_there() {
        let (_s, p) = scratch("recent");
        let there = p.data[0].join("a & b.txt");
        std::fs::write(&there, "x").unwrap();
        let href = format!("file://{}", p.data[0].join("a%20&amp;%20b.txt").display());
        let older = format!("file://{}", p.config.display());
        std::fs::write(p.data[0].join("recently-used.xbel"), format!(
            "<?xml version=\"1.0\"?>\n<xbel version=\"1.0\">\n<bookmark href=\"{older}\" added=\"x\" modified=\"2026-01-01T00:00:00Z\"><info><metadata><mime:mime-type type=\"inode/directory\"/></metadata></info></bookmark>\n\
             <bookmark href=\"{href}\" modified=\"2026-10-01T00:00:00Z\"><info><metadata><mime:mime-type type=\"text/plain\"/></metadata></info></bookmark>\n\
             <bookmark href=\"file:///nowhere/at/all\" modified=\"2026-12-01T00:00:00Z\"></bookmark>\n</xbel>")).unwrap();
        let r = recent(&p);
        assert_eq!(r.iter().map(|r| r.path.clone()).collect::<Vec<_>>(), [there, p.config.clone()]);
        assert_eq!(r[0].mime, "text/plain");
    }

    #[test]
    fn a_recent_file_on_a_share_that_does_not_answer_is_left_out_and_waited_for_only_once() {
        use crate::common::file_work::reach;
        let (_s, p) = scratch("recent-silent");
        let (live, dead) = (p.data[0].join("live"), p.data[0].join("dead"));
        std::fs::create_dir_all(&live).unwrap();
        std::fs::create_dir_all(&dead).unwrap();
        let mut xbel = String::from("<?xml version=\"1.0\"?>\n<xbel version=\"1.0\">\n");
        for (i, path) in [live.join("a.txt"), dead.join("b.txt"), dead.join("c.txt"), dead.join("d.txt")].iter().enumerate() {
            std::fs::write(path, "x").unwrap();
            xbel += &format!("<bookmark href=\"file://{}\" modified=\"2026-10-0{}T00:00:00Z\"></bookmark>\n", path.display(), i + 1);
        }
        std::fs::write(p.data[0].join("recently-used.xbel"), xbel + "</xbel>").unwrap();

        reach::never(&dead, true);
        let began = std::time::Instant::now();
        let names: Vec<_> = recent(&p).iter().map(|r| r.path.file_name().unwrap().to_string_lossy().into_owned()).collect();
        let took = began.elapsed();
        reach::never(&dead, false);
        assert_eq!(names, ["a.txt"], "files on a place that did not answer were listed as there");
        assert!(took < reach::ANSWER_WITHIN * 4, "three files on one silent place took {took:?}");
        assert_eq!(recent(&p).len(), 4, "a place that answers again was still left out");
    }

    #[test]
    fn the_chooser_lists_apps_by_name_for_a_screen_reader() {
        let a = App { id: "v.desktop".into(), name: "Viewer".into(), icon: "viewer".into(), exec: "view".into(), mimes: vec![] };
        assert_eq!(chooser(&[a], None, |i| i).said(), ["Viewer"]);
    }
}
