## 235. The lane budget measured the dispatcher, not the lane — 2026-09-25

*§170 gave lanes a context budget of their own, at 150k. From 2026-09-20 to 25
every lane that heard it was hearing its dispatcher's size. Two lanes stopped
with nothing done because of it, and the fix proposed for it would not have
reached a lane. Fixed in guardrails v0.3.7, `ecc2886` in `claude-questions`.*

  235.1   What went wrong
  235.2   How it was shown
  235.3   Why nothing caught it for five days
  235.4   What changed, and what did not

### 235.1 What went wrong

Every hook that fires inside a subagent is given the **session's**
`transcript_path`, not the lane's. The budget hook read the last usage row of
the file it was handed, so each lane was told its dispatcher's size as if it
were its own. No field of a PostToolUse payload names the lane's own file:
`agent_transcript_path` exists only on SubagentStop, in claude 2.1.281.

§170.4's dial is not what was wrong. It was chosen on the lanes' own
transcripts, through `just spend`, and it still describes them. It was just
never applied to them.

### 235.2 How it was shown

| what | lanes |
|---|---|
| warned 2026-09-22..25; the figure matches a row of the dispatcher's transcript | 58 of 58 |
| the same; the figure matches any row of the lane's own transcript | 0 of 58 |
| lanes since 2026-09-20 | 459 |
| warned | 76 |
| warned, but never reached 150k themselves | 51 |
| passed 150k themselves | 193 |
| passed 150k and were never warned | 168 |

A lane's first call is at 45–60k, not the 173–195k that was reported. On
2026-09-23 three separate lanes were all warned at exactly 155 218. That was the
clue: three different lanes do not share a context size, but three lanes of one
dispatcher share its transcript.

After the fix, replaying the hook over the 473 lanes of the last six days, with
the payload Claude Code really sends, gives each lane its own figure — 473 of
473, 308 of them in workflows. Live, with the fix loaded, a probe lane read a
large file in eight calls. It heard nothing from 34k to 134k, and was warned
once, at 150 168. That figure is a row of its own transcript and matches no row
of its dispatcher's.

### 235.3 Why nothing caught it for five days

**The test was built on a payload that never happens.** Its lane cases put the
lane's own figure in `transcript_path`, and sent `Stop`. Claude Code sends
neither. So the test passed because of its fixture, not because of the hook.
That is §77.6's and §173's shape again: the thing a check answers is a property
of the setup it runs in.

**An explanation was written down without being checked.** A memory note
explained the identical 155 218 as "the injected prompt alone", and nobody read
a single row of a lane's own transcript to test it. Two days later the
dispatcher had grown, the note said lanes had "grown to 173–195k", and its
suggested fix reached Mattias as something only he could do: a larger
`CLAUDE_CONTEXT_BUDGET`. Lanes do not read that variable; their dial is
`CLAUDE_LANE_BUDGET`. And raising either one would only have changed when a
lane heard about its dispatcher. CLAUDE.md's fifth principle is the rule that
was skipped.

**The one way out never reached a lane.** A lane fires SubagentStop, never
`Stop`, so the hook's lane `Stop` message never ran: 0 of the 76 warned lanes
saw it. The only sentence that let a brief overrule the warning was in that
message. The message lanes did see said to stop, with no exception, and the
two lanes whose briefs did not say "carry on" did stop: Q2 on its fifth call,
an F1 verifier on its first.

### 235.4 What changed, and what did not

- The hook finds the lane's own file,
  `<session>/subagents/[workflows/<run>/]agent-<id>.jsonl`, and searches every
  project directory for it. A session that enters a worktree after it starts
  keeps its own transcript under the directory it began in, but writes its
  lanes under the worktree's. 43 of the 473 lanes replayed were like that.
- If the lane's file cannot be found, the hook fails with a message. It does not
  fall back to the dispatcher's file. A probe lane found its own file on disk
  *during* its first call, so this cannot misfire on a lane's first call.
- There is now one lane message, and it carries the "if your brief told you to
  run past this point" sentence.
- The nine lane test cases now build the real layout. All nine fail against
  `0fc441f` and pass after.
- The `delegating` skill now says a lane expected to run long is told so in its
  brief, and that "carry on past it" should come out of briefs. It was right
  while the figure was wrong, and it now tells a lane to ignore a warning that
  is true.
- **No tag was cut.** `.claude/settings.json` still asks for v0.3.5, which has
  no lane arm at all. This machine runs the marketplace's working tree, so the
  pin does not decide what loads. Tagging v0.3.7 and moving the `ref` is
  Mattias's edit to make.
