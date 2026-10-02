## 378. Two quarter bends on one torus, end to end, under every rule — 2026-10-02

*Queue row Q754, which replaces Q198. This lane changed no source: it adds
`crates/rcad-topo/tests/two_quarter_bends_on_one_torus.rs` and this section.
Every number below was run on the base, `a189d01b`.*

In short: of the six fuses, four answer right and the two unions are refused
`ResultIsNotManifold { count: 4 }`. Selection keeps the right pieces. Assembly
then joins two different circles into one edge, because it names a closed
curve by its two surfaces alone, and a plane through a ring's axis cuts the
ring in **two** circles. That refusal is a gap: with the two circles kept
apart, in a scratch copy, both unions answer a half bend that validates and
measures `4π²`.

### 378.1 The pair, and what each rule should give

Ring four, tube one, both bends about `+z`: one from `+x` round to `+y`, the
next from `+y` round to `−x`. `a_quarter_bend` and its two consts are copied
from `two_frustums_stacked_on_one_cone.rs`, whose test fuses the same two with
the second one's axis turned round (§357.4, Q513).

The two bends meet only at one cap, the disk on `x = 0` round `(0, 4, 0)`. The
first faces `−x` there and the second `+x`. So their insides are apart, and
what each rule should give needs no measuring:

- **union**: one half bend from `+x` round to `−x`. Its two caps both lie on
  `y = 0` and face `−y`, one round `(4, 0, 0)` and one round `(−4, 0, 0)`.
  Pappus: a quarter is `π·1² · (2π·4)/4 = 2π²`, so the half is `4π²`,
  `39.47841760435743`.
- **intersection**: two solids that share only a disk share no volume, so the
  answer is no solid at all. That is §191's rule for every pair whose insides
  are apart, and `Model::fuse`'s doc says an empty answer is a success.
- **difference**: nothing of the first bend is inside the second, so the
  answer is the first argument whole, `2π²`.

### 378.2 What each fuse gives

`mise exec -- just t -p rcad-topo --test two_quarter_bends_on_one_torus`,
three tests, each fusing both ways round, all passing:

| rule | `+x→+y` first | `+y→−x` first |
|---|---|---|
| union | `ResultIsNotManifold { count: 4 }` | `ResultIsNotManifold { count: 4 }` |
| intersection | no solid | no solid |
| difference | the `+x→+y` bend | the `+y→−x` bend |

Every answer comes with an empty log and passes `Model::validate`. Each
difference has the first bend's three surfaces and measures inside the bound
`measure` gives with it at deflection `0.01`: `19.706118053894524 ±
0.3921329444569594` and `19.706144105509548 ± 0.3921333372476393`, against
`2π² = 19.739208802178716`. The two volumes are from a probe test that
printed them, on the same base.

**Q198's refusal is not reached any more.** Q198 said the wall piece's points
on the inner equator got `CannotFireARayAtAFacesSurface` from the other bend's
torus face, before Q102 (`5862b49e`) and Q667 (`7c980d78`) landed. With a
print in `select::choose` (378.3 says where), every one of the six pieces in
all six fuses gets placed. Against its own group each is `Outside`. Against
the other group each wall and each cap on `y = 0` is `Outside`. The two caps
on `x = 0` are `On` each other, facing opposite ways, so union and
intersection drop both and difference keeps the argument's.

### 378.3 Where the union is refused, and why

The probes ran in a copy of the base exported with `git archive` into the
session's scratchpad. They are temporary `eprintln!`s, and none of them is in
this branch.

**Raised at** `Assembly::darts_along_each_edge` in
`crates/rcad-topo/src/fuse/assemble.rs`. That is the first step of
`join_up_what_is_one_face`, which `assemble` calls straight after
`Assembly::from_the_pieces`. A print of every edge's claimants there, union
with `+x→+y` first:

- the edge made from the first bend's start circle, on `y = 0` round
  `(4, 0, 0)`, is claimed by four faces: that bend's cap on `y = 0` and its
  wall, **and the second bend's cap on `y = 0` and its wall**;
- the edge made from the shared circle on `x = 0` is claimed by the two walls,
  which is right.

The other order is the same, with the two bends' roles swapped. So the second
bend's end circle, round `(−4, 0, 0)`, became the same edge as the first
bend's start circle, round `(4, 0, 0)`.

**Why.** The arrangement hands each wall back as a piece with two wires of one
side each, the two circles. The seam along the inner equator is gone, so the
corners `(3, 0, 0)` and `(−3, 0, 0)` each do nothing but start a closed curve.
`Corners::with_each_closed_curve_started_once` then groups every such corner
by the curve it closes, and names that curve by
`names_of_the_two_surfaces` alone: the plane `y = 0` and the ring. Both
circles get that one name. A print of its candidates there:

    curve [y = 0, the ring]: vertex 4 at (−3, 0, 0), vertex 1 at (3, 0, 0), each closing only this curve
    curve [x = 0, the ring]: vertex 2 at (0, 3, 0), vertex 3 at (0, 3, 0), each closing only this curve

The function keeps the first candidate and hands every other one that closes
only this curve over to it. So vertex 1 becomes vertex 4's corner. `Edges::of`
then names an edge by its two corners and, for a closed one,
`which_curve_it_is`, which is the same two surfaces again. Both circles come
out as one edge, with four faces along it.

The `x = 0` line is the joining this function was written for: two vertices
at one point on one circle.

**The counter-probe.** In the scratch copy only, `on_curve` was keyed by the
two surfaces and the vertex's exact coordinates, so the two circles stayed
apart. Nothing else changed. Both union orders then answer one solid of four
faces: the two caps on `y = 0` facing `−y` and the two walls. The log is
empty, `Model::validate` passes, and the solid measures
`39.41226215940417 ± 0.7842662817045987` at deflection `0.01`, against
`4π² = 39.47841760435743`. The union test of this file then fails, `first
quarter first true: Ok(1)`, and the other two still pass. That is what shows
the pin can move. Comparing coordinates is not the fix, as 378.4 says: it is
only the shortest way to show that nothing after the naming refuses.

### 378.4 Gap or limit, and the fixes considered

**The union is a gap** (§16.10). The answer exists. The model can hold it:
each circle keeps its own vertex, as each bend already stores it. And the
counter-probe reached it with the rest of the fuse unchanged. The
intersection and the difference answer right, so they are neither.

The names `assemble.rs` builds corners and edges from are made of three
things: the surfaces' bits, the coordinates, and the input's own vertices. So
there are three families of fix:

1. **Tell the two circles apart by coordinates**, as the counter-probe did.
   Rejected. The module doc of `assemble.rs` says no coordinate decides what
   is one corner, only a name does. Keyed by coordinates, two input vertices
   at different points on one circle would also stop being handed to one
   corner, which `with_each_closed_curve_started_once` does on purpose. No
   test of that case was run here.
2. **Never let one input vertex take over another.** Rejected for the same
   reason: that hand-over is the function's job. Two input vertices on one
   circle are one curve, and only one of them can start its one edge.
3. **Name a closed crossing by its two surfaces and which of their closed
   crossings it is.** This is the `which` that `CornerName::Meet` already
   carries for three surfaces (§31.3), applied to two. A plane through a
   ring's axis is the case run here. `intersect` gives it one run per circle,
   each picked by its own half-space, and which one keeps a vertex is one
   exact sign. `holds.rs`'s `the_half_space_naming_the_meridian_circle_through`
   already asks it, to tell a cap's two disks apart (§250.5). This is the
   family to file. It has to reach every name keyed by the two surfaces
   alone: `on_curve` and `taken_over` in `with_each_closed_curve_started_once`,
   `CornerName::RoundACurve` and `HalfWayRoundACurve`, and the second field
   of `EdgeName`, which comes from `which_curve_it_is`.

Other pairs that cross in two closed curves were **not run**: a plane square
to a ring's axis, which cuts it in an inner and an outer circle; two rings;
and a plane through a ring's axis cutting a whole ring that has no vertex on
either circle, where both circles would be started at `RoundACurve` of one
name.

### 378.5 What is left

- **Q512** is already the row for this refusal. It was filed from §357.4 and
  asks for a trace first. This section is that trace, so Q512 can become the
  family 3 fix, with `crates/rcad-topo/src/fuse/assemble.rs` in its write
  set. The test it turns round is
  `two_quarter_bends_meeting_end_to_end_are_refused_as_not_a_manifold_when_united`.
  Once fixed, it should check the union validates and measures `4π²` inside
  its bound, both ways round.
- **Q513**, the same pair with the second bend's axis turned round, waits on
  Q512 and is not touched here.
- The three pairs at the end of 378.4 are guesses that nobody has run.
