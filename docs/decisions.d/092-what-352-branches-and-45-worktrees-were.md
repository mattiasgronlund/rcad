## 92. What 352 branches and 45 worktrees were actually holding, recorded 2026-09-13

*An audit of everything uncommitted or unmerged in this repo, run because the
question "is there anything outstanding" had no cheap answer. The answer is
**nothing** — §92.7, which overturns the one find §92.2 thought it had. §92.1 is
why the obvious commands give the wrong one, and §92.7 is that same lesson
catching the audit that wrote it.*

### 92.1 `git branch --no-merged` answers a different question than the one asked

On 2026-09-13 the repo had **352 local branches and 45 worktrees**, and 14 of
those branches were unmerged. Thirteen of the fourteen held nothing that was not
already in `main`.

This is §50.2's finding, which said it about two branches and predicted nothing
about the rate: *"An unmerged branch is not evidence of unlanded work; it is
evidence of an unrecorded merge, and the two look identical from `git branch
--no-merged`."* At 13 of 14 it is not an occasional accident, it is what the
command means here. A lane that is reworked before landing — renamed commits,
squashed, re-derived on a newer base — lands its *content* and never its
*commits*, and `merge-base --is-ancestor` only ever sees the commits.

**So judge a branch by its items, not by its ancestry.** The cheap version: take
the names the commit adds (`git show --format= <sha> | grep '^+.*fn '`) and ask
whether `main` has them. That separated the fourteen in one pass where reading
diffs would not have — `git diff main..<branch>` on these branches reports
thousands of changed files, because the branch is old and the diff is mostly
`main`'s own later work flowing backwards.

`git cherry` is the sharper tool and still not sufficient: it matches patch-ids,
so it catches a verbatim cherry-pick (it cleared `m5-predicates-scaling` and two
of three commits on a mesh branch) and misses every rework. One branch had its
doc-fix commit marked as landed and the two commits *underneath* it marked as
not, which is impossible as stated and exactly what a rework looks like.

### 92.2 The one real find, and the shape that distinguished it

**Wrong, and §92.7 says why.** `main` already had this row under other names.
Everything below about the `#[cfg(test)]` marker holds; what does not follow is
the conclusion that `main` lacked the code.

`arms-spike` at `b6de278` adds `where_a_plane_cuts_a_cone_in_a_parabola` and
`follow_the_parabola` to `crates/rcad-geom/src/walk.rs`. Neither name exists
anywhere in `main`'s `src/`, both sit **above** that file's `#[cfg(test)]`, and
both are called from the live dispatch. It is titled `spike:` like a dozen
throwaway probes and is the only one of them carrying production code.

**The test that separated it from the probes: where the added functions sit
relative to `#[cfg(test)]`.** Two other `spike:` branches add a dozen-odd
functions each with names that read like real geometry —
`the_two_flat_surfaces_a_coaxial_cone_and_pipe_cross_in`,
`building_the_cross_product_in_f` — and every one of them is below the marker, in
a probe module. Commit titles, branch names and function names all failed to
tell these apart; the marker did not.

### 92.3 A worktree that reads "clean" can be the only copy of something

`prune-worktrees` keeps a tree that is locked, dirty, holds `main`, is detached,
or is unmerged (§76). The detached arm is **unconditional** — kept even when the
tree's commit is provably an ancestor of `main` and it therefore holds nothing
unique. Lanes here mostly end detached, which is why 45 trees accumulated: 20 of
them were clean, unlocked, detached, and at commits already in `main`, and the
recipe will never touch one of those.

That is a defensible default and it is not the dangerous part. **The dangerous
part is that `git status --porcelain` does not see gitignored files, and
`fuzz/artifacts/` is gitignored.** Two trees in that "pure clutter" set held the
only copy of live fuzz evidence. `rcad-worktrees/fuzz-overnight` held
`slow-unit-5a19686d…`, in no corpus, on no branch, no second copy anywhere on
disk; it passed every test a sweep applies and came one command from being
reclaimed. §91 is the same story for `rcad-nightly-fuzz` and cost three days.

So a sweep needs a check no ancestry test provides: **for every gitignored
artifact in a tree about to go, refuse unless `main` already carries those exact
bytes.** Bytes, not filenames — a corpus file is named after its own content
hash, so a name match is nearly a content match and "nearly" is how the wrong
file gets kept while the right one goes.

**Key it on the bytes and nothing else.** The first version of this check looked
the artifact up by *filename* under `fuzz/corpus/` and compared bytes only once
it found a hit. That is broken, and it was broken against the very next seed to
land: `crash-cba396f8…` went into the corpus as
`crash-a-thin-pipe-on-a-fat-ones-skin-cba396f8…`, same blob
`3a1ced3a04e4ee856ae41cb47332e41ae7ad47af`, because the raw hash is unreadable
and naming a seed is what anyone would do. So the key was the one thing expected
to change, and a committed seed would have been reported as an orphan for ever.
`git hash-object` against the blob ids in `main`'s corpus tree is the key that
holds: it asks "does `main` have these bytes", which is the actual question, and
it cannot be confused by either a rename or a collision of names.

**Prove it refusing, not just accepting** — a guard that only ever says yes is
§77.6's shape, and this one is checked in four directions: it accepts a seed
under its own name, accepts one under a new name, refuses a file wearing a
committed seed's name over different bytes, and refuses bytes in no corpus.

**"In no corpus on `main`" is not a proxy for "worthless". It marks the window in
which a file is most dangerous to lose.** A crash seed is deliberately absent
from the corpus for exactly as long as the bug is live, because a crashing input
there kills every later night at corpus load. So the sweep refuses hardest
precisely where the loss would be worst, which is the property to keep rather
than a coincidence to note.

That window has an end, and it is the fix commit — §93 closed this one about four
hours after §92 was written. **So an orphan artifact never needs a human to
adjudicate it and the answer is never "delete it now": refuse, and re-check
later.** Either it has become a committed seed by then, or the bug is still live
and the file is still the only copy.

### 92.4 "Copied out" and "in the object store" are different claims

The session that landed `34ba0ad` first reported the seed copied out and its
worktree free to reclaim. It was not: the file was `??` in that worktree, under
`fuzz/corpus/` — **tracked territory, not a gitignored path, so `git clean -fd`
takes it and nothing reads as unusual first.** Untracked under a tracked
directory is a worse hiding place than gitignored, because the gitignored case at
least announces itself to `status --ignored`.

Three checks settle it and none of them is a peer's word: the commit is an
ancestor of `main`, `git cat-file -e main:<path>` answers for each file, and the
blob is `cmp`-identical to the original on disk. Run those before reclaiming a
tree, including when the session that owns the file says it is safe — that report
was made in good faith and was wrong about the one thing that mattered.

### 92.5 What the audit actually removed

334 branches, every one verified an ancestor of `main` first. The rest —
12 valueless branches, 38 worktrees, four untracked probe tests, and three
debug-only uncommitted edits (an `eprintln!("TRACE …")` block, a
`MEASURING_SLACK` loosened from 16 to 32 against the doc comment directly above
it, and a 141-line draft of a §41 that `main` already records) — needs
`git branch -D` and `git worktree remove`, which Claude Code's auto permission
mode refuses. That half is scripted and unrun; it is not evidence of anything
except a permission boundary. **It ran the same day — §92.6.**

Three branches are deliberately kept: `arms-spike` (§92.2), `ci-kache-bootstrap`
(PR #1, which `.github/workflows/ci.yml` already records as superseded in its own
comment), and the live session's own.

### 92.6 The sweep ran, and the remote half nobody had counted

Mattias ran it the same day. Two passes, because the first left one tree behind:
**49 worktrees to 10, 23 local branches to 8**, `just check-hub` green after
each. The tree the first pass missed was the one the *previous* session had been
working in, still locked to a session that no longer existed — so the second
pass needed a `git worktree unlock` before the lock guard would let it through.

The script needed one fix to survive a second pass at all. Its revert step named
worktrees by path under `set -e`, so the first `git -C` at a tree the first pass
had already removed killed the whole run at step 2, before it reached the trees
it still had work to do in. Reverts are now guarded on the tree existing. **A
sweep script is run more than once by construction** — anything it protects this
time it will be asked about again next time — so "runs correctly against a
half-swept repo" is one of the things it has to do, not a nicety.

**`origin` had 101 branches fully merged into `main`, and no one had looked.**
§92.1 counted 352 *local* branches and the sweep deleted 334 of them; the remote
was never in the question. `git for-each-ref --merged origin/main` answers it in
one call. They were the same population — the `worktree-agent-a…` lanes and the
`arms*`, `door*` and `m5-*` series — and every one an ancestor of `origin/main`,
so deleting the refs dropped no commit. `origin` went from 108 refs to three:
`main`, `arms-spike`, and `pin-the-marketplace` behind its open PR.

That is the general shape and it is worth stating on its own: **an audit scoped
to "this repo" means the local refs unless it says otherwise, and the remote is
a second set of the same refs that no local command mentions.** The 334 local
deletes left the clone's own listing clean and left a fresh clone pulling 108
branches, 101 of them noise.

### 92.7 The one real find was already on `main`, under other names

§92.2 is wrong, and it is wrong in exactly the way §92.1 predicts.

It named `arms-spike` the one unmerged branch carrying production code absent
from `main`, on the strength of two tests: neither
`where_a_plane_cuts_a_cone_in_a_parabola` nor `follow_the_parabola` exists
anywhere in `main`'s `src/`, and both sit above `walk.rs`'s `#[cfg(test)]`
wired into the live dispatch. Both statements are still true. The conclusion
does not follow.

`main` has the parabola row, and has had it for some time. `walk.rs` carries a
`struct Parabola { vertex, across, bend, size }` with the spike's four fields;
`the_parabola_a_plane_along_a_ruling_cuts` builds it with the spike's own
formulas — vertex `C/K` along `and` from the foot, bend `−and/K`; `walk`
dispatches `WhatARunLiesOn::Parabola` into `walk_an_endless_bend`; and
`the_seven_shapes_a_plane_cuts_a_cone_in.rs` tests it, including the
translation-invariance the spike's doc comment called its whole open question.
`main` also does the hyperbola branch through the same `AnEndlessBend` trait,
which the spike never reached, and publishes a roundoff bound the spike had
none of.

**`main` names the spike and says why it went the other way.** The doc comment
on how far an endless bend is walked rejects *"the 20-minute spike's
`√(size/‖bend‖)`"* by name: `size` is `how_big_this_is`, which reads the
*coordinates* of the vertex and the apex, so the same parabola written a
thousand units further out gets a longer stretch and different points — §19,
where a shape sits is not allowed to change how it is cut. The test
`moving_a_cone_and_its_plane_together_moves_the_walked_parabola_and_nothing_else`
is the demonstration. So this was not a rework that happened to lose the branch;
somebody read the spike, took its derivation, and replaced the one number it had
guessed at.

**The `#[cfg(test)]` marker test answers "is this production code", and §92.2
read it as answering "does `main` have this".** Those come apart precisely on a
rework. The marker cleared the spike of being a probe, correctly — and then the
*name* test was left carrying the whole claim about `main`, and a name test is
the one §92.1 had already shown fails here, because a rework lands its content
under whatever names the rewrite chose. Thirteen of fourteen branches went that
way; the fourteenth did too, and the marker made it look like the exception.

**So: to ask whether `main` has something, look in `main`.** Both tests §92.2
used are properties of the branch. Neither one reads `main` for the behaviour,
only for the string, and the behaviour is what the question is about. The cheap
version of the right test is the one that settled this in four commands —
`just src Parabola`, then grep `walk.rs` for the word.

`arms-spike` holds nothing that is not on `main`. The rework §92.2 implies would
have written a second parabola row beside the one already there.

### 92.8 §92.3's trap again, in a tree that had just been cleaned

`rcad-worktrees/fuzz-verify-main` was the tree holding 1907 untracked corpus
inputs, kept out of the sweep until someone decided about them. Discarding them
with `git clean -fd fuzz/corpus/` left it reading **0 dirt** — and holding **88
files named `crash-*` under `fuzz/artifacts/two_solids_fused_by_a_stranger/`**,
all dated 2026-09-07, in no commit on any ref, with no second copy anywhere
under `~/git/mattiasgronlund` or `~/rcad-fuzz-logs`.

The question that cleared the corpus inputs was "is any of them a `crash-*`",
and the answer, no, was a true statement about `fuzz/corpus/` that said nothing
about `fuzz/artifacts/`. Two gitignored paths, one question asked of one of
them. `git clean` scoped to `fuzz/corpus/` could not have reached the artifacts
and did not, so nothing was lost — but the tree came out of it looking emptier
than it was, which is the state §92.3 is about.

Replayed against `main` at `dba39bd`, all 88 pass: `cargo +nightly fuzz run
two_solids_fused_by_a_stranger <the 88> -- -runs=0`, 89 runs, no crash. They are
witnesses to bugs the six days since have fixed.

**Seven of the 88 earned a place in the corpus, and the measurement is not the
one §85, §88 and `b7bda36` used.** Those three justify a seed by subtraction
against the corpus row — *"746 of 827 seeds, 4217 points, 545 orders, 1089
scalings; remove it and it is exactly the 745 of 826 that was published
before"*. That row is the `where_two_surfaces_cross` test walking its corpus.
**`two_solids_fused_by_a_stranger` has no such row**: nothing in
`crates/` reads that corpus directory, so there is no published number to
subtract a seed from. Its inputs reach the code only through `cargo fuzz`.

So the subtraction that exists for a fuzz corpus is coverage, and libFuzzer
already does it: `-merge=1` copies an input into the destination only when it
adds a feature the destination does not have. Against the tracked 1740, the 88
take it to **1747 — seven added, 81 already covered**. Those seven are committed
under their content hashes, which is what the merge names them and what the rest
of that directory is named.

**The other 81 are not deleted, they are filed.** All 88 originals went to
`~/rcad-fuzz-logs/artifacts-rescued-2026-09-13/`, matching the
`artifacts-rescued-2026-09-11/` beside it. An input that buys no coverage today
is still the thing that reproduced a bug on 2026-09-07, and the archive is where
that belongs when the corpus will not have it. With the bytes in two places
neither of which is a lock, the tree stops being anything's only copy.

