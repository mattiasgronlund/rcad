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
  267.7   What changed after review

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
- **One thing the change costs.** A forked child that times out in the middle
  of a draw now loses up to a second of runs from before that draw. The
  thread used to merge them while the long draw ran (§262.2). 267.7 sizes it
  on one night: at most 0.90% of `where_two_surfaces_cross`'s runs, and about
  nothing on the other four targets. It is kept, not a row.
- **Only one target was paired.** The reader, writer and fuse targets have
  §259.3's own cause of divergence, so this section's pair cannot be run on
  them as it stands.
- **What a merge costs inside a draw (row candidate).** Not measured. The
  plan the review left: `strace -f -ttt -e trace=openat,flock` on a short
  `RCAD_FUZZ_TIMINGS` run, taking the time from opening `lock` to releasing
  it as one merge, counted per process per second.
- **Whether the merge still reaches ASan's `memcmp` (row candidate).**
  `read_back`'s `"runs"`, `"nanos"`, `"lost"` and `"b"` match arms, and
  `how_slow_the_name_says`'s `strip_prefix("slow-")`, compare strings inside a
  draw. Whether LLVM makes those inline compares or calls to `memcmp`, which
  libFuzzer's hook would see when they come out unequal, was not checked.
- **The loss at a normal end (row candidate).** A child that ends at its
  `-max_total_time` or on the stop file loses up to a second of runs. On
  2026-09-26 the most that could be was the reader's: 528 jobs, none of which
  timed out or crashed, against 28 770 s × 4 = 115 080 child-seconds, so at
  most 0.46% of its runs. It ends with `exit(0)`, so this is
  a gap, and 267.7 found a route that needs no `unsafe`: a thread-local whose
  `Drop` merges. Not built and not measured inside a libFuzzer child. Low
  value.

### 267.7 What changed after review

A review of `9e72d414` asked for three changes, and one more defect was found
after it. All are on this lane's branch, and each code change has a test that
fails without it.

**A hash set on the draw's thread.** `prune_to_the_slowest_kept` held the
inputs it had kept in a `HashSet`. Before this lane that ran on the merging
thread. Since item 3 of 267.3 it runs inside a draw, on libFuzzer's own
thread. Each `RandomState` made on a thread steps the keys of the next one
made there ("we increment one of the seeds on every RandomState creation",
`std/src/hash/random.rs`). So the clock picked when the target's own hash maps
changed the order they iterate in. `where_two_surfaces_cross` cannot show
this, because its oracle has no hash map (§259.6). The reader, writer and fuse
targets do use them (§259.3). The set is a `BTreeSet` now, and no other code in
the crate, outside its tests, makes a `HashMap`, `HashSet` or `RandomState`.

The test, `a_draw_that_merges_leaves_the_hash_keys_of_its_thread_where_it_found_them`,
reads the thread's next key before and after a draw that merges and prunes a
duplicate. The keys differ per thread, so one thread has to compare with
itself. No public API returns the key, but a `DefaultHasher`'s `Debug` prints
it. With the `HashSet` the test failed: "the draw made 1 `RandomState`s on its
thread".

**How a child ends.** `HOW_OFTEN_TO_MERGE` said libFuzzer kills its children
from outside, so there is no exit to hook. The parent never kills a child.
`WorkerThread` waits for each one (`FuzzerFork.cpp:303-309`), and `StopJobs`
only writes a stop file (`:368-373`). A child that reaches `-max_total_time`
or finds the stop file ends with `exit(0)` (`FuzzerDriver.cpp:930`), which
runs the exit handlers. A timeout, a crash or an out-of-memory ends in `_Exit`
(`FuzzerLoop.cpp:307`, `:238`, `:135` and `:322`), which runs nothing. So what
a child loses at a normal end is a gap, and what it loses at an `_Exit` is a
limit.

The review read that gap as one only `unsafe` could close, since registering
an exit handler is a call into C. It is not so on this box. A probe built with
rustc 1.96 under `#![forbid(unsafe_code)]`, whose main thread holds a
`thread_local!` with a `Drop` and then calls `std::process::exit(0)`, printed
from the `Drop`. glibc's `exit` runs the calling thread's thread-local
destructors, and libFuzzer's draws and its `exit(0)` are on the same main
thread. Whether a merge from that destructor works inside a libFuzzer child is
not measured. The doc now says all of this.

**A process that ended mid-merge could empty `totals`.** Found after the
review, and older than this lane. `merge_out` read the one shared `totals`,
added its counts and rewrote it with `std::fs::write`, which empties the file
at `open` and writes after. `read_back` reads an empty or cut file as no runs.
So a process that ended between the two reset the whole target's record,
`lost` included, and the next merge started the count again from its own
runs. The doc above `merge_out` said a failed write "loses just that one
merge's counts". Now `totals` and the report are written to a fixed temporary
name and renamed over, as slow inputs already were (§236.6), and the doc says
what a failed write costs.

The test, `a_process_killed_while_it_merges_never_sets_the_totals_back`,
starts copies of its own test binary that merge in a tight loop. It kills each
copy with `SIGKILL` at a different moment, 200 times, and reads `totals` under
the lock before and after every kill. Against the old write it failed in 20 of
20 runs, each time within the first 25 kills: "copy 8 was killed with `totals`
at 25 runs, and it read 0 after". With the rename it passed 20 of 20.

In the base, any `_Exit` on the main thread could land while the merging
thread was mid-write, so every timeout was a chance. On this lane's tip the
merge runs in a draw, so it takes a timeout that fires during a merge, or an
out-of-memory from libFuzzer's RSS thread. How often it happened on a real
night is §269's answer: about once in five target-nights. This lane only
fixed it.

**What removing the thread costs, sized.** §262 found that the merging thread
went on ticking through a hang (§262.2), so a timeout cost the probe about six
runs (§262.7). Without the thread, a child that times out loses the runs since
its last merge, up to a second of them. On 2026-09-26
(`~/rcad-fuzz-logs/2026-09-26/fuzz.log`: `-fork=4 -timeout=10
-max_total_time=28715`), `where_two_surfaces_cross` ran 592 047 inputs and
timed out 951 times. Take the 951 × 10 s of hanging out of 28 715 s × 4
children, as §262.7 does, and 105 350 child-seconds are left: 5.62 runs a
second. So the new loss is at most 951 × 5.62 ≈ 5 345 runs, 0.90% of the
night's runs. The cone and fuse targets timed out once each, at 19.5 and 19.9
runs a second (the last `#N` over its time × 4), so about 20 runs each. The
reader and writer did not time out, so 0. These are the review's figures, and
I re-derived them from the night's logs. 5.62 is an average over the night.
If the hangs tend to follow bursts of fast draws, the loss is larger, and that
was not measured.

**The thread's share of the leak re-runs was not separated.** Item 3 of 267.3
puts `#902` against `#835` on the uninstrumented build: 67 extra runs.
`offer`'s first 64 copies would account for 64 of them, which leaves about 3
for the thread. That fits both causes. It is not a measurement of either.

§259.5's pair was not rerun on this tip. The three changes are in the merge,
which runs inside a draw. The set frees what it allocates before the draw
ends, and the rename adds no string comparison. That is argued from the code,
not measured.
