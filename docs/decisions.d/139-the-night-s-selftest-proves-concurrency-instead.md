## 139. The night's selftest proves concurrency instead of measuring it, built 2026-09-19

*§137.6 deferred this as work and not a limit, and named the design. `spread <= 3`
asked that the five targets launched within three seconds of each other — a bound
on a quantity the machine owns, pinned at both ends, and resting on an accident
nothing stated. A barrier replaces it: each target says it has arrived and then
waits for the other four. A correct recipe clears it at any timeout and a
sequential one fails it at any timeout, so the number stops being a guess.*

### 139.1 The two things wrong with the bound

The first is §137's own failure a third time, and §137.6 has the arithmetic: the
bound sits above five `just` forks and below the spread the adversary produces, so
a loaded runner fails it and **raising 3 to 30 deletes the check rather than
weakening it** — 30 clears even a fully sequential recipe's 20.

This subsection first added "there is no slack to give a slow machine", and that
was 30 mistaken for the whole family. With the overrunner leading — the shipped
arrangement — the adversary spreads 11, so any bound from 1 to 10 discriminates
and 3 could have been 10. The slack was real and unused. What removes it is the
**second** problem below and not this one: ask that the check work whichever
target leads and the adversary spreads exactly 3, which 3 already admits. §139.7
has the correction.

The second is worse because it is silent. The 11 s of margin the bound lived on
existed only because `STUB_OVERRUN_TARGET` is `where_two_surfaces_cross`, which
is what `fuzz` maps to, and `fuzz` is `fuzz_all`'s **first** entry. The overrunner
led the chain, so the adversary's spread was its 11 s. Point the overrun anywhere
else and a non-overrunning target leads, the adversary spreads exactly 3, and the
arm keeps passing while it stops discriminating. Nothing in the recipe stated the
coupling and nothing checked it.

### 139.2 The barrier

Armed by `STUB_RENDEZVOUS` and by nothing else. Each target appends its name to
the named file, then polls until the file holds five lines or a deadline passes,
and writes what it saw into the log. The arm asserts that **all five saw five**.

Run together, all five arrive and each sees five. Run in turn, the first blocks
at the barrier and the other four are never launched, so the file ends at one
line and the first target sees one. The later four can still see five — they
arrive after the first has already written its line — which is why the assertion
is *every* target saw five and not *some* target did.

It sits after the `at=` printf and before the sleep. Before the printf, the log
would stop recording when a target launched and §137.2's paired
`max_total_time`/`at=` check would lose its input; after the sleep, the five
would have finished waiting for each other before they ever met.

### 139.3 The asymmetry that makes this a different fix

Loosening `spread <= N` costs all of the check's power once ordering-independence
is asked for, per §139.1. Loosening the barrier's deadline costs **none**: once
the five have arrived, a correct recipe clears it at every larger value and a
sequential one fails it at every value. The guess is removed rather than re-tuned,
which is what makes this a different family and not a better number — §16.10's
distinction, applied to a test rather than to arithmetic.

The deadline is not free in both directions, and §139.7 corrects the sentence that
said it was: set to zero it fails a correct recipe too, since the five cannot meet
in no time. It has a **floor and no ceiling**, where the spread bound had both and
they met. A number you satisfy by being generous is a different kind of number
from one pinned at both ends, which is the asymmetry — not that the deadline has
no constraint at all.

It also asserts something strictly stronger: that all five were alive at one
instant, which is the property the comment above the arm already claimed to be
testing, rather than that they started within three seconds of one another.

### 139.4 The mutants, and which of them the arm actually owns

| mutation | what fired | what it said |
| --- | --- | --- |
| fully sequential (`wait` after every launch) | arm One, not this one | a twelve-second window took 30 s, so the window is a budget and not a deadline |
| the first target waited for, then the other four, **overrunner leading** | this arm | `where_two_surfaces_cross saw 1 of 5`, the other four `saw 5 of 5` |
| the same, **overrunner last** | this arm | the same message, unchanged |

The third row is the one worth having, and it is the reason the barrier was built
rather than the bound re-tuned: **the arm does not care which target leads.** The
first row is worth recording for the opposite reason — a fully sequential recipe
never reaches this assertion, because the clock catches it four arms earlier. The
adversary this arm owns is the one that keeps the elapsed time honest.

**What is not measured here, and it is §137.6's arithmetic rather than a run.**
That `spread <= 3` *passes* the third row was not reproduced directly: the same
mutation against the pre-change recipe trips an earlier arm — all five targets
skipped as `no-time` — so the run does not isolate the bound. The claim in §139.1
is the arithmetic of §137.6 and stands on that, not on an observation. Anyone
wanting it observed will need a mutation that leaves the earlier arms untouched.

### 139.5 Why it is asserted before the clock

A recipe that fails this arm fails it by waiting out the barrier's deadline, and
that wait is itself time on the clock. Asserted after `elapsed <= 20`, every
sequential recipe would report the elapsed bound instead, and the symptom would
bury the cause. So the barrier is checked first and the clock second, which is
the opposite of the order the two were written in.

### 139.6 The costs, named in advance

§137.6 named three and all three are real, though the first was argued from a case
that cannot happen. The stub is shared by every arm, so the barrier **must** be
env-var armed or an arm launching fewer than five targets waits out the deadline
for targets that were never coming. `run 0` is **not** that case, though this
subsection first named it: a recipe with under a second left is skipped before the
stub is ever reached, so nothing arrives at the barrier to wait. §139.7. The
deadline is a deadlock escape and not a tuning knob: without it a recipe that
launches one target hangs `just gate` rather than failing it. And the
max-concurrency counter remains the weaker sibling — it still needs the sleeps to
outlast the launch spread, which is the same machine-speed guess wearing a
different hat.

Commands run to reach this: `mise exec -- just check-fuzz-overnight-selftest`
against the tree and against four mutations, one of them run twice with the
overrun aimed at a leading and then a trailing target; `git checkout main --
justfile` to put the pre-change recipe back for the attempt recorded in §139.4;
`mise exec -- just gate`.

### 139.7 What a claim-audit of this section found, 2026-09-19

Run under `practices:claim-audit` the morning §139 landed, on the recommendation
in `rcad-f1`'s handoff, pointed at §139.1 first. **Three over-claims, all of them
sentences and none of them the design.** The barrier is still the right fix and
every number in §139.4 stands; what was wrong is the arguing.

**A1 — "there is no slack to give a slow machine" was false as written.** §139.1
ruled out a family by testing one member. 30 does delete the check, because it
clears a fully sequential recipe's 20 — but §139.4's own first row says this arm
never sees the fully sequential case, the clock catches it four arms earlier. The
adversary this arm owns spreads **11** with the overrunner leading, so the usable
range was 1 to 10 and the shipped 3 was sitting on seven unused seconds. A loaded
runner could have been given more than three times the tolerance it had. The
sentence is corrected above, and the honest statement is narrower: the slack
exists and vanishes only when ordering-independence is demanded — at which point
the bound is not slack-free but already **broken**, since the trailing-overrunner
adversary spreads exactly 3 and `spread <= 3` admits it.

That reorders §139.1's two problems. They read as independent; they are not. The
second is the whole reason, and the first is its consequence.

**A2 — the deadline is free in one direction, not two.** §139.3 said a correct
recipe "clears it instantly at any value". At `STUB_RENDEZVOUS_TIMEOUT=0` the
deadline is already past when the first target tests it, so a correct recipe times
out too. The deadline has a floor — the instant all five have arrived, which is
the same launch-spread quantity the bound was measuring — and no ceiling. The
asymmetry that justifies the different family survives and is what the corrected
text now says: a floor cleared by generosity is not a bound pinned at both ends.

**A3 — `run 0` cannot exhibit the cost it was named for.** §139.6 justified the
mandatory env-var arming with `run 0`, "which correctly starts no targets". A
recipe with under a second left is `continue`d before it is ever launched, so its
stub is never invoked and nothing reaches the barrier. The cost is real — an arm
launching *between one and four* targets is the case, and §139.6's own next
sentence names it — but the example given was impossible.

**Why this is worth recording rather than just fixing.** A1 is the same species as
§137.5 and §137.6, which `rcad-f1` corrected within an hour of each other the day
before: a family named, one member ruled out, the conclusion drawn about the
family. That is three in §137 and a fourth here, all by different sessions, all in
prose that read as careful. §16.10's rule is not failing for lack of being known —
every one of these sections cites it. What catches them is running the arithmetic
on the *other* members, which costs a few minutes and is what this audit did.

None of the three changes a line of the recipe. That is the result: the audit
found the reasoning weak and the artefact sound, which is the outcome the method
is for and not a reason to skip it next time.

