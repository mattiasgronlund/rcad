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
