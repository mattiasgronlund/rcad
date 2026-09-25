# Queue

Every open item in rcad, one row each, in five tiers. `docs/decisions.md` §230
is the argument for the shape and the grammar below. The dispatcher edits this
file — to `lane:` when it dispatches a row, to `done:` when it merges one — and
a lane never writes it, so no two lanes conflict over it.

A row is one line inside a fenced block under its tier's heading, as `<id>
<tier> <state> <model> <write set> <what>`, the fields split by spaces. The
shape is only described here, never fenced: `just check-queue` reads every
fenced line as a row, so a fenced template would be a row nobody ranked.

- **id** — `T<n>`, `D<n>` or `Q<n>`, unique in the file. `T` and `D` numbers are
  carried over from the retired `docs/test-plan.md`.
- **tier** — `1`, `2`, `2.5`, `3` or `4` (§230.1).
- **state** — `todo`; `lane:<branch>`; `after:<id>[,<id>]`; `done:<sha>`;
  `dropped:§N`; or `parked:§N`.
- **model** — `opus` or `sonnet`; `-` only on a closed row that never recorded one.
- **write set** — comma-separated paths, directories (ending `/`), or
  `path#item`; `-` means the lane writes no tracked file but a new section of
  its own.
- **what** — ends in at least one `§N`/`§N.M`; on a `done` row the last one is
  the section that records the result.

A row is **ready** when its state is `todo`, or `after:` names only `done` rows.

## Tier 1

```
T7   1    after:Q66   sonnet  justfile#fuzz-reader,justfile#fuzz-target,justfile#check-fuzz-overnight-selftest   Seed the reader target from the real STEP files under RCAD_STEP_CORPUS, as a read-only corpus directory fuzz-target passes libFuzzer; its corpus is all fuzzer-grown. Built on lane-t7-reader-seeds-from-step-corpus (8689e498), not gated; merge it once Q66 says how, with Q66's changes. §226.1
Q66  1    todo    sonnet  -   Measure lane-t7-reader-seeds-from-step-corpus (8689e498) before it merges: the reader's fork-mode startup and execs/s with and without the STEP corpus (520 files, 375 MiB, four zips up to 60 MB; max_len rises from about 5 KB to 1 MiB), alternated to cancel load, and whether libFuzzer loads all 520 and writes nothing into it. Say whether to leave the zips out or pin -max_len. Runs a fuzz binary, so off 21:50-06:00Z. §226.1
Q61  1    todo    sonnet  -   Test §239.6's untested reading that 09-23's 4.6 M runs against 09-22's 36.7 M is the night starting from set C, 923 inputs at 17 ms each: run the 09-23 binary a fixed time from its corpus with and without C and count runs. Runs a fuzz binary, so off 21:50-06:00Z. §239.6 §239.8
Q53  1    after:T7    sonnet  justfile#fuzz-overnight,justfile#check-fuzz-overnight-selftest   fuzz-overnight writes each target's corpus file names into the night's log directory before the targets start, so an input's arrival night is the first list naming it and not its mtime, which a later night's merge rewrites. §240.3 §240.7
Q50  1    after:Q49   sonnet  -   Read the first night after Q49 lands: set its count of runs that could not be written against libFuzzer's own run count, to say whether merge_out's failures are the §236 undercount, and why a slow night would fail more often. §236.6
Q65  1    lane:lane-q65-mutator-in-replay   sonnet  crates/rcad-step/tests/fuzzing/writer.rs,fuzz/fuzz_targets/no_model_makes_the_writer_panic.rs   Move the writer target's still_a_model mutator into tests/fuzzing/writer.rs, so the replay suite can check that every mutation of a kept input still decodes to a model. §129.5
Q71  1    todo    sonnet  -   The old writer target's "came back the same" count swung from 1526 to 37 between two 60 s runs on the same seeds, with 163 against 781 panics; find why two runs of one target on one corpus differ that much. Runs a fuzz binary, so off 21:50-06:00Z. §241.3
```

## Tier 2

```
Q54  2    lane:lane-q54-torus-face-between-two-planes    opus    crates/rcad-topo/src/classify.rs,crates/rcad-topo/src/holds.rs,crates/rcad-topo/tests/building_a_solid_out_of_faces.rs,crates/rcad-topo/tests/how_often_a_fuse_with_a_torus_refuses.rs   Both bend pins stop at the ray from (4, 0, 0) in a bend's end cap: the other bend's torus face answers CannotFollowAFacesTrimming. Trace which holds.rs error becomes it, and let the classifier follow a torus face's trimming. §232.4 §232.5
Q55  2    todo    opus    crates/rcad-topo/src/holds.rs#what_a_round_wire_encloses   what_a_round_wire_encloses takes a torus wire's inside as one disk, where a plane through the axis leaves two; ask the meridian circle's half-space as select.rs's the_piece_holds now does. §232.2 §232.5
Q56  2    after:Q54,Q55   opus    crates/rcad-topo/src/fuse/select.rs#a_face_of_the_input_holds   A flat piece lying on an input disk cannot be settled ON: a_face_of_the_input_holds winds a flat input face's corners and a disk has one; holds.rs's round-wire rule would answer once Q55 fixes it. §232.5
Q5   2    lane:lane-q5-swallowed-pointed-cone-rerun   opus    crates/rcad-topo/src/fuse/select.rs,crates/rcad-topo/tests/   Finish lane-q5-swallowed-pointed-cone-rerun (2d1d61ea: 184962b9 merged onto the tip, gate red). Merge Q80's select.rs first; then check whether 184962b9's own tests fail without its change, and drop it if they do not. If they do, run rcad-topo with --no-fail-fast and re-derive every pin it moves — a_block_leaning_through_a_frustum_is_cut_along_the_cone_off_the_frustums_own_face and at least five more — checking each new answer is right, not just new. §229.7
Q72  2    lane:lane-q72-meets-ask-the-policy    opus    crates/rcad-topo/src/fuse/arrange.rs#place_the_meets   place_the_meets drops a meet whose box overlaps a node already on both its crossings without asking the policy, so two points 8.9e-16 apart become one node, where §17.4 leaves overlapping boxes to the policy to merge or refuse. Ask the policy, and find whether any fuse carries the drop to a wrong answer; §244.2's block edge grazing a ball at cx = 16 is the witness, and there a later orient3d refuses. §244.3
Q79  2    todo    opus    crates/rcad-mesh/src/triangulate.rs,crates/rcad-mesh/tests/cutting_a_patch_into_triangles.rs,crates/rcad-topo/src/mesh.rs,crates/rcad-step/tests/a_solid_written_out_and_read_back_is_the_same_solid.rs,fuzz/corpus/no_model_makes_the_writer_panic/   triangulate hands spade a corner below 2^-142 and spade refuses it TooSmall, so a bar moved by 2^-1007 is refused on read-back: flush each scaled corner through spade::mitigate_underflow before inserting it (tried and reverted in §246.5, every case came back bit for bit). Show a corner flushed onto a side's line cannot turn a triangle over or leave two patches drawing an edge differently without finish refusing; pin the disc, the moved bar's round trip, and check in the changed seed (offset 506 = 0x01). §246.5 §246.7
Q100 2    todo    opus    -   A round bar moved by 2^-1007 fused with a cube refuses where the unmoved bar fuses: "where a plane and a pipe cross" (interval) for a cube far away, "where three surfaces meet" (exact, enclosure inf) for one through the wall. Find why a move that small reaches either predicate. §246.6
Q75  2    lane:lane-q75-arc-edge-names-its-circle    opus    crates/rcad-topo/src/fuse/assemble.rs,crates/rcad-topo/tests/fusing_two_bars_side_by_side.rs   Hole B: Edges::of names an open arc by its two corners and the one-bit which-arc flag, never by its circle, so arcs of two different circles between the same two corners become one edge with four faces. Put the crossing's curved surface, named either way round, into the edge name wherever the flag is set, and not the plane (which_curve_it_is's doc says what the plane cost); check two coincident walls spelled two ways (§231.3) still share one edge. §245.4 §245.5
Q76  2    after:Q75   opus    crates/rcad-topo/src/fuse/assemble.rs,crates/rcad-topo/tests/fusing_two_bars_side_by_side.rs   Hole A: an input vertex where every kept side ending there runs along one crossing is not a corner of the answer, so its two sides are read as one, in Assembly::from_the_pieces beside Corners::with_each_closed_curve_started_once. Expect the one pair that answers today, the intersection with here as the argument, to lose its corners at (5, 0, 0) and (5, 0, 10). §245.3 §245.5
Q77  2    after:Q75,Q76   opus    crates/rcad-topo/tests/fusing_two_bars_side_by_side.rs   Rewrite two_overlapping_parallel_bars_stop_in_select_whichever_way_round as six fuses that answer, checking each answer is right, and take off its #[ignore]; ignored_tests drops by one. §245.7
Q78  2    after:Q76   opus    crates/rcad-topo/tests/fusing_two_bars_side_by_side.rs   Fuse the answer of the two bars' intersection, with here as the argument, once more, and see whether a corner with two curved edges survives it. §245.5 §245.7
Q69  2    lane:lane-q69-turned-bar-bits    opus    -   A round bar of radius 3, 5 tall, turned an eighth of a turn about z, comes back from its own STEP file with area 150.76324910914616 → 150.7632491091462 and its centroid about 1e-15 off; moved by (3, −5, 7) it fails the same way. Find whether the writer, the reader or measure in a moved frame loses the bits (a gap) or the bit-for-bit comparison asks more than decimal text carries (a limit, which needs a demonstration under §16.10). §241.5
Q44  2    todo    opus    -   Find whether any fuse refuses because the joiner leaves two spellings of one curved surface unjoined; if one does, join them by picking one spelling by bits and turning round every closed side read off the other, §231.3's second pass once more. §231.5
Q28  2    todo    opus    -   Find why a solid inside a solid crashes the fuse. §191.5
Q29  2    todo    opus    -   Find why a turned sound union fails validate, or whether the move or the check is wrong; shares its refusal with §124. §191.4
Q30  2    todo    opus    -   Find why a clean leaning union is called not a solid. §191.4
Q31  2    todo    opus    -   Find why line tangency between a cylinder and a plane or cylinder answers nothing, true of every fillet. §191.5
Q16  2    todo    opus    -   Demonstrate a real Model::fuse route to a closed rim between two faces of one sphere, or correct §219.7's claim that one exists, since §220.1 showed add_ball's route does not reach it. §220.1
Q17  2    todo    opus    -   Reproduce §220.1's two unconfirmed findings: a ball's polar cap with its corner off the pole refusing CornersCoincide, and a half-ball with a hole through the north pole refusing four different ways depending on coedge orientation. §220.1
Q41  2    todo    opus    crates/rcad-topo/src/validate.rs#check_this_solids_closed_edges_are_walked_the_way_their_surfaces_wind   §229.4's cone written twice, one ulp apart: the closed-edge check takes the face across the edge as the partner instead of the third surface the run names, so the sign it reads is wrong. §229.4
Q37  2    todo    opus    -   §229.4's torus written twice: the mesher answers CornersCoincide on a band that goes right round the ring, and the kernel today only builds a bend torus face. §229.4
Q14  2    todo    opus    crates/rcad-topo/src/mesh.rs   mesh.rs's own narrower copy of which-two-surfaces picks a different pair for a fuse rim between two identical faces, changing mesh output; needs a test of its own first (item 1). §222.1
Q15  2    todo    opus    -   Find whether which_two_surfaces_a_run_is_between's face-order dependence ever changes a fuse's answer when the two faces differ (item 3). §222.1
Q38  2    todo    opus    crates/rcad-step/src/brep.rs#every_face_holds_the_curve   A mitre ELLIPSE between two pipes fails the new closed-form check by about the size of its own allowance, in the Relay_DPDT/DPST_Fujitsu_FTR-F1 files. §229.5
Q39  2    todo    opus    -   Two relay files' only torus faces, both written .T., mesh the wrong way round; the lane's own geometry says .T. is right, so the kernel turns these faces over. §229.5
Q40  2    todo    opus    -   Relay_SPDT_Finder_34.51_Vertical's torus chart puts a corner's seam angle on the wrong side of ±π, crossing the mesher's boundary on itself; which mesher step picks the branch is not traced. §229.5
Q19  2    todo    opus    crates/rcad-topo/src/fuse/arrange.rs#fold_the_nodes   A half frustum cut down its own axis cannot be fused: two corners on two different rulings fold into one node and fold_the_nodes refuses EdgeHasNoLength. §215.2
Q20  2    todo    opus    -   A block big enough to hold the lens leaves a piece of its own wall bounded by a hyperbola arc with only two nodes. §215.2
Q21  2    todo    opus    -   measure.rs's cone tip has no bound, so any solid holding an apex has an unusable one; radius_of_each_face's doc comment writes out the fix. §208.6
Q7   2    todo    opus    -   Decide whether three flat fuse-fuzz seeds that give a solid one way round and refuse ResultIsNotManifold the other (2e9c4138…, 65452b22…, 1422656e…, header 0x94) are §48.2 item 5's expected flat refusal or a defect. §48.2
Q70  2    todo    opus    -   Nine in ten models the writer target makes never reach the read-back comparison, because the writer refuses them by name: ACurveTheFacesEitherSideOfItDoNotName on a flat cap meeting a cone or a torus and on the seam of a round bar turned off z, and AWireThatStandsStillAtAPole on every ball. §241.6 calls the first a gap; say which of these are gaps and which are limits. §241.6 §16.10
Q73  2    todo    opus    -   Reach round_a_closed_crossing's overlap arm with two different nodes, or show none can: a ball cut by y = b and by a tilted plane holding the same edge line bit for bit, so each circle's own meet call keeps a different one of the two cuts. §244.4
Q74  2    todo    opus    -   §244.1 found one way to a sweep proved zero with two nodes apart, on circles in axis-aligned planes; find whether a tilted circle plane or a ball–ball circle can also sum three products to exactly [0, 0] and reach the overlap arm. §244.4
Q18  2    parked:§216.3   opus    -   cut.kept's Neg conjunct in fuse/select.rs is a limit, not a gap: the outside of an ellipse-bounded arc has not been shown followable. §216.3
Q22  2    parked:§124     opus    -   §124's gap — deferred on §128.3's finding: a whole family refused by construction, no transversal control; wants a witness, not an argument. §128.3
Q27  2    parked:§136.4   opus    -   The fuse corner re-check is tighter than Model::validate — §75.6's forbidden direction; parked under §13, its repro fires at 100 m parts; §191.6 item 4 is a witness at ordinary size. §136.4
```

## Tier 2.5

```
Q8   2.5  todo    opus    -   Find why two fuse inputs (c0c1f6b…, 0d5a11d…) take about 500s in a debug build to refuse CannotMeshAFace{BoundaryCrossesItself}; lead: its_sides_cross in validate.rs re-meshes at half the deflection down to a floor. §228.3
Q10  2.5  todo    sonnet  -   Profile the cone-or-torus target's slow inputs, such as slow-000003289151us-55ef55b72ea716d8, about 3 seconds each. §227.5
Q60  2.5  todo    sonnet  -   Bisect the 84 commits ae124a29..79eedce7 for the one that made the fuse fuzz binary 10 to 340 times faster on the same inputs; set B's times on the 09-21 binary fall on three flat levels, which reads like a cap or a step limit being hit. Builds and runs fuzz binaries, so off 21:50-06:00Z. §239.7 §239.8
Q62  2.5  todo    sonnet  -   Find why set C, the inputs the 09-22 night kept, runs 1.6 times slower on the 09-22 binary than on 09-21's in all five batches, when sets A and B run 10 to 340 times faster on it. Runs fuzz binaries, so off 21:50-06:00Z. §239.7 §239.8
Q12  2.5  todo    sonnet  -   Measure how much of the night's slowdown five targets at four forks (20 processes on 24 threads) costs by itself, apart from the input-driven slowdown; §233 left load as the only cause standing for the two fast timeouts, by elimination, so a run that watches fuse bb8aa34… or cone-or-torus 694123d… cross ten seconds under that load closes it, and one that does not reopens it. sar kept nothing for 09-20 to 09-25, so only a live night can be measured. §227.5 §233.4 §234.5
Q23  2.5  todo    opus    crates/rcad-geom/src/march.rs   The marcher's step ceiling, MOST_A_STEP_MAY_GROW_TO in march.rs, needs a bound that does not come from the caller's ask. §125.6
Q35  2.5  parked:§89.7    opus    -   A cheap detector for a walk orbiting without closing, so reaching the point cap need not cost a million steps; no consumer, since the backstop already covers every corpus case. §89.7
```

## Tier 3

```
Q32  3    todo    sonnet  -   Find why the rest of the corpus still fails to import: KiCad 162 of 323, NIST 43 of 44. §229.4
```

## Tier 4

```
Q36  4    todo    opus    -   Decide rscene's demand, its roadmap §8.1 item 4: hand a meshed solid the two numbers that bound a ball round a cone's apex — how fine its mesh could have been cut (DeflectionBelowWhatAMeshCanReach's closest, on the success path) and how far a placed point can land from where it should be; check rscene's reading that the first is the ceiling over the ball and the second the floor. §13
Q33  4    todo    opus    -   Per-face colour out of STEP import: rscene's demand, reinstated over the withdrawal by its owner, so §13 no longer closes it; shape settled, unscheduled. §207
Q25  4    parked:§42.9    sonnet  -   Healing option C — imported-corner slop — is not built (item 12), because §42.9 refuses healing by name. §42.9
Q26  4    parked:§104.6   sonnet  -   §52.13 was never scored — the gate meant to score it, §53.7 charge 3, never ran; carried by §104.6. §104.6
Q34  4    parked:§104.2   opus    -   The hollow — shelling a plane-faced solid to one wall thickness; no named consumer, §44.1 is why it has a name and not a milestone. §104.2
```

## Closed

```
T1   1    done:e14b14ab   opus    -   The cone-or-torus target fell from 160 M runs a night to 2.3 M between the nights of 2026-09-20 and 09-21, same flags; and the fuse gets 2.0 M runs on 20 cores where it got 3.0 M on one — both because the corpus reached slow inputs, not the kernel or fork mode. §226.3 §227
T2   1    done:7e9d1b2c   opus    -   The fuse target asks each pair both ways round and compares the two answers, the way fuses_that_must_agree.rs's commutativity does (item 3); no disagreement on the replay. §129.5
Q1   1    done:b35ab4ce   sonnet  -   Seed docs/queue.md with every open item and retire docs/test-plan.md. §230.9
Q2   1    dropped:§230    opus    -   The queue recipes, first attempt: stopped on the context hook at call 5 with no commits; replaced by Q45. §230.9
Q45  1    done:40bd4b30   opus    -   just queue, queue ready and queue next N read this file and fill the free slots by §230.3; check-queue holds it to §230.2 in the gate; status prints the ready rows and dispatch-lanes takes its lanes from here. §230.9
T4   1    done:157b3b77   sonnet  -   The morning line says what is new tonight, by target and kind, counted against the night's start, instead of "nothing found" beside the pile of every night since the directory was made; on 2026-09-25 it would have named the fuse's three timeouts. §226.3
T5   1    done:8541b2b7   opus    -   The writer target reads back what it wrote and compares it with the model that went in (item 3): parse, references, Import::read, validate, solid count, then volume, area and centroid bit for bit. Nothing found; only two cubes reach it. §129.5
Q4   2    done:a2154ad9   opus    -   A flat piece on a torus's meridian circle offers both places the ring crosses its plane and asks which of the two disks it holds; a node on a crossing whose partner is the asked surface counts as on it. Both bends now stop one step on, at the classifier. §229.3 §232
Q3   2    done:39b278cc   opus    -   Finish lane-land-cone-and-copy: rustfmt, the two spellings of one pipe, the absorption corner, the two bars pinned. §229.6 §231
Q9   1    done:7968c2e1   sonnet  -   The two fast §228 timeouts take 1.6 s and 3.3–4.1 s alone in the fuzz build, already at the night's own commit 6664fef8: no later commit sped them up and the sanitizer build is faster than debug, so the night's load is the one cause left, by elimination. §228 §233
Q11  1    done:202ea584   sonnet  -   09-22's fuse night was fast not because it lacked the slow inputs: one of 09-21's slowest was in 09-22's corpus too, so "not reached early" is what is left, not shown; the fuse's own files were untouched going into 09-22 and rewritten going into 09-23, seen and not confirmed as a cause. §227.5 §234
Q13  1    done:bf9d4846   sonnet  -   The timing record loses runs on a slow night, not through timeouts, OOMs or crashes (0/0/0 all night) and fifty times past the per-job teardown bound; the fuse target lost a third of 09-21 too, so it is the shared timing crate, and merge_out dropping what it failed to write fits the shape, unproven. §227.5 §236
Q24  1    done:64e0bcf3   sonnet  -   composite_wildcards.awk counts the variants a composite match leaves unnamed, not one per site; it was fixed on 2026-09-03 and the row carried over from the test plan stale. §43.8 §44.9
Q47  1    done:60f09034   sonnet  -   12 of 09-21's 64 slowest fuse inputs were in the corpus before 09-22 began, so "in the corpus but not reached early" is the common case, not §234.3's lucky pick; none of 09-23's 7 kept slow inputs existed before 09-23's own night. §234.5 §238
T6   1    done:5a2019e5   sonnet  -   The reader target gets fuzz/step.dict through fuzz-reader, so every night too: 64 names the reader reads, the lane's 59 and five unit enumerations added at merge. A live run and both of a fork run's children printed "Dictionary: 59 entries" on the lane's tree. §226.1
Q42  1    done:30463253   sonnet  -   A round bar is the writer corpus's first curved seed to reach the read-back comparison, and a_curved_solid_in_the_corpus_reaches_the_comparison fails without it; the ball seed has a curved face but the writer refuses its pole first (§43.6). §129.5
Q51  1    done:64183903   sonnet  -   A corpus file's mtime is when a night's fork-mode merge last judged its bytes new, not when they were first kept: WriteToFile rewrites with no existence check and each night's tally starts empty. 09-23's slow input was very likely kept on 09-23 and rewritten on 09-24; mtime-split counts can only move one way. §238.3 §240
Q6   1    done:9b5dbd80   sonnet  -   The answer both fuse test files compare, and same_answer itself, live in one module, tests/fuzzing/comparison.rs, shared by #[path]; its own test that two different answers are refused runs in all three binaries that include it. both_ways_round_of stayed, since it needs proptest-only types. §129.5
T3   1    done:42df376e   sonnet  -   Decode the first timeouts the fuse ever wrote (three, 2026-09-24/25) and the cone-or-torus one (2026-09-25): what each input is, how long it takes on main, a hang or only slow. Two are one correct refusal that takes eight minutes; two finish in seconds here, why the night stopped them is open. §226.3 §228
Q49  1    done:9f76894b   sonnet  -   merge_out hands a failed merge's runs back to the live record and counts them lost once, however many ticks they stay stuck; the report says how many of its runs a write failure held back. Its verifier broke the hand-back, the set-not-add and the report line in turn, and the test caught each; an old totals file reads back as none lost. §236.6
Q64  1    done:de15479d   sonnet  -   merge_out marks a slow input written only once its fs::write returns Ok, so a blocked directory or one failed file write leaves it to be written on a later tick. Its verifier put the early insert back, then marked every attempted name, and each broke the one test aimed at it. §236.6
Q63  1    done:e8a00048   sonnet  -   both_ways_round_of asks both calls through one closure that notes and then fuses, and the_second_call_asks_the_pair_the_other_way_round fails when the backward call asks (one, other). The lane noted a second spelling of each pair beside the fuse call, which that mutation passed; the closure went in at merge. §129.5
Q52  1    done:a8a61b1e   sonnet  -   The fuzz side's both_ways_round now notes which pair each call was handed, and a test fails when both calls ask the same way; its verifier made the proptest side's both_ways_round_of ask one way twice and fuses_that_must_agree still passed 70 of 70, which is Q63. §129.5
Q46  1    done:25c0c093   sonnet  -   The 09-23 fuse binary runs the same inputs as fast as 09-22's (0.97, 0.96, 1.02), so the fuse/*.rs rewrite did not slow the night; it is 09-21's binary that is 10.6 times slower on its own corpus and about 340 times on what it added, so the change into 09-22, outside fuse/*.rs, sped the fuse up. §234.5 §239
Q48  1    dropped:§239    sonnet  -   Whether the libFuzzer seed decides when a night reaches its slow fuse inputs, rerunning 09-22's corpus under 09-21's seed: §239.7 found 09-21's own binary 10 to 340 times slower on the same inputs, so the 09-21/09-22 gap is the code and not when a seed reached them. §234.5 §239
Q59  1    done:0cd483a8   opus    -   Every keyword and enumeration name the reader compares by name is in names.rs, as newtypes only that module can make, and a unit test compares fuzz/step.dict with that list both ways; its verifier found 58 old literals against 58 new spellings and misspelling CIRCLE failed the dictionary test and 12 corpus tests. A literal handed to the public Instance::part or instances_named still compiles. Squashed at merge, since its WIP commit did not build alone. §226.1
Q58  1    done:06656aff   opus    -   The writer target's mutator keeps every input a model — a byte mutation kept only if it still decodes, a solid added, moved or turned, or a fresh model — and seven shape seeds (bars, ball, frustum, pointed cone, quarter bend) give it curved solids; its first minutes found a model whose own file the reader refuses and a turned bar a few ulps off on read-back (Q68, Q69). Its verifier found the mutator's own draws seed-pure but libFuzzer's byte mutation not, and §241.1 now says so. §129.5 §241
Q57  2    done:b27e6a29   opus    -   A block edge grazing a ball puts a closed crossing's walked start between two nodes 8.9e-16 apart; with their boxes apart every rule refuses, and with them touching place_the_meets folds the pair before round_a_closed_crossing sees it (Q72). No fuse in either suite reached the overlap arm with two different nodes; whether one can is Q73 and Q74. §232.3 §244
Q67  1    done:7e12dd78   sonnet  -   A slow input is written to one fixed temporary name in slowest/ and renamed onto its slow-…-<hash> name, so a write cut short never leaves a short file under a real name; one name is enough because merge_out writes under the target's blocking file lock. Its verifier put back the direct write, copied instead of renaming, and dropped the rename's error, and a test failed each time. §236.6
Q68  2    done:9f5bf6c6   opus    -   The changed byte moves the bar by 2^-1007 along y; the writer writes that bit for bit, and read-back's validate meshes the cap with the move baked in, where spade refuses a coordinate below 2^-142. The model that went in passes because a placed face is meshed in its placement's inverse frame. A gap in rcad-mesh, not the reader (Q79); the same bar also makes two fuse predicates undecidable (Q100). §241.5 §246
Q43  2    done:e4eeedc4   opus    -   Two bars side by side: selection keeps the right pieces in all six fuses, and none stops in select. Five refuse because assemble.rs names two things wrong: a bar's own circle starts at (5, 0, 0) and splits an arc the other bar's cap has as one cut (hole A, Q76), and Edges::of gives arcs of two different circles between one pair of corners one name (hole B, Q75). A gap. §231.4 §245
T8   1    dropped:§230    -       -   A gate check that holds this file's rows to their shape, the way check-plan holds docs/plan.md's; widened into check-queue instead. §226.4 §230
D1   1    done:0ac02509   -       -   The five fuzz targets run at once, each for the whole night. §129
D2   1    done:22684dad   -       -   Fork mode: a crash no longer ends a target's night. §134
D3   1    done:52ea7984   -       -   One input gets ten seconds. §135
D4   1    done:e115b2ed   -       -   Three families aimed where KiCad found bugs: rounded STEP text, turned curved solids, solids that touch. §191
D5   1    done:a736e351   -       -   The night fuzzes the tip of main, not the last green. §186
```
