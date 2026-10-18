## 455. The queue page lists every open row by tier, ready or blocked, and what each row led to — 2026-10-18

*Mattias, 2026-10-18, on the page `just queue-page` writes:*

> Could we update the status-page and replace Ready with Queue.
> And for each Tier headin we should add a table:
> - Ready / blocked
> - Model
> - Count
>
> instead of the comma separated list of Ready rows we could have have them grouped consequtive rows into groups:
> - Ready /blocked
> - Model
> - Comman separated list of Qnnn
>
> Then for each subpage for a specific row, we should add a resulted in table, just as the Came from table.

*Built by session `status-page-queue-tables` in `tooling/queue-page/render.py`,
under `CLAUDE.md`'s "Decide it yourself". Amends §444.1.*

**In short.** The front page's Ready list is now a Queue section. Under each
tier it puts two tables: how many rows are ready and how many are blocked under
each model, and then the rows themselves, one line per run of neighbouring rows
that share a status and a model. Each row's page gains **Resulted in**: the tree
of rows that came from it, read the same way as Came from.

### 455.1 What counts as blocked

A row is **blocked** when the table of rows the fill will not take files it
under any reason: it waits on a row not yet done, it is parked, or it meets a
running lane's write set. These are the reasons the Pinned table's "Blocked by"
column already gives, so the two cannot disagree. Every other row is **ready**.

Two consequences, each a change from the Ready list:

- **Parked rows show, as blocked.** The Ready list left them out. A section
  called Queue that dropped them would undercount each tier.
- **A row that runs does not show**, whether the claims file holds it or main's
  copy says `lane:`, as `just queue` counts both. The Ready list read main's
  copy alone, so a row the claims file held while main's copy still said
  `todo` showed as ready beside the Running list that also named it.

`just queue blocked` runs only in the tree holding main, and only on main's own
copy. When it cannot run, the rows it would have held back show as ready, and
a note above the tiers says that this only means they wait on nothing and are
not parked. This is the same thing the Pinned table already says in that case.

The rows of a tier come in the order main's copy lists them. The count table
lists ready before blocked, and each model by name.

### 455.2 Resulted in

§444.1 left a row's children off its page because the request then was only
where a row came from. This request asks for what it led to.

**Resulted in** is the tree below the row: every row whose parent it is, and
then their children, as nested lists. Each entry gives the same line the
child's own Came from gives it. It reads the same parents as Came from,
including the sources §433.6 reads from a row's text. So a row appears in X's
Resulted in exactly when X is on that row's Came from. A branch that comes back
to a row above it stops there and says so, as Came from does.

**A re-file only shows under the row it re-files when its word says so.** Take
Q2282, filed on 2026-10-18 to re-file Q2260. Its word is `from/Q2205`, which it
copied from Q2260, and not `was/Q2260`. So lineage hangs Q2282 beside Q2260
under Q2205, and Q2260's page says nothing came from it. `just lineage main
Q2282` gives the same parent. The page shows what lineage reads, and does not
second-guess it.

On main at 714b147e the largest tree is below Q28: 172 rows, 17 levels deep,
on a 68 KB page. A render with every cache full took 5.4 s of CPU, both before
and after this change.

### 455.3 How it is checked

`check-queue-page-selftest` checks, on its fourth commit, each tier's two
tables against counts done by hand. The fixture covers:

- rows blocked for each of the three reasons;
- rows running by the claims file and by main's copy, which must not show;
- two blocked rows with running rows between them, which share a line;
- a blocked sonnet row beside a blocked opus row, which starts its own line;
- two blocked rows with a ready row between them, which get a line each.

It then checks tier 3 again with the third reason not worked out.

For Resulted in it checks four trees. Q3's whole tree must leave out Q30,
which hangs on Q5 only when its text is not read. Q28 and Q29 name each other,
so Q28's tree must stop when it comes round. Q2's tree must grow one level when
a later commit files a row from Q21. Q4 must gain Q21 when a word is planted in
the lineage cache. The check that an unchanged page keeps its inode moved from
Q1 to Q6: the planted word moves Q21 into Q1's tree, so Q1's page now changes.

The old renderer failed the new selftest at the section order. So did each of
twelve mutations of the new one, each at the check aimed at it:

- running rows shown;
- each of the three reasons read as ready;
- runs grouped without regard to order;
- the model dropped from the runs, or from the counts;
- the note removed;
- the tree read without the text sources, cut to one level, or left without
  its stop.
