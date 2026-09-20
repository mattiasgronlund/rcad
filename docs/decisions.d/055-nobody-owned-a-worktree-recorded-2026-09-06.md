## 55. Nobody owned a worktree, recorded 2026-09-06

§54 is left for §53's unit record. This section is numbered 55 because a peer
session had already claimed 54 for that, in a handoff rather than in this file —
which is, in miniature, the whole subject of this section.

### 55.1 The alarm, and what was actually wrong

Sessions kept reporting *"another session is live in my worktree"*. Five Claude
sessions really were in this repository on the evening of 2026-09-05, so most of
those reports were true. One was not: a dispatcher had sent a second agent into a
worktree its own first agent still held, and could not tell the difference.

It could not tell because **nothing recorded who owned a worktree.** Not the
harness, not git, not the skills. Every method available was the wrong shape:

- `ps aux | grep claude` is worse than nothing. It matched twenty processes across
  three unrelated repositories, several stopped since June — and it cannot see the
  hazard at all, because **a subagent runs inside its parent's process**. The two
  agents that actually collided were one pid with one working directory.
- `ListAgents` lists peer sessions but shows no working directory, and sessions
  rename themselves mid-flight; one message bounced for that the same evening.
- A handoff document is a memory, and `/clear` had already destroyed what it was
  remembering. The 2026-09-05 handoff said *"No lanes running"* on the writing
  agent's own initiative. Nothing had checked, and nothing could have.

### 55.2 The demonstration

Measured 2026-09-05 and 2026-09-06 from the session transcripts under
`~/.claude/projects/-home-mattias-git-mattiasgronlund-rcad/` and from a probe.

| Fact | How it was shown |
|---|---|
| Two agents shared one worktree for 27 minutes | `agent-a0f7fc3c66d82650c` ran 18:25:48Z–19:54:40Z in `rcad-worktrees/spike-floor`; `agent-accf6823fe09975e8` was dispatched into the same worktree at 19:27:22Z |
| A report is not the end of a lane | the dispatcher had sent that spike three questions by `SendMessage`; it answered the first at 19:26Z and the dispatcher dispatched over it 100 s later |
| The harness does not lock what it creates | a live `isolation: "worktree"` agent reported **no `locked` line for its own worktree**, and no worktree in the repository was locked while it ran |
| The harness *does* set the agent's working directory | that probe's own `pwd` was its worktree |
| A hand-made worktree does not | every subagent in the 2026-09-05 transcripts has `cwd` = the primary tree, including those whose brief named a worktree |
| No worktree carried an owner | no `.git/worktrees/*/locked` file existed at 22:45Z, `spike-floor` included, with a live agent in it |

### 55.3 What it cost, on one evening

- A spike lost both its remaining answers: its probe was SIGTERMed mid-run and its
  output file filled with the other agent's output.
- A `just default` "green twice" that the dispatcher itself flagged as measured
  over another writer's uncommitted work, in a merge tree shared with a live
  session.
- A clean fast-forward merge handed back to the user undone, because `main` was
  checked out in the worktree that session was writing.
- An accidental write to the primary tree, from a lane that had to `cd` to reach
  the worktree its brief named — *"the second time today"*.
- `just prune-worktrees` crashing on a locked worktree, fixed the day before.

### 55.4 The lock is the record

**A worktree's git lock is this repository's statement of who owns it.** It was
chosen over inventing a registry for one reason: it is the only candidate that
survives the session that wrote it. It is a file in `.git/worktrees/<name>/locked`,
not session state, so `/clear` cannot lose it and a session that never dispatched
the lane can still read it. `prune-worktrees` already honoured it. Locking blocks
only `worktree remove` and `worktree prune`, so it never obstructs a lane.

**The reason string is ASCII.** `git worktree list --porcelain` C-quotes a reason
holding any non-ASCII byte — a `·` reaches a porcelain reader as `\302\267`. This
was found by `check-lanes-selftest` on its first run, which is the argument for
having written it. `just lanes` sidesteps it by reading the lock file directly;
`prune-worktrees` matches on the listing and does not.

### 55.5 What changed

| # | change | what proves it |
|---|---|---|
| 1 | `just lanes` — every worktree, whether held, by whom, and whether dirty | `just check-lanes-selftest`, in `default`; builds a repository with a held and a free worktree and asserts the reason comes back verbatim |
| 2 | `.claude/hooks/worktree-create.sh` locks what it creates, naming the agent and the ISO time | `.claude/hooks/check-worktree-create.sh`, in `check-hooks`; fails with *"came back with no lock"* when the lock is stripped out |
| 3 | the shared-tree memory drops `ps aux` for `just lanes`, `ListAgents` and `git status` in the specific tree | not in this repository; it is a user memory |
| 4 | `delegating`: `just lanes` before dispatching into an existing worktree, and **a lane is finished when its worktree unlocks, not when it reports** | prose |
| 5 | `handoff`: a required "still in flight" section, generated from `just lanes` and `ListAgents` rather than recalled | prose |
| 6 | `delegating`: the merge tree is yours alone and is not where anyone else has `main` checked out; unlock as you merge | prose |

Items 4–6 are prose because they are judgements about when to dispatch, not
behaviours a command can check. Items 1 and 2 are the ones that had to be
mechanical, and both carry a test that fails without the change.

### 55.6 What this does not close

**Nothing releases a lock automatically.** The create hook writes one; the
dispatcher is told to unlock at merge; if it does not, `prune-worktrees` keeps the
worktree for ever rather than reclaiming it. That is the safe direction to fail —
§50.8's 166 GB came from the other one — but it means an abandoned lock is rubbish
only a person clears, and the ISO date in the reason is the only thing separating
a live lane from a dead one. A `SubagentStop` hook that unlocks is the obvious
next move and is deliberately not built here: §13 wants the consumer named first,
and one run's worth of locks is not yet that consumer.

### 55.7 A claim this section does not make

The investigation was first written up around the sentence **"a subagent outlives
the session that dispatched it"**, and it is not in this section because it was
never demonstrated. Its one specimen was an agent still writing after its parent
session's apparent death, with a `/clear` fifteen seconds later that looked like
the successor. The parent was idle between turns, not dead: it went on to send a
long message twenty minutes afterwards. The `/clear` had started a different
session in a different terminal, which has six events and never ran.

Two timestamps close together were read as cause and effect and nothing checked.
Every item above stands on the other rows of §55.2, none of which depend on it.
The claim is cheap to settle — dispatch a long `isolation: "worktree"` agent,
`/clear`, and watch whether its transcript keeps growing — and it is left open
rather than assumed, in either direction.

