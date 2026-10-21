## 366. A cone's slope written as an angle — 2026-10-03

*Queue row Q662 (was Q406). Lane `lane-q662-a-cone-cap-written-in-its-stored-frame`,
off `b1136e72`. The writer refused `ACurveTheFacesEitherSideOfItDoNotName`
wherever a flat cap meets a cone. It now writes that circle, worked out in the
frame the two faces are stored in. Q70 found that a writer-only fix turns those
refusals into comparison panics on the cone's slope, so the half-angle is
searched and the writer target compares the slope as a measurement. Both land
together. Everything below was run, and each number names its test.*

*Landed by Q821, lane `lane-q821-a-cone-cap-written-in-its-stored-frame`, off
`c564dec5`, which merged Q662's branch and moved the pins and notes outside
its write set (366.5). 366.2's last paragraph and 366.6 are Q821's.*

  366.1   The cap circle, in the stored frame
  366.2   The half-angle written, and the slopes no half-angle carries
  366.3   The slope compared as a measurement
  366.4   What it changed in the writer target
  366.5   What this leaves
  366.6   A frustum and a block read back from rounded text
  366.7   A cone's slope put back from its corners

### 366.1 The cap circle, in the stored frame

`where_a_cap_square_to_a_cone_crosses_it` in `crates/rcad-step/src/write.rs`
takes the two faces either side of a closed edge in the frame they are stored
in (`Model::face(key).surface()`) and the location that places them
(`FaceView::location`). Where the cap is square to the axis there, with
`n × a` exactly zero, it works out the circle: centred where the axis meets
the cap, with the cone's radius that far from the apex. It then places the
centre and the axis by that location. A cap that is not square is refused, as
before. That crossing is an ellipse off the axis, and no builder or file this
writer has met makes one (§13).

**The stored frame is the point of it.** A builder makes the cap square along
`z`. A turn rounds the cap's normal and the cone's axis apart by an ulp, so
asked of the placed surfaces, the same cap reads as not square.

**The circle turns about the cone's axis, as a pipe's cap circle turns about
the pipe's.** That is the way `rcad-topo` stores a closed edge round a cone:
`Surface::the_way_its_first_coordinate_runs_at` on a cone is `a × (p − apex)`
on either nappe. The reader reads each use of the edge against that
(`Walk::which_way_round_a_closed_edge_is_stored`). So the cone does **not**
need `rcad_geom::which_way_a_closed_crossing_turns`, which Q70 found the torus
cap needs: a ring's coordinate runs round its tube, not round the circle a cap
cuts from it. The stored frame is kept per side in the writer's `Side` struct,
in `write.rs`, so Q407 can reach it for the ring.

**2026-10-04:** Since Q407 the plane-torus cap circle is written too, and it
needed no `which_way_a_closed_crossing_turns` either: it turns about the cap's
normal, or its reverse, by whichever sign picks the corner's side of the axis.
`how_often_a_model_the_mutator_draws_is_refused`'s 85 of 400 draws refused at
the plane–ring circle are now 0 (§241.6).

### 366.2 The half-angle written, and the slopes no half-angle carries

STEP writes a cone's half-angle, and the reader takes its tangent
(`cone_from`). `a_half_angle_that_reads_back_as` searches 64 `f64` steps either
side of `atan(slope)`. It writes the nearest angle whose tangent is the slope
bit for bit, or else the one whose tangent comes nearest.

`each_slope_the_writer_target_draws_reads_back_from_the_angle_written`: `0.25`
and `0.5` come back bit for bit. `atan(0.5)` alone reads back as
`0.49999999999999994`, and the angle one step above it reads back as `0.5`.
**One and two come back from no angle.** The `f64` nearest a quarter turn reads
back as the `f64` below one, and the next angle up reads back as the `f64` above
it. Two falls strictly between the tangents of `atan(2)` and its neighbour the
same way.

**This is a limit, and the demonstration is in
`most_slopes_over_one_come_back_from_no_half_angle`.** Over 100 000 slopes
drawn evenly from `1e-3` to `1e3`, it prints:

    100000 slopes from 1e-3 to 1e3: 539 come back from atan, 0 from an angle
    the search finds (furthest 0 steps), 99461 from none (99445 of the 99920
    over one)

Q70 found the same 539. For every slope that comes back from none, the test
checks two things across the 129 angles searched. Their tangents strictly
increase, and the slope lies strictly between the first and the last. So no
angle in that window reads back as the slope. Outside the window, that also
holds as long as `tan` is faithful (off by less than an ulp of the true
tangent): every angle below the window has a smaller tangent, and every angle
above it a larger one. Q70's count says the same of slopes over one as a
whole. There are about `4.5·10¹⁵` `f64` angles in `[π/4, π/2)` and about
`4.5·10¹⁶` slopes in `[1, 10³]`. Each angle reads back as one slope, so at most
one slope in ten can come back exactly.

**The family considered** is which `f64` half-angle to write. It is the only
number this reader turns into a slope: the writer puts the placement at the
apex and writes radius zero, so `cone_from` divides nothing. Angles outside
`(0, π/2)` are not considered. They are not a cone's half-angle:
`Cone::from_half_angle` refuses them, and other readers take a `semi_angle` as
written. `cone_from` taking the tangent of any angle is the reader's leniency.
It is not a channel. **A reader that recovers the slope some other way**, from
the corners for instance, was not examined. That is a change to the reader,
and this lane's write set did not hold it.

**2026-10-05:** The reader now does, where the file cut the half-angle and
the corners agree (366.7). A half-angle written to seventeen digits is never
read that way, and `atan(0.5)` written in its sixteen is not either
(`a_half_angle_written_to_every_digit_an_f64_has_keeps_its_tangent`). One
this writer writes in fewer digits could be, but only where the corners agree
to the bit on a slope whose angle rounds to the same text. The round trips
above are run again by the gate.

**`most_slopes_over_one_come_back_from_no_half_angle` shows the limit and
cannot pin the search.** Not one of its 100 000 slopes comes back from an angle
the search finds and `atan` misses ("0 from an angle the search finds"), so
every count it prints is the same with the search taken out. Q662's verifier
found that, and Q821 ran both ways of taking it out:

- **The writer's call site written back to `real(cone.slope().atan())`**: the
  one test that fails is
  `a_cone_of_a_slope_some_half_angle_carries_comes_back_with_that_slope_to_the_bit`,
  in `crates/rcad-step/tests/fuzzing/writer.rs`, at slope ½, `left:
  [4602678819172646911]` (`0.49999999999999994`) against `right:
  [4602678819172646912]` (`0.5`). It writes the frustum and the pointed cone at
  slopes ¼ and ½, reads each back, and asks for the slope bit for bit. Every
  other cone test passes, because the target compares the slope as a
  measurement (366.3) and lets that ulp through.
- **The search's body returning `slope.atan()`**: that test fails the same way,
  and so does `each_slope_the_writer_target_draws_reads_back_from_the_angle_written`,
  which asks the function itself.

### 366.3 The slope compared as a measurement

`assert_located_numbers_the_same`, in
`crates/rcad-step/tests/fuzzing/writer.rs`, still compares a cone's apex and
axis bit for bit. It compares the slope as §249.6 compares a measurement
(`a_slope_comes_back_as_near_as_a_half_angle_carries_it`). The slope must come
back within what the tangents of the two angles either side of `atan(slope)`
differ by. Over the 100 000 slopes above, the slope written and read back is at
worst a third of that reach off. The test asserts it is never more than the
whole.

The function now answers whether every number came back the same bits. Only
then does `read_back_and_compared` compare volume, area and centroid bit for
bit. An unplaced frustum of slope one comes back with a slope an ulp off, and
measuring it against the one that went in would be measuring two cones.
`every_cone_this_target_draws_comes_back_from_its_own_file` writes and reads
back the frustum and the pointed cone at all four slopes, where they are built
and turned a quarter about `x`. All sixteen come back the same.

### 366.4 What it changed in the writer target

`how_often_a_model_the_mutator_draws_is_refused` runs Q70's loop 400 times:
`still_a_model` from the checked-in seeds, with each draw's output added back
to the pool. Run by

    mise exec -- just t -p rcad-step -E \
        'binary(merged) & test(every_model_the_writer_fuzzer_kept_is_written_again) & test(how_often)' --no-capture

Before the writer change, at the lane's first commit:

    278 of 400 drawn models refused
       173  ACurveTheFacesEitherSideOfItDoNotName { first: "plane", second: "cone", closed: true }
    170 sound models refused where a flat face meets a cone

After it:

    128 of 400 drawn models refused
         1  ACurveTheFacesEitherSideOfItDoNotName { first: "plane", second: "cone", closed: true }
        85  ACurveTheFacesEitherSideOfItDoNotName { first: "plane", second: "ring", closed: true }
        38  AWireThatStandsStillAtAPole
    0 sound models refused where a flat face meets a cone

The cone cap was 173 of 400 draws on this base, near Q70's 40%. The one cone
cap still refused is on a model that is not sound. The test asserts that no
sound one is refused.

The same loop counts comparisons that panic. They are caught and counted, not
failed:

- **22, "a different surface"**, up from 3 before. The first of them is a turned
  cone whose axis direction comes back an ulp moved. `cone_from` builds its
  axis through `Axis::new`, which scales the written direction to length one
  again. `Axis::transformed`'s doc counts 24 750 of 100 000 directions that move
  when an axis is rebuilt from its own. Q497 is that gap for a pipe, and its
  write set holds `geometry.rs`, where `cone_from` is. Turned cones are now
  written, so they reach it too.
- **7, "a different edge run"**, 5 before. The first is a ball turned about `z`
  whose pole-to-pole seam is named by a plane whose normal comes back an ulp
  moved (`0.1660299346975594` read back as `0.16602993469755936`). Where the
  ulp goes was not traced. 366.5 files it.

*2026-10-03, Q822, lane `lane-q822-a-ball-seam-plane-read-back-an-ulp-off-2`
off `2ad40445`.* On that base the same command prints the same seven, and
`ASolidWithNoShell` four times, which the print above leaves out. All seven
are a ball of the mutator's turned about `z` by an angle `draw.fraction()`
gave. None came back reversed, so none is Q776's guess. The writer turns the
circle over only where `(B − A) × n · p` is negative, and for `add_ball`'s seam
that is `2r`: a turn keeps a triple product, so no ball the mutator builds
reaches Q776. Three causes, each seen alone:

- **The reader scaled the seam plane's normal again.** It built the plane off
  the circle's placement through `Plane::from_point_and_normal`, not through
  `plane_through`, which §355 gave a `PLANE`. All four places in `brep.rs`
  that build a circle's plane now call `plane_through`.
- **The ball's centre did not read back as the plane's offset.** The writer
  placed the circle at the ball's centre, and the reader's dot of that centre
  with the normal can land an ulp off the offset the turn left. The seam is now
  centred by `a_centre_its_plane_reads_back_from`, as Q767's arcs already were,
  and that function asks `plane_through` too. On two of the seven no one
  coordinate stepped alone lands on the offset — for the ball at
  `(10, 7, −15)` the offset is `0.80`, and one step of either coordinate moves
  the dot six ulps of it or more — so those two were refused by name until the
  function stepped two coordinates together, at most four each. It finds a
  centre for both, so that refusal was a gap.
- **The target compared the seam's picker by exact parallelism.** The kernel's
  picker is `(1, 0, 0)` turned; the reader's is `(B − A) × w`, which for the
  ball of radius six is `12·(n_y, −n_x, 0)` rounded per component, and the two
  are not along each other even in exact arithmetic. One of the seven came
  back with its plane the same bits and panicked on the picker alone.
  `the_same_run` now compares a picker beside a plane by which side of the
  chord it faces within that plane, which is which half it picks.

After, the same command:

    128 of 400 drawn models refused
         1  ACurveTheFacesEitherSideOfItDoNotName { first: "plane", second: "cone", closed: true }
        85  ACurveTheFacesEitherSideOfItDoNotName { first: "plane", second: "ring", closed: true }
         4  ASolidWithNoShell
        38  AWireThatStandsStillAtAPole
        22  written, and the comparison panicked: a sound solid's face came back from its own file with a different surface
    0 sound models refused where a flat face meets a cone

The test now fails on any comparison that panics on an edge's run and on any
ball whose seam is refused. The draws are pinned by
`a_ball_turned_about_its_poles_keeps_its_seam_plane_through_its_own_file`,
`a_ball_turned_about_its_poles_comes_back_from_its_own_file`,
`a_turned_ball_whose_seam_plane_passes_near_the_origin_comes_back_from_its_own_file`
and `a_seam_picked_from_the_other_side_is_a_different_run`.

*2026-10-05, Q1010, lane `lane-q1010-a-turned-ball-written-on-the-world-chart`
off `39218a67`.* The 38 `AWireThatStandsStillAtAPole` are gone: a whole ball
turned off `z` is written cut open on the `z` line, and the target compares
it by its surface, its poles and its seam (§404). The same command prints

    5 of 400 drawn models refused
         1  ACurveTheFacesEitherSideOfItDoNotName { first: "plane", second: "cone", closed: true }
         4  ASolidWithNoShell
    0 sound models refused where a flat face meets a cone or a ring

and no comparison that panicked. The test now also fails on any ball refused
at its pole.

### 366.5 What this leaves

- `every_shape_seed_is_the_shape_it_is_named_for`, in
  `crates/rcad-step/tests/every_model_the_writer_fuzzer_kept_is_written_again.rs`,
  pinned the frustum and pointed-cone seeds as refused. They now come back the
  same, so `comes_back` is `true` for both, and that file's module doc says only
  the quarter bend is refused. Done by Q821.
- `a_frustum_up_through_a_block_never_reaches_the_reader_because_the_writer_refuses_it`,
  in `crates/rcad-step/tests/a_fuse_read_back_from_rounded_text.rs`, pinned all
  48 frustum-and-block cases as refused, and failed as its own doc said it
  would. Q821 made it a row like the ball's,
  `a_frustum_up_through_a_block_read_back_from_rounded_text`, asserting nothing
  is left unwritten. 366.6 has what it prints.
- `crates/rcad-step/tests/no_cone_face_arrives_out_of_the_corpus.rs` said the
  writer refuses a pointed cone. Q821 built the control it wanted: its pointed
  cone is written, read back through `Import`, and must arrive with its one cone
  face. §241.6 has a dated note.
- The ball seam in 366.4 is a defect, filed as Q822.
- Q497 now covers turned cones as well as turned pipes, through `cone_from`.
- Q407, the torus cap, is still refused: 85 of the 400 draws.
- §191.3 still says the frustum is refused by the writer. Its file was in
  neither lane's write set; a row adds the dated note.

**2026-10-04:** Since Q407 the torus cap is written, and
`how_often_a_model_the_mutator_draws_is_refused` now refuses 0 of 400 draws at
the plane–ring circle, not 85 (§241.6).

### 366.6 A frustum and a block read back from rounded text

`a_frustum_up_through_a_block_read_back_from_rounded_text` now writes all 48
configurations and prints, run by

    mise exec -- just t -p rcad-step -E \
        'binary(merged) & test(a_frustum_up_through_a_block)' --no-capture

    as built:            144 decided, refused {}
    read back exactly:   144 decided, refused {}
    11 digits (48 of 48 files had a real moved): 0 decided, refused {}; ...
        reader refused {"ACornerIsNotOnTheCurvedFaceTheFileSaysItIsOn": 48}
    12 digits (48 of 48 files had a real moved): 0 decided, refused {}; ...
        reader refused {"ACornerIsNotOnTheCurvedFaceTheFileSaysItIsOn": 48}
    13 digits (48 of 48 files had a real moved): 21 decided,
        refused {"Undecidable": 3}; 3 refusals the rounding introduced ...
        reader refused {"ACornerIsNotOnTheCurvedFaceTheFileSaysItIsOn": 40}

It is the first family in that file whose curved faces carry a real that
rounding moves: the bar through a block and the ball and block are built in
whole numbers. **The reader refuses the frustum file at every digit count an
exporter writes**, all 48 at 11 and 12 digits and 40 of 48 at 13, where the
turned pair, whose faces are all flat, is read back at all three. Whether that
is §42.9's refusal of a corner provably off its face, which the house keeps
until healing has a consumer (Q25), or a gap in the room a corner on a cone is
given, was not traced. Neither were the three fuses the 13-digit rounding
turned from decided to `Undecidable`. Each has a row.

**2026-10-05:** Q827 traced it, and it was neither: the corners are exact and
the slope is what moved. 366.7 has the fix, after which every file at all
three digit counts is read back and all 144 fuses of each are decided.

**2026-10-05, Q828.** The three 13-digit `Undecidable` fuses were traced to
`FrustumAndBlock { side: 11, narrow: 1, wide: 3, up_per_unit_wider: 2, at: [7,
4] }`: the frustum's apex sits at `(7, 4, −3)`, slope one half, so the cone it
is cut from, swept to `z = 11` — the block's own roof — has radius exactly
seven, tangent to both `x = 0` and `y = 11`. Reading the file rounded to
thirteen digits and then written out again at seventeen — what an importer
that re-exports a file it read does — defeats 366.7's corner recovery: the
angle now carries seventeen digits of its own, so
`its_angle_is_written_as`'s half-unit bracket no longer holds the true angle,
and the reader takes the tangent of the thirteen-digit angle instead, a slope
`0.49999999999999234`, `7.66e-15` short of one half. At `z = 11` that is a
radius `1.07e-13` short of seven, and the sliver where the swept circle falls
short of the two walls is a C-shaped piece with two necks that thin. Since
Q897 (`a7e263e9`) the rungs of `rungs_towards_each_crossing` reach into a
piece this thin, and all three rules answer within
`how_far_rounding_can_move_a_volume` of the exact reading; before, at
`b00d9291`, the same piece was `Undecidable` for every rule. Pinned in
`a_frustum_tangent_to_two_walls_at_its_blocks_roof_is_read_back_rounded_and_reexported`,
in `crates/rcad-step/tests/a_fuse_read_back_from_rounded_text.rs`.

### 366.7 A cone's slope put back from its corners

*2026-10-05, Q955, lane `lane-q955-a-cone-slope-read-back-from-its-corners`,
off `3b1ffe9f`. Everything below was run, and each number names its test.*

**The gap.** Q827 traced `FrustumAndBlock { side: 8, narrow: 2, wide: 3,
up_per_unit_wider: 2, at: [4, 4] }` at twelve digits. The cut moves one real,
the half-angle, to `0.463647609001`, which `tan` reads back as the slope
`0.5000000000002424`. The corner `(7, 4, 1)`, written exactly, is then
`1.30e-12` off the cone, 52.3 times the `16·ε·L` a corner of a curved face is
given, and exactly on the cone of slope one half, whose angle `0.463647609001`
is to twelve digits. In all 48 files only the half-angle moved. The answer
exists and the arithmetic missed it, so §16.10 calls it a gap.

**What the reader does.** `cone_from`, in `crates/rcad-step/src/geometry.rs`,
asks `the_slope_of`, which takes the tangent of the angle written, as before,
unless all three of these hold. Then it takes the slope the corners agree on.

1. **The file wrote the angle in radians, as a real**, and it lies strictly
   between nought and a right angle. The tokenizer keeps the place of the last
   digit each real was written to (`LastDigit`, in `token.rs`): `-12` for
   `0.463647609001` and for `4.63647609001E-01`, and `-13` for
   `0.4636476090010`, whose last zero is a digit the file wrote. The `f64`
   cannot keep that, and its shortest spelling drops the zero. `Record` keeps
   it for each attribute that is a real or wraps one once (`last_digits`, in
   `graph.rs`). `Value::Real` is unchanged. A file in degrees takes the
   tangent of a product, and its twelve digits are not that number's digits.
   `a_pointed_cone_whose_half_angle_was_cut_in_degrees_keeps_the_tangent`
   pins it, and fails when the radian test is taken out.
2. **Every corner off the placement's own plane puts the cone at the same
   `f64` slope** (`a_slope_every_corner_agrees_on`). A corner `ρ` from the
   axis and `h` along it says `(ρ − R) / h`. The corners are the face's as
   the file wrote them: both ends of every edge of its `EDGE_LOOP`s, and the
   corner of a `VERTEX_LOOP` (`corners_as_written`, in `brep.rs`). They are
   read before the face is described, and so they are not counted in
   `integers_read_as_reals` (`point_read_ahead_at`). The loop's corner counts
   even though the walk drops it at the apex, because on a cone placed on its
   cap the apex is the one corner off the placement's plane.
   - **A face with no corner off that plane** has nothing to agree on, and
     takes the tangent. A cone placed at its apex finds its apex on the
     plane, and a cone placed on its cap finds the cap's corner there.
     Pinned by
     `a_cone_face_with_no_corner_off_its_placements_plane_has_no_slope_from_its_corners`.
   - **Corners that say two slopes**, or a slope of nought or below, take the
     tangent too. Agreeing means the same bits. That is what short corners
     along an axis written as a unit direction give. Corners that work out an
     ulp apart are not given one of their two answers. Pinned by
     `corners_that_say_two_slopes_or_none_have_no_slope_from_their_corners`
     and `the_slope_comes_from_the_corners_only_where_the_angle_was_cut_and_they_agree`.
3. **The angle written is that slope's angle to within half a unit in its
   last digit, in exact arithmetic** (`its_angle_is_written_as`). It never asks
   `atan`, which is the platform's and is bounded by nothing Rust publishes.
   `atan_between` brackets the angle with `+`, `−`, `×`, `/` and `sqrt` alone.
   Rust's docs say each of these gives the exact answer rounded, and each
   result is moved one `f64` outwards. A slope above one turns round on
   `π/2 − atan(1/s)`. `FRAC_PI_2` is below `π/2` and the next `f64` above it,
   which
   `frac_pi_2_and_the_next_f64_up_stand_either_side_of_a_right_angle`
   shows by comparing their exact decimals with `π/2`'s. Two halvings,
   `atan(x) = 2·atan(x / (1 + √(1 + x²)))`, bring the argument below `0.2`.
   There the series alternates and shrinks, so it stops below the angle after
   a term taken away and above it after a term added. The bracket holds each
   angle `bc -l` worked out to 45 digits
   (`atan_between_brackets_angles_worked_out_to_forty_five_digits`). It is at
   most 49 ulps wide over 100 001 slopes from `1e-3` to `1e3`
   (`atan_between_holds_the_platforms_atan_over_slopes_from_a_thousandth_to_a_thousand`).
   The text `D` is not an `f64` either. The `f64` it reads as is within half an
   ulp of it, and a whole ulp is taken off the half unit for that. The half
   unit is parsed, which rounds correctly, and taken one `f64` lower. So a yes
   means `D − half ≤ lo ≤ atan(slope) ≤ hi ≤ D + half`, and a no takes the
   tangent.

**A yes is never wrong, on any input**, by the chain above. A no can be wrong
only where the angle lies within the bracket's width and an ulp of the edge of
the half unit, and then the tangent is taken, as before. On the 48 files it is
nowhere near the edge: at twelve digits `|0.463647609001 − atan(0.5)|` is
`1.94e-13` (`bc -l`) against a half unit of `5e-13`, and the bracket at slope
one half is at most 64 ulps, `3.6e-15`, wide. **On an angle written whole the
answer is no**: this writer writes `atan(0.5)` as `0.4636476090008061`, whose
half unit, `5e-17`, is narrower than an ulp. The tangent is taken, and 366.2's
bit-for-bit round trips are untouched
(`a_half_angle_written_to_every_digit_an_f64_has_keeps_its_tangent`).

**A short angle is read at its own digits.** `0.46` with corners that agree
on one half is taken as one half, since `atan(0.5)` is `0.46` to two digits.
Reading a short text as exact would refuse what both its corners and its
text, at the digits it has, say. No floor on the digit count was set, and no
measurement asks for one. **No room was widened.** `Model::validate` checks
the corners against the cone this builds exactly as against any other. §174.4's
cut corners stay a limit under Q25.

**What it moved.**

    mise exec -- just t -p rcad-step -E \
        'binary(merged) & test(a_frustum_up_through_a_block)' --no-capture

    11 digits (48 of 48 files had a real moved): 144 decided, refused {}; ...
        reader refused {}; worst volume move 0.00e0 of the bound
    12 digits (48 of 48 files had a real moved): 144 decided, refused {}; ...
        reader refused {}; worst volume move 0.00e0 of the bound
    13 digits (48 of 48 files had a real moved): 144 decided, refused {}; ...
        reader refused {}; worst volume move 0.00e0 of the bound

Before, the reader refused 48, 48 and 40, and 3 of the 24 fuses of the eight
read at 13 digits were `Undecidable`. The test now asserts that nothing is
refused. `a_pointed_cone_whose_half_angle_was_cut_to_twelve_digits_has_its_slope_from_its_apex`
is the same gap on one hand-written cone, placed on its cap. Its tangent put
the apex `3.9e-12` above the corner the file wrote there.

`how_often_a_real_file_imports` counts the same before and after: kicad
`165 / 323 imported`, 95 refused for a corner off a curved face, nist
`1 / 44 imported`. Two refused files moved:

- `Fuseholder_TR5_Littelfuse_No560_No460.step`: its two 45° chamfer cones,
  `CONICAL_SURFACE('',#451,1.,0.785398163397)`, have corners that agree on
  slope one exactly, and the worst corner off a curved face is now 97.442
  spacings, not 362.007.
- `nist_stc_06_asme1_ap242-e3.stp`: its two drill points, half-angle
  `1.0297442587` to ten digits, carry a `VERTEX_LOOP` at `z = 70.4822675346`.
  The tangent put the apex at `70.48226753468838`, `8.8e-11` off, and the loop
  was refused, `ALoopThisReaderDoesNotWalk`. The apex corner says slope
  `1.6642794823614244`, whose angle is the text to ten digits. Now the loop
  stands on the apex, the solid reaches the builder (nist: 12 files, not 11),
  and the file is refused for a hole in a face that winds the same way as the
  outer wire. That reason was not traced. §376.5 called these apex corners a
  modelling tolerance. Q751 lists `nist_stc_06` among its four files, and for
  this file the miss was the angle's own cut.

**What this leaves.**

- A file that writes its angles in degrees and cuts them still takes the
  tangent of the product and is refused the same way. Shown only on the
  hand-written cone above, so a row of class C.
- `nist_stc_06_asme1_ap242-e3.stp`'s new first refusal, a hole winding the
  way its outer wire does, is untraced. It has a row.
- Q751's text still names `nist_stc_06` among the files its apex loops refuse.
  It no longer does.
