## 292. The timing crate built without coverage — 2026-09-27

*Queue row Q161, second run, from §267.6: build `rcad-fuzz-timing` without
coverage instrumentation on the nights, through the rustc stand-in §267.4
found works. Check it in under `fuzz/`, set `RUSTC` in `fuzz-overnight`'s
build and in `fuzz-target`, answer §267.4's kache question or keep kache out
of fuzz builds, and fail the night loudly if the timing rlib it built has any
sancov reference. Done when §267.5's first table holds on the night's own
build.*

  292.1   The wiring
  292.2   Kache: kept out, not answered
  292.3   The guard
  292.4   The two selftest arms
  292.5   The rerun, on the night's own build
  292.6   What is left

### 292.1 The wiring

`fuzz/rustc-stand-in.sh` (from the first run of this row, branch
`lane-q161-timing-without-coverage`, tip `f836432c`) is now called: both
`fuzz-overnight`'s `cargo +nightly fuzz build` and `fuzz-target`'s
`cargo +nightly fuzz run` set `RUSTC="{{ justfile_directory() }}/fuzz/rustc-stand-in.sh"`,
an absolute path since the build runs from inside a `cd fuzz` subshell.
`fuzz-target` needs it too, checked directly rather than assumed:
`fuzz-overnight` builds once and every `cargo fuzz run` after it is a
fingerprint no-op, but a hand `just fuzz` or `just fuzz-target` with no prior
build does the real build itself, through this same recipe.

Confirmed by grep that these are the only two places that build a fuzz
target: `grep -n 'cargo +nightly fuzz\|cargo fuzz' justfile` finds no third
call site. `check-fuzz-builds` uses stable `cargo check --all-targets`, which
type-checks but never links, so it needs neither `RUSTC` nor the guard below.

### 292.2 Kache: kept out, not answered

§267.4 left open whether kache, wrapped back around the stand-in by mise's
`cargo` shim unless `CI` is set, would in fact hand back an instrumented rlib
cached from a plain build. That question is not answered here either — kache
is kept out with `CI=1` on both call sites, same as the first run, rather than
resolved. The reasoning stands as written: kache's cache key is computed
before the stand-in appends its flag, so a hit could return a build made
before the stand-in existed, or with it broken, and there is no cheap way to
make the key depend on `RUSTC`'s own behaviour without changing kache itself.
Keeping kache out of exactly these two calls costs nothing else it does:
every other `cargo` and `just` call in this repo still goes through it
unchanged.

### 292.3 The guard

A stale build must not pass silently — cargo's fingerprint cannot see a flag
the stand-in adds on top of `RUSTFLAGS`, so a build made before the stand-in
existed, or made while it is broken, looks exactly as fresh as one made while
it works. `fuzz-overnight` now checks the artifact it just made rather than
trusting the exit code that made it.

`rcad-fuzz-timing`'s rlib does not land in `fuzz/target/.../release/deps/`,
where a `-L dependency=` search would look for it. It lands under
`build/<pkg>-<hash>/out/`, because that is the directory cargo passes as
`--out-dir` for a lib crate on this nightly (confirmed by watching the actual
`rustc` command line cargo runs for `rcad-topo`, which shows the same
`build/rcad-topo/<hash>/out` shape). The guard finds it by name:

    find fuzz/target/x86_64-unknown-linux-gnu/release/build/rcad-fuzz-timing \
        -name 'librcad_fuzz_timing-*.rlib'

then runs `nm` on it and counts lines matching `sancov`, case-insensitively.
Not found at all, or found with one or more references, both fail the night
the same way a build failure already does: the reason printed, `BUILD FAILED`
written to `{{fuzz_log_root}}/latest`, exit 1.

Measured on this lane's tip, built through the wired recipe
(`taskset -c 18-23`, `mise exec -- cargo +nightly fuzz build`, cold, no prior
`fuzz/target/`): the rlib
(`librcad_fuzz_timing-a0c1999e4697bad1.rlib`) has 0 lines matching `sancov`
under `nm`. Built the same way with `RUSTC` unset — the same tip, a separate
`CARGO_TARGET_DIR` so the two builds do not share a fingerprint — the same
crate's rlib has 47. §267.4 reported 43 or 45 on an earlier tip and a
different nightly; the number moves, the shape does not: zero with the
stand-in, several dozen without, on both nightlies tried.

### 292.4 The two selftest arms

`check-fuzz-overnight-selftest`'s stub `cargo` never ran a real build, so
`fuzz-overnight`'s existing fifteen arms exercised no real
`rcad-fuzz-timing` rlib at all — adding the guard above without changing the
stub would fail every one of them the first time it looked for a file the
stub never wrote. The stub's `build` case now writes one every time it is
called: a two-line `.s` file assembled and archived into an `.rlib`-shaped
`ar` archive under `target/x86_64-unknown-linux-gnu/release/build/rcad-fuzz-timing/selftest-<tag>/out/`,
with a global symbol named `clean_selftest_symbol` by default, or
`__sancov_selftest_symbol` when the arm sets `STUB_TIMING_SANCOV=1`. `nm`
reads this fixture exactly as it reads a real rlib — verified directly before
wiring it into the recipe — so it is not a mock of the guard's input, only of
the build that would normally produce it. The outer trap removes
`target/.../rcad-fuzz-timing/selftest-<tag>` alongside its other cleanup, so a
run leaves nothing behind in the real `fuzz/target/`.

**Arm Sixteen** sets `STUB_TIMING_SANCOV=1` and asserts the night fails with a
message naming a sancov reference, `BUILD FAILED` in the summary, and no
`fuzz run` line at all — the guard has to stop the night before any target is
started, not after. **Arm Seventeen** asserts the ordinary case, a clean
fixture, still passes: every arm before it already relies on this, and this
one says so directly. Reverting `fuzz-overnight` to its base (this row's own
check, since the base recipe has no guard to fail) makes Arm Sixteen's
`[ "$status" -ne 0 ]` see a 0, so the selftest fails — the arm is exercising
the added code and not passing on its own.

Run against the tree this row leaves: `mise exec -- just
check-fuzz-overnight-selftest` passes, printing nothing (the same silent-pass
shape as the recipe's other arms).

### 292.5 The rerun, on the night's own build

Not built by hand: `mise exec -- just fuzz-overnight`'s own build (`cd fuzz &&
CI=1 RUSTC=.../rustc-stand-in.sh ... cargo +nightly fuzz build`, `taskset -c
18-23`, cold) produced `fuzz/target/x86_64-unknown-linux-gnu/release/where_two_surfaces_cross`.
§267.2's own method was then run against that one binary — a fresh `cp -r` of
`fuzz/corpus/where_two_surfaces_cross` per run, `-seed=42 -runs=1500
-print_final_stats=1`, two runs with `RCAD_FUZZ_TIMINGS` unset and two with it
set to a fresh directory each — because running an already-built binary with
extra libFuzzer flags is not building it by hand; the build is still the
wired recipe's.

| | `RCAD_FUZZ_TIMINGS` | `INITED` at | `INITED` cov / ft / corp | ended at | final cov / ft / corp | new units |
|---|---|---|---|---|---|---|
| Run 1 | unset | `#835` | 3100 / 8771 / 503 | `#1500` | 3101 / 8835 / 519 | 16 |
| Run 2 | unset | `#835` | 3100 / 8771 / 503 | `#1500` | 3101 / 8835 / 519 | 16 |
| Run 1 | set | `#835` | 3100 / 8772 / 504 | `#1500` | 3102 / 8863 / 523 | 19 |
| Run 2 | set | `#835` | 3100 / 8772 / 504 | `#1500` | 3102 / 8863 / 523 | 19 |

This does not fully hold. Each pair agrees with itself exactly (unset with
unset, set with set, down to the same slowest-unit time in each pair's log),
which rules out a build that quietly forgot the seed or the corpus. But unset
and set no longer land on the same final numbers, as §267.5's table on tip
`9e72d414` did: 1 more `cov`, 28 more `ft`, 4 more `corp`, 3 more new units
with the variable set, on both runs of that arm alike. The gap is two orders
of magnitude smaller than the instrumented build's (517 `cov` there, in
267.5's second table), and `rcad-fuzz-timing`'s own rlib measures 0 sancov
references either way (292.3), so this is not the crate carrying its own
coverage again. It is closer to item 2 of §267.3: `timed`'s `OnceLock::get_or_init`
check for whether the watch exists compiles into the fuzz target itself,
which is instrumented regardless of what `rcad-fuzz-timing` carries, and the
branch actually taken differs between the two arms by construction. Whether
that is the whole of this gap, or the tip moved since 9e72d414 in some other
way, was not run down — the row's "done" test is 267.5's *first* table (the
stand-in build), and that one now shows a residual difference where it
previously showed none; this is reported rather than closed.

**Correction, 2026-10-08.** §305.1 finds that neither leg of this gap holds
and that nothing moved since `9e72d414`: "the split at `INITED` between 8771
and 8772 is not the variable and not `timed`: the **same binary**, run twice
with the same command and the same environment, lands on either" (305.1).
With address-space randomisation on, as every run in the table above had it,
each process draws one of the two values; §305.4 shows the split is a fixed
function of where the stack starts when randomisation is switched off,
flipping with every 16 bytes the stack is moved. §305.3 reran the unset and
set arms of a binary built the same way as this row's — one with the timing
crate not linked in at all (`nm | grep -c rcad_fuzz_timing` reads 0, §305.2),
so no reading of `timed`'s `OnceLock` check can be the cause — and got, across
five runs, `ft: 8771` then `3101/8858/521, 18 new` three times and `ft: 8772`
then `3101/8836/519, 16 new` twice; neither this row's unset line (`8771`,
then `3101/8835/519`, 16 new) nor its set line (`8772/504`, then
`3102/8863/523`, 19 new) came up there, nor in 20 more runs at fixed stack
positions (§305.3, §305.4). The exact numbers above are therefore not
reproduced, but §305.1's conclusion is: "So §292.5's 'does not fully hold' is
two draws from this lottery, not a residual cost of the timing crate"
(305.1). Where this row's own `corp: 504` draw came from is open (§305.7).
§305.7 names the methods that judge the variable properly instead of reading
a pair of single draws: the arms at fixed stack positions (`setarch -R`, with
the two environments padded to equal size), or several runs per arm with
randomisation on, compared by the spread of outcomes.

### 292.6 What is left

- **The residual difference in 292.5, unmeasured.** Whether it is `timed`'s
  own `OnceLock` check compiled into the target, something else in the crate
  boundary, or a divergence between tip `9e72d414` and this row's base that
  has nothing to do with `RCAD_FUZZ_TIMINGS`, was not run down. A next step:
  repeat 267.2's four-way probe with `RCAD_FUZZ_TIMINGS` compiled out
  entirely (a target built without the crate at all) against one built with
  it unset, to see whether the gap is already there before the variable is
  ever read.
- **§267.6's other four items are unchanged by this row** and are carried
  forward as they were: the merge cost inside a draw, whether the merge still
  reaches ASan's `memcmp`, the loss at a normal end, and the cost this row's
  own guard adds nothing to (it runs once, after the build, not once per
  child).
