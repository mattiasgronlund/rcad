//! A test binary carries the debug info a backtrace needs, and not the rest.
//!
//! Full debug info made every test binary more than twice its size, and each
//! worktree links about two hundred of them that the build cache cannot share,
//! because it caches compiled crates and not links. `[profile.dev]` asks for
//! line tables only. This file holds both halves of that: the binary carries no
//! variable locations, which only a debugger reads, and a backtrace still names
//! file and line.
//!
//! Variable locations are the test and not the size, because on a small binary
//! most of the debug info is the standard library's own and the setting barely
//! moves the ratio. On `fuses_that_must_agree` full debug info wrote 19 MB of
//! `.debug_loc`; line tables write none, the standard library's part included.
//!
//! Reads its own executable, so it is about the profile it was built in. A
//! release build carries no debug info at all and would fail the second half,
//! which is why the file is off there.
#![cfg(all(target_os = "linux", debug_assertions))]

use std::backtrace::Backtrace;

#[test]
fn a_test_binary_carries_no_variable_locations() {
    let binary = std::fs::read(std::env::current_exe().unwrap()).unwrap();
    let names = section_names(&binary);
    assert!(
        names.iter().any(|name| name == ".debug_line"),
        "no line tables at all: {names:?}"
    );
    assert!(
        !names
            .iter()
            .any(|name| name == ".debug_loc" || name == ".debug_loclists"),
        "variable locations are back: is `[profile.dev]` still `debug = \"line-tables-only\"`? {names:?}"
    );
}

#[test]
fn a_backtrace_still_names_this_file_and_line() {
    let trace = Backtrace::force_capture().to_string();
    assert!(
        trace.contains("a_test_binary_keeps_line_numbers_and_little_else.rs:"),
        "the backtrace lost its line numbers:\n{trace}"
    );
}

/// The section names of a 64-bit little-endian ELF file.
fn section_names(elf: &[u8]) -> Vec<String> {
    assert_eq!(&elf[..4], b"\x7fELF");
    assert_eq!(elf[4], 2, "not a 64-bit ELF file");
    assert_eq!(elf[5], 1, "not a little-endian ELF file");
    let header_table = u64_at(elf, 0x28);
    let header_size = u16_at(elf, 0x3A);
    let header_count = u16_at(elf, 0x3C);
    let names_index = u16_at(elf, 0x3E);
    let header = |i: usize| &elf[header_table + i * header_size..][..header_size];
    let names = &elf[u64_at(header(names_index), 0x18)..];
    (0..header_count)
        .map(|i| {
            let name = &names[u32_at(header(i), 0)..];
            let end = name.iter().position(|&b| b == 0).unwrap();
            String::from_utf8(name[..end].to_vec()).unwrap()
        })
        .collect()
}

fn u16_at(bytes: &[u8], at: usize) -> usize {
    usize::from(u16::from_le_bytes(bytes[at..at + 2].try_into().unwrap()))
}

fn u32_at(bytes: &[u8], at: usize) -> usize {
    usize::try_from(u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap())).unwrap()
}

fn u64_at(bytes: &[u8], at: usize) -> usize {
    usize::try_from(u64::from_le_bytes(bytes[at..at + 8].try_into().unwrap())).unwrap()
}
