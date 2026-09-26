## 257. Why two runs of the old writer fuzz target differ so much — 2026-09-26

*Queue row Q71, from §241.3: the old writer target's "came back the same"
count swung from 1526 to 37 between two 60-second runs on the same seeds, with
163 against 781 panics. This section names the cause, measured, and checks
whether the current target has the same spread.*

  257.1   Rebuilding the old target
  257.2   Same seed, same corpus, still not the same run
  257.3   Ruling out a bad corpus copy or a nondeterministic writer
  257.4   The cause
  257.5   The current target does not have this spread
  257.6   What is left

### 257.1 Rebuilding the old target

`fuzz/corpus/no_model_makes_the_writer_panic` holds 1183 files at both
`b94047f4^` (the old target §241.3 measured) and at `b94047f4` itself (`git
ls-tree -r <sha> --name-only -- fuzz/corpus/no_model_makes_the_writer_panic |
wc -l`, run at both shas) — the seven shape seeds §241.4 describes landed in a
later commit, `ed5349f7`, so both halves of §241.3's table read the same 1183
files.

The old target was rebuilt from `b94047f4^` in a scratch worktree (`git
worktree add --detach <scratch>/old-target b94047f4^`), with `cargo +nightly
fuzz build no_model_makes_the_writer_panic` under the same
`RUSTFLAGS="-C target-cpu=x86-64-v3"` the `fuzz-target` recipe uses. To get the
same live counts §241.3 printed, the target file (in the scratch worktree
only, never committed) was given the same kind of counter §241.3 describes: a
panic hook that counts instead of aborting, and a background thread that
prints `runs`, `reached`, `with_face`, `same` and `panicked` every 250 ms.
`writer::one_model_decoded_and_written` already returns a `Reached` struct
with exactly these fields, so the counter just sums it and does not touch the
crate under test.

### 257.2 Same seed, same corpus, still not the same run

Two runs of this rebuilt binary, from a fresh copy of the same 1183 seeds each
time, with an **explicit, identical** `-seed=42` and an identical
`-runs=60000` (a fixed count, not a time limit, so load cannot be a factor
here):

| | cov at `#1184 INITED`, before any mutation | final cov / ft | corpus grown to | new units added | panicked (last tick) |
|---|---|---|---|---|---|
| Run A1 | 8432 / 14646 | 8737 / 20077 | 1193 | 57 | 6 |
| Run A2 | 8441 / 14654 | 8696 / 18223 | 1188 | 68 | 2 (last tick before `DONE`; §241.3's own caveat that the printed counter misses at most the last quarter second applies here too) |

Both runs read "1183 files found" from their own corpus copy and hit
`#1184 INITED` before either one has mutated anything. The two `cov`/`ft`
numbers there already differ — from processing the same 1183 files, with the
same explicit seed. So an explicit `-seed=` does not fully pin a run: holding
the seed and the corpus fixed narrows the gap a great deal (nothing like a 41x
swing) but does not close it.

Commands run, in order: `cp -r` the 1183-file corpus into two fresh
directories; each run as `<binary> <its fresh corpus dir> -seed=42
-runs=60000 -print_final_stats=1`, output redirected to a log file and read
back with `Read`.

### 257.3 Ruling out a bad corpus copy or a nondeterministic writer

Two things that could produce 257.2's gap without saying anything about
libFuzzer's own randomness:

- **The two corpus copies might not really be identical.** `diff -rq` between
  the pristine 1183-file master and a fuzzed copy, filtered to the files both
  share, found no content differences — `cp -r` gives byte-identical trees,
  and the fuzzer only adds files, never edits the seeds it started from.
- **`rcad_step::write` might not be a pure function of its input.** A small
  standalone binary (`fuzz/probe.rs`, added as a scratch-only `[[bin]]` in
  `fuzz/Cargo.toml`, never committed) decoded every file in a corpus
  directory with `postcard::from_bytes::<rcad_topo::Model>`, and for every one
  that decoded, called `rcad_step::write` **twice in the same process** and
  hashed both outputs. Over the full 1183-file corpus: `decoded=19
  with_face=3 written=2 within_process_mismatches=0` — every one of the 19
  decodable seeds writes the same bytes twice in a row, every time. The one
  seed with a face that does write (`two_cubes_that_write`) was also run
  through the plain `probe` binary as **two separate process invocations**,
  not just twice in one process, and got the same hash both times
  (`len=10433 hash=a716e04174c7b17d`, both processes, both calls). So the
  75 non-decoding, non-writing files are not where the gap comes from, and
  neither is `write()` disagreeing with itself on the models the corpus can
  already reach.

Neither check found a defect in the writer or in the corpus. The gap in 257.2
is real, and it is not explained by either of these.

### 257.4 The cause

Two things, one measured and dominant, one measured and real but small:

1. **§241.3's own four runs passed no `-seed=` flag at all.** Unlike 257.2's
   runs, none of the four commands in §241.3 name a seed, so libFuzzer chose
   its own for each of the four processes, independently — the same way it
   chooses one whenever `-seed` is left out. Two such choices are not
   correlated with each other.
2. **The old mutator's hit rate is tiny.** §241.3 already states it: "one or
   two in a hundred" of the old target's draws reach the writer at all, next
   to nine in ten for the new one. A count built from a rate that low, over a
   fixed 60 seconds, is a small sample of a rare event — how many "came back
   the same" or "panicked" outcomes a run turns up depends on exactly *which*
   of the millions of possible byte mutations that run's own, unpinned seed
   happens to try in that minute. 257.2 shows this kind of seed sensitivity
   directly: even an **explicit, shared** seed on a **byte-identical** corpus
   for a **fixed** number of runs still lands on a different set of
   mutations (different `cov`/`ft` from the same 1183 files, different
   `new_units_added`, panicked 6 against 2). An unpinned seed, drawn fresh by
   each of the four original processes, only widens that same effect — §241.3's
   1526-vs-37 and 163-vs-781 are what that widening looks like on a rare-event
   count.

   What §257.2 does **not** explain is *why* an identical seed and an
   identical corpus still diverge at all — 257.3 ruled out the two most
   obvious reasons (a bad copy, a nondeterministic writer) but did not find
   what does cause it. That residual is real, smaller than the seed-choice
   effect, and unexplained; see 257.6.

The row's own load numbers (8.9 at run 1's start, 27.4 at run 4's, from
§241.3) also cut run 4's total attempts by about 23% (145,300 against 187,938
— `145300 / 187938 ≈ 0.773`, both figures from §241.3's table, not re-run
here) — a real effect on top of the above, shrinking run 4's sample further,
but on its own a 23% cut in sample size cannot turn 1526 into 37: the swing
is fundamentally a seed effect, not a load effect.

### 257.5 The current target does not have this spread

§241.3's own table already has two runs of the new (custom-mutator) target on
the same 1183-seed corpus, rows 2 and 3, not re-run here:

| | Runs | Reached | With a face | Came back the same | Panicked |
|---|---:|---:|---:|---:|---:|
| Row 2 | 12 398 | 11 164 | 11 140 | 1 090 | 161 |
| Row 3 | 13 493 | 12 291 | 12 263 | 1 115 | 174 |
| Row 3 ÷ Row 2 | 1.088 | 1.101 | 1.101 | 1.023 | 1.081 |

Every column moves by 2% to 10% between the two rows — the same size of swing
the load numbers (19.0 against 26.4) would explain on their own — and nothing
like the old target's 41x ("came back the same") or 4.8x ("panicked"). The
reason is 257.4's mechanism running in reverse: §241.1 and §241.3 already
state that about nine in ten of the new target's draws reach the writer,
against one or two in a hundred for the old one. Once the hit rate is that
high, a 60-second count is close to (rate × total draws), and total draws
only tracks the ~9% load difference between the two rows — it is no longer a
small sample of a rare event, so an unpinned per-run seed has far less room to
move the outcome. **The current target does not show the old target's
spread, and the fix is the same one §241 already made** — moving from a
1-2% hit rate to a 90% one — not a further, separate fix.

### 257.6 What is left

- 257.4's residual — an explicit, shared `-seed=42` and a byte-identical
  1183-file corpus still not producing the same run — is measured but not
  explained. It is not a defect in `crates/rcad-step/src/write.rs` (257.3
  ruled that out for every decodable seed in the corpus) and it is smaller
  than the seed-choice effect that explains the row, so no queue row is
  proposed for it here; whoever next touches `fuzz/fuzz_targets/no_model_makes_the_writer_panic.rs`
  or its runner should know libFuzzer's own `-seed` does not, by itself,
  make two runs of it identical.
