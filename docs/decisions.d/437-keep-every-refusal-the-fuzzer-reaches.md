## 437. Keep every refusal the fuzzer reaches: the morning replay, and limit or gap at the refusal — 2026-10-11

*Mattias, 2026-10-11, in a conversation about predicting which fuses will fail:
"we would actually like to remove all refusals, so we should not throw any of
them away." Then: "We should run the replay probe and write the decision
section, and then dig deeper into 'what a lane spends its first hour finding
out'!" Decided in the `dispatch-2026-10-03` session, across three contexts.
Every figure below was read off the output named beside it.*

In short: nothing new is collected during the night. The morning after, beside
the crash replay of §410, the fuse fuzzer's corpus is replayed through the
harness, and every refusal is counted into a group: its variant, and for
`Undecidable` the predicates that gave up. The groups are written into the
night's directory, and no rows are filed from them yet. Filing waits until a
refusal says, in code, whether it is a limit, a gap, or not known. That is
decided in the crate that owns the error, and every limit or gap it claims has a
test. No cargo feature, nothing inside `fuse`, and no trained model.

  437.1   What is thrown away today
  437.2   The probe
  437.3   What the night's corpus holds
  437.4   The key: the variant, and the predicates that gave up
  437.5   Report only, for now
  437.6   Limit, gap or not known, said where the refusal is made
  437.7   What is not built
  437.8   What a lane reads before its first edit
  437.9   What the probe met on the way
  437.10  Rows

### 437.1 What is thrown away today

The fuse fuzz target, `two_solids_fused_by_a_stranger`, runs each input through
`two_models_split_and_fused` (`crates/rcad-topo/tests/fuzzing/fuse.rs`). A
refusal is counted in `Reached::refused` and then dropped. libFuzzer keeps an
input in its corpus for the code it reached, and keeps a file in `artifacts/`
only for a crash. §410 and §418 turn crash groups into rows. Nothing turns a
refusal into anything. The gate's replay of the checked-in corpus,
`every_model_the_fuse_fuzzer_kept_is_fused_again.rs`, asserts only a floor:
`refused >= 195`.

The refusals are not lost, though. The inputs that reach them are in the corpus.
So keeping them needs a replay, not a new collector.

### 437.2 The probe

A scratch crate outside the repo included `crates/rcad-topo/tests/fuzzing/fuse.rs`
by `#[path]`, as `crates/rcad-fuzz-explain/src/lib.rs` does. Each input went
through `two_models_split_and_fused_noting_refusals`, the harness itself. Then it
went through a copy of the harness's draw: the cut into two halves, the split of
the solids at `header % (solids + 1)`, the rule from `(header >> 4) % 3`, the
policy from `header & 0x80`, and, for a two-sided union or intersection, the
second call with the solids the other way round. The copy labels each refusal
with its half, rule, policy and order. A file where the copy's refusals differ
from the harness's (variant, wording, predicates) is written `MISMATCH`. **No
file did**: 0 of the 1 753 checked in, and 0 of the night's 15 546.

Panics were caught per file. An input still running after 120 s was written
`HUNG` and left running, and the run ended with `process::exit`. Without that,
the first run sat for ten minutes on two inputs (437.9). Built in the `test`
profile, the one `fuzz-triage` uses, on `main` at `459e76d4`.

The probe lived in a scratchpad and is gone with it. The paragraphs above are
enough to build it again, and the first row of 437.10 builds the real one.

### 437.3 What the night's corpus holds

The night's corpus is `~/rcad-fuzz-corpus/two_solids_fused_by_a_stranger`,
libFuzzer's first and writable corpus directory. The checked-in
`fuzz/corpus/two_solids_fused_by_a_stranger/` is the second, read-only one. Only
8 of its 1 753 names are in the night's corpus. The 2026-10-12 night was running
while the probe ran, so the night's corpus grew between runs.

**Checked in: 1 753 files, 3.4 s on 4 workers.** 268 refusals in 209 files: 222
the first way round and 46 the other. 10 variants, 0 panics.

**The night's corpus: 15 546 files, 211 s on 8 workers, at 2026-10-11 23:43Z.**
The files themselves took 95 s. The rest was waiting out two inputs that hang.

- 31 064 halves, 14 108 decoded, 11 744 fused, 8 480 answered.
- **4 737 refusals in 2 990 files**: 3 264 the first way round and 1 473 the
  other; 1 574 under `Policy::certified`, 3 163 tolerant.
- 12 panics and 2 hangs (437.9).
- 18 variants: `Undecidable` 2 032, `ResultIsNotManifold` 890,
  `CannotTellTheCellsOfAFaceApart` 819, `Invalid(DynamicRangeTooWide)` 272,
  `CannotMeshAFace` 188, `ACrossingThisCannotBeOrderedAlong` 168,
  `CurvedFaceWithAHoleGoesTooFarRound` 95,
  `AClosedEdgeOfTheAnswerWouldStartAtAPole` 74, `TwoCurvesTouchWithoutCrossing`
  57, `Invalid(FacesDoNotAgreeWhichWayTheyFace)` 43,
  `TheFuseWroteABrokenAnswer` 19,
  `ACurvedEdgeThatIsPartOfACrossingCannotBeNamed` 17,
  `ACurvedFaceGoesRightRoundWithNoSeam` 16, `Invalid(EdgeHasNoLength)` 15,
  `AWireWillNotLieFlatOnItsFace` 14,
  `Geometry(CannotNameWhichPieceOfACrossingThisIs)` 13,
  `Geometry(TheseTwoSurfacesDoNotCrossInAConic)` 4,
  `Geometry(CannotWorkOutHowBigACrossingIs)` 1.

**Against the refusal-rate families.** `just t -p rcad-topo --test
how_often_a_fuse_with_a_cone_refuses --test how_often_a_fuse_with_a_torus_refuses
--test how_often_a_curved_fuse_refuses --no-capture` (18 passed) prints what each
family refuses. Two of the fuzzer's 40 groups (437.4) are among them: `Undecidable`
at "order along a crossing", and at "a point in a piece that every solid could
place". Two more family groups print only "could not settle 1 decisions", so
which predicate gave up there is not known, and they could be two more. The
torus families reach two refusals the fuzzer never does: a plane that cuts a
torus in a quartic, and a torus against another curved surface. The curved
lattice and turned families refuse nothing. So the families pin a rate for a few
kinds of refusal, and the fuzzer reaches many kinds without pinning any rate.
Neither does the other's job.

*Added 2026-10-12, after Mattias asked how the replay could be so much faster
than the night: the counts above are corpus files, not how often the night met a
refusal. libFuzzer keeps an input only when it reaches code, or takes a branch a
number of times, that no kept input does, and it swaps a kept input for a
smaller one with the same coverage. A refusal reached the same way a million
times is in the corpus once. One reached along many paths is in it many times.
On 2026-10-11 the fuse target ran 7 432 655 inputs in 28 675 s and kept 401: the
last progress line of `~/rcad-fuzz-logs/2026-10-11/fuzz-fuse.log`, and the
corpus going from 15 047 names to 15 448 in `corpus-names/` of that night and
the next. That is also why the replay takes 95 s: it runs each kept file once,
about 480 times fewer inputs than a night, at a similar rate per input. So the
corpus says which kinds of refusal the fuzzer reaches, and a count moves when a
fix lands, but the counts do not rank the groups by how often a user would meet
them. Counting every input during the night would not give that rank either: it
would count what the fuzzer's mutator draws, which is not what users model. What
is collected is not changed.*

### 437.4 The key: the variant, and the predicates that gave up

**What `Error::Undecidable` carries.** `must_decide` and `gave_up`, in
`crates/rcad-topo/src/fuse/mod.rs`, both refuse with `core::mem::take(log)`: the
whole log, not the one record that gave up. Under a tolerant policy the log also
holds every decision the policy settled on the way, each with its outcome turned
to `Decision::Zero` by `resolve_under_policy`. `Record::outcome()` tells the two
apart: a record that gave up is still `Decision::Undecidable`.

Three keys were measured on the night's corpus:

| key | groups |
|---|---:|
| the variant, and every predicate in the records | 92 |
| the variant, and the last record's predicate | 29 |
| the variant, and the predicates of records still undecided | 31 |

Every-predicate splits the same 13 predicates into 92 mixtures, because the
settled records come along. Of the 2 032 `Undecidable` refusals, every one has at
least one record still undecided, and the last record is always one of them.
1 927 have exactly one. Of the 105 with more, 6 have two different predicates
that gave up, `orient3d` beside another, and those 6 make the 2 groups between
the last two keys.

**The key is the predicates still undecided.** It is what the error means. The
last record is not always one that gave up: `Model::fuse` also refuses
`Undecidable` at `mod.rs:306` when every decision was settled but their wobble
added up past the budget, and there the last record is a settled one with
nothing to do with the refusal. That refusal was reached 0 times on this night.
Under this key it is a group of its own, with no predicate, and the report names
it "the settled decisions added up past the budget". Under the last-predicate key
it would be counted under whatever predicate came last.

**Every other variant is keyed by its wording, with every number written N.**
That splits `CannotMeshAFace` five ways (the boundary crosses itself; a corner
in the same place as an earlier one; a corner outside the range a triangulation
takes; a corner on a side; a side straight across a corner on a curved surface),
`TheFuseWroteABrokenAnswer` five ways, and `Invalid(DynamicRangeTooWide)` into a
range of N and a range of `inf`. Each of those is a different refusal behind one
variant, so the split is wanted. **40 groups in all**: 14 for `Undecidable` and
26 for the rest.

### 437.5 Report only, for now

§418 files a triage row when a crash group has no row naming it. Refusals wait,
for two reasons.

- **A replay cannot tell a limit from a gap** (§16.10). A row per group would ask
  a lane to find out what the refusal itself should have said. 437.6 puts that
  into the refusal.
- **New groups are rare.** Of the 40 groups, none had every one of its files
  arrive after the 2026-10-11 night began. That was checked against
  `~/rcad-fuzz-logs/2026-10-11/corpus-names/two_solids_fused_by_a_stranger`,
  the 15 047 names the corpus held when that night began, and against
  2026-10-12's. Under the every-predicate key the earlier context counted 3 new
  groups of 92, each a new mixture of predicates already seen. So the report's
  value is the standing list: 40 groups, their counts, one file to reproduce
  each, and how the counts move when a fix lands.

**What the morning writes.** `fuzz-triage` replays the corpus after it replays
the crash files, and writes `$night/fuzz-refusals.tsv`: one line per group, with
the variant, the key, the refusals, the files, how many were certified and how
many tolerant, how many the first way round and how many the other, and one file
to reproduce it. It prints the groups that appeared or vanished since the night
before. Four things hold, each because the probe showed it needed:

- **One process, with a time limit per input.** Q1721's two inputs do not return
  in 120 s. One hung input must not stop the rest. The probe's shape works:
  workers that can be left behind, a time limit, and `process::exit` at the end.
- **The labels come from the harness's own draw, not a copy.** The probe's copy
  agreed on every file, but a copy is a second draw that can drift from the
  harness. The harness hands its labels to the callback. `fuse.rs` is shared by
  the fuzz target and the gate's replay, so they must still run the same lines
  (§29.9, §95): the callback is told more, and the draw does not change.
- **A panic in the corpus is reported.** The crash replay reads only `crash-*`
  files. A corpus file that panics on `main` is a fault the crash replay cannot
  see, and this one found three (437.9).
- **It lives in `rcad-fuzz-explain`**, beside `--triage`, built in the `test`
  profile for the reason `fuzz-triage`'s own comment gives.

**As built (Q1770).** `rcad-fuzz-explain --refusals <corpus> [--faults <path>]
[--seconds <n>]` replays every file of `<corpus>` on as many worker threads as
the machine runs at once, and prints the groups to stdout as tab-separated text, header first: `variant`,
`key`, `refusals`, `files`, `certified`, `tolerant`, `first_way_round`,
`other_way_round`, `sample`. `--faults` writes each file it could not finish:
`fault` (`panicked`, `hung` or `unreadable`), `file`, and `said`, which for a
panic is its place and its words. A file is given 120 s unless `--seconds` says
otherwise. `fuzz-triage` runs it on `against`, over
`fuzz_corpus_root/two_solids_fused_by_a_stranger`, and copies the two into
`$night/fuzz-refusals.tsv` and `$night/fuzz-refusals-faults.tsv`. It then
compares the groups, by variant and key, with the latest earlier night that has
a `fuzz-refusals.tsv`, and prints an `appeared:` or `vanished:` line for each
group that came or went. Three things the paragraphs above left open:

- **It runs before the fold, and on a night with no crash file.** The corpus is
  there whether or not the night crashed, and the fold has many ways to stop
  early. So the replay is never skipped, and what the fold adds to the corpus
  this morning is replayed the next.
- **A panic outside a file's replay is not silenced.** A worker's panic is kept
  for its own thread only while that thread is replaying a file. A panic
  anywhere else reaches the hook that was there before, so a broken invariant of
  the replay itself still shows.
- **The callback is told more than the file keeps.** The harness hands it a
  `WhichFuse`: the half, the rule, the policy and the way round. The file counts
  by policy and by way round. The half and the rule are in no column.

*Measured 2026-10-12, on the Q1770 branch over `main` at `5817ca1e`, with 24
threads.* The checked-in corpus: 1 753 files, 268 refusals in 209 files, 18
groups, 0 panics, in 0.29 s. The night's corpus at 10:09Z: 15 839 files, **5 003
refusals in 3 148 files, 40 groups**, 9 panics, 2 hangs, in 126 s. The run
cannot end before the two hung files' 120 s are up. 3 437 refusals came the
first way round and 1 566 the other; 1 636 were certified and 3 367 tolerant.
`Undecidable` has 2 141 of them in 14 groups, and 17 other variants the rest in
26. All 9 panics are Q1477's, at `fuse/arrange.rs:2926`, "nodes_along proved
this pair the other way round". The 2 hangs are `d29b6665…` and `e2e383f1…`,
Q1721's. The three corpus files of 437.9 that panicked at `assemble.rs` on
`459e76d4` did not panic here: `58cbb42c…` is refused `ResultIsNotManifold`
through the one-file decoder.

### 437.6 Limit, gap or not known, said where the refusal is made

Today a refusal's kind is written only in prose: the doc comments on the error
variants, and the per-milestone tables §30.5, §39.4 and §43.6, which stopped at
M5. The queue's `refuses/A` to `D` is a different thing: how likely a user is to
meet it (§352.3, §403).

**Each refusal says one of three things: a limit, a gap, or not known.** It is
decided in the crate that owns the error enum, by a match with no wildcard arm,
so a new variant does not compile until somebody says which it is. There are
three cases.

1. **A variant that is always one kind.** `QuarterTurnsNeedAnAxisAlongXYOrZ` is a
   limit: only a turn about `x`, `y` or `z` moves each coordinate onto another,
   and `Isometry::rotation` holds the rest with rounding.
   `ACrossingBetweenATorusAndAnotherCurvedSurface` is a gap.

   `Invalid(DynamicRangeTooWide)` from the fuse was given here as this case's
   limit example: the answer's own shortest edge too small to resolve beside its
   coordinates (§43.6). It is not one. Since Q2304,
   `rcad_topo::Invalid::limit_or_gap` says not known for it instead (case 3): it
   is a limit where the sliver is in the true answer (Q2237), and was a gap
   where the fuse left the sliver itself — the plate less a block cornered on
   its round tab's rim was refused for an arc `1.78e-15` long that the plate,
   its answer, does not have, until Q2296 fixed that fuse.
2. **A variant that decides from its own fields.** §43.5's
   `ACornerIsNotOnTheCurvedFaceTheFileSaysItIsOn` was a gap for 90 files and a
   limit for 76, told apart by how far the corner was off. When the fields do
   not carry what the decision needs, the field is added at the refusal site.
3. **Not known.** `Undecidable` and `CannotTellTheCellsOfAFaceApart` say that
   the arithmetic could not settle a question, and whether the answer exists is
   that same question. Most of them will say not known.

**Three cautions.**

- **A limit or gap claim is a claim, and gets a test** that reaches it (§15.6,
  §16.10). Labels have been wrong: §43.6's amendment found that
  `TwoLinesThroughAConesApexAreOneCurveThroughItself` "was never a limit". Not
  known needs no test. It is the honest answer when nobody has shown either.
- **A limit is relative to what this kernel can hold.** `ResultIsNotManifold` is
  a limit today, because a non-manifold answer has no representation here. §43.6
  wrote "limit of today's kernel, gap for the roadmap" for
  `ASurfaceThisKernelDoesNotHold`. Removing a limit of that kind is a feature for
  the milestone that names its consumer (§13), not a fix row.
- **A refusal one way round and an answer the other way does not prove a gap**
  (§140.5). The answering order may be the unsound one. The night had 1 473
  refusals made the other way round. The report keeps the order so such pairs can
  be found, but a class never comes from them.

**Who uses it.** First, the morning report: a gap group becomes a row, a limit
group is listed with no row, and a not-known group becomes a row to find out
which. Filing those rows, as §418 files crash rows, is decided when this lands.
Second, a caller, who can tell "no answer exists" from "the arithmetic gave up".
The first is enough under §13, so the row waits for the replay (437.10).

**As built for `rcad-geom` (Q1771).** `rcad_geom::Error::limit_or_gap` returns a
new `#[non_exhaustive]` `LimitOrGap`, from one match with no wildcard arm. Its
`Display` gives `limit`, `gap` or `not known`, so a caller outside the crate
reads the class from the words and needs no wildcard arm of its own. Of the 48
variants, 27 are limits, 5 are gaps and 16 are not known.
`what_a_geometry_refusal_says_about_its_answer.rs` reaches all 32 limits and
gaps through the public API and asserts each one. Every gap there is refused
every way round: both orders of a pair, and all six of a triple.

- **Limits.** Twenty variants refuse numbers that describe no shape: a radius,
  slope, half-angle, deflection, size or weight that is not one, a knot vector
  that is not one, a parameter or split off the curve, and a run that names no
  place or no curve. Four are limits of today's kernel: `DegreeNotSupported`,
  `ATorusThatRunsThroughItself`, `DynamicRangeTooWide` (case 1 above), and
  `QuarterTurnsNeedAnAxisAlongXYOrZ`, since only a turn about `x`, `y` or `z`
  moves each coordinate onto another, and `Isometry::rotation` holds the rest
  with rounding. The last three, `ACrossingTooSmallForAnyPointToSitOnBothItsSurfaces`,
  `TheCrossingOfTheseTwoSurfacesIsNotClosed` and
  `TheseTwoSurfacesAreOneSurfaceAndNameNoCrossing`, are limits by their own
  demonstrations.
- **Gaps.** `ACrossingThatTouchesItselfIsNeitherOneCurveNorTwo`, whose tangent
  point `(-2, 0, 0)` is one `f64` holds exactly;
  `ACrossingBetweenAPlaneAndATorusThatIsAQuartic`;
  `ACrossingBetweenATorusAndAnotherCurvedSurface`;
  `CannotWorkOutHowBigACrossingIs`, refused at `1e300` and answered at `1e200`;
  and `WhereThreeSurfacesMeetNeedsTwoOfThemFlat`, whose corners `(±12, 0, 0)`
  are on all three surfaces exactly.
- **Not known.** `Predicate` and `Undecidable` are case 3. The rest are both
  kinds, or nobody has shown which, and five of them differ from what the
  record said before:
  - `CannotNameWhichPieceOfACrossingThisIs` was **gap** in its doc comment, and
    §30.5 filed one cause gap and another limit. Bores on `(3, 4, 0)` and `(4,
    0, 3)` have no flat surface written in `f64` that picks their halves
    (`two_pipes_of_one_width_on_crossing_axes.rs`), while that file's count of
    named pairs, raised from 32 to 1096 of 1224 by searching harder, shows
    pickers that existed and were not found.
  - `WhereALineMeetsARingNeedsTheLineExactly` was **gap** in its doc comment and
    in §43.6. A box can hold lines that meet the tube and lines that miss it,
    and then there is no single answer, as the ball's own refusal of a ray
    whose box straddles its skin says.
  - `WeightTooSmallToBlend`'s doc comment said refusing loses no curve. Weights
    of `5e-324` and `1e300` have no power of two that makes the smaller one
    normal while the larger stays below `f64::MAX`.
  - `CannotPlaceAPointThatCloseToACrossing` was **limit** in §30.5, but its
    `closest` is what the arithmetic could prove, not how close the nearest
    `f64` point is.
  - `TooManyPointsAlongOneCrossing` was a **limit of the request** in §30.5.
    §94 met the same cap from a marcher crawling towards a point on a loop it
    could not walk, so the cap does not say which happened.

  `PointsAreInALine` is both kinds, measured: `Plane::through_three_points`
  crosses the two edges unscaled, so `(0, 0, 0)`, `(s, s, 0)` and `(s, 2s, 0)`
  give the plane `z = 0` at `s = 1e-160` and are refused at `s = 1e-200`, where
  the cross product underflows to zero.

**What this leaves.** `LimitOrGap` was not exported from `rcad-geom`'s `lib.rs`,
which was outside this row's files. Q1798 exported it and built
`rcad_topo::Error::limit_or_gap`, delegating to `rcad_geom::Error::limit_or_gap`
for `Geometry` and the two `why` variants, and Q1799 made the class the refusal
report's tenth column, which `check-fuzz-triage-selftest` now pins. Each
not-known variant that is both kinds gets a row for the field that would tell
its two kinds apart (case 2).

### 437.7 What is not built

- **Nothing is collected during the night, and there is no cargo feature.**
  §144.9 proposed the workspace's first one, and §162 measured and declined it:
  it is a second build configuration the gate must also run. A fuzz-only feature
  would also break the rule that the fuzz target and the gate's replay run the
  same lines (§29.9, §95).
- **Nothing inside `fuse`.** A predictor that changes what `fuse` does from
  learned state is model state that silently changes results (§2.6).
- **No trained model to save lane time.** Its training data would be solved
  problems, and there are few: 594 rows have ever been tagged `refuses/`. A lane
  takes a sentence in its brief as fact, so a wrong hint with no reason behind it
  costs more than no hint. What a lane can use is facts looked up for it: the
  line that rebuilds the input (`rcad-fuzz-explain` prints one), the refusal
  site, the predicates, the class, and earlier rows and sections on the same
  variant. 437.8 measures how much those would save. Searching past rows by their
  text could come later, if exact lookup is shown to miss related cases.

### 437.8 What a lane reads before its first edit

Mattias's third ask. A read-only agent measured every lane transcript (meta
`agentType: lane`, relays left out, split transcripts joined with
`tooling/row-time/row_time.py`'s `read_transcript`): 977 lanes, 365 of them on
rows tagged `refuses/`. For each, it took the context at the first edit of a
file in the lane's tree, and sorted every tool result before that edit into a
class. The figures are for lanes whose transcripts were last written on or after
2026-10-01.

**The first edit comes after minutes, not an hour.** For `refuses/` lanes the
median is 2.3 minutes and 17 calls, at a context of 101k; p90 is 6.5 minutes and
158k. Other lanes: 3.4 minutes and 95k. Most of that difference is the model:
127 of the 331 other lanes ran on Sonnet, against 21 of 335. Opus lanes alone
reach their first edit at the same context: 106k on `refuses/` rows and 107k on
others.

**Where the context before the first edit went**, as a share of all of it:

| | `refuses/` | others |
|---|---:|---:|
| the brief and the system prompt | 29.3% | 32.3% |
| reading the code the lane then edits | 35.6% | 18.9% |
| the lane's own output and thinking | 12.8% | 18.2% |
| finding the refusal site: reading code it does not edit | 8.2% | 10.0% |
| history: decisions and the queue | 5.7% | 7.1% |
| tests it read and did not edit | 3.7% | 4.9% |
| rebuilding the input | 1.4% | 2.0% |
| other | 3.2% | 6.6% |

**What a triage row could fill in ahead of time is the site, the input and the
history: about 15% of what a `refuses/` lane reads before its first edit. Their
medians add to about 9k tokens a lane.** The largest share is the code the lane then
changes, and no row can read that for it. So the lookup facts of 437.7 are worth
writing into a row, because the replay knows them anyway. But they would save
about a tenth of a lane's start, not most of it. What a `refuses/` lane spends
more on than others is understanding the code it is about to change, not finding
where the refusal comes from.

**Two spot checks** against the transcripts, made from scratch rather than with
the agent's scripts. The median lane, `q1717-two-patches-of-the-fat-wall`, made
its first edit to `holds.rs` at a context of 101 175, 3.2 minutes in, after 15
calls. That is the agent's figure exactly. The 75th-percentile lane,
`q1162-a-ball-cut-by-two-crossing-planes`, first edited at 127 897, also as
reported. Its reading of `holds.rs` was six `Read` calls of 200 to 900 lines
each, not the whole-file reads the agent's report called them.

**This corrects §416.1's shares.** §416.1 charged each response's growth, less
its own output, to the tool results that followed. But the output a transcript
records is often a count taken early in the stream. In the median lane above, 24
of 38 responses record 10 output tokens or fewer, and one of them wrote 13 367
characters. So a lane's own output was charged to the tool results after it. The
agent fitted the real output from the responses that do record it (R² 0.988),
and a separate fit of context growth gave the same weight for thinking. By that
estimate a lane's own output and thinking is about 38% of its growth, not 14%.
`Read` stays at 31%. Grep falls from 16% to 7.3%, `sed` from 5% to 3.2%, `just
src` from 4% to 1.8%, and `just t` from 7% to 4.7%. §416.1's conclusion still holds: reading code is the largest
cost, and most of it is the files the lane goes on to edit.

The agent's scripts and its output are in that session's scratchpad, and they
go when the scratchpad does. Its method is in this subsection.

### 437.9 What the probe met on the way

- **9 corpus files panic at `fuse/arrange.rs`**, "nodes_along proved this pair
  the other way round". That is Q1477 (`crash/D`, tier 5), and the 2026-10-11
  triage also has that group failing on `main`.
- **3 corpus files panic at `fuse/assemble.rs:2115` on `459e76d4`**: `58cbb42c…`,
  `6ac746b0…` and `f2cd479f…`. It is the panic Q1752 names at `assemble.rs:1996`
  on `a5d31701` and says no longer happens from its two crash files. These are
  corpus files, not crash files, so the morning crash replay never ran them. The
  models are the same kind Q1752 describes. `58cbb42c…`'s second half is two
  10 mm cubes with a plane normal of (0, 2.17e-311, 1) and a corner at y =
  2.8e-306, a spread of about 3.5e306, far past §3.5's 1e11. So it is `crash/D`,
  not `refuses/D`, because what a user would see is a panic (§352.3). Q1752 is
  filed again with the three files (437.10).
- **Hangs**: `d29b6665…` and `e2e383f1…` did not return in 120 s, and
  `90e97046…` took 49.8 s in one run and 65.2 s in the other. These are
  Q1721's three files.

### 437.10 Rows

- **Q1770, the morning refusal replay** (437.5). Tier 2, finding defects better
  (§230.1, as §311 reordered it). Its selftest is `check-fuzz-triage-selftest`.
  Built as 437.5's "As built" says. The selftest's last arm replays the
  checked-in corpus for real; the unit tests in
  `crates/rcad-fuzz-explain/src/refusals.rs` pin the keys, the counts, a hang
  and the panics.
- **Q1771, limit, gap or not known on `rcad-topo::Error` and `rcad-geom::Error`**
  (437.6), with a test for each limit or gap claimed, and the class added to the
  report. Tier 2, `after:Q1770`.
- **Q1772, Q1752 filed again** with the three files of 437.9, as `crash/D`, tier
  5. Q1752 is dropped on this section.
