## 253. A meet folds into a corner only as far as rounding reaches — 2026-09-26

*Queue row Q109, from §248.6. Lane `lane-q109-corner-fold`, off `f39142d5`.
The question was whether `place_the_meets` may fold a meet of three surfaces
into a corner of the face without asking the policy, under `Policy::certified`,
when the arithmetic proves that corner is off one of the three surfaces.
**Sound, within the allowance a stored corner already has, and now held to
it.** Every certified fold in the workspace lies within a quarter of the
allowance `bake.rs` grants a stored corner. The rule did not require that,
and one tolerant corpus draw folded a meet whose box reached 2 012 times that
far. The fold now requires it. Past the allowance the meet stays a node, and
the policy is asked about it. **That draw's old answer was right** (253.7):
the point in the box is `4.1e-16` from the corner, and the box is wide only
because two of its planes are nearly parallel. So the one thing this change
moves in the workspace is a right answer that is now refused, a gap left for
a row.*

  253.1   The legs, and the one that broke
  253.2   What the workspace folds, measured
  253.3   The fold that reached past the allowance
  253.4   The rule now, and why it is not a new tolerance
  253.5   What H2 needed and did not get
  253.6   What this leaves
  253.7   What the verifier found: the old answer was right

### 253.1 The legs, and the one that broke

The brief's H1 had three legs. (a): a corner is written down rounded, so it
can sit ulps off its surfaces, and §75.6 says a solid `validate` accepts must
bake. (b): `derive_a_corner` already decided that this corner and these
surfaces agree within `what_writing_the_corner_down_cost`. (c): so the fold
reuses that decision, *provided* the meet lies within that allowance of the
corner.

**Leg (b) is false for every one of the 224 drops §248.6 counted.** A probe
patch logged how many distinct surfaces `place_a_corner` saw round each folded
corner. All 224 have two: the floor or wall plane, and the one cone face that
the first union joined back together round its seam. With fewer than three
surfaces, `place_a_corner` reads the stored corner out of the model and never
calls `derive_a_corner`. Nothing in `bake.rs` compares these corners with a
surface. What vouches for them is `Model::validate` on the answer of the
earlier fuse, which reads with the model-wide allowance.

**Leg (c) held in every fold measured, but nothing required it.** The fold
was taken whenever the meet's box touched the corner's. A meet's box can be
arbitrarily wide, and 253.3 shows one that was.

So what makes the fold sound is not a decision `bake.rs` already made. It is
the size of the move. A fold within `what_writing_the_corner_down_cost` of
the corner moves a node no further than `the_place_the_model_points_at`
already lets a meet stand in for a stored corner. It uses the same number from
the same function. Anything further is a decision.

### 253.2 What the workspace folds, measured

A probe patch on `f39142d5`, reverted before the change was written, logged
every fold `a_corner_is_this_place` allowed. For each fold it recorded the
policy, whether the meet's box was bit for bit the corner's, and `reach`: the
largest distance on any axis between a point of the meet's box and a point of
the corner's box. That bounds how far apart the two true points can be. It
also recorded `room`, which is `what_writing_the_corner_down_cost` of the
corner. `RCAD_Q109_PROBE=… just t --workspace`:

```
     Summary [ 109.935s] 2477 tests run: 2477 passed (3 slow), 12 skipped
identical certified=true   count=1131306  max reach/room=0.24349  outside=0
overlap   certified=true   count=83528    max reach/room=0.156249 outside=0
identical certified=false  count=40991    max reach/room=0.154    outside=0
overlap   certified=false  count=4404     max reach/room=2012.67  outside=6
```

- **No certified fold reaches past a quarter of the allowance.** That covers
  1 214 834 folds.
- **The 224 folds into a corner proved off a surface** are exactly §248.6's
  count, from the same five tests. Their largest reach is `8.9e-16`, which is
  `0.0069` of their `room` of `5.7e-14` (a probe of the five tests alone).
  Their meets reach one to four ulps from the corner. In every one of them,
  the surface the corner is proved off is the cone. The other two surfaces
  answer `Zero` exactly.
- **The six lines outside the allowance are one fold of one draw.** It is
  logged once for each of two corner nodes with one box, and the draw is
  fused three times: twice by the provenance door test, which walks each draw
  twice, and once by `every_seed_in_the_checked_in_corpus_is_fused_without_panicking`.
  All three runs are tolerant, because that harness hands a fuse
  `Policy::tolerant` unless the draw asks otherwise.

### 253.3 The fold that reached past the allowance

The draw is `fuzz/corpus/two_solids_fused_by_a_stranger/a45aa71b…`. The face
and its two partners are three planes. The corner's box is
`[4.999999999999997, 5.000000000000003] × [−3.1e-21, 2.1e-21] × [−8.9e-16,
2.2e-15]`, and the meet's box is `2.9e-10` wide in `y`. `reach` is `1.4e-10`
against a `room` of `7.1e-14`. The corner reached the second crossing by
`passes_through`. There the tolerant policy resolved an `Undecidable` from
`side_of_surface` as on the surface. Then the fold identified the two points
without asking anything.

Under the old rule the harness reports `ok: 1, solids_made: 2`. Under the new
rule the meet stays a node, and the same draw reports `ok: 0, refused: 1`.
That is `a_meet_further_from_a_corner_than_rounding_reaches_is_not_folded_into_it`
in `tests/a_meet_folds_into_a_corner_only_as_far_as_rounding_reaches.rs`. It
fails without the change with `ok: 1, refused: 0`: the rule was switched back
behind an environment variable in a probe build, and the test was run.

**Which predicate the refusal names was not read here.** The harness keeps a
refusal's records inside the `Error` and hands back only the tally. The
verifier read it: `Undecidable` on "order along a crossing", with an enclosure
of `2.86e-10` against a tolerant budget of `1.42e-12` (253.7). **The sentence
that stood here said the old answer rested on identifying two points `1.4e-10`
apart. That is wrong.** `1.4e-10` is how far the meet's *box* reaches; the two
points are `4.1e-16` apart, and the old answer was right.

A probe patch then logged every place the old rule would have folded and the
new one did not. Across the workspace, that was this one fold and nothing
else: three log lines, two from the provenance door test, which walks each
draw twice, and one from `every_seed_in_the_checked_in_corpus_is_fused_without_panicking`.
That run was stopped once every suite test had finished. A probe test of
this lane's own was still walking the corpus, reading the log file once per
draw.

### 253.4 The rule now, and why it is not a new tolerance

`a_corner_is_this_place` folds `place` into a corner when all of these hold:

- the corner is on both crossings (the model's word, as before);
- `place`'s box touches the corner's box (as before);
- **every point of `place`'s box lies inside the corner's box grown by
  `what_writing_the_corner_down_cost`**;
- **no other place's box reaches that grown box.**

Each new condition only narrows the old rule. If the grown box holds one
place, the bare box holds at most one. So no fold happens now that did not
happen before.

**This is not a tolerance added to make a refusal go away.** It is the
opposite: a threshold that was missing, and so was effectively infinite, is
now the argued one. The number is not new either. `the_place_the_model_points_at`
grows a stored corner by `what_writing_the_corner_down_cost` and lets a meet
whose box reaches it stand in for the corner. Its doc comment gives the
argument (§75.6): a face pins its plane through one corner, so the others
satisfy it only to a rounding. Within that allowance, a stored corner and a
meet are one point written down two ways. That is representation wobble,
which §17.2 keeps out of `must_decide`. Past it, two nodes whose boxes overlap
are §17.4's case, and `merge_points_that_are_one_point` asks the policy about
them.

The uniqueness test moved to the grown box for the same reason. Two places
of one meet are different points (§248.1). If both lie within the allowance
of one corner, the corner could be either of them, so neither is folded.

### 253.5 What H2 needed and did not get

H2 asked for a certified fuse in which the folded meet and the corner are
further apart than the allowance, and the fold carries that to a wrong answer.
**None was found under `Policy::certified`.** The measured maximum is a
quarter of the allowance. One construction was tried, and it did not reach
the fold. It was the block-and-frustum union with a third solid whose wall
passes exactly through the rounded corner `(−1.7320508075688772, 1, 4)`, at
angles `2⁻²` to `2⁻²⁰` off the floor circle's tangent. Every angle refused
first on `order along a crossing`, between two boxes round `(−√3, 1, 5)`.
That refusal was not traced. No fold with a reach past the allowance was
logged for any of those fuses. Which plane the folds logged at the corner
came from, the wall or `y = 1`, was not traced either.

The tolerant draw of 253.3 is the only fold past the allowance anywhere in
the workspace. Whether its old answer was wrong, rather than just unasked,
was not measured by the lane. The verifier measured it, and it was right
(253.7).

### 253.6 What this leaves

- `crates/rcad-topo/src/fuse/bake.rs`: `place_a_corner` returns a corner with
  fewer than three distinct surfaces round it straight from the model, and
  never asks those surfaces about it. 253.1 found all 224 certified off-surface
  folds on such corners. Whether that branch should grow the corner and ask,
  the way `derive_a_corner` does, was not looked at.
- `crates/rcad-topo/tests/fuzzing/`: the fuse harness keeps a refusal's
  records inside the `Error` it drops. So 253.3 cannot say which predicate
  the draw now refuses on.
- The `== 1` branch of `a_corner_is_this_place` still has no test that
  reaches it with two places (§248.6). It now counts places within the
  allowance rather than places touching the corner, and that widening of the
  count is also untested.

### 253.7 What the verifier found: the old answer was right

An independent verifier worked the draw out in exact rationals, ran
mutations, and merged the lane onto `732a849a` (1015 rcad-topo tests passed)
and against Q5 and Q76 (both clean).

- **The draw is a tolerant union of two blocks that do not meet.** On the
  base it gave two solids, which validate under both policies, with a total
  volume of 2000. That is the right answer.
- **The meet and the corner are `4.1e-16` apart, not `1.4e-10`.** The meet's
  *box* is `±1.43e-10` wide because two of its three planes are about `1.5e-6`
  off parallel (the determinant is `1.54e-6`). 253.3 read the box's reach as
  the distance between the points, and it is not.
- **So the refusal is a gap**: "order along a crossing", enclosure `2.86e-10`
  against a tolerant budget of `1.42e-12`. A right answer is now refused, and
  it is the only effect of the change anywhere in the workspace.
- **Merge anyway.** The change only narrows the fold, which the verifier
  checked from the code, and it sends an identification nobody asked about to
  the policy, as §17.4 wants. Refusing is the safe way to be wrong.
- **Mutations.** Growing the room a thousand times passes every test; three
  thousand times is caught only by the new test. Dropping the uniqueness
  condition passes all 2481 workspace tests. Both are untested, and fold into
  the `== 1` row above.
- At merge, the test file, its assertion message and the doc comment on
  `a_corner_is_this_place` were corrected to say the above, and the draw was
  copied beside the test as a fixture, so that minimising the corpus cannot
  drop it. When the meet's box is tightened the test's pin moves to one answer
  with two solids.
