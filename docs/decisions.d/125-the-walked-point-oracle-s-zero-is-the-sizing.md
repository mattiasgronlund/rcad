## 125. The walked-point oracle's zero is the sizing walk refusing, and it is a gap twice and a limit twice, measured 2026-09-18

§108.4 left one question open in as many words — four slow units that are "one
huge pipe against one small one … one question about scale and not four
findings", and "what it costs is open … left as a question rather than answered
here, because the answer is measurement this section did not do." This section
is that measurement.

The nights of 2026-09-17 and 2026-09-18 added three more units of the same
family. Counting the two already seeded, five were decoded. Four come back
`Asked { points: 0, orders: 1, scalings: 2 }` and one comes back
`Asked { points: 10, … }`.

### 125.1 A zero had four meanings and now has one

`every_point_of_every_crossing_is_on_both_surfaces` counted one number, and the
loop feeding it ends four ways: the coarse sizing walk refusing, the size it
hands back not being a length, the fine walk refusing, or every walked point
losing its box in `somewhere_within`. The fourth is the quiet one — `checked +=
1` sits *after* that `continue`, so a point whose box collapses never reaches
the counter, and a zero could have meant "walked plenty and silently dropped
them all".

**It is the first exit, on every run of all four, and never any of the other
three.** `how_every_crossing_was_walked` now reports per run rather than per
pair, and the public counter is its sum, so there is one body and not two.

The fourth exit is cleared rather than argued away: on the one pair that does
walk, `drawn == checked == 5` on both runs.

### 125.2 Two of them are a gap

`1bd43cf4` and `d8b562bb` are the seeded `7e62ebe6` with a y-component of
`3.85e-122` or `1.15e-129` put on the thin pipe's axis, and nothing else
changed. That cannot move where a pipe of radius `1.68e8` and one of radius
`1.375` cross. Both refuse `TooManyPointsAlongOneCrossing { most: 1048576 }`.

By §16.10 that is a **gap**: the answer exists and the arithmetic reaches it.
The same two runs walk at `0.01`, `1.0`, `1e10` and `1e16`, every point coming
back closer to the crossing than the thin pipe is wide. Only the harness's
`f64::MAX / 4.0` sizing ask does not.

The tilt is not a second bug. It is what takes this pair from *just* surviving
that ask to not surviving it, which the untilted `7e62ebe6` shows by walking ten
points through the same code.

### 125.3 Two of them are a limit

`5dddf2a9` and `3a4a8231` refuse at every ask, including the finest this harness
makes — `CannotFollowACrossingWhereItsSurfacesAreTangent` in microseconds at
`0.01`. Their loops are `4.34e-19` across where one step of the `f64` grid at
those coordinates is `5.82e-11`, so no `f64` point sits on both pipes.

By §16.10 that is a **limit**, and the family of fixes is named rather than
waved at: a finer ask is refuted by the tangent refusal at `0.01`, a coarser one
by the refusals at `1e16` and `f64::MAX / 4`, and a rescaling by the existing
`the_same_pair_is_walked_as_soon_as_its_thin_pipe_spans_a_grid_step`, where
nothing changes but the thin radius and at `1e-7` the same call walks. Here the
oracle checking nothing is correct. Only the seconds it takes to say so are a
gap, and that half is already pinned by
`a_crossing_thinner_than_one_step_of_the_grid_crawls_to_the_point_cap`.

> **2026-10-10 note (Q1598): the seconds are gone.** The thread through the
> huge pipe no longer crawls to the point cap. It is refused at its first step,
> `CannotFollowACrossingWhereItsSurfacesAreTangent` while bounding how far off
> a point sits, in hundredths of a second, and
> `a_crossing_thinner_than_one_step_of_the_grid_is_refused_at_its_first_step`
> replaces the pin named above. It is still a limit, for the reason given here.
> §347.9's Q1598 note says what changed in the march.

### 125.4 The seeded `5dddf2a9` stays, and "it asserts nothing" is false

It is in the corpus with a walked-point oracle that checks nothing, which looks
like a seed asserting nothing. It is not. Replayed it asserts that `intersect`
names the pair `Curves` rather than refusing, that naming it the other way round
agrees, and that writing it at another power of two does not change it. What it
does not assert is that some point sits on both pipes — which is not an
assertion anyone can make about this pair, because no such point exists.

It is also the only corpus member driving both the tangent refusal at a fine ask
and the point cap at a coarse one, and §91 says an artifact not in the corpus is
lost.

### 125.5 The record and the tree disagree about `7e62ebe6`, and this section does not explain it

§108.4 recorded all four of its slow units at `Asked { points: 0, … }` against
`a2a93b9a` on 2026-09-14. `7e62ebe6` measures `Asked { points: 10, … }` on
`19c6f3d1` today. So the pair started walking at some point in between, and the
10-against-0 contrast this section is about did not exist when §108.4 was
written.

**Which commit moved it has not been established**, and nothing here should be
read as naming one. It is recorded because a later reader comparing the two
sections will otherwise take one of them for a mistake.

**Established since, in §125.7:** `1f1da0bf`, on 2026-09-17, by bisect with the
parent measured by hand. Read that rather than guessing from this paragraph —
it also rules out the obvious suspect, and it changes what §125.2's gap is
standing on.

### 125.6 What is not done, and where it already lived

The fix is the marcher's step ceiling — `MOST_A_STEP_MAY_GROW_TO` in `march.rs`
needs a bound that does not come from the caller's ask. That gap was already
named, in the module doc of
`what_makes_a_huge_pipe_against_a_thin_one_slow_to_walk.rs`, and **nowhere in
this file**: `just decisions-find 'step ceiling'` and `'MOST_A_STEP_MAY_GROW_TO'`
both came back empty before this section. A gap recorded only in the test that
demonstrates it is a gap no roadmap or plan can see, which is the same failure
§91 records for an artifact left outside the corpus.

`1bd43cf4`, `d8b562bb` and `3a4a8231` were **not** seeded into the corpus. They
would cost a gate replay roughly 7 s, 7 s and 5 s each, and the tests added here
pin what they prove from the decoder's own `rebuild:` number lists, so the
inputs are not lost in §91's sense.

Commands run to reach this: `git log --oneline -1`; `git merge --ff-only main`;
`git merge --no-edit d1e573a4`; `git merge --no-edit
lane-points-oracle-checks-nothing`; `mise exec -- cargo build -q -p
rcad-fuzz-explain` then `./target/debug/rcad-fuzz-explain
where_two_surfaces_cross <each of the five inputs>`; `mise exec -- just decision
108.4`; `mise exec -- just decisions-find 'step ceiling'`,
`'MOST_A_STEP_MAY_GROW_TO'`; `git log --all --oneline -S'## 125. ' -- docs/decisions.md`;
`mise exec -- just t a_hair_of_tilt_does_not_stop_these_pipes_being_walked_at_an_ordinary_ask`
with `f64::MAX / 4.0` added to its ask list, which failed
`asked for 4.4942328371557893e307 of room and got TooManyPointsAlongOneCrossing { most: 1048576 }`
in 3.128 s and was reverted; `mise exec -- just gate`.

### 125.7 It was `1f1da0bf`, and the coarse ask is already being closed from one side, measured 2026-09-18

§125.5 left "which commit moved it" open and refused to name one. It is
`1f1da0bf fix(geom): the settling gave up on a subtraction, not on the
geometry`, 2026-09-17, and this is a measurement rather than a suspicion.

`git bisect run` over `a2a93b9a..19c6f3d1` — 121 commits, seven steps, one
input, the decision being `points: 0` against anything larger. It named
`1f1da0bf`. A bisect never runs the parent of its own answer, so the parent was
run by hand: `3b7fd249` gives `Asked { points: 0, orders: 1, scalings: 2 }` and
`1f1da0bf` gives `Asked { points: 10, … }` on the same bytes.

Three things that could have faked that, each ruled out rather than argued away:

- **The decoder.** The `case:` line the tool prints is byte-identical at both
  ends of the whole range — the same two pipes, radii `167774208.0` and `1.375`.
  The bytes still mean what they meant.
- **The harness's ask.** The commit touches exactly one file,
  `crates/rcad-geom/src/march.rs`, +212/−50. `tests/fuzzing/mod.rs` and its
  `f64::MAX / 4.0` are not in it.
- **The rest of the commit.** It carries three changes and the one that does
  this is a single line. On the tree as it stands, replacing
  `step = step.sqrt().min(step * 0.5)` with `step = step * 0.5` takes the pair
  back to `points: 0`; putting it back restores the ten.

`6db8b90c`, the correctly-rounded normalize, is **not** it. It is a *descendant*
of `1f1da0bf`, 84 commits later on the same day, so it lands after the pair had
already started walking. It was the obvious suspect, which is the whole reason it
was measured instead of written down (§5 of `CLAUDE.md`).

**The coarse ask §125.2 calls a gap is already being closed from one side, and
on purpose.** The doc comment `1f1da0bf` added says so in its own words — "a
coarse walk asking `f64::MAX / 4` of room could stop on a pair it crosses", and
"a refused trial is retried at the square root of the ask … fifty halvings of
`2.25e307`" reach only `9.3e43`. That ask is this ask. A commit whose subject
line is a subtraction in `settled` moved this threshold on the way past.

It moved **two** of §108.4's four and not one. The six corpus slow units, across
the commit:

| input | `3b7fd249` | `1f1da0bf` |
| --- | --- | --- |
| `262c17e4` | 0 | 10 |
| `7e62ebe6` | 0 | 10 |
| `5dddf2a9` | 0 | 0 |
| `b9068e9e` | 0 | 0 |
| `5a19686d` | 18 | 18 |
| `6a164a91` | 10 | 10 |

`262c17e4` and `7e62ebe6` are the pair §108.4 calls one case drawn at two places
along its own axis, so the two that moved are the two that are the same shape.
`5dddf2a9` still refuses, which §125.3 calls a limit and it still is.

**§125.6's fix is not superseded.** The retry shortens a trial that was
*refused*; `MOST_A_STEP_MAY_GROW_TO` bounds how far a step may *grow*. Two knobs
on one walk, and only the second is the ceiling that does not come from the
caller's ask — the tilted `1bd43cf4` and `d8b562bb` still refuse, which is
§125.2's gap and is untouched by this. What has changed is the reason to write
the ceiling down: it moves under work aimed at something else, and nothing failed
when it did.

Nothing failed because the claim was prose. The module doc of
`what_makes_a_huge_pipe_against_a_thin_one_slow_to_walk.rs` said all four come
back `points: 0` and put `262c17e4`'s two coarse walks at 7.30 s; both had been
false since 2026-09-17 and the gate was green throughout, because no test
asserted either. `0fbff52c` — §16.10, a claim can go stale and the audit above
will not catch it — is *inside* the range bisected here. The doc now says which
commit falsified it, and
`a_huge_pipe_and_a_thin_one_are_walked_even_when_asked_for_all_the_room_there_is`
pins both pairs at `f64::MAX / 4.0`, so the next move fails a test rather than a
reading. It fails for the right reason too: with the retry put back to halving it
refuses `CannotFollowACrossingWhereItsSurfacesAreTangent`, and with it in place
both pairs walk both runs in 7 ms together.

The gap is on `docs/plan.md` under 6.3 now, which is the roadmap item that owns
`march.rs`. §125.6 noted it lived only in a test's module doc; a line of its own
is not available, because `check-plan` derives the item list from the roadmap and
fails when the two disagree.

Commands run to reach this: `mise exec -- cargo build --release -p
rcad-fuzz-explain`; `./target/release/rcad-fuzz-explain where_two_surfaces_cross
<the input>` at `e4e2bab2`, `a2a93b9a`, `3b7fd249` and `1f1da0bf`; `git bisect
start 19c6f3d1 a2a93b9a` then `git bisect run` on a script that builds, decodes
and exits 0 on `points: 0`; `git merge-base --is-ancestor 1f1da0bf 6db8b90c` and
`git rev-list --count 1f1da0bf..6db8b90c`; the one-line edit to `march.rs:570`
and back, twice, once to measure and once to check the new test fails without it;
`mise exec -- just t
a_huge_pipe_and_a_thin_one_are_walked_even_when_asked_for_all_the_room_there_is`;
`mise exec -- just gate`.

> **2026-10-10 note (Q1598): the tilted `1bd43cf4` and `d8b562bb` walk.** The
> sentence above, that they still refuse, and §125.2's gap no longer hold. At
> the `f64::MAX / 4.0` sizing ask the march kept a first trial that had settled
> back onto its start, and crawled from there to the point cap. Q1598 tries
> such a trial again shorter instead (§347.9's Q1598 note). Both runs of both
> pairs now walk, and every point they draw is checked against both pipes:
> `the_pipe_tilted_by_a_hundred_and_twenty_two_orders_has_every_walked_point_checked`
> and its sibling at a hundred and twenty-nine orders. Q23, §125.6's step
> ceiling, stays open. `MOST_A_STEP_MAY_GROW_TO` still comes from the caller's
> ask, and these two pairs no longer show that it matters.

