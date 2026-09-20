## 179. One file per decision section, and the four things that are rcad's own

`docs/decisions.md` was 40 849 lines and 177 sections, and every session that
finished a decision appended one to the tail. So did every other session. §177
measured what that costs: two files carry every merge conflict this repository
has had, and this was one of them.

**Each section is now its own file under `docs/decisions.d/`, named by the
number it is cited as.** Two sessions writing `179-*.md` and `180-*.md` do not
touch. A session that takes the same number is the one collision left, and
`claim-section` already prevents that (§159) with `check-decisions` behind it.

rscene took this decision first, as its own §39, and the argument for the split
is not re-run here. What follows is only what did not port.

### 179.1 The split lost nothing, and that is the whole correctness claim

`just decisions-all` concatenates the files in number order, and at this
commit its output is **byte-identical** to the `docs/decisions.md` it replaced,
`f02c6b4a`'s copy, diffed and empty. Nothing was read to confirm it and nothing
needed to be: a diff over 40 849 lines is a stronger statement than any reading
of them.

`tooling/decisions-split/split.sh` is committed rather than run once and thrown
away, and that is not tidiness either. Sections land here daily, so a lane that
splits the file at one tip is stale before it merges; whoever merges re-runs the
script against the new tip instead of redoing the split by hand.

### 179.2 Four things did not port from rscene

**Subsections are numbered here.** rscene's `###` headings are free narrative
text, so its `decision` prints whole sections and stops. Every `### N.M` here is
an address — `just decision 173.10` is how one is read — and that had to keep
working. It does, by resolving the number to a file and then running the *same*
awk that read the single file, unchanged, over a file that now holds one section
instead of 177. `decisions-find` is the recipe that could not be ported at all:
rscene counts hits per file with `grep -c`, because a file is the finest unit it
has, and here the subsection is finer than the file. So that one searches
`decisions-all`'s stream, which is what keeps a hit attributed to the `### N.M`
it fell in.

**`claim-section` reads both shapes.** It walks every branch and every worktree
for the highest section number taken. Lane branches cut before today carry the
old single file with its sections in it, for as long as their unit runs. A
recipe reading only the new shape would see §178 as the top while a branch held
§180 and hand §179 out twice; one reading only the old shape would stop counting
the moment this landed. So it reads `## N.` out of `docs/decisions.md` on a ref
*and* the `NNN-` prefixes out of `docs/decisions.d` on a ref, and takes the max
of everything. This is the failure §159 was written about, and it is cheaper to
read two shapes for a month than to pay for it again.

**`check-where-the-work-is` needed the concatenation, not §0's file.** Half of
what it checks is inside §0 — the `reviewed:` date, the 50 non-blank-line cap —
and half is not: its last arm walks every `§N` and `§N.M` the block cites and
asks whether that heading exists anywhere. Pointed at `000-where-the-work-is.md`
alone it would have called every correct citation in §0 dangling, which is
louder than the failure it exists for and wrong every time. It reads
`decisions-all` instead, so the check and its fixtures are untouched. The cap
itself is older than it looks: it came in at `b1ceca2a` on 2026-09-14, not with
§178, which added prose and no code.

**Contiguity is not demanded.** rscene's `check-decisions` asks for sections
running 1..N with no hole, because a section written and never `git add`ed
leaves a hole rather than a conflict. That is a true failure and it is still not
worth the check here: `claim-section` hands a number out under a lock *before*
the section is written, so a claim that is never used leaves a permanent hole by
design. §54 and §65 are two such holes today, neither a fault, and a check that
asked for 0..N would have been red on the tree it was written against. Unique
and self-consistent is what is asked instead.

### 179.3 The preamble is two digits and a section is three

Both wanted to be `000-`: the fifteen lines above `## 0.` come first, and §0 is
a real section whose number is 0. So the preamble is `00-preamble.md` and a
section is `NNN-<slug>.md`. Two digits sorts before three under byte order —
`-` is 0x2D and `0` is 0x30 — and the lookup matches three digits, so the
preamble is invisible to `just decision` rather than special-cased in it.

**Every listing sorts under `LC_ALL=C`, and the first attempt did not.** A plain
`ls` collates under the user's locale, which ignores `-` entirely; `00-preamble`
then compares as `00preamble` against `009-...` as `009...`, and the preamble
lands between §9 and §10. The concatenation came back with its first fifteen
lines 785 lines down, and the byte-identity diff is the only thing that said so.
That is the §173 shape again — a recipe answering a property of the environment
it ran in rather than of the tree — and it is the reason the collation is named
in the justfile rather than left to `ls`.

### 179.4 What the check catches that nothing did before

`check-decisions` was a scan over one file's headings. Over a directory it has
three cases that could not exist before and loses two that no longer can.

New: a filename and a heading that disagree, which is what a section file copied
to start the next one leaves behind; two files claiming one number; and **a
section appended to the pointer file**, which is the habit of four sessions and
the failure the split invites — the text would be live, addressed by nothing,
and colliding again. A preamble that grows a `## ` heading is the same failure
one directory in.

Gone: two `## 1.` in one file is now "a file is one section", and `## 1.` after
`## 2.` cannot be written at all, because the order is the filenames' and the
filenames are sorted. The selftest carries a fixture directory per case, built
from the clean one so that each wreck differs from a passing tree by exactly the
thing it is named for.

### 179.5 What would retire this

A merge conflict in `docs/decisions.d/` that costs more than a minute, which
would mean sessions are editing each other's sections rather than appending
their own — the split is aimed only at the append.
