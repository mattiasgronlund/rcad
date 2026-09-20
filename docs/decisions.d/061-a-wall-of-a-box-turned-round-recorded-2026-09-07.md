## 61. A wall of a box turned round, recorded 2026-09-07

`just fuzz-fuse` found ten crashes in 600 s. All ten are one bug, none of them
is in the fuse, and what they are is `Model::validate` accepting a solid whose
skin does not close. The fuse then walks it and trips `Model::check_touched_edge`
— an assert where a refusal belonged, §38.2's family — and in a **release**
build not even that: `check_touched_edge` returns early and `assemble`'s
post-write sweep is `debug_assertions`-gated, so two flat boxes with one wall
turned round fuse to `Ok` and commit a model that does not validate. That is
core principle 1 inverted, and it is what makes this a unit rather than a
hardening exercise.

Two ways in, both through `FaceUse`: `orient: Reversed` on a face of a box
(seven of the ten), and a `loc` that is not the identity (the other three).

### 61.1 The rule was written down twice and read back nowhere

`Assembly::write_into` (`fuse/assemble.rs:343`) and `rcad-step`'s
`describe_the_face` (`brep.rs:117`) keep the same table, and `brep.rs` calls it
"the same two-line table the fuse already keeps":

```text
(Some(flat), true)      => (Surface::Plane(flat.reversed()), Forward)
(Some(_) | None, false) => (piece.surface,                   Forward)
(None, true)            => (piece.surface,                   Reversed)
```

A plane welds "which surface" to "which way", so a plane facing the other way is
stored turned round and then used **forwards**; a sphere and a pipe have no
turned-round form, so those carry the turn on the face use. **No face sitting on
a plane is ever used reversed**, and until now nothing said so.

`Model::check_no_flat_face_is_used_reversed` is that sentence read back, in the
sweep and at the `Deserialize` door both. It reads the turn the *solid* ends up
using rather than the one written on the face, because nothing in the crate
writes a reversed `ShellUse` but `Deserialize` will read one and it turns every
face of that shell round.

### 61.2 Why nothing else could see it

Turning one wall of a box round changes no coedge in the model, so
`check_edge_is_walked_by_two_opposite_coedges` — which reads the turns as
stored — sees nothing. The wall's own winding is unchanged, because reversing a
face turns its normal *and* walks its wire the other way, and `check_flat_face`
reads the raw face at `Isometry::identity()` and never sees how a shell uses it
at all. What is left is the volume, and flipping one wall of a `10³` box leaves
the divergence sum at `333.3` — still positive, still no complaint.

It is reachable through the public API and not only through a file, which is
§16.11's kind: `SolidBuilder::add_shell` takes an `Orientation` per face and its
own door reads the stored turns too, so a box described face by face with any one
wall turned round built and validated.

### 61.3 The mesh answer was tried and is wrong

The obvious fix is to run `every_side_of_the_skin_is_used_both_ways` over a
wholly flat solid as well as a curved one, and the balance really does separate
the flipped box from the sound one — 4 sides not used both ways against 0, over
18 sides.

**It refuses sound solids.** `check_shells_and_solids` gives a polyhedron no mesh
because its own fan *is* its volume, and a fan cannot refuse; a mesh can.
`the_edge_across_a_sliver_is_as_long_as_the_sliver_is` comes back
`CannotMeshAFace { CoordinateOutOfRange { position: 2, why: TooSmall } }`, so a
sound fuse answer stops validating — a refusal about the sweep rather than about
the model, which is the thing `as_coarse_as_a_mesh_gets` exists to keep the
curved branch clear of. Meshing every flat solid at every test boundary is also
what §4.3 says not to spend.

### 61.4 And no count over the skin works either

The next candidate is that the two faces along an edge walk it opposite ways *as
their shell uses them*. It is wrong, and it is wrong for a reason §36.24 already
recorded: a curved face turned round is **the same surface with its wire stored
walked the other way**, so a bar's wall used reversed walks its open edges the
same way as the planes notching it and is perfectly sound. Measured, it refused
`every_answer_over_a_block_turned_against_a_bar_is_a_solid_that_validates` and
three STEP corpus tests. `a_washer`'s hole says the same thing about closed
edges, which is what the earlier attempt on this had found.

So a torn skin and a sound one are combinatorially identical, and only a plane —
which is welded to a direction — can be asked without arithmetic. That is why the
check is the table above and not a balance.

### 61.5 The placement half, and what it retires

A `FaceUse` carrying a placement tears the skin without moving a corner.
`FaceView` hands its own location down to its wires, so a displaced face reads
its surface *and* its corners at the same place and stays self-consistent —
every check that looks at one face passes, the volume is right. What stops being
true is that the two faces along an edge agree where that edge is. The mesh
cannot see it either, because the mesher names a corner by its vertex and both
faces name the same vertex: a wall moved 1000 units away and a wall nudged 0.25
both validated.

**Decision: `FaceUse::loc` must be the identity, refused at the door.** Every
site in the crate writes one, `place_solid` composes onto the *shell's*
placement and never the face's (§44.8), and a face is used by exactly one shell,
so there is no sharing for a per-use placement to serve — §42.7's argument about
shells, made about faces by `Invalid::UsedMoreThanOnce`. The alternative,
checking that the faces along each edge place it identically, is a pass over
pairs and a decision about how near is near enough, which is a tolerance this
kernel exists to avoid. Taking the field away was the third option and is not
needed: an identity that must stay an identity costs one comparison.

What it retires is `a_location_on_a_shell_and_one_on_a_face_compose`, which put a
shift on every face use and checked the two levels composed. They do, and
`SolidView::faces` still composes them; there is no longer any way to reach it.
The shell half of §3.2 is what a caller actually has, and
`a_face_reached_through_a_placed_solid_comes_back_already_moved` is where that
lives.

### 61.6 The door, and the tests that could no longer build what they tested

Both checks go in `check_shape_is_possible` as well as in the sweep. §44.5's rule
admits them — one pass over the face uses, an `Orientation` or an `Isometry`
against a constant, nothing about a pair — and it matters because `Model::fuse`
does not validate its arguments, so a model read off a wire and handed straight
to the fuse is exactly the shape the fuzzer found.

The cost is that two tests built their inside-out cube by turning all six face
*uses* round, which is no longer a model at all.
`a_solid_turned_inside_out_is_caught` and
`measuring_a_solid_read_back_inside_out_is_refused_rather_than_crashing` now
build it the way a file has to — every plane turned round, and every wire walked
the other way so that it still winds with the normal it is given. Written that
way no single face is wrong and neither is any single wire, so the volume is
still the only thing that can notice, which is what both were for.
`a_solid_described_with_every_face_turned_round_is_refused_by_the_sweep` still
comes back from the sweep and now names the rule that was broken rather than the
consequence.

### 61.7 What it is measured against

**The instrument that found it, run again at the same budget.** 614 s of
`two_solids_fused_by_a_stranger` at `-fork=4`, 3 439 084 executions,
`oom/timeout/crash: 0/0/0`, against ten crashes from 600 s of the same target
the day before.

All ten crash artifacts also replay with no crash, and **that on its own is weak
evidence** — an artifact the target now declines to decode never reaches the
fuse, and an invocation that ran nothing would look the same. What carries it is
the reduced forms beside them, which have a before. `face4_reversed.bin`,
`face4_moved.bin` and `face4_nudged.bin` used to decode and validate and now do
not decode; `55f28d44_first.bin`, which used to answer with a model that does not
validate, no longer decodes either. The sound controls in the same directory —
`built_only.bin` and the seven negative-zero variants — still decode, fuse and
validate, which is the half that makes the other half mean anything.

### 61.8 What this leaves

**`SolidBuilder::add_shell` could refuse a reversed flat face at the call rather
than at the sweep.** The draft's surface is known there, and the builder's own
division is that what one call can decide is decided there. It is one face and
not the whole solid, so by that division it belongs at the call.
`a_solid_described_with_every_face_turned_round_is_refused_by_the_sweep` is the
test that would move with it.

**A reversed `ShellUse` is still a thing a file can write.** Nothing in the crate
writes one and nothing reads one for anything but composition. It is caught here
only through what it composes onto a flat face; a solid of nothing but curved
faces with its shell used reversed is left to the volume.

