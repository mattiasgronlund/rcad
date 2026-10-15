## 39. What M4 actually cost, recorded 2026-09-01

*Written by M4's closing gate. §12 did this for M0, §16 for M1, §17.8 for the
fuse, §26 for M2 and §30 for M3.*

M4 is complete in the only sense §31.1 gave it: `Model::fuse` no longer refuses a
curved solid, `Containment::NeedsFlatFaces` is gone, and a block with a round
hole through it, a bar notched by a block, a bar notched by a wedge and a ball
cut by a plane are fused, validated, meshed and measured against their closed
forms. The suite went from 985 tests at M3's close to **1112**.

`rcad-predicates` is at **0 files changed for the whole milestone**, across ten
lanes and this gate. So is `rcad-mesh`.

**And it does not do what a reader of §31.1 would expect it to do.** A ball with
a pipe through it — one of the two shapes §31.7 criterion 13 names — is refused,
and so are a bar coincident with itself, a ball cut down its polar axis, a block
corner exactly on a bar, and a bar cut by one plane off its axis. §39.4 is the
table of all of it, and §39.9 answers the question roadmap §12 asks.

### 39.1 The sixteen criteria, and the test that discharges each

§30.1's shape. Every row names a test that fails if the claim stops holding.
**Fourteen are met, two are met in part, and none is unmet.**

The two partial rows are 13 and 15, and the reason is the same in both: the
criterion named a *shape*, and the shape refuses. Four of the fourteen are met
with something worth reading in the row — **1** is met by a type the gate did not
know would exist and cannot be met as written, **5** is met in §28.4's form
rather than to the letter, **8** is met with a defect underneath it the audit
found, and **16** is met only because this gate re-derived the number it is
judged against.

| | Criterion | | Discharged by |
|---|---|---|---|
| 1 | a normal written square to a direction survives storage | **met, by a type the gate did not know would exist** | `a_normal_written_square_to_a_direction_is_still_square_to_it_once_stored` on `HalfSpace`. The criterion says *"once a plane has stored it"*, and §32.3 demonstrates that a `Plane` cannot: canonical under scaling and exactly square are incompatible, and `a_normal_written_square_to_a_direction_stops_being_square_once_a_plane_stores_it` pins that too |
| 2 | one pipe described twice is one pipe | **met** | `the_same_pipe_built_from_a_longer_axis_direction_is_the_same_pipe` |
| 3 | the half-space gap closed, the figures moved | **met** | `two_lines_off_the_lattice_are_told_apart_by_a_half_space_written_square_to_them` replaced the demonstration that asserted the opposite, and the shipped thresholds came down: a planar pair lacking a half-space from 2% to 0.5%, a quartic one from 15% to 4%, and `Undecidable` out of a turned quartic pair from 8% to 0.2% |
| 4 | a comparison too wide to square is squared elsewhere, and the window has a test | **met** | `a_comparison_too_wide_to_square_is_decided_after_it_is_slid_up_the_range` and `an_expansion_wider_than_the_window_cannot_be_squared_at_any_scale` |
| 5 | where three surfaces meet, certified on all three | **met, in §28.4's form** | `every_point_where_three_surfaces_meet_is_on_all_three_of_them.rs`, three families and a lattice control. "Certified ON" is "never certified OFF" one dimension down, because a box round a corner no `f64` names has real width; on the lattice, where nothing rounds, it is met to the letter |
| 6 | the count of meeting points is a sign, and the lattice refuses none | **met** | `a_certified_meeting_of_three_lattice_surfaces_never_refuses`; `MeetOfThree::At` carries one point or two and the discriminant's sign picks |
| 7 | three surfaces this cannot solve refuse by name | **met** | `three_curved_surfaces_meeting_at_a_point_are_refused_rather_than_solved` |
| 8 | points along a planar crossing ordered with no angle built | **met, and one authority short of what §23.8 asks** | `three_points_on_a_circle_come_back_in_the_order_they_run_round_it` and `three_points_on_a_bars_cap_circle_come_back_in_the_order_they_run_round_it` — a circle and an ellipse, no `atan2` anywhere. What the audit found under it is §39.3's fourth finding: `intersect` and `which_comes_first_round_a_crossing` decide "is this plane along the pipe's axis" two different ways and disagree |
| 9 | one authority on whether a curved face holds a point | **met** | `whether_a_curved_face_holds_a_point_agrees_with_a_mesh_of_the_same_face`, and `SolidView::face_holds_the_point` really is the one: `classify.rs`, `fuse/select.rs` and `validate.rs` all ask it and none has a second answer |
| 10 | point-in-solid over a curved solid, with the ON case | **met** | `the_middle_of_a_bars_wall_is_on_its_skin`, `a_point_a_hair_inside_a_balls_skin_is_inside_it` |
| 11 | the flat-face gates are gone and a block with a round hole is one solid | **met** | `a_block_with_a_round_hole_through_it_is_bounded_by_whole_circles`, which asserts one solid and the volume. `Error::FuseNeedsFlatFaces` and `Containment::NeedsFlatFaces` do not exist |
| 12 | the pieces of a cut curved face cover it and overlap nowhere | **met** | `the_pieces_a_curved_face_is_cut_into_cover_it_and_overlap_nowhere` |
| 13 | a fused curved solid validates, meshes and passes §23.4's oracle | **met in part** | Met for four shapes: `a_block_with_a_round_hole_meshes_into_a_closed_surface` under all three rules, `a_ball_cut_by_a_plane_meshes_into_a_closed_surface`, and `a_bar_notched_by_a_block_meshes_into_a_closed_surface` for a block and for a wedge. **The second shape the criterion names by test cannot be written**: `a_ball_with_a_pipe_through_it_is_refused_by_name` is what stands there, and §36.20's quartic ordering is why |
| 14 | roadmap §7.4's four remaining entries as a pass/refuse table | **met** | `the_four_curved_entries_as_a_table`, eight rows. Four are answered and four refuse, and every refusal is in §39.4 |
| 15 | the metamorphic properties over curved solids | **met in part** | `swapping_the_two_sides_of_a_curved_pair_changes_not_one_bit` and `uniting_a_block_and_a_bar_the_other_way_round_gives_the_same_solid` are met. `a_bar_with_a_copy_of_itself_taken_away_is_nothing_at_all` **cannot be**: two coincident cylinders refuse for want of a point inside a piece of a face, tolerant too, and the test asserts today's refusal by name with a block as the control that answers. It fails the day the refusal is closed, which is §19's shape and the right way to leave it |
| 16 | the two rates, family by family and never blended | **met, and re-derived at this gate** | `every_certified_fuse_of_a_lattice_curved_pair_is_decided_and_records_nothing` over three control families at 0%, and `how_often_a_certified_fuse_of_a_turned_curved_pair_refuses`. The turned rate is **0.00% of 384 fuses** and was judged against a ceiling of three fifths, written at §36.14 when the ball family beside it refused everything and never moved when §36.21 took that family to zero. It is asserted at zero now — see §39.6 |

**Criteria 5, 9 and 16 were named the load-bearing three and all three are met.**
The two that are not fully met are 13 and 15, and neither is a hole in the work:
both name a shape that refuses, both refusals are in §39.4 with a demonstration,
and in each case the test that stands there asserts the refusal rather than
quietly not existing.

### 39.2 What §31 got wrong, and the two predictions scored

§30.3 is the instruction manual and §31 followed it: mark every number confident
or guessed, name a test rather than claim a number, and list what the gate is
planning on top of a guess. That worked. **Every substantial thing §31 got wrong
is a number or an arithmetic route, and nothing it settled about meaning or
ownership had to be undone.** The list, because a settling document is worth what
its errors teach:

- **§31.2 row 4's degree was four too high** for two planes and a quadric —
  about 14 guessed, 10 measured — and §34.1 has the table.
- **§31.4's five candidates for `Plane` contained no fix**, and the fifth,
  "leave it", is what landed. The property the gate wanted went onto a new type
  that is not a surface (§33.1), which is the answer nobody at the gate saw.
- **§31.10 item 1's route 2 had the wrong precondition.** The gate said the
  gradient-determinant route decides "for a face whose arcs all curve the same
  way", which is convexity. §35.2 found the precondition is topological — a
  corner off the trimming surface — and has nothing to do with curvature.
- **`Picker::transformed` never came due.** §33.8 and §34.9 both said lane 4
  would need it. §36.5 says why it did not, and it is a finding rather than a
  reprieve: a seam is left out of the arrangement altogether.
- **§36.19 named the arc's blocker as one function in `rcad-geom`.** It was in
  `rcad-topo`, and the two candidate causes §36.19 offered were both innocent
  (§36.23). The picker was right and the joiner threw it away.
- **§31.3's seam is not an ordinary boundary edge**, and §31.3's "two routes for
  a corner" are three, because a pole of a ball has one surface round it.
- **§31.10 item 7 has not been answered.** Whether two tangent balls united
  coming back as two shells survives the manifold check is still a model this
  kernel calls valid and a machinist calls one part. The catalogue records the
  answer; nobody has said it is right.

**The prediction §31.10 made about where the money would go was right about the
file and wrong about the parts**, and §36.2 already scored it line by line:
`fuse/arrange.rs` was the place, and the cycle walk and `turns_anticlockwise`
were only names, while what had to be rewritten — which walks bound one region —
is not mentioned in the prediction at all.

**§30.8 item 11 against §31.8, settled.** §30.8 item 11 predicted M4 would make
`Undecidable` out of `mesh` reachable, because the first constructed corner in a
mesh domain would be a fuse's. §31.8 predicted it would not, on one line of
`assemble.rs`. **§31.8 is right, and this gate confirms it independently.**
`Assembly::write_into` calls `model.add_vertex(corner.point)` with a `Point` —
an exact triple — and the width goes onto the vertex as an `Approximation`
instead, which is where §17.2 says it goes. The only route to `Undecidable` out
of `mesh` is `refusal_for`'s `CannotTellIfLoopsTouch`, an exact `orient2d` over
two `f64` coordinates, and the exact level decides those. §26.6 item 4 named M3,
§30.8 item 11 named M4, and both were wrong for the same reason: what has to
change is what a `Model`'s vertices *are*, and that is still nobody's milestone.

### 39.3 What the claim audit found

Run before this section, per §16.10, in depth over `rcad-geom`,
`rcad-predicates` and `rcad-topo` — about sixty absence-arguing claims in the
first two and forty-seven in the third — and swept over `rcad-mesh` and `rcad`,
which changed **no files this milestone** and whose claims were last audited at
§26.5 and §30.4. Every `panic!` left in `rcad-mesh` is a `Surface`- or
`Decision`-grew-a-variant arm and all of them are dead against today's three and
four; the facade has none outside its doc examples.

It has now found real defects on five consecutive milestones. This time: **a
wrong answer handed back as a success, two panics on ordinary public input,
three more on a caller's bytes, a refusal classified a limit that is a gap
four public calls away, a regression this milestone put into somebody else's
crate, and eight sentences that read as care and answer the wrong question.**
That is the largest haul it has taken, against the milestone with the most new
surface, which is what §16.10 would predict.

**The worst one, and it would have merged.** `fuse/assemble.rs:1397` said, and
§36.23 repeated: *"`Model::validate` asks that both ends read exactly `Zero`
against `and`, and a corner this fuse solved for is exactly on a plane it was
solved from."* It is not. A corner of a curved arc is `meet_of_three_surfaces`
of two planes and a quadric, which is `through + t·direction` over a divisor, and
it lands on the plane exactly only when the line's direction is axis-aligned. A
unit block turned a fifth of a radian and notched into a bar of radius one is
enough: `Model::fuse` came back **`Ok` with a solid that fails
`Model::validate`** in a release build, and panicked in a debug one. A corrupt
answer returned as a success is what §4.1 ranks below a crash.

The fix is the one that same doc comment names as future work — the slop
`check_curved_face` already reads off a solved corner (§36.13), read in
`check_edges` too — and it went in with `Undecidable` there ceasing to be
reported as a broken model.

**Why nothing caught it is the finding under the finding.** Every family in
`how_often_a_curved_fuse_refuses.rs` turns *both* solids by one rotation. That is
an affine map of the pair: it preserves incidence exactly, so a cut plane stays
square to the bar's own frame in every draw and every corner keeps a coordinate
`f64` holds. **No test in this repository had ever turned one solid relative to a
curved one**, in the whole of M4. The new family does, sweeps every answer
through `Model::validate`, and fails on the tree before the fix.

**The second panic on ordinary public input, and it was fixed too.**
`Model::add_round_bar(1e-200, 10.0)` builds a bar; meshing it panicked, with no
file anywhere near it. `Conic::angle_of` divides by `and.length_squared()`,
which is `1e-400` and underflows to zero, so the angle a walk turns over comes
back `0.0/0.0`. `how_many_steps` has an `is_nan` check for exactly that and **it
could never fire**: `f64::max` hands back the other operand when one is `NaN`,
and the `.max(1.0)` in front of the check turned every `NaN` into a one. A whole
cap circle was then walked in one step with no points between its ends, and the
flat cap that reads those points back fanned into a single corner. The floor
goes on after the checks now, and the refusal is
`CannotWorkOutHowBigACrossingIs`, whose own doc already carries the
classification.

**What made that one worth the audit is where the claim sat.** `mesh`'s own
`# Panics` said *"Nothing does … a closed edge on a curved face is walked four
ways round whatever the deflection, because the evaluator will not step past a
quarter turn either."* The argument is sound and names the guard. The guard is
the one the `NaN` walks past.

**Two more that reach a public entry point, and one regression, none fixed
here.** Both are the milestone's own doing and both are §39.7's.

**`ACurvedFaceGoesRightRoundWithNoSeam` is reachable in four public calls
and is a gap, and §38.6 filed the family as a limit.** A closed curve on a
sphere leaves *three* kinds of piece — one holding one pole, one holding
none, one holding both — and only the first is handled. `add_ball(4)`
intersected with a `add_cube(20)` placed so its face sits at `y = 1` refuses;
the answer is a spherical cap of volume 84.823, which is the same solid
`a_ball_cut_by_a_plane_meshes_into_a_closed_surface` certifies when the plane
is `z = 1`. **And it is a §19 violation, which is the serious half**: turning
the *whole arrangement* — ball and slab together — about `x` flips a working
answer into a refusal, measured at 1.00 to 1.30 radians answered and 1.40 to
1.50 refused, either side of `acos(1/4) = 1.3181`, which is the angle at
which the kept cap stops holding a pole. A rigid motion of both operands is
not allowed to change the answer. §38.9's correction — *"a plane through the
ball's centre that does not contain the polar axis is answered"* — is true
and answers the wrong question: through the centre every piece holds exactly
one pole, and off-centre is where the family splits.

**M4's fuse builds the pipe `rcad-mesh` says nothing builds.**
`triangulate.rs` defers a gap in its own error bound on the grounds that
*"`rcad-topo` never builds one: `add_round_bar` puts the axis at the origin
and carries the placement on the solid"*. The curved fuse **bakes the
placement into the stored surface** — measured out of the serialized model, a
bar built at `z = 1e9` stores its axis origin at `0` before a fuse and at
`1e9` after one. So the same bar, in the same place, put through a
`Difference` against a block fifty units away that cuts nothing, goes from
meshing at `1e-5` to refusing everything under `3.07e-4`: **a 77-fold rise in
the mesh floor caused by a boolean that removed no material.** The true
radial error of those points is about `1e-16`, so a representable answer is
refused. A **gap**, a regression this milestone introduced, and the sentence
that hid it is the audit's canonical shape — an unreachability escape that
was true when written and that this milestone's headline feature falsified.

**Three panics reachable only through `Model`'s `Deserialize`**, and §39.7 hands
them to M5:

1. `Model::validate` panics — *"two planes cross in a straight line, which never
   closes"* — on a cube with one extra closed edge spliced into two of its wires.
   `validate` is the untrusted import boundary and `validate.rs:26` says so.
2. `SolidView::mesh` and `SolidView::measure` panic in `rcad-geom`'s
   `NamedRun::new` on a bar whose cap circle was given two different ends.
   Nothing checks that a `RightRoundWhereItsFacesCross` edge starts and ends at
   one vertex, and `SolidView::mesh` is public in its own right.
3. `SolidView::mesh` panics `TooFewCorners` on a bar whose wall face carries a
   `Plane` instead of its `Cylinder` — a four-step wire that fans into one corner
   in the plane's two coordinates.

**And one refusal that is nobody's panic.** `Exact::dot_of` gives up on a
direction with a subnormal component, and the sentence beside it said *"no
scaling of a **unit** direction reaches the answer"*. True, and about a family
nobody proposed. Multiplying both triples by `2^300` rounds nothing, cannot
change the sign of a dot product, and decides it. The fix is a slide inside
`Exact::dot_of`, which `Exact::square_minus` has had since §32.1.

**Two behaviour defects fixed here besides the first.** `Exact::product_of`
stored `[0.0, high]` for every product that came out exact, and
`exponent_that_makes_its_square_exact` read that zero as the smallest entry and
answered "already as low as it goes" — so a number with a good square somewhere
else was refused where it stood, at any magnitude. No call site hands it a raw
product today, which is why a green suite never saw it; the doc that said an
expansion's entries are non-zero is what kept anyone from looking. And
`HalfSpace::through_a_point_square_to` handed a zero `towards` refused for want
of a normal with three good ones already in hand, because a zero triple is
exactly square to everything, vacuously, and won the ranking.

**One disagreement between two authorities**, which is §23.8's own failure mode
and no code review sees it because neither site is wrong alone. `intersect`
decides "is this plane along this pipe's axis" with the certified cascade over
the exact dot; `along_a_crossing.rs` and `walk.rs` decide it with `== 0.0` on the
rounded `f64`. An axis along `(1, 0, 3)` against a plane facing `(−12, −12, 4)`
is a pair of small whole numbers where the two disagree completely: `intersect`
names one closed curve and `which_comes_first_round_a_crossing` refuses it as
"not a conic". Pinned by a test that fails the day it is closed.

**Seven sentences rewritten and none of them a defect**, which is the other half
of what this audit is for. `MOST_PIECES_A_FACE_IS_CUT_INTO` named the one
condition that provably cannot reach it. `FaceView::plane`'s "a bug in this crate
rather than bad input" is three lines from outside. `a_point_off_the_plane` said
`None` needs a normal that is not a real number, when what it needs is a corner
at the top of the range — the step *starts* at the corner's magnitude and
overflows on the first try, with a good answer at a smaller one. `MeetOfThree`'s
"the two can never be confused" is true of every route through `intersect` and
not of the type. And `Plane::meet_of_three` can hand back `Ok` holding a box with
a side that runs the whole number line, which its `# Errors` did not say.

**The rule this run adds**, and it is stronger than "check the public API":
**a `# Panics` clause on a `pub` item needs a demonstration that the arenas
cannot hold the shape, not an argument that geometry forbids it.** `Model` is
`Deserialize` and the arenas hold whatever the bytes say. That is the fourth
milestone running in which this exact species has been the audit's main catch.

### 39.4 Every refusal M4 can produce, limit or gap

§30.5 is the precedent and §16.10 the rule: a **limit** where the answer does not
exist, a **gap** where it does and the arithmetic could not reach it. Thirty-three
rows. **Every inherited row was re-checked rather than copied, four moved, and
three are new at this gate** — two of them found by the claim audit after the
rest of this section was written, which is the argument for running it last.

Rows M3 could already produce are §30.5's and are not repeated except where M4
moved them.

**Where two or three surfaces meet — `rcad-geom`**

| Refusal | Cause | | Demonstration |
|---|---|---|---|
| `WhereThreeSurfacesMeetNeedsTwoOfThemFlat` | fewer than two of the three are planes | **gap** | `three_curved_surfaces_meeting_at_a_point_are_refused_rather_than_solved`. §31.2 defers it and names the consumer: two capped bars crossing near a cap |
| `Undecidable` out of `meet_of_three_surfaces` | a product the exact arithmetic needs leaves `f64`'s range | **gap** | the `2^k` family's **10.83%** against 0.00% for the same triples on the lattice |
| `Predicate(PlanesDoNotMeetInAPoint)` | three planes meet at a point neither Cramer's rule nor the line route can bound | **gap** | inherited from `meet_of_three_planes` and now reached far less often |
| `Undecidable` read out of `meet_of_three_planes`'s own `Ok` | three planes meet **past the top of the range**, and Cramer's rule hands back a box with an infinite side | **limit — §34.10 said gap and was wrong** | `three_planes_meeting_past_the_top_of_the_range_are_refused_rather_than_placed_nowhere`. §34.10 argued it a gap because the same three planes lower down are placed; that is a different triple with a different answer, and *this* triple's answer is a point at about `1e315` that no `f64` triple names. Whose it is: **`rcad-predicates`'**, which returns the box, and `rcad-geom`'s `meet_of_three_surfaces` reads it as the refusal it is. The frozen crate keeps it and §39.7 hands it on |
| `TheseTwoSurfacesDoNotCrossInAConic` | two planes, or a plane down a pipe's own axis | **limit** | the question is a different one and the arrangement's dominant-axis rule answers it |
| `TheseTwoSurfacesDoNotCrossInAConic` | a plane whose dot with the axis cancels in `f64` and not exactly | **gap, new at this gate** | `a_conic_the_intersector_names_is_refused_here_because_the_two_ask_it_differently`. The pair is a conic, `intersect` says so, and this refuses it |
| `TheseTwoSurfacesDoNotCrossInAConic` | a ball against a pipe, or two pipes | **gap** | the curve is there and ordering points along it takes the marcher. **2026-10-09, Q1483:** closed for two pipes where the fat one swallows the thin one's whole cross-section — each of the two loops runs once round the thin pipe's axis and is ordered round it by the determinant, no marcher. Two pipes of one radius, two crossing in one loop, and a ball off a pipe's axis are still refused. The cross-drilled shaft that needed it is refused later, by the fuse's check (§421) |
| `TheCrossingOfTheseTwoSurfacesIsNotClosed` | a crossing that runs on for ever | **limit** | the vertex does not exist: such an edge is cut by corners at both ends rather than closing on itself |
| `Undecidable` out of `intersect` | a direction with a subnormal component underflows its own product | **gap** | `a_direction_with_a_subnormal_component_cannot_be_told_from_square_to_a_plane`, whose second half slides both triples by `2^300` and reads the sign |
| `CannotWorkOutHowBigACrossingIs` | **a bar too thin for its own squares** — `and.length_squared()` underflows, so the angle a walk turns over is `0.0/0.0` | **gap** | `a_bar_too_thin_for_its_own_squares_is_refused_rather_than_asserted_against`, with radius 1 and `1e-100` as the controls. It was a **panic** until this gate, on `add_round_bar` output and nothing else |
| `DeflectionBelowWhatAMeshCanReach` | **a curved solid that has been through a fuse**, whose placement the fuse bakes into the stored surface | **gap, and a regression M4 introduced** | a bar built at `z = 1e9` meshes at `1e-5` before a `Difference` against a block fifty units away that cuts nothing, and refuses everything under `3.07e-4` after it — a 77-fold rise from a boolean that removed no material, against a true radial error of about `1e-16`. §39.3, and undemonstrated in the suite |

**Whether a curved face holds a point — `rcad-topo`**

| Refusal | Cause | | Demonstration |
|---|---|---|---|
| `AFaceTrimmedByACurveThisCannotFollow` | an edge of the face runs along *part* of a crossing | **gap** | `a_face_trimmed_by_part_of_a_curve_is_refused_by_name`. **2026-09-29, Q596 (`4375e684`):** closed on the side-by-side answers — `holds.rs` now follows an arc of one pipe against another running the same way, the two discs of a union cap taken together, a disc less a disc, a pipe running the same way as a cut along a wall axis, and a reversed wall walk as the view gives it (§331.5). **2026-10-08, Q1367 (`dbb7585b`):** closed further — `holds.rs` now follows a face on a ring cut by planes through its axis, no two of them side by side and at most one plane square to it (`leaves_a_ring_in_one_piece`, `are_two_planes_through_a_rings_axis`): the top half of a quarter or a half bend's wall, pinned by `a_quarter_and_a_half_bends_wall_hold_their_own_points` |
| `AFaceMissesPartOfWhereItsSurfacesCross` | the face runs along some but not all of the curves its pair crosses in | **gap** | `a_face_along_one_of_two_loops_is_refused_rather_than_named_as_both`. **2026-10-11, Q1723:** that test is gone — Q1717 replaced it in `holds.rs` with `each_patch_a_thin_bar_keeps_of_a_fat_bars_wall_holds_its_own_end_alone`, since two pipes crossing in two loops are read now. The refusal still fires, demonstrated by `the_wall_holds_the_half_of_the_ring_its_wire_keeps` in `crates/rcad-topo/tests/a_half_rings_flat_face_read_back_from_bytes.rs`, naming the wall of a half-ring whose wire keeps different sides of `z = 0` at its two circles |
| `NothingSaysWhichSideOfItsTrimmingAFaceLiesOn` | every corner of a curved face sits on the surface trimming it | **gap, undemonstrated** | the side exists and the model does not record it |
| `NothingSaysWhichSideOfItsTrimmingAFaceLiesOn` | a **flat** face with a corner at the top of the range and a normal off the axes | **gap, new at this gate, undemonstrated** | `a_point_off_the_plane` grows its step from the corner's own magnitude, so the first try overflows and every one after it does; a smaller step reaches an answer. Reachable from `Deserialize` only |
| `Containment::CannotFollowAFacesTrimming` | any of the above, met while counting a ray | **gap** | the above, one layer up. Closed on the side-by-side answers by the same Q596 fix (§331.5) |
| `Undecidable` out of `face_holds_the_point` | a sign about a box could not be settled | **limit** where the box straddles the surface, **gap** where the arithmetic ran out | no generator reaches either |

**The curved arrangement and the commit — `rcad-topo`**

| Refusal | Cause | | Demonstration |
|---|---|---|---|
| `ACrossingThisCannotBeOrderedAlong` | a ball one rounding off a pipe's axis, with a bar through it — the quartic. Centred on the axis it crosses the bar in two circles instead, which the arrangement now orders (Q1140, `6e0940d4`), and the fuse answers under every rule and a tolerant policy too (Q1241, `a_ball_with_a_bar_through_its_centre_is_answered_under_every_rule`) | **gap** | `a_ball_one_rounding_off_a_bars_axis_is_refused_before_any_order`. **Not a budget question**: `Policy::tolerant` refuses it too (§37.7), verified again here. §36.20 and §34.10 |
| `Undecidable`, *order along a crossing* | a **turned notched bar** — §34.5's determinant, in the arrangement | **gap** | `a_turned_notched_bar_is_refused_for_want_of_an_order_along_a_crossing`, tolerant too, and the same shape unturned is decided and records nothing |
| ~~`Undecidable`, *order along a crossing*~~ | ~~a block turned by −π/4 to lean through a bar's rim start — a second cause, met ordering the meet and the rim start round the cap circle~~ | **closed** | a meet of two planes and a pipe now narrows by one checked step and the order reads that narrower box exactly where the unnarrowed one could not decide it; all six fuses answer as `a_block_turned_to_lean_through_a_bars_rim_start_answers_all_six_fuses_with_one_solid_each` (Q209, `164b8f73`, §401) |
| ~~`ResultIsNotManifold { count: 1 }`~~ | ~~under a **tolerant** policy, that same turned block is ordered, and its faces disagree on a corner~~ | **closed** | with the order proved, the tolerant policy is never asked at this meet and the fold that disagreed never happens; all six tolerant fuses answer too, as `under_a_tolerant_policy_the_turned_block_answers_all_six_fuses_with_one_solid_each_too` (Q209, `164b8f73`, §401, §284.8) |
| `Undecidable`, *a point inside a piece of a face* | a **bar cut by a single plane not through its axis** | **gap** | `a_bar_cut_by_one_plane_off_its_axis_is_refused_for_want_of_a_point_inside_a_piece`. §36.28, and §38.9 says the coincident-face gap below is entangled with it: a knife whose face is exactly the bar's own cap plane refuses here too |
| `Undecidable`, *a point inside a piece of a face* | two coincident curved faces | **gap** | `a_bar_and_a_copy_of_it_in_the_same_place_are_refused_for_want_of_a_point_inside_a_piece`. The answer is the bar. Tolerant too. **This is what criterion 15's second name runs into** |
| `Undecidable`, *a point inside a piece of a face* | two tools meeting on one plane that cuts a round wall | **gap** | `two_tools_meeting_on_a_plane_that_cuts_a_round_wall_are_refused_by_name`; one tool making the same cut is answered |
| `ACurvedEdgeThatIsPartOfACrossingCannotBeNamed` | **an arc whose chord rounds onto its own tangent** | **gap** | `an_arc_whose_chord_rounds_onto_its_own_tangent_is_refused_by_name`. The half-space exists; `f64` cannot reach it at that separation. §36.24, §36.26 |
| ~~`TwoCurvesTouchWithoutCrossing`~~ | ~~a plane tangent to a bar's wall, or a block sitting exactly on a bar's cap~~ | **closed** | the wall answers since Q649 (`a_wall_lying_on_a_bars_skin_runs_up_the_bars_axis_and_is_answered`) and the block on the cap since Q930, both ways round and turned by quarters (`a_block_sitting_on_a_bars_cap_meets_it_at_one_point_under_every_rule`); the Q649 and Q930 notes below |
| ~~`TwoCurvesTouchWithoutCrossing`~~ | ~~**a corner exactly on a curved face** — a `3 × 4` block corner on a bar or a ball of radius five~~ | **closed** | the ball's answers (`a_block_corner_exactly_on_a_balls_skin_is_answered`), and since Q930 the bar's, under every rule and a tolerant policy too: two straight crossings on one line are folded into one (`a_block_corner_exactly_on_a_bars_wall_is_answered`, §38.2, the Q930 note below) |
| `AClosedEdgeOfTheAnswerWouldStartAtAPole` | **a ball cut by a plane through its polar axis** | **gap** | `a_ball_cut_by_a_plane_down_its_polar_axis_is_refused_by_name`, with the equator and a leaning plane answered as controls. §38.3 |
| `AClosedEdgeOfTheAnswerWouldStartAtAPole` | **2026-10-14, Q1985: a ball centred on a bar's wall**, so both its poles lie on the wall and on the one loop the ball crosses it in — a ball is parametrised about world `z` whatever it is turned by | **gap** | `a_small_ball_centred_on_a_bars_wall_is_refused_where_its_poles_lie_on_the_wall` in `crates/rcad-topo/tests/a_walk_neither_winding_nor_the_axis_places.rs`: a ball of `0.5` on a bar of 2, every rule both ways round, with a slab along the bar and without, on the seam and a quarter turn round. A probe moved the centre `0.1` in or out and every fuse answered, volumes not checked. Reached since Q1777 (the `CannotTellTheCellsOfAFaceApart` row below) |
| `ACurvedFaceGoesRightRoundWithNoSeam` | a cut meets a ball's seam twice | **gap** | `a_ball_cut_by_a_plane_that_crosses_its_seam_twice_is_refused_by_name`. The answer is two caps of a ball |
| `ACurvedFaceGoesRightRoundWithNoSeam` | **a cut leaving a piece of a ball that holds no pole, or both** | **gap, and a §19 violation** | four public calls: `add_ball(4)` intersected with `add_cube(20)` whose face sits at `y = 1`. The answer is a spherical cap of volume 84.823 — the same solid `a_ball_cut_by_a_plane_meshes_into_a_closed_surface` certifies when the plane is `z = 1`. **Turning the whole arrangement flips it**: answered from 1.00 to 1.30 radians about `x` and refused from 1.40, either side of `acos(1/4)`, which is where the kept cap stops holding a pole. §38.6 filed the family a limit; a closed curve on a sphere leaves three kinds of piece and only the one-pole cap is handled |
| `ACurvedFaceGoesRightRoundWithNoSeam` | a face with no boundary at all on a surface that does not pinch | **limit** | such a face is unbounded and there is no answer. Reachable from `Deserialize` only |
| ~~`ACurvedFaceGoesRightRoundWithNoSeam`~~ | ~~**2026-09-29, Q553: a fuse of a ball and two blocks meeting its equator from opposite sides** — the skin comes out as pieces of the ball meeting along the equator, and the mesher reads which pole a cap reaches off its *first* coedge, which here has the ball on both sides~~ | **closed** | `which_pole_this_cap_reaches` now reads the pole off the walk round the cap's own rim instead of a partner coedge (Q508, `dfa75f1e`, §330.6); all six fuses answer, `a_ball_less_or_united_with_two_blocks_meeting_its_equator_from_opposite_sides_keeps_all_of_it_but_the_two_pieces` in `crates/rcad-topo/tests/two_flat_faces_on_one_plane_facing_apart_along_an_arc.rs`, measuring `468.744` against `448π/3` and `3268.744` against `2800 + 448π/3` (Q508, `a7ad3f4b`) |
| `CannotTellWhichCurveOfACrossingAFaceRunsAlong` | a boundary edge's corner is held by two of the curves its pair names, or by none | **gap, undemonstrated** | a picker's `Undecidable` rules nothing out (§33.8) |
| `CannotTellTheCellsOfAFaceApart` | two regions of one face were joined that the surface keeps apart | **gap, undemonstrated** | telling them apart takes a point inside each. **2026-10-14, Q1777 and Q1985:** `the_one_on_its_side_of_every_cut` in `fuse/arrange.rs` now tells the cells of a fat wall apart by which side of each cut surface each lies on, so a bar of radius 1 ending at the axis of a bar of radius 4 answers every rule both ways round with its closed forms (`a_thin_bar_ending_at_a_fat_ones_axis_and_each_answer_beside_a_far_cube_hold_their_closed_forms` in `two_loops_named_apart_across_a_far_cube.rs`). Four pins left this row with it, none for a wrong answer: in `a_plane_cut_along_a_cones_other_nappe.rs`, the steep cone with a block leaned `4` through its apex, turned `0` and `1`, is refused `Invalid(DynamicRangeTooWide)` on every rule, a limit pinned by ratio beside the unit cone's (`a_steep_cone_and_a_lean_four_block_through_its_apex_are_refused_as_too_wide_a_range`), and turned `3` is refused `a point inside a piece of a face` (Q464); and the ball on a bar's seam moves to the second `AClosedEdgeOfTheAnswerWouldStartAtAPole` row above |
| `AFaceMeetsItsNeighbourOnItsOwnSurface` | two faces on one cylinder with an edge between them | **gap**, undemonstrated here; §325.4 reaches it from the public API | what closes it is a joiner written in surfaces |
| `ACurvedFaceCannotBeCutSmallEnoughToMesh` | sixteen cuts that each narrow both halves | **gap, undemonstrated, no generator reaches it** | measured at one or two pieces over every curved fuse, at deflections from `1e-4` to 3 |
| `CurvedFaceWithAHoleGoesTooFarRound` | a curved face with a hole that is not a ring, going more than a quarter turn round | **gap, from a file only** | the fuse's ring is cut across into four patches and a hole that does not go right round has nothing to cut across to — but such a face has a triangulation and the cutting rule cannot find it. §31.10 item 5 filed the old doc as a claim M4 would falsify, and it did: *"the fuse refuses a curved solid outright"* stopped being true. What still keeps a fuse away from it is the row above — the only fused face with a hole in it is a ball with a pipe through it, and that refuses first |
| `Undecidable`, *which piece of a face a hole is in* | no ray through a lump of endless curves gave an answer | **gap**, unchanged from §17 | |
| `Undecidable`, *which way a crossing runs* | the direction worked out at a box straddles every axis | **gap**, and unreached since §36.8 | |
| `Undecidable`, *which way an edge runs round a closed crossing* | an edge's tangent at its start could not be signed against the arc's own direction | **gap, undemonstrated** | `which_way_an_edge_runs_round` in `fuse/arrange.rs`. Missing from this table since Q617; added by Q1243 |

> **2026-10-02 note (Q649): a plane or a bar lying along a bar.** The
> `TwoCurvesTouchWithoutCrossing` row for "a plane tangent to a bar's wall, or a
> block sitting exactly on a bar's cap" had stopped describing either: both
> refused `Undecidable` on *which way a crossing runs*, and not for the cause the
> last row names. Along the line where a plane or a pipe rests on a pipe the two
> gradients are parallel, so `∇S × ∇Q` is exactly nothing there. That is a
> **gap**: the line runs along the pipe's stored axis, which
> `fuse/arrange.rs`'s `AlongATouch` now carries. A wall on a bar's skin that
> also cuts the bar answers every rule, with the closed-form volumes
> (`a_wall_lying_on_a_bars_skin_runs_up_the_bars_axis_and_is_answered`). A bar
> clear of a wall, in an inside corner, or beside another bar answers its
> intersection and both differences, and its union refuses
> `ResultIsNotManifold { count: 4 }`. The likely cause is that both solids keep
> every face along the line, so it comes out one edge with four faces round it,
> but nobody has traced that. That union's answer is the two solids as they
> stand, so it is a gap too, and queue row Q371 traces and fixes it
> (`a_bar_lying_along_a_plane_or_a_bar.rs`; traced and fixed by Q760, in the
> note below). The block on a bar's cap refuses
> `TwoCurvesTouchWithoutCrossing` again, under every rule (traced and answered
> by Q930, in the note below). A plane
> lying along a cone's ruling still refuses on *which way a crossing runs*,
> unchanged: a ruling has no stored direction, and it is queue row Q473's
> (closed by Q473, in the note below).

> **2026-10-03 note (Q760): the union of a bar lying along a plane or a bar is
> answered.** Traced on the bar beside a cube's wall, with a print of the
> claimants where `darts_along_each_edge` counts them: the touching line
> `(4, 2, 1)–(4, 2, 3)` is one edge with four darts, two from the two pieces of
> the cube's wall either side of it and two from the bar's skin, which is one
> piece cut open along the line and walking it both ways. The guess held, with
> that one difference: the skin is one piece, not two. A union keeps both
> sides of the line in both solids, because the insides are apart, so the line
> is no edge of either input. `fuse/assemble.rs` now parts an edge with four
> darts into two where they are two pairs, each pair from pieces of **one input
> face** walking it opposite ways; the wall's half is dissolved as any edge
> between two pieces of one plane is, and the skin's stays as its seam. The
> union is the two solids as they stood, two lumps with the inputs' volumes, in
> all four cells of `a_bar_lying_along_a_plane_or_a_bar.rs` both ways round,
> the bar with its seam on a cube's wall, and the bar touching a tube's inner
> wall; `fuses_that_touch.rs` answers all 78, 84 and 60 fuses of its three
> line-touch cells. The parting fired in those nine tests and in no other test
> of `rcad-topo`. **Four darts from four input faces stay refused**: the frame
> and the comb, two cubes along part of an edge, the frame less the block over
> its corner, and two quarter bends end to end still refuse
> `ResultIsNotManifold { count: 4 }`, and §80.9's decision that two solids
> touching along an edge of both are not built is untouched. (Two of the four
> still refuse, each two solids touching along an edge of both: the frame and
> the comb, and the two cubes, both in `degenerate_fuses.rs`. The two quarter
> bends answer since Q756, §378.6: their four darts came from two circles
> given one name. The frame less the block answers since Q1913, in the note
> below.)

> **2026-10-06 note (Q803): a prism lying on one of its own edges parts four
> darts too.** `part_each_line_two_faces_rest_against_each_other_along` now
> also parts a line into two edges where only one of its two pairs is pieces of
> one input face and the other pair is two faces of the other solid — a prism
> resting on a block's top face along one of the prism's own edges — but only
> where those two faces lie on the outer side of the face they rest on and no
> other edge joins the two pairs into one shell
> (`a_prism_lying_on_one_edge.rs`). A block less a prism lying inside it along
> its edge
> (`a_block_less_a_prism_lying_inside_it_along_its_edge_is_refused_and_the_rest_answer`)
> and a solid lying on a block along one edge and reaching into it elsewhere
> (`a_solid_lying_on_a_block_along_an_edge_and_reaching_into_it_is_refused_united`)
> stay refused `ResultIsNotManifold { count: 4 }` (answered since Q1901, in the
> note below).

> **2026-10-13 note (Q1901, from Q1116): a solid touching itself along a line
> is an answer.** The two guards of the Q803 note are gone:
> `rest_on_the_outer_side_of` in `the_two_pairs_along`, and
> `a_line_whose_two_sides_are_joined_some_other_way` in
> `part_each_line_two_faces_rest_against_each_other_along`, both in
> `fuse/assemble.rs`, with `side_of_a_corner_of` and
> `the_three_planes_meeting_at_a_corner_of`, which only the first reached
> (§438's note of the same day). Refusing that answer was a **gap** by §16.10,
> not a limit. Q1116's lane argued it, and the dispatcher took it:
>
> 1. **The answer exists as a point set.** The block less the prism lying
>    inside it along its edge is `240 − 24 = 216`, the block united with the
>    solid that lies on it along one edge and reaches into it elsewhere
>    `240 + 42 − 6 = 276`.
> 2. **It can be written down.** §17.4 refuses a result that is not a
>    manifold because `check_touched_edge` asserts two coedges along each
>    edge, and a parted line has two along each of its two edges.
> 3. **Parting cannot pass two faces through each other.** Faces round a line
>    alternate in which way they walk it, and `the_two_pairs_along` pairs only
>    darts walking it opposite ways, which leaves out the one pairing that
>    would cross. So Q803's two guards were a choice of what to answer, not
>    what kept an answer sound.
> 4. **The kernel already wrote such answers.** Q953's wedge less the frustum
>    fuses on with a far cube: the union two solids, the difference one, the
>    intersection none. Q760's parting answers the stepped block `(0, 0)`,
>    `(6, 0)`, `(6, 1)`, `(4, 1)`, `(4, 4)`, `(0, 4)`, 4 high, united with a
>    round bar of radius 1 at `(5, 1.75, 1)`, 2 high, as one shell of 11
>    faces, `77.807457349804`, and it validates. Both as Q1116's lane
>    measured them.
> 5. **§80.9 is untouched.** It is about a line that is an edge of both
>    inputs, four darts from four input faces, with a different solid on each
>    side, and that stays refused. As first written, this point kept every
>    line of four input faces refused, since where neither pair is pieces of
>    one input face `the_two_pairs_along` still asks for two flat faces on one
>    plane facing one way. That was wider than §80.9, which is about two
>    touching solids: an L plate less a pocket cut into its corner square is
>    one solid touching itself along such a line, its two arms joined by the
>    plate's floor. **Since queue row Q1913 the same day**,
>    `pair_each_line_one_solid_touches_itself_along` in `fuse/assemble.rs`
>    parts such a line by input, the two faces round each input's edge a
>    pair, where a face of one pair and a face of the other walk the line the
>    same way and are joined without it. Faces round a line alternate in
>    which way they walk it, and so do the full and empty wedges between
>    them, so those two faces bound the two different full wedges, and joined
>    they are one solid. Joined means across an edge with two faces along it,
>    or across a line already parted this way. Where they are not joined, as
>    with two cubes along part of an edge, the line stays one edge, refused
>    under §80.9.
>
> Measured with the guards gone, in `a_prism_lying_on_one_edge.rs`: the block
> less the prism inside it is one solid of two shells, the block's six faces
> and the hollow's five, touching along the prism's edge; the block united
> with the hook is one shell of 13 faces; each validates
> (`each_answer_touching_itself_along_a_line_is_the_shells_its_faces_part_into`).
> Each fused with a cube of 1 far from it, every rule both ways round, gives
> the union as the two as they stand, the intersection as no solid, and each
> difference as its first solid whole
> (`each_answer_touching_itself_along_a_line_fuses_on_with_a_far_cube`). The
> two pins of the Q803 note now answer every rule both ways round, as
> `a_block_less_a_prism_lying_inside_it_along_its_edge_answers_every_rule_both_ways_round`
> and
> `a_solid_lying_on_a_block_along_an_edge_and_reaching_into_it_answers_every_rule_both_ways_round`.
> §417.9's block less the tipped cube, refused `ResultIsNotManifold { count:
> 4 }` under a tolerant policy at the five tilts
> `TILTS_THE_UNION_WAS_REFUSED_AT` of
> `a_cube_tipped_onto_one_edge_along_another_cubes_floor.rs`, is one solid of
> `1000` less what the two cubes share under either policy there
> (`the_block_less_the_tipped_cube_is_the_block_less_what_they_share_where_the_union_was_refused`).
> Queue row Q1902 asked whether Q760's touching answer for the stepped
> block is refused `ArgumentIntersectsItself` by every fuse after it;
> these two answers are not, and since Q1902 the input check reads two
> faces resting on each other along a line as no crossing, so that
> answer fuses on too
> (`a_bar_resting_on_a_stepped_blocks_wall_fuses_on_with_a_far_cube`,
> `a_bar_resting_on_a_stepped_blocks_wall_fuses_with_a_slab_across_the_touch`).
>
> The decision moved five pins outside Q1901's own files, which queue row
> Q1910 moved, each answer validating and holding its volume worked by hand.
> In `a_block_edge_grazing_a_ball_at_cx_16.rs`, the union either way round
> and the difference with the ball first, refused
> `ResultIsNotManifold { count: 4 }` before, now go on to the next refusal:
> `CannotMeshAFace` `CornersCoincide` at position 5, like the other three
> fuses there, except the union with the block first, which is refused
> `Invalid::FacesDoNotAgreeWhichWayTheyFace`, under either policy
> (`every_fuse_refuses_on_the_two_places_f64_holds_as_one_corner`). In
> `a_sliver_along_an_edge_of_a_face_it_lies_on.rs`, the leaning frame
> united with the comb answers one solid of `19`, both ways round. In
> `a_lens_whose_hole_is_half_a_disk.rs`, the bar at `(−3, 0)` less the half
> bar of radius 1 at `(0, −3)` answers `98.5π`, and less the half bar of
> radius 4 at the origin `76π`; the file had called the second a limit. Two
> kinds stayed refused `ResultIsNotManifold { count: 4 }` under point 5 above,
> each along a line that is an edge of both inputs, four input faces round
> it: the frame less the comb, `8`, and the two bars united less either half
> bar, `200π − 4L − 1.5π` and `176π − 4L` for the lens's area `L`. So did the
> L plate less the pocket in its corner square, which Q1910 ran as a shape a
> designer draws (`a_prism_lying_on_one_edge.rs`).
>
> Queue row Q1913 moved them, by the amended point 5. The L plate less the
> pocket answers one solid of `2.5`, and the two bars united less the half bar
> of radius 4 answers `176π − 4L`. So does the frame 3 by 5 less the block 4
> by 4 over its corner, which the Q760 note above lists as refused: one solid
> of `11`, one shell of 12 faces, which validates, upright and turned. The
> frame's ring is cut open at that corner, and its two ends touch along the
> hole's corner `x = 2, y = 4`
> (`a_frame_with_a_block_over_one_corner_fuses_turned_as_it_does_upright`,
> which compares turned with upright only). A pin no row had named moved too: the
> leaning comb united with the leaning ell standing on it, in
> `a_sliver_along_an_edge_of_a_face_it_lies_on.rs`, answers one solid of `60`
> both ways round. The ell stands on both teeth and touches the comb along
> the bottom of the gap between them, an edge of both. The two bars united less the half bar at
> the lens corner is parted, and then panics in `rings_round` as the union of
> the same two already did: the bars' cap touches itself at the lens corner
> `(0, −4, 4)` in both (queue row Q1290). The frame less the comb is still
> refused, rightly. It is not one solid of `8` but three lumps: the frame's
> ring with the near wall's piece left of the middle tooth, `6.5`; the square
> between the middle and right teeth, `1`; and the triangle `(3, 0)`,
> `(4, 0)`, `(3, 1)`, `0.5`. The square touches the ring along `x = 2,
> y = 1`, so a different solid lies on each side of that line, and §80.9
> leaves it unbuilt.
>
> Q1913's lane found the rule older than its own disagreeing with it. Where
> one input face runs straight through the line and the other input has an
> edge along it, the `(true, false)` arm of `the_two_pairs_along` parts the
> line without asking whether one solid lies on both sides. A block 4 by 2,
> 1 high, leaning back one along `x` for each step along `y`, less a block 1
> by 3, 1 high, at `(2, −2, 0)`, answers one solid of one shell of 12 faces,
> `7`, which validates: the block's lump of `6.5` and the triangle `(3, 0)`,
> `(4, 0)`, `(3, 1)` of `0.5`, touching along `x = 3, y = 1`, where the
> block's wall `x + y = 4` runs straight through. That is two touching solids
> written as one, where the rule above refuses the same relation at a line of
> four input faces. It was built only as a probe, not kept, and left to a
> queue row.

> **2026-10-04 note (Q473): a plane lying along a cone's ruling runs along the
> ruling.** The gap above, closed: `AlongATouch` carries the ruling,
> `(n·n)w − (n·w)n` for the plane's normal and the cone's axis, read only once
> `rcad-geom`'s two proved doors say the apex is on the plane and the lean is
> nothing, so a tolerant `intersect`'s touch is refused rather than read. A
> face on the cone lies on the side of the plane its nappe is on, read off its
> corners. Each other face of the solid meets the resting plane in an edge
> that touches its conic of the cone on the ruling, at a meet no box narrows;
> `turns_anticlockwise` reads that turn as the touch it is proved to be.
> `degenerate_curved_fuses.rs`'s wedge, its back corner moved to `(−5, 1, 8)`,
> answers the intersection both ways round and the frustum less the wedge, held
> to volumes worked out level by level. Left: the union both ways round and the
> wedge less the frustum refuse in the mesher, `CornersCoincide`; the union's
> answer exists (its floor less the frustum's disc touches its own edge at one
> point), so that is a gap, and whether the wedge less the frustum is a
> manifold at all is not demonstrated. The wedge's first back corner,
> `(−5, 0, 8)`, refuses *order along a crossing* under every certified rule: one
> face's ellipse starts inside the box of the node where the wedge's edge
> touches it. The order exists, so that is a gap too.

> **2026-10-05 note (Q930): one line drawn as two crossings is one crossing.**
> Traced with a print where `sorted_anticlockwise` refuses. The bar first, it
> refused on the bar's wall at `(2, 0, 0)`: the plane `x = 2` rests on the wall
> along `(2, 0, z)`, and the plane `y = 0` cuts the wall along the same ruling,
> the one of its two picked by `x ≥ 0`, so two ways out leave up the ruling,
> one along each, and bend alike. The block first, it refused on the block's
> wall `y = 0` at `(2, 0, 8)`, where the wall's edge with `x = 2` and the
> bar's line `x = 2` are the one line. The pin's guess, the wall `x = 2`, was
> not where it refused. `crossing_like` folds two partners only when they are
> one surface or, all three flat, cut one line; a plane and a pipe, or two
> planes on a pipe, it never folds. `fuse/arrange.rs`'s
> `fold_the_straight_cuts_along_one_line` now folds a straight cut into a
> straight crossing drawn before it when the two share a node whose ways out
> are proved parallel and not nothing, never settled by the policy. It runs
> **after** `merge_points_that_are_one_point`, not before: a quarter turn round
> the bar from its seam, the ruling ends on the cap circle at two meets, one per
> crossing, which are one node only once merged, and run before the merge the
> fold missed them and those placements still refused. The block on the cap
> answers every rule both ways round, with its corner on the rim at the seam, a
> quarter, a half and three quarters round, and mirrored across `y = 0`, and
> slid a tenth clear or a tenth along the touch: the union is the two solids,
> `32π + 8`, the intersection empty, and each less the other itself
> (`a_block_sitting_on_a_bars_cap_meets_it_at_one_point_under_every_rule`).
> **This is §38.2's family of fixes, for straight lines**, and it closes
> §38.2's own shape: the `3 × 4` block corner on a bar of radius five answers
> all three rules under a certified and a tolerant policy, the bar, the block
> and the bar with the block's room left empty in it
> (`a_block_corner_exactly_on_a_bars_wall_is_answered`). Turned by 0.4 or 1.0
> radians about the bar's axis instead of a quarter, the block on the cap is
> refused `Undecidable` on *side of plane* or *side of cylinder*, at a corner
> the turn rounds a hair off the touch. The fold asks nothing under the
> policy, so that refusal is not its.

> **2026-10-06 note (Q1136): a pointed cone drilled along its axis reaches
> `CannotTellTheCellsOfAFaceApart`.** A cone of slope one (base at `z = 10`)
> drilled by a bar of radius one from `z = -15` to `25` reached
> `the_walk_of_each_other_lump_each_walk_faces` on the bar's wall, under
> every rule both ways round (Q1091, `db0edb6e`). `which_side_of_everything`
> read a walk's side of a closed curve as the partner's sign at one of its
> nodes, and past the second of the two circles where the bar's wall meets
> the cone's two nappes (`z = 1` and `z = -1`) that sign turns back, so a
> walk on one circle matched both walks of the other and the bands
> `-15..-1` and `1..10` read the same sign and would have been glued.
> `whether_past_the_picker` in `fuse/arrange.rs` now reads a node past the
> flat half-space that picks a curve out, on a pipe whose axis that
> half-space faces along. The intersection both ways round and the cone less
> the bar now answer; the union both ways round and the bar less the cone
> still refuse `ResultIsNotManifold { count: 4 }`, queue row Q1135's
> (`a_pointed_cone_drilled_along_its_axis.rs`).

> **2026-10-07 note (Q1191): the union and the bar less the cone move twice
> more, and now answer.** `MeetPoint::WhereAClosedCurveStarts` named a closed
> crossing's start corner by its two surfaces alone, so the cone's two
> circles on the bar's wall got one corner and one edge, claimed four times;
> Q1148 (`6ce74099`) names that corner by which of the two circles picked it
> out as well, and the union and the bar less the cone moved from
> `ResultIsNotManifold { count: 4 }` to
> `TheFuseWroteABrokenAnswer { broken: AClosedEdgeIsWalkedAgainstItsSurface }`,
> which `validate.rs` gave when it read the bar's face's side of the cone off
> its far corner rather than where the face meets the edge. Q1190 (`9528b7d8`)
> reads it there instead, and all six fuses now answer, held to `364π` for the
> union and `92π/3` in two solids for the bar less the cone. §340.4.

> **2026-10-07 note (Q1114): a prism lying on its edge on a block's top is
> refused for want of a point inside a piece of a face, wherever moving it
> rounds its walls' offsets so that the edge leaves the top.** At `x = −1.5`
> (`a_prism_lying_on_one_edge.rs`) the edge sits `3.1e-17` inside the block,
> so the two overlap in a sliver, and every answer needs a side shorter than
> the gap between two doubles — a spread above `4e16` against the `1e11` a
> model supports (`docs/decisions.md` §3.5): **a limit**, pinned by
> `the_prism_with_its_edge_a_rounding_into_the_blocks_top_is_refused`. At
> `−2.5`, `−1.25` and `1.5` the edge stands a rounding above the top instead,
> so the two share no point and every answer is the two as they stand. Every
> rule both ways round refused the same way there, **a gap**, until Q1279:
> `by_a_plane` now places a piece with corners on the plane and one strictly
> in front, and all 18 fuses answer, held against volumes in
> `a_prism_lying_on_one_edge.rs`'s `LAID` (§365.8).

> **2026-10-08 note (Q329): what `Nearly` moved here.** Q100's `Nearly`
> (§322) came in after these tables, and keeps a product that falls below
> `f64`'s range. Of the rows above, only `Undecidable` out of
> `meet_of_three_surfaces` moved: its `2^k` family now refuses none of 2051
> triples, against 10.83%. **`Nearly` did not move it; §349's slide (Q327)
> did.** With `Nearly`'s keep turned off, that row comes back the same to the
> last digit, and with only the slide off it refuses 180. With the keep off, no
> test that pins a row here fails. The rows a kept product can reach were
> re-run and stand as written. §322.8 has the figures, the command behind
> each, and a line for each row the keep does not reach.

> **2026-10-10 note (Q1554): a ball's two loops round a pipe are ordered,
> and the quartic row moves to the mesh.** Q1554 orders the two loops a ball
> crosses a pipe in when it swallows the pipe's whole cross-section, round the
> pipe's axis (`the_way_a_loop_round_a_pipe_through_a_ball_winds` in
> `along_a_crossing.rs`). What that moves:
>
> - The `ACrossingThisCannotBeOrderedAlong` row above, the ball of 4 one
>   rounding off a bar's axis, is now refused `CannotMeshAFace` under every
>   rule both ways, under a certified and a tolerant policy alike:
>   `CornersCoincide` with the ball first and for the bar less the ball,
>   `ASideCutsOffOneCornerOfACurvedPatch` for the union and the intersection
>   with the bar first. Its test is now
>   `a_ball_one_rounding_off_a_bars_axis_is_refused_meshing_every_fuse`. Why
>   the mesh refuses is not traced; it is still a **gap**.
> - A ball of 2 with a bar through it off its axis, four placements (§347.9's
>   row): with the ball first, all 12 fuses answer
>   (`a_ball_with_a_bar_through_it_off_its_axis_crosses_it_in_two_loops_and_every_fuse_with_the_ball_first_answers`);
>   with the bar first, all 12 are refused `CannotMeshAFace { why:
>   CornersCoincide }`
>   (`a_ball_with_a_bar_through_it_off_its_axis_is_refused_meshing_every_fuse_with_the_bar_first`),
>   traced to the marcher's first step on the bar's wall in `march.rs`.
> - The ball of 2 against a square rod holed off its axis now answers all six
>   fuses (`a_ball_against_a_square_rod_holed_off_its_axis_is_answered_under_every_rule`).
> - A ball of `0.5` on a bar of 2's wall crosses it in one loop that runs back
>   and forth round the axis, which Q1554 does not order, and it is still
>   refused `ACrossingThisCannotBeOrderedAlong` before any cell
>   (`a_small_ball_on_a_bars_wall_is_refused_before_any_cell`), §34.10's gap.
>   *2026-10-11 (Q1766):* it now gets past the order and is refused
>   `CannotTellTheCellsOfAFaceApart` on the bar's pipe instead, in
>   `a_small_ball_on_a_bars_seam_is_refused_telling_the_pipes_cells_apart`;
>   Q1768 takes it. *2026-10-14 (Q1985):* since Q1777 told the pipe's cells
>   apart it is refused `AClosedEdgeOfTheAnswerWouldStartAtAPole` instead,
>   because the ball is centred on the wall and both its poles lie on the loop;
>   now `a_small_ball_centred_on_a_bars_wall_is_refused_where_its_poles_lie_on_the_wall`,
>   and §39.4's table has it.

> **2026-10-10 note (Q1598): both mesh refusals of the Q1554 note answer
> now.** Q1598 changes only the march: a trial that settles back onto its start
> is tried again shorter rather than kept (§347.9's Q1598 note). With that
> change alone, both shapes the Q1554 note left refused `CannotMeshAFace`
> answer, so that gap is closed:
>
> - The ball of 4 one rounding off a bar's axis answers all six fuses both ways
>   round, under a certified and a tolerant policy alike. Each answer
>   validates, has the faces of the ball centred on the axis, and measures
>   within its mesh's volume error of the closed form, and the union and both
>   differences add up against the ball and the bar measured alone. Its test is
>   now `a_ball_one_rounding_off_a_bars_axis_answers_every_fuse_as_one_on_it_does`
>   (Q1599). Which trial had been kept there was not traced.
> - The ball of 2 with a bar through it off its axis answers all 12 fuses with
>   the bar first as well:
>   `a_ball_with_a_bar_through_it_off_its_axis_crosses_it_in_two_loops_and_every_fuse_with_the_bar_first_answers`.

**Not refusals about arithmetic.** `Invalid::FacesDoNotAgreeWhichWayTheyFace` and
`Invalid::VolumeIsNotPositive` out of the fuse are this crate saying it cannot
assemble what it cut, before writing anything — `two_balls_that_really_cross_are_refused_by_name`
is the second. `Invalid::AClosedEdgeIsWalkedAgainstItsSurface`,
`Invalid::AFaceStraddlesSomethingTrimmingIt`, `AFaceMeetsItsNeighbourNowhere` and
`AnEdgeWithoutTwoFacesAlongItCannotBeWalked` are models whose topology and
geometry disagree, which §16.11 says arrive from a file.
`ResultIsNotManifold { count: 1 }` is **withdrawn** (§38.6).

> **§48 corrects this line: it is live, and it is a gap.** 83 268 fuses reach it
> in two of six families — turned pairs and a trio fused two-against-one — and
> `tests/an_edge_with_one_face_along_it.rs` pins it, including an exact tally
> that fails the day it closes. Its sharpest form needs no curve and no
> rounding: three axis-aligned boxes on the integer grid, where
> `fuse([far, near], [bar], Intersection)` refuses while `near ∩ bar` on its own
> answers with volume 2 and validates. **The argument that contributes nothing
> turns an answer into a refusal.** A tolerant policy refuses identically, so it
> is a gap and not a budget. The four families the earlier record was written
> from — two-solid lattice, flush, skew and elsewhere — reach it in none of
> 3 000 fuses each, which is why it looked closed.

> **§242.8 corrects §48's note: the trio is closed.** A piece flush on one tool
> facing the same way and on another facing the other is now inside them, and
> the three boxes answer `near`, volume 2, bit for bit. Only the turned pairs
> still reach `count: 1`, 104 of 624 in
> `how_often_one_pair_of_turned_boxes_leaves_such_an_edge`, and it is still a
> gap.

> **2026-09-29 note (Q479).** "The four families the earlier record was written
> from ... reach it in none of 3 000 fuses each" is about that swept sample, not
> the fuzz corpus. The corpus held three seeds whose two-solid lattice fuse
> answered `ResultIsNotManifold` one way round and a plain cube the other — a
> different cause from this row's turned-pair assembly bug: a stored corner a
> hair off the solid's own planes (§351). §351 fixed it; the replay's
> `a_solid_one_way_round_only` now reads 0. This row's family count and cause
> are unchanged.

> **2026-09-29 note (Q553): a separate family reached this refusal too, and
> is fixed rather than reclassified.** A ball of radius 5 between a block
> under its equator (`x ≤ −3`) and one over it (`x ≥ 3`) refused
> `ResultIsNotManifold { count: 1 }` in all twelve fuses the row's own test
> file builds, on the base before Q365 (commit `65a5db7f`'s own message,
> not re-measured here). Q365's fix — reading an arc's sign off the plane as
> each face spells it, §330.6's third bullet — gets every one of those
> twelve past `assemble`: two now answer in full and six stop one step
> later, in the mesher, on the `ACurvedFaceGoesRightRoundWithNoSeam` row
> added above. This family's `count: 1` is closed; it is a different bug
> from the turned-pair gap the rest of this note is about, and says nothing
> about that count.

**Seven rows are named, classified and undemonstrated**, marked above. §30.5 had
one such row and called it the one worth reading twice; seven is a different
quantity of the same thing, and §39.7 hands it on. Six are reachable in principle
from `Deserialize` and each needs a hand-built broken model, which is the reason
none has one. **The seventh is not**: the fused bar's mesh floor is four public
calls away and simply has no test, which is worse.

### 39.5 The numbers

| | at §30 (M3's close) | at §38 | now |
|---|---|---|---|
| tests (`cargo nextest`, whole workspace) | 985 | 1106 | **1112** |
| suite duration | 18.0 s | 2.9 s | **2.9 s** |
| doc tests | 19 | 19 | **19** |
| `FORMAT_VERSION` | 2 | 3 | **3** |
| fuzz corpus | 562 | 815 | **815** |
| `rcad-predicates` files changed | — | 0 | **0** |
| `rcad-mesh` files changed | — | 0 | **0** |

**The suite went from 18.0 s to 2.9 s and that is not a speedup.** §34.8 set
`[profile.test] opt-level = 2` because one draw of the band family marched a
nearly-tangent quartic loop in 18 365 points and was 90 of the suite's 99
seconds unoptimised. `debug-assertions` and `overflow-checks` are separate
settings and stay on, so every `debug_assert!` still fires and no test verifies
less than it did. Nothing in the kernel got faster.

**Files changed over the milestone**, whole tree: `rcad-geom` 20 files,
`rcad-topo` 24, `rcad` 2, `rcad-mesh` 0, `rcad-predicates` 0.

**Family by family, never blended (§17.7). Naming a crossing**, 2049 pairs each:

| Family | `Undecidable` | no half-space | runs through itself |
|---|---|---|---|
| control, whole numbers | 0.00% | 0.00% | 0.00% |
| quartic control, whole numbers | 0.00% | 0.00% | 4.25% |
| the band, a few bits off tangent | 0.00% | 3.76% | 1.07% |
| written at `2^k`, moved by `2^m` | 10.49% | 3.95% | 0.00% |
| leaning | 0.00% | 0.00% | 0.00% |
| **headline, turned** | 0.00% | **0.10%** | 0.00% |
| **quartic headline, turned** | 0.00% | **2.24%** | 0.00% |

The last two rows are criterion 3's, and they read 0.44% and 10.9% at §30.7.

**Where three surfaces meet**, 2050 triples each:

| Family | corners | certified `Zero` on all three | refused | widest box |
|---|---|---|---|---|
| control, lattice | 571 | 315 | **0.00%** | 2.2e-16 of the size |
| leaning | 1281 | 0 | **0.00%** | 1.8e-14 |
| written at `2^k` | 646 | 240 | **10.83%** | 3.0e-16 |
| turned | 682 | 0 | **0.00%** | 2.4e-15 |

**A curved fuse**, 384 fuses a family, all three rules and both ways round:

| Family | refused | recorded anything |
|---|---|---|
| lattice, a bar through a block | **0.00%** | 0 |
| turned about an awkward axis | **0.00%** | 0 |
| a ball resting on a block | **0.00%** | 0 |
| written at `2^k` | **0.00%** | 0 |
| **a block turned *against* a bar**, new at this gate | 7.8% | — |

That last row is §39.3's family and it is the only one here that turns one solid
relative to the other. 354 of its 384 fuses answer, every answer passes
`Model::validate`, and the thirty that do not are 24 `Undecidable` on *a point
inside a piece of a face* and six that are not solids.

**A planar fuse**, 423 fuses a family, unchanged from §38.7 and re-measured here:
lattice 0.0% refused and 4.7% not a solid; sharing coordinates 0.0% and 17.0%;
elsewhere 0.0% and 4.0%; **turned 22.7% and 4.0%**; skew 12.1% and 1.9%; leaning
alike 5.7% and 4.0%; a bar through a block 0.0% and 0.0%.

**`just fuzz 900`, run because this gate touches `rcad-geom`**: **24 486 343
executions in 901 seconds** at about 27 200 a second, coverage 2508 → **2515**
edges, and **no defect at all**. The seven new edges are the two branches this
gate added and the arithmetic behind them. Twenty-two working inputs were kept
and none is checked in: they are unminimised, and §30.7's rule is that an
unminimised handful is not worth the corpus. The shipped corpus stays at 815.

`just default` is green, `cargo-deny` included.

### 39.6 The turned family's threshold, and where its headroom went

§38.10 handed this on as *"the number most likely to go red next for a reason
that is nobody's regression"*: `TURNED_REFUSALS_MUST_STAY_BELOW` is 24%, the
measurement is 22.70%, and 1.3 points is six fuses. §38.8 put the shrinkage down
to the corpus growing — *"it read 20.71% of 420 when it was written and reads
22.70% of 423 now, and the corpus grew because the generator did"*.

**Measured, that is not what happened.** At `67a6b7f`, M3's close, the same test
on the same seed gives **423 fuses and 87 refusals, 20.6%**. The corpus was
already 423 and the 420 in the prose was stale from before M3. So the two points
were spent *inside* M4, and they were spent at one commit: `6a00ade`, the
expansion slide, leaves it at 87, and `c1e713e` takes it to 96.

`c1e713e` is the commit that made `Axis::new` reduce before it normalises, and
**every configuration in this family is turned about an `Axis`**. It changed the
rotation each pair is built with. So this is a **different corpus** and not the
same corpus refusing more — which is why nothing got worse anywhere else: the
family's own *order along a line* refusals went to zero over the milestone and
its *not a solid* count fell from 24 to 17.

**Decision: the threshold is not re-derived, and the risk is not handed on
vaguely either.** Three reasons, in the order they matter.

1. **Twenty-four per cent is doing its job and doing it sharply.** It fires on
   anything 1.3 points above the measurement. The smallest undeserved refusal on
   record was worth 3.5 points and §19.1's was worth 4.4, so the threshold has
   three times the sensitivity it needs. Widening it to buy drift room would
   trade a real property for a false comfort.
2. **A soak cannot re-derive it.** The threshold is judged on the shipped corpus
   only, deterministically, and a soak is a different corpus with a distribution
   of its own — six fresh seeds at ten times the corpus run 23.4% to 24.7%, four
   of them above this number with nothing wrong. Setting a shipped-corpus
   threshold from soak numbers is exactly the blending §17.7 exists to forbid.
   The soak the gate asked for would produce a number that must not be used.
3. **The thing that was actually wrong was the accounting, and it is fixed.**
   §38.8 could not say where the two points went because nobody had measured the
   ancestor. Now the doc comment names the commit. **When this test goes red, the
   answer is to attribute the move the way this section did and re-derive from
   the new corpus — never to raise the number.**

**The one threshold that was re-derived is criterion 16's**, and it went the
other way. `how_often_a_certified_fuse_of_a_turned_curved_pair_refuses` judged a
measurement of 0.00% against a ceiling of three fifths. That ceiling was written
at §36.14, when the ball family beside it refused every fuse; §36.21 took that
family to none and left the ceiling where it was. Sixty points of headroom over a
deterministic zero catches nothing. It is asserted at zero now, and the doc says
what that costs: the headline of this file now says exactly what its control
says, because **no draw of this generator reaches a shape that refuses**. The
shapes that refuse are all hand-written in `degenerate_curved_fuses.rs`. Widening
the generator until it reaches one is what would make a *rate* over this family
mean anything, and it is the day the number gets re-derived again.

### 39.7 What M4 hands to M5

§30.8's shape: numbered, each with its evidence and its named consumer, and §13
rule 3 applies — a part with no named consumer is not built. Roadmap §12 says M5
is **STEP import**, which changes what "named consumer" means for half of this
list: an importer's whole job is to turn a stranger's bytes into a `Model`.

1. **A ball cut anywhere but through a pole is refused, and a rigid motion of
   both operands changes the answer.** §39.3, with the angle at which it flips.
   This is first because it is a **§19 violation** and §19 is the property this
   kernel sells: the same two solids in the same relative position must give the
   same answer wherever the pair is put. §38.3 already wants the machinery — a
   cut closed face written out with a seam and poles of its own — and this says
   how much rests on it. **Consumer: the first imported ball anybody cuts.**

   **It wears three faces and not two, and this gate found the third only after
   §39.3 was written.** The same ball of radius four, the same slab with its
   face one unit from the centre, along the three axes: `z` answers, `y` refuses
   `ACurvedFaceGoesRightRoundWithNoSeam`, and **`x` refuses `CannotMeshAFace`** —
   there the fuse gets past naming the faces and the *triangulator* is what
   gives up, on a boundary that crosses itself. A fix that only stops the `y`
   refusal leaves the `x` one standing, so anyone scoping this from §39.3 alone
   would size it at two thirds of what it is.
   `the_same_ball_cut_the_same_way_along_three_axes_gives_three_different_answers`
   pins all three §19's way round, so closing this fails it loudly rather than
   quietly agreeing with it. What it should say once that lands is `Answered` on
   all three rows, one solid each, and the same spherical cap's volume in each.
2. **The fuse bakes a solid's placement into its stored surface, and the mesh
   floor goes up 77-fold for it.** §39.3, measured. A boolean that removes no
   material must not make the answer coarser than the input. Two families of fix
   and neither is obviously right: keep the placement on the solid through a
   fuse, or widen `rcad-mesh`'s bound so it charges the roundings the surface
   actually has rather than the largest number in it — `triangulate.rs:860` calls
   its own bound a §16.10 gap and says so. **Consumer: every fused curved solid,
   today.**
3. **Three panics on a caller's bytes, demonstrated.** §39.3's list: `validate`
   on a closed edge between two flat faces, `mesh` and `measure` on a closed edge
   with two different ends, `mesh` on a wire fanned onto the wrong surface.
   **Consumer: M5, immediately and unavoidably.** §16.11 says a caller's bytes
   may not panic, and a STEP file is a caller's bytes.
4. **The rule that generalises them.** A `# Panics` clause on a `pub` item needs
   a demonstration that the arenas cannot hold the shape, not an argument that
   geometry forbids it. `Model` is `Deserialize`. Four milestones running, the
   claim audit's main catch has been this species; M5 should sweep every
   `# Panics` in the tree against it once, rather than meeting them one at a
   time.
5. **`Exact::dot_of` does not slide and `Exact::square_minus` does.** §39.3's
   fourth item, with the input. Closing it is a change to `Exact` and therefore
   to every predicate that reaches level three, so it is a lane with its own
   re-measurement of every rate in §39.5. **Consumer: any caller wanting a
   direction with a subnormal component decided** — which a STEP file with an
   awkward unit scale can produce.
6. **Two authorities on "is this plane along this pipe's axis".** `intersect`
   decides it exactly; `along_a_crossing.rs` and `walk.rs` decide it with
   `== 0.0`. Demonstrated on `(1, 0, 3)` against `(−12, −12, 4)`. It moves
   `walk`'s dispatch and therefore the mesher, so it needs its own lane and its
   own re-measurement. **Consumer: any imported solid whose axis and cut plane
   are not both lattice-friendly**, which is most of them.
7. **`meet_of_three_planes` hands back `Ok` holding an unbounded box**, and
   refuses three mutually perpendicular planes written at `1e-200`. It is
   `rcad-predicates`', the crate has been frozen for ten lanes, and unfreezing it
   reopens the gate (§31.5) — so **this is the first item on M5's opening gate's
   desk**, not a lane's. The `-inf` case is a **limit** and correctly refused
   upstream; the scaled case is a **gap** whose fix is dividing each plane's four
   coefficients by a power of two before the determinants, which is exact and
   leaves Cramer's ratio unchanged.
8. **Seven refusals are named, classified and undemonstrated** (§39.4). Six need
   a hand-built broken model, which is what M5 will be building anyway; the
   seventh — item 2's mesh floor — needs four public calls and has no test.
9. **A ball with a pipe through it is still refused**, and it is criterion 13's
   second shape. The quartic ordering gap, §36.20 and §34.10, and it is not a
   budget question: tolerant refuses it too. **Consumer: any imported solid with
   a hole through a sphere**, and there is no shortage.
10. **Two coincident curved faces are refused**, which is criterion 15's second
   name and roadmap §7.4's first entry. **Consumer: STEP files are full of
   coincident faces** — this is the item most likely to be the first thing an
   importer hits, because duplicated and coincident geometry is what §9.2's
   healing list is about.
11. **A corner exactly on a curved face is refused**, and §38.2 wants a predicate
   that says whether one branch of `S ∩ Q₁` is the same curve as one branch of
   `S ∩ Q₂`. **Consumer: an ordinary part reaches it by accident**, which is
   §37.8's own ranking.
12. **A ball cut by a plane through its polar axis is refused**, and §38.3 wants
    the other half of what §38.1 built: a cut closed face written out with a seam
    and poles of its own. **Consumer: the same importer**, and the road is half
    built.
13. **A turned notched bar, and a bar cut by a single plane off its axis.** Both
    §36.28's, both refused under `Policy::tolerant` too, and §38.9 says the
    second is entangled with item 8. **Consumer: the first turned part anybody
    imports.**
14. **Solving three curved surfaces, and a crossing that runs through itself.**
    Unchanged from §31.9 and still deferred with their consumers named. Both are
    gaps whose family of fixes is a milestone of algebra.
15. **Cone and torus have a consumer for the first time.** §24.6's argument —
    a new `Surface` variant touches `validate`, the mesher, `is_wholly_flat`, the
    fuse and serialization — was true through M4 and stays true, but roadmap
    §12's own line for M5 is *"a STEP file that holds one"*. This is the
    milestone where that stops being hypothetical.
16. **The BVH, the bump arena and `rkyv` stay deferred**, consumers unchanged and
    no figure asking for any of them. §30.8 item 2's interval divide at 4% is
    still not where the money is, and M4 had no performance criterion.
17. **`Undecidable` out of `mesh` is still unreachable**, handed on for the third
    time. §39.2 settles that it needs a `Model` whose vertices are boxes, which
    is a change to what a model *is*. **It should stop being handed on as a
    milestone item and be recorded as a design consequence**, which this sentence
    does.

**What a STEP file walks straight into, said plainly.** Items 3, 4 and 8 are the
import boundary itself and are not optional. Items 1, 2, 10, 11 and 13 are not
about files at all — they are things the kernel does to solids it built itself,
and an importer inherits every one of them. Items 10, 11 and 13 are geometry
real files contain in quantity: coincident faces, a corner landing exactly on a
cylinder, a part that is not axis-aligned. Item 15 is a surface kind the format
has and the kernel does not. **A STEP importer built on this kernel today would
load a wide class of files and refuse a wide class of ordinary parts**, and the
refusals would be honest and named, which is the thing this kernel was built to
do — but they would be frequent, and item 1 says some of them would depend on
where the part happens to sit.

### 39.8 Corrections to earlier sections

- **§34.10's fourth row** classifies `meet_of_three_planes` handing back an
  infinite box as a **gap**. It is a **limit**: the meet is at about `1e315` and
  no `f64` triple names it. §39.4.
- **§38.8** attributes the turned family's 2.1 points to the corpus growing from
  420 to 423. The corpus was already 423 at M3's close and the rate was 20.6%;
  the points were spent at `c1e713e`. §39.6.
- **§36.23's paragraph on `the_surface_that_names_it_twice`** says a corner this
  fuse solved for is exactly on a plane it was solved from. It is not, and §39.3
  is the demonstration.
- **§36.16's row for `ACurvedFaceCannotBeCutSmallEnoughToMesh`** says reaching it
  means a piece is being cut without getting any smaller. `cut_in_two` hands back
  `None` for exactly that case and the piece never comes round again; reaching it
  needs sixteen cuts that each *do* narrow both halves.
- **§35.11's row for `NothingSaysWhichSideOfItsTrimmingAFaceLiesOn`** records the
  curved cause only. There is a flat one, and §39.4 has it.
- **§31.7 criterion 1** cannot be met by a `Plane` and §32.3 proves it. What
  meets it is `HalfSpace`, which did not exist when the criterion was written.
- **§31.7 criterion 16's threshold** was never derived from its measurement.
  §39.6.
- **§30.8 item 11 is wrong and §31.8 is right**, and it is settled rather than
  handed on again. §39.2.
- **§38.6's `ACurvedFaceGoesRightRoundWithNoSeam` row** says the family is a
  limit because what is left is half an endless tube. A ball leaves it too, in
  four public calls, and the answer is a spherical cap. §39.3, §39.4.
- **§38.9's correction about a plane through a ball's centre** is true and
  answers the wrong question: through the centre every piece holds exactly one
  pole, so the family that splits is the off-centre one.
- **§38.10 says the milestone's exit risk is closed.** It was closed for the
  shapes §37 named. Two panics on ordinary public input, a wrong answer, a §19
  violation and a mesh-floor regression were still in the tree, and this gate
  found all five. The lesson is §39.9's.
- **`rcad-mesh`'s `NoMeshCanGetThatCloseToTheSurface` defers a gap** on the
  grounds that *"`rcad-topo` never builds one"*. The curved fuse builds one.
  §39.3.
- **`mesh`'s `# Panics` said "Nothing does"** and argued from the evaluator's
  quarter-turn bound. `Model::add_round_bar(1e-200, 10.0)` walks past it. §39.3.
- **§31.8 item 9 and §36.14's watch on `MOST_POINTS_A_MESH_WILL_HOLD`** are
  answered, and not in the direction either expected: **the cap does not bound
  what it is named after.** Three call sites check it — the boundary walk and the
  two cutting steps — and `hand_over`'s loop over the triangulator's *interior*
  points, which is the whole bulk of a fine mesh, checks nothing. So one
  `add_ball(10)` and one `mesh` walk straight past it: measured here, 158 646
  points at a deflection of `1e-3` and 563 998 at `3e-4`, rising as `1/d`, so the
  four million is passed at about `4e-5` and the run takes minutes. No fused
  solid and no five curved edges are needed. It costs a long wait and a large
  allocation rather than a wrong answer, and it is M5's to bound.

### 39.9 Is this a kernel

Roadmap §12 says of M4: *"This is the point at which you have a kernel."*

**Not yet, and the gap is narrower than the refusal table makes it look.**

What is true: a caller can declare or compute a solid with curved faces, fuse it
with another under any of three rules in any order, get history maps back, have
the answer validated, meshed to a stated deflection and measured against its
closed form to a part in ten thousand, and write it out and read it back. Every
decision in that chain is a certified sign or a named refusal, the metamorphic
properties hold over curved pairs, and the whole of roadmap §7.4's catalogue is a
table with nineteen rows and no unanswered questions in it — nine answered, nine
refused by name, one that nothing in the public API can build. That is more than
"a boolean that works on the shapes it was tested on", which is what most of this
gets called.

What is not true is the sentence a user would need. **Nine of the nineteen
catalogue rows refuse**, and they are not exotic: two coincident faces, a corner
landing on a cylinder, a ball with a hole through it, a bar cut by one plane that
misses its axis. **A ball cut by a plane that misses a pole refuses too**, and
that one is not in the catalogue because nobody thought to put it there — it is
four public calls, it is the plainest curved boolean there is, and §39.3 found it
after this section was drafted. A kernel is a thing you can build a part on
without knowing which arrangements it dislikes, and this one still has
arrangements it dislikes that an ordinary part reaches by accident — §37.8 said
so of one of them and it is still true of five.

And the audit is the sharper answer. **This gate found `Model::fuse` returning
`Ok` with a solid that fails its own sweep**, on a block turned against a bar,
which is not a degenerate arrangement but the ordinary case of two parts that are
not aligned with each other. It survived ten lanes, a green suite of 1106 tests
and a code review, because no test in the milestone had ever turned one solid
relative to a curved one. It also found **a §19 violation** — a rigid motion of
both operands turning an answer into a refusal — and **a boolean that removes no
material making its own answer 77 times coarser to mesh**. A kernel is not a
thing whose three worst defects are found by the last person to look, and the
right conclusion is not that this gate was clever. It is that ten lanes of
green tests measured the things the lanes thought of.

So: **the architecture is a kernel's and the coverage is not.** Everything
roadmap §12 lists for M4 is built and measured. What M4 shows is that the list
was the wrong measure — the fuse working over curved faces was three lanes, and
the four milestones' worth of degeneracy that a real part contains is what M5 and
M6 are actually for. Roadmap §12's sentence should read *"this is the point at
which the kernel's shape stops changing"*, which is true, and which is worth
more than what it says.

---
