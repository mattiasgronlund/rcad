## 203. Test binaries carry line tables and nothing more — 2026-09-22

*Mattias asked why `rcad-worktrees` was 146 GB and whether kache was set up
wrong. Every number below was measured here, on `7bd07cdb`.*

### 203.1 kache was fine; what filled the disk is what it cannot cache

`kache doctor` passed every check and the cache was hitting (47% of builds, 37%
by compile cost). A full worktree still held 8–12 GB of `target/`, nearly all of
it in `target/debug/deps`: about two hundred test binaries. kache caches a
compiled crate. It does not cache a link, so every worktree links its own copy
of every test binary, and nothing is shared between them.

*Corrected by §373.2: under kache 0.27 test binaries are cached, 15 copies of
`fuses_that_must_agree` with 78 hits between them. What still holds is that none
is shared: every executable is restored as a private copy.*

Most of each binary was debug info. `fuses_that_must_agree` was 93 MB, of which
87 MB were `.debug*` sections: 36 MB of `.debug_str`, 23 MB of `.debug_info`,
19 MB of `.debug_loc`. `[profile.test]` sets `opt-level` and nothing else, so it
inherited the dev default, which is full debug info.

### 203.2 `[profile.dev] debug = "line-tables-only"`

A backtrace needs line tables and function names, and `line-tables-only` keeps
both. What goes is variable and type info, which only a debugger reads. Anyone
who wants it back for a session types `CARGO_PROFILE_DEV_DEBUG=true`.
`profile.test` inherits the setting, so one line covers both.

`cargo nextest run --workspace --no-run` into an empty target directory:

| | before | after |
|---|---|---|
| `target/` | 8.9 GB | 4.2 GB |
| `target/debug/deps` | 8.6 GB | 4.0 GB |
| `fuses_that_must_agree` | 93 MB | 39 MB |
| its `.debug_loc` | 19.4 MB | none |

The saving is half, not the 94% that §203.1's ratio suggests. Line tables still
record every inlined function by name, and `.debug_str` (17 MB) and
`.debug_info` (8 MB) are that record. Compressing the debug sections at link
time would come next, and it has **not** been tried or measured.

The first build after the change took 3m55s against 31s before, because the
profile is part of kache's key and every crate missed once. A warm build has not
been timed since.

### 203.3 The test is the variable locations, not the size

`crates/rcad-predicates/tests/a_test_binary_keeps_line_numbers_and_little_else.rs`
reads its own executable. It fails if `.debug_loc` or `.debug_loclists` is
present, or if `.debug_line` is missing, and it checks that a captured backtrace
names its own file and line.

The size ratio was tried first and could not tell the two settings apart: on
that small binary debug info was 86% before and 84% after, because most of it
is the standard library's own. `.debug_loc` is all or nothing. Line tables
write none of it, the standard library's part included. With
`CARGO_PROFILE_DEV_DEBUG=true` the first test fails and the second passes.
Release builds have no debug info at all, so the file is compiled out there,
like `what_only_a_release_build_does` is compiled out of debug builds.

### 203.4 What was reclaimed the same day

`just clean-targets` swept 48 GB from the unlocked lane trees, and `just
prune-worktrees` removed 7 merged ones and kept 55. Most of those 55 are clean
workflow-lane trees on a detached HEAD, which §76 keeps no matter what.
`rscene`'s version of the recipe (`eed893f`) removes a clean, unlocked detached
tree whose HEAD is on `origin/main`, once its `logs/HEAD` is an hour old and no
process has its working directory inside it. Porting that is open.

After the sweep `kache gc` evicted nothing. The store is 139 GB on disk against
its 150 GiB cap: `kache stats` shows 186%, but it counts before deduplication.
