## 158. The lookup trigger is worth nothing until the lane is cheap, measured 2026-09-19

*§144.4 asked for a trigger a session can actually evaluate, because "past two
probes" is phrased against a quantity nobody has, and proposed the session's own
context size. Re-measured, three things are wrong with that. The target is a
fifth of the third of the bill the section quotes. Context points the **wrong
way** — the longer hunt starts at the lower context. And at what a lookup lane
measurably costs, a trigger that fires in the middle of a hunt is negative at
every threshold there is. What is built instead: a trigger asked at the first
probe rather than the third, a lane rebuilt around the half of its price nobody
had looked at, and a `finding` report so the next survey re-runs this argument
rather than quoting it.*

### 158.1 What was measured, and the mistake caught halfway

`just spend 2026-09-12 2026-09-21 summary`, then `finding`, the report this
section builds. Everything below is from the second, and every number in it
moves with the week — quote the shares, name the report, re-run it (§157.5).

**The first pass of the lane price was wrong in the way §144.1 warns about, one
level down.** A dispatched agent is asynchronous: the tool result for the call
is a launch receipt, not an answer, and every receipt is the same length. Taking
its size for the answer's said "the lane hands back 296 tokens, median and
maximum both" — a number whose own shape should have given it away, and it was
read past once. What the agent found arrives later, as a notification carrying
the **same tool-use id**. `load()` now stitches the two, so `results` means
everything that came back for a call, which is what the asking session carries
and pays for. The `carry` report gets more correct as a side effect: an `Agent`
row that counted only receipts now counts answers too.

### 158.2 Two thirds of "finding" is nobody's to delegate

`summary` says a third of the money went on responses that only looked something
up, and §144.4 reasons from that third. Split three ways it does not survive:

- **In main sessions** — 20.3% of the week, $0.097 a response.
- **Inside lanes** — 12.5%, at $0.051 a response. A lane is short and starts
  low, so its probes are already the cheap kind, and it has nowhere to send them
  anyway. Nothing here is addressable.
- **Ritual, inside the main-session part** — 4.0%. `just lanes`, `just status`,
  the question file: a session asking about its own state, which is §144.5's and
  cannot be delegated to anyone, because the answer is needed in your own hands.

What is left is **16.2%**. That is the whole population a lookup trigger can
touch, against the 33% the lane was briefed on.

### 158.3 The context threshold would select the wrong probes

§144.4's "obvious candidate" is the pair a session does know: its own context
size, and whether the question is about a file it is editing. The first is
measurable against what actually happened, so it was.

Counting **runs** — maximal stretches of back-to-back lookups in one session,
which is the unit a lane replaces:

- a run of **3 or more** starts at a median context of **119 000**
- a run of **1 or 2** starts at **145 000**

The longer hunt starts *lower*. A threshold that fires above some context
therefore selects lone probes, which is exactly the population a lane cannot
help with — and lone probes are where the money already is: runs of one and two
are $117 of the $197 left to delegate.

The reason is not mysterious. Context measures how long the session has been
running, and a session that has been running a long time knows its way around;
it is the fresh one, still finding its feet, that hunts. Context predicts what a
probe **costs**, not whether it is part of a hunt, and §144.4 asked it to do the
second job because it is visibly good at the first.

### 158.4 At the lane's measured price, no mid-hunt trigger pays

The price was never measured before this. It can be, exactly: the launch receipt
names the lane's agent id and the lane's transcript is the file named for that
id, so each dispatch pairs with its own lane rather than being guessed at from
the brief. Over the nineteen dispatched in the window:

| | |
|---|---|
| the dispatching response | $0.133 — billed at the asking session's context, like any probe |
| the lane itself | $0.251, over eleven responses |
| carrying what came back | $0.015 median; over the nine answers that could be matched back, $0.04 |
| **all in** | **$0.399** |
| the median hunt of 3+ it would replace | **$0.375** |

**The lane costs more than the hunt.** So delegating the *tail* of a hunt — what
"past N probes" asks for — is negative at every N: −6.6% of the week at past
one, −2.8% at past two, −0.3% at past six, where the only reason the number
shrinks is that fewer hunts are left to spoil.

Delegating the **whole** hunt at its first probe is the only shape that pays,
and only just: +0.3% of the week for hunts of three or more, +1.0% for four or
more. That is the ceiling on §144.4 as briefed, and it is an order of magnitude
under the third of the bill the brief opens with.

**§144.4's arithmetic is right per call and wrong per question.** A lane's
response is five times cheaper than a probe in a fat session; a lane takes
eleven responses where the hunt it replaced took three or four, because it
starts knowing nothing and has to find its own way to the file. Five times
cheaper and three times as many is not a tenth of the cost. The section's
"starts near 35 000" is also high: measured, a lookup lane's first response is
**19 000**.

### 158.5 A lane is priced by what it reads, not by how many calls it makes

The half of the price nobody had looked at. Of what a lookup lane costs, **56%
is cache write** and 39% is read — the opposite shape to the session that
dispatched it, where two thirds is read. A lane is short, so every tool result
it pulls in is fresh tokens written at the write rate rather than re-read at a
tenth of it. Measured, one climbs from 19 000 to 53 000 to answer a question
whose answer is 1 300 tokens: it drags in about 34 000 tokens of material and
quotes a twentieth of it.

That is the lever, and it is a bigger one than the trigger. Holding everything
else fixed, the same hunts of three or more at **half** the lane's price are
worth +2.4% of the week instead of +0.3%, and at a quarter, +3.5%.

So `.claude/agents/lookup.md` is rewritten around it. Never read a whole file;
reach for the one-call readers this repo already has — `just src`, `just
recipe`, `just decision`, `git show <sha>:<path>` — and `grep -n -C3` then
`Read` with an offset when there is no reader. Six tool calls, then stop and say
what you learned and where you would look next. The lane is also told what it
costs and what it replaced, because a lane that does not know it nearly failed
to pay for itself has no reason to hurry.

### 158.6 The trigger: can you name the file?

§144.4's diagnosis is right — "past two probes" cannot be followed, because
there is no probe counter and by the time a hunt is obviously a hunt it is
over. Its prescription is what fails. The replacement has to be answerable at
the **first** probe, since that is the only place the money is.

> **If you cannot name the file, it is a hunt — hand it to a lookup lane before
> the first grep.** If you can name the file, the section or the recipe, read it
> yourself: one call inline beats a lane.

A session always knows which of those it is about to do. It is not a count, not
a threshold, and not a guess about the future; it is a question about the call
being made right now. `CLAUDE.md` carries it, with what not to delegate: your
own state, and anything at all from inside a lane.

### 158.7 What holds it

`just spend <window> finding`, and `check-spend-selftest` over the three pure
functions the report reasons from — `is_lookup`, `finding_runs`,
`delegation_net`. Each was run against the obvious wrong version of itself
before the cases were called done: ritual not excluded from lookups, runs that
ignore which session made them, a lane charged once for a whole sweep instead of
once per hunt. Six cases fail on those three mistakes, which is what makes them
cases rather than decoration.

The report exists because of §144.10's rule — a lane starts by re-running the
report its section names — and §144.4 named `summary`, which can see none of
this. An argument this long that lives only in a document is a number going
stale in a file. This one is a command.

### 158.8 What it owes

**The model dial is deliberately not turned.** Haiku bills at exactly half
sonnet's rate on every token class, so the same lane on it is the half-price row
of §158.4 almost by construction. It is not done here on purpose: the read-narrowing
above is a change to what the lane *does*, the model is a change to what it
*is*, and doing both in one week means a wrong answer has two candidate causes
and no way to tell them apart. Turn it once the next `finding` run says what the
rewrite alone was worth.

**The mechanical trigger is not built either.** A `PreToolUse` hook counting
consecutive probes is the honest answer to "a rule phrased against a quantity
nobody has" — give the quantity to the thing that has it, exactly as
`context-budget.sh` does — and prose alone has failed in this repo before, which
is why the patching rule is a hook. But at today's price such a hook would be
nudging sessions toward a net loss at every threshold it could fire on. Build it
when the lane measures at or under $0.20 all in, where firing early is worth
something.

**What settles both:** re-run `just spend <window> finding` in a fortnight. The
lane price is the number to watch — not the dispatch count, which is what
§144.4 watched and which moves for reasons that have nothing to do with whether
delegating was right.

