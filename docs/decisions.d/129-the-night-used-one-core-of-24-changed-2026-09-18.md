## 129. The night used one core of 24, changed 2026-09-18

*§81 built the night to fuzz five targets in turn, a fifth of the window each.
That is 8 core-hours of the 192 a night has, and the other 184 were idle. They
now run together, each for the whole window. Nothing was shared between them, so
the only thing sequencing ever bought was politeness — and `Nice=10` was already
buying that.*

### 129.1 What a night actually spent

The night of 2026-09-18, off `~/rcad-fuzz-logs/2026-09-18/`:

| target | runs | exec/s | edges at the start → at the end |
| --- | --- | --- | --- |
| `no_model_makes_the_writer_panic` | 98 739 488 | 17 196 | 3020 → 3137 |
| `no_step_file_makes_the_reader_panic` | 45 555 371 | 7 932 | 4420 → 4564 |
| `where_a_cone_or_a_torus_crosses` | 29 490 653 | 5 136 | 2286 → 2407 |
| `two_solids_fused_by_a_stranger` | 2 972 569 | 517 | 15 908 → 15 986 |
| `where_two_surfaces_cross` | 259 138 | 45 | 2892 → 2914 |

Five slices of about 5 740 s, one after another. The machine has 24 cores and
125 GB, and the peak resident of any one target was 689 MB.

### 129.2 What sequencing was buying, and what it cost

Nothing is shared between the five. Each has its own corpus directory under
`RCAD_FUZZ_CORPUS`, its own log, and its own `RCAD_FUZZ_TIMINGS/<target>/`. The
five `cargo fuzz run`s serialise for a few seconds at startup on cargo's build
directory lock, each doing the no-op build that `fuzz-overnight`'s own
`cargo fuzz build` already did, and that is the whole of the interference.

So the argument for one at a time was never about correctness. It is written
down in `rcad-fuzz.service`: *"One target at a time is one core of 24, but it is
a core somebody else may want more."* Politeness, by arithmetic. But the same
file already sets `Nice=10` and `IOSchedulingClass=idle`, and children inherit
both — so a lane compiling at three in the morning wins every core it asks for
whether the night is running one target or five. The arithmetic was buying a
second copy of something the scheduler was already doing.

**What it cost is the other 184 core-hours.** Those two lines in the service file
carried almost nothing for nine nights and carry the whole argument now, which is
why the comment beside them was rewritten rather than left to read as a note
about which core.

### 129.3 Wall-clock time cannot say whether they ran together

`check-fuzz-overnight-selftest` runs the whole recipe against a stub `cargo` in
about thirty seconds. Its first arm bounded how long a twelve-second window took,
and that bound is satisfied by both recipes: five targets sleeping half of a
fifth each takes about 9 s, and five sleeping half of the whole *together* takes
about 6 s. A night is as long as its slowest target however the rest are
arranged. **That is exactly how one core of 24 went unquestioned for nine
nights** — every assertion about the window held, and none of them was about the
machine.

Two arms now say it exactly, and neither reads the clock:

- **What each target was told.** Arm one takes every `-max_total_time=` off the
  stub's log, and asserts there is one distinct value and that it is the window
  rather than a fifth of it. Against the recipe this replaced it fails naming the
  five shares: `2 3 4 7`.
- **When each target started.** The stub logs `[at=<epoch>]`, and arm four
  asserts the five starts are within 3 s of each other. This is the one that
  survives a half-fix: a recipe that starts one target, waits for it, and then
  runs the other four together passes arm one and every clock bound there is, and
  fails here at `8 s apart`.

Arm four also changed sides. It used to assert that an overrun in the first
target spent the window and the other four were skipped — correct of a recipe
that ran them in turn, and the single worst property of that recipe. The same
fixture now asks whether an overrun costs the other four anything, and the old
assertion is what fails.

### 129.4 The copy-out lost the boundaries it was riding on

§91 copies each target's artifacts out of the gitignored `fuzz/artifacts/` and
into the night's own directory, because that is where findings go to be lost. It
did it after each target, which gave a night killed from outside four copy points
spread across it, about ninety minutes apart.

Run together, the five finish within seconds of each other at the far end of the
window. Those boundaries are gone, and a copy that waited for them would keep
nothing from a night killed at 04:00 — **strictly worse than what it replaced**,
and invisible until the night it mattered. So the copy is on a clock now, every
300 s, with the interval read from `RCAD_FUZZ_COPY_EVERY` so that an arm can
afford one. Arm ten sets it to 1 s and looks four seconds into a twelve-second
window, when the targets are still six seconds from returning.

That arm was written wrong the first time and the mutant caught it: with nothing
copied the directory does not exist, `find` exits 1, `pipefail` takes the recipe
down, and the test failed correctly and said nothing about which arm or why. It
is guarded now.

### 129.5 What is not done, and why this had to be first

Four changes were ranked by how much more a night would find. This is the second
of them, and it is first in the tree because the order they were ranked in is not
the order they can land in.

1. **A crash ends that target's night.** `cargo fuzz run` exits on the first
   crash, so five is the most a night can ever report and the first crash in a
   target hides everything behind it. `-fork=N -ignore_crashes=1` keeps fuzzing
   and collects each one. It needs `rcad-fuzz-timing` fixed first: it writes
   `RCAD_FUZZ_TIMINGS/<target>/report` with no pid in the path and prunes
   `slowest/` of entries the current top no longer names, so N fork children
   would stomp each other's report and delete each other's slow units.
2. **`where_two_surfaces_cross` is strangled by its own slow inputs.** p50 is
   0.009 ms and p100 is 19 327 ms; the 64 kept slowest average 17.3 s, which is
   1 107 s in 64 runs, and the 5 279 runs at or over p98 took at least 77% of the
   slice. It moved 22 edges in eight hours. No `-timeout` is passed, so
   libFuzzer's default of 1200 s applies and the night exits 0 over a 19-second
   predicate — §91.3 already settled that this shape is a defect.

   **`-timeout=10` cannot go in on its own**, and this is the reason the ranking
   and the landing order differ. `fuzz/corpus/where_two_surfaces_cross/` carries
   six `slow-unit-*` seeds, landed deliberately by §91.3, one of them costing
   63 s. libFuzzer would die on the initial corpus replay, before fuzzing a
   single new input, and the target would do zero runs a night. It is free once
   fork mode is in, where timeouts are ignored and collected rather than fatal.
3. **Three of the five targets assert little beyond "did not panic".** The
   writer is the highest-throughput target there is — 98.7 M runs a night — and
   never reads back what it wrote. `write` → `StepFile::read` → `Import::read`,
   compared against the model that went in, turns it into a correctness oracle,
   and both halves are already in the tree. The fuse asks three questions (§95.6)
   and none of `fuses_that_must_agree.rs`'s algebraic laws.

   **Amended by §249.6, §249.7 and §249.8.** "Compared against the model that went in"
   first meant volume, surface area and centroid, bit for bit, and that is not
   an answer every sound model can give: a curved solid placed with a turn is
   meshed in the frame it is stored in on the way out and in the frame the
   reader bakes it into on the way back, two different roundings of the same
   true surface, so the measurement can differ by a few ulps with neither side
   at fault (§249.5, a limit). `read_back_and_compared`
   (`crates/rcad-step/tests/fuzzing/writer.rs`) now checks every located
   number bit for bit for every solid — each face's surface, and for every
   coedge its two corners and its edge's run, in walk order (§249.1's own
   comparison, which a turned bar passed in every number: §249.2) — and
   keeps the bit-for-bit measurement only for a solid that sits wholly on
   planes or carries no placement at all. A move with no turn is still a
   placement (§249.4 item 2), so only a curved solid at the identity keeps
   the measurement check. `two_turns_that_write_the_same_placement.rs` pins
   the limit down: two turns that take the three axes to the same nine
   numbers, and so would be written as one STEP placement, still leave a bar
   measuring differently. Today's writer does not write that placement; it
   turns every corner into the file, and writes the two bars' corner one ulp
   apart.

   `read_back_and_compared` also keeps the bit-for-bit measurement only where
   every circle a flat face carries, drawn by a pipe, comes back walked round
   the centre the kernel walks it round (§249.8, a gap and closed). The
   writer names such a circle's pipe beside it, as a `CYLINDRICAL_SURFACE`,
   rather than leaving the reader to search the file for one that walks
   there.

### 129.6 Corrections to §81

- §81's opening says the night "fuzzes all four targets in turn inside a fixed
  window". Two numbers in one sentence are now wrong: it is five targets since
  §96, and they no longer run in turn.
- §81.2 stands unchanged. The window is still a deadline and not a budget; what
  changed is that the deadline is shared rather than divided.
- §81.3 — "a crash does not end the night" — stands, and is worth reading beside
  129.5's first item. A crash still ends *that target's* night. What §81.3
  bought is that it does not end the other four's, and that is a smaller claim
  than the heading reads.

Commands run to reach this: `ls ~/rcad-fuzz-logs/` and direct `Read` of
`2026-09-18/`'s five logs, its `timings/*/report`, and `latest`; `nproc`;
`free -g`; `git log --oneline -25 -- fuzz/ crates/rcad-fuzz-timing/ justfile`;
`mise exec -- just decision 81`, `91.3`, `95`; `mise exec -- just decisions-find
fuzz`, `slowest`; direct `Read` of the five `fuzz_targets/`, of
`crates/rcad-{geom,topo,step}/tests/fuzzing/`, of `crates/rcad-fuzz-timing/src/lib.rs`
and of `tooling/systemd/`; `mise exec -- just lanes`;
`mise exec -- just check-fuzz-overnight-selftest` against the tree and against
three mutants — the old sequential loop, a staggered start, and the copier taken
out; `mise exec -- just gate`.

