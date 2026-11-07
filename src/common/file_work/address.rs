//! Paths written as addresses: the file:// form and the %xx escapes.

use std::path::{Path, PathBuf};

pub fn uri_path(uri: &str) -> Option<PathBuf> {
    let rest = uri.strip_prefix("file://")?;
    Some(unpercent(&rest[rest.find('/')?..]))
}

pub fn unpercent(text: &str) -> PathBuf {
    use std::os::unix::ffi::OsStringExt;
    let (mut out, b) = (Vec::new(), text.as_bytes());
    let mut i = 0;
    while i < b.len() {
        let hex = |c: u8| (c as char).to_digit(16);
        if b[i] == b'%' && i + 2 < b.len() {
            if let (Some(h), Some(l)) = (hex(b[i + 1]), hex(b[i + 2])) {
                out.push((h * 16 + l) as u8);
                i += 3;
                continue;
            }
        }
        out.push(b[i]);
        i += 1;
    }
    PathBuf::from(std::ffi::OsString::from_vec(out))
}

pub fn percent(path: &Path) -> String {
    use std::os::unix::ffi::OsStrExt;
    let mut out = String::new();
    for &c in path.as_os_str().as_bytes() {
        if c.is_ascii_alphanumeric() || b"/-._~".contains(&c) {
            out.push(c as char);
        } else {
            out.push_str(&format!("%{c:02X}"));
        }
    }
    out
}

pub fn file_uri(path: &Path) -> String {
    format!("file://{}", percent(path))
}
