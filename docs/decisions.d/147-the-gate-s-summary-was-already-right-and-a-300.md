## 147. The gate's summary was already right, and a 300 KB log threw it away, fixed 2026-09-19

*§144.5's first part, taken. It said gate babysitting was a `CLAUDE.md`
paragraph and nothing more. It is not: the paragraph had nothing safe to ask
for until the `justfile` gave it a mode whose output fits in a tool result. The
measurement also moved three of §144.5's own claims, and those are recorded here
rather than quietly fixed in place.*

### 147.1 What §144.5 got wrong, measured

`just spend 2026-09-12 2026-09-20 ritual`, and a walk of the transcripts under
`$CLAUDE_CONFIG_DIR` for the same window. Three things.

**Gate babysitting is the smallest of the six rituals, not the first.** 147
responses and 1.3% of the week, behind the question file at 6.7%, lane admin at
5.8% and renumbering at 2.3%. §144.5 introduces the six as "largest first"; the
order it lists them in is `spend.py`'s `RITUALS` array, whose own comment says
"the order is the order of interest". Those are not the same order and the
section reads as though they were.

**Sessions do not poll in a loop.** Of 92 waiting episodes, **77 are a single
call**; the longest is four. The `until ! pgrep -f 'just gate'` that §144.5
quotes appears once in the week. Removing a loop that is not there would have
bought nothing.

**The cost is the read-back, which §144.5 does not name.** Of **272 gate
launches**, **262 redirected stdout to a file**, and **122 — 45% — were followed
within three tool calls by another call that went back for the summary**. That
is the whole of the addressable money here, and it is roughly 180 responses a
week once the waiting calls are added, against 147 the `ritual` report can see:
most of it is filed under other shapes because the command that pays it does not
mention the gate.

### 147.2 The block was already right; 300 KB defeated it

`gate` has printed a summary block since the week before — pass or fail, the
counts, the first failing step — precisely so that nobody would read the log
back. Its own comment measured the defect it was fixing: "145 of 415 gate runs
were followed by a second call that grepped the redirected log".

It did not work, and the reason is not that sessions ignored it. **A whole gate
run is 300 KB — about 75 000 tokens.** No session can let that into a tool
result, so every one of them redirects, and the block goes into the file with
everything else. The fix was correct and was posted through a letterbox nobody
could reach.

So the shape of the answer is forced: the block cannot be made louder, the log
has to get quieter.

### 147.3 What was built

`just gate quiet`. `gate` takes a `mode` parameter defaulting to `loud`, and the
loop sends each step's output to `target/last-gate.log` alone, printing `· <step>`
as it goes. On a failure it prints that step's own last sixty lines, then the
existing summary block. About fifty lines in all.

**The failing lines are the step's own, not the log's tail.** A plain `tail -40`
was the first version and it was wrong in the obvious fixture: the step before
the failure was `list`, so forty lines of recipe names crowded out the one line
that said what broke. The loop records `wc -c` of the log before each step, so
what comes out is everything the failing step wrote, capped.

**`loud` is untouched**, because Mattias types `just gate` in a terminal and CI
wants the whole stream in its own log. **A mode that is neither word exits 2**
rather than defaulting: a typed `quite` that silently ran loud would hand back
the 300 KB this exists to avoid.

`CLAUDE.md`'s Tooling section carries the calling convention — one plain Bash
call, `timeout` 600000, no redirect, no `| tail`, no wait loop.

### 147.4 The nine minutes are a cold-cache figure

`gate`'s `[doc]` said "~9 minutes", and that number is why sessions backgrounded
it and polled. Against the transcripts it does not hold for a warm tree: over
**171 foreground gate calls** in the week the median was **90 seconds** and the
slowest **254**, with **no timeouts**; over the 56 that passed, a median of 98
and a maximum of 244. The Bash tool's own maximum, 600000 ms, covers that with
two and a half times to spare.

This is what makes the one-call shape right and the background one a fallback.
§144.5 reached for `Monitor` and `run_in_background`; both cost two responses —
the launch and the read after the wake-up — where a foreground call costs one.
The fallback is kept for a genuinely cold tree, a fresh worktree or one just
through `clean-targets`, where the nine minutes are real.

**Foreground `sleep` is not blocked in this harness**, whatever the Bash tool's
own description says — it was tested. So the sentence in `CLAUDE.md` is the only
thing standing between a session and another `sleep 240; tail`.

### 147.5 What the test had to be able to do

`check-gate-summary-selftest` drives the loop with `--set gate_steps`, so it
already had the machinery; `quiet` adds four fixtures, and each of them is about
what the mode leaves *out*, which is the only thing that distinguishes it.

A quiet pass must keep a step's output out of the result **and** still have it
in the log — one assertion each way, because a mode that dropped the output
entirely would pass the first alone. A quiet failure must carry the failing
lines, must not spend them on the step before, and must still leave the block in
the last six lines, where `tail -6` looks for it. A misspelt mode must refuse.

Each was checked by breaking the recipe and watching the fixture catch it, not
by reading the assertion and agreeing with it.

### 147.6 What this does not close

The other two parts of §144.5 — the renumber recipe and applying
`question-check`'s own findings — are untouched, and the question file is five
times the money this was.

Nothing makes a session use `quiet`. The rule is prose in `CLAUDE.md`, which is
exactly what §144.4 says did not work for the lookup lane: a rule phrased
against something the session cannot check is a rule it cannot follow. This one
is at least checkable — it names one command — but the honest test is the
`ritual` report a week from now, and whether the read-back share falls.

Making `loud` quiet automatically when stdout is not a terminal was considered
and rejected: CI's stdout is not a terminal either, and a red run on the runner
needs its whole log in the Actions view.

