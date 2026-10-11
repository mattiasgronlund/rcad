## 250. A round wire on a ring encloses one of two disks — 2026-09-26

*Row Q55 (`lane-q55-round-wire-two-disks`, off `f36e93bc`). §232.2 fixed
`select.rs`'s `the_piece_holds` for a flat piece bounded by a torus's meridian
circle and named `holds.rs`'s `what_a_round_wire_encloses` as having the same
defect. §243.5 found that neither bend pin asks it about a point in the other
disk. Two public calls do ask that, and it answered wrongly there. `Model::fuse`
reaches the function too, but only at a cap's own centre, where the answer is
unchanged (250.6).*

  250.1   The wrong answer, and where it is reached
  250.2   The rule now
  250.3   What each pinned answer rests on
  250.4   No fuse asks it about the other disk today
  250.5   What this leaves
  250.6   What the verifier found

### 250.1 The wrong answer, and where it is reached

A quarter bend of ring four and tube one, built face by face through
`SolidBuilder` (`tests/rings/mod.rs`) and passing `Model::validate`. Its cap on
`y = 0` is the disk round `(4, 0, 0)`. The same plane cuts the ring in a second
circle, round `(−4, 0, 0)`, where there is no bend at all.

`what_a_round_wire_encloses` answered with the ring's sign alone, and the ring's
inside on `y = 0` is both disks. So on `f36e93bc`:

- `SolidView::face_holds_the_point` on the cap said `Held` at `(−4, 0, 0)` and
  `(−4, 0, 0.5)`, and `OnItsBoundary` at `(−3, 0, 0)`, `(−5, 0, 0)` and
  `(−4, 0, 1)`, the other circle. All five are off the cap.
- `SolidView::classify_point` said `OnBoundary` at `(−4, 0, 0)`,
  `(−4, 0, 0.5)`, `(−3, 0, 0)` and `(−5, 0, 0)`. All four are outside the bend.
  A ray from a point on the cap's plane asks the cap whether it holds the
  point, and a yes puts the point on the skin.

Both are **wrong answers, not refusals**, from public calls on a valid solid.

A plane **square** to the ring's axis was also wrong. A puck's lid on `z = 1`
rests on the ring of radius four and tube one along its one circle. The ring's
inside on that plane is empty, so the sign put the lid's own centre `(0, 0, 1)`
outside the lid: `Ok(NotHeld)`. That puck is built straight into the arena, as
§243.4's is, and has not been put through the builder.

### 250.2 The rule now

When the face across the round wire is a torus, `what_a_round_wire_encloses`
asks `intersect` for the face's plane and the ring, the way §232.2's
`the_pickers_of_the_meridian_circle_through` does:

- A plane through the axis gives the two meridian circles, each named by one
  flat half-space that `intersect` has proved clears both circles. The one
  that certainly keeps the wire's corner names this wire's circle. Each disk is
  exactly the part of the tube on its own circle's side, so a point the ring
  holds (`Neg`) or has on its skin (`Zero`) is in this disk or on this circle
  when that half-space keeps it, and outside the wire when it rules it out.
  A point on the half-space's own boundary is kept, as a picker keeps what is on
  it. If the ring is certain the point is outside (`Pos`), the half-space is
  not asked.
- Any other plane gives no such half-space. A plane square to the axis cuts
  circles round it, and no circle's disk is the tube's inside there. That is
  refused as `AFaceTrimmedByACurveThisCannotFollow`, the name this function
  already gives a round wire whose partner's inside is not what the wire
  bounds. A plane that is neither refuses inside `intersect`, as a quartic.
- A half-space that cannot say is `Error::Undecidable`, with the half-space's
  own predicate name on the record, so a ray grazes and tries its next target.

It takes `policy` and `log` now, for `intersect`. The only caller,
`what_a_flat_face_holds`, already had both. That call site is the one line
this lane changed outside the function.

The helper, `the_half_space_naming_the_meridian_circle_through`, is the same
question as `select.rs`'s `the_pickers_of_the_meridian_circle_through`, asked of
a face of a solid rather than of a piece. It is a second copy because
`select.rs` was outside this lane's write set. One of them should call the
other.

### 250.3 What each pinned answer rests on

`tests/a_bends_cap_holds_one_of_the_two_disks.rs` asks twelve points on
`y = 0`. Each answer is read off the distance to the circle centres `(±4, 0, 0)`
against the tube radius of one. Under one is inside that disk, exactly one is on
that circle, over one is outside it.

| point | distance to `(4,0,0)` / `(−4,0,0)` | answer |
|---|---|---|
| `(4, 0, 0)` | 0 / 8 | held, its own centre |
| `(4, 0, 0.5)` | 0.5 / — | held |
| `(3, 0, 0)` | 1 / 7 | on its boundary, its own corner |
| `(5, 0, 0)` | 1 / 9 | on its boundary |
| `(4, 0, 1)` | 1 / — | on its boundary |
| `(4, 0, 2)` | 2 / — | not held |
| `(0, 0, 0)` | 4 / 4 | not held, in neither disk |
| `(−4, 0, 0)` | 8 / 0 | not held, the other centre |
| `(−4, 0, 0.5)` | — / 0.5 | not held |
| `(−3, 0, 0)`, `(−5, 0, 0)`, `(−4, 0, 1)` | — / 1 | not held, on the other circle only |

The other circle is not part of the cap's boundary, so a point on it is off the
cap, not on its edge. The four `classify_point` points are the other disk's
centre, a point inside it, and two points on its circle. None is on the bend,
which runs from `+x` to `+y` and is half a turn away, so each is `Outside`.

`holds.rs`'s `a_pucks_lid_is_refused_rather_than_holding_nothing_but_its_rim`
asks the lid about `(0, 0, 1)`, `(2, 0, 1)` and `(5, 0, 1)` and expects the
refusal for each.

With the new check switched off (the function returning the sign as before),
all three tests fail: the twelve-point test on the five points of 250.1, the
classification test with `OnBoundary` at all four, and the lid test with
`Exact([0.0, 0.0, 1.0]): Ok(NotHeld)`. The two integration tests failed the same
way on unmodified `f36e93bc` source, before any edit to `holds.rs`.

### 250.4 No fuse asks it about the other disk today

§243.5's two bend pins ask it only at a cap's own centre (250.6). Three more
fuses were tried, all six rule and order combinations each, and none asks the
cap about a point in the other disk:

- The bend and a unit block round `(−4, 0, 0)`, one face on `y = 0` inside the
  other disk: every combination refuses `ACrossingBetweenAPlaneAndATorusThatIsAQuartic`.
  The block's faces `x = −4.5` and `x = −3.5` are parallel to the axis and off it.
- The bend and a block filling the third quadrant, with its faces `y = 0` and
  `x = 0` through the axis and its far faces at `−6` and then `−40`: the same
  refusal at both sizes. So a plane face far from the ring still has its
  crossing with the ring asked.
- The bend and a copy of it turned half a turn, written out exactly so its caps
  are the other disks on `y = 0` and `x = 0`: `SolidBuilder::build` refuses the
  copy, `TwoPatchesOfTheMeshDrewTheSameSide { drawn: 1, used: 1 }`. Its wall
  runs from angle `π`, where the torus's `atan2` branch is cut. That is a
  hypothesis for why, **not traced**.

So a partner that can be fused with a bend today has only planes through its
axis or square to it, and no bounded polyhedron is made only of those. A
curved partner refuses as a torus against another curved surface. The one
remaining partner, another bend on the same ring, is the turned copy the
builder refuses. That is why no fuse is pinned. Once any of those three
refusals is lifted, the classification in 250.1 is what a fuse would get.

*Q191's measurement (`b187bab9`), once the builder makes the turned copy
(Q190).* Fusing the quarter bend with a copy of itself turned half a turn is
exactly that lifted refusal: the copy's caps are the bend's other disks. A
temporary print in `what_a_round_wire_encloses` counted eight calls reaching
it in each of the six rule-and-order combinations, two per cap, each at the
other bend's cap centre — once through `coincident_face`
(`face_holds_the_point`) and once through the ray `classify_uncertain_point`
fires from the point. With the half-space not asked, both unions refuse
`ResultIsNotManifold { count: 1 }`; the intersections and differences still
come out right. Pinned in
`tests/a_bends_cap_holds_one_of_the_two_disks.rs`'s
`a_bend_fused_with_its_copy_turned_half_a_turn_comes_to_what_each_rule_says`.

### 250.5 What this leaves

- `select.rs` and `holds.rs` each have their own copy of the meridian-circle
  lookup (250.2). One of them should call the other. **Done, Q117 (250.8).**
- A quarter bend turned half a turn is refused by the builder's mesh check
  (250.4). Not traced.
- A puck's lid is now refused where it was wrong (250.1). Answering it needs
  the region a circle round the axis bounds, which is not the tube's inside.
  That is a gap, and it has no consumer yet: nothing dispatched builds a puck
  (§243.4). **A plane square to the axis that cuts the tube in two circles is
  answered since 250.9**; the lid's plane rests on the tube and touches it in
  one circle, which `intersect` names by no picker, so the lid stays refused
  (Q104).

### 250.6 What the verifier found

The verifier checked out `holds.rs` at `f36e93bc`, kept the new tests, and
saw all three fail with the wrong answers 250.1 quotes. Its hand arithmetic
agreed with 250.3 at six points, on both disks. It committed nothing and said
merge, with 250.4's wording corrected, which is done above.

**Every route to `what_a_round_wire_encloses`** from the public API:
`SolidView::face_holds_the_point`; `classify_point` and
`classify_uncertain_point`, through `classify.rs`; `Model::fuse`, through
`select.rs`'s `a_face_of_the_input_holds` and through the ray in
`settle_against_each`; and `Model::check_no_two_solids_share_space`, through
`overlap.rs`, where a refusal other than `Undecidable` becomes "found none" and
so can only weaken the search. A probe that logged every call with a torus
partner, over `just t --workspace --no-fail-fast` (2422 passed), found five
tests reaching it:

- The lane's puck test holds the only three calls whose plane does not hold the
  axis, so the new refusal moves no other answer.
- Its two bend-cap tests: every point the half-space put in the other disk was
  wrong on the base.
- `a_bend_built_face_by_face_is_the_bend_a_quarter_bend_builds`: five calls,
  all outside the tube, where the half-space is not asked.
- `a_bend_fused_with_a_copy_of_itself_is_nothing_less_it_and_refused_otherwise`:
  25 calls, all at a cap's own centre, all `Pos`, answer unchanged. 24 of them
  come from `Model::fuse`, which is why 250.4 no longer says no fuse reaches it.

**The copied helper** differs from `select.rs`'s
`the_pickers_of_the_meridian_circle_through` in four ways: its argument types,
its return type, and that it accepts only one `InFrontOf` where `select.rs`
accepts any list of them. None changes an answer, because
`intersect(plane, torus)` is the only producer and it gives one `InFrontOf`
per run through the axis.

**What was not done.** The verifier stopped at its budget before running any
mutation. From its probe counts: flipping the side would fail the bend-cap
test, and dropping the refusal would fail the puck test. **No test reaches the
half-space's `Undecidable` or `Zero` arm**, so a mutation there would pass.
`Zero` counts as kept, which matches `Picker::InFrontOf`'s own "on or in
front of". `select.rs`'s `every_picker_keeps` answers `None` on `Zero`
instead, so the two files disagree about a point on the half-space's plane.
The verifier argues that no point on the face's plane can land there, since
`two_meridian_circles` proves a margin between the circles first. That is
argued, not run.

**Since Q118 (c3920eb3), tests do reach both arms**, through
`face_holds_the_point`, which does not ask whether its point is on the face.
Three tests in `a_bends_cap_holds_one_of_the_two_disks.rs` hand the cap a
point off it: a point in the tube on `x = 0`, the plane between the two
disks, reaches `Zero` and is kept, and two boxes reaching across that plane
reach `Undecidable` — one round the other cap's centre, and one on a fat bend
turned an eighth of a turn, where the plane is `x = −y` and the box holds a
point of the cap. The argument above, that no point on the
face's plane lands there, still stands; the tests reach the arms only from off
the plane.

A half-ring's flat ring-shaped face through the ring's middle may have given
the same wrong answer on the base: the outer wire counts +1 and the hole −1,
so a point on the face comes out `NotHeld`. That is reasoned, not run.

### 250.7 A half-ring's flat face, run

*Row Q120 (`lane-q120-half-ring-face`, off `df9d24d8`).* The face could not be
asked on either commit, because `SolidBuilder::build` refuses the half-ring.

The half-ring is ring four and tube one, cut by `z = 0`, keeping `z ≥ 0`. It
was written face by face, as `rings::a_quarter_bend` writes a bend, in a
scratch test `crates/rcad-topo/tests/q120_half_ring_probe.rs` that was not
committed. The flat face is `z = 0`, facing `−z`, with two wires. The outer
wire is the circle of radius five through the corner `(5, 0, 0)`, and the hole
is the circle of radius three through `(3, 0, 0)`. Each is one
`RightRoundWhereItsFacesCross` edge. The wall is one face on the torus, on the
other side of both circles. It was written three ways, and each way was tried
with all eight choices of the two circles' directions and the wall's
orientation in the shell:

| the wall's wires | `build` under `Policy::certified()` |
|---|---|
| two wires, one circle each, no seam | `TwoPatchesOfTheMeshDrewTheSameSide { drawn: 3, used: 3 }` |
| one wire: circle, seam, circle, seam back, with the seam the half circle over `(4, 0, 1)` picked by `x ≥ 0` and `z ≥ 0` | `ADraftFaceCouldNotBeMeshed { face: 1, why: CornersCoincide { position: 5 } }` |
| the same, with the seam over `(−4, 0, 1)` picked by `x ≤ 0` and `z ≥ 0` | the same |

Face 1 is the wall. All 24 builds refused the same way on `2b932098`
(`f36e93bc^`; `f36e93bc` changes only `docs/`) and on `df9d24d8`. The tip ran
`mise exec -- just t -p rcad-topo --test q120_half_ring_probe --no-capture`.
The base ran the same file in a `git archive 2b932098` export, with
`mise exec -- cargo nextest run --manifest-path <export>/Cargo.toml -p
rcad-topo --test q120_half_ring_probe --no-capture`.

So none of the six points, `(4, 0, 0)`, `(3.5, 0, 0)`, `(0, 0, 0)`,
`(6, 0, 0)`, `(5, 0, 0)` and `(3, 0, 0)`, was asked of the flat face, and
`classify_point` was not asked about `(4, 0, 0)` or `(0, 0, 0)`. Neither the
base's `NotHeld` nor a refusal on the tip was seen.

The wall goes right round the ring. 280.5 found that the mesher does not
unwrap the ring angle, and `CornersCoincide` says the wall's loop names one
chart point twice. That fits, but it was not traced. A half-ring is a sound
solid, so this refusal is a gap (§16.10). It is wider than 280.6's fix, which
refuses a torus face that goes right round the ring on purpose. `Deserialize`
does not call `validate`, so it may be a door to this face.

*Q193 (`e5a7d95a`)* tried both doors. Through `Deserialize` the half-ring
reads back in all eight orientation choices, and its flat face refuses all the
same: `face_holds_the_point` gives `AFaceTrimmedByACurveThisCannotFollow` at
the six points above, `classify_point` gives `CannotFollowAFacesTrimming` at
`(4, 0, 0)` and `(0, 0, 0)`, and both `validate` and `mesh` refuse the wall
`AWireWillNotLieFlatOnItsFace`. So 250.6's reasoned `NotHeld` does not show on
this commit, and Q415 owns the flat face. **Answered in 250.9** (Q664, Q415
filed again). *2026-10-04, Q826: on `c11bf21f`, `validate` passes two of the
eight, one of them walking the flat face's hole the same way round as its
outline (§243.8).*

Through `SolidBuilder`, with the wall split into two faces along its top
circle, only one wall shape reaches the mesher at all: a rim
`RightRoundWhereItCrosses` the tangent plane `z = 1`, which refuses
`AWireWillNotLieFlatOnItsFace` in the eight consistent orientation choices and
`AClosedEdgeIsWalkedAgainstItsSurface` in the other twenty-four — Q372's band.
A pipe of radius four, or a `RightRoundWhereItsFacesCross` rim, gives
`ACrossingBetweenATorusAndAnotherCurvedSurface`, and a rim from
`PartOfWhereItCrosses` with no seam gives `ADraftWireIsTooShort { steps: 1 }`.

**Built since §390, in 250.10** (Q194): the table above is history. All three
spellings build the half-ring, and its flat face answers all six points
right.

### 250.8 One lookup, one rule, in `holds.rs` (Q117, Q118, Q199)

*Row Q117 (`lane-q117-one-meridian-rule`, merged at `c82e02f2`).* 250.5's first
bullet is done. `holds.rs` keeps one copy of the meridian-circle lookup,
`the_half_space_naming_the_meridian_circle_through` (`23d3df1a`), now
`pub(crate)` and returning one `Picker` rather than a list. The rule that reads
it, `the_meridian_half_space_keeps`, is a second `pub(crate)` function beside
it. `fuse/select.rs` calls both; its own copy of the lookup is gone.

The rule's argument, from its own doc comment
(`mise exec -- just src the_meridian_half_space_keeps`):

- **A point on the half-space's own plane is kept**, because that is what a
  `Picker::InFrontOf` is: everything on or in front of a flat surface. It is
  the reading the circle's own edge is picked by, and `fuse/select.rs` asks
  through this function too, so the two files cannot disagree about it again.
- **No point on the face's plane gets that far.** `intersect` names meridian
  circles only for a plane that holds the ring's axis exactly, and only once
  it has proved the half-space's plane clears both circles. So the tube's
  inside on the face's plane is the two disks, each strictly on one side of
  the half-space's plane, and a point on the half-space's plane that the tube
  holds is off the face's plane. `SolidView::face_holds_the_point` does not
  ask whether its point is on the face, so it can be handed one; a refusal
  here would be a check for that made in this one place and nowhere else.

**The Zero-arm panic probe.** *Row Q118 (`lane-q118-half-space-arms`, merged at
`c3920eb3`)* mutation-tested this same arm at the time: with the Zero case
folded into the arm that returns `false`, a workspace run of 2523 tests had
exactly one failure, `a_point_in_the_tube_on_the_plane_between_the_two_disks_is_kept`
— Q118's own off-plane pin, the only test that reached `Zero` before Q199.

This lane re-ran that probe itself, at the scope this row named:
`crates/rcad-topo/src/holds.rs`'s `Zero` arm was replaced with a temporary
`panic!`, and `mise exec -- just t -p rcad-topo --no-fail-fast` gave **1114
tests run, 2 failed**:

- `a_bends_cap_holds_one_of_the_two_disks::a_point_in_the_tube_on_the_plane_between_the_two_disks_is_kept`
  — Q118's pin, unchanged.
- `fuse::select::tests::every_picker_keeps_a_point_on_the_half_spaces_own_plane`
  — new since Q199, and reaching the same arm only because `select.rs` now
  calls through to it (this section's first paragraph), not because a second
  rule exists to disagree with the first.

The `panic!` was reverted before this file was written; `git diff` on
`holds.rs` is empty. This lane's own count (1114, `-p rcad-topo`) is a
different scope from Q118's (2523, the whole workspace), so the two are not
the same measurement re-run — only the "how many tests reach `Zero`" answer
is comparable, and it went from one to two once Q199 landed.

**Q199's pin** (`lane-q199-picker-pin`, merged at `1a3fd648`) is that second
test: a unit test in `select.rs`, `every_picker_keeps_a_point_on_the_half_spaces_own_plane`
(`1c300b27`), pinning `every_picker_keeps` keeping a point on the meridian
half-space's own plane — the same `Zero` case this section's rule argues for.

### 250.9 Two circles round the axis, answered (Q664)

*Row Q664 (`lane-q664-a-half-rings-flat-underside-read-back`, off
`c564dec5`).* The half-ring of 250.7, read back through `Deserialize`.

**Which `cannot()` fired, traced.** A temporary `eprintln!` went before each of
`what_a_round_wire_encloses`'s six refusals and before `walk_one_outline`'s, and
the eight read-back half-rings were asked 250.7's six points and the two
`classify_point` points. All 46 calls that reached the function came out at
the one after the lookup: `intersect` of `z = 0` and the ring gave two runs,
picked by `OutsideAPipe` and `InsideAPipe` of the pipe of radius four, and the
lookup took only a lone `InFrontOf`. 250.2 matched the refusal by its doc; the
doc was right.

**The rule now.** The lookup, `the_picker_naming_the_circle_through`, takes a
lone picker of either kind that certainly keeps the wire's corner.
`the_half_space_naming_the_meridian_circle_through` is that with anything but a
half-space dropped, so `fuse/select.rs` gets what it got. A plane square to the
axis cuts circles at `R ± s` with `s > 0`, and on it the ring holds the band
between them. So, with the ring's sign and the pipe's:

| circle | ring `Neg` | ring `Zero` or `Pos`, kept by the pipe | not kept |
|---|---|---|---|
| outer, `OutsideAPipe` | inside | the ring's sign | inside: nearer the axis than `R` |
| inner, `InsideAPipe` | outside | the ring's sign turned round | outside: further than `R` |

A point on the pipe is kept. On the face's plane such a point is in the band,
where the pipe is not asked, so which way `Zero` goes cannot move an answer.

**What is pinned**, in `tests/a_half_rings_flat_face_read_back_from_bytes.rs`,
in all eight orientation choices, each read off `r` against 3 and 5:
`(4, 0, 0)` and `(3.5, 0, 0)` held, `(0, 0, 0)` and `(6, 0, 0)` not held,
`(5, 0, 0)` and `(3, 0, 0)` on its boundary; `classify_point` at `(0, 0, 0)`
`Outside`. The model is written through Q193's mirror of the arenas, and the
file checks the mirror both ways: a kernel-built bar, bend and cube decode and
write back to the same bytes, and each half-ring reads back and writes out to
the bytes the mirror wrote. With `holds.rs` put back to `c564dec5`, both
answer tests fail, `classify_point` at `(0, 0, 0)` with
`CannotFollowAFacesTrimming` naming the flat face.

**What this leaves.**

- `classify_point` at `(4, 0, 0)`, on the flat face and so on the skin, is
  still refused `CannotFollowAFacesTrimming`, now naming the **wall**. Asked
  directly at `(4, 0, 1)` and `(4, 0, −1)`, the wall refuses
  `AFaceMissesPartOfWhereItsSurfacesCross` in all eight choices. **Answered in
  §243.8** (Q826): the route was traced as this item guessed it, and the wall
  is now the half of the ring its wire keeps. `(4, 0, 0)` is `OnBoundary` in
  the four choices whose flat face walks its two circles opposite ways round.
  In the other four the wall's wire keeps a different side of `z = 0` at each
  circle, and the wall is still refused, naming it.
  **2026-10-11, Q1730:** since Q1726 merged Q1722's `count_crossings` fix,
  `(4, 0, 0)` is `OnBoundary` in every choice
  `classify_point_finds_the_hole_outside_and_the_flat_face_on_the_skin` asks —
  the flat face holds it regardless of which way its two circles are walked.
  The wall itself, asked directly, is unchanged: it still refuses on the four
  choices above whose wire keeps a different side of `z = 0` at each circle.
- The puck's lid (250.5) rests on the tube in one circle that no picker names,
  and stays refused. Q104 holds it.

### 250.10 The half-ring built face by face, answered (Q194)

*Row Q194 (`lane-q194-a-half-ring-built-face-by-face`, off `ad706857`).*
250.7's half-ring, written through `SolidBuilder` in its three wall spellings
and eight orientation choices each, as 250.7 wrote it. The flat face is added
first and the wall second, and the shell lists them in that order. The third
spelling puts both corners on `−x`, at `(−5, 0, 0)` and `(−3, 0, 0)`, since its
seam over `(−4, 0, 1)` has to end at them. A choice is written
`(outer, hole, wall)`: which way the flat face walks each circle, and which way
the shell uses the wall. The wall walks each circle the other way from the flat
face.

**What builds**, on `ad706857`, each answer from `SolidBuilder::build` under
`Policy::certified()`, and `Model::validate` passing every one that builds:

| choice | no seam | seam over `(4, 0, 1)` | seam over `(−4, 0, 1)` |
|---|---|---|---|
| `(F, R, F)`, the half-ring | builds | builds | builds |
| `(F, F, ·)`, `(R, R, ·)` | `AClosedEdgeIsWalkedAgainstItsSurface` | the same | the same |
| `(R, F, R)` | `FacesDoNotAgreeWhichWayTheyFace` | the same | the same |
| `(F, R, R)` | `VolumeIsNotPositive` | `FacesDoNotAgreeWhichWayTheyFace` | the same |
| `(R, F, F)` | `FacesDoNotAgreeWhichWayTheyFace` | **builds, wrongly** | **builds, wrongly** |

So 250.7's two refusals are gone, as §390 said they would be. Four choices walk
the flat face's two circles the same way round, and Q949's check refuses them
(243.8). `(R, F, F)` is the next paragraph.

**The answers**, asked of the half-ring in each spelling, are all right:
`face_holds_the_point` on the flat face gives `Held` at `(4, 0, 0)` and
`(3.5, 0, 0)`, `NotHeld` at `(0, 0, 0)` and `(6, 0, 0)`, and `OnItsBoundary` at
`(5, 0, 0)` and `(3, 0, 0)`, each read off `r` against 3 and 5.
`classify_point` gives `OnBoundary` at `(4, 0, 0)` and `Outside` at
`(0, 0, 0)`. So 250.6's reasoned `NotHeld` does not show through this door
either, and `what_a_round_wire_encloses` is left as it is. Each half-ring holds
Pappus's `π² r² R = 4π² = 39.478`, half of the whole ring's `2π² r² R`, at
deflections `0.1`, `0.03` and `0.01`, each within the error its own
measurement states. All of it is pinned in
`tests/a_half_ring_built_face_by_face.rs`, and every refusal in the table by
name.

**`(R, F, F)` with a seam is a wrong answer.** It is the half-ring with every
loop of both faces walked backwards and both faces still used `Forward`. With
no seam it is refused. With either seam it builds, passes `validate`, and is
two shapes at once. An uncommitted probe in the new test file printed, for
both seams:

- `measure` holds the top half: `39.413` at `0.01` against Pappus's `39.478`;
- the wall's `face_holds_the_point` gives `NotHeld` at `(4, 0, 1)`,
  `(−4, 0, 1)` and `(0, 4, 1)`, and `Held` at `(4, 0, −1)` and `(0, −4, −1)`:
  the bottom half;
- `classify_point` gives `Outside` at `(4, 0, 0.5)`, `(−4, 0, 0.5)` and
  `(4, 0, 1)`, and `Inside` at `(4, 0, −0.5)`.

The half-ring's own builds, asked the same, give the top half every time,
which is right. A quarter bend with every loop walked backwards in the same way, its caps' circles
and its wall's wire turned round and every face used `Forward`, was refused
`AClosedEdgeIsWalkedAgainstItsSurface` by the same probe. So it is the seam
across a band that lets it through. The test does not pin the two seamed
`(R, F, F)` builds, as neither a solid nor a refusal
(`is_every_loop_walked_backwards_round_a_seam`).

Why `validate` passes it is **not traced**. A guess, not run: the wire keeps
`z ≤ 0` at both circles, so Q949's check, which compares the sides kept at
circles along one plane only with each other, agrees with itself. `holds.rs`
reads that side. The mesher cuts the patch the seam closes, which is the top
half whichever way the loop runs, so the mesh balances and the volume is
positive. Nothing compares the side the circles keep with the side the seam
is on. The class is `wrong/C`: only a probe has built it.

**What this leaves.**

- `SolidBuilder::build` and `Model::validate` pass a seamed ring band whose
  every loop is walked backwards, and `measure` and `classify_point` then
  disagree about which half it is. Filed as a row on `validate.rs`, to trace
  it and refuse it, and to turn the two seamed `(R, F, F)` choices in the new
  test into pinned refusals.
