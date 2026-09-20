## 26. What M2 actually cost, recorded 2026-09-01

*Written by M2's closing gate, per §25.10's reservation of this number. §12 did
this for M0, §16 for M1 and §17.8 for the fuse.*

M2 is complete. Topology holds curved faces, seams and poles; `rcad-mesh` cuts a
trimmed patch into triangles to a stated deflection; `SolidView::mesh` puts a
whole solid into one point table; `measure` weighs a solid that is not a
polyhedron and says in three separate units how much of the answer the triangles
could be hiding. `validate`'s two named gaps (§23.9) are closed. The suite went
from 796 tests to 860, and the claim audit that closes every milestone ran last
and found two defects and two overstated sentences.

`rcad-predicates` was touched once, at the gate, to add `Cause::FlatStandIn`.
§25.8's freeze held for the whole of the fan.

### 26.1 §23.8's pattern held, and there are seven of them now

§23.8 recorded four defects with one shape: **a helper that meant one thing
until a new kind of input arrived, and then quietly meant another.** It predicted
nothing about how many more there were. Three more turned up:

5. **`bounding_box` was not one** (§25.9). A round bar's two corners are both on
   its seam, so the box round them is a line while the bar bulges a radius out of
   it on every side, and a ball's answer was `None` — which the doc said meant a
   solid with no faces. Renamed `box_around_its_corners`.
6. **`measure` asserted its way to a panic** (§25.5). A round bar's caps fan into
   no triangles at all, so `six_volumes` came out a clean `0.0` and `measure`
   died on its own `assert!(six_volumes > 0.0)`. The assert was right that a
   solid enclosing nothing is broken and wrong about which of the two it was
   looking at.
7. **A ball asked for a rough picture meshed into nothing at all** — found by the
   claim audit, after all four lanes had landed and the suite was green. The
   mesher sized a step by how far it went round in *longitude*, and a ball's seam
   runs pole to pole: half a turn of latitude and no longitude whatever. Past a
   deflection of about `6.6 · radius` the seam came back one step long, the face
   was a rectangle whose only corners were its two poles, every triangle stood on
   one point twice and was dropped, and `mesh` handed back two points, no
   triangles, and a deviation that read perfectly honestly. `measure` then died
   on the *same* assert as item 6, down a road item 6's fix did not cover.

So the pattern held, and it sharpened. In items 1 to 5 the new input was a new
kind of **solid** — the first curved face. In items 6 and 7 it was a new kind of
**number**: a deflection, which is a knob a caller turns and no shape at all.
That is the harder half to see coming, because a new solid arrives with a new
builder and a review, and a new number arrives inside an existing signature.

The audit also found a fourth instance of the §16.10 species proper — three
places where the mesher **asserted** on a shape `Model`'s `Deserialize` can
deliver, one of them inside `validate`, which is the one function in the kernel
whose job is to hand back a refusal rather than crash. §26.5 has them.

### 26.2 §23.7 predicted the risk and thereby defused it

§23.7 named `measure` — shipped in M0, exact, bit-for-bit reproducible, and read
by every metamorphic property in the fuse suite — as M2's schedule risk, on the
grounds that rebuilding it on an approximate mesh puts inclusion–exclusion,
idempotence and absorption downstream of new arithmetic.

**Half right, and the half that was wrong is wrong for a good reason.** It did
not slip there, because the prediction was acted on: lane D left the exact path
untouched, byte for byte, and wrote the mesh path beside it behind
`is_wholly_flat`. `a_flat_solids_measurement_is_bit_for_bit_what_it_was` freezes
six shapes' volume, area and centroid as bit patterns read off the kernel before
the change, and it catches a turned cube routed through the mesh — its volume
moves by an ulp and its centroid much further. A prediction that causes the risk
to be designed around is a prediction working, not a prediction failing, and it
should be scored that way.

The unbudgeted time went to three things nobody had listed:

- **No patch may span more than a quarter turn.** A triangle is measured by its
  longest chord, and a chord stops growing with the turn it spans once that turn
  passes a half — right the way round it is zero again. A ball's patch therefore
  filled with triangles that each stood on one point twice, and the mesh came
  back empty. Half a turn is not enough either. Lane C found this with its own
  oracle, and the claim audit later found the same rule missing in the other
  direction (§26.1 item 7), which makes it the most expensive single idea in the
  milestone.
- **χ of a solid is not `V − E + F`**, twice over: a face with a hole is worth
  `2 − wires` rather than one, and a pole is not an edge. The oracle found the
  first of those on a shape the suite already trusted.
- **§25.4's `max_wobble` reading had to be amended mid-flight**, because taken
  literally it refused every curved solid at every reachable deflection.

**The running score, since §24.12 asks for one.** §12.1 stated the lesson after
the fact rather than predicting anything. §17.6 predicted, against §1.7 in the
same file, and was right. §23.7 predicted and was half right in the sense above.
§24.12's is made and not yet scored. So: **three predictions made in advance, one
right, one half right, one open.** §24.12 also says it is "the first made in
advance", which is not so — §23.7 was written before M2's lanes fanned out, and
this section is what scores it.

### 26.3 §25 was wrong about arithmetic and right about ownership, and that is why it paid

§25 exists because §13 rule 4 says a meaning two people settle separately gets
settled twice, differently. Four lanes ran off it. Three of them reported places
it was silent, incomplete or wrong:

- **§25.6's "`Loop::through` and `triangulate` take it as it stands" was false,
  and is now false three times over.** It was written about a ball's rectangle
  and it is true of the two refusals it names — but the rectangle a ball hands
  over cannot be triangulated *usefully*, because every triangle in it spans the
  seam. Lane C found the quarter-turn cut. The audit found the same rule missing
  in latitude.
- **§25.2's suggested bound shape was false.** "Within a stated multiple of
  `radius · ε`" reads perfectly and is wrong: the angle a pipe reads off a point
  is a difference of terms the size of the whole model, so a pipe of radius one a
  long way out loses the model's worth of bits and not one radius' worth. The
  constant landed as `256 · ε` of *the largest number involved*, and the worst a
  search over forty orders of magnitude can find is `2.4 · ε` of it.
- **§25.4's `max_wobble` reading was amended while the lane was open**, as above.
- **§25.10's file table under-counted a signature change.** Lane D was given six
  files and touched twenty, because `measure`'s new arguments ripple into every
  test that calls it. Nothing collided, because D ran last and alone; had it run
  beside another `rcad-topo` lane it would have.

**Was writing it first still worth it? Yes, and the split says why.** Everything
§25 got wrong was about *arithmetic* — a bound's shape, how coarse is too coarse,
what a budget compares against. Code finds those, cheaply, in the lane that hits
them. Everything §25 got right was about *meaning and ownership* — who refines a
boundary, what a `(u, v)` is for, which of three knobs decides what, where
identity lives, what a mesh of a solid is. Two people settle those differently
and neither is wrong, which is the expensive kind. Four lanes wrote across five
crates with one shared re-export file and collided over nothing; `Cause::FlatStandIn`
went in at the gate and the `rcad-predicates` freeze was real rather than
aspirational. A gate that is wrong about numbers and right about names has done
its job. It should stop suggesting numbers.

### 26.4 The numbers

| | at §25's gate | now |
|---|---|---|
| tests (`cargo nextest`) | 796 | 860 |
| doc tests | 23 | 24 |
| suite, wall clock | 16.0 s | 17.0 s |

What a curved solid costs, release build, 100 mm working size:

| | `validate` | `measure` at the model's own deflection | triangles |
|---|---|---|---|
| cube of side ten | 9.9 µs | 5.4 µs | 12 |
| bar, radius 3, height 5 | 30 µs | 272 µs | 112 |
| ball, radius 3 | 23 µs | 1.31 ms | 840 |

`validate` meshes every curved solid now and did not before, and it costs two to
three times a polyhedron's sweep to do it — because it asks for the coarsest mesh
there is, which is a sign and a count and neither gets truer with more triangles.
`measure` is two orders above `validate` on the same solid for the opposite
reason: it is summed over a mesh at the deflection the caller asked for.

Two published bounds and how much room the drawn cases leave:
`Surface::ROUND_TRIP_ROUNDOFF` is `256 · ε` of the largest number involved and
the worst drawn case uses `2.4`; `rcad-mesh`'s evaluator roundoff is `8 · ε` of
how far the surface reaches and the worst drawn case is inside ten times it, with
a test that fails if a constant ten times too big would also pass.

### 26.5 What the claim audit found, and what it could not break

The audit (`claim-audit`, per §16.10) ran after review, over every "cannot",
"nothing reaches this" and "by construction" the four lanes wrote. Two defects
and two sentences.

**Defects.**

1. **The collapsing ball** (§26.1 item 7). A gap, not a limit: the mesh exists,
   the arithmetic simply did not ask for enough of it. Fixed in the mesher, where
   the floor belongs. `validate` had already met this and worked around it by
   meshing a ball at its own radius instead of as coarsely as the mesher goes —
   and a workaround in one caller is a floor only that caller gets, which is
   exactly why `measure` panicked. The workaround is gone and the sweep is a
   constant again.
2. **Three shapes a file can hold that the mesher asserted on.** Lane C wrote
   *"a curved face with a hole is asserted against; nothing public builds one"*,
   which is §16.10's mistake verbatim: no *builder* makes one, and `Model`
   implements `Deserialize`. §16.11 already records that `validate` is the
   untrusted-import boundary, and since §25 that sweep meshes. The three are a
   curved face with a hole, a wire that comes back to a different longitude than
   it left, and a pole step on a flat face. All three are refusals now —
   `Error::CurvedFaceWithAHoleGoesTooFarRound` and
   `Error::AWireWillNotLieFlatOnItsFace`. The second one panicked *inside*
   `validate`: a ball missing one of its two pole steps is three wire steps that
   chain up and close, so nothing earlier in the sweep has anything to say about
   it.

**Sentences, corrected rather than fixed.**

3. **Is the deflection floor a limit, or does a scaling reach it?** §16.10 exists
   because "no ordering of the arithmetic helps" was true of orderings and silent
   about scalings, so both floors were asked the question directly.
   `Error::DeflectionBelowWhatAMeshCanReach` is a **limit**: it is a fixed
   multiple of the solid's own coordinates, so shrinking the whole model shrinks
   the floor in exact step and nothing *relative* is ever refused, and no
   ordering, rescaling or compensated sum puts a point between two neighbouring
   `f64`s. `rcad-mesh`'s `NoMeshCanGetThatCloseToTheSurface` said "no mesh" and
   meant "no mesh built from points bounded the way this crate bounds them" — a
   pipe is invariant along its own axis, so one whose axis origin sits a billion
   units up it is refused a deflection it could comfortably meet. That is a
   **gap in a bound rather than in an answer**; closing it needs the axis
   direction, because an error along `z` moves a point off a `z` pipe by nothing
   and off a slanted one by all of it. `rcad-topo` never builds one —
   `add_round_bar` puts the axis at the origin and carries the placement on the
   solid — so it is named and left, which is what §16.10 asks for when the family
   of fixes is known and the consumer is not.
4. **`validate` claims §23.4's oracle and checks something weaker.**
   `every_side_of_the_skin_is_used_both_ways` counts a *balance* and not a count,
   so a side used four times, twice each way, would pass. There is no input the
   kernel can build that fails the count without also failing the balance, so
   there is no test that could fail if it were strengthened — and a change with
   no failing test is not a change (`CLAUDE.md` rule 2). The sentence says so
   now, in both the sweep and the suite's own copy of the oracle.

**Two of the four flagged claims survived, and here is how hard they were
pushed.** Lane C's ball-seam comment says a ball whose seam went elsewhere would
get "a watertight mesh of the right ball cut in the wrong place", and that
nothing today could tell the difference. Built: a ball whose `EdgeRun` records
its seam a quarter turn from where `add_ball` puts it meshes into 276 triangles,
passes the whole combinatorial oracle, validates clean and measures the volume it
should. Nothing can tell, and the reason is that a sphere looks the same from
every meridian and only that one face uses that edge. It is a **gap** all the
same — the longitude is in the model, on `EdgeRun::PartOfWhereItCrosses`, and the
mesher does not read it — and it becomes a wrong answer the moment a second face
meets a ball along that seam. That is M3's, and §26.6 hands it over.

Lane D's `Undecidable` arm out of a triangulation is a **gap** and could not be
reached. §2.5 says `Undecidable` arrives through *constructed* points, and every
corner handed to the mesher today is the projection of an exact stored corner; the
only other road is a coordinate product overflowing or falling subnormal, and
`Profile::build` runs the same exact predicate on the same corners first, with the
triangulation's own `2^-142`–`2^201` window closing the rest. Probed across forty
orders of magnitude of radius and height, through the public builders, in every
combination: **not one panic and not one `Undecidable` — every extreme came back
as a named refusal.** So the arm is correct, correctly classified and currently
unreachable, and the first thing that reaches it is a mesh domain holding a
constructed corner, which is M3's crossings.

### 26.6 What M2 hands to M3

§24.4 says M3 does not open until curved `measure` has landed. **It has landed**,
with error terms in three units and its exact path frozen bit for bit, so that
precondition is met and M3's gate may open on §24.4's four items. Six things M3's
gate must now also deal with, which §24 did not know:

1. **The mesher can only walk an edge that is a straight line in the surface's
   own two coordinates.** That is exactly right for the three edges this kernel
   builds — a pipe's ruling, a cap circle, a ball's meridian seam — and false in
   general: a circle cut out of a ball by a plane that misses the centre is not
   one. §24.2 makes the mesher M3's consumer, so **M3's first crossing is also
   the first edge the mesher cannot walk.** This is the largest single item on
   this list and it is not in §24.
2. **A ball's seam longitude is readable off `EdgeRun::PartOfWhereItCrosses`, and
   §24.4 item 2 widens that very field from `Plane` to `Surface`.** The two are
   the same edit arriving from two directions. Whoever widens the field should
   also make the pole-to-pole walk read the meridian out of it instead of
   assuming longitude zero; doing it at the gate costs a few lines and doing it
   after M3's first sphere–plane crossing costs a debugging session.
3. **`validate` meshes now, so a new `Surface` variant is nine stops and not
   one.** Three in `rcad-topo/src/mesh.rs`, one in its `measure.rs` and five in
   `rcad-mesh/src/triangulate.rs` all match on `Surface` and panic on an arm they
   have not been taught — which is the right answer to a broken workspace
   invariant and a long list to walk. §24.6 says the surface set does not grow in M3, so
   nothing is due — but the cost of growing it has gone up and should be counted
   before, not during.
4. **`Undecidable` out of `mesh` becomes reachable when a mesh domain first holds
   a constructed corner** (§26.5). M3's crossing points are the first. The arm
   and its provenance record already exist; what does not exist is a test, and
   M3 is the milestone that can write one.
5. **`rcad-mesh`'s evaluator-roundoff bound over-charges a pipe placed along its
   own axis** (§26.5 item 3). Unreachable from `rcad-topo`. If M3 ever hands the
   mesher a surface it did not get from a `Model`, this becomes reachable, and
   the fix needs the axis direction.
6. **`measure` over a mesh still asserts `six_volumes > 0.0`.** After §26.5's fix
   nothing found can reach it — the coarseness sweep now runs to a hundred
   thousand times the solid, and `validate` meshes every curved solid as coarsely
   as the mesher goes and checks the sign. It is an assert on a number derived
   from a caller's deflection, and that is the shape §26.1 items 6 and 7 both
   had. Worth a second look the next time a new kind of surface arrives.

Carried forward unchanged from §23.9's third item: **arcs in a profile are still
their own piece**, and every 2D self-intersection check in `build.rs` would have
to become arc-aware. Nothing in M3 needs one.

### 26.7 Corrections to earlier sections

- **§24.12**: "this is the first made in advance" is not so. §17.6 named the
  fuse's risk against §1.7 before the work, and §23.7 was recorded before M2's
  lanes fanned out. §24.12's is the third made in advance and the only one still
  open; §26.2 keeps the score.
- **§25.5**: `measure`'s blanket refusal of a curved solid came back out when
  lane D landed, as that section said it would.
- **§25.6**: "`Loop::through` and `triangulate` take it as it stands" is true of
  those two refusals and not of the mesh that results. See §26.3.
- **§25.10**: the file table is a statement about the lane's *own* files and not
  about the ripple of a signature it changes. See §26.3.
- **§23.9**: all three gaps are addressed. A round cap facing outwards and a
  curved solid's volume are both checked over the mesh in
  `check_shells_and_solids`; arcs in a profile are deferred again, at item 3
  above.

---

