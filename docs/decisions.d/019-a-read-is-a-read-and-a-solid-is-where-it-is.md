## 19. A read is a read, and a solid is where it is

*Recorded 2026-08-31, fixing the three defects §17.9 recorded and not fixed.*

All three were gaps (§16.10) and all three are fixed. Between them they took
three more claims down with them, which is the part worth reading: each of the
three was load-bearing for a sentence somewhere else that turned out to be
false.

### 19.1 Reading a plane out of a model moved it, twice over

`FaceView::plane` is `surface.transformed(&location)` whatever the location is,
so **every read of every face pays whatever that function does to a plane**, and
a chain of fuses pays it once per fuse. `Plane::transformed` was doing two
things to a plane that `Isometry::identity()` should have left alone:

1. It rebuilt the offset as `moved_normal · movement(origin())`. `origin()` is
   `normal * offset`, so the round trip computed `offset · (nx² + ny² + nz²)`,
   and a squared length of `1.0000000000000002` — an ordinary one — walked the
   offset an ulp further out on every read. §17.9 found this.
2. It scaled the moved normal back to unit length. **Scaling a vector to unit
   length does not always give a vector whose length reads back as exactly
   one**: the direction `(6, 3, 7)` normalizes to something whose computed
   length is `0.9999999999999999`, so a second scaling grows all three
   components. This is the half §17.9 did not see, because the reproduction it
   had used a normal whose squared length rounded to exactly `1.0`.

The offset is carried over now — a turn leaves `normal · point` alone, so all a
shift can do is move the plane along its own moved normal — and the moved normal
is not scaled again. It was scaled once, where the plane was built.
`Axis::transformed` had the second defect too, so a pipe read out of a model
turned a little; same fix.

**What this cost, measured rather than argued.** `Plane`'s own documentation
already said that scaling an almost-unit normal "can move the last bit or two,
so a plane that has been written and read back is the same plane to within
rounding, not bit for bit" — about serialization. Doing it on every read made
that sentence true of reading, which is the thing §17.1 chose plane-authoritative
geometry to prevent.

### 19.2 The oracle that could not run, and the property that was too strong

`every_plane_of_the_answer_is_a_plane_of_an_input` — §17.1 stated as an
assertion — ran on about two thirds of this tree's fuses and was switched off for
the turned families, because there it would have measured the reader rather than
the fuse. It runs on all of them now, and **the turned families are where it
bites**: a turned normal is not a signed unit axis and nothing about it is exact
by construction. Widening it is what caught defect 1's second half, within a
second of wall clock.

The other half went the other way. `uniting_a_turned_solid_with_copies_of_itself_never_moves_a_plane`
asserted that `A ∪ A` gives back `A` bit for bit, corners included.
(It is `uniting_a_turned_solid_with_more_copies_of_itself_settles_after_the_first_union`
now, for the reason below.) **That is
not what §17.1 asks**, and the plane defect had been hiding the difference. §17.1
says the plane is authoritative and a stored corner is a *cached sample* of it,
and that the first fuse re-derives every corner as the meet of the planes around
it. So the first union is entitled to move a corner. Measured on a unit block
turned a sixty-fourth of a turn: the input cached the far corner's `y` as
`1.0932018670017576`, the exact meet of that corner's three stored planes is
`1.09320186700175740…`, and the fuse stores the `f64` nearest it,
`1.0932018670017574`. **The cached corner is the ulp that was wrong**, and
correcting it is the whole point of re-deriving.

What §17.1 forbids is that correction *repeating* — "the rounding in a vertex is
the rounding of one Cramer solve, never a sum over the operation history" — so
the property now measures from the first union, and every union after it has to
change nothing at all. The claim it used to make was strictly stronger than the
design and would have forced the fuse to prefer a stale corner over a correct
one.

### 19.3 The refusal rate does move, and by how much

§18.7 records a change that was obviously going to improve criterion 8's number
and measurably did not. This is the other outcome, on the same harness and the
same corpus of 414 turned fuses:

| | before | after |
|---|---|---|
| Turned pairs, the headline family | 105 refused = **25.4%** | 87 refused = **21.0%** |
| Not a solid, same family | 16 = 3.9% | 21 = 5.1% |
| The three control rows | 0 refused | 0 refused |

The corpus was held fixed for that measurement: two proptest regression seeds
were added by the failures the widened oracle found, and both figures above are
from before they went in. It is the same sentence §17.7 already wrote about why
that family refuses at all — this corpus is the lattice family *turned*, and the
lattice family is full of configurations that are exactly degenerate, so turning
them leaves two coincident faces whose stored planes disagree in the last bits.
The reader was manufacturing that disagreement. Eighteen of the 105 refusals
were it.

Five more answers are now not-a-solid rather than refusals, which is a real
change of outcome and not a change of shape: those fuses now get far enough to
find out that what they were asked for is not something a `Model` can hold.

### 19.4 A distance is not a length that scales

`InPlane::of` needs a point it can prove is off a face's plane, and stepped
along the normal by `1.0`, then `1024.0`, then `1048576.0`. Past an offset of
about `2^72` the `f64` spacing is wider than the largest of those, every
candidate lands back exactly on the plane, and the fuse refuses a fuse it could
plainly answer. The doc comment on that function spelled out its own
counterexample — "a bigger reach only helps when the plane's own offset is large
enough to swallow a smaller one" — and then hard-coded three reaches anyway.

The reach is a fraction of the plane's own offset now — the only length in sight
that scales with the plane — with a floor of one. **Both halves of that sentence
were measured rather than reasoned, and one of them was a surprise.** Dropping
the floor and stepping off a plane `0.05` from the origin by `0.05` leaves the
reference point so close to the face that the in-plane winding fan cannot be
decided, and the headline refusal rate goes from 20.7% to 37.9%: 72 more fuses
that could not place a point inside a piece of a face. The reference point has
to be off the plane *and* far enough off it to be worth measuring turns against,
and only the first of those is obvious.

The second try is the same distance the other way, and it is what a plane out at
`1e308` needs, where stepping further from the origin overflows and stepping
towards it cannot. Stepping towards the origin *first* was measured as well: it
is the same 37.9%, because a face far from the origin then gets a reference point
a whole offset away from it and the fan degenerates again. **This is why there is
no limit claim here.** The obvious sentence to write was that past about
`f64::MAX` both steps overflow and the point being looked for does not exist —
and the fix family that sentence would have hidden is the one now used as the
second try, which CLAUDE.md's rule about impossibility claims exists to make
somebody go and look for.

`signed_volume` and `measure` hung their tetrahedra from the world origin, so a
solid `2^m` out summed terms of size `2^{3m}` to reach a volume of order one and
lost every digit past `m = 28`. They hang from a corner of the solid now, which
makes the three subtractions exact by Sterbenz's lemma whenever the coordinates
share an exponent — which is exactly the case that was worst. **The corner is
the smallest one and not the first the arena hands over**, and that distinction
is not fussiness: two routes through a fuse reach the same solid with its faces
in different orders, and the first version of this fix made the centroid depend
on which came first. The associativity property caught it inside one run.

**Where the ranges end now, all measured.** A cube fuses with a copy of itself
out to `2^340`; at `2^341` six times its volume is past the largest `f64`. A
`3 × 3` frame encloses exactly eight out to `2^50`; past `2^53` a wall one unit
thick has both its corners on the same number and there is no frame to fuse.
Both ends are limits — the answer stops existing in the number type — and both
have a test that says which. `a_pair_somewhere_else` reached `2^30` because of
these two gaps and now reaches `2^280` against a measured edge of `2^339`, with
its shift raised from `2^24` to the `2^30` its own doc always said the dynamic
range allowed. §17.7's four measured rows do not move when it is widened.

### 19.5 Three claims that fell out with the fixes

Each was found by fixing something else, and each is the §16.10 shape: an
argument that was locally valid about a subtly different question.

**Criterion 7's demonstration was demonstrating the bug.**
`a_fuse_that_needs_a_decision_nothing_can_prove_refuses` built two towers `1e200`
tall and said "nothing in `f64` can multiply coordinates that far apart without
overflowing, so a decision the answer depends on cannot be proved". The fuse
never reached a predicate: `InPlane::of` could not step off the towers' top
plane. They fuse now, and enclose `1.75e200` as they should. The demonstration
moves to `1e308`, where `orient3d` genuinely overflows and the records say so —
and the test now also asserts that no record names `"a point off a face's
plane"`, so the demonstration cannot quietly become a bug again. §15.8's
distinction between a refusal and a *deserved* refusal, applied to a test rather
than to a rate.

**`check_dynamic_range` could not see where a solid is.** It read
`vertex.point` straight out of the arena, and a solid placed a long way out
keeps ordinary small numbers there and carries the distance in its location. So
a frame at `2^39` passed the check, and the same geometry failed it after a
fuse, because a fuse bakes placement in (§17.3) — the same model, two answers,
decided by where the placement happened to be written down. What the check is
about is the numbers the predicates see, and those are located. This is §16.11
again and worse than §16.11 thought: the ratio could not see absolute magnitude,
and it could not see the magnitude either.

**A debug sweep was blaming the fuse for what it was handed.**
`Assembly::commit` runs `Model::validate` over the whole model after writing and
panics on failure. The sweep covers parts of the model no fuse looked at, and
§4.3 is explicit that a caller may hold a model that does not pass it, since the
sweep runs at test and import boundaries and not after every mutation. So a
caller with one out-of-range solid got a panic from an unrelated fuse. It now
validates before writing too and asserts only when the model arrived sound,
which is what the panic message always claimed: *a fuse writes a valid model or
does not write at all*.

### 19.6 What this does not fix

- **The lower end of the scale range is still a limit and still there.** Below
  about `2^-359` unshifted, `orient3d`'s products fall subnormal, level 1
  declines by design and expansion arithmetic will not certify a product it
  cannot represent. §12.2's precondition, arriving end to end, and the honest
  answer.
- **`DynamicRange` is still a ratio.** It now measures the right coordinates,
  but §16.11's question — that a ratio cannot see absolute magnitude — is
  answered only in the sense that the two things it broke are fixed. A model at
  `2^300` with features at `2^290` still reports a comfortable ratio.
- **Nothing here widens the shape vocabulary.** §17.8 measured that three
  million fuses over the existing generators found nothing and one new shape
  family found a defect in forty; that is still the best-value work available,
  and generator families 3 to 6 are still unbuilt.

---

