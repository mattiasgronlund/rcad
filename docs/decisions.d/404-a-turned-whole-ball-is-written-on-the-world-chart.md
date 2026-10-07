## 404. A turned whole ball is written on the world chart — 2026-10-05

*Q1010, lane `lane-q1010-a-turned-ball-written-on-the-world-chart` off
`39218a67`. It takes the road Q1003's read-only probe found, in place of
Q1004's poles stored on `Sphere`. It amends §189.1 for the writer and the
reader, and §366.4's count.*

### 404.1 The problem

This kernel's `Sphere` is a centre and a radius. It pinches on the `z` line
through its centre wherever it is put (§189.1), and `Sphere::transformed`
moves only the centre. A ball `add_ball` built and a placement turned so its
poles leave that line is still sound where it is stored, but its poles, read
where it is placed, stand where its located ball does not pinch. So the writer
refused it `AWireThatStandsStillAtAPole`: the reader would put the poles back
at the file's corners, and `Model::validate` refuses a pole there. It was 38
of §366.4's 400 draws, and Q928 and Q736 found no road inside `rcad-step`.

### 404.2 Two ways to close it, and the one taken

**Q1004: give `Sphere` its two poles**, turned with it as a cone's apex is.
Every reader of a ball's pinch or chart goes through the poles — Q928 named a
dozen places in `rcad-geom`, `rcad-topo` and `rcad-step` — and `Sphere`'s
saved form grows, so `FORMAT_VERSION` goes to 4 and the 960 models of the two
postcard corpora are minted again.

**Q1003's: write the same ball on the world chart.** A whole ball — one wire,
a seam with two corners, a pole, the seam back, a pole — covers the whole
sphere wherever its seam runs. So the file need not say where the turn put the
poles: it says the same ball cut open where `add_ball` cuts it, poles at
`centre ± (0, 0, r)` and the seam in the plane through the centre square to
`y`, on the side facing `+x`. The reader reads that as the ball it is today.

**The second is taken.** It touches only `write.rs` and `brep.rs`, leaves
`FORMAT_VERSION` and both corpora as they are, and what it writes is a file
this kernel already reads: the file of the same ball moved to that centre and
not turned, byte for byte. Q1004 would also have carried a turned *part* of a
ball with a pole on it, which this does not (404.6). The probe's `part` mode
looked for one the fuse makes: a ball less a cube and a ball fused with one,
each cut at the equator and then turned half a radian about `x`, have ball
faces of two steps along and no pole, and both are written. That is two
shapes, not every fuse; the only part ball with a pole anyone has built is
the reader's (404.6).

### 404.3 The writer

`a_whole_ball_turned_off_its_chart` picks the face: a ball, one wire of the
four steps above on one edge, whose poles pinch where the face is stored and
not where it is placed. A ball turned about `z` keeps its poles on the line and
is written as before, seam and all. `Writing::a_whole_ball_on_its_chart` writes
the loop: one `EDGE_CURVE` between the two poles, walked there and back, on the
half circle `Writing::a_seam_of_one_ball` already writes for an upright ball.

**Where `z + r` or `z − r` rounds to `z` it is still refused
`AWireThatStandsStillAtAPole`.** The world chart has no two poles there to
write. This is a limit of that chart, and a ball of radius one at
`z = 1e20` is the test
(`a_turned_ball_too_small_for_its_height_is_refused_at_its_pole`).
`the_poles_of_a_ball_on_its_chart` in `brep.rs` is the one place both sides
work the two poles out.

**2026-10-07:** `a_whole_ball_turned_off_its_chart` is now
`a_whole_ball_written_on_its_chart`. Since Q1308 it also catches the second
kind named there: a ball turned about `z` keeps its poles on the line, but for
a turn in the two bands `a_ball_turned_about_z_is_written.rs` names, no centre
the reader holds carries the seam's own plane, and the writer rewrites it onto
the world chart the same as a ball whose poles left the line — a ball turned
about `z` outside those bands is still written seam and all. The writer's
fuzz harness (`tests/fuzzing/writer.rs`) only told the first kind apart from
an ordinary ball, by the poles, and so compared the second kind's rewritten
seam corner by corner and panicked on it as a false crash; it now accepts
either kind the same way the writer picks them, by comparing the seam that
went in against the one that came back, through
`assert_cut_again_on_the_z_line` (Q1319).

### 404.4 The reader, and the two choices the row left open

A file from elsewhere cuts a turned ball open along a slit between two corners
off the `z` line. Walked as written it is a wire of two steps, which
`SolidBuilder` refuses as too short. `Walk::a_whole_ball_off_its_chart` reads
such a face as the whole ball, on the world chart, when its one bound is one
`EDGE_CURVE` walked there and back, the curve a `CIRCLE`, and its two corners
not both on the `z` line. A bound whose corners both are is the kernel's own
seam, walked as before; the writer's files are always that, so this branch
never fires on them. Across the 313 `rcad-step` tests, corpus included, it
fires twice, both in the new slit tests.

**The slit and its corners are checked to the digits the file was written in,
not bit for bit.** They are dropped: the solid gets the world chart's poles
and seam, and nothing the file wrote there reaches the model. So the check only
tells a slit on the ball from a file that is wrong, the same job as the
twelve-digit allowance the reader gives an `ELLIPSE`'s sizes (§335). A ball
turned by an angle puts its corners at irrational coordinates, which a real
file writes in twelve or thirteen digits; those miss the ball by thousands of
roundings, so the reader's sixteen roundings
(`A_CIRCLE_MAY_MISS_ITS_FACE_BY`) alone would refuse the very file this
branch is for. The allowance is sixteen roundings plus
`A_SLIT_WRITTEN_IN_TWELVE_DIGITS_MAY_MISS_ITS_BALL_BY`, six numbers' worth of
half a unit in the twelfth digit; its doc comment adds the six up.
`a_slit_written_in_twelve_digits_is_read_as_the_whole_ball` fails with the
allowance at zero.

The probe checked the circle's radius bit for bit and nothing else, so a circle
of the right radius anywhere passed. The circle is now held to the ball by
`how_far_off_the_circle_can_be`, centre and radius together, as every circle on
a curved face is.

**The dropped corners are checked against the ball.** A file whose corners
are off its own ball is wrong, and reading it as a clean ball would hide that.
Kept, they would have met `Model::validate`. A slit whose circle or one of whose
corners is a thousandth off is walked as written and refused
(`a_slit_a_thousandth_off_its_ball_is_refused`). Where `z ± r` rounds to `z`
the reader does not read on the world chart either
(`a_slit_ball_too_small_for_its_height_is_refused`).

### 404.5 What it measured

On this branch, with Q1003's probe crate rebuilt against it
(`target/q1010/probe` in the lane's tree):

- 2000 of 2000 `add_ball`s turned about a random line by a random angle come
  back with one face, one edge and two corners, sound. All 2000 are written
  byte for byte as the same ball moved to its centre and not turned.
- A ball turned half a radian about `x`, fused with a cube, writes and reads
  back three solids, 20 faces, 37 edges and 23 corners, the union measuring
  `1234.290780923788` both ways.
- The writer target's 400 draws: 5 refused, down from 43, and no
  `AWireThatStandsStillAtAPole` (366.4's dated note).
- A turned ball's volume differs from the same ball at its centre by up to
  `1.79e-4` relative, as a plain moved ball's does: the mesh follows the
  stored chart. That is why `tests/fuzzing/writer.rs` measures no placed curved
  solid bit for bit, and it is not this change's.

### 404.6 What this leaves

- **A face that is only part of a ball, with a pole a turn moved off `z`, is
  still refused `AWireThatStandsStillAtAPole`.** Only the reader builds a part
  of a ball with a pole: `with_a_pole_where_it_turns_back` puts one where a
  file's loop turns back exactly on the `z` line. Such a face turned by a
  placement is the shape left. Nobody has built one and seen it refused, so it
  has no row.
- Q1004's change to `Sphere` is not needed for any shape this kernel builds.
  It would be, for that part ball, if one is ever seen.
