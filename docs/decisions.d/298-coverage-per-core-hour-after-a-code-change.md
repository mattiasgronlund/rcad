## 298. Coverage per core-hour on a night after a target's code changed — 2026-09-27

*Row Q214, read-only: `mise exec -- just fuzz-yield 2026-09-20 2026-09-27` against
`~/rcad-fuzz-logs/`, plus `git log`/`git diff` on the separate
`rcad-nightly-fuzz` worktree that the fuzz nights actually build from. No fuzz
binary run. §265.4 found that coverage jumps happen either on an
unusually-fast night or when new code arrives, and §272.7 left "does a night
after a code change gain more per core-hour" as the untested half of Q148's
rule. Eight nights, five targets, forty target-nights: the two targets whose
code is quiet often enough to compare (`where_a_cone_or_a_torus_crosses`,
`where_two_surfaces_cross`) show no rate difference that stands out from their
own night-to-night spread, and the other three targets never had a quiet
night in this window to compare against at all.*

  298.1   Which paths feed each target
  298.2   Which commit each night built from, and how "changed" is decided
  298.3   Per night, per target: changed, coverage gained, core-hours, rate
  298.4   The two targets that had both kinds of night
  298.5   The three targets that never had a quiet night
  298.6   Rebuild noise alone moves the count as much as a whole night's gain
  298.7   Verdict, and how many more nights it would take

### 298.1 Which paths feed each target

Every fuzz target `#[path]`-includes one shared test-suite file rather than
copying its own logic (`docs/decisions.md` §29.9), so "the target's code" means
that file plus the crate(s) it calls into, not just the eleven-to-thirty-line
file under `fuzz/fuzz_targets/`:

| target | its own file | the file it reaches into | the crates that feed it, transitively |
|---|---|---|---|
| `where_two_surfaces_cross` | `fuzz/fuzz_targets/where_two_surfaces_cross.rs` | `crates/rcad-geom/tests/fuzzing/mod.rs` | `rcad-geom`, `rcad-predicates` |
| `where_a_cone_or_a_torus_crosses` | `fuzz/fuzz_targets/where_a_cone_or_a_torus_crosses.rs` | `crates/rcad-geom/tests/fuzzing/mod.rs` (same file as above) | `rcad-geom`, `rcad-predicates` |
| `no_step_file_makes_the_reader_panic` | `fuzz/fuzz_targets/no_step_file_makes_the_reader_panic.rs` | `crates/rcad-step/tests/fuzzing/reader.rs` | `rcad-step`, `rcad-topo`, `rcad-mesh`, `rcad-geom`, `rcad-predicates` |
| `no_model_makes_the_writer_panic` | `fuzz/fuzz_targets/no_model_makes_the_writer_panic.rs` | `crates/rcad-step/tests/fuzzing/writer.rs` | same five crates as the reader |
| `two_solids_fused_by_a_stranger` | `fuzz/fuzz_targets/two_solids_fused_by_a_stranger.rs` | `crates/rcad-topo/tests/fuzzing/fuse.rs` | `rcad-topo`, `rcad-mesh`, `rcad-geom`, `rcad-predicates` (no `rcad-step`) |

The crate lists come from each crate's own `Cargo.toml`
(`rcad-geom` depends on `rcad-predicates`; `rcad-mesh` and `rcad-topo` each add
`rcad-geom` and `rcad-predicates`; `rcad-topo` adds `rcad-mesh`; `rcad-step`
adds `rcad-topo`, pulling in the rest). **"Changed" here means the transitive
set, not just the crate a target's doc comment names**, because `cargo fuzz
build` recompiles a target's binary whenever any dependency's source changed,
and a recompiled dependency is exactly the kind of "new code arrived" event
§265.4 already reads the between-night coverage steps as. A narrower
"only the crate the harness names" definition was considered and rejected: it
would call a night "unchanged" for the reader while `rcad-geom` — the crate
every cross-surface oracle the reader's STEP models are built from calls into
— visibly changed underneath it, which is not a night where the reader's
binary is the same binary.

`rcad-fuzz-timing`, which every target links for its `timed()` wrapper, is
tracked separately and not counted toward any target's own "changed": §272.7
already names it as ~570-600 edges of shared instrumentation noise, common to
all five targets alike, so a night where only it moved would not discriminate
between targets even if it were counted.

### 298.2 Which commit each night built from, and how "changed" is decided

The nightly fuzz run does not build in this worktree; it builds in a
separate worktree of the same repo, `/home/mattias/git/mattiasgronlund/rcad-nightly-fuzz`,
whose `.git` points at `rcad/.git/worktrees/rcad-nightly-fuzz`. Its reflog
records one `checkout` per night, always around 00:10-00:16, matching each
night's own `build.log` mtime:

| night | commit checked out | vs. the previous night's commit |
|---|---|---|
| 09-19 (baseline, outside the range asked for) | `5336ff31` | — |
| 09-20 | `146d0d62` | `git diff --name-only 5336ff31 146d0d62 -- crates fuzz` |
| 09-21 | `ae124a29` | `git diff --name-only 146d0d62 ae124a29 -- crates fuzz` |
| 09-22 | `79eedce7` | `git diff --name-only ae124a29 79eedce7 -- crates fuzz` |
| 09-23 | `055aa4a9` | `git diff --name-only 79eedce7 055aa4a9 -- crates fuzz` |
| 09-24 | `afdff061` | `git diff --name-only 055aa4a9 afdff061 -- crates fuzz` |
| 09-25 | `6664fef8` | `git diff --name-only afdff061 6664fef8 -- crates fuzz` |
| 09-26 | `c92b8fe5` | `git diff --name-only 6664fef8 c92b8fe5 -- crates fuzz` |
| 09-27 | `a628aa99` | `git diff --name-only c92b8fe5 a628aa99 -- crates fuzz` |

(`git -C /home/mattias/git/mattiasgronlund/rcad-nightly-fuzz reflog --date=iso`
for the commits; each `diff --name-only` run separately.) A target counts as
**changed** for a given night when that night's diff touches `src/` of any
crate in its §298.1 row, or the one shared test file it reaches into, or its
own `fuzz_targets/*.rs` file. It counts as **unchanged** only when none of
those paths appear in the diff — other files changing (a different target's
harness, a `tests/` file no target reaches into, `Cargo.lock`, docs) do not
mark it changed.

### 298.3 Per night, per target: changed, coverage gained, core-hours, rate

Coverage gained is `cov` at end of night minus `cov` at start of night, from
`just fuzz-yield 2026-09-20 2026-09-27`'s own `coverage: … at start, … at 30m,
… at end` line; core-hours is that same run's `fork × window` line, which was
31.9-32.0 every night (fork 4, ~8h window) so it barely moves the rates below.
`rate` is gained ÷ core-hours, arithmetic over those two numbers only, no new
measurement (a scratch `python3` script did the division; it is not checked
in, since it is the one-line formula this section already states).

**`where_a_cone_or_a_torus_crosses`** (feeds: `rcad-geom`, `rcad-predicates`):

| night | changed | gained | core-h | rate/core-hour |
|---|---|---|---|---|
| 09-20 | yes | 582 | 31.9 | 18.24 |
| 09-21 | **no** | 25 | 32.0 | 0.78 |
| 09-22 | yes | 20 | 31.9 | 0.63 |
| 09-23 | **no** | 14 | 32.0 | 0.44 |
| 09-24 | yes | 19 | 31.9 | 0.60 |
| 09-25 | yes | 16 | 31.9 | 0.50 |
| 09-26 | yes | 16 | 31.9 | 0.50 |
| 09-27 | **no** | 2 | 31.9 | 0.06 |

**`where_two_surfaces_cross`** (same feed crates as above, independent harness call but the same shared file):

| night | changed | gained | core-h | rate/core-hour |
|---|---|---|---|---|
| 09-20 | yes | 50 | 31.9 | 1.57 |
| 09-21 | **no** | 8 | 32.0 | 0.25 |
| 09-22 | yes | 25 | 31.9 | 0.78 |
| 09-23 | **no** | 24 | 32.0 | 0.75 |
| 09-24 | yes | 16 | 31.9 | 0.50 |
| 09-25 | yes | 15 | 31.9 | 0.47 |
| 09-26 | yes | 10 | 31.9 | 0.31 |
| 09-27 | **no** | 3 | 31.9 | 0.09 |

**`no_step_file_makes_the_reader_panic`**, **`no_model_makes_the_writer_panic`**
and **`two_solids_fused_by_a_stranger`** were **changed on all eight nights**
(their feed set includes `rcad-topo`, and its `src/` moved in seven of the
eight diffs; the eighth, 09-20, still moves `rcad-geom/src/`, which all three
also depend on). Their per-night gained/core-hour, for completeness:

| night | reader rate | writer rate | fuse rate |
|---|---|---|---|
| 09-20 | 1.13 | 0.94 | 0.75 |
| 09-21 | 0.53 | 0.69 | 0.69 |
| 09-22 | 1.97 | 0.19 | 21.19 |
| 09-23 | 0.66 | 1.00 | 4.94 |
| 09-24 | 1.16 | 0.13 | 3.26 |
| 09-25 | 0.50 | 0.50 | 1.03 |
| 09-26 | 0.72 | 41.88 | 2.45 |
| 09-27 | 0.60 | 4.36 | 1.22 |

(09-22's 21.19 for `fuse` and 09-26's 41.88 for `writer` are the two events
§265.4 already names: 09-22 was the fuse target's unusually fast night
(36.7M runs against its usual 2-4.6M), and 09-26 is the writer target's
rewrite, §241 — both large, both already attributed to something other than
an ordinary code change.)

### 298.4 The two targets that had both kinds of night

Only `where_a_cone_or_a_torus_crosses` and `where_two_surfaces_cross` had any
unchanged nights in this window (three of eight each: 09-21, 09-23, 09-27),
because they are the only targets whose whole feed set is `rcad-geom` +
`rcad-predicates`, and those two crates went eight-of-eight and five-of-eight
nights touched respectively, against `rcad-topo`'s seven-of-eight.

- **`where_a_cone_or_a_torus_crosses`**: mean rate on changed nights **4.09**/core-hour
  including 09-20, **0.56** excluding it (09-20 is itself the run-speed outlier
  §265.4 names, so counting it would credit the code-change signal with a gain
  that section already explains a different way). Mean rate on unchanged
  nights: **0.43**, over three nights ranging 0.06 to 0.78.
- **`where_two_surfaces_cross`**: mean rate on changed nights **0.73** including
  09-20, **0.52** excluding it. Mean rate on unchanged nights: **0.36**, over
  three nights ranging 0.09 to 0.75.

Once the one run-speed night is set aside, changed-night means (0.56, 0.52)
sit inside the unchanged-night nights' own range (0.06-0.78, 0.09-0.75) for
both targets — the "changed" mean is not outside the spread that "unchanged"
nights alone already show. Three unchanged nights and four-or-five changed
nights (per target) is not enough to call that a real difference either way;
it is consistent with no difference and equally consistent with a small one
hidden in the spread.

### 298.5 The three targets that never had a quiet night

`no_step_file_makes_the_reader_panic`, `no_model_makes_the_writer_panic` and
`two_solids_fused_by_a_stranger` have **zero** unchanged nights in the whole
eight-night window (§298.3), so this rule's other half — whether a changed
night beats an unchanged one — cannot be measured for them at all here, not
even weakly. It is not that the evidence is thin; there is none. Getting even
one comparison point for these three needs a night where none of
`rcad-topo/src`, `rcad-mesh/src`, `rcad-geom/src`, `rcad-predicates/src` (all
three), plus `rcad-step/src` (the reader and writer only) moved — and in this
sample `rcad-topo/src` alone moved on seven of eight nights. At that observed
rate a quiet night for these three is rare enough that no number of *more*
nights can be named with confidence; it is a "keep watching" rather than a
"N more nights" answer.

### 298.6 Rebuild noise alone moves the count as much as a whole night's gain

The coverage count printed at the *start* of a night is not always the same
as the *end* count from the night before, even though the corpus and binary
in between are the same fuzzer instance restarting — because a new commit
means a new binary, and a new binary's `SanitizerCoverage` edge table is not
guaranteed to number, or even count, the same edges as the old one, especially
when a caller elsewhere in the workspace changes what a generic function in a
shared crate gets monomorphized for (a plausible mechanism, not confirmed by
tracing an actual case — flagged here as a hypothesis, not a measured cause).
The size of that jump, read directly off consecutive `coverage: … at start` /
`… at end` lines:

| target | biggest overnight boundary swing | for comparison, mean changed-night gain (excl. the named outliers) |
|---|---|---|
| `where_a_cone_or_a_torus_crosses` | −125 (end of 09-26 → start of 09-27) | 0.56/core-hour × ~32h ≈ 18 |
| `where_two_surfaces_cross` | −136 (end of 09-26 → start of 09-27) | 0.52/core-hour × ~32h ≈ 17 |
| `no_step_file_makes_the_reader_panic` | +153 (end of 09-25 → start of 09-26) | ~30 |
| `two_solids_fused_by_a_stranger` | +1176 (end of 09-25 → start of 09-26) | dwarfed by §265.4's own 09-22 outlier |

For the two targets this row can actually compare (§298.4), a single
rebuild's swing (125-136 edges) is *larger* than nearly every whole night's
own gain (2-50 edges, §298.3) — bigger than any unchanged night's gain and
bigger than most changed nights' too. Whatever this swing is measuring, it is
not smaller than the effect Q214 asks about, which is on the same table.

### 298.7 Verdict, and how many more nights it would take

For the two targets where both kinds of night exist, the changed-vs-unchanged
gap (§298.4: 0.56 vs 0.43, and 0.52 vs 0.36) is smaller than the spread each
group already shows on its own (0.06-0.78 and 0.09-0.75), and smaller than a
single rebuild's own coverage-count swing (§298.6). **It does not stand out
from the night-to-night spread by enough to steer a fork count on this
evidence.** For the other three targets, there is no unchanged night in this
window to compare against, so the rule's "code changed → more coverage" half
is simply untested there, not weakly supported or weakly against — §272.7's
"on one commit it never fires" turns out to describe seven of these eight
nights for those three targets, not one.

More nights would help the two geom-only targets (three unchanged nights is
too few to trust a 0.43 vs 0.56 gap either way; a dozen or more of each kind,
which at the observed ~3-in-8 unchanged rate is several weeks out, would be a
reasonable floor before trusting a mean). For the other three targets, no
number of nights can be committed to in advance, because the limiting factor
is not how many nights pass but how often `rcad-topo/src` and `rcad-geom/src`
stay untouched for a whole night — an event this sample saw zero times in
eight tries. The honest next step is to keep the `git diff --name-only`
classification in §298.2 running alongside future `fuzz-yield` reads (or fold
it into that recipe) and revisit once a quiet night for those three actually
occurs, rather than fund a wait for one now.
