//! A list of the program's own things shown sorted, filtered and grouped, which says row by row what changed so a view can follow without being rebuilt.

use std::cmp::Ordering;
use std::ops::Range;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Change {
    Reset,
    Inserted(Range<usize>),
    Removed(Range<usize>),
    Changed(usize),
}

type Order<T> = Box<dyn Fn(&T, &T) -> Ordering>;

pub struct ListModel<T> {
    items: Vec<T>,
    order: Option<Order<T>>,
    keep: Option<Box<dyn Fn(&T) -> bool>>,
    group: Option<Box<dyn Fn(&T) -> String>>,
    shown: Vec<usize>,
    changes: Vec<Change>,
}

impl<T> ListModel<T> {
    pub fn new(items: Vec<T>) -> Self {
        let shown = (0..items.len()).collect();
        ListModel { items, order: None, keep: None, group: None, shown, changes: Vec::new() }
    }

    pub fn len(&self) -> usize {
        self.shown.len()
    }

    pub fn is_empty(&self) -> bool {
        self.shown.is_empty()
    }

    pub fn row(&self, row: usize) -> Option<&T> {
        self.shown.get(row).map(|&i| &self.items[i])
    }

    pub fn rows(&self) -> impl Iterator<Item = &T> {
        self.shown.iter().map(|&i| &self.items[i])
    }

    pub fn all(&self) -> &[T] {
        &self.items
    }

    pub fn take_changes(&mut self) -> Vec<Change> {
        std::mem::take(&mut self.changes)
    }

    pub fn sort_by(&mut self, order: impl Fn(&T, &T) -> Ordering + 'static) {
        self.order = Some(Box::new(order));
        self.reshow();
    }

    pub fn filter(&mut self, keep: Option<Box<dyn Fn(&T) -> bool>>) {
        self.keep = keep;
        self.reshow();
    }

    pub fn group_by(&mut self, group: impl Fn(&T) -> String + 'static) {
        self.group = Some(Box::new(group));
        self.reshow();
    }

    pub fn groups(&self) -> Vec<(String, Range<usize>)> {
        let Some(group) = &self.group else { return Vec::new() };
        let mut out: Vec<(String, Range<usize>)> = Vec::new();
        for (row, t) in self.rows().enumerate() {
            let name = group(t);
            match out.last_mut() {
                Some((last, rows)) if *last == name => rows.end = row + 1,
                _ => out.push((name, row..row + 1)),
            }
        }
        out
    }

    fn shows(&self, t: &T) -> bool {
        self.keep.as_ref().is_none_or(|k| k(t))
    }

    fn cmp(&self, a: usize, b: usize) -> Ordering {
        let (x, y) = (&self.items[a], &self.items[b]);
        let by_group = self.group.as_ref().map_or(Ordering::Equal, |g| g(x).cmp(&g(y)));
        by_group.then_with(|| self.order.as_ref().map_or(Ordering::Equal, |o| o(x, y))).then(a.cmp(&b))
    }

    fn reshow(&mut self) {
        let mut shown: Vec<usize> = (0..self.items.len()).filter(|&i| self.shows(&self.items[i])).collect();
        match &self.group {
            None => shown.sort_by(|&a, &b| self.cmp(a, b)),
            Some(group) => {
                let mut names = vec![String::new(); self.items.len()];
                for &i in &shown {
                    names[i] = group(&self.items[i]);
                }
                shown.sort_by(|&a, &b| {
                    names[a].cmp(&names[b]).then_with(|| self.order.as_ref().map_or(Ordering::Equal, |o| o(&self.items[a], &self.items[b]))).then(a.cmp(&b))
                });
            }
        }
        self.shown = shown;
        self.changes.push(Change::Reset);
    }

    fn place_of(&self, i: usize) -> usize {
        self.shown.partition_point(|&j| self.cmp(j, i) == Ordering::Less)
    }

    pub fn push(&mut self, t: T) {
        self.items.push(t);
        let i = self.items.len() - 1;
        if self.shows(&self.items[i]) {
            let row = self.place_of(i);
            self.shown.insert(row, i);
            self.changes.push(Change::Inserted(row..row + 1));
        }
    }

    pub fn remove(&mut self, row: usize) -> Option<T> {
        if row >= self.shown.len() {
            return None;
        }
        let i = self.shown.remove(row);
        self.shown.iter_mut().filter(|j| **j > i).for_each(|j| *j -= 1);
        self.changes.push(Change::Removed(row..row + 1));
        Some(self.items.remove(i))
    }

    pub fn edit(&mut self, row: usize, f: impl FnOnce(&mut T)) {
        let Some(&i) = self.shown.get(row) else { return };
        f(&mut self.items[i]);
        self.shown.remove(row);
        if !self.shows(&self.items[i]) {
            self.changes.push(Change::Removed(row..row + 1));
            return;
        }
        let to = self.place_of(i);
        self.shown.insert(to, i);
        if to == row {
            self.changes.push(Change::Changed(row));
        } else {
            self.changes.extend([Change::Removed(row..row + 1), Change::Inserted(to..to + 1)]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(m: &ListModel<(&'static str, u32)>) -> Vec<&'static str> {
        m.rows().map(|t| t.0).collect()
    }

    fn fruit() -> ListModel<(&'static str, u32)> {
        ListModel::new(vec![("pear", 3), ("apple", 5), ("plum", 1), ("fig", 2)])
    }

    #[test]
    fn sorting_and_filtering_reset_the_rows() {
        let mut m = fruit();
        m.sort_by(|a, b| a.0.cmp(b.0));
        assert_eq!(names(&m), vec!["apple", "fig", "pear", "plum"]);
        m.filter(Some(Box::new(|t: &(&str, u32)| t.1 > 1)));
        assert_eq!(names(&m), vec!["apple", "fig", "pear"]);
        assert_eq!(m.take_changes(), vec![Change::Reset, Change::Reset]);
        assert_eq!(m.all().len(), 4);
        m.filter(None);
        assert_eq!(m.len(), 4);
    }

    #[test]
    fn a_group_name_is_made_once_for_a_thing_however_many_times_it_is_compared() {
        let made = std::rc::Rc::new(std::cell::Cell::new(0));
        let counted = made.clone();
        let mut m = ListModel::new((0..200u32).map(|i| ("x", i * 7919 % 200)).collect());
        m.sort_by(|a, b| a.1.cmp(&b.1));
        m.group_by(move |t| {
            counted.set(counted.get() + 1);
            format!("Group {}", t.1 % 5)
        });
        assert_eq!(made.get(), 200);
        assert_eq!(m.groups().len(), 5);
        assert!(m.rows().map(|t| t.1).collect::<Vec<_>>().windows(2).all(|w| w[0] % 5 < w[1] % 5 || (w[0] % 5 == w[1] % 5 && w[0] < w[1])));
    }

    #[test]
    fn a_new_thing_goes_in_its_sorted_place_unless_filtered_out() {
        let mut m = fruit();
        m.sort_by(|a, b| a.0.cmp(b.0));
        m.filter(Some(Box::new(|t: &(&str, u32)| t.1 > 1)));
        m.take_changes();
        m.push(("banana", 4));
        m.push(("kiwi", 0));
        assert_eq!(names(&m), vec!["apple", "banana", "fig", "pear"]);
        assert_eq!(m.take_changes(), vec![Change::Inserted(1..2)]);
    }

    #[test]
    fn an_edit_changes_in_place_moves_or_leaves() {
        let mut m = fruit();
        m.sort_by(|a, b| a.1.cmp(&b.1));
        m.filter(Some(Box::new(|t: &(&str, u32)| t.1 < 10)));
        m.take_changes();
        assert_eq!(names(&m), vec!["plum", "fig", "pear", "apple"]);
        m.edit(1, |t| t.0 = "date");
        m.edit(0, |t| t.1 = 4);
        assert_eq!(names(&m), vec!["date", "pear", "plum", "apple"]);
        m.edit(3, |t| t.1 = 99);
        assert_eq!(m.take_changes(), vec![Change::Changed(1), Change::Removed(0..1), Change::Inserted(2..3), Change::Removed(3..4)]);
        assert_eq!(m.remove(0).map(|t| t.0), Some("date"));
        assert_eq!((names(&m), m.take_changes()), (vec!["pear", "plum"], vec![Change::Removed(0..1)]));
        assert_eq!(m.row(1).map(|t| t.0), Some("plum"));
        assert!(m.remove(5).is_none());
    }

    #[test]
    fn groups_stand_together_in_name_order_each_sorted() {
        let mut m = fruit();
        m.sort_by(|a, b| a.0.cmp(b.0));
        m.group_by(|t| if t.1 % 2 == 1 { "odd".into() } else { "even".into() });
        assert_eq!(names(&m), vec!["fig", "apple", "pear", "plum"]);
        assert_eq!(m.groups(), vec![("even".to_string(), 0..1), ("odd".to_string(), 1..4)]);
        m.push(("lime", 6));
        assert_eq!(m.groups()[0], ("even".to_string(), 0..2));
    }
}
