//! Builds the list of English word files (share/lang/en/*.ftl) into the program, so a new file needs no line of code.

fn main() {
    trindle_common::build_support::write_english_list();
}
