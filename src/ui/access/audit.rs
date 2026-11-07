//! For tests: checks that a screen reader can name and reach all a window shows.

use super::*;

fn unnamed_ids(update: &TreeUpdate) -> Vec<(NodeId, String)> {
    let mut wrong = Vec::new();
    for (id, part) in &update.nodes {
        let at = part.bounds().map_or(String::new(), |b| format!(" at {:.0},{:.0}", b.x0, b.y0));
        if part.role() == Role::Unknown {
            wrong.push((*id, format!("part has no role{at}")));
            continue;
        }
        let needs_a_name = matches!(part.role(), Role::Button | Role::CheckBox | Role::Switch | Role::Slider | Role::TextInput
            | Role::PasswordInput | Role::SearchInput | Role::ComboBox | Role::RadioButton | Role::RadioGroup | Role::ListBoxOption
            | Role::MenuItem | Role::Tab | Role::Link | Role::Image | Role::ProgressIndicator | Role::Terminal | Role::Menu);
        let named = [part.label(), part.value()].into_iter().flatten().any(|n| !n.trim().is_empty());
        if needs_a_name && !part.is_hidden() && !named {
            wrong.push((*id, format!("{:?} has no name{at}", part.role())));
        }
    }
    wrong
}

impl<M: Clone> Ui<M> {
    pub fn unnamed_controls(&mut self) -> Vec<String> {
        let update = self.access("A window");
        unnamed_ids(&update).into_iter().map(|(id, why)| {
            let mut key = self.spots.iter().find(|s| part_id(&s.id, Piece::Widget) == id).map(|s| s.id.clone());
            if key.is_none() {
                key = self.spots.iter().find(|s| match self.tree.at(&s.path).map(|n| &n.what) {
                    Some(What::Choice { options, .. }) => (0..options.len()).any(|r| part_id(&s.id, Piece::Row(r)) == id),
                    Some(What::List { rows, .. }) => (0..rows.len()).any(|r| part_id(&s.id, Piece::Row(r)) == id),
                    _ => false,
                }).map(|s| format!("a row of {}", s.id));
            }
            format!("{why}{}", key.map_or(String::new(), |k| format!(" (key {k:?})")))
        }).collect()
    }

    pub fn unnamed_lists(&mut self) -> Vec<String> {
        let update = self.access("A window");
        update.nodes.iter().filter(|(_, p)| matches!(p.role(), Role::ListBox | Role::Tree) && !p.is_hidden())
            .filter(|(_, p)| ![p.label(), p.value()].into_iter().flatten().any(|n| !n.trim().is_empty()))
            .map(|(_, p)| p.bounds().map_or(String::new(), |b| format!(" at {:.0},{:.0}", b.x0, b.y0))).collect()
    }

    pub fn skip_audit(&mut self) {
        self.audit_off = true;
    }

    #[cfg(feature = "testing")]
    pub(crate) fn audit_what_is_painted(&mut self) {
        if self.audited || self.audit_off {
            return;
        }
        self.audited = true;
        let mode = std::env::var("TRINDLE_AUDIT").unwrap_or_default();
        if mode == "off" {
            return;
        }
        for list in self.unnamed_lists() {
            eprintln!("AUDIT warning: a list has no name (give it one with `.called(..)`){list}");
        }
        let wrong = self.unnamed_controls();
        if wrong.is_empty() {
            return;
        }
        if mode == "report" {
            eprintln!("AUDIT [{}] {}", std::thread::current().name().unwrap_or("?"), wrong.join("; "));
            return;
        }
        panic!("a screen reader could not use these controls (TRINDLE_AUDIT=off skips the check): {}", wrong.join("; "));
    }

    pub fn kept_from_a_reader(&mut self) -> Vec<String> {
        use accesskit_consumer::{common_filter, Tree};
        let update = self.access("A window");
        let told: Vec<(NodeId, Role, String)> = update.nodes.iter()
            .map(|(id, part)| (*id, part.role(), part.label().unwrap_or_default().to_string())).collect();
        let mut wrong = Vec::new();
        let mut once = std::collections::HashSet::new();
        for (id, role, name) in &told {
            if !once.insert(*id) {
                wrong.push(format!("two parts share a number: {role:?} {name:?}"));
            }
        }
        if !wrong.is_empty() {
            return wrong;
        }
        let tree = Tree::new(update, true);
        let mut reached = std::collections::HashSet::new();
        let mut todo = vec![tree.state().root()];
        while let Some(part) = todo.pop() {
            reached.insert(part.locate().0);
            todo.extend(part.filtered_children(common_filter));
        }
        for (id, role, name) in told {
            if !matches!(role, Role::GenericContainer | Role::TextRun) && !reached.contains(&id) {
                wrong.push(format!("{role:?} {name:?}"));
            }
        }
        wrong
    }
}
