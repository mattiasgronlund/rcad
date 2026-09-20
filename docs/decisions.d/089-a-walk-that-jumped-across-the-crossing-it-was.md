## 89. A walk that jumped across the crossing it was following, recorded 2026-09-11

Nineteen seeds of the `where_two_surfaces_cross` corpus took **99.8% of the time
the whole corpus cost** — 65.7 seconds of 65.8, over 997 seeds that decode. That
is also why a night's fuzz throughput fell from 262 runs a second to 31: the
corpus had grown into a cliff, and a fuzzer spending its night on nineteen inputs
is not exploring.

All nineteen were cylinder against cylinder, so all nineteen went to the quartic
marcher rather than to a closed form. Every one of them ended in
`TooManyPointsAlongOneCrossing { most: 1048576 }`, having genuinely walked a
million points to get there.

### 89.1 It was not the walk the guess said it was

The handoff that carried this over guessed "a walk running to
`MOST_POINTS_ALONG_ONE_CROSSING`", and the cap is indeed where it ends, but not
because a million points were wanted. The points the oracle actually counted for
those seeds were **0, 3, 5 and 6**. `intersect` was not involved either: it
answers in 0.02 ms on all of them.

Worth recording because the shape recurs — the cap is where the cost *surfaces*,
and it was nowhere near where the cost was *made*.

### 89.2 A budget bigger than the crossing decides nothing

`march` starts with `step = room`, the caller's deflection budget, and
`one_step` accepts a trial when `sagitta <= room * SAGITTA_MAY_TAKE`. Hand over a
budget larger than the whole crossing and that test rejects nothing whatever, so
the first step — `here + tangent * 2.2e307` — is accepted wherever the settling
happens to put it.

Traced on one seed, it put it **4.34e45 away**. From there the walk fell into an
orbit that stayed `1.0066e8` from `start` while a step covered `5.6e5`, so the
loop-closing test `back_to_the_start.length() <= taken` could never fire again.
`travelled` was frozen at `8.674388521469274e45` from step 5 to step 1 000 000 —
adding `5.6e5` to it rounds to itself — so the `travelled > 3.0 * taken` half of
that test stopped carrying information too.

**Nobody asks for a budget like that by accident.** The harness asks for it on
purpose: `every_point_of_every_crossing_is_on_both_surfaces` walks each run twice,
the first time at `f64::MAX / 4.0`, as the cheapest way to find out how big the
crossing is before asking for the real one. The idiom is reasonable and the
kernel has to survive it.

### 89.3 A gap and not a limit, and the sweep is what says so

§16.10 asks which it is. Swept across the budget, the same run walks in
microseconds everywhere sane — 3 points at `2^48`, 214 at `2^8`, 3234 at `2^0`.
Only above about `2^90` does it break, and there it is not a limit but a
*scatter*: `2^840` a million points, `2^860` two points, `2^900` a million again,
`2^1020` a refusal. Four outcomes to one question, one power of two apart.

An answer that jumps between four outcomes with the budget is not the arithmetic
running out. It is a step nothing constrained.

### 89.4 Why the acceptance oracle watched this for nights and said nothing

The fuzzer ran over these pairs for nights without calling them wrong, and the
reason is worth keeping. `every_point_of_every_crossing_is_on_both_surfaces` asks
whether each point is on both surfaces **to within the bound the walk published
for it** — and the jumped-across walk published `2.6e46`. Every point passed,
enormously.

That is not the oracle being weak: a walk that gives up its accuracy and says so
is doing what §24.1 asks. What nothing checked was whether the published number
was any use. The new suite,
`a_coarse_walk_stays_on_the_crossing_instead_of_jumping_across_it.rs`, asserts
the other half: the crossing of two pipes is drawn *on* both of them, so a bound
wider than the smaller pipe has not located a point on it. `2.6e46` against a
pipe `2^20` across.

And that same published bound is what poisoned the second walk. The harness
divides it by `POINT_ALONG_A_CROSSING_ROUNDOFF` to decide how fine to ask next,
so a bound of `2.6e46` asks the fine walk for `1.1e133` — absurd in the same way,
and the fine walk jumps too. The whole million-point bill follows from one
unconstrained first step.

### 89.5 The fix: straighter than it is long

`SAGITTA_MAY_TAKE_OF_ITS_CHORD`, a quarter. A straight piece is a piece of the
curve only if its sagitta is small against **its own length**, which is a
question the caller's budget cannot make vacuous. On a circular arc the sagitta
is `tan(θ/4)/2` of the chord, so the quarter-turn rule already there sits at
`0.2071` and a quarter leaves room over it for curves that are not circles. What
it rejects measured `0.44` to `0.59` — half a turn and worse.

**Halving a rejected trial does not work here, and that is the second half of the
fix.** While the ask is longer than the crossing, the settling lands the trial in
the same place however much the ask is halved: measured, a chord of `1.1003e39`
sat still over twenty-one halvings, and the trial that finally passed did so
because the quarter-turn test drifted into accepting it, not because anything
converged. So a trial that came back shorter than half of what it asked for is
retried from **the chord**, which is the one thing it did measure about the curve.

The two cases are kept apart rather than merged into a `min`, and this was
measured rather than chosen: the merged form also shortens steps in the ordinary
fine walk, and that moved the marcher's published bound from `0.577` of its own
enclosure to `0.871` — still a bound, but past the three quarters
`how_far_the_marcher_strays_from_a_crossing_that_is_a_circle` publishes, which is
a published bound and so §15.6's to defend. Splitting on whether the trial
overshot leaves that measurement at `0.577` exactly, at both scales.

### 89.6 What moved

The refusal rate moved **down**, and the cost with it.

| | before | after |
|---|---|---|
| the corpus, 997 seeds that decode | 66.6 s | **0.161 s** |
| seeds taking over a second | 19 | **0** |
| p99 · p100 | 3103.9 ms · 6997.7 ms | **3.06 ms · 7.60 ms** |
| points the oracle checked | 8596 | **13 152** |

Nothing anywhere refuses something it used to decide. The checked-in corpus row
went from 1993 points to 4207 and the soak's from 400 342 to 400 412, with orders
and scalings unmoved in both — which is what a fix to the walk and to nothing
else should look like.

**The seed, promoted 2026-09-13.**
`fuzz/corpus/where_two_surfaces_cross/slow-unit-6a164a91…` is the input behind
the p100 above, and it is checked in because until then this section published a
26 000× number with nothing anybody could re-run. Two crossed pipes, radius 2048
along x against radius `0.0152740478515625` along `[-1.07e-300, 0, 1]`: at
`27b3125`, the commit before this section landed, **5.930367367 s** producing
`Asked { points: 0 }`; at `71c7474`, **226.882 µs** producing ten points. Six
seconds to yield nothing, against a fifth of a millisecond to answer.

It was written on 2026-09-10 by the soak that found it, into a `fuzz/artifacts/`
directory `.gitignore` hides, and it sat there for three days as the only copy of
the one input that witnesses this fix — in a worktree that was unlocked and due
to be reclaimed. §85 and §88 each landed their seed with their fix and did not
name it, which cost two sessions an evening (§85.5). This section landed no seed
at all, which is the same gap one step earlier: a regression test checks that the
behaviour holds, and only the input says what the behaviour was.

Adding it moves the corpus row of
`every_pair_of_surfaces_the_fuzzer_draws_is_named_honestly` from 745 of 826 seeds
to 746 of 827, and its points from 4207 to 4217 with one more order and two more
scalings. Measured by removing the seed and re-running, so the delta is this file
and not drift: without it the row is exactly the 4207 · 544 · 1087 published
before.

### 89.7 The backstop that lost its consumer

The fix was agreed as two parts: this one, and a cheap detector for a walk
orbiting without closing, so that reaching the cap need not cost a million steps.
The second part is **not built**, because after the first there is nothing left
for it to catch.

Counted over the corpus and a two-million-draw soak — 1 225 146 pairs between
them — the marcher reaches its point cap **zero times**. The only two
`TooManyPointsAlongOneCrossing` refusals in that census are a pipe against a
plane, which is a conic: the closed form predicts its point count and refuses up
front in 0.00 ms, at `walk.rs`'s two `steps > MOST_POINTS_ALONG_ONE_CROSSING`
tests. That is exactly the cheap refusal the backstop was for, and the conic path
has had it all along.

§13 says a part with no named consumer is not built, so this is deferred rather
than written. **What is still true** is that a caller asking directly for a very
fine deflection — `2^-24` on the swept seed — genuinely needs more than a million
points, and pays 2.9 seconds to be told so. Only a prediction of the count would
make that cheap, which is the quartic marcher's missing equivalent of
`how_many_steps`, and it needs a size for the crossing that the coarse walk is
what currently goes and finds out.
---

