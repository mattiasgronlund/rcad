## 373. One test binary per crate, because the SSD is wearing out — 2026-09-30

*Mattias asked whether `target/` could live on a 64 GB RAM disk, then asked for
the row this section files. Every figure below was measured in that session, on
09-29 and 09-30. The builds ran on `main` at `82712c06`, exported with `git
archive` into the session's scratchpad, and restored from kache.*

In short: the SSD has used 16% of its rated life, and at this week's pace it
uses the rest in about seven months. Most of what a tree writes is test
binaries: 327 of them, 6.2 GB, and every tree writes its own copy. A RAM disk
builds no faster and holds eight trees where nineteen exist. Fewer test binaries
would cut the bytes where they come from, so **Q729** tries one binary for
rcad-topo's integration tests and measures what it costs.

### 373.1 The drive

udisks reports the drive's own wear counter without root:

```
busctl --system call org.freedesktop.UDisks2 \
    /org/freedesktop/UDisks2/drives/Force_MP600_193182290001285507F2 \
    org.freedesktop.UDisks2.NVMe.Controller SmartGetAttributes 'a{sv}' 0
```

On 09-30 it said `percent_used` 16, 141.5 TB written in all, over 47,134
power-on hours. That is about 8.8 TB of writes per 1% of life.

Field 7 of `/sys/block/nvme0n1/stat` counts sectors written since boot. It gave
13,454 GB in 4 days 5.6 hours, which is about 3.2 TB a day. At 8.8 TB per 1%,
the 84% left lasts 215–235 days; the spread is `percent_used` being a whole
number. 100% is the maker's estimate of the drive's life, not the day it stops.

Who wrote it, as far as could be told without root:

- **The kache daemon** wrote 1,629 GB in its 37 hours up (`write_bytes` in its
  `/proc/<pid>/io`). The daemon is what copies a fresh compile into the store.
- **A restore is written by the build itself.** One warm `cargo nextest run
  --workspace --no-run` into an empty directory on disk: the build's own
  processes wrote 6.5 GB (`/usr/bin/time -f %O`) for a 6.7 GB tree, while the
  daemon wrote 183 MB.

The rest of the 3.2 TB a day was not split by cause. It is most likely rustc and
the linker writing into `target/`, but that is not shown.

### 373.2 kache does cache test binaries; what it cannot do is share them

§203.1 says kache "does not cache a link, so every worktree links its own copy
of every test binary". Under kache 0.27 the first half no longer holds. `kache
list fuses_that_must_agree` shows 15 stored executables of 40.7 MiB each, with 78
hits between them and 3 never hit. The source agrees: on Linux executables are
cached by default (`default_cache_executables`, 0.27 `src/config.rs`), and a test
counts as one (`is_user_facing_executable`, `src/args.rs`). Whether §203.1 was
right under kache 0.16 on 09-22 was not checked.

What still holds is that no two trees share one. `restore_link_strategy`
(`src/wrapper.rs`) restores every executable as a private copy, apart from a
proc-macro's library and a build script's binary. The reason kache gives is that
a later `strip` could rewrite the file in place and so change the stored
original. No setting reaches that branch. `shared_hardlink_restores` lifts only
the rule that one tree at a time may link a blob (#794). The notes for 0.28.0
and 0.28.1 change neither.

So every tree writes all of its test binaries, whether it compiled them or
restored them. The `[profile.dev]` comment in `Cargo.toml` repeats §203.1's
reason, and Q729 rewrites it.

### 373.3 Where the bytes are

The same build with `--cargo-message-format json-render-diagnostics`, summing
every test-profile executable it reported:

| crate | test binaries | size | largest |
|---|---|---|---|
| rcad-topo | 172 | 3,793 MB | 43 MB |
| rcad-step | 45 | 1,045 MB | 44 MB |
| rcad-geom | 80 | 896 MB | 17 MB |
| rcad | 15 | 300 MB | 27 MB |
| rcad-predicates | 7 | 64 MB | 13 MB |
| rcad-mesh | 3 | 36 MB | 16 MB |
| rcad-fuzz-explain | 2 | 33 MB | 26 MB |
| rcad-fuzz-timing | 3 | 22 MB | 8 MB |
| all | 327 | 6,190 MB | |

Every file directly under a crate's `tests/` is a binary of its own, and each
one links the kernel again. rcad-topo's 172 average 22 MB each, and most of
that is the same code 172 times. How small one binary would be is Q729's first
number; nothing here measures it.

### 373.4 A RAM disk buys no speed and holds too little

A warm restore of the whole workspace's test binaries, alternating between
`/dev/shm` (a 63 GB tmpfs) and the disk, each into an empty directory, at load
70–80 from other sessions:

| | RAM | disk |
|---|---|---|
| restore, run 1 | 16.0 s | 13.2 s |
| restore, run 2 | 11.0 s | 12.6 s |
| deleting the 6.7 GB tree | 4.1 s, 1.6 s | 3.2 s, 1.8 s |

The two are the same within the noise. The kernel's page cache held about 109
GB at the time, so these writes were already landing in RAM first.

It also holds too little. On 09-29, 19 of the 49 worktrees held 164 GB of
`target/`, 75–90% of each tree test binaries. 64 GB holds about eight trees, and
a full tmpfs fails a build the same way a full `/` does (§315).

Cargo cannot send just the test binaries to RAM. `build.build-dir` (stable in
1.96) moves them, but it moves every rlib and all incremental state with them.
Tried on a scratch crate: with `CARGO_BUILD_BUILD_DIR` set, the target directory
kept two lock files and a `CACHEDIR.TAG`.

What a RAM disk would still do is take `target/`'s writes off the SSD. The
store's writes would stay, and hardlinks cannot reach across filesystems. It is
the fallback if Q729 finds the cost of fewer binaries too high.

### 373.5 What Q729 has to show

- rcad-topo's integration tests in one binary: `autotests = false` and one
  `[[test]]`, whose root declares each file as a module. The files stay where
  they are, so every path the record cites still resolves. The subdirectories
  that hold shared helpers, and `what_only_a_release_build_does` (the justfile
  runs it by name with `--test`), may need to stay binaries of their own. The
  lane decides that.
- A test in that binary that fails when a file in `tests/` is not declared.
  With `autotests` off, a new test file would otherwise never compile, and
  nothing would say so.
- The numbers: the size of rcad-topo's test binaries before and after; the full
  build from empty and from warm; the rebuild after editing one test file, which
  is the price, since the whole merged crate recompiles; the rebuild after
  editing a file in `crates/rcad-topo/src/`; and `cargo nextest list -p
  rcad-topo` running the same tests before and after.
- Anything that names one of these binaries has to keep working: `--test` and
  `binary(` filters in the justfile, hooks and workflows, and the record's own
  commands.

If the edit-one-test rebuild costs more than the relinks it saves, the lane
records the numbers and lands nothing else.

**§374 has since compressed every test binary's debug info**, so 373.3's 3,793 MB
is no longer rcad-topo's before: §374.6 counts 1,460 MB in 173 binaries, and
the relinks Q729 would save are that much smaller. The lane measures its own
before.

### 373.6 What rscene found doing the same — 2026-10-18

*rscene, a sibling repo, put each of its crates' integration tests into one
binary (its §80, at `a1bd0a7`) and sent what it found to this tree's dispatcher.
Its numbers are rscene's tree, not rcad's. The lane measures every number it
uses again, here.*

- **Dropping one `mod` line loses tests without a word.** With `autotests =
  false`, rscene deleted one `mod` line from the root: cargo built with exit 0
  and no warning, and seven tests left the list, 690 before and 683 after. That
  is the failure 373.5's check is for, and a lane editing the root can cause it
  by accident.
- **rscene put the check in a recipe, not in a test.** `check-test-binaries`,
  with `check-test-binaries-selftest`, reads `cargo metadata` and fails on a
  second test target in a crate, on `autotests` left on, on a `[[bin]]` without
  `test = false`, and on a file in `tests/` that the root does not declare. Only
  the last is what 373.5 asks for; the other three are what keeps one binary
  one. The recipe is in node, because rscene's jq is not pinned. Which of the
  two rcad builds is the lane's call.
- **`tests/it/main.rs` was turned down, though not for `git blame`.** Blame
  follows a moved file, so moving the files would not lose their history.
  rscene kept them where they are anyway, with `autotests = false` and one
  `[[test]]`, because with `autotests` on, a new file dropped into `tests/`
  quietly becomes a binary of its own again.
- **Editing one test file writes the whole suite again.** In rscene that rebuild
  took between 2.3 and 6 s a crate, and wrote 20–45% more bytes than one old
  binary had. An edit under `src/` wrote 24 times less. This is the price 373.5's
  last paragraph weighs.
- **Build before and after from empty.** rscene's tree measured 727 MB after
  three incremental rebuilds and 703 MB built fresh, with the same code.
  Comparing an incremental build with a fresh one moves the answer by that much.
- **kache 0.27 caches a test binary and restores it as a private copy,** which is
  what 373.2 says. rscene read the same.

### 373.7 Who declares a new test file — 2026-10-19

*Q2344 asked for one of two roads. Either the lane that adds a file to
`crates/rcad-topo/tests/` adds its two lines to `tests/merged.rs`, or a build
script in rcad-topo writes the list of modules and no lane edits `merged.rs`.
Everything below was measured in Q2344's lane trees, off `ce88b698`.*

In short: the lane that adds a file declares it, in the file's place by name,
and its write set holds those two lines as one item of `merged.rs`. No build
script: one would rebuild rcad-topo's library, and every crate that uses it, at
each edit of a test.

**A build script costs what §373 set out to save.** A script that writes the
list has to run again when a file appears in `tests/`, so it has to watch the
directory. Cargo's `rerun-if-changed` on a directory watches every file in it.
In a scratch crate, editing one test file printed ``Dirty bsprobe ...: the file
`tests` has changed`` and compiled `src/lib.rs` again. A build script belongs
to its package, so when it runs again the package's library is built again, and
so is every crate above it.

In the real tree, `target/q2344-rebuild.sh` ran `/usr/bin/time -f 'elapsed %e
s, written %O blocks of 512 bytes' cargo nextest run --workspace --no-run`,
which is the build `just t` does. Each run came warm, after one edit to
`crates/rcad-topo/tests/a_ball_inscribed_in_a_cube.rs`: a new empty `#[test]`,
with a new name each time. The probe build script's whole body was
`println!("cargo::rerun-if-changed=tests");`. It writes nothing, so what it
costs is the rerun alone.

| | packages compiled | elapsed | written, in blocks of 512 bytes |
|---|---|---|---|
| no build script, run 1 | 1 | 31.19 s | 912,760 (0.47 GB) |
| no build script, run 2 | 1 | 31.55 s | 564,536 (0.29 GB) |
| probe build script, run 1 | 4 | 50.13 s | 4,409,072 (2.26 GB) |
| probe build script, run 2 | 4 | 43.20 s | 4,059,120 (2.08 GB) |

The four packages were rcad-topo, rcad-step, rcad and rcad-fuzz-explain. So
every edit to a test would write about 2.1–2.3 GB instead of 0.3–0.5 GB, the
writes this section exists to cut. Adding the script cost one build of 120.70 s
that wrote 8,214,928 blocks; the fresh tree's cold build had taken 64.28 s and
written 6,264,456.

Two other ways to write the list were not tried, so nothing here says what they
cost: a proc macro that reads `tests/`, and a helper crate with a build script
of its own, as a dev-dependency of rcad-topo.

**The row's first road, as written, would be refused.** It had the lane add its
two lines outside its write set. `files_outside` in
`tooling/verify-lane/verify_lane.py` lists every changed path that no entry of
the write set covers, with no exemption, and `unproven()` in
`.claude/workflows/lanes.js` refuses any branch that has one. Prep never widens
a write set; it answers `needs` (`.claude/agents/prep.md`, "The write set").
Nothing exempted `merged.rs` at `ce88b698`.

**So the two lines are inside the write set, as an item.** The lane that adds
`tests/<file>.rs` writes the two lines the guard prints, `#[path = "<file>.rs"]`
and `pub mod <module>;`, where the module is the file's name with each `-`
written `_`. Its write set names them `crates/rcad-topo/tests/merged.rs#<module>`.
No tool has to change for that:

- `entries_meet` in the `queue` recipe makes two entries on one path meet only
  when they name the same item or one names the whole file (424.8). Two rows
  adding different files do not hold each other back.
- `entry_holds` in `tooling/queue/queue.py` lets a hold on the whole file
  narrow to one of its items, so a lane taken on `merged.rs` narrows to its own
  entry at its first edit (424.7).
- `in_write_set` in `verify_lane.py` matches the part before `#`, so the
  verifier counts `merged.rs` inside. Whether the diff stays inside the item is
  the verifier's reading, as for every item entry.

A row that changes `merged.rs` for more than one file, as Q2346's fold does,
keeps the whole file.

**The list is held in order of name, so the branches seldom meet.**
`every_file_is_declared_as_a_path_and_a_pub_mod_in_order_of_its_name` holds
the pairs in byte order of file name, and
`every_file_in_tests_is_built_into_exactly_one_test_binary` prints the pair a
file is missing. A branch then adds its two lines between two that stay, so two
branches adding different files change different lines and git merges them.
Lines added at the end would make every two such branches meet. Only two files
that sort next to each other, with nothing declared between them, go in at one
place. `target/q2344-rows.sh` read the queue at `ce88b698`:

- the 232 declarations were already in byte order;
- 48 open rows each named a file directly under `crates/rcad-topo/tests/` that
  did not exist yet, and 3 of them named `merged.rs` too;
- 16 of the 48 files go in at a place another of them shares, at 6 places, one
  of which takes 6.

Only those can conflict, only when their lanes run at the same time, and then
the fix is to keep both pairs.

**What this leaves.**

- 45 of the 48 rows name no `merged.rs` entry. Prep stops each for it, and the
  dispatcher re-files it with the item (`dispatch-lanes`). A row teaches `queue
  add` to refuse such a row and prep to answer `needs` with the item, never the
  whole file.
- `just queue families` counts `merged.rs` as a shape's test file, so the rows
  that declare their files in it show as one family. At `ce88b698` it was 6:
  Q2344, Q2346, Q2353, Q2365, Q2369 and Q2386. Every row re-filed with its item
  adds one. They share a binary's root, not a shape, and prep's list of a row's
  family (450.3 rule 4) reads them the same way. A row teaches both to leave
  `merged.rs` out. Until then, the dispatcher traces no family on `merged.rs`,
  and a lane runs no row's shapes for sharing `merged.rs` alone.
