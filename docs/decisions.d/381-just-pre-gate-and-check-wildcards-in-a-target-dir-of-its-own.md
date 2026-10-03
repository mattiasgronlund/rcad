## 381. just pre-gate, and check-wildcards in a target dir of its own — 2026-10-03

*Not a queue row. §380 had a lane run `lint`, then `check-wildcards`, then its
gate. Mattias asked what running those checks and the formatting check at once
would cost, then asked for a `just pre-gate` recipe, so that the list lives in
the justfile and a change to it reaches the next lane without a new `claude`
process. He also asked whether a second target directory changes disk wear.
This section records what was built, what was measured, and the answer. It
changes the `justfile` and `.claude/agents/lane.md`, and replaces §380's rule.*

In short: a lane now gates with `mise exec -- just pre-gate gate quiet`.
`pre-gate` runs `check-fmt`, `lint` and `check-wildcards` at once, from
`pre_gate_steps`, and the gate starts only when all three pass. `check-fmt` is
a new step split out of `lint`. `check-wildcards` runs its clippy in
`target/wildcards`, so it and `lint` no longer re-check every crate each time
they take turns. A lane gate now spends about 8 to 11 s in clippy, against
about 31 s under §380's rule; a gate with nothing in front of it about 10 s,
against about 23 s. The second directory writes about 430 MB less per lane
gate, about 1% of what the drive writes, and takes about 1.1 GB of disk per
worktree.

### 381.1 One target directory made the two clippy runs re-check every crate

Cargo counts clippy's arguments as part of what it caches. `lint` passes `-D
warnings` and `check-wildcards` passes `-W clippy::wildcard_enum_match_arm`, so
in one target directory each found the other's flags and re-checked all 8
crates of the workspace, whether the edit reached them or not. Run at once in
one directory, one of the two waits on the other's lock.

Each run below followed a one-word edit to line 1 of
`crates/rcad-topo/src/lib.rs`, which reaches 4 of the 8 crates, with clippy's
cache left holding `lint`'s flags, the way a gate leaves it.

| What ran | Time | Crates re-checked |
|---|---|---|
| §380's rule, `lint` then `check-wildcards` (§380.3's A3) | 15.6 s | 4, then 8 |
| both at once, one directory (P1) | 14.4 s | 8 and 8; `lint` waited on the lock |
| both at once, wildcard clippy in `target/wildcards` (P2) | 11.9 s | 4 and 4; no wait |
| `lint` alone, nothing changed (R1) | 3.1 s | 0 |
| wildcard clippy in its own directory, nothing changed (R1) | 0.3 s | 0 |
| wildcard clippy, first run in its own directory (K0) | 4.9 s | 114 units, all served by kache |

R1's `lint` still checked formatting then: `cargo fmt --check` was 2.3 s of its
3.1 s. R1's wildcard run printed the same 84 warning lines as a cold run, so a
crate clippy does not re-check still has its warnings replayed, and that is
what `check-wildcards` counts.

A trap for the next probe: cargo counts `-Dwarnings` and `-D warnings` as
different flags. The first probe run, P0, did not spell the flags exactly as
the recipes do, and measured a re-check the recipes never cause. The probe now
passes exactly the recipes' arguments.

`check-wildcards-selftest` holds the change. It reads `just --show
check-wildcards` and fails when the clippy line's `CARGO_TARGET_DIR` is empty
or `target`.

### 381.2 check-fmt is split out of lint

Mattias asked for three steps, not two. `lint` used to run `cargo fmt --check`
and `taplo fmt --check` before clippy, and a recipe stops at its first failing
line, so a misformatted file stopped `lint` before clippy said anything. The
two formatting checks are now a recipe of their own, `check-fmt`, and `lint` is
clippy alone, so `pre-gate` runs formatting beside clippy.

`check-fmt` is a step of `gate_steps` and of `gate_reusable_steps`, since it
reads only the tree. It is in `gate_metered_steps` too: it builds nothing, but
`cargo fmt` needs the pinned toolchain, which the docs arm does not install,
and a prose commit cannot change a `.rs` file. It is not on the gate's cargo
chain, because it builds nothing. `check-gate-runs-every-check` now counts 54:
the 52 before, plus `check-fmt` and `check-pre-gate-selftest`.

`lane.md` still has a lane run `just fmt` before each commit. Mattias asked
whether that was still needed now that `pre-gate` checks formatting. It is: the
gate has always checked formatting, inside `lint` before this section, and
never fixed it. Without `just fmt`, `pre-gate` catches the slip in seconds but
after the commit, so the fix lands as a rustfmt-only commit on top of the real
change, which is what Q574's tip was (§367.6, Q688). Running `pre-gate` before
each commit instead would not fix the formatting either, and would cost 6 to
9 s and about 860 MB of writes per commit (381.4, 381.5) for checks the gate
makes anyway. None of §380.1's 24 lane gates failed on formatting; the `just
fmt` rule was in force for all of them (`2fcf03d4`, 2026-09-29).

### 381.3 Every step runs to its end, and a failure prints 60 lines

**No kill.** `pre-gate` starts every step at once, waits for all of them, and
prints one `· <step> (<s> s)` line as each ends. A failure does not stop the
others:

- stopping them would save at most 5 to 10 s, and only on a failure;
- 2 of §380.1's 4 cheap failures (Q766, Q767) broke `lint` and
  `check-wildcards` both, and a kill would have hidden the second until the
  next call, which costs another run and another model turn;
- it would kill cargo and kache mid-write.

It copies the gate's `wait -n -p` loop and keys its arrays by index rather than
by name, because a fixture list can name a step twice. Its logs go in a
`mktemp -d`, so a run nested in `check-pre-gate-selftest` inside a gate cannot
write over a real run's logs. It exits with the code of the first failing step
in list order.

**60 lines, not the whole output.** The first version printed each failing
step whole. With `mod  build;` written into the middle of `rcad-topo`'s `//!`
block, it printed 4,595 lines: `check-fmt` 1,136, `lint` 3,445 and
`check-wildcards` 9. A failing step now prints its first 60 lines, in list
order, with cargo's `Compiling`, `Checking`, `Fresh`, `Blocking` and `Finished`
lines left out of the 60, then `... N more lines in <log>`. The log directory
is kept only when something was cut. It is the same rule as `gate quiet`'s 60
lines (§147), except that it keeps the head rather than the tail, because the
first compile error is the one to fix.

The demo after the fix: `mod  build;` written after line 92 of
`crates/rcad-topo/src/lib.rs`, then `just pre-gate gate quiet`. It exited 1
after 90 lines. The steps ended `check-fmt` at 2.5 s (exit 1),
`check-wildcards` at 3.1 s (exit 1) and `lint` at 3.2 s (exit 101, E0428 among
others), and their output was printed in list order. `lint`'s was cut with
"2242 more lines". The gate never started.

### 381.4 What a second target directory writes

Mattias asked whether a second target directory changes disk wear. He supposed
not: in one directory a flag switch rewrites the same files, which writes as
much as a second directory would.

**After an edit he was right.** Both layouts write the same, about 430 MB per
clippy run. **With nothing changed they differ.** A flag switch in the shared
directory writes about 430 MB; the separate directory writes 0.

§380's rule made one such switch per lane gate, the gate's `lint`: in §380.3's
A3 the gate's `check-wildcards` re-checked 0 crates and its `lint` 8. So it
cost three clippy runs that compile, about 1.3 GB. `pre-gate gate quiet` costs
two, about 860 MB. That is about 430 MB less per lane gate, about a third. At
§380.1's 24 lane gates in 392 minutes it comes to about 10 GB, against the
about 870 GB the drive writes in 392 minutes (3.2 TB a day, §373.1). **About
1%, small either way.**

Bytes are the kernel's count for the run's process tree (`/usr/bin/time -f %O`,
times 512), plus the kache daemon's `write_bytes` from `/proc/<pid>/io`. The
daemon's share was at most 0.2 MB in every run but Q0's first (94 MB). Each
edit appended a word never compiled before to line 1 of
`crates/rcad-topo/src/lib.rs` (`zqe1x1105` to `zqg3x1121`), and E1 to E4
alternate the layouts.

| Run | Layout | Before it | Wildcard clippy | `lint` | Total |
|---|---|---|---|---|---|
| E1 | one directory | edit | 429.2 MB, 8 crates, 7.1 s | 431.9 MB, 8, 7.0 s | 861 MB |
| E2 | two | edit | 448.8 MB, 4, 7.3 s | 412.8 MB, 4, 3.6 s | 862 MB |
| E3 | one | edit | 432.2 MB, 8, 6.5 s | 433.1 MB, 8, 6.5 s | 865 MB |
| E4 | two | edit | 441.0 MB, 4, 3.7 s | 412.8 MB, 4, 3.5 s | 854 MB |
| N3 | two | edit (a `git restore`) | 447.9 MB, 4, 4.6 s | 449.4 MB, 4, 4.8 s | 897 MB |
| Q0 | one | flag switch only | — | 434.1 MB, 8, 7.2 s | — |
| N4 | two | nothing | 0.1 MB, 0, 0.3 s | 0.0 MB, 0, 0.3 s | 0.1 MB |

Q0's `lint` is "flag switch only" because the wildcard run just before it had
compiled the same source in the same directory.

**Why the bytes are equal after an edit.** From `kache report --format json`,
split per run by marker times:

- In every E run kache passed 222 to 244 compiles straight to rustc
  ("passthrough", reason "adaptive active"): the test targets of the 4 crates
  the edit reaches. For those it turns on incremental compilation in a folder
  of its own, `<target>/debug/incremental.kache-auto`. In N3 that folder took
  435.8 MB of each directory's writes and `deps` 2.8 MB.
- The 4 crates the edit does not reach are what the shared directory re-checks
  on every switch: 99 compiles, all served from kache's store, 1.8 MB restored
  in all. Those extra re-checks cost time and almost no writes.
- With nothing edited, the shared directory still recompiles the roughly 222
  passed-through units, because their flags changed (Q0). The separate
  directory recompiles none (N4).

kache's setting for this is `cache.adaptive_incremental` (env
`KACHE_ADAPTIVE_INCREMENTAL`), left at its default.

**Two readings on the way here were wrong.**

- The first answer in chat was that two directories write less because each
  switch re-checks all 8 crates. That holds only for a switch with nothing
  changed, and the reason was wrong: the extra crates cost about 2 MB.
- An earlier pair of runs, O2 and N2, seemed to show two directories writing 4×
  more after an edit: 102 MB against 450 MB per run. kache's report shows why.
  In O1 and O2 kache compiled the edited crates' 245 test targets through its
  store (result "dup") and wrote no incremental files; in N2 it passed 244
  straight through. kache decided differently between those runs, and the
  layout was not the cause. E1 to E4 alternate the layouts, and kache passed
  the same units through in every one.

**Disk.** `target/wildcards` is 276 MB after a cold first run (K0 4.9 s; K1
6.8 s, 333.9 MB written, all served by kache). It grows to about 1.1 GB once an
edit has gone through it, when the 831 MB incremental folder arrives.
`clean-targets` deletes it with the rest of `target/`.

### 381.5 What pre-gate costs a gate

Each run followed a new word in the same line.

| Run | Before the gate | Gate | Gate's `check-wildcards` | Gate's `lint` | `check-ignored` | Load |
|---|---|---|---|---|---|---|
| G1 | `pre-gate` 9.3 s (`check-fmt` 3.1, `lint` 4.8, `check-wildcards` 9.2) | 133.5 s | 1.0 s, 0 crates | 0.3 s, 0 | 41.3 s | 2.9 |
| G2 | — | 102.9 s | 6.2 s, 4 crates | 3.7 s, 4 | 9.8 s | 10.6 |
| G3 | `pre-gate` 6.2 s (4.4, 5.3, 6.1) | 99.8 s | 1.0 s, 0 | 0.3 s, 0 | 12.8 s | 8.0 |

All three gates failed only on `check-decisions`, because this section did not
exist yet. G1's `check-ignored` at 41.3 s matches §380.3's A1 (41.0 s), which
had no `pre-gate`, so `pre-gate` is not what made it slow.

- **A lane gate** spends about 8 to 11 s in clippy: 6.2 to 9.3 s in
  `pre-gate`, then 1.3 s in the gate. Under §380's rule it was about 31 s (A3:
  15.6 s before the gate, then 6.0 and 9.8 s in it).
- **A gate with nothing in front of it**, a verifier's or a merger's, spends
  9.9 s in clippy (G2), against about 23 s in §380.3's B1 and B2.
- **On a passing gate `pre-gate` costs about nothing.** `pre-gate` then the
  gate took 106.0 s (G3); the gate alone took 102.9 s (G2).

### 381.6 Tests

- **`check-pre-gate-selftest`** is new. It runs `pre-gate` with `--set
  pre_gate_steps` over fixture lists, beside a fixture `_pre-gate-selftest-long`
  that prints three of cargo's `Compiling` lines, then 100 lines, and fails. It
  holds that a pass prints one line per step and none of the step's output;
  that two failures are both printed and named, in list order, and neither
  stops `list` beside them; that a long failure is cut at 60 lines with cargo's
  lines left out, and names a log that exists and holds the rest; that three
  two-second steps take under 5 s; that a failing `pre-gate` stops `just
  pre-gate list` before `list`; and that an empty list fails. On the real list
  it holds that every step is a step of the gate, and that `lint` does not
  check formatting. It is in `gate_steps` and `gate_reusable_steps`.
- **`check-wildcards-selftest`** has the arm 381.1 describes.
- Both failed before the change and pass after it.
- **12 mutations, each caught by its own sentence.** `target/pre-gate-mutations.sh`
  edits copies of the justfile, with `tooling/` linked beside each, and runs
  the selftest against each copy. The 12: stops after the first failure; prints
  the failures last-first; prints a passing step's output; exits 0 on a
  failure; runs the steps in turn (behind `flock`); lets an empty list pass;
  `lint` checks formatting again; prints the whole output; keeps cargo's
  progress lines; names no log; deletes the named log; names a step not in the
  gate. "Deletes the named log" was at first caught only by `grep`'s "No such
  file" error. The arm now checks that the log exists and says so.

### 381.7 What this leaves

- **`lane.md` reaches only new `claude` processes.** A change to
  `pre_gate_steps` reaches the next lane branched after it lands, which was the
  point of putting the list in the justfile.
- **§380.5's first point no longer holds:** the checks run in one call, so a
  lane that breaks two of them sees both. Its second still does: a lane gate
  that would be served still pays for `pre-gate`.
- **CI is not measured.** kache is off there, so the second target directory
  means the dependencies are checked twice.
- **`check-wildcards` stays on the gate's cargo chain.** It no longer shares
  `lint`'s lock, so it could come off. That needs an exception in
  `check-gate-runs-every-check`, and is neither built nor measured.
- **About 1.1 GB per worktree** for `target/wildcards` (381.4).
- **For §373:** after an edit, each clippy run writes about 430 MB, nearly all
  of it into kache's incremental folder. That is part of the rest of the 3.2 TB
  a day that §373.1 left unsplit. Whether `check` or the test build does the
  same was not measured.
