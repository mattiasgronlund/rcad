## 190. No corpus file reaches the tangency wall, validate refuses a flat bent edge, and an interval square is a square — 2026-09-21

*Two batches of three lanes, dispatched by §175's skill. The first ran off
`f2937f52` (`just gate quiet` here: PASS, 2146 run, 2146 passed, 2 skipped); the
second off `8052d105`, which was the first batch merged and gated here (PASS,
2152 run, 2152 passed, 2 skipped). Every writing lane had a verifier. The
tangency wall, the refusal §0's unit is named for, is now reached by no corpus
file: 24 of the 29 files that stopped there import, and the KiCad import count
goes from 114 to 140.*

Merged as `0933b2fc`: `lane-led-dome-joint` (`e016418e`),
`lane-pole-check-questions` (`eeb99866`), `lane-bent-edge-sweep` (`f9c56693`,
which carries `lane-validate-flat-bent-edge`'s `802a4954`),
`lane-squaring-overflow` (`6a7dc6e7`) and `lane-dome-circle-named` (`ab49d4d9`),
plus two commits of mine. `just gate quiet` run here on `0933b2fc`: PASS, 2162
run, 2162 passed, 2 skipped. A number below is a lane's unless it names a
command run here.

  190.1   The LED body meets its dome: the walk starts at the file's corner
  190.2   The dome's circle is named by its plane, and the tangency wall empties
  190.3   validate refuses a bent edge with nothing curved along it
  190.4   An interval squared is a square, and §189.1's open question is a gap
  190.5   The cone arm of the pole check has tests
  190.6   What is left

### 190.1 The LED body meets its dome: the walk starts at the file's corner

§189.3's two halves are in, and the lane showed that each is needed.
`walk.rs`: when a closed quartic run names a corner, the marched loop is turned
so that it begins and ends beside that corner. `mesh.rs`: when the direction
rule gives None because the two faces touch, the points run the way the pipe's
longitude increases. With only the turn, face 8 still does not lie flat. With
only the direction, face 7 strays 1.054 past its deflection. With both, both
`_O1.27mm` LEDs get past face 8.

The coaxial closed form §189.3 proposed was **not** built, and it could not
have fired. Exact predicates put the file's ball centre about 2.1e-16 off the
file's pipe axis: the axis direction's x component is -5.55e-17. So a stored
ball and pipe that meet in a circle, certified by exact signs, is not what this
file holds. §190.2 is the fix that works.

The lane's gate was red on one wildcard arm in its own new test. I replaced it
with a `let ... else panic!`, which still fails loudly on any refusal it does
not expect.

### 190.2 The dome's circle is named by its plane, and the tangency wall empties

The file names the body-to-dome edge as a `CIRCLE`, so the answer exists and
this was a gap (§16.10). `brep.rs` now checks, in closed form, that the circle
lies on every face along the edge: on a ball and on a pipe, within
16 × `NEAREST_F64_POINT_ROUNDOFF` × the largest number involved. Where a ball
meets another curved face and the check holds, the run is named by the circle's
plane. `walk.rs`'s `the_two_surfaces_it_runs_between` gained an arm that reads
that plane for a curved pair, so the circle is drawn from [ball, plane] rather
than marched along a tangency.

A closed edge is `RightRoundWhereItsFacesCross`, which carries no `and`, so the
reader splits a checked circle into two pieces. The second corner is a quarter
turn on, `c + w×(A−c)`. The lane measured that choice: with the opposite point,
or a quarter turn the other way, the `O1.27 Z10` file is refused on its dome. I
did not re-run that. The verifier did not either; it took it on the lane's word.

**What it bought** (`just t -p rcad-step -E 'binary(merged) &
test(how_often_a_real_file_imports)' --no-capture` here, on `0933b2fc`): the
KiCad import count goes from 114 to 140,
and no file that imported before stops importing. On `f9c56693`, before this
lane, `just t -p rcad-step --test a_crossing_the_marcher_gave_up_on_names_its_edge
--no-capture` listed **29** files stopping at the tangency wall: 23 relays, the
fuseholder and five LEDs. On `0933b2fc` the 23 relays and the fuseholder import,
and the five LEDs (`D3.0 O6.35 Z2` and the four `D5.0 O1.27 Z3`) stop on "a
wire of this face will not lie flat". The rest of the 26 are two more relays.
That test's floor of one file then failed. Its message asks whether that is
good news or a reader that stopped reaching the marcher. It is good news, so
the test is retired. Its sibling in `what_the_mapping_refuses_and_by_what_name.rs`
now says so. `a_real_file_written_out_and_read_back_is_the_same_solid.rs` is
re-pinned to 140, and its 45 named round trips hold unchanged.

**Two things about the check.**
- It is plain f64 against a closed-form bound, not certified. It is
  conservative, and it trusts the file's circle only after holding it against
  both faces. It is still a new tolerance in the reader, written down here
  rather than argued away.
- Cones and tori are never checked, so their circles are named as before.

The walk arm's premise, that nothing the kernel builds names a plane between
two curved faces, rests on a grep of `PartOfWhereItCrosses`'s constructors. No
test proves it.

> **2026-10-06 note (Q1142): the premise no longer holds.** Since Q1118
> (`4605e8a5`), `fuse/assemble.rs`'s `the_plane_of_a_rim_between_two_curved_faces`
> writes a closed rim between two different curved faces — the dome's own
> pipe and ball, a frustum's cone and a bar's pipe, two cones base to base —
> as `RightRoundWhereItCrosses { and: <the plane> }`, earned by
> `meet_of_three_surfaces` answering `MoreThanAPoint` for the plane and the
> two curved surfaces, once `the_name_each_closed_curve_goes_by` has joined
> their two names into one. The kernel does build this now, through that one
> constructor; the grep above was of `PartOfWhereItCrosses` alone and never
> covered it.

**Where the nine LEDs stop now.** Both `D3.0 O1.27` files stop on `#223`, the
flat face with the pin holes: `BoundaryCrossesItself` at 8. Nobody has looked at
it. The seven siblings get past the body and stop on the dome (`FaceSlot 17`,
or `FaceSlot 1` for `D5.0 O1.27 Z15`) with `AWireWillNotLieFlatOnItsFace`. Which
of these depends on where the second corner lands. That is `mesh.rs`'s wire walk
over two pieces of the equator plus a seam to the pole, measured but not
diagnosed.

### 190.3 validate refuses a bent edge with nothing curved along it

§187.1's sweep half. `Model::validate`'s per-edge check refuses an edge whose
run is not straight, whose faces are all planes, and whose run names no curved
surface. The refusal is `Invalid::AnEdgeBendsButNothingAlongItIsCurved`. The
rule is `validate::nothing_curved_says_where_it_bends`, and `mesh.rs` now calls
it instead of keeping its own copy. `Model::check_touched_edge` runs it too.

The first lane broke three tests, so its branch was held back and a second lane
landed it. `SolidBuilder::build` runs the sweep, so a plane-named circle is now
refused at build time. `mesh()`'s own refusal is still reachable through
`Deserialize`, and the unit test in `mesh.rs` covers that.
`a_bent_edge_between_two_flat_faces_is_refused.rs` now expects the build to
refuse, under the test name §187 cites. The duplicate test file was folded into
it.

`what_the_whole_sweep_says_about_a_two_step_wire.rs` used a plane-named run to
stand in for "a run the builder has never seen", and the new check refuses that
stand-in. Flipping its assertions would have closed a hole on paper. The lane
found a stand-in that still works instead: a run naming a pipe whose axis runs
along the edge, which is curved by name and crosses every flat face in a straight
line. Both measurements hold on it. The zero-length edge is still refused as
`FeatureSizeNotPositive`, and **the flat two-step face that bounds nothing still
validates**. `builder.rs`'s hole is open, and its paragraph says so.

The lane saw a side finding once, with a stand-in it rejected: a curved-named arc
with no pickers on a flat face makes `the_ring_a_wire_traces` answer
`Error::Undecidable` rather than refuse by name. The verifier could not re-run
it, because the stand-in was never committed.

### 190.4 An interval squared is a square, and §189.1's open question is a gap

§189.1 asked whether an overflowed enclosure should count as "not provably on".
**No.** The read-only lane found that the enclosure was not the result of
overflow alone. `gap * gap` on an interval that straddles zero gives
[-inf, inf], because interval multiply does not know both factors are the same
number. Squaring it as a square, or scaling by a power of two first, reaches the
sign: the far corner of fuse seed `3482a32e` half 0 comes out Pos. Reading inf
as "not provably on" would turn a corner that really lies on a ball of radius
1e200 into `Error::Undecidable`. So this is a gap in the arithmetic, and it is
fixed.

All four quadrics had it: the sphere's gaps, the cylinder's and cone's cross
products, the cone's `up`, and the torus's lengths and `bulge`. A test for each
fails on base; the verifier re-ran all eight. `squared()` gives [0, max²] for a
range that straddles zero. `without_overflow()` re-runs the evaluation scaled by
one exact power of two, and only when the first answer straddles zero and is
unbounded, so no answer that was finite changes. inari rounds outward, so an
underflow widens an answer and never flips its sign. A test pins where that
costs: `(2^600, 2^-500, 0)` on a ball of radius 2^600 stays Undecidable, at a
finite width. The branch was red only on `cargo fmt`, which I ran.

**Counts.** 131 of §189.1's 132 fuse-corpus halves move from
`APoleStandsWhereTheSurfaceDoesNotPinch` to `VertexOffItsFaceSurface`: they
were far corners the on-surface check could not decide. One pole off its pinch
is left, `e8fe8935` half 0. Validated stays at 681. The seed itself is now
refused as `VertexOffItsFaceSurface` before the pole check runs.

`squared` lives in `sphere.rs` as `pub(crate)`, and the other three import it
from there. It belongs on `Interval` in `rcad-predicates`. The exact level of
each quadric still returns None when a product overflows, and could be scaled
the same way.

### 190.5 The cone arm of the pole check has tests

`a_pole_on_a_cone_must_stand_at_its_apex.rs` has four tests. A pole at the
apex builds, and one 2^-50 beside the apex builds. One halfway up is refused,
and one 2^-40 up the ruling is refused. The lane could not show the two
refusals fail without the pole check, because that means editing `validate.rs`.
Their corners lie exactly on the cone, so the on-surface check is not what
refuses them.

**A defect it found.** A pole 2^-50 *up the ruling* is refused as
`DynamicRangeTooWide` (1.8e16). The cone arm of `check_dynamic_range` counts
the pole corner's radius as a feature size, although the pole check treats that
corner as the apex.

### 190.6 What is left

- **The dome's wire in `mesh.rs`**, §190.2: seven LEDs, and the five from the
  tangency wall, stop on "will not lie flat", depending on where the second
  corner lands.
- **`#223` in the `D3.0 O1.27` pair**: a flat face with pin holes,
  `BoundaryCrossesItself`.
- **A closed run carrying `and`** (`meet.rs` plus every closed-edge site in
  `rcad-topo`), so the reader need not invent a corner. That is a design
  decision.
- **The circle check in `brep.rs`** is a plain-f64 tolerance, §190.2.
- **The cone's `check_dynamic_range` arm**, §190.5.
- **`builder.rs`'s two-step hole**, still open, §190.3. Also the picker-less arc
  that comes back Undecidable.
- **`squared` onto `Interval`**, and scaling at the exact level, §190.4.
- From §189.4: **the turned-bend band** and **fuse::bake's corner test**.

Commands: `just decision 189.3`, `189.1`, `187.1`, `16.10`; the `lanes` workflow
runs `wf_56166a41-927` and `wf_3f28ed8e-f4a`; `just gate quiet` on `f2937f52`,
`8052d105` and `0933b2fc`.
