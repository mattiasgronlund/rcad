## 217. The fuse writes the rim that names its cutter, one refusal name split in two, and the cone gate measured past — 2026-09-23

§0's first row, closed. Beside it the refusal-name split §216.4 left for whoever
owned the files, which cost three times what §216.4 costed it, and a measurement
lane sent to find out what sits behind the cone-face gate before anyone widens
it. Three lanes off `050aa7ac`, two writing and each verified by a Sonnet reader
that reran every number, one read-only.

`just gate quiet` on the merged tree: **PASS, 2314 tests run, 2314 passed, 6
skipped**, 25 doc tests across 9 crates, read on `afdff061`.

### 217.1 The pairing is not in the edge, and one frame up it is

§216.2 left the question narrow: at `Edge::how_it_runs`, can the answer's own
face pairing be known, or must the caller that already knows the two output
faces say so? **It cannot, and the caller can.** An `Edge` carries `ends`,
`carried_by`, `arc` and `from` and nothing about the answer. One frame up,
`Assembly::how_each_edge_runs` is a method on `Assembly`, whose `faces:
Vec<Face>` carry `Face::surface` and `Face::wires`, and the wires say which two
output faces walk each edge. So the decision moved to the caller: a new
`Assembly::the_surface_that_cut_each_closed_rim` builds that claimant map and
hands `how_it_runs` an `Option<Surface>`.

The fuse now writes `RightRoundWhereItCrosses { and }` for a closed rim whose two
output faces are the one surface and `RightRoundWhereItsFacesCross` everywhere
else, which is what keeps a round bar's cap circle right. Measured on the
frustum-and-block draw: the rim reads `and: Plane(normal [0,0,-1], offset -2)` —
the block's lower cap plane, turned to point out of the block — and the same
draw's bar carries four runs, all of them the old variant.

**A one-line change in `model.rs` is not separable from it.** A fuse's curved
wall comes back with no seam, so every wire holding one of these rims is one
coedge, and `Model::wire_closes_on_one_edge` asked `run ==
EdgeRun::RightRoundWhereItsFacesCross` with a bare `==`. With `assemble.rs`
alone the frustum-and-block union that used to answer comes back
`TheFuseWroteABrokenAnswer { broken: WireTooShort { count: 1 } }` — measured, and
reproduced by the verifier by reverting only that file. It is a `matches!`
naming both closing kinds, and it landed in its own commit so it can be dropped
on its own.

**What cost the lane most was finding a draw that reaches the variant at all**,
and four shapes that do not are worth not re-trying: two coaxial frusta
butt-joined at one circle refuse `Undecidable` with enclosure `inf` at the
shared seam corner; two coaxial frusta overlapping refuse `ResultIsNotManifold {
count: 1 }`, because only coplanar flat faces are joined and two coincident cone
faces are not; a block wholly inside a frustum refuses `CannotMeshAFace {
CoordinateOutOfRange }`; a coaxial bar inside a frustum refuses
`WhereThreeSurfacesMeetNeedsTwoOfThemFlat`. The one that works is
`a_frustum_and_a_block_that_do_not_touch.rs`'s, because the fuse puts the
block's cap plane to the cone face whether or not the solids meet.

### 217.2 §216.4 costed the split at three pins and it was ten, and one of the three was dead

`place_the_piece` raised one name for two different searches, so the fuse wrote
down the name of the search that *worked*. `INSIDE_A_PIECE` keeps the search it
truthfully names — no candidate was inside the piece — and the new
`PLACED_AGAINST_EVERY_SOLID`, *"a point in a piece that every solid could
place"*, names the one that follows.

§216.4 said three pins in two files. **Measured: ten tests across seven files**,
and one of the three it named moves not at all —
`a_narrow_lune_from_a_pole_corner.rs` listed the old string among the refusals
one of its lists allows, and neither name has ever fired for any case there. It
was dead before the split as well as after, and was deleted rather than having a
second dead name added beside it.

The direction was right where it was checked: the census
`how_often_a_fuse_with_a_cone_refuses` still passes with its 30 rows under the
old name, exactly as §216.4 predicted. It was the blast radius that was wrong,
by a factor of three, and this is the second time in two sections that a cost
written into a section without being run came out wrong (§216.5's *"widening
will move nothing"* is the other, and 217.4 is what it cost).

Also unmeasured and still unmeasured: `select.rs`'s 72 fuses / 37.9% at
`InPlane::of` was counted under one name. The comment now says the split has not
been measured rather than guessing it.

### 217.3 Two of the ten are not renames

**A row that refused now answers.** `fuses_that_touch`'s ball of radius two at
`(1, 2, 6)` on a cube of side four was pinned `Geometry(NotFinite)` and comes
back with the two solids the test's own title calls the right answer. Its wall
plane cuts the ball in a circle whose two faces are the one sphere, which is the
shape 217.1's run exists for. **Why that makes this particular coordinate finite
was not chased**, and the row is kept asserting the answer, because a row that
refused once and answers now is the cheapest guard against it refusing again.
The paragraph it carried about `Arrangement::the_whole_surface` described a
refusal that no longer happens and is gone.

**An asymmetry the single name was hiding.** `fusing_two_bars_side_by_side`
reaches both halves of the split, and what it splits on is the **order, not the
rule**: every rule behaves the same way round, and `here` on `there` finds a
point inside the piece and cannot place it against both bars while `there` on
`here` finds none at all. Both still refuse `Undecidable`, so that file's *"a
refusal that depended on which solid was the tool would be a different bug"*
still holds of the refusal; this is one step finer than that sentence reaches,
because the pieces being placed are the argument's faces one way round and the
tool's the other. **Recorded as found and not as diagnosed.** The pin is a map
per rule and order rather than a string, which is §19.1's re-derive rather than
widen.

Three tests were renamed for saying the wrong search out loud, the plainest
being `a_bar_and_a_copy_of_it_..._for_want_of_a_point_inside_a_piece`, which
asserted the opposite of what happens. §37's and §39's tables cite it under the
old name and keep it: they record what was measured when they were written, and
the test's own doc comment is the forwarding address — §210's rule, applied to a
test.

### 217.4 Nothing is behind the cone-face gate, and §216.5's proof is about a different population

§216.5 said to measure the gate's cost past the gate before building any of it,
because the plane arm was widened and moved 0. **Measured, and the warning does
not carry.** Both baselines reproduce exactly: 37 108 cone-face refusals, every
one of them `cuts=[plane, plane, plane]`, and all 12 541 plane-face calls
passing. Widen the cone arm crudely and **all 37 108 become answers and nothing
relocates** — 66 578 entries, 66 578 answers, **0** stops at `where_a_side_lies`'s
`Lies::Across` or at any other site in the function. The grid goes 23 094 → 30 101
decided of 30 101, 0 wrong, volume unchanged.

The reason is structural: the plane arm's blocker was a *straight* side read
against a cone, and a cone face cut by planes never presents that shape — its
sides are arcs, which `where_a_side_lies` already handles against a plane holding
one or both ends.

**§216.5's warning about the far nappe is real and is now a demonstration.** In
the run itself 0 calls land on the far nappe, but only because the answer solid's
box stops at the apex, so no cone point below it is ever asked — the lowest `z`
among the 7 007 non-apex refused points is 0.353. Asked directly through the
public `face_holds_the_point`, the widened gate calls `(0.5, 0, -1)` and
`(1, 0, -2)` **Held** on the far nappe: wrong, not refused. Putting
`is_on_the_nappe_the_face_is_on` in front costs nothing measurable — still 30 101
of 30 101 decided, 0 wrong, volume byte-identical at 14.834958529433301 — and
turns those into `NotHeld` and the 30 101 apex calls from `Held` into
`OnItsBoundary`.

**And the proof sketch does not reach any of it.** §216.5's connectivity
argument covers a cone face cut by *exactly two* planes. Every one of the 37 108
is three, and a partial sweep of the whole `a_plane_cut_along_a_cones_other_nappe`
binary found 133 257 cone-face refusals, every single one three planes and not
one of them two. Three is what a corner-up cube gives: the cone face round the
apex is cut by the three faces meeting at the cube's down corner. **Building only
the family the sketch settles would move 0** — the same outcome as the plane arm,
for a different reason.

So the arm is worth building and the proof is the cost, not the arithmetic: what
must be argued before it lands is that the part of a cone in front of **three or
more** planes is connected. Three planes can in principle leave a strip the
rulings enter and leave twice. 30 101 grid points coming out right is evidence
and not a proof, and §16.10 wants the demonstration or the counterexample. The
nappe guard goes in the same commit as the arm.

### 217.5 What is left, at the other two doors

Two things 217.1 leaves open, both found by the lane and neither blocking:

- **`SolidBuilder::closes_on_one_edge` is the same rule at the other door** and
  still a bare `==` against one variant, so a solid the fuse can now write cannot
  be described again through `SolidBuilder::add_wire`. §44.11 says the two doors
  must not drift and they have. It is one line of the same shape as `model.rs`'s
  — written, then backed out of this merge, because nothing reaches it today and
  a change with no test is not a change this repo lands.
- **`validate.rs`'s `check_edges` does not check the new variant's `and`.** The
  check that both ends are not provably off it sits under `if let
  EdgeRun::PartOfWhereItCrosses { and, picked_by }`, which the new variant does
  not match, so its surface is stored unchecked. Sound for the rims the fuse
  writes — a closed rim's two ends are one corner, on both surfaces of the
  crossing by construction — but nothing in the kernel says so, and an imported
  or hand-written solid can put any plane there.

Those two are one lane: the new variant at the two doors that have not heard of
it, with a test at each.
