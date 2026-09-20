## 100. A tool call is billed the context it is made at, decided 2026-09-14

A recipe that answers in three steps — the sessions, then the headings, then one
body — is cheap for a person and expensive for an agent, and `just question` was
built the first way and called the second way. This section is why the default
changed, and the rule it generalises to.

Do not update the figures below in place. §99.5 says why, and §77's CI tally went
stale twice inside a day to prove it. They are measurements of the week of
2026-09-07 and they stay that.

### 100.1 The measurement

Across the five longest transcripts in this repo, the mean assistant turn billed
144 000 to 182 000 input tokens against 850 to 1 300 output — a ratio between
110 and 198 to one — and 97 % of the input was cache reads. Over the whole week,
13 621 calls billed 1 805 M input for 6.0 M output.

The questions apparatus was $57 of a $1 163 bill. All of its recipe calls
together returned about 123 kB — roughly 31 000 tokens — for that $57, a ratio
near 470 to 1. `question-check` was the extreme: 128 calls, **median result 31
bytes**, because on a clean file it prints nothing. 92 % of question-work calls
did nothing else in the same call, at a median context of 150 000, which is 28 %
above the session-wide median — the work happens late, when a turn costs most.

### 100.2 The rule, and the number that makes it usable

**A call costs the context it is made at, not the bytes it returns.** Both sides
of that are cache reads at the same rate, so the trade is rate-independent and
holds for a Sonnet lane as well as an Opus session.

One avoidable round trip costs about what carrying 2 200 extra resident tokens
for 73 further turns costs — 166 000 divided by 2 200 at the main-session mean,
60 at the all-calls mean. The largest `open-questions.md` in the tree is 8 687
bytes, about 2 200 tokens. So printing a whole file beats **one** avoidable
follow-up in any session with fewer than about 73 turns left.

Two consequences, and they are not the same rule:

- **Batch independent probes into one call.** Three greps in one message cost one
  round trip; in three messages they cost three.
- **Write a file you already hold with one `Write`.** Batching does not extend to
  edits that depend on each other. Three `Edit`s in one message to this very
  document duplicated a block and cost two more round trips to find and fix than
  the single `Write` would have.

### 100.3 What it changed

`just question <hash>` prints every open question in full. `--labels` is the old
listing, `--all` includes answered entries, `--json` is for a program, and naming
labels prints those, answered or not. Violations are **named** under the output
rather than counted: a count is not actionable, and the second call that makes it
actionable pays for the whole conversation again to learn three lines.

`just question-tui` is the entry point for a person. Naming it that way is what
settles the default — before, every terseness in `just question` was justified by
a finite screen and cheap keystrokes, which describes the caller it does not have.

### 100.4 The better reason, which is not about money

A malformed entry has no parsable label. `compact` can say no more than `??×1`,
the label listing no more than `??` with a line number, and the TUI drops it
entirely (`questions-tui.py:493` keeps `if e["labels"]`). Printing bodies by
default is the only view in which a broken entry is readable and fixable.

That is a correctness gain and it would be worth making at zero token cost. A
question that goes missing while the line looks complete is the one failure this
apparatus cannot have — §59.2 is the time it happened.

