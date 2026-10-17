## 305. Where the fuzz-timing gap opens — 2026-09-27

*Queue row Q236, second pass, from §292.5 and §292.6: on
`where_two_surfaces_cross` at `-seed=42 -runs=1500`, §292.5 had
`RCAD_FUZZ_TIMINGS` unset end at 3101 / 8835 / 519 and set at 3102 / 8863 /
523, split already at `INITED` (8771 / 503 against 8772 / 504), where §267.5
on `9e72d414` had both equal. Build the target with the timing crate compiled
out, pair it with the unset build, and say whether the gap is `timed`'s
generic `OnceLock` check (§267.3 item 2) or a change since `9e72d414`. The
first pass (branch `lane-q236-timing-compiled-out`) was rejected: its verifier
rebuilt its three arms and got three different results.*

  305.1   The answer
  305.2   The builds
  305.3   The arms, two runs each
  305.4   One binary, one command, two answers: where the stack starts
  305.5   The two legs of the hypothesis
  305.6   9e72d414, built the same way
  305.7   What is left
  305.8   Two more trees: the path moves the flip, and the seed pass holds still

### 305.1 The answer

Neither leg holds, and nothing moved since `9e72d414`. The split at `INITED`
between 8771 and 8772 is not the variable and not `timed`: the **same
binary**, run twice with the same command and the same environment, lands on
either. With address-space randomisation switched off it becomes a fixed
function of where the stack starts, flipping with every 16 bytes the stack is
moved (305.4). With randomisation on, as every run in §267.5, §292.5 and the
first pass had it, each process draws one of the two.

On this lane's build of the base, unset and set give identical lines in every
run, the same lines §267.5 recorded on `9e72d414`. `9e72d414` rebuilt today
gives those lines too. The compiled-out build is the one that draws 8771 at
some stack positions and 8772 at others, and its 8772 runs end on exactly the
unset and set lines of the base build.

So §292.5's "does not fully hold" is two draws from this lottery, not a
residual cost of the timing crate. Leg (a), the variable's own code reaching
something libFuzzer counts, and leg (b), `timed`'s instance carrying coverage,
are both broken in 305.5.

### 305.2 The builds

Every build ran `fuzz-overnight`'s own build line, with `taskset -c 16-19`
added, from `target/q236/build.sh` under `mise exec --`, in this worktree
(`dispatch-next-worktrees/wf_39298cae-c5a-1`), in the foreground:

    cd fuzz && CI=1 RUSTC="<worktree>/fuzz/rustc-stand-in.sh" RUSTFLAGS="-C target-cpu=x86-64-v3" taskset -c 16-19 cargo +nightly fuzz build

`cargo` there is mise's shim (`/home/mattias/.local/share/mise/shims/cargo`),
so `CI=1` leaves `RUSTC_WRAPPER` empty as §292.2 intends. The compiler is
`rustc 1.100.0-nightly (0dfb098f3 2026-08-31)`. Its toolchain directory and
`~/.rustup/update-hashes/nightly-x86_64-unknown-linux-gnu` are both dated
2026-09-01, so §267 ran on this compiler too. Each build log records the tree's
`git log -1`, its `git diff`, the stand-in's sha256, and `env | grep RCAD_`,
which was empty for every build.

| Arm | Tree | Log | Timing rlib `sancov` | `rcad_fuzz_timing` symbols | `__sancov_cntrs` | Binary sha256 |
|---|---|---|---|---|---|---|
| present | `f85f926d`, clean | `build-present.log` (cold, 75 s) | 0 | 55 | `0x43aa` = 17322 | `730e1ce4…` |
| compiled-out | `f85f926d` plus the edit below | `build-compiled-out.log` | 0 | 0 | `0x43aa` = 17322 | `96534347…` |
| at-9e72d414 | `9e72d414`, stand-in copied in untracked | `build-at-9e72d414.log` | 0 | 48 | `0x43aa` = 17322 | `727fcc5e…` |

The **compiled-out edit** touched only
`fuzz/fuzz_targets/where_two_surfaces_cross.rs`. It dropped `use
rcad_fuzz_timing::timed;` and made the body bare, with no wrapping closure:

    fuzz_target!(|bytes: &[u8]| {
        if let Some(pair) = two_surfaces_from(bytes) {
            let _ = every_oracle_holds_of(pair);
        }
    });

The crate stays a dependency of the package for the other four targets, and it
is not linked into this binary: `nm | grep -c rcad_fuzz_timing` reads 0. The
edit was put back with `Edit` straight after the build. `git status --short`
then printed nothing, and the edit is not in this commit.

**`9e72d414` was built in this same worktree**, detached there with `git
switch --detach` and switched back afterwards. That keeps the tree's path the
same, and the path is compiled into the binary. The commit predates the
stand-in (`f836432c`), so `fuzz/rustc-stand-in.sh` was copied in as an
untracked file: sha256 `86834457…`, the same file the other two builds used.

Every log named here is under `target/q236/` in this worktree, which is
gitignored. The runs' corpus copies are under `target/q236/runs/<label>/`.

**Those files are gone.** `target/q236/` was deleted on 2026-09-30 while the
tree was locked, and the tree was removed on 2026-10-09 (§420). Every script
named in this section has to be written again from its text here, and every
binary rebuilt.

### 305.3 The arms, two runs each

Each run used §267.2's method through `target/q236/run.sh`: a fresh `cp -r` of
`fuzz/corpus/where_two_surfaces_cross` (834 files), then `taskset -c 16-19
<binary> corpus -seed=42 -runs=1500 -print_final_stats=1` in a fresh
directory. It ran under an explicit `env -u RCAD_FUZZ_TIMINGS` or `env
RCAD_FUZZ_TIMINGS=target/q236/timings/<label>`, and its log opens with `env |
grep RCAD_`. Runs were one at a time, not four at once, with address-space
randomisation on (`/proc/sys/kernel/randomize_va_space` = 2). The lines below
are verbatim from the named logs:

    run-present-off-1.log       #835  INITED cov: 3100 ft: 8772 corp: 503/26Kb exec/s: 43 rss: 321Mb
                                #1500 DONE   cov: 3101 ft: 8836 corp: 519/27Kb lim: 77 exec/s: 78 rss: 337Mb   (16 new)
    run-present-off-2.log       #835  INITED cov: 3100 ft: 8772 corp: 503/26Kb exec/s: 49 rss: 321Mb
                                #1500 DONE   cov: 3101 ft: 8836 corp: 519/27Kb lim: 77 exec/s: 88 rss: 337Mb   (16 new)
    run-present-on-1.log        #835  INITED cov: 3100 ft: 8772 corp: 503/26Kb exec/s: 46 rss: 321Mb
                                #1500 DONE   cov: 3101 ft: 8836 corp: 519/27Kb lim: 77 exec/s: 83 rss: 337Mb   (16 new)
    run-present-on-2.log        #835  INITED cov: 3100 ft: 8772 corp: 503/26Kb exec/s: 37 rss: 322Mb
                                #1500 DONE   cov: 3101 ft: 8836 corp: 519/27Kb lim: 77 exec/s: 68 rss: 338Mb   (16 new)
    run-compiled-out-off-1.log  #835  INITED cov: 3100 ft: 8771 corp: 503/26Kb exec/s: 34 rss: 321Mb
                                #1500 DONE   cov: 3101 ft: 8858 corp: 521/27Kb lim: 77 exec/s: 62 rss: 337Mb   (18 new)
    run-compiled-out-off-2.log  #835  INITED cov: 3100 ft: 8772 corp: 503/26Kb exec/s: 49 rss: 321Mb
                                #1500 DONE   cov: 3101 ft: 8836 corp: 519/27Kb lim: 77 exec/s: 83 rss: 337Mb   (16 new)

(The tab after `#835` and `#1500` is shown as spaces. "N new" is the log's
`stat::new_units_added`.)

- **Present, unset:** the pair agrees.
- **Present, set:** the pair agrees, and it equals the unset pair line for
  line. `timings/present-on-*/where_two_surfaces_cross/` holds `report`,
  `totals` and `slowest/`, so the variable was read and acted on.
- **Compiled out:** the pair **disagrees**. Both logs carry the same binary
  sha256, `96534347db43…`. Three more runs of it split the same way:
  `run-compiled-out-off-3.log` ends like run 2 (8772, then 3101 / 8836 / 519,
  16 new), and `-off-4` and `-off-5` end like run 1 (8771, then 3101 / 8858 /
  521, 18 new).

Against the verifier's figures from the first pass: its compiled-out line
(8771, then 3101 / 8858 / 521, 18 new) is reproduced here exactly, in 3 of 5
runs. Its unset line (8771, then 3101 / 8835 / 519, 16 new, the same as
§292.5's) and its set line (8772 / 504, then 3102 / 8863 / 523, 19 new) never
came up on this lane's base build: not in the 4 runs above, not in 20 more at
fixed stack positions (305.4). No run on any binary here ever reached `corp:
504` at `INITED`.

### 305.4 One binary, one command, two answers: where the stack starts

**Not the clock.** libFuzzer prints `Slowest unit:` when an input takes 10 s
or more (`-report_slow_units=10`). The input it named in
`run-compiled-out-off-1`, `slow-unit-a548ac27…`, runs in 9193 ms alone, so
whether it gets reported depends on the machine's load. But the report shows
up in 8771 runs and 8772 runs alike: `run-compiled-out-off-1` (8771, 15 s) and
`run-present-on-2` (8772, 13 s).

**Where the stack starts.** `target/q236/pads.sh` runs the binary under
`setarch x86_64 -R`, which fixes the address space. It adds one environment
variable, `Q236_PAD`, holding 0, 16, 32 or 48 `x`s, which moves the whole
stack down by that many bytes and changes nothing else. On the compiled-out
binary, with the §267.2 flags (`Q236_FULL=1`), two passes each:

| Pad | `full-aslroff-a-padNNN` | `full-aslroff-b-padNNN` |
|---|---|---|
| 0 | `ft: 8771`, then 3101 / 8858 / 521 | `ft: 8771`, then 3101 / 8858 / 521 |
| 16 | `ft: 8772`, then 3101 / 8836 / 519 | `ft: 8772`, then 3101 / 8836 / 519 |
| 32 | `ft: 8771`, then 3101 / 8858 / 521 | `ft: 8771`, then 3101 / 8858 / 521 |
| 48 | `ft: 8772`, then 3101 / 8836 / 519 | `ft: 8772`, then 3101 / 8836 / 519 |

Every `INITED` line here reads `cov: 3100 … corp: 503/26Kb`. With the address
space fixed, each position gives the same answer twice, and the answer flips
every 16 bytes. With randomisation on, the start moves for each process, so
each run draws a position. The two ends are the two lines of the compiled-out
pair in 305.3.

**Which kind of feature.** `counters-aslroff-padNNN` passes
`-use_counters=1` (libFuzzer's default, only there to lengthen `argv` the same
way) and alternates the same way: 8771 / 8772 / 8771 / 8772. With
`-use_counters=0`, libFuzzer counts each edge once and drops its hit-count
buckets. Then `nocounters-aslroff-padNNN` gives `INITED cov: 3100 ft: 3106
corp: 266/12468b` at all four pads. The feature that comes and goes is
therefore the hit-count bucket of an edge that both outcomes cover. It is not
a new edge, and it is not libFuzzer's stack-depth feature, which
`-use_counters=0` still counts (3106 − 3100 = 6 of them, the same at every
pad).

**ASan's fake stack is part of it.** Seed-only runs (`-runs=835`) at pads 0
and 16 give `ft: 8771` and `ft: 8772` (`seedonly-aslroff-pad000`, `-pad016`).
The same runs with `ASAN_OPTIONS=detect_stack_use_after_return=0` give `ft:
8773` at all four pads (`nouar-seedonly-aslroff-pad000` to `-pad048`). With
ASan's use-after-return frames switched off, the stack's position stops
mattering.

**Which edge's bucket: not found.** `-features_dir` is the flag that would
name it: it writes each kept input's features. But it hides the effect. In 34
seed-only runs with `-features_dir`, all the outcomes were 8772: 16 at fixed
pads 0 to 240 (`pads-compiled-out-padNNN`), 6 one at a time
(`seeds-compiled-out-aslr-N`), and 12 run four at once
(`conc-compiled-out-rR-K`). Why it hides it is not known.

**The base binary does not flip at the positions tried.** Its outcome was 8772
at every position: 12 seed-only pads from 0 to 176
(`present-off-seedsweep-padNNN`; the pad-192 run was killed and is not
counted), and 4 full runs each unset and set at pads 0 to 48
(`present-off-aslroff-padNNN`, `present-on-aslroff-padNNN`). Which positions
give which outcome is a property of the binary. This one did not show 8771 at
any position tried, and the compiled-out one shows it at half of them.

**The mutation phase has the same dependence.** Even with `-use_counters=0`,
the final lines still alternate with the pad: 3101 / 3107 / 267 against 3102
/ 3108 / 268. `pcs-nocounters-aslroff-pad000` and `-pad016` (with
`-print_pcs=1`) agree through `#1028` and part at `#1265 NEW`. The new edge
there, found only at pad 0, is binary offset `0x299f0c`, which is `addr2line`
line 371 of `crates/rcad-geom/src/sphere.rs` (`power_of_two_bringing_down`,
reached from `without_overflow` in `HowFarFromTheSkin::on_intervals`). Each
unit that run kept was then replayed alone as a one-file corpus at all four
pads (`target/q236/replay2.sh`): `cov: 471` and `cov: 456` at every pad. So
the kernel answers one input the same wherever the stack starts. It is the
mutation stream that parted, and where that stream reads a stack address was
not measured.

### 305.5 The two legs of the hypothesis

**(a) Code that runs only with the variable set reaches something libFuzzer
counts: broken on this build.** At each of four fixed stack positions, unset
and set give identical `INITED` and `DONE` lines: all eight are `ft: 8772`
and then 3101 / 8836 / 519 (`present-off-aslroff-padNNN` against
`present-on-aslroff-padNNN`). With randomisation on, the two unset and two set
runs in 305.3 are identical too. The set arm's own outcome in §292.5 and in
the verifier's run, 8772 / 504, never came up here in any arm. Where it comes
from is open (305.7).

**(b) `timed`'s instance is compiled into the target and carries coverage:
broken.** `timed` has no symbol of its own in the base binary. It is inlined
into `where_two_surfaces_cross::_::__libfuzzer_sys_run`, which calls
`start_the_clock` at `0x1d2a5b` and `stop_the_clock` at `0x1d56d6` through the
GOT (`readelf -r` resolves the two GOT slots to `0x1e88f0` and `0x1e76c0`).
`target/q236/fnstat.sh` compares that function across the three binaries:

| Binary | Bytes | Counter increments (`incb` on `__sancov_cntrs`) | `__sanitizer_cov_trace_*` calls | Calls through the GOT |
|---|---|---|---|---|
| present | 14 984 | 234 | 147 | 45 |
| compiled-out | 14 740 | 234 | 147 | 43 |
| at-9e72d414 | 14 984 | 234 | 147 | 45 |

The only difference is the two calls. All three binaries have the same 17 322
counters. The `OnceLock` check itself has not been in `timed` since §267.3
item 2 moved it into `start_the_clock`, which is not generic, is
`#[inline(never)]`, and sits in the crate the stand-in keeps uninstrumented
(timing rlib `sancov` = 0 in all three builds).

### 305.6 9e72d414, built the same way

    run-at-9e72d414-off-1.log   #835  INITED cov: 3100 ft: 8772 corp: 503/26Kb exec/s: 49 rss: 321Mb
                                #1500 DONE   cov: 3101 ft: 8836 corp: 519/27Kb lim: 77 exec/s: 88 rss: 337Mb   (16 new)
    run-at-9e72d414-off-2.log   #835  INITED cov: 3100 ft: 8772 corp: 503/26Kb exec/s: 34 rss: 321Mb
                                #1500 DONE   cov: 3101 ft: 8836 corp: 519/27Kb lim: 77 exec/s: 62 rss: 337Mb   (16 new)
    run-at-9e72d414-on-1.log    #835  INITED cov: 3100 ft: 8772 corp: 503/26Kb exec/s: 43 rss: 322Mb
                                #1500 DONE   cov: 3101 ft: 8836 corp: 519/27Kb lim: 77 exec/s: 78 rss: 338Mb   (16 new)
    run-at-9e72d414-on-2.log    #835  INITED cov: 3100 ft: 8772 corp: 503/26Kb exec/s: 43 rss: 322Mb
                                #1500 DONE   cov: 3101 ft: 8836 corp: 519/27Kb lim: 77 exec/s: 78 rss: 338Mb   (16 new)

Both pairs agree, set equals unset, and all four equal §267.5's first table
(3100 / 8772 / 503, then 3101 / 8836 / 519, 16 new). The base build gives the
same lines. So what moved between §267.5's 8772 and the compiled-out 8771 is
not a commit. The compiled-out binary draws 8771 at some stack positions,
while `9e72d414` and the base, as built here, gave 8772 in every run.

**No range is worth bisecting**, because its two ends agree. Were one needed,
it would be `9e72d414..f85f926d`, which is 316 commits (`git rev-list --count
9e72d414..HEAD`). Among the paths the target links, only
`crates/rcad-fuzz-timing/src/lib.rs` and `fuzz/rustc-stand-in.sh` change in it
(`git diff --stat 9e72d414 HEAD -- crates/ fuzz/ Cargo.toml Cargo.lock .cargo
mise.toml mise.lock rust-toolchain.toml` lists no file under `rcad-geom`,
`rcad-predicates`, `fuzz/fuzz_targets` or `fuzz/Cargo.lock`).

### 305.7 What is left

- **Which counter's bucket flips, and how ASan's fake stack reaches it (row
  candidate).** What is known: it is a hit-count bucket, it needs
  `detect_stack_use_after_return`, and it moves with every 16 bytes of stack.
  `-features_dir` cannot be the tool, because it hides the effect (305.4). One
  route that avoids it is a patched libFuzzer, or a debugger at the end of the
  seed pass, dumping `__sancov_cntrs` at pad 0 and at pad 16 with ASLR off and
  diffing the two. Whether the kernel's own answer can depend on this has one
  data point: two units replayed alone covered the same at every pad. That is
  not a proof for the corpus.
- **Why the verifier's and §292.5's builds landed differently.** Their unset
  runs gave 8771 and their set runs 8772 / 504, each consistent within its own
  pair, from the same source that here gave 8772 every time. Their binaries
  are not this lane's, and that was not measured. Its legs, if someone takes
  it up: those builds ran in other worktrees; the tree's path is compiled into
  the binary; a different layout moves which stack positions give which
  outcome. `corp: 504` at `INITED` is a third outcome this lane never
  reproduced, on any binary.
- **What the record should stop doing.** §267.5 and §292.5 compare a pair of
  single draws from this lottery. To judge whether `RCAD_FUZZ_TIMINGS` moves
  libFuzzer, compare the arms at fixed stack positions: `setarch -R`, with the
  two environments padded to the same size. Or take several runs per arm with
  randomisation on and compare how the outcomes are spread. By the fixed-stack
  comparison in 305.5 the timing crate moves nothing on this build.
- **The first pass's figures.** It reported `INITED ... ft: 8772 corp: 504`
  and `DONE ... 3102 / 8863 / 523` for all six runs, across all three arms. No
  run here reached `corp: 504`, so none of that is explained.

### 305.8 Two more trees: the path moves the flip, and the seed pass holds still

*Queue rows Q277 and Q278, read-only, 2026-10-09. Each built `f85f926d` with
305.2's build line in a worktree of its own, on the same compiler. Neither lane
wrote rows; this is written by their dispatcher from their reports and from
the logs they left, which were read again for every number here.*

**The path is in the binary, and it moves the layout (Q278).** Q278 built the
present arm twice: in its own tree,
`clean-up-dead-sessions-worktrees/wf_5b5d60fe-c73-1`, whose full path is 99
characters long (build A), and in a second detached worktree under that tree's
`target/`, 190 characters long (build B). `strings -a` finds the tree's path 27
times in each binary. Every section from `.text` on, and every symbol in them,
sits 0x500 bytes later in B: `.text` starts at `0x104a10` against `0x104f10`,
and `main` is at `0x2d3f60` against `0x2d4460` (`readelf -S`, `nm`). §305's own
tree, `dispatch-next-worktrees/wf_39298cae-c5a-1`, was 90 characters long.

**The run's end flips with the stack in both builds, and the path decides which
way (Q278).** The compiled-out arm (305.2's edit) of each build, run with the
pad script below: `RCAD_FUZZ_TIMINGS` unset, `setarch -R`, two passes per pad.
The two passes agreed at every pad. Each cell is the `DONE` line's cov / ft /
corp:

| Pad | A, path 99 | B, path 190 |
|---|---|---|
| 0 | 3101 / 8836 / 519 | 3101 / 8859 / 521 |
| 16 | 3101 / 8859 / 521 | 3101 / 8836 / 519 |
| 32 | 3101 / 8836 / 519 | 3101 / 8859 / 521 |
| 48 | 3101 / 8859 / 521 | 3101 / 8836 / 519 |

The flip still comes every 16 bytes, and the longer path puts each pad on the
other side. The present arm, run in A only, ended 3101 / 8836 / 519 at all four
pads in both passes, as §305's present build never flipped (305.4).

**The seed pass did not flip on either tree.** All 24 of Q278's runs read
`INITED cov: 3100 ft: 8772 corp: 503/26Kb`. Q277 ran 305.4's seed-only command
(`-runs=835`) on the compiled-out arm in its own tree,
`clean-up-dead-sessions-worktrees/wf_64f4aec9-1b3-1`, also 99 characters long.
It tried every pad from 1 to 16, then 32, 40, 44, 46, 48, 56 and 64, then 80,
96, 112, 128, 160, 192, 224 and 256, and repeated some of them. Its 51 logs that kept the default ASan options all read `ft:
8771`. (Its "pad 0" runs carried one byte, because `printf 'x%.0s' $(seq 1 0)`
prints one `x`; the other pads are as named.) So on these two trees the seed
pass gives one answer at every pad, and not the same answer: 8771 on Q277's and
8772 on Q278's, from the same source with paths of the same length. What
separates them was not measured. Candidates are the run length in `argv`
(`-runs=835` against `-runs=1500`), the bytes of the tree's name, and the
environment.

**ASan's use-after-return moved it, as in 305.4.** Q277's run with
`ASAN_OPTIONS=detect_stack_use_after_return=0` and an empty pad read `ft:
8772`, where 305.4's tree read 8773. The lane reports a second such run, and
one seed-only run at pad 46 that read 8772 and then 8771 in 7 exact repeats.
Later runs of the same name overwrote both logs, so neither can be checked.

**No counters were dumped.** No pair of pads gave two answers, so there was
nothing to diff, and the first item of 305.7 is still open. Whoever takes it
needs a tree whose seed pass flips with the pad, and §305's tree is gone
(§420).

**`corp: 504` is still unexplained.** No run on either tree reached it at
`INITED`. So the second item of 305.7 is narrower but not closed: the path
moves which pads give which ending, but none of these builds gives §292.5's or
Q236's first-pass figures.

**For Q279.** The run's end parts with the pad in both of Q278's builds, so
305.4's last finding reproduces off §305's tree, with counters on. Q279 asks
the same with `-use_counters=0`. It can start from the scripts below in any
tree, but it should first check that its pads part there.

**The scripts** (§420.4). Q278's build, run from the tree that becomes the
binary:

    #!/usr/bin/env bash
    set -euo pipefail
    cd "$(dirname "$0")/../.."
    pwd
    cd fuzz
    rustc +nightly -V
    env | grep RCAD_ || true
    CI=1 RUSTC="$(pwd)/rustc-stand-in.sh" RUSTFLAGS="-C target-cpu=x86-64-v3" taskset -c 16-19 cargo +nightly fuzz build where_two_surfaces_cross
    sha256sum target/x86_64-unknown-linux-gnu/release/where_two_surfaces_cross

Q278's pads, called as `pads.sh <binary> <label> <corpus> <out-dir>`:

    #!/usr/bin/env bash
    set -euo pipefail
    BIN="$1"
    LABEL="$2"
    CORPUS_SRC="$3"
    OUT="$4"
    mkdir -p "$OUT"
    for pad in 0 16 32 48; do
      pad_str=""
      i=0
      while [ "$i" -lt "$pad" ]; do
        pad_str="${pad_str}x"
        i=$((i + 1))
      done
      for pass in a b; do
        run_dir=$(mktemp -d)
        cp -r "$CORPUS_SRC" "$run_dir/corpus"
        log="$OUT/full-aslroff-${LABEL}-${pass}-pad$(printf '%03d' "$pad").log"
        {
          echo "pad=$pad pass=$pass bin=$BIN"
          env -u RCAD_FUZZ_TIMINGS Q236_PAD="$pad_str" setarch x86_64 -R taskset -c 16-19 "$BIN" "$run_dir/corpus" -seed=42 -runs=1500 -print_final_stats=1
        } > "$log" 2>&1 || true
        grep -E "INITED|DONE" "$log" || true
        rm -rf "$run_dir"
      done
    done

Q277's seed-only runs were this command with `-runs=835` in place of
`-runs=1500`, `Q277_PAD` in place of `Q236_PAD`, the seed corpus copied into
the run directory, and no `env -u`, so whether `RCAD_FUZZ_TIMINGS` was set
in its shell is not recorded. Each lane's logs stayed in its tree's `target/` and go at the next
`clean-targets`.

### 305.9 Without counters nothing parts, and use-after-return moves the seed pass

*Queue row Q279, read-only, 2026-10-09. Its lane built 305.2's compiled-out
arm of `where_two_surfaces_cross` at `012d3163`, with 305.8's build script, in
`clean-up-dead-sessions-worktrees/wf_4d21279a-961-1`, whose path is 99
characters long like Q278's build A (sha256 `a6dbcc1b…`, 2 m 30 s). This is
written by its dispatcher from the lane's report and from its 42 logs, which
were read again for every number here. The source is not 305.8's `f85f926d`,
so these numbers compare with one another and not with 305.4's or 305.8's.*

**Without counters, nothing parts.** Q279 asked what still moved with the
stack under `-use_counters=0`, since 305.4's two pads parted at `#1265 NEW`
even then. On this build nothing does. 305.8's pads script with
`-use_counters=0`, at 15 pads (every 8 bytes from 0 to 64, then every 32 from
96 to 256), two passes each: 29 of the 30 runs print the same `INITED`,
`NEW` and `DONE` lines, leaving out `exec/s` and `rss`. They read `INITED cov:
3271 ft: 3277 corp: 265/12397b` and `#1500 DONE cov: 3282 ft: 3288 corp:
270/12712b`. So the question has no live case on this build, and
`sphere.rs:371`, the edge 305.4 saw part first, cannot be tested on it.

**The 30th run moved with something other than the stack.** Pass a at pad 56
printed the same lines up to `#1151`, then ran 1719 units rather than 1500,
and its last unit came in by `RELOAD` at `#1719` rather than as `#1497 NEW`.
Pass b at pad 56 matched the other 28. The lane put this down to the slow-unit
report. That does not hold: four runs without counters that matched reported
a slow unit too, pass b at pad 56 among them. libFuzzer rereads its corpus on a
timer (`-reload`), which would fit a run that differs from its own repeat, but
that was not measured.

**With counters, the seed pass parts.** 305.8's pads script as printed, at
pads 0, 16, 32 and 48, two passes each:

| Pads | `INITED` cov / ft / corp | `DONE` cov / ft / corp |
|---|---|---|
| 0 and 32 | 3271 / 9203 / 498 | 3276 / 9297 / 513 |
| 16 and 48 | 3271 / 9204 / 499 | 3273 / 9274 / 517 |

At each pad the two passes printed the same trace line for line. The flip
comes every 16 bytes, as in 305.8. Unlike either of 305.8's trees, it is
already there at `INITED`, at the end of the seed pass. That is the case
305.7's first item needs and 305.8 could not find.

**With use-after-return off, the four pads agree.** The same runs with
`ASAN_OPTIONS=detect_stack_use_after_return=0`, one pass per pad, print one
trace at all four pads: `INITED cov: 3271 ft: 9204 corp: 498/25Kb` and `DONE
cov: 3275 ft: 9298 corp: 513/26Kb`. That is a third outcome, neither group's.
So on this build ASan's fake stack is all that moves with the stack when
counters are on, which is where 305.4 and 305.8 saw it move the seed pass.
libFuzzer's value profile and CMP tracing were not turned off one at a time.
With nothing left to explain on this build, they were not needed here; that
says nothing about another build.

**Replay.** A run replays from `-seed` when ASLR is off (`setarch -R`) and
the stack starts at the same place: the same pad, in an environment of the
same size. Every pair of passes at one pad agreed, with counters on and with
counters off, except pass a at pad 56 above. `-seed` alone does not replay
with counters on and ASan's default options, because the outcome moves with
the stack start. Runs with ASLR on were not made.

**For comparing two arms** (305.7's third item): on this build, either
`-use_counters=0` or `detect_stack_use_after_return=0` takes the stack out
of the comparison, shown at 15 pads and at 4 pads respectively. Each changes
what is measured, so a comparison made with one says nothing about a night
run without it.

**What is left.** 305.7's first item can now be reached: rebuild this arm at
`012d3163` in a tree of its own, check that two pads part at `INITED`, dump
`__sancov_cntrs` at the end of the seed pass at each, and diff them. Q277 and
Q278 had paths of the same length and still saw different seed passes
(305.8), so the rebuild must find its own parting pads and not assume 0 and
16. Filed as Q1525. The lane's tree is removed (§420.4), with its logs; the
lines that matter are above.

**The scripts.** The build is 305.8's script, unchanged. The runs without
counters are 305.8's pads script with `-use_counters=0` after
`-runs=1500`. The second batch is the same script with the pad list
`8 24 40 56 64 96 128 160 192 224 256`. The runs with counters are 305.8's
script exactly. The use-after-return runs are 305.8's script with one pass
per pad and `ASAN_OPTIONS=detect_stack_use_after_return=0` after
`Q236_PAD="$pad_str"`. Each run used the tree's own
`fuzz/corpus/where_two_surfaces_cross`, copied into a fresh directory.

### 305.10 The counter that parts is in the kernel, in `Exact::minus`

*Queue row Q1525, read-only, 2026-10-17. Its lane rebuilt 305.2's
compiled-out arm of `where_two_surfaces_cross` at `012d3163`, with 305.8's
build script, in `clean-up-dead-sessions-worktrees/wf_34a911b1-00b-2`, whose
path is 99 characters long (sha256 `324a95f6…`). This is written by its
dispatcher from the lane's report and its logs, which were read again for
every number here, and the address was mapped to its source line again on the
lane's binary.*

**The pads part again, the other way round.** 305.8's pads script as printed,
at pads 0, 16, 32 and 48, two passes each:

| Pads | `INITED` cov / ft / corp | `DONE` cov / ft / corp |
|---|---|---|
| 0 and 32 | 3271 / 9204 / 499 | 3273 / 9274 / 517 |
| 16 and 48 | 3271 / 9203 / 498 | 3275 or 3276 / 9297 / 513 |

These are 305.9's two outcomes with the pads swapped: there, 0 and 32 read
9203. The source and the path length are the same and the tree is not, which
is 305.8's finding again: the path moves which pads give which outcome. Every
pair of passes printed the same `INITED` line. At `DONE`, pad 16's two passes
read cov 3275 and 3276, the one pair that did not agree.

**`__sancov_cntrs` cannot be diffed at `INITED`.** libFuzzer clears the
counters before every unit (`TracePC::ResetMaps`), so at the end of the seed
pass they hold the last unit's run and nothing else. What does add up over
the seed pass is `InputCorpus::InputSizesPerFeature`: one 32-bit word per
feature, 2^21 words, each the size of the smallest input that showed that
feature, or 0 if none did. The lane stopped in gdb at `PrintStats` for
`INITED`, at pad 0 and at pad 16, dumped that array at each, and diffed the
two 8 MB dumps.

**gdb moves the run too.** Under gdb both pads read `corp: 498` at `INITED`
and the same `DONE` line, 3276 / 9298 / 513, which is neither group's. `ft`
at `INITED` still parted: 9204 at pad 0, 9203 at pad 16. A run under gdb
writes new units back into its corpus directory, so each one started from a
fresh copy.

**Two words differ.**

- **Word 46992 is in the kernel.** It is feature 46992 = 5874 × 8, the first
  hit-count bucket of counter 5874: a unit in which that line ran once. Pad 0
  holds 56 there and pad 16 holds 0, so it is the one feature pad 16 lacks,
  9204 against 9203. Counter 5874 is incremented by the `incb` at static
  address `0x236348` (`__start___sancov_cntrs+0x16f2`, and 0x16f2 is 5874),
  which `addr2line` places in `Exact::minus`, `crates/rcad-geom/src/exact.rs`
  line 196: the `map(|entry| -entry).collect()` over `other`'s entries. That
  line runs once per entry negated, so some seed unit at pad 0 negates one
  entry in all, and no unit at pad 16 does.
- **Word 180601 is libFuzzer's.** It is libFuzzer's one stack-depth feature.
  Both pads have it; only the smallest input that showed it differs, 21 bytes
  at pad 0 and 1 byte at pad 16. It does not move `ft`.

**What it says.** On the same seed files, the kernel takes a different exact
path depending on where the stack starts, under ASan's default options. 305.9
found that with `detect_stack_use_after_return=0` all four pads agree, so
ASan's fake stack is what lets it part. 305.7's first item asked whether the
kernel's own *answer* can depend on this. This shows that its *path* can. It
does not show the answer moving, and it does not name the seed unit.

**What is left.** Find the seed unit that runs that line at one pad and not
at the other, and say why the kernel's path moves on the same bytes: which
call, given what values at each pad, and whether the fuzz target's answer
moves with it. A path in code without `unsafe` moves only if what it reads
moves. So either something reads an address, a clock or memory never written,
in the kernel or under it; or state carried from one unit to the next moves;
or ASan or libFuzzer changes what the target is given. None of these was
looked at. Filed as Q2245. The lane's tree is removed (§420.4), with its
logs; the lines that matter are above.
