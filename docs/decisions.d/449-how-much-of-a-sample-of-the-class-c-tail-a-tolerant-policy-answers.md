## 449. How much of a sample of the class-C tail a tolerant policy answers — 2026-10-13

*Mattias, 2026-10-13, after 446.7 named a class-C sweep as the follow-up and
offered a random sample of 30 rows in place of all of them: "A subagent should
probably do the class-C sweep using the random sample of 30 rows." §446 ran
the 21 class-B rows. At `8d91e2eb`, tier 5 holds 143 open class-C rows that
refuse or crash. Q1955, filed with this section, runs the sample and writes
449.4 onward.*

**Result.** 29 of the 30 rows were run at `46e01222`: 579 runs, each under
both policies (Q725 names no shape, 449.7, and Q1350's 4 runs take no
`Policy`). `Policy::certified()` refuses 211 of them, 6 of those only when the
answer is measured. `Policy::tolerant` at the model's size answers 9 of the
211, refuses 60 another way, and refuses 142 the same way. §446 had 51, 25 and
143 of 219, so a tolerance reaches much less of class C than of class B. All
9 are Q1473's: 6 answer the file's closed form, and 3, on a model declared
`1e160`, answer no solid, as do the 3 runs at that size that certified answers
right. By row, tolerance answers every refused run of 1 of the 26 rows that
refuse under a `Policy`, Q1473, against 3 of 18 in §446. Q1096 and Q972
answer under both policies on the shapes they name. 24 functions raise the 211
certified refusals (449.5), about one for each row: only `order_along`,
`refusal_for` and `room_a_mesh_has_to_work_in` are met in more than one row.
The four that raise the most raise 103, from 7 rows, and 446.5's four raise 30,
from 5. By kind (449.6), 14 of the 27 rows that refuse are cases the code has
no road for, 7 are the mesher giving up, and 6 are a rounding of some sort, 2
of them of the two kinds §448 was written for. A kind 446.7's table lacks
holds Q768 and Q1021: a value a rounding or less off a special one, refused by
a test no tolerance reaches.

### 449.1 What it answers

The two questions of 446.1, for class C: how much of the tail a tolerance
removes, and whether a few places raise most of it. And a third, from 446.7:
how the rows sort by kind, and so how much of class C §448's theory could
reach.

Thirty rows of 143 give a share near a half to within about 16 points either
way, 95 times in 100 (finite population, `sqrt(0.25 / 30 * 113 / 142)` is
`0.081`, times 1.96). That is enough to tell most of the tail from a quarter of
it, and not enough to tell 30% from 40%.

### 449.2 The sample

Every open row whose class is `refuses/C` or `crash/C` at `8d91e2eb`, in the
order of the SHA-256 of its id, the first 30. The order depends on nothing in
a row's text, so nobody chose the rows.

```
git show 8d91e2eb:docs/queue.md | grep -E '^Q[0-9]+ ' \
  | awk '$3 !~ /^(done|dropped)/ && ($6 ~ /^(refuses|crash)\/C$/ || $7 ~ /^(refuses|crash)\/C$/)' \
  | python3 -c "import sys,hashlib; ids=[l.split()[0] for l in sys.stdin]; print(len(ids)); print(' '.join(sorted(ids, key=lambda i: hashlib.sha256(i.encode()).hexdigest())[:30]))"
```

It prints 143, then: Q972, Q725, Q1485, Q1559, Q1680, Q1097, Q1126, Q861,
Q1315, Q464, Q1096, Q514, Q1221, Q1020, Q1185, Q1674, Q1105, Q1350, Q566,
Q1577, Q1132, Q1021, Q1186, Q1292, Q1473, Q1584, Q758, Q768, Q1133 and Q652.

The list is fixed at that base. A row closed by the lane's base is written
down as closed, with its sha, and not replaced: replacing it would choose rows
by how they turned out.

### 449.3 What the row measures

As 446.3 and 446.5, with four differences.

- **Not every row is a fuse.** The sample holds `classify_point`, `intersect`,
  mesh, `validate` and STEP rows. Run the call the row names under both
  policies where it takes a `Policy`, and where it takes none, under the one
  it has, saying so. The tolerant policy is `Policy::tolerant(scale)` at the
  model's size, as in 446.3. A row with nothing that can be run is written
  down as not run, with the reason.
- **The raising function** may be named from the one line that builds the
  refusal, where only one line builds it. Use `gdb` as 446.5 did only where
  several lines build it.
- **Each row's kind**, by 446.7's table, from what the run showed and not from
  the row's text. A kind the table has no line for is named.
- **Answers are checked.** An answer is run through `Model::validate`, and a
  solid's volume is measured where the row gives or implies one. 446.4
  counted solids only (446.7).

**Re-measure.** Never quote a row's text or a doc comment as a result. Every
outcome in the tables is one the lane ran, with the command beside it.

**Probes** go under `/tmp`, or in a scratch test deleted before the commit.
Nothing in `src/` changes, nothing is fixed, and no rows are filed. A row that
answers on its shape is named in 449.4; a lane that takes it later closes it
(§433.8).

**What the lane writes:**

- 449.4, a table with one line per row and run, as 446.4;
- 449.5, the runs grouped by the function that raised them, as 446.5;
- 449.6, the rows by kind, and the shares beside §446's: answered by
  tolerance, by kind, and by the four functions 446.5 named;
- 449.7, the rows not run, and why;
- a **Result.** paragraph under the italic one above.

The table should be committed as it grows, so that a relay picks up from the
file rather than from the start.

### 449.4 Each row, run under both policies

Run at `46e01222` by scratch copies of each row's own test file with one
`q1955_probe_` test added and a shared `zz_q1955_report/mod.rs`, deleted
before the commit, with `mise exec -- just t -p rcad-topo --test <copy> -E
'test(/q1955_probe/)' --no-capture`. Each answer is run through
`Model::validate_solid` and measured at a deflection of `0.01` under the run's
own policy. "ab" is the first-named solid first. The tolerant column is
`tolerant` at the model's declared size, named in the row. The copies and the
prints are kept at `/tmp/q1955-probes/`; `python3 short.py <print> -s <row>`
groups a print by shape and outcome.

| Row | Runs | Certified | Tolerant |
|---|---|---|---|
| Q1126 | block 10 x 10 x 4 less the ball of 2 at (5, 5, 2), block first, 100 mm: 1 | `Undecidable`, "one shell inside another", enclosure infinite, 1 record | `tolerant(100)`: the same |
| Q1221 | the near block and the other, 50 mm, six fuses: 6 | U ab answers `2280`, I ba `60`, D ba `840`, each valid; U ba, I ab and D ab `ResultIsNotManifold { count: 1 }` | `tolerant(50)`: the same, run for run |
| Q1473 | wedge and block, 100 mm, six fuses: 6 | U both ways and D ab (wedge less block) `Undecidable`, 2 "orient3d" records, `1.7e-15`; I both ways and D ba `Invalid(DynamicRangeTooWide)`, ratio `2.7e16` | `tolerant(100)`: all 6 answer and validate: U `4.0878`, I `0.0757`, D ab `3.3878`, D ba `0.6243`, against `4.08782`, `0.07571`, `3.38782` and `0.62429` from the file's closed form |
| Q1473 | the same on a model declared `1e160`: 6 | U both ways and D ab answer, valid, the same volumes, 2 "orient3d" records left in the log; I both ways and D ba `DynamicRangeTooWide`, `2.7e16` | `tolerant(1e160)`: all 6 answer no solid, "a shell within the wobble of one of its planes taken as flat" |
| Q1559 | ball of 2 and bar of 1 at x = 1 touching it inside, 100 mm, six fuses: 6 | `Geometry(ACrossingThatTouchesItselfIsNeitherOneCurveNorTwo)` | `tolerant(100)`: the same |
| Q1577 | top half of ring 4 tube 1 and the quarter tube moved `1.1e-16`, `-5.55e-17`, `1e-12`, `0.25` and `-0.25`, six fuses each, 100 mm: 30 | `Undecidable`, "order along a crossing": `4.44e-15` ring first and `0` ring second at the three hairs, `4.41e-15` in all 12 at `±0.25` | `tolerant(100)`: `TwoCurvesTouchWithoutCrossing` 21 (ring first at the hairs, all at `±0.25`), `Invalid(EdgeHasNoLength)` 9 (ring second at the hairs) |
| Q1584 | quarter bend 4/1 and the ball of 3.5, the ball of 4.5 and the frustum, each turned 5pi/4, six fuses each, 100 mm: 18 | `Undecidable`, "order along a crossing": `3.0e-15`, `7.8e-15` and `3.2e-15` | `tolerant(100)`: `CannotTellTheCellsOfAFaceApart` 9 (bend first), `TwoCurvesTouchWithoutCrossing` 9 (bend second) |
| Q1096 | no fuse: `classify_point` of (0, 4, 0), (4, 0, 1/2) and (0, 0, 0) on the ring of 4 cut along its equators and along y = 0, 100 mm: 6 | answers `Inside`, `Inside`, `Outside` on both rings; no panic | `tolerant(100)`: the same |
| Q1097 | rings of 4 and 5, tube 1, about z, 100 mm, six fuses: 6 | `Geometry(ACrossingBetweenATorusAndAnotherCurvedSurface)` | `tolerant(100)`: the same |
| Q464 | the leaning-block family rebuilt from the file's own `leaning_blocks(THE_STEEP_CONE, …)` and `build`: leans 0.25, 0.375, 1 and 4, turns 0, 1 and 3, crossings 0, 0.5, −0.5 and 1.5, union, intersection and both differences, 100 mm: 192 | 180 answer (174 one solid, 3 two, 3 none), every one valid; 178 measure the file's closed form within their bound; 2 (lean 1, turn 0, crossing 0, union and cone less block) measure-refuse at `0.01` `CannotMeshAFace` `ASideCutsOffOneCornerOfACurvedPatch { corner: 9 }`; 12 `CannotTellTheCellsOfAFaceApart` (lean 4, crossing 0, every turn and rule). No "a point inside a piece of a face" | `tolerant(100)`: the same, run for run |
| Q514 | frustums 2..4 about z and 4..6 about −z, union both ways, 100 mm: 2; then each union with the block `[0,5] x [-5,5] x [3,5]`, six fuses: 12 | the unions answer one solid of 4 faces, valid, `54.4032 ± 0.56` against `54.4543`; all 12 next fuses `TwoCurvesTouchWithoutCrossing` | `tolerant(100)`: the same |
| Q758 | quarter bends x to y about z and y to −x about −z, union both ways, 100 mm: 2 | `Geometry(WhereThreeSurfacesMeetNeedsTwoOfThemFlat)` | `tolerant(100)`: the same |
| Q972 | the pointed cone of slope 1, base z = 20, and the block `[-12,24] x [-24,24] x [-1,18]`, six fuses, 100 mm: 6 | all answer, valid, within their bound: U `35359.39 ± 4.95` against `35359.76`, I `5849.07 ± 12.8` against `5849.83`, D cone first `2527.39`, D block first `26982.93` | `tolerant(100)`: the same |
| Q1185 | the bored block less the bar at (4, 0) and the foot, foot at `[-8,-5] x [4,7]` and moved to `[-8,-5] x [-2,2]`, z 4..6, six fuses each, 100 mm: 12 | pinned foot: all 6 answer, valid, on the closed forms (U `1511.26 ± 0.44` against `1511.19`); moved foot: 5 answer, valid, D bored first `TwoPatchesOfTheMeshDrewTheSameSide { drawn: 2, used: 4 }` | `tolerant(100)`: the same, run for run |
| Q1485 | the notched plate with its square hole and the wedge, x = −4 and −5.5, six fuses each, 100 mm: 12 | `Undecidable`, 1 record, "a point inside a piece of a face", enclosure infinite, all 12 | `tolerant(100)`: the same, all 12 |
| Q1186 | no fuse: the tip meshed at `1e-300`, under certified and three tolerances, 100 mm: 4; at `1e-6` certified: 1 | `1e-300`: `DeflectionBelowWhatAMeshCanReach`, closest `4.55e-11`; `1e-6`: meshes, 1404459 points; the model validates | `tolerant(100)`: as certified; `tolerant(1e4)` and `tolerant(1e9)`: `Geometry(TooManyPointsAlongOneCrossing { most: 1048576 })`. `tolerant(100)` at `1e-6` ran past 10 minutes and was stopped |
| Q514 | control: one frustum 2..6 about z and the same block, six fuses, 100 mm: 6 | all answer, valid | `tolerant(100)`: the same |
| Q861 | the pointed cone and `a_leaning_block(0.25, 0, 1)`, I cone first and D block first, measured at 0.002 to 0.05, 100 mm: 2 | both answer and validate; measure refuses `CannotMeshAFace` `ASideCutsOffOneCornerOfACurvedPatch` at 0.002, 0.005 and 0.01 (corners 19, 68, 9) and measures right at 0.02 (`26.15 ± 0.76` against `26.21`, `1773.85 ± 0.76` against `1773.79`) | `tolerant(100)`: the same |
| Q566 | no fuse: the bar in two bands, every pair of wall spellings, lower band first, unturned and turned x 0.25, x 1.5 and y −0.75, meshed at 0.01, 100 mm: 16 | unturned: meshes, valid, `12.5358 ± 0.12` against `4 pi`; turned: `DeflectionBelowWhatAMeshCanReach`, closest `105926`, `217946` and `308824` | `tolerant(100)`: the same, run for run |
| Q652 | no fuse: the bar of radius 3, z 0..4, its top rim naming the lid one ulp up, built pipe first and lid first, 100 mm: 2 | pipe first builds, valid, `113.0034 ± 0.38` against `36 pi`; lid first `AWireDoesNotFanIntoALoopToMesh { TooFewCorners { count: 1 } }` | `tolerant(100)`, the build under it: the same |
| Q1020 | ball of 1 and the cube of 4 turned 30 degrees, directions 0, 30, 180, 250, roundings ±1.01, ±0.99, ±0.5, six fuses each, 100 mm: 144; then `classify_point` at the file's 4 points on each answer | 132 answer one solid, valid; all 528 points classified right, no `CannotFollowAFacesTrimming`; 12 `CannotMeshAFace` `LoopsTouch` | `tolerant(100)`: the same, run for run |
| Q1133 | no fuse: `face_holds_the_point` on the cap at the file's 5 points, the four spellings by hole first, 125 mm: 8 | `NothingSaysWhichSideOfItsTrimmingAFaceLiesOn` at every point; the solid validates and measures `261.53 ± 1.57` against `261.80` | `tolerant(125)`: the same |
| Q1674 | no fuse: the bar whose lid is split by the half disc `1e-14` off it, its arc naming the parabola cone, half first and second, built, 10 mm: 2 | the build refuses `Invalid(AnArcBetweenTwoFlatFacesOnDifferentPlanes)` | `tolerant(10)`, the build under it: the same |
| Q1350 | no fuse: `is_a_family_this_follows` on the row's families, `x ≥ 3`, `x ≥ z`, `y ≤ 4` on the ball of 5, `x ≥ 1`, `y ≥ 2`, `x + y + z ≤ 5.1` on the ball of 3, `x, y ≥ 4`, `z ≤ 7` on the ball of 9, and the control `x, y ≥ 3`, `x + y ≤ 7` on the ball of 5, by the row's own two pins as they stand: 4 | all four refused; the grid counts one piece, one, one and two | takes no `Policy`: not run under one |
| Q1292 | no fuse: `meet_of_three_surfaces` of the ball of 5 at the origin written twice with the pipe of 3 up `z`, and with the ball of 5 at (6, 0, 0), the repeat first and last: 4 | `WhereThreeSurfacesMeetNeedsTwoOfThemFlat`, empty log | `tolerant(100)` and `tolerant(10)`: the same |
| Q1132 | no fuse: `intersect` of the cone of slope 1 about `z` with its apex at the origin and three balls through the apex, 2 at (0, 0, 2), 5 at (0, 0, 5) and 5 at (0, 0, −5), each order: 6 | `ACrossingBetweenAConeAndAnotherCurvedSurface` | `tolerant(100)` and `tolerant(10)`: the same |
| Q1680 | no fuse: `walk` of the short run of the thin pipe through the fat one in the row's test, picked by `y ≥ 0` alone and by `y ≥ 0` with `z ≤ 4.5`, at deflections 1 and 0.01, each order: 8 | `y ≥ 0` alone: `CannotNameWhichPieceOfACrossingThisIs`, all 4; with `z ≤ 4.5`: walked, no point between the corners, but 1 at deflection 1 thin pipe first | `tolerant(100)` and `tolerant(10)`: the same, run for run |
| Q768 | the bar of 2, 8 tall, leaning `4.78e-299`, `6.2e-320` and `8e-323` off its caps, less and with the block 2 at (1, −1, 3), six fuses each, built and read back from its own file, 8 mm: 36 | `4.78e-299` and `8e-323`: `Geometry(CannotNameWhichPieceOfACrossingThisIs)`, all 24; `6.2e-320`: `Undecidable`, 1 record, "where two planes cross", `5e-324`, all 12 | `tolerant(8)`: `CannotNameWhichPieceOfACrossingThisIs`, all 36, so at `6.2e-320` tolerance swaps the refusal for the row's |
| Q1105 | half the bar of 2 whose edge across the floor is named by the pipe, and the block of side 2 at (−1, −1, −1), six fuses, 4 mm: 6; then the same read back from its own file, that edge straight: 6 | built: `AFaceTrimmedByACurveThisCannotFollow`, all 6; read back: all 6 answer, valid, U `31.1035 ± 0.25`, I `2`, D half first `23.1035 ± 0.25`, D block first `6`, against `31.1327`, `2`, `23.1327` and `6` | `tolerant(4)`: the same, run for run |
| Q1021 | no fuse: `Import::read` of the pointed cone whose half-angle is written `26.5650511771` degrees, and the control written `0.463647609001` radians: 2 | degrees: `ALoopThisReaderDoesNotWalk`, `VERTEX_LOOP` #132 on face #135; radians: imports, valid, `133.9746 ± 0.60` against `134.0413` | `tolerant(8)` and `tolerant(100)`: the same |
| Q1315 | no fuse: the round bar of 5 by 13 turned `-1.793662034335766e-43` about `y` at (8, −28, 39), `1e-27` about `y` at the origin and `1e-43` about `x` at (8, −28, 39), measured, written, read and measured at `Deflection::for_scale`, 100 mm: 6 | built: valid, `1016.12 ± 19.7` against `1021.02`, all 3; read back at (8, −28, 39), both turns: valid, measure refused `CannotMeshAFace` `CoordinateOutOfRange { TooSmall }`; `1e-27` at the origin: valid, `1020.36 ± 2.64` | `tolerant(100)`, the read under it too: the same, run for run |

Q1350, Q1292, Q1132 and Q1680 take no model. Q1350 was run by its own pins,
`mise exec -- just t -p rcad-topo
a_circle_touching_or_biting_what_two_planes_keep_is_refused
a_plane_just_past_the_corner_two_others_meet_at_cuts_its_tip_off`, which
assert the refusals and the grid counts. Q1292, Q1132 and Q1680 were run by
one scratch test added to `walk.rs`'s tests, kept as
`/tmp/q1955-probes/zz_q1955_geom_walk.patch`, with `mise exec -- just t -p
rcad-geom --lib q1955_probe_geom --no-capture`, print `run-5.txt`. Q768,
Q1105, Q1021 and Q1315 were run by copies of their two `rcad-step` test
files, print `run-6.txt`. Where no model names a size, the tolerant runs take
`tolerant(100)` and `tolerant(10)`.

Q1105's built solid is refused `AFaceTrimmedByACurveThisCannotFollow`, not
the "edge that runs along part of a curve" the row names, and the solid read
back cuts, as the row says. Q1315's bar turned `1e-27` about `y` at the origin
measures at this base; the row's two shapes moved to (8, −28, 39) still
refuse.

Q1096's panic is not met at this base: neither ring panics, and both answer
as the row says they should. Its second case, pair 3's fuse past Q618, is not
run: Q618 is open at this base, and pair 3's fuses refuse before `choose`.
Q1473's `ResultIsNotManifold { count: 4 }` is not met either: the wedge less
the block is refused `Undecidable` under `Policy::certified()` and answers its
closed form under `tolerant(100)`.

### 449.5 The refusals by the function that raises them

**How it was traced.** Traced at `b19495b3`, whose `crates/` equals
`46e01222`'s (`git diff --stat 46e01222 b19495b3 -- crates/` prints nothing),
so the probe binaries of 449.4, built there, run the code 449.4 ran. They were
run again under `gdb`, with no change to the source, as 446.5 did. A first
pass put a breakpoint on every line outside tests that builds one of the
refusals the runs met, and on `Record::new`. A second pass, for the refusals
the first could not pin, put one on the entry of every function that holds
such a line, and on `ProvenanceLog::push`. Each hit printed 16 frames. A run's
raising function is the last hit before the run's report line on a line, or in
a function, that builds the refusal it reported; for `Undecidable`, the caller
of the last provenance record, since a certified decision makes none. The
runner `q1955-all.sh`, the breakpoint lists `sites.txt` and `sites2.txt`, the
matchers `trace.py` and `trace2.py`, and their matched outputs `trace-1.txt`
and `trace-2.txt` are kept at
`/home/mattias/rcad-dead-run-probes-2026-10-14/q1955-b-traces/`. The `gdb`
prints themselves are gone. Since `crates/` did not change, every refusal
449.4 recorded was raised again, with the same counts.

**Limits.** The test profile is opt-level 2, so some breakpoints landed on
another line and were not hit. Those refusals were pinned in the second pass,
by function entry: Q1559, Q1132, Q1292, Q768's
`CannotNameWhichPieceOfACrossingThisIs`, and the
`DeflectionBelowWhatAMeshCanReach` of Q566 and Q1186. Where that function has
two lines that build the refusal, the table names both. A line written
`.ok_or(...)` or `.map_err(...)` is hit on every call, refusing or not:
`fuse/assemble.rs:1573` is the only line that builds `ResultIsNotManifold`,
and `holds.rs:902` is the last such line passed before Q1133's call returned.
Q1020's `LoopsTouch` hit the triangulator's line but no breakpoint in
`refusal_for`. It is named from the code: `hand_over` sends every error of the
`triangulate` call at `mesh.rs:3548` through `refusal_for` at `3559`, and
`LoopsTouch` returns at `8861`. `CannotMeshAFace` carries the triangulator's
own reason, raised by `refine_the_inside`, `rcad-mesh/src/triangulate.rs:1892`,
for Q464 and Q861; `refuse_a_side_with_a_corner_on_it`, `1105`, for Q1020; and
`the_refusal_the_flush_replaced`, `767`, for Q1315, a line that was hit, though
`160`, a `.map_err` line that builds the same reason, had no breakpoint.

Counts are runs, each counted once, certified then tolerant at the model's
size. Lines are at `b19495b3`.

| Function, file | Site | Rows | Certified | Tolerant |
|---|---|---|---|---|
| `order_along` (`must_decide`), `fuse/arrange.rs:3746` | `Undecidable`, "order along a crossing" | Q1577 30, Q1584 18 | 48 | 0 |
| `two_lines_told_apart` from `where_a_plane_and_a_pipe_cross`, `rcad-geom/src/intersect.rs:3215`/`3222` | `CannotNameWhichPieceOfACrossingThisIs` | Q768 at `4.78e-299` and `8e-323` 24; tolerant, all 36 | 24 | 36 |
| `refusal_for` from `hand_over`, `mesh.rs:8861` (`LoopsTouch`) and `8874` | `CannotMeshAFace` | Q1020 12 in the fuse; Q464 2, Q861 2 and Q1315 2 when measured | 18 | 18 |
| `room_a_mesh_has_to_work_in`, `mesh.rs:551` | `DeflectionBelowWhatAMeshCanReach` | Q566 12, Q1186 1 | 13 | 13 |
| `choose` (`gave_up`), `fuse/select.rs:317` | `Undecidable`, "a point inside a piece of a face" | Q1485 12 | 12 | 12 |
| `the_walk_of_each_other_lump_each_walk_faces`, `fuse/arrange.rs:6021` | `CannotTellTheCellsOfAFaceApart` | Q464 12 | 12 | 12 |
| `what_a_turn_at_the_apex_is_about`, `fuse/arrange.rs:5416` | `TwoCurvesTouchWithoutCrossing` | Q514 12 | 12 | 12 |
| `where_two_planes_cross`, `rcad-geom/src/intersect.rs:282` | `Undecidable`, "where two planes cross" | Q768 at `6.2e-320` 12 | 12 | 0 |
| `what_a_curved_face_holds`, `holds.rs:902` | `NothingSaysWhichSideOfItsTrimmingAFaceLiesOn` | Q1133 8 | 8 | 8 |
| `check_under_policy`, `rcad-geom/src/scale.rs:291`, from `check_dynamic_range`, `fuse/assemble.rs:609` | `DynamicRangeTooWide` | Q1473 6, 3 at 100 mm and 3 at `1e160` | 6 | 0 |
| `where_a_ball_and_a_pipe_cross`, `intersect.rs:737` | `ACrossingThatTouchesItselfIsNeitherOneCurveNorTwo` | Q1559 6 | 6 | 6 |
| `intersect`, `intersect.rs:256` | `ACrossingBetweenATorusAndAnotherCurvedSurface` | Q1097 6 | 6 | 6 |
| `where_a_cone_and_a_ball_cross`, `intersect.rs:4421`/`4446` | `ACrossingBetweenAConeAndAnotherCurvedSurface` | Q1132 6 | 6 | 6 |
| `place_the_piece`, `fuse/select.rs:712` | `AFaceTrimmedByACurveThisCannotFollow` | Q1105 6 | 6 | 6 |
| `where_three_curved_surfaces_meet`, `three_surfaces.rs:1601`/`1626` | `WhereThreeSurfacesMeetNeedsTwoOfThemFlat` | Q1292 4 | 4 | 4 |
| `the_arc_between`, `rcad-geom/src/walk.rs:1418` | `CannotNameWhichPieceOfACrossingThisIs` | Q1680 4 | 4 | 4 |
| `darts_along_each_edge`, `fuse/assemble.rs:1573` | `ResultIsNotManifold` | Q1221 3 | 3 | 3 |
| `decide`, `classify.rs:1761`, its `orient3d_tolerantly`, from `look_at_triangle` and `count_a_polygon` | `Undecidable`, "orient3d" | Q1473 3 | 3 | 0 |
| `where_one_plane_and_two_curved_surfaces_meet`, `three_surfaces.rs:1696` | `WhereThreeSurfacesMeetNeedsTwoOfThemFlat` | Q758 2 | 2 | 2 |
| `check_flat_faces_put_an_edge_in_one_place`, `validate.rs:581` | `Invalid(AnArcBetweenTwoFlatFacesOnDifferentPlanes)` | Q1674 2 | 2 | 2 |
| `one_shell_is_inside_another` (`gave_up`), `fuse/assemble.rs:4095` | `Undecidable`, "one shell inside another" | Q1126 1 | 1 | 1 |
| `finish`, `mesh.rs:3895` | `TwoPatchesOfTheMeshDrewTheSameSide` | Q1185 1 | 1 | 1 |
| `hand_over`, `mesh.rs:3528` | `AWireDoesNotFanIntoALoopToMesh` | Q652 1 | 1 | 1 |
| `loop_of_bound`, `rcad-step/src/brep.rs:642` | `ALoopThisReaderDoesNotWalk` | Q1021 1 | 1 | 1 |
| `sorted_anticlockwise`, `fuse/arrange.rs:5062` | `TwoCurvesTouchWithoutCrossing` | Q1577 15, Q1584 3 | 0 | 18 |
| `leaves_the_node_the_same_way`, `fuse/arrange.rs:5537` | `TwoCurvesTouchWithoutCrossing` | Q1577 6, Q1584 6 | 0 | 12 |
| `check_edge`, `validate.rs:392` | `Invalid(EdgeHasNoLength)` | Q1577 9 | 0 | 9 |
| `spread_the_winding` from `count_the_windings`, `fuse/arrange.rs:7753` | `CannotTellTheCellsOfAFaceApart` | Q1584 9 | 0 | 9 |
| answered | | | 368 | 376 |
| not run under `tolerant` | | Q1186 at `1e-6` (449.7) | | 1 |
| **all runs** | | | **579** | **579** |

The counts and the pairing come from
`python3 /home/mattias/rcad-dead-run-probes-2026-10-14/q1955-probes/shares.py`
over `run-1.txt` to `run-6.txt` in the same directory, which pairs each run's
certified line with its first tolerant line, the one at the model's size. Its
`ALL` line prints 579 runs; 205 refused in the call and 6 when measured; 368
answered; 9 that tolerance answers, 60 it swaps, 142 it refuses the same way.
"Answered" counts an answer of no solid: Q1473's 6 runs under
`tolerant(1e160)` all answer none, "a shell within the wobble of one of its
planes taken as flat", the 3 certified answers right among them. Whether no
solid is what a tolerance that wide should give is not settled here.

Q1132, Q1292, Q1680 and Q1021 also ran a second tolerance (449.4), and the
tolerant lines of `trace-1.txt` and `trace-2.txt` count those runs too (Q1132
12, Q1292 8, Q1680 8, Q1021 2): every one was raised by the same function. Q1186
ran two more, `tolerant(1e4)` and `tolerant(1e9)`, both refused
`TooManyPointsAlongOneCrossing` by `how_many_pieces_an_endless_bend_takes`,
`rcad-geom/src/walk.rs:1983`. Q1350's 4 runs are in no count here: they take no
`Policy`, and its pins assert that `is_a_family_this_follows`, `holds.rs:5174`,
answers no for each family, which is the refusal its row names.

Several refusals come from a mesh or a check run inside the call, not from
the work the call is named for. Q1185's `finish` ran in a mesh that
`validate.rs`'s `the_coarsest_mesh_whose_sides_do_not_cross` asked for inside
the fuse; Q1020's `LoopsTouch` is a mesh refused inside the fuse, and Q652's a
mesh refused inside the build. Q1577's tolerant `EdgeHasNoLength` comes from
`check_everything_but_the_coordinate_range_in`, which the fuse reaches only
through the check its `commit` runs on its result, `fuse/assemble.rs:739`.

What this says to question 2: the class-C tail does not come from a few
places. 24 functions raise the 211 certified refusals, across the 26 rows that
refuse under a `Policy`, and only three are met in more than one row:
`order_along` (Q1577, Q1584), `refusal_for` (Q1020, Q464, Q861, Q1315) and
`room_a_mesh_has_to_work_in` (Q566, Q1186). The four that raise the most,
`order_along` 48, `two_lines_told_apart` 24, `refusal_for` 18 and
`room_a_mesh_has_to_work_in` 13, raise 103 of 211, but from 7 rows. In §446, 18
functions raised 219, and the top four 120, from 9 of 18 rows (446.7). 446.5's
four functions raise 30 of the 211 here, `choose` 12 and `refusal_for` 18, from
5 rows (Q1485, Q1020, Q464, Q861, Q1315); `cut_up`'s
`AWireWillNotLieFlatOnItsFace` and `gave_up_turning_a_corner` are not met.

### 449.6 The rows by kind, and the shares beside §446's

Sorted by 446.7's kinds from what the runs of 449.4 and the trace of 449.5
showed, not from the rows' text. 27 rows refuse: the 26 that refuse under a
`Policy`, and Q1350. Q1096 and Q972 answer under both policies on the shapes
they name. What 449.4 found not met at this base is not sorted: Q464's "a
point inside a piece of a face", Q1020's `CannotFollowAFacesTrimming`, Q1096's
panic and Q1473's `ResultIsNotManifold`.

| Kind | Rows | Tolerant |
|---|---|---|
| Two faces touch exactly, but a point on the touch was computed or turned, so the numbers miss it by a rounding (446.7's first) | Q1473: "orient3d" at `1.7e-15`, and a coordinate range of `2.7e16` | answers all 6 at 100 mm |
| The same, and tolerance swaps the refusal for another | Q1577, "order along a crossing" at `4.4e-15` and `0`; Q1584, at `3.0e-15` to `7.8e-15`; Q768's 12 runs at `6.2e-320` | swaps |
| Rounding has already made several things of one | Q1221: two blocks overlap on one plane spelled an ulp apart, and 3 of 6 fuses are `ResultIsNotManifold` | no change |
| **New:** a lean or a value a rounding or less off a special one, refused by an exact test or by a reader that no tolerance reaches | Q768's 24 runs at `4.78e-299` and `8e-323`, refused by `two_lines_told_apart`; Q1021, whose half angle written in degrees is refused where the same angle written in radians imports, both cut to 12 digits | no change |
| The code has no road for the case | Q1126 (the ball's poles touch the block, and `one_shell_is_inside_another` gives up with an infinite enclosure, as for Q1903), Q1485 (`choose`, an infinite enclosure), Q1105, Q514 (a cone face made of two frustums; the control made as one answers), Q758, Q1292, Q1559, Q1097, Q1132, Q1133, Q1350, Q1674 (a lean of `1e-14` that `tolerant(10)` did not take in), Q1680, and Q464's 12 cells, refused alike at every turn, where a rounding would change with the turn | no change |
| The mesher gives up (446.7's two thin bars, widened: none of these is two thin bars) | Q566, Q1186, Q652, Q1020, Q1185, Q1315, Q861; and Q464's 2 measured runs, Q464 being counted above | no change |

The rows, kind by kind, are 1, 2, 1, 2, 14 and 7: Q768 is counted once, in the
new kind, though its runs at `6.2e-320` are of the second.

| Share | §449, class C | §446, class B |
|---|---|---|
| runs refused under certified | 211 of 579 | 219 of 282 |
| of those, tolerance answers | 9, 6 of them right | 51 |
| tolerance refuses another way | 60 | 25 |
| tolerance refuses the same way | 142 | 143 |
| rows whose every refused run tolerance answers | 1 of 26 | 3 of 18 |
| rows of the first kind | 1 of 27 | 5 of 18 |
| of the second | 2, and Q768's runs at `6.2e-320` | 2 |
| of the third | 1 | 2 |
| of the new kind | 2 | none named |
| no road | 14 | 4 |
| the mesher | 7 | 4 |
| not traced | none | 1 |
| runs 446.5's four functions raise | 30 of 211 | 120 of 219 |
| rows they raise them in | 5 of 26 | 9 of 18 |

A row counts as one draw of the 30. 1 of 26 rows has a 95% Wilson interval of
`0.007` to `0.189`, before the finite-population factor, which only narrows it,
so the share of class-C rows a tolerance answers whole is told from a quarter,
as 449.1 asked. No road, 14 of 27, gives `0.34` to `0.693`, and the mesher, 7 of
27, `0.132` to `0.447`.

What this says to question 3: the two kinds §448 was written for, the first
and the third, are 2 of the 27 rows, Q1473 and Q1221. With the second kind and
the new one, a rounding of some sort is behind 6 of 27, against 9 of 18 in
§446. Whether carrying a coincidence from how the shape was built reaches any
of the six is not measured here. Most of class C is a case the code has no
road for, 14 rows, or the mesher giving up, 7, and neither is a sign refused
inside a narrow enclosure.

### 449.7 Rows not run, and what was not checked

- **Q725** is not built. Its row names
  `Model::room_a_corner_of_this_solid_gets` in `validate.rs` and says the
  shape it asks about is not built: building it is the row's own work, so
  there was nothing to run.
- **Q972**'s named shape was run, and answers (449.4). The shape its row says
  still refuses, one "like it that shares no side", is not named, so it was not
  run.
- **Q1096**'s second case, pair 3's fuse past Q618, was not run: Q618 is open
  at this base, and pair 3's fuses refuse before `choose`.
- **Q1186** under `tolerant(100)` at a deflection of `1e-6` ran past ten
  minutes and was stopped. Its certified run at `1e-6` meshes.
- **Q1350** takes no `Policy`, so it was run once, by its two pins, and not
  under a tolerance.

Not checked: Q1680's 4 answered walks, with `z ≤ 4.5`, were not held against
an expected answer; and Q1020's 132 answers were validated and had their
points classified, but their volumes were not held against a closed form.
