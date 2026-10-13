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

Q1096's panic is not met at this base: neither ring panics, and both answer
as the row says they should. Its second case, pair 3's fuse past Q618, is not
run: Q618 is open at this base, and pair 3's fuses refuse before `choose`.
Q1473's `ResultIsNotManifold { count: 4 }` is not met either: the wedge less
the block is refused `Undecidable` under `Policy::certified()` and answers its
closed form under `tolerant(100)`.
