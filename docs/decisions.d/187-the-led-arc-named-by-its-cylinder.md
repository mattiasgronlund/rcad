## 187. The LED arc is named by its cylinder, the silent chord is refused, and the mesh soak finds a wrong mesh — 2026-09-21

*Three lanes off `29ab552e`, dispatched by §175's skill against a base gated
green here first. §176.5's first two items are done: an edge that bends between
two flat faces is now refused by name instead of meshed as a chord, and a
circle drawn on a flat face reaches the model as a cylinder crossing that face.
**None of the LED files imports yet** — there are 20 of them, not 18, and
`Model::validate`'s coarsest mesh is what stops them now. §184.6's question is
answered with a count. And the two mesh suites, soaked for the first time,
found two defects, one of which is a **mesh that is wrong rather than
refused**.*

Merged as `f25f96b1` (`lane-led-arc` at `e73fca03`, `lane-far-pipes-sweep` at
`69b6f7df`, `lane-mesh-suites-soak` at `b3bfbce7`) plus `0b02b833`, which moves
`ignored_tests` from 1 to 3. `just gate quiet` on `0b02b833`, run here: PASS,
2133 tests run, 2133 passed, 3 skipped. Every other number below is a lane's.
`led-arc`'s were re-run by its verifier and matched, except two diagnostic
numbers from instrumentation that was since removed; the two test-only lanes
had no verifier, so their numbers are their own.

### 187.1 The silent chord is refused, and the hedge behind it is an assert

`mesh.rs` now refuses an edge whose faces are all planes when its run is not a
straight line and names no curved surface. The refusal is the new
`Error::AnEdgeBendsButEveryFaceAlongItIsFlat`. §176.5 ruled out
`AFaceMeetsItsNeighbourOnItsOwnSurface` because it claims more than the mesher
can prove. On `29ab552e` the same slab, built through the public builder,
meshed silently as a square (`Ok`, 12 points). The test is
`a_circle_named_only_by_the_plane_it_is_drawn_on_is_refused_rather_than_meshed_as_a_square`
in `crates/rcad-topo/tests/a_bent_edge_between_two_flat_faces_is_refused.rs`.

`walk_a_wire_round`'s `.unwrap_or_default()` is now an assert. The first pass
visits every face and must either walk every edge that is not straight or
refuse it. A missing entry therefore means that pass is broken, which is an
internal invariant and not bad caller input, so it panics.

**Still open, the other half of §176.5's item:** `Model::validate` still passes
such a solid. Only `mesh()` refuses it. §176.5 said either the sweep says so or
`mesh.rs` refuses by name. `mesh.rs` does now. Having the sweep say so as well
is `validate.rs`, which was not in the lane's write set.

### 187.2 The circle reaches the model; no LED file imports yet

`brep.rs`'s `the_arc_between` names a CIRCLE arc whose faces are all flat by
the cylinder of its radius about its axis. `walk.rs` reads the run's curved
surface when both faces are flat, and `mesh.rs`'s flat-face walk places the
edge from that pair. An ELLIPSE keeps its plane and gets §187.1's refusal on
purpose. A tilted cylinder would cross the face in that ellipse, but its tilt
is a free choice the ellipse does not fix, and §13 wants a consumer before
anyone picks one. `brep.rs`'s doc comment says so. A whole CIRCLE with both
ends on one vertex still names no surface. The reader could name it with the
same cylinder, and until it does the mesher refuses it by the new name.

**The count is 20, not 18**: 8 `LED_D3.0mm_Horizontal_*` and 12
`LED_D5.0mm_Horizontal_*`. That is how many files were refused by the new name
with the cylinder branch switched off, which the verifier reproduced. **0 of 20
import** after the change, and the kicad rate is 114/323 either side. No pin
moved.

**What stops them now** is `validate.rs`'s
`how_much_space_a_curved_solid_encloses`. It meshes at
`as_coarse_as_a_mesh_gets()`, which is `f64::MAX`, so each quarter arc becomes
one step. That chord cuts across the disc's pin holes: 18 files refuse with
`BoundaryCrossesItself` and 2 with `CornersCoincide`. The lane tried a probe at
deflection 0.01, and neither committed nor kept it. With it, the disc meshes
on all 20 files, and each file then stops on a later blocker: the quartic
marcher's 50 shortenings, "too nearly parallel to follow", a point placeable
only within 21.33, and a side straying 1.667. **So by §45.2, fixing the coarse
sweep imports none of the 20 by itself.** That is the lane's probe and was not
re-run here.

### 187.3 §184.6 answered: far out, off the axes, nothing wrong

`crates/rcad-geom/tests/where_two_leaning_pipes_far_out_cross.rs` sweeps
pairs of pipes along one direction off the coordinate axes. The first origin is
1e3 to 1e8 out, the radii run from 1e-7 to 1e4, and a quarter of the pairs are
within 1e-12 of touching. Every answer is checked exactly against the true
lines, which are polynomials in the stored f64s and are held in the test's own
exact type. Nothing the kernel rounded is read. The shipped run asks 8000
times: 7994 answered, 6 refused (all `CannotNameWhichPieceOfACrossingThisIs`),
**0 wrong**. Five soak seeds at factor 25 asked 1 000 000 times with 0 wrong.

**§184.6's question, as a count:** an exactly-square normal through a stored
axis origin separates the two true lines in 4228 of the 8000 askings. None of
the 6 refused askings has such a plane, so by that test no refusal is a gap.
The anchor between the lines is crate-private and was not checked, so calling
the 6 refusals limits is **not proven**. §16.10 wants that anchor checked
first.

**How much this sweep proves.** Run on the trees before §182 and before §184.7,
it also finds 0 wrong. So in this family the older defects changed no answer,
and the sweep cannot claim it would have caught them. A separate test builds
three wrong answers from a real one and checks that the sweep's check refuses
each. That shows the check can catch those shapes. It is not a red run against
a real defect.

### 187.4 The mesh soak finds two defects, one a wrong mesh

Both mesh suites now carry a seeded proptest generator that reads
`RCAD_SOAK_FACTOR` and `RCAD_SOAK_SEED` the way `fuses_that_must_agree.rs`
does, which closes §176.4's finding. The default gate cost is unchanged within
noise, by the lane's timings. At factor 50 each suite failed on its first
failure. Both failures are kept as named `#[ignore]`d tests with literal
numbers:

1. **A wrong mesh.**
   `a_bar_whose_top_leans_five_across_for_one_up_meshes_into_one_closed_surface`:
   a bar of radius 1 and height 10, with its top cap leaning 5 across for every
   1 up, meshed at 0.1. Near the tip of the ellipse, the cap and the wall both
   cut off the same ear. That gives two triangles on the same three points,
   facing opposite ways, and one side then used four times. χ is 3. It happens
   at leans of 5 and above, and never at 2 or below. **`mesh()` returns this
   as `Ok`.** That is §25.1's crack, and by the first core principle it comes
   before any refusal.
2. **A band of refusals.**
   `a_turned_quarter_bend_meshes_at_every_deflection_coarser_than_one_it_meshed_at`:
   a turned quarter bend meshes at 1.46, is refused with "the boundary crosses
   itself" from 1.47 to 1.57, and meshes again at 1.58. It is a refusal and not
   a wrong answer, but it breaks the property that a coarser deflection meshes
   whenever a finer one did.

Neither has been diagnosed. Each soak stopped at its first failure, so the rest
of the factor-50 corpus has not been searched.

**2026-10-05 note (Q1102): item 2's band no longer refuses, as of this base.**
`a_turned_quarter_bend_meshes_at_every_deflection_coarser_than_one_it_meshed_at`
meshes at 1.5 now too, and every other test in the file still passes. Which of
the several changes since this was written fixed it was not diagnosed. Un-
ignored in `44a6b707994866a8ff1b3eb41b6d4877975cfa9d`.

### 187.5 What is left

- **The wrong leaning-bar mesh**, §187.4 item 1. It is first because the mesh
  it returns is wrong. Use the `fuse-debugging` and `testing` skills' approach
  on the mesher, in `rcad-mesh` or `rcad-topo`.
- **The LED files' coarse sweep**, §187.2: `validate.rs` meshes at `f64::MAX`.
  Here the fix is only a step. It frees none of the 20 by itself, and the
  blockers behind it are §187.2's list, each unmeasured as a count.
- **The sweep half of §187.1**: `Model::validate` passing a flat-only bent
  edge that nothing names.
- **The turned-bend band**, §187.4 item 2.
- **The 6 far-pipe refusals**, checked against the anchor between the lines
  before anyone calls them a limit.

Commands: `just decision 176.2`, `176.4`, `176.5`, `184.6`, `184.7`; the
`lanes` workflow run `wf_1e1865b4-079`; `git merge lane-led-arc
lane-far-pipes-sweep lane-mesh-suites-soak`; `just gate quiet` on `0b02b833`.
