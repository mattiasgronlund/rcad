## 152. The question file's cost was a call a hook had already answered, fixed 2026-09-19

*§144.5's second part, taken, and its premise did not survive the measurement.
The section says `just question-check` names every violation precisely and
nothing applies them, so a session reads the list and hand-edits at full
context. Of 174 `question-check` calls in the week of 2026-09-12, **three**
printed a violation. There was nothing to apply. What the money bought was the
asking: a hook had run the same checker on every write since 2026-09-14, and
said nothing when it found nothing, which looks exactly like a hook that is not
installed.*

### 152.1 What the bucket is actually made of

`just spend 2026-09-12 2026-09-21 ritual` and a walk of the transcripts for the
same window, classified by what each call addressed rather than by what its
text mentioned:

```
ran question-check      173 responses   $17.58
Write the question file 114             $17.74
Edit the question file  105             $13.19
ran just question        46             $ 4.58
Read the question file    7             $ 0.59
TOTAL                   445             $53.68
```

**Bytes are not the cost, and the retyping theory is wrong.** Of 114 `Write`s,
64 rewrote a file that already existed, and the lines identical to what was
already on disk came to 33 812 tokens for the week — **$0.85** priced as Opus
output. A `Write` of the whole file and an `Edit` of one line cost the same
thing, which is one response at a median context of 146 000 tokens. Any fix
aimed at the size of what is written is aimed at 1.6% of this bucket.

**Round trips are the cost.** 148 bursts of question-file activity, 393 calls,
2.66 calls a burst — and the commonest burst is exactly two, 80 of the 148: a
write, then the check.

### 152.2 Three found something, and the hook found the rest

`check-question-file.sh` has run the checker as a PostToolUse hook on every
`Write` and `Edit` to `open-questions.md` since 2026-09-14. It blocked 13 times
in the window. In the same window the hand-run recipe found 3 — and 148 of its
174 calls came *after* the hook landed.

That is not two checks disagreeing. It is one check running first and the other
arriving at a file it has already cleaned. The recipe found three because the
hook had found the rest.

### 152.3 Why the round trip survived a hook built to remove it

Two reasons, and neither is the session being careless.

**A PostToolUse hook that exits 0 in silence is indistinguishable from a hook
that is not installed.** The session has no evidence the check happened. Asking
again is the only way to find out, and asking costs a tenth of a dollar.

**The skill told it to.** `asking-questions` said *"Run `just question-check
<hash>` on your own file, in the turn you write a question"* and *"run the
checker in the same call"* — written before the hook existed and never revised.
Of 393 question-file responses that week, **not one** put the write and the
check in the same response. The instruction was both obsolete and unfollowed.

### 152.4 What landed

In the `guardrails` plugin, version `0.3.0`, which `.claude/settings.json` asks
for as the marketplace tag `v0.3.1` — the tag moved once because this section
did, and a tag that has been pushed is not moved a second time:

- **A clean file says so**, in one line, through
  `hookSpecificOutput.additionalContext` — plain stdout on a PostToolUse hook
  reaches the debug log and not the model, which would be the old silence under
  another name. It names the open labels, because the file has just been parsed
  and those are the one thing a session would otherwise read it back for. About
  38 tokens on a call already being made.
- **`no-hand-run-question-check.sh` refuses the recipe.** It is the fourth hook.
- `lib/shell-code.sh` now holds the heredoc split both `PreToolUse` hooks need,
  rather than a second copy of the awk.

And in `questions`, the `asking-questions` skill says the opposite of what it
said, and names the escape hatch.

**Refusing is safe because the two hooks ship together.** A machine without the
refusal has no automatic check either — they are one plugin — so the refusal
cannot fire anywhere the call is still needed. That is what makes this a
refusal rather than a warning, which §50.8's history says is the difference
between a rule that holds and a rule that is written down.

**It refuses a call, not a mention.** A segment whose head resolves to `just`
with `question-check` among its words is the call; `grep -n question-check
justfile`, a commit message quoting the rule, and `just question` are not.
Reaching past the recipe to `questions.py check` is left open on purpose: it is
the escape hatch the refusal message names, and having to reach is the signal
that the reason was real.

11 new cases in `check-question-hook.sh`, 16 in all, including the five that say
what must still pass.

### 152.5 The `ritual` report was measuring handoffs

The figure §144.5 reasoned from was $74.80 over 570 responses. The true figure
is $53.68 over 445. The difference is a bug in `spend.py`'s own classifier:
`RITUALS` searched the whole tool input as one blob, so a **handoff document
that listed the open questions** — which the handoff skill tells every session
to write — counted as question-file work, and since first match wins it was
taken off `handoff writing` at the same time.

Each ritual now says which part of a call it reads: `address` is the `file_path`
of a file tool and the `command` of a shell call, `content` is the bytes an
`Edit` or `Write` carries. Over the same window that moves five of the six rows:

```
                    was                  now
question-file       570  $74.80    461  $56.05
handoff writing     179  $24.90    223  $36.12
section renumbering 215  $25.46    292  $35.50
merge-conflict      118  $11.45     53  $ 5.98
gate babysitting    162  $15.57    133  $12.04
TOTAL             1,754 $217.93  1,599 $200.93   19.4% -> 17.6%
```

Renumbering rose because its content pattern had never matched anything.
`^## 1[0-9][0-9]\. ` was being searched against a JSON blob, where a newline is
the two characters `\n` and `^` under `re.M` can only match at the blob's own
start — which is `[`. Half that rule was dead for as long as it existed, and a
green report never said so.

`just check-spend-selftest` holds `ritual_of` to 17 cases and is in `gate_steps`;
three of them fail against the classifier as it was, which is the half a green
run cannot show. It needs no transcripts, so it runs on a runner like any other
check — `spend` itself still cannot be a `check-`, for the reason its own
comment gives.

### 152.6 What is left, and why no recipe is coming

After the refusal, the file costs about $36 a week: writing questions, editing
them, reading them back. Those are calls that do the work.

**§144.5's `question-fix` recipe should not be built.** It would have applied
three violations in a week, and the hook applies them earlier by refusing the
write that made them.

**Nor should a recipe that marks an answer.** 61 of 105 `Edit`s are the same
mechanical flip — `- [ ]` to `- [x]` with an `→ answered:` line — which reads
like the obvious thing to automate, and automating it saves nothing: a `Bash`
call and an `Edit` call are both one response at full context. The only case
where a tool could help is the burst that answers several at once, and there
were 10 of those in the week; the skill already covers them by saying to write
the whole file once rather than edit entry by entry. The `justfile`'s "there is
deliberately no recipe that *writes* a question" stands unchanged.

§144.5's third part, the renumber recipe, is still open.

### 152.7 This section lost the same race §144.5 lost, while fixing §144.5

It was written as §151 after `git log --all -S'## 151. '` came back empty —
the check the memory note prescribes, run correctly, answering correctly. About
an hour later `main` carried §151, from the lane that took §149's oracle, and
this became §152.

§144.5 made exactly this mistake while being written and drew the right
conclusion from it: no rule addressed to the person picking can work, because
the read is against a value another session is about to write. That part of it
survives the measurement intact, and it is the part still open.

The honest test of all this is the `ritual` report a week from now. The number
to watch is `question-check` calls, which should be zero.

