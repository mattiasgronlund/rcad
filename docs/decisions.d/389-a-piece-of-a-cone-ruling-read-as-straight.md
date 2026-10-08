## 389. A piece of a cone's ruling read as straight — 2026-10-03

*Queue row Q814 (refuses/B, §334, §16.10), which re-ran Q650. Q650's lane made
the `holds.rs` changes on `lane-q650-a-half-frustum-and-a-block-are-fused`
(`2c019161`, `7c3cc087`, off `d34654bd`), merged here onto `b1136e72` with its
shas kept. The tests are in `a_frustum_cut_in_half_down_its_axis.rs`,
`a_flat_cap_cut_by_a_cones_rim.rs`, `a_solid_carrying_a_run_along_a_cones_ruling.rs`
and `holds.rs`'s own.*

The half frustum built face by face, fused with the block
`[−10, 10] × [−10, 10] × [6, 12]` across its top, stopped in `select.rs` under
all six fuses (§334.1, corrected there). Every answer exists, so that was a gap
(§16.10). Three changes to `holds.rs`, and all six answer: one solid each, the
model validates, a corner at each of `(0, ±3, 6)`, and the closed-form volume.

### 389.1 A piece of a ruling is the segment between its corners

`is_a_piece_of_a_ruling` admits an edge stored as part of where a plane crosses
a cone when the plane's own sign at the cone's apex is exactly zero and the
edge's two ends are in line with the apex: three `orient2d`s, the three
components of `(start − apex) × (end − apex)`, each exactly zero.

A plane through the apex cuts a cone in straight lines through the apex — two,
one, or none — so a piece of that crossing is straight unless it turns at the
apex from one line onto the other. Its two ends in line with the apex rule the
turn out: a piece that left the line at the apex could only come back to it at
the apex, and an edge does not pass one point twice.

It is read that way in three places: `walk_one_outline`, so a flat face whose
sides are all straight or rulings is walked as a polygon;
`a_side_of_it_is_part_of_a_crossing`, so such a flat face goes to that walk;
and `every_reading_of_the_sides_of`, so a cone face's ruling side is a
straight side. Every
number asked is stored, so a corner rounded off its line fails the test and is
refused as before, and a plane leaned off the apex cuts a curve and is never
read as straight.

### 389.2 A cut side by side with the floor costs nothing on the skin

The half frustum's skin is cut by three planes: the small cap, which the apex is
behind (the floor); the big cap, which the apex is in front of; and `x = 0`,
through the apex. The arm for one cut the apex is behind took at most two cuts
in all. A cut whose normal lies along the floor's is, as a function of space,
`μ` times the floor plus a constant, so along the ruling at `φ` the condition
for a point to survive both is `a(φ)·(h − g·μ) ≥ 0`, whose sign does not depend
on `φ`: every ruling or none. So such a cut is not counted, and the arm still
allows one other. `is_a_family_this_follows`'s doc carries the argument.

### 389.3 At most one cut through the apex, and why

Along each ruling a plane through the apex keeps all of it or only the apex, by
the sign of how fast it grows along that ruling. That rate is affine in `cos φ`
and `sin φ`, so one plane keeps one arc of the circle of rulings. Two arcs can
meet in two: keeping `|x| ≤ z/10` on the cone of slope a half leaves
`|cos φ| < 1/5`, one run round `+y` and one round `−y`, two sheets that meet
only at the apex. `held_in_front_of_all` would then hold the other sheet's
points, which is a wrong answer.

Before 389.1 such a face never reached that arm: its ruling sides were refused
first, as arcs. After it they would have reached it, so the cone arm refuses two
or more cuts through the apex.
`two_planes_through_the_apex_leave_a_nappe_in_two_sheets` walks the rulings.

### 389.4 The half frustum's answer is the right one

Q650 did not merge because
`an_arc_trimmed_face_refuses_point_in_solid_by_name_and_a_closed_one_does_not`
moved from `CannotFollowAFacesTrimming`, naming the cone face, to `Inside` for
`(1, 0, 6)`. Against the half frustum's own sizes:

| Face | Its side | `(1, 0, 6)` |
|---|---|---|
| bottom cap, `z = 4`, facing down | `z > 4` | `6 − 4 = 2` |
| top cap, `z = 8`, facing up | `z < 8` | `8 − 6 = 2` |
| flat face, `x = 0`, facing `−x` | `x > 0` | `1` |
| cone of slope ½, nappe `z > 0` | `x² + y² < (z/2)²` | `1 < 9` |

The point is strictly on the solid's side of all four. The half frustum is
convex, being the inside of one nappe cut by three half-spaces, so it is exactly
the part of space on the solid's side of all four, and the point is inside and
on no face. `Inside` is right. The test is now
`point_in_solid_answers_a_half_frustum_as_its_own_sizes_say`, and it asks seven
points, each checked against that arithmetic before the kernel is asked.

### 389.5 The two things Q650's lane did not run

**The fold comes back without the picker question.** With
`place_the_corners`'s `nothing_rules_the_point_out` replaced by `true` on this
branch, all six fuses come back `Invalid(EdgeHasNoLength)`. The sentence in
`a_half_frustums_corners_on_two_rulings_stay_two_corners`'s doc stays.

**A quarter of a pointed cone refused, and now answers (Q820).** It is the
`x ≥ 0`, `y ≥ 0` part of the cone of slope a half, eight tall, built face by
face. Its skin is cut by `x = 0` and `y = 0`, both through the apex, so 389.3
asks `is_a_family_this_follows`'s two-apex-cut arm,
`do_two_planes_through_the_apex_meet_inside`, which it passes: the two planes
meet along the axis, inside the cone, so the wedge they keep is one run of
rulings, `0 < φ < 90°`. Reading the skin's two sides that start at the apex
wanted its own fix: `which_side_of_its_neighbour_a_side_keeps` now reads such a
side at its *end* instead, since the cone has no gradient at the apex itself
(§389's own doc on that function). Measured on `b1136e72` and after Q820:

| Asked | `b1136e72` | After Q820 |
|---|---|---|
| the skin, at any point | refused, `AFaceTrimmedByACurveThisCannotFollow` | answers, right |
| each flat triangle | refused, the same name | answers, right |
| `classify_point((1, 1, 6))`, inside | `CannotFollowAFacesTrimming`, the skin | `Inside` |
| `(3, 3, 6)`, `(1, 1, 9)`, `(1, 1, −6)` | `Outside` | `Outside` |

`a_quarter_of_a_pointed_cone_holds_its_points` pins it, on either nappe, in
place of
`a_quarter_of_a_pointed_cone_has_its_skin_refused_by_name_though_it_is_one_piece`.
The other three quarters of the same cone still have their skin and cap
refused by name, which is a gap and not this one: `holds.rs` follows each
face's two cuts, but the part in front of both is the opposite quarter, so
answering the other three needs "in front of any", which `holds.rs` has only
for a sphere's faces and two discs together. Filed as Q943.
`three_quarters_of_a_pointed_cone_have_their_skin_and_cap_refused_by_name`
pins it.

**Since Q1009 (`0e4c04d5`), the skin answers too, as the part in front of the
cap and of either plane.** `the_cap_and_the_two_cuts_a_cone_face_is_in_front_of_either_of`
(`crates/rcad-topo/src/holds.rs`) takes a cone face cut by two planes through
its apex that meet strictly inside the cone and one closing cap the apex is
strictly in front of, whose side on each plane lies strictly behind the
other and in front of or on the cap, and whose rim runs from a corner on one
plane to a corner on the other, leaving the first in front: the part behind
both planes is one run of rulings, the rim leaving one plane in front is the
rest of the cap's boundary curve, and what is left is one piece with the
face's own boundary. The test is renamed
`three_quarters_of_a_pointed_cone_hold_their_points`
(`crates/rcad-topo/tests/a_solid_carrying_a_run_along_a_cones_ruling.rs:591`)
and runs on both nappes, each ruling and the rim held, the missing quarter's
points not, and `classify_point` checked over a `16³` grid against the closed
form.

### 389.6 What moved, and what this leaves

- Five tests fail with `holds.rs` put back to `b1136e72`:
  `a_half_frustums_corners_on_two_rulings_stay_two_corners`,
  `the_face_down_the_axis_holds_its_four_corners_and_not_the_other_nappe`,
  `the_skin_of_a_half_frustum_holds_its_own_half_between_the_caps`,
  `point_in_solid_answers_a_half_frustum_as_its_own_sizes_say` and
  `a_quarter_of_a_pointed_cone_holds_its_points`.
- §334.1's last paragraph is corrected: the six fuses stopped at the cone skin,
  not the flat face.
- The other three quarters' skin and cap, refused by name for want of "in
  front of any" (389.5), filed as Q943, both answer since Q1009
  (`0e4c04d5`): nothing is left of this item.
