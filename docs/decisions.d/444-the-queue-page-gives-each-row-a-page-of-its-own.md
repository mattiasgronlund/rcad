## 444. The queue page gives each row a page of its own, with where it came from — 2026-10-13

*Mattias, 2026-10-13, on the page `just queue-page` writes to
`~/rcad-queue-page`:*

> I would like some improvements in the status web-page.
> 1. Move the definition for each row into it's own subpage.
> 2. Replace all Qnnnn references with real clickable references into the respective subpage.
> 3. Rework "Rows the fill will not take now", to instead of mapping row to reason, they should be mapped reason to list of rows (Qnnnn references).
> 4. On each rows subpage, describe outside between the Qnnnn heading and the raw row text:
> 4.1 Which tier the row belongs in.
> 4.2 Its categorisation (A/B/C/D).
> 4.3 Full provinance for the issue.
> 4.4 What it expects to change, e.g. crates/rcad-step/src/write.rs.

*All four are built, in `tooling/queue-page/render.py` and
`tooling/lineage/lineage.py`. This amends the page of §230 and §263. It
changes how §432's lineage reads the history, and not what it finds.*

### 444.1 What each row's page says

Every row of main's copy gets `rows/<id>.html` beside `index.html`. Between
the id and the row's own text it gives:

- **Tier**: the row's tier field.
- **Class**: the letter and what it means, then the whole tag and what kind
  of fault it is, both cut from §352.3. A row tagged `small` says so instead,
  and a row with no tag says "none".
- **Came from**: the row, the row it came from, and so on up to a row nothing
  led to, a fuzz target or a STEP file. This is `just lineage main <id>`'s
  reading (§432, §433), with the sources §433.6 reads from a row's text. Each
  step names the commit that added the row, its day and subject, and how
  lineage found the parent: by the row's own word, from the commit, from the
  commit's message, or from the text. Where the text changed the answer, the
  step also says what it would be without the text. A chain that comes back to
  a row it already passed stops there and says so.
- **Expects to change**: the write set, one file per line, or "no tracked
  file" for `-`.

A row's children are not shown. The ask was where a row came from, not what it
led to.

The front page now names a row by its id. The ready and lately-closed lists
give the id alone, and the running list the id with its branch and tree. All
three used to carry each row's text, which is on the row's page now.

### 444.2 Which ids are links

An id that sits where the queue's grammar puts one (a running row, an
`after:` row waited on, a holder in a `blocked` line) links whatever its
letter. In free text, the row's description or a commit subject, only `Q`
ids link. A `T` or `D` id in prose is as often something else: the queue says
"§264's arm D2", and D2 is also a row, so linking it would send the reader to
the wrong place. An id main's copy does not hold is printed plain, since it has
no page. On the real page at c8a9eda5 that is one id across all 1828 files,
Q248, which Q249's text says "was never committed".

### 444.3 The table of rows the fill will not take, turned round

The table is now Why | Rows. Within each kind (waits on a row not yet done,
parked, meets a running lane's write set):

- a reason's rows come in the order a fill walks them;
- the reasons come in the order of the first row each holds back, so the
  reason in front of the row a fill would reach first leads;
- the kind's heading counts rows, each once however many reasons name it. A
  row waiting on two unfinished rows sits under both reasons.

A "meets" reason leaves out the row's own path, so two rows that meet the same
hold through different paths of their own share one reason. A "shared" reason
keeps the shared path, since that path is the reason. The pinned table's
"Blocked by" is the same reasons, joined with "; ", so the two tables cannot
disagree.

### 444.4 The page had been crashing on §424.4's skip line

`just queue blocked` prints a second form of line since §424.4:

```text
skip Q1821 (tier 1): its crates/rcad-topo/src/fuse/arrange.rs is shared, and Q1892 (held by …) and Q1815 (held elsewhere by …) already hold it
```

The old `render.py` asserted that every line was the "meets" form. So every
render failed while such a line was printed, and `~/rcad-queue-page` kept its
last good copy, written at 09:00Z on 10-13. The page reads both
forms now, and still asserts on a third.

### 444.5 Lineage keeps a summary of each commit, and the page keeps them on disk

**Why a cache at all.** The old `just lineage main Q1868 Q1046 Q1108 Q18` took
6 min 28 s on main. `docs/queue.md` is 1.7 MB. On main at 3ca67d02, 2523
commits touch it, and those with their parents hold 2296 distinct copies.
Parsing one copy with `rows.Queue` takes about 0.09 s, so the parsing alone is
over 200 s. A walk that long cannot run on every queue verb, and the page
renders after every one.

**What is kept.** `lineage.summarize` reads one commit's copy and its parents'
copies and keeps only what the rules read: each row the commit added, with
its text, and each row it closed, with done or dropped and its text. `Graph`
takes a dict of these by commit sha and fills in the commits it lacks. The
page keeps that dict in `<out>/.lineage-cache.json`, so a render parses only
the commits that are new since the last one. `just lineage` passes an empty
dict and works as before.

Even cold, the walk is cheaper than it was: each distinct copy is parsed once,
not once per commit holding it. The old and new full `just lineage main`
reports on main's real history were identical, all 751 lines.

**Why keying by commit sha is exact.** A summary is a function of the commit's
own copy, its parents' copies and the parsing rules. A sha fixes the first
two for good. The third is guarded by `SUMMARY_SHAPE` in `lineage.py`: a
cache written under another shape is dropped whole and rebuilt.

**That guard is bumped by hand.** The summary reads through
`tooling/queue/rows.py`: which lines are rows, a row's id and text, and which
states close it. A note above `Row` there says so. Keying the cache on a hash
of `lineage.py` and `rows.py` instead would never miss a change, but rows.py
changed 8 times in the month to 10-13. Each change would then cost the next
queue verb a cold walk of about four minutes, since `just queue` runs the page
in the foreground. A missed bump costs a wrong "Came from" line on a status
page until the shape is bumped or the cache file is deleted. It never touches
the queue itself.

### 444.6 What a render costs

The real page at c8a9eda5, rendered into a scratch directory:

| Caches | Wall time | Peak memory |
|---|---|---|
| All empty | 235 s | 190 MB |
| All full | 3.4 s | 66 MB |

Of the warm 3.4 s, `just queue blocked` took 1.26 s, the fuzz section 1.24 s,
lineage 0.64 s (0.29 s of it listing every fuzz input and STEP file the
history held, for §433.6), and the 1827
row pages 0.26 s. Under a load average of 22–37 the same warm render took
14.3 s, every part about four times slower. The two biggest parts were there
before this section.

A row page is written only when its content changed. A queue verb changes a
row or two. Writing all 1827 every time took 0.30 s of a 4.0 s render, and
reading each back to compare takes 0.08 s.

### 444.7 How it is checked

`check-queue-page-selftest` builds a scratch history and checks:

- that the ids link;
- the Why | Rows table and its order;
- the shared-file skip line;
- every field of a row page;
- each way lineage can find a parent: by word, by a lane's merge, from the
  commit, from a crash file the text names, and from a STEP file;
- a word that names no row, and a chain that goes round;
- the lineage cache: a kept commit is served (a summary edited by hand
  shows on the page), a new commit is worked out, and a cache under another
  shape is dropped.

Eleven mutations of `render.py` each made it fail, and the unmutated copy
passed. It does not cover lineage's message and unknown labels, or the
`small` class. `check-lineage-selftest` covers those two labels, and on the
page each of the three is one dictionary entry or one branch.
`check-lineage-selftest` failed when `summarize` dropped the closed rows.
