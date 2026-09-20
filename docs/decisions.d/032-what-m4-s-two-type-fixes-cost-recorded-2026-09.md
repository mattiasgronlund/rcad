## 32. What M4's two type fixes cost, recorded 2026-09-01

Lane 1 of §31.6: the expansion slide and the two types, against a still-flat
fuse, before `Error::FuseNeedsFlatFaces` comes off.

Two of the four criteria are met, one is met by a different route than the gate
named, and **the fourth cannot be met at all in the shape §31.4 gives it** —
which is the finding this section exists for.

### 32.1 The expansion slide, and the row it moved is not the row §31.4 named

§31.4's first half went in as written. `Exact::square_minus` slides a number
into the window where squaring it is exact, slides the other side of the
comparison by twice as much so the two stay comparable, and slides nothing
back: a sign is unchanged by multiplying by a positive power of two, and
multiplying an expansion by one is exact. The slide is **zero wherever the
square already fits**, so every comparison that was decided yesterday is decided
through exactly the same arithmetic today.

The window §30.4 published now has the test §15.6 asks for, and it is a
demonstration rather than a paragraph:
`an_expansion_wider_than_the_window_cannot_be_squared_at_any_scale` writes an
expansion one binary order wider than the window at **every** placement in the
normal range — 1050 of them — and is refused at all of them, and one exactly as
wide as the window is squared at every placement. The two ends are `2^-484` and
`2^512`, so the widest spread that fits is 995 binades.

**The turned quartic family's `Undecidable` rate went from 4.64% to 0.00%.** All
95 refusals were the 505-order expansion, and the slide reaches every one of
them. Over twenty times the draws the rate is 0.03% — fourteen of 40 961 — and
those are expansions whose own spread is past the window, which no scale
narrows.

**§31.4's other named row does not move, and this is a correction rather than a
regression.** §31.4 says the slide also brings down §30.7's 10.49% on pairs
written at `2^k`. It does not, and cannot: those 215 refusals are named
`where a plane and a ball cross`, `where a plane and a pipe cross`,
`where two balls cross` and `where two planes cross`, and every one of them is
`Exact::product_of(radius, radius)` refusing two raw `f64` whose product would
land among the subnormals. There is no expansion yet to slide. What reaches that
row is scaling the *inputs* — the working frame §24.8 defers and §31.9 says
still has no figure asking for it. It has one now, and it is 10.49%.

**One test had its number re-derived and one had its draws re-derived.**
`A_TURNED_QUARTIC_CROSSING_MAY_REFUSE` came down from 8% to 0.2%, which is six
times the 0.03% measured over twenty corpora and four draws of the shipped 2048.
And `every_turned_quartic_crossing_that_refuses_was_exactly_tangent_before_the_turn`
now draws twenty times what the rows beside it draw, because the refusal it is
about fell by the same factor and 2048 draws reach none of it — a property with
nothing to measure passes for the wrong reason, which is what §14.7 is about.
All fourteen refusals it does find are still knife edges a rotation made. (The
fourteen went to none at §118.11, and **§119 retired the property** for exactly
the reason this paragraph names: its vacuity guard is what failed.)

### 32.2 `Axis` had §22.6's defect and now does not

`Axis::new` normalized the direction it was handed without dividing by the
largest component first, so what it stored depended on that vector's length and
not only on its direction. Re-measured over every integer direction in `-6..=6`
with a length, against every whole-number multiple of it from two to
ninety-seven: **80 396 of 210 816 pairs — 38.1% — stored a different direction
for the same line**, and reducing first brings that to none of them.

§31.11's last correction records 58% from a different draw; the two are
different families and both are honest. This one is exhaustive over a lattice
and reproduces exactly.

`the_same_pipe_built_from_a_longer_axis_direction_is_the_same_pipe` is the
consumer: two solids flush against each other on one cylinder describe that
cylinder twice, off two builders, and `intersect` answered `Curves` where the
truth is `SameSurface`. It answers `SameSurface` now, for multiples of two,
three, five, seven, eleven and ninety-seven, and for the same pipe pointed
backwards.

`Vector::scaled_to_length_one_from_anywhere`'s doc claimed the first division is
exact. It is not — `a/|m|` is one correctly rounded division — and what is exact
is that the largest component comes out as `±1`. Corrected in place.

### 32.3 A plane can be canonical under scaling or it can keep a normal square, and not both

**This is the section §31.4 asks for and the answer is not one of its five.**

§31.4 got the diagnosis right and §31.11's correction of §29.8 stands:
`Plane::reduced_by_largest_component` loses exact squareness on its own, before
the normalization anyone had blamed. `a_normal_written_square_to_a_direction_stops_being_square_once_a_plane_stores_it`
is that, on the three normals a caller can write exactly square to `(1, 3, 5)` —
and two of the three read exactly `0.0` in plain `f64` afterwards while the
exact level sees a sign, which is why it hid.

§31.4's family of fixes turns on reducing by a **power of two**, which rounds
nothing. Half of that is right, and
`a_power_of_two_reduction_keeps_a_normal_square_and_stops_one_wall_being_one_plane`
shows both halves:

1. All three normals written square to `(1, 3, 5)` are **still exactly square**
   after a power-of-two reduction.
2. Two multiples of one wall — `(0, −1, 0)` and `(0, −3, 0)` — reduce to
   `(0, −1, 0)` and `(0, −1.5, 0)`, which differ by a factor that is **not** a
   power of two. One rotation later they are not exactly proportional at all,
   because every component of each is rounded on its own, and
   `Plane::agreement_with` answers `Different`, correctly, about two faces that
   are one wall.

**A canonical name beside the stored numbers does not reach the second half,
which is what §31.4's candidate 4 assumes.** §31.4 names three call sites that
compare bits. There are more than three, and the ones that matter are not
comparisons of bits at all: `Plane::agreement_with`, `planes_are_parallel` and
`relation_of_two_lines` are *exact predicates over the coefficients*, and the
two coefficient vectors above are genuinely not proportional, so no key
computed from them can say otherwise without inventing a tolerance.

Measured, with the whole of candidate 4 in place — power-of-two storage, a
`Plane::canonical_coefficients` key, the three named call sites moved onto it,
and a fourth found on the way (§17.4's "keep once" cell picks one of two
coincident pieces by argument index, which stops being order-free the moment two
coincident faces can hold different numbers):
`swapping_the_two_sides_of_a_turned_pair_changes_not_one_bit` **refused 112 of
its 136 fuses, 106 of them naming `where three planes meet`, and could compare
12 of the 124 pairs it set out to compare against a floor of three tenths.**
Two walls that are one wall each cut the other, and three planes that share a
line have nowhere to meet.

Before that it compared all of them. The lattice control was worse still: it did
not refuse, it **panicked** — `the two coedges of an edge must run opposite
ways`, which is a broken solid rather than an honest refusal, because a "keep
once" that picks by the plane drops both of a coincident pair in the one cell
where the two are not interchangeable.

The general statement, and it is short. Canonical under scaling means dividing
by something the direction alone decides. Exact means dividing by a power of
two. Two multiples of one vector have the same power of two only when the
multiple is itself a power of two. **So the two properties cannot both hold**,
and §31.4's "you cannot have both" is right about more than it knew.

**What lands is candidate 5, and the half-space gap stays.** Its rates are
re-measured below and its classification is corrected: it is still a **gap**,
because a separating half-space exists, but it is not a gap any change to
`Plane` reaches. The family of fixes, and why each is the only one left:

| | |
|---|---|
| Change what `Plane` stores | Demonstrated above not to be one |
| Pick a different square normal | A caller *can* write one — swap two components of the direction and negate one, and the dot product is two products that are exact negatives. Measured over 7201 turned pipe axes, **10.5% of them have at least one of the three such normals survive being stored**, against 1.7% for the cross product the code writes today. Not a closure, and a tenth of a percentage point on the planar row is not worth the branch |
| Test the separation rather than arrange it | §29.8: a piece that runs on for ever leaves any half-space whose normal leans into its direction, however far along, so there is nothing bounded to test against |
| A half-space that is not a `Plane` | **The one that reaches it**, and it is a new `Surface` variant — which §24.6 says touches `validate`, the mesher, `is_wholly_flat`, the fuse and serialization. A milestone's decision and not a lane's, so this lane reports rather than takes it, the way §31.5 has a lane report rather than take a change to a frozen crate |

`FORMAT_VERSION` stays at **2**. §31.4 expected it at 3 because a plane's four
numbers would change; they do not.

### 32.4 The numbers

Family by family and never blended (§17.7). 2048 pairs per family,
`Policy::certified()`.

| Family | `Undecidable`, at §30.7 | now | no half-space, at §30.7 | now |
|---|---|---|---|---|
| lattice, planar | 0.00% | 0.00% | 0.00% | 0.00% |
| lattice, quartic | 0.00% | 0.00% | 0.00% | 0.00% |
| turned, planar | 0.00% | 0.00% | 0.44% | **0.44%** |
| turned, quartic | 4.64% | **0.00%** | 10.93% | **11.86%** |
| leaning | 0.00% | 0.00% | 0.24% | **0.20%** |
| the band | 0.00% | 0.00% | 10.79% | 10.79% |
| written at `2^k` | 10.49% | 10.49% | 3.95% | 3.95% |

**The turned quartic row's half-space count went up and its total went down.**
Nineteen pairs that used to refuse for want of a sign now get as far as being
told they are two loops, and refuse there instead. Taken together that family
refused 15.57% and now refuses 11.86%.

The leaning row moved from 0.24% to 0.20% because §32.2 changed what a leaning
pipe's stored axis direction is.

| | at §31 | now |
|---|---|---|
| tests (`cargo nextest`) | 985 | **994** |
| suite duration | 18.3 s | 18.6 s |
| doc tests | 19 | 19 |
| `FORMAT_VERSION` | 2 | 2 |
| fuzz corpus | 562 | 562 |

### 32.5 Corrections to §31, and what lanes 2 and 3 have to be told

- **§31.4** says the expansion slide brings down §30.7's 10.49% on pairs written
  at `2^k`. It does not; §32.1 says why, and that row's fix is the centred frame
  §31.9 files as having no figure asking for it. It has one.
- **§31.4's five candidates for `Plane`.** Candidate 2 the gate already showed is
  not a fix. Candidates 1, 3 and 4 are not fixes either, and §32.3 is the
  demonstration. What lands is candidate 5.
- **§31.4** lists three call sites that compare a plane's bits. The count is at
  least four — `select.rs`'s "keep once" cell is the fourth — and counting them
  is the wrong question anyway, because the ones that decide are exact
  predicates rather than bit comparisons.
- **§31.7 criterion 1** names
  `a_half_space_written_square_to_a_direction_is_still_square_to_it_once_stored`.
  That test cannot pass, and what stands in its place is the pair of
  demonstrations in §32.3, written §19's way round: they assert what the code
  does, so a fix would fail them loudly.
- **§31.7 criterion 3** asks for the half-space figures to move. They have not,
  and §32.3 says what would move them.
- **§31.11's last correction** puts `Axis::new`'s cost at 58% of drawn
  proportional pairs. Re-measured exhaustively over a small lattice it is 38.1%;
  both are honest measurements of different families.
- **The trap for lanes 2 and 3**, and it is one sentence: **two faces that are
  the same plane are guaranteed to hold the same four numbers, and everything in
  the fuse rests on that.** It survives because `Plane` still scales its normal
  to length one. Any lane tempted to change what a plane stores — to make a
  normal exactly square to something, to widen a range, to save a division —
  should read §32.3 first, because the failure is not local and does not look
  like the change that caused it.

