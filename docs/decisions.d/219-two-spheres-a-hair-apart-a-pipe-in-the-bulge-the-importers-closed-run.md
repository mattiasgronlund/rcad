## 219. Two spheres a hair apart, a pipe in the bulge, the importer's closed run, and who else reads the old closed kind — 2026-09-24

§0's next unclaimed row and the three things §218.7 left beside it, one lane
each. Four lanes off `aa498760`, three writing and each verified by a Sonnet
reader that reran every number, one read-only.

`just gate quiet` on the merged tree `cafc0242`: **PASS, 2328 tests run, 2328
passed, 6 skipped**, 25 doc tests across 9 crates. The base `aa498760` was gated
by the dispatching session before any lane was cut — PASS at 2319, 6 skipped —
rather than taken from the greens file, which only agreed afterwards.

One lane came back with a red test it was right not to fix, and the cause was
the brief; §219.5 is the whole of it.

  219.1   A ball written twice a hair apart answered with its own great circle   
  219.2   Two balls round one centre have no circle, and that is a limit with two names
  219.3   The bulge counts a pipe, and the doc's warning was real and was one arm
  219.4   Three pinned refusals over bars became answers                 
  219.5   The two bars stop at one wall now, and the brief sent the lane to the wrong file
  219.6   Bit equality was the empty fix at the importer's door          
  219.7   Who else reads the old closed kind: three gaps, and the worst is silent
  219.8   What this leaves                                               

### 219.1 A ball written twice a hair apart answered with its own great circle

§218.6's wrong answer, closed — and narrower than §218.6 reads, which is the
first thing to write down.

**The headline case was already right.** §218.6 describes a ball cut by a
**plane**, both faces the one sphere. The lane measured that case first and it
answers correctly today, both bit-identical and 1e-15 apart: §189.3's arm in
`the_two_surfaces_it_runs_between` — `both_curved && and.as_plane().is_some()
&& a_ball.is_some()` — catches it before the fallthrough. So §218.6's "a rim the
fuse writes is immune, and nothing else is" is too wide. What is not immune needs
`and` to be **curved**.

The reproduced case is a ball of radius 5 at the origin, written a second time
with its centre at `(1e-15, 0, 0)`, crossed with a second ball of radius 5 at
`(0, 0, 6)`. The rim is centre `(0,0,3)`, radius 4, flat at `z = 3`. Before the
change the walk came back with seven points standing 2.05 to 3.65 from a centre
they must stand exactly 4 from, with `z` running 3.44 to 4.97 on a curve that is
flat at 3 — the ball's own great circle, radius 5, in the plane bisecting the two
spellings, exactly the shape `part = 1/2` predicts. **Those figures come from a
throwaway probe the lane deleted before committing**, so the verifier could not
rerun them and said so; what is independently confirmed is that the committed
test binary does not compile against the base, and that after the change the same
run walks eleven points every one of which is within 1e-12 of the true rim, byte
for byte the answer the ball written once gives.

**A gap, not a limit** (§16.10): the rim exists, is representable, and the run
names outright the surface it lies on. What could not reach it was the choice of
pair.

**The fix reads the writer's record the other way round, and that is the part
worth keeping.** The obvious move — trust `and` whenever a `PartOfWhereItCrosses`
carries one — breaks, and the lane broke it from the source rather than by
argument. `assemble.rs`'s `the_surface_that_names_it_twice` writes **one of the
two face surfaces** into `and` when the faces genuinely do cross ("an arc's two
faces already name both, so either is a true answer"), so unconditional trust
hands `where_two_balls_cut` the pair `[A, A]` — §219.2's NaN — whenever the sort
picks the near face. `rcad-step`'s `round_run` writes the curve's own plane into
every arc's `and`, so it would repoint every imported arc as well.

So the new arm asks whether `and` is a **third** surface that neither face is:
`both_curved && and.as_plane().is_none() && and != along[0] && and != along[1]`.
It is still bit equality, but in the direction where bit equality is the robust
reading — a writer that means "`and` adds nothing here" copies a face verbatim,
which is what `the_surface_that_names_it_twice` does. That dependence is named in
the doc comment, so moving that writer moves a sentence that names it.

Four other families were rejected and are in the commit message: a tolerance on
the two faces (no `&Policy` reaches here); a typed argument in arm 3's style (two
balls cross in a *circle*, and a circle lies on a plane, a ball, a cone or a
torus, so no kind of `and` contradicts the pair); refusing in `where_two_balls_cut`
on a small gap (two balls 1e-15 apart really do cross in a near-great circle —
the fault is entirely upstream); and checking the answer against the run's own
ends (ends sit honestly up to ~1e-13 off a curve, so it needs a slop, and it
detects the wrong answer without ever naming the right one).

### 219.2 Two balls round one centre have no circle, and that is a limit with two names

§218.6's "a fix nobody has to reason about", and it turned out to want one
distinction. Two bit-identical balls gave `squared_gap = 0`, `part = 0/0`, a NaN
midpoint, and `Plane::from_point_and_normal` refusing `NotFinite { what: "point
on a plane" }` — the radical plane being blamed for a fault in the pair it was
built from.

**A limit**: two balls round one centre have no crossing circle at any precision.
Of one radius they share their whole skin and every point is common; of two radii
they share none of it. Neither has a curve to hand back.

Those are **two** refusals and not one, which is the sufficiency point §16.10
asks for — "two centres in one place" ranges over equal and unequal radii.
`Error::TheseTwoSurfacesAreOneSurfaceAndNameNoCrossing` is new for the first; the
existing `TheseTwoSurfacesDoNotCrossInAConic` covers the second.

### 219.3 The bulge counts a pipe, and the doc's warning was real and was one arm

§218.5's gap, closed. `Cylinder::is_cut_in_a_closed_curve_by` is the one-sided
sign of `n·a` in `rcad-geom`, beside `Cone::is_cut_in_a_closed_curve_by`, and
`in_the_bulge_of` asks it. There is no apex case to dispose of first, which is the
whole of why the pipe's version is shorter than the cone's.

**The cost the doc warned of was real, and probing it rather than quoting it is
what found its size.** With the pipe counted in `select.rs` alone,
`just t --no-fail-fast` gave 2319 run / 2314 passed / **5 failed**, and two of
those five were the predicted cost: `two_tools_meeting_on_a_plane_that_cuts_a_round_wall`
and the catalogue table's coincident-bar row both came back
`ResultIsNotManifold { count: 4 }` — "two faces into four along one edge",
arriving as a manifold refusal rather than as a corrupt solid.

One arm in `assemble.rs`'s `which_of_them_a_closed_crossing_runs_round` removed
both and turned them into answers. That arm's precondition is **stronger** than
the cone's and is written into its doc: the sign rule needs `∇round × ∇plane` to
vanish nowhere along the curve; a pipe's gradient is square to its axis, and a
closed crossing has `n·a ≠ 0`, so the two can never point the same way.

No face count moved anywhere else. §217.2 and §181.6 were two cases of an
unmeasured cost coming out wrong; this one came out right in kind and small in
size, and neither outcome was available without running it.

The predicate is tested at the rounding boundary, not only in the middle:
`a_plane_a_rounded_dot_calls_flush_with_a_pipes_axis_still_closes_round_it` takes
`arrange.rs`'s own fixture, a normal and an axis direction whose `f64` dot is
exactly `0.0` and whose exact sign is not, and asserts `Some(true)`. A rounded
spelling reads no closed curve there, so no bulge, so a refusal on a pair nothing
had run out of arithmetic on.

### 219.4 Three pinned refusals over bars became answers

Counting the pipe turned three rows of `degenerate_curved_fuses.rs` from pinned
refusals into answers with closed-form volumes, and nothing predicted any of
them:

- a bar cut by one plane off its axis — one solid measuring `8·(8π/3 + √3)`;
- the tangent row's control at `y = −1.9` — one solid, `32π + 12 − 2·(0.55√2.79 +
  2·arcsin 0.55 + 2π/3 − 1.1 − √3/2)`. The tangent case above it still refuses on
  "which way a crossing runs", so the row's control is sharper than it was: a
  refusal one side of one number and an answer the other;
- two tools meeting on a plane that cuts a round wall — two solids measuring
  `24π`, **the same answer one tool making the same cut gives**, which is what
  §16.10 said the answer would have to be.

The zero-length-edges row moves from `GaveUp("a point inside a piece of a face")`
to `Answered`. `docs/decisions.d/037` and `039` each list two of these as open
gaps in tables that are historical records of what M4 cost; they stay as written,
with this section as the forwarding address.

### 219.5 The two bars stop at one wall now, and the brief sent the lane to the wrong file

The deliverable's own pin is
`two_overlapping_parallel_bars_stop_in_select_whichever_way_round`, and it lives
in `crates/rcad-topo/tests/fusing_two_bars_side_by_side.rs`. **The brief said
`degenerate_curved_fuses.rs`, which is where §218.5 names the *case* and not
where the pin is.** So the one test the lane existed to move was outside its
write set. It stopped, reported the exact replacement map, and left the branch
one red test short of green — which is the right call and cost nothing but the
dispatching session applying four lines at the merge.

Worth keeping as a briefing rule: a brief that names a file for a test is making
a claim about the tree, and a claim about the tree is checkable before the lane
runs. Naming the **test** costs the lane one `grep`; naming the wrong **file**
costs it the deliverable.

With that edit, all three rules and both orders now stop at *"a point in a piece
that every solid could place"* — one refusal name where §217 had split two. The
helper is renamed off `the_walls_measured_when_217_split_them`, which carried a
section number for a state that is over.

**They still do not fuse**, exactly as §218.5 predicted: the order that gets a
point dies at the next rule, and both orders now die there together. That is a
different search and a separate piece of work.

### 219.6 Bit equality was the empty fix at the importer's door

`round_run` returned `RightRoundWhereItsFacesCross` for every full `CIRCLE` or
`ELLIPSE` whose ends are one vertex, without asking whether the faces either side
are one surface. It now asks, through `the_faces_along_it_name_no_crossing`, and
writes `RightRoundWhereItCrosses { and }` with the `and` the arc branch two lines
down already built. A gap: the answer existed the whole time — the circle's own
cylinder crosses the flat face in exactly the circle, which is §176.2's fix for
arcs — and `round_run` never reached for it.

**The test is two arms, and the interesting half is which one does the work.**

    faces.len() >= 2
        && (faces.iter().all(|face| face.as_plane().is_some())
            || faces.windows(2).all(|pair| pair[0] == pair[1]))

The lane wrote the bit-equality arm alone first — the brief's suggestion — ran
the corpus, and **the rate did not move and no file budged**. Probing
`surfaces_along_each_edge` said why: at `EntityId(1234)` of
`SW_Tactile_SPST_Angled_PTS645Vx31-2LFS.step` the two planes either side of the
circle are `normal [-8.88e-16,-1.0,-2.5e-16] offset -2.150000000000006` against
`normal [-8.88e-16,-1.0,-7.7e-16] offset -2.1500000000000044` — **one plane
written twice, 1e-15 apart, §218.6's shape at the importer's door, and the
dominant case there rather than the edge case.** Bit equality alone is an empty
fix at this door; the flat arm is what catches those files, and it carries no
tolerance at all, because two planes cross in a straight line and a straight line
does not close.

A nearness on the surface parameters, entity identity, and asking `rcad_geom` to
cross the two surfaces inside the import loop were all rejected; the commit
message has each and why.

**The counts did not move, and that is the claim §45.2 wants stated.** nist
1/44 = 2.3%, kicad 161/323 = 49.8%, the same before and after, with
`assert_pinned_kicad_imports` — which names every file that starts or stops
importing — passing on both sides. Four files changed *which* refusal they stop
on without changing direction: the four `SW_Tactile_SPST_Angled_PTS645V*` files
moved from "a closed edge is walked between two flat faces" to "a corner is not
on the curved face the file says it is on, miss=515.744 spacings".

**What stops them now is one crate up and demonstrated rather than asserted.**
`validate.rs`'s `a_closed_crossing_is_walked_the_way_it_is_stored` takes only
`[&Surface; 2]` of the two **faces** and returns
`Invalid::ClosedEdgeBetweenTwoFlatFaces` when neither is curved. It never sees
the run, so a closed edge whose run carries a perfectly good `and` is refused
anyway. Its caller already matches on `edge.stored_run()` and could hand the
`and` through.
`two_spellings_of_one_plane_stop_at_a_check_that_reads_the_faces_and_not_the_run`
is the standing demonstration and becomes the passing case the day that check
reads the run.

A closed `ELLIPSE` on flat faces still gets the ellipse's own plane, which names
nothing. That is a **limit** and not a gap: no single surface this kernel holds
crosses a plane in a given ellipse without somebody choosing a tilt, and §13
gives that choice no consumer. Nothing in the corpus or the suite exercised it —
such a shape's refusal name would have moved to
`AnEdgeBendsButNothingAlongItIsCurved`, and no name changed that way.

### 219.7 Who else reads the old closed kind: three gaps, and the worst is silent

The read-only audit §218.7 asked for. Twenty-one sites in `crates/*/src` read
`EdgeRun::RightRoundWhereItsFacesCross`; **fifteen already handle
`RightRoundWhereItCrosses` correctly**, including all four §216 was guessed to
have updated. Three have real gaps, ranked wrong-answer first:

1. **`mesh.rs:738`, and it is the worst thing in this section.** Pole insertion
   is gated on the old variant by name:
   `if *run.run() == EdgeRun::RightRoundWhereItsFacesCross { put_in_the_poles_a_closed_crossing_passes(...) } else { 0.0 }`.
   A closed rim written the new way takes the `else`, so no pole is guaranteed
   among the walked points, and
   `put_in_the_poles_a_closed_crossing_passes`'s own doc says what that costs:
   the walk "jumps half a turn of longitude with nothing to say which way" — a
   **wrong mesh, produced silently, under no error name at all.** It is reachable
   through the public API and the chain is named: `Model::fuse` on a ball whose
   cutting plane holds the polar axis leaves a closed rim between two faces of the
   one sphere; `the_surface_that_cut_each_closed_rim` sets `cut_by = Some` for
   exactly that case and `Edge::how_it_runs` then writes the new variant;
   `SolidView::mesh` walks it. This is §176.2's class — a mesh that is wrong where
   a refusal was available — one surface up.

2. **`meet.rs:84` and `walk.rs:2152`, which are one gap in two places.**
   `NamedRun::new`'s sole invariant check is `run == RightRoundWhereItsFacesCross
   && ends[0] != ends[1]`, so a caller can build a new-variant run with two
   different ends and get `Ok` back, although the doc claims the check for "a
   closed curve" generally. `where_a_plane_cuts_a_cone` then has the same
   old-variant-only test, so such a run with `leaning == 0.0` comes back
   `Ok(Parabola(..))` — an open, unbounded curve — where the caller has already
   asked `is_closed()` and been told yes. `NamedRun::new` is `pub`, so the whole
   chain is public-API reachable; a well-formed fuse-written rim has `leaning > 0`
   by construction and does not reach it.

3. **`validate.rs:404`** names the wrong `Invalid` for the new variant —
   `AnEdgeBendsButNothingAlongItIsCurved`, the message meant for open edges,
   instead of `ClosedEdgeBetweenTwoFlatFaces`. Still a refusal, under the wrong
   name.

Two sites name only the old variant and are **proven unreachable** by the new
one, not argued: `build.rs:1000` has the fixed input pair `[Cylinder, Plane]`,
never one surface twice, and `intersect.rs` constructs the new variant nowhere
(0 hits); `arrange.rs:3676`'s `assert!` is a private fn called only on
`Crossing::run`, built directly by `intersect`. The second is a latent trap if
that wiring moves, and is recorded as one.

§218.6's claim about `Model::add_closed_edge` — "today used only for honest
cylinder-and-plane cap circles" — is **confirmed**, and by reading rather than by
repeating: its only production caller is `add_round_bar`, whose cap circles have a
`Cylinder` and a `Plane` either side and never one surface twice; `build.rs` has
no `#[cfg(test)]` module at all, and the other five call sites are inside test
modules.

`holds.rs:850` reaches the same refusal for both variants, so it was a wildcard
naming the third variant and not the fourth. §40 asks a wildcard to name what
exists; it now does.

> **2026-10-10 note (Q377): item 1's chain does not run, and the gate it named
> is gone.** `put_in_the_poles_a_closed_crossing_passes` does not appear in
> `crates/rcad-topo/src/mesh.rs` any more — `git log -S` finds it last at
> `1cff1006` (Q1270), which renamed it to `put_in_the_poles_a_crossing_passes`
> and widened it to run along an open crossing as well as a closed one. The
> `if *run.run() == EdgeRun::RightRoundWhereItsFacesCross { .. } else { 0.0 }`
> gate this item named is gone with it: `mesh.rs` now matches
> `EdgeRun::RightRoundWhereItsFacesCross | EdgeRun::RightRoundWhereItCrosses { .. }`
> in one arm (`mesh.rs:969`) and calls the renamed function for either, so a
> closed rim written the new way gets its poles exactly as one written the old
> way does. There is no live gap left to be silent about.
>
> §220.1 then asked whether `Model::fuse` ever reaches that shape at all, and
> found the chain this item named does not run on `add_ball`: a plane through
> the polar axis cuts the great circle into **two open arcs** through the two
> poles, not one closed rim, so `EdgeRun::RightRoundWhereItCrosses` is never
> written there in the first place. The only route §220.1 found to a genuinely
> closed rim through a pole was through `SolidBuilder` directly (a ball split
> into two lunes), and that shape meshes correctly — pole and all, at all seven
> deflections tried — since the fix above.
>
> The Q171 draw (`a_rim_that_names_its_cutting_surface.rs`, §400) is the fuse
> route that was found to a closed rim between two faces of one ball: a block
> and a ball that touch at one point, cut by the block's plane `y = 0` outside
> the block. That rim misses both poles — the plane crosses the ball a half
> unit off its centre, nowhere near either pole — and
> `a_ball_touching_a_block_cut_in_two_by_the_blocks_plane_names_that_plane`
> pins the rim and `the_union_is_the_same_answer_both_ways_round_with_the_rim_written_down`
> pins the mesh both ways round.
>
> Q871's draw (`a_ball_cut_through_both_poles_by_a_linked_blocks_plane.rs`,
> §400.5) is the one that does cut a unit ball through both poles: a linked
> block's plane `x = 0`, reached through a chain of three solids. Before
> §229.2 that cut the great circle into two arcs that folded into one closed
> run with no ends and panicked `fuse/assemble.rs`; `the_corners_each_closed_run_keeps`
> fixed the panic and the union kept the ball in two faces either side of
> `x = 0`, the two arcs still there as the seam between them. **Since §229.2
> the ball joiner joins those two faces into one before the next fuse, and the
> arcs are gone with the seam** — the test file's own header says so: "Now the
> two arcs are gone (§229.2)." So neither of the two fuse routes tried leaves a
> closed rim through a pole standing in an answer, and the one shape that does
> reach one — `SolidBuilder`'s own lunes — already meshes it right. No silent
> wrong mesh is shown; nothing is filed.

### 219.8 What this leaves

`holds.rs`'s `is_a_family_this_follows` is a **gap** now rather than a limit, and
its doc comment says so. It takes only the family square to the axis and its
argument rests on "telling an ellipse from a hyperbola is a sign this does not
take" — which stopped being true when §208.6 built that sign. Nothing there
behaves differently; the claim was rewritten, not the code.

Newly open, none of it in the tree:

- **`mesh.rs:738`'s pole** (219.7). A silent wrong mesh, reachable through
  `Model::fuse`, with the producing chain named end to end. A wrong answer
  outranks a refusal. *Closed by Q1270 (`1cff1006`), which folded the gate
  this named into one arm for both run kinds; the chain through `Model::fuse`
  was then shown not to run, by §220.1 and the dated note after 219.7 below.*
- **`meet.rs:84`'s invariant and `walk.rs:2152`'s guard** (219.7), one gap in two
  places, both public-API reachable.
- **`validate.rs`'s closed-edge check reading the faces and not the run** (219.6),
  with a standing demonstration already committed, and `validate.rs:404`'s wrong
  refusal name beside it.
- **Two curved faces that are one surface a hair apart, at the importer's door**
  (219.6). `round_run`'s `==` arm still misses them; the answer exists, so it is a
  gap.
- **No property coverage over spellings of one surface.** A generator that draws a
  surface and re-spells it a few ulps away would exercise every arm of
  `the_two_surfaces_it_runs_between` at once. A lane of its own.
- **`mesh.rs:5148`'s test oracle** carries the same `along[0] == along[1]` bit
  equality 219.1 fixed, and its last arm swallows `RightRoundWhereItCrosses` into
  `[along[0], along[1]]` — so §216's variant has no oracle there. An oracle that
  cannot see a wrong place is what §218.6 warned about.
- **`arrange.rs:4489`'s `a_plane_lies_along_a_pipes_axis`** spells the same `n·a`
  sign a second time, in expansion arithmetic. §23.8 wants one spelling; closing
  it properly wants a `how_a_plane_leans_against_a_pipe(flat, pipe) -> Decision`
  in `intersect.rs`, the pipe's twin of the cone's, because `arrange` needs the
  `Decision` with its enclosure and not an `Option<bool>`.
