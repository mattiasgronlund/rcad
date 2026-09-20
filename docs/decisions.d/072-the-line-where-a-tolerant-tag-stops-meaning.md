## 72. The line where a tolerant tag stops meaning anything, measured, recorded 2026-09-07

§68.4 left one thing open: whether `Policy::tolerant` should **refuse** a span
past `1.0 / f64::EPSILON` — `4.5036e15` — where the tolerance tag it hands back
stops being narrower than the feature it is describing. It gave four reasons for
not taking the line, and said what the argument was missing: the refusal-rate
measurement nobody had made.

The measurement is here. **The answer is that the line should not be taken**, and
one of §68.4's four reasons is withdrawn on the way, because the measurement says
it was never a cost at all.

### 72.1 The three published rates, before and after

The change measured is three lines at the top of `DynamicRange::check_under_policy`'s
`Severity::Warn` arm, in `crates/rcad-geom/src/scale.rs`:

```rust
if self.ratio > 1.0 / f64::EPSILON {
    return Err(Error::DynamicRangeTooWide { ratio: self.ratio });
}
```

Both runs are one command:

```
mise exec -- just t -E 'test(/how_often_a_certified_fuse_with_a_cone_in_it_refuses/)
  + test(/how_often_a_certified_fuse_with_a_torus_in_it_refuses/)
  + test(/how_often_a_certified_fuse_of_a_turned_curved_pair_refuses/)' --no-capture
```

| rate | before | after |
|---|---|---|
| turned curved pair, 384 fuses | 0.0% | 0.0% |
| a frustum against a corner-up cube, 96 | 90.6% | 90.6% |
| a pointed cone against a corner-up cube, 96 | 81.2% | 81.2% |
| a frustum against a block, 96 | 93.8% | 93.8% |
| a frustum against a bar, 96 | 100.0% | 100.0% |
| two frustums on one axis, 96 | 100.0% | 100.0% |
| a bend against a slab square to the axis, 96 | 100.0% | 100.0% |
| a round bar against a slab square to the axis, 96 | 75.0% | 75.0% |
| a bend against a wedge through the axis, 96 | 100.0% | 100.0% |
| a round bar against a wedge through the axis, 96 | 0.0% | 0.0% |
| a bend against a bar through the hole, 96 | 100.0% | 100.0% |
| a round bar against a bar through the hole, 96 | 0.0% | 0.0% |
| a bend against a ball in the bend, 96 | 100.0% | 100.0% |
| a round bar against a ball in the bend, 96 | 100.0% | 100.0% |
| a bend against a second bend, 96 | 100.0% | 100.0% |

Not a percentage moved, and not a *name* moved either: all four of the cone's
corner-up breakdown, all four of the frustum-against-a-block breakdown, every
count identical.

**They cannot move, and the reason is one line of `Policy`.** All three files
fuse under `Policy::certified()`, and `Policy::certified` sets
`dynamic_range_check: Severity::Fail`. The line lives in the `Severity::Warn`
arm. `Policy::tolerant` is the only constructor in the crate that asks for
`Warn`, and none of the three ever builds one — the word `tolerant` does not
appear in any of the three files.

So **§68.4's third reason is withdrawn.** "It would move published refusal rates
and nobody has measured what it costs" was a fair thing to say without the
measurement, and with the measurement it is false: the cost to the published
rates is exactly zero, by construction and in fact. A future argument for this
line does not have to answer it.

### 72.2 What does move: four tests, and every one of them is about this

`mise exec -- just t` — the whole suite, 1800 tests — with the same three lines
in place:

```
Summary [27.378s] 1332/1800 tests run: 1328 passed, 4 failed, 0 skipped
```

The run stops at the first failures, so 1332 is what got as far as running, not a
count of what passes. The four are:

- `a_fuse_that_builds_a_sliver::the_edge_across_a_sliver_is_as_long_as_the_sliver_is`
- `a_model_only_the_policy_that_built_it_calls_valid::a_tolerant_fuse_writes_a_sliver_only_a_tolerant_sweep_calls_valid`
- `a_model_only_the_policy_that_built_it_calls_valid::the_two_sweeps_split_the_same_way_a_long_way_from_overflow`
- `a_model_only_the_policy_that_built_it_calls_valid::the_inputs_span_one_and_a_half_and_the_answer_the_fuse_builds_spans_far_more`

There is no collateral damage anywhere in the kernel. What breaks is four tests
whose whole subject is this behaviour, three of them written by §68 itself to
record it. That is not a bill to be weighed against a benefit; it is four
statements that the line is wrong, and taking the line means answering them
rather than paying for them.

So **§68.4's first reason is now the whole of the case against**, and it is
unchanged: `the_edge_across_a_sliver_is_as_long_as_the_sliver_is` deliberately
pins that a tolerant fuse *writes* a `1e-200` sliver, at a span of `1.5e201`.

### 72.3 Nothing past the line came back wrong, and the family looks empty

§68.4's fourth reason — "the failure it would prevent has not been shown to
exist" — is the one §16.10 says needs a demonstration rather than a paragraph.
Four shapes were built and fused both ways round under `Policy::tolerant(100.0)`,
and every answer measured against a closed form. Two ten-millimetre blocks, the
second at `(5, hair, 5)`, so the union is `1750 - 75·hair` mm³ and every hair
here is far too small to move that from `1750.0`.

| | span of the answer | shortest edge | volume, `[low, high]` | volume, `[high, low]` |
|---|---|---|---|---|
| A. hair `3.469446951953614e-17` at the origin | `4.323455642275676e17` | `3.469446951953614e-17` | `1750.0` | `1750.0` |
| B. hair `1e-200` at the origin | `1.5e201` | `1e-200` | `1750.0` | `1750.0` |
| C. A, whole pair moved `2^40` along `x` | `3.1691265006138082e28` | `3.469446951953614e-17` | `1750.0` | `1750.0` |
| D. A, whole pair moved `2^40` along `y` | `2.199023255572e11` | `5.0` | `1750.0` | `1750.0` |

Exact, in every cell, at every order. And a *second* tolerant operation on the
wide answer is exact too: cutting A's and B's answers with a 30 mm block covering
everything above `y = 5` leaves `875.0` mm³ against a closed form of
`500 + 75·(5 - hair)`, which is `875.0`.

**C and D together are why.** C is seven thousand billion times past the line —
`3.17e28` against `4.5e15` — and bit-for-bit right, because the largest
coordinate is on `x` and the hair is on `y`, where the coordinates are small and
`f64` has digits to spare. D is the same pair with the hair moved *onto* the big
coordinate, and `2^40 + 3.469446951953614e-17` is `2^40`: the hair is rounded
away before anything is fused, there is no sliver to build, and **the span falls
back to `2.2e11` with a shortest edge of `5.0`** — an ordinary model.

That is the shape of the argument, and it is a property of `f64` rather than of
these four shapes. Two distinct `f64` near a magnitude `L` differ by at least
`ulp(L)`, which is about `L · f64::EPSILON`. So a chord of length `h` whose two
ends differ on an axis where both are near `L` has `h ≳ L · f64::EPSILON`, and
`L / h ≲ 1 / f64::EPSILON` — *at* the line, never past it. **Past the line, the
short feature's ends must agree exactly on every axis where they are large, and
differ only where they are small.** There is no third case. The reason the
answers above are exact is the reason they had to be.

**What this does not show.** It is a demonstration over translated blocks with
flat faces, plus one second operation. It says nothing about a hair carried in a
plane's own coefficients rather than in a translation — §68.6's `fb4102cd`, with
a normal of `[6.3e-322, -1.0, 0.0]` — where the small number never becomes a
coordinate difference at all and the argument above does not reach. Nor does it
cover curved faces, where a coordinate is the *output* of a construction and the
short chord's ends need not be a difference of two stored numbers. Those are the
two places to look if anyone wants to reopen this; what is settled is that the
straightforward reading of "the tag is wider than the feature" does not produce a
wrong answer, and the reason it does not is structural.

So the fourth reason is upgraded rather than repeated: not "no case has been
shown", but "here is why the case cannot be built out of coordinate differences,
and here are the two families that argument does not cover".

### 72.4 The verdict

**No.** `Policy::tolerant` keeps warning past `1.0 / f64::EPSILON` rather than
refusing. Of §68.4's four reasons, the third is withdrawn as measured-at-zero,
and the other three stand:

1. Four tests pin the opposite, deliberately, and three of them are §68's own.
2. It belongs in `check_under_policy` — that much §68.4 had right, and this
   lane put it there to measure it — but `Model::fuse` and `Model::validate`
   would then agree on a refusal that nothing has shown is earned.
3. *Withdrawn.* Zero of the three published rates move; all three are certified,
   and the line only touches `Warn`.
4. Every answer past the line measured exact, at spans up to `3.17e28`, and the
   arithmetic says why.

The honest reading of §68.4's observation is that the tag is **conservative**
rather than meaningless. `coordinate_spacing` is a whole-model bound —
`largest_coordinate_magnitude · f64::EPSILON` — hung on a feature that lives
where the coordinates are small. A bound that is loose is a bound; it never
claims the feature is missing, only that nothing smaller than that is
*guaranteed* to be resolved anywhere in the model. Refusing on it would be
refusing an answer `f64` holds exactly, which §16.10 calls a gap and not a limit,
and this kernel does not manufacture gaps.

`DynamicRange::LARGEST_SUPPORTED_RATIO` stays the one number that decides, at
`1e11`, with `Severity` deciding how loud to be about crossing it. No second
threshold inside the warn band.

### 72.5 What would reopen it

A demonstration, in the shape §16.10 asks for: one tolerant fuse past the line
whose answer is *wrong* — not merely wide — measured against a closed form or
against the same fuse asked the other way round. The two families 72.3 does not
cover are where to look. A rate measurement is no longer owed; 72.1 is it.

