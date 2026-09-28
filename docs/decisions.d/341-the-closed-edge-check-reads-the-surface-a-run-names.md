## 341. The closed-edge check reads the surface a run names — 2026-09-28

*Queue row Q41, from §229.4. Lane `lane-q41-the-run-names-the-partner-3`, off
`91896051`, merging the first run's `lane-q41-the-run-names-the-partner`
(`de22591a`). `Model::validate`'s
`check_this_solids_closed_edges_are_walked_the_way_their_surfaces_wind` read
each use of a closed edge against the face across it. It now reads it against
the second surface `which_two_surfaces_a_run_is_between` names, which is the
face across only where the two faces really cross. **A gap** (§16.10): the
direction was a fact, and the check read it off two spellings of one cone.*

### 341.1 What changed

Where the run names the face across, nothing changed. Where it names a third
surface — both faces on one surface, on two spellings of it, or on a ball and a
pipe that only touch — that surface is the partner. It is no face of the
solid, so it has no use to turn it round: its own gradient goes in, and the
face's side of it is read off the corners that are not on it
(`which_side_of_a_surface_a_run_names_this_face_lies_on`). The corners on it
are the ends of every edge of the face whose run names it.

Which way the partner faces does not matter. Turning it round turns both the
gradient the rule reads and the side the face is on, and the rule reads their
product. Every new test writes the plane both ways.

### 341.2 What it settles

Each of these fails with the check reading the face across, measured by
forcing that branch on this lane's tree:

| shape | reading the face across | reading the named surface |
|---|---|---|
| §229.4's frustum, cone written twice one ulp apart, from a file | refused `AClosedEdgeIsWalkedAgainstItsSurface` | validates, measures `312π` |
| the same frustum built by hand, walked the rule's way | refused the same | builds and validates |
| that frustum walked against the rule, bit for bit or one ulp apart | the check passes it; the mesh refuses `AWireWillNotLieFlatOnItsFace` | refused `AClosedEdgeIsWalkedAgainstItsSurface` |
| a half ball on a pipe of its own radius, walked against the rule | the check passes it; the mesh refuses `AWireWillNotLieFlatOnItsFace` | refused `AClosedEdgeIsWalkedAgainstItsSurface` |

Bit for bit, the two cones' gradients cross to the zero vector, so the check
read nothing at all. One ulp apart they cross to `(0, 1.07e-14, 0)`, and the
lower band's far corner is `-7.1e-15` inside the upper cone. Both of those are
rounding, and together they said the lower band walks the circle the way it
does not.

Tests: `validate.rs`'s `a_frustum_in_two_bands_walked_the_way_its_cone_winds_is_sound`
and `..._walked_against_its_cone_is_caught`,
`rcad-step/tests/a_closed_circle_between_one_cone_written_twice.rs`, and
`rcad-topo/tests/a_half_ball_on_a_pipe_of_its_own_radius.rs`.

### 341.3 The first run and Q15's half ball

The first run was rejected for breaking
`two_bars_on_one_pipe_answer_the_same_however_the_pipe_is_written`. With Q384
(§336) under it, that test passes, and so does every test in `rcad-topo` and
`rcad-step` on this lane's merged tree.

Q15 measured the half ball on a pipe building in neither way round with this
change and Q384 together: the check read the ball's latitude against the rim's
plane, and the mesher read the ball's longitude against the pipe. Q14 has since
made the mesher read the same pair, and on this tree the half ball builds,
validates and measures its closed form in both face orders and with the plane
facing either way. `mesh.rs` needed only its doc.

### 341.4 Q399 is not settled by this

A rim whose corner is on the ball's pole reads nothing in either place. The
ball's longitude and latitude are both zero there, so the check's rule answers
`Zero` and skips the edge. The mesher's rule has nothing to read either, and
since Q14 its fallback is handed the ball and the plane, not the pipe, so the
walker's order is kept. Measured with a probe on this tree (the half ball of
§341.2 with its corners moved to `(0, 0, 1.5)` and `(0, 2, 1.5)`, not
committed):

| rim's plane faces | ball uses the rim | builds |
|---|---|---|
| up | forward | refused `AWireWillNotLieFlatOnItsFace` |
| up | reversed | yes |
| down | forward | yes |
| down | reversed | refused `AWireWillNotLieFlatOnItsFace` |

The same in both face orders. Each one that builds meshes each face on its own
side of `y = 0` and measures the closed form at `0.05`, `0.0137` and `0.001`.
So which way round a hand-built solid has to be written depends on which way
its rim's plane faces, which is not a fact about the shape. That is Q399's to
decide.

Commands: `mise exec -- just t -p rcad-topo`, `mise exec -- just t -p
rcad-step`; the forced branch was `if partner == across.surface() || true`, run
as `mise exec -- just t -p rcad-topo -p rcad-step -E
'test(/frustum_in_two_bands|half_ball_on_a_pipe|cone_written_twice/)'`.
