## 137. The night's selftest measured a share against the window it asked for instead of the window it got, fixed 2026-09-19

`check-fuzz-overnight-selftest` turned `main` red on `bcff8e25`, a commit that
changed **one line of `docs/decisions.md`**. It is a flaky arm, and the flake was
not a race between two threads but a check comparing a measurement against the
wrong constant.

### 137.1 What it asserted and why that was wrong

§129 made the night run all five targets at once rather than a fifth of the
window each, and the arm that holds it asks what each target was *told*:

```
[ "$granted" -ge 8 ]   # of a twelve-second window
```

The five targets are handed `left = deadline - now`, computed once, where
`deadline` was set to `now + 12` at the top of the recipe. So what a target
receives is **twelve minus the preamble** — the watermark, the git calls, a
nested `just` per target. On this machine the preamble rounds to nothing and
`granted` is 12, measured twice. On the GitHub runner it was **9 seconds of the
12**, and the *correct* recipe granted 3.

The arithmetic is readable straight off the failing run's own log. All five
targets logged `at=1789769029` with `-max_total_time=3`, so the deadline was
`1789769032` and the recipe started at `1789769020` — nine seconds earlier.

**The worse half is not the red build.** A fifth of twelve is 2. At a preamble of
nine the correct recipe grants 3 and the mutation this arm exists to catch grants
2, so the arm had lost almost all of its power to tell them apart. A check that
fails the correct answer on a slow machine is also a check that would have passed
the wrong one there.

### 137.2 The fix: each grant against its own launch instant

Every grant is held against the time remaining when **that** target launched,
paired with it out of the stub's own log line:

```
pairs=$(sed -n 's/.*max_total_time=\([0-9]*\).*\[at=\([0-9]*\)\].*/\1 \2/p' "$dir/log")
[ "$(grep -c . <<<"$pairs")" -eq 5 ] || fail ...
while read -r granted at; do
    whole=$(( started + 12 - at ))
    [ "$granted" -ge "$whole" ] || fail ...
done <<<"$pairs"
```

`started` is read *before* `fuzz-overnight` begins, so `started + 12` is at or
before the deadline the recipe sets, and each `whole` is a **lower bound** on what
a correct recipe grants that target. The assertion therefore cannot fail for a
slow machine — only for a recipe that shares the window out. No fixture grew and
the gate got no slower; the twelve-second window stands.

**It replaced two checks with one, and the `distinct == 1` check went with it.**
That check required all five grants to be the same number. §137.5 first recorded
that as a latent flake and it is not one — see there, the claim was wrong — but it
is unsound for a different reason: **it forbids a correct recipe.** One computing
`deadline - now` inside the launch loop hands a target that started two seconds
later two seconds less, which is right, and a set-wide check calls that a divided
window. Allowing the values to differ instead is what lets a genuinely divided
window through. No bound on the five values *together* separates those two cases,
so the property does not belong on the set. Paired, it needs no new
instrumentation: `$*` in the stub's `printf` is its whole argv, so every
`-max_total_time=N` is already on the same line as the `at=` stamp of the target
it was handed to.

Raised by `rcad-f1` on 2026-09-19 against §16.10, which asks a claim that
something cannot be done to name the family of fixes it considered. §137.5 had
considered exactly one — relaxing the set-wide bound — and pairing is a second
family it never named. The remedy is theirs; the reason below is not the one
either of us started with.

### 137.3 The demonstration, both directions

A test whose subject is a test still owes the two runs. `fuzz-overnight` was
mutated in the tree, measured, and restored; `git diff` afterwards shows the
assertion change alone.

| probe | grants | `whole` | old `-ge 8` | old `distinct == 1` | paired |
|---|---|---|---|---|---|
| as shipped | 12 | 12 | pass | pass | **pass** |
| `sleep 9` before the launch, case one only | 3 | 3 | **fail** | pass | **pass** |
| `left` divided by five | 2 | 12 | fail | pass | **fail** |
| `left` recomputed inside the launch loop, `sleep 1` between | 11, 10, 9, 8, 7 | 10, 9, 8, 7, 6 | pass | **fail** | **pass** |

Row two reproduces the runner exactly, down to the message: *"each target was
given 3 s of a twelve-second window"*, the same sentence CI printed. Row three is
the one that matters for the fix's honesty — the new assertion still catches the
mutation, and names it: *"a target was given 2 s of the 12 s still left when it
launched"*.

**Row four is the one that condemns `distinct == 1`.** Five different grants, each
the whole of what was left when that target launched — a correct recipe, and the
set-wide check rejects it as "still being divided" while the paired check passes
it. That is not a hypothetical: it is what the obvious refactor of
`fuzz-overnight` looks like.

Row three needed the `elapsed -ge 4` check relaxed to be reached at all, because
a divided window makes the night finish in a second and that guard fires first.
Both guards catch the mutation; only the size one names it. Rows two and four are
confined to case one by testing `{{ window }}` against 12, because a blunt `sleep`
starves the later sub-cases, whose windows are 8, 6 and 0 — the first attempt at
row two did exactly that and failed on "a night that found nothing exited
non-zero" instead.

### 137.4 Why it went unnoticed

This arm landed on 2026-09-18 in `0ac02509` and was green on the runner twice
before this. Its own comment already records a `date +%s` truncation race found
by rcad-15 on 2026-09-09, fixed by sleeping *half* the slice — so the family was
known and this member of it was missed anyway. The reason is that both the
fixture and the assertion were written on a machine where the preamble rounds to
zero, which makes `12` and `12 - preamble` the same number.

That is §77.6's mistake once more: a check that passes for a property of the
environment it ran in rather than of the tree. The three there were counting
colour codes and asking after a plugin; this one hard-codes a constant that is
only true on a fast machine. **The way to find the next one is the same: take
away what the environment supplies and run it again.** Here that was `sleep 9`.

### 137.5 The claim this section first made about `distinct == 1`, and why it was wrong

This subsection first said the `distinct == 1` check was a latent flake of the
same shape: that on a machine slow enough to straddle a second boundary, two
targets would be told 3 and three told 2. **That cannot happen, and the reason is
four lines above the check it describes.** `left` is computed *once*, before the
launch loop:

```
left=$(( deadline - $(date +%s) ))
pids=()
for recipe in "${recipes[@]}"; do
```

and the per-target recipe spells it straight into `-max_total_time={{duration}}`.
So however far apart the five launches drift, they are all told the same number by
construction. Measured, with `sleep 1` forced between the launches: the `at=`
stamps spread over five seconds — `…163, …164, …165, …166, …167` — while every
target was handed `max_total_time=6`. One value, five instants.

So the sentence described a mechanism that does not exist, in a section whose
whole subject is a check that measured the wrong thing. §16.10 asks for the family
of fixes considered; the prior text named one, and it was answering a question
nobody had.

**The check was unsound anyway, for the opposite reason**, and §137.2 has it: far
from false-failing on a slow machine, `distinct == 1` false-fails a *correct*
recipe — the per-target one of row four. Paired, the property no longer lives on
the set and the check is gone.

### 137.6 What is left: the launch-spread bound, which really is environment-shaped

`spread <= 3` in sub-case four is the one check in this recipe still measured
against a constant that is a property of the machine. It reads the `at=` stamps
and asks that the five launches happened within three seconds, and its own comment
says why three: *"the five `just` invocations are five processes to fork, and
`date +%s` truncates to whole seconds either side."* A runner loaded enough to
take four seconds over five forks fails it, exactly as the preamble failed
`-ge 8`.

**The bind is tighter than "any bound is a guess", and that is what makes it
interesting.** The bound has to sit *between* two quantities that both move with
the machine and the window: below it, five `just` forks, which is why three and
not zero; above it, the spread a sequential recipe would produce. Sub-case four
runs a six-second window and the stub sleeps `granted / 2`, and its overrun target
sleeps `3 + 8`: the named adversary — run one, wait for it, then start the other
four — spreads **11 s**, and a fully sequential recipe stamps 0, 11, 14, 17, 20 and
spreads **20**. So the bound **cannot be loosened for a slow runner at all**:
raising 3 to 30 does not weaken the check, it deletes it. Any number in that gap is
pinned at both ends.

**And the 11 s of margin rests on something nothing states and nothing checks.**
It is there only because `STUB_OVERRUN_TARGET` is `where_two_surfaces_cross`,
which is what `fuzz` maps to (`justfile:2429`), and `fuzz` is `fuzz_all`'s **first**
entry (`justfile:2088`) — so the overrunner leads the chain and its 11 s is what
the adversary's spread becomes. Point `STUB_OVERRUN_TARGET` at any other target, or
reorder `fuzz_all`, and a non-overrunning target leads: the adversary then spreads
exactly 3, `spread <= 3` admits it, and **the arm keeps passing while it stops
discriminating.** That is this section's own failure mode a third time, live in the
check as it stands, and it is an argument for the rendezvous beyond the bound —
a barrier does not care which target leads.

**A fix exists and it is a different kind of fix, not a better bound: make the
recipe prove concurrency instead of measuring it.** A rendezvous — armed by an
environment variable for this sub-case alone, each stub appends its target name to
a file and then polls until the file holds five lines or a generous deadline
passes; the arm asserts every target saw five. A concurrent recipe has all five
arrive, each sees five, all proceed, and machine speed is irrelevant. A sequential
one blocks target one at the barrier, so targets two through five never launch and
the file ends at one line. Deterministic in both directions.

What makes that a different family rather than another guess is an asymmetry:
loosening `spread <= N` costs all of the check's power, per the paragraph above,
while loosening a rendezvous timeout costs **none** — a correct recipe clears the
barrier instantly at any value and a sequential one fails it at every value. The
guess is removed rather than re-tuned. It also asserts something strictly stronger
than what is there now: that all five were alive at one instant, which is the
property the comment at `justfile:2780` already claims to be testing, rather than
that they started within three seconds of each other.

Named costs, so they are not discovered later: the stub is shared by every arm in
the recipe, so the barrier must be env-var armed or any arm running fewer than
five targets hangs; the timeout is a deadlock escape and not optional, since
without it a broken recipe hangs `just gate` rather than failing it. A
max-concurrency counter — stub increments on entry, decrements on exit, assert the
maximum reached five — is the weaker sibling and is **not** the fix: it still needs
the sleeps to outlast the launch spread, so it is a measurement again and inherits
the same machine-speed guess.

**So this is deferred as work to do, not as a limit.** §16.10's distinction: the
answer exists and the arithmetic reaches it, which makes it a gap. It has never
been seen to fire — all five stamped the same second even on the runner that
failed above — so it is not urgent, and it is not this session's to write with
`fc2e38a5` already in flight in the same recipe and `rcad-f1`'s renumber touching
nine more of its sites.

Raised by `rcad-f1` on 2026-09-19, the second time in one day it held a subsection
of §137 to §16.10 and was right. The first correction, §137.5's, was to a claim
that was false; this one is to a claim that no fix was evident, which is the
softer and more common failure — and both were written by the same session in the
same hour, which is the argument for the rule rather than against the session.

The leading-overrunner finding is `rcad-f1`'s too, and it arrived by looking for a
*third* thin claim rather than by being told one was there: the comment at
`justfile:2780` says what a sequential recipe fails to satisfy, and whether that
sentence is true turns entirely on which target happens to be first. It is true
today. Nothing holds it true tomorrow.

### 137.7 Provenance

Found by pushing `bcff8e25` and watching CI, 2026-09-19. Run `35399208851`
failed; `35399049264` on `5336ff31` and `35398576360` on `912cd168` both
succeeded, and `5336ff31` differs from `bcff8e25` by one line of prose in a file
no part of this recipe reads — which is what said "flaky arm" before any of the
arithmetic was done. The assertion was already on `a47d990a`, the green base the
§121 stack landed on, so no commit of that stack introduced it.

**The `-ge 8` half landed first, as `4ed17d23`, with `distinct == 1` left standing
and §137.5 defending it on a mechanism that does not exist.** `rcad-f1` read that
section the same day and held it to §16.10: one family of fixes considered is not
an argument that the family is the only one. It named pairing, and named the
instrumentation — `$*` in the stub's `printf` already carries the grant and the
stamp on one line.

Checking it is what found that §137.5's premise was false: `left` is computed once
before the launch loop, so the straddle it feared is impossible, and the forced
`sleep 1` measurement proves it. The remedy is `rcad-f1`'s, arrived at from a
reading of the recipe that was wrong in the same place §137.5 was wrong — both of
us had the five grants varying with launch time when they cannot. What justifies
pairing is not straddling but row four of §137.3: the set-wide check rejects a
correct per-target recipe.

Which is the case for reading a peer's objection by checking the code rather than
by weighing the argument. The objection was right, its reason was not, and the real
reason was better than either.

