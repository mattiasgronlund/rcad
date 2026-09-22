## 195. A ball cut down its polar axis answers, the box round the point across is measured, and the `brep.rs` circle allowance is published — 2026-09-22

*One batch of three lanes, dispatched by §175's skill, workflow run
`wf_3a26e43b-424`. `just gate quiet` run here: on `b3701c4c` before the batch,
PASS, 2215 run, 2215 passed, 6 skipped; on the merge `6b18afbe`, PASS, 2221
run, 2221 passed, 6 skipped. A number below is a lane's unless it names a
command run here.*

### 195.1 A wire whose two corners are both poles walks

§194.5's first item was a gap under §16.10, and it is closed.
`lane-pole-to-pole-lune` (`c4c68524`) drops the exclusion in
`across_a_circle_that_starts_at_a_pole` that left a circle uncut when the point
across it is the other pole. With only that change, the fuse refused
`AWireWillNotLieFlatOnItsFace`. The lane's probe showed why: the lune's walk
sets off from one pole and never gets back to it. The other pole's side got a
corner at each end, but the closing side did not, so the walk ended half a turn
from where it began.

`mesh.rs`'s new `arrive_back_at_the_pole_the_walk_set_off_from` writes the
missing corner. It crosses that pole side with the same which-way rule and
`how_far_a_pole_side_runs` the walk uses in the middle. There is no new
tolerance. It is skipped when the first and last steps run along one edge, so
`a_wire_that_will_not_lie_flat_is_refused_rather_than_asserted_against` stays
refused. The lane's first version walked that as a whole ball, and the test
caught it.

`degenerate_curved_fuses.rs`'s refusal test is now
`a_ball_cut_by_a_plane_down_its_polar_axis_is_answered`. It covers both
knives and all three rules, and checks one solid, `Model::validate`, and the
volume against the closed form: under it, within 0.5%. The lane measured the
intersection at 133.667 against 2/3·π·4³ = 134.041. The verifier reverted
`mesh.rs` and `arrange.rs` and saw each of the two refusals come back in turn.

The replaced test took the wildcard ledger's only arm with it, so I moved the
ledger to 91. `fuses_that_must_agree.rs` still carried a note saying the cut
refuses, and I changed it.

Not measured: a lune narrower than half a turn that sets off from a pole corner
now gets that corner written twice. All 2221 tests pass, but no test is aimed
at that case.

### 195.2 The box round the point across holds, 200 times over

`lane-pole-cut-box` (`ed9a8848`) is test only. It replays
`across_a_circle_that_starts_at_a_pole` with the same public calls in the same
order, and finds the true point across in double-double arithmetic. Across
40 000 ball-and-plane pairs from 1e-3 to 1e6, 3 824 reached the box. The worst
was 0.005 of the box's half-width. No rounded point across was certified on
both surfaces, so the box is the path that always runs. §194.5's claim-audit
item is closed.

**It found something else.** 11 of 36 scaled copies of the turned ball-and-block
union refuse, and every one is turned about `x` at a size that is not a power
of two. Nine refuse because a corner of a loop sits where an earlier one does.
Two, at 0.001, refuse `ACurvedFaceGoesRightRoundWithNoSeam`. These are
refusals, not wrong answers: all 25 that answer pass `Model::validate`. The
test sets a floor of 25 answered, so fixing them still passes. The lane did
not find the cause. Its guess is that the point across lands a few ulps from
the other pole, and its own objection to that guess is that radius 1 does the
same and answers. Reproduce with `a_turned_ball_and_block_united(3.0, [1, 0,
0], 16)`. This is §0's next item.

### 195.3 The `brep.rs` circle check is a published allowance, not a sign

§190.6 carried "the circle check in `brep.rs` is a plain-f64 tolerance".
`lane-step-circle-check` (`cc3746d1`) measured that a certified sign cannot do
this job, so this is a limit, and the allowance stays. The files place these
circles off their faces in exact arithmetic. In `LED_D3.0mm_Horizontal_O1.27mm_Z10.0mm`
the body circle misses its pipe's axis by 2.1e-16. The new test,
`a_circle_the_file_puts_on_a_pipe_is_off_it_in_exact_arithmetic.rs`, shows
that exactly. Set to 0, the allowance imports 115 KiCad files, not 160. Set to
1, it imports 158. Every value tried from 4 to 1000 imports 160.

The 16 is now `A_CIRCLE_MAY_MISS_ITS_FACE_BY`, and its doc comment carries
those figures. The worst circle in any file that imports is 1.20 ×
eps·largest, on the two `Relay_3PST_COTO` files, and a unit test pins that
circle. The corpus-wide figures came from a temporary probe that the lane
removed. The verifier did not re-run them. It confirmed the relay circle, the
160/323 count and the gate. The comparison itself is still plain f64, which is
a few ulps against 13× room.

### 195.4 What is left

- **The turned ball-and-block at sizes that are not a power of two**, §195.2.
- From §194.5: a named refusal in `mesh.rs` for a closed edge that neither
  direction rule answers; `AClosedEdgeOfTheAnswerWouldStartAtAPole` for a pole
  start against a ball, pipe, cone or torus, and §39.4's row for it; how often
  §194.3's move-onto-the-pole fires.
- A lune narrower than half a turn from a pole corner, §195.1.
- Carried over from §193.3: the slit cut's limits, a closed run carrying `and`,
  `builder.rs`'s two-step hole, `squared` onto `Interval`, the turned-bend band
  and `fuse::bake`'s corner test.

Commands: `just decision 194.5`, `16.10`, `190.2`, `15.6`.
