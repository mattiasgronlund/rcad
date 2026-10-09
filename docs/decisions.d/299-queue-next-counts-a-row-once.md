## 299. `queue next` counts a row once, through a role handoff — 2026-09-27

*Queue row Q217. `just queue running` double-counted a row once its lane had
handed over to its verifier or merger. Fixed in `tooling/queue/queue.py`.*

### 299.1 The bug

§263.4's fallback adds a "taken, no tree yet" line for a held row whenever no
currently locked tree carries its exact recorded branch. That was sound before
§266: the dispatcher ran the verifier and the merge by hand, and the lane's own
tree, on its own branch, stayed locked for the row's whole life.

§266.2 changed that on purpose: "a row holds one slot at a time. The verifier
unlocks the lane's tree when it starts, the merger unlocks the verifier's, and
the merger unlocks its own last." Once the verifier's tree takes over, it is
detached — no branch to match — and a merger's tree carries `merge-<label>`,
not `lane-<label>`. The fallback's exact-branch test does not know either
shape, so from the moment a lane hands off, `running` counts the row twice:
once for the tree that is actually doing the work, anonymously, and again
through the fallback, because the row's own recorded branch matches nothing
any more. At 00:40Z on 09-27, Q124 and Q128 were both stuck exactly there — a
release nobody wrote (their claims holds outlived the rows), so it was the
*verifier's* tree, not a fresh one, that made the miscount visible.

### 299.2 The fix

`has_a_tree(branch, branches, trees)` in `tooling/queue/queue.py` widens the
match:

- the exact branch, as before (the lane's own tree);
- `merge-<label>`, derived from `lane-<label>` (§266.3's naming), for the
  merger's tree — exact and unambiguous, since a label is unique to its row;
- for the verifier's tree, which is detached and carries neither, ancestry:
  the row's branch tip, read fresh with `git rev-parse` (the branch survives
  its worktree's removal; only an explicit `git branch -d` would lose it), must
  be an ancestor of some detached locked tree's `HEAD`. A verifier is checked
  out from the lane's tip and only ever adds commits on top of it, so the tip
  stays an ancestor for as long as the verifier runs — checked live on this
  machine: a lane's tree and its verifier's detached tree shared one `HEAD`
  exactly (`agent-abf7a9365b7098176` `lane-q150-torn-totals` and the detached
  `agent-ac4af49f33c73d902`, both at `0cd483a8`).

That ancestry test alone is too wide: if the branch's tip has already landed on
`main` (a stale hold nobody released), it is an ancestor of *every* tree at or
past that point, including an unrelated one. `already_landed` excludes that
case, so a landed-but-still-held row keeps the old, over-counting behaviour
(erring toward full, §263.4's cheaper mistake) rather than being wrongly
absorbed into someone else's tree.

### 299.3 What this does not fix — closed by Q234, §299.5

If a row's lane branch is deleted outright (not just its worktree removed)
while the row is still held and its verifier's tree is detached, no signal
links the two any more, and the row goes back to being double-counted. That is
a narrower, further-degraded state than Q124/Q128's (deleting an unmerged
row's branch is itself the mistake), and is left as a known gap rather than
chased further here.

### 299.4 The test

`check-queue-api-selftest` (`justfile`) gained two cases: one row whose lane
worktree is removed while its verifier's detached tree stays locked (the
count must show the tree once, not the tree plus a "no tree yet" line), and a
second, independent row in the same state at the same time (the reverse
miscount: two held runs must not collapse onto one slot). Reverting
`has_a_tree`'s call in `running()` to the old `hold["branch"] not in branches`
fails the first case (`5 running` instead of `4`, with `lane-gone` showing "no
tree yet" beside its own already-counted verifier tree); the tests were not
reverted.

### 299.5 Q234 closes §299.3, by matching on count rather than identity

*748f6342, `queue: count a held row once when its lane branch is deleted
outright`.*

**Why nothing persisted outlives the branch.** A `take` event records the
dispatcher's own tree and the branch name (`Claims.holds()` stores `time`,
`tree`, `branch` and `write_set` — nothing else), never the lane's own tree and
never a tip. That is not an oversight to fix by adding one: a tip recorded at
take time would only be the base commit, since `take` happens as the branch is
created off it, before the lane has made a commit of its own — indistinguishable
from every other row taken off the same base, and useless for telling rows
apart once one of them is gone. So once `git branch -D` removes the branch,
`branch_tip`'s `git rev-parse --verify` has nothing left to read, and
`has_a_tree`'s ancestry test — the only thing that ties a row to its detached
verifier tree — has no tip left to test ancestry against.

**The design.** `has_a_tree` now returns `None` for a branch with no ref left
to resolve, kept apart from `False` (a row with a live branch and provably no
tree of its own). `running` sets every such row aside in a `branch_gone` list
instead of deciding on the spot, finishes matching every row that still has a
live branch — each ancestry match claims its detached tree's path into a
`claimed` set shared across the whole call, so two rows never claim the same
tree — and then counts the detached lane trees no row's ancestry claimed
(`spare`). That many of the `branch_gone` rows are given a pass: no "no tree
yet" fallback line, on the assumption that a spare tree is theirs. The rest of
`branch_gone` get the fallback line, as before. This is a match by **count**,
one row to one tree, not by identity: once the tip is gone there is no signal
left that could say which branch-gone row a given spare tree belongs to.

**`check-queue-api-selftest`'s two cases** (`justfile`, added by 748f6342).
Both build a row in exactly this state — a lane worktree removed, its
verifier's detached tree still held and locked, then the lane branch itself
deleted with `git branch -D` — and read `running`'s output:

- one row in this state: its detached verifier tree must be counted once, and
  the row must not also print a "no tree yet" line beside it;
- a second, independent row in the same state at the same time: both detached
  trees must be counted, and neither row may print "no tree yet" — the reverse
  miscount, two held runs collapsing onto one slot.

Both fail against `queue.py` reverted to base, with the exact miscount §299.3
named.

**What the match does not cover.** The count-only match assumes every spare
tree belongs to a branch-gone row, and nothing checks that. A detached tree
left over from a row that has already landed on `main` while its claims hold
outlived it — §299.1's Q124/Q128 shape — is real, still locked, and never
added to `claimed`: `has_a_tree` returns `False` rather than `True` for a
landed tip **on purpose**, so that a stale hold is counted rather than hidden
behind an unrelated tree (its own doc comment says so). The same is true of a
tree whose row was instead matched by exact branch name or by its `merge-`
branch — but those two never leave a *detached* tree behind to spare, since
the lane's own tree and the merger's tree both carry a real branch name, not
`(detached)`. The landed-tip case does, though: it counts as `spare` like any
other unclaimed detached tree. So if a branch-gone row with genuinely no tree
left anywhere coexists with a landed-but-still-held row's leftover detached
tree, the spare count can absorb the branch-gone row against a tree that is
not its own, and its fallback line is wrongly dropped — undercounting
`running` by one. This was read out of the code at merge time, not reproduced
with a test, so it is named here as a reasoned gap rather than a measured one.

### 299.6 Q1511: a read-only lane that leaves its branch is found by its reflog

**The bug.** On 2026-10-09, from Q278's take at 06:47:15Z to its done at
07:09:24Z, `just queue running` counted Q278 twice: once as the detached
locked tree `wf_5b5d60fe-c73-1`, with no row, and once as "Q278 taken from
clean-up-dead-sessions, no tree yet". It printed 6 against 4 slots. The lane
ran `git switch -c lane-q278-the-path-in-the-binary 6d970f8b`, then `git
switch --detach f85f926d` to build an old commit. Q278 writes nothing, so its
branch tip stayed at its base, which is on main. `has_a_tree` reads a held
branch whose tip is on main as a stale hold (`already_landed`, §299.2), and the
tree is detached at a commit the tip is not an ancestor of, so neither the
branch name nor ancestry finds it. Any read-only lane that leaves its branch
is counted twice for the whole of its run.

**The rule.** A detached lane tree whose own HEAD reflog says it left a held
branch is that row's tree. `branches_left` in `tooling/queue/queue.py` reads
each detached lane tree's reflog, newest first, and takes the first
`checkout: moving from <name> to ...` whose `<name>` is not a full hash; if
that name is a held branch, the tree is the row's. `running` claims those
trees before it matches any row, so neither another row's ancestry match nor
§299.5's spare count can take one. `has_a_tree` answers yes for such a branch
before it reads the tip, so a tip on main does not matter.

This does not undo §299.2. §299.2 refuses to give a landed tip an *unrelated*
tree by ancestry. The reflog is the tree's own record that it stood on that
branch, so the tree it names is the row's own. A stale hold whose own lane
tree still stands, detached off the branch, is now counted once, as that tree.
That tree holds a slot until someone unlocks it, so once is right.

**What the reflog says, measured.** In a scratch repository, `git switch -c
lane-q278 main` then `git switch --detach <old>` then `git switch --detach
main` left, newest first: `checkout: moving from <40 hex digits> to main`,
`checkout: moving from lane-q278 to <40 hex digits>`, `checkout: moving from
worktree-wf-1 to lane-q278`. A move between two detached commits names the one
it left by its full hash, which is why the walk skips hashes. The live
verifier trees `wf_9ac233ed-029-2` and `wf_cddff73b-695-2` read `checkout:
moving from worktree-wf_9ac233ed-029-2 to lane-q103-three-quarter-bend-validates`
and the same shape for Q345: the hook's own branch is on the left, which is no
held branch, so a verifier's tree still goes to ancestry as before. A tree made
with `git worktree add --detach` has an empty reflog and names nothing.

**The page makes the same match.** `running_rows` in
`tooling/queue-page/render.py` calls `has_a_tree` row by row and asks
`tree_for` which tree matched. It now passes `left` too, and `tree_for` answers
with the tree the reflog named, which is never among the newly claimed ones.
Without that, `tree_for` raises its `AssertionError` on any such row.

**The tests.** `check-queue-api-selftest` makes a locked tree on its own
`worktree-…` branch, switches it to `lane-left` at main and then detached to
the root commit, takes a row with write set `-` onto `lane-left`, and checks
that `running` lists the tree and no "no tree yet" line for `lane-left`. A
second case holds `lane-stale`, a branch on main that no tree left, and checks
that it still gets its "no tree yet" line. `check-queue-page-selftest` adds
Q17, whose tree `wf_page-3` left `lane-q17` the same way, and checks that its
running line names `wf_page-3`. Against HEAD's `tooling/` with the new
justfile, the first fails with "a read-only lane that left its branch was
counted again as treeless" and the page case with "Q17's running line"; the
stale case passes there too, as it should.

**What it does not cover.** A reflog that cannot be read, because the tree's
directory is gone or reflogs are off, names nothing, and the row is counted
twice: the cheaper mistake of §263.4. It narrows §299.5's gap only for
leftover trees whose reflog names their branch. The two verifier trees
measured above name the hook's branch instead, so a landed-but-held row's
leftover verifier tree still counts as spare.

**Q692's gap, seen on the way.** A first draft of the test compared
`running --count` before and after the take, and the count rose by one (8
after). After the take, `lane-q2`, a hold whose branch was never made, printed
"no tree yet". Before it, the new tree belonged to no held branch, so it
counted as spare, and the count was one lower. That fits §299.5's gap: a
branch-gone row absorbed by a tree that is not its own. Only the two totals
were measured, not the line, so this is a lead for Q692, not a reproduction.
