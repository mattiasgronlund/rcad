## 53. A plane, a cone and a coaxial pipe — the unit after four rows closed, recorded 2026-09-05

*Planning section. Written before any lane runs, against `main` at `97525a9`.*

**Every number below was re-derived from the tree and names the command that
produced it.** That is §51.6's rule, and §52.9 showed it has a hole — a number
can come from exactly the test that was named, and the test can be blind — so
§53.7 charge 3 covers the hole and §53.9 extends the rule to a spike.

### 53.1 The name, and what the unit is

`three_surfaces.rs` wants two flat surfaces. That row has been open for two units
— §51.7 opened it, §52.11 carried it unchanged — and it is the wall
`AFrustumAgainstABar` stops at, having been let through `intersect` by §50's work
only to stop one step further down.

Re-derived on this HEAD by

```text
cargo nextest run -p rcad-topo --test merged -E \
    'test(~how_often_a_fuse_with_a_cone_refuses::how_often_a_certified_fuse_with_a_cone_in_it_refuses)' --no-capture
```

> `a frustum against a bar: 96 fuses, 0 answered, 96 refused (100.0%):`
> `{"geometry: where three surfaces meet is only worked out when two of them are`
> `flat, and fewer than two of these are": 96}`

So the family is not merely at zero: **every one of its 96 refusals is this one
wall, by name**, which is what makes it a consumer for §13's purposes rather than
a wish.

**This unit is not the general one-flat solve.** `error.rs:571-576` classes that
as a **gap** and not a limit — the points are there and a real algebraic solver
reaches them — whose family of fixes is Dupont–Lazard–Lazard–Petitjean machinery
or a certified numerical solver with a separation bound of its own, and both are
a milestone rather than a lane (§31.2).

This is the narrow reduction that avoids it: **two curved surfaces crossing in a
conic put that conic in a plane, so `{P, S1, S2}` becomes `{P, Q, S1}`, which the
existing two-flat road already solves.**

**It is a third road of a shape the file already has, not a new mechanism.**
`error.rs:566-569` names the two that run today — two pipes the same way with a
cap plane, which is a union of two overlapping parallel bars; and a ball and a bar
both resting on a block's floor — and `meet_of_three_surfaces`'s own doc refuses
only *"when neither road back to two of them runs"*. A cone and a coaxial pipe is
the third. That matters for where A puts it and for how much of `three_surfaces.rs`
it is entitled to disturb: **the answer is a road beside `where_a_plane_and_two_quadrics_of_one_bend_meet`
(`three_surfaces.rs:591`), not a new solver.**

### 53.2 Six corrections, before any lane reads them

They exist because §51.6's countermeasure keeps being walked through, and one of
them is §52.14's shape found in the wild before a lane ran.

1. **`arrange.rs:1232` does not panic, and the line is 1232.** An earlier reading
   in planning said 1233 and said it could panic. `u8::try_from(which)` fails only
   above 255 and `which` reaches 3. The `expect("three surfaces meet at most
   twice")` **message** is false; the code is safe. A false message is D's
   deliverable, not a bug.

2. **`MeetOfThree::At` already promises four points, and it lives in the spine
   lane's own file.** `three_surfaces.rs:88`. Its doc already reads *"one, two,
   three or four"* — a straight line meets a ring four times, which is what makes
   a torus not a quadric. **What the reduction breaks is not the count. It is the
   order clause**, *"running along the line the two planes cross in"*, which
   cannot survive two **parallel** planes because there is no single line. Planning
   had this as an `arrange.rs` job beside the spine; it is inside the spine's file
   and cannot be concurrent with it.

3. **§51.3's "76 bytes" is right for two of the three pairs it names and wrong for
   the third**, and the conclusion it supports is untouched. `BYTES_PER_NUMBER = 5`
   (`crates/rcad-geom/tests/fuzzing/mod.rs:59`) and one kinds byte, against
   `Cone { axis, slope }` = 7 numbers and `Torus { axis, ring_radius, tube_radius }`
   = 8:

   | pair | bytes |
   |---|---|
   | a torus against a cone | `1 + 40 + 35` = **76** |
   | a torus against a pipe | `1 + 40 + 35` = **76** |
   | a torus against another torus | `1 + 40 + 40` = **81** |

   The sentence at `every_pair_of_surfaces_the_fuzzer_draws_is_named_honestly.rs:56`
   says 76 for all three. **This is §52.14's specimen exactly**: a reason sound
   about two thirds of its subject, which therefore reads as more thorough than a
   bare assertion and deters the checking a bare one would invite. The conclusion
   survives in all three cases, because the longest seed checked in is 71 —
   re-derived by `find fuzz/corpus/where_two_surfaces_cross -type f -printf '%s\n'
   | sort -n | uniq -c`: **824 seeds, longest 71, 280 of them exactly 71, shortest
   1**. 71 is also `MOST_BYTES_A_PAIR_READS` (`fuzzing/mod.rs:64`), so the corpus
   number and the decoder's cap agree because the fuzzer was told the cap — it is
   not independent evidence, and §53 does not use it as any.

4. **Parabolas and hyperbola branches are moot for the reduction.** They are
   produced only inside `where_a_plane_cuts_a_cone`, so a *curved* pair never
   crosses in one.

5. **No `walk.rs` type needs to be made public.** The identity touches neither
   `Conic` nor the normalized `up`.

6. **`cuts.rs` and `three_surfaces.rs` are in different crates.**
   `crates/rcad-geom/src/three_surfaces.rs` is 2110 lines;
   `crates/rcad-topo/src/fuse/cuts.rs` is 495. §51.7 and §52.11 name them in one
   breath as *the unaudited pair*, which reads as one lane's worth of one crate's
   work and is two crates. §53.4 splits the audit on that line.

### 53.3 The sentence the unit earns

> **Where three surfaces meet is worked out when a cone and a pipe are proved
> coaxial and one plane cuts them.**

Falsifiable two ways, both measured rather than argued:

- `where_one_plane_and_two_curved_surfaces_meet` (`three_surfaces.rs:543`) has a
  road for the coaxial cone-and-pipe pair, placed **after**
  `what_a_plane_that_grazes_one_of_them_leaves` (`three_surfaces.rs:934`) so that
  no answer that road gives today moves; and
- `MeetOfThree::At`'s order clause is replaced by one that survives two parallel
  planes.

**The sentence does not promise 96 of 96 answered, and neither does any lane.**
The reduction's domain is the triple `{plane, cone, pipe}`. A triple of the 96
holding no plane at all, or two curved surfaces that are not a coaxial cone and
pipe, is outside it and stays refused. What is promised is that **the wall moves
and the pin is re-derived to whatever the measurement then is** — which is what
the pin's own text already demands: *"re-derive the number over this whole family
and pin the measured one — do not relax it to a floor and do not guess it
(§19.1)"*. Promising the 96 here would be a number invented before its
measurement, which is the thing four consecutive units have been caught doing.

### 53.4 The shape of the waves, and what each lane may write

| step | one deliverable | may write |
|---|---|---|
| **A the spine** | the reduction: `the_two_flat_surfaces_in_a_fixed_order`, the new road, and `MeetOfThree::At`'s replaced order clause | `crates/rcad-geom/src/three_surfaces.rs`, `crates/rcad-geom/tests/where_a_plane_a_cone_and_a_coaxial_pipe_meet.rs` (new), `crates/rcad-geom/tests/where_a_coaxial_cone_and_pipe_cross.rs` |
| **B the geom audit** | every site in `three_surfaces.rs` where a bare `f64` decides something, reported | **nothing. Read-only** |
| **C the fuse audit** | the same over `crates/rcad-topo/src/fuse/` — `cuts.rs`, `bake.rs`, `arrange.rs` | **nothing. Read-only** |
| **D the contract** | `arrange.rs:1232`'s false `expect` message | `crates/rcad-topo/src/fuse/arrange.rs` |
| **E the pin** | `AFrustumAgainstABar` re-derived, not relaxed | `crates/rcad-topo/tests/how_often_a_fuse_with_a_cone_refuses.rs` |
| **G1 the numbers gate** | charges 1 and 3 | its report |
| **G2 the claims gate** | charge 2 | its report |

**A, B and C run concurrently.** B and C have **empty write sets**, which is
§52.12's own finding put to use — *a lane with an empty write set costs nothing to
run concurrently* — and it is what makes wave one three lanes rather than two,
which is what CLAUDE.md's floor of three would otherwise have caught here.

**Splitting the audit by crate is not bookkeeping.** §51.8 and §52.12 both record
that two gates beat one *and that the split matters*, each finding what the other
did not. The same argument applies to an audit, and here the split is free: the
two files are in two crates, `cuts.rs`'s neighbours are `bake.rs` and `arrange.rs`
rather than anything in `rcad-geom`, and a lane that has to hold two crates' worth
of call graph in its head is the lane that runs past forty-five minutes. **C reads
`arrange.rs` and D writes it**, which is not a conflict because D is wave two, but
it is named here so that nobody makes them concurrent later.

**D and E are wave two, behind A.** Neither can be written before A has settled
what the order actually is: D's message and E's number both describe A's result.
D is one string and is not a lane on its own — it is dispatched with whichever
audit handback lands beside it.

**Wave two's real size is not known now, and that is deliberate.** B and C's
handbacks become wave-two lanes, named at dispatch. §51.8 and §52.12 both record
that a handback became the next lane's whole deliverable — four times and three
times respectively — so reserving the room is a plan and guessing the contents is
not.

### 53.5 The rules whose pass and fail are written before the lane runs

Fourth unit running (§47.6, §49.5, §51.8, §52.12).

- **A fails** if it makes any `walk.rs` type public; if it changes any answer
  `what_a_plane_that_grazes_one_of_them_leaves` (`three_surfaces.rs:934`) gives
  today; if `MeetOfThree::At`'s replacement order clause is anything weaker than
  *a fixed order depending only on which three surfaces they are* — the doc is in
  A's own file, so leaving the old sentence standing is a fail and not an
  oversight; if it states the domain wider than **a cone and a pipe proved
  coaxial**; or if the new road introduces **any division, or any comparison on a
  bare `f64`**.

  That last one is the spine's own claim turned into its fail condition. The whole
  reduction rests on the pencil identity being exact — products and sums of stored
  numbers, with `t` never computed — so a division appearing on the road means the
  claim in §53.9 is false. It is also mechanically checkable, and it is exactly
  the class §51.2's volume sign and §51.7's audit row are about: **A must not
  quietly become the thing B and C are looking for.**

- **B and C fail** if either writes anything under `crates/`; if a reported site
  is given without its file and line; or if a negative claim — and an audit's
  output is mostly *"this one is fine"* — is stated without citing **where the
  refusal actually happens**, per §52.14. A paragraph is not a demonstration.

  **The threshold, and it governs whether §53 keeps its spine.** The count is
  **wrong answers only, not missing arguments.** A site with no written argument
  means *write the argument, or write the test* — the spine survives. A site that
  gives a **wrong answer** becomes the next lane's whole deliverable. **Two wrong
  answers and §53 changes spine**, because at two the unaudited pair is the unit
  and the reduction is not.

- **D fails** if the replacement message states a bound the code does not enforce.
  The current one is false in exactly that way, and replacing one false message
  with another is worse than leaving it, per §52.14's direction-of-error rule.

- **E fails** if it relaxes the pin to a floor, or pins a guessed number, or
  reports the family's rate without re-deriving it over the whole family.

**At forty-five minutes a lane commits what is done, writes what is left, and
hands back** (CLAUDE.md). A's handback point is named: the exact-planes helper and
its tests land as one commit, the road that calls it as a second, so *what is
left* is never a tree that does not compile.

### 53.6 What stays out

| out | why |
|---|---|
| **the general one-flat solve** | a milestone by `error.rs`'s own doc. §53.1 |
| **three curved surfaces** | no plane, no reduction, and no consumer asking |
| **the fuzz decoder widening** | §51.3, corrected by §53.2 item 3. Wants the corpus re-minimised with it, which is a lane of its own |
| **`ResultIsNotManifold` with `count` other than 1** | unchanged since §51.7 |
| **`Conic::size` grows with where the conic sits** | §52.10's runaway term. Recorded as a row with its numbers — crosses the floor at a lean of about 10.8, `1.85e8×` at `2e9` — and **offered to whoever takes `walk.rs` next**. It is a §19 nerve and it is not this spine |
| **§52.11's `Record::an_operation_settled`** | out, **with a written condition**: if B, C or any wave-two lane lands in `rcad-predicates` for its own reasons, this is that lane's second deliverable. It adds no `Cause` variant and so pays none of §25.8's renumbering cost |

**§51.7's other rows are not restated here.** The writer, fillets, the torus,
cone against cone, the slid pipe origin, the split provenance — all unchanged, all
already written down once. §51.10 argues that a copy restated in the copier's own
words can be wrong and internally consistent, and then it passes; §53 cites
§51.7 and §52.11 rather than reproducing them.

### 53.7 The gates' three charges, split across two gates

1. **Numbers.** Re-derive every load-bearing number from the tree, independently
   of the lane that reported it — including the ones in this section.
2. **Claims.** `claim-audit` over every negative claim in the two audited files,
   and over B's and C's own reports. Not instead of a code review.
3. **Mutate what each load-bearing test claims to watch, and confirm it fails.**
   Take every test whose failure would be the sole evidence for a published bound,
   and mutate the thing it claims to watch — not the code near it.

**Charge 3 is how §53 scores §52.13's prediction**, and it exists because §52
walked straight through §51.6's *name the command* rule and the rule did not help.
The cone bound's §15.6 search reported `0.39447768211554846` after a fix, against
`...857` before — one ulp, under a change that cut a frustum's mesh from 56 880
triangles to 14 550. The triangle family sat exactly on the subspace where the old
and new formulas agree. **The number came from precisely the test that was named,
and the test was blind.** A search whose number does not change under a fix that
changes what it computes is not reassurance; it is the first thing to distrust.

Charges 1 and 3 go to G1, charge 2 to G2. Split by kind rather than by lane, which
is §51.8's split; §52.12 used a per-lane split and found the same regression twice,
so both splits work and neither is the point — **two gates is the point.**

### 53.8 The machinery this unit was going to change, and does not

**Nothing. §53 dispatches against the tree as it stands.** §50.8 changed six
things before its spike ran and each earned its place. This unit's one candidate
did not, and why it did not is worth more than the guard would have been.

**A guard against a lane staged-but-uncommitted.** §51.8 asked for it after a
`sonnet` lane staged its whole deliverable and never committed it, and reasoned
*"had it been killed, all of it was lost"*.

**The shape it was asked for cannot be built, and finding that out cost one
question.** Planning settled on *a blocking `SubagentStop` hook*. Against the
published hooks reference:

- **`SubagentStop` cannot block.** It runs **after** the subagent has already
  completed, so exit code 2 has nothing left to stop. It is a post-action hook and
  it runs for side effects only.
- **It fires** on normal completion, on failure, and on a user-initiated stop.
- **Whether it fires on a forced kill is not documented either way.** The
  documented *"does not fire on user interrupts"* is about `Stop`, the
  session-level event, and does not carry to `SubagentStop`.

**Two mechanism claims were made in planning this unit and neither survived
opening the reference**, which is why this reads as a finding and not as a
paragraph of setup:

| the claim | what is actually established |
|---|---|
| §51.8: the hook would have saved work *"had it been killed"* | it cannot block and it runs after the lane is done, so it saves nothing on any exit; and the kill case is undocumented rather than covered |
| §53's own first draft: *"`SubagentStop` does not fire when a subagent is killed"* | **not established.** The docs are silent. The sentence was inherited from `Stop`'s documented behaviour and restated about a different event |

The second one is this section's own, and it is the shape §53.10 predicts —
**a claim carried by nothing, written into the section that argues against claims
carried by nothing.** It was caught because the sentence was about to become
load-bearing and therefore acquired a reader with a reason to be adversarial
(§51.10), which is the third time that has paid and the first time it has paid
against rcad's *planning* rather than against its code. **A correct fact about
`Stop` restated about `SubagentStop`** is §51.10's *at least four steps* restated
as *at least four points*, one repository over.

**What the guard is for, restated on what is left.** Not loss prevention — no
hook event reaches a killed process. **Handback honesty**: a lane must not report
*done* with its deliverable staged and uncommitted, so the merge step cannot miss
it.

**And it is not built.** Three shapes remain buildable — a report-only
`SubagentStop` hook, one that commits the staged work itself, or a `just` recipe
at the **merge gate**, which is the only one a test can drive — and none of them
is bought. §51.8 wanted a hook because *recording a rule had been tried*; that
argument was made about the heredoc ban, where thirteen lanes produced three
offenders. **The staged-and-never-committed failure has happened once, cost
nothing, and the guard now available is weaker than the one that was asked for.**

That is §13's rule pointed at machinery rather than at a feature: a part with no
named consumer is not built, and *one lane, once, no loss* is not a consumer.
The row stays open and this section is where the next unit finds the mechanism
already worked out, so the question does not have to be re-derived to be
re-decided.

**Both false claims here were found by opening a reference nobody had opened**,
at a cost of one question. The lesson is not about hooks: **the machinery section
is where a unit is likeliest to spend effort on something whose justification
nobody checked**, because machinery is the part that gets no test and no gate
charge. §53.7's three charges cover lanes and tests; nothing in this record has
ever scored a claim about the harness.

### 53.9 The spike passed, and every claim it makes names the test that will carry it

Branch `worktree-agent-a48dec4c50ee6f0fc`, commit `f743ea4`, 442 lines added to
`three_surfaces.rs` in a `#[cfg(test)] mod spike_two_circles_are_two_planes`.
**Probe code is never merged.**

The identity, which is the whole spine: for the pipe's stored direction `w`, its
stored radius `r`, and the cone's apex and slope `s`,

> `t²·Q_cone − Q_pipe = (r − s(x·w))(r + s(x·w))`

so the two flat surfaces are `s·(w·p) = s·(w·apex) ± r`. **A pencil identity, not
the `r/slope` scaling route.** Every number in it is a product or a sum of stored
ones; nothing divides, and `t` is never computed.
`what_a_stored_plane_and_an_exact_one_share` (`three_surfaces.rs:816`) admits the
two planes **unchanged**.

**Q16's rule, applied to §53 itself.** A spike's code is deleted and its
conclusions are not, so each conclusion below names the merged test that will
carry it. A row with no test is written as prose and is not leaned on.

| what the spike established | the merged test that carries it |
|---|---|
| the two planes are exact and need no division | `the_reduction_finds_all_four_corners_with_no_width_and_no_log` — asserts an **empty** provenance log, so a certified answer that logged anything fails it |
| the merge yields **four** points, not two | the same test |
| the order is invariant under the pipe's stored direction | `the_pipes_stored_direction_flips_the_order_the_corners_come_out_in`, whose spike form demonstrates the flip **and** the fix; the merged form asserts the invariance |
| the order is invariant under the cone's stored direction | `the_cones_stored_direction_moves_the_order_as_well` |
| no answer the grazing road gives today moves | `the_grazing_road_and_the_reduction_agree_where_both_answer` |
| the scaling worry does not exist — `Axis::new` scales the direction to length one before storing, so two pipes down one line hold the same three numbers up to sign | `a_pipe_built_from_a_longer_direction_stores_the_scaled_one`. **Not in the planning handoff's list of the spike's findings**, and it is the one that closes off a whole family of spellings |
| the triple is refused today though all four corners are whole numbers | `today_the_triple_is_refused_though_all_four_corners_are_whole_numbers` — the failing-first test CLAUDE.md rule 2 asks for |
| a turned axis, where nothing lands on a whole number, is answered | `a_turned_axis_is_answered_by_the_reduction_and_refused_by_the_road` |
| **the domain is one pair of surface kinds — a cone and a pipe proved coaxial, not a family** | **no test carries this.** It is a scope statement and it is written as prose, which is why it is A's fail condition (§53.5) rather than an assertion |

**Ordering was a no as built, and the fix is demonstrated rather than argued.**
The same bore stored as `[0,0,1]` and as `[0,0,-1]` gave the same corners in two
orders. Pinning the shared normal's sign by its first exactly-nonzero component,
and sorting the two offsets by an exact `minus().sign()`, makes them identical.
That canonicalization is the load-bearing part of A's first commit, and §53.7
charge 3 re-runs it rather than reading about it.

**What the audit's site count is, and is not.** A recon pass in planning reported
**14** sites where a bare-`f64` result feeds a comparison, sign test, division or
`Decision` — 10 in `three_surfaces.rs`, 4 in `cuts.rs` — against **4** hits for
the comparison-operator grep §51.7 already warns is not evidence. **Neither number
is quoted here as a bound.** A naive `[<>]=?` sweep on this HEAD returns 159 lines
in `three_surfaces.rs` and 23 in `cuts.rs`, most of them generic brackets, so the
4, the 14 and the 159 are three different questions and none of them has its
criterion written down. **B's and C's first deliverable is a count with its
criterion stated**, and §51.7's instruction stands unchanged: follow every
`f64`-returning call, and — §52's addition, found twice behind a **bound** rather
than a comparison — every place a sum is collapsed to a max.

### 53.10 The prediction for the next unit

§51.6 predicted an inherited claim and fired. §52.13 predicted a blind test and is
scored by §53.7 charge 3. Neither reaches the thing §53 is actually built on.

**Two consecutive units have now been planned around a spike whose code is never
merged**, and §52.8 already carries two false spike sentences — *"neither probe
refuses"*, when one refused in 8 of 20 cells; *"eight boundary sides"*, when there
were sixteen and they were not what the spike called them. A spike's evidence is
deleted at the moment its conclusions become load-bearing.

> **The next unit's largest error will be in a claim its spike made that no merged
> test carries.** Score it by taking every sentence in the planning section
> sourced to the spike and naming the merged test that would fail if it were
> false. A sentence with no such test is the specimen.

The countermeasure is cheap, specific, and already applied to this section:
**a planning section citing a spike names the merged test that will carry the
claim, or writes the claim as prose.** §53.9 is that table, and its last row —
the domain statement, carried by nothing — is the section declaring its own
specimen in advance rather than waiting to be caught with it.

That row is also why the earlier candidate for this prediction was withdrawn.
*"The audit's worst site will be in neither audited file"* is nearly checkable in
advance from §53.4's own brief, which already orders B and C to follow every
`f64`-returning call across file boundaries. A prediction its own plan half
guarantees is not one.

