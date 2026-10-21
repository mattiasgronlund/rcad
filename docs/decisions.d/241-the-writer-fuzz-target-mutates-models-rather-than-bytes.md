## 241. The writer fuzz target mutates models rather than bytes — 2026-09-25

*Queue row Q58, from §129.5: 19 of the writer target's 1183 seeds decoded into
a `Model` and 16 of those were empty, so a mutation almost never reached the
writer. The target now has a mutator that keeps every input a model, and the
corpus has seven shape seeds. Running it found three things wrong on its first
minute, listed in 241.5.*

  241.1   What the mutator does
  241.2   Why a mutator over the old input, and not a new input format
  241.3   One minute of the old target against one minute of the new
  241.4   What the seven seeds cover
  241.5   What the first minutes found
  241.6   What is left

### 241.1 What the mutator does

`still_a_model` in `fuzz/fuzz_targets/no_model_makes_the_writer_panic.rs`
replaces libFuzzer's mutator through `fuzz_mutator!`. It decodes the input with
`postcard`, the same door the target and a stranger's saved file use, and makes
one change a model can take:

- three times in eight, libFuzzer's own byte mutation, kept only if the bytes
  still decode (four tries, then one more solid instead). This is the half that
  reaches the writer with models `Model::validate` would refuse — a corner
  nudged off its face — which no builder makes;
- two in eight, one more solid;
- two in eight, one solid moved or turned — only in a model that validates,
  because `place_solid` panics on a solid that is already broken;
- one in eight, a fresh model.

An input that does not decode becomes a fresh model. The solids are the
kernel's cube, round bar and ball, and three it has no builder for — a frustum,
a pointed cone and a quarter bend of pipe — described face by face through
`SolidBuilder` with every corner exactly on its curved surface. Moves are by
whole units, and half of them also turn, by a multiple of an eighth of a turn
or by any angle, about `x`, `y`, `z` or a slanted axis. The mutator's own draws
come from the seed libFuzzer hands it, so a mutation repeats with its seed — all
but the three in eight that are libFuzzer's own byte mutation. That seed is one
draw of libFuzzer's generator (`Mutate_Custom` in `FuzzerMutate.cpp`), and
`LLVMFuzzerMutate` goes on drawing from the same generator, which the whole run
has moved. Nothing leans on it: a crash is replayed from its saved bytes, and
that never runs the mutator. Found by the lane's verifier.

### 241.2 Why a mutator over the old input, and not a new input format

The other way to make every input a model is to change what the bytes mean: read
them as a recipe — which shape, what size, where — and build the model from it.
Every byte string would then be a model. That was not taken, for three reasons:

1. **The input stays the stranger's route in.** A recipe only ever builds what
   the builders build. Kept as `postcard` bytes of a `Model`, a byte mutation can
   still hand the writer a model no builder makes, which is the case the writer
   owes an `Err` for (§42.12).
2. **Everything already keyed to the format keeps working.** The 1183 seeds, the
   crash files of past nights, the control seeds, and
   `every_model_the_writer_fuzzer_kept_is_written_again.rs`, which replays the
   corpus in `just gate`, all read `postcard` bytes of a `Model`. A recipe format
   would orphan all of them at once.
3. **The fix is local.** It is one file, the target's own; the shared harness in
   `crates/rcad-step/tests/fuzzing/writer.rs` is untouched, so the property the
   fuzzer explores and the one the suite replays are still the same lines.

The cost is that the mutator decodes and re-encodes on every call, and
validates the model before a move. 241.3 measures what that costs.

### 241.3 One minute of the old target against one minute of the new

Old is `b94047f4^`'s target, new is `b94047f4`'s. Both were copied to the
scratchpad with one change: a counter around `one_model_decoded_and_written`
that tallies its `Reached`, and catches a panic and counts it rather than
letting it end the run — without that, all four runs of a first round died in
under three seconds (241.5), which measures nothing. Each was built with
`cargo +nightly fuzz build` under the `fuzz-target` recipe's `RUSTFLAGS`, then
run as
`<binary> <fresh copy of the 1183 seeds> -max_total_time=60 -print_final_stats=1`,
in the order old, new, new, old, because other sessions were loading the
machine (load average at each start: 8.9, 19.0, 26.4, 27.4, on 24 cores).

"Reached the writer" is a model that decoded — the harness hands every one to
`rcad_step::write` — counting the ones that went on to panic. Runs are
libFuzzer's `stat::number_of_executed_units`; the other columns are the
counter's last line, printed every quarter second, so they miss at most the
final quarter second.

| Run | Target | Runs | Reached the writer | With a face | Came back the same | Panicked |
|----:|--------|-----:|-------------------:|------------:|-------------------:|---------:|
| 1   | old    | 187 938 | 2 778  | 2 601  | 1 526 | 163 |
| 2   | new    | 12 398  | 11 164 | 11 140 | 1 090 | 161 |
| 3   | new    | 13 493  | 12 291 | 12 263 | 1 115 | 174 |
| 4   | old    | 145 300 | 1 350  | 1 213  | 37    | 781 |

- **The new target reaches the writer four to nine times as often a minute**,
  with a fifteenth of the runs: it does about 210 runs a second to the old one's
  2 400 to 3 100, and nine in ten of its runs reach the writer where one or two
  in a hundred of the old one's do. Of the inputs its mutator made — the runs
  after libFuzzer's 1184-run replay of the seeds, of which 19 decode — 11 145 of
  11 179 and 12 272 of 12 309 decoded.
- **Coverage at the end of the minute:** 9 685 and 9 566 edges for the new
  target, 9 292 and 9 301 for the old; features 38 360 and 38 296 against
  26 238 and 24 291. Both binaries carry the counter.
- **"Came back the same" is not four to nine times higher**, and the reason is
  the writer, not the mutator: of the models the new target decoded and did not
  panic on, 9 855 of 11 003 and 10 933 of 12 117 — nine in ten — were refused
  by name. 241.5 item 3 says which refusals. The old target's own count swings
  from 1 526 to 37 between its two minutes, so one minute of it is not a steady
  figure either. What the old target's 1 526 were made from was not examined.

### 241.4 What the seven seeds cover

Seven files in `fuzz/corpus/no_model_makes_the_writer_panic/`, each one solid
in a model of working size 100 mm, drawn by the target's own shape functions (a
scratch program that copied them and called them with these sizes):

| Seed | Shape | What the writer does |
|------|-------|----------------------|
| `a_round_bar_of_radius_three` | radius 3, 5 tall | written, comes back the same |
| `a_round_bar_moved_by_whole_units` | the same, moved by (3, −5, 7) | written, comes back the same |
| `a_round_bar_turned_a_quarter_about_x` | the same, turned a quarter about `x` | refused: `ACurveTheFacesEitherSideOfItDoNotName { pipe, pipe, closed: false }` |
| `a_ball_of_radius_three` | radius 3 | refused: `AWireThatStandsStillAtAPole` |
| `a_frustum_of_a_half_slope_cone` | slope ½, from 2 to 6 up the axis | refused: `… { plane, cone, closed: true }` |
| `a_pointed_cone` | slope ½, 6 tall | refused: `… { plane, cone, closed: true }` |
| `a_quarter_bend_of_pipe` | ring 6, tube 2 | refused: `… { plane, ring, closed: true }` |

Each decodes to a one-solid model with faces and validates.
`every_shape_seed_is_the_shape_it_is_named_for` holds each to its face kinds,
its middle to within 0.05, and whether it is written or refused.
`at_least_ten_seeds_decode_into_a_model_with_a_face` is the floor: it fails with
the seven taken out — "only 3 seeds decode into a model with a face",
`two_cubes_that_write`, `a_ball_whose_pole_the_writer_refuses` and the Q42 round
bar — and passes with them in. The replay now reports
`Reached { draws: 1190, decoded: 26, faces: 34, written: 4, refused: 22, …,
came_back_the_same: 4 }` (`mise exec -- just t -p rcad-step -E
'binary(merged) & test(every_model_the_writer_fuzzer_kept_is_written_again)'
--no-capture`).

**A turned seed that is written is not among them, because the only one there is
panics** — 241.5 item 2. The turned seed that is in is refused before the
comparison, so it covers the turn through the writer's refusal path only.

*Q730 changed the ball's row on 2026-09-30. The writer now leaves a ball's
poles unwritten and the reader puts them back where the seam turns, so
`a_ball_of_radius_three` and `a_ball_whose_pole_the_writer_refuses` are both
written and come back the same. The replay counts above are from before it.*

*Q738 renamed that seed to `a_ball_of_radius_two` on 2026-10-03, for the same
reason the paragraph above gives: since Q730 it is written and comes back the
same, so a name that said the writer refuses it was no longer true. §95.3 is a
record of its day and keeps the seed's old name.*

### 241.5 What the first minutes found

All three are reproduced outside libFuzzer by the same scratch program, calling
`one_model_decoded_and_written` on the saved input. None is fixed here; each is
the writer's or the reader's, not the target's.

1. **A sound model whose own file the reader refuses.** The Q42 round bar seed
   (`1bf987c1…`, radius 2, 8 tall, working size 8 mm) with **one byte changed**
   — byte 507 of 549 counting from one, 0 to 1, by `cmp -l` — still decodes and still validates, and the file
   the writer makes of it is refused on the way back: *"a sound model was written
   as a file the reader refuses: #20, in the solid #53, is not one this kernel
   can build"*. **The old target finds this, in under three seconds, twice out
   of two** — at run 131 034 and run 119 500 of the first round — so it is not
   the new mutator's; it is what §129.5's read-back leg finds as soon as it is
   pointed at a curved seed. Inputs: first-round crash files `crash-b64fd34b…`
   and `crash-5d117bfc…`.
2. **A sound solid that comes back from its own file a few ulps off.** A round
   bar of radius 3 and 5 tall, **turned an eighth of a turn about `z`**, is
   written and read back, and the comparison panics: the volume matches but the
   area is `150.76324910914616` going in and `150.7632491091462` coming back, and
   the centroid differs at the 1e-15 level. The same bar, turned and also
   moved by (3, −5, 7), fails the same way; moved by (3, −5, 7) with no turn it
   passes (§249.4). A move by whole units passes for that bar, but
   not for every solid: the new target's first-round crash is a cube and a round
   bar of radius 6, 4 tall, placed so the bar's middle is at the whole-number
   point (9, −7, −13), whose volume is `452.10882097631367` going in and
   `452.1088209763134` coming back. Whether that placement includes a turn was
   not read out of the bytes. The comparison asks for the same bits
   (§129.5 item 3), and it was not weakened here. Whether the writer's numbers,
   the reader's, or `measure` in a moved frame owns the difference was not
   examined.
3. **Nine in ten models the new target makes are refused by name.**
   `ACurveTheFacesEitherSideOfItDoNotName` refuses the circle where a flat cap
   meets a cone or a torus, and the seam of a round bar turned so its axis no
   longer points up `+z` — a quarter about `x`, a half about `y` and one radian
   about a slanted axis were all refused, and an eighth about `z` was written;
   `AWireThatStandsStillAtAPole` refuses every ball. A model with one such solid
   in it is refused whole. So the frustum, the cone, the bend, the ball and those
   turned bars reach the writer and stop at its door, and none of them reaches
   the read-back comparison. These are named refusals, not crashes,
   but they are what caps 241.3's "came back the same".

*Since Q730 (2026-09-30) a ball upright on `z` is written and comes back the
same. A ball turned so its poles leave the `z` line through its centre was
still refused `AWireThatStandsStillAtAPole` until Q1010 (2026-10-05), which
writes it as the same ball cut open on that line and reads it back as that
(§404, `a_ball_turned_off_z_comes_back_as_the_same_ball_with_its_poles_on_z`).*

### 241.6 What is left

- 241.5 items 1 and 2 are defects, each with a saved input, and want a row each.
- 241.5 item 3 is a gap in the writer: naming the plane–cone and plane–torus
  circles and a turned cylinder's seam would let three of the six shapes — the
  frustum, the cone and the bend — and the turned bars reach the comparison,
  where item 2 is what a turned bar already finds there.
- A night of this target in fork mode will collect item 1 and item 2 over and
  over. That is the target working, but it will fill a night's crash count with
  two causes until they are fixed.

*Since Q405 (2026-10-02) a turned cylinder's seam is named: an open edge the
model says runs straight is written as a `LINE`, and the seed
`a_round_bar_turned_a_quarter_about_x` is written and comes back the same
(`every_shape_seed_is_the_shape_it_is_named_for`). The plane–cone and
plane–torus circles are still refused (Q662, Q407).*

*Since Q662 (2026-10-03, landed by Q821) the plane–cone circle is named too:
a flat cap square to a cone's axis is written as the circle worked out in the
frame the two faces are stored in (§366). The seeds
`a_frustum_of_a_half_slope_cone` and `a_pointed_cone` are written and come
back the same (`every_shape_seed_is_the_shape_it_is_named_for`), so of the
three shapes the item above names only the bend is still refused, at the
plane–torus circle (Q407).*

*Since Q407 (2026-10-03, lane `lane-q407-a-bend-cap-written-in-its-stored-frame`,
off `25860052`) the bend is written and read back, and all three shapes the
item above names come back the same; the seed `a_quarter_bend_of_pipe` does
too. Three changes, each in its own doc comment:*

- *The writer names a closed edge where a flat cap holding a ring's axis
  crosses it: the circle round the tube on the corner's side of the axis,
  worked out in the stored frame and placed
  (`where_a_cap_holding_a_rings_axis_crosses_it` in `write.rs`). It turns the
  way the ring winds round its tube at the corner, which is the cap's normal
  or its reverse by the same sign that picks the side, so
  `which_way_a_closed_crossing_turns` is not needed and Q70's four sign
  choices are not searched. Turned the other way, the seed's file is refused
  by the reader.*
- *The writer names the seam, a quarter of the inner equator, as an arc in the
  plane its run names (`where_a_plane_square_to_a_ring_crosses_it_nearest`),
  which was the second refusal behind the first.*
- *The reader adds the pipe of the ring's own radius to the half-space that
  picks such an arc, inside it for the inner circle and outside for the outer
  (`which_of_a_rings_two_circles` in `brep.rs`). A half-space alone holds part
  of both circles, which `rcad_geom`'s own crossing of a plane and a ring says
  needs the pipe, and the comparison found the read-back seam picked by one
  picker where the kernel's had two. **The reader was the side that was short,
  not the comparison.***

*Q70's figures, run again on this base. `how_often_a_model_the_mutator_draws_is_refused`
counted 85 of 400 draws refused at `{ first: "plane", second: "ring", closed: true }`
before and none after; 128 of 400 refused before and 43 after. The seed
measures `118.38670869374307` built and read back
(`the_quarter_bend_seed_measures_the_same_read_back_from_its_own_file`), which
is the number Q70 saw read back; Q70's built `118.38685376813798` and the loop
starting one coedge later were not seen on this base, which builds its bends
the way a file writes them (§361). That §361 is what moved them is not
checked.*

*What this leaves: of the 85 draws now written, 3 are refused by the reader
with `CannotNameWhichPieceOfACrossingThisIs` and 7 more stop at "a different
surface", a turned bend's axis read back an ulp moved, which is Q497's gap
reaching tori as it reached cones. `how_often_a_turned_bend_comes_back_from_its_own_file`
turns the bend about `x`, `y` and `z` by every tenth of a radian: of 372, the
reader refuses 28, all about `x` or `y`, and the writer refuses one seam. Neither
was traced.*

*Since Q843 (2026-10-04) the writer writes the one turned bend it refused,
ring 7 and tube 1 turned about y by 5.0 and moved by (3, 5, 10). Its seam's
plane has as offset the turned normal dotted with the move in plain f64, 32
ulps from the ring's centre dotted and rounded once, and of every f64 centre
the reader could hold on the ring exactly one reads back as that plane: 23
steps of x and 19 of z out, at 15.88 of the 16 units the reader allows. So a
gap and not a limit (a_turned_bends_seam_has_one_centre_its_plane_reads_back_from
in write.rs). The writer now looks that far for a ring's seam
(the_nearest_centre_a_ring_holds_its_circle_at), asking the reader's own
rule (`a_surface_holds_the_circle` in brep.rs, shared rather than copied
since Q957), and how_often_a_turned_bend_comes_back_from_its_own_file
writes all 372; the reader still refuses 28 (Q842).*

*Since Q842 (2026-10-04) the 28 are traced, and the gap is Q497's. The
reader builds a `TOROIDAL_SURFACE`'s axis with `Axis::new`, which scales the
written direction to length one again, and keeps a seam's `PLANE` normal as
written (§313.3). The writer writes both as one direction, so where the
scaling moves a bit the seam's plane and the ring's axis are two spellings of
it: turned about x by 2.5, `(0, -0.5984721441039565, -0.8011436155469336)`
against `(…, -0.8011436155469337)`, crossing to `(5.55e-17, 0, 0)` and not
exactly nothing. `rcad_geom`'s walk tells a plane square to a ring's axis from
one holding it by that exact zero (`where_a_plane_cuts_a_torus`), so it takes
the seam's plane for one holding the axis, and neither of
`the_two_meridian_circles` in walk.rs holds the seam's two ends:
`which_of_these_circles_this_run_is` refuses. The reader's own
`which_of_a_rings_two_circles` asks the same zero and adds no picker, which is
why the row saw the same count with it and without. Q489's
`two_meridian_circles` in intersect.rs is not reached: a file's run is never
named by `intersect`.*

*Of the 372, the bends whose placed ring axis keeps its bits under
`Axis::new` number 296, and every one comes back the same; of the 76 whose
axis moves, 28 are refused and 48 are read back with the axis moved, so none
comes back the same (`how_often_a_turned_bend_comes_back_from_its_own_file`
now pins all five counts). With the reader's torus arm keeping a direction
whose `|d·d − 1|` comes out within 64ε in plain f64, the reach
`Plane::NORMAL_LENGTH_ROUNDOFF` gives a plane's normal, made by hand in
geometry.rs and point.rs, measured and taken out again, all 372
came back the same, every located number bit for bit, and of
`how_often_a_model_the_mutator_draws_is_refused`'s 400 draws the 3 the reader
refused went to none and the 29 that came back with a different surface went
to 24, the rest being pipes and cones. So the answer exists in the file and
the reader loses it: a gap, and its fix is Q497's write set, not this row's.
The walk's own zero test is not the gap — a plane that is not exactly square
to a ring's axis does not cross it in circles round the axis, and drawing them
there would need a bound on the lean charged to the walk's deviation that
nothing names.*

*Since Q1000 (2026-10-05, §355.6) the reader keeps a pipe's, a cone's and a
ring's axis as the file wrote it (`Axis::as_written`), and a saved model
loads with the axis bits it was saved with. Of the 372 turned bends all 372
come back the same and none is refused
(`how_often_a_turned_bend_comes_back_from_its_own_file` pins `(372, 0, 372, 0,
0)`, and 76 for the ring axes `Axis::new` would have moved). Of
`how_often_a_model_the_mutator_draws_is_refused`'s 400 draws, the 29
comparisons that stopped at a different surface and the 3 at a file the
reader refused went to none. The walk's own zero test is unchanged, for the
reason the paragraph above gives.*
