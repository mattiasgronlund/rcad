## 309. One call for several queue verbs, all or nothing — 2026-09-27

*Mattias asked on 09-27 whether `just queue` should take several verbs in one
call, after a dispatcher took Q241, Q270 and Q283 in three calls. This section
measures what the separate calls cost and what they risk, and sets the shape of
the verb. Nothing is built here; the row that builds it cites this section, and
sits first in tier 1 at his word.*

### 309.1 What the transcripts show

Every `*.jsonl` under `~/.claude/projects/*rcad*/`, subagents included: 1 704
files. A queue write is a Bash call whose command matches `just queue
(take|add|done|drop|park|after|rank|pick)`. A run is a stretch of such calls
with no other tool call between them. Each call after the first in a run is
one a batch verb would have saved, and its price is the context of the response
that made it: input, cache read and cache creation together. The window is the
first verb call, 2026-09-26T09:29Z, to 2026-09-27T19:52Z.

- 329 queue writes in 192 runs: 121 alone, and 71 of two or more (37 of two, 18
  of three, 8 of four, 3 of five, 2 of six, 3 of seven).
- A batch verb would have saved 137 calls, made at 13.3M tokens of context in
  all, 97K on average. 90 of them were subagents' and 47 top-level sessions'.
  The commonest mix in a run was `add` with `done`, 20 of the 71: a merge
  closing its row and adding the rows it leaves.
- Counting only calls that hold nothing but queue verbs, `cd`, `git status` and
  `git diff`, so that `git merge ... && just queue done` is left out: 66 calls,
  5.5M tokens.
- All the context billed in the window: 1 223M tokens over 11 375 responses.

So a batch verb saves between 0.45% and 1.1% of what the window billed. That is
real and small, and §157.2's warning about the implied size of the prize applies:
it is not the main reason to build this.

### 309.2 Why chaining with `&&` is not the answer

47 calls in the window already chain two or more verbs with `&&`, so one call is
possible today. What a chain cannot do:

- **It is not all or nothing.** Each verb takes the lock, checks, appends to the
  claims file and writes `docs/queue.md` on its own. 48 queue writes were
  refused in the window. 15 of them came after another write call in the same
  run, and 2 were chains in which a verb went through before a later one was
  refused: the first part written and held, the rest not.
- **Each verb checks and renders again.** `check-queue` runs on the file and
  `queue-page` redraws it once per verb, so N times for N verbs.
- **The lock is let go between verbs**, so another dispatcher's verb can land in
  the middle of a fill.

### 309.3 The shape

```
just queue apply take Q241 lane-a + take Q270 lane-b + add 2 sonnet - 'what §N' + after @1 Q270
```

1. **Not JSON.** 221 of the 294 rows in `docs/queue.md` at 8ca95786 contain an
   apostrophe, and a JSON argument quoted in `'...'` ends at the first one. Each
   step keeps the arguments its verb takes alone, so a step is written exactly
   as the single call would be, and a standalone `+` separates the steps. The
   lane shows `just` passes the `+` through unchanged, or picks another token and
   says why. A split that leaves a step whose first word is not a verb refuses
   the batch.
2. **All or nothing.** Under one hold of the claims lock, apply every step to the
   file in memory, in order, each step seeing the steps before it: a `take`
   sees the holds earlier steps took or released, and an `add` hands out the id
   after the one the previous `add` in the batch handed out. Run `check-queue`
   once on the result. Only then append every claims event and write the file,
   and render `queue-page` once. A refused step is named by its number and its
   text, and nothing is written to either file.
3. **A step can name an id the batch hands out.** 29 `after` calls in the
   window; 24 named a row that an `add` had handed out earlier in the same
   transcript, 16 where the new row waits and 8 where an older row waits on the
   new one. 22 of the 71 runs mix `add` with `after`. `@1`, `@2` and so on stand
   for the ids of the batch's first, second and later `add`s, wherever an id
   goes. Without that, these runs stay two calls, since the id is not known
   until `add` prints it.
4. **It prints what each verb prints alone**, in order, so `@n`'s id can be read
   off its `add`'s line.
5. **One path, not two.** `just queue <verb> ...` becomes a batch of one step.
   Every write verb there is when the lane runs goes through it, including the
   ones Q249, Q252, Q255 and Q281 add: whichever of those lands second changes
   its verbs to fit the other's shape.

### 309.4 The tests it needs

`check-queue-api-selftest` holds that a batch of three `take`s whose third is
refused leaves `docs/queue.md` and the claims file byte-identical, and that the
check fails when `apply` writes after each step instead of at the end. It holds
that `add ... + after @1 <id>` gives the new row `after:<id>`, and that two `add`s
in one batch get consecutive ids.

`dispatch-lanes`, where a fill takes its rows, and `merger.md`, where a merge
closes its row and adds the rows it leaves, say to do each in one `apply`.
