## 449. How much of a sample of the class-C tail a tolerant policy answers — 2026-10-13

*Mattias, 2026-10-13, after 446.7 named a class-C sweep as the follow-up and
offered a random sample of 30 rows in place of all of them: "A subagent should
probably do the class-C sweep using the random sample of 30 rows." §446 ran
the 21 class-B rows. At `8d91e2eb`, tier 5 holds 143 open class-C rows that
refuse or crash. Q1955, filed with this section, runs the sample and writes
449.4 onward.*

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
