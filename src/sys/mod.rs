//! Every call into the kernel or a C library that Rust cannot check, each behind a small safe function; no other file holds `unsafe`.

pub mod disk;
pub mod fd;
#[cfg(feature = "ui")]
pub mod gbm;
#[cfg(feature = "ui")]
pub mod gl;
#[cfg(feature = "ui")]
pub(crate) mod lean;
#[cfg(feature = "ui")]
pub mod mem;
pub mod os;
pub mod signal;
#[cfg(feature = "ui")]
pub(crate) mod sock;
