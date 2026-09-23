## 211. An ellipse arc gets its centre, and it is the pole of the plane — 2026-09-23

§0's *"an ellipse arc has no centre to be cut by"*, closed. §208.6's second and
third bullets with it — and the third one turns out to have been measured wrong,
which 211.6 is about.

### 211.1 What refused, and why widening the family did not help

`the_way_into_an_arc` in `crates/rcad-topo/src/holds.rs` reads, out of a curved
side of a flat face, the direction square to the side's chord that points into
*this* arc rather than its sibling. Its second line asked
`the_point_a_circle_cut_by_a_plane_is_centred_under`, which answered for a ball
and, for a cone, only when the plane was square to the axis. A plane leaning off
the axis cuts an **ellipse**, so that returned `None` and the whole predicate
returned `None` before any other test was reached.

This is why `lane-one-closed-curve-test` moved the nappe floor 9326 → 9326
(§209.4): it widened the *family* test, so the code agreed a leaning plane cuts
a closed curve, and then still had no centre for it.

### 211.2 The centre is the pole, and the door is its own denominator

`Cone::which_way_a_closed_crossings_centre_lies` is new in `rcad-geom`. With
apex `p`, stored axis direction `w` (**not** normalised), slope `s`, and the
plane `n·x = e`, a point `x` with `v = x − p` is on the cone exactly when
`(1 + s²)(v·w)² − |w|²|v|² = 0`, so the quadric is `A = (1 + s²)·w wᵀ − |w|²·I`.

The conic's centre is the **pole** of the cutting plane — the point `c` on the
plane with `A(c − p)` along `n`, where the quadratic form has no linear part in
the plane's own two directions and the curve is therefore symmetric about `c`.
Taking

```text
along = (1 + s²)(n·w)·w  −  s²|w|²·n
```

gives `A·along = s²|w|⁴·n`, so `along` runs the way `A⁻¹n` does and walking it
from the apex to the plane lands on the pole.

The load-bearing identity is that

```text
n·along = (1 + s²)(n·w)² − s²|n|²|w|²
```

is `Cone::is_cut_in_a_closed_curve_by`'s own test written out. So **the
denominator is the door**: circle and ellipse have it positive, the parabola has
it zero and has no centre at all, and the hyperbola has it negative. The
hyperbola *does* have a centre and is still refused, for a different reason —
its two branches are not one closed curve for a chord to split. A plane square
to the axis falls out as `along = |w|²·n`, the foot of the perpendicular from the
apex, which is the circle this widens.

### 211.3 A direction and a point, because the centre would round

The centre needs a division by `n·along`, and §168 is the whole reason this
predicate is asked where the face is stored — handing a rounded `Point` to the
next step throws away what the exactness was for. So the routine returns the
direction and the apex, and a caller wanting a sign clears the division:

```text
sign(h·c − d_h) = sign( (n·along)(h·p − d_h) + (e − n·p)(h·along) )
```

every term a polynomial in stored numbers, and `n·along > 0` known because that
is the door the caller came through.

**The circle case is byte-identical for a ball and not for a cone**, and the
difference is worth writing down because the natural claim is that it is
identical for both. A ball's answer is its own centre with `along = n`, so the
expression reduces to the old one term for term. A cone square to its plane
gives `along = |w|²·n` mathematically, but `along` is computed in intervals and
the arithmetic cannot collapse the `s²` between the two terms, so the result is
`|w|²` times the old value in an enclosure a few ulps wider — the same sign,
very slightly more refusable. Keeping a separate exact path for the square-to
case would buy those ulps back at the price of two spellings of one question,
which is what 211.5 is about; it was not taken.

### 211.4 Soundness on an ellipse is affine, not perpendicular

`the_way_into_an_arc`'s argument for a circle ended *"a midpoint sits a radius
from the centre square to the chord"*. That sentence is **false on an ellipse**
and could not be carried across.

What carries is affine invariance. Everything happens inside the conic's own
plane `P`. Restricted to `P` the picker is an affine functional `f`, the
hypothesis is `f ≥ 0` on the arc and `f(c) ≤ 0`, and the conclusion is *"`f`
grows across the chord from the sibling arc toward this one"*. Hypothesis and
conclusion both survive an affine map of `P` — a chord stays a chord, the two
arcs stay the two arcs, the centre stays the centre, a half-plane stays a
half-plane — and perpendicularity, which is *not* preserved, appears in neither.
Some affine map of `P` carries the ellipse to a circle, and the circle case is
the argument already there. The doc comment says the perpendicular step is the
circle's alone, because the next reader will otherwise re-derive the false
version.

The tangent paragraph carries over unchanged, and it was checked: a line meets a
conic in at most two points, so the chord still meets the ellipse only at the
arc's own ends.

### 211.5 One spelling instead of three

§208.6's second bullet: `square_to_the_axis` in `fuse/select.rs`, a private copy
of it in `holds.rs`, and `a_plane_cuts_a_cone_in_a_closed_curve` in
`fuse/assemble.rs` were three spellings of one question, which §23.8 says is how
two spellings come to disagree. They are now one, `Cone::is_cut_in_a_closed_curve_by`,
lifted from `lane-one-closed-curve-test`'s `5d83b4b0` — the `holds.rs` copy
existed only because `mod select;` is private to `fuse`, so the question belongs
on `Cone` in `rcad-geom`.

`which_arc_of_a_cones_rim_a_side_is.rs`'s module doc named the deleted spelling
and now names the surviving one.

### 211.6 The nappe floor was never the ellipse arc's to hold

**§208.6's third bullet and §0 both said the ellipse arc is what holds the nappe
floor at 9 326. That was measured wrong, and the arc machinery landing did not
move the floor: 9326 → 9326.**

Instrumented, the floor's holders are the side-6 **corner-up cube** centred on
the apex and turned `0.7`, on the pointed cone and on the frustum. A corner-up
cube's faces lean `acos(1/√3) ≈ 54.7°` off the axis, which for slope one half
was **already inside** the closed family — so those planes were cutting
ellipses that the old code already called closed, and they were never the refused
case. Every leaning block sits above 10 500; the next counts up are 10512, 10646
and 11465.

So the count is evidence and not the goal, and it says the deliverable's third
part is not reachable this way. What leaves 9 326 grid points undecided round an
apex-in-a-cube is **unnamed**, and naming it wants the manoeuvre §204.3 used for
the frustum's big cap: an instrumented run reporting which *face* the undecided
points refuse on. That is a lane of its own and nothing here depends on it.

§212 then moved the floor's holders from three answers to four and the file's
answers from 128 to 152, which is the same shape reaching the same floor and not
a new cause.

### 211.7 The pin that moved, and which way

`a_block_leaning_through_a_frustum_is_cut_along_the_cone_far_above_the_frustum`
asserted a refusal at `18.0 < z < 19.0`. §209.4 left the correctness question
open, and it is answered here rather than bumped: **the old expectation was
wrong.** The piece it named sat on the block's **top** face, which is an ellipse
plane, and it is answered correctly now. What is left refused is the `y = −10`
wall's hyperbola on the nappe below the apex, at
`(-2.0944521032730146, -10.0, -20.433964824566456)` and
`(6.218057161726872, -10.0, -23.551155798941416)` — which is the file's own
thesis shown better than the old pin showed it, a wall parallel to the axis
cutting a curve that never closes. The test asserts that wall by name now, and
that both planes agree with `is_cut_in_a_closed_curve_by`.

### 211.8 What this cost, and what is still open

`just gate quiet` on the merged tree of §211–§214 is in §212.5. Left open:
§211.6's unnamed floor; and `fuse/arrange.rs`'s own ball-only
`where_the_circles_centre_sits`, a fourth spelling of the centre question that
§23.8 would fold in — it predates this work and was not touched.
