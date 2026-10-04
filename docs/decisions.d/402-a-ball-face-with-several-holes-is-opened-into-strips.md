## 402. A ball face with several holes is opened into strips, and a rim of several arcs is opened like a round one — 2026-10-04

*Queue row Q451, with Q443 folded in. Lane
`lane-q451-a-ball-face-with-several-holes`, off `a9a3835f`.
`BuildingAMesh::cut_off_what_goes_too_far_round` in
`crates/rcad-topo/src/mesh.rs` opened a ball's skin along a seam (§202.1) only
when the skin had one wire, that wire was one closed curve, and the material
was outside it. **Two shapes fell past that and were filled inside out: one
wire of several arcs with the material outside, and several wires that are all
holes.** Both are now cut, and what still cannot be cut is refused by name.*

### 402.1 What reached it on the base, measured

Q456 (§400) took Q316's balls clear of a block away from the mesher, so the
row's own draws no longer reach this site. Q443's two turned pairs do. Each is
a cube of side 2 turned one radian about an axis through the origin and a ball
of radius 1 at a point in the cube's frame, fused under each rule in both
orders under `Policy::certified()`. A throwaway probe on the base measured:

| pair | union | intersection | cube less ball | ball less cube |
|------|-------|--------------|----------------|----------------|
| about `(1,1,1)`, ball at `(0.62, 0.71, 0.66)` | answers | `Invalid(FacesDoNotAgreeWhichWayTheyFace)` | `Invalid(FacesDoNotAgreeWhichWayTheyFace)` | answers |
| about `(2,−1,1)`, ball at `(0.8, 0.7, 0.65)` | answers | `TwoPatchesOfTheMeshDrewTheSameSide { drawn: 2, used: 4 }` | `Invalid(FacesDoNotAgreeWhichWayTheyFace)` | answers |

Each refusal was the same in both argument orders. The answers measured
within their own bounds of the closed forms in 402.5.

**The first pair's skin inside the cube is two faces, not one.** The three
caps the cube's faces cut off overlap two at a time and never all three, so
they leave a small triangle of skin pointing at the corner, besides the rest of
the ball. The rest holds both poles, and its one wire is three long arcs.
`it_passes_each_of_its_points_once` was the missing test: three arcs are not
one closed curve, so the skin fell through to `cut_each_narrow_enough`, which
fills the inside of the loop.

**The second pair's skin inside the cube has two holes**: a round one where
`x = 0` cuts the ball, and one of two arcs where `y = 0` and `z = 0` do, since
the ball crosses only the edge `y = z = 0`. That is Q451's shape. The first
hole was handed over as the outer loop.

### 402.2 Where a face with several holes comes from

The row guessed a ball pierced right through by a bar of square section. It
does not leave one: the union's skin came back as eight faces, each with one
wire. **Every plane that crosses the ball cuts the skin along its whole
circle**, inside the bar or not, so the bar's four planes cut the skin into
pieces between them. A face with several holes needs crossings that do not
meet: a slab across the ball, whose two planes cut two circles that never
touch; small balls that each cut the big one in one circle; or Q443's second
pair, where the circle of `x = 0` meets neither of the other two.

### 402.3 The cut

**One wire of several arcs**: the seam rectangle of §202.1, with the wire as
its island. Any wire that passes each of its points once is an island the same
way; a wire through a pole, along a seam or with a slit passes a point twice
and goes where it went before.

**Several holes**: `open_the_ball_into_strips` puts a meridian in every stretch
of longitude no hole reaches, at its middle, the first of them in the widest
stretch. Each strip runs from pole to pole and holds one run of holes whose
longitudes overlap. Each meridian's points are made once and shared by the two
strips beside it, and every strip shares the two poles. Then
`cut_holes_up_to_both_poles` cuts each strip in one of two ways:

- **One above the other**, when each hole's lowest corner is strictly above the
  next hole's highest. A cut runs from the top pole down to the highest hole,
  from each hole to the next, and from the lowest hole down to the bottom pole.
  A cut between two holes runs strictly between their latitudes, where no hole
  reaches, and stays in the strip because a strip holds every straight line
  between two of its points.
- **Side by side** (`cut_holes_side_by_side_up_to_both_poles`), when every
  hole's cut straight up from its highest corner and straight down from its
  lowest meets a pole side before any other loop. Cuts along different
  longitudes never cross, and no cut crosses a hole, so the holes come in one
  order along the top pole and the bottom one. A strip where they do not is
  refused.

**One hole is cut exactly as before.** Its strip is the §202.1 rectangle, built
by the same function, and the one-hole cut is the old body: the same corners,
the same two cuts made in the same order. Every piece still has to enclose
some of the chart the way the face's material does, or the face is refused.

A cut between two holes carries at least one point of its own, since its two
ends are corners of the model and a face across both holes could draw the same
pair (`cut_in_two`'s argument).

### 402.4 What is still refused, and why it is a gap

A strip whose holes neither lie one above the other nor clear each other's
straight cuts is refused `CurvedFaceWithAHoleGoesTooFarRound`.
`two_holes_each_in_the_way_of_the_others_cut_are_refused_by_name` builds one:
two small balls, one at longitude `0` and latitude `−0.4` and the other at
`0.42` and `0.4`, whose holes reach the same latitudes by a few hundredths
while each lies across the other's cut. **That is a gap and not a limit**: the
skin exists, and a cut that ends on the other hole, a vertical decomposition,
would reach it. The family tried here is cuts along one longitude from a hole
to a pole, plus straight cuts between holes stacked by latitude. That family
cannot cut it, which is all the refusal claims.

The overlap is not rare once it happens. Q443's second pair has two strips at
the fuse's own coarse deflection and one strip of two overlapping holes at a
measuring deflection of `0.01`, because a finer walk finds more of each
curve's longitude. Both are cut, one way and then the other.

### 402.5 What moved, and the tests

- `tests/a_ball_face_holding_both_poles_inside_three_arcs.rs`: Q443's first
  pair under every rule, in both orders, against the ball less three caps plus
  the three pieces beyond two faces. That piece is
  `2/3·atan(s/ab) + 2/3·abs − (a − a³/3)·acos(b/√(1−a²)) − (b − b³/3)·acos(a/√(1−b²))`
  with `s = √(1 − a² − b²)`, checked against its slices summed one by one. A
  second test pins the two faces of three arcs each.
- `tests/a_ball_face_with_several_holes.rs`: Q443's second pair; a slab across
  a ball (two strips); two small balls one above the other (one strip, cut as
  a stack); two small balls side by side (one strip, cut side by side); and the
  refusal of 402.4. Each answer is held to its closed form under every rule and
  both orders, and the answer under one named rule is checked to hold a face
  on the ball with two holes. Throwaway prints confirmed each test reaches the
  path its name says.
- **Four of the draws of
  `tests/a_ball_shaved_by_a_turned_blocks_face.rs` now answer**: off `−0.3`
  turned `2` and off `0.3` turned `0.5`, Q669's, pinned
  `AHoleGoesTooFarRound`, and off `0.3` and off `0.5`, both turned `5`,
  Q670's, pinned `FacesDoNotAgreeWhichWayTheyFace`. Each has a face on the
  ball with two holes, so Q669's question gets its answer: these draws are
  this site. A probe measured each in both orders and under both policies.
  Each validates. The first is one solid of `12.168` within `0.127` of
  `8 + 4π/3 = 12.189`. The other three are the block at `8` and a ball of
  `4.168` or `4.169`, within `0.127` of `4π/3`. The pin was outside this
  lane's write set and was left red.

  *Q904, which landed this branch, moved it:*
  `what_each_ball_shaved_by_a_turned_blocks_face_comes_to` now holds each of
  the four to answering, checked to validate, counted in solids and measured
  within its error of `8 + 4π/3` under both policies and both orders. Its
  trace of the base found which cut declined for Q669's two and which faces
  disagreed for Q670's two, and that the strips are what answers both. §346.6
  has it.

The single-hole path is unchanged bit for bit, and the slow tests that cover it
(`a_ball_less_a_cap_built_face_by_face_meshes_closed_and_measures`,
`a_ball_cut_through_its_south_pole_keeps_its_north_pole_and_measures_a_cap`)
pass, with the rest of the whole suite but that one pin.

### 402.6 What this leaves

- The pin of Q669's four draws in `a_ball_shaved_by_a_turned_blocks_face.rs`
  has to move to answering, checked against the block's `8` and the ball's
  `4π/3`, with that file's module doc ("None answers yet"). Q669's row holds
  the file, and the remaining draws are its own. *Q904: done, in 402.5; the
  ten draws left are Q668's.*
- §400.5's chain of three whose upright rim left "a band of two loops,
  `CurvedFaceWithAHoleGoesTooFarRound`, Q451's site" was not run again here.
- `crates/rcad-topo/src/error.rs`: the doc of
  `CurvedFaceWithAHoleGoesTooFarRound` says a hole that does not go right round
  has nothing to cut across to. That was already untrue of the half ball cut
  again, and is now untrue of a ball's skin with only holes. The doc of
  `ACurvedFaceGoesRightRoundWithNoSeam` speaks of one curve; a rim of several
  arcs is answered now too.
- The gap of 402.4: a cut from a hole that ends on another hole.
