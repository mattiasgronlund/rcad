## 140. A crossing answers one way round and refuses the other, found 2026-09-19

*The night of 2026-09-19 wrote one crash that is not carry-over and still fails
on `main`. Two exactly parallel cylinders, radii `5.05e153` and `5.40e-8`: asked
one way the pair refuses with `CannotNameWhichPieceOfACrossingThisIs`, asked the
other way it answers with two curves. **Not fixed, and not a lane yet** — this
section is the reproducer and the framing, so the finding survives the tree the
timer rewrites.*

### 140.1 That it is tonight's, and the only one

Thirteen artifacts were copied out. Four of them are `crash-`, and `just
fuzz-triage` replays every one against `main` at `30f24be0`: three come back
*"replayed and succeeded"* and one does not.

The three that pass are old — libFuzzer's own mtimes in the worktree put two of
them at Sep 12 and Sep 14, and the fourth target's pair at Sep 19 00:16, which
is the *copy* time and not a find. This is §134.4's rule doing exactly its job:
every copy under `~/rcad-fuzz-logs/2026-09-19/artifacts/` carries 00:16, and the
one artifact written **during** the night carries 02:04. Read off the copies
alone, tonight's find is indistinguishable from a week-old one.

### 140.2 The pair, and the reproducer

| | axis origin | direction | radius |
| --- | --- | --- | ---: |
| first | `(0, 0, 0)` | `(1, 0, 0)` | `5.047645337156084e153` |
| second | `(5.047645337156084e153, 5.047645337156084e153, 41581179)` | `(1, 0, 0)` | `5.4016709327697754e-8` |

Exactly parallel — the same direction vector, not a near-parallel pair — with a
radius ratio of about `1e161` and the axes offset by ~5e153.

```
every_oracle_holds_of(two_surfaces_from(&bytes_for([2, 2], &[0.0, 0.0, 0.0,
127.0, 0.0, 0.0, 5.047645337156084e153, 5.047645337156084e153,
5.047645337156084e153, 41581179.0, 142606335.0, 0.0, 0.0,
5.4016709327697754e-8])).unwrap())
```

That line is the durable form and the reason it is written here rather than
promoted: `fuzz/artifacts/` is gitignored, and the tree it lives in is the one
the timer keeps checked out and rewrites nightly. Promoting the input into
`fuzz/corpus/` was considered and **refused for now** — the corpus is replayed at
the start of every run, so a still-panicking seed costs a **hand-run** `just fuzz`
its whole session before it fuzzes anything. It does not cost *the night* that,
and §140.8 corrects the sentence that said it did: the night runs `-fork` with
`-ignore_crashes=1`, which §133 added for exactly this reason. It becomes a corpus
seed when it is fixed, which is what `crash-a-thin-pipe-on-a-fat-ones-skin`
already is.

### 140.3 Which oracle, and what disagreed

`every_oracle_holds_of`, at `crates/rcad-geom/tests/fuzzing/mod.rs:537` — the
one that asks the same pair in both orders and requires the same answer:

    left: ("refused", 0)      right: ("curves", 2)

### 140.4 It is not §126.6's missing counterexample

Checked rather than assumed, because §0 carries §126.6 as open and the two would
be easy to conflate. They are different in all three of the ways that matter:

| | §126.6 | this |
| --- | --- | --- |
| subject | a **fuse** of a turned lattice pair | a **surface crossing** |
| oracle | `fuses_that_must_agree` | `every_oracle_holds_of` |
| disagreement | the answer differs **by a face** | a **refusal against an answer** |

So this does not close §126.6, and §126.6's advice — raise `SOAK_FACTOR` on the
fuse properties — does not hunt this. They are two order-dependences, not one
seen twice.

### 140.5 Limit or gap is not yet decided, and the obvious reading is a trap

§16.10 asks whether a refusal is a *limit* (no answer exists) or a *gap* (it
does and the arithmetic could not reach it). The tempting inference is that the
second order answering proves the answer exists, so the refusal is a gap.

**That inference is not available here, and saying so is the point.** A
commutativity failure says one of the two orders is wrong; it does not say
which. Either the refusal is a gap and the other order reaches the answer, or
the pair genuinely cannot be decided and the answering order is **unsound** —
claiming two curves it has not earned. The second is the worse defect of the
two, and nothing measured so far separates them. Whoever takes this decides that
question first, because it determines whether the fix adds reach or removes a
false answer.

### 140.6 What this cost the night, which is the before-picture and not a regression

The crash ended `where_two_surfaces_cross` at 02:04 of an eight-hour window,
because the night ran at `5336ff31` — before §134 landed — where one crash calls
`StopJobs()` and ends that target. It lost about six hours, and its report shows
251 620 runs in 6 581 s against the other four targets' full windows.

From `30f24be0` that stops being possible: `-ignore_crashes=1` is passed, so the
same crash is written and fuzzing continues. This night is the last one that
could lose six hours this way, which makes it the clearest evidence §134 was
worth having — and it arrived the morning §134 landed, by coincidence rather
than by design.

### 140.7 What this owes, decided rather than asked

**It owes a lane, and it is not going on the deferred list.** §13's
no-named-consumer rule is the usual way a finding waits, and it is not available
here: CLAUDE.md says in terms that §13 "governs **features, never correctness
gaps**". A crossing that answers one way round and refuses the other is not a
feature nobody asked for; it is the kernel giving two answers to one question.

That is a disposition and not a schedule. **What it does not decide is §140.5**,
which stays open on purpose: whether the refusal is the defect or the answer is,
is the lane's first question and not something this section guessed at to look
finished.

Recorded under §138 rather than left as a question. The session that found it
was at the end of its context and could not run the lane, which is a reason to
write the disposition down and hand it over — not a reason to park the finding
until someone is asked a question they have already answered generically.

Commands run to reach this: `mise exec -- just fuzz-triage`, which replayed all
thirteen artifacts against `main`; `ls -la` of
`~/git/mattiasgronlund/rcad-nightly-fuzz/fuzz/artifacts/where_two_surfaces_cross/`
and of the night's copy, for the mtimes §140.1 turns on; `cat` of the five
`report` files and of `~/rcad-fuzz-logs/latest`; `mise exec -- just decision
126.6` to rule out §140.4.

### 140.8 A4: the reason §140.2 gave for not promoting the input was already false

`rcad-f1` flagged its own sentence for audit within the hour of writing it,
suspecting the reason was narrower than the claim. It is worse than narrow: for
the night, it is **false**, and what falsified it was that session's own work
earlier the same morning.

§140.2 refused promotion because "seeding a still-panicking input would end that
target's night on its own corpus before fuzzing anything". That was true of the
night as it ran until 2026-09-19. It is not true of the night as it runs now: the
night computes `fork=$(nproc)/5`, which is never zero, so it passes
`-ignore_crashes=1` — and the `justfile` comment above that flag says in terms
that it "is the flag that buys the night: without it the first crash calls
`StopJobs()` and the target is done for the evening". §133 and §134 added it
hours before §140.2 was written. A crashing corpus seed now costs the night one
child process, not an evening.

Where the sentence still holds is a **hand-run** `just fuzz 60`, which defaults
`fork="0"` and so passes none of the fork flags. That is the same hand-run/night
split the `-timeout=10` comment four lines above already draws for exactly this
reason, which is the part that stings: the distinction was on the screen.

**The disposition survives and the family is still unexamined.** Not promoting is
defensible on the hand-run alone. But the sentence ruled out one destination on
grounds true of that destination, and the question it never asked is where else a
reproducer can durably live — and the answer was already being built, since lane A
is committing an `#[ignore]`d test carrying these bytes. That is the member of the
family that is neither gitignored nor replayed at startup.

**A4 is the fifth of this species in two days**, after §137.5, §137.6, and A1 and
A3 of §139.7. Three different sessions, five sections, every one of them citing
§16.10. The pattern is now specific enough to name: each was a claim about a
mechanism, written by the session that had most recently changed that mechanism,
and checkable in minutes against code that was already open.

