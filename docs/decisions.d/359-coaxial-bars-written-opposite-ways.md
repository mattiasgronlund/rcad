## 359. Two bars on one pipe written opposite ways intersect and difference — 2026-10-04

*Queue row Q595, which replaced Q385 (§336.5). Lane
`lane-q595-coaxial-bars-written-opposite-ways`, off `3fe994aa`. Two round bars
on one axis, `z` 0 to 4 and 2 to 6, whose pipes are written pointing opposite
ways, refused `TheFuseWroteABrokenAnswer { CoedgesRunTheSameWay }` when
intersected or differenced. **A gap** (§16.10): each answer is a bar two high,
and the fuse now gives it.*

  359.1   What refused on the base, and whether Q575 moved it
  359.2   Why: a disc read against its own bar's pipe
  359.3   The fix: a side says what it was read against
  359.4   What the tests pin, and why the builder's bars are radius 1
  359.5   What this leaves

### 359.1 What refused on the base, and whether Q575 moved it

Measured on the base `3fe994aa` with the two tests this lane added, before the
fix:

- **Read from a STEP file**, radius 3, each pipe's axis written up or down,
  either bar handed over first, under each of the three rules: 24 fuses, and
  **8 refused**, every intersection and every difference of two opposite
  spellings. The other 16 answered one solid that validates and measures
  within its own error of `54π` (united) or `18π`.
- **Through `SolidBuilder`**, radius 1, the same four spellings and six rule
  and order pairs, unturned and turned by `1` and `−1` about `x`: 72 fuses, and
  **24 refused**, the same rule — every intersection and difference of opposite
  spellings, at every turn.

Q385 counted the same 8 of 24 from the file on `b3101151`. Since then Q575
(`9ff21bfa`) began baking the faces of one pipe as one face
(`faces_baked_as_one` in `fuse/bake.rs`), and **that changed none of the 24**:
the same 8 refuse, with the same refusal, and the same 16 answer.

### 359.2 Why: a disc read against its own bar's pipe

Each refused answer has a rim where one bar's flat disc meets the other bar's
wall: the intersection's lower rim is the upper bar's bottom disc against the
wall kept between `z` 2 and 4, and each difference has one such rim too.

`Edges::of` reads which way a side walks a closed edge against the curved
surface of that side's own crossing pair. The disc's piece is bounded by the
disc's own cap circle, the crossing of its plane and **its own bar's pipe**, so
that is what the disc's side was read against. The answer stores the edge by
the curved one of the two faces either side of it, which is the **other bar's
wall**. Written opposite ways, those two spellings wind opposite ways, and the
disc's side came out the wrong way round.

`every_closed_edge_read_off_one_spelling` was there for exactly this, written
for two curved faces on two spellings (§231.3 item 2). It compared each face's
own surface with the reference, and a flat face has no winding to compare, so
it skipped every edge with a flat face along it.

### 359.3 The fix: a side says what it was read against

`Side` carries `read_against`: for a closed side made in `Edges::of`, the curved
surface its way round was read against, which is `the_curved_one_of` the
piece's own pair; `None` on every other side (`from_one_input_solid`, the
joiner's `sides_of`, and the sides built by hand in the unit tests).

The pass now takes as reference the curved one of the two faces' surfaces —
the lower by bits where both are curved, as before — and turns a side round
when what it was read against is another spelling of that surface (one surface
under `name_of_the_surface_either_way_round`, different bits) whose winding at
the corner has a negative dot with the reference's. That is Q385's trial,
written in place of the old arm rather than beside it.

**The arm §231.3 fixed reads as before.** A curved piece cut by a plane is
read against its own surface, so for two curved faces the surface compared is
the face's own, as it was. The new turns are on sides read against something
other than their face's surface, and the disc is one.

**The dot is still exact**: turning an axis round negates every product
`the_way_it_is_parametrised_about` is made of, so two spellings' windings are
exact negatives.

**The path for a circle from a ball's pole needs nothing.** It reads against
the lid's normal rather than a winding, but what it was read against is a ball,
and a ball has no second spelling (`name_of_the_surface_either_way_round` folds
nothing for it), so no other surface is one surface with it and the pass leaves
such a side alone. `which_way_a_closed_edge_is_walked`'s doc says so.

### 359.4 What the tests pin, and why the builder's bars are radius 1

`a_flared_bar_reads_whichever_way_its_pipe_points.rs` (`rcad-step`) pins all 24
fuses from the file, beside the union Q41 needed: one solid each, validated,
measured within its own error of `54π` or `18π`.

`two_spellings_of_one_pipe_mesh_each_face_on_its_own_side.rs` (`rcad-topo`)
pins the 72 through the builder: one solid, validated, measured within its
error of `6π` or `2π`, and every face meshed within the span of its own
corners. Q385 never ran this route.

**The builder's bars are the file's own radius 1, not the 3 the row named.** At
radius 3, turned by `1` about `x`, every one of those fuses refused
`Undecidable` on "a point in a piece that every solid could place" — all four
spellings, all three rules, both orders, on one spelling as much as on two —
measured on the base with the test's radius set to 3 for the run. That is a
refusal of its own, filed as a row (359.5), and not this one. Radius 1 keeps
every turn the file already uses, including the turn that changes which
spelling sorts first, which is what this fix depends on; the STEP test keeps
radius 3 unturned.

### 359.5 What this leaves

- **Two bars of radius 3 on one pipe, `z` 0 to 4 and 2 to 6, turned by `1`
  about `x`, refuse `Undecidable`** at "a point in a piece that every solid
  could place" under every rule, both orders and every spelling, and do not
  at radius 1 or unturned. A row.
- A side read against a curved surface that is **not** a spelling of either
  face's — a third surface named by a run — is left as it was read, as before
  this section. No shape reaching that has been built.
