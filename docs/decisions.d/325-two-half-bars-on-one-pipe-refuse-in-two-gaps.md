## 325. Two half bars stacked on one pipe refuse in two gaps — 2026-09-28

*Queue row Q116, from §247.5's fourth item and §247.6's last. Lane
`lane-q116-two-half-bars-on-one-pipe`, off `cdd0cb52`. Every fuse of two half
bars stacked on one pipe still refuses `AFaceTrimmedByACurveThisCannotFollow`,
with one spelling of the wall or two. It is raised in `fuse/arrange.rs` now,
not in `holds.rs` as §247.3 said. `AFaceMeetsItsNeighbourOnItsOwnSurface` is
met on the way to the answer by another route, once the wall is spelled twice.
**Both are gaps** (§16.10): the answers exist, and the kernel reaches them
when the block is cut off after the whole bars are fused. No `src/` changed.
`crates/rcad-topo/tests/two_half_bars_stacked_on_one_pipe.rs` pins the
answers and not the refusals.*

### 325.1 The fuses, rebuilt

As §247.3 built them. A bar of radius 2 on `z ∈ [0, 2]` and one on
`z ∈ [2, 4]`, each built face by face so the wall names its axis where the
builder says, and each cut to a half bar as the bar less a block over
`x < 0` that reaches well past it. The lower bar and its block are made
first, so the lower half bar's faces are slots 10 to 13, as in §247.3's
model. A probe test, since rewritten, ran all three rules both ways round,
with the upper wall naming its axis at `z = 0` (one spelling) and at `z = 2`
(two):

```
PROBE fuse spelling 0 Union lower on upper: Err(AFaceTrimmedByACurveThisCannotFollow { face: FaceKey { slot: FaceSlot(10v1), .. } })
PROBE fuse spelling 0 Union upper on lower: Err(AFaceTrimmedByACurveThisCannotFollow { face: FaceKey { slot: FaceSlot(23v1), .. } })
```

and the same two faces for `Intersection` and `Difference`, and for the
second spelling, twelve refusals in all. `FaceSlot(10v1)` is §247.3's line
exactly. It is the lower half bar's bottom half-disc; 23 is the upper one's.
Half bars cut from `Model::add_round_bar` bars give the same six. None of
the twelve gives `AFaceMeetsItsNeighbourOnItsOwnSurface` (325.4 has where
that one is met).

**Where it is raised.** A copy of the base in the scratchpad, with a
backtrace printed at every place that builds the refusal, gave for all
twelve fuses the one in `Arrangement::which_way_an_edge_runs_round`,
`fuse/arrange.rs:2001`, and none in `holds.rs`. §247.3's "raised in
`holds.rs`" was true of some state since then or never measured; either way
it is not true on this base.

`holds.rs` refuses the same half bar too, one layer up.
`SolidView::face_holds_the_point`, asked about a corner of each of the
lower half bar's faces, refuses on the two half-discs (10, 11) and the half
wall (12), and answers on the flat face at `x = 0` (13). `classify_point`
of `(1, 0, 1)`, `(−1, 0, 1)` and `(1, 0.5, 0.5)` all come back
`CannotFollowAFacesTrimming` on face 11; `(1, 0, 3)`, outside the box, comes
back `Outside`. Every one of those refusals was raised at `holds.rs:1131`,
in `the_sides_of`, reached through `what_a_face_cut_by_part_of_a_crossing_holds`.

### 325.2 Why both places refuse

Both read which of a circle's two arcs a side is through a
`the_way_into_an_arc`: `fuse/arrange.rs:3740` for the arrangement, and
`holds.rs:1238` for a face of a solid. Both need the circle's centre, from
`where_a_closed_crossing_is_centred` in `holds.rs`, which has an arm for a
ball and one for a cone. A pipe falls to
`Surface::Plane(_) | Surface::Cylinder(_) | Surface::Torus(_) | _ => None`,
so a half bar's rim arcs, which are halves of the circle a cap cuts the
wall in, have no centre, and both readers answer `None`.

The centre exists. A plane that is not parallel to a pipe's axis cuts it in
a circle or an ellipse centred where the axis crosses that plane, so the
cone arm's form fits with the axis origin as `from` and the axis direction,
turned to read positive against the plane's normal, as `along`.

**Behind it, read and not run.** A fix to the centre alone will not answer
`face_holds_the_point` on these faces. In `holds.rs`,
`which_side_of_its_neighbour_a_side_keeps` refuses a straight side on a
curved face, and the half wall's two rulings along `x = 0` are that.
`is_a_family_this_follows` admits no pipe, neither as the face's surface
nor as a cut on a flat face. The argument that admits a cone's closed curve
on a plane, that its inside is convex, holds for a pipe's circle or
ellipse word for word. Nobody has checked the argument for a pipe face cut
by planes.

### 325.3 Why it is a gap and not a limit

The half bars are `L − B` and `U − B`, so the six fuses are, as sets,

- `(L − B) ∪ (U − B) = (L ∪ U) − B`,
- `(L − B) ∩ (U − B) = (L ∩ U) − B`,
- `(L − B) − (U − B) = (L − U) − B`, and `(U − L) − B` the other way round.

The right-hand sides are fuses the kernel answers. With one spelling, both
orders:

```
PROBE2 spelling 0 Union lower first true: whole bars gave 3 faces, cylinders 1, volume Ok((50.20344246266881, 0.24863623216737799))
PROBE2   then less the block: 4 faces, cylinders 1, volume Ok((25.102553644750728, 0.12097583462323822))
PROBE2 spelling 0 Difference lower first true: whole bars gave 3 faces, cylinders 1, volume Ok((25.101721231334363, 0.12431811608338475))
PROBE2   then less the block: 4 faces, cylinders 1, volume Ok((12.551276822375359, 0.06048791731146799))
```

The intersection of the whole bars answers no solid. `8π = 25.1327…` is
0.030 from the union's half bar, inside its bound of 0.121, and
`4π = 12.5664…` is 0.015 from the difference's, inside 0.060. So every
refused fuse has an answer, a solid this kernel builds, and the arithmetic
is not what stops it. What stops it is that two readers were not taught a
pipe. `cutting_the_block_off_after_fusing_the_whole_bars_answers_what_the_half_bars_should`
pins those answers for all six, and
`a_point_a_half_bar_cannot_place_is_inside_the_bar_and_outside_the_block`
pins the answer to the refused `classify_point` of `(1, 0.5, 0.5)`: the
bar says `Inside` and the block says `Outside`.

### 325.4 `AFaceMeetsItsNeighbourOnItsOwnSurface`: the other route, spelled twice

With the upper wall naming its axis at `z = 2`, the whole bars' union
answered with its wall as two faces (§247.1, and
`spelling_the_wall_of_a_stacked_bar_twice_leaves_the_wall_one_face`, which
pinned two faces at the time). Cutting the block off that refuses, both
orders:

```
PROBE2 spelling 2 Union lower first true: whole bars gave 4 faces, cylinders 2, volume Ok((50.20344246266877, 0.2486362321677122))
PROBE2   then less the block: AFaceMeetsItsNeighbourOnItsOwnSurface { face: FaceKey { slot: FaceSlot(8v1), .. } }
```

That refusal is built in one place, `Arrangement::crossing_for`,
`fuse/arrange.rs:1012`. The backtrace reached it from the loop over a
face's own edges (`arrange.rs:959`), splitting a face in
`Scratch::split_both_groups`. The two surfaces it printed were the two
spellings of the wall, axis origins `z = 0` and `z = 2`, which `intersect`
calls one surface. The difference, spelled twice, answers `4π` the same as
with one spelling. Its whole bars leave one wall face, so nothing meets its
own surface.

**A gap.** The answer is the same half bar as with one spelling, and the
kernel builds that one. This is §39.4's row "two faces on one cylinder with
an edge between them", filed as undemonstrated, now reached from the public
API in two fuses.

**2026-09-29, since Q354 (84538a4e).** The joiner now joins the two spellings
of the wall onto one face instead of leaving one per spelling
(`spelling_the_wall_of_a_stacked_bar_twice_leaves_the_wall_one_face`), so the
union above no longer has two faces meeting on their own surface, and cutting
the block off it answers the same `8π` half bar
(`a_wall_spelled_twice_is_cut_to_the_same_half_bar_as_a_wall_spelled_once` in
`two_half_bars_stacked_on_one_pipe.rs`). The gap this row named is closed.

### 325.5 Rows for a fix lane

1. **A half bar's arcs have no centre.** Smallest reach: two half bars,
   one spelling, `Union` lower on upper, refused at `fuse/arrange.rs:2001`
   on `FaceSlot(10v1)`; the answer is one solid of four faces with volume
   `8π` inside its bound. Smaller still, with no second fuse:
   `classify_point((1, 0.5, 0.5))` on one half bar, refused at
   `holds.rs:1131`, answer `Inside`. The first step is a pipe arm in
   `where_a_closed_crossing_is_centred`. The two `holds.rs` items of 325.2
   come after it. `crates/rcad-topo/src/holds.rs`,
   `crates/rcad-topo/src/fuse/arrange.rs`.
2. **Two spellings of one wall stay two faces, and the next fuse refuses.**
   Smallest reach: the whole bars' union with the wall spelled twice, less the
   block, refused at `fuse/arrange.rs:1012`; the answer is the same `8π`
   half bar. The two fixes on the table: the joiner could join pieces whose
   surfaces `intersect` calls one surface rather than only pieces with the same
   bits (§229.2), or `crossing_for` could let an edge between two spellings
   through. Neither has been tried. `crates/rcad-topo/src/fuse/assemble.rs`,
   `crates/rcad-topo/src/fuse/arrange.rs`.

Once row 1 lands, the half bars reach `assemble` for the first time. Then
they are §247.3's shape for putting an open arc between two spellings of
one wall, and they can test the fold in an arc's name (§247.5's third item).
