## 192. The LED dome's wire walks, the cone's two apex checks agree, and `#223` hides the dome — 2026-09-21

*One batch of three lanes, dispatched by §175's skill off `f6820853`
(`just gate quiet` here: PASS, 2162 run, 2162 passed, 2 skipped). Every writing
lane had a verifier. Merged as `3326fa85`: `lane-led-dome-wire` (`3b35db9e`),
`lane-cone-range` (`c6db7b92`) and one test commit of mine. `just gate quiet`
here on `3326fa85`: PASS, 2164 run, 2164 passed, 2 skipped. `lane-pin-hole-face`
changed nothing and reported instead. A number below is a lane's unless it names
a command run here, and the verifier re-ran each number it quotes.*

  192.1   The dome's wire walks
  192.2   `#223` is reported, but the dome's triangulation is what stops them
  192.3   The cone's dynamic-range arm skips the corners the pole check calls the apex
  192.4   What is left

### 192.1 The dome's wire walks

§190.2 left the LED domes refused by `AWireWillNotLieFlatOnItsFace`. The lane
counted **18** `LED_D*_Horizontal` files refused this way at `f6820853`, not
the twelve §190.2 named. It found the cause with a print in
`walk_a_wire_round`. In the ball's own coordinates, the dome's rim is two
meridians half a turn apart, running through both poles. The file's corner is
the north pole. The quarter-turn second corner (§190.2) left the south pole
inside a piece of the rim. At a pole, the walk opened the pole side the short
way round. `the_short_way_round` answers exactly +π with −π, because `round`
rounds away from zero, so the walk came back a whole turn from where it began.
On the `D3.0 O3.81 Z6` dome it began at −1.571 and ended at −7.854.

The fix has two halves, and the lane measured that each alone is not enough:

- `mesh.rs`: an ordinary corner at a pole, reached once the walk has a
  longitude, opens its pole side by the face-on-the-left rule `cross_the_pole`
  already uses. `finish()` no longer overwrites that side.
- `brep.rs`: the second corner of a checked closed circle is the **opposite
  point**, which on every one of these domes is the other pole. This reverses
  §190.2's measured choice of a quarter turn. That choice was right for the old
  mesher and is wrong for the new one. With the quarter turn plus the mesh rule,
  18 files stay on the dome. With the opposite point and no mesh rule, 20 do.
  With both, none does.

The KiCad count stays **140 of 323**. The refused set is the same 183 files
before and after; the lane diffed the two lists and the verifier re-ran both
counts. `an_led_dome_wire_lies_flat.rs` names the `Z6` file and fails at base,
which the verifier confirmed by reverting both source files.
`a_sibling_led_gets_past_its_body_and_its_dome_wire` in
`a_bar_capped_by_a_dome_of_its_own_radius_imports.rs` is my re-pin: that LED now
stops on `#223`, and the gate is what caught it.

**Not handled:** a rim that passes a pole with no corner there. None does in the
corpus now, because the opposite-point corner makes the pole a corner. The
lane's print shows the stored rim misses the pole by about 7e-17, and the mesher
cannot put a pole in the middle of an edge.

### 192.2 `#223` is reported, but the dome's triangulation is what stops them

`lane-pin-hole-face` found neither the triangulator nor the reader wrong about
`#223`. At coarse deflections (Z10: 6.95, 3.475, 1.7375) the half-circle edge
`#98` gets only two chords, and the outer loop of `#223` really does cross a pin
hole once it is flattened. The brep wire is right: three circle edges and two
square holes, all inside the disc with 0.0956 to spare. At 0.869 and finer,
`#223` meshes. Every finer pass then fails on the dome (FaceSlot 17, a ball of
radius 1.5) with **`CornersCoincide`**. The dome's seam from the pole to the rim
is used twice, and in the ball's chart both uses land on the same points: corners
8 and 10 are both `[4.712389, 0.785398]`. So the patch is a slit.

`validate.rs`'s `the_coarsest_mesh_whose_sides_do_not_cross` counts
`CornersCoincide` as "sides cross" and keeps halving. At the end it hands back
the **coarsest** refusal. Its doc comment says the finer refusals "said the same
thing about the same solid", and here that is false: they were about a different
face, for a different reason. So `#223` is what gets reported, and the dome is
what stops the file. The lane reports that the other 16 LEDs refused on a pin
face may be the same.

All of this comes from prints the lane added and then removed; nothing was
committed. That is a diagnosis, not a demonstration, and §192.4 treats it that
way.

### 192.3 The cone's dynamic-range arm skips the corners the pole check calls the apex

§190.5's defect: a pole 2^-50 up the cone's ruling was refused as
`DynamicRangeTooWide` (1.8e16). The pole check accepts a corner as the apex when
a box around it reaches the stored apex. The box's half-width is the larger of
the corner's recorded slop and `what_writing_a_point_down_costs`, which is 16 ×
`f64::EPSILON` × the largest coordinate: about 2.8e-14 for this cone. A pole
2^-50 up the ruling is inside that box, so the pole check calls it the apex and
the cone should build.

Both checks now call one function, `Model::stands_where_it_pinches`, with the
same reach. No new threshold. The new test,
`a_pole_inside_the_room_of_the_apex_up_the_ruling_builds`, fails at base with
the 1.8e16 ratio, which the verifier reproduced to the last digit.

§190.5 also asked for this demonstration: with the pole check switched off, the
halfway-up pole **builds**, a folded model accepted. The 2^-40 pole is refused
under the wrong name, `DynamicRangeTooWide` 1.76e13. Both existing refusals
therefore depend on the pole check. `fuse/assemble.rs`'s `check_dynamic_range`
measures only edge chords and has no cone arm, so it cannot have this defect.

### 192.4 What is left

- **The dome's slit**, §192.2. The seam used twice lands on the same chart
  points, and the triangulator refuses coincident corners. This is `mesh.rs`
  again, and the next blocker of all 18 LEDs if the lane's reading holds.
- **`the_coarsest_mesh_whose_sides_do_not_cross` reports the wrong face**,
  §192.2. It returns the coarsest refusal even when a finer one names another
  face. Also open: whether `CornersCoincide` belongs in `its_sides_cross` at
  all, since a wire that doubles back does not go away with a finer mesh.
- A rim through a pole with no corner there, §192.1.
- Carried over from §190.6: a closed run carrying `and`, the plain-f64 circle
  check in `brep.rs`, `builder.rs`'s two-step hole, `squared` onto
  `Interval`, the turned-bend band and `fuse::bake`'s corner test.

Commands: `just decision 190.2`, `190.5`, `16.10`; the `lanes` workflow run
`wf_d61b137f-407`; `just gate quiet` on `f6820853` and `3326fa85`.
