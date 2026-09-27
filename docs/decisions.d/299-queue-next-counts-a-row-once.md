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

### 299.3 What this does not fix

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
