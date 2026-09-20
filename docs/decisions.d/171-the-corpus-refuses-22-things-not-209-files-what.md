## 171. The corpus refuses 22 things, not 209 files — what lanes are cut from, 2026-09-20

*Asked how several people could work here at once, the answer turned out not to
be a scheduler. `blocker_depth_probe` already computes the list. Over both
corpora **22 reasons are the only thing standing between some file and an
import**, and five of them carry 158 of the 198 files that have exactly one
thing wrong. So the KiCad half is not 209 pieces of work; it is a couple of
dozen, and four of them can run at once. What was missing was never a queue
engine — it was that the list is printed to a terminal and thrown away, while
§0, which is prose, names one item and calls it unclaimed. This section records
the run, the three ways its own output is easy to misread, the rule that says
which items may run together, and what is deliberately **not** built.*

### 171.1 The run

```text
RCAD_STEP_CORPUS=<path> mise exec -- cargo run -q -p rcad-step --release \
  --example blocker_depth_probe
```

on `f34f8321`, 2026-09-20, over 367 files — `nist/` 44, `kicad/` 323. The
example is §56.7's: no assertions, no gate cost. It asks `rcad_step::probe`'s
question — the walk that keeps going past a refused face, bound or edge — and
not the reader's, which stops at the first one.

The pin `KICAD_FILES_THAT_IMPORT` lists **114** of the 323, counted two ways on
the same commit, so 209 KiCad files do not import. The doc comment above that
array still says the equal-pipes lane "moved it from 110 to 112"; it is two
behind, and the count written in prose is the one to distrust.

### 171.2 The pricing table: 22 items, five of which carry four fifths

Files where a reason is the **only** one the probe found, both corpora:

```text
  75  a corner is not on the curved face the file says it is on
  26  both coedges of the edge at <N> run the same way round
  22  crossing too nearly parallel to follow, settling a point onto both
  18  triangulating: the corner at <N> sits on the side leaving corner <N>
  17  a curve this kernel does not hold: B_SPLINE_CURVE_WITH_KNOTS
   6  a surface this kernel does not hold: SURFACE_OF_REVOLUTION
   6  a torus crossing another curved surface is degree up to eight
   4  a loop this reader does not walk: VERTEX_LOOP
   3  a solid with a void in it                                    [fatal]
   3  no half-space square to both surfaces separates the two pieces
   3  a vertex of a face is <N> away from the face's plane
   2  a placement this reader cannot apply: NEXT_ASSEMBLY_USAGE_…  [fatal]
   2  crossing too nearly parallel, bounding how far off a point sits
   2  triangulating: the corner at <N> is where an earlier one is
   2  the file declares two different length units                 [fatal]
   1  each of: HYPERBOLA, SURFACE_CURVE, B_SPLINE_SURFACE_WITH_KNOTS,
      could not settle <N> decisions, an edge walked by <N> coedges,
      a face walking a closed edge against its surface's winding,
      the file has no size                                         [last: fatal]
```

22 reasons; 198 files carry exactly one of them; the top five carry 158.

**This prices a fix from above and is not a forecast.** §45.2's rule, and the
probe's own footnote is sharper about why: the probe asks its own question, so a
file whose only *probe* reason is fixed may still be refused by the reader for
something the probe walked past. Freeing the sole reason is necessary. It is not
shown to be sufficient, and the way to show it is to re-run
`how_often_a_real_file_imports` and watch the pin, never to quote 75 as a yield.

### 171.3 Three ways to misread this output

**The histogram's `0` bucket is not "imports".** `report_depth_histogram` counts
`finding.reasons.len()` and never `fatal`; its doc comment says so. So the eight
files the walk could not get past at all sit in the `0` bucket. The table above
counts them — it takes both `(1 reason, 0 fatal)` and `(0 reasons, 1 fatal)` —
which is the whole of the arithmetic 190 + 8 = 198 that otherwise looks like a
mistake. Read in source on `f34f8321`, not inferred from the totals.

**The co-occurrence ranking is not the pricing ranking.** Deliverable 4 ranks by
how many files hold a reason at all; deliverable 3 by how many hold it alone.
They disagree, and the disagreement is the interesting part:
`B_SPLINE_CURVE_WITH_KNOTS` is in 68 files and alone in 17, while both coedges
running the same way round is in 26 and alone in all 26.

**Co-occurrence is pairwise and is not a union.** It says
`B_SPLINE_CURVE_WITH_KNOTS` appears with `B_SPLINE_SURFACE_WITH_KNOTS` 22 times,
with `SURFACE_OF_REVOLUTION` 18, with `BOUNDED_SURFACE` 10, with `VERTEX_LOOP`
7. Those sets overlap by an unknown amount, so **what the whole B-spline family
is worth cannot be computed from this run.** An earlier draft of this argument
said "68 files, one lane" and that number does not exist. Anyone wanting it must
ask the probe for the union.

### 171.4 Which items may run at once

Two corpus items are concurrent when no file holds both. That is the measurement
the delegating skill's write-set table normally has to guess, for this source at
least — but it is a statement about *files*, not about *source files*. It says
fixing them separately frees files. It does not say the two fixes touch
different modules, and the write-set table still has to be built by hand before
anything is dispatched.

By that rule, on this run:

- **both coedges the same way round** — 26 files, and it never appears beside
  another reason in any file. The cleanest lane available.
- **a corner not on its curved face** — 76 files, sharing a file with another
  reason exactly once.
- **triangulating, a corner on a departing side** — 18 alone.
- **the B-spline family** — one lane and not four, because splitting it leaves
  every file still holding a sibling. Worth an unknown amount; see §171.3.

The 22-file near-parallel class is §0's present unit and already held.

### 171.5 Three kinds of work, and why only one is a queue

The four sources this came from — failing KiCad files, failing NIST files, new
features, and making the nightly find more — are three kinds of thing:

- **A, priced by what it frees.** Corpus reasons, and fuzz finds that still fail
  on `main`. Measured and rankable. Lanes come from here.
- **B, workable but deferred on purpose.** Roadmap 4.3, 4.4, 8, 9.2, 10.1.
  Nothing blocks them and no arithmetic is missing; §13 holds them for want of a
  named consumer, §42.9 refuses healing by name, §104.2 parks the hollow. A B
  item becomes an A item when `docs/consumers.md` names a consumer — that is
  what §13 says, and it is checkable.
- **C, what changes the rate at which A appears.** Test strategy. §13 does not
  govern it, because it is not a part; it is the instrument.

KiCad and NIST are both A. So of four sources there are three kinds, one of
which must be kept out of any dispatch list.

### 171.6 What is not built, and why

**There is no unified queue over all four sources, and there should not be.**
Rendering a §13-deferred feature beside a measured corpus item invites a lane to
take the feature, which is how §13 gets broken by a tool rather than by an
argument. Two weaker reasons were tried first and are recorded because they were
wrong: that such a queue would be large (it would not — every source already has
a natural key, and the probe's cache cost falls on any renderer of it), and that
it would duplicate output (nearer, but it is really that nothing can honestly
interleave "75 files" with "roadmap 4.3 todo", so the merged view is three lists
under one name).

What is worth building instead is small: a claim that outlives a read, and one
renderer for A.

- **A claim.** §159's `claim-section` is the same primitive and exists for the
  same race. Writing this section took 171 while the tree's highest heading was
  168; §169 and §170 landed before the merge, and §172 landed *during* it. Three
  numbers moved under one session in one afternoon, which is the argument.
- **A renderer for A**, cached against the commit, with claimed items hidden.
  Not named `blockers`: the probe's word means a thing that stops a *file*, and
  nothing in this list stops any *work*. That absence of real blocking is
  exactly why the list parallelises, and a name suggesting otherwise would cost
  a re-reading every time.

**§170 is the other half, written the same day.** It generalises the dispatch
into `.claude/workflows/lanes.js`, which takes a base and a list of lanes, with
a report schema, a pinned base sha and a verifier per writing lane. That is how
a lane is *run*; this section is what the list is *cut from*. The seam between
them is the write-set table of §171.7, which neither of them computes.

### 171.7 What is left

- **The write-set table**, before any dispatch. §171.4 gives item independence,
  not file independence, and those are different questions.
- **`KICAD_FILES_THAT_IMPORT` is a merge hazard and should be forbidden to
  lanes.** Every lane that fixes anything wants to append to it, and an array
  whose entries change without changing any string count is invisible to `git
  log -S`. The lane proves its fix; whoever merges regenerates the pin.
- **C has no record.** `docs/plan.md` carried roadmap 11.3, "finding hard
  cases", as `done`, evidenced by five fuzz targets and the nightly apparatus.
  That is true of a thing that was built and cannot hold work that continues.
  Mattias said plainly on 2026-09-20 that it is not done, and this section moves
  it to `partly`. What shape its record takes is not settled here, except that a
  find-rate is confounded by the moving watermark of §90 and too slow to read,
  so whatever is written down should carry its own evidence per line the way
  `docs/plan.md` does rather than ask to be read as a trend.

Write set: this section, and `docs/plan.md`'s state for 11.3.

