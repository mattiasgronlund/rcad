## 210. The tree that holds `main` is called `main`, and `just ff-main` catches it up — 2026-09-23

*Mattias's, on reading `just lanes`: the tree called `wave-1` has nothing to do
with any wave, it is the tree merges land in, and there should be a recipe that
fast-forwards it instead of a paragraph asking him to.*

### 210.1 The name said nothing

`rcad-worktrees/wave-1` was created for the first batch of lanes and outlived
the batch by a month. Nothing there was ever about wave one; what is true of it
is that `main` is checked out in it, which is §60's other half — a hub holding
`main` drags every merge back into the one tree whose owner cannot be recorded,
so `main` lives in a worktree like any lane.

A name nobody can derive is a name every document has to carry. Six of them
carried it: five sections of this file and four separate memory notes, each
spelling the path out so the next session could type it. None of the tooling
did — `justfile`, `tooling/` and `.claude/` mention it nowhere, because every
recipe that needs that tree already finds it by the branch it holds.

The sentence that stood here said the rename therefore cost exactly the
documents and the code was already immune. That was wrong within the hour, and
210.4 is what it cost.

It is `rcad-worktrees/main`, moved with `git worktree move` on 2026-09-23.
`just lanes` now prints `free clean …/main [main]`, where the two halves agree.

**The sections above are left as they were written.** §60, §76, §111, §119,
§126 and §146 describe runs made in a tree that was called `wave-1` at the
time, and editing the name into them would make each one a claim about a run
that did not happen. This section is the forwarding address.

**Git's own name for it is still `wave-1`.** `git worktree move` updates the
paths and leaves the administrative directory — `.git/worktrees/wave-1` — under
the name it was added with. Nothing in this repo reads that name: `lanes` finds
the lock file through `git rev-parse --git-dir`, and everything else goes
through `git worktree list --porcelain`, which does not expose it. Removing and
re-adding the worktree is the supported way to make the two agree, and it is
not worth it: between the remove and the add, `main` is checked out nowhere,
and that is a worse state to be interrupted in than a stale string in a
directory nobody opens.

### 210.2 `just ff-main`

The catch-up was a hand-typed line, passed between sessions in memory notes:

    git -C ~/git/mattiasgronlund/rcad-worktrees/wave-1 merge --ff-only origin/main

It is a recipe now. It reads `git worktree list --porcelain`, takes the tree
holding `refs/heads/main`, and fast-forwards it. **It finds that tree by the
branch and never by the name**, which is the half that makes 210.1 free: the
next rename costs nothing, because nothing spells the path.

Four of its five outcomes are refusals, and each shape means something
different:

- **Behind** — it fast-forwards, and names the tree, the count and both shas.
- **Already there** — a pass that prints the sha, rather than going quiet.
- **Ahead** — the commits are local and want `git push origin main`. A
  fast-forward here is a rewind.
- **Diverged** — somebody rewrote one side. Picking a resolution here would
  pick it unseen, so it prints the `--left-right` log to read first.
- **Dirty** — it touches nothing, whatever the shas say. That tree is left
  unlocked by convention (§60), so uncommitted work in it names no owner; the
  answer is to ask the peers, not to merge over it.

**It fetches**, which `just` itself must never do — §146 refused a fetch in the
hub re-park because that runs before every recipe, and a network round trip on
every `just t` is §144's cost in another currency. This one is typed on
purpose, once, and "aligned with `origin/main`" is a claim about the remote and
not about whatever a stale remote-tracking ref last said.

`check-ff-main-selftest` pins it against a repository it builds itself, with a
real `origin` on disk, because the live one is in one of the five states at a
time and the one it is in most often is the one a broken recipe still passes.
The dirty case asserts the bytes survive and not just the exit code: it is the
only outcome whose wrong answer is unrecoverable. Breaking the guard makes that
case fail, which is how it was checked.

### 210.3 What a worktree-isolated session can reach

A session whose worktree is isolated has its Bash calls screened, and
`git -C <sibling>` is refused there: *"a worktree-isolated session's git
operations must target its own worktree"*. The hand-typed line above is exactly
that shape, so the sessions that most need to land work were the ones that
could not, and the paragraph asking Mattias to run it by hand is what grew in
its place.

`just ff-main` is a reviewed operation on a named tree with five outcomes and a
selftest, rather than an arbitrary path typed into `git -C`. That is the
difference the screen is drawing, and it is why the recipe is the right shape
for this and a looser one would not be.

### 210.4 The move stalled a peer session, and `just lanes` could not have warned

Within the hour, a peer reported every lane of a four-lane batch dying in
130 ms, before any agent started:

    WorktreeCreate hook failed:
    /home/mattias/git/mattiasgronlund/rcad-worktrees/wave-1/.claude/hooks/worktree-create.sh: not found

That session had been launched in the tree this section renamed.
`CLAUDE_PROJECT_DIR` is fixed at session launch and does not follow a
`git worktree move`, and **all five** repository-local entries in
`.claude/settings.json` are relative to it — `session-start-hub.sh`,
`session-start-guardrails.sh`, `worktree-create.sh`, `decisions-zero.sh` and
the status line. Every one of them pointed at a path that no longer existed.
The hook bodies were never the problem: they derive the repository from
`git rev-parse --show-toplevel` at the cwd, so only the lookup path was stale.

**`free clean` on that row is the one cell in `lanes` that carries no
information.** The check made before the move was `just lanes` plus
`git status --porcelain`, and both were clean. Neither could have said
otherwise: that tree is left unlocked *by convention* so any session can merge
there (§60), which is the same hole as a tree everyone merges in being a tree
nobody owns. A session having it as its project directory is occupancy that
neither a lock nor a status can see. So on that row, and only that row, `free`
means "no lock was taken", never "nobody is there".

**Before moving or removing a worktree, ask the peers.** `ListAgents` and one
`SendMessage`, not a reading of `lanes`. This was already the rule for finding
uncommitted work in that tree; the move is the case that shows it applies to a
tree that is spotless.

**The symlink is refused, and it should stay refused.** The obvious repair is
`ln -s main …/rcad-worktrees/wave-1`, which is inert to git — `worktree list`
reports real paths — and fixes all five entries at once. The peer's classifier
declined it as a shared-resource change and the peer did not ask anyone else to
type it, which is right twice over: being the hands for another session's
refusal is the same rule read backwards, and the symlink restores the exact
name 210.1 removed. A bridge that is never taken down is the stale path back.
The route is to restart the affected session, which re-resolves
`CLAUDE_PROJECT_DIR` — and which also moves it out of the tree that holds
`main`, where no session should have been launched in the first place.

**The fragility is general and outlives this rename.** Any `worktree move`
breaks the hooks of any session launched in the tree that moved. Deriving the
repository at run time instead of trusting `CLAUDE_PROJECT_DIR` would close it,
and that is a `.claude/settings.json` change, which is Mattias's to make and
not a lane's.
