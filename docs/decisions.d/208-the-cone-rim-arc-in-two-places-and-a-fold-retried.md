## 208. The cone's rim arc in two places, a fold retried, and the ring that closes round an apex — 2026-09-23

*Four Opus lanes off `075b487e`, three of them checked by a Sonnet verifier that
re-derived the claims from the branch. Three merged; the fourth is on disk and
§208.4 says why it is not. Every number below that is not marked as a lane's is
from a run made here on the merged tree. `just gate quiet` on the merge
`09724a52`: **PASS, 2280 tests run, 2280 passed, 6 skipped**.*

### 208.1 A flat face cut by a cone square to it is a family `holds.rs` can follow

`is_a_family_this_follows` admitted a plane cut by planes and by the insides of
balls, and a sphere cut by planes through its centre. It now admits a plane cut
by a **cone that stands square to it, material inside**.

The argument is re-made rather than the `match` widened. Square to the axis the
crossing is a circle on one nappe, and the cone's inside on that plane is the
disc that circle bounds — convex, so the part of the plane in front of every cut
is still one piece, which is what
`what_a_face_cut_by_part_of_a_crossing_holds`'s whole argument rests on. A
leaned plane stays refused: the ellipse is not decided yet, and the hyperbola's
inside is two regions, which breaks connectedness outright.

**The per-arc condition carried over whole, and *why* is the finding.** It lives
in `a_picker_that_tells_the_two_arcs_apart`, whose one non-flat ingredient is the
circle's centre. Square to the axis that centre is where the axis crosses the
plane, which is the foot of the perpendicular from the apex — so
`where_the_circles_centre_sits`'s existing polynomial reads it with the **apex in
the ball centre's place**, proved algebraically in the new doc comment.
`the_way_into_an_arc` therefore asks one new question only.

Two other places needed the cone: `gradient_at`, as `|w|²x − (1 + s²)(x·w)w`
with the factor of two dropped because only signs are read; and a straight side
against a cone cut, which lies inside by the **disc's** convexity and not the
double cone's.

The nappe file's smallest `decided` over its 104 answers was **8 879** and is
**9 326**; its assert moves 8 800 → 9 300. Both figures are the lane's, from a
temporary `println!` that is in no commit. The file's 104 fuse tallies did not
move at all — those refusals are `fuse/select.rs`'s, ahead of `holds.rs`.

### 208.2 The two arcs of a cone's rim circle are told apart, and the door is whether the crossing closes

`which_arc_of_a_circle_this_is` returned `None` for anything but a ball and a
plane, so two arcs of a cone's rim circle between one pair of corners were named
as one edge.

**The sign rule in its doc comment was never about a ball.** What it needs is a
crossing that **closes**. The new door `a_plane_cuts_a_cone_in_a_closed_curve`
decides that exactly, off the two signs `intersect` already splits its own table
on — the apex off the plane, and `E = (n·w)² − slope²‖n × w‖²` above nothing.
That is a cap's circle and a steeper plane's ellipse; a parabola and a hyperbola
stay `None`, so two corners on an endless bend still leave one arc.
`mine_is_the_ball` was generalised in place, not copied.

The seam guard is the **ball's alone** and now says so: the only maker of
`MeetPoint::APoleOfAClosedSurface` is `Arrangement::the_whole_surface`, whose
first statement refuses every surface but a ball, so no cone side can carry one.

| Where | Base | With the change |
|---|---|---|
| `fuses_that_must_agree.rs` comparisons made | 110 | 114 |
| …wider grid, of 336 fuses: refused / not a solid | 108 / 8 | 108 / **0** |
| `ordering_points_along_an_endless_bend.rs` answered of 84 | 53 | 57 |
| …refusals by kind | point in a piece 27, four faces 4 | point in a piece 27 |
| `how_often_a_fuse_with_a_cone_refuses.rs` answered | 15 of 96 | 15 of 96 |

**No fuse that answered on the base refuses now**; every answered count went up
or stayed, as §19.1 requires.

**Two things the lane flagged rather than claimed, and they are flagged here
too.** The 24 frustum-against-a-block walls that used to stop at *"an edge would
have four faces along it"* do **not** answer now — all 24 stop one step further
on, at `intersect` refusing a pair of surfaces, 9 as a cone crossing another
curved surface and 15 as a crossing in two pieces with no half-space square to
both. That a frustum-and-block fuse reaches a cone-against-another-curved-surface
crossing at all is unexplained; the lane's guess is that splitting the rim circle
splits the cone face in two and a later step asks `intersect` about a cone
against a cone, and it wrote that down as a guess. And the **ellipse arm earns no
count today**: everywhere it would decide, `select.rs`'s bulge count stops first.

The rim circle does **not** survive as two edges in the answer. The
pairs-of-corners census of the four answers has every pair carrying one edge: the
four faces were a naming collision during assembly, not a shape the answer keeps.

### 208.3 The zero-enclosure ladder retries a fold, and nothing asks it to

`how_much_space_a_curved_solid_encloses` now retries
`TwoPatchesOfTheMeshDrewTheSameSide` on its halving ladder; every other refusal
still breaks it.

**§16.10 verdict: neither a limit nor a gap — a guard.** The answer exists, the
arithmetic reaches it, and no solid asks. The **fold** half is live and curable:
over the two families that can put every point of a curved solid on one plane —
§41.6's cap and §206.3's sliver, 168 solids by 10 rungs — **38 of 1680 meshes
fold and all 38 give way at a finer rung**. The **zero** half is not reachable:
instrumented over the whole suite, four solids reach the halving loop at all, all
from `a_solid_whose_numbers_are_too_big_to_mesh.rs`, each stopping on its first
rung with `DeflectionBelowWhatAMeshCanReach` and `folds = 0`. §206.3's own case
is closed at source by `cut_in_two`'s two-step cut, which decoupled the zero from
the fold. Both censuses are the lane's.

**So no test fails without the arm, and it was merged anyway.** That is against
the rule that every change needs a test which fails without it, so it is written
down rather than hidden. What it rests on: the retry costs **0** extra meshes
anywhere in the gate, measured — every logged ladder had `folds = 0`; the doc
comment's termination argument came out **stronger**, because both refusals that
end the ladder are raised before a triangle is drawn while a fold is only found
by `finish`, so a solid that folds at every deflection walks the same ladder and
stops on the same rung; and taking the arm out again is six lines. A lane is out
to close the hole by building the repro, and §206.3's own caveat says where it
should be — `cut_in_two`'s narrowing is a cost bound and not a proof, so a curved
patch bounded by **more than one** closed curve can still take a bare cut.

### 208.4 The ring that closes round an apex walks, and lands on an unbounded bound

`lane-cone-apex-ring` is **on disk and not merged**. It closes the mesher gap
§206.6 named: all eight of §200.3's fuses answer, validate, and give one solid.

**§206.6's diagnosis was close and not right.** The wire does close. The walk's
longitude gain stops one step short of the loop, so the ring read
`-5.235987755982988` against a whole turn of `6.283185307179586`; `climbed`, one
line further down in `finish`, had already counted the closing step and comes to
`-6.283185307179586` exactly. The refusal now reads off `climbed`.

The design question the brief posed — which side of the ring the face is —
needed no new code. `which_pole_this_cap_reaches` already reads the wire's step
in space against `∇S × ∇T`, and for a cone both of its candidates are the apex,
so it answers when the apex is on the material side and `None` when it is not.
`None` becomes `ACurvedFaceGoesRightRoundWithNoSeam`, which is right: the other
side of the ring is the unbounded rest of the nappe and no face at all.

**Why it is not merged: it reaches `measure.rs`'s own documented gap for the
first time.** `radius_of_each_face` gives a cone face that reaches its apex a
radius of 0, so `how_much_space_hides_under_a_flat_stand_in` and
`how_much_more_surface_there_can_be` both come back `inf`. Three of the four red
tests on that branch are that and nothing else; the fourth is a refusal that
changed name, `AWireWillNotLieFlatOnItsFace` → `ACurvedFaceGoesRightRoundWithNoSeam`,
still a refusal and the accurate one.

Two of the nappe file's pins are now **unreachable rather than moved** — they
panic at *"a bound of inf checks nothing"* before reaching their tally. And
`a_pointed_cone_knows_its_inside.rs`'s volume check turns out to be **already
vacuous**: its `slack` is `measured.volume_error() + 1e-3`, which is `inf` for
any solid holding a cone's apex, so the comparison passes whatever the volume is.
Nothing in this batch made that true; it was true before, and a bare pointed cone
already published `inf` on the base.

The lane's recommendation is taken: **close the measure gap rather than move the
pins.** `radius_of_each_face`'s doc comment names the whole fix — split the face
at `v = 2·deviation·√(1 + slope²)/slope`, charge the tip its whole size — and
closes with *"its consumer is a pointed cone somebody wants a measured volume
for"*. §13 held it for want of a named consumer. There is one now, and it is this
branch.

Found on the way and not done: a ring bounding a face that **also has a hole**.
The lane's first commit let those through and
`a_corner_up_cube_round_a_frustums_apex_answers_right_or_refuses` caught it — the
fuse answered and the measure then came back `BoundaryCrossesItself` where the
face used to be refused outright. The second commit narrows the new arm to the
face's one wire, which restores that test exactly. Which of the two pieces the
ring cuts the surface into the hole sits in is nobody's answer yet.

### 208.5 A lane's self-reported sha is not evidence

`cone-rim-arcs` reported a `landed_sha` of
`39f9d7d09f3e5b9c0c0a3c6d3e5f1b9e00000000` — the right 8-character prefix, then
a placeholder tail of zeros. The branch tip is
`39f9d7d09fc41df8fd28b8d46054014d913a2904`. Its verifier caught it by running
`git rev-parse` rather than reading the field, and every other number that lane
reported reproduced exactly. Nothing was lost, and the lesson is cheap: a sha in
a lane's report is a claim like any other, and the one that counts is the one git
prints.

### 208.6 What is left

- **`measure.rs`'s cone tip has no bound**, so any solid holding an apex has an
  unusable one and `lane-cone-apex-ring` cannot merge. The fix is written out in
  `radius_of_each_face`'s doc comment and wants a bound taking two numbers rather
  than one. A lane is out.
- **One question is spelled three times** — `square_to_the_axis` in
  `fuse/select.rs`, a private copy of it in `holds.rs`, and
  `a_plane_cuts_a_cone_in_a_closed_curve` in `fuse/assemble.rs`. §23.8 says that
  is how two spellings come to disagree. The `holds.rs` copy exists because
  `mod select;` is private to `fuse`. A lane is out to make it one, and to admit
  the **ellipse** in all three at once — which is what holds the nappe floor at
  9 326, since a block's face cut by the cone carries an ellipse or hyperbola arc
  that `the_way_into_an_arc` refuses before the family test is reached.
- **The fold repro** that would pin §208.3's arm. A lane is out.
- **The 24**, from §208.2: why a frustum-and-block fuse reaches a
  cone-against-another-curved-surface crossing. Unexplained, and only a guess is
  written down.
- **A ring bounding a face that also has a hole** (§208.4).
- A stale paragraph: `a_solid_carrying_a_run_along_a_cones_ruling.rs`'s doc
  comment on `an_arc_trimmed_face_refuses_point_in_solid_by_name_and_a_closed_one_does_not`
  says what it catches is *"the top cap, a plain flat face trimmed by half a
  circle"*. After §208.1 that very cap answers; the refusal must now come from the
  flat face down the axis, whose sides are the apex rulings. The test still
  passes and no count moved — it is the prose that is stale.
- Still deferred, unchanged: §124, §136.4, §125.6, §191.6, §207.

Commands: `just decision 206.6`, `206.2`, `206.5`, `200.3`, `41.6`, `16.10`, `23.8`.
