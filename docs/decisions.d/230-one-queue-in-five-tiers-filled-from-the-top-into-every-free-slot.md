## 230. One queue in five tiers, filled from the top into every free slot — 2026-09-25

*Asked by Mattias on 2026-09-25: "How can we optimise the work queue so that: We
make sure that dispatch-lanes can make progress on as many parallel lanes as
possible. We make sure that we always prioritize: 1. Lanes which make us more
effective in finding defects 2. Analyse and resolve new defects 3. Fix failures
in our STEP file corpus 4. Add new functionality" — and then: "Aha I missed the
long execution times, they must be at place 2.5 in the list I gave before." This
section writes down the answer given in the terminal that day, and who builds it.*

  230.1   The order, and what it amends
  230.2   One file, a row per item
  230.3   Every free slot takes the highest ready row that fits
  230.4   A slot is refilled when its lane finishes, not when the batch does
  230.5   A defect is two rows: find the cause, then fix it
  230.6   Write sets that name a function, for the three fuse files
  230.7   Every merge adds the rows it leaves
  230.8   How many slots
  230.9   What is built, and by whom

### 230.1 The order, and what it amends

Five tiers, and the order is strict: a row of a lower tier gets a slot only when
no row above it can use that slot.

| tier | what | where it came from |
|---|---|---|
| 1 | Finding defects better — what the tests and the nights search | §171.5's C; every row of `docs/test-plan.md` |
| 2 | New defects: a wrong answer, a crash, a regression, a wrong answer kept behind `#[ignore]`, a defect a lane reports | §171.5's A, the fuzz half |
| 2.5 | Long execution times: a fuzz timeout, a slow unit, a test or a refusal that takes seconds | new |
| 3 | STEP corpus files that fail | §171.5's A, the corpus half |
| 4 | New functionality | §171.5's B, still under §13 |

*§311 swapped tiers 1 and 2 on 2026-09-27, at Mattias's word: new defects are
tier 1 and finding defects better is tier 2. §307 amends this table again: tier
2.2, for rows that change how the work is done, sits between tiers 2 and 2.5.
§403 added tier 5 on 2026-10-04, below tier 4: every open defect of class C or
D (§352.3) goes there, and tier 1 keeps A, B and the defects with no class.*

This amends two sections. §171.5 kept C out of every dispatch list, because C
frees no file and so loses any ranking by what a lane frees. §226.2 answered that
with one reserved lane. Mattias's order puts C first, above everything, which is
stronger than one reserved slot: when tier 1 has three rows ready that fit, it
takes three slots. So the reserved lane stops being a rule of its own — tier 1
gets it anyway — and §0's line "Beside the unit, always one lane" goes.

§171.5's A splits in two. A fuzz find that still fails on `main` is a new
defect, tier 2; a corpus file that fails to import is tier 3. The unit §0 names —
Mattias's pick of which corpus reason to work on — becomes the order of the rows
inside tier 3.

Long execution times sit between the two, where Mattias put them. A slow input
costs every night that meets it: §227.2 measured the cone-or-torus target's
inputs running about 5 000 times slower once its corpus reached them, and the
night's run count fell 70×. The time a slow row buys back is time the nights
spend finding defects. A wrong answer still ranks above it, because a slow answer
is still an answer.

Tier 4 keeps §13 as it is: a row there is `todo` only when `docs/consumers.md`
names its consumer, and until then it is `parked` on the section that holds it.

### 230.2 One file, a row per item

`docs/queue.md` holds every open item and absorbs `docs/test-plan.md`, whose
rows keep their `T` and `D` numbers. A new row takes the next `Q` number. As in
the test plan, the number is a name, and the order of the lines inside a tier is
the rank.

A row is one line inside the file's fenced blocks: five fields with no spaces in
them, then the description, which runs to the end of the line.

```
<id>  <tier>  <state>  <model>  <write set>  <what, in one line, ending in the §s that argue for it>
```

- **id** — `T<n>`, `D<n>` or `Q<n>`, unique in the file.
- **tier** — `1`, `2`, `2.5`, `3` or `4`, or `2.2` (§307).
- **state** — `todo`; `lane:<branch>` while a lane holds it; `after:<id>[,<id>]`
  while it waits on other rows; `done:<sha>`; `dropped:§N`; or `parked:§N` when
  a section holds it — Mattias parked it, §13 defers it for want of a consumer,
  or it is a limit and not a gap (§16.10). §308 lets an `after:` name a date as
  well as an id, so a row can wait for a calendar date to come rather than for
  another row to close.
- **model** — `opus` or `sonnet`, or `-` on a closed row that never recorded one.
- **write set** — comma-separated. An entry is a path, a directory ending in
  `/`, or `path#item` naming one function in a `.rs` file or one recipe in the
  `justfile`. `-` means the lane writes no tracked file except a new section of
  its own: `claim-section` numbers sections, so two lanes' sections never collide.
  §308 adds `cores:<n>` and `cores:all`, an entry that pins hardware threads
  rather than a file.
- **what** — one line, ending in at least one `§N` or `§N.M`. On a done row the
  last one is the section that records the result.

The file has a heading per tier — `## Tier 1`, `## Tier 2`, `## Tier 2.5`,
`## Tier 3`, `## Tier 4`, in that order — each over one fenced block holding that
tier's open rows, parked ones included. Then `## Closed`, over one block of the
done and dropped rows. §307 adds `## Tier 2.2` between `## Tier 2` and
`## Tier 2.5`, and §307.3 amends "a heading per tier" itself: a tier with no
rows may leave its heading out. The tier field is what a recipe reads; the heading a row
sits under has to agree with it.

A row is **ready** when its state is `todo`, or `after:` with every row it names
`done`.

The dispatcher edits the file — to `lane:` when it dispatches a row, to `done:`
when it merges it. A lane never writes it, so no two lanes conflict over it.

*§352.2 added a tag on 2026-09-29: the first word of a description may be a
class, on a tier 1 row only, or `small`, on a row of any tier.*

### 230.3 Every free slot takes the highest ready row that fits

Two write sets **meet** when they share a path, or one names a directory that
holds the other's path — unless both name an item in that file and the items
differ. `-` meets nothing.

A free slot takes the first ready row of the highest tier whose write set meets
no `lane:` row's and no row already picked in the same fill. That repeats until
no slot is free or no ready row fits. §308 adds a second way a row can fail to
fit: a `cores:` entry whose thread count does not fit beside every other
row's own `cores:` entry, summed against the machine's threads.

That is what makes the order strict without leaving a slot empty. Tier 3 gets a
slot only when every ready row of tiers 1 to 2.5 is either running already or
meets a running lane — typically, when every fuse fix is waiting on a file held
by another fuse fix. The slot goes to whatever can use it, and the higher tiers
lose nothing by it.

*§352 changed the walk on 2026-09-29: tier 1 goes in class order (§352.3), and
ready `small` rows go out as bundles, one lane each (§352.4).*

### 230.4 A slot is refilled when its lane finishes, not when the batch does

The `lanes` workflow starts its lanes together and returns when the last one is
verified. A slot whose lane finished first sits empty until the slowest lane and
the whole close-out are done. Rolling dispatch starts each lane with its own
`Agent` call, `isolation: "worktree"`, and refills the slot from the queue when
the lane's completion notice arrives.

What the workflow does that single calls do not, the dispatcher does by hand. It
writes the base sha into each brief and checks it in the lane's report, because a
lane's worktree branches from the dispatcher's own `HEAD`, not from `origin/main`.
And it dispatches a Sonnet verifier for each writing lane when that lane reports,
rather than for the batch. The workflow stays for lanes that really are one
batch: several rows off one base, all ready at once, with a dispatcher who means
to wait for all of them.

### 230.5 A defect is two rows: find the cause, then fix it

A defect enters tier 2 or 2.5 as a diagnosis row with write set `-`. That meets
nothing, so it can take a slot even while every fuse file is held. It ends by
writing its fix row — same tier, the write set down to the function, a model, and
the section that found the cause — and the fix row waits for a slot like any other.

A defect whose cause is not known yet has no honest write set. Given a guessed
whole file, it holds that file against every other lane for as long as it runs.
Given `-`, it holds nothing until it knows what it needs.

### 230.6 Write sets that name a function, for the three fuse files

`assemble.rs` is 4 380 lines, `arrange.rs` 7 820 and `select.rs` 1 364 (`wc -l`
on 2026-09-25). Most fuse fixes land in one of the three, so at the level of
whole files any two fuse fixes run one after the other. `path#item` lets two
lanes share a file when they change different functions. When their edits land
close together the merge conflicts, and the dispatcher resolves it then — which
costs less than a slot left empty for a lane's whole run.

A lane that finds it must change an item outside its write set says so in its
report, as `out_of_write_set_needs`, and does not make the change. This is not a
split of the files, and none is proposed.

### 230.7 Every merge adds the rows it leaves

When the dispatcher merges a lane, the lane's `remaining` and
`out_of_write_set_needs` become rows, and so does the "what this leaves" of any
section the merge brings — in the same commit that flips the lane's own row to
`done`. §226.2 made that rule for changes to what the tests search. This extends
it to every lane and every section, because a "what is left" that becomes no row
is what §226.1 found stuck for a week.

§0 stops listing what is open. Its "Also open" line and its deferred list become
a pointer to `just queue`, and each item on them becomes a row. Those two are the
longest part of §0, and §0 stood at 50 of its 50 lines when this was written
(`just check-where-the-work-is`). The morning
line, row T4, will add each night's new findings as tier-2 or tier-2.5 rows.

*§352 kept this rule on 2026-09-29 and set §348.7's proposal aside, with one
exception: what a lane's own change leaves stale inside its write set is fixed
there, not filed (§352.5).*

### 230.8 How many slots

Three at a time is the `practices:delegating` rule of thumb. What this machine
measured on 2026-09-25: 24 cores (`nproc`) and 125 GB of memory (`free -g`);
289 GB free on `/` (`df -h`); a built lane tree of 2.6 to 5.4 GB (`du -sh` over
three lanes' `target/`). The dispatcher before the handoff of that evening ran
five lanes at once at a load of about 11, by its own reading, and four read 5.3
at 18:58Z (`uptime`). Disk is not what limits it.

Two things do. Gates that overlap share the 24 cores, and the gate is the one
step every writing lane runs. And every lane costs the dispatcher its brief, its
report, a verifier's report, and a merge with its gate summary: the context
budget hook fired at 201k tokens that evening, after one batch closed and five
lanes were in flight.

So the cap is **six** slots, and a measurement moves it: up while the one-minute
load stays below the core count with two gates overlapping, down when it does not.
Read-only rows count toward it, because they build and run tests too.

### 230.9 What is built, and by whom

This commit writes the section and changes nothing else. Two lanes build the
rest, and their write sets are disjoint:

- **Q1**, Sonnet — `docs/queue.md`, seeded with every open item: the test plan's
  rows, §0's "Also open" and deferred list, and the 2026-09-25 handoff's list,
  each with its tier, write set and section. It deletes `docs/test-plan.md` and
  points `docs/plan.md`'s 11.3 at the queue.
- **Q2**, Opus — `just queue`: the ready rows by tier, then those running, then
  those waiting. `just queue next N`: the fill of §230.3, against the file's
  `lane:` rows. `check-queue` in the gate, holding every row to §230.2's shape —
  row T8, widened to the new file. `just status` printing the ready rows after
  §0. And the `dispatch-lanes` skill taking its lanes from the queue instead of
  from §0's "Next".

When both have landed, §0's "Next", "Beside the unit", "Also open" and deferred
list become one pointer to `just queue`, in the commit that merges them.
