# Where a rule for an agent goes

An **agent file** is `CLAUDE.md` (and `AGENTS.md`, which is a link to it) or
one of `.claude/agents/*.md`. This page says what belongs in one, and where a
rule goes instead. `docs/decisions.md` §479 is the argument and has what the
last mistakes cost.

## Why it matters

A running claude reads `.claude/agents/*.md` once, when its process starts,
and `CLAUDE.md` when its session starts or clears (§266.6). So an edit to an
agent file leaves every running dispatcher on the old text. It goes on sending
lanes, verifiers and mergers with the old rules, or it drains its runs and
leaves slots empty until Mattias starts a new process.

An agent file is also read whole on every run of its role, whether the case a
rule covers comes up on that run or not.

## What an agent file is for

- **The role**: who the agent is, what it owns, what it hands back.
- **The order of its steps.**
- **A rule that holds on every run**, such as lane.md's "every command you run
  stays in the foreground" or "stage with an explicit `git add <path>`".

A rule that applies only when some case comes up belongs nearer that case. Try
the three places below, in this order, before the agent file.

## 1. The tool enforces the rule, and the sentence goes

If a tool can refuse the wrong thing, or do the right thing itself, make it do
that and drop the sentence. A rule the tool keeps holds for every agent, every
human and every old process at once.

- **Q3068.** lander.md gained "A `+` line goes only between two steps, never
  after the last ... (`just land` drops it before it checks the hash, but do
  not rely on that)". The same commit made `land.py` drop the lone `+`. The
  sentence describes a case the tool already handles. (Q3157 has since made
  the steps JSON, and the sentence went with the line form.)
- **Q3102.** lane.md gained "Stop every process you start before you report".
  The same commit made `prune-worktrees` stop what still runs in a tree a
  merger marked finished. What the paragraph adds is the time between the
  lane's report and that mark, and for a run handed back unmerged there is no
  mark at all, so the tree is never pruned that way. If that costs cores, the
  tool that ends the lane's run is where to close it, not a paragraph every
  lane reads.

## 2. The tool prints the instruction beside the case it applies to

If the rule is about reading a tool's output, change the output so it cannot be
misread, or print the instruction on the line it is about. The print is made
fresh on every run, and only when its case is there.

- **Q2975 and Q3108.** `just queue show` prints a row's write-set column, and
  under it, on some rows, `narrowed to <set>, taken with <...>` or ``write set
  set to <set> by `writes`, <column> in docs/queue.md``. Q2975 rewrote
  prep.md's paragraph, and Q3108 wrote one into verifier.md, each saying to
  read the newer set off those lines and not the column. Q3166 made `show`
  print one `write set in force:` line, so prep.md and verifier.md name that
  line and need neither paragraph.
- **Q2996.** `just verify-lane` gained the key `commands_that_cannot_run`, and
  verifier.md gained a bullet saying what to do when it is not empty. Q3166
  made the tool print that instruction beside the key, in
  `about_commands_that_cannot_run`, only on a run where the key is not empty.

## 3. A file the agent reads at the step that needs it

If the rule is judgement that only one step needs, put it in a file under
`docs/` and have the step name the file. The agent reads it with `Read` when it
reaches that step, so an edit reaches the next run at once, and the runs that
never reach the step never read it.

- **Q3096.** prep.md gained a paragraph saying a row that asks for a test that
  fails without its change is not `sameBehaviour`. It concerns one step,
  choosing a mark. It now lives in `docs/prep-marks.md`, which prep reads when
  it is about to set one; prep.md names that file and no more.

## When it has to be the agent file

Some rules do belong there: a new step, a change to the order of the steps, or
a rule that holds on every run. Then:

- **Say in the commit why the rule had to go there**: which of the three places
  above you weighed, and why none of them would do.
- **Say in your report or handoff that every running dispatcher needs a new
  process** to see it. A `/clear` does not reload the agent types (§266.6).

## What points here

- `.claude/hooks/agent-file-written.sh`, after a `Write` or `Edit` to an agent
  file, puts a one-line pointer to this page in front of whoever wrote it.
- `just queue add` and `just queue writes` print the same line on stderr when a
  write set names an agent file, or a directory that holds `.claude/agents/`.
