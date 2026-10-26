## 212. A ring of arcs may bound a face that also has a hole — 2026-09-23

§208.4's last paragraph asked *"which of the two pieces the ring cuts the surface
into does the hole sit in"* and left it as nobody's answer. It has one, so it was
a **gap** and it is closed. §208.6's fifth bullet with it.

### 212.1 There is no choice to make

§208.4 taught `mesh.rs` to walk a closed ring of arcs round a cone's apex, but
only when that ring was its face's **one** wire. The narrowing was put in because
the first attempt let through a face bounded by such a ring that *also* has a
hole, and `a_corner_up_cube_round_a_frustums_apex_answers_right_or_refuses`
caught it: the fuse answered and the measure then came back
`BoundaryCrossesItself` where the face used to be refused outright.

The open question assumed a choice. There is none. **Two wires that each go right
round bound the band between them and nothing else**: the two loops cut the chart
into three stretches, and only the middle one has both of them in its boundary.
So the walk now lets any seamless wire come back a whole turn on, and `cut_up`
refuses — under the same name, `AWireWillNotLieFlatOnItsFace` — only a face with
one wire that winds beside one that does not.

§208.4's own precedent cuts this way: a ring of arcs with no seam was called
unwalkable until it was walked, and nothing in the argument for refusing it had
been cone-specific.

### 212.2 Letting the ring in exposed a second defect, already on `main`

`cut_a_ring_across` takes its four quarters off the outer loop's first corner and
matches each to the inner loop's nearest corner. But the two loops start wherever
the pairs that cut them started, so a quarter could fall off the end of the inner
loop and **the same end corner answered twice**. Each quarter is now brought into
the inner loop's own turn first, and the cut's far end is put at whichever of its
longitudes is nearest the corner the cut leaves from.

This was not only the new case. **A bar through a block was being cut that way
too** — four quarters landing on inner corners `[1, 2, 3, 3]`, so the ring became
one patch with no width along the bottom and one spanning half the turn. Three
rings measured, before and after: `[1,2,3,3] → [1,2,3,0]`,
`[8,12,16,16] → [8,12,16,3]`, `[0,1,5,9] → [14,1,5,9]`.

That is why this section owns a repin in a file it does not own; 212.4 has it.

### 212.3 What it bought

`a_corner_up_cube_round_a_frustums_apex_answers_right_or_refuses` reads
`Tally { right: 48, refused: {} }` against its written `right: 24` and 24
`AWireWillNotLieFlatOnItsFace`. The intersection's cone face there is the
frustum's **band**: ringed by arcs round the apex and holed by the frustum's own
near cap, which is exactly the ring-plus-hole shape.

`tally` panics on a wrong answer, so 48 right is 48 *validated* — each answer's
solids counted, its volume held against the column-sum truth inside the measured
bound, and over 9 300 grid points classified. A refusal became a correct answer,
which is §19.1's good direction, so the test is renamed `..._answers_right` on
the precedent its own sibling set in §208.4, and `AWireWillNotLieFlatOnItsFace`
leaves that file's `REFUSALS`. Dropping it makes the file **stricter**, because
`tally` panics on a refusal no family expects.

A new file, `a_ring_round_an_apex_on_a_face_that_also_has_a_hole.rs`, pins three
tests that all refuse on the base with `AWireWillNotLieFlatOnItsFace`. Its shapes
are all frustums on purpose: §208.6's first bullet, `measure.rs`'s cone-tip
bound, was not attempted, and a frustum keeps its apex off every face so its
bounds stay finite and a measured bound is worth asserting.

### 212.4 The repin this forced, and its argument

`what_a_deflection_costs_a_measured_volume.rs`'s
`a_curved_pair_obeys_inclusion_and_exclusion_inside_the_error_the_four_declare`
pinned the share of the declared tolerance the law uses at under a hundredth, and
at a deflection of `0.1` it now reads `0.0276`. The **first** assert — `out_by`
inside the four declared bounds — still passes; it is the share that moved.

That comment said of itself: *"it is a pin on this pair's arithmetic and it will
move if the mesher does"*, and 212.2 is the mesher moving. The pin is now a
thirtieth. All four deflections were re-derived on the merged tree rather than
taken from the branch: `0.0276`, `0.0103`, `2e-12`, `0.0104`, so `0.03` covers
them and the worst leaves a factor of about thirty-five.

### 212.5 The gate, and what the lane's own gate said

**The lane's branch did not gate green, and its verifier returned `merge`
anyway** — with `gate: FAIL` copied verbatim into its own verdict. A red gate is
not authorisation to merge whatever the verdict field says, and the workflow's
cross-examination does not check the gate, so nothing flagged it. The two
failures were precisely the two readings above, both outside the lane's write
set, both reported by the lane. They are repinned at merge by the dispatcher,
which is where a write-set casualty belongs.

The merged tree of §211–§214, gated whole: **PASS, 2299 tests run, 2299 passed,
6 skipped**, 25 doc tests across 9 crates.

### 212.6 What is still refused, and it is a gap

A face with **one** wire that goes right round beside one that does not. The
answer exists — the winding loop cuts the chart in two and the other wire lies in
one of the halves, and on a cone whose second coordinate starts at the apex only
the apex-side half is bounded, so it is determinate there. What stops it is that
`cut_a_cap_up_to_its_pole` hands back four patches with one loop each and has
nowhere to put an island.

So it is a **gap**, and §209.5's warning applies to whoever writes it down as a
limit: what refuses it is a shape of the code, not the absence of an answer. It
is refused by name in `cut_up` and demonstrated by `mesh.rs`'s own
`a_curved_face_with_a_hole_is_refused_rather_than_asserted_against`. No test in
the tree reaches it except that one, which builds the shape through
`Deserialize`-shaped arena surgery, and nothing in this batch needs it.

**2026-10-04, Q923: a fuse reaches it on a ball.** Since Q453 the cube of side
2 and the unit ball at `(0.8, 0.7, 0.65)` (§400.5) puts a ball's cap, cut up to
its pole, beside an island — no arena surgery needed. On a ball the shape now
answers: an island no cut meets goes whole into its patch as a hole, and a cut
whose meridian meets one goes through it, each patch beside it taking half the
island (pinned in `a_ball_face_ringing_its_pole_round_a_round_hole.rs` and
`a_block_poking_into_a_ball_whose_top_plane_cuts_the_rest.rs`). **Since Q921,
it answers too when the island is large enough that two of the rim's cuts up
to the pole land on it**: the strip between two such cuts is split into two
patches, one under the island and one over it
(`cuts_through_one_island` in `mesh.rs`), and
`an_island_two_cuts_meet_is_refused_by_name` is renamed
`an_island_two_cuts_meet_measures_under_every_rule`
(`a_ball_face_ringing_its_pole_round_a_round_hole.rs`).

**Since Q2811, a solid reaches it on a cone too.** A pointed cone fused with a
ball poking out of its side, with a bar leaning through it, or with a pin
pushed through it square to its axis each leave the cone's face a cap round
its apex with the crossing loop as an island in it — exactly this section's
shape — and every fuse of all three answers under every rule
(`a_tapered_shaft_with_a_cross_hole.rs`).
