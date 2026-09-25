## 245. Two bars side by side: selection was right, and `assemble` names two things wrong — 2026-09-25

*Q43. §231.4 ignored `two_overlapping_parallel_bars_stop_in_select_whichever_way_round`
because its first fuse refuses `ResultIsNotManifold { count: 1 }`, with only
a guess for the cause. This lane measured it on `543c05aa`. It added temporary
prints of every piece, of what `select` did with each one, of how `Edges::of`
named every kept side, and of every edge that did not have exactly two faces.
It then ran the test's six fuses, each on a fresh model so the slot numbers
match from one printout to the next. The prints are gone again. This section
is the only change.*

  245.1   What the six fuses do
  245.2   Selection kept the right pieces
  245.3   The guess's three legs: the first and third held, the second broke
  245.4   A second hole: arcs of two different circles become one edge
  245.5   A gap, and where each fix goes
  245.6   The other five fuses
  245.7   What this leaves

### 245.1 What the six fuses do

```
===== Union here on there: Err ResultIsNotManifold { count: 1 }
===== Union there on here: Err ResultIsNotManifold { count: 4 }
===== Intersection here on there: Ok Fused { model: ModelId(3), solids: [SolidKey { slot: SolidSlot(3v1), ... }], ... }
===== Intersection there on here: Err ResultIsNotManifold { count: 1 }
===== Difference here on there: Err ResultIsNotManifold { count: 4 }
===== Difference there on here: Err ResultIsNotManifold { count: 1 }
```

The `Ok` line is cut short where it says `...`, and so is the `chosen:` line
in §245.2. Every other quoted line is exactly as printed.

None of the six refuses `Undecidable`, and all six got through `select` and
printed what they kept. So the test's name and its pin, which says every fuse
stops in `select`, are out of date for all six fuses and not just the first.

There are two separate holes, called A and B below. Each refusing fuse hits
one or both:

| Fuse | Refusal | Hole A: one face each | Hole B: four faces |
|---|---|---|---|
| Union, here on there | `count: 1` | edges 0, 1, 10 and 3, 4, 13 | edges 2 and 5 |
| Union, there on here | `count: 4` | — | edges 0 and 4 |
| Intersection, here on there | answers | — | — |
| Intersection, there on here | `count: 1` | edges 1, 6, 7 and 3, 8, 9 | — |
| Difference, here on there | `count: 4` | — | edges 0 and 1 |
| Difference, there on here | `count: 1` | edges 2, 8, 9 and 5, 10, 11 | — |

`darts_along_each_edge` goes through the edges in the order they were made and
reports the first bad one. In the first fuse, edge 0 is made before edge 2. So
that fuse reports `count: 1` even though it has both holes.

### 245.2 Selection kept the right pieces

Each bar is cut into six pieces. Each cap splits into the lens and the rest,
and each wall splits into the part inside the other bar and the part outside.
For the union with `here` as the argument, `here` is solid 0, and pieces 0–5
are `here`'s and 6–11 are `there`'s:

```
select piece 0: On { solid: 1, same_way: true } -> KeepIfItComesFirst
select piece 1: Outside -> Keep
select piece 4: Inside -> Drop
select piece 5: Outside -> Keep
select piece 6: On { solid: 0, same_way: true } -> KeepIfItComesFirst
select piece 7: Outside -> Keep
select piece 10: Outside -> Keep
select piece 11: Inside -> Drop
chosen: [... piece: 0 ..., piece: 1 ..., piece: 2 ..., piece: 3 ..., piece: 5 ..., piece: 7 ..., piece: 9 ..., piece: 10 ...]
```

That is the union, piece for piece: `here`'s copy of each lens, the rest of all
four caps, and the outer part of each wall. The other five fuses kept the right
pieces for their rule too. So the first suspect that the `fuse-debugging` skill
names for this refusal, selection keeping or dropping the wrong piece, is not
the cause here. Both holes are in `assemble.rs`, in how the kept sides are
named. That naming happens before any edge's faces are counted.

### 245.3 The guess's three legs: the first and third held, the second broke

**Leg 1 held.** `add_round_bar` puts its bottom corner at `(radius, 0, 0)`.
So `here`'s bottom corner is `(5, 0, 0)` (`VertexSlot(1v1)`) and `there`'s is
`(11, 0, 0)`. The lens cut from `here`'s bottom cap has the first one as a
corner:

```
piece 0: anc=[FaceSlot(1v1)] P(n=0.0,0.0,-1.0 d=0) Forward whole=false
    w0: M[P(n=0.0,0.0,-1.0 d=0)|Cyl@x=0|Cyl@x=6]#1(3,4,0) -> VVertexSlot(1v1)(5,0,0) across Cyl@x=0 along [EdgeSlot(2v1)]
    w0: VVertexSlot(1v1)(5,0,0) -> M[P(n=0.0,0.0,-1.0 d=0)|Cyl@x=0|Cyl@x=6]#0(3,-4,0) across Cyl@x=0 along [EdgeSlot(2v1)]
    w0: M[P(n=0.0,0.0,-1.0 d=0)|Cyl@x=0|Cyl@x=6]#0(3,-4,0) -> M[P(n=0.0,0.0,-1.0 d=0)|Cyl@x=0|Cyl@x=6]#1(3,4,0) across Cyl@x=6 along []
```

**Leg 2 broke.** `(5, 0, 0)` is on `here`'s circle, since `5² + 0² = 25`. The
arc of that circle from `(3, 4, 0)` to `(3, −4, 0)` runs through it, and that
arc is one of the two that bound the lens. So the corner is on the lens's
edge, not strictly inside it: piece 0 above *is* the lens, and `(5, 0, 0)` is
one of its three corners. What is true, and it is all the mechanism needs, is
something weaker: the corner is strictly inside `there`'s disk, since
`(5 − 6)² = 1 < 25`. So it lies on an arc that `there`'s cap also has, as a
line where `here`'s wall cuts it.

**Leg 3 held, and these are the faces.** `here`'s lens walks that arc as two
sides through `(5, 0, 0)`, because `here`'s own circle starts there. `there`'s
cap outside `here` (piece 7) has the same arc as a cut. A cut starts and stops
only where three surfaces meet, so there it is one side from `(3, −4, 0)` to
`(3, 4, 0)`:

```
piece 7: anc=[FaceSlot(4v1)] P(n=0.0,0.0,-1.0 d=0) Forward whole=false
    w0: M[P(n=0.0,0.0,-1.0 d=0)|Cyl@x=0|Cyl@x=6]#1(3,4,0) -> VVertexSlot(3v1)(11,0,0) across Cyl@x=6 along [EdgeSlot(5v1)]
    w0: VVertexSlot(3v1)(11,0,0) -> M[P(n=0.0,0.0,-1.0 d=0)|Cyl@x=0|Cyl@x=6]#0(3,-4,0) across Cyl@x=6 along [EdgeSlot(5v1)]
    w0: M[P(n=0.0,0.0,-1.0 d=0)|Cyl@x=0|Cyl@x=6]#0(3,-4,0) -> M[P(n=0.0,0.0,-1.0 d=0)|Cyl@x=0|Cyl@x=6]#1(3,4,0) across Cyl@x=0 along []
```

Both pieces are kept, so three edges end up with one face each. Corner 1 is
`(5, 0, 0)`, and corners 0 and 2 are `(3, 4, 0)` and `(3, −4, 0)`:

```
edge 0: ends [0, 1] carried P(n=0.0,0.0,-1.0 d=0) / Cyl@x=0 from ["EdgeSlot(2v1)"] claimed by [(0, ["FaceSlot(1v1)"])]
edge 1: ends [1, 2] carried P(n=0.0,0.0,-1.0 d=0) / Cyl@x=0 from ["EdgeSlot(2v1)"] claimed by [(0, ["FaceSlot(1v1)"])]
edge 10: ends [0, 2] carried P(n=0.0,0.0,-1.0 d=0) / Cyl@x=0 from [] claimed by [(5, ["FaceSlot(4v1)"])]
```

The top cap does the same at `(5, 0, 10)`, with edges 3, 4 and 13. This is
**hole A**.

No arithmetic is involved. Every corner is a box of no width (the file's own
`a_cap_plane_against_two_side_by_side_pipes_meets_them_at_three_and_four`).
And `(5, 0, 0)` is not where three surfaces meet at all. It is only where the
bar's builder chose to start a closed circle. So the arrangement of `there`'s
cap cannot see it: that cap is cut by surfaces, and every surface through
`(5, 0, 0)` that cuts the cap contains the whole arc.

**Why one intersection answers and the other does not.** With `here` as the
argument, the lens and the part of the wall inside `there` are both `here`'s.
Both split the arc at `(5, 0, 0)`, so every kept side along it agrees. With
`there` as the argument, the lens that is kept is `there`'s, which does not
split the arc, while the wall is still `here`'s, which does:

```
side of FaceSlot(4v1) 1->0 along P(n=0.0,0.0,-1.0 d=0) / Cyl@x=0 named false arc Some(true) straight false -> new
side of FaceSlot(3v1) 0->4 along Cyl@x=0 / P(n=0.0,0.0,-1.0 d=0) named false arc Some(true) straight false -> new
side of FaceSlot(3v1) 4->1 along Cyl@x=0 / P(n=0.0,0.0,-1.0 d=0) named false arc Some(false) straight false -> new
```

The difference with `there` as the argument is the same. There, `there`'s cap
outside `here` meets `here`'s wall inside `there`, which is kept turned round.

### 245.4 A second hole: arcs of two different circles become one edge

The first fuse has a second bad edge, and this one has four faces:

```
edge 2: ends [0, 2] carried P(n=0.0,0.0,-1.0 d=0) / Cyl@x=6 from [] claimed by [(0, ["FaceSlot(1v1)"]), (1, ["FaceSlot(1v1)"]), (1, ["FaceSlot(1v1)"]), (4, ["FaceSlot(3v1)"])]
```

On the cap plane, four arcs join `(3, 4, 0)` to `(3, −4, 0)`, two on each
circle, because both circles pass through both corners. `Edges::of` names an
open side by three things: its two corners, the bit from
`which_arc_of_a_circle_this_is`, and whether it is straight. It does not name
the curve the side runs on. `which_curve_it_is` answers `None` for an open
side on purpose, and its doc says why. The bit only tells apart the two arcs
of **one** circle. `there`'s arc through `(1, 0, 0)` and `here`'s arc through
`(−5, 0, 0)` both leave corner 0 in the direction their own circle runs, so
both get `Some(true)`:

```
side of FaceSlot(1v1) 2->0 along P(n=0.0,0.0,-1.0 d=0) / Cyl@x=6 named false arc Some(true) straight false -> new
side of FaceSlot(1v1) 2->0 along P(n=0.0,0.0,-1.0 d=0) / Cyl@x=0 named false arc Some(true) straight false -> edge 2
side of FaceSlot(1v1) 0->2 along P(n=0.0,0.0,-1.0 d=0) / Cyl@x=6 named false arc Some(true) straight false -> edge 2
side of FaceSlot(3v1) 0->2 along Cyl@x=0 / P(n=0.0,0.0,-1.0 d=0) named false arc Some(true) straight false -> edge 2
```

In order, those four sides are: the lens along `there`'s arc; the rest of
`here`'s cap along `here`'s outer arc; the rest of `here`'s cap along
`there`'s arc; and `here`'s outer wall along `here`'s outer arc. That is two
good edges with two faces each, given one name. Hole B has nothing to do with
`(5, 0, 0)`, because neither of those arcs passes through it.

The difference with `here` as the argument shows it most plainly. The rest of
`here`'s cap has only two sides, one on each circle, and they came out as the
same edge:

```
side of FaceSlot(1v1) 0->1 along P(n=0.0,0.0,-1.0 d=0) / Cyl@x=0 named false arc Some(false) straight false -> new
side of FaceSlot(1v1) 1->0 along P(n=0.0,0.0,-1.0 d=0) / Cyl@x=6 named false arc Some(false) straight false -> edge 0
```

This is the case `which_of_them_a_closed_crossing_runs_round`'s pipe row did
not reach. That row's doc describes the same `count: 4` for two arcs of **one**
ellipse, which the bit now tells apart. Here the two arcs are on two different
circles.

### 245.5 A gap, and where each fix goes

**A gap, under §16.10.** The answer exists. Every corner is exact. None of the
six fuses refused `Undecidable`. Selection kept exactly the union's pieces.
The same pair answers as an intersection when `here` is the argument. Both
holes are in how the answer is named, before any edge's faces are counted, and
neither needs a number that the arithmetic failed to reach.

**Hole A goes in `crates/rcad-topo/src/fuse/assemble.rs`,
`Assembly::from_the_pieces`, beside `Corners::with_each_closed_curve_started_once`.**
That pass (§229.6, `63281be7`) already settles the same disagreement for a
**closed** side, where two coincident faces start one curve at two different
corners. This is the same problem on an open arc. The rule would be: take an
input vertex where every kept side that ends there runs along one crossing (the
same two surfaces, named either way round) and nothing else ends there. That
vertex is not a corner of the answer, and the two sides through it are read as
one before any edge is named. The rule depends only on which sides are kept,
not on where anything is.

Two other places to put it are both ruled out by the record:

- Splitting `there`'s cut at `here`'s vertex in `arrange.rs` would make the
  pieces depend on more than the face and the set of surfaces that cut it. The
  doc of `with_each_closed_curve_started_once` already refuses that (§17.4).
- Asking whether `(5, 0, 0)` lies on the cut would be deciding by coordinate
  something the names already settle.

**The fix lane should expect one answer that passes today to change.** The
intersection with `here` as the argument keeps `(5, 0, 0)` and `(5, 0, 10)` as
corners with only two edges each. Its history maps `VertexSlot(1v1)` to a
vertex of the answer, and `EdgeSlot(2v1)` to two edges. The rule above would
drop both corners and join each pair of edges. That answer is a corner in the
middle of a curved edge, the curved form of the case §17.4 joins away on a
flat one. This lane did not test whether it fuses again.

**Hole B goes in `crates/rcad-topo/src/fuse/assemble.rs`, `Edges::of`,** in
the `EdgeName` key. Wherever `which_arc_of_a_circle_this_is` answers `Some`,
the key should also carry the name of the crossing's curved surface:
`name_of_the_surface_either_way_round` of `carried_by[round_one]`. Then arcs
of two different closed crossings between the same two corners are two edges.
It should add only the curved surface and not the plane.
`which_curve_it_is`'s doc records what happened when a plane went into an open
side's key: an edge shared by two solids pressed flush against each other was
split, and the share of the skew family that came out as not a solid rose from
under three in a hundred to three and a half.

The curved surface is the one §231.3 found spelled two ways by two coincident
walls. Naming it either way round folds together an axis turned round, and an
origin slid along the axis where that folds exactly. The fix lane should check
that two coincident walls spelled two ways still name their shared arc as one
edge.

### 245.6 The other five fuses

These can be told without a fix, from the printout. All six fuses have already
got past `select`, so none of them can still stop in `select`, which is what
the test is named for and pinned to.

- **Union with `here` as the argument** has both holes. Fixing A alone leaves
  edge 2 with four faces, so it would refuse `count: 4`.
- **Union with `there` as the argument, and difference with `here` as the
  argument**, have hole B only.
- **Intersection and difference with `there` as the argument** have hole A
  only.
- **Intersection with `here` as the argument** answers today.

In all six printouts, every other edge has exactly two faces, because only the
edges listed in §245.1 were printed. So nothing here says any of the five
would refuse again at the manifold check once both holes are fixed. Whether the
joiner, grouping into shells, nesting and the throwaway sweep then accept them
is beyond what this lane could see without a fix.

### 245.7 What this leaves

- Fix hole B: the curved surface goes into an arc's edge name, in `Edges::of`,
  `crates/rcad-topo/src/fuse/assemble.rs`.
- Fix hole A: an input vertex that only splits one crossing is not a corner of
  the answer, in `Assembly::from_the_pieces` and `Corners`,
  `crates/rcad-topo/src/fuse/assemble.rs`.
- Once both are fixed, rewrite
  `two_overlapping_parallel_bars_stop_in_select_whichever_way_round` as six
  fuses that answer, and take off its `#[ignore]`. `ignored_tests` then drops
  by one. `crates/rcad-topo/tests/fusing_two_bars_side_by_side.rs`.
- Fuse the answer of the intersection with `here` as the argument once more,
  and see whether a corner with two curved edges survives it.
  `crates/rcad-topo/tests/fusing_two_bars_side_by_side.rs`.
