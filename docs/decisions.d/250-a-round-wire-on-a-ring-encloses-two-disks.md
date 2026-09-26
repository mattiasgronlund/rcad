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

### 250.5 What this leaves

- `select.rs` and `holds.rs` each have their own copy of the meridian-circle
  lookup (250.2). One of them should call the other.
- A quarter bend turned half a turn is refused by the builder's mesh check
  (250.4). Not traced.
- A puck's lid is now refused where it was wrong (250.1). Answering it needs
  the region a circle round the axis bounds, which is not the tube's inside.
  That is a gap, and it has no consumer yet: nothing dispatched builds a puck
  (§243.4).

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

A half-ring's flat ring-shaped face through the ring's middle may have given
the same wrong answer on the base: the outer wire counts +1 and the hole −1,
so a point on the face comes out `NotHeld`. That is reasoned, not run.
