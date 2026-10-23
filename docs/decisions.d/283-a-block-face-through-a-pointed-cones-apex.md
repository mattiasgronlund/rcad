## 283. A block face through a pointed cone's apex gets no node where the two rulings cross — 2026-09-26

*Row Q126 (`lane-q126-apex-cells`, off `c82e02f2`). §252.8 found that all six
fuses of a pointed cone and a block whose face `x = 0` runs through the apex
refuse `CannotTellTheCellsOfAFaceApart { face: 8v1 }`. Face `8v1` is the
block's face on `x = 0`, not the cone's skin. The plane cuts the cone in two
rulings that cross at the apex, and `place_the_meets` puts no node there,
because it skips every pair of crossings with the same partner. So two arcs
cross between nodes, the walks round the drawing come out wrong, and
`spread_the_winding` refuses. All six answers exist, so it is a **gap**.
`tests/a_block_face_through_a_cones_apex.rs` pins it. Nothing in `src/` changed.*

  283.1   The shape, and which face `8v1` is
  283.2   What the arrangement drew
  283.3   What the face really holds, and the hypothesis tested
  283.4   A gap: the six answers by hand
  283.5   Where it refuses, and what a fix has to do there
  283.6   §39.4's row
  283.7   Where the numbers come from

### 283.1 The shape, and which face `8v1` is

§252.8 did not say how its cone and block were built, so this lane picked sizes
that can be worked by hand. The cone is `cones::a_pointed_cone(model, 0.5,
8.0)`: apex at the origin, slope one half, cap a disc of radius 4 at `z = 8`.
The block is a 6 × 12 rectangle extruded 12 and moved by `(0, −6, −2)`, so it
is `[0, 6] × [−6, 6] × [−2, 10]`. It holds the cone's `x > 0` half whole, and
the only thing that cuts the cone is the block's face on `x = 0`.

The cone is built first, and it makes faces `1v1` (cap) and `2v1` (skin). The
block's extrusion comes next and makes bottom, top, then one side face per
profile edge, `3v1` to `8v1`. The last profile edge runs from `(0, 12)` to
`(0, 0)`, so **`8v1` is the block's face on the plane `x = 0`**, with normal
`(−1, 0, 0)`. The test checks this: each refusal has to name the block face
whose plane is `x = 0`. All six fuses refuse on it. That is union,
intersection and difference, each with the cone first and with the block
first.

### 283.2 What the arrangement drew

A temporary `eprintln!` in `count_the_windings` and `spread_the_winding`
printed the arrangement of face `8v1` for the first fuse. It was removed
before the commit. Seven crossings are drawn on the face: the four block faces
round it, the cap's plane `z = 8` (crossing 4), and two for the cone. Those two
are **crossing 5, `y = z/2`**, and **crossing 6, `y = −z/2`**. Both have the
cone as partner, and each has its own half-space pickers.

There are sixteen nodes: four block corners, and one where each pair of
crossings with different partners meets. **None of them is at the apex
`(0, 0, 0)`.** Crossing 5 has nodes at `(y, z) = (−6, −12), (−1, −2), (4, 8),
(5, 10), (6, 12)`, and crossing 6 has nodes at `(−6, 12), (−5, 10), (−4, 8),
(1, −2), (6, −12)`. So arc 18, `(−1, −2) → (4, 8)` on crossing 5, and arc 23,
`(−4, 8) → (1, −2)` on crossing 6, both pass through the apex, and they cross
there with no node.

The walks round that drawing are these:

- **Walk 2**, `(1, −2) → (−1, −2) → (4, 8) → (−4, 8) → (1, −2)`. This is a bow
  tie: the triangle inside the cone and the small triangle down in the other
  nappe, joined through the apex.
- **Walk 0** has 36 steps and goes along **both sides** of six of the face's
  own edges: arcs 4, 5, 7, 9, 11 and 12. It is wound once, because it starts on
  the inner side of arc 0.
- Walks 1, 3 and 4 are right: two triangles outside the face, and the strip
  above the cap.

`spread_the_winding` starts at walk 0 and crosses arc 5. That arc is the face's
edge on `y = −6` between `z = 8` and `z = 10`, crossed from inside to outside.
The walk on the other side of it is walk 0 again, which should now be wound
`1 − 1 = 0`. It is already wound 1, so the function returns the refusal from
its "two ways round disagree" arm.

### 283.3 What the face really holds, and the hypothesis tested

On `x = 0`, in `(y, z)`, the face is the rectangle `[−6, 6] × [−2, 10]`. The
lines `y = ±z/2` and `z = 8` cut it into seven regions:

| region | corners | area | in the cone? |
|---|---|---|---|
| A, inside the cone | `(0, 0), (4, 8), (−4, 8)` | 32 | yes |
| B, above the cap between the rulings | `(−4, 8), (4, 8), (5, 10), (−5, 10)` | 18 | no |
| C, down in the other nappe | `(0, 0), (1, −2), (−1, −2)` | 2 | no — the solid has no other nappe |
| D and F, either side below the cap | e.g. `(0, 0), (1, −2), (6, −2), (6, 8), (4, 8)` | 43 each | no |
| E and G, either side above the cap | e.g. `(4, 8), (6, 8), (6, 10), (5, 10)` | 3 each | no |

The total is `32 + 18 + 2 + 2·43 + 2·3 = 144 = 12 · 12`. **A, C, D and F meet
at the apex and touch nowhere else.**

The brief's hypothesis came from the skin. There, the two rulings cut the skin
into two pieces that touch only at the apex, and the hypothesis was that this
touching point is what the arrangement cannot sort out. The refusing face is
the block's flat face, not the skin. On it the point is the same, but four
regions meet there, not two. **The hypothesis holds in this form:** the
arrangement cannot sort out the point where regions touch, **because it has no
node there**. Walk 2 is A and C joined through that point, and walk 0 is D and
F and everything outside joined through the same point. The skin was never
reached. Face `8v1` refuses first in all six fuses, and nothing here shows
what the skin would do.

**Two controls, both pinned.**

- **Move the block to `x = 1`.** Its face misses the apex and cuts the cone in
  a hyperbola. All six fuses answer, one solid each, and the model validates.
  So the refusal is about the apex.
- **Keep the face on `x = 0` and raise the block's floor to `z = 1`.** Now the
  apex is just below the face. All six fuses get past the arrangement and
  refuse a stage later, `Undecidable` on "a point inside a piece of a face".
  The record's corners are `(0, −6, −12)`, `(0, 0.5, 1)`, `(0, −0.5, 1)` and
  `(0, 6, −12)`. These are two stretches, one on each ruling, that cross at the
  apex, and again there is no node there. That the record comes from the same
  missing node is likely, but it is **not shown**: nobody printed the walk the
  piece came from.

### 283.4 A gap: the six answers by hand

The face cuts the cone exactly in half. Half the cone is
`π · 4² · 8 / 6 = 64π/3 ≈ 67.0206`, and the block is `6 · 12 · 12 = 864`.

| fuse | the answer | volume |
|---|---|---|
| union, either order | the block, with the cone's `x < 0` half standing out of its face `x = 0` | `864 + 64π/3 ≈ 931.0206` |
| intersection, either order | the cone's `x > 0` half | `64π/3 ≈ 67.0206` |
| cone less block | the cone's `x < 0` half | `64π/3 ≈ 67.0206` |
| block less cone | the block with a half-cone groove in its face `x = 0` | `864 − 64π/3 ≈ 796.9794` |

Each answer is one solid. Every corner is exact in `f64`: `(0, 0, 0)`,
`(0, ±4, 8)`, `(0, ±5, 10)`, `(0, ±1, −2)` and the block's own corners. Every
curve is a ruling, a half of the rim circle, or a straight edge. The face
`x = 0` of the union and of the block less the cone is the rectangle with
region A taken out. The flat face of the two halves is A itself. So **every
answer exists, and this is a gap** (§16.10). The test's assertion messages
carry these answers.

The raised-floor control is a gap for the same reason. The cone below `z = 1`
is a tip of volume `π/12`, so the block holds `64π/3 − π/24 = 511π/24` of the
cone and is itself `6 · 12 · 9 = 648`. The test's messages carry those
answers too.

### 283.5 Where it refuses, and what a fix has to do there

**The refusal comes from `Arrangement::spread_the_winding`** in
`crates/rcad-topo/src/fuse/arrange.rs`, from its "two ways round disagree" arm.
**The cause is earlier, in `Arrangement::place_the_meets`**, which starts with
`if self.crossings[first].partner == self.crossings[second].partner {
continue; }`. Two curves of one pair never get a node where they cross.
`where_the_ray_crosses`'s doc comment says "every place two crossings meet
already has a node (`place_the_meets`)". That is not true for two crossings
with the same partner, and this shape is the counterexample.

**What a fix has to do, in two parts:**

1. **Put a node where the two rulings cross.** For a plane and a cone, that is
   the apex. It is stored exactly (`Cone::apex`). Whether the plane runs
   through it is already decided: `a_plane_cuts_a_cone_in_rulings` asks for
   exactly that `Zero`. No meet of three surfaces is involved, because only
   two surfaces pass through that point. Both crossings' pickers are
   half-spaces through the apex with offset 0, so `nothing_rules_the_point_out`
   has to keep the apex on both.
2. **Give each ruling a direction out of that node.** `way_out_of` and
   `turns_anticlockwise` take a curve's direction from `∇S × ∇Q`, and
   `gradient_at` for a cone is `|w|²x − (1 + slope²)(x·w)w` with
   `x = at − apex`. **At the apex that is the zero vector.** Four ways out
   with no direction make `sorted_anticlockwise` see `Zero`, and then
   `leaves_the_node_the_same_way` refuses `TwoCurvesTouchWithoutCrossing`. A
   ruling is a straight line, so its direction can be read off the line
   itself — from the apex towards another node on it — and not off the
   gradient. **This part comes from reading the code and was not run.** Part
   1 alone may therefore swap this refusal for `TwoCurvesTouchWithoutCrossing`
   rather than close it.

**The fixes considered:**

- **A point inside each cell**, which is what §39.4's row names. It does not
  help here. The walks themselves are wrong — walk 0 lies on both sides of the
  face's own edges — so there is no proper cell to put a point inside. The
  drawing has to be flat first.
- **Folding the two rulings into one crossing.** A curve that crosses itself
  has no order along it, and `HowItRuns::AlongAnAxis` orders points along one
  line.
- **Splitting each ruling at the apex into two half-lines.** Each half-line
  still has to end somewhere, and that end is the node of the first fix under
  another name.

What is left is the family "a node wherever two arcs meet". That is the
invariant the walk already assumes, and the only question inside it is where
the point comes from. Here it can only come from the pair itself, because no
third surface passes through the point.

Other pairs that name two curves could reach the same skip. One example is a
plane touching a torus on two sides, whose two circles cross twice. **None was
tried.**

The brief said not to fix it: `arrange.rs` is outside this lane's write set,
and Q202 will need that file.

### 283.6 §39.4's row

The row is `CannotTellTheCellsOfAFaceApart`, "two regions of one face were
joined that the surface keeps apart", classified **gap, undemonstrated**, with
the demonstration column saying "telling them apart takes a point inside
each".

- **The word "undemonstrated" can go.** It could have gone at §49.6, which
  wrote "no longer undemonstrated" against a different cause: a rim wire of
  one closed edge, pinned by `a_ball_and_a_bar_in_one_block.rs`'s
  `the_same_three_one_pair_at_a_time_stop_somewhere_else`. §39.4 itself was
  never edited. This section adds a second demonstration, with a second cause,
  in `a_block_face_through_a_cones_apex.rs`.
- **"Telling them apart takes a point inside each" is not the fix for this
  cause** (283.5). What this cause needs is a node where two curves of one pair
  cross. The variant's doc comment in `error.rs` says the same thing about a
  point inside each region.
- The classification stays **gap**.

This lane does not own `039-*.md` or `error.rs`, so neither was edited. The
report lists both.

### 283.7 Where the numbers come from

- The six refusals, and the face they name: `mise exec -- just t -p rcad-topo
  --test merged -E 'test(/^a_block_face_through_a_cones_apex::/)'`, where
  `every_fuse_of_a_block_whose_face_runs_through_the_apex_is_refused_by_name`
  matches `Error::CannotTellTheCellsOfAFaceApart { face }` with
  `face == ` the block's face on `x = 0`.
- The controls: the same command, through
  `the_same_block_one_unit_off_the_apex_answers_every_fuse` and
  `the_same_block_with_the_apex_just_below_its_face_is_refused_a_stage_later`.
  The raised-floor record's corners came from a probe on the base that printed
  the last `Record`.
- The drawing in 283.2: the temporary `eprintln!` in `arrange.rs`, run on the
  base through the same test file. It was removed, and `git diff c82e02f2 --
  crates/rcad-topo/src` is empty.
- The volumes: `python3 -c` over `64π/3`, `864 ± 64π/3`: `67.02064327658225`,
  `931.0206432765823`, `796.9793567234177`. The areas in 283.3 were worked by
  hand and add up to the face's 144.
