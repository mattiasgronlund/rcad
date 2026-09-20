## 116. The refusal's surfaces are the caller's, not the mesher's, fixed 2026-09-17

§113.6 recorded a published field that lies, one day old: the `surfaces` on
`Error::CannotFollowACrossingWhereItsSurfacesAreTangent` reach a caller of
`SolidView::mesh` in the mesher's own shifted frame. This is the fix and the
argument for choosing it over the other two.

### 116.1 What was wrong, said in numbers

`crates/rcad-topo/src/mesh.rs`'s `BuildingAMesh::walk` works a face's edges out
in `the_frame_this_face_is_meshed_in`, which for a **placed** face is the
placement inverted and for an unplaced one a shift that brings the numbers near
the origin (§17.3, §39.7 item 2). The points ride back out on
`back_out_of_the_frame_this_face_is_meshed_in`. The refusal did not.

A ball of radius one placed at `(1024, 2048, 4096)`, its seam given a pipe of
the same radius through the centre and square to the meridian, refuses. Before
this change the refusal carried

```text
Sphere { centre: Point([0.0, 0.0, 0.0]), radius: 1.0 }
```

and the model holds that ball at `Point([1024.0, 2048.0, 4096.0])`. The origin
is not where any model has a ball; it is where this file's own private frame
put one. `a_placed_solids_tangency_refusal_names_its_surfaces_where_the_model_holds_them`
in `mesh.rs` is that measurement, and neutralising the one call this section
adds is what prints the line above.

**Both variants had it, not one.** §110.1 split the shortening budget out into
`CannotFollowACrossingBecauseNoStepStaysOnIt` on the day the field landed, and
it carries the same `Box<[Surface; 2]>` through the same `map_err`. §113.6
names only the tangency one because that is the one the corpus census was
about. A fix that moved one would have left the other publishing the mesher's
frame, so both move.

### 116.2 Three fixes were open, and why this one

**Chosen: move the pair back into the caller's frame before the refusal leaves
the mesher.** `Error::with_its_surfaces_moved` is a new method on
`rcad_geom::Error`; `mesh.rs` calls it with the movement the points already
ride out on. The field then means what its name says everywhere: the two
surfaces, in the frame whoever asked was working in. A direct caller of
`rcad_geom::walk` already got that and is untouched.

**Rejected: name the frame in the type.** Either a new `Isometry` field beside
the pair, or a doc comment saying which frame the numbers are in. Both fail on
the same point, and it is about the **public API** rather than today's callers
(§16.10): the frame has no name a caller can say. `the_frame_this_face_is_meshed_in`
is a private function of `rcad-topo`'s `mesh.rs`; its answer depends on whether
the face carries a placement and, when it does not, on a per-coordinate shift
`a_shift_that_loses_nothing` works out from that face's own corners. A doc
comment can therefore only say "the frame the caller of `walk` was in", which
tells a caller of `SolidView::mesh` nothing it can act on — and a caller of
`mesh` never called `walk`. The field variant is worse still: it turns a
one-step answer into a two-step one whose second step is optional, and the
whole reason §110.1 added the pair was that a caller holding a solid had **no
way back** to it. An answer that is only right if you remember to apply a
transform is the same trap in politer handwriting.

**Rejected: remove the field.** It is the strongest of the three arguments —
a value nobody can interpret is worse than no value — and it only holds while
the value cannot be interpreted. §110.1's reason for the field stands: four
raise sites used to be one unit variant, and a caller matching the name has no
other road to the pair. Removing it would also give back nothing, because the
pair is recoverable: the mesher knows the movement, it is already computing it
one line later for the points, and applying it costs two surface transforms on
a path that has already refused. Deleting a field to avoid a call that was
sitting there is not a trade.

### 116.3 Moved, not swapped, and the seam is why

The mesher knows exactly which two surfaces it handed the marcher, so it could
put **those** back instead of moving what came out — exact, no rounding at all.
It must not.

`rcad_geom::walk` reads the second surface off the run itself when an edge has
one face on both sides, which is what a seam is — `the_two_surfaces_either_side`
lists the one face twice on purpose, and says so. So the pair the refusal is
about is not always the pair that was passed in, and a swap would hand back a
surface the marcher never looked at, confidently. That is §4.1's ranking: a
wrong answer below a refusal. Moving names the pair that was actually refused
about, and pays a rounding for it.

### 116.4 What the move costs, measured

`crates/rcad-geom/tests/what_moving_a_refusals_surfaces_costs.rs`. A ball of
radius one centred at `(3, 5, 7)`, moved by a movement and back by its inverse:

| movement | how far the centre came back from where it started |
| --- | --- |
| a shift of `(1024, 2048, 4096)` down the axes | `0` — bit for bit |
| the same shift with a `0.25` radian turn about `(1, 2, 3)` on it | `1.1368683772161603e-13` |

So a caller matching a refusal's pair against surfaces it already holds gets
the bits back under a pure shift and does not under a turn. **That matters to
one plan and not to this fix**: §110.4 wanted to match the pair against the
table of faces `rcad-step`'s reader keeps, by value. Under a turn that match
has to be by shape and not by equality. The stray is about a rounding of the
coordinate it is measured at — `1.14e-13` against a position `4.7e3` from the
origin is `2.4e-17` of it — and it is the **surface's** round trip, not the
movement's: `back_out_of_the_frame_this_face_is_meshed_in` is already the exact
inverse of the frame rather than that frame inverted, for the ulp §17.3's own
doc comment measures.

**A first version of this measurement said turns were free**, because it moved
a ball centred on the origin and `R·0` is `0` however awkward `R` is. The
number above is from a ball that is somewhere.

### 116.5 The fixture is an honest tangency, and it is not a rounding

A ball and a pipe of the ball's own radius, the pipe's axis through the centre
and square to the meridian, **touch** all the way round that meridian rather
than crossing it. At every point of it `how_far_off_and_which_way` gives the
ball `out/‖out‖` and the pipe `sideways/‖sideways‖`, and `sideways` is `out`
with nothing taken off it — the point is square to the axis there, so that dot
product is exactly zero. The two normals are the same bits, `‖u − v‖²` is
exactly zero, and `settled` has nothing to divide by. That is
`WhereFollowingACrossingGaveUp::SettlingAPointOntoBothSurfaces`, the site all 44
corpus files of §110.3 stop at, reached by construction rather than by a corpus.

It is a **limit** and not a gap (§16.10): there is no direction to be had at any
precision, because the two surfaces do not cross there at all.

**The route is `Deserialize` and not a builder**, and that is not a shortcut.
`SolidBuilder::build` walks the crossing in its own sweep and refuses this
description with `ADraftEdgeHasACrossingThatCannotBeFollowed` before a model
exists — so there is no public builder road to a model that meshes this far and
then stops. `SolidView::mesh` is public in its own right and `Model` is
`Deserialize`, which is the same argument the two tests beside it make (§16.11).
That is also why the test lives in `mesh.rs`'s own test module rather than under
`crates/rcad-topo/tests/`: `Model::data_mut` is `pub(crate)`, and the three
neighbouring "refused rather than asserted against" tests are there for exactly
this reason.

### 116.6 What is left

`walking_along_this_edge_refused`'s doc comment in
`crates/rcad-topo/src/error.rs` still reads as though matching a refusal's pair
back to a model by value were hopeless — *"the mesher walks a face in a frame of
its own, so the surfaces that reach the marcher are a rigid motion away from the
ones the model stores"*. Every sentence of it is still true **of the marcher**,
and the conclusion a reader draws from it is now false: what leaves the mesher
is in the model's frame. That file belonged to no lane of this unit and was not
edited here. It is a doc comment that went stale the same way §110's `# Errors`
clause did within hours of being written.

**Corrected while this section was merged**, by the dispatcher rather than by a
lane, since no lane of this unit owned the file. The paragraph now says what is
true of each half separately — the marcher does still work in a frame of the
mesher's own, and the pair no longer leaves the mesher in it — and points at
§116.4 for what a match by value costs, which is nothing for a shift and about
a part in `1e13` for a turn.

Not attempted here: §110.4's other half, carrying the **face keys** so
`rcad-step` can turn a pair into two `#id`s. Moving the surfaces makes a match
by value possible where the placement is a shift and not a turn (§116.4), which
is better than it was and is not the same thing as carrying the keys.

