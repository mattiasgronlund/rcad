## 328. A plane through a slanted axis holds it exactly, and a bend round one is answered — 2026-09-28

*Queue row Q105, from §243.7. Two lanes off `0a7142ff`:
`lane-q105-axis-dot-decided-exactly` made the change (`b22a05be`) and stopped
at its budget before running anything; `lane-q105-axis-dot-decided-exactly-2`
ran the probe, wrote the tests and this section. **A gap, closed.** On five of
the six slanted planes probed, the stored normal is exactly square to the
stored axis, and `holds.rs` refused the bend's wall because an interval dot
straddled zero. On the sixth the exact dot is not zero, the plane cuts the ring
in a quartic, and "no" is the true answer: a limit, and still refused.*

### 328.1 What changed

`holds_the_axis_of`, a `bool`, is now `how_the_axis_lies_on`, a `Decision`:
`Zero` when the whole axis lies on the plane, `Pos` or `Neg` when it does not,
`Undecidable` when it cannot tell. The normal-dot-axis question goes through
`rcad_predicates::decide` as the predicate `SquareToTheAxis`: level 2 is the
interval dot it always was, level 3 an exact sum of three `two_product`s. The
centre-on-the-plane question is `Plane::side_of_point`, as before. A "no" from
either question is the answer, because either alone means the plane does not
hold the axis; short of that, a "cannot tell" from either is.

The caller, `what_a_ring_face_between_two_planes_holds`, goes on at `Zero`,
refuses `Pos` and `Neg` with `AFaceMissesPartOfWhereItsSurfacesCross` as every
refusal there already was, and turns `Undecidable` into `Error::Undecidable`
through a new `gave_up_on_the_axis`. That record is filed under
`WHERE_A_PLANE_AND_A_TORUS_CROSS`, the name `intersect` asks the same two
questions under, so a log reads the same whichever of the two gave up.

`intersect` settles its `runs_along` with `Exact::dot_of`, which is
`pub(crate)` in `rcad-geom`, so `SquareToTheAxis` does the same three products
with `rcad_predicates::expansion` directly. Same arithmetic, same floor
(§328.4).

### 328.2 §243.7's table, rebuilt

§243.7's probe was deleted uncommitted. The first lane reconstructed its planes
from the prose; this lane ran them, as a unit test in `holds.rs` that can reach
the private function, then kept the slanted rows as
`a_plane_through_a_slanted_axis_holds_it_exactly_where_intersect_names_two_circles`.
Each row is `Torus::new(Axis::new(origin, axis), 4, 1)` cut by
`Plane::from_point_and_normal(origin, n)`, with `n` worked out from the axis's
stored direction `w`. Working `n` out from the axis as written instead of from
`w` gave the same stored normal on every row. "Before" is the old reading, the
interval dot's sign alone, where anything but `Zero` was "no"; `intersect` runs
under `Policy::certified()`.

| axis | `n` from `w` | stored `n` | before | exact dot | after | `intersect` |
|---|---|---|---|---|---|---|
| `(0.6, 0.8, 0)` | `(0, 0, −1)` | `(0, 0, −1)` | `Zero` | zero | `Zero` | two circles |
| `(0.6, 0.8, 0)` | `(w₁, −w₀, 0)` | `(0.8, −0.6, 0)` | straddles, `1.1e-16` | zero | `Zero` | two circles |
| `(1, 1, 0)` | `(0, 0, −1)` | `(0, 0, −1)` | `Zero` | zero | `Zero` | two circles |
| `(1, 1, 0)` | `(w₁, −w₀, 0)` | `(0.7071067811865476, −0.7071067811865476, 0)` | straddles, `2.2e-16` | zero | `Zero` | two circles |
| `(0, 1, 1)` | `(1, 0, 0)` | `(1, 0, 0)` | `Zero` | zero | `Zero` | two circles |
| `(0, 1, 1)` | `(0, w₂, −w₁)` | `(0, 0.7071067811865476, −0.7071067811865476)` | straddles, `2.2e-16` | zero | `Zero` | two circles |
| `(1, 2, 3)` | `(w₁, −w₀, 0)` | `(0.8944271909999159, −0.4472135954999579, 0)` | straddles, `5.6e-17` | zero | `Zero` | two circles |
| `(1, 2, 3)` | `(w₂, 0, −w₀)` | `(0.9486832980505138, 0, −0.31622776601683794)` | straddles, `1.1e-16` | **below zero**, about `−1.48e-17` | `Neg` | `ACrossingBetweenAPlaneAndATorusThatIsAQuartic` |
| `(1, 2, 3)` | `(0, w₂, −w₁)` | `(0, 0.8320502943378437, −0.5547001962252291)` | straddles, `1.1e-16` | zero | `Zero` | two circles |

The widths are the interval's own, `Decision::Undecidable`'s `enclosure`. The
exact dot's size is the sum of the expansion's entries, printed once.

**Six slanted planes, where §243.7 counts five.** The reconstruction holds one
plane more than the verifier asked, and which one it left out cannot be
recovered. Its one named refusal — axis `(1, 2, 3)`, `n = (w₂, 0, −w₀)`, an
enclosure `[−1.1e-16, 0]` wide — is the `Neg` row here, with the same width.

**So the first lane's guess holds.** `intersect` already decided this dot
exactly, and §243.7's four disagreements were `holds.rs` refusing exact zeros.
Now the two agree on every row. Whether a normal worked out this way comes out
exactly square depends on how the rescale to length one rounds its two slanted
components against the axis's, and the three rows round `(1, 2, 3)` show both
outcomes; nothing short of the exact dot says which, and the exact dot's answer
is the right one either way. That is §16.10's line: where it is zero the answer
existed and the arithmetic had not reached it, a gap; where it is not, the
stored plane leans off the stored axis, and no arithmetic makes that a circle.

### 328.3 The bend that was refused and is answered now

`tests/a_bend_round_a_slanted_axis.rs` describes a bend through `SolidBuilder`,
as `tests/rings/mod.rs` describes its bends about `+z`: ring four, tube one,
round `(0, 1, 1)`, from the corner `(3, 0, 0)` to the corner `(1, 2, −2)`, a
little over seventy degrees round. The near cap faces `(0, −1, 1)` and the far
one `(−4, 1, −1)`; both are planes through the axis whose stored normals
straddle zero against the stored axis and are exactly square to it. The seam
runs along the inner equator, picked inside the ring's own pipe and by `x ≥ 0`.

The brief suggested a half bend, corners `(±3, 0, 0)` and both caps on one
plane. Its wall cannot tell the change: `what_a_ring_face_between_two_planes_holds`
turns away a plane stored twice, because each circle has to lie strictly in
front of the other cap's plane and a half bend's lies on it. So the far corner
moved to `(1, 2, −2)`, and that is the only choice. *2026-10-03, Q753: a plane
stored twice is no longer turned away for that. When it holds the axis, the
wall is read as the half of the ring its wire keeps (§377.3).* A scratch search over every
point with coordinates in sixteenths, `|x| ≤ 6` and `|y − z| ≤ 12`, found eight
points on this ring: `(±3, 0, 0)`, `(±5, 0, 0)`, `(1, ±2, ∓2)` and
`(−1, ±2, ∓2)`, all on the inner or outer equator. The tests ask the wall about
all eight, and ask `classify_point` about three points inside or near the tube.

With `holds.rs` put back to `0a7142ff` and the test file kept, both tests fail:
the wall refuses all eight points with `AFaceMissesPartOfWhereItsSurfacesCross`,
and `classify_point` answers `CannotFollowAFacesTrimming` for all three. With
the change, the wall holds its two circles' points as its boundary and turns
away the other five. `(3.25, 1.625, −1.625)`, in the tube half way round
between the caps, is `Inside`. Its mirror across the near cap, and
`(0, 3, −3)` past the far cap, are `Outside`.

`(0, 2.8, −2.8)` was tried first for the last of those and came back
`CannotFireARayAtAFacesSurface`, since a ray at a torus needs a line written
out exactly. That is `Containment`'s own limit and not this row's.

### 328.4 What is still refused

1. **A cap that leans off the axis by its rounding** — the `Neg` row. Pinned by
   `a_wall_whose_cap_leans_off_a_slanted_axis_by_its_rounding_is_refused`,
   built straight into the arena. `SolidBuilder` cannot build it, because that
   cap's circle is not a crossing `intersect` names. A limit.

2. **The first lane's underflow case was a wrong guess.** Normal
   `(−2⁻¹⁰⁰⁰, 1, 2⁻⁵⁰⁰)` against axis `(1, 0, 2⁻⁵⁰⁰)`: both products are
   `f64`s, `2⁻¹⁰⁰⁰` is a normal number, the interval rounds nothing and says
   `Zero` at level 2, and level 3 is never asked. `intersect` gets past the dot
   too and stops later, at `CannotNameWhichPieceOfACrossingThisIs`. That later
   refusal was measured and not looked into.

3. **Row Q328's family reaches this question, with a different case.** Normal
   `(−0.8·2⁻⁹⁶⁹, 1, 0.6·2⁻⁹⁶⁹)` against axis `(0.6, 0, 0.8)`, both stored as
   written. The dot is exactly zero. Neither product is an `f64`, so the
   interval straddles, and both are below `2⁻⁹⁶⁹`, so `product_is_exact` refuses
   them. So `how_the_axis_lies_on` says `Undecidable`, and `intersect` refuses
   it as `Error::Undecidable`. It is a gap: `two_product` holds each product
   exactly, and the lowest set bits of each pair of factors (`2⁻¹⁰²¹` and
   `2⁻⁵³`, or `2⁻¹⁰²²` and `2⁻⁵²`) add up to `2⁻¹⁰⁷⁴`, which is Q328's test.
   Pinned by
   `a_normal_square_to_the_axis_through_products_below_two_to_the_minus_969_is_refused`,
   which also holds the underflow case of item 2 at `Zero`.

   **Q328's fix in `rcad-geom` does not reach this pin by itself.**
   `SquareToTheAxis` calls `rcad_predicates::expansion::product_is_exact`
   directly, not `Exact`. Whoever lands Q328's test has to point
   `SquareToTheAxis::exactly` at it too. The pin then fails, and flips to
   `Zero`. That is Q328's row and not a new one.
