## 221. A pipe keeps its rim, a hyperbola arc keeps its branch, and a cone face with its apex behind a cut is followed — 2026-09-24

§0's next row (§220.4) and two of §219.8's items, one lane each, off `4ae37b4f`,
which the dispatching session gated green before any lane was cut: **PASS, 2342
tests, 7 skipped.** Two of the three lanes merged as they came back; the third
turned up a wrong answer in the fuse, and two more rounds of lanes went after it.

`just gate quiet` on the merged tree `2b774de4`: **PASS, 2355 tests run, 2355
passed, 6 skipped**, 25 doc tests across 9 crates. The first two lanes landed on
`main` earlier as `ada53e48`, gated on their own at 2347 passed, 6 skipped.

**The part to read is §221.3.** The fuse named each arc of a hyperbola by the
half-space through its chord, and that half-space holds the whole other branch
as well. So the walk could take an arc across the cone's apex onto the other
nappe. Three named refusals in the block-and-frustum family were this one fault.

### 221.1 A pipe written twice keeps its rim, because the importer now checks the plane

§220.4 closed, and not in the way §220.4 said to close it.

§220.4 said to widen §189.3's arm from "a ball" to "either face". Done on its own,
that is wrong. `rcad-step`'s `the_surface_that_names_the_curve` wrote the curve's
own plane, **unchecked**, into `and` for every arc between two curved faces with
no ball among them. The walk used to ignore that plane and read the two faces.
Widening the arm would have made the walk trust the plane instead. The lane
counted what that would have trusted: 494 unchecked planes in files that import.
The first one is a 45° mitre in `Relay_DPDT_FCL_FTR-B4C.step`, where two pipes
cross in an ellipse and not in a circle at all.

The fix has two halves, both in `30e33515`:

- The importer runs `a_circle_every_face_holds` for **every** pair of curved
  faces, not only a pair with a ball in it. Where the check fails, it writes one
  of the faces into `and` instead of the plane, choosing the ball first so that
  no ball run changes. After this, a plane `and` between two curved faces always
  means a circle that has been checked.
- `the_two_surfaces_it_runs_between`'s arm is then widened to `both_curved &&
  and is a plane`. It pairs the plane with the ball if there is one, and with
  `along[0]` otherwise.

`a_pipe_written_twice_a_hair_apart_walks_the_circle_its_plane_names` is no longer
`#[ignore]`d, and `ignored_tests` went from 7 to 6. `how_often_a_real_file_imports`
now asserts that every plane written between two curved faces is one a circle
check passes: 354 pass and 0 fail. With the importer half reverted, the same
count is 494 that fail. **No file moved**: nist 1/44 and kicad 161/323 before and
after, and all 205 refusal lines are the same (§45.2). The property's fallthrough
bucket now holds only two flat faces meeting in a line.

Three other fixes were rejected, and the commit message gives each one and why:
an arm for "one surface a hair apart", which needs a tolerance, and no `&Policy`
reaches this code (§219.1 rejected the same thing); widening the arm alone; and
checking the plane inside the walk, which never sees the file's circle.

§220.4's two gaps in coverage are still open: an `and` that repeats a face when
the pair is one surface written twice (11 draws, skipped), and very large or
very small coordinates.

### 221.2 One spelling of how a plane leans against a pipe

The last item on §219.8's list. `rcad_geom::how_a_plane_leans_against_a_pipe`, in
`40ed3154`, is the pipe's twin of the cone's. Like the cone's, it runs the sign
cascade under no `Policy`, so no tolerance a caller passes can soften it.
`where_a_plane_and_a_pipe_cross` asks the same private question. `arrange.rs`'s
`a_plane_lies_along_a_pipes_axis` now just calls it, and the exact
arithmetic `arrange` used to do itself is deleted.

The cascade's exact level refuses under the same `product_is_exact` guard as the
deleted code and sums the same way, so it decides every case the old code
decided. `how_a_plane_leans_against_a_pipe.rs` keeps the old sum as an oracle over
60 000 draws: 57 953 were decided by both, and the two agreed on every one; 2 047
only the cascade decided; 0 neither did. `arrange`'s two tests against a rounded
`f64` dot pass unchanged.

### 221.3 A hyperbola arc was named by its chord, and walked across the apex

The `holds.rs` lane (`0552f4d1`, §221.4) made the lens from
`the_way_into_an_ellipse_arc_in_a_fuse.rs`, intersected with the bar, fuse to
3/8, which is the closed form. Union and Difference of the same pair then failed
with `CannotMeshAFace { why: CoordinateOutOfRange { TooLarge } }`, where before
they had been refused as undecidable. A face that meshes alone and cannot be
meshed once fused looked like a wrong solid, so a second lane went after it
before anything merged.

**It was a wrong solid, and the fault was not in `holds.rs`.** The fuse cuts the
lens's cone along the planes of the bar's walls. Those planes run along the axis,
so they cross the cone in hyperbolas. `fuse/assemble.rs` named each hyperbola
arc by the half-space through its chord. That half-space holds the arc, and it
also holds the whole other branch. `rcad_geom::walk` chose a branch by asking
the picker at each branch's vertex, so it chose the wrong one: the arc from
`(0.5, −1.68, 3.5)` to `(0.5, −2.19, 4.5)` was walked through `(0.5, −1.92, −3.97)`,
on the nappe below the apex. The cone face then spread across both nappes, and
the scale the triangulator uses for its domain became infinite. That is where
`TooLarge` came from.

`e2cd175d` names a hyperbola arc by the half-space `rcad_geom::intersect` builds
for its branch: through the apex, square to the axis. With that fix, Union and
Difference validate and match the closed forms within their mesh error:

    lens       = 128π/(27√3) + 64/9 = 15.709863652665922
    union      = lens + 5/16          = 16.022363652665923   kernel 16.02000 ± 0.0264
    difference = lens − 3/8           = 15.334863652665922   kernel 15.33250 ± 0.0264

A midpoint sum on an 8000 × 16000 grid gives the lens as 15.7098637, with no
kernel involved. The two kernel figures come from a probe the lane deleted.
What is committed and was rerun is
`the_lens_and_the_bar_unite_to_the_closed_form` and
`the_bar_taken_out_of_the_lens_leaves_the_closed_form`, which fail with
`assemble.rs` put back to `0552f4d1` and pass with the fix.

**The same misnamed arcs were three of the block family's four walls.** Before
the fix, 30 of the 96 block-and-frustum draws answered. Now 66 do. 'A corner too
far out for the triangulator' (18), 'a crossing in two pieces with no half-space
to tell them apart' (15) and 'a face whose boundary crosses itself' (9) are all
gone, and the only wall left is 'a point inside a piece of a face' (30). The
catalogue rows §39.4 keeps for those three walls, and the figures in §217 and
§219, describe the fuse before `e2cd175d`.

**The new answers are checked, not just counted** (`c9f3d332`, a test-only lane).
`every_block_and_frustum_fuse_that_answers_adds_up` takes every row where the
block cuts the frustum and answers: 7 rows, times 3 operations, times 2 argument
orders, which is the 42 the old test asserted was 0. For each row it checks
U + I = A + B and both differences against the closed-form volumes, with
`validate` on every answer and both argument orders compared. The worst
U + I − A − B is −3.26e-2 against a bound of 4.11e-1. Every residual is
negative, from −3.95e-3 to −3.26e-2: the mesh comes up short on the curved wall.
Measuring A and B from their own meshes instead of the closed forms brings one
row's residual from about −3e-2 to −3.4e-5. The two argument orders differ by
at most 5.3e-6. The family's floor in `how_often_a_certified_fuse_with_a_cone_in_it_refuses`
moved from 24 to 66.

**The walker no longer trusts the picker to choose the branch** (`8578fdc0`).
`rcad_geom::walk` is public and takes any run, so fixing the writer was not
enough (§16.10 asks about the public API, not today's callers).
`which_branch_of_a_hyperbola_this_run_is` now asks which side of the apex's
square plane each of the run's two ends is on. That is a new sign,
`SideOfTheApex`, with an interval level and an exact level. It refuses with
`CannotNameWhichPieceOfACrossingThisIs` when that sign is zero or undecided, or
when the two ends are on different sides. It asks the picker only for a run with
no ends, which is what `intersect` hands out. `a_hyperbola_arc_picks_its_branch_from_its_ends.rs`
walks a chord-picked arc through the public door. Before the fix it walked a
point at `z = −1.589` with both ends above the apex. A probe the lane deleted
found the new choice agreeing with the old one on all 8 274 hyperbola walks with
ends in the `rcad-topo` and `rcad-step` suites, with every sign decided at the
interval level. Merging it cost one wildcard arm. §40 counts those, and
`2b774de4` replaces the arm with the `let … else` that `intersect.rs`'s `ask`
uses.

### 221.4 The gap `holds.rs` was said to have was not where §219.8 put it

§219.8 said `is_a_family_this_follows` takes only a plane cut square to a cone's
axis. That was stale on the day it was written. The `Surface::Plane` arm has
read `Cone::is_cut_in_a_closed_curve_by` since §211, so a flat face cut by a cone
in an ellipse, at any lean, was already followed. Both lid tests in the new
`a_plane_face_cut_by_a_cone_it_crosses_in_an_ellipse.rs` pass on `4ae37b4f`.

The gap §208.6's sign could close was in the `Surface::Cone` arm, which refused
any cone face whose apex is behind one of its cuts. `0552f4d1` follows a cone
face with **exactly one** cut behind the apex, when that cut crosses the cone in
a closed curve and there is at most one other cut. Every ruling then crosses the
cut behind the apex exactly once, and whether a ruling holds a point comes down
to one expression in `cos φ` and `sin φ` being non-negative. That is one arc of
rulings, so the region is one piece.

Two cases are still refused, and both are limits with a shape behind them
(§16.10). Two cuts behind the apex is §218's existing test. One cut behind and
two or more others is the new unit test
`one_plane_behind_the_apex_and_two_in_front_leave_a_nappe_in_two_pieces`: a
floor cutting a circle and two roofs cutting ellipses leave two pieces. Reaching
that one would mean comparing where the two roof arcs meet on the cone, which is
a part with no named consumer (§13).

### 221.5 What this leaves

Closed: §220.4; §219.8's `arrange.rs:4489` and `holds.rs` items; and a wrong
answer nobody had named, the chord-named hyperbola arc (§221.3).

Still open:

- **§23.8 is still two answers in two crates** (§220.3, §220.5), and it is now the
  next row. `the_two_surfaces_it_runs_between` is private to `rcad-geom`.
  `validate.rs` has its own narrower copy, and `mesh.rs:5148`'s test check has a
  third that still uses §219.1's bit equality and has no case for
  `RightRoundWhereItCrosses`. One public answer in `rcad-geom`, read by both,
  closes all three.
- `round_run`'s `==` for a **closed** run between two curved faces a hair apart
  (§219.6). §221.1 changed the arc branch. The closed branch still asks for the
  same surface bit for bit.
- Nothing has run associativity, idempotence or absorption over a cone. The block
  family's answers are now checked for inclusion–exclusion and argument order by
  volume, but not for bit-identical corners.
- §220.1's two findings nobody has reproduced, §219.7's `Model::fuse` chain, and
  the rest of §0's list, all as they were.
