## 361. The mesher turns a torus face the way it faces, and the kernel's bends are built the way a file writes them — 2026-09-29

*Row Q644 (`lane-q644-turn-the-meshers-torus-faces`, off `34052a6d`). §329
found that a torus faces out of its tube for the standard, the reader,
`Model::validate` and `classify.rs`, and into it for the mesher alone, and that
the kernel's bends were built turned round to suit the mesher. This is
§329.5's change: the mesher turned, `classify.rs`'s torus arm put back, and
every bend the kernel's tests build rebuilt the way a file writes one. It
answers Q39 and Q82.*

  361.1   What changed
  361.2   Which answers moved
  361.3   What this leaves
  361.4   Landed with the bends it left (Q667)

### 361.1 What changed

- **`mesh.rs`'s `hand_over`** reads `inside_out` as
  `face.orientation().is_reversed() != matches!(stored, Surface::Torus(_))`,
  the one line §329.3's "turned" rows ran (`surface` since the merge, 361.4). The triangulator draws every
  triangle anticlockwise in the chart, and a torus's chart takes the tube angle
  first, so the chart's turn points into the tube (§329.1 (c)).
- **`classify.rs`'s `gradient_at`**: the torus arm is no longer negated.
  `5e856162` turned it round to fit the bends built turned round; with the
  bends rebuilt it is the quartic's own gradient again, out of the tube.
  Q102's beside-a-box route, from the same commit, is kept.
- **Every bend built in a file this row owns** now uses its wall `Forward` and
  walks every closed-edge coedge the other way, caps and wall alike:
  `tests/rings/mod.rs` (`a_bend`, `the_top_half_of_a_ring`); `holds.rs`'s
  `a_piece_of_pipe` and the two pucks; `mesh.rs`'s
  `a_band_between_two_circles`; `classify.rs`'s `a_box_beside_a_blocks_side`
  (moved out by Q721, 361.3's note below); and the bends written inline in
  `a_cone_and_a_torus_through_the_front_door.rs`,
  `two_loops_side_by_side_on_a_torus.rs` and `a_ray_from_beside_a_box.rs`. A
  seam is walked both ways in one wire, so turning a wall's circles over is the
  same as reversing its wire (§329.5 item 2).
- Not changed, as §329.5 item 4 says: `validate.rs`, `mesh.rs`'s
  `gradient_of`, `rcad-geom`'s `Torus`, the reader.

### 361.2 Which answers moved

With the mesh turned and the builders in `rings`, `holds.rs`, `mesh.rs` and
`classify.rs` rebuilt, before any pin moved, `mise exec -- just t --workspace
--no-fail-fast` ran 2802 tests and failed 19 (§329.4 counted 42 of 2617 with
the mesh turned alone). Each answer that moved, and the command that shows it
now:

- **A bend as the standard writes it** validates, measures `2π²` within its
  error at deflection `0.01`, and places all eight exact points as the shape
  does, both past a cap `Outside`; so does `rings::a_quarter_bend`. The bend
  with every turn turned over — how the kernel built bends until now — is
  refused `FacesDoNotAgreeWhichWayTheyFace`: its wall faces into the material
  and its caps out of it. The three KiCad relays import. `mise exec -- just t
  -p rcad-step --test which_way_a_torus_faces_normal_points`, three tests.
- **KiCad files that import: 161 to 164**, the three relays and nothing else
  moving either way; the files that round trip are the same 45. The run
  before the pins moved printed `left: 164, right: 161` and named the three.
  `mise exec -- just t -p rcad-step --test how_often_a_real_file_imports
  --test a_real_file_written_out_and_read_back_is_the_same_solid`.
- **A bend united or intersected with a copy of itself is the bend**: one
  solid, which validates and measures `2π²` within its error. It was
  `TheFuseWroteABrokenAnswer { CoedgesRunTheSameWay }` (§243.5), which is
  Q82's refusal; the bend written back was the bend turned round. The quarter
  bend now goes through every test `copying_a_solid.rs` runs on a fuse with a
  copy — bit for bit the same as with a bend built again, under all three
  rules. `mise exec -- just t -p rcad-topo --test
  how_often_a_fuse_with_a_torus_refuses --test copying_a_solid`.
- **A bend less a second bend turned an eighth of a turn is one piece**,
  holding `π²` within its error, and the bend built face by face gives the same
  numbers bit for bit: `building_a_solid_out_of_faces.rs`'s bend now takes
  `the_same_numbers` like the cube, the bar and the ball, as its doc said it
  would when the refusal closed. `mise exec -- just t -p rcad-topo --test
  building_a_solid_out_of_faces`.
- **A bend growing out of a block's side builds** when the side walks the cap
  circle forwards, validates and measures `32 + 2π²`; walked the other way it
  is refused `AClosedEdgeIsWalkedAgainstItsSurface`. Both walks were refused
  before. `mise exec -- just t -p rcad-topo --test a_ray_from_beside_a_box`.

The three-quarter bend in `holds.rs` and the three-quarter band in
`two_loops_side_by_side_on_a_torus.rs`, rebuilt the standard way, are still
refused `AClosedEdgeIsWalkedAgainstItsSurface` by the sweep. So §329.5 item
6's doubt about Q103 is settled the other way: the refusal was not two turns
disagreeing, and Q103's premise stands. `mise exec -- just t -p rcad-topo -E
'test(/three_quarter/)'`.

### 361.3 What this leaves

**The change does not land green on this row's write set alone.** Five files
outside it build a bend turned round, and fail with `FacesDoNotAgreeWhichWayTheyFace`
or a moved answer until each is rebuilt the same way (Q667 did, 361.4):

- `crates/rcad-step/tests/fuzzing/writer.rs`'s `a_quarter_bend`, behind both
  failing tests in `every_model_the_writer_fuzzer_kept_is_written_again.rs`
  (the seed `a_quarter_bend_of_pipe` measures `VolumeIsNotPositive`);
- `crates/rcad-topo/tests/a_bend_round_a_slanted_axis.rs`, two tests;
- `crates/rcad-topo/tests/a_joined_flat_face_whose_rings_enclose_no_area.rs`,
  its half bends;
- `crates/rcad-topo/tests/two_frustums_stacked_on_one_cone.rs`, its two
  quarter bends;
- `crates/rcad-topo/tests/a_bend_less_a_turned_copy_of_itself.rs`, where the
  difference is now answered with one solid, and the pin has to move to what
  that solid is.

`crates/rcad-topo/src/error.rs`'s doc of `TheFuseWroteABrokenAnswer` names a
quarter bend united with its copy as its first example and a test this row
removed; that example no longer refuses.

`classify.rs`'s `a_box_beside_a_blocks_side` pins were written in `src/`
because no public call built a solid with a torus face and a flat face with
straight sides. `a_ray_from_beside_a_box.rs` now builds one, so they can move
out.

*2026-10-11, Q721: moved.* The seven pins are out of `classify.rs`; four
tests in `tests/a_ray_from_beside_a_box.rs` now ask the same boxes through
the public `classify_uncertain_point` of the glued bend —
`a_box_beside_a_side_of_the_block_a_bend_grows_out_of_is_answered`,
`a_box_beside_the_top_of_a_turned_block_a_bend_grows_out_of_is_answered_outside`,
`a_box_in_the_notch_of_an_l_a_bend_grows_out_of_is_answered_outside` and
`a_box_in_the_notch_of_a_turned_l_a_bend_grows_out_of_is_still_refused`
(§323.3's matching note).

### 361.4 Landed with the bends it left (Q667)

*Row Q667 (`lane-q667-land-the-turned-torus-mesh`, off `7fc73410`) merged
Q644's branch at `3e51e291` and did 361.3's list.*

- **The merge** had no textual conflict and one line that did not compile:
  Q645 renamed `hand_over`'s `stored` to `surface`, because a flat face is now
  cut up where the solid sits. For a torus face the surface it is cut up on is
  still the one it is meshed on, so the turn reads `surface`. After it,
  `mise exec -- just t --workspace --no-fail-fast` ran 2821 tests: 7 failed,
  10 skipped, exactly the tests of 361.3's five files.
- **Four bends rebuilt**, each wall `Forward` with every closed-edge coedge
  turned over: `fuzzing/writer.rs`'s `a_quarter_bend` (its `a_wall` now takes
  which way round the first circle goes, `Forward` for the frustum),
  `a_bend_round_a_slanted_axis.rs`, and `two_frustums_stacked_on_one_cone.rs`.
  The half bend in `a_joined_flat_face_whose_rings_enclose_no_area.rs` has no
  closed edge. Its wall's loop, two arcs and two short sides, already runs
  anticlockwise seen from above, which is right-handed about the way out of the
  tube, so only the wall's use turns. It validates, measures `2π²` and is
  refused as before.
- **A bend and a copy turned an eighth of a turn answer under every rule**
  (Q341). Each answer is one solid, and `Model::validate` passes. At
  deflection `0.01` the difference and the intersection each hold `π²`, an
  eighth of the ring by Pappus, and the union holds `3π²`, all within
  `measure`'s bound. The difference and the intersection have 3 faces. The
  union has 5, because its wall stays in three pieces. `mise exec -- just t
  -p rcad-topo --test a_bend_less_a_turned_copy_of_itself`.
- **A bend's cap against a block: its control answers 90 of 90**, 105 laws
  checked. It was 45, the fuses that keep the bend having stopped writing it
  back. The touching row still refuses all 66 as a quartic. `mise exec -- just
  t -p rcad-topo --test fuses_that_touch a_bends_cap_flush_against_a_block
  --no-capture`.
- `error.rs`'s first example for `TheFuseWroteABrokenAnswer` is now history.
  No test in `rcad-topo` pins a sound pair that still ends there.

**The writer fuzzer's checked-in seed was drawn again.**
`fuzz/corpus/no_model_makes_the_writer_panic/a_quarter_bend_of_pipe` held
the old bend as `postcard` bytes, and it measured `VolumeIsNotPositive {
volume: -84.29124343776743 }`, so `every_shape_seed_is_the_shape_it_is_named_for`
failed. The builder alone was enough for the
mutator: `every_seed_mutated_by_a_fixed_seed_still_decodes_into_a_model`
passed. A throwaway probe drew `a_quarter_bend(model, 6, 2)` in a 100 mm model
again. The new bytes are 681 long like the old, differ from them in 5 bytes,
validate, have faces plane, plane and torus, have their middle at
`(3.9254, 3.9255, −0.0021)` against the test's `3.9257`, and are refused by
the writer. Those are the test's four checks. The corpus was outside the first
run's write set, so a second run (`lane-q667-land-the-turned-torus-mesh-2`,
off `6273ce5e`) merged the first without a rebase and checked the new seed in
on its own, sha256 `86943d84…f87a`: `mise exec -- just t -p rcad-step --test
every_model_the_writer_fuzzer_kept_is_written_again` ran 13 and passed 13,
and `mise exec -- just gate quiet` passed 2825 of 2825.

**One doc outside the row is left stale.** The module doc of
`a_plane_is_never_one_surface_with_a_curved_face.rs` says its cube and bend
twenty units apart are refused `TheFuseWroteABrokenAnswer` for the union and
for the bend less the cube. On the second run's branch both answer, the
certified log empty and `Model::validate` passing: the union two solids, the
cube's `8` exactly and the bend's `19.7061 ± 0.3921` at deflection `0.01`
(`2π² = 19.7392`), and the bend less the cube one solid, the bend. No test
there pins the refusal, so nothing fails; Q463 holds the file.
