## 161. Search then read, priced — §153.5 measured, and the one-hit find refused, 2026-09-19

*§153.5 said §144.6's second half owed a report before anyone guessed, and
guessed anyway, carefully: when `decisions-find` matches one section, print it,
and the second call goes away. The report now exists and says that fix reaches
four calls a week. It also says the whole two-call shape on `docs/decisions.md`
costs under 1% of the bill, so the wording half is not worth a `CLAUDE.md`
change either. Nothing is built but the report.*

### 161.1 The report

`just spend <since> <until> pairs`. A pair is two back-to-back responses in one
session, the first searching a file and the second reading the same file. The
search was needed anyway, so only the second response is priced. The matcher
is `targets_of` in `spend.py`, and `check-spend-selftest` holds it to fifteen
cases. Among them: a grep's pattern is not a file even when it looks like one,
`sed`'s script is not a file, and a grep across a directory names no file and so
opens no pair.

### 161.2 What it says

Week of 2026-09-12 to 2026-09-20, $1,271 over 13,762 responses:

- **260 pairs over every file, and their second calls cost $18.63 — 1.5% of the
  bill.** `docs/decisions.md` is 119 of them ($8.44), the justfile 43 ($3.41),
  and no Rust file reaches 15.
- On `docs/decisions.md`, **60 pairs open with a `decisions-find` and 59 with a
  grep.** So the two-call shape is not mainly a habit of skipping the tool. The
  tool makes that shape by design, as §153.5 said.
- Of 95 `decisions-find` calls, 17 matched nothing and 53 were followed straight
  by `just decision`. In those 53, broken down by how many sections the find
  named:

  | the find named | then read one it named | read another |
  |---|---:|---:|
  | nothing | 0 | 4 |
  | 1 section | 4 | 4 |
  | 2–3 sections | 4 | 7 |
  | 4 or more | 12 | 18 |

- 134 greps of the file were not `decisions-find`. In 89, the session had not
  yet tried `decisions-find`; in 45, it had.

The week before, 2026-09-05 to 2026-09-12, reads the same way: 145 pairs on the
file for $9.74, and **no** find with exactly one hit whose section was then read.

### 161.3 Why the one-hit find is not built

It would remove the second call in the "1 section, read one it named" cell:
four pairs in one week and none the week before. Each second call there costs
about seven cents, so the fix saves under a dollar a fortnight. Printing every
section when a find names at most three reaches eight pairs, still under a
dollar. Most follow-up reads come after a find that named four or more
sections. There the session is choosing between them, and no rule about what to
print can make that choice for it.

The cell that is larger than it looks is **"read another"**: 33 of 53. The
session searched, the find named sections, and it read a section the find did
not name. Transcripts show why. The phrase missed, or matched dozens of
sections, and the session read the section it already had in mind. That is a
search that did not help, not a two-call shape. No change to what the find
prints helps a phrase that does not match.

### 161.4 Why the wording is not changed either

In §153.5's terms, the wording is the problem only if sessions grep without
trying `decisions-find` first, and 89 of 134 do. But a grep that became a
`decisions-find` would still be one call of a two-call pair. The 59 pairs that
open with a grep cost $5.11 this week, and moving every one to the recipe would
move that money rather than save it. The only thing a wording change could win
is the answer arriving as a section number rather than a line number. §161.3
shows that number is often not the one the session goes on to read.

A sample of the grep-opened pairs has a second population too: `wc -l`, then
`grep -n '^## 1[12][0-9]\.'`, then a `Read` of the tail. That is a session
finding the end of the file to append a section. `Edit` needs the `Read` first,
so no reader recipe replaces it, and §159's `just claim-section` has already
taken the numbering half of it.

### 161.5 What this closes, and what it does not

§144.6 is closed: its first half by §153's `just recipe`, its second by this
measurement. The shape it named is priced, and it is small. The justfile's 43
pairs predate `just recipe` for most of the window, and the report will show
whether they fall. Re-run `pairs` over a week that starts after 2026-09-19
before quoting a justfile figure.

The matcher counts a pair only when the second response reads **the same file**
the first one searched. A grep that finds a symbol in one file and then reads
another is not counted, so every figure above is a lower bound on that looser
shape, and the looser shape is not this section's claim.

Write set: `tooling/session-spend/spend.py` and its cases, and the `spend`
recipe's doc.

---
