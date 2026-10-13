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
