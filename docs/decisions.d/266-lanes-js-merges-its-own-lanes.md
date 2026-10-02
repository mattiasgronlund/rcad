## 266. `lanes.js` merges its own lanes — 2026-09-26

*Queue row Q142. A dispatcher now runs one `lanes` workflow per queue row: the
lane, then its verifier, then a merger that lands it on `main` and hands back
about twenty lines. The merge used to happen in the dispatcher's own context.
This section says why, what the merger does, and where the role rules went. It
amends §230.2 and §230.4.*

  266.1   Why: rcad-18's measurements
  266.2   One run per row (amends §230.4)
  266.3   The merger (amends §230.2)
  266.4   No new lock, and a bound on the retry
  266.5   A lane that names no failing test
  266.6   Agent types, and where the rules went
  266.7   The test
  266.8   What this leaves

### 266.1 Why: rcad-18's measurements

These were measured by the dispatcher session rcad-18 on 2026-09-26 and handed
to this lane in its brief. They were not re-run here. The scripts it ran were in
its scratchpad,
`/tmp/claude-1000/-home-mattias-git-mattiasgronlund-rcad-worktrees-lane-budget-rca/3de7f292-6efc-4978-a300-48cc49dadfac/scratchpad/`
(`chain.py`, `ctx.py`, `modes.py`, `modes2.py`, `thinking.py`, `wakeups.py`,
`lanes.py`, `handoffs.py` and others), and were still there when this lane
started.

- The dispatch-next chain, rcad-c0, ran 54 lanes and 21 verifiers as plain
  `Agent` calls and no workflows, over 12 sessions. That cost 36K of
  dispatcher context per lane, a handoff about every 28 minutes, and 127
  wake-ups for 54 lanes.
- The turns that ran `git merge` held 64% of that chain's working growth
  (1.23M of 1.92M tokens). The other 36% was wake-ups when a lane or a
  verifier finished, and background gates.
- Every session starts at 44K to 65K. Thinking stays in context, about 25% of
  a dispatcher's context at xhigh (19 of 19 responses checked).
- Lane slots in use averaged 2.3 to 4.6 of 6. Workflow runs with several lanes
  used 50% to 99% of their slot time — 64% in the 4-lane run `wf_0276dfee`.
- While merging Q140 and Q141 by hand, `main` moved under the dispatcher's gate
  three times in about an hour, and each time the answer was the same: merge
  `origin/main`, gate again, push.

So the merge is what to take out of the dispatcher's context, and it is
mechanical enough to take out: every step of it was already written down in
the old step 7 of the `dispatch-lanes` skill.

### 266.2 One run per row (amends §230.4)

§230.4 said the workflow starts its lanes together and returns when the last
is verified, so a slot whose lane finished first sat empty; rolling dispatch
used one `Agent` call per lane instead, and the dispatcher did the base check,
the verifier and the merge by hand.

With one run per row, the run ends when its row is merged, so the slot still
refills when its own lane is done — §230.4's rule is kept — and the wake-up that
refills it brings a merged result instead of a report to act on. The base check
and the verifier per writing lane, which §230.4 had the dispatcher do by hand,
the script does, as it always did inside a run.

A run can still carry several rows. Their merges then wait for one another:
`lanes.js` starts each merger on one line, `lastMerge.then(() =>
agent(MERGE(…)))`, and chains the next behind it. A probe ran the script's body
under node with `agent()` and `pipeline()` stubbed, three lanes and a merger
that sleeps 20 ms: no two mergers overlapped, and with the chain cut to
`Promise.resolve().then(…)` one pair did (`overlap` 0 and 1, from
`/home/mattias/.local/share/mise/installs/node/26.10.0/bin/node
<scratchpad>/probe-lanes.mjs <lanes.js> three`). The probe is not in the gate:
this repo pins no node, and a check that needs this machine's node is §173's
kind. The same probe ran one lane, a mix of a docs lane, a rejected lane, a
lane on the wrong base and a read-only lane, and a writing lane with no queue
id, which the script refuses before starting anything.

A row holds one slot at a time. The verifier unlocks the lane's tree when it
starts, the merger unlocks the verifier's, and the merger unlocks its own last.
The brief had the merger unlock both the lane's tree and the verifier's; that
would count one row as two slots while it is checked and three while it is
merged, because `just queue next` counts every locked agent tree (§263.4), and
neither tree is used again once the next role has started.

### 266.3 The merger (amends §230.2)

§230.2 had the dispatcher edit `docs/queue.md` — `lane:` at dispatch, `done:` at
merge — and §263.2 made both go through `just queue`. Now the dispatcher still
takes the row, and the merger closes it and adds the rows it leaves. A lane
still never writes the file.

`.claude/agents/merger.md` holds what it does. It is Opus, because it judges
the verifier's findings. It works on its own branch, `merge-<label>`, made from
`origin/main` after a fetch, and merges the lane's branch into it. It makes the
verifier's fixes when each is small and inside the lane's write set, and
otherwise does not merge and says why. Then, all in the merge commit: `just
queue done`, a `just queue add` for everything the lane, the verifier and the
lane's section leave behind (§230.7), a §0 edit if the merge closes what §0
names (§172.5), and `just check-queue`. Then `just gate quiet` on that tree,
the push of 266.4, `just ff-main`, and at most twenty lines back.

**`done` takes the lane's last commit, not the merge commit.** The brief's
usage line says `done <id> <merge sha>`, but the merge commit carries the row
and cannot name its own sha. The rows already closed name the lane's tip:
Q140's `29798e6a`, Q141's `b114fc81` and Q145's `574d8fc5` each have one
parent (`git log --format='%h %p %s' -1 <sha>`), and each was merged by a
separate commit (`f7ea1d09`, `6e0433ee`, `7b30d79c`).

**The dispatcher pushes its `take` before the run.** The merger reads the queue
from `origin/main`. If the `lane:` is there, the merger's flip to `done:` is a
change on one side only, and the row merge driver (§263.6) takes it. If the
`take` sits only on the dispatcher's branch, its next merge of `main` finds the
row changed on both sides, and that costs a `just queue pick`. Step 5 of the
skill says so.

What the workflow returns per row is lines, not records: the merger's result
rendered one fact to a line, or, for a row that did not merge, why, the lane's
summary, the verifier's findings and the trees to unlock. A read-only lane's
answer still comes back whole, because it is the answer. When a merger's result
runs past twenty lines the script says so in its log and returns all of them;
it does not cut them.

### 266.4 No new lock, and a bound on the retry

Merges from different runs and different sessions race for `main`, and they
need no lock. The merger pushes only when `git merge-base --is-ancestor
origin/main HEAD` holds just after a fetch, so its push is a fast-forward, and
`git push` refuses one that is not. `ff-main` refuses anything that is not a
fast-forward too (§210). A merger that loses — the check fails, or the push is
refused — merges `origin/main` again, runs `check-queue`, gates again and tries
again.

The bound is three attempts, `MERGE_ATTEMPTS` in `lanes.js`, passed to the
merger in its brief and reported back as `push_attempts`; the script logs a
count past it. Three is a judgement, not a measurement: `main` moved three
times in an hour under rcad-18's hand merges, and a merger that loses three
gates in a row is racing something the dispatcher should look at. When the
attempts are spent it stops with its branch as it is and says so.

### 266.5 A lane that names no failing test

`lanes.js` skipped the verifier for a lane that named no test failing without
its change, so Q141's CLAUDE.md split needed an audit of its own, which checked
that every moved paragraph landed in exactly one place with its meaning
unchanged. That audit is now the verifier's **moved text** steps: the script
picks them when `tests_that_fail_without_the_change` is empty. Any changed line
that could change what runs makes `merge` impossible, since such a change owes
a test (`CLAUDE.md`'s second principle); until §375 that was any changed file
not ending in `.md`. Every removed paragraph is traced to one
place, or to the record the lane says holds it; every new sentence stating a
fact about the repo is checked against the tree.

A test lane is still refused a merge by the script itself when the verifier did
not see its tests pass on the branch and fail with the source reverted, or when
the lane declared no source to revert — the check the old script only logged.

### 266.6 Agent types, and where the rules went

§261.6 found that `lanes.js` sent every agent without a type, so
`.claude/agents/lane.md` and `verifier.md` never loaded in a workflow. The
workflow-authoring skill documents `agent()`'s options, and one is `agentType`:
*"uses a custom subagent type … resolved from the same registry as the Agent
tool; composes with schema."* So every `agent()` call now names `lane`,
`verifier` or `merger`. That is the documentation's word and not a run's: this
lane had no `Workflow` tool, so the first real run is the demonstration. If a
type does not resolve, that row comes back as "a stage threw".

- **`lane.md`** took `COMMON`'s standing rules: why the switch matters, the
  write set, `git add <path>`, about 45 minutes, a number carries its command,
  no merge and no push. It adds that `landed_sha` comes from `git rev-parse
  HEAD`, and that a lane changing only `.md` files names no failing test. The
  lines that carry this lane's values — the base, the `switch -c`, its refusal —
  stay in `lanes.js`, where `check-lanes-workflow` reads them (§237).
- **`verifier.md`** took `VERIFY`'s steps and is now their one copy. Step 6 of
  the skill described the same job, and now points at the file.
- **`merger.md`** is new; 266.3.
- **Cut, because `CLAUDE.md` says it** and every role loads it: "Read then
  Edit; never patch a tracked file through a shell heredoc".
- **Kept in `lanes.js`, as one block all three roles get**: `mise exec --` on
  every `cargo` and `just`, and the Bash guard's rule against paths it cannot
  resolve. Both concern every role, so they belong in `CLAUDE.md`. That file is
  outside this row's write set, and one block in the script beats three copies
  in the agent files.
- **`lookup.md`** lost its copies of `just src`, `just recipe`, `just
  decision`, `just decisions-find` and the 129 000-token figure (§261.5 holds
  it), and keeps a one-line pointer to the readers. Its $0.10 and 19 000 are
  §158's and stay.

Step 4 of the skill said to take one fewer row for each verifier running,
because `next` counted `lane:` rows. Since §263.4 it counts the trees agents
hold, verifiers included, so the sentence now says that.

**A session cannot see an agent file added after it started.** Added at merge by
rcad-18. The lane had no Workflow tool, so it never ran a type. At merge,
`Agent(subagent_type: "lane")` from rcad-18's session answered *"Agent type
'lane' not found. Available agents: claude, claude-code-guide, Explore,
general-purpose, lookup, Plan, statusline-setup"*. That was so although
`.claude/agents/lane.md` had been in that session's own worktree since
`adf706cd`, half an hour earlier. So a session reads its agent types once, when
it starts, and a dispatcher begun before its tree held the three files gets a
thrown stage on every row. That failure is loud, which is why this merged
anyway. A fresh session's first `lanes` run is still the only test that a type
resolves together with `isolation: 'worktree'` and a schema.

Both halves of that were checked after the merge, on 2026-09-26. **`/clear`
does not re-read the agent files.** rcad-18 was cleared in its tree at
`e079e592`, which holds all three. The new `CLAUDE.md` came back after the
clear, but its Agent tool still listed only `claude, claude-code-guide,
Explore, general-purpose, lookup, Plan, statusline-setup`. **A new process
does.** `claude -p` started in that same tree, then at `c1c57d98`, listed
`claude, Explore, general-purpose, lane, lookup, merger, Plan,
statusline-setup, verifier`. So the files load from the session's own
worktree, not from the hub, whose `.claude/agents/` holds only `lookup.md`. A
handoff that needs the three types needs a new `claude` process, started in a
tree that holds them, and not a `/clear`.

### 266.7 The test

`check-lanes-workflow` keeps its three arms for §237 and gains three:

4. every `agent(` call names an `agentType`, all three roles appear, and each
   type is a file in `.claude/agents/` whose `name:` matches;
5. exactly one line starts the merger, it does so behind `lastMerge`, and the
   next line moves `lastMerge` on;
6. `merger.md` has, once each and in this order, `git fetch origin`, `git
   merge-base --is-ancestor origin/main HEAD`, `git push origin HEAD:main` and
   `mise exec -- just ff-main`, and neither `lanes.js`, `lane.md` nor
   `verifier.md` says `git push`.

Each arm is proved by cutting the real files, eight cuts in all: the three old
ones, a `lanes.js` without the verifier's `agentType`, an agents directory
without `merger.md`, the merger started on `Promise.resolve()`, the ancestor
check and the push swapped, and a `git push` added to `lane.md`. With the base
`lanes.js` put back, the check fails with *"starts 2 agents and names a type
for 0"*. It reads text only: it runs no workflow, pushes nothing and never
opens `docs/queue.md`.

### 266.8 What this leaves

- The first real run of the workflow is what shows that `agentType` loads the
  agent files and works beside `isolation: 'worktree'` and a schema (266.6).
- `mise exec --` and the Bash guard's rule belong in `CLAUDE.md`, and should
  leave `lanes.js` when they get there (266.6).
- §230's own text is not edited. This section records the amendment, as §263
  did.
- `merge-<label>` branches are kept like `lane-*` ones, so a label reused for a
  second merge stops the merger at its switch, as it stops a lane (§237).
- Each merger works in a fresh tree, so its gate runs on a cold `target/`, as a
  verifier's does. If that shows up in a row's wall time, the verifier's tree
  is the obvious one to reuse.
