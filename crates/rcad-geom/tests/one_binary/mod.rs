//! The two checks that keep a crate's merged test binary whole, shared by
//! rcad-geom's `tests/merged.rs` and rcad-step's, which reaches it by `#[path]`.
//! Each root passes its own text and crate directory, since `include_str!` and
//! `env!` here would name this file's crate and not the caller's.
//! `docs/decisions.md` §373.

use std::collections::BTreeMap;
use std::path::Path;

/// With `autotests` off, cargo builds a file in `tests/` only where the root
/// or a `[[test]]` names it, and says nothing about a file nothing names: its
/// tests just never run. A file named twice runs its tests twice.
pub fn every_file_in_tests_is_built_into_exactly_one_test_binary(crate_dir: &Path, root: &str) {
    let manifest = std::fs::read_to_string(crate_dir.join("Cargo.toml")).unwrap();
    assert!(
        manifest
            .lines()
            .any(|line| line.trim() == "autotests = false"),
        "{}/Cargo.toml does not say `autotests = false`, so cargo builds every file \
         declared in tests/merged.rs a second time, as a binary of its own",
        crate_dir.display()
    );
    let mut times_built: BTreeMap<String, usize> = test_files_on_disk(&crate_dir.join("tests"))
        .into_iter()
        .map(|file| (file, 0))
        .collect();
    let built = test_targets_in(&manifest, crate_dir)
        .into_iter()
        .chain(files_declared_in(root));
    let mut not_in_tests = Vec::new();
    for file in built {
        match times_built.get_mut(&file) {
            Some(times) => *times += 1,
            None => not_in_tests.push(file),
        }
    }
    let never: Vec<&String> = times_built
        .iter()
        .filter(|(_, n)| **n == 0)
        .map(|(f, _)| f)
        .collect();
    let twice: Vec<&String> = times_built
        .iter()
        .filter(|(_, n)| **n > 1)
        .map(|(f, _)| f)
        .collect();
    let declarations: String = never
        .iter()
        .map(|file| declaration_of(file.strip_prefix("tests/").unwrap()))
        .collect();
    assert!(
        never.is_empty() && twice.is_empty() && not_in_tests.is_empty(),
        "built by no binary, so its tests never run: {never:?}\n\
         declare each in {}/tests/merged.rs, in its place by file name, as\n{declarations}\
         built more than once: {twice:?}\n\
         built but not a file directly under tests/: {not_in_tests:?}",
        crate_dir.display()
    );
}

/// A branch that adds a file puts its two lines between two that stay, so two
/// branches adding different files change different lines and merge without a
/// conflict, unless their files sort next to each other. Lines added at the
/// end would make every two such branches meet. `docs/decisions.md` §373.7.
pub fn every_file_is_declared_as_a_path_and_a_pub_mod_in_order_of_its_name(root: &str) {
    let lines: Vec<&str> = root.lines().collect();
    let mut previous: Option<&str> = None;
    for (at, line) in lines.iter().enumerate() {
        let Some(file) = test_file_on(line) else {
            continue;
        };
        let written = format!("{line}\n{}\n", lines.get(at + 1).unwrap_or(&""));
        assert_eq!(
            written,
            declaration_of(file),
            "line {} of tests/merged.rs is not followed by its `pub mod`",
            at + 1
        );
        if let Some(previous) = previous {
            assert!(
                previous < file,
                "{file} is declared after {previous}, on line {}; each file goes in its \
                 place by name",
                at + 1
            );
        }
        previous = Some(file);
    }
}

/// The files cargo would find in `tests/` with `autotests` on: each `*.rs`
/// directly under it, and each `main.rs` one directory down. A helper such as
/// `tests/support/mod.rs` is neither.
fn test_files_on_disk(tests_dir: &Path) -> Vec<String> {
    let mut files = Vec::new();
    for entry in std::fs::read_dir(tests_dir).unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_name().unwrap().to_str().unwrap().to_owned();
        if path.is_dir() {
            if path.join("main.rs").is_file() {
                files.push(format!("tests/{name}/main.rs"));
            }
        } else if name.ends_with(".rs") {
            files.push(format!("tests/{name}"));
        }
    }
    files
}

/// The file each `[[test]]` in the manifest builds: its `path`, or, as cargo
/// infers it, `tests/<name>.rs` or else `tests/<name>/main.rs`.
fn test_targets_in(manifest: &str, crate_dir: &Path) -> Vec<String> {
    let mut targets: Vec<(Option<String>, Option<String>)> = Vec::new();
    let mut in_a_test = false;
    for line in manifest.lines().map(str::trim) {
        if line.starts_with('[') {
            in_a_test = line == "[[test]]";
            if in_a_test {
                targets.push((None, None));
            }
        } else if in_a_test {
            let target = targets.last_mut().unwrap();
            if let Some(name) = quoted_value_of("name", line) {
                target.0 = Some(name);
            } else if let Some(path) = quoted_value_of("path", line) {
                target.1 = Some(path);
            }
        }
    }
    targets
        .into_iter()
        .map(|(name, path)| match (name, path) {
            (_, Some(path)) => path,
            (Some(name), None) if crate_dir.join(format!("tests/{name}.rs")).is_file() => {
                format!("tests/{name}.rs")
            }
            (Some(name), None) => format!("tests/{name}/main.rs"),
            (None, None) => panic!("a [[test]] in Cargo.toml has neither a name nor a path"),
        })
        .collect()
}

/// `value` from a manifest line `key = "value"`.
fn quoted_value_of(key: &str, line: &str) -> Option<String> {
    let rest = line
        .strip_prefix(key)?
        .trim_start()
        .strip_prefix('=')?
        .trim();
    Some(rest.strip_prefix('"')?.strip_suffix('"')?.to_owned())
}

/// The file each `#[path = "..."]` in a root names, as a path from the crate.
fn files_declared_in(root: &str) -> Vec<String> {
    root.lines()
        .filter_map(test_file_on)
        .map(|file| format!("tests/{file}"))
        .collect()
}

/// The file a root's line `#[path = "<file>"]` names, as a path from `tests/`.
/// A `#[path]` that leaves `tests/`, as rcad-step's root does to reach this
/// file, names a helper and not a test file.
fn test_file_on(line: &str) -> Option<&str> {
    let file = line
        .trim()
        .strip_prefix("#[path = \"")?
        .strip_suffix("\"]")?;
    (!file.starts_with("../")).then_some(file)
}

/// The two lines that declare `file`, a path from `tests/`. A `-` in the name
/// cannot be in a module's name, so it is a `_` there.
fn declaration_of(file: &str) -> String {
    let stem = file
        .strip_suffix("/main.rs")
        .or_else(|| file.strip_suffix(".rs"))
        .unwrap();
    let module = stem.replace('-', "_");
    ["#[path = \"", file, "\"]\npub mod ", &module, ";\n"].concat()
}
