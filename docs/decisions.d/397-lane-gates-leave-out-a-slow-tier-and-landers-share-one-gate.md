## 397. Lane gates leave out a named slow tier, landers share one gate, no cap on gates, and four slots — 2026-10-03

*On 10-03 Mattias asked four things:*

- *which tests give the most and the least for the CPU they take; he leaned
  towards running the low-value ones in one gate of ten or twenty;*
- *whether several verified lanes could land behind one gate;*
- *whether `verify-lane` runs the whole gate;*
- *whether the `just gate` runs going at once could be capped machine-wide.
  He suspects that overlapping gates cause the CPU spikes, and that the lane
  count is held down for those spikes while lanes mostly think.*

*Session main-2b measured all four and took its own recommendations, under
`CLAUDE.md`'s "decide it yourself". Mattias has decided none of it. This
amends §266.4, §372.4, §372.7 and §372.13.*

In short:

- **Lane and verifier gates leave out 30 named slow tests**, 57% of the test
  step's time, and the land gate runs them. It is not a rotation, because the
  same code gives the same answer every time (§339).
- **A lander waits on a lock and lands every lane waiting with it, behind one
  gate.** Landers stop racing each other for `main`.
- **No cap on gates at once.** The load is CPU, one build fills the machine on
  its own, and a cap of two would have held 39% of gates for a median 269 s,
  inside calls that end at 600 s.
- **Four slots once the slow tier lands.** §372.7's bar, which no slot count
  could meet, is replaced by one read from `sar`.

### 397.1 Where the numbers come from

- **Transcripts**: every session under `~/.claude/projects/` since 2026-08-30,
  about 1 300.
  - A failure is a nextest `FAIL` line in a tool result. It counts only when
    the test was already on `main` when it failed. A test is dated by the first
    commit on `origin/main`'s first-parent line that adds its `fn`, so a lane's
    own new test going red does not count.
  - Verifier runs are left out, since a verifier reverts on purpose.
  - A command runs from the last record of the response that issued it to its
    tool result.
- **`git log origin/main`** for merges and catch-ups.
- **`/var/log/sysstat/sa25` to `sa03`**: sar's ten-minute samples, read with
  `sadf -d <file> -- -u`, `-q` or `-d`, in UTC.
  - `%user` leaves out niced processes, so the overnight fuzzer (`Nice=10`) is
    not in it.
  - The run queue and the load average do count the fuzzer. That is why §397.7
    reads daytime samples only.
- **What a test costs**: each `PASS [ n s]` line of one gate's test step, in
  the `zlib-debug-sections` tree at c94d6d22, 2026-10-03 20:46 UTC. The
  `main` tree's gate of 15:26 gives the same order, within about 20%.

The scripts are in main-2b's scratchpads under
`/tmp/claude-1000/-home-mattias-git-mattiasgronlund-rcad-worktrees-zlib-debug-sections/`,
in `bd04a1c1-…/scratchpad/` and `f55f2b4c-…/scratchpad/`. A reboot removes
them, so each table below says how it was made.

### 397.2 What a test costs, and what it has caught

What a test costs:

- 2 838 tests, 886 test-seconds, and 75 s of wall for the test step.
- The 2 464 tests under 0.1 s take 38 s together, 4%.
- The 20 dearest take 54% of the test-seconds, and the 100 dearest 84%.
- **The longest test sets the step's wall.** In rcad-topo's
  `a_plane_cut_along_a_cones_other_nappe`,
  `a_corner_up_cube_round_a_frustums_apex_answers_right` takes 66.8 s and
  `a_corner_up_cube_round_a_pointed_cones_apex_answers_right` 65.6 s. 886 s
  spread over 24 cores would be 37 s.
- That gate ran on a busy machine, so a test-second ranks tests and is not a
  CPU price.

What a test has caught: 4 151 failure lines in 877 red runs. 215 of those runs
ran the whole suite (1 000 tests or more, or a gate), on 73 branches. In a
whole-suite run, a test **caught alone** when it was red and no test under 1 s
was red beside it.

| leave out tests costing at least | tests | share of test-seconds | longest kept | whole-suite red runs where only left-out tests were red |
|---|---|---|---|---|
| 20 s | 7 | 33% | 19 s | 0 |
| 10 s | 18 | 52% | 8.6 s | 0 |
| 5 s | 37 | 67% | 4.9 s | 7, on 4 branches |
| 1 s | 112 | 86% | 1.0 s | 25, on 16 branches |

At 10 s, one left-out test was red in 26 runs, and a cheaper test was red
beside it every time. Most of the misses at 5 s are two tests worth what they
cost:

- `every_kicad_file_that_imports_round_trips_to_the_same_solid`, 19 s, the only
  red test on 4 branches. One of those was the merger's gate of 18:13 on
  `merge-q563`.
- `how_often_a_real_file_imports`, 8.6 s, the only red test on 3 branches.

**The slow tier is the 30 tests that cost 5 s or more and never caught
alone.** They take 503 of the 886 test-seconds, 57%. Without them the longest
test is `a_block_below_a_frustum_is_cut_where_the_frustum_has_no_face…`, at
20.3 s. Of the 215 whole-suite red runs, 15 had a slow-tier test red, and in
none were only slow-tier tests red.

**That zero is in-sample.** The tier was picked from the same history it is
scored on. The 10 s cut's zero is the honest one, since that cut was chosen on
cost alone.

| s | binary | test |
|---|---|---|
| 66.8 | rcad-topo::a_plane_cut_along_a_cones_other_nappe | a_corner_up_cube_round_a_frustums_apex_answers_right |
| 65.6 | rcad-topo::a_plane_cut_along_a_cones_other_nappe | a_corner_up_cube_round_a_pointed_cones_apex_answers_right |
| 60.9 | rcad-step::whether_a_tolerant_policy_buys_anything_a_certified_one_does_not | no_tolerant_budget_reads_a_corpus_file_a_certified_policy_refuses |
| 33.0 | rcad-topo::a_plane_cut_along_a_cones_other_nappe | a_pointed_cone_with_its_apex_near_a_steeper_face_answers_right_or_refuses |
| 28.5 | rcad-topo::a_plane_cut_along_a_cones_other_nappe | a_steeper_plane_across_a_frustum_answers_right_or_refuses |
| 20.6 | rcad-topo::a_plane_cut_along_a_cones_other_nappe | a_corner_up_cube_centred_on_a_pointed_cones_apex_answers_every_rule_right |
| 19.1 | rcad-topo::whether_a_coarser_deflection_always_meshes | which_refusals_never_turned_into_a_mesh_anywhere_on_the_ladder |
| 19.1 | rcad-topo::whether_a_coarser_deflection_always_meshes | the_verdict_is_scoped_to_the_family_that_was_swept |
| 18.2 | rcad-step::the_same_file_read_twice_by_two_readers | the_two_readers_agree_about_every_corpus_file_except_where_escapes_are_decoded |
| 15.0 | rcad-topo::a_cap_whose_rim_runs_through_the_other_pole | a_ball_cut_through_its_south_pole_keeps_its_north_pole_and_measures_a_cap |
| 12.2 | rcad-topo::a_piece_of_a_ball_holding_both_poles | a_ball_less_a_cap_built_face_by_face_meshes_closed_and_measures |
| 11.9 | rcad-topo::meshing_a_plane_cut_along_a_cones_other_nappe | the_eighteen_held_to_the_familys_four_checks |
| 10.6 | rcad-topo::a_plane_cut_along_a_cones_other_nappe | a_frustum_whose_apex_is_inside_the_block_answers_right_or_refuses |
| 10.3 | rcad-topo::what_a_deflection_costs_a_measured_volume | the_volume_a_mesh_misses_falls_in_step_with_the_deflection |
| 8.3 | rcad-topo::fuses_that_must_agree | every_fuse_a_turned_curved_fixture_refuses_has_an_answer_turned_from_the_upright_one |
| 8.0 | rcad-geom::why_the_walked_point_oracle_checks_nothing_on_the_slow_pipes | the_zero_on_the_pipe_tilted_by_a_hundred_and_twenty_two_orders_is_the_sizing_walk |
| 7.7 | rcad-geom::why_the_walked_point_oracle_checks_nothing_on_the_slow_pipes | the_zero_on_the_pipe_tilted_by_a_hundred_and_twenty_nine_orders_is_the_sizing_walk |
| 7.7 | rcad-topo::a_ball_cut_through_its_pole | a_block_taken_out_of_a_ball_through_its_south_pole_leaves_the_same_cap |
| 7.5 | rcad-topo::a_ball_cut_through_its_pole | a_ball_cut_through_its_south_pole_by_a_block_keeps_a_cap_that_measures_right |
| 7.4 | rcad-topo::fuses_that_must_agree | what_a_block_and_a_leaning_bar_take_up_adds_up_inside_a_stated_share_of_what_they_declare |
| 7.4 | rcad-step::styling_is_never_among_the_shapes_walked_past | a_file_with_styled_items_reports_no_styling_keyword_walked_past |
| 7.3 | rcad-topo::whether_a_coarser_deflection_always_meshes | a_coarser_deflection_meshes_whenever_a_finer_one_did_on_solids_drawn_at_random |
| 7.2 | rcad-step::no_cone_face_arrives_out_of_the_corpus | no_cone_face_arrives_out_of_the_corpus |
| 7.1 | rcad-topo::fuses_that_must_agree | how_often_a_certified_fuse_of_a_turned_frustum_and_block_refuses |
| 6.8 | rcad-topo::what_the_answered_block_fuses_actually_say | every_block_and_frustum_fuse_that_answers_adds_up |
| 6.5 | rcad-topo::whether_a_coarser_deflection_always_meshes | the_floor_refusal_needs_an_absolute_deflection_and_then_it_behaves |
| 6.1 | rcad-geom::why_the_walked_point_oracle_checks_nothing_on_the_slow_pipes | the_zero_on_the_moved_hair_thin_pair_is_the_sizing_walk_refusing |
| 5.7 | rcad-topo::how_often_a_curved_fuse_refuses | every_answer_over_a_block_turned_against_a_bar_is_a_solid_that_validates |
| 5.6 | rcad-topo::fuses_that_must_agree | what_a_block_and_a_ball_take_up_adds_up_inside_a_stated_share_of_what_they_declare |
| 5.1 | rcad-topo::a_ball_cut_through_its_pole | the_sliver_that_holds_no_pole_is_answered_or_refused_by_name |

The names are as of c94d6d22. `main` has moved 206 commits since, so the row
that builds this checks each name on its own base (§397.4 item 5).

What these numbers cannot say:

- "Red" includes a deliberate pin update, a flaky test, and a lane editing
  that very test.
- The transcripts hold failures and not passes, so there is no failure rate
  per run.
- The cone sweeps landed between 09-23 and 09-26, so they have about ten days
  of history.

### 397.3 Why not one gate in ten

Mattias leaned towards running the low-value tests in one gate of ten or
twenty. That is not taken, for three reasons.

- **The low-value tests are cheap.** Most tests that never caught anything
  cost under 0.1 s, and all 2 464 of those together are 4% of the time. The
  time is in a few long tests.
- **A test skipped now finds nothing later that it would have found now.**
  Every property draws from one fixed seed (§339), so a test gives the same
  answer on the same code. Running a test in one lane gate of ten searches
  nothing new. It only catches a defect earlier for the lane that happened to
  draw it. With the land gate running every test, nothing reaches `main`
  either way.
- **A gate that draws its tests breaks serving.** §337 serves a verdict only
  because running the same code again would give the same one (§339.2). Two
  gates of the same code that ran different tests have no one verdict to
  serve. §97.6 turned down a cheap rotation of mutants for the same kind of
  reason: a split by position moves as the tree changes.

### 397.4 Lane gates leave out the slow tier, and the land gate runs it

What the row builds:

1. **`slow_tests`**, a justfile variable beside `gate_steps`, names the 30
   tests of §397.2 by binary and test.
2. **`test` runs every test but those, and a new step, `test-slow`, runs only
   those.** Both go in the cargo chain and in `gate_reusable_steps`.
3. **A word to `just gate` leaves `test-slow` out**, and `pre-gate` passes the
   word on. `lane.md` and `verify_lane.py` use it, and nothing else does.
   - Everything else runs every test: bare `just gate`, `land.py`, the Opus
     merger, CI, and a session landing its own work.
   - A caller that forgets the word pays time, never coverage.
   - A check fails when the word appears anywhere but those two files, in
     `check-gate-runs-every-check` or an arm beside it.
4. **A verdict record names the steps it passed, and a gate serves only the
   steps its record names.**
   - Today a run that leaves out a reusable step records nothing (the
     `left_out` branch of `gate`). So every lane gate would go unrecorded, and
     its verifier's gate would run fresh.
   - With the steps named, the verifier's gate is served from the lane's. A
     land gate on the same code is served everything but `test-slow`, which
     runs.
5. **Every gate checks the list, lane or not.** It counts the tests
   `slow_tests` selects and fails unless the count is the list's length.
   - A renamed or split test then fails the gate of the lane that renamed it,
     not the land gate after it.
   - A name that matches nothing would otherwise leave the test in `test`,
     which is safe. A name that matches two tests would take one out of every
     lane gate unseen, and that is what the count is for.
6. **`lane.md` gains a line**: the slow tier is listed in `slow_tests`, and a
   lane runs the ones that cover what it changed with `just t`.

How a test joins the list and leaves it:

- **It joins** only when someone scores both halves again: the cost from a
  gate's test log and the catches from the transcripts, as §397.2 does. Being
  slow is not enough.
- **It leaves** when it fails at a land gate after its lane's gates passed. It
  has then caught something alone. It leaves in the commit that fixes what it
  caught.

What it buys, and what it costs:

- A lane's test step runs 383 test-seconds instead of 886, and its longest
  test takes 20.3 s instead of 66.8 s.
- It does not touch compiling, which is the rest of a fresh gate.
- **Without the split row, the land gate gets slower.** `test` and
  `test-slow` both build with cargo, so they are chained, one after the other
  (§372.10). A whole gate would wait about 21 s for the rest and then 67 s for
  the slow tier. Today the two share one 75 s step.
- The split row cuts every slow-tier test over 21 s into pieces of about 20 s
  each, with the same shapes or files in them. There are five: the cone sweeps
  of 66.8, 65.6, 33.0 and 28.5 s, and rcad-step's 60.9 s
  `no_tolerant_budget_reads_a_corpus_file_a_certified_policy_refuses`.
  - Cutting only the two 66 s sweeps would not do. The 60.9 s test would then
    set `test-slow`'s wall, and the two test steps would take about 82 s,
    against 75 s today.
  - With all five cut, `test-slow` takes about 21 s (503 s over 24 cores), and
    a whole gate is faster than today.
  - The split row also puts the new names in `slow_tests`.

**2026-10-04, since Q849 (b80f5e88).** Built as above, with four differences
and the times measured.

- **The times.** A fresh whole gate on b80f5e88: `test` 27.8 s (nextest's own
  wall 24.0 s, 2920 tests), then `test-slow` 60.0 s (59.5 s, 30 tests), so
  87.8 s in all, against the 88 s predicted above. A `gate-lane fresh` on the
  same commit, while two other sessions were busy, took 44.1 s for `test`
  (36.2 s). A plain gate over that lane gate's record was served every
  reusable step but `test-slow`, ran it in 57.9 s, and added it to the record.
  Until Q850 cuts the five long tests, the whole gate is about 13 s slower
  than today's single 75 s step, as predicted.
- **`gate-lane` is a recipe, not a word to `gate`** as item 3 says. It mirrors
  `gate-docs`. `check-gate-lane-callers` then has one literal to search for.
  And `just pre-gate gate-lane` is two recipes in one call, so `pre-gate` has
  nothing to pass on. The check is its own step, not an arm of
  `check-gate-runs-every-check`. `lane.md`, `verifier.md` and `verify_lane.py`
  may call it: the verifier runs a lane's gate too, which item 3 left out.
- **The filter is exact.** Each name in `slow_tests` becomes
  `binary_id(=…) & test(=…)`, so a name selects at most one test. The count of item 5 therefore catches a name
  that selects nothing and a name written twice. Item 5's "a name that matches
  two tests" cannot happen.
- **A `--set` on the outer `just` does not reach a nested one.** `test` and
  `test-slow` first read `slow_tests` through a nested `just`. A selftest
  that set a fixture list counted the fixture and then filtered on the real
  list. `_slow-tests-filter` now takes the list as an argument. The stub-cargo
  arm of `check-gate-docs-selftest` caught it and now holds it.

**2026-10-04, since Q850 (7022e4a7).** The five tests are cut as the split row
says. On a busy machine `test-slow` did not come down to 21 s.

- **The cut.** rcad-step's corpus sweep is three tests, one per tolerant
  budget, and each compares 367 pairs. Each cone family is two tests, one per
  lean, or four, one per side and turn. Each family's counts add up to its old
  pin. `slow_tests` names 40 tests where it named 30.
- **Each piece alone.** rcad-step's three took 13.2, 13.2 and 19.1 s of CPU
  (`/usr/bin/time` on the test binary). Run one case at a time, the cone
  families took 54.8 s (frustum cube), 51.2 s (pointed cube), 22.4 s (pointed
  cone, steeper face) and 15.3 s (frustum, steeper plane) in all. The cone
  file's 19 tests run together at a load of about 13 took 20.2 s, and that was
  its longest piece.
- **In the gate.** A whole gate on 7022e4a7, starting at a load of about 14 on
  24 cores: `test` 48.2 s, then `test-slow` 55.6 s (nextest's wall 54.5 s).
  The 40 tests added up to 939 test-seconds, against the 503 the arithmetic
  above used, and 22 of them took over 20 s. Pieces that take 13 to 20 s alone
  took 40 to 44 s. `just test-slow` on its own at a load of 25 to 31 took
  43.9 s.
- **What that says.** `test` runs the same tests it ran on b80f5e88 and took
  48.2 s against 27.8 s, so the whole machine was slower, not the slow tier
  alone. With no piece over about 20 s alone, the longest test no longer sets
  `test-slow`'s wall; the free cores do. "About 21 s (503 s over 24 cores)"
  holds only when the slow tier has the 24 cores to itself, and here it had
  about ten. Nobody has measured it on a quiet machine. How often the machine
  is not quiet is §397.7's first bar, which Q853 measures.
- **Three comments outside the write set** named an old test and now name a
  new one: `rcad-step/src/brep.rs`, `rcad-topo/src/holds.rs` and
  `meshing_a_plane_cut_along_a_cones_other_nappe.rs`.

### 397.5 Landers share one gate (amends §266.4)

What `origin/main` shows, from 09-29 12:00 to 10-03 21:00 UTC (104.9 h at
three slots):

- 79 lane merges, 73 of which changed code. The median gap was 14.8 min.
- Of 72 code merges after the first, 11 came within 3 min of the one before,
  14 within 5 min and 21 within 10 min.
- 53 of the 73 merged onto a `main` whose code had moved since the lane
  branched, so their land gate could not be served (§337).
- 17 catch-ups: a merge branch merging `origin/main` again after losing the
  push race. 12 took in code, and each of those ran a second fresh gate inside
  the same merger's or lander's call.
- The merges come in bursts: 50 on 09-29, 7 on 09-30, none on 10-01, 8 on
  10-02 and 34 on 10-03.

**`verify-lane` does run the whole gate**: `gate quiet` on the lane's tip, in
`verify_lane.py`. It is served whenever the lane gated the same commit, so it
costs little. The land gate is the one that costs.

The design is **take everything waiting**:

1. **`just land` becomes one detached run**, which each call follows for at
   most 540 s, with `just land wait` to follow it again. That is §385.2's
   pattern, for §385.2's reason. Today a lander that loses one race to code
   already runs two fresh gates in one 600 s call, 2 × 269 s at the median,
   and waiting on a lock adds more.
2. **The run writes its lane's entry to a spool**: every argument of `just
   land`, the steps, and its pid. The spool sits in the git common directory.
   The run then waits on a `flock` there. The kernel drops the lock when the
   process dies, so a dead lander never holds it.
3. **Once the run holds the lock, it looks for a result file for its own
   entry.** If an earlier holder landed its lane, it reports that merge and
   ends.
4. **Otherwise it lands every entry whose run is still alive, oldest first,
   its own among them.** It fetches and branches from `origin/main`. For each
   entry it does what `just land` does today: the sha check, the merge, the §0
   check, one `queue apply` with that lane's `done` and `add` steps,
   `check-queue`, and a commit.
   - So each lane keeps its own merge commit, and its rows close at its own
     tip.
   - An entry that stops at any of these steps is reset away and left in the
     spool for its own run.
5. **One gate for them all.** If it is green, the push loop runs as today.
   Once the push is through, the run writes a result file for every entry it
   landed, runs `ff-main`, and unlocks every entry's trees and marks them
   finished.
6. **If the gate is red and other lanes were in the batch, the run goes back
   to `origin/main` and lands its own lane alone**, as today. The others stay
   for their own runs, and each gets the lock in turn.
7. **An entry whose run is dead is never taken.** `lanes.js` has already
   handed that lane to the Opus merger (§384.3), and taking it as well would
   land it twice.
8. **The Opus merger takes no lock.** It races as §266.4 says, and the
   fast-forward check still keeps that correct. A merger works for minutes on
   judgement, and holding the lock through that would hold every lander
   behind it.
9. **`already-on-main` stays a stop when no result file explains it.**

What it should save at three slots:

- Lanes landing within minutes of each other share a gate: 11 to 14 of the
  72 merges in the period.
- No lander loses a race to another lander.
- Together that is about 12 to 15 fewer fresh gates in the 104.9 h, roughly a
  fifth of the gates landing ran. It is an estimate, not a replay.
- The saving grows faster than the merge rate. The chance that two landings
  overlap grows with the square of how often lanes land, and four slots
  (§397.7) raise that rate.

Not chosen: **waiting on purpose to fill a batch.** A five-minute wait would
have batched 14 of 72 merges. It would also add five minutes to every landing,
and a verified lane would hold its files the whole time.

**The amendment.** §266.4 said merges "need no lock". On correctness that
still holds, since the fast-forward check makes a lost race harmless. What a
lost race costs, a second fresh gate, was not measured then. Landers now take
a lock.

**2026-10-04, since Q851 (d5625791, 0b47f03c).** Built as items 1 to 9 say,
with two departures, one gap closed, and one cost named.

- **What was built.** `just land` starts `land.py --detached-run` in a session
  of its own and follows its log for at most `LAND_WAIT` seconds, 540 by
  default, and `just land wait` follows it again. The spool, the lock and the
  result files sit under `rcad-land/` in the git common directory. An entry's
  run is alive when its pid is there, is not a zombie, and started when the
  entry says, so a pid used again by another process is not taken for the run.
- **The holder lands its own lane first**, then the others oldest first. Item
  4 had its own lane among them in age order. This way a stop of its own comes
  before any other lane's `queue apply`, and leaves none to throw away.
- **The result files are written last**, after `ff-main` and `mark-finished`.
  Item 5 wrote them first. This way each holds every field the lander reports.
  A holder killed between its push and its results leaves those lanes on
  `main` with no result, so each of their runs stops at `already-on-main`,
  which item 9 keeps and the Opus merger settles.
- **`rows_closed` and `rows_added` fill only on a merge.** Before, a stop after
  the queue apply reported rows it had not closed on `main`. Nothing reads them
  on a stop: `lanes.js` hands the Opus merger the stop line alone.
- **Item 7 assumed that a lane reaches the Opus merger only once its run is
  dead, and a detached run outlives its lander.** A lander that dies, runs out
  of turns, or gives up at the first `still running` leaves its run waiting or
  landing, and `lanes.js` starts the merger all the same. The merger's ancestor
  check could then come before the run's push, and the lane would land twice:
  two merge commits and two `queue apply`s. So `just land wait-for <lane
  branch>` waits, at most 540 s a call and exit 3 while it still waits, for as
  long as a live run's entry names the lane or anyone holds the lock. The
  merger's brief runs it before `git fetch origin` and the ancestor check.
  - Both legs are needed. A holder can be landing an entry whose run died after
    the holder listed it. A run can be alive and waiting while the lock is
    free; the selftest makes one with `SIGSTOP`.
  - Waiting on any holder is wider than it needs to be. Which lanes a holder
    took cannot be read without its lock, so `wait-for` does not try.
  - `merger.md` is outside the row's write set, so the brief carries the step.
    The `still running` line names `wait-for` too, for a merger briefed by a
    `lanes.js` older than this.
- **A reset leaves its marks in the claims file.** `queue apply` records each
  `done` as a `release` and each `add` as an `add` in the claims file every
  tree shares (`tooling/queue/queue.py`), and `git reset` takes neither back.
  So a lane reset out of a batch, at a stop of its own or when a red batch
  falls back, leaves its row released and its new ids used up, and its retry
  takes new ids. A lone lander's stop after its `queue apply` cost the same
  before; a batch resets more often.
- **The selftest** keeps its 9 stops and its clean arm and gains four arms:
  two landers share one gate, a red batch falls back to one lane, a dead entry
  is left alone, and a killed holder drops the lock. The dead-entry arm also
  holds `wait-for` to both legs, and the briefs are held to the `wait` and
  `wait-for` lines. At a load of 43 to 53 on 24 cores it took 26.5 s without
  `wait-for`, against 22.8 s before Q851. At a load of 3 to 6, three runs
  each, taken in turn: 12.6 to 15.1 s without `wait-for` and 14.6 to 14.9 s
  with it. Thirteen mutations of `land.py` and the briefs each failed it; the
  two commits name them.
- **The changeover.** A dispatcher whose `claude` process started before this
  landed keeps the `lander.md` and `lanes.js` it loaded, and `/clear` does not
  reload them. Its landers run the new `land.py` with the old prompt, which
  runs `just land` once. So such a lander reports `merged: false` on any
  landing that takes over 540 s, where today one catch-up over code already
  costs 2 × 269 s at the median. Its Opus merger gets the old brief, without
  `wait-for`. The one thing that names `wait-for` to it is the `still running`
  line, which the old `lander.md` has the lander copy into
  `not_merged_because`. Restart such a dispatcher's `claude` process once this
  is on `main`.

### 397.6 No cap on gates at once, yet (amends §372.13)

What the load is made of:

- **CPU, not disk.** Over the busy windows, `iowait` averaged 0.55% of all
  cores, about an eighth of one, with a highest window of 7.2%. On average
  0.47 tasks were blocked at a sample. The load average is the run queue.
- **One build fills the machine by itself.**
  - 09-25 05:40 UTC: a run queue of 62, with `just t` running (1.2 on
    average over the window) and no gate.
  - 09-26 03:00 UTC: a run queue of 80, with less than one fresh gate on
    average over the window.
  - A cap on gates can stop spikes from stacking. It cannot stop them from
    happening.
- **They do stack.** The worst sample, 09-28 22:10 UTC, had a run queue of 97,
  with 3.5 fresh gates and 1.4 `just t` running on average over the window.

How long a fresh gate takes beside other heavy runs, since three slots. A
heavy run is a fresh gate, `land`, `verify-lane`, `just t` or `pre-gate`.

| other heavy runs beside it, on average | fresh gates | median | 90th percentile |
|---|---|---|---|
| under 0.1 | 62 | 106 s | 264 s |
| 0.1 to 1 | 30 | 260 to 296 s | 353 to 385 s |
| 1 to 2 | 49 | 315 to 343 s | 403 to 463 s |
| 2 or more | 57 | 351 to 375 s | 433 to 454 s |

Part of that is contention. Part is which gates run when the machine is busy:
a lane's gate after an edit rebuilds more than a dispatcher re-gating its
base. The transcripts cannot separate the two.

**A replay of a cap.** Every fresh gate, `land` and `verify-lane` run since
09-29 12:00, 250 runs, sent through N machine-wide slots, first come first
served, each keeping its measured duration:

| cap | runs that wait | median wait | 90th percentile | longest | waits over 300 s |
|---|---|---|---|---|---|
| 2 | 97 of 250, 39% | 269 s | 1 326 s | 1 846 s | 45 |
| 3 | 35 of 250, 14% | 219 s | 617 s | 712 s | 15 |

- **The waits are too high**, because a run held back would have had less
  beside it and run faster. By how much is not known.
- Even so, a lane runs its gate in one foreground call that ends at 600 s
  (`lane.md`). A cap that holds a gate for minutes needs the gate to run
  detached first, and that is a bigger change than the cap.
- Of the time any gate-like run was going, one ran alone 59% of the time, two
  at once 23%, three 9%, and four or more 8%.

**Decision: no cap now.** The slow tier (§397.4) and the shared land gate
(§397.5) cut the demand itself: what each lane gate does, and how many gates
landing runs. A cap would only move the same demand later.

**What would bring a cap:** if, after both, §397.7's share at four slots goes
past its bar, and the worst samples show three or more gates at once. The cap
would then be three, behind a gate that runs detached.

**The amendment.** §372.13 is right that the gate should not throttle itself
by load, and nothing here changes that. Its last line made §372.4's three
slots the lever for a busy machine. The levers come in this order instead:
what each gate runs, how many gates landing needs, and then the slot count,
by §397.7's bar.

### 397.7 Four slots once the slow tier lands (amends §372.4 and §372.7)

**§372.7's load bar cannot be met.** It asked that the one-minute load stay
under 24 with three lanes gating, and one `just t` with no gate reached 62
(§397.6). The replacement is read from `sar` alone, so it outlives the
transcripts and needs no join:

- **Daytime**: samples from 06:10 to 21:40 UTC. That keeps out the niced
  overnight fuzzer, which the load counts.
- **Busy**: at least one core of `%user`. Whole days do not work: on 09-30
  and 10-01 the machine sat idle, at 0.1 cores and with 7 and 0 merges.
- **The bar**: the share of busy daytime samples whose one-minute load is 24
  or more.

| slots | busy daytime samples | user cores, mean | 90th percentile | 99th percentile | load ≥ 24 | load ≥ 48 |
|---|---|---|---|---|---|---|
| six, until 09-29 12:00 | 259 | 8.1 | 14.9 | 19.0 | 28% | 3% |
| three, since | 121 | 6.0 | 11.0 | 15.2 | 12% | 1% |

**It tells six slots from three**, and that is the test a bar has to pass.

**Mattias's reading holds.** Even at six slots, a busy ten-minute window
averaged 8 cores of 24, and the 99th percentile was 19. Short spikes hold the
machine back, when builds and test runs stack. The lanes being busy does not.

**§372.7's other bar is already met.** Since three slots there have been 17
catch-ups in 79 merges, so at most 22% of merges needed a second try.
§372.2's baseline was 72 of 146, 49%.

**Decision: four slots once the slow-tier row has landed.**

- **Back to three** if either holds over at least 100 busy daytime samples at
  four slots:
  - the share at load 24 or more passes 20%;
  - catch-ups pass 35% of lane merges over the same days, counted from
    `git log` as §397.5 counts them.
- **Both bars are judgement.** A straight line between three slots' 12% and
  six slots' 28% puts four at about 17%, and 20% leaves room for the noise
  of a hundred samples. 35% lies between today's 22% and §372.2's 49%.
- **Disk is not the limit.** `/` has 398 GB free of 913 GB. A tree's own share
  is about 5.7 GB, so a fourth lane and its verifier cost about 12 GB.
- **§372.4's counting stands:** read-only rows, verifiers and mergers each
  count toward the four.

**2026-10-13: the first bar is passed, and replaced.** From 10-05 to 10-13,
43% of 716 busy daytime samples were at load 24 or more, and no single day was
under 23%. But on 24 threads over 12 cores a load of 24 means full, and past
full the gate's test step ran slower without failing. §447.3 keeps four slots,
one of them for a Sonnet row, and replaces the load bar with the share of
fresh test steps that run past 600 s.

### 397.8 The claims this rests on (§16.10)

- **"Nothing reaches `main` without the whole gate"** is a sufficiency claim.
  It ranges over every way code reaches `main`, and each runs bare `just gate`:
  - `land.py`'s `gate()`;
  - the Opus merger, by `merger.md` step 4;
  - a session landing its own work, by `CLAUDE.md`;
  - CI.

  Only `lane.md` and `verify_lane.py` pass the word that leaves the slow tier
  out. §397.4 item 3's check is what keeps it to them.
- **"The slow tier never caught alone" is an in-sample zero** (§397.2). The
  land gate is what makes being wrong cheap: a defect that only a slow test
  sees is caught at landing, before `main`.
- **"Running a test in one gate of ten searches nothing new"** rests on §339's
  fixed seed. §339.4 leaves `HashMap` order unaudited. If a test's answer
  hangs on that order, it searches by accident in every gate alike, so the
  case for a named tier over a rotation stands either way.
- **"Landers stop racing each other"** is a sufficiency claim, and it ranges
  over the landers on this machine:
  - every lander takes the same `flock` before it fetches;
  - the lock lives in the git common directory, which every worktree here
    shares;
  - a dead process drops it.

  It does not cover the Opus merger, which still races (§397.5 item 8). CI
  never pushes to `main`.
- **"A cap cannot stop spikes"** is shown by the samples where one `just t` or
  less than one gate reached a run queue of 62 or 80 (§397.6).
- **"The replay's waits are too high"** gives the direction and not the size
  (§397.6).

### 397.9 The rows, in order

Filed on 2026-10-03:

- **Q849, the slow tier**: §397.4, items 1 to 6. Tier 2.2, opus.
- **Q850, the split**: the five slow-tier tests over 21 s, after Q849, and
  after Q846, which moves a pin in the cone sweeps' file. Tier 2.2, sonnet. It
  is in the same tier as Q849 because, until it lands, Q849 makes every land
  gate slower (§397.4).
- **Q851, the shared land gate**: §397.5, after Q849, since both may reach
  into `verify_lane.py`'s detached run. Tier 2.2, opus.
- **Q852, four slots**: `queue_slots` and its pins, after Q849. Tier 2.2,
  sonnet, small.
- **Q853, the measurement**: §397.7's two bars, after Q852, once there are 100
  busy daytime samples at four. Tier 2.2, sonnet.

What this leaves:

- **Recording passes**, nextest's JUnit kept with each gate's sha, would give
  a failure rate per run. Nothing needs one yet (§13). Re-scoring the tier
  reads failures from the transcripts, as §397.2 did. If the transcripts turn
  out too thin for that, the re-scoring is the consumer.
- **A cap on `just t`.** It is the other source of spikes (§397.6), and nothing
  here touches it.
