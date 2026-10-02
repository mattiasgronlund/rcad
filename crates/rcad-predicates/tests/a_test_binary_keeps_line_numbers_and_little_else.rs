//! A test binary carries the debug info a backtrace needs, not the rest, and
//! carries it compressed.
//!
//! Every worktree writes its own copy of every test binary, some 330 of them
//! (§373). `[profile.dev]` asks for line tables only, which halved them (§203),
//! and `.cargo/config.toml` has the linker compress what is left with zlib,
//! which takes two thirds off again (§374). This file holds all three: the
//! binary carries no variable locations, which only a debugger reads; its debug
//! sections are compressed with zlib; and a backtrace still names file and line.
//! The last is what zstd would break, because the standard library's backtrace
//! cannot read it.
//!
//! Variable locations are the test and not the size, because on a small binary
//! most of the debug info is the standard library's own and the setting barely
//! moves the ratio. On `fuses_that_must_agree` full debug info wrote 19 MB of
//! `.debug_loc`; line tables write none, the standard library's part included.
//!
//! Reads its own executable, so it is about the profile it was built in. A
//! release build carries no debug info at all and would fail every test here,
//! which is why the file is off there.
#![cfg(all(target_os = "linux", debug_assertions))]

use std::backtrace::Backtrace;

#[test]
fn a_test_binary_carries_no_variable_locations() {
    let binary = std::fs::read(std::env::current_exe().unwrap()).unwrap();
    let names: Vec<String> = sections(&binary)
        .into_iter()
        .map(|section| section.name)
        .collect();
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
fn the_debug_info_is_compressed_with_zlib() {
    let binary = std::fs::read(std::env::current_exe().unwrap()).unwrap();
    // `.debug_gdb_scripts` is loaded into memory when the program runs, so the
    // linker leaves it as it is.
    let debug: Vec<Section> = sections(&binary)
        .into_iter()
        .filter(|section| section.name.starts_with(".debug_") && section.flags & SHF_ALLOC == 0)
        .collect();
    assert!(
        debug.iter().any(|section| section.name == ".debug_info"),
        "no debug info at all: {debug:?}"
    );
    for section in &debug {
        assert!(
            section.flags & SHF_COMPRESSED != 0,
            "{} is not compressed: does `.cargo/config.toml` still link with `--compress-debug-sections=zlib`?",
            section.name
        );
        assert_eq!(
            u32_at(&binary, section.offset),
            ELFCOMPRESS_ZLIB,
            "{} is compressed with something other than zlib, and the standard library's backtrace reads only zlib",
            section.name
        );
    }
}

#[test]
fn a_backtrace_still_names_this_file_and_line() {
    let trace = Backtrace::force_capture().to_string();
    assert!(
        trace.contains("a_test_binary_keeps_line_numbers_and_little_else.rs:"),
        "the backtrace lost its line numbers:\n{trace}"
    );
}

const SHF_ALLOC: usize = 0x2;
const SHF_COMPRESSED: usize = 0x800;
/// The first field of the header a compressed section starts with.
const ELFCOMPRESS_ZLIB: usize = 1;

/// One entry of an ELF file's section header table.
#[derive(Debug)]
struct Section {
    name: String,
    flags: usize,
    /// Where the section's bytes start in the file.
    offset: usize,
}

/// The sections of a 64-bit little-endian ELF file.
fn sections(elf: &[u8]) -> Vec<Section> {
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
            let entry = header(i);
            let name = &names[u32_at(entry, 0)..];
            let end = name.iter().position(|&b| b == 0).unwrap();
            Section {
                name: String::from_utf8(name[..end].to_vec()).unwrap(),
                flags: u64_at(entry, 0x08),
                offset: u64_at(entry, 0x18),
            }
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
