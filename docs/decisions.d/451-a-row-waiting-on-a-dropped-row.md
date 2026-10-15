## 451. A row waiting on a dropped row: `after <id> -` frees it, and the drop asks for it — 2026-10-15

*Mattias, 2026-10-15, on a dispatcher's note that the trace Q2037 must close
with `done` and not `drop`, "because only a done row frees the rows that wait
on it": "Which seems like a limit in the tooling which we should fix." Decided
by session `queue-drop-frees-waiters` under `CLAUDE.md`'s "Decide it
yourself". Amends §263.2.*

**In short.** `just queue after <id> -` sets a row that waits back to `todo`.
A drop that leaves rows waiting on it is still refused, and the refusal now
names the `after` step for each of them, so the drop and the steps go in one
`queue apply`.

### 451.1 What the limit was

The note was half right. Only `done:` ends a wait (§263.2). But a drop does not
leave its waiters stuck without a word: check-queue refuses a row that waits on
a dropped row, so `drop` refuses itself. On a copy of `docs/queue.md` at
ce36a7c6, `just queue drop Q2037 §450` was refused four times, once each for
Q1905, Q1570, Q493 and Q861: `Q1905: waits on Q2037, which was dropped and will
never be done`.

The limit was what came next. Q1570 waits on Q1569 as well, and `after Q1570
Q1569` points it at that row alone. Q1905, Q493 and Q861 wait on Q2037 and
nothing else, and no verb set a waiting row back to `todo`: `after` needed at
least one id. So dropping Q2037 meant parking or dropping those three rows too,
or pointing them at some done row they never needed. Closing every trace
`done`, whatever became of it, was the way round that.

### 451.2 What changes

- `just queue after <id> -` sets a row whose state is `after:` back to `todo`.
  The `-` stands alone. Any other state is refused, and so is a pinned row, as
  it is for every `after` (§430).
- check-queue's refusal says what to do with each waiter. A row that waits on
  the dropped row alone reads `Q1905: waits on Q2037, which was dropped and will
  never be done; just queue after Q1905 - frees it, and just queue after Q1905
  <id> makes it wait on a row that replaces Q2037`. A row that waits on others
  too reads `…; just queue after Q1570 Q1569 keeps its other waits, with any row
  that replaces Q2037 added to them`.

So dropping Q2037 and freeing its members is one batch:

```
just queue apply drop Q2037 §450 + after Q1905 - + after Q493 - + after Q861 - + after Q1570 Q1569
```

### 451.3 Why the drop does not free them itself

A drop means different things to the rows that wait on it. When a trace is
abandoned, its members go back to being done one at a time, and freeing them is
right. When a row is dropped because another row files it again
(`was/<id>`), its waiters need that new row, and freeing them would send a lane
before the change it waits for has landed. When a row is dropped as not worth
doing, its waiters have to be read again. The tool cannot tell these apart, and
a wrong guess sends a lane. So the drop asks, and its refusal says how to
answer.

A trace that ran still closes `done`. §450.3 already says so: a trace that has
filed its fix rows has done its work, and the members it left alone go on
waiting for it to close.

### 451.4 The tests

`check-queue-selftest` holds both refusals word for word: Q5 waiting on the
dropped D1 alone, and on D1 and Q2. `check-queue-api-selftest` holds the verb:
a drop with two waiters refused, naming the step for each; the same drop going
through with those steps in one `apply`, leaving one waiter `todo` and the
other on its remaining row; and `-` refused beside an id, and on a row that
does not wait.
