## 123. The loops-touch refusal says where, and the `FaceKey` half is not this lane's, built 2026-09-18

§113.7 recorded a third bucket and nobody chasing it: files that get past the
tangency wall and stop in the mesher instead, publishing

```text
TheSolidTheFileDescribesIsNotOne { solid: #15, why: CannotMeshAFace {
  face: …, why: LoopsTouch { corner: 7, side: 4 } } }
```

Two complaints are folded together in that line, and they come apart cleanly.
`corner: 7, side: 4` is the mesher's own, and this section fixes it. `face: …`
is a `FaceKey` of a throwaway model, and fixing it is the §113.4 move in three
files this lane does not own — §123.5 names them.

### 123.1 What it says now

The refusal carries the place in space where the two loops touch, in the
model's own frame. Measured on the tree this landed on, with a throwaway probe
over the corpus that printed the `Debug` of every refusal whose text holds
`LoopsTouch`:

```text
LED_D3.0mm_Horizontal_O1.27mm_Z10.0mm.step
  TheSolidTheFileDescribesIsNotOne { solid: EntityId(15), why: CannotMeshAFace {
    face: FaceKey { slot: FaceSlot(2v1), model: ModelId(81) },
    why: LoopsTouch { corner: 7, side: 4, at: Point([-0.23, -1.27, 9.999999956464]) } } }
```

against §113.7's record of the same file, which is the `corner: 7, side: 4`
form with no `at` at all. **18 corpus files** stop here, every one of them under
`kicad/` and every one of them an `LED_*_Horizontal_*` variant.

Two other counts, each with the command that produced it, because §113.7's
warning about quoting a stale one is the reason this section carries any:

```text
mise exec -- just t how_often_a_real_file_imports --no-capture
```

— nist `1 / 44` imported, kicad `114 / 323`, and under kicad's refusal groups
`18` for the loops-touch text and `22` for the tangency one.

```text
mise exec -- just t every_file_that_stops_at_the_tangency_wall_names_the_edge_it_was_following --no-capture
```

— `24`. That is more than the census's 22 and not in tension with it: the
census groups by the refusal's *text* and this test selects on the geometry the
refusal bottoms out in, which §113's own test doc says catches two shapes.

### 123.2 A place, and the demonstration that the place is findable

**What a caller can act on here is a point, and the point is one the file
itself wrote.** `#75 = CARTESIAN_POINT('',(-0.23,-1.27,9.999999956464));` in
`LED_D3.0mm_Horizontal_O1.27mm_Z10.0mm.step`, used by `#74 = VERTEX_POINT('',#75);`
— the three coordinates of the refusal, bit for bit. That is not luck and it is
not a search: §113.2 settled that `Import` stores every solid with the file's
own numbers at the identity, and the point published here is read out of the
solid's own point table rather than worked out again from `(u, v)`, so a corner
that came from a `VERTEX_POINT` carries that vertex's exact bits.

**A place is weaker than an `#id` and it is what exists.** §113 answered the
tangency wall by naming the edge, and the honest thing to say is that the same
answer does not fit here, for a reason that is about what a corner *is*:

- A corner of the domain is not an entity of the description. Some corners are
  vertices, and the two above are; the rest are points the edge walk dropped on
  a bend, which no `#id` names because they did not exist when the file was
  written. A refusal that named an entity would have to refuse to name one for
  every corner of the second kind, and the wall is reached by both kinds.
- Reading the value back is a **search of the file and not an address into it**,
  and it can answer more than once. In `LED_D5.0mm_Horizontal_O6.35mm_Z9.0mm.step`
  the published point `(-1.23, -6.35, 9.0)` is written twice: `#75` under a
  `VERTEX_POINT` and `#134` as a `LINE`'s origin. A reader gets a place to look
  and not a unique name, and saying so is cheaper than pretending otherwise.

The family of fixes considered, so that this is a §16.10 argument and not a
paragraph: **name the vertex** — fails for a walked corner, which is most of
them; **name the wire** — the loops handed to the triangulator are not the
face's wires one for one, because a seam splits one wire into two loops, so the
index would be a second private numbering with none of the first one's honesty;
**name the face** — that is the other half, it is real, and §123.5 has it;
**name nothing and keep the indices** — what was there, and §123.3 is why it was
worse than it looked.

### 123.3 The corner index was already naming the wrong point

`refusal_for` had one arm that did translate a corner, for
`CannotTellIfLoopsTouch`, and it translated it wrongly:

```rust
let at = face.wires().flat_map(WireView::points).nth(corner)
    .unwrap_or_else(Point::origin);
```

`WireView::points` is one point per coedge. A walked loop is those points **and
every point the edge walk put between them**, and `walk_a_wire_round` starts
from wherever `steps_from_somewhere_that_has_a_longitude` chose rather than from
the stored first coedge. So the two lists agree only on a face whose edges are
all straight and whose walk happened not to be turned; on anything else the
provenance record named a different corner, and past the end of the wires it
named the origin. The hedge is the tell: a lookup that cannot fail was written
as one that can.

Both arms now go through `where_the_model_holds_this_corner`, which reads
`corner_is` — the list `hand_over` fills loop by loop as it hands the corners
over, so it lines up with the triangulator's numbering by construction and an
index past its end is a panic rather than an origin.

### 123.4 The frame move is §116's, and the text is left alone

`triangulate` fills `at` from the domain's own surface, which for the kernel's
mesher is the face's surface in the frame `the_frame_this_face_is_meshed_in`
picked — so a refusal published straight through names a point of no model at
all. That is exactly §113.6's defect and §116's fix, one refusal along, and the
test is built the same way: a box at `(1024, 2048, 4096)` with a hole in its
cap reaching out to the outline. Without the move the refusal says
`Point([8.0, 3.0, 0.0])`; with it, `Point([1032.0, 2051.0, 4096.0])`.

**`Display` is unchanged on purpose**, and §110.5 is why. The message stays
`the corner at 7 sits on the side leaving corner 4, in another loop` for every
file that hits this wall, so `ProbeReport` keeps grouping all 18 under one line;
a message carrying coordinates would be a different string per file and split
one wall into eighteen reasons. §113.4 made the same call for the edge.

### 123.5 What is left, and it is three files this lane did not own

The `FaceKey` half is untouched, and it is the half that makes the corpus line
say `#15`. The shape of the fix is §113.4's, verbatim, one entity kind along:

- `crates/rcad-topo/src/error.rs` — a draft-face variant beside
  `ADraftEdgeHasACrossingThatCannotBeFollowed`, carrying the position the
  description put the face at.
- `crates/rcad-topo/src/builder.rs` — `Written` already says *"Nothing else the
  sweep names needs this yet"* beside its `edges: Vec<EdgeSlot>`; it wants a
  `faces` list and a `which_described_face_the_sweep_meant` next to
  `which_described_edge_the_sweep_meant`.
- `crates/rcad-step/src/brep.rs` — one arm in `which_entity_the_builder_meant`
  reading `face_names`, which is already built and already indexed the right way.

Done, that line reads `ThisEntityOfTheSolidWasRefused: solid #15, entity #… (ADVANCED_FACE)`
and the place this section publishes says which corner of it. Until then the
refusal names a point of the caller's model and a key of one it never saw,
which is better than two indices and a key and is not the finished article.

