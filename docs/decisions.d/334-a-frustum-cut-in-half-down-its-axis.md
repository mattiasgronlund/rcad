## 334. A frustum cut in half down its axis, measured again — 2026-09-28

*Queue row Q19, from §215.2. Lane `lane-q19-a-half-frustum-cut-down-its-axis`,
off `96da3252`. No source changed. The tests are
`crates/rcad-topo/tests/a_frustum_cut_in_half_down_its_axis.rs`.*

§215.2 named a stopping place: `fold_the_nodes` refusing
`Invalid::EdgeHasNoLength` for a half frustum cut down its own axis, because
two corners on two different rulings were folded into one node. **That is no
longer true**, and this section corrects it, because §215.2's own file is not
this lane's to edit.

### 334.1 The fold is gone, and `f2fc7d1d` is what removed it

The corners of the hand-built half frustum, `(0, ±2, 4)` and `(0, ±4, 8)`, are
on the cone, and the cone crosses `x = 0` in two rulings. `place_the_corners`
used to put a corner on every crossing whose partner the model lists round it,
so each corner went on **both** rulings. A ruling is ordered along `z`, the axis
it climbs fastest, and `(0, 2, 4)` and `(0, −2, 4)` tie on `z`. So along the
wrong ruling the two corners were one point, and two different input corners
folded into one node is `EdgeHasNoLength`.

`f2fc7d1d` (2026-09-25, two days after §215) made the corner ask the crossing's
picker as well. **Demonstrated, not argued:** with that picker question replaced
by `true`, all six fuses of the half frustum and a block across its top come
back `Invalid(EdgeHasNoLength)`, and
`a_half_frustums_corners_on_two_rulings_stay_two_corners` fails. On the base
all six get through the arrangement and stop in `select.rs`, for want of a
point in a piece that both solids can place.

**Corrected by §389: that was not the flat face.** Traced on `b1136e72` in the
union with the half frustum first, each ray `select.rs` fired asked the top cap,
which answered, and then the cone skin, which refused; the flat face was never
reached. Asked on their own, the skin and the flat face both refused, and both
are bounded by pieces of the two rulings. Since §389 both answer, and so do all
six fuses, each one solid of the closed-form volume. The picker question above
still holds them apart: replaced by `true` on §389's branch, all six come back
`Invalid(EdgeHasNoLength)` again. `the_face_down_the_axis_is_still_refused` is
now `the_face_down_the_axis_holds_its_four_corners_and_not_the_other_nappe`.

### 334.2 Where the block's floor is decides the answer

The brief's shape is `cones::a_frustum(model, 0.5, 4.0, 8.0)` fused with a block
whose face `x = 0` runs down the axis. The block's face holds the two rulings
too, and on that face they run past the frustum and cross at the apex. Three
floors, six fuses each, all under `Policy::certified()`:

| Block | What all six do |
|---|---|
| `[−10, 0] × [−10, 10] × [2, 12]`, apex below the face | Answer. One solid each, the model validates, and every volume is `56π/3` or `2000 ± 56π/3` within the mesh's own error |
| `[−10, 0] × [−10, 10] × [0, 12]`, the brief's example | Refuse `Undecidable`, *which way a crossing runs*, on the block's face `x = 0` |
| `[−10, 0] × [−10, 10] × [−2, 12]`, apex inside the face | Refuse `CannotTellTheCellsOfAFaceApart` on that face — Q207's shape, not pinned again |

The first row is pinned with its volumes and with a corner of every answer at
each of the four points. The second is pinned by name.

### 334.3 The floor through the apex is a gap, and its fix is Q207's

With the floor at `z = 0` the apex is a node on the block's bottom edge, where
that edge meets both rulings. The arrangement reads which way a crossing runs as
`∇S × ∇Q` at the first node on it, and for each ruling that node is the apex.
The cone's gradient is nothing there, so the record's input is the zero vector.

Every answer exists, so this is a **gap** (§16.10). But it is not a gap in
*where* the direction is read. Two probes, both reverted:

- Reading at the **last** node on each crossing, which is off the apex: the
  floor at `z = 0` gets one step further and then refuses
  `TwoCurvesTouchWithoutCrossing` on the same face in all six. The apex node
  still has to put its ways out in order, and `way_out_of` there is `∇S × ∇Q`
  at the apex, which is nothing.
- The same change turns the floor at `z = 2` from six answers into six refusals
  (*a point inside a piece of a face*), and the record names `(0, 10, −20)` and
  `(0, −10, 20)` beside the frustum's own corners: ruling nodes on both nappes,
  where the block's walls `y = ±10` meet the two rulings. `∇S × ∇Q` points the
  opposite way on the far side of the apex. **So which node a ruling's
  direction is read at decides whether this shape answers**, and the first
  node, which the base reads, is one that works here. Why it is on the
  frustum's side — the rim crossings are drawn before the cuts, so their meets
  are placed first — is read off the code and not measured. That dependence is
  a finding for Q207, not a wrong answer this lane measured.

The fix needs one direction per ruling that holds at the apex and on both sides
of it, and a turn at a point where the cone has no gradient. That is family 3
of `how_a_crossing_runs`'s own note — store the direction on the crossing, and
give the turn a second form — and it is what Q207 needs too. Q207 also needs a
node where the two rulings cross; this shape has one already, from the block's
bottom edge.

### 334.4 What moved

- §215.2's `EdgeHasNoLength` stopping place: gone since `f2fc7d1d`.
- `the_way_into_an_ellipse_arc_in_a_fuse.rs`'s module doc called it "a defect
  of its own, still there"; corrected.
- `a_solid_carrying_a_run_along_a_cones_ruling.rs`'s
  `a_fuse_with_an_apex_run_in_it_either_answers_soundly_or_refuses_by_name`
  prints `undecidable` for all three rules, as before: no source changed.
