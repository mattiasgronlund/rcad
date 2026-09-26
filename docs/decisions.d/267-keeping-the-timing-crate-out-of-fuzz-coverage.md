## 267. Keeping the timing crate out of fuzz coverage — 2026-09-26

*Queue row Q136, from §259.5 and §259.8: with `RCAD_FUZZ_TIMINGS` set, two
seeded runs of `where_two_surfaces_cross` stop repeating, about 640 more edges
are counted at `INITED`, and libFuzzer runs more inputs than `-runs` asked for.
The row asked for `rcad-fuzz-timing`'s code to be kept out of what libFuzzer
counts, or a demonstration that it cannot be, and was done when §259.5's pair
with the variable set matches the pair without it.*

  267.1   The answer
  267.2   Method
  267.3   Five ways the timings reached libFuzzer
  267.4   Leaving the crate uninstrumented: three routes that fail, one that works
  267.5   The pair, rerun on this lane's tip
  267.6   What is left

### 267.1 The answer

It can be done, and the pair then matches in every figure (267.5). It takes
two changes. One is in the crate, and it is on this lane's branch: four things
the crate did besides being instrumented, each of which steered libFuzzer on
its own (267.3). The other is in the build, and it is not on the branch: the
crate has to be compiled without coverage instrumentation, and no route
inside this lane's write set does that (267.4). The route that works needs a
new file under `fuzz/` and the `fuzz-overnight` recipe, which is where the
night builds its targets. So the row's "done" test passes only with the build
change applied by hand. With the crate change alone, the set pair still
diverges.

### 267.2 Method

§259.1's method, with the binary built from this lane's tree:
`RUSTFLAGS="-C target-cpu=x86-64-v3" cargo +nightly fuzz build
where_two_surfaces_cross`. For each binary, four runs start at once, each on a
fresh `cp -r` of `fuzz/corpus/where_two_surfaces_cross` in a scratch
directory, with `-seed=42 -runs=1500 -print_final_stats=1` and nothing else.
Two runs have `RCAD_FUZZ_TIMINGS` unset. Two have it naming a fresh directory
per run. The corpus directory is the only corpus argument, as in §259.

The box was loaded to between 28 and 35 on 24 cores throughout, from other
lanes. The figures compared below are coverage, features, corpus and run
counts, not rates.

"Uninstrumented" below means the build went through a rustc stand-in (267.4)
that adds `-Cllvm-args=-sanitizer-coverage-level=0` to the one rustc call that
compiles `rcad_fuzz_timing`. Each time, `nm` on that crate's rlib counted the
references to `__sanitizer_cov_trace_cmp` or `sancov`: 43 or 45 when
instrumented, 0 when not.

### 267.3 Five ways the timings reached libFuzzer

**1. The crate's own code is counted.** This is §259.8's reading, and it
holds. On the baseline binary, the set runs start at 3755 and 3736 edges at
`INITED`, against 3107 unset. Built uninstrumented with the code unchanged,
they start at 3115 and 3123. So about 615 to 640 edges are the crate's own
code.
The 8 to 16 left over are the next item.

**2. `timed` is generic, so it is compiled into the fuzz target.** The target
is instrumented whatever is done to the timing crate. Before this lane,
`timed` held the `OnceLock` branch, `Once::call_once`, the thread spawn, the
`Mutex` lock and `Instant::elapsed`. Each of those went one way with the
variable set and another without it, and the lock also went one way or
another depending on whether the merging thread held it. Now `timed` is two
calls and the draw. `start_the_clock` and `stop_the_clock` are neither generic
nor inlined, so they are compiled into the timing crate once. I did not
measure this item on its own: the residual 8 to 16 edges in item 1 went away
together with items 3 and 4.

**3. A draw that ends with more `malloc`s than `free`s is run again.**
libFuzzer's `TryDetectingAMemoryLeak` (`FuzzerLoop.cpp:680-721` in
libfuzzer-sys 0.4.13) re-runs the input to look for a leak, and the re-run
counts against `-runs`. Its malloc hook (`:98-122`) counts every thread's
allocations. Two things in the crate tripped it:

- `offer` copied each kept slow input into a fresh `Vec`: a `malloc` with no
  `free`, for the first 64 kept per process.
- The merging thread allocated in whichever draw happened to be running.

That is §259.8's "INITED lands 67 and 68 runs later". Here it was `#901` and
`#903` on the baseline binary, and `#902` in both runs of the uninstrumented
build, against `#835` unset. I did not separate the two sources. Removing
both put all four runs at `#835`.

The fix has two parts. First, every buffer the slowest runs will ever use is
allocated when the watch is made, in the first draw. That draw is the empty
input libFuzzer runs once at start-up and never checks for leaks. Each buffer
starts with room for 64 bytes, because growing a buffer that has room is a
`realloc`, which counts one of each, while growing an empty one is a `malloc`
alone. Second, there is no thread any more. A draw that finds a merge due
makes it before it returns, and frees everything it allocated.

**4. `memcmp` hands both sides to the mutator.** ASan intercepts `memcmp` for
every caller in the process, instrumented or not. libFuzzer's hook
(`FuzzerTracePC.cpp:623-629`) puts both sides of any comparison longer than
one byte into the table its `CMP` mutations draw from, provided the
comparison happened inside a draw and came out unequal. `offer` compared each
draw's bytes with every held slow input of the same length, and which inputs
are held is the clock's choice.

With items 1 to 3 fixed, the uninstrumented build matched to `INITED` in all
four runs (`#835`, 3100 / 8772 / 503). The set runs then parted from the
unset runs at the second `NEW` line: `#867 ... MS: 3 CMP-ChangeByte-CMP- DE:
"\012\000\000\000\000\372...UUUUUUU..."`. That dictionary entry is a whole
input, and the unset runs never produce it.

Now `SlowRun::is` compares hashes first, and compares the bytes only when the
hashes match, where `memcmp` returns 0 and the hook does nothing. The pruning
of `slowest/`, which runs inside a draw now, sorts and deduplicates by the
numbers parsed out of the file names, not by comparing names as text.

**5. `-runs` is overshot by the corpus reload, not by the timings.** This
answers §259.8's "which inputs libFuzzer runs again, and why", for the part
of it past `#1500`. libFuzzer rereads its corpus directory every second
(`-reload=1`, the default). `RereadOutputCorpus` runs before the `-runs`
check at the top of the loop (`FuzzerLoop.cpp:886-892`), and nothing stops it
passing the limit. Every run that ended past 1500 printed a `RELOAD` line:
`#1650` on the baseline binary, `#1589` after items 1 to 3 were fixed, and
`#1577` on this tip as cargo-fuzz builds it. The converse does not hold: a
reload early in a run is absorbed, and one run printed `RELOAD` and still
ended at `#1500`. In every pair that matched, no run printed `RELOAD`. A
reload that finds nothing new prints nothing, so this shows up only in runs
that have already diverged.

### 267.4 Leaving the crate uninstrumented: three routes that fail, one that works

cargo-fuzz passes coverage to every crate through one `RUSTFLAGS`
(`-Cpasses=sancov-module -Cllvm-args=-sanitizer-coverage-level=4 ...`). The
row came with a lead: switch coverage off for one crate with a second
`-sanitizer-coverage-level`.

That flag does work, but only as the last one on the command line. Compiling a
one-function probe with cargo-fuzz's flags left 29 `sancov` or
`__sanitizer_cov` references in its IR. Adding
`-Cllvm-args=-sanitizer-coverage-level=0` after them left 0.
`rustc +nightly -C llvm-args=--help-hidden` (LLVM 23.1.0) lists no
sancov option that takes an allow-list or deny-list, so the level is the only
switch.

**Profile rustflags in `fuzz/Cargo.toml` fail twice.** The entry tried was
`cargo-features = ["profile-rustflags"]` with
`[profile.release.package.rcad-fuzz-timing] rustflags =
["-Cllvm-args=-sanitizer-coverage-level=0"]`. First, it breaks
`check-fuzz-builds`: stable `cargo check` in `fuzz/` answers "the cargo
feature `profile-rustflags` requires a nightly version of Cargo". Second, on
nightly the build succeeds and changes nothing. Cargo
puts profile rustflags *before* `RUSTFLAGS`. In the `rcad_fuzz_timing` rustc
line of `cargo fuzz build -v`, `level=0` sits at byte 669 and cargo-fuzz's
`level=4` at byte 1270, so `level=4` wins. The rlib had 43 sancov references
with the profile entry and 43 without it.

**The same entry in a cargo config fails the same way on stable.** Passing it
as `cargo check --config <file>` with `[unstable] profile-rustflags = true`
answers "config profile `release` is not valid ... feature
`profile-rustflags` is required". So `fuzz/.cargo/config.toml` would also
break `check-fuzz-builds`. I did not build this form on nightly. It goes
through the same profile code, so I expect the same ordering, but that is not
measured.

**Environment variables are not read.** Building with
`CARGO_UNSTABLE_PROFILE_RUSTFLAGS=true` and
`CARGO_PROFILE_RELEASE_PACKAGE_RCAD_FUZZ_TIMING_RUSTFLAGS=...` recompiled
nothing. The build log had no `Compiling` lines at all.

**A rustc stand-in works.** Set as `RUSTC`, it is a script that appends
`-Cllvm-args=-sanitizer-coverage-level=0` to any call carrying `--crate-name
rcad_fuzz_timing` and runs the nightly rustc. The rlib had 0 sancov
references, and 267.5 is the result.

It needs three things this lane may not write. The first is the script
itself, a new file under `fuzz/`. The second is `RUSTC` set in the
`fuzz-overnight` recipe's `cargo +nightly fuzz build`, as well as in
`fuzz-target`: the night builds once, and then `cargo fuzz run` does not
rebuild, because cargo's fingerprint cannot see a flag added by the stand-in.
The third is an answer about kache. `cargo` here is a mise shim, and it puts
`RUSTC_WRAPPER=kache` back whatever the caller set, so kache runs *around*
the stand-in. kache computes its key before the flag is added, so it could
hand back an instrumented rlib cached from a plain build. I switched kache
off for these builds with `CI=1`, which makes mise leave `RUSTC_WRAPPER`
empty. Whether kache would in fact mix the two up is not measured.

Two related points. First, profile rustflags cannot help even with
`check-fuzz-builds` moved to nightly, because of the ordering above. Second,
the crate's code alone is not enough either: 267.5's last table is this tip
built as cargo-fuzz builds it today.

### 267.5 The pair, rerun on this lane's tip

Tip `9e72d414`, built through the stand-in (timing rlib: 0 sancov references):

| | `RCAD_FUZZ_TIMINGS` | `INITED` at | `INITED` cov / ft / corp | ended at | final cov / ft / corp | new units |
|---|---|---|---|---|---|---|
| Run 1 | unset | `#835` | 3100 / 8772 / 503 | `#1500` | 3101 / 8836 / 519 | 16 |
| Run 2 | unset | `#835` | 3100 / 8772 / 503 | `#1500` | 3101 / 8836 / 519 | 16 |
| Run 1 | set | `#835` | 3100 / 8772 / 503 | `#1500` | 3101 / 8836 / 519 | 16 |
| Run 2 | set | `#835` | 3100 / 8772 / 503 | `#1500` | 3101 / 8836 / 519 | 16 |

The same was true of the build just before the tip, whose code differed only
in formatting and doc comments. There, with the stand-in, a line-by-line diff
of the unset and set logs differs only in the two `INFO: Loaded` lines, which
print load addresses. Every `NEW` line and the recommended dictionary are the
same. On the tip, all four runs took 17.7 to 17.9 s. On the baseline binary
the set runs took 40 and 70 s, against about 25 and 26 s unset.

The record still does what it did. The first set run's report reads `736
runs, 17.1 s in all`, and `slowest/` holds 64 inputs. The 764 draws after the
last merge were not merged because the process exited less than a second
later. That is the loss at exit that `HOW_OFTEN_TO_MERGE` already describes.

The same tip as cargo-fuzz builds it today (timing rlib: 45 sancov references):

| | `RCAD_FUZZ_TIMINGS` | `INITED` at | `INITED` cov / ft / corp | ended at | final cov / ft / corp | new units |
|---|---|---|---|---|---|---|
| Run 1 | unset | `#835` | 3108 / 8779 / 503 | `#1500` | 3109 / 8879 / 527 | 24 |
| Run 2 | unset | `#835` | 3108 / 8779 / 503 | `#1500` | 3109 / 8879 / 527 | 24 |
| Run 1 | set | `#835` | 3621 / 9426 / 505 | `#1500` | 3622 / 9477 / 527 | 22 |
| Run 2 | set | `#835` | 3610 / 9411 / 505 | `#1577` | 3624 / 9541 / 528 | 17 |

The leak re-runs are gone (`#835` in all four), and so is about a fifth of
the extra coverage. The rest is item 1, and it still diverges.

### 267.6 What is left

- **The build change (row candidate).** Check in the stand-in and set `RUSTC`
  in `fuzz-overnight`'s build and in `fuzz-target`. Answer the kache question
  in 267.4, or keep kache out of fuzz builds. Have the night fail loudly if
  the timing rlib it built has any sancov reference, since a stale build would
  otherwise pass silently. The write set is a new file under `fuzz/`, the
  `fuzz-overnight` and `fuzz-target` recipes, and any other recipe that builds
  fuzz targets. Done when 267.5's first table holds on the night's own build.
- **One thing the change costs, not yet seen on a night.** A forked child
  killed in the middle of a draw, as a `-timeout` does, now loses up to a
  second of runs from before that draw. The thread used to merge them while
  the long draw ran. How often children die that way on a real night has not
  been counted.
- **Only one target was paired.** The reader, writer and fuse targets have
  §259.3's own cause of divergence, so this section's pair cannot be run on
  them as it stands.
