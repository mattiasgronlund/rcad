## 153. `just recipe` — the justfile had no one-call reader, built 2026-09-19

*§144.6 named the most common two-call shape of the week — grep a file for a
line number, then `Read` it at that offset — and the two files it happens to
most. `docs/decisions.md` has `just decision` and is still grepped; the justfile
had no such reader at all, so there the behaviour was forced rather than merely
chosen. This is the half of §144.6 that was buildable without a measurement.
The other half, why `just decision` loses to `grep`, is not done, and §153.5
says what it needs before anyone guesses at it.*

### 153.1 `just --show` already exists, and prints one line of the argument

The first thing to settle was whether this was building something `just` ships.
It is not, and the reason is precise:

```
$ just --show check-src-selftest
# commit is enough for that, and needs no identity configured.
[private]
check-src-selftest:
    …the body…
```

`--show` prints the attributes, the header and the whole body. What it prints of
the comment block is **one line**, because that is just's own doc-comment rule —
the last comment line above a recipe is its description. In this justfile the
comment block is where the argument lives: 22 lines above `src`, 6 above
`gate_steps`, 17 above `RCAD_STEP_CORPUS`. So `--show` truncates exactly the
part a reader came for, and the line it keeps is a fragment torn out of
mid-sentence — which is the same complaint `list`'s own comment records about
`just --list`.

And it reaches recipes only:

```
$ just --show gate_steps
error: Justfile does not contain recipe `gate_steps`
```

There are twenty-one `:=` assignments in this file, `RCAD_STEP_CORPUS` and
`gate_steps` among them, and no reader could address any of them.

### 153.2 What `just recipe` prints, and the five rules

An address line — `justfile:1453`, which is a clickable address and the one
`decisions-find` deliberately does not give — then the comment block, the
attributes, the header and the body. Assignments too, `export` included.

1. **The extent is exact rather than guessed**, and this is the one place it
   beats `just src`. `src` reads a Rust item's end off rustfmt's layout and has
   a real failure mode when the closing brace is not where rustfmt would put it,
   and its own comment says so. Here indentation *is* just's
   grammar: a body is every following line that is blank or indented, and no
   line inside one can begin at column 0. There is nothing to guess and no way
   to run on to the end of the file.
2. **A name that matches nothing fails**, as `decisions-find` does and for the
   reason that recipe gives: an empty answer reads as "this justfile has no such
   recipe", and a typo can forge that claim.
3. **An inexact name that is contained in several names lists them; contained in
   exactly one, it prints that one.** `src`'s rule, with the wrinkle `src` does
   not need — an exact hit wins outright, because `check-hub` is exact and also
   a substring of `check-hub-selftest` and `check-hub-align-selftest`.
4. **No name at all is the index rather than an error.** It is the only listing
   that shows the `_`-prefixed and `[private]` definitions, and it is where
   §153.3's sweep gets its single call. `just list` hides them on purpose and
   they are therefore the ones nobody can find.
5. **A listed row's one line is the `[doc]` text, or the comment block's first
   line — not its last, which is just's own rule.** `list`'s comment already
   records what the last line gives here: "35 rows of fragments torn out of
   mid-sentence."

Measured on the tree this landed on: 307 404 characters over 105 definitions,
mean answer 2 934 characters — a hundredth of the file, in one call — and the
largest, `check-fuzz-overnight-selftest`, 40 344.

### 153.3 The selftest, and the half of it that is about silence

A fixture justfile in a temp dir, reached through a `recipes_file` variable the
selftest overrides — `decisions_file`'s pattern, because the subject here is a
justfile and a selftest that read the real one would be pinned to whatever
recipes happen to exist today. It holds the comment block, a `[private]`
recipe's attributes, an `export`ed assignment, a blank line kept inside a body
and trailing blanks trimmed, exact-beats-substring, the one-hit-prints rule, the
index, the first-line rule, a missing name and a name that is not a name.

**The sweep at the end is the other half, and it is the half about silence.** A
definition the scanner cannot see does not announce itself: it drops out of the
index and out of every listing, and `just recipe <it>` then says the file has no
such recipe — the empty answer rule 2 exists to refuse. So the scanner's own
index is held against two greps written a different way, and neither side is
allowed to come back short, because two empty lists compare equal and prove
nothing.

Three deliberate breakages, to find out which assertions are load-bearing:

| broken | caught by |
| --- | --- |
| the comment-block rule in `oneline` | the first-line row assertion |
| assignment detection in `defname` | the `export answer` assertion |
| `_`-prefixed names invisible to `defname` | **the sweep, and nothing else** |

The third is the one that matters. The fixture has no `_`-prefixed name, so no
fixture assertion could see it, and the eight `_` recipes in the real file would
have gone missing in exactly the silent way the sweep is for.

### 153.4 `list`'s comment said "the nine selftests" and there are twenty-two

Found by the new recipe in its first real use: `just recipe selftest` lists
twenty-two. The sentence is from the day `[private]` was added. It now names no
number, per §77's rule that a tally goes stale on the next push — and the
replacement says what it used to say and when, because that part cannot go
stale.

### 153.5 What §144.6 still owes, and the one thing to do before guessing

The second half — why `docs/decisions.md` is still the most-grepped file in the
repo when `just decision` and `just decisions-find` both exist — is not done.
Two things stand in the way of anyone doing it well:

**§144.6 names no report, and `spend.py` has none that produces this shape.**
Its six reports are `summary`, `classes`, `ritual`, `sessions`, `carry`,
`ceiling`; none pairs adjacent calls. §144.10 requires a lane to re-run the
report it names, and a report that does not exist cannot be re-run — so the
finding that motivated this section is, today, a number nobody can check. The
first commit of that work is `report_pairs`: adjacent read-only calls where call
B reads the file call A searched.

**There is a concrete hypothesis, and it is a tool problem rather than a wording
one.** A session that has a phrase and wants the text must make two calls by
design: `decisions-find` answers in section numbers, then `decision N` fetches
the text. The tooling is charging exactly the shape §144.6 objects to. The fix
mirrors rule 3 above — **when `decisions-find` matches exactly one section,
print that section** instead of a one-row table — and it is measurable: how many
`decisions-find` calls were followed straight by a `decision`, and how many of
those had a single hit. Only if the pairs report shows sessions reaching for a
bare `grep docs/decisions.md` *without* trying `decisions-find` first is
`CLAUDE.md`'s wording the thing that is wrong.

**Measured in §161, and neither fix is built.** A one-hit find reaches four
pairs in a week. Sessions do grep without trying the find first, but moving
those greps to the find would move the second call, not remove it.

