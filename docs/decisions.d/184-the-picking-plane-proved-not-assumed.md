## 184. The picking plane is proved, not assumed: §183.5's three changes, built and measured 2026-09-21

*§183 measured that a plane separating §181.2's two lines exists far from the
origin and named three changes to reach it. All three are built, inside
`two_lines_between_two_pipes`, with `HalfSpace` unchanged. §181's three
artifacts that refused both ways round now **answer** both ways round, each
run holding exactly one of the two lines. On 3972 draws of the same family,
refusals from this function go from 793 to **0**, the one pair that answered
in one order and refused in the other is gone, and no answer was found wrong.
**The published refusal rates did not move**, and that is measured rather than
waved at: none of their pairs gets past the first half-space tried.*

### 184.1 What was built

1. **The separation is proved, not assumed.** Three anchors — between the
   lines (§164's), this pipe's stored axis origin, the other pipe's — each with
   the normal `through_a_point_square_to` ranks first, then each exactly-square
   fallback turned to lean the way the asked-for direction does. The first
   half-space `two_lines_told_apart` proves is used. Today's half-space is tried
   first, so a pair named before is named by the same plane. A half-space that
   cannot be built at all is still the error it was; one that proves nothing is
   passed over. `normals_exactly_square_to` became `pub(crate)` for this, and
   nothing else in `half_space.rs` changed.
2. **The stored axes are extra anchors, not a replacement.** §164's argument
   for anchoring between the lines stands for pairs near the origin, and the
   between-the-lines anchor is still first.
3. **The middle is one polynomial over one division.**
   `(at_here·(δ² − r₁² + r₂²) + at_there·(δ² + r₁² − r₂²)) / 2δ²`. It is the
   brief's form with `at_there − at_here` multiplied out, so no difference of
   two readings is formed. `TwoLinesEitherSideOfThePicker` now carries the
   whole reading, `middle`, rather than a point and a step. The plane-and-pipe
   caller reads its axis and adds its own step, as before. **As first built
   (`2abe365b`) `δ²` was the squared length of the rounded step, so this was
   not a polynomial in stored numbers and the window could miss the middle.
   `67893d98` reads `δ²` as `A/W` instead. §184.7.**

**One thing §183.5 did not foresee: at this pair the exact level cannot run
unaided.** `2δ²` alone is `5.1e307` and a reading is `3.7e21`, so their product
overflows and `Exact::times` returns `None`. Every term is degree two in the
widths and the gap between the axis origins, top and bottom alike, so they are
slid down by the
smallest power of two that makes the top fit. That rounds nothing while the slid
numbers stay normal, and it cancels out of the quotient. The interval form of
the same polynomial is kept for when even that cannot run. It then overflows to
"everything" at this pair, so it cannot be what answered here: the exact route
did. That is worked out, not instrumented.

### 184.2 What the pair looks like now, measured

Replaying §181.2's four artifacts both ways round (the probe crate
`/tmp/rcad-183-probe/crash-probe/`, pointed at this tree):

| artifact | fat pipe first | thin pipe first | picking plane |
|---|---|---|---|
| `crash-0f06237e` | 2 runs | 2 runs | `z = 41581179`, both runs |
| `crash-477e4468` | 2 runs | 2 runs | `z = 41581179`, both runs |
| `crash-b76d3903` | 2 runs | 2 runs | `z = 41581179`, both runs |
| `crash-87e70e7f` | 2 runs | 2 runs | unchanged from `8ddf072f`, normals and offsets bit for bit |

For the first three, each run's picker was asked about `(0, R1, Z ± R2)`, two
points within about `1.4e-111` of the true lines, and each run holds exactly one
of them, a different one from the other run. `z = 41581179` is the thin pipe's
stored axis `z` with the `along × ŷ` fallback, which is §183.3's plane. It needs
both of the first two changes: it is not the ranked normal, and its anchor is
not the computed midpoint, whose `z` the probe measured at `41418752`, `162427`
off.

**The third change is needed as well, and the probe shows it.** Run against
`8ddf072f`, which still read the middle as `at_here + share·(at_there −
at_here)`, the same plane — thin axis, `along × ŷ`, offset `41581179` exactly —
gave `middle = [-2.62144e6, 3.145728e6]` against `either_way` starting at
`1.328125`, and refused. §183.3 quotes a window of `1.984375` for this row. The
probe measured the lower end of `either_way` at `1.328125`, with an upper end
of infinity. Only the lower end enters the test, so the verdict is the same.
The `1.328125` is the number to use.

The three corpus artifacts under `fuzz/corpus/where_two_surfaces_cross/` replay
as the baseline lane found them on `8ddf072f`: `crash-3f9c76a3` one run,
`crash-a2acceac` two, `crash-a-thin-pipe-on-a-fat-ones-skin-cba396f8` two.

### 184.3 Whether three fallbacks are enough: the family, swept

§183.6 left this as the lane's measurement. The sweep takes §140's pair and
draws both axis origins' `z` — a sign, then either a whole number below `2²⁶`
or a mantissa in `[1, 10)` times `10^k` for `k` up to 150 — with `|Δz| ≤ 1e150`
so the two lines stay near `z = thin_z ± R2`. It asks both orders, and checks
sides wherever `thin_z ± R2` is representable. It uses 3972 pairs from a fixed
seed (`/tmp/rcad-183-probe/sweep-probe/`, and a copy pointed at the
baseline lane's clean `8ddf072f` tree for the before column):

| | both answer | both refuse | disagree | refused earlier | runs side-checked | wrong |
|---|---|---|---|---|---|---|
| `8ddf072f` | 830 | 793 | 1 | 2348 | 868 | 0 |
| this change | 1624 | **0** | **0** | 2348 | 1664 | **0** |

**For this family, the three fallbacks and three anchors are enough.** Every
pair that got as far as naming two lines is named, and no pair gets a different
answer depending on which pipe is asked first. The one disagreement before was
`fat_z = 4.4400491042010585e15`, `thin_z = -2.7199704356576648e39`: fat first
refused, thin first answered. That is §181's defect again, so the family held
more of it than the four the fuzzer drew.

**This is a claim about the family and not about every pair of pipes.** It says
nothing about pipes that are not along a coordinate axis, where the three
fallbacks have no zero component to make the offset a stored number. Nothing
here measured that, so it stays open. If a pair there refuses, the honest
answer is a refusal with this section's number on it, not a fourth candidate
picked to fit.

**The 2348 in the "refused earlier" column are not this function's.** They
are `Error::Undecidable` from `where two pipes cross`, the shape question asked
before the two lines are named, with the enclosure `inf` at the interval level.
They are the same pairs before and after. That is a separate gap, and 59% of
this family.

### 184.4 The published rates did not move, and why

`how_often_naming_a_crossing_refuses`, measured on this tree:
the band `1.22%` for want of a half-space and `1.22%` running through
themselves, the turned quartic family `1.27%` and `0.05%`, crossing somewhere
else `10.58%` for want of a sign (217 of 2052), the quartic control `4.24%`
running through themselves, and every other rate `0.00%`. The baseline lane's
figures on `8ddf072f` are the same.

**They could not move.** A counter in `two_lines_between_two_pipes`, added and
removed for the measurement, counted 288 calls over the whole of that file, and
all 288 were proved by the first half-space tried. None was rescued and none
refused. So none of those refusals reaches this function. They belong to other
rows, and no doc comment's rate changes.

The same counter over `where_two_side_by_side_pipes_stop_being_told_apart.rs`
and `a_pair_of_parallel_pipes_answers_one_way_round_and_refuses_the_other.rs`
counted 12 022 calls, all proved first time. **§164's thirty directions never
reach the retry**, so what §164.4 argued about the lean is untouched. Both
files pass.

### 184.5 The regression test now asserts the answer

`two_pipes_far_from_the_origin_name_their_crossing_the_same_both_ways_round.rs`
keeps its agreement test and gains
`a_pipe_on_a_fat_pipes_skin_far_from_the_origin_crosses_it_in_two_lines_either_way_round`,
whose module doc is rewritten. Counting runs was not enough, because two runs
is also what §182's wrong answer gave. The test asks each run's picker which of
the two points on the true lines it holds. Run against `8ddf072f`'s
`intersect.rs` and `half_space.rs`, it fails with
`asking pipe 0 first did not name the crossing:
Err(CannotNameWhichPieceOfACrossingThisIs)`. On this change it passes.

### 184.6 What stays open

- **Pipes off the coordinate axes, far from the origin.** Not swept. Whether
  some exactly-square normal still has a stored offset there is the question.
  §29.8's remark that off the lattice only *some* normals come out exactly
  square suggests the answer can be no. That is a suggestion, not a
  measurement.
- **The 2348 upstream refusals** in §184.3, which are the shape question's.
- **§183.3's `1.984375`** is not what the probe measured. See §184.2.

Commands this section was written from: `just decision 183`, `183.1`–`183.6`,
`182`, `182.4`–`182.6`, `181.2`, `164`, `93`, `29.8`, `140.5`; `just src
two_lines_between_two_pipes`, `two_lines_told_apart`,
`TwoLinesEitherSideOfThePicker`, `how_far_in_front`, `between_the_two_lines`,
`the_step_across_to_the_other_axis`, `product_is_exact`; §183's probe
`probe-182-anchors.diff` applied to `8ddf072f`, run with `just t
probe_182_which_anchors_separate_the_two_lines --no-capture` and reverted; the
probe crates `/tmp/rcad-183-probe/crash-probe/`, `sweep-probe/` and
`sweep-probe-base/`, run with `cargo run --release`; `cargo run -p
rcad-fuzz-explain -- where_two_surfaces_cross <file>` on each of the three
corpus artifacts; `cargo nextest run -p rcad-geom -E 'binary(merged) &
test(how_often_naming_a_crossing_refuses)' --no-capture`, and the same for
`where_two_side_by_side_pipes_stop_being_told_apart` and
`a_pair_of_parallel_pipes_answers_one_way_round_and_refuses_the_other`, with
and without a temporary counter; the new test run against `8ddf072f`'s two
source files; and `just gate quiet`.

### 184.7 The middle read through the rounded step missed the true middle

*§184.1's third change said the middle was a polynomial in stored numbers. As
built in `2abe365b` it was not: `δ²` was `|towards|²`, and `towards` is the step
`the_step_across_to_the_other_axis` rounds once per component. Off the
coordinate directions that rounding put the true middle outside the window
119 763 times in 279 636. `67893d98` reads `δ²` as `A/W`, from stored numbers
only, and the count is 0.*

**Why it did not show before — argued, not measured.** While the middle was
read as `at_here + share·(at_there − at_here)`, `share` was an enclosure at
least as wide as the step's rounding, so the rounding could sit inside it. Read
as one exact quotient, the window is an ulp of the answer. A rounding of `δ²`
by under an ulp then moves the middle by far more than one wherever the two
readings nearly cancel. §184.3's sweep checks answers, not windows, and its
row is the same on `f26e52ea` and `67893d98`, so over that family the defect
changed no answer; whether a window there missed was not asked. Nobody ran the
old share-based reading over the search's pairs.

**The search.** Pipes along the same direction off the coordinate axes, 23 303
pairs, every half-space the row may try asked of each: 279 636 windows, of which
119 763 did not hold the true middle, decided exactly from the stored numbers.
After the fix, 0. The search was a temporary harness in the session that wrote
`67893d98` and was not kept; these three numbers are that session's, in its
commit message, and were not re-run here.

**One pair, checked the long way.** Two pipes along `(3, 5, 7)` with origins
`(1.8894988582774552, 1.7546217903306627, 5.900896671941921)` and
`(20.032907041561735, 41.90286208415537, 41.15749672697364)`, widths
`8.668044147417756` and `3.5094414748036575`. Worked in Python `fractions` from
those numbers, the true middle is `-1.209827636296209e-16`; the old window was
`[-1.0538915031078247e-16, -1.0538915031078239e-16]`, which holds the value read
through the step and not the true one. `|step|²` is `3.16e-17` of itself off
`A/W` — a rounding, turned into a middle 15% off by the two readings nearly
cancelling. Re-run for this section; it prints the same.

**The fix.** `δ² = A/W`, with `A = ‖(c − o) × w‖²` and `W = w·w`, both exact.
Multiplied through by `W`, top and bottom are polynomials in the two axis
origins, `w` and the widths:
`(at_here·(A − W·(r₁² − r₂²)) + at_there·(A + W·(r₁² − r₂²))) / 2A`. The slide
moves from the step to the gap between the origins and the widths; the bound on
it gains `w`'s largest component, which does not slide. `Exact::slid_by` became
`pub(crate)` for the gap, which is itself an exact difference.

**The test.** `the_window_around_the_middle_of_two_leaning_pipes_holds_the_true_middle`,
in `intersect.rs`'s `mod tests`, is the pair above: every half-space the row may
try, each window checked exactly against the true middle. **It was not run red.**
`how_far_in_front_of_the_middle` lost an argument in the fix, so the test does not
compile against `f26e52ea`. What stands in for the red run is the search on
`f26e52ea` printing this pair and normal as a miss, which is the same containment
the test asserts.

**What was re-run on `67893d98`**, since the fix touches every middle reading:

- §184.2's four artifacts, both ways round: the same as §184.2's table, 2 runs
  each way, each run holding one of the two points, picking plane `z = 41581179`
  for the first three.
- The three corpus artifacts: `crash-3f9c76a3` one run, `crash-a2acceac` two,
  `crash-a-thin-pipe-on-a-fat-ones-skin-cba396f8` two.
- §184.3's sweep: 1624 both answer, 0 both refuse, 0 disagree, 2348 earlier,
  1664 side-checked, 0 wrong — the same row.
- §184.4's rates, the same to the last pair: the band 1.22% and 1.22%, the turned
  quartic 1.27% and 0.05%, somewhere else 10.58% (217 of 2052), the quartic
  control 4.24%, the rest 0.00%.

**Still open, and now sharper.** §184.6's first item, pipes off the coordinate
axes far from the origin, is exactly where this defect lived, and a sweep there
still does not exist. §184.3's side check assumes the lines sit at
`thin_z ± R2`, so it needs points on the true lines found another way.

Commands: `just decision 184`; `git show 67893d98`; the probe crates
`crash-probe/` and `sweep-probe/` copied from `/tmp/rcad-183-probe/` and pointed at
this tree, `cargo run --release`; `cargo run -p rcad-fuzz-explain --
where_two_surfaces_cross <file>` on the three corpus artifacts; `python3
check_184_7.py`; `just t -p rcad-geom -E 'binary(merged) &
test(how_often_naming_a_crossing_refuses)' --no-capture`; `just t
the_window_around_the_middle_of_two_leaning_pipes_holds_the_true_middle`.
