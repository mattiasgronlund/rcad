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
came_back_the_same: 4 }` (`mise exec -- just t -p rcad-step --test
every_model_the_writer_fuzzer_kept_is_written_again --no-capture`).

**A turned seed that is written is not among them, because the only one there is
panics** — 241.5 item 2. The turned seed that is in is refused before the
comparison, so it covers the turn through the writer's refusal path only.

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
   the centroid differs at the 1e-15 level. The same bar also moved by
   (3, −5, 7) fails the same way. A move by whole units passes for that bar, but
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

### 241.6 What is left

- 241.5 items 1 and 2 are defects, each with a saved input, and want a row each.
- 241.5 item 3 is a gap in the writer: naming the plane–cone and plane–torus
  circles and a turned cylinder's seam would let three of the six shapes — the
  frustum, the cone and the bend — and the turned bars reach the comparison,
  where item 2 is what a turned bar already finds there.
- A night of this target in fork mode will collect item 1 and item 2 over and
  over. That is the target working, but it will fill a night's crash count with
  two causes until they are fixed.
