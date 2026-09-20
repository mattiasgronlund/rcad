## 98. A night that crashed on its own oracle, fixed 2026-09-14

`~/rcad-fuzz-logs/latest` read

```text
2026-09-14 08:10  9b82e9e (0 days old, 0 behind main)  4 targets, 1 TO LOOK AT (fuzz:crash)
```

and the crash was real, reproducible on `main`, and **in the test harness**.
The other three targets ran clean: 112 097 627 runs of the reader, 293 178 960
of the writer, 4 292 040 of the fuse.

§91 is the night before this one, and it is the opposite shape. There the night
lost a finding it had. Here the night had a finding that was not one — and the
way to tell those apart is the same either way, which is to reproduce the thing
before believing the label on it.

### 98.1 The pair, and why both answers were right

`writing_a_pair_at_another_power_of_two_does_not_change_it` panicked: two
leaning pipes that cross in one closed loop as written, and in nothing at `2⁻¹`.

```text
Cylinder { axis: { origin: (-8, -1, 251659264), direction: (0, 0, 1) },
           radius: 1.0737299303163968e128 }
Cylinder { axis: { origin: (R, R, R), direction: (0.6739228415507578, 0, 0.7388017349980658) },
           radius: 1 }
```

The second direction has **no `y` component at all**, so every point of that
axis sits at `y = R`, the first axis sits at `y = -1`, and the distance between
them is exactly `R + 1`. `where_two_leaning_pipes_cross` asks `ρ₁ + ρ₂ − δ`
first, and with `ρ₁ = R` and `δ = R + 1` that whole question reduces to

```text
ρ₂ − 1  =  1/√(a²+c²) − 1
```

for the second direction's `a` and `c`. Exactly, in rationals:

| spelling | `a` | `a²+c² − 1` | so |
|---|---|---|---|
| as the fuzzer decoded it | `0.6739228415507578` | `−1.87e-17` | `ρ₂ > 1`, one closed loop |
| after the harness rebuilt it | `0.6739228415507579` | `+1.31e-16` | `ρ₂ < 1`, nothing |

**Both certified answers are right about the pipes they were handed.** The two
spellings are not one pair written at two scales; they are two pairs, one bit
apart, either side of a knife edge. Nothing in the kernel is wrong here, and the
`Ok` on a margin of `1e-17` against coordinates of `1e128` is the exact level
doing what §29 built it for.

### 98.2 `Axis::new` is not idempotent, and a test that rebuilt an axis paid for it

The harness scaled a surface by rebuilding it, and rebuilt the axis with
`Axis::new(scaled_origin, axis.direction())`. `Axis::new` scales the direction
it is given to length one — which is not a no-op on a direction that is already
one, because it divides by the largest component and normalizes again, and both
round. Measured over 100 000 random directions: **24 750 come back with other
bits**, pinned by `rebuilding_an_axis_from_its_own_direction_moves_it_and_moving_its_origin_does_not`.

**This is a footgun and not a broken promise, and the difference matters.**
`Axis::new` claims that two vectors *exactly* along one line give the same axis
bit for bit, and §22.6 is why. That claim holds —
`an_axis_is_the_same_however_long_the_direction_naming_it_was` and the 210 000
pairs beside it already say so. A stored direction is the rounded normalizing of
whatever built the axis, so it lies exactly along nothing, and the claim has
nothing to say about it.

Nothing in `src/` rebuilds an axis this way. `Axis::transformed` was written
against exactly this and says so: *"a read that moves bits is a read nobody can
do twice."* What was missing was the same thing for a scaling, which is not an
isometry, so `Axis::moved_to` is it — the origin moves, the direction is carried
across untouched, and there is nothing left to round.

### 98.3 What a factor of one proves

`scaled(pair, 2⁰)` — a factor of **one**, no scaling at all — already turned
`Curves` into `Nothing`. That is the whole diagnosis in one call, and it is what
says the oracle and not the arithmetic: a scaling helper that changes an answer
at factor one is not testing scaling.

The seed is checked in as
`fuzz/corpus/where_two_surfaces_cross/d04e004125fd3d455bd578e6ccfe36949b05bf9e`,
so `every_seed_in_the_checked_in_corpus_names_its_crossing_honestly` replays it
in the ordinary suite. Without the fix it panics there in the same words the
night did; with it, 831 seeds pass.

**The oracle was weakened by this and is not weakened by the fix.** It was
comparing the pair against a rescaled *and* slightly tilted copy, so every
disagreement it could ever report was a disagreement about one of two things and
it could not say which. It now compares the pair against the pair.

### 98.4 The night was running a justfile from two days earlier

The same night kept none of the six artifacts it found. `latest` named the crash
and no artifact count, and the only copies sat in the gitignored
`fuzz/artifacts/` — which is the exact loss §91 added the copy-out to stop, two
days after it landed.

`just` hands bash the whole recipe body before the first line of it runs, and
`fuzz-overnight` then checks out the watermark. The reflog of the nightly
worktree:

```text
6e71fe6  checked out 2026-09-11 22:04:17, and held there all of 2026-09-13
9b82e9e  checked out 2026-09-14 00:10:02, by the night that was already running
```

`34ba0ad`, which added the copy-out, was committed 2026-09-13 11:52. So the body
running that night had been read from a commit that predated its own fix. **Every
change to this recipe would have taken effect the night after it landed**, which
is a cost nobody had priced because nobody had noticed paying it.

The recipe now re-execs itself after the checkout, so a night runs the justfile
it is fuzzing. **A night advances once**: the commit it lands on is the newest
green there was a moment ago, so the recipe taking over would almost always find
nothing to move to and stop by itself — but "almost always" is a bound on CI's
timing and not on the loop, so `RCAD_FUZZ_ALREADY_ADVANCED` makes it a bound on
the loop. Once is also the honest reading of §90: the night follows the last
green commit as of when it started.

Two arms of `check-fuzz-overnight-selftest` hold it. The fixture's two commits
disagree on purpose — the night starts on the real recipe and advances to one
whose `fuzz-overnight` does nothing but name itself — so the marker in the
output is which body ran, and the arm reads the exit status last, because a
night that carried on with the wrong body goes on to build and fuzz and then
fails or passes for reasons of its own.

### 98.5 A crash that had already been fixed, sitting where nothing would re-find it

`fuzz/artifacts/where_two_surfaces_cross/` also held
`crash-cba396f88aa1063b9d879100bded7b9fcd0d5f95`, dated 2026-09-12. It replays
clean on `main`: fixed in the two days since, and never re-found, because
libFuzzer does not read that directory back as a seed. Nobody was told either
way. Everything there is now copied to
`~/rcad-fuzz-logs/artifacts-rescued-2026-09-14/`.

