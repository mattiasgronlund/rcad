## 170. Dispatch tooling: one lanes workflow instead of two retyped scripts, and a lane told when to stop — 2026-09-20

*Two Workflow runs on 2026-09-19 and 2026-09-20 used scripts ~90% identical,
retyped the second time. They are now `.claude/workflows/lanes.js`, taking the
base and a list of lanes. The second half is the context budget hook, which
exited on any `agent_id` because "a lane is short by construction and has
nowhere to hand off to". The second clause was wrong — a lane's handoff is its
report — and the first was never measured and is false: 134 lanes over nine
days reached a median peak of 134k tokens and a worst of 335k. A lane is short
in minutes, which the 45-minute rule caps; minutes are not tokens. It now hears
its own dial at 150k and is told to commit, report and stop.*

### 170.1 What the two scripts had in common, and what the file keeps

The generalisation keeps the parts that were doing work in both, and they are
the parts that turn a CLAUDE.md sentence into something the script holds:

- **A `schema` on every `agent()` call.** A lane cannot return prose. The report
  carries `start_sha`, `base_sha`, `landed_sha`, `branch`, `outcome`,
  `files_changed`, `tests_that_fail_without_the_change`,
  `numbers: [{what, value, command}]`, `out_of_write_set_needs` and `remaining`.
  The shape of `numbers` is the enforcement: "never restate a number you did not
  run yourself" is a required `command` field rather than a sentence in a brief.
- **The base-sha check in script code.** A worktree-isolated lane branches from
  the session's HEAD, not from `origin/main`, so a brief that does not pin the
  base gets a lane measuring a different tree. If `base_sha` is not the pinned
  base the lane is not verified at all and the script says so: verifying those
  claims against this base would only report that they disagree, which is known.
  It never fired in either run, and it is four lines.
- **A Sonnet verifier per *writing* lane**, handed the lane's JSON and told not
  to trust it: the branch touches only the declared files, the claimed tests
  pass, the non-test source reverted to base makes them **fail**, `just gate
  quiet` on the branch tip, every number re-run, and every "cannot" or "limit"
  claim held against §16.10. It caught two real things across the two runs — a
  gate not run on a docs commit, and an unbacked §16.10-shaped claim in §169.4.
- **Read-only lanes get no verifier** and are told to write no tracked file.
- Verification is skipped, out loud, when the lane was blocked or named no
  failing test.

One thing is new. A `merge` verdict is now cross-examined by the script: if the
verifier came back `merge` while `tests_fail_without_fix` is not true, or the
lane declared no source directory to revert, or the branch touched files outside
the write set, the run says so beside the verdict. A `merge` that never watched
the tests fail is a merge on the verifier's word, which is the one thing the
verifier exists to replace.

### 170.2 A named workflow is its `meta.name`, not its filename

Worth writing down because guessing it wrong costs a dispatch. Claude Code
2.1.276 loads `.claude/workflows/*.js` from the project, parses the `meta`
block, and registers the workflow under `meta.name` — the loader's own line is
`return {source: s, name: c.meta.name, ...}`, and the plugin variant prefixes it
as `${plugin}:${meta.name}`. The filename is never the handle; it only has to
end in `.js`, and the file has to be under 512 KB.

The registry is **memoised for the life of the session** (`s.workflows ??=`), so
a workflow written mid-session does not resolve by name in that session — the
attempt here answered *`Workflow "lanes" not found. Available: deep-research`*
with the file already on disk and correct. It resolves from the next session on.
`{scriptPath: ...}` bypasses the registry and was how this one was smoke-tested:
two runs with deliberately bad `args`, zero agents, 10 ms and 6 ms, each
stopping at the guard it was aimed at. That is the whole test the file has —
there is no JS runtime in `mise.toml`, so the interesting half, the pure
functions that decide which lane gets a verifier, is checked by reading and by
the next real dispatch. §170.5 says what that would cost to fix.

### 170.3 A lane is not short in tokens

Measured over 2026-09-12 to 2026-09-21, lane transcripts only:

```
just spend 2026-09-12 2026-09-21 ceiling subagents
```

134 lanes, 6,698 API responses, **$433.55**. The `subagents` argument is
`spend.py`'s path filter, and a lane transcript is the only kind with
`subagents` in its path, so the existing report already answers this question
without a new one.

Peak context reached per lane — the figure the hook compares against a budget:

| p50 | p75 | p90 | p95 | p99 | worst |
|---|---|---|---|---|---|
| 134,189 | 170,271 | 220,617 | 237,450 | 299,691 | 334,658 |

So the median lane goes two thirds of the way to the session budget and one lane
in ten passes it. "Short by construction" was true of the clock and assumed of
the context: lanes do obey the 45-minute cap — median 14 minutes over this
window — and a lane can re-read a great deal in 14 minutes.

The tail is what makes a warning worth sending. A lane that crosses 140k bills a
median of 22 further responses and a p90 of 75, so the warning does not arrive
as the lane is already finishing. That is the opposite of the failure the
session dial has, where the warning was on time and ignored.

### 170.4 Why 150k, and why not the session's 200k

The session dial was set on this test: the line where about a third of the spend
is billed above it — 200k took 35% of that week, and 250k was rejected at 16%.
The same statistic for lanes, spend billed in responses whose own context was at
or above the line:

| 100k | 120k | 140k | **150k** | 160k | 175k | 200k |
|---|---|---|---|---|---|---|
| 72.1% | 58.7% | 43.7% | **37.7%** | 31.0% | 22.4% | 12.9% |

150k lands on the precedent almost exactly. It warns 53 of the 134 lanes (40%),
a median of two rungs each and eight in the worst, at the session's 25k step.

**The session's 200k would not have been worth a separate dial**, which is the
point of not simply deleting the exemption: it reaches 12.9% by this measure and
1.3% by `ceiling`'s, against 5.9% at 150k. The two measures disagree because
they price different things — `ceiling` bills cache reads at `min(context, cap)`
and lets the work carry on, while the table above counts everything billed past
the line as if it stopped. Neither is the real saving, which sits between them:
a lane that stops hands its remainder to a fresh lane that re-reads some of it.
What both agree on is the ordering, and that 200k is near enough to nothing.

Three things the lane branch does that the session branch does not:

- **Its own state file.** Several lanes share one `session_id`, so a ladder
  keyed on that alone would let the first lane's rung silence every other lane
  of the same dispatch. The key is `$session-$agent_id`.
- **A different instruction.** It is told to commit, report the remainder and
  stop — never to call the handoff skill, because there is no next session to
  pick it up — and it is not offered the budget-raising advice, because a lane
  cannot set its own environment. The dispatcher can, with `CLAUDE_LANE_BUDGET`.
- **A different headline.** "Lane budget passed at N tokens" rather than
  "Context budget", so the two stay countable apart in a transcript and
  `spend.py`'s `SAID` regex keeps matching only session warnings.

Six cases in `check-context-budget.sh` cover these; all six fail against the
hook as it was at `3e476a4` and pass after, checked by restoring that version
into a copy of the plugin and running the new cases against it.

### 170.5 What is left

- **`lanes.js` has no selftest**, only the two zero-agent smoke runs. A real one
  needs a JS runtime in `mise.toml`, which puts a node install on every runner
  and in every fresh worktree for one script. Deferred rather than refused: the
  cost is a toolchain, the benefit is catching a `meta` that stopped being a
  pure literal or a `Date.now()` that crept in, both of which currently cost a
  dispatch to find. Worth revisiting if a second workflow script appears.
- **The lane dial is dated.** §170.3's command re-measures the money; the peak
  and rung distributions came from a one-off script and would need rebuilding —
  read every `agent-*.jsonl` under `<project>/<session>/subagents/`, take each
  response's `input + cache_creation + cache_read`, dedupe on
  `(requestId, message.id)`, and take the max per file. A `lanes` report in
  `spend.py` would make that a command, which is this repo's own rule; it was
  not built here because §13 wants a named consumer and the only one so far is
  this section.
- **Nothing measures whether lanes obey it.** The session rung's own finding was
  that being warned on time is not the same as heeding it. The equivalent
  question for lanes — what is billed after a lane warning — needs the warning
  to have been firing for a while first.

