## 446. How much of the class-B tail a tolerant policy answers — 2026-10-13

*Mattias, 2026-10-13, after asking why so few rows had closed that day and why
so many were filed for each one that did: "Would it be reasonable to think that
they would eventially start to fade away, or are we digging deeper and deeper
into more an more uncommon cases, and as we do not use tolerances as most
kernels do there isn't really and end where it will fade out, or?" The session
offered a sweep: run every open class-B row under a tolerant policy, and sort
the refusals by the function that raises them. He answered "Yes I whant it".
Q1930, filed with this section, runs it and writes 446.4 onward.*

**In short.** All 21 open class-B rows at `704063f0` were run: 282 fuses
(Q104's 5 are face questions, not fuses), each under both policies.
`Policy::certified()` refuses 219 of them. `Policy::tolerant` at the model's
size answers 51 of those 219, refuses 25 another way, and refuses 143 the same
way. By row: tolerance answers every refused run of 3 rows (Q1834, Q1854,
Q1839) and Q1842's turned runs. 3 rows answer under both policies, though
their text says refused (Q1718, Q1749, Q1791). Tolerance swaps the refusal for
another in 4 (Q1765 at pi/6, Q1737, Q1867, Q1842 turned then shifted). It
changes nothing in 11 (Q104, Q1734, Q1748, Q1777, Q1793, Q1896, Q1897, Q1903,
Q1909, Q1929, Q1936). What tolerance answers is the narrow-enclosure kind:
orient3d or side of plane at `2e-16` to `6e-13`, and a coordinate range past
the certified limit. Of the 143 it leaves, 84 are the mesher's, from 4 or 5
functions in `mesh.rs` and one in `validate.rs`, and 26 are two choices that
refuse with an infinite enclosure. Neither kind is a sign refused inside a
narrow enclosure. 18 functions raise all 219 certified refusals, or
19 if 3 runs narrowed to two functions split between them (446.5). Four of
them raise 120: `mesh.rs` `cut_up` 37, `holds.rs`
`gave_up_turning_a_corner` 32, `mesh.rs` `refusal_for` 27, and `fuse/select.rs`
`choose` 24.

### 446.1 The two questions

1. **How much of the tail a tolerance removes.** The queue cannot say. At
   `84ddac4a`, 21 of the 220 open hard rows (`refuses/`, `crash/` or `wrong/`)
   mention the tolerant policy at all. Most were run under
   `Policy::certified()` alone.
2. **Whether a few places in the code raise most of them.** If a handful of
   decision sites raise most class-B refusals, more precision at those sites
   could close many rows at once, rather than one shape per lane. §24.17
   records that resultants, the Steiner special cases and an exact algebraic
   representation of the curve were weighed for the general quartic and put
   down. A finding that a few sites dominate argues with that record. The
   argument is Mattias's to have; the lane counts.

### 446.2 What the asking session had measured

Session `dispatch-2026-10-03-a8` measured these from git and the `lanes` run
records on 2026-10-13, before the question. Its scripts were in its scratchpad
and are not kept, so these are its figures, not re-run here.

- A closed refusal row, or any other Opus row, left 1.44 to 1.53 new rows
  behind it; a small Sonnet row left 0.05. The rate did not fall with depth in
  the tree of rows that grew from one row: 1.05 at the first row, and 1.1 to
  1.75 at every depth out to 8 and past it.
- About 80 of the 404 hard rows filed since 10-03 trace back to a fuzz finding.
- The rows grow more degenerate. By the week they were filed, the share naming
  a tiny gap (2^-51, 1e-16, "a rounding") went 27%, 35%, 48%, and the share
  naming a special position (flush, tangent, apex, seam) went 43%, 53%, 61%.

Measured here, from `just queue` at `84ddac4a`: tier 1 holds 23 open rows of
class A or B (22 `refuses/`, 1 `wrong/`); tier 5 holds 166 of class C and 28
of class D. So the sweep covers the hard rows the fill takes first, and not
the larger pile behind them. A second sweep over the class-C rows would be the
follow-up, and nobody has asked for it.

### 446.3 What the row measures

**The rows**: every open row whose class is `refuses/B` or `crash/B` at the
lane's base:

```
git show <base>:docs/queue.md | grep -E '^Q[0-9]+ ' \
  | awk '$3 !~ /^(done|dropped)/ && ($6 ~ /^(refuses|crash)\/B$/ || $7 ~ /^(refuses|crash)\/B$/)'
```

At `84ddac4a` that is 23 rows, all `refuses/B`. One is parked (Q104), and two
are in lanes (Q1889 and Q1911). A row that is in a lane is run at the base like
the others.

**Each row** is run on the shape it names, under every rule and order it names,
twice:

- under `Policy::certified()`;
- under `Policy::tolerant(scale)`, where `scale` is the working size of the
  row's model. Use the size its text or its test uses, and otherwise the
  model's size in millimetres: 25 on a 25 mm model, 100 on a 100 mm one. Say
  which.

**For each run, record:**

- the outcome: answered, or the refusal's name and fields;
- the decision site: the label of the provenance record (for example "order
  along a crossing"), or the `Error` variant where no record names one;
- the function that raises it, by file and name, at the base.

A row with no shape that can be run is written down as not run, with the
reason.

**Re-measure.** Never quote a row's text or a doc comment as a result. Every
outcome in the tables is one the lane ran, with the command beside it.

**Probes** go under `/tmp`, or in a scratch test deleted before the commit.
Nothing in `src/` changes, and nothing is fixed.

**What the lane writes:**

- 446.4, a table with one line per row and run;
- 446.5, the runs grouped by the function that raised them, then by site,
  with counts under each policy. That answers question 2;
- 446.6, the rows it did not run, and why;
- an "In short" paragraph under the italic one above, saying how many rows a
  tolerant policy answers and how many functions raise how many refusals.

The table should be committed as it grows, so that a relay picks up from the
file rather than from the start.

At `704063f0`, the base Q1930 ran at, the command gives 21 rows, all
`refuses/B`: Q104, Q1718, Q1734, Q1737, Q1748, Q1749, Q1765, Q1777, Q1791,
Q1793, Q1834, Q1839, Q1842, Q1854, Q1867, Q1909, Q1896, Q1897, Q1903, Q1929
and Q1936. Q1889 and Q1911 were done by then. Q1724 had been dropped and
filed again as Q1943, with no class.

### 446.4 Each row, run under both policies

Run at `704063f0` by a scratch test, `crates/rcad-topo/tests/zz_q1930_simple.rs`
with a shared `zz_q1930_report/mod.rs`, deleted before the commit, with
`mise exec -- just t -p rcad-topo -E 'test(/q1930_probe/)' --no-capture`. A run
is one shape and one fuse. "ab" is the first-named solid first, "ba" the other
way round. U, I and D are union, intersection and difference. The tolerant
column is `tolerant(100)`, the model's size, unless it says otherwise. The site
is the predicate of each record still `Undecidable`, with its enclosure, or the
`Error` variant where no record names one. 446.5 names the function that
raises each.

| Row | Runs | Certified | Tolerant |
|---|---|---|---|
| Q1718 | lens of the thin bar (stored direction `(0, -1, 2.2e-16)`, as the row says) and the fat bar, then U with the far block both ways, D and I lens first: 4 | answers: 2, 2, 1 and 0 solids | the same |
| Q1748 | thin bar on the seam, three placements, six fuses: 18 | `Undecidable`, "a point inside a piece of a face", enclosure infinite | the same |
| Q1749 | thin bar off the seam, six placements, six fuses: 36 | answers: 1 solid in 30, none in the 6 thin-less-thick | the same |
| Q1765 | turned pi/6, wall 4 and 2 high, U and D both ways: 8 | `Undecidable`, "side of plane", `1.45e-13` | `CannotTellTheCellsOfAFaceApart` |
| Q1765 | turned 1 radian, the same 8 | `Undecidable`, "side of plane", `1.75e-13` | answers: 2 solids for U, 1 for D, 2 records |
| Q1791 | the cube less the turned cube, corner at the origin, six fuses: 6 | answers, 1 solid | answers, 1 solid |
| Q1834 | the two halves united, 5 cuts, 3 bars, both ways: 30 | `Undecidable`, "orient3d", `1.9e-16` to `8.9e-16` | answers, 1 solid, 2 records |
| Q1854 | six fuses: 6 | `Undecidable`, "orient3d", `8.5e-16` | `tolerant(25)`, the row's: answers, 1 solid, 10 records |
| Q1896 | axes meeting at the origin, U and I both ways and D ab: 5 | answers: 1, 1, 1, 1 and 2 solids | the same |
| Q1896 | at the origin D ba, at (0,1,0) D ba, at (1,1,1) D ba: 3 | `TwoPatchesOfTheMeshDrewTheSameSide` | the same |
| Q1896 | the other 8 meeting points' other fuses: 46 | `AWireWillNotLieFlatOnItsFace` 23, `ACurvedFaceGoesRightRoundWithNoSeam` 9, `CannotMeshAFace` 13 (`ASideCutsOffOneCornerOfACurvedPatch` 4, `BoundaryCrossesItself` 5, `CornersCoincide` 4), `Invalid(FacesDoNotAgreeWhichWayTheyFace)` 1 | the same line in every one of the 46 |
| Q1897 | at the origin, (1,0,0) and (11,0,0), six fuses: 18 | `AWireWillNotLieFlatOnItsFace` 9, `ACurvedFaceGoesRightRoundWithNoSeam` 4, `CannotMeshAFace` `BoundaryCrossesItself` 5 | the same line in every one |
| Q1897 | at (11,-3.25,0.5), six fuses: 6 | `Geometry(CannotNameWhichPieceOfACrossingThisIs)` | the same |
| Q1903 | the bar at (3,2,1) and at (1,2,1), D block first: 2 | `Undecidable`, "one shell inside another", enclosure infinite | the same |
| Q104 | no fuse: the puck the two `src/holds.rs` tests build, asked `face_holds_the_point` of the rim at (3,0,0) and (5,0,0) and of the lid at (0,0,1), (2,0,1) and (5,0,1): 5 | rim `AFaceMissesPartOfWhereItsSurfacesCross`, lid `AFaceTrimmedByACurveThisCannotFollow`, no records | the same |
| Q1734 | ring 4, tube 1, the copy turned pi/2, six fuses, each fuse then validate then measure: 6 | U ab, D ab and D ba answer 1 solid, validate and measure; U ba, I ab and I ba `CannotMeshAFace` `CornersCoincide`, from the fuse itself | the same |
| Q1734 | the copy turned pi, six fuses: 6 | I both ways answers 0 solids; U and D both ways `CannotMeshAFace` `CornersCoincide`, from the fuse | the same |
| Q1737 | 5 mm model, both notches, U, I and D, bar first: 6 | `Undecidable`, "order along a crossing", `2.0e-13` (seam notch) and `8.2e-13` (wall notch) | `tolerant(5)`: `CannotTellTheCellsOfAFaceApart` |
| Q1777 | six fuses: 6 | `CannotTellTheCellsOfAFaceApart` | the same |
| Q1793 | six fuses: 6 | U both ways `Geometry(CannotNameWhichPieceOfACrossingThisIs)`, I ab `ACurvedFaceGoesRightRoundWithNoSeam`, I ba and D ab `CannotMeshAFace` `CornersCoincide`, D ba `AWireWillNotLieFlatOnItsFace` | the same in each |
| Q1929 | six fuses: 6 | U and D both ways `AWireWillNotLieFlatOnItsFace`, I both ways `ACurvedFaceGoesRightRoundWithNoSeam` | the same in each |
| Q1909 | U both ways, the bar's end at 1.875, 1.9375, 2, 2.5, 3, 3.0625 and 3.125: 14 | `Geometry(WhereThreeSurfacesMeetNeedsTwoOfThemFlat)` | the same |
| Q1867 | the nine two-knife wedges, the half cut certified first: 9 | `Invalid(DynamicRangeTooWide)`, ratio `5.3e15` to `4.1e16` | `tolerant(25r)`, the model's size: `ACurvedEdgeThatIsPartOfACrossingCannotBeNamed` 7, `Geometry(CannotNameWhichPieceOfACrossingThisIs)` 2 (45 degrees at radius 1000, 10 degrees at 0.001) |
| Q1842 | the sound half bar read back, unplaced and shifted, D and I: 4 | answers: the half bar, `141.257` (45 pi), for D, nothing for I | the same |
| Q1842 | turned, D and I: 2 | `Undecidable`, "orient3d", `5.8e-13` | answers as above, 1 record |
| Q1842 | turned then shifted, D and I: 2 | `Undecidable`, "side of plane", `3.6e-15` | `TwoCurvesTouchWithoutCrossing` |
| Q1936 | the sound pyramid read back, shifted, turned and turned then shifted, D and I: 6 | `Undecidable`, "a point inside a piece of a face", enclosure infinite | the same |
| Q1839 | the tetrahedron at the four placements, D and I: 8 | answers 3 (D unplaced, 45; I turned and I turned then shifted, 15); `Invalid(DynamicRangeTooWide)` 5 (D shifted, turned, turned then shifted; I unplaced and shifted), ratio `1.3e15` to `4.5e16` | all 8 answer, 45 for D and 15 for I, 1 record each |

The rows from Q104 on were run by a second set of scratch tests, also deleted
before the commit: `zz_q1930_more.rs` (Q1734, Q1737, Q1777, Q1793, Q1929), and
copies of the rows' own test files with one `q1930_probe_` test added,
`zz_q1930_pipe.rs` (Q1909), `zz_q1930_lune.rs` (Q1867) and `zz_q1930_corner.rs`
(Q1842, Q1936, Q1839), run with the same command; 9 tests passed and printed
162 lines. Q104's puck is built by crate-private calls, so it was run by a
print put beside each assert of
`a_pucks_rounded_rim_is_refused_rather_than_holding_the_inside_of_its_ring` and
`a_pucks_lid_is_refused_rather_than_holding_nothing_but_its_rim` in
`src/holds.rs`, run with `just t a_pucks_`, and taken out again. The prints are
kept at `/tmp/q1930-probes/more.txt` and `q104.txt`.

The Q1896 and Q1897 counts come from the probe's print, one line per run, by
`grep -E '^Q1930\|Q1896\|.*\|certified\|'` and `uniq -c` on the variant. Every
one of their 156 lines matches its partner under the other policy once the
model id is taken out. The print is kept at `/tmp/q1930-probes/simple.txt`,
with the script that groups it, `short.py`.

Q1718, Q1749 and Q1791 answer at this base under both policies, where their
rows say refused. Q1791's cube may not be the shape its crash file built, since
the crash file is not in the tree: the probe built both cubes from a square
with a corner at the origin, as the row reads.

### 446.5 The refusals by the function that raises them

**How it was traced.** The same probe binaries, built at `704063f0` in the
relays' trees, were run again under `gdb`, with no change to the source. A
breakpoint went on every line outside tests that builds one of the refusals
the runs met (116 lines and `Record::new`), and each hit printed 16 frames.
A run's raising function is the last hit, before the run's report line, on a
line that builds the refusal it reported. For `Undecidable` it is the caller
of the last provenance `Record::new`, since a certified decision makes no
record. The prints (`gdb-all-*.txt`), the breakpoint list, the runner
`q1930-all.sh` and the matcher `q1930-trace.py` are at `/tmp/q1930-probes/`;
`python3 q1930-trace.py gdb-all-*.txt` prints the table's counts. Each print gave
the same report count as 446.4's runs: 392, 72, 28, 18, 44 and 10 lines.

Two limits. The test profile is opt-level 2, so a few lines have no code of
their own and their breakpoint lands a line or two on. A line written
`.ok_or(Error::...)` is hit on every call, whether it refuses or not. So
`mesh.rs:8770` and `8783` cannot be told apart (both are in `refusal_for`),
`how_it_runs` is named from its `.ok_or` line, and 5 runs are not pinned to a
line: they are listed as such below.

Counts are runs, each counted once, certified then tolerant. Lines are at
`704063f0`.

| Function, file | Site | Rows | Certified | Tolerant |
|---|---|---|---|---|
| `cut_up`, `rcad-topo/src/mesh.rs:1081` | `AWireWillNotLieFlatOnItsFace` | Q1896 23, Q1897 9, Q1929 4, Q1793 1 | 37 | 37 |
| `gave_up_turning_a_corner` from `walk_one_outline`, `holds.rs:793` | `Undecidable`, "orient3d" | Q1834 30, Q1842 turned 2 | 32 | 0 |
| `refusal_for` from `hand_over`, `mesh.rs:8770`/`8783` | `CannotMeshAFace` | Q1896 13, Q1734 7, Q1897 5, Q1793 2 | 27 | 27 |
| `choose` (`gave_up`), `fuse/select.rs:317` | `Undecidable`, "a point inside a piece of a face" | Q1748 18, Q1936 6 | 24 | 24 |
| `passes_through` (`must_decide`), `fuse/arrange.rs:1826` | `Undecidable`, "side of plane" | Q1765 16, Q1842 turned then shifted 2 | 18 | 0 |
| `check_under_policy`, `rcad-geom/src/scale.rs:291`, from `check_dynamic_range` | `DynamicRangeTooWide` | Q1867 9, Q1839 5 | 14 | 0 |
| `where_one_plane_and_two_curved_surfaces_meet`, `three_surfaces.rs:1696` | `WhereThreeSurfacesMeetNeedsTwoOfThemFlat` | Q1909 14 | 14 | 14 |
| `cut_off_what_goes_too_far_round` from `cut_up`, `mesh.rs:1404` | `ACurvedFaceGoesRightRoundWithNoSeam` | Q1896 9, Q1897 4 | 13 | 13 |
| not pinned: no breakpoint on its lines was hit | `ACurvedFaceGoesRightRoundWithNoSeam` | Q1929 2, Q1793 1 | 3 | 3 |
| `order_along` (`must_decide`), `fuse/arrange.rs:3606` | `Undecidable`, "order along a crossing" | Q1737 6 | 6 | 0 |
| `turns_anticlockwise` (`must_decide`), `fuse/arrange.rs:5087` | `Undecidable`, "orient3d" | Q1854 6 | 6 | 0 |
| `four_half_ellipses_where_two_pipes_of_one_width_meet`, `intersect.rs:1856` | `CannotNameWhichPieceOfACrossingThisIs` | Q1897 6 | 6 | 6 |
| `the_walk_of_each_other_lump_each_walk_faces`, `fuse/arrange.rs:5880` | `CannotTellTheCellsOfAFaceApart` | Q1777 6 | 6 | 6 |
| `finish`, `mesh.rs:3863` | `TwoPatchesOfTheMeshDrewTheSameSide` | Q1896 3 | 3 | 3 |
| `which_side_of_a_rings_circle`, `holds.rs:512` | `AFaceTrimmedByACurveThisCannotFollow` | Q104 3 | 3 | 3 |
| `one_shell_is_inside_another` (`gave_up`), `fuse/assemble.rs:4095` | `Undecidable`, "one shell inside another" | Q1903 2 | 2 | 2 |
| `which_of_these_circles_this_run_is`, `walk.rs:4921` | `CannotNameWhichPieceOfACrossingThisIs` | Q1793 2 | 2 | 2 |
| `what_a_ring_face_between_two_planes_holds`, `holds.rs:1060` | `AFaceMissesPartOfWhereItsSurfacesCross` | Q104 2 | 2 | 2 |
| `every_side_of_the_skin_is_used_both_ways`, `validate.rs:3080` | `Invalid(FacesDoNotAgreeWhichWayTheyFace)` | Q1896 1 | 1 | 1 |
| `spread_the_winding` from `count_the_windings`, `fuse/arrange.rs:7612` | `CannotTellTheCellsOfAFaceApart` | Q1765 pi/6 8, Q1737 6 | 0 | 14 |
| `how_it_runs`, `fuse/assemble.rs:4640` | `ACurvedEdgeThatIsPartOfACrossingCannotBeNamed` | Q1867 7 | 0 | 7 |
| `sorted_anticlockwise`, `fuse/arrange.rs:4921` | `TwoCurvesTouchWithoutCrossing` | Q1842 turned then shifted 2 | 0 | 2 |
| not pinned: no breakpoint on its lines was hit | `CannotNameWhichPieceOfACrossingThisIs` | Q1867 2 | 0 | 2 |
| answered | | | 63 | 114 |
| **all runs** | | | **282** | **282** |

`FacesDoNotAgreeWhichWayTheyFace` is built on one line outside tests, so its
function is named from that line; its breakpoint landed three lines on and was
not hit. The 3 unpinned seam runs ran in a binary whose breakpoint for
`mesh.rs:1404` landed on line 1436, and one for `1289` on 1291; both lines are
in `cut_off_what_goes_too_far_round`, and `1514` (`cut_a_band_right_round_the_ring`)
landed on 1517. So those runs are in one of those two functions.

What this says to question 2: the certified refusals come from a few places.
Four functions raise 120 of 219. Of the 143 that tolerance leaves as they
were, 84 come from the mesher (4 or 5 functions in `mesh.rs` and one in
`validate.rs`) and 26 from 2 choices whose enclosure is infinite. None of those
110 is a sign refused inside a narrow enclosure, and a tolerance did not reach
them. Whether more precision would is not measured here.

### 446.6 Rows not run

None. Every one of the 21 rows was run under both policies. Two were run
another way than through `Model::fuse`:

- **Q104** has no fuse. Its puck is built by crate-private calls, so it was run
  by a temporary print beside the asserts of its two `src/holds.rs` tests,
  taken out again before the commit (446.4).
- **Q1791**'s crash file is not in the tree. It was run on a shape built from
  the row's words, which may not be the shape the crash file built (446.4).

### 446.7 The answers checked, the rows by kind, and what Mattias chose

*Written on 2026-10-13 by the session that took over
`dispatch-2026-10-03-a8`'s handoff and talked the sweep through with Mattias.*

**The answers 446.4 counted.** Q1930's verifier checked the sums and about 20
of the cited lines, and re-ran no fuse. Its probe's "answers" is an `Ok` and a
count of solids: it ran no `validate` and measured no volume, except on Q1734.
So the two rows that answer where their text says refused were run again at
`8d91e2eb`, by a scratch test that ran each answer through `Model::validate`
and measured each solid at a deflection of `0.001`, then deleted:
`mise exec -- just t -p rcad-topo -E 'test(/zz_probe_446/)' --no-capture`.

- **Q1749**, the six placements, six fuses, both policies: 72 runs. Every
  answer is valid, and every volume is within its measured error of the exact
  one: the union `113.0879 ± 0.038` against 36π, the intersection
  `6.2816 ± 0.0062` against 2π (`12.5633` against 4π for the tall thin bar),
  thick less thin `106.8063 ± 0.044` against 34π (`100.5247` against 32π), thin
  less thick nothing. Thick less thin, which the row asked might be a limit,
  is among them.
- **Q1791**, six fuses, both policies: 12 runs, every one valid. The
  intersection is `602.1680`, the union `1397.8320` and each difference
  `397.8320`. Clipping one square by the other in Python, apart from the
  kernel, and multiplying by the height gives `602.16799`, `1397.83201` and
  `397.83201`.

Both rows simply no longer fail on the shapes they name. Nothing is filed for
them: `lane.md` has a lane run its row's shape on its base first and stop with
`does_not_reproduce`, and §433.8 then closes the row at that base.

**The rows by kind, not by run.** A run count leans on how many placements a
row tried, and Q1834 alone is 30 of the 51 runs tolerance answers. 18 of the 21
rows refuse. Sorted from each row's own text, not traced:

| Kind | Rows | Tolerant |
|---|---|---|
| Two faces touch exactly, but a point on the touch was computed or turned, so the numbers miss it by a rounding | Q1834, Q1854, Q1839, Q1765, Q1842 | answers 3, part of 2 |
| The same, and tolerance swaps the refusal for another | Q1737, Q1867 | swaps |
| Rounding has already made several things of one: four planes meet a rounding apart by an apex and leave specks (Q1936), a torus band is a rounding wide (Q1734) | Q1936, Q1734 | no change |
| The code has no road for the case: no point offered inside a piece of two sides (Q1748), a point asked on the touching line (Q1903), three surfaces with fewer than two flat (Q1909), Q104's puck | Q1748, Q1903, Q1909, Q104 | no change |
| Two thin bars crossing, given up on by the mesher; Q1929 guesses they share a cause | Q1896, Q1897, Q1929, Q1793 | no change |
| Not traced | Q1777 | no change |

The rows of the first kind show the pattern in their own words. Q1834 answers
cut through the axis and refuses at every cut off it. Q1765 answers at a
quarter turn and refuses at pi/6 and 1 radian. Q1842 answers shifted and
refuses turned. Where the numbers come out exact it answers; where they are
computed it refuses. That is the part of the tail Mattias's question was
about. The third kind is the same loss found later: the rounding has already
cut the specks, and there is no yes-or-no question left for a tolerance to
answer.

By rows, the four functions that raise 120 runs (446.5) cover 9 of the 18. And
the function that gives up is where the code stops, not why: `choose` gives up
for Q1748, a case with no road, and for Q1936, rounding specks. So a few places
holds; a few causes is not measured.

**What Mattias chose.** He agreed that Q1749 and Q1791 need no row. He asked
for three things:

- one trace row over the two-thin-bars family, to test whether a few places
  are a few causes: Q1953, which writes 446.8;
- to try out carrying coincidences from how the shape was built, the reading
  the first and third kinds point at: §448 and Q1954;
- a sweep of a random sample of 30 class-C rows, done by a lane: §449 and
  Q1955.

### 446.8 The two-thin-bars family, traced

*Written on 2026-10-14 by Q1953, a first lane and three relays, each carrying
on from the last one's commit. Every count is from a scratch test run on
`10a716f3`'s source, which the branch did not change.*

**What it found.** The four rows refuse 85 runs at the base, in six different
errors, and they come to two causes, both gaps. One test is behind 76 of them:
`assemble` asks whether two pipes are tangent at a corner by an exact `f64`
nought of `∇S × ∇T` at the corner's rounded point. Where two pipes of one width
touch, that point is one `f64` holds only when both axes lie in a coordinate
plane, so elsewhere the test reads a rounding and says no. The hair off z = 0
that Q1793 and Q1929 name matters only that way: it takes their axes out of
z = 0. The other 9 are Q1896's second bar less the first at each of its nine
meeting points: the mesher folds where the two walls touch. So the few places
446.5 found are fewer causes than places: one test is behind five different
errors. The sweep found two more causes on the same shapes at other angles,
not traced. "Each cause, gap or limit" near the end says what each is, and
what lost it, for 448.4.

Run at `10a716f3` by a scratch test, `crates/rcad-topo/tests/zz_q1953_probe.rs`,
deleted before the commit and kept at `/tmp/q1953-probes/`, with
`mise exec -- just t -p rcad-topo -E 'test(/q1953_probe_/)' --no-capture`. All
under `Policy::certified()`, at 100 mm, six fuses each. "Stored direction" is
the pipe's axis direction as the model hands it out.

**The four rows still refuse at the base, as 446.4 has them.** Q1896 answers 5
of its 54 runs (at the origin, the second bar less the first refused
`TwoPatchesOfTheMeshDrewTheSameSide`), Q1897 none of 24, Q1793 and Q1929 none
of 6 at 120 degrees.

**The stored directions are a hair off z = 0 in three of the four rows.** Q1793
and Q1929's second bar is `(-0.5, 0.866…, -2.2e-16)` at 120 degrees, as their
rows say. Q1897's two bars, built as `add_bar_cut_off_by` with a turn about
`ẑ × way`, are `(0.6, 0.8, 1.1e-16)` and `(0.8, -0.6, 1.1e-16)`. Q1896's first
bar is `(0.6, 0.8, 1.1e-16)`, its second `(0.8, 0, 0.6)` exactly.

**The hair is the cause in Q1897, Q1793 and Q1929.** Three ways round:

- **An angle sweep.** The probe turned the second bar to 41 angles: every 5
  degrees from 30 to 150, and every degree from 111 to 129. The stored
  direction is off z = 0 at 24 of them, and at all 24 both shapes refuse all
  six fuses. It lies in z = 0 at 17. There the bar starting where the axes meet
  (Q1929's shape) answers all six at 15, and the long bar through (Q1793's)
  answers all six at 14. The exceptions refuse some other way, in all six
  fuses: Q1929's shape at 85 and 95 degrees `Undecidable`, and Q1793's at 80,
  112 and 116 degrees `CannotNameWhichPieceOfACrossingThisIs`.
- **One ulp of angle.** The first three angles above 120 degrees in radians
  whose stored direction lies in z = 0 are 1, 4 and 7 ulps on. Q1929's shape
  answers all six at each of them. Q1793's answers all six at 1 and 7, and
  refuses all six `CannotNameWhichPieceOfACrossingThisIs` at 4. Moved the other
  way, the first three angles above 50, 60 and 100 degrees whose direction is
  off z = 0 refuse all six, both shapes, at all nine.
- **Q1897 laid flat.** With each bar turned about z from x by the angle of its
  way, instead of about `ẑ × way`, the stored directions are
  `(0.6000000000000001, 0.8, 0)` and `(0.8, -0.6000000000000001, 0)`. Then all
  24 of Q1897's fuses answer, at all four meeting points, including
  (11, -3.25, 0.5), where with the hair all six refuse
  `CannotNameWhichPieceOfACrossingThisIs`.

**The hair is not Q1896's cause.** Its first bar laid flat the same way, all 54
of its fuses refuse, at all nine meeting points, the origin's five included.
Q1896's two axes span a flat that is no coordinate plane, before any rounding.

**A lead, not run.** Two pipes of one width whose axes meet touch at `c ± r·n̂`,
`n̂` square to both axes. In the flat cases `n̂` is `ẑ`, and both points are
exact in `f64`. A unit vector with every coordinate a binary fraction lies along
a coordinate axis (`a² + b² + c² = 4^k` in whole numbers has only the trivial
solutions, by halving), so with the hair, and in Q1896 always, the two points
are not numbers `f64` holds. Q1793's row has the fuse asking `∇S × ∇T` for
exactly nought at those points. Whether the touch points are the one cause of
all four rows is what the trace has to show next.

**The first step that goes wrong, where the hair is the cause: `assemble`
names an arc from a touch point by a direction that is rounding.** Run at
`448acd99` by a scratch module at the end of `crates/rcad-topo/src/fuse/assemble.rs`,
removed before the commit and kept as `/tmp/q1953-probes/assemble-trace.patch`
(`git apply` it, then `mise exec -- just t -p rcad-topo -E
'test(/q1953_trace_/)' --no-capture`; its print is kept beside it as
`zz_trace_at_448acd99.txt`). It runs `Scratch::split_both_groups`, `choose`,
`assemble` and `Assembly::nest_the_shells` one at a time for the union,
intersection and difference, each shape with the hair and without: Q1929's at
120 degrees against 120 degrees + 1 ulp, and Q1897's at the origin and at
(11, −3.25, 0.5), against the same bars laid flat.

- **Everything before the arcs is the same.** The split gives the same pieces,
  the same corner names along every wire, and no record. `choose` keeps the
  same pieces. `assemble` makes the same corners and the same edges between
  them.
- **The arcs differ.** On every edge between a touch point and another corner,
  the flat build names the arc from the other corner, since `∇S × ∇T` reads
  exactly nought at the touch point. With the hair it reads
  `(1.2e-17, 6.9e-18, 0)` there in Q1929, not nought, so `which_arc_this_is`
  (`assemble.rs:7966`) names the arc from the touch point with that rounding as
  its direction. `Edge::the_way_across_the_chord_into_the_arc`
  (`assemble.rs:4808`) reads the crossing at whichever end compares first, the
  same way, so an arc named from its good end still gets a rounding for its
  side where the touch point compares first.
- **So the picker is wrong.** In Q1929's union, the arcs from the top touch
  point to (10.57, −3, 0.5) and from the bottom one to (10.86, −3.5, 0.5) are
  picked by half-spaces facing exactly the other way from the flat build's:
  `(0.00586, −0.00338, −0.0135)` against `(−0.00586, 0.00338, 0.0135)`. That
  is the other arc of the ellipse. Four more are picked by half-spaces whose
  normals are about `1e-18` long, which is rounding.
- **Then the mesher gives up**, in `nest_the_shells`, which meshes each shell
  to measure it: `AWireWillNotLieFlatOnItsFace`, `ACurvedFaceGoesRightRoundWithNoSeam`
  or `CannotNameWhichPieceOfACrossingThisIs`. Every flat build nests.

Q1793's intersection and difference differ the same way. Its union differs
earlier: with the hair `assemble` makes 2 new corners, not 6, and three
pipe-to-pipe edges all between the same two corners, where the flat build has
seven. Pieces and `choose` are the same. That fits Q1793's row, which
has `the_corners_of_each_run_between_two_tangent_points` (`assemble.rs:3420`)
asking for `∇S × ∇T` exactly nought at both ends. This was not traced
further.

**A reading, not yet argued in full.** The cause is one test, "the two
surfaces are tangent here", made as an `f64` nought of `∇S × ∇T` at a corner's
rounded point. It holds only where the touch point is an `f64` triple. The
two stored pipes still have one radius and axes that meet exactly, so they
still touch, and `intersect` still splits them by
`four_half_ellipses_where_two_pipes_of_one_width_meet`. The answer exists, so
this reads as a gap. The fact the code needs is that the corner is a touch
point, which is a fact of the crossing's own two surfaces, 448.1's (b), and the
turn does not take it away. What the turn takes away is only how exact the
point is.

**The reading demonstrated: told a touch point another way, every hair case
answers.** Q1953's third lane, at `9c9edfcf`. `assemble.rs` tests tangency by an
exact nought of `∇S × ∇T` in three places: `which_arc_this_is` (`:7983`),
`Edge::the_way_across_the_chord_into_the_arc` (`:4840`) and
`the_corners_of_each_run_between_two_tangent_points` (`:3453`). A scratch change
put all three behind one test that, with `ZZ_TOL` set, takes
`|∇S × ∇T|² ≤ (64ε)²|∇S|²|∇T|²` as tangent, and with it unset is the exact test
as it stands. It is kept as `/tmp/q1953-probes/r2/relay2-scratch.patch`, with
the probe at `r2/zz_q1953_probe.rs` and the runner `r2/zz_run.sh`
(`./zz_run.sh <out> 1 '<filter>'` sets `ZZ_TOL`). This is a probe of the cause,
not the fix: the fix carries the fact, below.

- `q1953_trace_` with `ZZ_TOL`: every hair case nests (Q1929, Q1793, Q1897 at
  the origin and at (11, −3.25, 0.5), all three rules); unset, the same binary
  refuses them as above.
- `q1953_probe_checked`, six fuses through `Model::fuse`, each answer through
  `Model::validate` and measured at a deflection of `0.001`: Q1929 and Q1793 at
  120 degrees and Q1897 at its four meeting points answer all six with the
  hair, every answer valid, and every volume equal to the flat build's within
  its measured error. The intersections against `16r³/(3 sin a)`: Q1897
  `0.08325 ± 0.00033` against `0.08333`, Q1793 `0.09608 ± 0.00116` against
  `0.09623`, and Q1929, the half, `0.04805 ± 0.00058` against `0.04811`.
- Relay 0's sweep with `ZZ_TOL`: Q1929's shape answers all six at 38 of 41
  angles, and Q1793's at 27. Q1897 laid flat answers 24 of 24 as before.
- **Q1896 shares it.** With `ZZ_TOL` it answers 45 of its 54 fuses, against 5 at
  the base, the same 5 of 6 at every one of its nine meeting points. Its touch
  points are never `f64` triples, so the exact test never fires there. The
  fourth lane ran the 45 through `Model::validate` and measured them as above,
  with `q1953_probe_q1896_checked`, added to the probe and kept as
  `/tmp/q1953-probes/r3/zz_q1953_probe.rs` (`ZZ_TOL=1 mise exec -- just t -p
  rcad-topo -E 'test(/q1953_probe_q1896_checked/)' --no-capture`, print kept as
  `r3/q1896-checked-tol.txt`). Every answer is valid, and the same at all nine
  points within `0.00001`. With `sin a = 0.8773`, the union is
  `0.68987 ± 0.00516` against `π/4 − 1/(12 sin a) = 0.69041`, the intersection
  `0.09487 ± 0.00114` against `1/(12 sin a) = 0.09499`, and the first bar less
  the second, two solids, `0.29751 ± 0.00315` against `π/8 − 1/(12 sin a) =
  0.29771`.
- **The 85 refusals at the base, by error** (the same probe, `ZZ_TOL` unset):
  `AWireWillNotLieFlatOnItsFace` 37, `CannotMeshAFace` 20,
  `ACurvedFaceGoesRightRoundWithNoSeam` 16, `CannotNameWhichPieceOfACrossingThisIs`
  8, `TwoPatchesOfTheMeshDrewTheSameSide` 3, and 1 `Invalid(FacesDoNotAgreeWhichWayTheyFace)`,
  Q1896's first bar less the second at (11, −3.25, 0.5): the fuse's own check
  caught a solid the wrong arcs had built. With `ZZ_TOL` all of them answer but
  the 9 below, which are the three `TwoPatches` and six that refused otherwise
  first.

**What is left are three other causes, each its own first step.**

- **Q1896, the second bar less the first** (the bar along `(4, 0, 3)` less the
  one along `(3, 4, 0)`), at all nine points with `ZZ_TOL`, and both
  differences of Q1896 laid flat: refused `TwoPatchesOfTheMeshDrewTheSameSide
  { drawn: 4, used: 4 }` in `nest_the_shells`. Split, `choose` and `assemble`
  give the mirror of the first-less-second assembly, with pickers of ordinary
  length. A scratch print in `mesh.rs`'s `finish` (in the same patch) names the
  side: two faces, the first bar's outer piece of pipe and the second bar's
  inner piece turned over, each cut the touch-point corner off by the same chord
  between the two mesh points next to it on its two arcs, so the same sliver
  triangle is drawn twice, facing opposite ways. At the touch point the two
  faces are tangent and the solid pinches to nothing; the retry at finer
  deflections in `how_much_space_a_curved_solid_encloses` meets the same fold
  at every rung. The answer exists, and the other difference of the same pair
  meshes, so it is a gap.
- **Q1793's shape at 80, 105, 110–114, 116, 119, 122, 125, 128, 129 and 135
  degrees, and at 120 degrees + 4 ulp**, with `ZZ_TOL`, in all six fuses:
  refused `CannotNameWhichPieceOfACrossingThisIs` in `Scratch::split_both_groups`,
  before `assemble`, with the stored direction in z = 0 or off it alike. Not
  traced past the split; 446.5 names `which_of_these_circles_this_run_is`
  (`walk.rs`) for Q1793's 2 runs of that error.
- **Q1929's shape at 85, 90 and 95 degrees**, with `ZZ_TOL`, in all six fuses:
  refused `Undecidable`, "order along a crossing", in the split, at the touch
  point (11, −3.25, 0.25). The second bar starts at the meeting point, so its
  cap plane passes through both touch points; three surfaces meet there. Not
  traced further.

**Each cause, gap or limit, and what lost it.** 448.4 asks of each refusal
which of 448.1's (a) to (d) knew the coincidence and where it was lost, and
names these rows as well if their cause is a coincidence a turn lost. None of
the four causes is.

- **The touch point read from its numbers: a gap, and the turn kept the
  coincidence.** Two pipes of one radius whose axes meet touch at `c ± r·n̂` in
  exact arithmetic, whatever the turn. Turned or not, the stored pipes have one
  radius, and `four_half_ellipses_where_two_pipes_of_one_width_meet` asks `c`
  exactly of each stored axis before it names the four halves that meet at the
  two touch points: the split gives the same pieces with the hair as without
  it. So the tangency is a fact of the crossing's own two surfaces, 448.1's
  (b), and it is still in hand when `assemble` asks. What a rounding loses is
  the touch point's place, which is not an `f64` triple. The corner holds a
  rounding of it, and the three tests in `assemble` ask that rounding instead
  of the fact. The fix carries the fact, that this corner is a touch point,
  from where `intersect` cut the ellipses there. It is not a tolerance (448.3):
  the scratch test above was a probe of the cause, not the fix.

  The row asked which of (a) to (c) knew the bar lay in z = 0. None of them.
  That is (d), the user's turn about z, and `f64` rounds it off by a hair at 24
  of the 41 angles. The fix needs nothing of it: lying in z = 0 only put the
  touch points on `f64` triples, so the exact test happened to work. Q1896's
  axes never lie in a coordinate plane, and it needs the same fact and answers
  by it.
- **Q1896's fold at the touch point: a gap, and no coincidence is lost.** In
  exact arithmetic the two walls are tangent at the touch point, and each piece
  of the answer thins to nothing there. That is the shape, not a turn or a
  rounding, so a carried fact has nothing to answer: the mesher has to cut the
  corner off once and not twice. The answer exists: the first bar less the
  second, the same pinch the other way round, meshes and validates with the
  right volume. It is the pattern of §187.4's first defect, where a cap and a
  wall cut off the same ear near the tip of an ellipse.
- **Q1793's shape at 14 angles, `CannotNameWhichPieceOfACrossingThisIs` in the
  split: untraced, so neither a gap nor a limit yet.** It refuses at 80, 112
  and 116 degrees with the direction exactly in z = 0, even at the base, so the
  hair is not its cause. Nothing here says whether a coincidence is lost.
- **Q1929's shape at 85, 90 and 95 degrees, `Undecidable` "order along a
  crossing" in the split: untraced.** One coincidence is there at every angle:
  the second bar's cap plane passes through `c` square to its axis, and `n̂` is
  square to that axis, so both touch points lie on the cap plane and three
  surfaces meet at each. The build made that coincidence, since the bar starts
  where the axes meet. But it is there too at the 38 angles that answer, so it
  is not shown to be the cause.

**Which rows each closes.** The touch-point cause: all of Q1897's 24 refused
runs, Q1929's 6, Q1793's 6, and 40 of Q1896's 49. The fold: Q1896's other 9.
At the base the fold shows at 3 of its 9 points (the origin, (0, 1, 0) and
(1, 1, 1)); at the other six the touch-point cause stops the run first. The
two untraced causes close no open row. They are shapes the sweep found.

**The rows filed**, one writing row per cause, by Q1953's report:

- the touch point: carry it from `intersect` to `assemble`'s three tests, make
  the four rows answer and pin their volumes, and turn Q1793's and Q1929's
  pinned refusals round. It closes Q1897, Q1929 and Q1793, and all of Q1896
  but the second bar less the first.
- the fold: make Q1896's second bar less the first mesh, in `mesh.rs`, and pin
  its volume. It closes the rest of Q1896. At the origin it can be worked at
  the base, so it does not wait on the first.
- Q1793's shape at its 14 angles, and in a row of its own Q1929's at 85 to 95
  degrees: name the first step that goes wrong in the split, then fix it, or
  demonstrate a limit by §16.10.

For §448: the first row carries a fact of kind (b) from where it was made to a
question that asked the numbers instead. If 448.4 finds (b) behind three of its
nine rows, its writing row is the same kind of work, and the two may share a
road.
