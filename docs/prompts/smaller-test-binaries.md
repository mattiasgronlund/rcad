# Make this repo's test binaries smaller, so builds wear the SSD less

Each Cargo build writes its test binaries from scratch, and each worktree or
clone writes its own copy. In rcad, a sibling repo on this machine, test
binaries were 75–90% of each `target/` directory. They were the main reason
the SSD was being written about 3.2 TB a day in a busy week. Two settings made
them about 60% smaller there, and backtraces still show file and line. Do the
same here, measure it, and add tests that guard it.

This repo's own rules for tests, commits and where decisions are written down
win over anything below.

## 1. Measure before

Build every test binary and add up their sizes. Keep the command, because you
will run it again afterwards:

```
cargo test --workspace --no-run --message-format=json \
  | jq -r 'select(.profile.test == true and .executable != null) | .executable' \
  | sort -u | xargs stat -c %s \
  | awk '{n++; s+=$1} END {print n" binaries, "s/1e6" MB"}'
```

If the repo runs tests through nextest, `cargo nextest list --workspace
--list-type binaries-only` lists the same binaries. Write down the largest
binary too, and run `readelf -S -W <it> | grep debug` to see how much of it is
debug info.

Also read each of these before you change anything:
- `[profile.dev]` and `[profile.test]` in the workspace `Cargo.toml`
- `.cargo/config.toml`, in the repo and in `~/.cargo/`
- every place that sets `RUSTFLAGS` or `CARGO_ENCODED_RUSTFLAGS`: the justfile
  or Makefile, scripts, CI workflows, and `[env]` in `mise.toml`

## 2. Keep only line tables

In the workspace `Cargo.toml`:

```toml
[profile.dev]
debug = "line-tables-only"
```

`[profile.test]` takes this from `[profile.dev]` unless it sets `debug` itself.
If it does, change it there too.

This keeps what a backtrace needs: function names, file and line. It drops
variable and type info, which only a debugger reads. Anyone who wants a
debugger session can set `CARGO_PROFILE_DEV_DEBUG=true` for that session.

In rcad this halved `target/`, from 8.9 GB to 4.2 GB.

## 3. Have the linker compress the debug info with zlib

In `.cargo/config.toml`:

```toml
[target.'cfg(target_os = "linux")']
rustflags = ["-C", "link-arg=-Wl,--compress-debug-sections=zlib"]
```

The linker compresses before it writes, so the full-size file never reaches
the disk. rcad checked this with `strace`: the output file is sized once, at
the compressed size, and then renamed into place.

In rcad, the biggest test binary went from 42.7 MB to 14.0 MB. All 328 test
binaries went from 6.2 GB to 2.3 GB. Each link took about 0.1 s longer.

**Use zlib, not zstd.** zstd compresses a little more, but the standard
library's backtrace cannot read it, so backtraces lose file and line. This was
checked on Rust 1.96.0.

**Traps in how Cargo picks up flags. Check each one in this repo:**

- Cargo takes extra rustc flags from exactly one place, and the first that is
  set wins:
  1. `CARGO_ENCODED_RUSTFLAGS`
  2. `RUSTFLAGS`
  3. every matching `[target.<triple>]` and `[target.'cfg(...)']` table,
     joined together
  4. `[build] rustflags`

  What that means for this change:
  - If the repo has `[build] rustflags`, adding the table above **silently
    drops them**. Move them into a `[target...]` table.
  - Anything that sets `RUSTFLAGS` (a script, CI, coverage, fuzzing) replaces
    the config's flags, so it links without compression. List every such
    place in your report. Fix the ones that run often, or add the link-arg to
    their flags.
- The linker has to accept the flag. rust-lld, lld, GNU ld and mold do. That
  is why the table is Linux-only: the macOS linker does not take it.
- A build cache such as kache or sccache keys on the profile and the flags.
  The first build after this change therefore misses the cache for every
  crate. In rcad that build took about 4 minutes, against 31 s before. It only
  happens once.

## 4. Guard it with tests

Add one integration test file that reads its own executable and checks three
things:

1. It has `.debug_line` but no `.debug_loc` or `.debug_loclists`. That means
   line-tables-only is still in force.
2. Every `.debug_*` section that is not loaded when the program runs (no
   `SHF_ALLOC` flag) has the `SHF_COMPRESSED` flag, and its compression header
   says zlib. `.debug_gdb_scripts` is loaded when the program runs and the
   linker leaves it alone, which is why the `SHF_ALLOC` filter is there.
3. A forced backtrace names this test file followed by `:`, so file and line
   are still there.

Make sure each check fails without the setting it guards:
- Run the file once with `RUSTFLAGS` set to the flags from before your change.
  Setting `RUSTFLAGS` replaces the config's flags, so the compression check
  should fail. It replaces every one of them, even when you set it to nothing:
  if a dependency needs a flag the config sets (inari needs
  `-C target-cpu=haswell` or later), `RUSTFLAGS=""` stops the build at that
  dependency's `compile_error!` before any test runs, which reads as nothing
  having gone red. Put the flags the dependency needs into `RUSTFLAGS` too.
- Run it once with `zstd` in place of `zlib`. The compression check and the
  backtrace check should both fail.
- Run it once with `debug = true`. The variable-locations check should fail.
- Run it once with `debug = "line-directives-only"`. Only the backtrace check
  should fail: `.debug_line` is still there, so it is the one witness for this
  side.
- Cargo merges every `.cargo/config.toml` in a directory above the one it runs
  in. If your worktrees sit under the hub, a mutation that deletes a `[target]`
  table in a worktree is masked by the hub's copy. `cargo -Zunstable-options
  config get target --show-origin`, on nightly, names every file a value came
  from.

rcad's version needs no dependencies. Copy it and change the file name in
`a_backtrace_still_names_this_file_and_line`, plus the comments, to fit this
repo. It is off in release builds because a release build carries no debug
info, so every check would fail there.

```rust
//! A test binary carries the debug info a backtrace needs, not the rest, and
//! carries it compressed with zlib.
//!
//! `[profile.dev]` asks for line tables only, and `.cargo/config.toml` has the
//! linker compress what is left with zlib. This file holds all three: the
//! binary carries no variable locations, which only a debugger reads; its debug
//! sections are compressed with zlib; and a backtrace still names file and line.
//! The last is what zstd would break, because the standard library's backtrace
//! cannot read it.
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
```

## 5. Measure after, and report

Run step 1's command again, into the same kind of target directory as before.
Report:
- the number of test binaries and their total MB, before and after
- the largest binary's size, before and after
- how long a warm rebuild takes
- every place that sets `RUSTFLAGS`, and what you did about each one
- the command behind every number

## Not worth doing (rcad measured or checked each one)

- `debug = "line-directives-only"`: backtraces lose file and line.
- `split-debuginfo = "unpacked"`: each test still writes its own `.dwo` files,
  34 MB in all against 42.7 MB, and a build cache may not store them.
- zstd compression: see step 3.
- `target/` on a RAM disk: no faster, because the page cache already keeps
  these writes in memory, and it holds too few trees.
- Hardlinks through kache: kache restores every executable as a private copy,
  so it cannot share test binaries between trees on ext4.

## Optional, only if the repo has many integration test files

Every file in `tests/` becomes its own binary, with the whole crate linked
into it again. Putting them all in one binary cuts the count:
`autotests = false`, and one `[[test]]` whose root file declares each test
file as a `mod`, with the files left where they are. The cost is that editing
one test file then recompiles all of them.

rcad has not measured this yet. It is queued there as its row Q729. So do not
land it on reasoning alone. Measure, before and after:
- the total size of the test binaries
- a full build
- a rebuild after editing one test file

Land it only if it pays. If you do land it, add a test that fails when a file
in `tests/` is not declared. With `autotests = false` a new test file would
otherwise never compile, and nothing would report it.
