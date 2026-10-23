## 377. A half bend fused with a quarter bend on one torus — 2026-10-02

*Queue row Q753, which replaces Q197 (lane
`lane-q753-a-half-bend-and-a-quarter-bend`, off `0c8a44fc`). §232.6 left a
half bend's wall that `classify_uncertain_point` could not follow at any
point, and guessed that §243's step 3 was why. It was not. This lane changed
`crates/rcad-topo/src/holds.rs` and added
`tests/a_half_bend_fused_with_a_quarter_bend_on_one_torus.rs`.*

In short: the half bend's two caps are stored on one plane, so the wall never
reached §243's rule at all. The general rule refused it because every corner of
the wall is on that plane. The wall is now read as the half of the ring on the
side its wire keeps. Of the 24 fuses in 377.1, 14 now answer correctly. The
other 10, five for a quarter inside the half and five for a quarter against one
of its caps, stop later, in `fuse/assemble.rs`.

**Answered by Q756, 377.4**: those ten were §378.3's refusal, two circles on
one plane through the ring's axis given one name. With a closed crossing named
by which circle it is (§378.6), all 24 answer correctly.

### 377.1 The replay on the base

Q667 (§361) rebuilt every bend after §232.6 was written, so the lane measured
again first. Ring four, tube one, a half bend from `+x` round through `+y` to
`−x` (`rings::a_bend_turned_and_written_out(model, 4, 1, 2, 0)`), and the
quarter bend turned 0, 1, 2 and 3 quarter turns. Every pair was fused under
union, intersection and difference, with the half bend first and then second.
That makes 24 fuses, all run on `0c8a44fc` with `holds.rs` untouched:

- **All 24 refused** with `Undecidable`, on "a point in a piece that every
  solid could place". The point was `(0, 3, 0)` for quarters 0 and 1, and
  `(0, −3, 0)` for 2 and 3.
- `classify_uncertain_point` on the half bend answered
  `CannotFollowAFacesTrimming` on its wall at `(±4, 0, 0)` (its caps' centres),
  `(0, 4, 0)`, `(0, ±5, 0)` and `(5, 0, 0)`.
- `face_holds_the_point` on the wall answered
  `Err(NothingSaysWhichSideOfItsTrimmingAFaceLiesOn)` at all six. That is not
  `AFaceMissesPartOfWhereItsSurfacesCross`, the error §243's step 3 raises.

Command: a probe in the new test file, `mise exec -- just t -p rcad-topo --test
a_half_bend_fused_with_a_quarter_bend_on_one_torus --no-capture`.

### 377.2 The step that refused

The error named the step. To confirm it, a temporary `eprintln!` in
`what_a_curved_face_holds` printed what `what_trims_this_face` handed back for
the wall and whether `is_a_ring_face_between_two_planes` took it. Every time
it printed `1 trimmings, corners [2], between two planes false`, and then
`corners read None, trimmed once true`. A quarter bend's wall in the same runs
printed `2 trimmings, corners [1, 1], between two planes true`.

So: both caps are stored as the plane through the origin facing `−y`. These
two `Surface`s are equal, and `what_trims_this_face` gathers edges by the
surface across them, so the wall had **one** trimming with two edges along it.
`is_a_ring_face_between_two_planes` wanted two trimmings, so the wall went to
the general rule. That rule's `the_side_this_face_lies_on` skips every corner
on the trimming, and both of the wall's corners are on it, so it read nothing.
`the_side_the_apex_is_on` answers only for a cone. The result was
`NothingSaysWhichSideOfItsTrimmingAFaceLiesOn`, which `classify.rs` reports as
`CannotFollowAFacesTrimming`.

§243's step 3 does refuse one plane stored two ways, as §243.2 says, for
example a cap stored facing `+y` and used reversed. That shape makes two
trimmings, and each circle is on the other's plane. No builder here stores a
half bend that way. The unit test
`a_half_bends_wall_holds_the_same_half_with_its_near_cap_stored_turned_round`
builds one.

### 377.3 The fix, and why it is sound

The plane through both of a half bend's ends cuts the ring in two circles,
and cuts the rest of the ring into two halves, one on each side of the plane. A
face whose only edges are those two circles is one of the two halves. An
answer exists, so the refusal was a gap (§16.10). Which half it is gets read
off the face's own wire, not off a tolerance:

1. `is_a_ring_face_between_two_planes` now takes a ring face whose trimmings
   are all planes with **two edges between them**. That is two planes with one
   edge each, as before, or one plane with two.
2. `what_a_ring_face_between_two_planes_holds` already builds one cut per
   edge. Each cut checks that its plane holds the ring's axis exactly, and
   reads the side the face keeps off the way the wire walks that circle
   (§243.3). Nothing about that changed.
3. **New:** two planes that both hold the ring's centre, with normals along
   one line (`the_normals_lie_along_one_line`, exact), are one plane. Each
   cut's side is read against its own stored normal. Two such normals are
   exact multiples of each other, so whether they point the same way is
   whether the signs of one non-zero component agree, with no arithmetic. The
   two sides must be equal when the normals agree and opposite when they do
   not. Then the face is the part of the ring on that side of the one plane,
   and `point` is asked against that plane alone. If the sides disagree, the
   wire leaves one circle on each side of the plane, which no half of the ring
   does, and the face is refused with `AFaceMissesPartOfWhereItsSurfacesCross`.
4. Two planes that are not one plane go to step 3 of §243 as before.

**What the side check catches.** A wall whose two edges are the same circle,
far corner `(5, 0, 0)` on the near circle, is refused for every way of storing
and using its far cap
(`a_wall_round_one_circle_twice_is_refused_rather_than_read_as_a_half`). Its
wire walks that circle once each way, and the two sides disagree. The lane
first wrote a second check, that the two corners lie on different circles,
using the half-space `intersect` names each circle by. A probe without it
showed the side check refusing all three such walls first. So it was dropped,
which also saves an `intersect` per query.

*2026-10-06, Q790: the check is back. Those walls walk the circle once each
way. A wall that walks it twice the same way round, built straight into the
arena (`a_piece_of_pipe_walking_each_circle` in `holds.rs`), passed the side
check, and all six (two walks times three ways of storing and using the far
cap) were read as a half. `are_on_two_circles` now runs after the side check
passes, naming each corner's circle by the picker `intersect` tells it by, and
refuses the wall `AFaceMissesPartOfWhereItsSurfacesCross` unless the names
differ (`74780e25`, pinned by
`a_wall_round_one_circle_twice_the_same_way_is_refused_rather_than_read_as_a_half`).
The lane's probe, since removed, found `Model::validate` refusing all six.*

**One refusal is renamed.** A ring face with two edges along one plane that
does not hold the axis, such as the top half of a ring on `z = 0`, used to be
refused by the general rule with `NothingSaysWhichSideOfItsTrimmingAFaceLiesOn`.
It now reaches this rule, and step 1 refuses it with
`AFaceMissesPartOfWhereItsSurfacesCross`. Both are refusals. No test pins
either name for a torus face, and `SolidBuilder` refuses both forms of the top
half of a ring before any face is asked
(`building_a_solid_out_of_faces.rs`).

*Since 2026-10-04 (§243.8) this shape is answered rather than refused: a
plane square to a ring's axis is read the same way as one holding it, and the
flat half of a ring on `z = 0` is the piece its wire keeps.*

### 377.4 The answers now, pinned

Every answer that comes back is one solid or none, passes `Model::validate`,
and measures at deflection `0.01` to within its own `volume_error` of Pappus.
Each quarter of the ring is `2π²`, out of `2π²Rr² = 8π²` for the whole ring.

| quarter turned | rule | half first | half second |
|---|---|---|---|
| 0, 1 (inside the half) | intersection | `2π²` | `2π²` |
| 0, 1 | difference | `2π²` (the other quarter) | nothing |
| 0, 1 | union | `4π²` (the half) | `4π²` (the half) |
| 2, 3 (against a cap) | intersection | nothing | nothing |
| 2, 3 | difference | `4π²` (the half) | `2π²` (the quarter) |
| 2, 3 | union | `6π²` | `6π²` |

When this row landed, the three cells now `4π²` and `6π²` were refused
`ResultIsNotManifold { count: 4 }`, and the test pinned them so. Q756 moved
them; 377.5 says why they answer.

For example, quarter 0 with the half first: the intersection measured
`19.7061 ± 0.3921` and the difference `19.7061 ± 0.3899`, against
`2π² = 19.7392`. On the wall itself, `(0, 5, 0)` and `(−3, 4, 0)` are held,
`(0, −5, 0)` and `(3, −4, 0)` are not, and `(5, 0, 0)` and `(−3, 0, 0)` are on
its boundary. `classify_uncertain_point` puts the caps' centres `(±4, 0, 0)`
on the skin, `(0, 4, 0)` inside and `(0, −4, 0)` outside. All of this is in
`tests/a_half_bend_fused_with_a_quarter_bend_on_one_torus.rs`. The wall's
answers for the half bend built straight into the arena, both caps facing out
and one stored turned round, are in `holds.rs`'s
`a_half_bends_wall_holds_the_half_*` unit tests.

`mise exec -- just t -p rcad-topo --no-fail-fast` ran 1240 tests and all 1240
passed, `how_often_a_fuse_with_a_torus_refuses.rs` included, so no pinned
answer outside this row's write set moved.

### 377.5 What this leaves

**The ten `ResultIsNotManifold` refusals were the next stop, and Q756 answered
them.** They are five fuses, each at two quarter turns. They should answer the
half bend (`4π²`) for the union with a quarter inside it and for the half less
a quarter against its cap, and three quarters of the ring (`6π²`) for the
union with a quarter against a cap. `fuse/assemble.rs` raised the error
(`.map_err(|_| Error::ResultIsNotManifold { count: by.len() })`) after the
pieces were chosen, and all ten reported `count: 4`. This row left it untraced,
and Q788 was filed to trace and fix it once Q756 landed.

It was §378.3's refusal. The half bend's two caps lie on `y = 0`, a plane
through the ring's axis, and cut the ring in two circles, round `(4, 0, 0)`
and round `(−4, 0, 0)`. Assembly named a closed crossing by its two surfaces
alone, so both circles had one name: one input vertex was handed over to the
other and the two circles became one edge with four faces along it. Each of
the ten answers has both circles on `y = 0` as edges: the half bend's two cap
edges, or one cap edge and the seam where the quarter's wall meets the half's.
Q756 names a closed crossing by which circle of the ring it is as well
(§378.6), and changes nothing else in `crates/rcad-topo/src`. With the base's
`assemble.rs` put back, both moved tests fail on their first such fuse with
`ResultIsNotManifold { count: 4 }`, and with Q756's they pass. So **Q788 is
answered by Q756** and needed no fix of its own.

The pins moved with Q756's second run
(`lane-q756-two-quarter-bends-end-to-end-on-one-ring-2`). Each of the ten
comes back one solid that passes `Model::validate` and measures inside
`measure`'s bound at deflection `0.01`. The test prints each measurement:

| quarter turned | rule | half first | half second |
|---|---|---|---|
| 0 | union | `39.41224 ± 0.78427` | `39.41222 ± 0.78427` |
| 1 | union | `39.41224 ± 0.78427` | `39.41226 ± 0.78427` |
| 2 | difference | `39.41224 ± 0.78427` | (the quarter, as before) |
| 3 | difference | `39.41224 ± 0.78427` | (the quarter, as before) |
| 2 | union | `59.11849 ± 1.17640` | `59.11849 ± 1.17640` |
| 3 | union | `59.11833 ± 1.17640` | `59.11833 ± 1.17640` |

against `4π² = 39.47842` and `6π² = 59.21763`. Command: `mise exec -- just t
-p rcad-topo --test merged -E 'test(/^a_half_bend_fused_with_a_quarter_bend_on_one_torus::/)'
--no-capture`.

The first run did not run the test file against the base `holds.rs` in its
final form: its attempt to check the base file out over its own was refused by
the session's permissions, and the base replay in 377.1 stood as the evidence.
The second run (`lane-q753-a-half-bend-and-a-quarter-bend-2`) put the base
dispatch back by hand, the old `is_a_ring_face_between_two_planes` body and
the one-plane branch switched off, with the tests kept. Of the new file's
tests and the bend wall unit tests in `holds.rs`, 4 passed and 6 failed, with
the errors 377.2 names. Its verifier's own hand revert failed the same six.
