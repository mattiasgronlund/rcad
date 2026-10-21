## 376. A vertex loop at a cone's apex is read as no bound — 2026-10-02

*Queue row Q746, which replaces Q735. The STEP reader refused every
`VERTEX_LOOP` as `ALoopThisReaderDoesNotWalk`. It now reads one whose corner is
at the apex of the cone its face sits on as no bound at all, and refuses every
other one by the same name as before. This lane changed
`crates/rcad-step/src/brep.rs` and `refusal.rs`, and the tests in
`what_the_mapping_refuses_and_by_what_name.rs`.*

In short: 18 of the 136 NIST cone faces that carry an apex loop now get past
it. The other 118 put the loop's corner thousands to a million roundings away
from where the cone pinches, so they stay refused. No whole file imports that
did not before, in either corpus.

### 376.1 The count, measured again

Q730's row said 136 `CONICAL_SURFACE` faces of the NIST files bound their apex
with a `VERTEX_LOOP` (`nist_ctc_05` `#1340`). Nobody had measured it since.
Counted from the text of the files on the base, `311b06e2`:

```text
python3 <scratchpad>/vertex_loops.py <corpus>/nist
python3 <scratchpad>/vertex_loops.py <corpus>/kicad
```

The script lists every face whose bound holds a `VERTEX_LOOP`, grouped by the
kind of surface. NIST has 136, all of them `CONICAL_SURFACE`, in 11 files.
KiCad has none. Every one is a `FACE_BOUND` beside a `FACE_OUTER_BOUND` that is
the cone's base circle.

So the 136 stands. KiCad has no apex loops, so no KiCad number can move.

The script lived in the lane's scratchpad and is gone. The word count it
rests on can be had again without it: `grep -ro VERTEX_LOOP <corpus>/nist |
wc -l` prints 136, the same grep with `-rc` names the 11 files, and over
`<corpus>/kicad` it prints 0 (merger, 2026-10-02).

### 376.2 Why no bound is sound for this kernel's cone

This kernel's pointed cone has no wire at its apex. `rcad-topo`'s
`a_pointed_cone` builds the cone face from one wire, its base circle. The mesher
closes the face at the apex the `Cone` stores, and the measure charges the tip
its whole size (`what_a_cone_tip_costs_a_bound.rs`). A STEP file says the same
face with a second bound: a
`VERTEX_LOOP` of the apex. If that corner really is the apex, dropping the
bound leaves exactly the face this kernel builds itself.

The catch: dropping the loop drops its corner too. After the reader, nothing
sees that corner again. If the corner is not at the apex, the file describes a
face the reader did not build, and no later check would notice. So the reader
has to decide "at the apex" itself, and decide it to the standard the
validator would have used.

### 376.3 "At the apex": how Q730 decided it, and why a cone needs more

Q730 (`49751de3`) puts a ball's pole back where a loop turns back along its
seam. It asks `Surface::the_way_its_first_coordinate_runs_at(at).is_none()`,
exactly and with no tolerance. That works because the pole goes back in as a
corner, and `Model::validate` then checks the pole is where the ball pinches
(`stands_where_it_pinches`, with `what_writing_a_point_down_costs` of room).

On a cone the same test is not enough, for two measured reasons:

- **It answers `None` on the whole axis, not only at the apex.** Run on the
  base, 102 of the 136 corners are exactly on the axis. 87 of those are
  `2.5e3` to `1.0e6` roundings along it from the apex the reader computes.
  They are inside the cone, and the certified `side_of` says `Neg` for every
  one of them. Q730's test alone would read all 87 as the apex.
- **Exact apex matches none.** Not one of the 136 corners is the stored apex
  bit for bit. A file places a cone at its base, and the reader works the apex
  out as `origin − d·(radius / tan θ)`, which rounds. The nearest 18 corners
  are 0.6 to 8.5 roundings away.

So the reader asks the validator's own question for a pole. It builds a box of
`16 · NEAREST_F64_POINT_ROUNDOFF` times the largest number involved round the
corner, and asks whether the apex is inside it. Sixteen is `MEASURING_SLACK`,
the room `Model::validate` gives a corner on a curved face. The reader already
holds a circle to that same allowance, as `A_CIRCLE_MAY_MISS_ITS_FACE_BY`. No
new tolerance comes in.

"The largest number involved" is the larger of the two points' coordinates and
`radius / slope`. `radius / slope` is how far the reader walked back to reach
the apex, so it is the size of the numbers the apex rounded from. Without it, a
cone with its tip at the origin gets a box sized by a point at zero, which is
no box at all. The hand-written cone in the test is that case.

The validator measures against the whole solid's largest coordinate, which this
crate cannot read. The reader's box is never larger than the validator's would
be, so it can refuse a corner the validator would have kept, but it cannot keep
one the validator would refuse.

Measured on the branch, through `rcad_step::probe` on every corpus file, with a
temporary print of each verdict:

```text
RCAD_STEP_CORPUS=<corpus> mise exec -- cargo run -q -p rcad-step --release --example blocker_depth_probe
```

| Corners | Distance from the apex, in roundings | Read as |
| ---: | --- | --- |
| 18 | 0.6 to 8.5 | no bound |
| 118 | 2507 to 1 040 639 | `ALoopThisReaderDoesNotWalk`, as before |

The nearest refused corner is `nist_ftc_06_asme1_rd.stp` `#1967`. The file
writes `70.4822675346`, ten decimals. The apex is `70.48226753468838`, which is
`8.8e-11` away and 2507 roundings. That is the file's own precision, not ours.
The other 117 are further off still.

The road this lane did not take is a wire of only a pole. It needs a fourth arm
in `SolidBuilder::add_wire`'s `check_the_steps_close` and in
`Invalid::WireTooShort`. It would not change this table: the validator would
then refuse those 118 corners as off their face.

### 376.4 Before and after

`mise exec -- just t -p rcad-step --no-capture -E
'binary(merged) & test(=how_often_a_real_file_imports::how_often_a_real_file_imports)'`,
on the base behaviour and on the
branch:

- NIST imports 1 of 44 on both, and KiCad imports 164 of 323 on both. Nothing
  in `KICAD_FILES_THAT_IMPORT` moved.
- The tally of first refusals is the same, line for line. `a loop this reader
  does not walk: VERTEX_LOOP` is the first refusal of 4 files on both:
  `nist_ctc_04_asme1_ap203.stp`, `nist_ftc_06_asme1_rd.stp` (two copies) and
  `nist_stc_06_asme1_ap242-e3.stp`. In all four the apex corner is off by a
  modelling tolerance.

Through `blocker_depth_probe`, which keeps walking past a refusal, the reason is
gone from 2 files and stays in 9:

| File | Apex loops | Read as no bound | What the file still meets |
| --- | ---: | ---: | --- |
| `nist_ctc_05_asme1_ap242-e1.stp` | 10 | 10 | `B_SPLINE_CURVE_WITH_KNOTS`, `BOUNDED_SURFACE` |
| `nist_stc_10_asme1_ap242-e2.stp` | 8 | 8 | `B_SPLINE_CURVE_WITH_KNOTS`, `BOUNDED_SURFACE` |

The 18 freed faces meet nothing new at the face level: neither file gets a
reason it did not have before. Both files already had both of the other
reasons, so neither reaches `SolidBuilder::build`. That means the corpus never
builds a cone closed this way. The hand-written cone in
`a_vertex_loop_at_a_cones_apex_bounds_nothing_and_the_cone_imports` is the
evidence that one does: it imports, validates under a certified policy, and
measures `πr²h/3` within its own bound. Without this change, that test fails
with `ALoopThisReaderDoesNotWalk`.

This is §44.11 item 5 again. The number of faces a blocker stops is an upper
bound on what removing it is worth. Here it was worth no file at all.

### 376.5 What is a limit and what is a gap

(§16.10.)

- **The 118 corners off the apex are a limit, not a gap.** The answer "this
  corner is the apex" does not exist for those numbers, because the corner is
  a modelling tolerance away from it. The same corner kept as a pole would be
  refused by `Model::validate` as off its face. Reading it anyway would be
  healing, and M5 does no healing (§42.9).
- **A `VERTEX_LOOP` on any other surface is a gap with no file behind it.**
  Neither corpus has one. It would need the single-pole wire of §376.3, in
  `rcad-topo`.
- **This leaves §241.6 where it was.** Its writer gap for the cone, naming the
  circle where a plane meets a cone, is about the writer, and this lane
  changed only the reader.

> **2026-10-05 note (Q1023): one of the four files was not a modelling
> tolerance.** `nist_stc_06_asme1_ap242-e3.stp`'s two apex `VERTEX_LOOP`
> corners (`z = 70.4822675346`) were the half-angle's own cut to ten digits,
> not a modelling tolerance away from the apex. Since Q955 (§366.7), the
> corners' slope, `1.6642794823614244`, is read back from the corners
> themselves and puts the apex exactly on them, so the loop now stands and
> the file is no longer among Q751's four. It is refused first for a
> different reason, a hole that winds the same way as its outer wire, which
> is Q1022's to trace.

### 376.6 What is left

- `VERTEX_LOOP` is matched by its spelling, as `A_LOOP_OF_ONE_CORNER` in
  `brep.rs`. `crates/rcad-step/src/names.rs` and `fuzz/step.dict` were
  outside this row's write set, so the reader fuzzer's dictionary does not
  carry the word yet. Moving it costs one line in each file and a swap in
  `brep.rs`.
- `crates/rcad-step/src/write.rs` lines 117–119, on
  `CannotWrite::AWireThatStandsStillAtAPole`, say a `VERTEX_LOOP` is
  something "the reader does not walk yet". That is now only half true: the
  reader reads one at a cone's apex. The sentence is about a ball's pole,
  which is still refused, so it wants "anywhere but a cone's apex" added.
  That file was outside this row's write set too.
