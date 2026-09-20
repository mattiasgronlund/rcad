## 121. `Plane` still rounds twice, and that is the other half of §120, booked 2026-09-18

*§120 fixed `Axis::new` and nothing else. Every other surface in this kernel
normalizes the way `Axis::new` did before it, and a plane does it through the
very function §120 measured as wrong. Booked here rather than fixed, for the
reason §120.3 gave about a different neighbour.*

### 121.1 Which constructors still run the old arithmetic

`Vector::scaled_to_length_one_from_anywhere` — the exact power-of-two rescale
plus the hundred-bit length — has exactly **three** callers in the whole
workspace outside its own tests: `Axis::new`, and two lines in `walk.rs` that
name a crossing's two directions. **It appears nowhere in `plane.rs`.**

Every other normalize in `crates/*/src` is one of two older shapes:

| | what it runs | where |
| --- | --- | --- |
| `Plane::from_point_and_normal` | `reduced_by_largest_component` then `normalized_as` | `plane.rs:127`, `:128` |
| `Plane::from_coefficients` | the same two, same order | `plane.rs:187`, `:189` |
| `Plane::through_three_points` | calls `from_point_and_normal` | `plane.rs:164` |
| `TryFrom<[f64; 4]> for Plane` | calls `from_coefficients` | `plane.rs:766` |
| `Plane::basis` | bare `normalized_as`, no reduce | `plane.rs:330` |
| `Cylinder::basis`, `Cone::basis`, `Torus::basis` | bare `normalized_as` | `cylinder.rs:130`, `cone.rs:186`, `torus.rs:187` |
| `Segment::new` | bare `normalized_as` | `segment.rs:57` |
| the plane a circle is written in | bare `normalized_as` | `conic.rs:216` |

**The public API was checked, not today's callers.** `Plane::transformed`,
`reversed`, `moved_to` and `holding` normalize nothing at all — the whole file
normalizes at three lines, 128, 189 and 330, and the complete list of
`normalized`/`normalized_as` call sites under `crates/*/src` is the evidence. So
a `Plane`'s stored normal is whatever one of its two constructors rounded, and
nothing rounds it again.

**Three corrections to the list as it was first drawn.** `Cylinder::new`,
`Cone::new` and `Torus::new` normalize nothing: they take an `Axis` that is
already built, so a pipe's *axis* is correctly rounded today and only its
`basis()` is not. A `Conic`'s two radii are not normalized either — they are a
unit basis vector from some surface's `basis()` multiplied by a radius, so they
inherit whatever that `basis()` rounded rather than running a normalize of their
own. And `HalfSpace` normalizes nothing anywhere, deliberately: §33.8 gives it no
canonical form at all, so it is exempt rather than unfinished.

**A bare `normalized_as` with no reduce is a third thing, not §120's old path.**
It rounds twice like the old path and is *also* not a function of the ratios
alone, so `basis()` and `Segment::new` carry both defects where `Plane` carries
one. Whether the second one matters anywhere is a question this section does not
answer; it is named so the fix does not assume there are only two shapes.

### 121.2 The plane path's error rate is a live measurement, not a memory

`the_old_way`, the three-line closure inside
`a_unit_component_is_the_nearest_f64_to_the_exact_ratio`, divides each component
by the largest-magnitude component and then calls `normalized()`. That is the
same two operations in the same order that `Plane::from_point_and_normal` makes,
with the same arithmetic in each — the two differ only in which `Error` variant
a zero or a non-finite input produces and in the label on it.

So the row §120.6 prints as "the arithmetic §120 replaced" is not a historical
figure at all: **it is a measurement of what every `Plane` in this kernel still
runs**, on the same tree, in the same test run. It misses the nearest `f64` on
**2 759 of 16 680 components, 16.54%**, over a population spanning 900 binades.
That is the sharpest single thing in this section, and it is sharp only because
the closure and the constructor really are the same arithmetic — which is worth
re-checking rather than inheriting if either one is ever touched.

### 121.3 A gap and not a limit

§16.10's test: the answer exists and the arithmetic could reach it. Both hold.
The correctly rounded answer is a function of the exact ray, `2 759` components
prove today's answer is not it, and the function that *does* reach it is
`scaled_to_length_one_from_anywhere` in `point.rs` — same crate, already written,
already gated, already measured at zero misses above `2⁻⁹⁰⁰`. Nothing has to be
invented. So this is a gap, and §16.10 says fix a gap.

### 121.4 Why it is booked rather than fixed here

Fixing `Plane` moves the stored coefficients of every plane in the kernel, and
the kicad corpus counts move with them. §120.3 declined exactly that blast radius
rather than take it inside §119, and §120.7 and §120.8 are what taking it costs
when it is a lane's own subject: fourteen tests moved, three witnesses dissolved
and needed replacing rather than renumbering, and the corpus had to be re-run by
name and not by rate. Taking the plane half as a rider on §120 would be the same
mistake §120.3 refused, with a different neighbour. It gets its own lane.

### 121.5 §120.5 named the wrong tell

§120.5 said the fix owed "the bit-pinning sites updated in one commit with the
change", and named `the_same_wall_built_from_a_longer_edge_is_the_same_plane`,
with its assertion against `FRAC_1_SQRT_2.to_bits()` as "the tell".

**That test builds a `Plane` and never could have moved.** It ran before §120 and
after it and the literal is the same, because `Plane::from_point_and_normal` does
not go near the function §120 changed. §120.7 records that it did not move; this
says why it was never going to, so a reader following the thread meets the
correction where the prediction was made rather than only where it failed.

The site §120.5 was reaching for is
`the_two_finder_relay_pipes_written_with_one_radius_are_one_width`, in
`a_pipe_written_at_another_axis_scale_is_the_same_pipe.rs`. That one is an `Axis`
pin, and it did move: from `FRAC_1_SQRT_2.next_down()` to `FRAC_1_SQRT_2` itself.
It is also where the comment above the plane pin in `fuses_that_must_agree.rs`
went wrong, predicting that §120's fix "moves the literal below by one ulp" — the
comment now says the plane path is untouched and points here.

Both mistakes are the same one: **`Axis` and `Plane` were being read as one
normalize**, and they have not been one since §120 landed. Anything written about
"the normalize" in this kernel now has to say which.

### 121.6 What the fix owes when it runs

- `Plane::from_point_and_normal` and `Plane::from_coefficients` on
  `scaled_to_length_one_from_anywhere`, with `reduced_by_largest_component`
  deleted if nothing else wants it — it is private to `Plane` and has no other
  caller.
- A decision, argued rather than assumed, on the bare `normalized_as` in
  `Plane::basis`, `Cylinder::basis`, `Cone::basis`, `Torus::basis`,
  `Segment::new` and `conic.rs:216`. They are a different defect (§121.1) and may
  well want a different answer.
- The bit-pinning sites updated in the same commit as the change, and this time
  found by running the tests rather than by reading: at least
  `the_same_wall_built_from_a_longer_edge_is_the_same_plane` and the four doc
  sentences about `Plane` dividing by its largest component —
  `half_space.rs:25`, `plane.rs:210`, `plane.rs:1010`,
  `fuses_that_must_agree.rs:2807` — which are true today and become false then.
- Re-measured corpus counts **by name**, not by rate: `assert_pinned_kicad_imports`
  compares sets and fails both ways, and §120.8 is the argument for trusting that
  and not a percentage written into prose.
- §17.1's claim re-checked rather than assumed — two faces that are the same plane
  holding the same four numbers. §120.1 says a correctly rounded normalize keeps
  the same-ray property by construction, and §120.6's 210 000-pair test is the
  shape the plane half owes as well.
- §120.6's table gains a row, because the closure it measures is the thing being
  replaced: after the fix, `the_old_way`'s rate stops describing anything live.

