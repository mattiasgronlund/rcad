## 180. `gate-fresh` records its green, and the stale watermark that stranded the night — 2026-09-20

*Mattias asked whether the night fuzzes an up-to-date commit or the last one CI
went green on. Neither: it was **stranded**, 61 commits back, and had been since
the Actions allowance ran out. §173.9 had already built the way out — a local
`just gate` counts as green — and the night could not reach it, because the
night reads its own decision procedure out of the tree it is deciding about.
That is fixed by hand. What Mattias asked for is the second half: `gate-fresh`,
the strongest green this machine can produce, was the one green that was never
written down, because `exec docker` left no process to write it.*

### 180.1 The night reads its watermark from the tree it is standing on

`rcad-fuzz.service` runs `just fuzz-overnight 28800` with `WorkingDirectory` set
to `rcad-nightly-fuzz`, a worktree parked on a detached HEAD. So the recipe
body, and the `fuzz-watermark` it calls, are read from **whatever commit that
worktree already has checked out** — not from `main`. `fuzz-overnight` knows
this about itself and says so: it `exec`s a second copy of itself after
advancing, with the comment that otherwise *"a change to this recipe takes
effect the night after it lands"*.

The re-exec is conditional on advancing. When the watermark declines to move,
the night never reaches the newly checked-out justfile, because there is no
newly checked-out justfile. **So a change to `fuzz-watermark` that is needed in
order to advance can never take effect.** It is a fixed point: the recipe that
would unstick the night is only ever read after the night has already become
unstuck.

This is §173's class turned inwards. There, a check answered a property of the
environment it ran in rather than of the tree. Here, a recipe answers a question
about which commit to move to, using a copy of itself taken from the commit it
is trying to move off.

### 180.2 What that cost, measured

The worktree stood on `146d0d62` (2026-09-19 19:56), detached and locked
`overnight fuzzing`.

- **61 commits behind** `main`'s tip `a3f8ffe9` — `git rev-list --count
  146d0d62..a3f8ffe9`.
- The newest green `ci.yml` run on `main` was `146d0d62` **itself**: `gh run
  list --workflow ci.yml --branch main --status success` names it first.
- Every run on `main` of 2026-09-20 failed in two to six seconds with **zero
  steps**. The annotation is §172's and not a regression: *"The job was not
  started because recent account payments have failed or your spending limit
  needs to be increased."*
- `146d0d62:justfile` contains no `rcad-greens` and no `record-green` — grep
  returns nothing. §173.9's two-source watermark landed in `6cd17bba`, which is
  newer, so the running copy asked GitHub and nothing else.

Run for real in that worktree, the stale recipe printed the whole failure in one
line, to stderr, where only a night reads it:

    staying on 146d0d62: already on the last green

Stdout was empty, which is how that recipe says *do not move*. It was correct
about its own question and wrong about the night: `146d0d62` **was** the last
CI-green commit, and would have stayed the last one for as long as the billing
stayed unpaid. Meanwhile the local record it could not read held **14 greens
from 2026-09-20 alone**, `a3f8ffe9` among them, recorded at 18:57.

### 180.3 Unstuck by hand, because nothing else can reach it

`rcad-nightly-fuzz` was checked out to `a3f8ffe9` — a commit the local record
names green, which is exactly the move §173.9's watermark would have made if it
had been the copy running. From there the night reads the current justfile and
maintains itself again.

**This is the honest shape of the fix and not a workaround.** A bootstrap that
requires one manual step is what a fixed point costs; the alternative is the
night executing a `fuzz-watermark` fetched from a commit it has not established
anything about, which trades a stranded night for an unreviewed one and is the
wrong side of §81.4.

What is *not* fixed is the silence, and **§81.4's indicator is the thing that
failed quietest**. The night does print how far behind it is, and last night it
printed this:

    2026-09-20 08:12  146d0d62 (0 days old, 6 behind main)  5 targets, ...

Six. That reads like a night keeping up. It is measured at 00:10, against a
`main` that had not yet had its day, and by the time Mattias asked it was 61 —
the same commit, the same night, an indicator ten times out. §81.4 replaced
*age* with *distance* because a commit can be one day old and sixty-eight
behind; distance taken once at the start has the same defect in the other
direction. Neither number is wrong. Both are answers about the moment the night
began, printed into a log nobody opens unless they already suspect something,
and `just status` — which does report `main` against the last green — knows
nothing about where the night stands at all. §180.5.

### 180.4 `gate-fresh`'s green died inside the container

`gate-fresh` ended with `exec docker run`, and the gate inside the container
reaches `_record-green` like any other: it wrote `/rcad/.git/rcad-greens`, in
the throwaway clone, which the `trap` deletes. Nothing reached the host.

So the ordering was backwards. A plain `just gate` in a warm tree counted
towards the night's watermark; **the whole gate on a fresh clone with this
machine's environment taken away — §77.6's manoeuvre, the arm that survives a
spent allowance (§172) — counted for nothing.** The strongest green available
was the only one leaving no trace.

`exec` is the whole of the cause, and it took two other things with it. It
replaces the shell, so no line could follow the container; and the `trap` that
removes the clone could never fire either, leaving every run's tree behind in
`$TMPDIR`.

The recipe now runs docker rather than `exec`ing it, carries the container's
status out unchanged, and on a pass calls `_record-green` on the **host**,
naming both `--justfile` and `--working-directory` so the tree the record
describes is the one `git clone` read. `_record-green` re-checks cleanliness
itself, which is what catches a tree written to during the ten minutes the
container was running.

### 180.5 The indicator, named and not built

The stranding was silent for a day, and the cheap thing that would have spoken
is a row in `just status` beside the one that already reads

    main tip a3f8ffe9, last green a3f8ffe9 (a local gate on linuxws, ...), 0 commit(s) on main not yet green

saying where the night stands and how far back that is. `status` already has
every ingredient: the nightly worktree is in `git worktree list`, and the
distance is one `rev-list --count`.

**Deferred under §13, deliberately, with the consumer named.** The consumer
exists — this section is it — but the failure it watches for has happened once,
and the hand-advance in §180.3 has already removed the condition. Building it
now would be guessing at which of several numbers matters: how far behind `main`
the night stands, or how far behind *the last green*, or whether its justfile
differs from `origin/main`'s in `fuzz-watermark` specifically, which is the
thing that actually strands it. If it recurs, the last of those three is the one
to build, because it is the only one that is a fixed point rather than an
ordinary lag.

### 180.6 What is checked

`check-gate-fresh-selftest`, run from `gate`.

The container is stubbed, not run: ten minutes and a docker daemon are not what
the question is about. The question is whether a pass reaches `_record-green` on
the host and a failure does not, and a stub exiting with whichever status the
arm wants answers exactly that. It is the same fixture shape as
`check-fuzz-watermark-selftest`'s stub `gh`, and for the same reason — no arm
needs the network, a daemon, or the state of this tree. `RCAD_STEP_CORPUS`
points at an empty directory for the same reason again: the recipe only checks
that one is there, and reading the real one would make the arm answer a question
about the machine (§77.6).

Two arms:

- **a container that passed is a green on the host**, under the sha that was
  cloned. This is the arm `exec docker` fails *while exiting 0*, which is the
  whole reason it went unnoticed.
- **a container that failed records nothing**, and its status is what comes out.
  Bookkeeping that ran anyway would write a green for a gate that went red, and
  that is the one entry no reader of `rcad-greens` could catch.

Run side by side rather than argued: against a copy of the `justfile` with
`exec` put back, arm one fails with *"it passed and wrote no green for
27bc974"*; against the tree as it stands, both pass.

`check-gate-runs-every-check`'s declared count moves 41 → 42 in the same commit,
which is that recipe's own instruction and the only way a new `check-*` lands.
