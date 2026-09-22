## 204. The pointed cone's box lands, and §200.3's wrong answer is now a refusal — 2026-09-22

*One Opus lane, dispatched by §175's skill: workflow run `wf_62a24930-0a6`, off
`e0b6ef6a`. That base is §202.3's `149fdd66` with `main` (`7bd07cdb`) merged in,
not rebased, because §202.3 cites the sha. `just gate quiet` run here on
`7bd07cdb` before dispatch: PASS, 2259 run, 2259 passed, 6 skipped. The lane's
verifier gated the tip `b058fc6c`: PASS, 2266 run, 2266 passed, 6 skipped, and
saw the four new tests fail with `view.rs` and `fuse/` put back to base.
`batch-200` fast-forwarded to `b058fc6c`. A number below is the lane's unless it
names a command run here.*

### 204.1 A closed edge cut where nothing meets it is joined back whole

The `assemble.rs:992` panic from §202.3 ("a run of edges that became one edge
has two ends left"), seed `cc 9bb0dbcd2316588ce1eba07feb6857499c6ff685e3c2f29b9bcde3bd93de3910`.
A pointed cone stands on its apex on a block, and the walls x = −1 and y = 1
touch the cone's rim at (−1, 0, 2) and (0, 1, 2). The rim was cut there and at
its own corner (1, 0, 2). Joining then folded away all three corners and left
one edge with no ends. The lane showed this by instrumenting the join on the
seed.

The fix (`f108acbe`, `fuse/assemble.rs`): when every piece of such a run comes
from one input edge, the run keeps one corner, the input's own vertex, and
becomes one closed edge again. Each face's walking direction is read from a
piece's arc by the rule `validate` uses. A variant that closed *every* closed
run up failed 22 of 859 `rcad-topo` tests and was dropped.

`fuses_that_touch.rs` gains
`a_pointed_cone_on_its_apex_whose_rim_touches_two_walls_fuses_by_every_rule`,
and the seed is in its `.proptest-regressions`. The cone-on-block cell, when
the two only touch, answers 33 of 90; the control answers 66 of 66.

### 204.2 A cone face's box holds the apex only when the face can reach it

`c709a342`, `view.rs`. The apex goes into the box unless the face's boundary
stays on one side of it *and* the face's trimming test says the apex is off
the face. So a frustum's box stops at its small cap again, and a pointed cone's
box still holds its apex
(`a_frustums_box_stops_at_its_small_cap_and_a_pointed_cones_holds_its_apex`).
`a_block_below_a_frustum` is back from 32 right answers to 56.

The bar wall's pinned box in `a_solid_with_a_curved_face.rs` is re-pinned: exact
in x and y, only a rounding margin in z. The old pin was the loose box the
turning points replaced. The 2^-40 margin is still measured, not derived; it was
the cause of no failure.

**§200.3's pair now refuses by name.** The side-4 cube stood on its corner round
the pointed cone's apex gives 0 right and 8 refused "a point inside a piece of a
face", over both turns, pinned by
`a_corner_up_cube_centred_on_a_pointed_cones_apex_refuses_rather_than_answering_wrong`.
`main` no longer answers it wrong. This is a **gap**, not a limit (§16.10). The
solids share volume, so every rule has an answer to give.

### 204.3 The nappe file's floor was lowered, and that is not a fix

`a_plane_cut_along_a_cones_other_nappe.rs` required more than 10 000 decided
grid points per answer. The two side-6 corner-up cube union families decide
9442, and the fewest over the file is 8879. The tighter box of 204.2 did not
restore them. The lane measured the cause with a debug print that is not in the
diff. The cone face's box reaches z ≈ 17.74, where a cube plane's ellipse turns
far above the arc the face keeps. The solid's box grows to hold that, so the ray
targets move, and 16040 of the 20780 undecided points then refuse on the
frustum's big cap. **The floor is now 8000.** The comment there said the ray
caster "cannot follow" that cap, and was changed here to "did not follow". No
demonstration backs a *cannot*, and the verifier could not reproduce the
figures.

`wildcard_arms` went 97 → 98 (`b058fc6c`). The new test names every
`Containment` it refuses and keeps an arm for the ones not yet made.

### 204.4 What is left

- §200.3's pair refuses "a point inside a piece of a face". Work that refusal
  through to an answer (204.2).
- Put the nappe file's floor back to 10 000. Either the turning-point box learns
  which arc a curved side keeps, or the point test learns to follow the
  frustum's cap outline (204.3).
- The closed-run join handles only runs whose pieces all come from one input
  edge (204.1). No soak at `RCAD_SOAK_FACTOR=100` was run.
- From §202.4, still open: the 9 difference slivers the pole cut lost, in
  `mesh.rs`; whether a closed edge that neither direction rule answers arises
  anywhere but a pole cap; the fuse cutting along a cone's other nappe (§201.5).
- Disk: `/` had 304G free at dispatch (`df -h /`). §202.4's one lane at a time
  is no longer forced.

Commands: `just decision 202.3`, `200.3`, `16.10`.
