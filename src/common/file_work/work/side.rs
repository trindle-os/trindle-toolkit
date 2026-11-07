//! What a program using the work adds of its own: kinds of work the work itself does not do, places only it can reach, and what it calls them.

use std::convert::Infallible;
use std::fmt::Debug;
use std::path::{Path, PathBuf};

use super::looks::name_on_disk;
use super::plan::{Ctl, News, Over, Plan};

pub trait Side: Clone + Debug + Default + PartialEq + Eq + Sized {
    type Job: Clone + Debug;
    type Key;
    type Links: Default;
    type Undo: Clone + Debug + PartialEq + Eq;
    type Locked: Clone + Debug;

    fn elsewhere(_plan: &Plan<Self>, _ctl: &Ctl<Self>, _tell: &dyn Fn(News<Self>)) -> Option<Over<Self>> {
        None
    }

    fn job(_plan: &Plan<Self>, _ctl: &Ctl<Self>, _tell: &dyn Fn(News<Self>)) -> Over<Self> {
        Over::default()
    }

    fn undo(_step: &Self::Undo) -> Result<Self::Undo, Option<PathBuf>> {
        Err(None)
    }

    fn name_of(path: &Path) -> String {
        name_on_disk(path)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Plain;

impl Side for Plain {
    type Job = Infallible;
    type Key = Infallible;
    type Links = ();
    type Undo = Infallible;
    type Locked = Infallible;
}
