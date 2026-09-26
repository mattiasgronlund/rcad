## 263. Queue changes go through `just queue` — 2026-09-26

*Queue row Q140. Two dispatchers were editing `docs/queue.md` by hand, where
§230.2 had pictured one. The brief behind Q140 reports a merge conflict in the
file (428deed4), an `after:Q80` written while Q80 existed only in the other
session's copy (against 2f0c0bd6), and `just queue next` saying "took 1 of 1"
with every slot full, because it counted `lane:` rows in one copy. This section
says what replaced the hand edit, and amends §230.2 and §230.8.*

  263.1   What was wrong
  263.2   The verbs, and the rules they hold (amends §230.2)
  263.3   One claims file, under one lock
  263.4   What `queue next` counts (amends §230.8)
  263.5   `after:` on a row only another copy holds
  263.6   A merge driver that merges by row
  263.7   A guard on the file itself
  263.8   The tests
  263.9   What is left

### 263.1 What was wrong

§230.2 said the dispatcher edits the file, and that no two lanes conflict over
it because a lane never writes it. Both halves held; neither covered two
dispatchers. Each had its own copy in its own worktree, each edited by hand,
and nothing was shared between them until a merge — which is exactly when a
text merge of one file both had changed near the same rows goes wrong. The
fill had the same blind spot from the other side: it counted slots from the
`lane:` rows of the copy it read, so a lane the other dispatcher had sent, and
not yet merged, held a slot that `next` believed was free.

### 263.2 The verbs, and the rules they hold (amends §230.2)

The dispatcher still writes the file, and a lane still never does. What
changes is how: through one verb per change, never by hand.

```
just queue take <id> <branch>
just queue add <tier> <model> <write set> <what, ending in §N>
just queue done <id> <merge sha>
just queue drop <id> <§N>
just queue park <id> <§N>
just queue after <id> <id>[,<id>...]
just queue pick <id> ours|theirs
just queue show <id>
just queue running
```

`just queue`, `just queue ready` and `just queue next [N]` read as before. Each
verb acts on one row, takes the lock of §263.3, and before it writes runs
`check-queue` on the file as it would be — so every rule of §230.2 that the
gate holds, the verb holds at the moment of writing, from the same awk, with no
second copy of it. On top of that:

- `take` refuses a row that is not ready — `todo`, or `after:` with every row it
  names `done` in this copy — and a row held elsewhere: by another worktree's
  hold in the claims file, or `lane:` in another dispatcher's copy, committed or
  not. It writes `lane:<branch>`, as a hand edit did. A `lane:` row this
  worktree holds can be taken again for a new branch, which is a failed lane
  sent again.
- `add` hands out the id, one past the highest `Q` on every branch, in every
  worktree's copy and in the claims file, the way §159 hands out section
  numbers. It puts a `todo` row at the foot of its tier and prints the id.
- `done` takes the merge sha, refuses one that is not a commit here, and moves
  the row under `## Closed`. `drop` takes a section and does the same. `park`
  takes a section and leaves the row in its tier. All three end the row's hold.
- `after` refuses an id that exists nowhere — §263.5.
- `pick` settles a row the merge driver left in conflict — §263.6. Every other
  verb refuses while the file holds a conflict.

The file format does not change. A file edited by hand still parses and still
passes `check-queue`, because the other dispatcher keeps editing by hand until
this lands, and a hand edit is still what `check-queue` checks.

### 263.3 One claims file, under one lock

`rcad-queue-claims` in the git common directory, beside §159's
`rcad-section-claims`: shared by every worktree of the clone, carried by no
branch, appended to under `flock` on `rcad-queue-claims.lock`. It is a log, a
line per event — `add`, `take`, `release` — tab-separated, with the time and
the worktree that wrote it. A hold is the latest `take` of a row not followed by
a `release`.

The hold is what closes the race: a second worktree's `take` of the same row is
refused under the lock, before either worktree has committed `docs/queue.md`.
Reading the other dispatcher's copy catches the rest while that dispatcher
still edits by hand, since a hand edit writes no hold.

### 263.4 What `queue next` counts (amends §230.8)

§230.8's cap stays at six. What fills it is now **the machine**: every worktree
an agent has locked that is not a session's own tree, plus every row taken with
no tree yet. `just queue running` lists them; `next` uses the count.

That is the right count because it is the only record both dispatchers already
write without being asked. The worktree hook locks every tree it makes with
`agent <name> since <time>` (§55), and puts a session's tree in
`<hub>-worktrees/<name>` and a subagent's in `<session tree>-worktrees/<name>`.
So an agent lock one level below the hub's directory is a dispatcher's own
tree, which holds no slot, and one anywhere else is a lane or a verifier, which
does — including a verifier, which has no row at all (§230.4), and a lane the
other dispatcher sent and has not committed. A `lane:` row counts nothing; a
hold whose branch has no tree yet counts one, so a `take` followed by the
`Agent` call is never a slot counted free in between.

It errs toward full. A lane stays locked until the dispatcher unlocks it at
merge, so a finished lane waiting on its verifier holds a slot beside that
verifier, and a lock nobody released holds one for good. Both leave a slot
empty rather than overfilling one, which is the cheaper mistake of §230.8's two
limits, and `just lanes` shows which tree it is. A dispatcher running from the
hub, which §60 forbids, would have its lanes read as session trees.

Write sets still come from rows: `lane:` rows in this copy, and rows another
dispatcher holds, from its hold in the claims file or from its own copy. A
ready row that another worktree holds is not offered at all.

On 2026-09-26, while this lane ran, `just queue running` listed five:
two lanes the other dispatcher had sent, one of them `lane-q143-…`, a row this
copy did not have, one lane still on its `worktree-agent-…` branch, and the two
lanes of this batch.

### 263.5 `after:` on a row only another copy holds

`after` accepts an id that is not in this copy when it exists anywhere else: a
claim in the claims file, another worktree's copy, or a branch. It refuses one
that exists nowhere. That is the 2f0c0bd6 case made legal on purpose — a row
waiting on the other dispatcher's new row is a true statement — while a typo
is refused at the moment it is typed.

`check-queue` therefore accepts an `after:` naming a `Q` id the file does not
hold, and says how many on its pass line. It cannot do better: it checks a
tree, and a tree cannot see another worktree, so on the runner a real row not
merged yet and a typo look the same. Checking against the claims file would
pass here and fail on CI for the same commit, which is §173's kind of check. A
`T` or `D` id is never handed out again, so a missing one is still refused.

### 263.6 A merge driver that merges by row

`.gitattributes` names `merge=queue-rows` for `docs/queue.md`, and
`tooling/queue/merge.py` is the driver. Per row id: the same on both sides, or
changed on one side only, takes that side, in that side's place — so a row
`done` moved under `## Closed` arrives there. Rows added on both sides under
different ids are both kept. A row changed differently on both sides, or
added twice under one id with different text, is left with both versions
between conflict markers where it stands; the driver names it and exits 1, so
git marks the file unmerged. `just queue pick <id> ours|theirs` keeps one side
and puts it under the heading its state belongs to. The text outside the rows
is taken from whichever side changed it; if both did, the whole file goes to
`git merge-file`, git's line merge, and says so.

git ignores a driver `.gitattributes` names and its config does not define,
and falls back to the line merge without a word. So `check-queue-merge-driver`
fails when this clone's `merge.queue-rows.driver` is missing or spelt
differently, and names `mise exec -- just queue-merge-driver`, which writes it.
One registration serves every worktree, since they share the config. It was
registered on this clone on 2026-09-26 by this lane.

**Skipped, not passed, where there is no Claude Code** — on CI and under
`gate-fresh`, keyed on `~/.claude` as `check-plugins` is (§77). What it asks is
whether this clone merges the queue by row, and a clone nobody dispatches from
has no answer to give; registering the driver there first would only prove the
recipe can write a config. The driver itself is tested everywhere:
`check-queue-api-selftest` registers it in a scratch repository and merges
through it.

### 263.7 A guard on the file itself

`.claude/hooks/queue-write-guard.sh` is a PreToolUse hook, and the judgement is
`tooling/queue/guard.py`. It refuses Edit and Write on `docs/queue.md`, and a
Bash command that writes it: `sed -i` or `perl -i` on it, a redirect or `tee`
into it, `cp` onto it, `mv` or `rm` of it, `awk -i inplace`, and a Python,
perl, ruby or node script that opens it for writing — the Python case read
with `ast`, following a path through a variable, `pathlib` and `sys.argv`. The
refusal names the `just queue` verbs.

It does not refuse a command that only mentions the path. On 2026-09-26 the
patching hook refused a read-only Python analysis for naming queue.md, and a
guard that cannot tell reading from writing is one people route round. So the
command is tokenized and each simple command judged on what it does: grep,
`git show main:docs/queue.md`, `git diff`, a script that reads it and writes
its report to the scratchpad, and a commit message that describes a write all
pass. git is left alone entirely, since git is how a conflict is settled.

`.claude/settings.json` is Mattias's to change, so the lane built the guard and
left it unregistered. He registered it on 2026-09-26 in 5909be09, merged to
`main` at ccbbcee8: a `PreToolUse` hook with matcher
`Bash|Edit|Write|MultiEdit|NotebookEdit` running
`"$CLAUDE_PROJECT_DIR/.claude/hooks/queue-write-guard.sh"`.

A session keeps the hooks it started with. rcad-18's tree was merged up to
ccbbcee8, and an `Edit` to `docs/queue.md` from that session still went
through, with no hook error, and was put back. So the guard holds for a session
*started* in a tree at or after ccbbcee8. A session already running when it
merges `main` can still edit the file by hand, and should use the verbs anyway.

### 263.8 The tests

`check-queue-api-selftest`, in the gate, builds a hub, two session trees and
lane trees locked as the worktree hook locks them, under `target/`, and never
touches the live file. It holds: a second worktree's `take` of a held row
refused, and a `take` of a waiting row refused; four `add`s at once from two
worktrees getting Q5 to Q8 once each; `after` accepting an id only the other
worktree holds and refusing one that exists nowhere; `running` counting a
lane, its verifier and a take with no tree as three, and two session trees and
an unlocked tree as none; `next` not offering the row the other worktree holds;
`done` moving a row under Closed; two branches that changed different rows
merging cleanly through the driver, where the line merge conflicts; a row
parked on one branch and dropped on the other conflicting loudly, every verb
but `pick` refusing the file, and `pick` settling it. It runs in 5.05 s
(`/usr/bin/time`).

With `tooling/queue/merge.py` moved away, the clean merge fails with git's
`CONFLICT (content)`, so the selftest fails there; without `tooling/queue/` the
first `take` fails.

`check-hooks` runs `.claude/hooks/check-queue-write-guard.sh` over
`queue-write-cases.json`, 40 cases in the shape of
`.claude/patching-cases.local.json`, each writing form above refused and each
reading form allowed, and every refusal naming the verbs.

`check-queue-selftest` changed in two lines: its fixtures pass
`queue_running=1` in place of a machine, since a file under `mktemp` has none
and counting whatever machine the selftest runs on is §173's check; and the
dangling `after:` it refuses is now a `T` id.

### 263.9 What is left

- `add` puts a row at the foot of its tier. There is no verb to rank a row
  higher, and since ccbbcee8 the guard refuses ranking by hand. Q151 adds the
  verb. **Closed by Q151 (b4501695):** `just queue rank <id> before <id>`,
  within one tier.
- A row closed in another copy, or on `main`, and still open in this one is
  offered by `next`: the copy is behind, and `next` reads it as it stands.
  Merging `main` first is the answer today. **Closed by Q151 (b4501695):**
  `next` no longer offers such a row, `just queue` names it as closed
  elsewhere, and it holds no files in the write-set meeting.
- The patching hook in the `guardrails` plugin still refuses a read-only Python
  heredoc that names a tracked `.md` file. That is the plugin's, and this guard
  does not change it.
