## 74. A vertex placed at `NaN`, recorded 2026-09-08

`where_a_closed_crossing_starts` ended its conic arm in `Ok(conic.at(0.0))` with
nothing between that point and the caller. The question put to this lane was
whether that `Ok` could carry a point that is not a place.

**It could, and the input is three lines long.** A pipe of radius `1e300` about
the z axis, against the plane `Plane::from_point_and_normal(origin, (1, 0,
1e-20))`, on a run stated as `RightRoundWhereItsFacesCross`, answered
`Ok(Point([NaN, NaN, NaN]))`. Both surfaces are ones `Cylinder::new` and
`Plane::from_point_and_normal` accept, both public; the function is public and
re-exported from `rcad`. This is §16.10's public-API door and not a walk through
today's callers.

### 74.1 Why the degenerate check let it past

`what_a_run_between_these_two_lies_on` refuses a conic that shrank to a point,
and it asks that question **only of a conic whose size is finite** — deliberately,
because an overflowed conic arrives looking exactly like a shrunk one and has a
different and better name one step later (`CannotWorkOutHowBigACrossingIs`, a gap
rather than a limit). `walk` takes that step: `deviation = ROUNDOFF · size` is
infinite, and it refuses.

`where_a_closed_crossing_starts` never took it. It has no bound to publish, so
there was nothing there to notice the infinity, and the two public doors onto one
crossing disagreed about whether it could be answered at all.

The `NaN` rather than an honest infinity comes from `Conic::at`:

```text
at(0) = centre + across·cos 0 + and·sin 0
```

`sin 0` is exactly `0.0`, and the ellipse a nearly-parallel plane cuts out of a
pipe is stretched down the slope by `‖n‖ / |n·w|` — so `and` is a semi-axis of
infinity, `∞ · 0` is `NaN`, and the `NaN` reaches every coordinate through the
sum.

### 74.2 What the caller saw

`rcad-topo`'s `fuse::arrange` takes this answer at two places. At
`arrange.rs:3509` an `Err` is folded into `ACrossingThisCannotBeOrderedAlong`, so
only an `Ok` gets through — and an `Ok` here was a `NaN` triple handed straight
to `PredicatePoint::from`, which is §27.7's mistake: an exact triple made of
noise, offered to a predicate as something to certify a sign about.

At `arrange.rs:1305` the answer goes to `where_the_crossing_starts`, whose first
line is

```rust
assert!(
    from.is_finite(),
    "a closed crossing is walked between two surfaces built from finite numbers, so where it \
     starts is a real place"
);
```

with a doc comment saying "a seed that is not a place is a bug in the walk". That
sentence was right about whose bug it is and wrong about it being impossible. It
is exactly the shape §16.10 warns about: an argument for an absence, well
written, and deflecting scrutiny in proportion. Its premise — *the surfaces were
built from finite numbers* — is true and does not carry the conclusion, because
what overflowed is not a surface's own number but a ratio of two of them.

**Not demonstrated here: that a `Model::fuse` reaches that assert.** Getting
there needs a fuse whose arrangement holds a plane face and a pipe face crossing
in an ellipse this stretched, with no node already on the crossing, and building
one was outside this lane. What is demonstrated is the function's own answer,
which is the door §16.10 asks about.

### 74.3 The fix, and why it is on the point and not on the size

One check, in the conic arm:

```rust
let start = conic.at(0.0);
if !start.is_finite() {
    return Err(Error::CannotWorkOutHowBigACrossingIs);
}
Ok(start)
```

The name is the one `walk` gives the identical pair, and the one this function's
own `# Errors` section already promised — "when the crossing's size **or a seed
on it** falls out of `f64`'s range" — so the doc was true of the quartic arm and
false of the conic arm, and this makes it true of both. It is a **gap**: the same
pipe and plane written a hundred orders of magnitude down start at a whole
number, so what ran out is `f64`'s exponent and not the answer's existence.

The obvious alternative was to mirror `walk` — refuse when
`the_size_it_was_worked_out_from()` is not finite. It catches this pair too, and
**it is sound today**: the enumeration was done, and all six conic builders put
both semi-axis *scalars* into `how_big_this_is`'s list and scale them by vectors
of length one, so a finite size does imply a finite start.

It was not taken because that sentence is a **sufficiency claim** (§16.10's third
species) and the point check is not. The claim ranges over six builders and the
provenance of every direction vector each of them uses, and **this lane got it
wrong on the first pass**, which is the evidence that it is not free to maintain.
Two of the six — `the_two_meridian_circles` and
`the_two_circles_a_cone_and_a_pipe_cut` — scale by `up = along/‖along‖²`, which is
not a unit vector on its face; a torus axis written `(1e-200, 1e-200, 1e-200)`
squares to `3e-400`, which is zero in `f64`, so `up` would be infinite while the
two radii in the size list stayed exactly what the caller wrote. That was written
down here as a demonstrated hole, a test was written for it, and the test failed:
the refusal that came back was `CannotNameWhichPieceOfACrossingThisIs` and not
`CannotWorkOutHowBigACrossingIs`, because `Axis::new` scales every direction to
length one — and so does the `TryFrom` a deserialized axis goes through — so
`‖along‖²` is one and `up` is `along`. The fact that rescues the claim lives two
files away from the builder that depends on it.

So: the size guard holds, on a chain that has to be re-walked whenever a builder
is added or a direction stops being unit. The point guard is one line, is the
property the caller actually needs, and has nothing to re-walk.
`every_axis_a_caller_can_build_has_a_direction_of_length_one` in the test file
pins the step that was misread, so the next reader finds it checked rather than
asserted.

§69.5's objection to a guard on a branch nothing can reach does not apply here:
this branch was reached, with a printed `Ok(Point([NaN, NaN, NaN]))` before the
change and a named `Err` after it, on the same input.

### 74.4 What was tested, and what the boundary does not cover

`crates/rcad-geom/tests/where_a_closed_crossing_starts_is_a_place_or_a_refusal.rs`.

`a_conic_too_big_to_hold_a_point_is_refused_and_the_same_pair_smaller_starts_somewhere`
pins the pair above, and asserts the other half of the gap: the same pipe at
`1e200` starts at a finite point away from the origin, and `walk` agrees with
this function on both radii.

`every_conic_crossing_this_answers_starts_at_a_place` sweeps six configurations
through all four conic builders at every tenth power of two from `2⁻⁵⁰⁰` to
`2⁵⁰⁰`, both ways round: 1,096 of 1,212 asks start at a place and 116 refuse, and
every `Ok` is checked for being finite. Both counts are held to a floor, so a
change that turns the sweep into all-refusals fails it.

`every_axis_a_caller_can_build_has_a_direction_of_length_one` is the third, and
it is 74.3's own correction rather than a check on this function.

**What that boundary does not cover**, named rather than left silent:

- **Placements far from the origin whose differences are small.** The sweep
  scales whole configurations, which is the case one power of two fixes; the
  case it cannot fix (§16.9's limit) is not in it.
- **Degree-four pairs.** A ball against a pipe and two leaning pipes leave
  through `the_seed_this_run_is_picked_up_at`, not through `Conic::at`, and
  `a_quartic_picked_up_at` already refuses a seed that is not a point.
- **The two circle-picking builders.** A plane against a torus and a coaxial cone
  against a pipe refuse a run with no picker on it as a tie before any of this is
  reached, so the sweep says nothing about them. They carry a guard of their own
  in `which_of_these_circles_this_run_is`, which asks the centre and both
  semi-axes directly.
- **Underflow.** A conic whose size runs off the *bottom* of `f64` is a different
  question. `has_an_angle_round_it` asks `Vector::length` rather than
  `length_squared` precisely so that a bar of radius `1e-200` is told apart from
  a circle of no radius, and nothing found here moves that.

