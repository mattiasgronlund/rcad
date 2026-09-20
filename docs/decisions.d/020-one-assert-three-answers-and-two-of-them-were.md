## 20. One assert, three answers, and two of them were never checked

*Recorded 2026-08-31.*

`SolidView::measure` returns volume, surface area and centroid. It had one
check:

```rust
assert!(six_volumes.is_finite() && six_volumes > 0.0, ...);
```

Two things are wrong with that line and they are unrelated to each other.

### 20.1 The assert was doing two jobs and only one was its own

The justification written above it says a closed solid with its faces pointing
outwards cannot enclose non-positive space, "which makes this a broken model
rather than bad input". That is true, and it is about **geometry**. The check is
about **arithmetic**. A cube of side `5e102` points its faces outwards perfectly
well; what it did was overflow. So:

- **not finite** — the sum ran out of room. Bad input, and bad input is data
  rather than a bug (§4.1). Return `Err`.
- **finite and not positive** — the faces point inwards. A corrupt solid, and a
  crash beats a corruption. Still panics.

Measured before the fix, all three of these built and then panicked:
`add_cube(5e102)`, `add_cube(1e140)`, `add_cube(1e200)`. Same shape as the
`unreachable!` in `ring_runs_anticlockwise` that §16.11 found: a claim about
what geometry can do, standing guard over what `f64` can hold.

### 20.2 The worse half: two answers nobody was checking at all

`six_volumes` is the **last** of the three quantities to overflow, and it was
the only one guarded. Measured across the range, on cubes:

| side | what came back |
|---|---|
| `2^256` (`1.16e77`) and up | volume fine, `surface_area` **infinity**, centroid **infinity on all three axes** — returned, no complaint |
| `2^341` (`4.5e102`) and up | `six_volumes` overflows, panic |

That is 85 binades wide. In all of it, `measure` handed back a `Measurement`
with a real volume and an infinite centroid and said nothing. The assert existed
to stop exactly that, and it was watching the one field that survived longest.

Both crossovers are exactly powers of two and both mechanisms are plain:
`Vector::length` is `sqrt(x² + y² + z²)`, so it squares components of size
`side²` and dies at `side⁴ = 2^1024`; the moment sum accumulates
`six_volume × offset ≈ side⁴` and dies at the same place.

**And both are §16.10 gaps, not limits.** A `1e100` cube's surface area is
`6e200` and its centroid is `5e99`. Both are ordinary `f64` numbers. Only the
arithmetic on the way to them overflowed — which is the definition.

### 20.3 The fix: scale by what each quantity is invariant under

§18's technique, applied three times with the scale chosen by what is being
measured. Every factor is an exact power of two, so every sum comes back bit for
bit what it would have been, and the whole change is invisible to models that
were already in range — a claim the existing exact-equality and
`changes_not_one_bit` tests check on every run.

- **Volume and centroid: one power of two per axis.** A 3×3 determinant has one
  entry from each column in every term, so scaling a column scales the
  determinant by exactly that factor. The moment inherits it.
- **Surface area: one power of two per face.** A length does *not* come apart
  that way — `x² + y² + z²` only factors if all three were scaled alike — so
  each face is measured whole. **Per face and not per solid**, because a tower
  one unit across and `1e200` tall scaled to its own height would push its top
  and bottom faces to `1e-400`, and the two units of area they carry would
  silently vanish.
- **Undo the scaling in steps, never in one multiplication.** `2^exponent` need
  not be representable when the answer is. Every step moves the same way, so the
  running value walks towards the answer and never past it, and an overflow
  means what it should.

Refusals are now `Error::VolumeTooLargeToMeasure` and
`Error::SurfaceAreaTooLargeToMeasure`. They are separate because area and volume
grow at different rates: a wide thin slab can run out of room for its area while
its volume still fits.

### 20.4 `validate` was calling a sound model broken

`Model::validate` and `fuse/assemble.rs` shared the same conflated test and so
reported `Invalid::VolumeIsNotPositive { volume: inf }` for a `1e300` cube.
§16.11's closing paragraph had already noticed and filed it as *"the protection
was always accidental"*. It is not a broken model — the same cube classifies
points `Inside` with an empty log, which is the kernel certifying that it works.
`Invalid` now means only what its name says, and too-large is its own error.

### 20.5 A published bound that was the wrong side of §16.10

`a_solid_a_long_way_from_the_origin.rs` said the cube range ends at `2^341`, and
gave the reason:

> a cube that big encloses `2^1023` of space, six times which is past the
> largest `f64`

That sentence fails §16.10's test on its own evidence. `2^1023` is a perfectly
ordinary `f64`. Only the `6 ×` on the way to it was not, because the sum was
scaled back in one multiplication. The answer existed and the arithmetic could
not reach it — a **gap**, written down two lines beneath a sentence calling it a
limit, and pinned by a test asserting the refusal so it could not drift back.
§15.6 says a published bound needs a test; this is the case where the test held
the wrong bound in place.

The range is now one binade wider. `2^341` measures — volume `2^1023`, area
`6 × 2^682`, centroid at half the side, all exact — and `2^342` refuses, because
`2^1026` genuinely is not a number `f64` has. That end is real.

### 20.6 The regression this caused, and what it taught

The first fix used **one** scale for the whole solid, taken from its largest
offset. `just` went from one failure to a different one: two towers a unit
across and `1e200` tall now fused to a volume of exactly zero. Scaling by the
tallest axis put the two short ones at `2^-664`, and their product with the
third is `2^-1328` — nothing.

Worth keeping because the fix and the bug come from the same place. A uniform
scale is right for a *shape*, and a solid is not obliged to be one size in all
three directions. The dynamic-range check would not have warned: the tower's
ratio is `1e200`, wildly past `LARGEST_SUPPORTED_RATIO` of `1e11`, and the test
asserting it works has been passing all along. §16.11's inversion again — the
ratio check is blind to the case that actually breaks the arithmetic.

### 20.7 Corrections owed to earlier sections

Measured at `898ac58` and recorded rather than quietly edited, which is what
§16.10 is for.

1. **§16.11's "the large end is a different bug" is stale.** `1e140`, `1e200`
   and `1e300` cubes all classify `Inside` with an **empty log**. Fixed by
   `a23d73d`'s rescaling, before this work started.
2. **The cause §16.11 recorded for it was wrong.** It blames the ray target
   from the bounding box overflowing. That does not overflow until side
   `7.7e307`; at `1e140` the overflow was inside `orient3d`. The attached
   hypothesis — that a ray target relative to the bounding box "would likely
   just work" — proposed a fix for a problem that was not there.
3. **The same wrong cause is repeated in `classify.rs`**, claiming a `1e140`
   cube returns `Undecidable` with thirty-six records. It returns `Inside` with
   none.

---

