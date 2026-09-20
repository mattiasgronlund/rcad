## 27. What opening M3's gate settled

*Recorded 2026-09-01, before either stage opens. §24 is the specification and it
stands; this is what its four gate items turned out to be once somebody read the
discriminants, plus the six things §26.6 handed over that §24 did not know about.
A gate and not a milestone, so it is short.*

### 27.1 The two stages on paper: every sign question, and what it costs

§24.4 item 4 cannot be done without knowing what the stages will ask, so here is
the whole list. Every question is a **sign**, every one is a polynomial in the
numbers a `Surface` stores, and none of them is a distance compared against a
tolerance.

Write `N = n·n` for a plane's normal, `W = w·w` for a pipe's axis direction —
both about one and exactly one only for a coordinate direction.

| The question | The polynomial | Degree | Dimension | Who decides it |
|---|---|---|---|---|
| are two directions parallel | three 2×2 minors of `w₁, w₂` | 2 | ratio | exact, `orient2d`'s shape |
| is a plane parallel to an axis | `n·w` | 2 | ratio | exact, two products |
| do two axes meet | `(o₂−o₁)·(w₁×w₂)` | 3 | volume | `orient3d`'s determinant, over stored numbers |
| is a centre on an axis | three components of `(c−o)×w` | 2 | length | exact, `orient2d`'s shape |
| plane against ball | `N·R² − (n·c−d)²` | 6 | area | new, in `rcad-geom` |
| plane against pipe, parallel to it | `N·r² − W·(n·o−d)²` | 6 | area | new |
| ball against ball | `(R₁±R₂)² − ‖c₂−c₁‖²` | 2 | area | new |
| ball against pipe, centre on the axis | `W·R² − r²` | 4 | area | new |
| ball against pipe, in general | `V² − 4r²A`, with `A = ‖(c−o)×w‖²` and `V = A + r² − W·R²` | 8 | **four lengths** | new |
| pipe against pipe, parallel axes | `(A·W₂ − r₁²W₂ − r₂²W₁)² − 4r₁²r₂²W₁W₂` | 12 | **four lengths** | new |
| pipe against pipe, one surface | `A = 0` and `r₁²W₂ = r₂²W₁` | 4 | area | new |

Four things fall out of that table, and three of them are corrections to §24.

One row needs a note. "Do two axes meet" is `orient3d`'s three-by-three
determinant exactly, but not `orient3d`'s *signature*: feeding it four points
means adding a direction to an origin, and that sum rounds. The determinant is
written over the stored numbers instead, which is what every other row does too.

**The discriminant has to carry `n·n` and `w·w`, and the textbook one does not.**
A plane's normal is scaled to length one on the way in and that division rounds,
so `Plane::side_of_point` is certified against the plane those *stored* numbers
name; the same is true of a pipe, where `Cylinder`'s own doc says the certified
radius is `r / ‖w‖`. Write the discriminant as `R² − (n·c−d)²` and it answers
about a ball of radius `R/‖n‖` — a different ball, by about `1e-16·R`. Then
criterion 3, which wants every evaluated point certified **ON** both surfaces by
`SideOfSurface`, fails by exactly that much on any surface that is not
axis-aligned. Multiplying through by `N` costs three products and two sums.
**The rule, and it is the load-bearing one: an intersector's discriminants are
polynomials in the same stored numbers `SideOfSurface` reads, with no
normalization anywhere.** Otherwise the two halves of M3 are certified about two
different solids.

**There is no near-tangency band because there is no constructed input.** §24.10
says the band exists only for constructed inputs and inside the marcher. Sharper:
`intersect` takes two `Surface`s, a `Surface` is made of `f64`, and level 3
decides any polynomial in exact `f64`. The naming half of M3 has **no**
constructed input at all. Its only way to refuse is exact arithmetic declining
because a product would overflow or fall into the subnormals — which §16.10 calls
a **gap** wherever a rescaling reaches the answer, and
`the_same_pipe_and_point_scaled_up_to_fit_put_the_point_outside` already
demonstrates that shape one degree down. So criterion 5's 0% is not a hope about
a narrow band; it is structural, and a refusal from the closed-form pairs at
ordinary magnitudes is a bug rather than a close call.

**The two quartic pairs do not fit the expansion buffers this workspace uses, and
that is the schedule risk nobody had listed.** `square_of_expansion` refuses more
than eight entries; a pipe's `‖(c−o)×w‖²` alone runs to 384. Squaring a
400-entry expansion needs `2·400²` — about 320 000 `f64`, 2.6 MB — and that is
not a stack array. The family of fixes considered: **(i)** a heap scratch used
only at level 3, which runs rarely, costs one allocation and refuses nothing;
**(ii)** decline past some length and report `Undecidable`, which trades
criterion 5's 0% away for a buffer size; **(iii)** drop the degree, which needs
`√A` and so leaves polynomial arithmetic altogether. **(i) is the answer**, and
it belongs to stage 2 — but `expansion`'s crate doc says *"every function writes
into a caller-supplied slice, so a predicate that knows its own sizes never
allocates"*, and a predicate that cannot know its own size within a stack budget
is a case that sentence did not have. Zero elimination keeps the drawn lengths
tiny; it is the worst case a fixed buffer would have to hold.

**None of the four planar pairs constructs a plane.** §24.5 files sphere ∩ sphere
as *"named exactly, by reduction to the row above"*, through the radical plane.
The reduction is needed to *evaluate points along* the circle and not to *name*
it: two spheres name their circle between them, so the run is
`RightRoundWhereItsFacesCross` and nothing is built. Plane ∩ sphere and the
ellipse of plane ∩ cylinder are the same. `picked_by` is only ever needed where a
crossing comes in **two pieces** — two lines from a plane parallel to a pipe's
axis, two circles from a ball centred on a pipe's axis, two loops of a quartic —
and in the first two the picking plane is exactly representable.

### 27.2 What went into `rcad-predicates`, and what was weighed and put down

Two things, and the crate is frozen from here until M3 closes.

1. **`Measures::ManyLengths { how_many }`.** The table above has two entries at
   the fourth power of a length, and `Measures` stopped at a volume. An
   enclosure from one of them filed as a `Volume` reads a whole working size too
   wide — so a decision the policy would have taken comes back undecided
   instead, which is the timid filter §24.10 says a 0% control cannot tell from
   genuine undecidability. §15.3's units bug, one degree above where §15.3 could
   see it.
2. **`Cause::PointAlongACrossing`.** §15.7 said the `Approximation` field for how
   far a computed curve strays has no producer yet and §24.3 answers that the
   fidelity belongs to *evaluating points along* a crossing. That is a point put
   where a curve is, and `Cause::FlatStandIn` is a triangle put where a surface
   is — one dimension apart, in the same mesh, and only the cause tells them
   apart. Last in the enum, per §25.8.

Weighed and **not** taken:

- **Multiplying one expansion by another.** Every discriminant of degree six and
  up needs it. It is not a `rcad-predicates` change: `rcad-geom` already owns
  `square_of_expansion` privately, §7.2 puts the quadric predicates in that crate
  because the quadric types are there, and the freeze does not reach it.
- **A public `Interval` divide.** The marcher will divide ranges.
  `Interval::divided_by` is `pub(crate)` and `rcad-geom` has `quotient_enclosing`
  of its own; both stages live in `rcad-geom`, so moving that function inside
  that crate is stage work and costs the freeze nothing.
- **`Measures` for a dimensionless value.** "Are these two axes parallel" and "is
  this plane parallel to that axis" have no units. Their inputs are direction
  components of order one, every product is exact, level 3 always decides, so no
  record is ever written and no arm would ever be read. §13 rule 3.
- **A fifth `Decision` for "tangent".** Tangency is a certified `Zero` on a
  discriminant. A variant for it would be a second answer to a settled question,
  which is §1.5's shape.
- **`Policy` knobs for the marcher.** §24.7 asks for two, and they exist: how
  fine is `Deflection`, which is not a policy, and what happens when the
  deviation cannot be proven, which is `Policy::wobble_is_acceptable`.
- **Provenance name constants for the new questions.** `&'static str` at the call
  site in `rcad-geom`, exactly as `SIDE_OF_SPHERE` is.

### 27.3 The type split, and three places §24.9 needs amending

`EdgeRun` **moves to `rcad-geom` whole**, which §24.9 left open. Nothing in it
names a key: it holds a second `Surface` and the half-spaces that pick one piece,
and both are geometry. `EdgeData` — which holds vertex slots — stays where it is.
`EdgeRun` therefore becomes public, so §24.15's export list is three names and
not two.

Three amendments:

- **`Curves` carries a `Vec`, not a `SmallVec<[NamedRun; 2]>`.** Two `NamedRun`s
  inline come to over five hundred bytes, every arm of an enum is as wide as its
  widest, and `SurfaceMeet::Nothing` is what most pairs of faces in a fuse
  answer. Widening the common answer to save an allocation on the rare one is
  backwards.
- **`RightRoundWhereItsFacesCross` still carries nothing.** §24.5 calls this gate
  work, and what landed is the corrected *claim* rather than a field: the variant
  says "one closed curve", which is true of every crossing the four planar pairs
  produce and false of a quartic that leaves two loops. The families of fixes are
  half-spaces on the closed variant, or a variant of its own. Neither is built,
  because the only consumer is stage 2, stage 2 is one owner in one worktree
  (§24.12), and §13 rule 4's settle-it-twice hazard needs two people. The format
  is versioned and explicitly unstable, so a third version is cheap.
- **The fuse's `FaceMeet` did not move and did not change.** It is a plane–plane
  meet whose ends are `MeetPoint`s naming input vertices, and it belongs to
  M2.5's planar fuse. §24.9's "`rcad-topo` wraps it with the `along` bookkeeping"
  describes M4's shape, and M4 is where a fuse over curved crossings is the first
  consumer that has to merge the two.

`NamedRun`'s ends are `rcad_predicates::Point`, so they may be a box rather than
a point. That is what makes §26.6 item 4 reachable, and it is deliberate: an end
handed over as an exact triple would buy a certified answer to the wrong question
(§2.5).

### 27.4 §24.12's prediction is aimed at a divide M3 does not call

§24.12 predicts the money goes to `SideOfSurface` for sphere and cylinder and to
the local interval divide beneath it. Both were read at the gate.

`SideOfSurface` needs no work. What needs the care is its *caller*: §27.1's first
finding is exactly the way to get a sphere predicate right and a sphere
intersector wrong.

**There is no interval divide beneath `SideOfSurface`.** Neither `Sphere::side_of`
nor `Cylinder::side_of` divides at any of the three levels — they are products,
sums and differences all the way down. `quotient_enclosing` sits beneath
`BSplineCurve::evaluate_with_enclosure` and its surface twin, and §24.1 dropped
the free-form half of M3, so **nothing in this milestone calls it**. §16.8's "M3
is the first caller that will care" is right that a marcher divides ranges and
wrong that this is the divide it will use: the marcher's is a new one, over a
different expression. So half of §24.12's prediction is misaimed before the work
starts, and the honest scoring at M3's close is against the marcher's own divide
and against `SideOfSurface`'s callers.

### 27.5 Who walks an edge, and what crosses that seam

§26.6 item 1: the mesher walks an edge by stepping in a straight line through the
surface's own two coordinates, which is right for a pipe's ruling, a cap circle
and a ball's meridian and wrong for the ellipse a tilted plane cuts out of a
pipe. Criterion 8 needs a solid whose edges were *computed* to mesh, so this is a
seam between M3's two stages and M2's mesher, and §13 rule 4 says settle it now.

**Settled: the evaluator in `rcad-geom` walks every edge, and `rcad-topo` walks
none.** Two authorities on where the points along an edge are is the shape §23.8
kept finding, and the mesher's straight line is one of them.

What crosses the seam:

- The mesher hands over a **`NamedRun`** — the edge's own run and its two ends,
  which §27.3's test shows every edge the kernel builds already is — together
  with **the surfaces of the two faces either side** (the run names the second
  surface only when the faces do not), and a **`Deflection`**, a `&Policy` and a
  log.
- It gets back **3D points**, in the edge's stored direction, ends not included,
  and one `Approximation` blaming `Cause::PointAlongACrossing`.
- It does **not** get a `(u, v)`. A shared edge lands in two different coordinate
  systems and the mesher already reads each with `Surface::coordinates_of`. That
  is what keeps the seam narrow: everything downstream of `walk` works from 3D
  points and needs no change at all.
- §23.4 is unchanged: the points are worked out **once** and the identical values
  go to both faces. The evaluator is the single authority; neither face
  re-derives.

Nothing of this is built at the gate. What the gate checked is that the types can
say it: `every_edge_the_kernel_builds_is_already_a_named_run`.

### 27.6 §26.6's six items, sorted

| | | |
|---|---|---|
| 1 | the mesher cannot walk a curved edge | **gate** for the seam, stage for the code — §27.5 |
| 2 | a ball's seam longitude is not read | **gate**, done — §27.7 |
| 3 | a new `Surface` variant is nine stops | **stage**, and nothing is due: §24.6 keeps the surface set fixed |
| 4 | `Undecidable` out of `mesh` becomes reachable | **stage**; the gate only had to make the ends of a run able to be a box, and they are |
| 5 | the evaluator bound over-charges a pipe on its own axis | **stage**, and only if a stage hands the mesher a surface that did not come from a `Model` |
| 6 | `measure` still asserts `six_volumes > 0.0` | **stage**, watched: §26.6 says to look again when a new kind of surface arrives, and M3 brings none |

### 27.7 What the gate found that the plan did not

**A pole set the walk's starting longitude, and a sound ball could not be
meshed.** Reading the meridian out of `EdgeRun` (§26.6 item 2) turned up a second
defect the first one was hiding. A ball's wire opens at a pole; a pole names
every longitude, so `coordinates_of` answers `atan2(0, 0)`; the walk took that
zero as where it started and unwrapped every corner after it towards longitude
zero. Half a turn away that is exactly the boundary the wire check refuses at, so
a ball seamed down the far half of its own meridian — a perfectly sound solid —
came back `AWireWillNotLieFlatOnItsFace`. A pole no longer starts a walk, and the
turns counted between poles are shifted rather than dropped. That is §23.8's
pattern for the eighth time, and the new kind of input was neither a solid nor a
number: it was **a field that had never been read**.

**The witness on the equator had to be handed over as a box.** Working out which
half of a meridian a seam is on means asking a half-space about a point on the
equator, and that point comes out of trigonometry. Given to the predicate as an
exact triple, `sin(π)` coming back `2.4e-16` instead of zero is enough to certify
a confident sign about noise and pick a half on the strength of it — §15.5's
mistake by a new road. Given the box it is really known to be inside, the
predicate answers `Undecidable`, neither half is ruled out and the mesher refuses.
`Error::ASeamBetweenTwoPolesNamesNoMeridian` is that refusal, and it has to be a
refusal rather than an assert because `validate` cannot catch it: both ends of a
pole-to-pole seam satisfy every half-space its true meridian satisfies, so
`check_edges` has nothing to look at.

**`Measures` had three arms and the pair table needs four.** §27.2 item 1. Worth
naming separately because it is the one gate item that would otherwise have been
taken *inside* the frozen window, which is what §15.9's precedent is about.

### 27.8 The numbers, and corrections to earlier sections

| | at §26 | now |
|---|---|---|
| tests (`cargo nextest`) | 860 | 878 |
| `FORMAT_VERSION` | 1 | 2 |

Corrections:

- **§24.4 item 1**: `EdgeRun` had to move to `rcad-geom` for `NamedRun` to hold
  one, and it could. §24.9 did not say where it would land.
- **§24.9**: the curve list is a `Vec`; `RightRoundWhereItsFacesCross` keeps
  carrying nothing and the gap is named instead; `FaceMeet` is M4's to merge.
  See §27.3.
- **§24.10**: "there is no near-tangency band for exact inputs" is true and
  understated — the naming half has no constructed input at all, so the only
  refusal it has is exact arithmetic running out of range. See §27.1.
- **§24.10 again**: the discriminants are not the textbook ones. They carry `n·n`
  and `w·w`, or the intersector and `SideOfSurface` are certified about two
  different solids.
- **§24.12**: `SideOfSurface` has no interval divide beneath it and M3 never
  calls the one §16.8 measured. See §27.4.
- **§26.4**: the doc-test count is 19 by `cargo test --workspace --doc`, not 24.
  It has not moved at this gate.

---

