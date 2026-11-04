## 479. Where a rule for an agent goes — 2026-11-04

*Queue row Q3162, filed and pinned at Mattias's word. It extends §266.6, which
found when a running claude reads its agent files, with what that costs and
where a rule goes instead.*

**In short.** An agent file — `CLAUDE.md`, `AGENTS.md` (a link to it) or one
of `.claude/agents/*.md` — reaches a running dispatcher only when it starts
again, so every edit to one empties slots until Mattias starts a new process.
Most of the last six edits held a rule a tool could have printed or kept. So
`docs/where-a-rule-goes.md` now says what an agent file is for and the three
places that beat it, a PostToolUse hook points a write to an agent file at that
page, and `just queue add` and `writes` point a row whose write set names one
at it too.

### 479.1 What the six edits cost

Six commits changed `.claude/agents/` in the 30 hours from 2026-11-03 03:40Z
to 2026-11-04 09:32Z, by their author dates: Q2975, Q3096, Q2996, Q3108, Q3068
and Q3102. Run in this lane:

```
git log --format=%h --since=2026-11-03T00:00:00Z --until=2026-11-04T15:20:00Z -- .claude/agents/ | wc -l
6
```

§266.6 showed a running claude reads its agent types only when its process
starts, and Q643 found a dispatcher on 2026-09-29 sending lanes with a lane.md
older than main's. So after each of the six landed, the dispatcher drained its
runs, rather than send more on the old types, and its slots stood empty until
Mattias started a new process. **Q3162's own text measured that, and this lane
did not run it again:** *"29.3 of the 65 empty slot-hours from 2026-11-03
00:00Z to 2026-11-04 15:20Z, read from just row-time --json against the
dispatcher transcripts."*

A seventh, Q3157 (`cdafa1d80`, 2026-11-04 16:27Z), changed lander.md after
that window. It changed what step 1 copies, from a `+` line form to JSON, which
is a change to the step itself and so belongs in the agent file under 479.2.

### 479.2 What an agent file is for, and what beats it

**Decided:** an agent file holds the role, the order of its steps, and a rule
that holds on every run. A rule that only applies when some case comes up goes,
in this order of preference:

1. **into the tool, which enforces it, and the sentence goes;**
2. **into the tool's print, beside the case it applies to;**
3. **into a file the agent reads at the step that needs it,** which the step
   names.

Why this order: a rule the tool keeps holds for every agent, every human and
every old process at once, and needs nobody to read anything. A print is made
fresh on every run, and only on runs where its case is there. A file read at
its step is read fresh too, and only by the runs that reach the step. An agent
file is none of these: it is read once per process and whole on every run.

When a rule does go into an agent file, its commit says which of the three
places was weighed and why none would do, and the report or handoff says a
new process is needed to see it.

`docs/where-a-rule-goes.md` is the one copy of the working guide, with a
worked example from the six for each place. This section holds the argument
and the record; it does not repeat the guide.

### 479.3 Where each of the six would have gone

Read from each commit's diff of `.claude/agents/` (`git show <sha> --
.claude/agents/`):

| Row | Agent file | What it added | Place |
|---|---|---|---|
| Q3068 (`a6514a8e5`) | lander.md | no lone `+` after the last step | 1: `land.py` drops it, in the same commit |
| Q3102 (`1bcacb1d6`) | lane.md | stop every process you start | 1: `prune-worktrees` stops them in a finished tree, in the same commit |
| Q2975 (`6e624336e`) | prep.md | read the newer set off `show`'s two lines | 2: one line from `show` naming the set in force |
| Q3108 (`366a11afa`) | verifier.md | the same, for the verifier | 2: the same line |
| Q2996 (`f3965fad5`) | verifier.md | what to do when `commands_that_cannot_run` is not empty | 2: `verify-lane` prints it beside the key |
| Q3096 (`59cca1ef9`) | prep.md | a row asking for a failing test is not `sameBehaviour` | 3: a file on the marks, read when prep sets one |

Five of the six are about reading a tool's output or were already kept by the
tool. Q3096 is judgement for one step.

The Q3102 row is the weakest of the two under place 1. `prune-worktrees` stops
a finished tree's processes only when a merger marks the tree finished, so a
lane's leftover runs from its report until then. If that time costs cores, the
tool that ends the lane's run is where to close the gap.

**Not done here:** moving the six rules out of the agent files. That changes
`show`, `verify-lane` and four agent files, and it is a row of its own (479.5).

### 479.4 What points at the guide

**The hook.** `.claude/hooks/agent-file-written.sh` runs after `Write` and
`Edit`, beside `decisions-zero.sh` in `.claude/settings.json`, and is shaped as
that hook is: it reads `tool_input.file_path` with `jq`, matches with one
`case`, and answers in `additionalContext`, since a PostToolUse hook's plain
stdout reaches the debug log and not the model (152.4). It matches `CLAUDE.md`,
`AGENTS.md` and `.claude/agents/*.md` by name, at the top of a path or after
any `/`. `AGENTS.md` is a link to `CLAUDE.md`, and the payload names whichever
path the write was given, so both names match.

It points and does not block. Some edits to an agent file are right — a new
step, a changed order — and a block would cost each of them a round trip for
nothing. Its check, `.claude/hooks/check-agent-file-written.sh`, run from
`check-hooks`, writes to one path of each name and wants the whole line, and
one write to the guide itself and wants nothing.

**The queue.** `just queue add` and `just queue writes` print the same line on
stderr, after the row and the entries that matched, when a write set names an
agent file: an entry `CLAUDE.md` or `AGENTS.md` at any depth, a `.md` under
`.claude/agents/`, or a directory that holds `.claude/agents/`, such as
`.claude/`, since that row may write any agent file. It reaches whoever files
or widens such a row, before a lane is dispatched on it.

It goes to stderr, and only once the batch is written. `just land` reads the
ids an `apply` added off its stdout (`apply_queue` in `tooling/land/land.py`),
and so does every `$(just queue add ...)` in the selftests; a refused batch
added no row, so it points at nothing. `check-queue-api-selftest` runs the hook
for the line it wants, so the hook and `queue.py`, which each hold the text,
cannot drift apart without that selftest failing.

### 479.5 What this leaves

- **Moving the six rules** out of prep.md, verifier.md, lander.md and lane.md
  into the places 479.3 names. Filed by this lane as its own row.
- **Whether a dispatched lane reads `CLAUDE.md` from its own tree or from the
  dispatcher's loaded copy** was not measured. This lane's own `CLAUDE.md`
  arrived under the dispatcher's tree's path. Until someone measures it, the
  pointer says only what §266.6 showed.
