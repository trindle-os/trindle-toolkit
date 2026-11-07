//! What a test needs of the computer it is run on, asked for in one way.

use std::path::{Path, PathBuf};

const PACKAGES: &[(&str, &str)] = &[("fc-list", "fontconfig"), ("fc-match", "fontconfig")];

pub fn find(name: &str) -> Option<PathBuf> {
    let mut places: Vec<PathBuf> = std::env::var_os("PATH").map(|p| std::env::split_paths(&p).collect()).unwrap_or_default();
    places.extend(["/usr/sbin", "/sbin"].map(PathBuf::from));
    places.into_iter().map(|d| d.join(name)).find(|p| p.is_file())
}

fn package_of(name: &str) -> &str {
    PACKAGES.iter().find(|(program, _)| *program == name).map_or(name, |(_, package)| package)
}

pub fn program(name: &str) -> PathBuf {
    find(name).unwrap_or_else(|| {
        let package = package_of(name);
        panic!("{name} is not on this computer, and this test checks nothing without it: install Debian's {package} (sudo apt install {package})")
    })
}

pub fn programs(names: &[&str]) {
    for name in names {
        program(name);
    }
}

pub fn file_with(path: &Path, what: &str, how: &str) -> PathBuf {
    assert!(path.is_file(), "{} is not on this computer, and this test checks nothing without {what}: {how}", path.display());
    path.to_path_buf()
}

pub fn file(path: &Path, what: &str) -> PathBuf {
    file_with(path, what, &format!("install {what}"))
}

pub fn folder(path: &Path, what: &str) -> PathBuf {
    assert!(path.is_dir(), "{} is not on this computer, and this test checks nothing without {what}: run it from a whole checkout of Trindle", path.display());
    path.to_path_buf()
}

#[cfg(test)]
mod tests {
use super::*;

#[test]
fn a_missing_program_fails_naming_it_and_its_package() {
    let said = std::panic::catch_unwind(|| program("trindle-test-not-a-program")).unwrap_err();
    let said = said.downcast_ref::<String>().cloned().unwrap_or_default();
    assert!(said.contains("trindle-test-not-a-program is not on this computer") && said.contains("apt install trindle-test-not-a-program"), "{said}");
    assert_eq!(package_of("fc-match"), "fontconfig");
    assert_eq!(package_of("other"), "other");
    assert!(find("sh").is_some());
}

#[test]
fn a_missing_file_fails_naming_it_and_how_to_have_it() {
    let said = std::panic::catch_unwind(|| file_with(Path::new("/nowhere/x.ttf"), "the X font", "install Debian's fonts-x")).unwrap_err();
    let said = said.downcast_ref::<String>().cloned().unwrap_or_default();
    assert!(said.contains("/nowhere/x.ttf is not on this computer") && said.contains("install Debian's fonts-x"), "{said}");
}
}
