## 463. Two faces on one patch pass validate, and what a measured or moved fuse answer can be held to — 2026-10-22

*Mattias asked how to make the fuzzers check volume. Three proposals came back:
hold every fuse answer to its rule's volume law, fix the writer's one volume
failure (Q2528), and later make a curved solid's volume error tighter. His
answer, verbatim:*

> I think all three might prove useful, so we should test for them.
>
> But:
> 1. Can a solid that passes Model::validate have two faces that overlap each
>    other: the same patch of one surface, back to back or doubled? If that
>    isn't valid, which check refuses it, and does any test or fuzz target build
>    an answer that would?
> 2. If the whole input to a fuse is moved (every solid by the same
>    translation, including ones that round) should the answer's area, volume
>    and centroid relative to the inputs be the same within the published
>    errors? If so, could the fuzz harness and the turned families check that,
>    comparing area as well as volume?

*Probed on `d9b71c87` with an example that was not committed (§463.8). Every
number below is its output.*

  463.1   Yes: four solids with two faces on one patch, and nothing refuses them
  463.2   What they do downstream: one wrong `Ok`
  463.3   Two sound solids fused flush keep no such pair
  463.4   The writer's crash has no overlapping faces
  463.5   A volume law cannot see a bad input
  463.6   The moved-fuse law, and why area needs a condition
  463.7   What exists, and where the law can run
  463.8   What ran this
  463.9   The rows

### 463.1 Yes: four solids with two faces on one patch, and nothing refuses them

Each solid is axis-aligned boxes put into **one shell** through the public
`SolidBuilder`, every face `Forward`, every outer wire anticlockwise about its
outward normal. All five built, and all five pass `Model::validate` and
`Model::validate_solid` under `Policy::certified()` with an empty log.

| | the solid | measured volume | true | measured area | true |
|---|---|---|---|---|---|
| P1 | cube `[0,2]³` and block `[0.5,1.5]²×[1,2]`: the two tops doubled, facing the same way | **9** | 8 | **30** | 24 |
| P2 | cube `[0,2]³` and block `[0.5,2.5]²×[2,4]`: back to back over `[0.5,2]²` | 16 | 16 | **48** | 43.5 |
| P3 | cube `[0,2]³` and a crack: two faces at `z = 1` over `[0.5,1.5]²`, on the same four edges | 8 | 8 | **26** | 24 |
| P4 | cube `[0,2]³` and `[0,2]²×[2,4]`: whole faces back to back, corners in one place and not shared | 16 | 16 | **48** | 40 |
| P4b | cube `[0,2]³` and `[0,1]×[0,2]×[2,4]`: back to back, two corners in one place | 12 | 12 | **40** | 36 |

**No check refuses them, because none looks at two faces together.**
`validate.rs`'s checks are per edge, wire and face, plus a solid's whole signed
volume above zero (`check_this_solid_encloses_space`). Each box closes on its own
edges, so every edge has two faces walking it opposite ways, and P1's volume of 9
is above zero. The fuse's own check that an argument does not cross itself skips
coplanar pairs, and says so: `fuse/cuts.rs`'s *"Two gaps, left open on purpose"*
— "A solid built by folding one face back onto another is therefore not caught."
P4's corners are two vertices in one place; nothing refused that either.

**So this is a gap and not a limit** (§16.10): the solids exist, the public
builder makes them, and the STEP reader's route into the kernel is that builder.
Under §352.3 it is `wrong/C`: a wrong `Ok` (§463.2), inside §3.5's envelope, built
only by a probe.

### 463.2 What they do downstream: one wrong `Ok`

Each was fused against a cube far away (`[10,12]³`) and against one overlapping
it (`[1,3]³`), under all three rules, both ways round. True answers come from a
voxel count on a half-unit grid.

- **P1 against the far cube answers `Ok`, wrongly, and the answer validates.**
  The union is three solids, of 8, 1 and 8, where the truth is two of 8. The
  difference is two, of 8 and 1, where the truth is one of 8. The fuse split the
  cube and the block inside it into two solids that share space. Against the
  overlapping cube all six fuses refuse `ResultIsNotManifold`.
- **P2 and P4b keep the back-to-back pair in every answer that still has it.**
  Volumes are right; areas are over by the pair. P4b's union with `[1,3]³`
  measures 56 against a true 52, its difference 40 against 36. P2's union is
  56.5 against 53.
- **P3 and most of P4 refuse.** Every P3 fuse refuses
  `AFaceMeetsItsNeighbourOnItsOwnSurface`. P4 refuses
  `Invalid(VolumeIsNotPositive)` with *"a solid encloses 0 of space"* wherever
  its answer keeps the pair, and answers the two fuses that drop P4 rightly
  (the intersection with the far cube, nothing; the far cube less P4, 8).

### 463.3 Two sound solids fused flush keep no such pair

The same shapes as two solids, each sound on its own, united:

| | union of | faces | volume | area | true area |
|---|---|---|---|---|---|
| P6 | `[0,2]³` and `[0.5,2.5]²×[2,4]` | 12 | 16 | 43.5 | 43.5 |
| P6b | `[0,2]³` and `[0,2]²×[2,4]` | 6 | 16 | 40 | 40 |
| P6c | `[0,2]³` and `[0,1]×[0,2]×[2,4]` | 8 | 12 | 36 | 36 |
| P7 | `[0,2]³` and `add_round_bar(1, 2)` placed at `(1,1,2)` | 11 | 14.28165 ± 0.00615 (true 14.28319) | 36.56560 ± 0.04193 | 36.56637 |

Both ways round for P6 to P6c. So the fuse carries a pair it is handed, and in
these four it makes none. **That is not a demonstration that no fuse of sound
inputs makes one.** The check §463.9's first row adds to `validate` runs on every
answer the gate fuses, and that run is the demonstration.

### 463.4 The writer's crash has no overlapping faces

Q2528 says crash `83d1740b`'s cylinder "has two coincident top caps". Decoded,
the second solid is a radius-2 height-4 cylinder whose top is **two faces on the
plane `z = 4` that do not overlap**: an annulus, its outer wire the rim and its
inner wire two half circles of radius 1, and the disc of radius 1 bounded by the
same two arcs walked the other way. They measure 9.332 and 3.078, which sum to
the rim's 12.41. One arc's pipe has its axis origin at `z = −2³⁴`, as Q2528
says. So that crash is not evidence for §463.1, and §463.9 files Q2528 again with
the description corrected.

### 463.5 A volume law cannot see a bad input

The per-rule law compares an answer with its inputs as measured: a union at least
its largest input and at most their sum, and so on. **P1 passes it.** P1
measures 9, its union with the far cube measures 8 + 1 + 8 = 17, and 17 is the
sum. The input is wrong in the same way its answer is, so a law between them
holds. The law finds a fuse that loses or invents volume from sound inputs; only
a `validate` that refuses P1 finds P1. Neither row replaces the other.

**The law has to allow for a side whose solids overlap each other.** Measured
volumes add per solid, so a side of two overlapping solids measures more than it
holds (the trap in §284.12). Sound against that: a union is at least the
largest single solid's volume and at most the sum of all; an intersection is at
least 0 and at most the smaller side's sum; a difference is at most the
arguments' sum and at least the largest argument less the tools' sum. Each is
loosened by the published `volume_error` of every solid in it. The three
disjoint parts give a law that needs no input at all — `A ∪ B` holds
`A \ B + A ∩ B + B \ A` — at the price of four fuses per draw.

### 463.6 The moved-fuse law, and why area needs a condition

**Volume and centroid: yes, within both answers' published errors.** If every
input is moved exactly by `t`, the true answer is the true answer moved by `t`:
the same volume and area, the centroid `t` further on. `volume_error`,
`area_error` and `centroid_wobble` each bound how far one measurement is from its
own answer's truth (`measure.rs`), so two measurements of answers whose truths
agree are apart by at most the sum of their two errors:
`|V₂ − V₁| ≤ e₁ + e₂`, and each coordinate of `(c₂ − t) − c₁` by at most
`w₁ + w₂`. **The moved answer's own error must be used, not the unmoved one's
twice**: a curved face's mesh error carries `POINT_ROUNDOFF` times its reach from
the origin (`Mesh::deviation`), so it grows with the move.

**A move that rounds adds a term.** The moved inputs are then not exactly the
translates, and the true answers differ by up to the volume the inputs'
boundaries moved through: the sum over inputs of area times how far the rounding
moved that input's boundary. `turning_a_fuse.rs` derives the same kind of
bound for a turn (`how_far_the_volume_may_move` and the two beside it). For a
fuzz draw it is negligible against a curved mesh error and not against a flat
solid's, whose published errors are its rounding alone.

**Area: only where neither answer gained or lost a contact.** Volume and
centroid change smoothly when the inputs move a little; area does not. Two
blocks flush, moved so that the rounding opens a gap of one ulp, unite into two
solids where they were one, and the area jumps by twice the contact.
`turning_a_fuse.rs` already allows for exactly this (`how_much_a_lost_contact_can_add`).
So area is held to `a₁ + a₂` (plus the rounding term) where both answers have
the same number of solids and of faces, and the draws that differ are counted.
Whether the same counts are enough to call it one contact pattern is for the
lane to show, by counting the draws where it is not.

**A measured share does not travel.** The turned families hold volume to
`A_CURVED_LAW_MAY_USE_THIS_MUCH_OF_WHAT_THE_FOUR_DECLARE` (0.05) of the sum,
which is measured on their own shapes. A fuzz draw is not those shapes, so the
fuzz harness holds the whole sum.

**On a curved answer the law sees only what is larger than the summed
errors, and the sum is loose for a reason a tighter bound does not remove.**
One curved solid's error is not far off its own miss: P7's volume misses by
0.0015 and declares 0.0062. But the four sides of inclusion and exclusion run
70 to 150 times the law's residual, because their meshes charge the same shells
with opposite signs and the misses cancel
(`A_CURVED_LAW_MAY_USE_THIS_MUCH_OF_WHAT_THE_FOUR_DECLARE`'s doc, from
`what_a_deflection_costs_a_measured_volume.rs`). Two meshes of one answer, moved
and not, cancel the same way. What shrinks the sum is a curved measure that
misses less to begin with, such as integrating the surfaces, not a tighter bound
on the same mesh. That measure has its consumer in these laws (§13).

### 463.7 What exists, and where the law can run

Read on `d9b71c87`:

- `fuses_that_must_agree.rs`: lattice pairs under quarter turns and whole
  shifts, and a bar or a ball in a block under whole shifts of at most 4, all
  compared bit for bit. The three *volume under a turn* families (a bar, a ball
  and a frustum against a block, turned by `Isometry::rotation`) and
  `a_turned_ball_united_with_a_block_holds_what_the_upright_one_does` compare
  **volume only**.
- `turning_a_fuse.rs`: flat lattice pairs turned, compared on volume, area and
  centroid against bounds of its own times `TURNED_FUSE_SLACK`, not the
  published errors.
- `tests/fuzzing/fuse.rs`: no measurement and no move. Its four checks are crash,
  atomicity, every key resolving with every `Ok` validating, and union and
  intersection the same both ways round.

Nothing compares the area or centroid of a moved curved answer with the unmoved
one.

**A claim there is stale.** `a_quarter_turn_of_a_round_solid_cannot_be_asked_for_exactly`
(`fuses_that_must_agree.rs`, 2026-09-04) says the only turn `rcad-geom` builds
is `Isometry::rotation`, so an exactly turned round solid is "not reachable
through the public API", a limit. `Isometry::quarter_turns` has existed since
`e2f336f1` (Q1320, 2026-10-07), and it turns about an axis along `x`, `y` or `z`
exactly. Whether a round pair turned that way fuses to the turned answer bit for
bit is now a question a test can ask, and so it is a gap.

So the law can run in two places: the fuse fuzz harness, as a second fuse per
answered draw on a copy with every solid moved by one translation
(`Model::place_solid`), and the turned families, with area and centroid beside
volume.

### 463.8 What ran this

`crates/rcad-topo/examples/probe_overlapping_faces.rs`, run as
`mise exec -- cargo run -q -p rcad-topo --example probe_overlapping_faces`, and
with the crash file's path as its one argument for §463.4. It built each solid
with `SolidBuilder`, ran `validate`, `validate_solid` and `measure` at a deflection
of 0.001, and fused it against each partner cube built the same way. The true
volumes and areas came from a voxel count on a half-unit grid. Neither was
committed: §463.9's first row pins P1 to P4b as tests.

### 463.9 The rows

Filed with this section:

- **Q2620**, tier 5, `wrong/C`: `validate` refuses P1 to P4b, and the gate's
  sweep of every fuse answer through the new check is §463.3's demonstration.
- **Q2621**, tier 2: the per-rule volume law in the fuse fuzz harness, and the
  three-part law in its corpus replay (§463.5).
- **Q2622**, tier 2, after Q2621: the moved-fuse law in the same harness
  (§463.6).
- **Q2623**, tier 2: area and centroid in the turned families, and exact quarter
  turns of round pairs in place of the stale claim (§463.7).
- **Q2624**, tier 2, after Q2622 and Q2623: a curved volume that misses less,
  with those laws' shares as its consumer.
- **Q2625**, tier 5, after Q2527: Q2528 filed again with §463.4's correction.

Q2620 and Q2625 name a dashed test file beside the real one, because
`queue.py` looks for a `merged.rs` item's file under a dashed name only. That
is Q2569, filed already.

**Mattias's three proposals map onto these rows.** The volume law is Q2621.
Q2528's fix is Q2625, with the bit-for-bit compare kept (§249.8). The tighter
curved volume is Q2624.
