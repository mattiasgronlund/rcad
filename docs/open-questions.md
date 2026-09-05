# Open questions

Decisions waiting on Mattias, and the ones already made.

This file exists because a question asked only in the terminal is a question
that scrolls away. On 2026-09-05 two of them did, and the session had to be
asked to repeat them — which costs a turn at full context to recover something
that was never written down.

**The status line reads this file.** `.claude/statusline.sh` shows the count and
the first three, so the index is visible without `!cat`, which would add the
file to the conversation and cost a re-read on every later turn.

## How to write a row

One line per question, newest last:

```
- [ ] **Q11** — Short question in one line? → *recommend (a), because …*
```

- `- [ ]` is open, `- [x]` is answered. The status line counts the open ones.
- The `→` carries the recommendation and the narrowing. Everything before it is
  what shows in the status line, so put the question there and the argument
  after.
- Answer in the terminal as you always have — `Q11 (a)`, or a sentence. Whoever
  is dispatching ticks the box and appends `→ answered: …` the same turn.
- A question that turned out not to need an answer gets ticked with
  `→ withdrawn: …` and why. Do not delete rows; the record of what was asked is
  worth more than a tidy file.

Long context — the full argument, the options, what each costs — belongs in
`docs/decisions.md` under the section it will become. This file is the index.

## Waiting

<!-- none right now -->

## Answered

- [x] **Q0** — Does this file earn its place, or is it one more thing to keep
  current? → answered: keep it. The failure it prevents already happened once,
  and the status line makes reading it free.
