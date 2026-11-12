//! Why a thing was left or not done, in the words the person reads.

use std::collections::HashSet;
use std::sync::Arc;

use super::Which;
use crate::t;

pub fn not_looked() -> &'static str {
    t!("kit-work-not-looked")
}

pub fn too_big_to_check() -> &'static str {
    t!("kit-work-too-big")
}

pub fn too_big_known() -> Arc<HashSet<Which>> {
    Arc::new(HashSet::from([Which::TOO_BIG]))
}

pub fn not_there_why(known: Option<&HashSet<Which>>) -> &'static str {
    if known.is_some_and(|k| k.contains(&Which::TOO_BIG)) { too_big_to_check() } else { not_looked() }
}
pub fn changed_since() -> &'static str {
    t!("kit-work-changed-since")
}
pub fn no_safe_restore() -> &'static str {
    t!("kit-work-drive-no-safe-restore")
}
pub fn taken_meanwhile() -> &'static str {
    t!("kit-work-taken")
}
pub fn no_trash() -> &'static str {
    t!("kit-work-no-trash")
}
pub fn stayed() -> &'static str {
    t!("kit-work-stayed")
}
pub fn not_kept() -> &'static str {
    t!("kit-work-not-kept")
}
pub fn changed_why() -> &'static str {
    t!("kit-files-changed")
}

pub fn trouble(name: &str, why: &str) -> String {
    t!("kit-work-trouble", name = name, why = why)
}

pub fn not_all() -> &'static str {
    t!("kit-work-not-all")
}
