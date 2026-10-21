## 31. What opening M4's gate settled

*Recorded 2026-09-01, before any lane opens. §15 settled the shared meanings
before M1 and M2.5 fanned out, §23 before M2's lanes, §24 before M3's stages,
§25 before M2's last four and §27 at M3's inner gate. This is the same, for §13
rule 4's reason.*

**§30.3 is the instruction manual for this section and it is followed
literally.** A settling document is right about meanings and ownership and wrong
about arithmetic, the numbers are cheap to be wrong about because the first
person who needs one does the sums, and the dangerous sentences are the ones
that specify a *test* rather than claim a number. So: every number below is
marked confident or guessed in the row it sits in, every criterion in §31.7
names a test that does not exist yet rather than one that already passes, and
§31.10 is a list of the places this gate is planning on top of a guess.

### 31.1 What M4 is, now that the roadmap's version of it already ships

Roadmap §12 says M4 is *"General fuse, multi-argument, history maps. Run the
metamorphic suite."* Read that against the tree before believing it describes
work to be done:

- `Model::fuse` takes `&[SolidKey]` for both the arguments and the tools, and
  has since M2.5. Roadmap §7.2's "multi-argument from the start" is done.
- `Fused` carries `images_of_face`, `images_of_edge` and `images_of_vertex`,
  with generated, modified and deleted derived from them. Roadmap §7.1 step 6 is
  done.
- `Rule` holds Union, Intersection and Difference, read as one table and one
  `match` (§17.4). Roadmap §7.1 step 5 is done.
- `crates/rcad-topo/tests/fuses_that_must_agree.rs` is the metamorphic suite —
  commutativity, associativity, idempotence, absorption, `A \ A`, batched
  against chained, bit-identical under exact isometries. Roadmap §11.2 is done.
- `crates/rcad-topo/tests/degenerate_fuses.rs` is roadmap §7.4's catalogue,
  nine of thirteen entries, as a pass/refuse table.

All of it works, and all of it works **for planes only**. §1.7 already records
why: the planar boolean was built before intersection, on purpose, so that the
decomposition, the selection table, the history maps and the metamorphic
oracles would all be shipped and measured before a curved crossing arrived to
be classified against them. Roadmap §12 is the long-range plan and §1 says it is
not a description of this codebase.

**So M4 is one sentence: the same fuse, over curved faces.** Its one deliverable
is that the five lines at the top of `Model::fuse` come out —

```rust
for &solid in arguments.iter().chain(tools) {
    if !self.solid(solid).is_wholly_flat() {
        return Err(Error::FuseNeedsFlatFaces);
    }
}
```

— and everything below them keeps working. `classify` loses its
`Containment::NeedsFlatFaces` arm for the same reason and at the same time,
because §17.4 makes classification the thing selection is built on and a fuse
that cut curved faces and could not place them would have got half way.

**What M4 is not.** It is not a new surface kind (§24.6's argument is unchanged
and cone and torus still have no consumer), it is not the free-form marcher
(§24.1), and it is not a performance milestone — §31.9 keeps the BVH and the
bump arena deferred and says what figure would call each due.

### 31.2 Every question a curved fuse asks, and who decides it

§27.1's descendant, and the same discipline: every row is a **sign** or a
**construction**, every one is about the numbers a `Surface` stores, and none of
them is a distance compared against a tolerance.

Write `n` for a plane's stored normal and `d` for its offset, `c` and `R` for a
ball's centre and radius, `o`, `w` and `r` for a pipe's axis origin, axis
direction and radius, and `∇S(x)` for a surface's gradient at `x` — which is `n`
for a plane, `x − c` for a ball, and `|w|²(x − o) − (w·(x − o))w` for a pipe,
each of them a polynomial in the stored numbers with no normalization in it.

| | The question | Where the fuse asks it | The polynomial | Degree | Who decides it | Today |
|---|---|---|---|---|---|---|
| 1 | where two faces cross | `cuts::gather` | §27.1's eleven rows | 2–12 | `intersect` | **exists** |
| 2 | are two faces on one surface | `cuts::cuts_between`, the ON columns | as row 1 | 2–12 | `intersect`, `SurfaceMeet::SameSurface` | **exists, and it does not say which way round** — §31.3 |
| 3 | do two faces touch at one point | `cuts::gather` | as row 1 | 2–12 | `intersect` | **exists** |
| 4 | where three surfaces meet | every corner of every arrangement | three planes: one 3×3 determinant of stored coefficients. Two planes and a quadric: the discriminant of the quadratic the pair-of-planes line cuts out of it, cleared of `\|n₁ × n₂\|²` | 3 / about 14 | `Plane::meet_of_three` / **new**, `rcad-geom` | **a third of it** |
| 5 | how many points three surfaces meet in | the same | the sign of row 4's discriminant | as row 4 | **new** | **no** |
| 6 | is this corner on that cut curve | `place_the_corners` | `SideOfSurface` certified `Zero` on the partner surface | 1–2 | `side_of_surface` | **exists** |
| 7 | are two cut curves one curve | `line_for` | all planes: four 3×3 minors of coefficients. Otherwise: are the two partners one surface, which is row 2 | 3 / 2–12 | `relation_of_two_lines`, `intersect` | **half** |
| 8 | which of two points on an arc comes first | `nodes_along`, `order_along` | straight: a comparison on the direction's dominant axis. A conic: `orient3d(A, B, C, a point off its plane)`, because a conic is convex and three points on a convex curve are in cyclic order exactly when they turn that way | 1 / 3 | the dominant axis rule / **new caller of `orient3d`** | **half** |
| 9 | which of two arcs leaving a corner is anticlockwise | `turns_anticlockwise` | `orient3d(∇S(x), ∇T₁(x), ∇T₂(x), origin)` — §17.4's determinant of three normals with gradients in their place | 3 in the gradients | `orient3d`, **new caller** | **the predicate exists** |
| 10 | does this trimmed curved face hold this point | selection, ray casting, `validate` | **not settled here** — §31.10 item 1 | ? | **new** | **no** |
| 11 | where a ray meets a curved face's surface | `classify_uncertain_point` | ball: `(v·(c−b))² − \|v\|²(\|c−b\|² − R²)`. Pipe: the same with the axis projected out | 4 / 6 | **new** | **no** |
| 12 | which way a ray crosses a curved face | the same | `v · ∇S(x)` | 2 | **new** | **no** |
| 13 | is a piece on a face of the other group, and which way | the ON columns | row 2, **and** the two faces' stored orientations | — | **new**, and it is two questions — §31.3 | **half** |
| 14 | are two corners one corner | `Corners::of` | never a coordinate: the names of the three surfaces, sorted, plus which of their meets this is | — | **new naming, no arithmetic** | **half** |
| 15 | does a wire keep the material on its left | `check_curved_face` | **not settled here** — §31.10 item 2 | ? | **new** | **no** |

**Which of those numbers I am confident in.** Rows 1, 2, 3, 6, 7 and 14 are
statements about code that exists and I am confident in them. Row 9 I am
confident in and it is the nicest thing in the table: §17.4's derivation of
`turns_anticlockwise` uses only that an arc of `S ∩ T` runs along
`∇S × ∇T` and that `(a×b)×(a×c) = a·((a×b)·c)`, and neither step used flatness
anywhere, so the planar formula is the curved formula with a constant gradient
substituted in. Row 8's conic half I am confident in for the same reason a
convex outline can be wound: an ellipse is convex, so the cyclic order of three
points on it is the sign of the area they sweep, and no centre and no angle has
to be constructed. **Row 4's "about 14" is a guess**, and so is row 11's 4 and 6
— I have not written the expansions out, and §30.3 says the first person who
needs one will do that in an afternoon and be right where this is wrong.

**Three rows are the milestone, and they are 4, 10 and 11.** Everything else is
either shipped or a rename.

**The limit under row 4, stated as a prediction and classified.** Two planes and
a quadric is a quadratic in one unknown and its discriminant is a polynomial in
stored numbers. One plane and two quadrics is a quartic, three quadrics is a
degree-eight system, and neither has a discriminant this milestone is going to
write. **M4 answers row 4 when at least two of the three surfaces are planes and
refuses by name otherwise.** That refusal is a **gap** and not a limit — the
points exist, and a real algebraic solver reaches them — so §16.10 says fix it
rather than defer it, and M4 defers it anyway. The reason is the one §29.6
already found for the two-pipe discriminant, one degree up: the family of fixes
is the Dupont–Lazard–Lazard–Petitjean machinery or a certified numerical solver
with its own separation bound, and both are a milestone rather than a lane. The
consumer that will call it due is named: **two capped bars crossing near one of
their caps**, where the quartic where the two pipes meet runs into a cap circle,
so the corner is a plane and two pipes. Two bars crossing away from their caps
have no such corner and are M4's.

Nothing else in the milestone is blocked by it. A block with a round hole
through it, a round bar cut by a block, a ball with a pipe through it and two
bars crossing in the middle all have corners where at least two of the three
surfaces are planes, or no corners at all.

### 31.3 What a fuse over curved faces means

§25.7 settled "what a mesh of a solid is" and that was the most useful thing in
its section. This is the same job for a curved fuse. Every item below is a
meaning, not an algorithm.

**A face *is* its surface, and §17.1 is the same sentence with one word
changed.** A piece cut out of a curved face inherits the parent's `Surface` bit
for bit, so chaining fuses never degrades a sphere, and the stored corners are
cached samples of it.

**§17.1's re-derivation does not generalize, and the reason is worth having.**
A corner of a planar solid has three planes round it and Cramer's rule places
it. A corner of a *round bar* has two surfaces round it — the cap's plane and
the wall's cylinder — and the whole cap circle satisfies both. There is nothing
to solve. **Settled: a corner with three or more surfaces round it is
re-derived, and a corner with two is read out of the model.** That is not a
retreat, because `check_curved_face` has already certified that the stored
corner is `Decision::Zero` on its surface — a corner of a curved face is proved
to be exactly on the curve it sits on, which is a stronger promise than the
box a Cramer solve gives.

**A corner read out of the model and then moved is a box.** `bake` composes
placement, and `Isometry::apply_to_point` rounds. A transformed corner is
therefore *near* the transformed surface and not certified on it, so a
`side_of_surface` asked about it as an exact triple would certify a confident
sign about a rounding — §15.5's mistake by the road §27.7 already found once.
**Settled: a baked corner with two surfaces round it is a `PredicatePoint`
whose box is the placement's own rounding, tagged `Cause::ConstructedPoint`,
and it is representation wobble that refuses nothing (§17.2).** How wide the box
is, is arithmetic and §31.10 item 6 says it is a guess.

**A cut is by the second *surface*, whole, and never by the curve.** §17.4's
"everything that matters to a face is a pair of planes" becomes **everything
that matters to a face is a pair of surfaces**. `BakedFace::cuts` holds
`Surface` rather than `Plane`, the arrangement's `Line` holds a partner
`Surface`, and cuts.rs's argument for why no face pair may be skipped —
a plane reaches past the face carrying it, so skipping the pair leaves an edge
split on one side and not the other — reads the same with "surface" in it.
Nothing constructs a curve anywhere, which is §24.3 kept.

**A pair of surfaces can name two curves, and that is what `picked_by` is
for.** Where the planar arrangement has one line per pair, the curved one has
one *arc family* per pair, told apart exactly as `EdgeRun::PartOfWhereItCrosses`
tells them apart. So the arrangement's line list is a list of `EdgeRun`s, which
is a type that already exists and that the fuse can hand to `walk` without
translating — §24.3's requirement that nothing needs translating to become an
edge.

**A seam is an ordinary boundary edge, and a crossing that runs across one is an
ordinary node.** A seam's two faces are the *same* face, so `BakedEdge::across`
cannot be "the plane of the face on the other side": there is no other side.
It is read out of the edge's own `EdgeRun::PartOfWhereItCrosses::and`, which is
the field that exists for exactly this. After that the arrangement has nothing
to say about seams at all — a cut curve crossing a ball's meridian is a corner
where three surfaces meet, like any other. **This is where §15.5 pays: because
no decision in the arrangement is taken in `(u, v)`, the periodic-surface
degeneracy that roadmap §7.4 lists is not a case here.** It is a case one layer
down, in the mesher, where M2 already solved it.

**A split curved face may have no outer wire.** A ball with a pipe through it
leaves the ball's face as a sphere with two disks taken out, and neither of its
two closed boundary curves is more "outer" than the other. On a closed surface
"outline and holes" is not a property of the geometry; it is an order the face
stores, and `FaceData`'s doc already says the first wire is the outer one by
fiat. **Settled: the invariant a curved face has to satisfy is that every wire
keeps the face's material on its left, about the surface's outward normal —
not that the first winds one way and the rest the other.** The flat case is the
special case where those two coincide. That gives `check_curved_face` the
winding check it has never had; how the check is written is §31.10 item 2.

**A closed crossing with no corner on it still needs one vertex.** A wire needs
a coedge, an edge needs two ends, and `wire_closes_on_one_edge` needs both ends
to be one vertex. So the fuse has to place a vertex somewhere on a closed
crossing, and where it places it must depend on the two surfaces and on nothing
else, or `fuse(A, B)` and `fuse(B, A)` disagree. **Settled: the vertex goes
where `walk` starts, because `walk` already starts a whole closed run at a place
that is a function of the pair alone, and asking the question twice is the shape
§23.8 kept finding.** `rcad-geom` grows one small entry for it, and that is the
whole of the change.

**Which way a closed edge runs is decided by row 9, and that closes §30.8 item
10.** The mesher currently picks a closed edge's direction by the sign of its
turn round the face, which works for a bar's cap circle and says nothing about
a hole. The direction is the crossing's own stored one; which of the two faces
walks it forward is the face whose material lies on its left, and that is
`orient3d(∇S, ∇T, the arc's direction)`. One rule, both cases, and the mesher
stops having a rule of its own.

**`TouchAtOnePoint` cuts nothing.** Two tangent balls contribute no cut curve
and no vertex, so neither face is split and neither solid learns about the
other. Union leaves two shells that touch at a point and share no vertex, so
they come back as two solids; intersection is a single point, which is not a
solid, so the answer is empty and that is a success (§17.4). Roadmap §7.4's
tangent-spheres and tangent-cylinder–plane entries are that, plus the same
question asked of a tangency along a whole curve, which `intersect` answers as
one curve and the fuse cuts along like any other.

**Since Q1337, that is only true where no curve of the face already runs
through the touch point.** `fuse/arrange.rs`'s `Arrangement::place_the_touches`
hands the touch to the face as a cut when the point lies on one of its
crossings, and puts the point on that curve as a node — a frustum's floor
touching a steeper cone at its apex, with the wall beside it already cutting
that cone in two rulings through the same point, is the case this closes
(§347.8).

**`SameSurface` is §17.4's coplanar case with one word changed.** Two coplanar
faces contribute one cut line per edge of the other, each named by the pair
`{S, U}` for the face `U` across that edge. Two faces on one *cylinder* do
exactly the same thing, and `boundary_of` needs no change beyond its type. That
is the whole of the coincident-curved-face case, and it is free.

**But `SurfaceMeet::SameSurface` does not say which way round, and
`Plane::agreement_with` does.** The ON columns of §17.4's selection table need
same-facing against opposite-facing, and for a plane that is carried in the
stored coefficients because a reversed plane is a different four numbers. A
sphere has no reversed form; a face on the outside of a ball and a face on the
inside of a cavity are the same `Sphere` used with opposite `Orientation`.
**Settled: the ON question splits in two — "is it one surface", which is
geometry and `intersect` answers it, and "does it face the same way", which is
topology and the faces' own orientations answer it.** For a plane the two are
welded together today, and unwelding them is what makes one rule cover all
three surface kinds. §17.4's ON row is corrected in §31.11.

**The merged meet type, which §27.3 handed here.** `FaceMeet` keeps its shape
and widens:

```rust
pub(crate) struct FaceMeet {
    carried_by: [Surface; 2],
    picked_by: SmallVec<[Surface; 1]>,
    ends: [MeetPoint; 2],
    along: SmallVec<[EdgeKey; 1]>,
    approximation: Option<Approximation>,
}

pub(crate) enum MeetPoint {
    Existing { vertex: VertexKey, point: PredicatePoint },
    OfThreeSurfaces { surfaces: [Surface; 3], which: u8, point: PredicatePoint },
}
```

`which` is the new field and it is load-bearing: three surfaces can meet at more
than one point, so the three sorted surfaces no longer name a corner on their
own, and `Corners::of` would join two different corners into one. It counts
along the arc's own order, which row 8 decides exactly, so it depends on the
three surfaces and not on which face asked first. `approximation` finally gets
a producer that is not zero, which is what §15.7 predicted and §17.5 corrected
twice.

At the commit each `FaceMeet` becomes an `EdgeRun`, by a table and not by a
judgement:

| The two surfaces either side | The ends | Written as |
|---|---|---|
| two planes | two different corners | `Straight` |
| any pair naming one closed curve | one corner twice | `RightRoundWhereItsFacesCross` |
| a pair naming two curves, or an arc of one | anything | `PartOfWhereItCrosses { and, picked_by }` |
| one surface, twice — a seam | two corners | `PartOfWhereItCrosses { and, picked_by }` |

### 31.4 The two type fixes that go first, and the one that is not a fix

The sequencing is settled elsewhere and this section plans inside it: §30.8
items 5 and 4 land as one lane against a still-flat fuse, where the existing 985
tests certify them, before `Error::FuseNeedsFlatFaces` comes off. Both change
types every predicate in the kernel is certified about, and re-certifying a
moving type underneath fanned-out lanes is the expensive order.

**Item 4 first, and it is small.** §29.6 measured a 22-entry expansion spanning
505 binary orders that cannot be squared where it stands, and showed the same
expansion written `2^300` higher squaring without complaint. The fix is not a
working frame and not centred inputs: **what the caller wants is a sign, a sign
is unchanged by multiplying by a positive power of two, and multiplying an
expansion by a power of two is exact.** So `Exact` slides the expansion into the
window before squaring and slides nothing back, and there is no accuracy
question to argue about anywhere in it. §30.4 already published the window —
every pairwise product exact needs the smallest entry above `2^-484` and the
largest below `2^512`, a spread of about 996 binary orders — and §15.6 says a
published bound needs a test, which that one does not have yet.

What has to be true afterwards: the two-pipe comparison decides wherever the
spread fits, `Error::Undecidable` out of the quartic pairs falls, and §30.7's
two moving rows — **4.64% `Undecidable` on turned quartic pairs and 10.49% on
pairs written at `2^k`** — are re-measured and the thresholds in
`how_often_naming_a_crossing_refuses.rs` moved down to match. A rate that does
not move is the lane's own regression, not a surprise about the world.

**Item 5 is `Plane`, and this gate found that the fix as written is not one.**

§29.8 diagnoses the half-space gap as the *normalization*: "every `Plane`
constructor scales its normal to length one and that division rounds, so which
normals come out exactly square to a given direction is not something a caller
chooses". §30.8 item 5 turns that into "`Plane` storing an unscaled normal".
Both sentences are about `normalized_as`. **They are both silent about the step
before it, and that step has the same effect.**

`Plane::reduced_by_largest_component` divides all three components by the
largest, and that division is correctly rounded rather than exact. Worked
through, for a pipe whose stored axis direction is `w = (1, 3, 5)` — the three
natural normals square to it, each of them exactly square before the plane
stores it:

| the caller writes | reduced to | the exact dot with `w` |
|---|---|---|
| `(3, −1, 0)` | `(1, −1/3, 0)` | `2⁻⁵⁴` |
| `(0, 5, −3)` | `(0, 1, −3/5)` | `2⁻⁵³` |
| `(5, 0, −1)` | `(1, 0, −1/5)` | `−2⁻⁵⁴` |

Not one of them survives, and level 3 of the cascade is exact so it reports
every one of them as genuinely not square. For `w = (1, 2, 3)` the normal
`(2, −1, 0)` *does* survive, because 2 is a power of two and dividing by it is
exact. **That is the real rule, and it explains §29.8's own observation better
than §29.8 does: reduction preserves exact squareness exactly when the largest
component is a power of two, which on the lattice it always is — "on the lattice
it is a coordinate axis and comes out right, and off it, sometimes."**

So the property to aim at is not "an unscaled normal". It is this, and it is
what the lane's test has to say:

> **A normal a caller wrote to be exactly square to a direction it already holds
> is still exactly square to that direction after a `Plane` has stored it.**

The family of fixes, with what each keeps and what each gives up:

1. **Store the four numbers the caller handed over, unchanged.** Gets the
   property outright. Gives up §22.6 — two faces built from the same plane by
   routes that differ by a scale factor store different numbers, and
   `CornerName::Meet`, `canonical_cut_planes`'s dedup and
   `plane_of_the_joined_face` all compare bit patterns.
2. **Keep the reduction, drop only the normalization.** This is §30.8 item 5 as
   written and the table above says it is **not a fix**.
3. **Reduce by the power of two nearest the largest component.** Exact, so it
   keeps the property; keeps §22.6 for scale factors that are powers of two and
   loses it for the rest, which is the case §22.6 was written about.
4. **Separate the stored numbers from the canonical name**: store what the
   caller handed over, and give `Plane` a canonical key that the three
   bit-comparing call sites use instead. Keeps both properties and adds a second
   answer to "are these the same plane" beside `agreement_with`, which is
   §23.8's pattern for the ninth time.
5. **Leave it**, and keep paying 0.44% of turned planar pairs and 10.9% of
   turned quartic ones.

**Not settled here, and deliberately.** Which of 1, 3 and 4 to land is a
question about what §22.6 actually needs, and §22.6's own test —
`the_same_wall_built_from_a_longer_edge_is_the_same_plane` — is the thing that
answers it. What is settled is that **the lane must land one of them, must say
in §32 which of the two properties it gave up, and must leave a test behind for
each.** Choosing 2 because §30.8 named it is the one outcome this gate exists to
prevent.

**And `Axis` has §22.6's bug, unfixed, and M4 is its first consumer.**
`Axis::new` normalizes and does not reduce. Drawing 200 000 integer directions
and scaling each by an integer between 2 and 97, **115 834 of them — 58% — store
different bits from the direction they are proportional to**. Reducing first and
then normalizing brings that to zero out of 200 000. Nothing has noticed because
nothing builds the same pipe two ways; a curved fuse does, the moment two solids
are flush against each other on one cylinder, and then `intersect` answers
`Curves` where it should answer `SameSurface` and the ON columns never fire.
`the_same_pipe_built_from_a_longer_axis_direction_is_the_same_pipe` is the test
that fails today, and it belongs in this lane because it is the same change to
the same kind of type.

**What certifies the whole lane.** The 985 tests that pass today, unchanged in
count and in what they assert, plus:

- the squareness property above;
- `Plane::SIGNED_DISTANCE_ROUNDOFF` re-derived, because its derivation uses
  `|n₁| + |n₂| + |n₃| ≤ √3`, which is a statement about a unit normal — §15.6;
- every reader of `offset()` **as a length** found and given something that is
  one: `InPlane::of`'s reach in `select.rs`, `Surface::ROUND_TRIP_ROUNDOFF`'s
  "largest number involved", `walk`'s conic size, `Plane::origin`, `project`,
  `basis`, `plane_coordinates_of`, `point_at`, `signed_distance_to` and
  `validate`'s `allowed_wobble`;
- `FORMAT_VERSION` to **3**, with a round trip, because the four numbers a plane
  writes out change;
- the moved figures, measured and recorded: the 0.44% and 10.9% of §29.8.

### 31.5 `rcad-predicates` takes nothing at this gate, and is frozen

The first milestone that needs no change to it. §15.9's rule stands: a change
reopens this gate and stops the fan, so a lane that finds it needs one comes
back here rather than taking it.

Weighed and put down:

- **A `Cause` for a corner moved by a placement.** §31.3 tags a baked curved
  corner `Cause::ConstructedPoint`, whose doc says "a point that had to be built
  rather than read ... landed between two `f64`". A read corner that had to be
  moved is the same quantity told the same way, and a second variant would be a
  second answer to a settled question (§1.5).
- **A `Measures` arm.** `ManyLengths { how_many }` is parametric and §31.2's
  highest row is a fourteenth-degree question, which it already spells.
- **`Measures::Dimensionless`.** §28.6 and §30.8 item 7 keep it one line away.
  It is still one line away and M4 is still not its consumer: nothing in
  §31.2 asks a question without units.
- **A fifth `Decision`.** Same answer as §27.2.

### 31.6 Who owns what, and in what order

§13 rule 2 shapes a milestone gate, fan, gate; §15.9 gives each crate one owner
during a fan; §13 rule 6 caps work in flight inside one crate at two or three.

| Lane | What it is | Crate it owns | Opens |
|---|---|---|---|
| **0. This section** | §31 | `docs/decisions.md` | done |
| **1. The two type fixes** | §31.4, both of them, expansion first | `rcad-geom`, and it ripples everywhere | now, **alone** |
| **2. Where three surfaces meet** | §31.2 rows 4, 5, 8, and one point on a closed crossing | `rcad-geom` | when 1 lands |
| **3. Does a curved face hold this point** | §31.2 rows 10, 11, 12, 15 | `rcad-topo` (`classify.rs`, a new file, `validate.rs`) | when 1 lands, **beside 2** |
| **4. The curved arrangement** | §31.3, `fuse/bake.rs`, `fuse/cuts.rs`, `fuse/arrange.rs` | `rcad-topo` | when 2 and 3 land |
| **5. Curved selection and the commit** | `fuse/select.rs`, `fuse/assemble.rs`, `fuse/mod.rs`, `fuse/history.rs` | `rcad-topo`, **same owner as 4** | when 4 lands |
| **6. The four curved catalogue entries** | roadmap §7.4's remainder, test-only | `rcad-topo/tests/` | alongside, from when 5 lands |
| **7. The metamorphic suite over curved solids** | test-only | `rcad-topo/tests/` | alongside, from when 5 lands |
| `rcad-predicates` | nobody | frozen (§31.5) | — |
| `crates/rcad/src/lib.rs` | everybody | re-export lists only | — |

**Lane 1 runs alone and nothing else is open.** That is the sequencing decision
this gate was told to plan inside, and reading the code agrees with it: `Plane`
is in the signature of `orient3d`'s callers, `validate`, the mesher, the
splitter, the intersector and the serialized format, and a lane holding any of
those open while the type moves underneath it would be debugging someone else's
regressions, which CLAUDE.md rule 4 forbids by name.

**Inside lane 1, the expansion goes first.** It is the smaller change, it is in
a different file, and it moves two of §30.7's measured rows. Doing it after
`Plane` would mean measuring those rows against a `Plane` that had just moved,
with no way to say which change moved them.

**Lanes 2 and 3 run side by side, and that is the only concurrency in the
milestone.** They are in different crates, §15.9's rule is satisfied literally,
and neither consumes the other: lane 2 answers questions about three surfaces
and never looks at a face; lane 3 answers a question about one face and its
wires and never needs three surfaces at once. Lane 3 is written against lane 2's
absence, not against its contract.

**Lanes 4 and 5 are one owner, in sequence.** §13's table files "boolean
classification and shell assembly — graph traversal" under *does not split*, and
§17.6 kept the same two pieces in one head for the planar fuse. This is that,
again, and §24.12's argument against file-level ownership inside one crate
applies unchanged.

**§24.12's prediction was scored wrong in both halves and this section does not
repeat its mistake.** §30.2's conclusion was not "stop predicting" — it was that
half of §24.12 was scored at the gate, by reading two functions for an hour, and
that checking a prediction is far cheaper than the prediction is accurate. So
§31.10 makes one, and M4's first closing record scores it.

**Generators, per §13 rule 5.** Lane 2 draws triples of planes, balls and pipes.
Lane 3 draws point grids against a bar and a ball. Lane 4 draws blocks with
round holes. Lane 5 draws the same shapes under the three rules. Lanes 6 and 7
draw the catalogue and turned pairs of curved solids. **Lanes 4, 5, 6 and 7 all
want a bar and a block, and that is worth naming rather than glossing, exactly
as §25.10 named C and D sharing the bar and the ball.** What separates them is
the oracle: lane 4's is that the pieces of a face cover it and overlap nowhere,
lane 5's is a dense point sampling against a mesh, lane 6's is a pass/refuse
table, and lane 7's is metamorphic. No one wrong answer satisfies all four.

**Section numbers, reserved now so no two lanes collide over them.** §32 is the
two type fixes, §33 is the curved fuse, §34 is M4's closing gate.

### 31.7 M4 exit criteria

Sixteen, each naming a test that does not exist yet. §30.3 warns that the
dangerous sentences in a settling document are the ones that specify a test
rather than claim a number, because a number gets checked by the first person
who needs it and a test that already exists does not — so **every name below is
a test that has to be written, and criterion 3 is the one that reuses an
existing file and therefore the one to read twice.**

1. A normal a caller wrote square to a direction is still square to it once a
   plane has stored it —
   `a_half_space_written_square_to_a_direction_is_still_square_to_it_once_stored`.
2. The same pipe described by two proportional axis directions is one pipe —
   `the_same_pipe_built_from_a_longer_axis_direction_is_the_same_pipe`.
3. The half-space gap is closed and the figures moved: the shipped thresholds in
   `how_often_naming_a_crossing_refuses.rs` come down, and
   `two_lines_no_f64_half_space_can_tell_apart_are_refused_rather_than_guessed`
   is replaced by one asserting the same pair off the lattice is now named.
4. A comparison too wide to square where it stands is squared somewhere else —
   `a_comparison_too_wide_to_square_is_decided_after_it_is_slid_up_the_range` —
   and the published window has a test —
   `an_expansion_wider_than_the_window_cannot_be_squared_at_any_scale`.
5. Where three surfaces meet is answered as boxes, every one certified `Zero`
   on all three under a certified policy —
   `every_point_where_three_surfaces_meet_is_on_all_three_of_them`. This is
   M3's criterion 3 one dimension down and it is the load-bearing one.
6. The count of meeting points is a sign, and a lattice control refuses none —
   `a_certified_meeting_of_three_lattice_surfaces_never_refuses`.
7. Three surfaces the milestone cannot solve refuse by name rather than
   guessing —
   `three_curved_surfaces_meeting_at_a_point_are_refused_rather_than_solved`.
8. Points along a planar crossing are ordered with no angle constructed —
   `three_points_on_a_conic_come_back_in_the_order_they_run_round_it`.
9. One authority answers whether a trimmed curved face holds a point, and it
   agrees with a fine mesh of that face wherever both can tell —
   `whether_a_curved_face_holds_a_point_agrees_with_a_mesh_of_the_same_face`.
10. Point-in-solid works on a curved solid, with the ON case —
    `the_middle_of_a_bars_wall_is_on_its_skin`,
    `a_point_a_hair_inside_a_balls_skin_is_inside_it`.
11. `Error::FuseNeedsFlatFaces` and `Containment::NeedsFlatFaces` are gone, and
    a block with a round hole through it is one solid —
    `a_block_with_a_round_hole_through_it_is_one_solid`.
12. The pieces a curved face is cut into cover it and overlap nowhere —
    `the_pieces_a_curved_face_is_cut_into_cover_it_and_overlap_nowhere`.
13. A fused curved solid validates, meshes, and passes §23.4's oracle —
    `a_block_with_a_round_hole_meshes_into_a_closed_surface` and
    `a_ball_with_a_pipe_through_it_meshes_into_a_closed_surface`, both through
    `the_mesh_closes_up`.
14. Roadmap §7.4's four remaining entries are a pass/refuse table —
    `the_four_curved_entries_as_a_table`: tangent spheres, tangent
    cylinder–plane, seam-crossing intersections, closed periodic surfaces.
15. The metamorphic properties hold over curved solids —
    `swapping_the_two_sides_of_a_curved_pair_changes_not_one_bit`,
    `a_bar_with_a_copy_of_itself_taken_away_is_nothing_at_all`,
    `uniting_a_block_and_a_bar_the_other_way_round_gives_the_same_solid`.
16. The two rates, reported family by family and never blended (§17.7): a
    lattice-aligned curved control at **0%** under `Policy::certified()` —
    `every_certified_fuse_of_a_lattice_curved_pair_is_decided_and_records_nothing`
    — and a turned curved family measured and asserted below a threshold
    recorded in §34 — `how_often_a_certified_fuse_of_a_turned_curved_pair_refuses`.

**No number is offered for criterion 16's threshold, on purpose.** §26.3 ended
with "it should stop suggesting numbers" and §30.3 said two milestones is enough
and the convention should be a rule. This is the rule being kept: the lane
measures it, §34 records it, and this section does not guess it.

Criteria 5, 9 and 16 are the load-bearing three. Five is the only new
construction the whole milestone rests on; nine is the question three separate
call sites ask and the one place two answers would silently disagree; sixteen is
the control that makes the headline mean anything, and §17.7's precedent is that
the control is what tells a timid filter from real undecidability.

The claim audit runs before §34, per §16.10, and every refusal M4 can produce is
classified limit or gap by name, as §30.5 did for M3.

### 31.8 §30.8's eleven items, sorted

The way §27.6 sorted §26.6's six.

| | | |
|---|---|---|
| 1 | `classify` and `fuse` gated on `is_wholly_flat` | **M4's, and it is the whole milestone** — §31.1, criterion 11 |
| 2 | the interval divide is 4% and the money is in settling | **not work.** A figure with no target is not an optimisation (§24.16), and M4 has no performance criterion. §34 may re-measure it beside a curved fuse |
| 3 | the general quartic is only ever marched, so a fuse over two crossing pipes spends criterion 7's 0.577 in anger | **M4's**, and it lands as lane 5's two-bars case rather than as a criterion of its own |
| 4 | the two-pipe discriminant needs its expansion slid up the range | **lane 1**, first half — §31.4 |
| 5 | `Plane` storing an unscaled normal | **lane 1**, second half, and §31.4 says the fix as written is not one |
| 6 | a crossing that runs through itself needs a third `EdgeRun` variant and a marcher that can start away from a singular point | **deferred, and the consumer is named but is not M4's** — §31.9 |
| 7 | `Measures::Dimensionless` | **not M4's** — §31.5 |
| 8 | the mesher's floor moved by about fifty | **not work.** Nothing needs it finer, and M4 does not change that. §34 watches it |
| 9 | `MOST_POINTS_A_MESH_WILL_HOLD` is shadowed by the per-crossing cap, and the first solid with five curved edges is a fused one | **M4's, and it is a watch rather than a task** — a block with a round hole has three curved edges and a ball with a bar through it has two, so it may still not fire |
| 10 | a closed edge that is a hole in a curved face has no direction rule | **M4's, and §31.3 settles it** with row 9's determinant. Lane 4 |
| 11 | `Undecidable` out of `mesh` is still unreachable | **not work, and this gate predicts M4 does not reach it either** — see below |

**Item 11 deserves the paragraph, because it has now been handed on twice.**
§26.6 item 4 named M3 as the milestone that would reach it and §30.1 recorded
that M3 could not, because `walk` hands the mesher 3D points and the mesher
reads them back through `coordinates_of`, so every corner a triangulator sees is
an ordinary `f64` pair. §30.8 item 11 says the first constructed corner in a
mesh domain is a fuse's. **It is not**, and the reason is one line of
`assemble.rs`: `write_into` stores `corner.point`, which is `middle_of(point)` —
a `Point` and not a box. A committed vertex has always been an exact triple and
a curved fuse does not change that; the width goes on the vertex as an
`Approximation` and never into the coordinate. So a mesh domain will still hold
no constructed corner after M4. **This is a prediction and §34 scores it.** What
would make it reachable is a `Model` whose vertices are boxes, which is a change
to what a model *is* and is nobody's milestone yet.

### 31.9 What stays deferred, and what would call it due

§13 rule 3: a part with no named consumer is not built. Each row names the
consumer and, where there is one, the figure that would call it due.

| Deferred | Consumer | What would call it due |
|---|---|---|
| The BVH broad phase (roadmap §7.1, sent to M4 by §17.4) | a fuse over hundreds of tools | A measured pair count. §17.4's argument that a *face*-level box test changes an answer still holds, and the solid-level one is already there. Nothing in M4 has a target |
| The bump arena (§11, sent to M4 by §17.5) | the same fuse | A measured allocation share of a curved fuse. §30.8 item 2 says the money in curved geometry is in settling, not in allocation, and §1.4 says an optimisation with no number it must beat is the mistake. §34 can produce the number |
| `rkyv` (§6, §24.14) | still nobody | Unchanged |
| Cone and torus (§24.6) | a STEP file that holds one (M5), or draft (M6) | Unchanged. §24.6's argument — a new `Surface` variant touches `validate`, the mesher, `is_wholly_flat`, the fuse and serialization — is *more* true during M4, not less |
| The B-spline fit of a marched crossing (§1.8) | unchanged | Unchanged |
| A crossing that runs through itself (§29.6, §30.8 item 6) | **named as M4's fuse, and this gate moves it out** | Two equal pipes whose axes meet, or a ball reaching exactly across a pipe. Both are knife edges a fuse reaches only when a caller asks for them exactly, they need a third `EdgeRun` variant *and* a marcher that can start away from a singular point and stop at it, and M4 already has one refusal of that shape (§31.2's row 4 limit). It is deferred to the milestone that builds the algebraic solver, and until then it refuses by name and `the_four_curved_entries_as_a_table` records the refusal |
| Solving three curved surfaces (§31.2) | two capped bars crossing near a cap | A real algebraic solver, and it is a milestone rather than a lane |
| A working frame that centres its inputs (§24.8) | — | **Not deferred and not built.** §30.8 item 4 said the deferral came due; §31.4 says what came due was sliding an expansion by a power of two, which is a different and much smaller thing. Centring inputs still has no figure asking for it |

**Two of those rows are gaps deferred rather than fixed, and CLAUDE.md says fix
a gap and defer a limit.** They are named here rather than buried for exactly
that reason: a crossing that runs through itself and a meet of three curved
surfaces are both gaps, both have answers, and both are deferred because the
family of fixes is a milestone's worth of algebra. Saying so is the difference
between "cannot" and "we chose not to", which §24.17 is about.

> **2026-10-10 note (dispatch 66, Q1603): half of the self-crossing row came
> due.** Two equal pipes whose axes meet are not a knife edge. They are the
> equal tee and the cross, which a caller asks for exactly whenever two bores
> share a drill size, so a fuse reaches them without anyone pushing a shape to
> a limit. Nor do they need a marcher. With unit axes `a` and `b` through a
> common point, a point on both pipes has `(p·a)² = (p·b)²`, so the crossing
> lies in the planes `p·(a − b) = 0` and `p·(a + b) = 0`, and each plane cuts
> both pipes in the same ellipse. What they do need is a name for two curves of
> one pair that cross each other, and a node where they do (§347.9). Q1603,
> filed as Q1558 by §347.9, builds that and answers the Steinmetz cross.
> *Since Q1611 (2026-10-10) the cross is named and gets its nodes, but every
> fuse of it is still refused `CannotTellTheCellsOfAFaceApart`, inside the
> arrangement; §347.9's last note says why.* The
> other two shapes stay as this row has them: a ball reaching exactly across a
> pipe (Q1559), and a pipe resting on the inside of a wider one. Each crosses
> in a quartic through a point where the two surfaces are tangent, which needs
> the marcher this row says is missing. Q1603 answers the second only if it can
> do so without that marcher, and otherwise leaves it refused under this row.

> **2026-10-10 note (Q1625): equal bores leaning at each other are named at
> more angles, and every fuse of them is still refused.** The namer picks each
> half by two flat surfaces through where the axes meet, and each has to pass
> exactly through that point. Facing along a pipe's stored direction misses it
> for every direction scaled to length one that does not run along a coordinate
> axis, since that dot rounds. Since Q1625 a surface that cannot face its own
> axis faces a direction with whole-number components from −2 to 2 instead,
> lying exactly in the flat of the two axes and leaning towards its own, all
> three asked exactly. That took the test's family from 32 to 408 of 1224 pairs
> named in four halves (`mise exec -- just t -p rcad-geom -E 'binary(merged) &
> test(two_pipes_of_one_width_on_crossing_axes) &
> test(the_namer_names_this_family_in_four_halves_and_the_walker_draws_every_piece)'
> --no-capture`). The other 816 are a gap: bores on `(3, 4, 0)` and `(3, 0, 4)`
> are pinned refused, and a facing slid a few `2⁻⁵³` along `(0, 1, −1)` would
> pass exactly through. Three families of fix were weighed: change which
> direction a surface faces (this one), slide that direction until its offset
> is exact (the row left for the 816), and a picker that holds a point and a
> facing rather than a rounded offset, which reaches every pair but touches
> `half_space.rs`, the walker and the fuse.
>
> A fuse gets no further yet. Two bars of radius 1/4, one along `x` and one
> turned 50°, 60°, 70° or 100° about `z`, crossing at `(11, 0, 0)` or with the
> second starting at `(11, −3.25, 0.5)`, were refused at `intersect` before
> Q1625. Now they are refused in the arrangement instead, as
> `WhereThreeSurfacesMeetNeedsTwoOfThemFlat` or `Undecidable`, in all six fuses
> of each; none answers and none panics (a scratch probe, not kept). One
> reason is in `fuse/arrange.rs`: `a_half_of_an_ellipse` knows a half only by
> pickers bit-equal to the pipes' stored directions, so these halves get no
> node where they meet. It is not the only one: the bar turned 120° about `z`
> through `(11, 0, 0)`, whose stored direction does pass exactly through and
> which was named before Q1625, is refused the same
> `WhereThreeSurfacesMeetNeedsTwoOfThemFlat` before and after.
> *Since Q1646 (2026-10-10) `a_half_of_an_ellipse` knows a half by which pipe
> each picker leans towards, asked exactly, and a far cap's circle is shown
> clear of the other pipe. The bars through `(11, 0, 0)` at 50°–100° now stop
> where the cross does, `CannotTellTheCellsOfAFaceApart` (Q1622); at 120° they
> get past the arrangement and are refused meshing a face or building a wire
> (Q1656). The bars starting at `(11, −3.25, 0.5)` are refused as before
> (Q1655), and since Q1645 names the crossing at 120° that bar is refused the
> same way. `two_equal_bars_leaning_at_each_other.rs` pins all of them.*

> **2026-10-10 note (Q1645): a facing slid a few units in its last place names
> 704 of the 1224, and the pair left pinned needs a picker that holds the exact
> middle.** Where neither the stored direction nor a whole-number one passes
> exactly through the middle `c`, the namer now slides the stored direction
> along each short whole-number way in the flat of the two axes, one step at a
> time, and keeps the first facing that still lies exactly in the flat, still
> leans towards its own axis and dots with `c` to a number `f64` holds. Those
> are the three exact checks Q1625 asks, so nothing became a tolerance. After
> the slide it tries two to four of the stored direction plus fewer of the
> other one, so a pair named by the slide keeps its surfaces. That took the
> family from 408 to 688 with the slide and to 704 with the sums (`mise exec --
> just t -p rcad-geom -E 'binary(merged) &
> test(two_pipes_of_one_width_on_crossing_axes) &
> test(the_namer_names_this_family_in_four_halves_and_the_walker_draws_every_piece)'
> --no-capture`), and
> `bores_on_3_4_0_and_3_0_4_are_picked_by_their_directions_slid_until_they_pass_exactly`
> and
> `bores_on_3_minus_6_2_and_2_3_6_are_picked_by_four_of_one_direction_and_one_of_the_other`
> pin one pair of each. A step is the unit in the last place of the smallest
> component the way moves, not a fixed `2⁻⁵³`: a component from ¼ to ½ sits on
> a grid of `2⁻⁵⁴`, and a fixed `2⁻⁵³` names 644 rather than 704 (measured by a
> temporary edit). The slide stops at 64 steps either way. One step moves the
> dot by an odd number times `2ᵉ`, so `2ᵏ` steps in a row run through every
> pattern of the `k` bits between `2ᵉ` and the dot's own grid and one of them
> clears all `k`; at the test's middle `(11, −3.25, 0.5)` `k` is at most 7, and
> 4096 steps name the same 704 in ten times the time (also a temporary edit).
> `LONGEST_SLIDE` in `intersect.rs` carries the whole argument.
>
> The 520 pairs still refused miss 233 surfaces between them (scratch scripts
> in exact fractions over the namer's failed searches, not kept). 180 lie in a
> flat with no whole-number way from −2 to 2, and the sums miss them too. 53
> have such a way, but no slide of up to 4096 steps of `2⁻⁵³` to `2⁻⁵⁶` along
> it finds a facing: own `(0, 0.6, 0.8)` against `(0.8, 0.6, 0)` along
> `(1, 0, −1)` moves the dot in steps of `21·2⁻⁵⁴`, and the dot has a bit at
> `2⁻⁵⁵` that no number of them clears.
>
> One of the 180, bores on `(3, 4, 0)` and `(4, 0, 3)` through
> `(11, −3.25, 0.5)`, has no flat surface at all, and
> `no_flat_surface_written_in_f64_picks_the_halves_of_bores_on_3_4_0_and_4_0_3`
> shows it. The two ellipses meet on the line through `c` square to the flat,
> so a picker has to hold that line, which puts its facing in the flat; and
> `HalfSpace::through_a_point_facing` rounds `f·c` once, so its surface holds
> `c` only when `f64` holds `f·c`. The test lists every facing `f64` holds in
> that flat, twelve up to a power of two, and none of them dots exactly with
> `c`. Of Q1625's three families of fix, the first two, which direction a
> surface faces and how far it is slid, can only ever give one of those twelve,
> so both are used up for this pair. The four halves are there, since two pipes
> of one width whose axes meet always cross in two ellipses, so this is a gap
> and not a limit, and the third family is the fix: a picker that holds the
> exact `c` and a facing rather than a rounded offset. Whether each of the
> other 179 is the same was not worked out. A row is filed for that picker, and
> one each, with no class since neither has been run, for the 53 and for a
> middle whose coordinates are long binary fractions, like a decimal `1.85`,
> which puts `k` near 52, where a searched slide cannot reach and the step
> count would have to be solved for.

> **2026-10-10 note (Q1646): what stopped the bars crossing right through in
> the arrangement.** Q1625's arrangement refusal for the bars through
> `(11, 0, 0)` came from `place_the_meets` asking about a bar's pipe, its own far
> cap and the other pipe. The cap cuts its pipe in a circle, and nothing could
> prove that circle misses the other pipe. Q1292 is not the cause. Q1646 added
> `a_cut_of_a_pipe_is_clear_of_the_third`, which boxes the cap's circle and asks
> the third surface about the box, and `a_half_of_an_ellipse` now knows a half
> by which pipe each picker leans towards, asked exactly as
> `(n·E)²‖U‖² > (n·U)²‖E‖²`. Where those bars stop now is in the dated sentence
> that ends Q1625's note above. Pinned in `two_equal_bars_leaning_at_each_other.rs`
> (`mise exec -- just t -p rcad-topo --test two_equal_bars_leaning_at_each_other`).

> **2026-10-10 note (Q1653, Q1654): the slide is solved for rather than
> searched, and names 1096 of the 1224.** `a_flat_surface_through_where_the_walls_touch`
> now tries two more facings after everything Q1645 tried, and every one still
> passes the three exact checks. The first is the stored direction in its
> lowest terms (`own_in_lowest_terms`). `(2, 3, 6)/7` is stored as exactly
> `p·2⁻⁵⁴·(2, 3, 6)` with `p = (2⁵⁴ − 1)/7`, since `2/7`, `3/7` and `6/7` each
> sit a seventh of a unit in the last place above their grid points. So its dot
> with the middle is `p`, 52 bits, times a short sum, and it rounds where the
> triple's does not
> (`bores_on_2_3_6_and_2_6_3_are_picked_by_the_whole_number_triple_one_is_a_multiple_of`).
> The second is a slide of the stored direction, and of its lowest terms, along
> each whole-number way, with the step count solved for rather than searched
> (`slides_solved_for`). Steps run from `2⁻⁵³` down to `2⁻⁶¹`
> (`FINEST_SOLVED_STEP = 8`). In units of its lowest bit the dot is `X + n·Y`.
> Where `Y` is odd, the counts that put it on `f64`'s grid are
> `n ≡ −X·Y⁻¹` modulo a power of two, one for each size of dot. Where `Y` is
> even, the counts taken are those that carry the dot down to a size whose grid
> holds its low bit. Those are families (a) and (b) of the row. The row's
> example, own `(0, 0.6, 0.8)` slid along `(1, 0, −1)`, is named by a slide of
> about `0.124` that brings the dot from about `−1.55` to just under `−¼`
> (`bores_on_0_3_4_and_4_3_0_are_picked_by_a_slide_solved_for`). Family (c), the
> whole set of facings `f64` holds in the flat, was not built, because (a), (b)
> and the lowest terms named all 53 surfaces.
>
> At `(11, −3.25, 0.5)` the count goes from 704 of 1224 to 752 with the solved
> slide alone, and to 1096 with both. The 56 surfaces still refused there all
> lie in a flat of two `(3, 4, 0)/5` directions with no whole-number way from
> −2 to 2, the shape
> `no_flat_surface_written_in_f64_picks_the_halves_of_bores_on_3_4_0_and_4_0_3`
> shows has no facing for one pair. At `(11.85, −3.15, 0.45)`, a middle in
> decimal millimetres, the count goes from 100 to 168 and then 328
> (`the_namer_names_this_family_crossing_at_a_middle_in_decimal_millimetres`).
> 371 surfaces are still refused there: 196 in a flat of two `(3, 4, 0)/5`
> directions with no whole-number way, 162 in one with a way, and 13 in a flat
> of two `(1, 1, 0)` directions with a way. Whether a facing `f64` holds exists
> for those was not asked, and Q1657's picker holding the exact middle would
> name them either way. Steps coarser than `2⁻⁵³` were tried, down to `2⁻¹`,
> and gave facings like `(−0.5, 2.09e15, −2.09e15)` at an offset of `−7.5e15`,
> which the walk then refused, so none is taken. The count test at
> `(11, −3.25, 0.5)` runs in 1.36 s against 2.3 s at the base. Measured with
> `mise exec -- just t -p rcad-geom
> -E 'binary(merged) & test(two_pipes_of_one_width_on_crossing_axes) &
> test(/the_namer_names_this_family/)' --no-capture`. The 752 and 168 were
> measured with the lowest terms taken out. The 371 were counted with a
> temporary print before the namer's `Ok(None)` and sorted by a scratch script;
> neither is kept.

> **2026-10-13 note (Q1892): the picker that holds the exact middle is built.**
> Where no facing the namer tries passes exactly through the middle, a surface
> is now `Picker::InFrontOfAFlatThrough`, which holds the middle and the pipe's
> own direction and asks its side exactly, with no offset to round. §443 says
> what it is, why it is a new arm rather than a change to `HalfSpace`, and how
> a model and a STEP file write it out.

### 31.10 What this gate does not know

§30.3's whole lesson is that a settling document written before anyone has read
the arithmetic gets things wrong, and that the cheap ones are the numbers. These
are the places this section is planning on top of a guess, worst first.

1. **How to decide whether a trimmed curved face holds a point.** §31.2 row 10,
   and it is the schedule risk of the milestone. The planar answer fans the
   outline from a corner and asks `orient3d` against a reference point off the
   plane, so nothing is projected (§15.5). There is no such reference point off
   a sphere that works for the whole face, and unrolling a cylinder goes through
   `atan2`, which makes any decision on the result a decision on projected
   coordinates. Three routes I can see and cannot choose between: a ray *along
   the surface* from the point, crossing-counted against the wires, which turns
   the question into more three-surface meets; the sign of the gradient
   determinant at each arc, which is exact but only decides for a face whose
   arcs all curve the same way; and cutting the face into pieces the flat
   argument does reach. **I do not know which, and the lane's first day should be
   spent finding out rather than building.**
2. **How `validate` checks that a wire keeps the material on its left.**
   §31.3 settles what the invariant *is* and not how to ask it. Today
   `check_curved_face` checks corners and nothing else, and the flat winding
   check has no curved twin. It may turn out to be row 9's determinant at each
   corner, in which case it is cheap; it may need row 10, in which case it is
   row 10 again.
3. **Whether `Plane`'s fix can keep §22.6.** §31.4 lists five candidates and
   demonstrates that the one §30.8 named is not a fix. Which of the other three
   to land, and what it costs the three bit-comparing call sites, is the lane's
   and this gate does not know it.
4. **The degrees in §31.2 rows 4, 5, 11 and 12.** Marked as guesses in the row.
5. **Whether the mesher can mesh what the fuse hands it.** A fused curved face
   is the first with a hole in it, and `mesh.rs` refuses one that goes more than
   a quarter turn round with `CurvedFaceWithAHoleGoesTooFarRound`, on a doc
   comment whose reason is *"the fuse refuses a curved solid outright"*. That
   sentence stops being true in lane 5. `MOST_PIECES_A_FACE_IS_CUT_INTO` rests
   on "four pieces is the answer for every face this kernel builds", which is
   also about today's builders. **Neither is a bug yet and both are claims that
   the milestone falsifies**, which is §16.10's shape, and the claim audit
   before §34 is where they get re-argued.
6. **How wide a moved curved corner's box has to be.** §31.3 says it is the
   placement's own rounding; whether that is one `Isometry::POINT_ROUNDOFF` or
   two — because `Surface::transformed` rounds the surface as well as the point
   — is a derivation nobody has done.
7. **Whether `TouchAtOnePoint` cutting nothing survives the manifold check.**
   §31.3 settles that two tangent balls united come back as two solids sharing
   no vertex. `Model::validate` has no way to notice that two shells touch, so
   the answer is a model the kernel calls valid and a machinist would call one
   part. That may be right and it may be the thing §34 has to correct.

**The prediction, recorded before the work and to be scored at §32 rather than
at the milestone's end** (§30.2's cheaper half). §12.1's lesson — the money goes
to code that was already written and looked finished — has the precondition
§30.2 discovered: it holds for milestones that mostly *rewire* and not for ones
that mostly write. M4 mostly rewires. So the lesson should hold, and the
specific place is **`fuse/arrange.rs`, and not the parts about lines.** The
cycle walk, the winding count and `piece_around`'s ray are all written in the
vocabulary of `Line::axis` and `forward_climbs_the_axis` — a dominant axis and a
direction that either climbs it or does not — and every one of those is a
straight-line idea sitting inside a function whose name says nothing about
straight lines. `first_arc_met` compares positions along one axis at every turn
and has no other way of asking where anything is.
The second guess is `mesh.rs`, for item 5's reason. The third, and the one I
would most like to be wrong about, is that `classify_uncertain_point` costs
nothing again: §17.6 predicted it as M2.5's schedule risk and §17.8 recorded
that the cost was elsewhere, and predicting the same function twice in a row is
how a prediction becomes a habit.

### 31.11 The numbers at the gate, and corrections to earlier sections

| | at §30 | now |
|---|---|---|
| tests (`cargo nextest`) | 985 | 985 |
| suite duration | 18.0 s | 18.3 s |
| doc tests | 19 | 19 |
| `FORMAT_VERSION` | 2 | 2 |
| fuzz corpus | 562 | 562 |

A gate that writes one file moves nothing, which is what it should look like.
`just default` is green, `cargo-deny` included.

Corrections:

- **§29.8** blames the half-space gap on `Plane` scaling its normal to length
  one. The reduction before it has the same effect and §31.4 demonstrates it on
  three normals square to `(1, 3, 5)`. §29.8's own observation — right on the
  lattice, sometimes off it — is explained by the reduction and not by the
  normalization: reduction is exact exactly when the largest component is a
  power of two.
- **§30.8 item 5** says the fix is "`Plane` storing an unscaled normal". As
  written that is item 2 of §31.4's five, and it is not a fix.
- **§30.8 item 11** predicts M4 reaches `Undecidable` out of `mesh`. §31.8 says
  it does not, and why, and files that as a prediction of its own.
- **§17.1** says every corner is re-derived as the meet of the planes around it.
  That is true of a corner with three surfaces round it and false of one with
  two — a round bar's cap corner — which is read out of the model instead and is
  certified onto both its surfaces by `check_curved_face`. §31.3.
- **§17.4's ON row** treats "is it the same surface" and "does it face the same
  way" as one question that `Plane::agreement_with` answers. For a curved face
  they are two questions and the second one is topology. §31.3.
- **§24.9** leaves what a fuse does with `TouchAtOnePoint` and `SameSurface` to
  M4. Settled in §31.3: a touch cuts nothing, and one surface is §17.4's
  coplanar case with one word changed.
- **§27.3** leaves `FaceMeet` for M4 to merge. Settled in §31.3, and
  `MeetPoint::OfThreePlanes` gains a field rather than only widening — three
  surfaces can meet at more than one point, so the three names no longer name a
  corner.
- **`crates/rcad-topo/tests/fusing_solids.rs`** says of
  `every_face_of_the_answer_has_exactly_one_input_face_behind_it` that it is
  *"deliberately removed at M3: a curved intersection can produce a face that is
  no input face's child"*. This gate believes that is wrong — every piece is
  still cut out of an input face, and what a curved fuse invents is edges rather
  than faces. Lane 5 keeps that property alive over curved inputs, and if it
  fires, the property is what says so and `Fused::generated_faces` stops being
  empty for the first time.
- **`Axis::new`** normalizes without reducing, so it is not canonical under
  scaling. 58% of drawn proportional pairs store different bits. §31.4, and it
  is lane 1's.

