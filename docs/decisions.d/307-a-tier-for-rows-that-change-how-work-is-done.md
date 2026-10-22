## 307. A tier for rows that change how work is done — 2026-09-27

*Queue row Q255. Lane `lane-q255-a-tier-for-tooling`, off `ba1873da`. Amends
§230.1, which had five tiers, and §230.2, which asked for a heading per tier.*

**Tier 2.2 holds the rows that change how the work in this repo is done**, and
it sits directly below tier 2 and above tier 2.5. Mattias agreed to a tier of
its own for these rows on 2026-09-27, below tier 2. Q255's row says why: they
were filed in tier 1, finding defects better, so they outranked every bug fix;
it names Q140, Q151, Q216, Q217, Q234, Q249 and Q252 as filed there, and tier
1's open rows growing from 7 to 25 between 09-25 and 09-27. Those figures are
the row's, not this lane's.

### 307.1 Which rows go in tier 2.2 (amends §230.1)

A row goes in tier 2.2 when what it changes is the way work is done here, and
not the kernel, and not what the tests and the nights search for:

- **the queue and its verbs**: `just queue`, `check-queue`, the merge driver,
  the write guard, the queue page, and the sections that record them;
- **the lanes workflow and its roles**: `.claude/workflows/lanes.js`, and
  `lane.md`, `verifier.md` and `merger.md` under `.claude/agents/`;
- **hooks**, under `.claude/hooks/`;
- **skills**;
- **`CLAUDE.md`**;
- **recipes that are neither tests nor fuzz nights**: `status`, `lanes`,
  `claim-section`, `ff-main`, `prune-worktrees` and their like, with their own
  selftests.

What stays where it was: a recipe that runs a night or reads one
(`fuzz-overnight`, `fuzz-yield`) is what the nights search, so tier 1; a test is
tier 1 or 2 as before; a wrong answer is tier 2.

§230.1's table, its `what` column shortened and its third column left out, with
the new row:

| tier | what |
|---|---|
| 1 | Finding defects better — what the tests and the nights search |
| 2 | New defects: a wrong answer, a crash, a regression, a defect a lane reports |
| 2.2 | How the work is done: the queue, the lanes workflow and its roles, hooks, skills, `CLAUDE.md`, recipes that are neither tests nor fuzz nights |
| 2.5 | Long execution times |
| 3 | STEP corpus files that fail |
| 4 | New functionality |

It ranks below tier 2 because a wrong answer from the kernel costs more than a
slower way of working, and above 2.5 because that is where Mattias put it.

### 307.2 Why the number is 2.2

No tier that exists changes its number. Rows, sections and the queue page cite
tiers 1, 2, 2.5, 3 and 4, and a renumber would break every one of those cites.
So the new tier takes a number strictly between 2 and 2.5, and each place that
compares or orders tiers puts such a number where it has to be:

- **The awk walk in `queue`** goes over `split("1 2 2.2 2.5 3 4", tiers, " ")`
  by position, so the order is the list's own, whatever the numbers are. A row
  joins the walk where `tier[i] == tiers[t]`; both sides look like numbers, so
  awk compares them as numbers, and 2.2 equals no other entry, so a row lands in
  one tier only. A row whose tier is not in the list is now refused (`cannot
  read the tier "5"`); before, it matched no entry and was left out of the fill
  without a word, which is what a list missing 2.2 would have done to every row
  moved there.
- **`_queue-check`** ranks headings by position in the same list with `closed`
  after it, keys `istier` by the tier's text, and reads a heading through
  `key()`, whose pattern now has `2\.2` in it. The out-of-order and not-a-tier
  messages are built from the list, so they cannot fall behind it.
- **The Python side** orders by position too: `tooling/queue/rows.py`'s
  `TIERS_IN_ORDER`, which `heading_key`'s pattern is built from, and which
  `block_for_tier` reads to put a missing heading above the first heading after
  it. `add` and `tier` refuse a tier that is not in it. Nothing under
  `tooling/queue/` sorts tiers as numbers or as text.
- **Were anything to sort them** as numbers, or as text under `LC_ALL=C`, 2.2
  still falls between 2 and 2.5: `2` is a prefix of `2.2`, and in the third
  character `2` comes before `5`.

2.2 rather than 2.1 or 2.25: it is three characters, as 2.5 is, so the column
`add` writes (`{tier:<4}`) and `tier` keeps stay lined up; and it leaves 2.1
above it and 2.3 and 2.4 below it for whatever comes next.

### 307.3 A tier with no rows may leave its heading out (amends §230.2)

§230.2 asked for a heading per tier, and `check-queue` held every one. With
tier 2.2 in that list, the live file failed the gate the moment the tooling
knew the tier, before any row had moved, on this lane's own tree:

```
$ mise exec -- just check-queue
docs/queue.md: heading "## Tier 2.2" is missing
docs/queue.md: 1 problem
```

The write guard refuses adding the heading by hand (§263.7), and `docs/queue.md`
is the dispatcher's and the merger's, not a lane's. So with the rule as it was,
the tier could only land as a lane whose gate was red until a merger moved a row
there in the same commit — and a merger that moved none could not push.

So `check-queue` now asks for `## Closed`, and for each tier heading at most
once and in order, but a tier with no rows may leave its heading out. Nothing
is lost: an open row may only sit under its own tier's heading, so a missing
heading can only mean an empty tier. `add` and `tier` write the heading when a
row first goes to that tier, above the first heading that comes after it.

**Corrected 2026-10-22 (Q2611, from Q286):** this used to say `pick` still
refused when the side it keeps belongs to a tier with no block, needing one
side of a merge to have lost a heading, which no verb did. `pick` now writes a
missing tier heading itself, through `put_in_its_block`, the same way `add`
and `tier` do; it refuses only when the row belongs under `## Closed` and that
heading itself is missing, which a well-formed file never leaves out.

### 307.4 The queue page keeps five tiers for now

`rows.py`'s `TIERS` has one reader, `tooling/queue-page/render.py`, and that
file is not this row's. It has a label for five tiers (`TIER_LABELS`) and six
colours, the five and the parked band, and `series_color_css` counts six. Run
in memory with `TIERS` set to all six tiers (a probe in the scratchpad that
writes nothing), the page fails:

```
render.TIERS: ['1', '2', '2.2', '2.5', '3', '4']
tally: {'by_tier': {'1': 29, '2': 73, '2.2': 0, '2.5': 7, '3': 2, '4': 8}, 'parked': 14, 'closed': 145}
render_page raised KeyError: '2.2'
```

So `TIERS` would break every render — the one `queue` runs after each verb that
writes, and the one `check-queue-api-selftest` checks for — and the `by_tier`
it tallies gains a `"2.2"` key that `check-queue-page-selftest`'s three
expected tallies do not have. `rows.py` therefore keeps `TIERS` at five,
derived from `TIERS_IN_ORDER` with 2.2 left out, and says why beside it.

That is a wrong page until the page is changed, and it is wrong in two named
ways. Run the same way with `TIERS` as committed and Q249 moved to tier 2.2 in a
copy of the text, the page renders, the tally moves Q249 from tier 1's band to
the parked band (tier 1 29 → 28, parked 14 → 15), and Q249 is missing from the
page's ready list. That is `tally()`'s fallback for a tier it does not know.
Nothing the queue does reads the page, so the fill, the verbs and the gate are
right; the page is what a person looks at.

### 307.5 `just queue tier <id> <tier>`

```
just queue tier <id> <tier>     move an open row to the foot of another tier, writing that tier's heading if missing
```

It changes the row's tier field and moves the row to the foot of the new tier,
as `add` places a new one; `rank` then places it within the tier. The row keeps
its state, so a `lane:` row keeps running and keeps its hold, and no claims
event is written. It refuses a closed row, a row already in that tier, and a
tier not in the list. Like every verb it takes the claims lock, refuses a file
holding a conflict, runs `check-queue` on the file as it would be and writes
only if that passes, and `queue` runs `queue-page` after it. The write guard's
refusal names it beside the others.

### 307.6 The tests

- **`check-queue-selftest`**: the good file has a `## Tier 2.2` block with a
  row and its pass line counts it (8 rows, 3 ready); a copy with the empty
  `## Tier 4` heading and block deleted passes; a tier 2 row under `## Tier 2.2`
  is refused, and so are `## Tier 2` and `## Tier 2.2` swapped (`"## Tier 2" is
  out of order`). The fill's order fixture has a tier 2.2 row written above tier
  2: four slots take T11 T12 Q20 Q22 and five add Q25. A file holding a row of
  tier 5 is refused by `queue next`.
- **`check-queue-api-selftest`**: `tier Q4 2.2` in a tree whose file has no
  `## Tier 2.2` writes the heading between `## Tier 2` and `## Tier 2.5`, puts
  Q4 under it with its state kept, and passes `check-queue`; `tier` refuses Q4
  to its own tier, Q2 to tier 5, and the closed T1. `add 2.2` in another tree
  whose file has no such heading writes it over the new row.

What fails without the change, by what each arm needs: without the list in
`queue`'s awk, Q22 is never walked and four slots take T11 T12 Q20 Q25; without
`_queue-check`'s, the good file is refused (`tier "2.2" is not`, and a fenced
block under `"## Tier 2.2"`); without `queue.py`'s verb, `queue tier` is not a
verb. Both selftests pass on the lane's tip. Running the API selftest with
`tooling/queue/`'s Python put back to its base was refused by the permission
system, so that it fails there is read, not run.

### 307.7 What this leaves

- **The queue page learns tier 2.2**: `rows.py`'s `TIERS` becomes the six (or
  the page imports `TIERS_IN_ORDER`), `render.py` gains a label and a seventh
  colour checked with the `dataviz` skill's palette validator, and
  `check-queue-page-selftest`'s three expected tallies gain `"2.2":0`. Write
  set `tooling/queue/rows.py`, `tooling/queue-page/`,
  `justfile#check-queue-page-selftest`, in one lane. *Done by Q982 on
  2026-10-04, with tier 5 too; §403.8 says how.*
- **`docs/queue.md`'s own text** still says "in five tiers" and "`1`, `2`,
  `2.5`, `3` or `4` (§230.1)". No verb edits the text outside the rows, and the
  guard refuses the hand edit.
- **The rows to move**: the lane's report lists each open row it judges belongs
  in tier 2.2, for the merger to move with `just queue tier`.
