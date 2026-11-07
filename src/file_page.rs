//! A file chooser as a page in the window: one folder's folders and files, gone into by opening them, with a box for the name when saving.

use std::path::{Component, Path, PathBuf};

use crate::bars::path_bar;
use crate::dialog::dialog;
use crate::lang::format;
use crate::t;
use crate::widget::{entry, item, list, main_action, note, row, switch, word_button, form_label, Length, Node, Tone};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Purpose {
    Open,
    Save,
    Folder,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Found {
    pub name: String,
    pub folder: bool,
    pub bytes: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub enum FileStep {
    Choose(usize),
    Open(usize),
    Up,
    Path(usize),
    Name(String),
    Hidden(bool),
    Done,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FilePage {
    pub purpose: Purpose,
    pub folder: PathBuf,
    pub name: String,
    pub hidden: bool,
    pub chosen: Option<usize>,
    pub problem: Option<String>,
    found: Vec<Found>,
}

impl FilePage {
    pub fn new(purpose: Purpose, folder: impl Into<PathBuf>) -> FilePage {
        let mut page = FilePage { purpose, folder: folder.into(), name: String::new(), hidden: false, chosen: None, problem: None, found: Vec::new() };
        page.read();
        page
    }

    pub fn shown(&self) -> Vec<&Found> {
        self.found.iter().filter(|f| self.hidden || !f.name.starts_with('.')).filter(|f| f.folder || self.purpose != Purpose::Folder).collect()
    }

    fn read(&mut self) {
        (self.chosen, self.problem) = (None, None);
        self.found = match std::fs::read_dir(&self.folder) {
            Ok(dir) => dir.flatten().map(|e| {
                let meta = std::fs::metadata(e.path()).ok();
                Found { name: e.file_name().to_string_lossy().into_owned(), folder: meta.as_ref().is_some_and(|m| m.is_dir()), bytes: meta.map_or(0, |m| m.len()) }
            }).collect(),
            Err(_) => {
                self.problem = Some(t!("kit-filepage-unreadable").to_string());
                Vec::new()
            }
        };
        self.found.sort_by_cached_key(|f| (!f.folder, f.name.to_lowercase()));
    }

    fn go(&mut self, folder: PathBuf) {
        self.folder = folder;
        self.read();
    }

    pub fn update(&mut self, step: FileStep) -> Option<PathBuf> {
        let picked = |page: &FilePage, i: usize| page.shown().get(i).map(|f| (*f).clone());
        match step {
            FileStep::Choose(i) => {
                self.chosen = Some(i);
                if let Some(f) = picked(self, i).filter(|f| !f.folder && self.purpose == Purpose::Save) {
                    self.name = f.name;
                }
            }
            FileStep::Open(i) => match picked(self, i) {
                Some(f) if f.folder => self.go(self.folder.join(f.name)),
                Some(f) if self.purpose == Purpose::Open => return Some(self.folder.join(f.name)),
                Some(f) => self.name = f.name,
                None => {}
            },
            FileStep::Up => {
                if let Some(up) = self.folder.parent().map(Path::to_path_buf) {
                    self.go(up);
                }
            }
            FileStep::Path(i) => {
                let up: PathBuf = self.folder.components().take(i + 1).collect();
                self.go(up);
            }
            FileStep::Name(name) => (self.name, self.problem) = (name, None),
            FileStep::Hidden(on) => (self.hidden, self.chosen) = (on, None),
            FileStep::Done => return self.answer(),
        }
        None
    }

    fn answer(&mut self) -> Option<PathBuf> {
        let chosen = self.chosen.and_then(|i| self.shown().get(i).map(|f| (*f).clone()));
        let refused = match self.purpose {
            Purpose::Open => match chosen {
                Some(f) if f.folder => {
                    self.go(self.folder.join(f.name));
                    return None;
                }
                Some(f) => return Some(self.folder.join(f.name)),
                None => t!("kit-files-choose-first"),
            },
            Purpose::Folder => return Some(chosen.map_or_else(|| self.folder.clone(), |f| self.folder.join(f.name))),
            Purpose::Save => {
                let name = self.name.trim();
                if !name.is_empty() && name != "." && name != ".." && !name.contains('/') && !name.contains('\0') {
                    return Some(self.folder.join(name));
                }
                t!("kit-files-name-needed")
            }
        };
        self.problem = Some(refused.to_string());
        None
    }

    pub fn view<M: Clone + 'static>(&self, title: impl Into<String>, wrap: impl Fn(FileStep) -> M + Clone + 'static, cancel: M) -> Node<M> {
        let parts: Vec<String> = self.folder.components().map(|c| match c {
            Component::RootDir => t!("kit-files-computer").to_string(),
            c => c.as_os_str().to_string_lossy().into_owned(),
        }).collect();
        let rows = self.shown().into_iter().map(|f| if f.folder {
            item(f.name.clone()).icon("folder-symbolic").folder()
        } else {
            item(f.name.clone()).icon("text-x-generic-symbolic").detail(format::size(f.bytes))
        }).collect();
        let (choose, open, path, hidden, name, done) = (wrap.clone(), wrap.clone(), wrap.clone(), wrap.clone(), wrap.clone(), wrap.clone());
        let gap = crate::theme::gaps().medium;
        let (lead, answer) = match self.purpose {
            Purpose::Open => (t!("kit-files-open-lead"), t!("kit-files-open")),
            Purpose::Save => (t!("kit-files-save-lead"), t!("kit-files-save")),
            Purpose::Folder => (t!("kit-files-folder-lead"), t!("kit-files-folder")),
        };
        let mut page = dialog(title, lead, cancel)
            .holds(row(vec![
                word_button(t!("kit-files-up")).on_press(wrap(FileStep::Up)),
                path_bar(&parts, 3, move |i| path(FileStep::Path(i))).width(Length::Fill),
                form_label(t!("kit-files-hidden")),
                switch(self.hidden, t!("kit-files-hidden"), move |on| hidden(FileStep::Hidden(on))),
            ]).gap(gap))
            .holds(list(rows, self.chosen, move |i| choose(FileStep::Choose(i))).on_open(move |i| open(FileStep::Open(i)))
                .called(t!("kit-files-list")).height(Length::Fill).key("files-list"));
        if self.shown().is_empty() && self.problem.is_none() {
            page = page.holds(note(t!("kit-files-empty")));
        }
        if self.purpose == Purpose::Save {
            page = page.holds(entry(self.name.clone(), t!("kit-files-name"), move |s| name(FileStep::Name(s))).on_submit(wrap(FileStep::Done)).key("files-name"));
        }
        if let Some(problem) = &self.problem {
            page = page.holds(note(problem.clone()).tone(Tone::Problem));
        }
        page.answer(main_action("", answer).on_press(done(FileStep::Done))).node()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::Theme;
    use crate::ui::Ui;

    fn folder(name: &str) -> PathBuf {
        let at = std::env::temp_dir().join(format!("trindle-file-page-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&at);
        std::fs::create_dir_all(at.join("Reports")).unwrap();
        std::fs::create_dir_all(at.join(".cache")).unwrap();
        std::fs::write(at.join("notes.txt"), "John Smith").unwrap();
        std::fs::write(at.join("Agenda.txt"), "").unwrap();
        std::fs::write(at.join("Reports/june.txt"), "").unwrap();
        at
    }

    fn names(page: &FilePage) -> Vec<&str> {
        page.shown().into_iter().map(|f| f.name.as_str()).collect()
    }

    #[test]
    fn folders_come_first_hidden_ones_wait_and_opening_goes_in_and_out() {
        let at = folder("walk");
        let mut page = FilePage::new(Purpose::Open, &at);
        assert_eq!(names(&page), ["Reports", "Agenda.txt", "notes.txt"]);
        page.update(FileStep::Hidden(true));
        assert_eq!(names(&page), [".cache", "Reports", "Agenda.txt", "notes.txt"]);
        page.update(FileStep::Hidden(false));
        assert_eq!(page.update(FileStep::Open(0)), None);
        assert_eq!((page.folder.clone(), names(&page)), (at.join("Reports"), vec!["june.txt"]));
        assert_eq!(page.update(FileStep::Open(0)), Some(at.join("Reports/june.txt")));
        page.update(FileStep::Up);
        assert_eq!(page.folder, at);
        let deep = at.components().count();
        page.update(FileStep::Path(deep - 2));
        assert_eq!(Some(page.folder.as_path()), at.parent());
        std::fs::remove_dir_all(&at).unwrap();
    }

    #[test]
    fn an_answer_that_cannot_be_taken_says_why_in_red_words() {
        let at = folder("answer");
        let mut open = FilePage::new(Purpose::Open, &at);
        assert_eq!(open.update(FileStep::Done), None);
        assert!(open.problem.is_some());
        open.update(FileStep::Choose(2));
        assert_eq!(open.update(FileStep::Done), Some(at.join("notes.txt")));

        let mut save = FilePage::new(Purpose::Save, &at);
        for bad in ["", "  ", "a/b", ".."] {
            save.update(FileStep::Name(bad.into()));
            assert_eq!(save.update(FileStep::Done), None, "{bad:?}");
            assert!(save.problem.is_some());
        }
        save.update(FileStep::Choose(1));
        assert_eq!(save.name, "Agenda.txt", "choosing a file takes its name to write over");
        save.update(FileStep::Name("plan.txt".into()));
        assert_eq!((save.update(FileStep::Done), save.problem.clone()), (Some(at.join("plan.txt")), None));

        let mut folders = FilePage::new(Purpose::Folder, &at);
        assert_eq!(names(&folders), ["Reports"]);
        assert_eq!(folders.update(FileStep::Done), Some(at.clone()));
        folders.update(FileStep::Choose(0));
        assert_eq!(folders.update(FileStep::Done), Some(at.join("Reports")));

        let gone = FilePage::new(Purpose::Open, at.join("not-there"));
        assert!(gone.problem.is_some() && gone.shown().is_empty());
        std::fs::remove_dir_all(&at).unwrap();
    }

    #[test]
    fn the_page_is_a_named_dialog_driven_by_keys() {
        use crate::input::{Input, Keysym};
        use std::time::Instant;
        let at = folder("keys");
        let mut page = FilePage::new(Purpose::Save, &at);
        page.update(FileStep::Choose(0));
        let shown = || {
            let mut ui = Ui::new(page.view("Save the notes", |s| s, FileStep::Done), Theme::new(false, "#fbc711"));
            ui.lay_at(800, 600);
            ui
        };
        let mut told = shown();
        assert!(told.unnamed_controls().is_empty());
        assert!(told.access("T").nodes.iter().any(|(_, n)| n.role() == accesskit::Role::Dialog && n.label() == Some("Save the notes")));
        let mut ui = shown();
        ui.keyboard_to("files-list");
        let key = |sym| Input::Key { sym, text: None, mods: Default::default(), repeat: false };
        let sent = ui.input(&key(Keysym::Down), Instant::now()).messages;
        assert!(sent.iter().any(|m| matches!(m, FileStep::Choose(_))), "{sent:?}");
        let sent = ui.input(&key(Keysym::Return), Instant::now()).messages;
        assert!(sent.iter().any(|m| matches!(m, FileStep::Open(_))), "{sent:?}");
        std::fs::remove_dir_all(&at).unwrap();
    }
}
