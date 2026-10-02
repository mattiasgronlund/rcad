## 374. Test binaries link their debug info compressed with zlib — 2026-10-02

*Mattias asked how many test binaries the project builds and whether there
could be fewer, then whether less debug info had been tried, then whether the
compressed file is ever written uncompressed first, then said to land it. Every
figure below was measured in that session, on `main` at `0e9dafcc`, in target
directories in the session's scratchpad, except 374.6, which counts the gate's
own build of the change.*

In short: 84% of a test binary is debug info, even with §203's line tables only.
The linker can compress it with zlib before it writes the file, which takes
`fuses_that_must_agree` from 42.7 MB to 14.0 MB and costs about 0.1 s a link.
Backtraces keep their file and line. zstd would compress a little more and
loses them. Across the workspace a tree's 328 test binaries come to 2.3 GB
where 327 were 6.2 GB. This does not touch the number of binaries, which is
Q729's (§373); the two add up.

### 374.1 What a test binary is made of

`readelf -S -W` on `fuses_that_must_agree`, 42,730,432 bytes, built as the gate
builds it:

| section | MB |
|---|---|
| `.debug_str` | 18.5 |
| `.debug_info` | 9.0 |
| `.text` | 4.3 |
| `.debug_ranges` | 4.2 |
| `.debug_line` | 3.9 |
| everything else | 2.8 |

The `.debug_*` sections are 35.9 MB of it, 84%. The machine code is 10%.
§203.2 named what is left after line tables only: the name of every inlined
function, which is `.debug_str` and `.debug_info`. It also named compressing at
link time as the next step, untried.

### 374.2 Every way of shrinking it that was tried

| debug info | backtrace names file and line | written per test binary |
|---|---|---|
| line tables only (§203, today) | yes | 42.7 MB |
| `debug = "line-directives-only"` | **no** | not measured |
| line tables, `split-debuginfo = "unpacked"` | yes | 20.5 MB binary + 13.7 MB of its own `.dwo` files |
| line tables, compressed with zstd | **no** | 12.1 MB |
| **line tables, compressed with zlib** | **yes** | **14.0 MB** |

The families are: less debug info, debug info moved out of the binary, and debug
info compressed. Less than line tables loses file and line, which §203.2 already
said is what a backtrace needs; `line-directives-only` is the only setting
between line tables and nothing, and it lost them.

Whether a backtrace keeps file and line was checked on a three-line program that
panics, built by the pinned 1.96.0 with line tables only and `opt-level = 2`,
and run with `RUST_BACKTRACE=1`. For zstd it was also checked on the real test:
built with `--compress-debug-sections=zstd`,
`a_backtrace_still_names_this_file_and_line` fails. The zstd size is
`objcopy --compress-debug-sections=zstd` on the plain binary, not the linker.

Split debug info saves less than it looks. The `.dwo` files of the libraries
are shared by every test binary in a tree, but most of the inlined functions'
names are made inside the test's own code, so each test still writes 13.7 MB
of `.dwo` files of its own: 34 MB in all against 42.7. It would also need kache
to store and restore the `.dwo` files beside each crate, which was not checked.

### 374.3 The linker compresses before it writes

`strace -f` on the link, plain and compressed. `rust-lld` creates a temporary
file, sizes it once, writes it through a memory map, and renames it into place:

- plain: `ftruncate(3, 42668432)`, then the rename
- zlib: `ftruncate(3, 14043392)`, then the rename

So the binary is never on disk at its uncompressed size, and the copy kache
stores is the compressed one.

What is still written uncompressed is the linker's input. rustc writes the
test's own object files, the linker reads them, and they are deleted after the
link. Kept with `-C save-temps`, `fuses_that_must_agree`'s were 17 files and
26.1 MB. Whether files that live for about a second ever reach the SSD was not
measured. The disk is ext4 and `vm.dirty_expire_centisecs` is 3000, so they
usually would not, but the drive's write counter could not separate them from
other sessions' writes. Compressing them as rustc writes them needs a nightly
`-Z` option. The libraries' `.rlib` files are uncompressed too, written once per
crate per tree rather than once per test binary.

### 374.4 What it costs

Rebuilding `fuses_that_must_agree` after a real edit, a new `const` appended to
the test file in a copy of the tree so that kache misses, four times each: 0.54
to 0.57 s plain, 0.64 to 0.66 s compressed. About 0.1 s per link.

The first build after this change misses kache for every crate, as §203's did,
because the flags are part of kache's key.

### 374.5 Where it is set, and what it does not reach

`.cargo/config.toml`, in a table of its own under `cfg(target_os = "linux")`,
since it is about the linker and not the processor. Cargo joins the flags of
every table that matches. That the `x86_64` table still applies is shown by
`rcad-predicates` building at all: it depends on `inari`, which refuses to build
without `target-cpu=x86-64-v3` (§2.4).

`the_debug_info_is_compressed_with_zlib`, in
`crates/rcad-predicates/tests/a_test_binary_keeps_line_numbers_and_little_else.rs`,
reads its own executable and checks that every `.debug_*` section that is not
loaded at run time is compressed, and compressed with zlib.
`.debug_gdb_scripts` is loaded at run time, and the linker leaves it alone. With
the old flags (`RUSTFLAGS="-C target-cpu=x86-64-v3"`, which replaces the config's
flags exactly) it fails with `.debug_abbrev is not compressed`. With zstd it
fails, and so does `a_backtrace_still_names_this_file_and_line`.

Anything that sets `RUSTFLAGS` itself replaces the config's flags and so links
uncompressed. In the justfile that is the three fuzz builds, which are built
once in the fuzz tree and are left as they are. CI runs on Linux, so it links
compressed too and runs the test; a runner is thrown away, so there it saves no
wear.

### 374.6 The whole workspace

The gate's own build of `ae860d08`, counted with `cargo nextest list --workspace
--list-type binaries-only` and summed with `stat`, set against 373.3's count of
`82712c06` with the old flags:

| crate | binaries | before (373.3) | after | largest after |
|---|---|---|---|---|
| rcad-topo | 173 | 3,793 MB | 1,460 MB | 14.1 MB |
| rcad-step | 45 | 1,045 MB | 364 MB | 12.1 MB |
| rcad-geom | 80 | 896 MB | 335 MB | 6.0 MB |
| rcad | 15 | 300 MB | 106 MB | 9.6 MB |
| rcad-predicates | 7 | 64 MB | 25 MB | 4.8 MB |
| rcad-mesh | 3 | 36 MB | 13 MB | 5.6 MB |
| rcad-fuzz-explain | 2 | 33 MB | 14 MB | 10.9 MB |
| rcad-fuzz-timing | 3 | 22 MB | 9 MB | 3.2 MB |
| all | 328 | 6,190 MB | 2,326 MB | |

62% less. The two columns are two commits three days apart, and rcad-topo has
one test file more than it had; a before on `ae860d08` itself was not built,
because it would have written the 6 GB this section is about.

### 374.7 A broken cargo registry, found on the way

The first whole-workspace build for this section failed in `cc`:
`error[E0583]: file not found for module 'apple'`. On 09-30 between 08:00 and
09:00, something deleted every directory named `target` under
`~/.cargo/registry/src`, and `cc` keeps its sources in `src/target/`. All seven
extracted versions of `cc` lost it. `criterion` needs `alloca`, which builds with
`cc`, so every build of rcad's or rcad-geom's tests that kache could not serve
failed. What deleted them was not found; `clean-targets` walks git worktrees
only. Mattias deleted the extracted `cc` directories, and cargo unpacked them
again from the downloaded `.crate` files.
