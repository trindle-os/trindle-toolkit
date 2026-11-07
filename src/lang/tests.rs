//! Tests of the language files: how words are found, fall back and are checked.
use std::path::PathBuf;

use super::check::{check, domains_at, Domain};
use super::*;

const FILES: &str = "
files =
    { $n ->
        [one] { $n } file
       *[other] { $n } files
    }
";

const RUSSIAN: &str = "
files =
    { $n ->
        [one] { $n } файл
        [few] { $n } файла
        [many] { $n } файлов
       *[other] { $n } файла
    }
";

const POLISH: &str = "
files =
    { $n ->
        [one] { $n } plik
        [few] { $n } pliki
        [many] { $n } plików
       *[other] { $n } pliku
    }
";

fn say(lang: &str, text: &str, n: f64) -> String {
    let c = Catalogue::new(Bundle::from_text(lang, text), Bundle::from_text("en", FILES));
    c.with("files", &[("n", n.into())])
}

fn english_only(text: &str) -> Catalogue {
    Catalogue::new(Bundle::default(), Bundle::from_text("en", text))
}

#[test]
fn english_plurals_for_nought_one_two_and_five() {
    let words: Vec<String> = [0.0, 1.0, 2.0, 5.0].iter().map(|n| say("en", FILES, *n)).collect();
    assert_eq!(words, ["0 files", "1 file", "2 files", "5 files"]);
}

#[test]
fn russian_has_four_forms_and_the_teens_are_not_one_or_few() {
    let words: Vec<String> = [0.0, 1.0, 2.0, 5.0, 11.0, 21.0, 22.0, 25.0, 101.0, 112.0].iter().map(|n| say("ru", RUSSIAN, *n)).collect();
    assert_eq!(words, ["0 файлов", "1 файл", "2 файла", "5 файлов", "11 файлов", "21 файл", "22 файла", "25 файлов", "101 файл", "112 файлов"]);
    assert_eq!(say("ru", RUSSIAN, 1.5), "1.5 файла", "a fraction takes the other form");
}

#[test]
fn polish_has_its_own_rules_not_russian_ones() {
    let words: Vec<String> = [0.0, 1.0, 2.0, 5.0, 12.0, 22.0, 21.0].iter().map(|n| say("pl", POLISH, *n)).collect();
    assert_eq!(words, ["0 plików", "1 plik", "2 pliki", "5 plików", "12 plików", "22 pliki", "21 plików"]);
}

const ARABIC: &str = "
files =
    { $n ->
        [zero] لا ملفات
        [one] ملف واحد
        [two] ملفان
        [few] { $n } ملفات
        [many] { $n } ملفًا
       *[other] { $n } ملف
    }
";

#[test]
fn arabic_uses_all_six_forms() {
    let words: Vec<String> = [0.0, 1.0, 2.0, 3.0, 10.0, 11.0, 99.0, 100.0, 102.0, 103.0, 111.0, 2.5].iter().map(|n| say("ar", ARABIC, *n)).collect();
    assert_eq!(words, ["لا ملفات", "ملف واحد", "ملفان", "3 ملفات", "10 ملفات", "11 ملفًا", "99 ملفًا", "100 ملف", "102 ملف", "103 ملفات", "111 ملفًا", "2.5 ملف"]);
}

#[test]
fn other_languages_follow_their_rules() {
    assert_eq!((plural::category("mr", 0.0), plural::category("mr", 1.0), plural::category("hi", 0.0)), ("other", "one", "one"));
    assert_eq!(plural::category("ja", 1.0), "other");
    assert_eq!(plural::category("fr", 0.0), "one");
    assert_eq!(plural::category("fr", 2.0), "other");
    assert_eq!(plural::category("ar", 0.0), "zero");
    assert_eq!(plural::category("ar", 2.0), "two");
    assert_eq!(plural::category("ar", 7.0), "few");
    assert_eq!(plural::category("ar", 11.0), "many");
    assert_eq!(plural::category("cy", 3.0), "few");
    assert_eq!(plural::category("cs", 3.0), "few");
    assert_eq!(plural::category("pt-BR", 1.0), "one");
    assert_eq!(plural::category("xx", 1.0), "one", "a language not listed counts like English");
}

#[test]
fn an_exact_number_beats_the_category() {
    let text = "left =\n    { $n ->\n        [0] None left\n        [one] One left\n       *[other] { $n } left\n    }\n";
    let c = english_only(text);
    let said = |n: f64| c.with("left", &[("n", n.into())]);
    assert_eq!((said(0.0), said(1.0), said(4.0)), ("None left".to_string(), "One left".to_string(), "4 left".to_string()));
}

#[test]
fn a_missing_key_falls_back_to_english_per_message() {
    let chosen = Bundle::from_text("fr", "kit-one = Un\n");
    let english = Bundle::from_text("en", "kit-one = One\nkit-two = Two\n");
    let c = Catalogue::new(chosen, english);
    assert_eq!((c.plain("kit-one"), c.plain("kit-two")), ("Un", "Two"));
}

#[test]
fn a_key_found_nowhere_shows_nothing_and_never_the_key() {
    let c = english_only("a = A\n");
    assert_eq!(c.plain("kit-lang-test-nothing-here"), "");
    assert_eq!(c.with("kit-lang-test-nothing-here", &[]), "");
    assert!(unknown_keys().iter().any(|k| k == "kit-lang-test-nothing-here"), "listed for the tests");
}

#[test]
fn a_broken_file_never_breaks_the_program() {
    let bad = [
        "", "\n\n", "key", "key =", "= value", "ключ = значение", "k = { $", "k = { $n ->", "k = { $n -> [one] x }", "k = { $n -> *[a] x *[b] y }",
        "k = {", "k = { \"abc", "k = { { $x } }", "k = { -", "k = }", "  indented = no", "k = { $n ->\n  [one\n}", "k = { NUMBER( }",
        "k = a\n.attr = b", "\u{feff}k = ok", "k = \u{0}\u{1}\u{2}", "😀 = 😀", "k = { other }\nother = { k }",
    ];
    for text in bad {
        let b = Bundle::from_text("xx", text);
        let c = Catalogue::new(b, Bundle::from_text("en", "k = fine\n"));
        let _ = c.with("k", &[("n", 1.into())]);
        if !parse(text).messages.contains_key("k") {
            assert_eq!(c.plain("k"), "fine", "{text:?} gives English");
        }
    }
    for text in ["key", "= value", "k = {", "  indented = no", "k = a\n.attr = b"] {
        assert!(!parse(text).faults.is_empty(), "{text:?} is reported");
    }
    let closed = |depth: usize| format!("k = {}x{}\n", "{ $n -> *[other] ".repeat(depth), " }".repeat(depth));
    let fine = parse(&closed(3));
    assert!(fine.faults.is_empty() && fine.messages.len() == 1, "{:?}", fine.faults);
    let deep = parse(&closed(2000));
    assert!(deep.messages.is_empty(), "too deep is refused, not followed");
    assert!(deep.faults.iter().any(|f| format!("{f:?}").contains("too deep")), "{:?}", deep.faults);
    let mixed = parse("good = Good\nbroken = { $n\nalso-good = Also good\n");
    assert_eq!(mixed.order, ["good", "also-good"].map(String::from), "one broken message does not take the others");
    assert_eq!(mixed.faults.len(), 1);
}

#[test]
fn a_message_that_refers_to_itself_stops() {
    let c = english_only("a = A { b }\nb = B { a }\n");
    let said = c.with("a", &[]);
    assert_eq!(said, "A B ", "each message is written once on the way round");
}

#[test]
fn many_references_to_the_same_words_write_a_bounded_amount() {
    let mut text = String::from("m0 = x\n");
    for i in 1..12 {
        text.push_str(&format!("m{i} = {}\n", format!("{{ m{} }}", i - 1).repeat(4)));
    }
    let said = english_only(&text).with("m11", &[]);
    assert!(said.len() <= 2 * parse::MOST_WRITTEN, "wrote {} bytes", said.len());
}

#[test]
fn a_file_that_is_too_big_or_not_text_is_left_alone() {
    let dir = scratch("big");
    std::fs::write(dir.join("big.ftl"), "k = x\n".repeat(FILE_MOST as usize / 6 + 1)).unwrap();
    std::fs::write(dir.join("binary.ftl"), [0xff, 0xfe, 0x00, 0x80]).unwrap();
    std::fs::create_dir_all(dir.join("folder.ftl")).unwrap();
    for name in ["big", "binary", "folder", "absent"] {
        assert!(read_file(&dir.join(format!("{name}.ftl"))).is_none(), "{name}");
    }
    let b = load(&["big".into(), "binary".into(), "folder".into()], std::slice::from_ref(&dir));
    assert!(b.is_empty());
    let lang = scratch("big-folder");
    std::fs::create_dir_all(lang.join("xx")).unwrap();
    std::fs::write(lang.join("xx/big.ftl"), "k = x\n".repeat(FILE_MOST as usize / 6 + 1)).unwrap();
    std::fs::write(lang.join("xx/binary.ftl"), [0xff, 0xfe, 0x00, 0x80]).unwrap();
    std::fs::write(lang.join("xx/ok.ftl"), "fine = Fine\n").unwrap();
    let b = load(&["xx".into()], std::slice::from_ref(&lang));
    assert!(b.get("fine").is_some() && b.get("k").is_none(), "only the good file in the folder is read");
}

#[test]
fn the_size_limit_holds_exactly_at_the_edge() {
    let dir = scratch("edge");
    let line = "k = x\n";
    let fill = |n: u64| format!("{}{}", line.repeat(n as usize / line.len()), "#".repeat(n as usize % line.len()));
    std::fs::write(dir.join("at.ftl"), fill(FILE_MOST)).unwrap();
    std::fs::write(dir.join("over.ftl"), fill(FILE_MOST + 1)).unwrap();
    assert_eq!(read_file(&dir.join("at.ftl")).map(|t| t.len() as u64), Some(FILE_MOST));
    assert!(read_file(&dir.join("over.ftl")).is_none());
}

#[test]
fn variables_and_text_are_put_in_and_a_missing_one_is_empty() {
    let c = english_only("hello = Hello, { $name }. You have { $n }.\nquoted = { \"{\" }braces{ \"}\" }\nrefs = { -brand } { hello }\n-brand = Trindle\n");
    assert_eq!(c.with("hello", &[("name", "John Smith".into()), ("n", 2.5.into())]), "Hello, John Smith. You have 2.5.");
    assert_eq!(c.with("hello", &[]), "Hello, . You have .");
    assert_eq!(c.with("quoted", &[]), "{braces}");
    assert!(c.with("refs", &[("name", "x".into())]).starts_with("Trindle Hello, x"));
    assert_eq!(parse::number(3.0), "3");
    assert_eq!(parse::number(-7.0), "-7");
}

#[test]
fn comments_and_multiline_values_read_as_written() {
    let p = parse("# a comment\n## group\n### file\nlong =\n    First line\n    second line\nshort = Short\n");
    assert!(p.faults.is_empty(), "{:?}", p.faults);
    let c = english_only("long =\n    First line\n    second line\nshort = Short\n");
    assert_eq!(c.plain("long"), "First line\nsecond line");
    assert_eq!(c.plain("short"), "Short");
}

#[test]
fn a_message_with_values_asked_for_plainly_gives_english_in_another_language_and_nothing_when_english_needs_values_too() {
    let chosen = Bundle::from_text("fr", "kit-more-lines = { $n } lignes\nkit-x = { $n } x\n");
    let english = Bundle::from_text("en", "kit-more-lines = lines\nkit-x = { $n } x\n");
    let c = Catalogue::new(chosen, english);
    assert_eq!(c.plain("kit-more-lines"), "lines");
    assert_eq!(c.plain("kit-x"), "", "no values given and none in English: nothing, never the key");
}

#[test]
fn the_language_comes_from_the_person_then_lc_all_lc_messages_lang() {
    let s = |v: &str| Some(v.to_string());
    assert_eq!(choice_from(s("de"), s("fr"), s("es"), s("it")), ["de"]);
    assert_eq!(choice_from(None, s("fr_FR.UTF-8"), s("es"), s("it")), ["fr-FR", "fr"]);
    assert_eq!(choice_from(None, s(""), s("es_ES"), s("it")), ["es-ES", "es"]);
    assert_eq!(choice_from(s("auto"), None, None, s("it_IT.UTF-8@euro")), ["it-IT", "it"]);
    assert!(choice_from(None, None, None, s("C.UTF-8")).is_empty());
    assert!(choice_from(None, None, None, s("POSIX")).is_empty());
    assert!(choice_from(None, None, None, None).is_empty());
    assert_eq!(choice_from(None, None, None, s("zh_TW")), ["zh-TW", "zh-Hant", "zh"]);
    assert_eq!(choice_from(None, None, None, s("zh_CN.UTF-8")), ["zh-CN", "zh-Hans", "zh"]);
}

#[test]
fn a_language_name_cannot_reach_another_folder() {
    for bad in ["../../etc/passwd", "fr/../x", "a b", "/etc", "fr_../x", "12", "é"] {
        let tags = tags_of(bad);
        assert!(tags.iter().all(|t| !t.contains('/') && !t.contains("..")), "{bad:?} gave {tags:?}");
    }
    assert_eq!(tags_of("fr_CA.UTF-8"), ["fr-CA", "fr"], "a good tag still comes back");
    let (outer, secret) = (scratch("reach"), scratch("reach-secret"));
    let (lang, cwd) = (outer.join("lang"), outer.join("lang/en"));
    std::fs::create_dir_all(&cwd).unwrap();
    std::fs::write(secret.join("x.ftl"), "leak = leaked\n").unwrap();
    std::fs::create_dir_all(secret.join("en")).unwrap();
    std::fs::write(secret.join("en/x.ftl"), "leak = leaked\n").unwrap();
    let up = format!("../../{}/en", secret.file_name().unwrap().to_string_lossy());
    for tag in [up.as_str(), "..", "a/b", "/etc"] {
        assert!(load(&[tag.to_string()], std::slice::from_ref(&lang)).get("leak").is_none(), "{tag:?} reached outside the language folder");
    }
}

struct Scratch(PathBuf);

impl std::ops::Deref for Scratch {
    type Target = PathBuf;
    fn deref(&self) -> &PathBuf {
        &self.0
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn scratch(name: &str) -> Scratch {
    static COUNT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let n = COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("trindle-lang-test-{}-{n}-{name}", std::process::id()));
    std::fs::create_dir(&dir).unwrap();
    Scratch(dir)
}

#[test]
fn the_persons_own_folder_wins_and_the_exact_tag_beats_the_plain_one() {
    let (own, shared) = (scratch("own"), scratch("shared"));
    for d in [&own, &shared] {
        std::fs::create_dir_all(d.join("fr")).unwrap();
    }
    std::fs::create_dir_all(shared.join("fr-CA")).unwrap();
    std::fs::write(own.join("fr/own.ftl"), "a = A de moi\n").unwrap();
    std::fs::write(shared.join("fr/one.ftl"), "a = A partage\nb = B partage\n").unwrap();
    std::fs::write(shared.join("fr/two.ftl"), "c = C partage\n").unwrap();
    std::fs::write(shared.join("fr-CA/one.ftl"), "b = B canadien\n").unwrap();
    let b = load(&tags_of("fr_CA"), &[own.to_path_buf(), shared.to_path_buf()]);
    let c = Catalogue::new(b, Bundle::from_text("en", "a = A\nb = B\nc = C\nd = D\n"));
    assert_eq!((c.plain("a"), c.plain("b"), c.plain("c"), c.plain("d")), ("A de moi", "B canadien", "C partage", "D"));
}

#[test]
fn asking_for_plain_english_words_gives_the_same_built_in_text_every_time() {
    let c = english_only("a = Alpha\n");
    let (first, second) = (c.plain("a"), c.plain("a"));
    assert_eq!(first.as_ptr(), second.as_ptr(), "the same built-in text every time, so nothing is built per call");
}

#[test]
fn the_macro_gives_plain_words_or_a_string_with_values() {
    let c = Catalogue::new(Bundle::default(), Bundle::from_text("en", "plain = Plain\nwith = { $a } and { $b }\n"));
    with_catalogue(c, || {
        let plain: &'static str = crate::t!("plain");
        assert_eq!(plain, "Plain");
        assert_eq!(crate::t!("with", a = "one", b = 2), "one and 2");
    });
}

#[test]
fn the_toolkits_own_english_file_reads_cleanly() {
    let mut keys = 0;
    for (name, text) in KIT_ENGLISH {
        let p = parse(text);
        assert!(p.faults.is_empty(), "{name}: {:?}", p.faults);
        assert!(p.order.iter().all(|k| k.starts_with("kit-")), "{name}: every toolkit key starts kit-");
        assert!(!p.order.is_empty(), "{name} holds no words");
        keys += p.order.len();
    }
    assert!(keys > 0);
}

fn kit_domain() -> Domain {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    Domain {
        name: "toolkit".into(),
        lang_dir: root.join("share/lang"),
        sources: vec![root.join("src"), root.join("examples")],
        skip: vec!["lang/mod.rs".into(), "lang/check.rs".into(), "lang/tests.rs".into()],
    }
}

#[test]
fn every_toolkit_word_is_used_and_every_word_used_is_written() {
    let report = check(&[kit_domain()]);
    assert!(report.problems.is_empty(), "{}", report.text());
}

#[test]
fn the_check_finds_what_a_language_lacks_and_what_does_not_match() {
    let dir = scratch("check");
    let (lang, src) = (dir.join("lang"), dir.join("src"));
    std::fs::create_dir_all(lang.join("en")).unwrap();
    std::fs::create_dir_all(lang.join("fr")).unwrap();
    std::fs::create_dir_all(&src).unwrap();
    std::fs::write(lang.join("en/a.ftl"), "one = One\ntwo = Two { $n }\nthree = Three\nunused = Never used\n").unwrap();
    std::fs::write(lang.join("en/b.ftl"), "one = Again\n").unwrap();
    std::fs::write(lang.join("fr/a.ftl"), "one = Un\ntwo = Deux { $m }\nextra = En trop\nbroken = { $\n").unwrap();
    std::fs::write(lang.join("stray.ftl"), "z = Z\n").unwrap();
    std::fs::write(src.join("a.rs"), "fn f() {\n    t!(\"one\");\n    t!(\"two\", n = 3);\n    t!(\"three\", n = 1);\n    t!(\"nowhere\");\n    format!(\"no\");\n    // t!(\"commented\")\n    let _ = t!(\"three\");\n    lang::t_with(\"viapath\", &[]);\n    t_with(\"bare\", &[]);\n}\n").unwrap();
    let report = check(&[Domain { name: "x".into(), lang_dir: lang.clone(), sources: vec![src.clone()], skip: vec![] }]);
    let text = report.text();
    for expect in [
        "asks for nowhere, which no English file has",
        "asks for viapath, which no English file has",
        "asks for bare, which no English file has",
        "one is in both en/a.ftl and en/b.ftl",
        "stray.ftl is not in a language folder",
        "unused, which no code asks for",
        "gives [n] to three, which takes []",
        "fr/a.ftl has extra, which English does not",
        "fr/a.ftl two uses [$m] but English uses [$n]",
        "fr/a.ftl line 4",
    ] {
        assert!(text.contains(expect), "missing {expect:?} in\n{text}");
    }
    assert!(report.notes.iter().any(|n| n.contains("fr lacks 2 of 4 keys: three, unused")), "{text}");
    let all = report.missing_text(None);
    assert!(all.contains("fr: 2 of 4 words (50%)") && all.contains("fr lacks:\n  a.ftl: three\n  a.ftl: unused"), "{all}");
    assert!(!report.missing_text(Some("de")).contains("lacks:"));
    assert!(!text.contains("commented"), "a commented line is not a use");
    assert!(!text.contains("asks for no,"), "format! is not t!");
}

#[test]
fn a_long_run_of_blank_lines_is_read_in_one_pass() {
    let text = format!("a = x{}\n y\nb = z\n", "\n".repeat(300_000));
    let start = std::time::Instant::now();
    let p = parse(&text);
    assert_eq!(p.order, ["a", "b"]);
    assert!(start.elapsed() < std::time::Duration::from_secs(2));
    assert_eq!(super::parse::raw_messages(&text).len(), 2);
}

#[test]
fn the_check_reports_a_translation_that_would_not_be_shown() {
    let dir = scratch("mismatch");
    let (lang, src) = (dir.join("lang"), dir.join("src"));
    std::fs::create_dir_all(lang.join("en")).unwrap();
    std::fs::create_dir_all(lang.join("ru")).unwrap();
    std::fs::create_dir_all(&src).unwrap();
    std::fs::write(lang.join("en/a.ftl"), "files =\n    { $n ->\n        [one] { $n } file\n       *[other] { $n } files\n    }\n").unwrap();
    std::fs::write(lang.join("ru/a.ftl"), "files =\n    { $n ->\n        [one] { $n } fail\n       *[other] { $n } faily\n    }\n").unwrap();
    std::fs::write(src.join("a.rs"), "fn f() {\n    t!(\"files\", n = 1);\n}\n").unwrap();
    let report = check(&[Domain { name: "x".into(), lang_dir: lang.clone(), sources: vec![src.clone()], skip: vec![] }]);
    assert!(report.text().contains("ru/a.ftl files is not shown: its plural choice on n has no few line"), "{}", report.text());
}

#[test]
fn a_variant_names_its_chain_from_the_most_exact_folder_to_plain_english() {
    assert_eq!(tags_of("en-GB-oxendict"), ["en-GB-oxendict", "en-GB", "en"]);
    assert_eq!(tags_of("en_US"), ["en-US", "en"]);
    assert_eq!(tags_of("zh-Hans"), ["zh-Hans", "zh"]);
    assert_eq!(tags_of("zh-Hant-TW"), ["zh-Hant-TW", "zh-Hant", "zh"]);
    assert_eq!(tags_of("EN-gb-OXENDICT"), ["en-GB-oxendict", "en-GB", "en"]);
    assert!(tags_of("en-GB-../x").is_empty());
}

#[test]
fn a_variant_folder_holds_only_what_differs_then_falls_back_to_english_then_the_built_in_text() {
    let dir = scratch("variants");
    for d in ["en-GB-oxendict", "en-US", "en-GB", "en"] {
        std::fs::create_dir_all(dir.join(d)).unwrap();
    }
    std::fs::write(dir.join("en-GB-oxendict/a.ftl"), "organise = Organize, with the Oxford comma\n").unwrap();
    std::fs::write(dir.join("en-US/a.ftl"), "organise = Organize\ncolour = Color\n").unwrap();
    std::fs::write(dir.join("en/a.ftl"), "colour = Colour from a folder\n").unwrap();
    let english = || Bundle::from_text("en", "organise = Organise\ncolour = Colour\nplain = Plain\n");
    let say = |tag: &str| {
        let c = Catalogue::new(load(&tags_of(tag), std::slice::from_ref(&dir)), english());
        (c.plain("organise"), c.plain("colour"), c.plain("plain"))
    };
    assert_eq!(say("en-GB-oxendict"), ("Organize, with the Oxford comma", "Colour from a folder", "Plain"));
    assert_eq!(say("en-US"), ("Organize", "Color", "Plain"));
    assert_eq!(say("en-GB"), ("Organise", "Colour from a folder", "Plain"));
}

const ENGLISH_MIX: &str = "
hello = Hello { $name }, you have { $n } new
files =
    { $n ->
        [one] { $n } file
       *[other] { $n } files
    }
plain = Plain words
";

fn shown(lang: &str, text: &str, key: &str) -> String {
    let c = Catalogue::new(Bundle::from_text(lang, text), Bundle::from_text("en", ENGLISH_MIX));
    c.with(key, &[("name", "Ann".into()), ("n", 2.into())])
}

#[test]
fn a_translation_that_drops_a_variable_shows_english() {
    assert_eq!(shown("fr", "hello = Bonjour { $name }\n", "hello"), "Hello Ann, you have 2 new");
}

#[test]
fn a_translation_with_an_extra_variable_shows_english() {
    assert_eq!(shown("fr", "hello = Bonjour { $name } { $n } { $who }\n", "hello"), "Hello Ann, you have 2 new");
}

#[test]
fn a_translation_that_renames_a_variable_shows_english() {
    assert_eq!(shown("fr", "hello = Bonjour { $nom }, { $n }\n", "hello"), "Hello Ann, you have 2 new");
}

#[test]
fn a_translation_missing_a_plural_form_shows_english() {
    let no_few = "files =\n    { $n ->\n        [one] { $n } файл\n        [many] { $n } файлов\n       *[other] { $n } файла\n    }\n";
    assert_eq!(shown("ru", no_few, "files"), "2 files");
    let flat = "files = { $n } fichiers\n";
    assert_eq!(shown("fr", flat, "files"), "2 files");
}

#[test]
fn a_translation_of_a_named_selector_is_kept_in_a_language_with_several_plurals() {
    let english = "where =\n    { $kind ->\n        [drive] On the drive\n       *[part] On the part\n    }\n";
    let c = Catalogue::new(Bundle::from_text("ru", "where =\n    { $kind ->\n        [drive] На диске\n       *[part] На части\n    }\n"), Bundle::from_text("en", english));
    assert_eq!(c.with("where", &[("kind", "drive".into())]), "На диске");
}

#[test]
fn a_translation_that_does_not_parse_shows_english_and_the_rest_stay() {
    let text = "hello = Bonjour { $name\nplain = Simple\n";
    assert_eq!(shown("fr", text, "hello"), "Hello Ann, you have 2 new");
    assert_eq!(shown("fr", text, "plain"), "Simple");
}

#[test]
fn a_good_translation_is_untouched() {
    let ru = "files =\n    { $n ->\n        [one] { $n } файл\n        [few] { $n } файла\n        [many] { $n } файлов\n       *[other] { $n } файла\n    }\n";
    assert_eq!(shown("ru", ru, "files"), "2 файла");
    assert_eq!(shown("fr", "hello = Bonjour { $name }, { $n }\nplain = Simple\n", "hello"), "Bonjour Ann, 2");
    assert_eq!(shown("ja", "files = { $n } 個\n", "files"), "2 個", "a language with one form needs no selector");
    assert!(dropped_translations().iter().all(|d| !d.starts_with("ja: files")));
}

#[test]
fn a_dropped_translation_is_logged_once() {
    let text = "plain = { $x }\n";
    shown("qq-once", text, "plain");
    shown("qq-once", text, "plain");
    assert_eq!(dropped_translations().iter().filter(|d| d.starts_with("qq-once: plain")).count(), 1);
}

#[test]
fn messages_with_no_plural_choice_never_work_out_the_plural_rules() {
    let before = plural::CALLS.with(|c| c.get());
    let plain = (0..500).fold(String::new(), |mut s, i| {
        let _ = std::fmt::Write::write_fmt(&mut s, format_args!("m{i} = Plain {{ $n }}\n"));
        s
    });
    let c = Catalogue::new(Bundle::from_text("ru", &plain), Bundle::from_text("en", &plain));
    assert_eq!(c.chosen.len(), 500);
    assert_eq!(plural::CALLS.with(|c| c.get()), before, "the rules were worked out for messages that have no plural choice");
}

#[test]
fn the_list_of_left_out_translations_says_once_when_it_is_full() {
    let mut all = Vec::new();
    assert_eq!(admit(&mut all, "a", 2), Admit::Listed);
    assert_eq!(admit(&mut all, "a", 2), Admit::Known);
    assert_eq!(admit(&mut all, "b", 2), Admit::Listed);
    assert_eq!(admit(&mut all, "c", 2), Admit::FullNow);
    assert_eq!(admit(&mut all, "d", 2), Admit::Full);
    assert_eq!(admit(&mut all, "a", 2), Admit::Known);
}

#[test]
fn no_language_file_has_an_em_dash() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut seen = 0;
    for domain in domains_at(&root) {
        for lang in std::fs::read_dir(&domain.lang_dir).unwrap().flatten() {
            for (name, text) in folder_files(&lang.path()) {
                seen += 1;
                let at = text.lines().position(|l| l.contains('\u{2014}')).map(|n| n + 1);
                assert!(at.is_none(), "{}/{name} line {at:?} has an em dash", lang.path().display());
            }
        }
    }
    assert!(seen > 0, "no language files were found to look at");
}

#[test]
fn a_message_with_only_an_attribute_and_a_selector_starting_on_its_next_line() {
    assert!(!parse("a =\n    .x = y\n").faults.is_empty(), "an attribute is not text");
    let p = parse("n =\n    { $c ->\n        [one]\n            One thing\n       *[other] Things\n    }\n");
    assert!(p.faults.is_empty(), "{:?}", p.faults);
    let c = english_only("n =\n    { $c ->\n        [one]\n            One thing\n       *[other] Things\n    }\n");
    assert_eq!(c.with("n", &[("c", 1.into())]), "One thing");
}

#[test]
fn parsing_alone_leaks_no_plain_text_and_a_catalogue_keeps_its_own() {
    let p = parse("a = Plain\nb = With { $x }\n");
    assert!(p.messages["a"].plain.is_none() && p.messages["b"].plain.is_none());
    assert_eq!(english_only("a = Plain\n").plain("a"), "Plain");
}

#[test]
fn the_check_reads_a_call_over_several_lines_and_a_quote_inside_a_string() {
    let dir = scratch("wrapped");
    let (lang, src) = (dir.join("lang"), dir.join("src"));
    std::fs::create_dir_all(lang.join("en")).unwrap();
    std::fs::create_dir_all(&src).unwrap();
    std::fs::write(lang.join("en/a.ftl"), "two = { $a } and { $b }\n").unwrap();
    std::fs::write(src.join("a.rs"), "fn f() {\n    t!(\n        \"two\",\n        a = \"say \\\"x\\\", b = 1\",\n        b = 2,\n    );\n}\n").unwrap();
    let report = check(&[Domain { name: "x".into(), lang_dir: lang.clone(), sources: vec![src.clone()], skip: vec![] }]);
    assert!(report.problems.is_empty(), "{}", report.text());
}

#[test]
fn the_check_names_a_reference_to_nothing_and_a_circle() {
    let dir = scratch("refs");
    let (lang, src) = (dir.join("lang"), dir.join("src"));
    std::fs::create_dir_all(lang.join("en")).unwrap();
    std::fs::create_dir_all(&src).unwrap();
    std::fs::write(lang.join("en/a.ftl"), "a = A { b }\nb = B { a }\nc = C { nowhere }\n").unwrap();
    std::fs::write(src.join("a.rs"), "fn f() {\n    t!(\"a\");\n    t!(\"b\");\n    t!(\"c\");\n}\n").unwrap();
    let text = check(&[Domain { name: "x".into(), lang_dir: lang.clone(), sources: vec![src.clone()], skip: vec![] }]).text();
    assert!(text.contains("c names nowhere, which no English file has"), "{text}");
    assert!(text.contains("a names itself again: a names b names a"), "{text}");
}

#[test]
fn a_tag_with_a_variant_but_no_country_keeps_the_variant() {
    assert_eq!(tags_of("ca-valencia"), ["ca-valencia", "ca"]);
    assert_eq!(tags_of("de_1996"), ["de-1996", "de"]);
    assert_eq!(tags_of("es-419"), ["es-419", "es"]);
}

#[test]
fn a_file_that_grows_past_the_limit_is_not_read_whole() {
    let dir = scratch("limit");
    std::fs::write(dir.join("exact.ftl"), "k = x\n".repeat((FILE_MOST / 6) as usize)).unwrap();
    assert!(read_file(&dir.join("exact.ftl")).is_some());
    std::fs::write(dir.join("over.ftl"), "k".repeat(FILE_MOST as usize + 1)).unwrap();
    assert!(read_file(&dir.join("over.ftl")).is_none());
}

#[test]
fn a_catalogue_in_a_test_is_put_back_even_when_the_test_panics() {
    let before = t("kit-copy");
    let result = std::panic::catch_unwind(|| with_catalogue(english_only("kit-copy = Changed\n"), || panic!("fails")));
    assert!(result.is_err());
    assert_eq!(t("kit-copy"), before);
}

#[test]
fn a_plural_line_for_a_form_the_language_never_gives_is_a_problem() {
    let dir = scratch("plurals");
    let (lang, src) = (dir.join("lang"), dir.join("src"));
    std::fs::create_dir_all(lang.join("en")).unwrap();
    std::fs::create_dir_all(lang.join("ja")).unwrap();
    std::fs::create_dir_all(lang.join("ru")).unwrap();
    std::fs::create_dir_all(&src).unwrap();
    std::fs::write(lang.join("en/a.ftl"), "files =\n    { $n ->\n        [one] One file\n        [few] A few files\n       *[other] { $n } files\n    }\nstate =\n    { $s ->\n        [charging] Charging\n       *[other] On battery\n    }\n").unwrap();
    std::fs::write(lang.join("ja/a.ftl"), "files =\n    { $n ->\n        [one] 1\n       *[other] { $n }\n    }\n").unwrap();
    std::fs::write(lang.join("ru/a.ftl"), "files =\n    { $n ->\n        [one] 1\n        [few] 2\n        [many] 5\n       *[other] { $n }\n    }\n").unwrap();
    std::fs::write(src.join("a.rs"), "fn f() {\n    t!(\"files\", n = 1);\n    t!(\"state\", s = \"x\");\n}\n").unwrap();
    let text = check(&[Domain { name: "x".into(), lang_dir: lang.clone(), sources: vec![src.clone()], skip: vec![] }]).text();
    assert!(text.contains("en/a.ftl files is wrong: its plural choice on n has a few line, which en never uses"), "{text}");
    assert!(text.contains("ja/a.ftl files is wrong: its plural choice on n has a one line, which ja never uses"), "{text}");
    assert!(!text.contains("ru/a.ftl"), "{text}");
    assert!(!text.contains("state is wrong"), "a choice by name is not a plural: {text}");
    assert_eq!((plural::category("es", 1e6), plural::category("es", 1000.0), plural::category("fr", 2e6)), ("many", "other", "many"));
}

#[test]
fn portuguese_in_portugal_counts_nought_as_other_and_icelandic_fractions_as_one() {
    assert_eq!((plural::category("pt-BR", 0.0), plural::category("pt-PT", 0.0), plural::category("pt-PT", 1.0)), ("one", "other", "one"));
    assert_eq!((plural::category("PT", 0.0), plural::category("is", 0.5), plural::category("mk", 21.0), plural::category("ln", 1.0)), ("one", "one", "one", "one"));
}

const INVITED: &str = "
invited =
    { $gender ->
        [male] He invited you
        [female] She invited you
       *[other] They invited you
    }
";

fn invited(lang: &str, text: &str, gender: Option<Gender>, n: f64) -> String {
    let c = Catalogue::new(Bundle::from_text(lang, text), Bundle::from_text("en", INVITED));
    let mut args = vec![("n", n.into())];
    args.extend(gender.map(|g| ("gender", g.into())));
    c.with("invited", &args)
}

#[test]
fn a_gender_line_is_chosen_by_the_gender_given_and_other_is_the_default() {
    for (g, want) in [(Some(Gender::Male), "He invited you"), (Some(Gender::Female), "She invited you"), (Some(Gender::Other), "They invited you"), (None, "They invited you")] {
        assert_eq!(invited("en", INVITED, g, 1.0), want);
    }
    assert_eq!(Arg::from("female"), Arg::from(Gender::Female), "the words in a file are the same as the type's");
}

#[test]
fn a_gender_and_a_number_choose_together() {
    let nested = |one: &str, other: &str| format!("{{ $gender ->\n            [female] {one}\n           *[other] {other}\n        }}");
    let (single, many) = (nested("Она пригласила вас", "Он пригласил вас"), nested("Они (жен.) пригласили вас", "Они пригласили вас"));
    let both = format!("invited =\n    {{ $n ->\n        [one] {single}\n        [few] {many}\n        [many] {many}\n       *[other] {many}\n    }}\n");
    let english = "
invited =
    { $n ->
        [one] { $gender ->
            [female] She invited you
           *[other] He invited you
        }
       *[other] { $gender ->
            [female] They (women) invited you
           *[other] They invited you
        }
    }
";
    let c = Catalogue::new(Bundle::from_text("ru", &both), Bundle::from_text("en", english));
    let with = |g: Gender, n: f64| c.with("invited", &[("gender", g.into()), ("n", n.into())]);
    assert_eq!(with(Gender::Female, 1.0), "Она пригласила вас");
    assert_eq!(with(Gender::Male, 1.0), "Он пригласил вас");
    assert_eq!(with(Gender::Female, 1.5), "Они (жен.) пригласили вас");
    assert_eq!(with(Gender::Other, 1.5), "Они пригласили вас");
}

#[test]
fn a_language_with_no_gendered_words_may_leave_the_gender_choice_out() {
    assert_eq!(invited("zh-Hans", "invited = 他邀请了你\n", Some(Gender::Female), 1.0), "他邀请了你");
}

#[test]
fn a_language_may_choose_on_gender_where_english_does_not() {
    let english = "left = Left\n";
    let ru = "left =\n    { $gender ->\n        [female] Ушла\n       *[other] Ушёл\n    }\n";
    let c = Catalogue::new(Bundle::from_text("ru", ru), Bundle::from_text("en", english));
    assert_eq!(c.with("left", &[("gender", Gender::Female.into())]), "Ушла");
    assert_eq!(c.with("left", &[("gender", Gender::Male.into())]), "Ушёл");
}

#[test]
fn a_gender_line_set_with_no_other_line_shows_english() {
    let no_default = "invited =\n    { $gender ->\n        [male] Er hat dich eingeladen\n        [female] Sie hat dich eingeladen\n    }\n";
    assert_eq!(invited("de", no_default, Some(Gender::Male), 1.0), "He invited you");
}

#[test]
fn a_program_with_the_toolkit_fetched_elsewhere_still_finds_its_words() {
    let root = crate::common::source::toolkit_root().join("target").join(format!("no-toolkit-here-{}", std::process::id()));
    let far = root.join("program");
    std::fs::create_dir_all(far.join("share/lang/en")).unwrap();
    let domains = domains_at(&far);
    let _ = std::fs::remove_dir_all(&root);
    let names: Vec<&str> = domains.iter().map(|d| d.name.as_str()).collect();
    assert_eq!(names, ["toolkit", "trindle"]);
    assert_eq!(domains[0].lang_dir, crate::common::source::toolkit_root().join("share/lang"));
}
