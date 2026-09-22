## 197. A closed crossing is walked round, the two-knife panic is a refusal, and the turned ball-and-block fails at random on `main` — 2026-09-22

*Two batches, dispatched by §175's skill off `b91a8d12`: workflow runs
`wf_8eb1cce4-3c7` (one fix lane, two diagnosis lanes) and `wf_1b8e51f9-2ca`
(two fix lanes). `just gate quiet` run here: on `b91a8d12` before the batches,
**FAIL** — 2226 run, 2225 passed, 1 failed, 7 skipped, the failure being
197.1's; on the merge `b5f1ddfb`, PASS, 2230 run, 2230 passed, 6 skipped. A
number below is a lane's unless it names a command run here.*

### 197.1 The turned ball-and-block fails at random on `main`, and the green on `b5f1ddfb` does not cover it

`turning_a_ball_and_block_together_changes_no_answer_s_volume`
(`fuses_that_must_agree.rs`) draws 48 fresh cases a run. On `b91a8d12`, which
had gated green at 06:42 the same morning, it drew
`BallAndBlock { side: 4, radius: 1, at: [2, 2] }` turned 16 steps about
`(-1, -3, 0)` (seed `7d4e1f9a`). The fuse answered, and `measure` refused the
answer with `ACurvedFaceGoesRightRoundWithNoSeam`. `just t` on the persisted
seed replayed it: 1 failed, in 0.27 s. Two more seeds of the same shape turned
up in the lanes: `a8b648d9`, turned about `(3, -2, 0)`, and one turned about
`(1, 3, 0)`. The second verifier reproduced the `(1, 3, 0)` failure byte for byte
with the fuse change reverted, so it is on `main` and not a lane's.

So the green on `b5f1ddfb` is a run that happened not to draw one of these
cases. It is not evidence that the family is sound, and none of the three
seeds is committed on `main`: committing one would pin the gate red.

What is known of the cause, all from `mesh.rs`, all from the lanes' printed
probes:

- The fuse's plane through the ball's centre is rounded, so the exact circle
  misses both poles by a rounding: both poles are strictly `Neg` of the plane
  by the exact predicates, for both seeds. `put_in_a_pole` leaves out a pole
  the circle misses, and the walk then jumps about half a turn at it.
- `WalkingAWire::finish` (through `how_far_the_longitude_climbed`) and
  `runs_up` add the corners up **the short way round**. A pole-side step of
  exactly `±π` folds to the wrong sign (`round()` rounds away from zero), so a
  lune reads as a cap wound right round. The doc comment claiming no step is
  more than a quarter turn is false for pole-side steps.
- `lane-ball-block-turned-16` (`2d1e5077`) put a pole in when the circle's
  nearest point reads as the pole, and decided "right round" from the walk's
  own longitude. That fixes `7d4e1f9a`, but turns `a8b648d9` from a named
  refusal into `FacesDoNotAgreeWhichWayTheyFace`: there only the south pole
  reads as a point of the circle, the walk passes the north pole in two
  ordinary steps, and `cut_a_cap_up_to_its_pole` cuts toward the wrong pole
  and leaves four one-triangle patches that are all dropped.
- Tried and reverted, and not to come back: a "reach" rule putting a pole in
  within `POINT_ALONG_A_CROSSING_ROUNDOFF`; falling back to the short-way sum
  past a quarter turn; `runs_up` alone reading the walk's winding. Each broke
  `the_point_across_a_circle_cut_at_a_pole_is_in_its_box.rs`'s `[0, 1, 0]`
  sweep, and nobody found out why.

It is a gap under §16.10: the answer exists. The lanes' proposed direction is
to decide the winding from the exact side of each pole against the plane,
which the predicates already certify, rather than from `atan2` widths. It is a
proposal, not a finding. `lane-ball-block-turned-16` (`45cc016c`) and
`lane-ball-block-winding` (`27eef507`) are left unmerged, both red on their own
gates. The second carries fixes that belong with the family's fix, not before
it: the skew-pair "not a solid" floor, which the regressions file's length
moves (435 fuses with one more seed fails 0.23 %), turned into a count of one;
and `(135, 1)` coming off the lune's refused-by-the-mesher list.

### 197.2 An edge on a closed crossing covers the arc its own tangent points into

§196.2's panic at `fuse/arrange.rs:1743` is gone. It was not what §196.2
guessed. `lane-two-knife-panic-cause` showed that the edges that clash are the
half ball's own two pole-to-pole edges on the `x = 0` circle. The second knife
is not involved: a block `x ≤ 0.5` or `z ≤ 0.5` panics too.
`arcs_an_edge_covers` gave any edge with two different ends the stretch
`min..max`, which on a closed crossing is right for one of the two edges and
wrong for the other. The assert was right.

`lane-closed-crossing-wrap` (`2daaf8e0`, `9b81105c`, `83eaa590`) walks round the
crossing modulo its node count, and takes the direction from the edge itself.
`which_way_an_edge_runs_round` is the sign of the crossing's forward tangent at
the edge's start, dotted into the arc the edge's picker names (the reading
`holds.rs`'s `the_way_into_an_arc` makes). It goes through `must_decide`, so it
is a decision and not a guess. `BakedEdge` now carries its pickers. The lane
chose a tangent over a point in the middle of the edge, because an edge stores
no curve (§22) to read one off. `material_is_left_of_forward` cannot tell the
two directions apart here: it defaults to forward when every corner sits on
the partner. The lane showed that with printed probes: an intersection half
ball read forward/forward and a union half ball backward/backward, on the
same plane.

Any other closed crossing whose edge has two different ends is now refused as
`AFaceTrimmedByACurveThisCannotFollow`, where it used to take the `from < to`
guess. That guess answered wrong without failing whenever an edge covered the
last step round. No existing test reaches the new refusal.

Three new tests in `a_half_ball_cut_again_away_from_its_poles.rs` panic on the
base, which the verifier confirmed by reverting `fuse/`. They are now pinned
refusals: `CannotTellTheCellsOfAFaceApart` twice and `ResultIsNotManifold`
once. Each carries its closed form and fails, asking to be turned into an
answer test, the day it is answered. The two-knife wedge is now a refusal pin
too: 36 "a point inside a piece of a face" at 90, 10 and 170 degrees, and 12
"order along a crossing" at 45. `ignored_tests` goes from 7 to 6.

### 197.3 The off-seam half ball: two gaps, both in `mesh.rs`

`lane-off-seam-cut-cause` corrected §196.2's last bullet. The fuse's refusals
at (45°, r = 1, 1000, 3) and (170°, r = 1) are not raised at `arrange.rs:905`.
They come from `mesh.rs:839`, which the fuse reaches through
`nest_the_shells` → the coarse validation mesh. The turned knife's stored
plane does not hold the axis: `Plane::transformed` keeps `offset + n'·t'`,
which is a rounding (+8.88e-16 at 45°, r = 1). When that is positive, the kept
piece of the sphere holds both poles, and the mesher cannot yet build "the
whole ball less a disc". The size only decides the sign of that rounding. The
(135°, r = 1) refusal at `mesh.rs:857` is the exactly-π fold of 197.1. Both are
gaps. The knife's doc comment now says what the stored plane does, with the
offsets `lane-ball-block-winding` measured.

§196.1's question about the two frames was checked in every case here, and
the two frames were not the cause in any of them: the meshing frame is a pure
translation, and 0 of 84 sphere faces meshed on a different surface. That is
not a proof that they always agree.

### 197.4 What is left

- **The turned ball-and-block family, 197.1**, first: it makes the gate red at
  random. The fix is in `mesh.rs` and must keep the `[0, 1, 0]` sweep green.
  Take `lane-ball-block-winding`'s test commits with it.
- `mesh.rs:839`: build the piece of a ball that holds both poles, 197.3. This
  also writes `mesh.rs`, so it runs after the item above, not beside it.
- The half ball cut again: the three pinned refusals of 197.2, starting at
  `group_the_cycles_into_cells`.
- Still open from §196.4: the narrow lune, and the named refusal in `mesh.rs`
  for a closed edge that neither direction rule answers.

Commands: `just decision 196.1`, `196.2`, `16.10`, `45.2`.
