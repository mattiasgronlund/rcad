## 127. The `FaceKey` half lands, and the corpus line says `#15`, fixed 2026-09-18

§123.5's three files, the §113.4 move one entity kind along.

`Error::ADraftFaceCouldNotBeMeshed { face: usize, why: rcad_mesh::CannotTriangulate }`
sits beside `ADraftEdgeHasACrossingThatCannotBeFollowed` in `error.rs`. `Written`
gained a `faces: Vec<FaceSlot>` beside its `edges`, filled by `write_into` in the
same pass; `which_described_face_the_sweep_meant` reads a `CannotMeshAFace`'s
`FaceKey` back against it exactly the way `which_described_edge_the_sweep_meant`
already read an `EdgeKey`, and `SolidBuilder::build` chains the two `map_err`s.
`brep.rs` gained one arm in `which_entity_the_builder_meant`, reading
`face_names` — already built, already indexed the right way, nothing else to
change there.

**The corpus line does say `#15`.** Twenty KiCad `LED_*` files now publish
`ThisEntityOfTheSolidWasRefused` naming an `ADVANCED_FACE`, where before every
one of them named only the solid — `LED_D3.0mm_Horizontal_O1.27mm_Z10.0mm.step`
is the first, its `MANIFOLD_SOLID_BREP` is `#15` in the file's own text
(`grep MANIFOLD_SOLID_BREP` on it, not assumed), and the mapping now names the
face the mesher gave up on rather than the solid as a whole.

**Two tests, at two levels.**
`builder::tests::a_face_the_sweep_cannot_mesh_names_its_own_position` in
`rcad-topo` builds a tube whose "hole" is exactly as wide as its own wall —
two cylinders of the same radius, each capped by an annulus of zero width —
through `SolidBuilder` alone. Nothing before the mesh notices: `check_curved_face`
asks only that a corner sit on its own surface, `check_flat_face` asks only
that each wire of a cap wind the right way round, and neither asks where the
other wire of the same face sits. Reverting the second `map_err` and rerunning
shows the failure this section closes, verbatim:
`CannotMeshAFace { face: FaceKey { slot: FaceSlot(3v1), model: ModelId(2) },
why: LoopsTouch { corner: 4, side: 0, at: Point([2.0, 0.0, 0.0]) } }` — a key
of a model nothing outside `build` ever holds. With the fix restored it is
`ADraftFaceCouldNotBeMeshed { face: 2, .. }`, `2` being the bottom cap's own
position in the description.
`a_face_the_mesher_cannot_cut_up_names_its_own_advanced_face` in `rcad-step`
is `a_crossing_the_marcher_gave_up_on_names_its_edge.rs`'s shape one entity
kind along: whichever corpus files bottom out in `ADraftFaceCouldNotBeMeshed`,
every one of them names an `ADVANCED_FACE`, with a floor of one file rather
than a pinned count or a named file, for the same reason §112 gave — a rescue
can empty the set, and emptying it is good news a pinned test would fail on.

~~`mise exec -- just gate` is green on this tree.~~ **It was not, and nobody
had run one:** §127.1.

Commands run to reach this: `git log --oneline -1`; `git checkout -B
lane-face-key d1e573a4`; `mise exec -- just decision 123`, `123.5`, `113.4`;
`mise exec -- just t a_face_the_sweep_cannot_mesh_names_its_own_position`;
`mise exec -- just t a_face_the_mesher_cannot_cut_up_names_its_own_advanced_face
--nocapture`; `grep MANIFOLD_SOLID_BREP` on
`kicad/LED_D3.0mm_Horizontal_O1.27mm_Z10.0mm.step`; `mise exec -- just gate`
(started, never read — §127.1).

### 127.1 The gate the lane started and never read, and the one test it would have caught

The sentence above was written while a background gate was still running. The
lane's last words were *"standing by for the gate to finish; will report the
exit code once the background run completes"*, and then the session it ran in
ended. So "green on this tree" was never a measurement — it was a forecast, and
it was wrong.

`a_two_step_face_whose_two_edges_are_the_same_curve_is_caught_by_the_mesh` in
`crates/rcad-topo/tests/a_face_bounded_by_two_steps.rs` asserted the old
`Error::CannotMeshAFace`. That refusal now arrives out of `SolidBuilder::build`
as `ADraftFaceCouldNotBeMeshed { face: 0, why: CornersCoincide { position: 3 } }`,
which is this section's whole point, so the test was asserting the answer
§127 exists to change. It fails on `lane-face-key`'s own base as readily as on
the merged tree: `git diff d1e573a4..ef787559` on that file is empty, so this
is the lane's own regression and not the merge's.

**The write set is where it went wrong, not the change.** §127 touched
`error.rs` and `builder.rs` and stopped there; the test that reads the variant
it replaced lives in `rcad-topo/tests/`, outside what the lane was given. That
is the habit already booked once this week — a lane whose change flips an
answer gets the tests asserting the old answer inside its write set from the
start — and this is its second instance.

The fix is one assertion, and it is strictly stronger than what it replaces:
the pattern now pins `face: 0` as well as the triangulator's reason, so the
position §127 adds is asserted at the `SolidBuilder` level by two tests rather
than one. `below` is the first face handed to the builder in `a_segment_plate`
and the first of the two zero-area flats the mesher reaches.

**What it does not change:** every other `CannotMeshAFace` site in the tree is
a `SolidView::mesh` refusal, not a `build` one, and `build` is the only path
`which_described_face_the_sweep_meant` sits on. Checked, not assumed —
`a_solid_with_a_cone_face.rs`, `how_far_out_a_ball_can_sit_and_still_mesh.rs`,
`whether_a_coarser_deflection_always_meshes.rs`,
`a_curved_solid_through_the_front_door.rs`, `measure.rs`, `mesh.rs` and
`rcad/src/lib.rs` all read or name the variant on the mesh path and are right
to keep it.

Commands run to reach this: `mise exec -- just gate` on the merged tree (FAIL,
exit 100, one test); `mise exec -- just t
a_two_step_face_whose_two_edges_are_the_same_curve_is_caught_by_the_mesh
--no-capture`; `mise exec -- just t --no-fail-fast` (1999 run, 1998 passed, 1
failed — so the one failure is the only one); `git cat-file -e
d1e573a4:crates/rcad-topo/tests/a_face_bounded_by_two_steps.rs` and `git diff
d1e573a4..ef787559` on it; `grep -rn 'CannotMeshAFace\|ADraftFaceCouldNotBeMeshed'`
over `crates/`; `mise exec -- just gate` again after the fix.

