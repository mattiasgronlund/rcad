## 62. A sweep that could only ever return one answer, recorded 2026-09-07

A fourth way a written-down finding stops being worth what it looked worth, and
it is not the three in `docs/consumers.md` — those are all *the world moved under
a true sentence*, and this one is *the sentence was never able to be true*. The
shape:

> **A sweep whose sample set could only ever return one of the two answers.** The
> run completes, the samples are real, every assertion passes — and the finding
> was settled before the first sample was drawn, because the sample set could not
> have produced the other one.

§16.11 is the nearest neighbour and is a different failure: there a check was
right about what it caught and every sentence citing it was reasoning about the
other half. Here the measurement is right, the sentences about it are right, and
the *sample set* is the thing that was never able to say the other thing.

The unifying statement across all four, which `docs/consumers.md` also carries
and which rscene asked be credited to this side:

> The artefact stayed internally consistent while the world it described moved,
> or was never fully sampled — and review checks consistency.

The first three arms are the *moved* half. This is the *never fully sampled*
half, and it is the one that says why a reading pass catches none of them.

### 62.1 The instance, and nothing in it was wrong

`crates/rcad-geom/tests/what_the_dynamic_range_check_is_a_margin_against.rs`
asks how small a wedge's feature can get against its coordinates before the apex
rounds onto the base and `orient3d` answers `Zero` about a model nobody built.
As first written it swept the coordinate at `1`, `2^10`, `2^20` and `2^40`.

Measured here at `b4e698f` by reading the bit patterns: all four have a mantissa
of exactly zero. That is the whole defect. A feature of exactly half an ulp puts
the apex exactly halfway between two doubles, round-half-to-even then decides,
and a zero mantissa is the even side of every such tie — so all four coordinates
round the same way, and a **cliff** was the only shape of answer the sample set
could return. It returned one, and it looked complete returning it.

**Nothing it asserted was false and the code under it was not buggy.** Worth
saying plainly, because it is what makes the arm hard to see: the run was green
for the right reason, on real samples, about real arithmetic. Its verdict — that
the spread the kernel publishes as supported sits far below where a written
feature dies — stands untouched, and one binade of head-room on half the
coordinates does not move it. The failure is that the run *looked complete*.

### 62.2 What the axis it held still actually does

Another lane sampled every representable placement across a whole binade and
found the law, in `rcad-topo`'s
`tests/how_far_out_a_ball_can_sit_and_still_mesh.rs`: refusal is **not monotone
in where a solid stands**. For a ball of radius `r` centred at `c` everything is
`r / ulp(c)`:

- above a half, the poles land in different ulp buckets and it meshes wherever
  it stands;
- below a half, both poles round back onto `c` and it refuses wherever it
  stands — and this does not depend on `c` at all;
- at exactly a half the poles sit on a tie, and round-half-to-even decides on
  the last bit of `c`'s mantissa. Even rounds inward and refuses; odd rounds
  outward and meshes.

`ulp` is constant across a binade, so the third case is not a knife edge: it
holds for a whole factor-of-two stretch, and the answer flips at every
representable placement in it. That lane's census — **32 meshed and 32 refused
over 64 consecutive placements** — is **their** measurement, not one re-taken
here. What was re-run here at `b4e698f` is the file itself: 13 tests, 13 passed.

A cliff cannot produce that. It cannot even produce the report that started it:
two placements, the lower refusing and the higher meshing, which is the one pair
a threshold of any height is forbidden to explain.

### 62.3 Why this is a category and not carelessness

**rcad-51's sentence, kept as they wrote it, because it is the section:**

> The `COORDINATES` doc comment reasoned explicitly about its sampling and still
> missed the axis, **because powers of two are simultaneously the natural thing
> to sweep and the one family with no mantissa to be odd**.

The obvious choice and the blind choice were the same choice. That is what earns
this a name of its own. A thoughtful comment was not the missing ingredient —
the thoughtful comment had been written, and read, and it argued about the
sampling and came out blind anyway, because the argument was about spanning the
range and the blindness was one bit below where it was looking.

It also rules out the two repairs that suggest themselves. *"Never sample only
powers of two"* is too narrow: powers of two are the right sample for a question
about magnitude, and the next instance of this will be some other convention
that happens to be constant in whatever the next answer turns on. *"Sample
more"* is useless and, here, wrong — the fix went from four coordinates to
eight and did not go one step finer in magnitude.

**A second instance, and it is rcad-51's own, committed within the hour of
writing the argument above.** Asking which files under `crates/rcad/tests`
mention `rcad_geom`, it grepped for `rcad_geom` and not for the hyphenated
`rcad-geom` that prose uses. The natural spelling and the blind spelling were
the same choice, one level down from the powers of two.

**Its own qualification, which goes in with it.** The load-bearing claim — which
files *import* `rcad_geom` — was never at risk, because a Rust import can only be
spelled with the underscore, so the narrow pattern found every import there was.
What the sample set could not reach was the claim actually written down, which
was about **mentions**. Re-measured here at `b4e698f`:
`grep -rn "rcad_geom\|rcad-geom" crates/rcad/tests crates/rcad/benches` gives six
hits across four files, `grep -rn "rcad_geom::" …` gives one, and there is no
`crates/rcad/examples`. So five of the six are prose, and a grep for the
underscore alone sees one of six while answering a question about all six.

That makes it the muddier illustration: it is two arms at once, a blind sample
set **and** a sentence that drifted off its evidence, and only the first of those
is this section. Where it is genuinely the stronger one is that its blind sample
set was **a spelling a person chose**. Nothing about binary floating point is
available to explain it away, which is rcad-51's point and their framing: the
fourth arm is about the **sampler, not the sampled**, and a sampler is as blind
in a grep pattern as in a `COORDINATES` array.

### 62.4 The rule, sharp enough to use before writing a sweep

**A sweep is blind exactly when its sample set has fewer degrees of freedom than
the answer does.** So, before running it:

1. Name every quantity the answer can turn on — not the ones being swept, all of
   them. Rounding puts more on that list than the shape does: exponent,
   mantissa, parity of the last bit, sign, which side of a tie.
2. For each, name **two samples in the set that differ on it and agree on
   everything else.** Not "the set covers a wide range" — two named members.
3. Anything with no such pair is a **constant of the run**, not a variable. The
   finding is then a statement about that constant, and it should be written
   down as one or the sample set should be widened until it is not.

Step 2 is the whole rule. It is the one that fails on this instance and it fails
in one line: for *parity of the last mantissa bit*, there is no such pair among
`1`, `2^10`, `2^20`, `2^40`, because all four are zero.

The trap the rule is aimed at: **the family reached for by convention is usually
the family with the fewest degrees of freedom.** Powers of two are the canonical
case — what anyone would pick to span a range, and constant in the one bit this
answer turned on — but tidy integers, unit radii, the origin and the identity
placement are the same trap. Convention is what makes a family easy to write
*and* what makes every member of it agree.

**What it costs**, and it is not nothing. Step 1 wants somebody who knows what
the arithmetic can turn on, which is the expensive ingredient and is not
purchasable by sampling harder. Step 2 costs a doubling of the sample set per
axis it finds — here 4 coordinates became 8 and the run stayed instant, but an
axis found in a family that already takes minutes is a real bill, and it is
sometimes right to name the constant instead of paying it. Naming it is the
cheap outcome and it is a legitimate one: *"this is a statement about even
mantissas"* is a true, useful, much smaller finding than the one that was
claimed.

### 62.5 The standing form: make the sweep prove it saw both answers

Steps 1-3 are done by a person once. What stops the sample set silently
narrowing again is an assertion, and the fixed file carries it:
`the_spread_a_written_feature_survives_to` counts how many coordinates kept the
feature at the tie and how many lost it, and asserts the two counts are
**equal** — with a message saying the sweep is not sampling both sides of the
tie and can only report one cliff. Measured here at `b4e698f` from the test's own
output: *"a written feature survives to a spread of `2^52` at worst and `2^53`
at best; at the tie 4 kept the feature and 4 swallowed it; 108 still there, 44
gone."*

That is a **positive control on the sample set**, and it is the same argument
§56.7 makes about an absence that could not have been a presence, one level
down: there the control proves the sweep could have failed, here it proves the
sweep could have answered the other way. §42.12's tenth criterion is the same
idea for a rate.

It is also §15.6's rule pointed at a sample set rather than at a bound. A
published bound wants a test; a published finding wants a test that the sample
set could have contradicted it.

### 62.6 The `testing` skill already had this, and what it does not reach

*"Vary what the family cannot vary, not just more of what it can"* is in the
skill already, under **"A second family for what the first cannot draw"**. Read
in full rather than skimmed to its examples, it goes further than those examples
suggest: its two worked cases are shapes — the notched star whose corners never
give two edges overlapping bounding boxes, and `a_simple_polygon` beside it —
but its closing clause is *"the knots in the spline families are not all small
whole numbers for the same reason"*, and a knot value is a number. **"Not all
small whole numbers" is already a warning against a degenerate numeric sample
set.** So the rule covers a family's parameters, numbers included, and asking it
to say "this is about numbers too" would be redundant with a clause it already
has. Saying so explicitly here, because a section about sample sets that only
sampled the two shape examples would be its own subject matter.

**What the rule does not reach is the coordinates a family is *placed at*.**
Those are not parameters of the family. Nothing in the family generates them,
shrinking never touches them, and a family drawn entirely correctly can be
evaluated at one placement for its whole life. They are the arithmetic context
the family runs in — and they are the axis both defects behind this section were
blind on.

So the sentence that is missing is: **a family's placement is part of its sample
set**, with §62.4's degrees-of-freedom check as how to apply it. That is a change
to `.claude/skills/testing/SKILL.md`, and this lane does not own that file; it is
reported rather than made.

### 62.7 The uncomfortable half

This family is why the `claim-audit` skill would have caught none of the four
arms, and the fourth is the proof rather than another example. A claim audit
reads a sentence and asks whether it is consistent and whether it names
something real. This doc comment was both, it had already had exactly that pass
in writing, and it was blind. Saying *review harder* would have changed nothing,
so it is not what is recorded.

The skill is not thereby useless — it is aimed at claims that something cannot
be done or cannot be reached (§16.10), and it finds those. What it cannot see is
a claim that is true about everything it sampled and silent about what it did
not. **The control for that is arithmetic, not reading**: a balance assertion,
or a census one level finer than the claim.

### 62.8 Six instances, three of them from inside the work of writing this section

The two arms above are the mechanism, found in a test file's own sampling. This
week it also fired in prose, about the tools this project reads its own state
through — and it fired a third time inside the sessions doing the finding.
Mattias asked for the count; it moved twice while the question was open, which
belongs in the record beside the instances themselves. There are six.

1. **Four became seven, in two files.** `justfile` and `CLAUDE.md` both said
   four `rcad-step` test files skip without a corpus. `grep -rln
   RCAD_STEP_CORPUS --include=*.rs` gives seven, and none of them skip —
   `every_file_in_the_corpus_reads.rs:149` panics rather than passing, on
   purpose, with the message "this test does not skip on a missing corpus."
   Fixed in `fe602b8`.
2. **A header outlived the sentence it opened with.** `.github/workflows/ci.yml`
   opened "This workflow has never run. There is no remote and no CI." `origin`
   had pointed at GitHub for at least a day by the time that sentence was still
   being read as current. Fixed in `1f880ea`.
3. **"Ten consecutive failures, every one in the tool install, none reaching a
   check."** `fe602b8` wrote that into `CLAUDE.md`, read off `gh run list`'s
   first page — ten rows, every one a push to `main`, the only branch this
   repository's CI runs on. Run `33916660058` sits on that same page: it took
   11m28s and passed `just check` and `just lint` before failing on the corpus.
   `949ea07` corrected it by counting all 26: zero successes, ever — a fact
   only a fix can falsify, where a count goes stale on the next push.
4. **The same sentence, in a different file, independently, within the hour.**
   `1f880ea` wrote "fired on ten pushes to `main` … every one failed in 11–25
   seconds without reaching a single check" into `ci.yml`'s own header — the
   same first page, read by the session correcting the *previous* false
   header. `bb90345` corrected it, in its own words: "That was false, and it
   was mine."
5. **A handoff document dropped its own `Q22`.** A session's list of its own
   open questions, kept in its scratchpad, silently lost one entry and read as
   complete. The scratchpad dies with the session by design (this file's own
   "Never in the tree"), so this instance is not re-checkable the way the other
   five are — it stands on the session's contemporaneous report and nothing
   else.
6. **375 MB is the directory; the tests read 133.4 MiB of it.** `ci.yml` and a
   session's notes called the corpus "375 MB of third-party files," and `du -sh`
   on the corpus directory does give `375M` — true, of the directory.
   `every_file_in_the_corpus_reads.rs:175-180` opens only `.stp` and `.step`,
   case-insensitively. Counted directly: the corpus holds 520 files, of which
   367 are `.stp`/`.step` totalling 139867388 bytes (133.4 MiB); the other 153
   — `.pdf`, `.prt`, `.SLDPRT`, `.CATPart`, `.ipt` and others — are never opened
   by a test.

Three and four are the sharpest pair, and worth naming on their own: **`gh run
list` defaults to one page, and for a repository whose CI runs only on pushes
to `main`, that page is every push to `main`.** The natural invocation and the
blind invocation are the same invocation — §62.3's "the obvious choice and the
blind choice were the same choice," said about a tool instead of a float, and
it took two sessions saying it independently inside one hour to notice.

### 62.9 Two more, in tooling rather than prose

A run of `mise exec -- just default` was still in progress when a watcher timed
out on it and grepped `gh run view --log-failed` anyway. On a run that has not
finished, that grep returns nothing, so it printed "no failure found" about a
job that had not failed — it had not run its course yet. The rule this earns,
in its own words rather than a repeat of "assert both directions": **an
observable that reads empty in the failure case cannot be the whole test**,
because it reads exactly as empty when the run is simply not done.

Run `34150494501` came back `conclusion: failure` after 0 seconds — `createdAt`
and `updatedAt` are the same second, `jobs` is empty, and
`repos/mattiasgronlund/rcad/commits/47264160.../check-runs` gives
`total_count: 0`. That is not a fast failure, it is a run that never started —
its push, `4726416`, carried a workflow file GitHub could not schedule a job
from — and there is nothing to annotate. A 0 s failure and a failing step want
reading differently, and the trap under both is the same one: the evidence that
would tell them apart is missing in exactly the case you need it.

### 62.10 A verification that inherited the claim's own blind spot

This crate's own name is spelled two ways — `rcad_geom` in code, `rcad-geom` in
prose about paths and dependencies — and `CLAUDE.md` says so in words. The trap
was already named. Two sessions walked into it anyway, from opposite
directions, inside a day.

Re-derived at `de7a932` (branch `facade-range-check-pairing`): `grep -rn
"rcad_geom" crates/rcad/tests crates/rcad/benches` gives 3; `grep -rnE
"rcad_geom|rcad-geom" crates/rcad/tests crates/rcad/benches` gives 7, across 5
files. Of the 7, 4 spell it with a hyphen and are invisible to the narrower
grep.

**Not the same shape as §62.1/§62.2.** Those were a sweep whose sample set
could not reach the answer it needed. Here a number had already been checked
once: a stale claim of "prose in five places" was re-measured down to 3, and
that re-measurement is what `de7a932`'s own commit message publishes. The
grep behind it was `rcad_geom` alone — the same underscore-only blind spot the
claim it was correcting already had. **Checking a number is supposed to be the
activity that catches this, and the check inherited the error instead of
catching it.** `rcad-51` caught it only by reproducing the grep rather than
reading the commit — a third pass, and the only one of the three that
recomputed instead of trusting the pass before it.

The chain is a claim, a verification that repeats the claim's own error, and a
third pass that catches it only because it recomputed. That is the sharper
finding: a verification is not exempt from the sample-set question just
because it is a verification. "Did the check see both spellings" has to be
asked again at every hop, including the hop whose job is to ask it.

### 62.11 A correction that was written, correct, and could not reach anyone

Every arm above is about a claim that was never able to be true. This one is
about a claim that **was** repaired, on time, by someone who did the work — and
went on spreading anyway, because a fix that has not landed cannot be read.

On 2026-09-08 three sessions told the dispatcher that rcad's CI had never
succeeded. One quoted "26 runs, 26 failures"; another said the next run would be
"the interesting one" because none had ever gone green. Both false: three
successes, two of them pushes to `main`, re-derived twice from `gh run list`.

**Neither session was careless, and the path is the finding.** `origin/main`'s
CLAUDE.md still said both things, because the correction sat gated and unlanded
in `0a917e9`. One session then traced its own copy and found it had come from
*another session*, not from the file. So the route was CLAUDE.md → session →
session → dispatcher, and by the time the correction existed the error was
replicating between agents without touching the document it came from.

**Landing the fix does not retract what is already in three contexts.** It took
two messages from the dispatcher and a third from a peer to catch up with a
sentence that was already wrong when it was copied. A record that cannot be read
is worse than no record, because it is trusted.

That is a different shape from §62.8–62.10. Those are about a sample set that
could only return one answer. This is about **latency**: the correction was
written, was right, was gated, and was unreachable, while the error kept moving.
The rule that falls out is not about sweeps at all — **a correction is not
delivered when it is written, it is delivered when it lands.** Until then the
old sentence is still the one being read, and anyone told the new one is holding
it on your word rather than the record's.

**The same day gave the duplicated-copy version of it.** "No `just default` run
has gone green on `main`" was true when written and lived in three files —
`ci.yml`'s header, §77.5, and CLAUDE.md. A green run made all three false at
once, and two sessions between them had flagged only two of the three. Landing a
repair in one copy leaves the others standing and *looking* authoritative. This
is §64's rule arriving from the other direction: §64 says do not keep a second
copy of a fact because it goes stale; here the going-stale had already happened
and the cost was one more file that nobody had listed.

**And it gives §62 the case it did not have: verification that worked.** The
dispatcher was corrected twice that afternoon by peers who re-derived instead of
reading, and both were right. §62.10 asked what a second opinion has to *do* to
be one; the answer both times was that it recomputed. A peer session read one
clippy log two ways — anchored grep 0, escapes stripped 83 — and settled §77.6's
cause with a measurement the dispatcher's own reproduction had not produced.
Cheap, independent, and pointed at the thing rather than at the argument about
the thing.

