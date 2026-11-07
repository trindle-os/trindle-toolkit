//! The work that moves files about: copying, moving, the Trash, deleting and putting back, and undoing them. It runs on a thread of its own and says how it is getting on; files are copied on several threads at once, or one at a time where a spinning disk is involved.

mod bin;
mod copy;
mod crew;
#[cfg(any(test, feature = "testing"))]
pub mod faults;
mod finish;
mod leave;
mod looks;
mod plan;
mod run;
mod said;
mod side;
mod undo;
mod walk;

pub use crate::common::files::{free_in, open_in, open_same, through, Which};
pub use bin::to_trash;
pub use copy::{rotational, threads, CHUNK};
pub use crew::{crew, Bench};
pub use looks::*;
pub use plan::*;
pub use run::run;
pub use said::*;
pub use side::{Plain, Side};

#[cfg(test)]
mod tests;
