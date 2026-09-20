## 33. The half-space gap, and a type that is not a surface, recorded 2026-09-01

Lane 1b of §31.6, taken out of §32.3's last row: **the half-space gap is closed,
and closing it moved every family that had one.** §31.6 reserved §33 for the
curved fuse; this lane takes the number and the curved fuse becomes §34's
neighbour, so the closing gate is not surprised by the shift.

### 33.1 §24.6 stands, and the argument is that a picker is not a surface

§24.6 says the surface set does not grow, and CLAUDE.md core principle 6 says a
decision is argued and amended rather than routed around. **It is neither
amended nor routed around, because what landed is not a `Surface`.**

§24.6's argument is that a new `Surface` variant touches `validate`, the mesher,
`is_wholly_flat`, the fuse and serialization. Every one of those is a place that
asks *what is this face sitting on*. A half-space is never a face's geometry: it
picks one piece of a crossing out of the rest, and §24.5 already widened that
field once without touching any of them, because the field is not `Surface`'s
job.

So the type split follows the question. `EdgeRun::PartOfWhereItCrosses` holds a
`SmallVec<[Picker; 1]>`, and

```rust
pub enum Picker {
    InFrontOf(HalfSpace),
    OutsideABall(Sphere),
    OutsideAPipe(Cylinder),
}
```

has no arm that is a plane, so there is no second way to say "flat picker" and
nothing has to decide which of two spellings it is holding. `validate`, the
mesher and `is_wholly_flat` never see a `HalfSpace`, because the type system
will not let one reach them — which is the point. **A `Surface::HalfSpace`
variant would have been a lie the type system was telling**: every `match` on
`Surface` in `rcad-topo` would have grown an arm for a case that cannot occur,
and each of those arms would have been an unreachable claim of exactly the kind
§16.10 says needs a demonstration and never gets one.

The cost §24.6 names is therefore not paid, and the cost that *is* paid is one
file in `rcad-geom`, three call sites in the intersector, one dispatcher in
`walk`, four constructions in `rcad-topo`'s tests and the format version.

### 33.2 A normal a caller wrote square to a direction stays square

`HalfSpace` stores `normal` and `offset` exactly as written. No reduction, no
normalization, no rescaling on the way back in from the format.

That is safe only because of an invariant the type system does not state, and
the doc comment says so: **a half-space is never a face's geometry and is never
compared for face identity.** §17.1 rests on two faces that are the same plane
holding the same four numbers, and `Plane` earns that by scaling its normal to
length one (§32.5's trap). Two half-spaces describing one flat surface may hold
quite different numbers, and nothing anywhere asks whether two of them are the
same.

`a_normal_written_square_to_a_direction_is_still_square_to_it_once_stored` is
§31.7's first criterion, which §32.5 recorded as impossible. It is the same three
normals §32.3 used, and it asserts both halves: all three are still exactly
square after a `HalfSpace` stores them, and not one of them is after a `Plane`
does. The two demonstrations §32.3 left in `plane.rs` are untouched and still
pass, because nothing about `Plane` changed.

### 33.3 The separating direction is not the one you can write down, and does not have to be

This is the part §32.3 did not reach, and it is why "pick a different square
normal" looked like a tenth of a percentage point rather than a closure.

A caller can write exactly three normals square to a direction `w`: `w × x̂`,
`w × ŷ` and `w × ẑ`, each two of `w`'s own components with one negated and a
zero, so the dot product is two products of the same pair of numbers with
opposite signs. None of them is the direction that separates two pieces of a
crossing. **It does not have to be.**

What separates two endless lines is any half-space whose normal is square to
their common direction and has a non-zero component along the direction they are
apart in. What separates two loops of a leaning-pipe crossing is the same, and
the price of a normal that is not the ideal direction is a real number rather
than a refusal. Writing `n̂ = α·â + β·b̂` in the two directions across the fat
pipe's axis, a loop point sits `a` along `â` and `b` along `b̂` with
`a² + b² = ρ₂²` and `|a| ≥ √(ρ₂² − (δ+ρ₁)²)`, so `|b| ≤ δ + ρ₁` and the reading
is at least `α·√(ρ₂² − (δ+ρ₁)²) − |β|·(δ + ρ₁)`.

The published floor on `α` is **`1/√3`**: it is `max |b̂ᵢ| / √(1 − ŵᵢ²)`, which
is at least `max |b̂ᵢ|`, which is at least `1/√3` because `b̂` has length one.
§15.6 asks a published bound for a test and
`the_best_exactly_square_normal_is_never_less_than_a_third_aligned_with_what_was_asked`
searches every whole-number direction in a nine-cube against every other and
finds **`1/√2`** and never less — the worst case is a direction along a
coordinate axis, where one of the three normals is nothing at all and the other
two are a right angle apart.

**The asked-for direction goes in first**, and that is not an optimisation. On
the lattice the separating direction is *already* exactly square, and taking a
coordinate crossing instead put the lattice quartic control at 1.12% — a
regression the control did not catch, because nothing had ever asserted the
control's half-space count. It does now.

### 33.4 A cross product that cancels, where a difference of squares does not

A ball crossing a pipe off-axis leaves two loops either side of the plane
through the ball's centre square to the pipe's axis, and the picker's normal is
now that axis direction, bit for bit. How much of it lies *across* the axis is
therefore exactly nothing — but only if the arithmetic can see it.

`√(‖n‖² − (n·ŵ)²)` cannot. The two squares are equal to a couple of last bits,
and the square root of a couple of last bits is about `1e−8`, which then
multiplies the ball's radius. `‖n × ŵ‖` can, and summed through `Exact` it is
exactly zero. **Eight orders of magnitude, from reading the same fact a
different way**, and it is the same lesson §29's exact sums taught about a
nearly-parallel cross product.

### 33.5 The numbers

Family by family and never blended (§17.7). 2048 pairs per family,
`Policy::certified()`, share refusing
`CannotNameWhichPieceOfACrossingThisIs`.

| Family | at §32.4 | now |
|---|---|---|
| lattice, planar | 0.00% | 0.00% |
| lattice, quartic | 0.00% | 0.00% |
| turned, planar | 0.44% | **0.10%** |
| turned, quartic | 11.86% | **2.24%** |
| leaning | 0.20% | **0.00%** |
| the band | 10.79% | **3.76%** |
| written at `2^k` | 3.95% | 3.95% |

`Undecidable` does not move in any family, which is expected: nothing here
changes a sign question. The `2^k` row does not move either, and that is the same
finding §32.1 recorded — those refusals are products landing among the
subnormals, and what reaches them is the centred frame §31.9 files as deferred.

**Every refusal that remains is a different one from the one this lane closed.**
Instrumented site by site over all seven families, the arm that says *this
picking normal is not exactly square to the direction the piece runs along*
fires **zero times**, in every family. That arm can now only be reached when
`Exact::dot_of` declines because a product of two of the direction's own
components lands among the subnormals, and no generator here draws one.

What is left, classified:

| Refusal | Cause | | Demonstration |
|---|---|---|---|
| `CannotNameWhichPieceOfACrossingThisIs` | the two lines a plane cuts from a pipe are not provably either side of the half-space between them | **gap** | the bounded half §29.8 closed; what is left is a plane so close to the pipe's wall that the two lines nearly meet, and the same pair drawn away from the wall is named |
| `CannotNameWhichPieceOfACrossingThisIs` | the best exactly-square normal is not aligned enough with the direction separating two loops | **gap** | the loops are there and are tellable apart; what runs out is `f64`'s supply of vectors exactly square to a given direction. §33.3's floor is `1/√3` and a fat pipe barely wider than what is inside it needs better than that |
| `CannotNameWhichPieceOfACrossingThisIs` | two loops of a ball-and-pipe crossing nearly touch | **gap** | the reach between them is smaller than the picker's own offset rounding; the same pair drawn further apart is named |
| `CannotNameWhichPieceOfACrossingThisIs` | a picking normal is not exactly square and cannot be made so | **gap, and no generator reaches it** | named and not demonstrated. It needs `Exact::dot_of` to decline on two components of a stored direction, which is the subnormal-product family §30.5's second row is about |

§30.5's fourth row — *"a picking normal must be exactly square to an endless
direction and `Plane` scales every normal"* — is **discharged**, and its
demonstration `two_lines_no_f64_half_space_can_tell_apart_are_refused_rather_than_guessed`
is replaced by
`two_lines_off_the_lattice_are_told_apart_by_a_half_space_written_square_to_them`,
which asserts the opposite three ways: the pair is named, the picking normal is
exactly square to the pipe's stored axis, and a point a million million steps
along one of the two lines is still on the side that picked it.

| | at §32 | now |
|---|---|---|
| tests (`cargo nextest`) | 994 | **1001** |
| suite duration | 18.6 s | **94.4 s** |
| doc tests | 19 | 19 |
| `FORMAT_VERSION` | 2 | **3** |
| fuzz corpus | 562 | **699** |

**The suite duration is the one number that got worse, and it is worth reading
rather than glossing.** Ninety of the ninety-four seconds are one test:
`every_point_along_a_crossing_a_hair_from_tangency_is_on_both_the_surfaces_that_named_it`.
Measured by forcing every picking site to refuse, that row walks 33 runs and
44 212 points and takes 1.5 s; with the pickers working it walks 43 runs and
45 442 points and takes 90 s. **The extra thousand points cost eighty-eight
seconds because they are *marched*** — nearly-tangent quartic loops that this
crate used to refuse to name at all, whose per-point cost is the marcher's, and
whose per-point cost in a debug build is about thirty-five times what it is in a
release one, where the whole row takes 2.7 s.

The draws stay at 128. Those thousand points are the most valuable in the file —
they are §24.7's acceptance test applied to exactly the pieces the milestone just
learned to name — and cutting them to buy back CI time would be §14.7's failure
on purpose. What would buy it back honestly is an optimised test profile, and
that is a repo-wide tooling decision rather than a lane's.

### 33.6 Corrections to §30, §31 and §32

- **§32.3's last row** says the half-space "is a new `Surface` variant". It is
  not, and §33.1 is the argument. The row's conclusion — that a change to `Plane`
  cannot reach the gap — stands untouched and is why this lane exists.
- **§32.3's third row** files "test the separation rather than arrange it" as
  unreachable, quoting §29.8: *"a piece that runs on for ever leaves any
  half-space whose normal leans into its direction, however far along, so there
  is nothing bounded to test against."* That is exactly right about two straight
  lines down a pipe, which are endless. **It is not right about two loops of a
  leaning-pipe crossing, which are closed curves and therefore bounded.** Nothing
  turns on it — arranging the squareness is cheaper than bounding the extent of a
  loop that runs a long way up a nearly-parallel pipe, and it is what landed —
  but the sentence covers one case and was applied to two.
- **§32.4's band row**, the 10.79%, is §30.5's **fourth** row reaching the band
  family, not its fifth. §30.5's fifth row — *the half-spaces on a run pick both
  arcs between its ends, or neither* — is a refusal `walk` produces about a
  meridian, and this table only calls `intersect`. It is still a limit, it is
  still `a_seam_that_picks_neither_half_of_its_meridian_or_both_is_refused`, and
  it did not move. The band's intersector rate did, from 10.79% to 3.76%.
- **§31.7 criterion 1** is met, and §32.5's correction of it is withdrawn. The
  test cannot pass about a `Plane` and does pass about a `HalfSpace`.
- **§31.7 criterion 3** is met in both halves.
- **§31.4's** list of what certifies the lane includes `FORMAT_VERSION` to 3.
  It gets there, for a different reason than §31.4 gave: a plane's four numbers
  did not change, and the field that holds a picker did.
- **§31.5 held.** `rcad-predicates` took nothing. `SideOfSurface` is a trait with
  one method over a `Point`, and a new implementor in `rcad-geom` is not a change
  to it — see §33.7.

### 33.7 `SideOfSurface` needed nothing, and the reason generalises

The one thing that could have made this design more expensive than a `Surface`
variant was the side question: if answering *which side of this picker* had
needed a predicate that only exists for `Surface`, the split would have cost
what it saved.

It does not, and the reason is that a half-space is bounded by a plane, so the
question reduces to the sign of `normal · p − offset` over raw coefficients.
`SideOfPlane` was already that expression and nothing in it wanted a unit
normal: its level-1 error bound is a fraction of the permanent, which carries the
normal's own size, and levels 2 and 3 are an interval evaluation and an exact
one. So the predicate was renamed `SideOfAFlatSurface`, given the four numbers
instead of a `Plane`, and shared. `Plane::side_of_point` and
`HalfSpace::side_of_point` are now one line each over the same arithmetic, and
they differ only in the name they log.

`SIDE_OF_A_HALF_SPACE` is a separate provenance name, so a record still says
which of the two gave up.

### 33.8 The trap for lanes 2 and 3

§32.5's trap stands word for word and is the more dangerous of the two: **two
faces that are the same plane are guaranteed to hold the same four numbers, and
everything in the fuse rests on that.** Nothing this lane did weakens it, because
nothing this lane did touched `Plane`.

The new one is its mirror image, and it is one sentence: **a `HalfSpace` has no
canonical form and two of them describing one flat surface may hold quite
different numbers, so nothing may ever compare two of them for equality, dedupe
them, or use one as a key.** `PartialEq` is derived because a test wants to say
"the picker that came back out of the format is the one that went in", which is a
question about *bits* and not about geometry. A lane that finds itself asking
whether two half-spaces are the same flat surface has the wrong type in its hand;
what it wants is `Plane::agreement_with`, and what it should do first is ask why
a picker escaped into a place that decides face identity.

Three smaller ones:

- **`Picker` has no `transformed`.** Nothing composes a location onto an
  `EdgeRun` today — `validate` compares a picker against raw vertex points in the
  solid's own frame — and giving one to a `HalfSpace` would be the second
  authority §17.1 rejected, arriving through a different door. Lane 4 is where
  this comes due, when a fused solid holds edges from two placed arguments.
- **A picker's `Undecidable` is a real answer and rules out nothing.** Both
  `walk` and `validate` already treat it that way, so both pieces come back
  picked and the caller refuses rather than guessing. A new consumer that treats
  `Undecidable` as "not held" silently picks the wrong piece.
- **`FORMAT_VERSION` is 3 and the old bytes still parse.** A version 2 file's
  `Surface::Plane` reads as a `Picker::InFrontOf` without complaint and describes
  the same flat surface with different numbers. The version check is the only
  thing between them, and
  `a_plane_and_a_half_space_facing_the_same_way_are_the_same_bytes_saying_different_numbers`
  is why it has to be.

