## 213. The crown of a ring is a gap, and §208.3's arm is still untested — 2026-09-23

§209.5 said a lane had called a family a limit when what actually refused it was a
defect elsewhere. This repins that family as a **gap** with the mesher's own
reason, sweeps the family §208.3's arm was supposed to be for, and reports —
rather than closes — the hole §208.3 left.

### 213.1 The limit that was not one

`lane-fold-repro-hunt`'s argument ran: for a coarse mesh to enclose zero every
point sits on one plane, so both bounding curves lie in that plane, so the surface
is a torus square to its axis, so the shape is the crown of a ring; and the crown
of a ring cannot be meshed, so the family is a limit under §16.10.

The last step does not hold, and §16.10 draws the line at whether the **answer
exists**. The crown of a ring is an ordinary solid with an ordinary volume: it is
built through `SolidBuilder`, the shell closes, and `20`/`5`/`4` is a Pythagorean
triple on purpose so both corners are exact on the quartic — the form at
`(23, 0, 4)` is `920² − 4·20²·23²`, a difference of two products equal number for
number.

What refuses it is `ADraftFaceCouldNotBeMeshed { why: CornersCoincide { .. } }`,
because the face goes right round the **ring** while a walked loop carries one
wrapping coordinate and `Torus::point_at` spends it on the tube. That is a gap in
the mesher, not the absence of an answer, and a verdict conditional on someone
else's defect evaporates the day that defect is fixed. §208.4 is what that costs:
a ring of arcs with no seam was unwalkable until it was walked.

So the demonstration is kept — the crown built exactly, refused by name — and
repinned as a gap.

### 213.2 The family was overstated, and the sweep is quiet

§208.3's brief described the family as solids cut in two closed curves. The code
asks `piece.closes_on_one_curve`, which is **weaker**: a loop of several arcs
fails it with no second closed curve at all, so a sliver chopped by further planes
qualifies too.

288 of those are built here, by narrowing the block that cuts the ball until its
own side faces do the chopping. Of the 270 that answer, **not one of their 2 700
meshes folds** — the chop puts the rim's own arc off the plane of the cut, so none
of them comes back flat either. The bare cut *is* reached: 93 of the 2 798 cuts
that sweep takes are `steps = 1`. It just meets nothing.

### 213.3 §208.3's arm still has nothing behind it, and that is the finding

§208.3 merged a six-line arm into `how_much_space_a_curved_solid_encloses` so the
zero-enclosure halving ladder retries `TwoPatchesOfTheMeshDrewTheSameSide`. **It
was merged with no test that fails without it**, against this repo's rule that
every change needs one, and it was written down rather than hidden. Closing that
hole was this lane's real deliverable and **it is not closed.**

The arm cannot be shown off by the ordinary "revert the change and watch the test
fail", because it is already on the base. So it was taken out by hand. Re-run at
merge on the whole workspace rather than one crate:

```text
$ mise exec -- just t --no-fail-fast          # with the arm removed
     Summary [  63.289s] 2299 tests run: 2298 passed, 1 failed, 6 skipped
        FAIL a_mesh_that_folds_at_one_deflection_and_not_the_next the_arm_removal_experiment
```

**One test in 2 299 fails, and it is the one that reads the arm's own source
text.** Neither family this file adds reaches the rescue's ladder: the crown of a
ring is refused before it can be meshed, and the chopped sliver takes the bare cut
but never comes back flat. The arm remains a guard with nothing behind it.

`the_arm_removal_experiment` is kept, and what it is must not be mistaken. It is
**not** a behavioural test; it asserts that the text the experiment above was run
against is still in the tree, so that if the arm moves or goes the record stops
describing what is there and says so. Its committed transcript previously showed
`884 passed` twice for `-p rcad-topo`; that was taken before the test existed and
stopped being reproducible the moment it did, since removing the arm is exactly
what it fails on. It carries the run above instead.

### 213.4 A lane that owns no `src/` cannot be cross-examined the usual way

This lane owned one test file and no source, so the workflow's "put the source
back to base and watch the tests fail" step is a no-op — `git diff --stat` after
the revert is empty. Setting `verify: false` was the other option and is what left
`lane-fold-repro-hunt` uncross-examined in the first place (§209.4), so the lane
was verified anyway and the workflow logged *"verifier said merge, but the tests
were never seen to fail with the source reverted"* beside the verdict. That log
line is the honest state of a test-only lane, and the substitute is the experiment
above, re-run by the dispatcher rather than taken on the lane's word.

The verifier ran it too and reported the canary firing as the arm's test working
correctly. It is worth being exact: the canary firing proves only that
`include_str!` saw the text change. The four substantive tests passing with the
arm gone is the finding, and it is 213.3's.
