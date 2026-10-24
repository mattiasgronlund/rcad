## 384. The lane writes its rows, `just land` merges, and a lander that throws goes to the Opus merger — 2026-10-03

*Q792 builds what §383.4 items 1 to 3 and §383.5 settle. This section records
what was built, the three things §383 left open, and how each is tested.*

In short: a writing lane now reports `rows`, the `add`, `tag` and `after` steps
of one `just queue apply`. When its verifier says `merge` and every step is one
`just land` takes, `lanes.js` sends a Haiku `lander`. The lander writes the
steps to a file and runs `just land` once. A `fix_first`, rows `just land`
cannot take, every stop, and a lander that throws go to the Opus merger.

### 384.1 What was built

- **`lane.md`** gains the paragraph on `rows`. Each step is a list of words,
  so `lanes.js` never has to split text. The rules for tier, class, `small`,
  write set and `after` moved there from `merger.md` step 3.4. The lane looks
  for an open row in its own copy of `docs/queue.md` before it adds one.
  `reportSchema` requires `rows` from a writing lane; an empty list means
  nothing to add.
- **`verifier.md`** gains "The rows": each step matches `remaining`, none
  repeats an open row on `origin/main`, and each follows `lane.md`'s rules. A
  bad step is a `fix_first`.
- **`merger.md`** step 3.4 now takes the lane's `rows` as its `add` steps, and
  writes its own only for a finding it did not fix, or for an older lane with
  no `rows`. Step 1 gains the paragraph for a merge `just land` did not finish.
- **`tooling/land/land.py`**, run by `just land`, does `merger.md`'s steps 1,
  3, 4 and 5 with `MERGE_ATTEMPTS` 3 and `DOCS_CATCH_UPS` 5, as `lanes.js`
  has them. It is Python because the tree it runs from is switched under it:
  Python reads the whole file before it starts, bash reads as it goes. It ends
  with one line of JSON holding `mergedSchema`'s fields and `stop`, and exits 1
  on a stop. `[no-exit-message]` keeps `just`'s own error line from landing
  after that JSON.
- **`.claude/agents/lander.md`**, Haiku, written on the pattern of
  `pruner.md`.
- **`lanes.js`**: `merging()` picks the lander or the merger, and `lastMerge`
  holds all of it, the lander and any merger after it, behind the merge before.

§383.4 item 2 names six stops: `sha-differs`, `conflict`, `section-0`,
`check-queue`, `gate` and `attempts`. The script names more, each for a case
where going on would be a guess:

- `queue-apply`: a refused batch writes nothing, the same case as `check-queue`.
- `already-on-main`: `origin/main` holds the lane's tip, so a merge already
  landed it.
- `push`: the push was refused with `origin/main` still behind this branch, so
  no race explains it, and retrying would loop.
- `usage`, `steps` and `tree`: input it cannot use, or a hub or dirty tree. All
  three stop before anything changes.
- `fetch`, `switch`, `commit` and `git`: a git command failed.

On every stop it still unlocks its own tree, last, as `merger.md` step 5 does.
Without that, the tree holds a slot (§263.4). It runs `ff-main` and
`mark-finished` only after its push goes through.

### 384.2 How the steps travel: a file the lander writes, one word to a line

The Bash guard of a worktree-isolated agent refuses `$( )`, variables and
loops. The lander's command must also start with the text
`.claude/settings.json` allows, `mise exec -- just land`. The row, branch, sha,
merge branch and trees are plain words, so they go on the command line as
literal text. The steps cannot go there as plainly, because a row's text holds
quotes, backticks, `$` and `§`.

There were two ways to send them:

- **One quoted argument.** `lanes.js` would shell-quote every word, using
  `'\''` for each apostrophe, and the lander would paste the result. Whether
  the guard and the permission prefix match accept a backtick or `$(` inside
  single quotes was not tested. If either one asks instead, nobody is there to
  answer.
- **A file.** The lander runs `git rev-parse --show-toplevel` and writes
  `target/land-steps.txt` under that path with the Write tool, which every
  lane uses on every run. The recipe takes it as `--steps`. A file with one
  word to a line and a `+` line between steps needs no escaping anywhere:
  neither the shell nor the lander ever reads it as code. `target/` is
  gitignored, so the file leaves `git status --short` clean.

The file was taken. It costs the lander two more turns. It also avoids the
untested question about the guard rather than answering it. `lanes.js` checks
the shape of each step before it sends a lander (`stepsProblem`). Rows it
cannot send go to the Opus merger. `land.py` checks them again and stops at
`steps`.

### 384.3 A lander that throws goes to the Opus merger

A new agent type reaches only a `claude` process started after its file
exists (§266.6, §383.7). The dispatcher that runs Q793 started before
`lander.md` existed, and it runs Q793 through this `lanes.js`. Its lander will
throw "Agent type 'lander' not found". `merging()` catches the throw and hands
the lane to the Opus merger as a stop. The row does not end there. The
merger's branch is `merge-<label>-after-land`, since the lander's
`merge-<label>` may exist.

**Every merger sent after a lander first checks `git merge-base
--is-ancestor <lane branch> origin/main`, not only after a throw.** From
`lanes.js`, three things look the same: a lander that died after its push, a
`just land` the Bash tool ended at its ten minutes after the push, and a
lander whose JSON went missing. On exit 0 the merger merges nothing. It runs
`ff-main` and `mark-finished`, and reports the merge that is already there.

**The ten minutes.** `just land` runs in one Bash call, which the harness
ends at 600 s. A served gate takes about a minute (§367). A fresh gate in a
cold tree can take the nine minutes the gate's `[doc]` names, and a catch-up
over code adds another gate. A call ended there prints no JSON, and `lanes.js`
sends it to the merger like a throw.

**Measured 2026-10-24**, over the same `lander` and `merger` agents §383.6
counts, since Q792 landed on 2026-10-03: 70 of the 793 landers (8.8%) were
followed by a merger on the same lane-attempt, found by joining `(run, lane)`
and checking that both roles are present. Of those 70, 45 ended with
`land.py`'s own named `stop` and a completing line of JSON — a clean exit,
just not a mergeable one: `queue-apply` 22, `gate` 20, `push` 2, `conflict` 1.
The other 25 — **36% of the 70 escalations, 3.2% of all 793 landers, about one
lander in 32** — are this paragraph's case: `stop` came back empty and the
recipe's last line was not JSON, because the run was killed mid-recipe, most
often during `just gate quiet`. The lander's own report says so in
`not_merged_because`, in its own words each time: "Recipe was terminated by
signal 15 during gate execution", "the run (pid 1316687) ended without
finishing; .../target/land/log holds what it ran", "Exit code 137: timeout
during second `just gate quiet` run", and two reports that left
`not_merged_because` blank and gave no reason at all, a further way a lander
can under-report this case (§384.2's instructions ask for the last lines; not
every lander gave them). A further 117 lane-attempts reached the merger with
no lander at all in the same window — `fix_first`, rows `just land` cannot
take, or an older lane with no `rows` — outside this count, since they never
hit the ten minutes to begin with. Methodology as in §383.6.

### 384.4 How it is tested

- **`check-land-selftest`**, in `gate_steps` and `gate_reusable_steps`. It has
  12 arms, each in a throwaway repository with a bare `origin`. A stub on
  `LAND_JUST` stands in for `just`. One clean merge lands: the lane's tip is
  the second parent, and one `queue apply` holds the `done` and the lane's
  steps. The stub logs `queue apply`, `check-queue`, `gate`, `ff-main` and
  `mark-finished`, in that order, and all three trees end up unlocked. A
  catch-up over docs alone costs no attempt. Each other arm stops by name and
  pushes nothing. Three breaks were run, and each failed it:
  - with the sha check disabled, the `sha-differs` arm says "it exited 0 on a
    stop";
  - with every catch-up counted as an attempt, `attempts-over-docs` says "3
    attempts, 0 catch-ups and 3 gates";
  - with `land.py` moved away, the `clean` arm says its last line is not JSON.

  `check-gate-runs-every-check` counts 55.
- **`check-lanes-workflow`** gains an eighteenth arm, and the fourth, fifth,
  sixth and eleventh arms now cover the lander as well:
  - the eighteenth holds the land-only-on-`merge` line, the line that sends a
    `fix_first` to the merger, the stop line, the throw line, the merger's own
    branch, and the `origin/main` check in both the brief and `merger.md`;
  - the fourth wants a `lander` type and its file;
  - the fifth holds lander and merger behind `lastMerge`;
  - the sixth bars the lander from `git push`;
  - the eleventh wants the foreground rule in `lander.md`.

  There are 12 new cuts, 62 in all. With `lanes.js` at the base, the check
  fails: "sends no agent of type lander".
- **`lanes.js` under node**, outside the gate. `mise exec node@26.10.0 --`
  parsed it inside an async function, with `export const meta` made `const`,
  and ran it against stub agents:
  - `merge` with rows went lane, verifier, lander, pruner;
  - `merge` with no rows went the same way, with no `--steps`;
  - a lander that stopped went lane, verifier, lander, merger on
    `merge-x-after-land`, and so did one that threw;
  - `fix_first` and malformed rows went lane, verifier, merger on `merge-x`.

### 384.5 What this leaves

- **`.claude/skills/dispatch-lanes/SKILL.md`** still says, in its step 5, that
  every stage throws in a session older than the agent files. A missing
  `lander` type no longer does. Its step 6 does not mention the lander's tree.
  After a stop, that tree is unlocked but not marked finished. Outside this
  row's write set.
- **The lander's tree after a stop past the commit** stays on its
  `merge-<label>` branch, with a merge commit that is not on `main`. So
  `prune-worktrees` keeps it, as it keeps a merger tree that never pushed
  today.
- **§383.7's settings line** is committed now (5deb8706), so the lander's
  command does not stop to ask.
- **Q793**, `just verify-lane`, follows. It changes `lanes.js` and
  `verifier.md` too. Done 2026-10-03 (§385).
