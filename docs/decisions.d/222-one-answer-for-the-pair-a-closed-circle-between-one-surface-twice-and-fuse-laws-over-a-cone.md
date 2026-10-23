## 222. One answer for which two surfaces a run is between, a closed circle between one curved surface written twice, and the fuse laws over a cone — 2026-09-25

§0's next row (§221.5's first bullet) and §221.5's other two, one lane each, off
`6664fef8`, which the dispatching session gated green before any lane was cut:
**PASS, 2355 tests, 6 skipped.** All three merged as they came back.

**The merged tree `bd8e4442` has no clean `just gate`.** It stopped three times
out of three at `check-fuzz-overnight-selftest`, a different arm each time.
Every step after that one was then run by hand on the same tree, and all
passed: 2369 tests passed, 11 skipped, and lint, the release build and `deny`
were green. It was merged on that evidence and §222.5's diagnosis. This is a
decision taken knowingly, not a green being claimed.

**The part to read is §222.3.** Five associativity cases over a cone answer one
way round and refuse the other. None of them is a wrong answer yet, but one is an
answer set against `ResultIsNotManifold`.

### 222.1 §23.8 is one answer now

`rcad_geom::which_two_surfaces_a_run_is_between(&EdgeRun, [&Surface; 2]) ->
[Surface; 2]` replaces walk.rs's private `the_two_surfaces_it_runs_between`,
validate.rs's `what_says_which_way_this_closed_edge_winds`, `mesh.rs`'s walk-test
oracle, and a fourth copy the brief did not name: the mesher's pole pair near
`mesh.rs:752`. All of them now call it. `every_spelling_of_one_surface_names_the_same_crossing.rs`
no longer restates the guards. It checks, for every draw an arm catches, that the
re-spelt pair equals the pair written once, and its arm counts match its pinned
table.

**The public answer is not order-dependent where the old one was.** The old one
answered `RightRoundWhereItCrosses { and }` with `[along[0], and]` whatever `and`
was. So when `and` was bit for bit `along[0]`, it answered one surface twice, and
the same edge listed the other way round answered the real pair. That is why
validate had kept a narrower copy. Now, when `and` repeats one of the two faces,
the faces stand as they are, for this variant and `PartOfWhereItCrosses` alike.
`one_answer_for_which_two_surfaces_a_run_is_between.rs` has five tests. Two of
them fail with that arm disabled: the face-order swap, and the same swap made
through `where_a_closed_crossing_starts`.

Nothing moved: rcad-geom 1002, rcad-topo 917 (6 skipped), rcad-step 142, nist 1 of
44, kicad 161 of 323, all as the lane ran them and as the Sonnet verifier re-ran
them.

What the lane left alone, and why:

1. `mesh.rs:688-691` still swaps in `the_curved_surface_a_run_names` when every
   face is flat. It is a narrower copy of the same question. Switching it would
   change the pair `the_way_a_closed_edge_is_stored_running` reads for a fuse rim
   between faces `[s, s]`, from `[s, s]` to `[s, cutter]`, and that changes mesh
   output. It needs a test of its own first.
2. Validate's winding check now reads `[face, and]` for a
   `RightRoundWhereItCrosses` between curved faces, where it used to read the two
   faces. That is the pair the fuse itself picks with (`assemble.rs:1281`,
   `carried_by`). No test moved.
3. Where `and` is a third surface and the two faces differ, the answer still
   takes `along[0]` (or the ball). That still depends on face order, and nothing
   has shown it matters.

### 222.2 A closed circle between one pipe written twice keeps its plane

§219.6's closed branch of `round_run` still used bit equality for curved faces.
The lane's test is a round bar in two bands with a closed circle between them. On
base, with the upper band's `CYLINDRICAL_SURFACE` radius one ulp off, the import
is refused with `AWireWillNotLieFlatOnItsFace`. The bit-for-bit spelling imports,
validates and meshes, and the test uses it as the oracle.

The fix is §221.1's arc branch applied to closed runs. When every face along a
closed `CIRCLE` is curved and the faces are not one surface, `a_circle_every_face_holds`
checks the file's circle against every face. If it holds, the run is
`RightRoundWhereItCrosses { and: <the circle's plane> }`. If not, the run stays as
it was. There is no tolerance on surface parameters. This is also right for two
really different curved faces that share the circle, for example §189.3's ball
and a pipe of its radius that only touch there. The circle's plane crosses each
face in exactly that circle, so `[face, and]` names the same curve either way
round.

**The corpus did not move, and the reason is that the fix fires nowhere in it.**
nist 1/44, kicad 161/323 before and after. All 239 refusal and report lines are
identical. `how_often_a_real_file_imports` now counts plane runs between curved
faces, and it finds 354 it can check, **0 of them closed**. So this is §16.10's
gap closed on a demonstration the lane built, not on a file. The verifier
reproduced every one of these numbers.

Two gaps remain, and both are gaps and not limits: the answer exists and the
check could reach it.

- A closed `ELLIPSE` between curved faces: `a_circle_every_face_holds` returns
  false for it, so it stays `RightRoundWhereItsFacesCross`. The ellipse's own
  plane crosses a pipe in exactly that ellipse, and the check could be written in
  closed form.
- A closed circle on a cone or a torus written twice: `how_far_off_the_circle_can_be`
  has bounds only for a ball and a pipe.

The corpus holds none of either that anybody has counted.

### 222.3 Associativity over a cone answers one way round and refuses the other

`crates/rcad-topo/tests/fuse_laws_over_a_cone.rs` runs associativity, idempotence
(`A∪A'`, `A∩A'`, and a block-and-cone union against its copy) and absorption, with
A as the block and then as the cone. It runs them over §49.6's frustum grid, the
16 rows of the answered-block family and a pointed cone. It compares piece count,
corners bit for bit, and volume within measure's bound. It runs in 0.27 s.

**Where both sides answered, no law disagreed.** But few did. Absorption compared
5, 4, 4 and 1 cases. For A the cone, those rows are ones where the block misses,
so A∩B is empty and the check is really A∪∅. Associativity compared 3.
Idempotence of a cone with its copy compared **0**. All 46 of those fuses refuse
on "a point in a piece that every solid could place", and that includes the
pointed cone, which has no seam. So §36.16's row is wider than "a surface with a
seam", since a ball does answer there.

Five trios answer in one grouping and refuse in the other. Each is committed as
an `#[ignore]` test that fails on base, and `ignored_tests` goes from 6 to 11:

1. `a_frustum_a_block_and_a_cube_inside_the_block_unite_one_way_and_not_the_other`.
   `block∪(frustum∪cube)` comes back `ResultIsNotManifold` at the second union.
   The other grouping answers. This is the strongest of the five, because an
   answer stands against a refusal saying no model can hold the result.
2. `a_swallowed_pointed_cone_and_a_cube_through_it_…`: `cone∪cube` is Undecidable
   on "a point inside a piece of a face".
3. `a_swallowed_pointed_cone_and_a_cube_grazing_it_…`: `cone∪cube` refuses with
   `ACurvedFaceGoesRightRoundWithNoSeam`.
4. and 5. `a_frustum_cut_by_a_block_and_a_far_cube_…` and `…_a_cube_in_the_frustum_…`.
   `block∪frustum` answers, and uniting that answer with a cube refuses with
   `AFaceTrimmedByACurveThisCannotFollow`, even with a cube that touches nothing.

The lane's reading of 4 and 5 is that **the fuse cannot take back an answer of
its own that carries a hyperbola arc.** Idempotence of union points the same way:
`AFaceTrimmedByACurveThisCannotFollow` ×8 and `AFaceMeetsItsNeighbourOnItsOwnSurface`
×12 on `U∪U'`. Nobody has shown this. It is a hypothesis for a `fuse-debugging`
lane. Each case reproduces with
`mise exec -- just t -p rcad-topo --test merged -E 'test(/^fuse_laws_over_a_cone::/)' --run-ignored ignored-only <name>`.

The doc comment of `what_the_answered_block_fuses_actually_say.rs` said
`fuses_that_must_agree.rs` has no cone in it. It has run commutativity over
`FrustumAndBlock` since §47.7, and the comment now says so.

### 222.4 What this leaves

Closed: §23.8, and §221.5's three bullets.

Open, all new here:

- §222.3's five asymmetries, case 1 first, and whether the fuse can take back its
  own hyperbola arcs.
- Idempotence over a cone compares nothing, because two copies of a cone always
  refuse.
- §222.2's two gaps: a closed ellipse between curved faces, and a closed circle
  on a cone or a torus written twice.
- §222.1's items 1 and 3.
- §222.5: stub `fuzz-watermark` in the selftest.

### 222.5 The overnight selftest reads the whole repo's history of greens

On the night of 2026-09-24, `check-fuzz-overnight-selftest` failed six times
and passed three, on a tree whose only tooling change was `ignored_tests`. A
Sonnet lane read the recipes. Most of the selftest's arms call the real
`fuzz-overnight`, and none of them sets `RCAD_FUZZ_ALREADY_ADVANCED`. So each of
those arms runs the real `fuzz-watermark`. The selftest stubs its `gh` half and
not its local half. That local half does a `git fetch`. Then, for every sha in
the shared `.git/rcad-greens`, it makes up to three git calls: 117 entries,
**3.6 s** for the loop alone, timed at a load average near 30, with the real
night on 20 of 24 cores. The deadline is taken before the lookup, so the lookup
comes out of windows of 4 to 20 s. The run that reported `no-time` for all five
targets is the recipe's skip-every-target branch: the lookup had used the whole
8 s window. The arms that never reach `fuzz-watermark` never failed: the hub
arm, the two fresh-repo arms, and the one that sets `ALREADY_ADVANCED`. The
lookup's first check is for a dirty tree, and it returns in 0.09 s, so the slow
path runs only on a clean tree. That matches the lane's two passing runs, made
while this section was being written.

It is §173's class: a check answering a property of the environment, here the
length of a file every worktree appends to, rather than of the tree. It will
get worse as `rcad-greens` grows. The fix is to stub `fuzz-watermark`, or point
the arms that do not test it at an empty greens file. That was not done here:
it is a justfile lane of its own.
