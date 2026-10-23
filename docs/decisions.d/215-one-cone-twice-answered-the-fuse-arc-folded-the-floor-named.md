## 215. One cone twice answered, the fuse's ellipse arc folded, and the nappe floor named — 2026-09-23

§0's *"nine fuses refuse naming a surface the model has not got"*, closed, and two
of the three things §0 listed as also open with it. Three lanes off `3ce21631`,
two writing and one read-only.

### 215.1 The 9 are answered, and the arm guesses one thing

§214.4's fix (a), built as recommended and nothing else.
`what_a_run_between_these_two_lies_on` has a same-surface arm above the refusing
one: one cone twice, with `run = EdgeRun::RightRoundWhereItsFacesCross`, is the
circle through the run's own end square to the cone's axis. Guarded and not a
wildcard, so a cone against a *different* cone still refuses and a sixth kind of
`Surface` is still a compile error in that `match` (§40).

Measured, `just t --test merged -E 'test(/^how_often_a_fuse_with_a_cone_refuses::/)' --no-capture`: the
block family answers **15 of 96 → 24 of 96**, its wall map goes from five named
things to four, and the cone-crossing entry goes **9 → 0**. The 15 that §214.4
called an honest gap are **unmoved**, and so are the other three walls — a point
inside a piece of a face 30, a corner too far out for the triangulator 18, a face
whose boundary crosses itself 9. Two frustums on one axis stay 0 of 96, all of
them still *a cone crossing another curved surface*, which is the guard doing its
job rather than a thing left undone.

**What the arm guesses.** The run carries no record of the plane that made the
cut, and a plane leaning across the axis cuts a cone in an **ellipse**, which is
closed too. So a run off such a cut is indistinguishable here from a circle and
this names the circle — wrong about that curve, which is the direction §4.1 ranks
worst. §214.4's (d), writing the cutting plane into the run, is what would tell
them apart, and it is a change to what `assemble` writes rather than to what the
mesher reads. The limit is written into
`the_circle_a_closed_run_round_one_cone_lies_on`'s own doc comment rather than
claimed unreachable, because there is no demonstration that it cannot be reached
and §16.10 asks for one.

What *is* measured is that nothing reaches it today: the two corner-up-cube
families, whose every plane leans across the cone and so cuts an ellipse, answer
45 of 96 and 33 of 96 and **neither has a single cone-crossing refusal in its
map** — before the change as well as after — so the arm is never handed one of
their runs, and the change moved no answers there, only refusals in the
axis-square block family. **Why it is unreached is not answered here**, and the
obvious guess is wrong: the nappe file's refusals of 215.3 are in point
containment, a different phase from the shell-nesting mesher this arm sits under,
so closing those gaps does not expose this one. A run on one cone that is not
`RightRoundWhereItsFacesCross` keeps the old refusal under a name that
misdescribes it; `EdgeRun::Straight` never reaches the `match` at all
(`walk.rs:339` answers it), which is where §214.3's frustum seam is handled and
why nothing moved there.

### 215.2 §211.8's fourth spelling was a live defect, not a tidy-up

§211.8 recorded `fuse/arrange.rs`'s ball-only `where_the_circles_centre_sits` as a
fourth spelling of the centre question that predated the work and was not
touched. It was more than that. `arrange.rs`'s own `the_way_into_an_arc` opened by
pulling a `Surface::Sphere` out of the pair with `?`, so a plane and a **cone**
returned `None` before any other test ran — §211.1's defect word for word, still
live in the fuse's own arrangement a section after it was fixed in `holds.rs`.

`arrange.rs`'s copy and both its ball-only helpers are gone; it asks `holds.rs`'s
`where_a_closed_crossing_is_centred` and `a_picker_that_tells_the_two_arcs_apart`,
both now `pub(crate)`. No `rcad-geom` change was needed or made: the per-kind
arithmetic is already there in `Cone::which_way_a_closed_crossings_centre_lies`,
and what sits in `holds.rs` is the dispatch and §211.4's soundness argument, whose
two callers are both in `rcad-topo`.

**Reachable, and demonstrated rather than argued.** An `eprintln` probe in
`the_way_into_an_arc`, driven from `Model::fuse` and reverted after, printed the
pair `Plane(normal (0.7071067811865476, 0, 0.7071067811865476), offset
2.8284271247461903)` against `Cone(axis z, slope 0.5)` — the ellipse — in both
orders, and 45 calls to the folded centre question, every one `Neg` and every one
from the cone's apex. The verifier reproduced that probe and those counts
independently, coordinate for coordinate.

§211.3's claim that the ball case reduces to the old expression term for term
**holds**, and was checked against the deleted code rather than taken on trust:
the only difference is that `arrange.rs`'s dot helper folded from an exact `0.0`
and `holds.rs`'s does not, and adding exact zero is a no-op in this arithmetic.

The new test asserts *"not `AFaceTrimmedByACurveThisCannotFollow`"* rather than a
fused solid, because the same pair then hits an unrelated wall in `fuse/select.rs`
— `Undecidable` naming a point inside a piece of a face, on a flat face it cannot
wind a point round. That is a measured stopping place and is written into the test
file's doc comment, with two more beside it: `fold_the_nodes` refusing
`Invalid::EdgeHasNoLength` for a half frustum cut down its own axis, where a cut
plane meets the cone in two straight rulings and corners on two different rulings
fold into one node — which is why `a_flat_cap_cut_by_a_cones_rim.rs`'s half
frustum cannot be fused today — and a block big enough to hold the lens leaving a
piece of its own wall bounded by a hyperbola with only two nodes.

### 215.3 The nappe floor: two causes, four faces, and no arithmetic in either

§211.6's unnamed floor, named. It is **not** arithmetic. Over all 152 answers of
`a_plane_cut_along_a_cones_other_nappe.rs`, every undecided grid point is
`Containment::CannotFollowAFacesTrimming` and **not one** is `Undecidable`,
`OnBoundary` or `CannotFireARayAtAFacesSurface`. Nothing there ever returned
`Undecidable`, so no widening of a policy or a filter can move the number — which
is what makes the whole §204.3-shaped hunt for a tolerance the wrong hunt.

Both causes sit in `is_a_family_this_follows`, `crates/rcad-topo/src/holds.rs`:

- **`holds.rs:1091`, `Surface::Cone(_) => false`** — a cone face cut by part of a
  crossing is never followed, whatever cuts it. That alone floors every answer of
  this shape at about 23 094, costing 7 007 points on every answer.
- **`holds.rs:1079`'s `cut.kept == Decision::Neg` conjunct** — a flat face cut by
  a cone is followed only when it keeps the cone's **inside**. The three upper
  planes of the corner-up cube keep its outside in `Union` and in
  block-less-cone, and only those two rules of the four sit on the floor: **9 326
  against 23 094** for the same shapes.

**Both are gaps under §16.10, not limits and not defects.** For the second the
demonstration is direct: the very same plane, cut by the very same cone, is
followed in the `Intersection` answer of the same case and refused in the `Union`
answer, and the only thing differing is which side of the cone the face keeps. For
the first the code says `false` by a literal match arm with no arithmetic in it.

**The premise that one face holds the floor is false.** Four faces refuse in every
floor answer, and which one a given point is attributed to is decided by face
order and ray direction — the same case splits 7588/6282/3788/3117 under `Union`
and 148/7548/9568/3511 under block-less-cone. §211.6 asked which face, and the
answer is that the question was singular and the thing is not.

§211's ellipse work is exonerated by the same run: `is_cut_in_a_closed_curve_by`
answered `Some(true)` for all 20 plane faces measured, so the second conjunct of
1079 never fails. That is §211 doing exactly what it was built to do, and it is
why the count did not budge when it landed — 9326 → 9326, as §211.6 recorded.

**One step is inference and is marked as such**: `cut.kept` itself was not read,
because that meant editing `holds.rs`, which the read-only lane was told to stay
off. Either `kept` is `Pos` and the refusal is the 1079 conjunct, or some other
reading makes it `Neg` and the refusal falls through to the same function's
closing `Err`; both sites are in
`what_a_face_cut_by_part_of_a_crossing_holds` and the plain-words cause is the
same either way. A fix lane with `holds.rs` in its write set settles it with one
`dbg!(cut.kept)`. The reproducer — a 558-line probe and a two-line `eprintln`
patch that changes no behaviour — is carried in that lane's report rather than
left to be rebuilt, which is the half §204.3 got wrong.

### 215.4 A pin's own prose was an inference, and it was measurable

`what_the_answered_block_fuses_actually_say.rs` pinned 15 answers and explained
them: *"six from the row at slope · far = 0.75, and both intersections and `block
\ frustum` from each of the three at slope · far = 1"*. 215.1 moves that count to
24, and the lane that moved it flagged its own replacement sentence as **inference
and not measurement** rather than writing it in — which is the right way round,
and the reason it is measured here instead.

Measured: **four** rows answer all six of their fuses — `(0.125, 4, 6)` at
slope · far = 0.75 and the three at slope · far = 1, `(0.125, 1, 8)`,
`(0.125, 4, 8)` and `(0.125, 2, 8)` — and the fifth clear row, `(0.125, 1, 10)`
at slope · far = 1.25, answers **none**. Four sixes is the 24. The three rows at
slope · far = 1 used to answer only their two intersections and `block \ frustum`;
what filled in the other three each is 215.1's arm, and 9 is exactly that.

The test now prints the rows that answered, so the next count that moves is
re-derived from a run rather than from the sentence above it — §15.6's rule
applied to the sentence a pin carries, not only to the pin.

### 215.5 The verifier read the gate this time

§212.5 recorded a verifier copying `gate: FAIL` into its own verdict and still
saying `merge`. The same machinery, on the same shape of problem, returned
**`fix_first`** here and named the one red test as the reason. What it caught was
the pin of 215.4, legitimately outside the lane's write set, and the lane had
disclosed it. So the cross-examination §212.5 asked for is working, and the rule
it produced — read `check.gate_summary` yourself whatever `verdict` says — cost
nothing to keep and is kept.

### 215.6 What this cost, and what is open

`just gate quiet` on the merged tree of §215: **PASS, 2303 tests run, 2303 passed,
6 skipped**, 25 doc tests across 9 crates — four tests more than the base's 2299,
which is the two new test files. The pin base `3ce21631` was gated green, read and
not forecast, before the lanes were cut.

Left open, and each is a lane:

- **§214.4's (d)**, writing the cutting plane into the run, which is what turns
  215.1's guess into an answer. It is the follow-on §214.4 already named; what is
  new is that a reader now depends on the guess.
- **The two gaps of 215.3.** The second is the cheaper and the better understood:
  a flat face cut by a cone, keeping the cone's outside, is followable and is not
  followed. Worth 9 326 → about 23 094 on that file's floor by the same run's
  arithmetic.
- **The three walls of 215.2** — `select.rs` on a point inside a piece of a face,
  `fold_the_nodes` on a half frustum's folded node, and a wall bounded by a
  hyperbola with two nodes. All measured, all written down where the next reader
  will find them.
- `measure.rs`'s cone-tip bound (§208.6), §212.6's island gap, §124, §136.4,
  §125.6, all untouched.
