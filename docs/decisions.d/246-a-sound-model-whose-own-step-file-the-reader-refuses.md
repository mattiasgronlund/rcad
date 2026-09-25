## 246. A sound round bar whose own STEP file the reader refuses — 2026-09-25

*Queue row Q68, from §241.5 item 1. Lane `lane-q68-written-file-refused`, off
`c0b29166`, read-only on `crates/`. **The changed byte moves the bar by
`2⁻¹⁰⁰⁷` along `y`.** The writer writes that correctly. The reader builds the
bar with the move baked into every corner, and `Model::validate` then meshes the
bottom cap. The triangulator refuses any coordinate that is not zero and smaller
than `2⁻¹⁴²`. The model that went in passes `validate`, because a placed face is
meshed in its placement's inverse frame, where the move cancels to exactly zero.
**A gap, and not the reader's:** it is in `rcad_mesh::triangulate`, and a
one-line flush to zero there brings back every case measured, bit for bit.*

  246.1   What the byte changes
  246.2   What `#20` and `#53` are, and what refuses `#20`
  246.3   Why `validate` passes the model that went in
  246.4   Whose fault it is
  246.5   Where the fix goes
  246.6   The fuse refuses the same bar, for another reason
  246.7   What this leaves

Measured with a test file under `crates/rcad-step/tests/` that took the
harness's own steps — `postcard::from_bytes`, `Model::validate`,
`rcad_step::write`, `StepFile::read`, `Import::read`, and `measure` on both
sides — and printed each answer. It was deleted before this commit.

**The repro.** Take the seed
`fuzz/corpus/no_model_makes_the_writer_panic/1bf987c187b377606ff669042712474c52227800`
(549 bytes). Set the byte at offset 506 counting from zero (byte 507 counting
from one) from `0x00` to `0x01`. Hand the result to
`one_model_decoded_and_written` in `crates/rcad-step/tests/fuzzing/writer.rs`.

### 246.1 What the byte changes

Both byte strings decode. The `{:#?}` of the two models differs in three
lines. Two of them are the run-local counters `ModelId` and `WhichModelThisIs`,
which live on `Model` and not in the `ModelData` that reaches the file. The
third is the solid's one `ShellUse`, where its `loc` translation goes from

```text
translation: [0.0, 0.0, 0.0]
```

to

```text
translation: [0.0, 7.291122019556398e-304, 0.0]
```

So it is not a flag, a coedge's direction or a face's sense. **It is where the
solid is put.** `7.291122019556398e-304` is `2⁻¹⁰⁰⁷` exactly, and its bits
are `0x0100000000000000`. The changed byte is the top byte of that `f64`,
which postcard writes little-endian. `0x01` there sets the exponent field to 16,
giving `2^(16 − 1023)`.

**No fuzzer is needed to make this model.** The public API makes it byte for
byte:

```text
add_round_bar + place_solid by (0, 2^-1007, 0) is the changed seed byte for byte: true
add_round_bar placed at birth is the changed seed byte for byte: true
```

That is `Model::new(Scale::millimetres(8.0))`, then
`add_round_bar(2.0, 8.0, &Isometry::identity())`, then `place_solid` by
`Isometry::translation(Vector::new(0.0, 2⁻¹⁰⁰⁷, 0.0))`. Passing that
translation to `add_round_bar` directly gives the same bytes. The model
validates and measures:

```text
changed seed measures: Ok(Measurement { volume: 100.4326197214823, … centroid: Point([4.440892098500626e-16, 6.089739010625635e-17, 4.0]), … })
changed seed's placed corners: [Point([2.0, 7.291122019556398e-304, 0.0]), Point([2.0, 7.291122019556398e-304, 8.0]), …]
```

### 246.2 What `#20` and `#53` are, and what refuses `#20`

The file the writer makes, cut to the bottom cap and the solid:

```text
#6=CARTESIAN_POINT('',(0.0,0.0,0.0));
#7=DIRECTION('',(0.0,0.0,-1.0));
#8=AXIS2_PLACEMENT_3D('',#6,#7,$);
#9=PLANE('',#8);
#10=CARTESIAN_POINT('',(0.0,7.291122019556398E-304,0.0));
#11=DIRECTION('',(0.0,0.0,1.0));
#12=AXIS2_PLACEMENT_3D('',#10,#11,$);
#13=CIRCLE('',#12,2.0);
#14=CARTESIAN_POINT('',(2.0,7.291122019556398E-304,0.0));
#15=VERTEX_POINT('',#14);
#16=EDGE_CURVE('',#15,#15,#13,.T.);
#17=ORIENTED_EDGE('',*,*,#16,.F.);
#18=EDGE_LOOP('',(#17));
#19=FACE_OUTER_BOUND('',#18,.T.);
#20=ADVANCED_FACE('',(#19),#9,.T.);
…
#52=CLOSED_SHELL('',(#20,#35,#51));
#53=MANIFOLD_SOLID_BREP('',#52);
```

**`#53` is the bar and `#20` is its bottom cap**: the plane `z = 0`, facing
down, bounded by one circle of radius 2 round `(0, 2⁻¹⁰⁰⁷, 0)`, with its one
corner `#15` at `(2, 2⁻¹⁰⁰⁷, 0)`. Against the unchanged seed's file, the only
lines that differ are the six points that carry the move — `#10`, `#14`,
`#25`, `#29`, `#36` and `#41`. The two plane points `#6` and `#21` do not
change, because the plane `z = c` moved along `y` is the same plane.

The whole refusal, with its sources:

```text
ThisEntityOfTheSolidWasRefused {
    solid: EntityId(53),
    entity: EntityId(20),
    why: ADraftFaceCouldNotBeMeshed {
        face: 0,
        why: CoordinateOutOfRange { position: 0, why: TooSmall },
    },
}
because: triangulating: the corner at 0 is outside the range a triangulation takes: TooSmall
```

**No check in the reader refuses `#20`.** The reader describes the solid to a
`SolidBuilder`, and `SolidBuilder::build` writes it into a scratch model and
runs `Model::validate` over it. For a solid that is not wholly flat,
`check_this_solid_encloses_space` meshes it:
`how_much_space_a_curved_solid_encloses` → `SolidView::mesh` →
`BuildingAMesh::hand_over` → `rcad_mesh::triangulate`. `triangulate` inserts
each corner of the patch into spade's `ConstrainedDelaunayTriangulation`, and
maps spade's `InsertionError` to `CannotTriangulate::CoordinateOutOfRange`.
`which_described_face_the_sweep_meant` turns the sweep's `CannotMeshAFace` into
`ADraftFaceCouldNotBeMeshed { face: 0 }`, and the reader's
`which_entity_the_builder_meant` names draft face 0 as `#20`.

Spade's rule, from `validate_coordinate` in `spade-2.15.1`:

```rust
} else if as_f64.abs() < MIN_ALLOWED_VALUE && as_f64 != 0.0 {
    Err(InsertionError::TooSmall)
```

with `MIN_ALLOWED_VALUE = 1.793662034335766e-43`, which is `2⁻¹⁴²`. Corner 0 of
the cap's patch is `#14`. On the plane `z = 0` its two coordinates are `2` and
`±2⁻¹⁰⁰⁷`, and the second is the one spade refuses. That is the reading of
the code. The table in 246.3 is the measurement: the line falls exactly at
`2⁻¹⁴²`.

### 246.3 Why `validate` passes the model that went in

`the_frame_this_face_is_meshed_in` in `crates/rcad-topo/src/mesh.rs` meshes a
face that carries a placement **in that placement's inverse**. The cap's corner
`(2, 2⁻¹⁰⁰⁷, 0)` is carried back to the stored `(2, 0, 0)`, since
`2⁻¹⁰⁰⁷ − 2⁻¹⁰⁰⁷` is exactly zero, and spade is handed `0`. A face at no
placement is meshed where it is stored, apart from `a_shift_that_loses_nothing`,
which only moves faces whose numbers are far from the origin. A face read from a
file is at no placement, and so is one a fuse wrote (§17.3). There the move is
part of the stored numbers and reaches spade as it stands.

So **`Model::validate` gives two answers for one bar, decided by where the move
is written down.** `check_dynamic_range`'s doc comment calls that wrong for the
range check ("the same model, two answers, decided by where the placement
happened to be written down"). It is the same thing here for the mesh.

A round bar of radius 2, 8 tall, moved by `(0, d, 0)` with `place_solid`, then
validated, written, read back and compared, without any fix:

| `d` | `validate` on the model | read back |
|---|---|---|
| `2⁻¹⁰⁰`, `2⁻¹⁴¹`, `2⁻¹⁴²` | `Ok` | `Ok`, and the same bit for bit |
| `2⁻¹⁴³`, `−2⁻¹⁴³`, `2⁻²⁰⁰`, `2⁻¹⁰⁰⁷`, `2⁻¹⁰²²` | `Ok` | refused, `#20` `TooSmall` |
| `2⁻¹⁰²³`, `2⁻¹⁰⁷⁴` (subnormal) | `Ok` | refused, `#20` `TooSmall` |

A cube of side 2 moved by `2⁻¹⁰⁰⁷` comes back the same bit for bit. `validate`
never meshes a wholly flat solid, because its volume is its own fan.

**Build `2⁻ᵏ` from its bits, not with `powi`, when `k` is only known at run
time.** In this lane's first table, `2f64.powi(-k)` with `k = 1074` coming from
an array returned `0.0`, and the row read "same bit for bit" for a bar that had
not moved at all. The written file had `0.0` where the move should be, and was
the same length in bytes as the unchanged bar's. The same call with the literal `-1074` printed
`5e-324`. The table above builds `d` with `f64::from_bits`, and prints each
placed corner's `y` to show the move is really there.

### 246.4 Whose fault it is

**Not the writer's.** Every number in the file is the placed model's own,
bit for bit:

```text
changed seed's placed corners: [Point([2.0, 7.291122019556398e-304, 0.0]), …
#14=CARTESIAN_POINT('',(2.0,7.291122019556398E-304,0.0));
the file's number parses to 2^-1007 exactly: true
```

The same writer and reader bring the same bar back unchanged at `2⁻¹⁴²`, one
power of two further from zero. What the file says is what the model says.

**Not `Model::validate`'s, for passing the model.** Nothing is wrong with the
solid. It is a round bar of radius 2 moved a distance too small to matter, the
public API makes it byte for byte (246.1), and it measures. `validate` is right
to pass it. Its wrong answer is the other one — the refusal of the same bar with
the move baked in — and that answer comes from the triangulator, not from
anything `validate` checks about the solid.

**So it is the second of the three: the reader refuses a file that correctly
says a solid this kernel holds.** By §16.10 that is a gap: the answer exists.
The same disc triangulates at `2⁻¹⁴²`, and at `0`. But the refusal is not the
reader's. It is `Model::validate` meshing a face at no placement, the same
sweep a fused solid gets, so **the fix does not belong in `rcad-step`**.

### 246.5 Where the fix goes

In `triangulate`, `crates/rcad-mesh/src/triangulate.rs`, in the loop that
inserts the patch's corners: pass each scaled corner through
`spade::mitigate_underflow` before `mesh.insert`. That sets any coordinate
smaller than `2⁻¹⁴²` to zero. Spade offers the function for "vertices [that]
come from an uncontrollable source like user input". The corner moves by less
than `2⁻¹⁴²` in the domain spade sees, and nowhere else. `hand_over` keeps each
corner's 3D point by its index (`corner_is`), so no point of the mesh moves.
What can change is which triangles spade draws. The interior needs nothing:
`refine_the_inside` already skips a midpoint spade will not take
(`let Ok(handle) = mesh.insert(at) else { continue; }`), so the corner loop is
the only place this refusal comes from.

**Measured, with a one-line probe patch reverted before this commit.** Every
row of the table in 246.3 was written, read back, validated and compared the
same bit for bit: every move from `2⁻¹⁰⁰` down to `2⁻¹⁰⁷⁴`, `−2⁻¹⁴³`, the
changed seed and its public-API twin. The cube still came back the same.

Two other families were looked at and not taken:

- **Scaling the domain by a power of two.** That is exact, but this cap's
  coordinates run from `2⁻¹⁰⁰⁷` to `2`, a ratio of `2¹⁰⁰⁸`. Spade's window,
  `2⁻¹⁴²` to `2²⁰¹`, has a ratio of `2³⁴³`, so no single scale fits both ends.
- **Shifting an unplaced face**, the way `the_frame_this_face_is_meshed_in`
  unplaces a placed one. A shift by `(0, 2⁻¹⁰⁰⁷, 0)` clears this bar only
  because every tiny coordinate in it is the same tiny number. It says nothing
  about a solid whose corners carry different ones. Not built.

**What the fix still has to show, and this lane did not:** that a corner
flushed onto the line of a side cannot turn a triangle over, or leave the two
patches along an edge drawing it differently. If it does, `finish` in
`crates/rcad-topo/src/mesh.rs` has to refuse the mesh rather than let it
through. The flush makes such a corner collinear in spade's view, where it was
off the line by less than `2⁻¹⁴²`. None of the cases above had a corner like
that.

### 246.6 The fuse refuses the same bar, for another reason

The same bar moved by `2⁻¹⁰⁰⁷` and fused with a cube of side 2 refuses. Unmoved,
the same fuses go through and the result validates:

```text
union_with_a_cube_far_away: FUSE REFUSED: could not settle 1 decisions the answer depends on / Undecidable { … predicate: "where a plane and a pipe cross", inputs: [Exact([0.0, 2.0, 0.0]), Exact([0.0, 7.291122019556398e-304, 0.0])], level: Interval, outcome: Undecidable { enclosure: 8.881784197001252e-16 } … }
union_with_a_cube_far_away (not moved): fused, validate after Ok(())
difference_with_a_cube_through_the_wall: FUSE REFUSED: could not settle 1 decisions the answer depends on / Undecidable { … predicate: "where three surfaces meet", inputs: [Exact([1.0, 0.0, 0.0]), Exact([0.0, 0.0, 0.0]), Exact([0.0, 7.291122019556398e-304, 0.0])], level: Exact, outcome: Undecidable { enclosure: inf } … }
difference_with_a_cube_through_the_wall (not moved): fused, validate after Ok(())
```

The union's cube sits at `(10, 0, 0)` and touches nothing. The difference's cube
sits at `(1, −1, 3)` and goes through the wall. These are predicates refusing,
not the mesher, and the flush patch left both refusals as they were. Not
diagnosed here.

### 246.7 What this leaves

- `crates/rcad-mesh/src/triangulate.rs`: `triangulate` flushes each scaled corner through `spade::mitigate_underflow` before inserting it, with a test in `crates/rcad-mesh/tests/cutting_a_patch_into_triangles.rs` that a disc whose one corner sits at `v = 2⁻¹⁰⁰⁷` triangulates (246.5).
- `crates/rcad-topo/src/mesh.rs`: a test that a corner flushed onto the line of a side cannot turn a triangle over, or have the two patches along an edge draw it differently, without `finish` refusing (246.5, last paragraph).
- `crates/rcad-step/tests/a_solid_written_out_and_read_back_is_the_same_solid.rs`: a round bar moved by `2⁻¹⁰⁰⁷` along `y` is written and read back the same. It fails today and passes once the flush lands.
- `fuzz/corpus/no_model_makes_the_writer_panic/`: check in the changed seed (offset 506 set to `0x01`) once the flush lands. Until then it panics `every_model_the_writer_fuzzer_kept_is_written_again.rs`.
- `crates/rcad-topo/src/fuse/`: why a bar moved by `2⁻¹⁰⁰⁷` makes "where a plane and a pipe cross" (interval) and "where three surfaces meet" (exact, enclosure `inf`) undecidable, when the unmoved bar fuses (246.6).
