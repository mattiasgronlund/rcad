## 166. A lune's pole side runs between its two meridians, not a whole turn — §165.4's quarter ball fixed, 2026-09-19

*A quarter of a unit ball built face by face passed `Model::validate` and
measured `5.209` where `π/3` is the truth. The cause is in the mesher: crossing a
pole, it moved the walk a whole turn round the ball, which is right for a ball's
own seam turning back on itself and wrong for a lune, whose wire arrives at the
pole down one meridian and leaves up another. So the lune's skin came out a whole
turn wider than it is. The quarter is a sound solid and the answer exists, so this
was a gap and the mesher is what changed: a pole side now runs from the meridian
the walk arrived on to the one it leaves by, a whole turn only when those are one
edge, and the way it runs is read off where the walk came from rather than
alternated from the first pole crossed. The quarter now measures `1.0424 ± 0.0315`
against `1.0472`. `classify_point` still refuses on the wedge, and that is a
different cause, left open.*

### 166.1 The cause, printed

`BuildingAMesh::walk_a_wire_round` turns each wire into corners in the sphere's
own two coordinates, longitude first and latitude second, and a
`WireStep::Pole` goes through `WalkingAWire::cross_the_pole`, which added a whole
turn to the running longitude and flipped the sign for the next pole. A temporary
`eprintln!` of the sphere face's walked corners in `cut_up`, on the quarter at
`[0°, 90°]`, the first and coarsest of the walks it printed:

```text
before: [[0.0, 1.5708], [0.0, 6.1e-17], [0.0, -1.5708], [6.2832, -1.5708], [7.8540, -1.5708], [7.8540, -6.1e-17], [7.8540, 1.5708]]
after:  [[0.0, 1.5708], [0.0, 6.1e-17], [0.0, -1.5708], [1.5708, -1.5708], [1.5708, -6.1e-17], [1.5708, 1.5708]]
```

Down the meridian at longitude `0`, then along the south pole: before the fix
that side ran from `0` through `2π` to `5π/2`, a turn and a quarter, and the patch
closed back over the north pole from `5π/2` to `0`. A sphere face `5π/2` wide has
area `5π`, which is the "about `5π`" §165.4 measured. After, the pole side runs
`0` to `π/2`.

**Why the three quarters measured right**: the whole turn was added and then half
of it taken back. After the pole the next corner is unwrapped the short way from
where the walk stood, so the side came out `2π + d` for a gap `d` between the
meridians taken the short way. For `d` under half a turn that is a whole turn too
wide; the three quarters' meridians are `3π/2` apart, the short way is `−π/2`, and
`2π − π/2` is right by luck. So every lune narrower than half a turn was a turn too
wide, and every wider one was right.

**Not a `validate` defect.** The quarter's faces meet along their edges, each wire
closes, the three quarters built by the same function with the same step order
measure right, and the answer exists: nothing about the model is malformed. The
mesher asked the wire the wrong question.

### 166.2 The fix

All in `crates/rcad-topo/src/mesh.rs`:

1. **A pole side between two uses of one edge is a whole turn, as before**
   (`a_seam_turns_back_at`). That is every ball the kernel builds and every whole
   closed surface the fuse writes (`fuse/assemble.rs` writes a `WireStep::Pole`
   only for `is_a_whole_closed_surface`), so what they walk is unchanged.
2. **Between two different edges it runs to the next meridian**, however far that
   is in the direction the side runs, from just over nothing to a whole turn
   (`how_far_a_pole_side_runs`). Its far end is not known at the pole, so
   `WalkingAWire` holds it open (`crossing`) until the next corner with a
   longitude of its own arrives, or until the loop closes on its first corner.
3. **Which way it runs is read off the walk**: arriving at a pole from above, the
   pole is the bottom of the patch and the side runs with the longitude; from
   below, against it (`which_way_the_pole_side_runs`). A ball's own coordinates are
   right-handed about its outward normal and a wire keeps its face on its left,
   so a patch is walked anticlockwise. The sign used to start at `+1` and
   alternate, which is the same thing only when the first pole the walk crosses is
   a bottom one.
4. **A walk no longer opens on a pole step** (`steps_from_somewhere_that_has_a_longitude`):
   with no corner off a pole, it opens on the first edge instead. A lune's wire
   has no corner off a pole, and opened on a pole step there is nowhere walked to
   say which way the side runs. A pole side still open when the first corner with
   a longitude arrives, which only an edge with no points strictly along it can
   cause, is refused as a wire that will not lie flat rather than guessed.

Item 3 is not tidiness. With items 1, 2 and 4 in and the sign still alternating,
a quarter whose sphere wire is written starting up the second meridian is refused
by `SolidBuilder::build` with `Invalid(FacesDoNotAgreeWhichWayTheyFace)`: the
walk goes the long way round the first pole, the wrong way.

### 166.3 The test, and what it printed before the fix

`crates/rcad-topo/tests/a_quarter_of_a_ball_built_face_by_face.rs`, two tests
against a closed form: a wedge spanning a fraction `f` of a turn holds `f · 4π/3`
and has `f · 4π + π` of skin, and the mesh must land within its own published
`volume_error` and `area_error` of both, at a deflection of `0.01`.

- `a_quarter_of_a_ball_measures_a_quarter_of_a_ball_wherever_it_sits` — the
  quarter at `[0°, 90°]`, `[−45°, 45°]` and `[135°, 225°]`, and the three
  quarters as the control that measured right before.
- `a_quarter_of_a_ball_measures_the_same_whichever_step_its_skin_starts_at` —
  the quarter at `[0°, 90°]` with its sphere wire written from each of its four
  steps, two of which open on a pole step and one going up.

The builder is a copy of `a_wedge_of_a_ball` from `fuses_that_must_agree.rs`,
widened to the diagonals and to a starting step; a test file cannot reach
another's functions. Against the mesher at `146d0d62`, both failed:

```text
the wedge between [[1.0, 0.0], [0.0, 1.0]], skin starting at step 0, should hold 1.0471975511965976, measured 5.209149896376255 give or take 0.15767215351449354
```

After the fix, printed by a probe since removed:

| wedge | volume, measured ± error | true | area, measured ± error | true |
|---|---|---|---|---|
| quarter `[0°, 90°]` | `1.04240 ± 0.03155` | `1.04720` | `6.26855 ± 0.16582` | `6.28319` |
| quarter `[−45°, 45°]` | `1.04231 ± 0.03138` | `1.04720` | `6.26839 ± 0.16498` | `6.28319` |
| quarter `[135°, 225°]` | `1.04231 ± 0.03138` | `1.04720` | `6.26839 ± 0.16498` | `6.28319` |
| three quarters | `3.12257 ± 0.08865` | `3.14159` | `12.53044 ± 0.27420` | `12.56637` |

`just gate quiet` on the code commit, read back: `gate: PASS`, `2098 tests run:
2098 passed, 1 skipped`, `3 tests run: 3 passed, 0 skipped`, `doc tests: 25
passed, 0 failed, across 9 crate(s)`.

### 166.4 What is left

**`classify_point` still refuses on the wedge.** Asked at `(0.3, 0.3, 0)`,
`(0.5, 0.2, 0.3)`, `(−0.3, −0.3, 0)`, `(0.3, −0.3, 0.1)` and `(0.1, 0.6, −0.5)`, every
one of the four wedges above answers `CannotFollowAFacesTrimming` naming face slot
1, after the fix as before. Whether that slot is the sphere face or a half-disc
was not checked, and neither was which edge it cannot follow. `classify.rs`
names nothing in the mesher, and the answers did not move with the fix.
A named refusal and not a wrong answer, so it is reported and left.

Write set: `crates/rcad-topo/src/mesh.rs`, the new test file, this section, and
the pointer at the end of §165.4.

**§167 answers this**: slot 1 is the sphere face, all three faces refused, and
a wedge of a ball now says which points are inside it.

