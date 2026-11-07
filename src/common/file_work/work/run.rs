//! The start of every piece of work: what is checked first, and which work it then is.

use std::fs;
use std::path::{Path, PathBuf};

use super::bin::bin;
use super::looks::once_each;
use super::plan::{Kind, News, Over, Plan, Sure};
use super::said::{changed_since, trouble};
use super::side::Side;
use super::walk::{Mode, Walk};
use super::{Ctl, Which};

fn still_it(s: &Sure) -> bool {
    fs::symlink_metadata(&s.path).is_ok_and(|m| Which::of(&m) == s.which)
        && s.held.as_ref().is_none_or(|held| held.metadata().is_ok_and(|m| Which::of(&m) == s.which))
}

pub fn run<S: Side>(mut plan: Plan<S>, ctl: &Ctl<S>, tell: &dyn Fn(News<S>)) -> Over<S> {
    if matches!(plan.kind, Kind::Copy | Kind::Move | Kind::Trash | Kind::Delete) {
        plan.items = once_each(std::mem::take(&mut plan.items));
    }
    let other = |item: &PathBuf| ctl.sure.iter().any(|s| s.path == *item && !still_it(s));
    if let Some(bad) = plan.items.iter().find(|item| other(item)) {
        return Over { failed: vec![trouble(&S::name_of(bad), changed_since())], ..Over::default() };
    }
    if let Some(over) = S::elsewhere(&plan, ctl, tell) {
        return over;
    }
    let mode = match &plan.kind {
        Kind::Copy => Mode::Copy,
        Kind::Move => Mode::Move,
        Kind::Restore => Mode::Restore,
        Kind::Undo(_) => Mode::Undo,
        Kind::Trash | Kind::Delete => return bin(&plan, ctl, tell),
        Kind::Other(_) => return S::job(&plan, ctl, tell),
    };
    let mut w = Walk::new(ctl, tell, mode);
    w.go_through(&plan);
    let mut ends: Vec<&Path> = vec![plan.to.as_path()];
    ends.extend(plan.items.iter().map(PathBuf::as_path));
    w.finish(&ends)
}
