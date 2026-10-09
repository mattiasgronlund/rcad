## 202. A ball less a disc meshes, a ball cut through its pole fuses, and the cone's box is right but not landed — 2026-09-22

*One batch, dispatched by §175's skill off `4514eff4`: workflow run
`wf_e20ea19a-0a0`, three Opus lanes. **The disk filled again during it.** The
pole-cut lane's verifier never got a tree, so its check was made here (202.2).
`just gate quiet` run here on the merge `3887322a`: PASS, 2259 run, 2259 passed,
6 skipped. A number below is a lane's unless it names a command run here.*

### 202.1 A ball less a disc is opened along a seam half a turn from its rim

`lane-ball-both-poles-mesh` (`07d1ffcb`, `7b395d1a`) closes the oldest mesh item,
`mesh.rs:839` from §198.3. It was a gap. A ball's skin with one rim and the
material outside it is a ball's pole-to-pole chart rectangle with the rim as an
island in it, which `cut_a_hole_up_to_both_poles` already cuts.
`open_the_ball_along_a_seam` builds that rectangle, with a seam half a turn
from the rim. What it cannot open is still refused
`ACurvedFaceGoesRightRoundWithNoSeam`. On base, 96 of 96 pieces built face by
face and 4 of 4 fuses were refused that way. Now all 100 close with V − E + F = 2
and measure within 3δ/r (`a_piece_of_a_ball_holding_both_poles.rs`). The
verifier saw both new tests fail with `src/` put back.

Two pins in `degenerate_curved_fuses.rs` now answer, and were re-pinned here as
answer tests against closed forms:
`what_is_round_two_crossing_balls_is_their_union` (both balls less the lens) and
`a_ball_cut_off_its_polar_axis_is_answered_under_every_rule`.

The named refusal for a closed edge that neither direction rule answers was
**not** added. The only walks that reach that case are the 63 caps of
`a_cap_whose_rim_runs_through_the_other_pole.rs` (252 walks, the lane's word from a
probe since removed), and every one of them answers. Whether it can matter
anywhere else is open.

### 202.2 A ball cut through its pole gets its corner at the pole

`lane-ball-pole-cut` (`0118f61e`, `bf791bc5`, `239771ba`) closes §200.2's fuse
gap. **§200.2's explanation was wrong:** the fuse drops seams when it flattens
its inputs, so the seam is not the cause. The cause is where the circle got
its one corner, which was wherever the walk happened to start. The fix is in
three parts, all in `fuse/`:

1. A ball-and-plane circle through exactly one pole gets its corner at that pole.
2. The inside-count tries lines toward the middles of sides when the line
   toward the first corner cannot be settled. That was the
   `CannotTellTheCellsOfAFaceApart`.
3. At a pole the edge is stored the way the mesh walks the circle.

The north cap now answers 63 of 63 by intersection and by difference, where base
answered 27 and 45. **The sliver that holds no pole loses 9 answers by
difference that base had** (net 24 → 27). It is refused
`VolumeIsNotPositive` or `TwoPatchesOfTheMeshDrewTheSameSide`. Winning them
back is mesh work: cutting up a cap whose one corner is a pole and which holds
no pole.

Every turned bar and ball fuse the shipped corpus draws now answers, so
`fuses_that_must_agree.rs` pins their demonstrated refusals at zero (bar 6 → 0,
ball 3 → 0). The frustum still has some.

**Scoped to the 128 configurations the fixed seed draws at `RCAD_SOAK_FACTOR`'s
default of 1 (`MEASURED_CASES`), which is what "the shipped corpus" means
above and nothing wider.** A soak past that corpus draws configurations the
128 never do, and §290.4 B found one: a turned bar through a block, case 687
of `RCAD_SOAK_SEED=112 RCAD_SOAK_FACTOR=66`, that refused a union the shipped
128 never drew. It is not a new kind of gap — it is the same turned-fixture
gap this section already names, just at a configuration outside what "now
answers" above was ever measured against.

Checked here, since the verifier never ran: `fuse/arrange.rs` and `assemble.rs`
put back to `4514eff4`, then `just t -p rcad-topo --test
a_ball_cut_through_its_pole` gave 3 run, 3 failed.

`wildcard_arms` went 91 → 97 (`3887322a`) with seven new arms and two removed.
All of them are listed in that commit.

### 202.3 The pointed cone's box is fixed on a branch, and not landed

`lane-cone-box-reaches-apex` (`149fdd66`) found the cause of §200.3's wrong
fuse. The cone's box ran from z = 4 to 12 and the cube's from −3.46 to 3.46, so
`shares_space_with` answered `Apart` and the broad phase saw no pairs. Its fix in
`view.rs`: a cone face's box always holds the apex, and each curved side adds
the `up` values where its crossing can turn. It also showed §16.10's pipe
claim **false**. A bar cut on a slant, with its one corner at the lowest point
of the ellipse, had a box stopping at z = 1 while the ellipse reaches z = 2
(`a_box_holds_every_curved_face.rs`). The whole cone grid now answers 991
inside, 29634 outside, 0 wrong and 0 refused, and §200.3's fuse now refuses
rather than answering wrong.

**It is not merged, because the branch does not gate.** Its verifier reproduced
5 failures:

- `fuses_that_touch::a_pointed_cone_standing_on_its_apex_on_a_block` panics
  at `assemble.rs:992` ("a run of edges that became one edge has two ends
  left"), seed `cc 9bb0dbcd…3910`, shrinking to `AConesApexOnABlock { slope:
  Half, at: [1, 3] }, Touching`. The old box hid this pair from the fuse.
- `a_plane_cut_along_a_cones_other_nappe.rs`, `a_block_below_a_frustum`: right
  answers fall 56 → 32, because a frustum's box now always reaches its apex.
  The two corner-up side-6 cube Union families decide 9442 grid points, under
  the file's 10000.
- `a_solid_with_a_curved_face.rs:476`, the bar wall's pinned loose box.

`main` therefore still gives §200.3's wrong answer.

### 202.4 What is left

- **The cone box, 202.3.** First, because `main` still answers wrong. Continue
  from `lane-cone-box-reaches-apex` by merging `main` into it (not rebasing),
  with `view.rs`, `fuse/` and the five test files in one lane. Include the apex
  only when the face can reach it, fix the `assemble.rs:992` panic, and re-pin.
- The 9 difference slivers the pole cut lost (202.2), in `mesh.rs`.
- Whether a closed edge that neither direction rule answers can arise anywhere
  but a pole cap (202.1).
- The fuse cutting along a cone's other nappe (§201.5).
- **Disk.** It filled twice today with three lanes running. `just clean-targets`
  leaves 30G, with 146G in worktrees and 166G in `~/.cache/kache`. Only Mattias
  can clear the cache. Run one lane at a time until he does.

Commands: `just decision 201`, `200.2`, `200.3`, `198.3`, `16.10`, `40`.
