## 144. What a week of sessions is spent on, measured, surveyed 2026-09-19

*Mattias asked where the work in this repo could be made cheaper. The answer is
not what the rules in `CLAUDE.md` assume. Almost the whole bill — measured over
2026-09-12 to 2026-09-20 — is re-reading the conversation, so cost is set by how
long a session has been running when it makes a call, not by what the call does.
A fifth of it is recurring ritual that no kernel change needs. This section is
the register of what was found; each subsection below that says **lane** is one
session's work, and none of them depends on any other.*

### 144.1 How it was measured, and why no total is written here

`just spend` reads the Claude Code transcripts under `$CLAUDE_CONFIG_DIR` for
this repo and every worktree of it. `tooling/session-spend/spend.py` is the whole
tool. Six reports: `summary`, `classes`, `ritual`, `sessions`, `carry`,
`ceiling`, and a sixth argument narrows to one lane.

**There is one way to get this wrong and it is silent.** A transcript writes one
record per content block, and every record of the same API response repeats that
response's `usage`. A response that thought and then called a tool is two records
carrying the same numbers. Summing records double-counts by about 2x, and nothing
about the shape of the answer looks wrong afterwards — the first pass of this
survey did exactly that and reported $2,142 for a week that cost $990. The unit
of billing is `(requestId, message.id)`, and `load()` groups on that pair.

The instrument was checked against a figure this repo had already written down
by other means: `CLAUDE.md` quotes "$1 163" for the week of 2026-09-07, and the
tool answers $1,146 for that window — 1.5% apart. It is the same instrument.

**No total is written into this section, and none should be.** §77's rule about
the gate's numbers applies here with more force: a count goes stale on the next
session, and this one goes stale while you read it. Every finding below names the
report that measures it, and a lane that picks one up **starts by running that
report**, because the number it was written against is two weeks old by then.
Prices are in one table at the top of `spend.py` and dated; re-check them before
quoting dollars, and prefer the shares, which move far more slowly.

### 144.2 Nearly the whole bill is re-reading the conversation

`just spend <window> summary`. Of the money, about two thirds is cache read, a
fifth is cache write, and an eighth is output. **Output — the tokens that are the
actual work — is the smallest part by a wide margin.**

That one fact reframes everything else in this section. A tool call does not cost
what its answer costs; it costs what the whole conversation costs, again. So the
levers that matter are the ones that shorten the conversation or move work into a
smaller one, and the levers that do not matter are the ones that make an
individual answer smaller.

It also explains why a lane is cheap and a long main session is not: `carry`
shows that a result's true price is its size times the number of responses that
come after it in the same session. A 15k-token file read in the first minute of a
session that runs for 200 more responses is not a 15k-token expense.

### 144.3 A context ceiling is the largest single lever, and nothing enforces one — lane

`just spend <window> ceiling` and `sessions`. Most of the bill is spent by the
sessions that ran longest: the ones that passed 200k of context are a small
minority of sessions and about half the money. Billing cache read at a ceiling
instead of what was really spent, 150k saves about a tenth of the bill, 125k
about a seventh, 100k about a fifth. Those are upper bounds — the handed-off work
still has to happen and the session inheriting it re-reads some of what it got —
but the gap between 250k and 150k is real money and it is the largest number in
this survey.

The budget hook already asks for a handoff, and roughly half of sessions end in
one, so the mechanism works. **It fires too late.** The median session is billed
at about 130k and the expensive ones run to 250k and beyond.

What the lane does: find the threshold in the `guardrails` plugin's
`context-budget.sh`, work out what it is actually measuring, and lower it until
the `sessions` report stops showing a long tail past 200k. Then re-run `ceiling`
on the following week and see whether the shape moved. **The thing to be careful
about** is that a handoff is not free — §138 already says a session ends finished
or written down — so a ceiling set too low buys more handoffs than it saves. The
measurement that settles it is cost per week at a steady amount of work, not cost
per session, which a lower ceiling improves trivially and meaninglessly.

Write set: the `guardrails` plugin, which is in the marketplace repo and not this
one. Nothing here changes.

**Taken, and the brief's premise was wrong — §160.** The hook was already at 200k
when this was written and catches the crossing within 3%, so there was no
lateness to fix and the dial did not move. 22 of the 23 points billed above the
budget are a warning that could not repeat itself and could not fire inside a
turn; both are fixed in `guardrails` v0.3.5, and `just spend <window> warnings`
is the report that measures it — this section's "re-run `ceiling`" was the wrong
instrument, because `ceiling` prices a cap nothing enforces. Something did change
here after all: that report, and its cases.

### 144.4 Finding is a third of the bill, and the lookup lane is barely used — lane

`just spend <window> summary`, the finding block. About a third of the money goes
on responses that only looked something up and changed nothing. Week over week
that share went **up**, not down, while `CLAUDE.md`'s "past two probes, hand the
question to a lookup lane" was already written and in force.

The lane exists, is read-only, runs on a cheaper model, and was used eighteen
times in a week. Probes made deep inside a fat session are the addressable part:
the same question asked by a fresh lane starts near 35k instead of 200k and bills
at a fifth the rate, so it costs roughly a tenth.

**Why the rule did not take is the interesting half.** "Past two probes" is not
something a session can count while it is happening — there is no probe counter,
and by the time it is obvious that a hunt is under way the hunt is over. A rule
phrased against a quantity nobody has cannot be followed.

What the lane does: replace the trigger with one a session can actually evaluate.
The obvious candidate is the pair the session does know — its own context size,
which the status line already shows, and whether the question is about a file it
is currently editing. Write it into `CLAUDE.md` as a threshold, then measure the
finding share over the following week.

Write set: `CLAUDE.md`, possibly `.claude/agents/lookup.md`.

**Taken up by §158, and it came out against this brief.** The target here is
five times the addressable one, the context threshold proposed above selects the
wrong probes, and the lane costs more than the hunt it replaces — so the trigger
was not the thing to fix first. Read §158 before re-running any of this.

### 144.5 A fifth of the bill is ritual that no kernel change needs — lane, or three

`just spend <window> ritual`. Six recurring shapes, none of which is kernel work,
and together about a fifth of the money — all of it done by the strongest model
at full context. Largest first: keeping the question file to its grammar; lane
and worktree admin; renumbering sections after a collision; babysitting a running
gate; resolving conflicts in this file; writing handoffs.

Each is either scriptable or belongs to a cheaper model, and a script is better
than a cheaper model because a script costs nothing at all.

**Gate babysitting is the purest of them** and the easiest to remove. Sessions
poll with `until ! pgrep -f 'just gate'; do sleep 10; done`, and every one of
those polls re-reads the whole conversation. The harness has a background mode
and a `Monitor` that waits without billing a turn. This is a `CLAUDE.md`
paragraph and nothing more.

**Renumbering is second and is a collision, not a mistake.** Sections are
appended at the tail by whoever finishes first, from as many as four concurrent
sessions; `check-decisions` catches a duplicate at the gate, which is late. The
fix is to stop two lanes reaching for the same number, not to catch them after.

**This section lost that race while it was being written, which settles what
the advice is worth.** It was drafted as §142 after checking `git log --all
-S'## 142. '` and finding nothing on any branch — the check the memory note
prescribes, run correctly, answering correctly. Forty minutes later `main` had
§142 and §143, both from the `fuzz-triage-two-nights` lanes, and this had to be
renumbered to §144. The check is not wrong; it is a read against a value another
session is about to write, and nothing holds the answer for the time it takes to
write a section. **So no rule addressed to the person picking can work**, and
the lane should not write one. What is needed is a claim that outlives the read
— the number taken at the moment of asking rather than at the moment of
committing. That is a `justfile` recipe, and it is the whole of the work here.

**The question file is the largest and the least obvious.** `just question-check`
already names every violation precisely. Nothing applies them, so a session reads
the list and hand-edits, at full context, repeatedly.

What the lane does: take one of the three. They are independent and a session
that takes all three will overrun, which is §144.8's whole finding.

Write set: `CLAUDE.md` for the gate paragraph; the `justfile` for a renumber or
question-fix recipe; the `questions` plugin if the fix belongs to the parser.

**The gate part is taken, and §147 corrects three things above.** It was not a
`CLAUDE.md` paragraph and nothing more — the paragraph had nothing safe to ask
for until `just gate quiet` existed, because a whole run is 300 KB. Gate
babysitting is also the smallest of the six here and not the first, the six are
not listed largest first, and sessions were not polling in a loop.

**The question-file part is taken, and §152 says the paragraph above it is
wrong.** `question-check` found a violation 3 times in 174 runs, so there was no
list to apply and no recipe to write: a hook had been running the same checker
on every write since 2026-09-14 and saying nothing when it found nothing, and
what the money bought was sessions asking a question already answered. §152 also
corrects the figure this section reasons from — $74.80 was $53.68, and the
difference was handoff documents that listed the open questions. The renumber
recipe is the one part still open — and §152 was written as §151 and lost the
race to a lane, which is this section's own point made twice.

**The renumber part is taken too, in §159**: `just claim-section`. All three
parts of this section are now done.

### 144.6 Locating a line and then reading it, on the two files with a one-call reader — lane

The most common two-call shape in the week is: grep a file for a line number,
then `Read` that file at that offset. Both calls are billed against the whole
conversation, so one lookup costs two full re-reads.

**The two files it happens to most are the two that already have a better way
in.** `docs/decisions.md` has `just decision N` and `just decisions-find`, and
`CLAUDE.md` says in as many words never to grep it and never to guess a line
range — and it is still, by a wide margin, the file this shape is used on most.
The `justfile` is second, and it has no such reader at all: there is `just src`
for a Rust item and nothing for a recipe.

That asymmetry is the finding. Where a one-call reader exists it is under-used;
where it does not exist the behaviour is forced. The second half is buildable
today: **`just recipe <name>`**, printing one recipe with the comment above it,
exactly as `just src` prints one Rust item with its doc comment.

What the lane does: build `just recipe`, with a selftest beside the other
`check-*-selftest` recipes. Then find out why `just decision` loses to `grep` —
the likeliest answer is that a session searching for a phrase does not know which
section it is in, and `decisions-find` answers in section numbers but is not
reached for. That half may be a `CLAUDE.md` wording problem rather than a tool
problem, and it should be settled by reading transcripts, not by guessing.

Write set: `justfile`, and `CLAUDE.md` if the wording is the half that is wrong.

### 144.7 The batching rule asks for something that mostly cannot be done

`CLAUDE.md` says independent read-only probes go in one call, and prices the
split. The share of responses calling more than one tool moved by about one point
across the two weeks, which is nothing.

**The reason is that the rule is aimed at the wrong population.** A sample of
thirty consecutive probe-after-probe pairs from the week is mostly *dependent*
pairs: grep for a line, read at that offset; count the lines, read the tail; find
the file, then open it. The second call needs the first call's answer and cannot
be issued beside it. The paragraph asks sessions to batch calls that are not
batchable, which is why two weeks of it produced no movement.

Genuinely independent pairs do exist and batching them is still right. But the
honest size of that lever is small, and the pairs the paragraph's own arithmetic
was counting are largely §144.6's shape — which is fixed by a better single tool,
not by asking for a batch.

This is a correction to `CLAUDE.md`, not a lane: say what batching can deliver,
and stop it competing for attention with the levers that pay.

### 144.8 A lane's cost is set by re-entry, not by the work — evidence

`just spend <window> classes fuzz-triage-two-nights`, against the same window
without the filter.

That lane ran twelve sessions over two and a half days. Every one began from
`/clear` and a handoff document, climbed, and closed by writing the next handoff.
The last of them entered at 57k and reached 289k in forty-seven minutes. Cost per
hour rose across the chain, because each session re-enters low and re-climbs.

**Split by what the money was spent on, the lane is inverted against the week.**
Across the repo, kernel work and its tests are the larger share and the
paperwork around them the smaller. Inside this lane it is the other way round:
decisions, questions and the `justfile` took appreciably more than the kernel,
its tests and running them combined — and writes to `crates/*/src/*.rs` across
the whole lane number two.

This is the shape behind "fixing a gap always overruns". The gap is not what
overruns. The gap is small — often a few lines, sometimes none. What overruns is
the twelve re-entries and the ritual of §144.5, and both are counted against the
gap because they happened in the lane that was fixing it.

Nothing to build here. It is the evidence §144.3 and §144.5 are motivated by, and
it is recorded so the next survey has something to compare against.

### 144.9 Provenance is free when certified and not when refused, which is the path fuzzing takes — lane

`ProvenanceLog` says in its own doc comment that an empty log does not allocate,
"so a certified operation pays nothing for carrying one". That is true and it is
the right design for the kernel's ordinary use.

**It optimises the path a fuzz run never takes.** `Record` owns a `Vec<Point>`,
so every recorded decision allocates, and the non-certified path is the one a
fuzz target spends its whole night on — finding refusals is the point of it. So
the cost the design avoids is avoided exactly where it was never going to be
paid, and met in full where the machine runs for hours.

No crate in this workspace declares `[features]`, and there is no `cfg(feature =
…)` in any `src/`. This would be the first.

The cheap shape, if the measurement supports it: a stub `ProvenanceLog` behind a
feature that is on by default, whose `push` does nothing and whose log is always
empty. That keeps every `&mut ProvenanceLog` parameter — there are about twenty
in `intersect.rs` alone — so it is not an API change, and the fuzz harness turns
the feature back on to re-run a seed it wants explained. Which is the workflow
that was asked for.

**Measure before building.** §16.10's rule that a claim owes a demonstration
cuts both ways: "logging is expensive here" is a claim, and `RCAD_FUZZ_TIMINGS`
already exists to settle it. If provenance is a rounding error against the
predicates it records, the feature is not worth the second build configuration.

Two things the lane must settle, not assume. A feature that is off compiles a
second configuration that nothing exercises, so `just gate` has to run both or
the off path rots — that is a real cost and it is the reason to want a number
first. And with provenance stubbed out, `Error::Undecidable` can no longer say
why, so the feature must never be off in a build anyone reasons from; whether
that is enforceable, or only documented, is the lane's question.

Write set: `crates/rcad-predicates/`, the fuzz targets, `justfile` if the gate
gains a configuration.

### 144.10 What it owes

Five lanes, none blocking any other: §144.3 the context ceiling, §144.4 the
lookup trigger, §144.5 one of its three rituals, §144.6 `just recipe`, §144.9 the
provenance measurement. §144.7 is a `CLAUDE.md` correction and not a lane.

Each starts by re-running the report it names. A lane that quotes this section's
motivation without re-measuring is reporting a fortnight-old number as today's,
which is §77.6's mistake in a new place — a check that passes for a property of
the week it ran in rather than of the thing it is about.

