## 446. How much of the class-B tail a tolerant policy answers — 2026-10-13

*Mattias, 2026-10-13, after asking why so few rows had closed that day and why
so many were filed for each one that did: "Would it be reasonable to think that
they would eventially start to fade away, or are we digging deeper and deeper
into more an more uncommon cases, and as we do not use tolerances as most
kernels do there isn't really and end where it will fade out, or?" The session
offered a sweep: run every open class-B row under a tolerant policy, and sort
the refusals by the function that raises them. He answered "Yes I whant it".
Q1930, filed with this section, runs it and writes 446.4 onward.*

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
`Error` variant where no record names one. **Which function raises each
refusal is not traced yet.**

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
