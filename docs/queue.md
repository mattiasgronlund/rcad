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
Q6   1    lane:lane-q6-shared-comparison  sonnet  crates/rcad-topo/tests/fuses_that_must_agree.rs,crates/rcad-topo/tests/fuzzing/fuse.rs,crates/rcad-topo/tests/fuzzing/   Move both_ways_round_of and same_answer into one module both files share by #[path], instead of the copy lane T2 made. §129.5
T6   1    todo    sonnet  fuzz/step.dict,justfile#fuzz-reader,justfile#fuzz-target,justfile#check-fuzz-overnight-selftest   A dictionary of the STEP entity names Import::read handles, given to the reader target with -dict=. §226.1
T7   1    todo    sonnet  fuzz/fuzz_targets/no_step_file_makes_the_reader_panic.rs   Seed the reader target from the real STEP files under RCAD_STEP_CORPUS; its corpus is all fuzzer-grown. §226.1
Q24  1    todo    sonnet  crates/rcad-step/tools/composite_wildcards.awk   composite_wildcards.awk counts sites hit, not variants covered, so a check can pass without every variant firing (item 8). §43.8
Q42  1    todo    sonnet  fuzz/corpus/no_model_makes_the_writer_panic/,crates/rcad-step/tests/every_model_the_writer_fuzzer_kept_is_written_again.rs   Check in a seed for a curved solid the writer writes, so the read-back is tested where volume, area and centroid come from a mesh; today only two cubes reach it. §129.5
Q46  1    todo    sonnet  -   Replay one fixed set of fuse inputs on binaries built at ae124a29, 79eedce7 and 055aa4a9 (the nights of 09-21, 09-22, 09-23), order swapped to cancel load as §227.4 did, to say whether the fuse/*.rs rewrite going into 09-23 made the same inputs slower. §234.5
Q47  1    todo    sonnet  -   Hash all 64 slowest fuse inputs of 09-21 and of 09-23 and check each against the corpus and 09-22's 00:10 checkout, to say whether "in the corpus but not reached early" is the rule or §234.3's one example of three was a lucky pick. §234.5
Q48  1    todo    sonnet  -   Show directly whether the libFuzzer seed decides when a night reaches its slow fuse inputs: first whether -seed= pins a -fork=4 job schedule at all, then 09-22's corpus under 09-21's seed; each rerun costs a night's window, so off 22:00-06:00Z. §234.5
Q49  1    todo    sonnet  crates/rcad-fuzz-timing/src/lib.rs   merge_out takes a process's runs out of the live record before writing them and drops them when create_dir_all or the shared lock fails; hand them back for the next tick and count what could not be written, so a night's report says how much it lost, with a test that makes the directory uncreatable. §236.6
Q50  1    after:Q49   sonnet  -   Read the first night after Q49 lands: set its count of runs that could not be written against libFuzzer's own run count, to say whether merge_out's failures are the §236 undercount, and why a slow night would fail more often. §236.6
```

## Tier 2

```
Q4   2    lane:lane-f2-half-space-test   opus    crates/rcad-topo/src/fuse/select.rs,crates/rcad-topo/src/fuse/arrange.rs   A torus arm for the_middle_of_a_round_wire, and reading a point's own crossings instead of assuming a closed curve cuts the surface in two, in which_side_of_everything. §229.3
Q5   2    after:Q3   sonnet  -   Re-run lane-swallowed-pointed-cone (184962b9) once lane-cone-and-copy-finish lands. §229.7
Q43  2    todo    opus    -   Diagnose two_overlapping_parallel_bars_stop_in_select_whichever_way_round, ignored: the first union refuses ResultIsNotManifold{count:1}; the untested guess is a rim corner at (5, 0, 0) inside the lens splitting one side's arc and not its neighbour's. §231.4
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
Q18  2    parked:§216.3   opus    -   cut.kept's Neg conjunct in fuse/select.rs is a limit, not a gap: the outside of an ellipse-bounded arc has not been shown followable. §216.3
Q22  2    parked:§124     opus    -   §124's gap — deferred on §128.3's finding: a whole family refused by construction, no transversal control; wants a witness, not an argument. §128.3
Q27  2    parked:§136.4   opus    -   The fuse corner re-check is tighter than Model::validate — §75.6's forbidden direction; parked under §13, its repro fires at 100 m parts; §191.6 item 4 is a witness at ordinary size. §136.4
```

## Tier 2.5

```
Q8   2.5  todo    opus    -   Find why two fuse inputs (c0c1f6b…, 0d5a11d…) take about 500s in a debug build to refuse CannotMeshAFace{BoundaryCrossesItself}; lead: its_sides_cross in validate.rs re-meshes at half the deflection down to a floor. §228.3
Q10  2.5  todo    sonnet  -   Profile the cone-or-torus target's slow inputs, such as slow-000003289151us-55ef55b72ea716d8, about 3 seconds each. §227.5
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
Q3   2    done:39b278cc   opus    -   Finish lane-land-cone-and-copy: rustfmt, the two spellings of one pipe, the absorption corner, the two bars pinned. §229.6 §231
Q9   1    done:7968c2e1   sonnet  -   The two fast §228 timeouts take 1.6 s and 3.3–4.1 s alone in the fuzz build, already at the night's own commit 6664fef8: no later commit sped them up and the sanitizer build is faster than debug, so the night's load is the one cause left, by elimination. §228 §233
Q11  1    done:202ea584   sonnet  -   09-22's fuse night was fast not because it lacked the slow inputs: one of 09-21's slowest was in 09-22's corpus too, so "not reached early" is what is left, not shown; the fuse's own files were untouched going into 09-22 and rewritten going into 09-23, seen and not confirmed as a cause. §227.5 §234
Q13  1    done:bf9d4846   sonnet  -   The timing record loses runs on a slow night, not through timeouts, OOMs or crashes (0/0/0 all night) and fifty times past the per-job teardown bound; the fuse target lost a third of 09-21 too, so it is the shared timing crate, and merge_out dropping what it failed to write fits the shape, unproven. §227.5 §236
T3   1    done:42df376e   sonnet  -   Decode the first timeouts the fuse ever wrote (three, 2026-09-24/25) and the cone-or-torus one (2026-09-25): what each input is, how long it takes on main, a hang or only slow. Two are one correct refusal that takes eight minutes; two finish in seconds here, why the night stopped them is open. §226.3 §228
T8   1    dropped:§230    -       -   A gate check that holds this file's rows to their shape, the way check-plan holds docs/plan.md's; widened into check-queue instead. §226.4 §230
D1   1    done:0ac02509   -       -   The five fuzz targets run at once, each for the whole night. §129
D2   1    done:22684dad   -       -   Fork mode: a crash no longer ends a target's night. §134
D3   1    done:52ea7984   -       -   One input gets ten seconds. §135
D4   1    done:e115b2ed   -       -   Three families aimed where KiCad found bugs: rounded STEP text, turned curved solids, solids that touch. §191
D5   1    done:a736e351   -       -   The night fuzzes the tip of main, not the last green. §186
```
