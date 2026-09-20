## 156. §150 is a picking plane rounded from its anchor's coordinates, diagnosed 2026-09-19

*§150 asked why one pair of eighty-five answers one way round and refuses the
other. It is `HalfSpace::through_a_point_facing` at `half_space.rs:110` storing
an offset rounded from a plain `f64` dot of the normal with a point `1.8e15`
out. §16.10's **gap**, decisively. The fix is dispatched as a lane, not landed
here.*

### 156.1 Everything is identical both ways round except one stored `f64`

Instrumented in a standalone copy of `rcad-geom`, both orders side by side:
`towards` is an exact negation, and `apart`, `across`, `how_far`, `out`,
`leaning` and `either_way` are **bit-identical**. `how_well_aligned` ranks by
`.abs()`, so a negated step picks the same candidate and only
`leaning_the_right_way` flips a sign that then cancels in both places the normal
is read.

The one number that does not follow: the picking plane's **stored offset**,
`0.0` one way and `-0.5` the other. The true `n·from` is `0.2865091987060316`
— exact rational and a `mul_add`-compensated dot agree to the last digit. The
stored `0.5` is two products of `1.8e15`, whose ulp is `0.25`, cancelling to a
quarter: the plane lands `0.2134908012939684` out of place, which is
`0.3335636787994107` of real distance on pipes one radius wide. The two lines
read `0.4386…` and `−0.1521…`; `0.2865…` is strictly between them and `0.5` is
past both, so `two_lines_told_apart` correctly reports both lines on one side.
`side[0]` and `side[1]` are strictly positive with widths `~1.1e-15` — **no sign
is lost and nothing is undecidable.**

### 156.2 Two numbers in §150 were wrong, and one was this session's own

**`1.6583123951777` was wrong.** §150.2 published it as "measured here". The
axes are `1.7575736716416024` apart. Confirmed three ways: the kernel's own
`apart` interval, the lane's exact-rational projection, and an independent
exact computation by this session that reproduces
`scaled_to_length_one_from_anywhere`'s documented contract and gets
`1.7575736716416026` — agreement to the last ulp.

**How it was nearly missed is the useful part.** The first independent attempt
here normalised the direction the obvious way, `w/√Σw²`, and got
`1.7673482164723309` — close enough to look like a rounding difference and far
enough to settle nothing. It was neither: the separation here is set entirely by
the rounding residue of `round(d̂·1e16)`, so a one-ulp difference in `d̂` makes
it a *different pair*. `scaled_to_length_one_from_anywhere` works its length out
to about a hundred bits (§120), which the naive form does not. **A replication
that does not reproduce the kernel's own rounding is measuring something else**,
and at `1e16` it will not look like it.

**And "a step at `7.68e15` is `2`" was wrong**, in the test file that landed with
the diagnosis. One ulp there is `1`. Because the shared direction leans out of
`z`, a step moves the axes `√(1 − d_z²) ≈ 0.64` apart — near a third of the
touching distance of `2`. Computed exactly at nudges 0 through 4: `0.6135`,
`1.1503`, `1.7576`, `2.3821`, `3.0131`. The behaviour the sentence was defending
is right and is asserted — one step back still crosses in two runs, one step on
misses entirely — only the arithmetic beside it was not.

### 156.3 A gap, and the fix is the anchor's rounding

The answer exists at this precision and one rounded dot product is the whole of
what could not reach it. The fix is to round the offset from the exact dot:

```rust
let offset = Exact::dot_of(facing.components(), through.coordinates())
    .as_ref()
    .map_or_else(|| facing.dot(through - Point::origin()), Exact::nearest_f64);
```

**Right rather than merely effective here**: the picking normal is exactly
square to `along` and the anchor is a point on the axis, so `n·from` is
invariant under sliding the anchor along that axis. Its true value is the axis's
perpendicular offset — an ordinary-sized number however far out the stored
origin sits — so a correctly rounded offset always lands within a half-ulp of an
ordinary number, where the naive dot carries the ulp of the *coordinates*.

Repairing it locally by choosing a better anchor was considered and rejected:
`off_the_picker` is already an honest reading of where the stored plane is, so
the defect is the plane's placement, and **every other `HalfSpace` built through
a far-out point has the same flaw**.

That breadth is also why it is not landed here. `through_a_point_facing` is
called all over the kernel and the diagnosis only ever ran `rcad-geom`, in a
copy. What the fix does to refusal rates, to `fuses_that_must_agree.rs` and to
the cost of building a half-space in an inner loop is unmeasured, and a lane is
measuring it.

Same shape as §141.4 and a different quantity. §141.4 rules out the anchor's
magnitude because it enters as a zero-width `Interval::exactly` — still true
*inside* the enclosure arithmetic. Here the anchor does its damage earlier, in
the plain `f64` dot that becomes the offset, before any interval exists.

