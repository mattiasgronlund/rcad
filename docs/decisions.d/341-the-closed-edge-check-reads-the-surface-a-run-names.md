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
decide. §341.5 decides it.

Commands: `mise exec -- just t -p rcad-topo`, `mise exec -- just t -p
rcad-step`; the forced branch was `if partner == across.surface() || true`, run
as `mise exec -- just t -p rcad-topo -p rcad-step -E
'test(/frustum_in_two_bands|half_ball_on_a_pipe|cone_written_twice/)'`.

### 341.5 At the pole the faces' own walks name the way

*Queue row Q665, which was Q417 and Q399. Lane
`lane-q665-a-half-ball-on-a-pipe-cornered-on-its-pole`, off `fc6d6c00`.*

**All four ways are one solid.** Where the rule reads nothing at the corner,
validate skips the edge, so nothing fixes which way the edge is stored running:
the ball using the rim forwards and the pipe backwards is the same solid as the
other way round with the edge's direction turned. What is a fact is which way
each face walks it. A face's material lies on its left, and left of
`outward × ∇P` is `P`'s negative side (§36.6), with `P` the second surface the
run is between. So a use walks along `outward × ∇P` when its face lies inside
`P`, against it when outside, and the edge runs the way its forward use walks.
That is validate's own rule with the reference that vanishes at the pole left
out, so it needs no longitude at all.

`BuildingAMesh::walk` reads it only where both readings at the corner, the rule
and the touch fallback, have said nothing, because it reads every corner of a
face. It asks the face walking the edge first and then the face across. The
ball's face has no corner off the rim's plane, so it has no side to read, and
the pipe's corner on the lid answers. The sides are read on the corners
validate reads, by a copy of validate's private
`which_side_of_a_surface_a_run_names_this_face_lies_on` in `mesh.rs`, which
§341.6 leaves to be made one.

The probe of §341.4 is now
`a_half_ball_on_a_pipe_cornered_on_its_pole_builds_however_it_is_written`, and
on this lane's tree every one of the eight builds, validates, meshes each face
on its own side of `y = 0` and measures `π·r²(2r/3 + h)` at `0.05`, `0.0137`
and `0.001`:

| rim's plane faces | ball uses the rim | before | after |
|---|---|---|---|
| up | forward | refused `AWireWillNotLieFlatOnItsFace` | builds |
| up | reversed | builds | builds |
| down | forward | builds | builds |
| down | reversed | refused `AWireWillNotLieFlatOnItsFace` | builds |

The same in both face orders. The "before" column was measured again here on
`fc6d6c00`, and matches §341.4. With the corner on the equator nothing moved:
forward builds in all four, reversed is refused
`AClosedEdgeIsWalkedAgainstItsSurface` in all four.

Commands: `mise exec -- just t -p rcad-topo --test
a_half_ball_on_a_pipe_of_its_own_radius`, and `mise exec -- just t -p
rcad-topo` (1261 run, 1261 passed).

### 341.6 What this leaves

- **One reading of a face's side, not two.** `mesh.rs` and `validate.rs` each
  hold `which_side_of_a_surface_a_run_names_this_face_lies_on`; validate's is
  private and outside this lane's files. Make one `pub(crate)` and drop the
  other. Small.
- **A circle through a ball's pole where the faces cross, not touch,** with
  both faces curved: a ball and a cone on one axis meeting in the ball's great
  circle through its pole. The touch fallback is asked before the faces' walks
  and would answer with the cone's longitude, which turns with the way the
  cone's axis is written. Nobody has built it.
