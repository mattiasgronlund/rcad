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

    mise exec -- just t -p rcad-step \
        --test every_model_the_writer_fuzzer_kept_is_written_again how_often --no-capture

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

### 366.6 A frustum and a block read back from rounded text

`a_frustum_up_through_a_block_read_back_from_rounded_text` now writes all 48
configurations and prints, run by

    mise exec -- just t -p rcad-step --test a_fuse_read_back_from_rounded_text \
        a_frustum_up_through_a_block --no-capture

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
