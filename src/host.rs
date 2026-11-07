//! What the program using the toolkit decides for itself; a program that says nothing gets the plain answers.

use std::sync::OnceLock;

#[cfg(feature = "ui")]
use crate::ui::Labels;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Act {
    PersonsOwn,
    Waiting,
    Unknown,
}

pub trait Host: Sync {
    #[cfg(feature = "ui")]
    fn labels(&self) -> Labels {
        Labels::Brief
    }

    #[cfg(feature = "ui")]
    fn double_click(&self) -> std::time::Duration {
        crate::ui::MULTI_CLICK
    }

    fn moving(&self) -> bool {
        true
    }

    fn drawing_thread(&self) {}

    fn missing_icon(&self, _id: &str, _shown: &str) {}

    fn own_icon_set(&self) -> Option<&'static str> {
        None
    }

    fn end_trial_if_over(&self) {}

    fn asks_only_the_person(&self, _name: &str) -> bool {
        false
    }

    fn act(&self, _act: &str) -> Act {
        Act::Unknown
    }

    fn reader_switched_on(&self) -> bool {
        false
    }

    fn words(&self) -> &'static [(&'static str, &'static str)] {
        &[]
    }

    fn language(&self) -> Option<String> {
        None
    }

    fn numerals(&self) -> Option<String> {
        None
    }
}

struct Plain;

impl Host for Plain {}

static HOST: OnceLock<&'static dyn Host> = OnceLock::new();

pub fn set(host: &'static dyn Host) -> bool {
    HOST.set(host).is_ok()
}

pub fn host() -> &'static dyn Host {
    #[cfg(any(test, feature = "testing"))]
    if let Some(asked) = ASKED.get() {
        return asked;
    }
    HOST.get().copied().unwrap_or(&Plain)
}

#[cfg(any(test, feature = "testing"))]
thread_local! {
    static ASKED: std::cell::Cell<Option<&'static dyn Host>> = const { std::cell::Cell::new(None) };
}

#[cfg(any(test, feature = "testing"))]
pub fn with<R>(host: &'static dyn Host, during: impl FnOnce() -> R) -> R {
    struct Restore(Option<&'static dyn Host>);
    impl Drop for Restore {
        fn drop(&mut self) {
            ASKED.set(self.0);
        }
    }
    let _restore = Restore(ASKED.replace(Some(host)));
    during()
}
