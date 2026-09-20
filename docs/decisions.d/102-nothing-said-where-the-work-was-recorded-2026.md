## 102. Nothing said where the work was, recorded 2026-09-14

### 102.1 What it cost to find out

Asked "where does this repo stand, and what is next", a session starting cold
spent twelve tool calls and one lookup lane. The answer, once assembled, was
four sentences long: M0–M5 are built, M6 has not started, two branches carry
unlanded work, and the named-unit chain stopped on 2026-09-05.

Every part of that was in the repository. None of it was in one place.

`docs/roadmap.md` has **one commit in the whole history** — `0a8aa33`, the
scaffold, alongside 236 on `docs/decisions.md`. That is not neglect, it is §1
working: the roadmap is the north star and not a description of this codebase,
so it lists M0–M6 with no status markers and correctly never gains any. It can
never answer "what is next" because answering that would make it the second
thing it was told not to be.

`docs/decisions.md` is 27 000 lines under a 98-line table of contents that is
genuinely good, and still cannot answer it either. The TOC is ordered by
recording date and its titles are incidents — *"A box that rounded itself
away"*, *"A night that crashed on its own oracle"*. That is the right shape for
a record of what was settled and the wrong shape for a summary of where the
settling has got to.

Outside `decisions.md`, grepping every `.md`, every `.toml` and the `justfile`:
the only mentions of M5 or M6 anywhere are the roadmap's plan list and one
past-tense cost anecdote in `CLAUDE.md`.

### 102.2 §0, and why not a file of its own

§60's argument applies directly: a second record beside the thing itself is a
second thing to disagree. A `docs/status.md` would have to carry the deferrals
— §43.8 item 8, §43.8 item 12, §89.7 — and those are already sentences in
`decisions.md`. Two copies of a deferral is one deferral that gets closed in
one place.

So the summary goes **into** `decisions.md`, as `## 0. Where the work is`.
Section zero rather than an unnumbered block because `check-decisions` requires
every heading to be addressable by number and checks that the numbers ascend:
`0` satisfies both with no change to that recipe, sorts above §1 in
`decisions-toc`, and `just decision 0` reaches it like any other section. An
unnumbered `## Where the work is` would have had to be exempted, and an
exemption in a check is where the next unaddressable heading goes to hide.

### 102.3 What the check can and cannot hold it to

`check-where-the-work-is` cannot check the judgement. Whether the unit named in
§0 is still the unit is not a property of the file, and a check that pretended
otherwise would either fail on every new section or pass on a block two
milestones stale.

It holds the three things that are properties of the file, each of which makes
a stale block read as a current one:

- **A cited section that no longer exists.** §0 points at the sections that
  defer its work, and §71 is four renumberings in one session. A block citing a
  §43.8 that became §44.8 still reads as current.
- **A `reviewed:` date `date -d` cannot parse.** `just status` prints the
  block's age from that line; an unparsable date prints no age at all, which
  looks exactly like a block reviewed this morning.
- **Fifty non-blank lines.** `decisions.md` is 27 000 lines because nothing
  capped it, which is right for a record and fatal for a summary. The cap is
  what keeps §0 a thing that gets re-read.

Age is printed rather than enforced, for the same reason: how stale is too
stale is a judgement, and a survey that refuses to run because nobody has
touched §0 in a fortnight is a survey that gets deleted.

### 102.4 `git cherry`, not ahead-of-main

The row nothing printed before is which branches carry work `main` does not
have, and `just lanes` cannot answer it: it lists worktrees, and a worktree is
not a branch.

The obvious implementation is wrong. On 2026-09-14 this tree had 28 local
branches; `git rev-list --count main..<branch>` called five of them unmerged.
Three of those five carried nothing: `merge/justfile`'s single commit added
`fuzz-cones-and-tori`, a recipe that had been on `main` for days by another
route, and two `worktree-agent-*` branches held commits that reached `main`
split into two. Ahead-of-main counts reachability; what a reader wants is what
is unlanded, and the two disagree on every branch whose commits got there by a
cherry-pick, a rebase or a split.

`git cherry main <branch>` compares patch ids and marks an already-applied
commit `-`. Counting only its `+` lines turns the five-branch answer into a
two-branch one with no human reading five diffs. It also skips merge commits
entirely, which is the same thing wanted here: a lane that merged `main` into
itself has landed nothing.

The remaining 23 branches are counted and not listed. The count is what says
the sweep ran at all — §55's rule that an empty answer must fail rather than
pass quietly — and listing 23 names nobody has to act on is how a survey stops
being read, which is §99's finding about the recipes nobody ran.

