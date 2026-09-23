## 214. The 24 stop in `walk`, on one cone twice, and not in `intersect` — 2026-09-23

§208.2's last flagged line, and §208.6's fourth bullet. A read-only lane; nothing
is fixed here, and what the next lane should build is 214.5.

### 214.1 The question

§208.2 ended with something flagged rather than claimed: the 24 frustum-against-a-
block walls that used to stop at *"an edge would have four faces along it"* all
stop one step further on, 9 as *a cone crossing another curved surface* and 15 as
*a crossing in two pieces with no half-space square to both*. A frustum and a
block are a cone, two planes and six planes. **Nothing in that list is a second
curved surface.** Either one is manufactured on the way, or the refusal is raised
about a pair that is not the pair it names.

### 214.2 Nothing calls `intersect`

The guess was that splitting the rim circle splits the cone face in two and a
later step asks `intersect` about a cone against a cone. **The splitting half is
right; the asker half is wrong twice over.**

Instrumented at `intersect`'s own same-surface arm, the arm never fires and the
fuse still refuses. The stack, from a panic at the refusal site, is:

```text
what_a_run_between_these_two_lies_on (walk.rs)  <- walk  <- Mesher::walk
  <- walk_the_edges_this_face_can_place  <- mesh
  <- the_coarsest_mesh_whose_sides_do_not_cross
  <- how_much_space_a_curved_solid_encloses  <- how_much_space_it_encloses
  <- nest_the_shells  <- assemble  <- fused_under  <- fuse
```

No call to `intersect` is on it. The refusal is raised by `rcad_geom::walk`,
reached from the mesher inside **the fuse's own volume measurement for shell
nesting** — which is why it looks like a crossing question and is not one.

### 214.3 The pair is one cone, twice

For the 9, the pair handed to `walk` is the same cone twice: `==` true, bit for
bit. `intersect` has an arm for exactly that pair — `(Surface::Cone(one),
Surface::Cone(other)) if one == other => Ok(SurfaceMeet::SameSurface)` — and
answers. `walk`'s `what_a_run_between_these_two_lies_on` has **no** such arm and
falls through to *a cone crossing another curved surface*.

The refusing edge, for the smallest case, is a closed circular run whose two faces
are both on the one cone, with `run = RightRoundWhereItsFacesCross` — *right round
where its faces cross*, when its two faces are one surface and cross nowhere. The
frustum's own seam is walked in the same run, with `run = Straight` and the same
`along = [Cone, Cone]`, and does **not** refuse.

Why only these rows: the block's planes are put to the cone face whether or not
the solids meet, because `fuse/cuts.rs` keeps no box test on purpose (§17.4). So
a disjoint block's cap plane still cuts the cone face in a circle, and the answer
carries two cone faces with that circle between them.

### 214.4 Which of §16.10's three, for each half

**The 9 are a defect** — the wrong question is being asked. The model holds one
cone, two cap planes and six planes; the refusal names a second curved surface
that does not exist. Worse, it is raised on a question that *has* an answer: a
closed run on one cone whose ends are one point is the circle through that point
square to the cone's axis, and nothing about it is degree four.

**The 15 are a gap**, and their name is honest: the pair `walk` is working on
there really is the block's wall plane and the cone, a genuine hyperbola whose two
branches are both held by the run's picker. `rcad-geom/src/error.rs` already says
so and says why.

The family of fixes considered for the 9, and why one of them:

- **(a) Give `what_a_run_between_these_two_lies_on` a same-surface arm** that
  reads the run, mirroring `intersect`'s. For a `RightRoundWhereItsFacesCross` run
  on one cone this is answerable from the run's single end plus the cone.
  Smallest, and the only one that also covers a cone seam whose run is not
  straight. **Recommended.**
- **(b) Widen `Mesher::walk`'s substitution guard**, which fires only when both
  sides are flat although `the_two_surfaces_either_side`'s own doc says a face
  listed twice is what tells `walk` to read the second surface off the run. This
  **cannot** work as things stand: `the_curved_surface_a_run_names` returns `None`
  for `RightRoundWhereItsFacesCross`, so there is nothing to read. It needs (d).
- **(c) Merge the two cone pieces back into one face** when the edge between them
  has the same surface on both sides. Removes the edge rather than answering it,
  changes face counts, and would want its own census.
- **(d) Record the cutting plane in the answer's run** — store the circle as
  `PartOfWhereItCrosses` rather than `RightRoundWhereItsFacesCross` when the face
  that made the cut is not in the answer. Then (b) works and the mesher needs no
  new arm. Bigger, and it touches what `assemble` writes rather than what the
  mesher reads. The follow-on if the run kind turns out wrong for other readers.
- **(e) Skip a tool plane whose face misses the argument.** Rejected: `fuse/cuts.rs`
  and §17.4 both argue a face-level box test changes answers.

### 214.5 Two wrong claims in the record, and what to build

`how_often_a_fuse_with_a_cone_refuses.rs` said, in two places, that the 24 stop at
`intersect` refusing a pair of surfaces. Nothing calls `intersect`; both are
corrected.

It also said that the 9 and the 15 being the *same* 24 *"is measured and not
inferred: `tests/which_arc_of_a_cones_rim_a_side_is.rs` walks this family's own
draws"*. That file walks §49.6's four-configuration grid and contains **no**
occurrence of `Family` or `Draw(` at all, so it cannot have pinned rows of this
family, and the claim had no measurement behind it. It has one now, from a census
of the family itself, and they are the same 24. This is §15.6's rule reaching a
third kind of claim: a sentence saying *"measured and not inferred"* owes the
measurement.

What the next lane should build: fuse the two solids of the smallest of the 9 —
slope `0.125`, near `1`, far `8`, block `[1,1,8]` at `(1,1,2)`, two solids that do
not touch — under `Union` and assert an answer. That single case is the whole of
the 9: the three draws differ only in `near`, and all three refuse on `Union` both
ways and `Difference` cone-first. Then re-derive
`what_stops_a_block_and_a_cone_now_is_five_named_things_and_not_the_ordering`,
which should lose its 9.

One loose end, not chased: a third disjoint draw answers, so whatever decides that
a cut survives into the answer differs between it and the three that refuse.
