## 68. Sixteen answers only the policy that built them calls valid, recorded 2026-09-07

Sixteen halves out of a 9 975-input `two_solids_fused_by_a_stranger` corpus fuse
to `Ok` and then fail `Model::validate` under `Policy::certified()`, always with
`Invalid::DynamicRangeTooWide`. This section is what the sixteen turned out to
be, and why the answer is a sentence in a doc comment and a test rather than a
new refusal.

The short version: **all sixteen fused under `Policy::tolerant`, all sixteen
validate under that same tolerant policy, and it is the fuse and not the caller
that made the span wide.** Nothing contradicted itself and nothing was
corrupted. What the sixteen are is the shadow of a sentence nobody had written
down: *"valid" is a property of a model and a policy, not of a model on its
own.*

  65.1    The measurement, all sixteen                            48 lines
  65.2    The exit check is live, and the policy stood it down     31 lines
  65.3    Why not at the door                                      18 lines
  65.4    Why not a refusal past the point the tag stops meaning anything   58 lines
  65.5    Atomicity was never at risk here                         16 lines
  65.6    What was written instead, and what is left open          36 lines

### 68.1 The measurement, all sixteen

For each corpus entry the fuzz target's own decode was replayed, the model's
span measured before the fuse and again after, and the model swept a second time
under `Policy::certified()`. Span is the model's largest located coordinate over
its shortest open-edge chord — the two numbers `Model::check_dynamic_range`
measures, without the radii, because every face in all sixteen is a plane.

| # | corpus entry | half | rule | span in | span out |
|---|---|---|---|---|---|
| 1 | `05a2c9cf` | second | Difference | `1.5` | `inf` |
| 2 | `1c2757f6` | first | Difference | `1.5` | `inf` |
| 3 | `24ad8af5` | second | Difference | `1.5` | `inf` |
| 4 | `2fb467fc` | first | **Union** | `1.0` | `1.8328898507823975e252` |
| 5 | `310b77d5` | first | Difference | `1.5` | `inf` |
| 6 | `3c405544` | second | Difference | `1.5000053564996183` | `3.2766741885823735e246` |
| 7 | `3c7a0cd6` | second | Difference | `1.5` | `inf` |
| 8 | `3c7c9338` | first | Difference | `1.5` | `7.308985508549999e289` |
| 9 | `75d25a59` | second | Difference | `1.5` | `inf` |
| 10 | `7e19c77c` | first | Difference | `1.5` | `4.50920175892576e111` |
| 11 | `875b4288` | first | Difference | `1.5` | `inf` |
| 12 | `909f5d0c` | first | Difference | `1.5` | `inf` |
| 13 | `98d5bb7a` | second | Difference | `1.5` | `inf` |
| 14 | `bd34edca` | second | Difference | `1.5` | `4.323455642275676e17` |
| 15 | `f91807c9` | first | Difference | `1.5000000000042746` | `8.426686567238631e307` |
| 16 | `fb4102cd` | second | Difference | `1.5` | `inf` |

Fifteen `Difference` and one `Union`. Every one of the fifteen has
`split == solids.len()`, so `tools` is empty and `A \ nothing` is the union of
both argument solids — the sixteen are all unions, and thirteen of them are two
ten-millimetre blocks.

**Every input span is between `1.0` and `1.5000053564996183`**, three orders
inside `DynamicRange::LARGEST_SUPPORTED_RATIO`. **Every output span is at least
`4.323455642275676e17`.** And the largest coordinate is *unchanged by the fuse
in all sixteen* — `15.0` in and `15.0` out for fifteen of them,
`2.3611832414348226e21` both ways for the Union. So the whole growth is at the
bottom: the fuse built a short edge, and every one of them is the sliver
`Assembly::check_dynamic_range` was written for. The narrowest,
`4.323455642275676e17`, is two ten-millimetre blocks one of which is tilted by
about `7e-18` radians, and its sliver is `3.469446951953614e-17` across.

### 68.2 The exit check is live, and the policy stood it down

The reading these sixteen invite is that the answer's range was never measured.
It was. `fuse/mod.rs` measures it at the exit, one line before the only mutation
of the whole operation:

```rust
if the_inputs_fit_in_f64(self, arguments, tools)? {
    assembly.check_dynamic_range(policy, log)?;
}
```

`the_inputs_fit_in_f64` is the guard that keeps a fuse from being blamed for a
range its caller handed it, and §68.1 says it passes for all sixteen — their
inputs span `1.5`. So `Assembly::check_dynamic_range` ran on all sixteen answers
and measured exactly the spans in the table. What it did with them is
`DynamicRange::check_under_policy`, and that is where the policy enters:
`Severity::Fail` returns `Error::DynamicRangeTooWide`, `Severity::Warn` pushes a
record and hands back an `Approximation` for the caller to hang on what it
builds. `Policy::tolerant` carries `Warn`.

`a_certified_fuse_of_the_very_same_pair_refuses_the_answer_it_would_have_built`
is the control: the identical pair of blocks under `Policy::certified()` comes
back `Err(Invalid::DynamicRangeTooWide { ratio: 4.323455642275676e17 })`, which
is the same number the tolerant fuse wrote a model for. One pair, two policies,
two outcomes, and neither is wrong.

### 68.3 Why not at the door

Because the measurement rules it out, and this is the whole reason the
measurement was the deliverable. Refusing a fuse whose *inputs* span too wide a
range would have to refuse a span of `1.5`, which is two ordinary blocks — every
fuse in the kernel. Nothing about any of the sixteen inputs is out of range, out
of the ordinary, or even slightly unusual: they are hundred-millimetre-scale
boxes.

That is also what `the_inputs_fit_in_f64`'s doc comment already says, in the
other direction — a frame at `2^40` with millimetre walls is out of range before
anything is fused, and refusing *that* here would report somebody else's dynamic
range as this fuse's bug. The sixteen are the mirror case and they land the same
way: the door is not where this belongs.

### 68.4 Why not a refusal past the point the tag stops meaning anything

This is the candidate that is actually tempting, and it needs an argument rather
than a shrug.

There is a principled line short of "tolerant never refuses". `Severity::Warn`
hands back `Approximation::new(self.coordinate_spacing(), Cause::TolerantFallback)`,
and `coordinate_spacing` is `largest_coordinate_magnitude * f64::EPSILON` with
its own doc saying **"no feature smaller than this can be told apart from the
one next to it"**. So once the span passes `1.0 / f64::EPSILON` — `4.5036e15` —
the tag a tolerant policy hands back is *wider than the feature it is describing*.
At entry 14 the tag is `15.0 * f64::EPSILON`, about `3.33e-15`, hung on a sliver
`3.469446951953614e-17` across: the tag says the edge might not exist. All
sixteen are past that line; the narrowest, `4.32e17`, is by a factor of 96.

Four reasons that line is not taken here, in order of weight.

**It contradicts a decision that is already taken and already tested.**
`a_fuse_that_builds_a_sliver.rs::the_edge_across_a_sliver_is_as_long_as_the_sliver_is`
fuses two blocks a `1e-200` hair apart under a tolerant policy and asserts the
fuse *writes* the answer, with the sliver edge exactly one hair long and not
`0.0`. Its span is `1.5e201`. Taking the line above turns that test's subject
into a refusal, which is a change to what `Policy::tolerant` means and not a bug
fix. §16.10 asks which of *limit* and *gap* a refusal would be; this one is
neither, because there is no refusal to justify — the answer exists, `f64` holds
every coordinate of it exactly, and the model round-trips.

**It belongs in `rcad-geom`, not in one operation.** The line is a property of
`DynamicRange::check_under_policy` — the place where a span meets a severity —
and putting it in `Assembly::check_dynamic_range` would give the fuse a
strictness `Model::validate` does not share, which is a second way for two parts
of the kernel to disagree about the same model. That file is owned by neither
this lane nor this section.

**It would move published refusal rates and nobody has measured what it costs.**
`how_often_a_fuse_with_a_cone_refuses.rs`, `how_often_a_curved_fuse_refuses.rs`
and `how_often_a_fuse_with_a_torus_refuses.rs` publish numbers that a new
tolerant refusal moves. A line drawn from sixteen fuzz inputs, without that
measurement, is a number chosen for the inputs that produced it.

**The failure it would prevent has not been shown to exist.** A caller who fuses
tolerantly and then tightens the policy gets a *refusal*, not a wrong answer. All
sixteen were swept a third time under the policy their own fuse ran under, and
**sixteen of sixteen pass**. Not one of them is a model that its own strictness
calls broken; the split is entirely between two strictnesses, and the stricter of
the two refuses rather than answering. There is no case here of a tolerant answer
being silently wrong at the strictness it was built for.

### 68.5 Atomicity was never at risk here

Worth recording because it is the first thing to check about any refusal in a
fuse, and the answer is clean. `Assembly::check_dynamic_range` is called at
`fuse/mod.rs:196` and `Assembly::commit` — the first and only mutation of the
whole operation — at `fuse/mod.rs:200`. So the refusal that exists lands four
lines before anything is written and the model is byte-identical, which is what
`Model::fuse`'s **Atomic** section promises.
`a_certified_fuse_of_the_very_same_pair_refuses_the_answer_it_would_have_built`
asserts the model still holds its two input solids after the refusal.

Any refusal added later at this same point inherits that for free. A refusal
that had to land *after* `commit` would not, and would be a different section.

### 68.6 What was written instead, and what is left open

Two things, and no change to what anything computes.

**`Model::fuse` now says what `Ok` promises.** It promised "the answer is valid";
it now says the answer passes `Model::validate` under `policy` and under no
other, that a tolerant fuse therefore writes solids a later certified caller is
told are broken, and that both readings are correct. The sentence that was
missing is the one this section opens with, and a caller reaching for
`Policy::tolerant` is where it is worth reading.

**`a_model_only_the_policy_that_built_it_calls_valid.rs` measures it.** One
model, unchanged, accepted by the tolerant sweep and refused by the certified
one, at a span of `1.5e201` and again at `4.323455642275676e17` — the second so
that `inf` cannot be read as "the numbers were absurd". Beside them the two
halves of §68.1's verdict: that the inputs span `1.5` and the answer the fuse
builds spans `4.32e17`, and that the certified fuse of the same pair refuses.

One thing found on the way and left where it was found: **a pair of translated
blocks cannot be made to span `inf`.** At a hair of `2.78e-308` the tolerant fuse
comes back `Undecidable` instead, because *where three surfaces meet* is an area,
an area is two such hairs multiplied, and that underflows to nothing — the
enclosure comes back `inf` and the wobble gate refuses. The corpus's ten `inf`
answers got past that by carrying their hair in a plane's own coefficients
instead — `fb4102cd` has a normal of `[6.3e-322, -1.0, 0.0]` at an offset of
`3.16e-321` — where no area is ever formed from two of them. Nobody has asked
what else that asymmetry reaches.

**Left open:** whether `Policy::tolerant` should refuse a span past
`1.0 / f64::EPSILON`, where the tag it hands back stops being narrower than the
feature it describes. §68.4 says why this lane did not take it; it is a
`rcad-geom` change, and what it needs is the refusal-rate measurement nobody has
made.

---

