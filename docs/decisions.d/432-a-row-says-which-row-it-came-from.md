## 432. A row says which row it came from — 2026-10-11

*Mattias, 2026-10-11, after asking whether we can track which queue rows are
added because another row was run, and how far each first row grows before all
the rows it led to are closed: "We must record it, as it would allow us to
better understand what is happening here."*

### 432.1 What git can tell, and where it stops

§348.2 measured the same growth on 09-28 by reading it back out of git, and
recorded nothing. It can still be read back that way. Walk `git log
--full-history main -- docs/queue.md`: each row is added in the one commit
whose parents all lack it, and its parent is the row that commit closed. That
is the lane's row on a lane merge, else the row closed `done`, else the row
dropped. A row is filed again when at least half its words match a row dropped
in the same commit. On 2026-10-11 that gave:

- 1622 rows, each added exactly once. 1401 of them get a parent, 211 of those
  as a row filed again, and 221 get none.
- 96 of the 221 name an earlier row in their own text, and about half of those,
  sampled by hand, are real children filed in a later commit than the one that
  closed their parent. Git cannot see those, so the tree it gives is a lower
  bound.
- Since 10-03, each row closed `done` filed 0.95 new rows (596 for 625). Before
  that it was about 1.5. The four rows filed on 09-25, Q1, Q28, Q5 and Q4, had
  led to 713 rows between them, 145 still open.

The gap in the second bullet is why the row must say it, at the moment it is
filed, by whoever knows.

### 432.2 One word

The description carries one word saying where the row came from:

- `from/<id>`: running `<id>` is why this row exists.
- `was/<id>`: this row files `<id>` again. `<id>` may be done as well as
  dropped: Q1049 filed Q1031 again after a bundle merge had closed Q1031 though
  nothing had been done on it.
- `from/-`: no row led to it. Mattias asked for it, a fuzz night found it, or a
  session found it by looking.

`was/-` is refused: a row cannot be filed again from nothing.

### 432.3 Where it goes: after the pin and the tag

As in `pinned/2 wrong/A from/T9 a defect the first row found §230`. This amends
§230.2's grammar for **what**, as §352.2 and §430.2 did before it.

The pin and the tag stay where every reader already looks for them, the first
words. `Row.with_tag` in `tooling/queue/rows.py` and the awk that reads the tag
in the `queue` and `_queue-check` recipes need no change: `tag` puts a class in
front of the first word,
so a class added later lands in front of `from/` by itself, and `tag -` takes
it off and leaves `from/` first again. The fill never reads the word.

### 432.4 Who writes it

**`just queue add` refuses a row without the word.** It takes the word first or
right after the tag, and stores it after the tag either way, so that `just
land` can put it in front of a lane's row without reading its tag. It checks
that a named row exists, in this copy or anywhere `after:` looks
(`exists_elsewhere`). It runs the refusals of a class with punctuation after it
(Q1505) and of a pin (§430.4) on the words behind it, so the word cannot be used
to slip either past.

**No default.** A batch that closes one row can add a row that has nothing to
do with it. 200110dd closed Q1659 and filed Q1679 again as Q1683 in one commit.
A default taken from the batch would have marked Q1683 `from/Q1659`, and it is
`was/Q1679`.

**`just land` writes it for a lane** (`said_where_from` in
`tooling/land/land.py`): `from/<the lane's first row>` in front of every `add`
step whose first two words name no source. A lane's rows come from the row it
ran, so a lane need not know about the word. A bundle's lane names the row
itself for an item that is not its first, and so does a lane that files a row
again. `merger.md`, `lane.md`, `verifier.md` and the dispatch-lanes skill say
so for the merger, the lane, its verifier and a dispatcher filing a returned
lane's rows.

### 432.5 What `check-queue` holds

The word's shape, after the pin and the tag; that a row does not come from
itself; and that a `T` or `D` id is a row in the file, since no new row takes
those letters. A `Q` id this copy cannot see passes, for the reason §263.5
gives for `after:`: the other dispatcher's new row may not be merged here yet.
A tag written after the word is refused, because the fill reads the tag only in
front of it. Rows filed before this section carry no word, so a row without one
passes.

### 432.6 Why not in the claims log

The claims log in the git common directory already gets an event for every
`add`. It is not in git, so a fresh clone loses it, and it names the worktree
that ran `add`, which for a lane's rows is a `wf_…` merger tree and not the
row. The queue file is in git and travels with every copy.

### 432.7 What this leaves

- **The header of `docs/queue.md`** still describes **what** without the word.
  The write guard refuses every hand edit to that file, its prose included: an
  `Edit` adding the bullet was refused on 2026-10-11. Open row Q983 is the verb
  that would let the header change; the bullet waits for it.
- **A recipe that prints the growth**: for each first row, how many rows it led
  to and how many are open, reading the word on rows filed since this section
  and §432.1's commit inference on older ones. Filed as a row with `from/-`.
