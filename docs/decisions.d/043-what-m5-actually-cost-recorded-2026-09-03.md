## 43. What M5 actually cost, recorded 2026-09-03

*Written by M5's closing gate. §12 did this for M0, §16 for M1, §17.8 for the
fuse, §26 for M2, §30 for M3 and §39 for M4.*

**Every number below was re-derived at this gate from the tree at `25cea9a`, not
copied from a lane's report.** Twenty-two lanes across nine waves left
twenty-eight reports in the scratchpad; the lanes corrected each other four
times while they ran, and this gate broke six more claims, which are in §43.9.
Where a claim did not survive it is written down rather than dropped.

**§42.10 reserved §43 for the builder, §44 for the reader and §45 for this
section. Neither §43 nor §44 was ever written** — the builder lane and the
reader lane wrote code and commit messages and reported to the scratchpad, which
is what §13 rule 2's fan is supposed to do. So the closing gate takes the next
free number, and the reservation is void. §42.14's prediction, which said it
would be "scored at §43", is scored here.

`main..m5` is **103 commits**, 3 277 files and +29 435 lines — but 3 180 of those
files are fuzz corpora. The code is **91 files, +25 508 / −727**, and it adds a
sixth crate.

### 43.1 M5 built a builder first and a reader second, and the reader is the part that measures

§42's headline was that M5's first deliverable is not a STEP reader but a public
way to build a solid out of faces, because there was no route from outside
`rcad-topo` into an arena. That held. `SolidBuilder` exists, six levels deep, and
a cube, a round bar, a ball and a bend of pipe are each taken apart through
`FaceView`/`EdgeView`/`VertexView` alone and described again to the same volume,
area and centroid **bit for bit**
(`crates/rcad-topo/tests/building_a_solid_out_of_faces.rs`). Nothing public hands
out a `&FaceData`.

Then M5 did what roadmap §12 asked, in one sentence: **a stranger's file becomes
a `Model`, or it is refused by a name that says which entity and why.** 367 real
files parse, 46 of them import, and every refusal carries the `#id` that stopped
it.

**And it did two things §42 explicitly deferred.** §42.13 put the cone and the
torus behind "the spike's count", and the spike came back at 9.1% and 73.1% —
so both were built. That decision is the largest single thing about this
milestone and it is why the numbers below read as they do:

- `Surface` went from **three** variants to **five**. A cone is a quadric with a
  pinch; a torus is not a quadric at all, and its sign predicate is a quartic,
  one step past everything M3 and M4 built.
- The reader's ceiling moved with it. The spike measured files built only from
  plane, sphere and cylinder at **9.1% NIST / 73.1% KiCad**; counting the five
  surfaces and three curves the reader now holds, the same walk gives **34.1% /
  83.6%**. Two lanes moved the ceiling 25 and 10 points.
- And the *rate* is nowhere near the ceiling: **14.2% KiCad, 0% NIST**. The
  distance between the two is the whole of §43.6 and §43.10.

**What M5 did not do is the half §42.12 criteria 8, 11, 12 and 13 name.** No
pairwise check over solids exists; `add_extrusion` still stores its lift on the
shell rather than baking it; the mesher still charges a placed face
`Surface::ROUND_TRIP_ROUNDOFF`; and `rcad-predicates` is at **0 files changed for
the whole milestone**, so `Approximation` still has no boundary guard. Three of
those four were §42.12's "three inherited defects" lane, which never ran.

### 43.2 The fourteen criteria, and the test that discharges each

§39.1's shape. **Eight are met, two are met in part, and four are unmet.** Of the
three §42.12 called load-bearing — 1, 8 and 10 — one is met, one is met in part,
and one was never started.

| | Criterion | | Discharged by |
|---|---|---|---|
| 1 | a solid built face by face is the solid `add_cube` builds | **met** | `a_cube_built_face_by_face_is_the_cube_add_cube_builds`, and three more shapes beside it: a round bar, a ball with its pole and its seam, and a bend of pipe. Volume, area and centroid `assert_eq!`, then the same `Rule::Difference` against a third solid with every piece compared bit for bit |
| 2 | no sequence of builder calls panics; every misuse is a named `Err` | **met** | `crates/rcad-topo/tests/no_sequence_of_builder_calls_panics.rs`, three families — 512 drawn scripts, a cube with a drawn surface on one face so the sweep is reached, and 512 orderings of one cube as the control. Soaked at `RCAD_SOAK_FACTOR=200`: 102 400 scripts, zero panics. Twenty more in `what_the_solid_builder_refuses.rs`, one per refusal |
| 3 | a hand-written AP203 file holding one block reads in and measures its closed form | **met** | `a_block_written_by_hand_in_a_step_file_measures_what_its_numbers_say`, and `a_quarter_disc_plate_written_by_hand_measures_what_its_numbers_say` beside it — the plate is the half that matters, because it has a cylindrical face and two arcs between two corners |
| 4 | a surface kind the kernel cannot hold is refused by the name the file uses | **met, and not in the shape it was written in** | The criterion's own example was a cone, and M5 built the cone. `a_surface_kind_this_kernel_does_not_hold_is_refused_by_the_name_the_file_gives_it` uses `B_SPLINE_SURFACE_WITH_KNOTS` and asserts the keyword comes back on the refusal; `a_curve_kind_this_kernel_does_not_hold_...` is its sibling. On the corpus this is 20 files by surface and 48 by curve |
| 5 | no sequence of bytes makes the reader panic | **met, and it found one** | `fuzz/fuzz_targets/no_step_file_makes_the_reader_panic.rs`. It crashed in the first five minutes — `\S\` applied to an apostrophe stepped over one byte of the Part 21 pair and an `assert_eq!` fired on the other — fixed in `642c6ba`, then 12 370 267 and 9 348 785 executions found nothing. The target walks everything a caller can reach *after* the read, because a reader that returns `Ok` and panics when you look at what it read has only moved the crash |
| 6 | the file's unit reaches `Scale::unit`; an unknown unit is refused | **met** | `a_file_written_in_inches_comes_back_in_inches` measures one cubic inch and not 16.387 of anything, and `a_unit_the_kernel_does_not_have_is_refused_rather_than_scaled`. No length conversion happens anywhere: §3.5 says a unit is a label, so `CONVERSION_BASED_UNIT` is matched against an exact table of eight `(factor, base)` pairs and a factor outside it is refused with the number printed |
| 7 | a file asking for more precision than its numbers carry says so | **met, and vacuous on the corpus** | `a_file_declaring_more_precision_than_its_numbers_carry_says_so`, one comparison of two numbers the file wrote. **0 of 367 real files reach it** — every one declares something around `1e-7` on a part tens of millimetres across, about `10^9` coordinate spacings and comfortably keepable. The check is real and the corpus does not exercise it |
| 8 | two solids that occupy the same space are not called a valid model | **unmet** | Nothing was built. The tree pins the opposite: `two_solids_described_at_the_same_place_are_still_called_a_valid_model` builds a cube, describes the same cube again through `SolidBuilder`, and the certified sweep returns `Ok(())`. A grep for a public interference pass over `crates/*/src` returns nothing. §42.10's lane 4 never ran. The test fails the day the criterion is met, which is §19's shape and the right way to leave it |
| 9 | every solid the corpus yields validates, or the refusal names one | **met, under another name** | Not `every_solid_in_the_corpus_validates_or_the_refusal_names_one` but the harness's own `.expect("and every imported model validates under a certified policy")` inside `how_often_a_real_file_imports.rs`, run over every KiCad import — 46 when this was written, 97 measured today — and all 88 control files. **No imported model has ever failed the sweep**, and structurally none can: `SolidBuilder::build` writes into a throwaway model, runs the real sweep over that, and only writes into the caller's model if the sweep is happy |
| 10 | the import rate, family by family and never blended, with a control at 100% | **met in part** | `how_often_a_real_file_imports` measures and publishes three families — control **88/88**, KiCad **97/323 = 30.0%**, NIST **1/44 = 2.3%** — with per-family refusal breakdowns and a corner-miss distribution. The control's 88 is asserted and one refusal there is one failure. **The real-file rate is asserted too, by name and not by rate.** `d23c700` pinned the KiCad set as `KICAD_FILES_THAT_IMPORT`, and `assert_pinned_kicad_imports` compares it against what imported as a *set*, so it fails **both ways**: a file that starts importing fails it exactly as a file that stops does, and the message names which files moved rather than handing back a count (§44.10). What is still missing is only that the whole measurement sits behind `RCAD_STEP_CORPUS`, so it is not in the suite a run without that variable covers. Row rates re-measured 2026-09-06 with `mise exec -- just t how_often_a_real_file_imports --no-capture`; the row said `46/323 = 14.2%` and `0/44`, which is the tree at `d23c700` and before the lanes that moved both — `KICAD_FILES_THAT_IMPORT`'s own doc comment accounts for the fifty and then the one |
| 11 | `add_extrusion` bakes its lift | **unmet** | `crates/rcad-topo/src/build.rs:571` still writes `ShellUse { shell, loc: *placement, .. }`. No test of that name exists anywhere. §41.13's §19 violation is untouched |
| 12 | the mesher reads a placed face's stored surface instead of round-tripping it | **unmet** | `crates/rcad-topo/src/fuse/select.rs:1045` still charges `Surface::ROUND_TRIP_ROUNDOFF`, which is `256 · f64::EPSILON`. §41.11 item 19 is untouched |
| 13 | `Approximation` gets the boundary guard `Scale` has | **unmet** | `Approximation` still derives a plain `Deserialize` with no `try_from` in `crates/rcad-predicates/src/provenance.rs`. `rcad-predicates` is at **0 files changed for the whole milestone** |
| 14 | N occurrences come through as N solids at N placements, and what that costs is measured | **met in part** | `five_copies_of_one_shape_cost_five_times_the_entities` gives the exact multiple — five cubes are exactly `5 × 8` vertices, `5 × 12` edges, `5 × 6` faces, because nothing is shared anywhere. **The 500-occurrence memory and mesh-time figure §42.7 asked for was never taken**, so nothing has a target to beat. And the corpus turned the question round: only 2 of 367 files carry real `NEXT_ASSEMBLY_USAGE_OCCURRENCE` instancing, and the reader refuses both by name |

**The gap between criterion 1 and criterion 8 is the shape of the whole
milestone.** The one that had to exist before anything else could be built was
built first, in the first wave, and everything downstream of it landed. The one
that needed a new primitive nobody had — "do these two solids share space", with
a `Decision` and a named refusal — was never started, and §42.14 item 3 already
guessed it would be the second-most expensive thing here. It was not expensive;
it was skipped.

### 43.3 A `# Panics` that argues where it should demonstrate, eleven times

§16.10's species, and this milestone is where it stopped being a curiosity.
**Eleven asserts, `expect`s or `panic!`s were demonstrated this milestone to be
reachable with data a caller can hand over**, every one of them carrying a
sentence explaining why it could not be. Each row names the input that broke it.

| Where | The sentence that was wrong | What reached it | Fixed |
|---|---|---|---|
| `read.rs`'s string decoder | an `assert_eq!` on an invariant of the escape grammar | fuzzed bytes: `\S\` applied to an apostrophe, which Part 21 writes as two | `642c6ba` |
| `walk.rs` `the_conic_between` | *"the marcher should already have this run"* — a `# Panics` clause arguing both callers guard first | **a real KiCad file, through `Import::read`**: a pipe of radius 0.5 and a bend at `(0,0,16.15)`. The guard it trusted is the marcher's list of three pairs, and a cone and a torus joined neither list | `ee8eab7` |
| `walk.rs:1053` `where_a_plane_cuts_a_cone` | *"`where_a_plane_and_a_cone_cross` refuses every one of those by name"* — true, and about a different function | `NamedRun::the_whole_of` is public and `EdgeRun` is `Deserialize`. All four refused shapes reached it | `169589d` |
| `assemble.rs:1888` | *"so no run carrying either exists"* — the same sentence a third time | the same public route. Here the arm returns `false` rather than crashing, so the cost was a wrong reason and not a panic | `a6742f0`, comment only |
| `validate.rs:360` → `ShellView::data` | a `Model` off the wire naming a shell that is not in the arena | fuzzed bytes through `rcad_step::write`, at 17 827 556 executions | `7227ab7` |
| `validate.rs:124` | an edge's derived coedge index used to index the coedge arena | a probe: take one coedge out of a sound cube and sweep. **A second crash in `validate` nobody had found** | `7227ab7` |
| `model.rs:142` `Model::shell` | *"panics if the key names an entity that is not there"* — written for a key from the same model | the same fuzzed bytes, one crate over, through `rcad_step::write`'s own walk | `7227ab7`, at the door |
| `view.rs:565` `FaceView::outer_wire` | `expect("a face has at least one wire")` | a cube written out with one face's wire list emptied, read back | `9d09441` |
| `model.rs:566` `Model::place_solid` | `check_touched_wire`'s assert that a wire closes | a cube written out with a wire's first two steps swapped, read back | `9d09441` |
| `measure.rs:196` | *"a solid must enclose a positive amount of space"* — §20.1 settled it as a corrupt solid | a cube written out with every face use reversed, read back. `validate` names it; `measure` crashed on the same bytes | `72c7eba` |
| `assemble.rs:235` `Assembly::commit` | `.expect("a fuse writes a valid model or does not write at all")` | fuzzed bytes into `Model::fuse`, at 165 786 executions — two 10 mm boxes a hair apart | `f3ecb04` |

**Three of them are the same sentence** — *"nothing builds such a run"* — written
three times about `walk`'s three pair lists, and all three are wrong for one
reason: `EdgeRun` and `NamedRun` are public and `Model` is `Deserialize`, so a
run is caller data and never an internal state. **Four were found by fuzzing
rather than by reading**, and two of those four — the writer's shell panic and
the fuse's commit — had survived every review this project has run.

**The cost is the number that matters.** Four fuzz targets exist where there was
one; three of them found a real defect on their first serious run; and the door
they all knock on — `Deserialize for Model` — now runs `check_keys_resolve` and
`check_shape_is_possible` before a `Model` exists, which is what closed five of
the eleven rows above at once.

**The rule this suggests is not new, only earned:** a `# Panics` clause whose
argument is about *what today's callers do* is worth nothing on a type that
derives `Deserialize` or is reachable from a `pub` constructor. §16.10 already
said to check the public API rather than today's callers. Eleven rows say the
check was not being done.

### 43.4 A hand-kept list of surface pairs that a new kind never joined, seven times

The other species, and it is §40's own subject. A `match` over `[Surface; 2]` or
`(&Surface, &Surface)` naming some pairs and sweeping the rest into `_`. **Seven
sites, in four crates**, and neither `cargo check` nor clippy nor
`check-wildcards` could see any of them.

| Site | What the `_` swallowed | What it cost | Fixed |
|---|---|---|---|
| `walk.rs` `a_crossing_that_runs_to_a_quartic` + `the_conic_between` + `where_a_quartic_can_be_picked_up` | three lists that had to agree, and nothing checked that they did | the `# Panics` in §43.3 row 2, reached from a real file | `219f6f5`, folded into one classification with no wildcard |
| `along_a_crossing.rs` `the_way_this_crossing_winds` | every cone and torus pair, into `TheseTwoSurfacesDoNotCrossInAConic` | a plane against a torus refused for a reason that is false — it does cross in a conic, and each circle is flat and convex | `6b60290` named all twenty-five, then `ee58ed2` answered the two circle families |
| `fuse/arrange.rs` `a_pair_that_crosses_in_a_conic` | plane-and-torus | **every crossing a capped bend has, refused as `ACrossingThisCannotBeOrderedAlong` — a false statement about the arithmetic.** Every one of a bend's crossings is a circle and every one orders fine | `769dada`, then `933ab9f` named all twenty-five |
| `fuse/arrange.rs` `a_plane_along_a_pipes_axis` | fifteen ordered pairs | nothing — the `false` is right about the *question*, which is exactly the kind of `false` a sixth kind would break | `01eddc5` |
| `fuse/assemble.rs` `the_crossing_of_these_runs_straight` | ball/plane, ball/ball, ball/pipe and every torus pair | nothing wrong, and the lane that reported it named the two dead pairs and missed the live one: **ball-and-plane arrives twelve times** in one run of the `rcad-topo` suite | `01eddc5` |
| `rcad-step` `write.rs::curve` | twenty-two of twenty-five pairs | nothing — the refusal was already right | `2ad6d93` |
| `rcad-mesh` `how_this_domain_is_scaled` | **`Torus` was named, and a bare `_` was left in the same arm beside it** | a kind this crate has not been taught gets `(1.0, 1.0)` back, silently, with no signal — the exact shape of the bug the torus arm was added to fix | `fa95e7b` named it, `d770219` split the arm |

**The last row is the one to read twice.** Somebody had already been bitten by
this arm, had added `Surface::Torus(_)` to it by name, and had left the `| _`
sitting on the same line. The fix that was applied made no difference at all to
the next kind.

**What actually catches a sixth kind, measured at this gate** on a throwaway
clone with `Surface::ProbeSixth(Plane)` added:

| Crate | `cargo check` E0004 | with `--all-targets` | clippy wildcard sites, five kinds → six |
|---|---:|---:|---|
| `rcad-geom` — owns `Surface` | **15**, across 7 files (18 arms once the cascade settles) | **31**, across the same 7 files | 41 → 40; the compiler is its guard, not the lint |
| `rcad-topo` | **0** | **0** | **30 → 50**, 20 new, 0 gone |
| `rcad-mesh` | **0** | **0** | 0 → 4 |
| `rcad-step` | **0** | **0** | 3 → 6 |
| `rcad` | 0 | 0 | 8 → 8 |
| `rcad-predicates` | 0 | 0 | 0 → 0 |
| **workspace** | | | **82 → 108** |

**So the ratchet works everywhere except the seven places it is needed.**
Clippy's `wildcard_enum_match_arm` asks only what the scrutinee's own type is,
and all seven sites match over an array or a tuple.
`composite_wildcards.awk` sees them — all seven, plus a geom test — but it counts
**sites**, and a site that names every pair today keeps counting as exactly one
after a sixth kind arrives. Demonstrated at this gate: the composite count is
**8** at five kinds, and the same three `rcad-topo` sites are still one apiece at
six.

`wildcard_arms` is **90 = 82 direct + 8 composite**, up from 76 at §42.16. The
90th is `a_pair_that_crosses_in_a_conic`'s own `_`, bought deliberately in
`933ab9f` so that the twenty-five pairs could be named.

**The only place a sixth `Surface` kind can be a compile error is the crate that
declares it.** That is §43.8 item 7, and it is the fix the two lanes that
measured this reached independently.

### 43.5 The numbers

| | at §42.16 | now | how |
|---|---|---|---|
| tests (`cargo nextest`, whole workspace) | 1141 | **1427** | `mise exec -- just default`, green twice |
| `#[test]` attributes under `crates/` | 1141 | **1427** | `git grep -c`, and it agrees with nextest exactly at both ends |
| suite duration | 2.759 s | **5.813 s** | the same run |
| doc tests | 19 | **21** | 4 + 1 + 0 + 13 + 1 + 2 |
| crates | 5 | **6** | `rcad-step` is new |
| `Surface` variants | 3 | **5** | `Cone`, `Torus` |
| `FORMAT_VERSION` | 3 | **3** | a fifth variant takes the next `postcard` tag and moves none of the first four, asserted byte by byte |
| wildcard arms over our own enums | 76 | **90** (82 direct, 8 composite) | `just --set wildcard_arms 0 check-wildcards` |
| fuzz targets | 1 | **4** | reader, writer, fuse, surfaces |
| fuzz corpus entries | 824 | **824 / 907 / 1088 / 1180** | one per target |
| public ways to build a solid out of faces | 0 | **1** | `SolidBuilder` |
| `rcad-predicates` files changed | — | **0** | for the whole milestone, as in M4 |
| `rcad-mesh` files changed | — | **1** | `triangulate.rs` |

**The test count at the start of the run was 1141 and not 1113.** 1112 is M4's
close (§39); 1141 is what §42.16 recorded and what the merge base `2bc1361`
measures. The delta is **+286**. Per crate: `rcad-geom` 528 → 647, `rcad-topo`
382 → 452, `rcad-step` 0 → 92, `rcad` 42 → 45, `rcad-mesh` 26 → 28,
`rcad-predicates` 163 → 163.

**Import, per corpus, never blended.** `RCAD_STEP_CORPUS` pointing at 44 NIST
`.stp` files and 323 KiCad `.step` files, re-run at this gate:

| | parse | import | round trip |
|---|---|---|---|
| control — files this kernel wrote itself | — | **88 / 88** | — |
| `kicad/` | **323 / 323**, 1 852 229 instances | **46 / 323 = 14.2%** | **45 / 46**, and `scale_metadata_differed = 0` |
| `nist/` | **44 / 44**, 514 369 instances | **0 / 44 = 0.0%** | not attempted; nothing imports |

**2 366 598 entities** compared against `step-io`'s own Part 21 parser, id for
id, type name for type name, attribute count for attribute count and value for
value, with the test asserting the real-disagreement list is empty. The one class
of difference is **388 strings in NIST**, where `step-io` leaves `\X\` and `\X2\`
escapes as written and this reader decodes them; `step-io`'s own doc comment says
it defers that step, so both are right at their own layer.

**Why 46 and not 270.** The refusals, counted once per file, named by what
stopped it first:

| KiCad | | NIST | |
|---:|---|---:|---|
| 153 | a corner is not on the curved face the file says it is on | 16 | a wire needs three steps unless its one edge closes |
| 59 | a wire needs three steps unless its one edge closes | 15 | `B_SPLINE_CURVE_WITH_KNOTS` |
| 29 | `B_SPLINE_CURVE_WITH_KNOTS` | 5 | a corner is not on the curved face |
| 13 | `B_SPLINE_SURFACE_WITH_KNOTS` | 4 | `VERTEX_LOOP` |
| 8 | a corner is not on the plane of the arc it ends on | 2 | two different length units |
| 23 | the tail: `SURFACE_OF_REVOLUTION` 6, a void 3, `SURFACE_CURVE` 2, `NEXT_ASSEMBLY_USAGE_OCCURRENCE` 2, `HYPERBOLA` 1, `SURFACE_OF_LINEAR_EXTRUSION` 1, and 8 more | 2 | `BOUNDED_CURVE` 1, the file has no size 1 |

**The headline refusal is two problems wearing one name**, and the harness splits
them rather than reporting a median. Of the 161 KiCad files with a corner off a
curved face, **88 are inside one coordinate spacing** — the file wrote the
nearest `f64` there is, `x² + y² − r²` worked out exactly is still not zero, and
no writer could have done better. The other 73 run out to **20 777 spacings**,
which is a modelling tolerance somebody chose and which no arithmetic anywhere
fixes. NIST's five split 2 and 3, at up to 13 169 spacings.

**A bend of pipe fuses with nothing, and the refusal is now true.** 96 fuses per
family, deterministic, re-run at this gate:

| tool | a bend | a round bar of the same size in the same place |
|---|---:|---:|
| a slab square to the axis | **0 of 96** — a plane cuts this torus in a quartic that is not a pair of circles | **24 of 96** |
| a wedge through the axis | **0 of 96** — the same | **96 of 96** |
| a bar through the hole | **0 of 96** — a torus crossing another curved surface is degree up to eight | **96 of 96** |
| a ball in the bend | **0 of 96** — the same | **0 of 96**; this control measures the tool, not the torus |
| a second bend | **0 of 96** — the same | not applicable |

The rate did not move all milestone. What moved is the reason. It used to be
`ACrossingThisCannotBeOrderedAlong`, which was **false** — every crossing a capped
bend has is a circle and every one orders fine — and the arrangement simply had no
plane-and-torus row. `769dada` added it, and the two refusals above are
statements about geometry that hold.
`no_box_of_planes_can_close_round_a_torus_axis_without_a_quartic` shows there is
no drawable flat-sided tool that avoids the first of them.

**A length no longer squares where the squares do not fit.** `Vector::length` was
`nalgebra`'s `norm()`: sum three squares, take the root. A vector `3.8e-211` long
came back **exactly `0.0`** and one `1e200` long came back `inf`. Both answers are
ordinary `f64`s, so both are gaps. Measured at this gate:

- **1047 of 2094** — scaled copies of one 3-4-5 triangle the old arithmetic got
  wrong, at every power of two from `2^-1072` to `2^1021`. Exactly half.
- **48 297 of 200 000** — lengths in the unit cube where `f64::hypot` gives
  different bits, by up to 2 ulps. That is why the fix is an exact power-of-two
  rescale and not `hypot`: `hypot` would have moved a quarter of every length in
  the kernel to fix a range nothing ordinary is in.
- **0 of 6552** — lengths of an ordinary size the rescale moves. Bit for bit,
  every direction on the lattice `−4..=4` cubed at nine exponents.

**The fuse crash, replayed at this gate from the pinned corpus file.** The bytes
decode to **two 10 mm boxes**, `[0,10]³` and the same box at
`(5, 3.80218313259032e-211, 5)`. Both pass a certified sweep. `10.0 + 3.8e-211`
is `10.0`, so the top faces are one plane and the bottom faces are two — the
union really does have a step in it one hair high. The answer has **51 edges, of
which exactly one is the sliver**. Before `f3ecb04` its length was measured as
`0.0` — `3.8e-211` squared is `1.4e-421`, under the smallest `f64` there is — so
`DynamicRange::measure` returned `FeatureSizeNotPositive { size: 0.0 }` before any
policy was consulted, which is why a *tolerant* fuse crashed. Now:

```
first: answer has 51 edges
first: 51 open edges, shortest chord 3.80218313259032e-211
first: revalidate certified Some(Invalid(DynamicRangeTooWide { ratio: 3.94510192616128e211 }))
first: revalidate tolerant None
```

Two defects in series, of two different kinds. The measurement was a **gap** and
is fixed as arithmetic. What is left is a **limit**: the quantity is a length over
a length, so scaling the whole answer multiplies both and leaves the ratio
bit-identical — there is no scaled copy in which the sliver and the coordinates
fit in `f64` together, and nothing can make the sliver bigger, because it is
where the two inputs really are.

### 43.6 Every refusal M5 leaves, limit or gap

§39.4's shape. **Limit** = the answer does not exist under the constraints;
**gap** = it exists and the arithmetic could not reach it (§16.10). M5 adds
**77 named refusals** across four public enums, plus the ones it moved.

**`rcad-step::Problem` — bytes to an entity graph. 23 variants, and 367 of 367
real files reach none of them.**

| | limit / gap |
|---|---|
| `NotAPart21File`, `FileStopsEarly`, `NotWhatWasExpected`, `StringNeverEnds`, `CommentNeverEnds`, `EscapeThisReaderDoesNotKnow`, `EntityIdTooLarge`, `IntegerTooLarge`, `RealTooLarge`, `ExponentWithoutADecimalPoint`, `EntityIdUsedTwice`, `ComplexInstanceHasNoParts`, `ReferenceInTheHeader`, `HeaderEntityMissing`, `SchemaNamesAreNotAListOfStrings`, `ReferencePointsNowhere` | **limit** — the file is not the file it says it is |
| `KeywordIsNotUpperCase` | **limit** — folding the case is a guess about which entity is meant |
| `ComplexInstanceRepeatsAType` | **limit** — ISO says the parts are distinct, and nothing could say which of two the caller meant |
| `NestedTooDeeply` | **limit** — the alternative is a stack overflow, which is a crash and not an `Err`. `DEEPEST_NESTING` is 64; the corpus maximum is **3** |
| `SectionThisReaderDoesNotRead` | **gap**, deferred — edition-3 `ANCHOR`/`REFERENCE`/`SIGNATURE` |
| `StringIsNotText` | **gap**, deferred — an ISO 8859-1 file writing high bytes raw. Guessing Latin-1 would be healing. 0 of 367 hold a byte ≥ 0x80 anywhere |
| `AlphabetThisReaderDoesNotHave` | **gap**, deferred — the other 8-bit code pages |
| `BinaryValueThisReaderDoesNotRead` | **gap**, deferred by §13 rule 3 — no named consumer |

Two more a file can hold that refuse without a name of their own, both gaps, both
at 0 of 367: `&SCOPE`…`ENDSCOPE` refuses as `NotWhatWasExpected` naming the byte
`&`, and `DATA('…');` with parameters refuses as `NotWhatWasExpected`.

**`rcad-step::Refusal` — an entity graph to a `Model`. 22 variants**, with the
corpus count each reaches. Every one has a test in
`what_the_mapping_refuses_and_by_what_name.rs` that reaches it from bytes, by
starting from a file that imports and cutting one line.

| | limit / gap | corpus |
|---|---|---:|
| `ASurfaceThisKernelDoesNotHold` | **limit of today's kernel**, gap for the roadmap: no variant, no crossing, no sign predicate, no mesher chart | 20 |
| `ACurveThisKernelDoesNotHold` | **limit of today's kernel** — an edge is named by its two surfaces (§24.3), so a B-spline edge has nothing to be named by until a B-spline is a `Surface` | 48 |
| `ACornerIsNotOnTheCurvedFaceTheFileSaysItIsOn` | **a gap for 90 files and a limit for 76** — §43.5. The kernel has the mechanism (§17.2's `Approximation`) and no honest number to tag with: a tag equal to the miss makes the check pass by construction | 158 |
| `ACornerIsNotOnThePlaneOfTheArcItEndsOn` | **gap** — the corner is on the curve; what it is off is the *stored name* of the curve. `d343191` gave `check_edges` the plane's own rounding to spend, `Model::what_writing_a_plane_down_costs`, which let 4147 of 4174 refused corners through. Three files' worth of tail is still refused, at 139, 8189 and 112 800 spacings | 8 |
| `ThisEntityOfTheSolidWasRefused`, mostly a two-step wire | **gap** — a face bounded by an arc and its chord has two steps, and `SolidBuilder::add_wire` and `Invalid::WireTooShort` both want three or one closed edge. It is why the hand-written control is a *quarter*-disc plate and not a half | 75 |
| `ALoopThisReaderDoesNotWalk` | **gap** — a `VERTEX_LOOP` is a `DraftStep::Pole`, which a ball already uses twice; a wire of only a pole has one step | 4 |
| `ASolidWithAVoidInIt` | **gap** — `SolidData::shells` is a list and nothing in the kernel means anything by two | 3 |
| `APlacementThisReaderCannotApply` | **gap, and smaller than it was** — `HalfSpace::transformed` and `EdgeRun::transformed` landed in `0dc984d`, so it is a topo-side change now | 2 |
| `TheFileDeclaresTwoDifferentLengthUnits` | **limit** — one model, one unit (§3.5) | 2 |
| `TheFileHoldsNoSolid` | **limit** — a tessellated export or a drawing | 1 |
| `TheFileHasNoSize` | **limit** — every coordinate is zero | 1 |
| `AnArcThisReaderCouldNotName` | **gap, and nothing has reached it** — needs coordinates that overflow the dot product; tested at `1e200` | 0 |
| `ALengthUnitThisKernelDoesNotHave` | **limit, deliberately** — converting is the silent rescale criterion 6 forbids | 0 |
| `AFaceThatDoesNotSayWhichBoundIsTheOuterOne`, `AnArcThatStartsAndEndsInTheSamePlace`, `TheFileDeclaresNoLengthUnit`, `AnEntityIsNotTheKindTheSchemaWants`, `AnAttributeIsNotWhatTheSchemaWants`, `AnIntegerTooBigToBeTheRealItStandsFor`, `AReferencePointsNowhere`, `TheGeometryIsNotAWorkingShape`, `TheSolidTheFileDescribesIsNotOne` | **limit** | 0 |

**`rcad-step::CannotWrite` — a `Model` to a file. 8 variants.**

| | limit / gap | reached by |
|---|---|---|
| `TheModelHasNoSolid`, `ASolidWithNoShell`, `AnEdgeThatIsNotWalkedTwice` | **limit** | an empty `Model`; the other two unreached |
| `AWireThatStandsStillAtAPole` | **gap** — the same two-step-wire gap from the writing side | `Model::add_ball` |
| `ACurveTheFacesEitherSideOfItDoNotName` | **gap** — an ellipse where a slanted cap meets a bar. `EdgeView::run` is now public and located (`2297970`), so the writer could read the run instead of deriving it, and does not yet | `add_bar_cut_off_by` with a slanted cap; 1 of the 46 corpus round trips |
| `ASolidWithMoreThanOneShell` | **gap** — `SolidBuilder` refuses to build one and `Model::validate` does **not** refuse one off the wire | unreached |
| `ASurfaceThisWriterHasNoEntityFor`, `ALengthUnitThisWriterHasNoSpellingFor` | **gap, unreachable today** — all five surfaces and all six units are spelled | unreached |

**`rcad-geom::Error` — eleven new variants, and one of them is already dead.**

| | limit / gap |
|---|---|
| `SlopeNotPositive`, `HalfAngleIsNotACone`, `ATorusThatRunsThroughItself` | **limit** — the numbers do not name a surface. The third is load-bearing rather than tidy: at `r ≥ R` the quartic **stops being a sign predicate** and certifies the wrong side for a point plainly inside the tube, which §4.1 ranks below a crash. `a_tube_wider_than_its_ring_would_certify_the_wrong_side` writes the sign out by hand |
| `TwoLinesThroughAConesApexAreOneCurveThroughItself` | **limit** — no half-space picks one of two lines that cross, because one holding either holds half the other. Splitting them wants a *point*, which is topology's |
| `ACrossingThatIsAParabola`, `ACrossingThatIsAHyperbola` | **gap** — `EdgeRun` has no arm for one curve that runs on for ever and bends. The shortest fix is on the variants' own docs: name the piece as `PartOfWhereItCrosses` picked by the half-space through the apex square to the axis, which is expressible today, and teach `walk` an arm for a curve that is not a centre and two semi-axes. **Its consumer is a frustum against an ordinary upright block: 96 of 96 refusals** |
| `APlaneTouchingAConeAlongOneOfItsRulings` | **gap, and the smallest one here** — the crossing is one endless straight line and `EdgeRun::Straight` says exactly that. What is missing is one signature change in `how_a_crossing_runs` |
| `ACrossingBetweenAConeAndAnotherCurvedSurface` | **gap** — a degree-four space curve with no seed. The shortest consumer-bearing half is the **coaxial** cases: a ball centred on a cone's axis, a pipe sharing it, a second cone sharing it all cross in circles square to that axis, one sign question each. That is a countersink in a round bar. Since Q2686, a further case is named rather than refused: two cones that share one apex but whose axes lean apart cross in the two rulings `the_rulings_two_cones_on_one_apex_share` picks out. What still refuses sits one level up, at the fuse: the order along the crossing cannot be told where a ruling reaches both rims at once, and every rule order refuses there — `two_cones_on_one_apex_leaning_apart_are_refused_where_each_ruling_meets_both_rims` is the pin |
| `ACrossingBetweenAPlaneAndATorusThatIsAQuartic` | **gap** — Cassini ovals, figures of eight, Villarceau pairs. The marcher already carries a torus's distance and gradient; what is missing is a seed per loop, and whether a plane's quartic falls into one loop or two is an eighth-degree discriminant nobody has derived. **Refuses every bend against a flat-sided tool** |
| `ACrossingBetweenATorusAndAnotherCurvedSurface` | **gap** — degree up to eight. **Refuses every bend against a curved tool** |
| `WhereALineMeetsARingNeedsTheLineExactly` | **gap** — a quartic's roots are found by evaluating its sign along the line, and a coefficient that is a box has no sign to evaluate near a root. The fix is named: bisect on interval signs and hand back the stretches where the sign is undecided |
| `CannotFollowACrossingWhereItsSurfacesAreTangent` | **limit** — `∇S × ∇T` is zero at every point of a plane resting on a torus's tube, so the direction does not exist. A convention picked out of thin air is not the crossing's direction (§17.4) |
| `WhereALineMeetsATorusIsAQuartic` | **neither: it is dead.** `800fd85` solved it, and the variant is `pub` and is **constructed nowhere in `crates/*/src`.** Kept because removing a public variant is a breaking change; it should be deprecated or removed at the next break |

**Amended 2026-09-03, by the unit called *the six arms*. The first four rows
above are closed, and one of them was a limit that was really a gap.**

- `ACrossingThatIsAParabola` and `ACrossingThatIsAHyperbola` — **deleted.** Both
  are named and drawn. The shortest fix this table named turned out to be the
  fix that landed: `PartOfWhereItCrosses` for naming, and `walk` taught arms for
  curves that are not a centre and two semi-axes.
- `APlaneTouchingAConeAlongOneOfItsRulings` — **deleted**, answered as
  `EdgeRun::Straight` exactly as this row predicted. What this row got wrong was
  the price: *"one signature change in `how_a_crossing_runs`"* is wrong about the
  word, since it already takes the run, and §46.8 item 1 records the correction
  being mis-priced in its turn.
- `TwoLinesThroughAConesApexAreOneCurveThroughItself` — **deleted, and it was
  never a limit.** The row's argument — *"a half-space holding either holds half
  the other"* — has both halves true and the conclusion false: a piece is picked
  by **every** half-space on it, so a pair of opposite half-spaces picks what
  lies in one flat surface, and the flat surface through the apex holding one
  line meets the other at the apex alone. Two straight runs told apart by
  pickers, which `two_lines_told_apart` was already doing for another pair. **A
  limit is a claim that the answer does not exist; this one existed and was one
  file away.**
- `CannotFollowACrossingWhereItsSurfacesAreTangent` stands, and the unit's fuse
  lane hit it again from the other side: the single ruling of a plane resting
  along a cone's own side still refuses, because `∇S × ∇Q` is exactly nothing
  there. Same limit, one more surface pair.

**Two `rcad-geom` refusals M5 did not close, and should say so out loud:**

- `CannotWorkOutHowBigACrossingIs` still refuses **a round bar of radius
  `1e-200`**, which `Model::add_round_bar` accepts and the public builder writes.
  A **gap**, and its own doc says so. `Vector::length`'s rescale does not reach
  it: the quantities that underflow — `squared_normal`, `squared_along`,
  `and.length_squared()` — are squares in their own right and not lengths. It
  wants the same treatment one level up: work the conic out at a power-of-two
  scale and scale the size back. `crates/rcad-topo/tests/meshing_a_curved_solid.rs:737`
  is the recorded case.
- ~~`Point::distance_to` still sums squares.~~ **This gate wrote that and it is
  false**, checked against the tree after the merge: `distance_to` is one line,
  `(other - *self).length()`, and `Vector::length` took the power-of-two rescale
  in the same lane. What still squares is `length_squared` and
  `distance_squared_to`, deliberately and documented, because the square of a
  separation of `1e-200` is not an `f64` at all — that one is a **limit**, not
  the gap this row claimed. The rest of the row described a model at very small
  *absolute* scale — coordinates at `1e-200`, features at `1e-210` — which has a
  dynamic range of only `1e10` and passes both range checks; that shape is real
  and is covered by the `CannotWorkOutHowBigACrossingIs` row above. **Gap**, one
  line, and nobody took it.

**`rcad-topo::Error` — thirteen new builder refusals.** Eleven are limits. The two
**gaps** are `ADraftWireIsTooShort` — a face bounded by two arcs, which is a lune
on a sphere, a D-shaped pin or a rounded slot end, inherited from
`Invalid::WireTooShort` and worth 75 of 367 corpus files — and
`ASolidBuiltFaceByFaceHasOneShell`, which is the **one builder refusal serde walks
past**: `the_sweep_does_not_refuse_a_solid_with_two_shells` puts two cubes' shells
under one solid and the certified sweep says `Ok`.

**One refusal M5 added to the fuse, and it is a limit.**
`Invalid::DynamicRangeTooWide` out of `Assembly::check_dynamic_range` — the
answer's own shortest edge is too small to resolve beside its own coordinates.
Demonstrated by
`a_certified_fuse_refuses_a_sliver_too_thin_to_resolve_rather_than_writing_it`.
**It is not tolerant-proof**: `Policy::tolerant` warns on a wide range rather than
failing, so a tolerant fuse of the same pair writes a model the certified sweep
then refuses. Verified at this gate on the pinned corpus file (§43.5).

### 43.7 Amendments to earlier sections

House convention: the amendments live here, not as edits to the sections they
amend.

> **§20.1, amended.** §20.1 settled that `SolidView::measure` **panics** on a
> solid enclosing a non-positive amount of space: *"finite and not positive … a
> corrupt solid, and a crash beats a corruption. Still panics."* That was written
> about a solid **this crate built**, where an inverted one is a bug in the
> kernel. `Deserialize for Model` is a second route in, and it makes the same
> shape **caller input**, which CLAUDE.md says is an `Err`. Verified at this gate:
> a cube written out with every face use reversed and read back gave `validate` →
> `Err(Invalid(VolumeIsNotPositive { volume: -8.0 }))` and `measure` → a panic at
> `measure.rs:196`, from the same bytes.
>
> **`measure` now returns `Err(Invalid::VolumeIsNotPositive { volume })` — the
> same variant with the same number in it that `validate` gives** — so the two
> answers are comparable rather than merely both being failures (`72c7eba`).
> `measuring_a_solid_read_back_inside_out_is_refused_rather_than_crashing`
> replaces the `#[should_panic]` test that held §20.1 in place, and
> `measuring_a_ball_read_back_inside_out_...` covers the second sum: a ball is the
> shape only the volume can catch, because its one seam runs pole to pole and
> `AClosedEdgeIsWalkedAgainstItsSurface` has nothing to say about it. **The panic
> §20.1 defends is still right for a solid the kernel built; what changed is that
> a solid off the wire is not one.**

> **§24.6's list of what a new `Surface` variant touches, and §42.16's correction
> to it, both amended again.** §24.6 named five places. §42.4 corrected that to 22
> files in `src` and 38 with tests, by counting files that name one of the three
> variants — and both counts answer a question nobody asked. The number that
> matters is **how many places a new kind is caught**, and it is not the same
> number:
>
> | | files naming a variant | places the compiler or a lint catches a sixth kind |
> |---|---:|---|
> | at §42.4, three variants | 22 in `src`, 38 with tests | — |
> | now, five variants | **25** in `src`, **54** with tests | `rcad-geom` **15** compile errors across 7 files, 31 with `--all-targets`; everywhere else **0** compile errors and **26** new clippy warnings; **7 hand-kept pair lists caught by neither** |
>
> The cone lane touched 21 `src` files and the torus lane 20, and only seven and
> ten of those respectively were compile errors. **The right sentence for §39.7
> item 15's successors is: a new `Surface` kind is a compile error in `rcad-geom`
> and a silent wrong answer everywhere else, unless somebody looks at the clippy
> delta.** §43.8 item 7 is the fix.

> **§39.4 gains four rows it never had.** Grepping `docs/decisions.md` for
> `ACrossingBetweenAPlaneAndATorusThatIsAQuartic`,
> `ACrossingBetweenATorusAndAnotherCurvedSurface`,
> `CannotFollowACrossingWhereItsSurfacesAreTangent` and
> `WhereALineMeetsARingNeedsTheLineExactly` returns nothing. The catalogue is not
> drifting — it never recorded them — but **two of the four now answer for every
> torus fuse this kernel can be asked for**, which makes them the most
> load-bearing rows in it. All four are in §43.6; three are gaps and the tangent
> one is a limit, and all four carry their family of fixes on the variant's own
> doc.
>
> **And `ACrossingThisCannotBeOrderedAlong`'s three existing entries — lines 8805,
> 9811 and 10644 — describe it as the ball-and-pipe quartic. That is now the only
> thing it means.** Before `769dada` it also swallowed every plane-and-torus
> crossing, falsely, and no row said so.

> **§42.10's section reservation is void**, and §42.12 criterion 4's example is
> stale: the criterion said *"a file holding a cone is refused by the name the
> file gives it"*, and M5 built the cone. It is discharged by a B-spline patch
> instead, which is the honest reading of what it was for.

> **§42.11's opening sentence is a deliverable and it shipped, wider than it was
> written.** `crates/rcad-step/src/lib.rs` opens with it: *"This reader maps
> solids bounded by planes, spheres, cylinders, cones and tori, whose edges run
> along straight lines, circles and ellipses. Every other surface and curve kind
> is refused by the name the file gives it. A model that imports is internally
> sound and is not checked for interference with anything else in the file."*
> §42.11's row saying a cone or a torus refuses the file immediately is void; the
> rows for a B-spline surface, a surface of revolution and a surface of linear
> extrusion stand, and they are 20 files of the corpus.

> **§42.9's healing fork is settled as option A**, which is what the user chose
> and what roadmap §13's risk register already said. Nothing is healed: no vertex
> is merged, no gap is sewn, no loop is reordered, no missing value is defaulted,
> and the declared uncertainty buys nothing anywhere. Three places take a decision
> rather than refuse, and all three are counted and published — `1` read as `1.`
> where the schema says `REAL` (0 of 46 corpus imports needed it), a `MAPPED_ITEM`
> whose two ends are not both placements is not followed, and the outer bound of a
> multi-`FACE_BOUND` face is read off the winding in plain `f64` as a **hint that
> only chooses an order**, with `Model::validate` asking the same question with
> the certified predicate moments later. **Option C is designed for M6** and
> §43.8 item 12 carries it.

> **§42.14's prediction, scored.** The gate predicted the money would go to *the
> public builder's error surface*, and named parsing and the AP mapping as the
> things it would not go to. **Wrong on the first and wrong on the third.** The
> builder came in inside one wave, with thirteen new `Error` variants and 29
> tests, and the fifty-eight `# Panics` clauses' worth of audit it feared did not
> materialise, because the lane chose a design — a description checked as a whole
> and committed at the end — that turned the question into one function. The money
> went to **two new surface kinds** (`rcad-geom` +119 tests, two lanes, the
> largest single item in the milestone) and to **the panic sweep**, which was not
> on the list at all and which cost six lanes and closed eleven reachable crashes.
> §12.1's lesson — the money goes to code that was already written and looked
> finished — **held**, and §42.14 predicted it would not.

### 43.8 What M5 hands to M6

§39.7's shape. Each item has its evidence and its named consumer.

1. **NIST is at 0 of 44, and it is two things.** 16 of the 44 are stopped by a
   **wire of two steps** — a face bounded by an arc and its chord — and 15 by
   **`B_SPLINE_CURVE_WITH_KNOTS`**. Neither is the reader's. The two-step wire is
   `rcad-topo`'s: `SolidBuilder::add_wire` and `Invalid::WireTooShort` both want
   three steps or one closed edge, `winding_of` reads a wire's winding off its
   corner polygon alone so a face bounded by two half-circles sweeps exactly zero
   and cannot be told from a hole, and the smallest file that reaches it is in the
   tree (`a_plate_bounded_by_an_arc_and_its_chord`). **Consumer: 75 of 367 corpus
   files, and it is the largest single unblocked item.** The B-spline half is
   `BSplineCurve`/`BSplineSurface` reaching an arena, which §42.13 already deferred
   and which wants the free-form marcher §24.1 sent away. **Consumer: 48 files by
   curve and 20 by surface.**
2. **`KnotVector::span_containing` and six sibling getters panic on caller
   input.** `KnotVector::new` is public, `span_containing` is public, and
   `kv.span_containing(f64::NAN)` asserts. The family is seven, not five:
   `span_containing`, plus `control_point` / `weight` /
   `homogeneous_control_point` on each of `BSplineCurve` and `BSplineSurface`.
   Each has a precise `# Panics` clause and a public predicate the caller can ask
   first (`contains`, `control_point_count`), which is the contract `slice[i]` has
   next to `len()` — so this is **not a gap**, and changing one and not the others
   is worse than either. Changing all seven is a public-API break across **110
   call sites in the workspace, 47 of them in `src`**. A milestone-owner's
   decision, not a lane's.
3. **`Policy::tolerant` and `Measures::wobble_of` panic on a bad numeric
   parameter**, where `Scale::new` in `rcad-geom` validates the same shape of
   input and returns `Result`. CLAUDE.md is direct: *a bad parameter is data, not
   a bug*. `Policy` is not `Deserialize`, so this is not the model-corruption
   class — but `Policy::tolerant` is facade-exported and directly callable, and its
   `scale` is exactly the caller-supplied number the rule is about. **47 call
   sites across 15 files, in four crates.** Either accept the panic as a
   documented, argued design choice and say so in the doc the way `Record::new`
   does, or make it a cross-lane fix. **Consumer: any caller who computes a
   tolerance rather than typing one.**
4. **`CannotWorkOutHowBigACrossingIs` still refuses a round bar of radius
   `1e-200`**, which the public builder accepts. §43.6 has the diagnosis and the
   fix — the same power-of-two rescale `Vector::length` just took, one level up,
   which `Vector::length` itself does not reach. **Gap. Consumer:
   `crates/rcad-topo/tests/meshing_a_curved_solid.rs:737`, already in the tree.**
   Its neighbour is **not** `Point::distance_to`, which this gate said still
   squares and which does not — see §43.6's struck row.
5. **`arrange.rs:203` is unsettled, and it is the least-covered assert in the
   fuse** — 4 hits in a whole suite run, against 1 394 856 for the busiest. Its
   doc says *"Both come from the same pair of surfaces, so both pick the same
   axis"*, and `check_it_does_not_cross_itself` (`cuts.rs:175-177`) calls
   `stretches_of_a_line_inside` twice with **the argument order swapped**, while
   the axis is chosen from the *ordered* pair. Three of the four legs were checked
   and are order-symmetric; the fourth was not — the plane-and-pipe branch takes
   gradients at **two different sampled points**, one on each face, and nobody
   demonstrated that both lie on the crossing. **§16.10's shape exactly: a claim
   that names a shorter fact than it needs.** Small, and worth one focused lane.
6. **The fuse's 46 `expect`s and `unwrap`s are unjudged.** All **19** of its
   production `assert*` sites were instrumented and reached by the existing
   degenerate corpora — 1 394 856 hits at the busiest, 4 at the thinnest, and
   **none fired** — and the two that looked like the counting species were settled
   by construction. The other 46 got nothing. An `.expect()` on an arena lookup is
   a different risk from an `assert!` about a shape, and §43.3 is eleven reasons
   to take the difference seriously.
7. **`rcad-geom` should own the surface-pair question.** The only place a sixth
   `Surface` kind can be a compile error is the crate that declares it, and
   `rcad_geom::walk`'s own claim to be a compile error is **true** where
   `rcad-topo`'s three identical claims were false and are now corrected. The three
   lists in `rcad-topo`'s fuse — *does this pair cross in a conic*, *is this a
   plane along a pipe's axis*, *does this crossing run straight* — are geometry
   questions asked in the wrong crate; if `rcad-geom` answered them,
   exhaustiveness would bite. **Consumer: the seven drifts in §43.4, five of which
   cost something and one of which cost every capped bend its reason.**
8. **`composite_wildcards.awk` counts sites and not uncovered variants.** The
   `justfile`'s own comment says the number moves for a new *variant* — true for
   clippy's half, **false for the composite half**, demonstrated at this gate: the
   three `rcad-topo` pair lists count one apiece at five kinds and one apiece at
   six. That is the hole all seven drifts fell through, and it is still open. The
   fix lives in `crates/rcad-step/tools/`. Note the honest alternative first: the
   mechanism that would do this properly is `non_exhaustive_omitted_patterns`,
   which is **unstable on the pinned 1.96**, demonstrated rather than assumed.
9. **`StepFile` hands back four types the facade cannot name.**
   `header() -> &Header`, `instances() -> impl Iterator<Item = &Instance>`,
   `instance() -> Option<&Instance>` and
   `references_that_point_nowhere() -> &[BrokenReference]` are all reachable
   through `rcad::StepFile`, whose `Header`, `Instance`, `Record`, `Value` and
   `BrokenReference` are deliberately not re-exported. A caller can iterate a
   `&[BrokenReference]` and read its public fields without ever naming the type, so
   this compiles and works — but it is a half surface: **reachable structurally,
   unnameable in a signature.** Either re-export the five or narrow the four
   methods. §13 rule 3 says the raw graph has no facade-level consumer today, which
   argues for narrowing.
10. **A `Model` with one face used twice deserializes and then crashes the fuse.**
    `check_shape_is_possible` checks face wires, wire closure and coedge counts,
    and does **not** count face uses; `Model::validate` does
    (`Invalid::UsedMoreThanOnce`), and `Model::fuse`'s doc says it panics on an
    input `validate` rejects. So `bake.rs:386` is a documented panic on caller data
    that nothing forces a caller through `validate` to avoid. §17.4 settled the
    contract; §43.3 is eleven reasons to look at it again.
11. **`SolidBuilder::build` takes no placement, and both blockers are gone.**
    `HalfSpace::transformed` and `EdgeRun::transformed` exist (`0dc984d`) and
    `EdgeView::run` is public and located (`2297970`). What is left is a design
    decision: **place it** — write the `ShellUse` with the caller's isometry, ten
    lines, exact by construction — or **bake it**, moving every point and run on
    the way in, which is what §42.8 chose and what `builder.rs` commits to in
    prose. Baking costs something measurable: a turned picker loses exact
    squareness in **17 228 of 17 472 turns**, so it wants a published bound and a
    §15.6 search behind it. **Consumer: criterion 11, and every occurrence an
    importer places.**
12. **Healing, deferred as option A with option C designed for M6.** §42.9's
    objection to C is weaker than it reads, and this milestone made that concrete:
    *"these two vertices are one point"* is **one decision, one record, one
    `Cause`, one measured distance**, and 90 of the 166 files refused for a corner
    off a surface are refused for a miss **no writer could have avoided** — the
    file wrote the nearest `f64` there is. The kernel already has the mechanism
    (§17.2's `Approximation`, which `check_curved_face` consults *before*
    refusing) and no honest number to put in it: **a tag equal to the miss makes
    the check pass by construction**, which is a check that has stopped asking
    anything. What C needs is a decision about what an imported corner's slop *is*,
    plus a new `Cause` variant and a `Measures` arm — the shape §15.3 and §17.2
    already used twice. **Consumer: 166 of 367 files, and it is the single largest
    lever on the import rate.**

**Two smaller ones, named so nobody re-derives them.** `rcad-topo`'s
`where_the_ray_meets` is now a **second answer** to a question
`rcad_geom::where_a_line_meets_a_surface` answers (§1.5's shape) and should be
deleted in favour of it, which also closes
`Containment::CannotFireARayAtAFacesSurface` for a corner on the lattice. And
`Error::WhereALineMeetsATorusIsAQuartic` is public and constructed nowhere.

### 43.9 The claim audit at this gate, and the six numbers it broke

Run per CLAUDE.md and §16.10, over the twenty-eight lane reports and over the
brief this gate was given. **Six load-bearing numbers did not survive
re-derivation.** They are here because a gate that prints only the claims that
held has not shown its method.

1. **"4321 false zeros in 200 000 pairs one ulp apart" does not reproduce.** The
   claim is about `where_a_plane_cuts_a_torus` asking `n × w == 0` in plain `f64`.
   Re-derived at this gate through the same pipeline
   (`reduced_by_largest_component`, then `scaled_to_length_one_from_anywhere`),
   with the exact cross product taken in rationals: **2071 in 200 000 (1.04%)**
   for one component nudged one ulp toward `+∞`, **2035 (1.02%)** for one
   component nudged in a random direction, and **462 (0.23%)** for all three
   nudged. None is 4321 and none is 2.2%. **The counterexample itself reproduces
   exactly**: `(1, 2, 6)` with `z` nudged up one ulp gives an `f64` cross product
   of `(0, 0, 0)` and a true first component of `5.2016311887317167e-17`, which is
   the figure the lane reported and which
   `a_cross_product_of_unit_triples_can_round_to_zero` pins in the tree. **The
   claim that survives is the counterexample and the gap it names, not a rate**,
   and the rate should not be repeated: it is a property of an unrecorded draw and
   nothing in the tree reproduces it. The doc comment `d84947d` left is right
   about the thing that matters — what saves that arm is the whole unit of gap
   between the two families `intersect` certified, not the arithmetic.
2. **The two `E0004` probes did not disagree about the tree.** "31 across seven
   files" and "15 across eight" are the same measurement with and without
   `--all-targets`. At this gate `cargo check -p rcad-geom` gives **15** and
   `--all-targets` gives **31**, and both are across **seven** files —
   `surface.rs` 6/12, `three_surfaces.rs` 3/6, `march.rs` 2/4, `walk.rs` 1/3,
   `moved_corner.rs` 1/2, `intersect.rs` 1/2, `along_a_crossing.rs` 1/2. Every
   file doubles under `--all-targets` because the lib is compiled twice; `walk.rs`
   gains a third from a site inside its own test module. **"Eight files" is wrong
   either way.** Applying rustc's own suggestions settles at **18 arms**.
3. **The test count at the start of the run was 1141, not 1113.** §42.16 recorded
   1141 and the merge base measures 1141. 1112 is M4's close.
4. **`Policy::tolerant` is 47 call sites across 15 files, not 46 across 14.** The
   tree grew a file — `a_fuse_that_builds_a_sliver.rs` — after the lane measured.
5. **`span_containing`'s family is seven getters, not five, and the API break is
   110 call sites, not ten.** `span_containing` alone has 16, of which 9 are
   outside its own unit tests.
6. **`StepFile`'s structural leak is four methods, not two.** `instances` and
   `instance` hand back `&Instance` for the same reason `header` hands back
   `&Header`.

**Two further findings of this gate's own, neither of them a number:**

- **Criterion 10's rate is published and not asserted.**
  `how_often_a_real_file_imports` prints 46/323 and 0/44 and asserts nothing about
  either. §30.3's warning — *a criterion that cannot fail is not a criterion* —
  applies. And the whole measurement is behind `RCAD_STEP_CORPUS`, so `just
  default` is green at 1427 without it. What *is* asserted is the control (88, by
  `assert_eq!`, one refusal is one failure) and the corner-miss distribution —
  both halves must be present and the tail must exceed a thousand spacings — which
  is more than nothing, and is not the rate.
- **The fuse's crash-site count is 65, not 66**: 19 `assert*` and 46
  `expect`/`unwrap`/`panic!`. The higher figure counted `cfg!(debug_assertions)`
  and a function called `assert_approximation_did_not_grow`, neither of which is
  an assertion.

**Checked hardest and sound.** Every headline this milestone rests on reproduced
exactly at this gate, from the tree and not from a report: the three import rates
with their round-trip and parse figures; the five bend families at 0 of 96 with
their four controls at 24, 96, 96 and 0, and the refusal names; all three
`Vector::length` figures; the `rcad-topo` clippy delta 30 → 50 with 20 new and 0
gone, and the twenty new sites being the same twenty; the composite scan at 8 and
unmoved by a sixth kind; and the pinned fuse crash, replayed byte for byte — two
10 mm boxes, 51 edges, one sliver `3.80218313259032e-211` long where the old
arithmetic read `0.0`.

**Was it worth running?** Yes, and for the reason §42.15 gave: the finding that
mattered most was the one this gate could not reproduce, and it was a number
several people had already repeated.

### 43.10 Is this a kernel?

§39.9 asked this at M4 and answered that a kernel whose list is built and whose
coverage is not is a demonstration and not a kernel. **M5's answer is different
in kind, because for the first time the question has an outside measurement
against it, and the measurement is 14.2%.**

**What it is.** A stranger's file becomes a `Model` or is refused by a name that
says which entity and why, and there are 367 real files behind that sentence
rather than an argument. 88 of 88 files the kernel wrote itself come back. 45 of
46 imported files go out and come back the same solid, bit for bit, with their
`Scale` unchanged. 2 366 598 entities agree with an independent parser. A solid
can be built out of faces from outside the crate, and every misuse of that door
is a named `Err` — 102 400 drawn scripts and zero panics. Four fuzz targets watch
the four doors a stranger's bytes reach, and three of them found something real.
None of that existed six weeks ago.

**What it is not.** A CAD kernel is judged on whether it can hold the file a user
hands it, and **85.8% of a real, mainstream, ordinary corpus of small mechanical
parts does not come in.** Not exotic parts: KiCad connector housings. The two
biggest reasons are not exotic either — a corner a hair off a cylinder, and a face
bounded by an arc and its chord. **A D-shaped pin is not an advanced feature.**
And the NIST corpus, which is what the industry uses to certify that a system
reads STEP correctly, is at **0 of 44**.

**And the geometry underneath it has a shape that is now visible.** M5 added two
surface kinds and both arrive able to be *built, validated, measured and meshed*
and unable to be *fused* with much. A frustum against an ordinary upright block is
96 of 96 refusals. A capped bend of pipe fuses with **nothing at all** — not with
a slab, not with a wedge, not with a bar, not with a ball, not with a bit-for-bit
copy of itself. Every one of those refusals is honest, named, and carries its
family of fixes; not one is a wrong answer. But a kernel that can build a shape
and cannot cut it is a modeller you can put things into and not take things out
of.

**The honest sentence: M5 turned a demonstration into an importer, and the
importer's own numbers say the kernel behind it is the bottleneck and not the
reader.** §42.4 predicted exactly that — *"a reader that reads eleven surface
kinds hands ten of them to a kernel that holds three"* — and then M5 took the
kernel from three to five and the rate went from a *ceiling* of 9.1% and 73.1% to
a *measured* 0% and 14.2%. **The ceiling moved and the floor did not follow**, and
the distance between them is §43.6's table: 166 files waiting on what an imported
corner's slop is, 75 on a wire of two steps, 68 on B-splines. Those are three
decisions and two lanes, not a milestone.

So: **not yet, and for the first time the answer has a number and a shortest
path.** M4's answer was "the list is built and the coverage is not". M5's is "the
door is open, and three known things stand in it". That is a better kind of no.

**The prediction for M6, recorded before the work and to be scored at its closing
gate** (§30.2's cheaper half). §12.1's lesson held at M5 against §42.14's guess,
so it is bet on again: **the money goes to the corner-slop decision**, not to
B-splines and not to the two-step wire. Not because the arithmetic is hard — it is
one `Cause` and one `Measures` arm — but because it is the first decision this
project has had to make about what it is willing to *believe* rather than what it
can *prove*, and §42.9's fork has been open since the milestone opened. The second
guess is criterion 8, unstarted for a second milestone running. The third, and the
one this gate would most like to be wrong about, is that the fuse's 46 unjudged
`expect`s hold one more of §43.3's eleven.

