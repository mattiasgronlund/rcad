#!/usr/bin/env python3
"""`row_time.py` against made-up run records, transcripts and claims, every number worked out by hand.

`row-time` reads this week's sessions, so its own answer moves with them and
cannot be held. What can be held is what it decides: how a step's time is
split into buckets, how a row's elapsed time is split into the agents and the
waits between them, and how long each tree held a slot. The fixtures here are
made up, not copied from real transcripts (§411.9), and between them they hold
every case of §411.9's list:

- a transcript split across two session folders, in two project folders;
- a bundle, one lane for two rows;
- a shared gate, a lane another run's lander landed;
- a retried row, including a run killed and started again under one take;
- two tools running at once;
- a tool that starts before its response ends;
- a land wait with timestamps and one without;
- a tree still locked when the run ended;
- a lane relayed once, whose relay keeps the lane's own role (§416);
- a row whose prep refused it, with no lane ever started (§396.4);
- a row closed at the base, with no lander or merger (§433.8);
- a shared gate whose holder itself relayed mid-`just land`, found by its raw
  label rather than by `role_of`'s own lane field (§411.5, §416, Q1935);
- a shared gate whose holder's own run is gone from the retained transcripts,
  named rather than crashing the whole read (§411.5, Q1935);
- a response priced from its usage, a streamed group keeping only the record
  with the largest `output_tokens` (§411.1);
- and a fixture whose parts do not add up, which must make `row-time` fail
  and print nothing.

Run through `just check-row-time-selftest`.
"""

import contextlib
import datetime
import io
import json
import os
import sys
import tempfile

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "session-spend"))

import row_time  # noqa: E402
import spend  # noqa: E402

REPO = "-home-me-git-rcad"
BASE = row_time.ms_of_stamp("2026-10-05T10:00:00Z")
FAILURES = []

# The usage §411.1's cost beside each time is priced from, in a made-up run
# whose transcripts carry nothing else worth asserting (Q2596).
COST_MODEL = "claude-sonnet-5"
COST_USAGE_1 = {"input_tokens": 100_000, "output_tokens": 200, "cache_read_input_tokens": 500_000,
                "cache_creation": {"ephemeral_1h_input_tokens": 1_000, "ephemeral_5m_input_tokens": 2_000}}
COST_USAGE_2 = {"input_tokens": 1, "output_tokens": 50}
COST_USAGE_3 = {"input_tokens": 10, "output_tokens": 5}


def main():
    with tempfile.TemporaryDirectory() as scratch:
        world = World(scratch)
        build_the_week(world)
        world.write()
        assembled = assemble(world)
        check_a_plain_lane(assembled)
        check_a_bundle_and_a_shared_gate(assembled)
        check_a_retried_row(assembled)
        check_a_relayed_lane(assembled)
        check_a_prep_refused_row(assembled)
        check_a_close_at_base_row(assembled)
        check_a_relayed_holder_lands_another_run(assembled)
        check_a_missing_holder_lands_another_run(assembled)
        check_a_response_priced_from_its_usage(assembled)
        check_cost_in_the_summary(world)
        check_every_step_adds_up(assembled)
        check_the_printed_report(world)
    with tempfile.TemporaryDirectory() as scratch:
        check_parts_that_do_not_add_up(scratch)
    check_command_keys()
    check_a_cut_off_line()
    if FAILURES:
        for failure in FAILURES:
            print(f"FAIL {failure}")
        print(f"check-row-time-selftest: {len(FAILURES)} failed")
        return 1
    print("check-row-time-selftest: every case holds")
    return 0


def same(what, got, want):
    if got != want:
        FAILURES.append(f"{what}: got {got!r}, want {want!r}")


def at(ms):
    """A transcript timestamp for BASE + ms."""
    when = row_time.EPOCH + datetime.timedelta(milliseconds=BASE + ms)
    return when.strftime("%Y-%m-%dT%H:%M:%S.") + f"{(BASE + ms) % 1000:03d}Z"


def second(ms):
    """A claims-file or `land` line time for BASE + ms, which must be a whole second."""
    assert ms % 1000 == 0, ms
    return row_time.fmt_time(BASE + ms)


# ---------------------------------------------------------------------------
# The made-up week


def build_the_week(world):
    a_plain_lane(world)
    a_bundle_whose_lander_lands_another_run(world)
    a_retried_row(world)
    a_relayed_lane(world)
    a_prep_refused_row(world)
    a_close_at_base_row(world)
    a_relayed_holder_lands_another_run(world)
    a_missing_holder_lands_another_run(world)
    a_response_priced_from_its_usage(world)


def a_plain_lane(world):
    """Q1: lane, verifier, lander and pruner, the lane's transcript split over two project folders.

    The lane runs from 10 s to 80.2 s. Its first response reads a file whose
    result comes before the response's last block, so `Read` is charged
    nothing; its second runs `just t` and `git status` at once, so the 10 s
    both wait are split 5 s each. The lander's hold line carries its time.
    """
    lane = Talk("/trees/r1-1")
    lane.prompt(10_000)
    lane.attachment(10_500)
    lane.response((12_000, thinking()), (13_000, call("t1", "Read", file_path="/x")),
                  (15_000, call("t2", "Bash", command="cd /trees/r1-1 && mise exec -- just gate quiet | tail -5")))
    lane.result(13_500, "t1")
    lane.cut()
    lane.result(40_000, "t2")
    lane.response((41_000, call("t3", "Bash", command="mise exec -- just t how_often_a_thing")),
                  (41_500, call("t4", "Bash", command="git -C /trees/r1-1 status")))
    lane.result(51_500, "t4")
    lane.result(61_500, "t3")
    lane.response((62_000, prose()))
    lane.prompt(70_000, "Your response above was cut off mid-stream.")
    lane.response((80_000, call("t5", "StructuredOutput")))
    lane.result(80_100, "t5")

    verifier = Talk("/trees/r1-2")
    verifier.prompt(80_400)
    verifier.response((81_000, call("t6", "Bash", command="mise exec -- just verify-lane abc lane-q1-a-plain-lane")))
    verifier.result(99_000, "t6")
    verifier.response((99_500, call("t7", "StructuredOutput")))
    verifier.result(99_600, "t7")

    lander = Talk("/trees/r1-3")
    lander.prompt(101_000)
    lander.response((102_000, call("t8", "Bash", command="mise exec -- just land Q1 lane-q1-a-plain-lane abc1234 "
                                                         "merge-q1-a-plain-lane /trees/r1-1 /trees/r1-2")))
    lander.result(150_000, "t8", f"+ git rev-parse --show-toplevel\n{second(110_000)} land: waiting for the land lock\n"
                                 f"{second(120_000)} land: holding the land lock\nland: still running (pid 7).")
    lander.response((151_000, call("t9", "Bash", command="mise exec -- just land wait")))
    lander.result(190_000, "t9", "+ git worktree unlock /trees/r1-1\n+ git worktree unlock /trees/r1-2\n"
                  + land_json("merge-q1-a-plain-lane"))
    lander.response((191_000, call("t10", "StructuredOutput")))
    lander.result(191_100, "t10")

    pruner = Talk("/trees/dispatcher")
    pruner.prompt(200_200)
    pruner.response((201_000, call("t11", "Bash", command="mise exec -- just prune-worktrees")))
    pruner.result(209_000, "t11")
    pruner.response((209_500, prose()))

    run = world.run("wf_r1", REPO, "sess-a", [lane_of("q1-a-plain-lane", "Q1")])
    run.agent("q1-a-plain-lane", "l1", 10_000, 80_200, lane, split_into=(REPO + "-worktrees-other", "sess-b"))
    run.agent("verify:q1-a-plain-lane", "v1", 80_400, 100_000, verifier, queued=80_300)
    run.agent("land:q1-a-plain-lane", "d1", 101_000, 200_000, lander, queued=100_500)
    run.agent("prune:q1-a-plain-lane", "p1", 200_200, 210_000, pruner, queued=200_100)
    world.claim("take", "Q1", 5_000, "lane-q1-a-plain-lane")
    world.claim("release", "Q1", 150_000, "done")


def a_bundle_whose_lander_lands_another_run(world):
    """Q2 and Q3 share one lane, whose lander holds the lock and lands Q4's lane, from another run, under its gate.

    Neither `land` output here carries a time, so each hold is the start of the
    call that printed it: a lower bound.
    """
    lane = Talk("/trees/r2-1")
    lane.prompt(300_000)
    lane.response((309_000, call("t20", "StructuredOutput")))
    lane.result(309_500, "t20")

    verifier = Talk("/trees/r2-2")
    verifier.prompt(310_100)
    verifier.response((311_000, call("t21", "Bash", command="timeout 600 mise exec -- just verify-lane wait")))
    verifier.result(319_000, "t21")
    verifier.response((319_500, prose()))

    lander = Talk("/trees/r2-3")
    lander.prompt(320_100)
    lander.response((321_000, call("t22", "Bash", command="mise exec -- just land --steps target/land-steps.txt "
                                                          "Q2,Q3 lane-q2-two-rows abc1234 merge-q2-two-rows "
                                                          "/trees/r2-1 /trees/r2-2")))
    lander.result(390_000, "t22", "land: waiting for the land lock\nland: holding the land lock\n"
                  + land_json("merge-q2-two-rows"))
    lander.response((391_000, call("t23", "StructuredOutput")))
    lander.result(391_100, "t23")

    pruner = Talk("/trees/dispatcher")
    pruner.prompt(400_100)
    pruner.response((400_900, prose()))

    run = world.run("wf_r2", REPO, "sess-a", [lane_of("q2-two-rows", "Q2,Q3")])
    run.agent("q2-two-rows", "l2", 300_000, 310_000, lane)
    run.agent("verify:q2-two-rows", "v2", 310_100, 320_000, verifier)
    run.agent("land:q2-two-rows", "d2", 320_100, 400_000, lander, queued=320_000)
    run.agent("prune:q2-two-rows", "p2", 400_100, 401_000, pruner)
    for row in ("Q2", "Q3"):
        world.claim("take", row, 299_000, "lane-q2-two-rows")
        world.claim("release", row, 395_000, "done")

    lane = Talk("/trees/r3-1")
    lane.prompt(300_500)
    lane.response((304_000, call("t30", "StructuredOutput")))
    lane.result(304_100, "t30")

    verifier = Talk("/trees/r3-2")
    verifier.prompt(305_100)
    verifier.response((306_000, call("t31", "Bash", command="just verify-lane abc lane-q4-landed-by-another")))
    verifier.result(314_000, "t31")

    lander = Talk("/trees/r3-3")
    lander.prompt(315_100)
    lander.response((316_000, call("t32", "Bash", command="mise exec -- just land Q4 lane-q4-landed-by-another "
                                                          "abc1234 merge-q4-landed-by-another /trees/r3-1 "
                                                          "/trees/r3-2")))
    lander.result(391_000, "t32", "land: waiting for the land lock\nland: holding the land lock\n"
                                  "land: an earlier run landed lane-q4-landed-by-another, in def5678\n"
                  + land_json("merge-q2-two-rows"))

    pruner = Talk("/trees/dispatcher")
    pruner.prompt(392_100)

    run = world.run("wf_r3", REPO + "-worktrees-other", "sess-c", [lane_of("q4-landed-by-another", "Q4")])
    run.agent("q4-landed-by-another", "l4", 300_500, 305_000, lane)
    run.agent("verify:q4-landed-by-another", "v4", 305_100, 315_000, verifier)
    run.agent("land:q4-landed-by-another", "d4", 315_100, 392_000, lander, queued=315_000)
    run.agent("prune:q4-landed-by-another", "p4", 392_100, 393_000, pruner)
    world.claim("take", "Q4", 300_000, "lane-q4-landed-by-another")
    world.claim("release", "Q4", 380_000, "done")


def a_retried_row(world):
    """Q5: a run killed 4 s in, started again under the same take, handed back, then taken again and handed back.

    The killed run's agent has no `durationMs` and no transcript. The second
    attempt's tree is never seen unlocked, and neither is the third's verifier
    tree; the third's lane tree is unlocked by hand, as before §385.
    """
    killed = world.run("wf_r4", REPO, "sess-a", [lane_of("q5-retried", "Q5")], status="killed",
                       start=500_900, end=505_000)
    killed.agent("q5-retried", "l5a", 501_000, None, None)

    lane = Talk("/trees/r5-1")
    lane.prompt(510_000)
    lane.response((519_000, call("t50", "StructuredOutput")))
    lane.result(519_100, "t50")
    again = world.run("wf_r5", REPO, "sess-a", [lane_of("q5-retried", "Q5")])
    again.agent("q5-retried", "l5b", 510_000, 520_000, lane)

    lane = Talk("/trees/r6-1")
    lane.prompt(601_000)
    lane.response((609_000, call("t60", "StructuredOutput")))
    lane.result(609_100, "t60")
    verifier = Talk("/trees/r6-2")
    verifier.prompt(610_100)
    verifier.response((611_000, call("t61", "Bash", command="git worktree unlock /trees/r6-1")))
    verifier.result(611_200, "t61")
    verifier.response((614_000, call("t62", "StructuredOutput")))
    verifier.result(614_100, "t62")
    third = world.run("wf_r6", REPO, "sess-a", [lane_of("q5-retried-2", "Q5")])
    third.agent("q5-retried-2", "l5c", 601_000, 610_000, lane)
    third.agent("verify:q5-retried-2", "v5", 610_100, 615_000, verifier)

    world.claim("take", "Q5", 500_000, "lane-q5-retried")
    world.claim("take", "Q5", 600_000, "lane-q5-retried-2")
    world.claim("release", "Q5", 700_000, "park")


def a_relayed_lane(world):
    """Q6: a lane relayed once; the relay keeps the lane's own role and joins its attempt (§416)."""
    lane = Talk("/trees/r10-1")
    lane.prompt(700_000)
    lane.response((704_000, call("t70", "StructuredOutput")))
    lane.result(704_100, "t70")

    relay = Talk("/trees/r10-1")
    relay.prompt(706_000)
    relay.response((708_000, call("t71", "StructuredOutput")))
    relay.result(709_100, "t71")

    run = world.run("wf_r10", REPO, "sess-a", [lane_of("q6-relayed", "Q6")])
    run.agent("q6-relayed", "l6", 700_000, 704_100, lane)
    run.agent("relay 1:q6-relayed", "l6b", 706_000, 709_100, relay, queued=705_200)
    world.claim("take", "Q6", 699_000, "lane-q6-relayed")
    world.claim("release", "Q6", 709_000, "done")


def a_prep_refused_row(world):
    """Q7: prep refused the row; only a `prep` agent ran, in the dispatcher's own tree, no lane (§396.4)."""
    prep = Talk("/trees/dispatcher")
    prep.prompt(800_000)
    prep.response((802_000, call("t80", "StructuredOutput")))
    prep.result(802_100, "t80")

    run = world.run("wf_r11", REPO, "sess-a", [lane_of("q7-prep-refused", "Q7")])
    run.agent("prep:q7-prep-refused", "p7", 800_000, 802_100, prep)
    world.claim("take", "Q7", 799_000, "lane-q7-prep-refused")
    world.claim("release", "Q7", 802_000, "sent_back")


def a_close_at_base_row(world):
    """Q8: a lane and its verifier, closed at the base with nothing to land; `close` takes the merger's places (§433.8).

    The closer's one `Bash` call unlocks all three trees at once, so its own
    role is held by its own unlock of its own tree, and the lane's and the
    verifier's are freed by that same call too.
    """
    lane = Talk("/trees/r12-1")
    lane.prompt(900_000)
    lane.response((904_000, call("t90", "StructuredOutput")))
    lane.result(904_100, "t90")

    verifier = Talk("/trees/r12-2")
    verifier.prompt(904_200)
    verifier.response((906_000, call("t91", "Bash", command="mise exec -- just verify-lane abc lane-q8-closed-at-base")))
    verifier.result(909_000, "t91")

    closer = Talk("/trees/r12-3")
    closer.prompt(911_000)
    closer.response((913_000, call("t92", "Bash", command="git worktree unlock /trees/r12-1 && "
                                                          "git worktree unlock /trees/r12-2 && "
                                                          "git worktree unlock /trees/r12-3")))
    closer.result(915_000, "t92")

    run = world.run("wf_r12", REPO, "sess-a", [lane_of("q8-closed-at-base", "Q8")])
    run.agent("q8-closed-at-base", "l8", 900_000, 904_100, lane)
    run.agent("verify:q8-closed-at-base", "v8", 904_200, 909_000, verifier)
    run.agent("close:q8-closed-at-base", "c8", 911_000, 915_000, closer, queued=909_500)
    world.claim("take", "Q8", 899_000, "lane-q8-closed-at-base")
    world.claim("release", "Q8", 915_000, "closed_at_base")


def a_relayed_holder_lands_another_run(world):
    """Q10 and Q11: Q10's lander relays mid-`just land` (§416); its relay lands Q11's lane under its gate.

    The relay's own label is `relay 1:land:q10-relayed-holder`: `role_of` keeps
    its role `lane` and keeps the `land:` prefix in the lane field it computes,
    so it is grouped under neither Q10's own lane (a separate, known gap: its
    time falls to `unattributed` rather than Q10's row) nor found as a lander
    by `agent["lane"]`. `land_lane_of` reads the raw label instead, which is
    what lets Q11's lander still find it as the holder (Q1935).
    """
    lane = Talk("/trees/r20-1")
    lane.prompt(1_000_000)
    lane.response((1_004_000, call("t200", "StructuredOutput")))
    lane.result(1_004_100, "t200")

    verifier = Talk("/trees/r20-2")
    verifier.prompt(1_004_200)
    verifier.response((1_005_000, call("t201", "Bash", command="mise exec -- just verify-lane abc "
                                                                "lane-q10-relayed-holder")))
    verifier.result(1_009_000, "t201")

    lander = Talk("/trees/r20-3")
    lander.prompt(1_010_000)
    lander.response((1_010_900, call("t202", "Bash", command="mise exec -- just land Q10 lane-q10-relayed-holder "
                                                              "abc1234 merge-q10-relayed-holder /trees/r20-1 "
                                                              "/trees/r20-2")))
    lander.result(1_049_000, "t202", "land: waiting for the land lock\nland: holding the land lock\n"
                                     "land: still running (pid 7).")

    relay = Talk("/trees/r20-3")
    relay.prompt(1_051_000)
    relay.response((1_051_900, call("t203", "Bash", command="mise exec -- just land wait")))
    relay.result(1_090_000, "t203", "+ git worktree unlock /trees/r20-1\n+ git worktree unlock /trees/r20-2\n"
                 + land_json("merge-q10-relayed-holder"))

    run = world.run("wf_r20", REPO, "sess-e", [lane_of("q10-relayed-holder", "Q10")])
    run.agent("q10-relayed-holder", "l10", 1_000_000, 1_004_100, lane)
    run.agent("verify:q10-relayed-holder", "v10", 1_004_200, 1_009_000, verifier, queued=1_004_150)
    run.agent("land:q10-relayed-holder", "d10a", 1_010_000, 1_050_000, lander, queued=1_009_500)
    run.agent("relay 1:land:q10-relayed-holder", "d10b", 1_051_000, 1_091_000, relay, queued=1_050_500)
    world.claim("take", "Q10", 999_000, "lane-q10-relayed-holder")
    world.claim("release", "Q10", 1_091_000, "done")

    lane = Talk("/trees/r21-1")
    lane.prompt(1_055_000)
    lane.response((1_059_000, call("t210", "StructuredOutput")))
    lane.result(1_059_100, "t210")

    verifier = Talk("/trees/r21-2")
    verifier.prompt(1_059_200)
    verifier.response((1_060_000, call("t211", "Bash", command="mise exec -- just verify-lane abc "
                                                                "lane-q11-landed-by-relay")))
    verifier.result(1_060_900, "t211")

    lander = Talk("/trees/r21-3")
    lander.prompt(1_061_000)
    lander.response((1_061_900, call("t212", "Bash", command="mise exec -- just land Q11 lane-q11-landed-by-relay "
                                                              "abc1234 merge-q11-landed-by-relay /trees/r21-1 "
                                                              "/trees/r21-2")))
    lander.result(1_070_900, "t212", "land: waiting for the land lock\nland: holding the land lock\n"
                                     "land: an earlier run landed lane-q11-landed-by-relay, in def5678\n"
                  + land_json("merge-q10-relayed-holder"))

    run = world.run("wf_r21", REPO + "-worktrees-other", "sess-f", [lane_of("q11-landed-by-relay", "Q11")])
    run.agent("q11-landed-by-relay", "l11", 1_055_000, 1_059_100, lane)
    run.agent("verify:q11-landed-by-relay", "v11", 1_059_200, 1_060_900, verifier, queued=1_059_150)
    run.agent("land:q11-landed-by-relay", "d11", 1_061_000, 1_071_900, lander, queued=1_060_950)
    world.claim("take", "Q11", 1_054_000, "lane-q11-landed-by-relay")
    world.claim("release", "Q11", 1_071_000, "done")


def a_missing_holder_lands_another_run(world):
    """Q12: landed by a holder whose own run is gone from the retained transcripts, not merely mislabeled (Q1935).

    No agent anywhere in the made-up week carries `q-vanished-holder`, straight
    or relayed, so the search for a candidate comes back empty rather than
    merely late or early.
    """
    lane = Talk("/trees/r22-1")
    lane.prompt(1_100_000)
    lane.response((1_104_000, call("t220", "StructuredOutput")))
    lane.result(1_104_100, "t220")

    verifier = Talk("/trees/r22-2")
    verifier.prompt(1_104_200)
    verifier.response((1_105_000, call("t221", "Bash", command="mise exec -- just verify-lane abc "
                                                                "lane-q12-ghost-holder")))
    verifier.result(1_109_000, "t221")

    lander = Talk("/trees/r22-3")
    lander.prompt(1_110_000)
    lander.response((1_110_900, call("t222", "Bash", command="mise exec -- just land Q12 lane-q12-ghost-holder "
                                                              "abc1234 merge-q12-ghost-holder /trees/r22-1 "
                                                              "/trees/r22-2")))
    lander.result(1_185_900, "t222", "land: waiting for the land lock\nland: holding the land lock\n"
                                     "land: an earlier run landed lane-q12-ghost-holder, in def5678\n"
                  + land_json("merge-q-vanished-holder"))

    run = world.run("wf_r22", REPO, "sess-g", [lane_of("q12-ghost-holder", "Q12")])
    run.agent("q12-ghost-holder", "l12", 1_100_000, 1_104_100, lane)
    run.agent("verify:q12-ghost-holder", "v12", 1_104_200, 1_109_000, verifier, queued=1_104_150)
    run.agent("land:q12-ghost-holder", "d12", 1_110_000, 1_186_900, lander, queued=1_109_500)
    world.claim("take", "Q12", 1_099_000, "lane-q12-ghost-holder")
    world.claim("release", "Q12", 1_186_000, "done")


def a_response_priced_from_its_usage(world):
    """Q13: a lane and a verifier, each priced by `spend.py`'s `cost_of` from its own responses' usage (§411.1).

    The lane's first response streams three records of growing
    `output_tokens`; only the last, which also carries the cache-write split,
    is the one `cost_of_step` must keep, mirroring `spend.py`'s own grouping
    rule. Its second response is one record. The verifier's one response adds
    a third cost. Timing is not the point here, so nothing about the buckets
    or the waits is asserted again.
    """
    lane = Talk("/trees/r30-1")
    lane.prompt(1_200_000)
    lane.response(
        (1_201_000, call("t300", "StructuredOutput"), {"input_tokens": 1, "output_tokens": 10}),
        (1_201_500, thinking(), {"input_tokens": 1, "output_tokens": 60}),
        (1_202_000, prose(), COST_USAGE_1),
        model=COST_MODEL,
    )
    lane.result(1_201_100, "t300")
    lane.response((1_203_000, call("t301", "StructuredOutput"), COST_USAGE_2), model=COST_MODEL)
    lane.result(1_203_100, "t301")

    verifier = Talk("/trees/r30-2")
    verifier.prompt(1_204_000)
    verifier.response((1_205_000, call("t302", "StructuredOutput"), COST_USAGE_3), model=COST_MODEL)
    verifier.result(1_205_100, "t302")

    run = world.run("wf_r30", REPO, "sess-h", [lane_of("q13-known-usage", "Q13")])
    run.agent("q13-known-usage", "l13", 1_200_000, 1_203_200, lane)
    run.agent("verify:q13-known-usage", "v13", 1_204_000, 1_205_200, verifier, queued=1_203_900)
    world.claim("take", "Q13", 1_199_000, "lane-q13-known-usage")
    world.claim("release", "Q13", 1_205_200, "done")


def land_json(merge_branch):
    return json.dumps({"merge_branch": merge_branch, "merged": True, "merged_sha": "def5678", "stop": ""})


# ---------------------------------------------------------------------------
# What row-time must make of it


def assemble(world):
    options = row_time.parsed([world.root, REPO, world.claims_path, "--since", "2026-10-05", "--until", "2026-10-06"])
    loaded = row_time.load(options["root"], options["repo"], options["claims"])
    return row_time.assemble(loaded, options)


def check_a_plain_lane(assembled):
    rows = {row["id"]: row for row in assembled["rows"]}
    steps = assembled["steps"]
    same("the rows picked", sorted(rows), ["Q1", "Q10", "Q11", "Q12", "Q2", "Q3", "Q4", "Q5", "Q6", "Q7", "Q8"])
    lane = steps[("wf_r1", "l1")]
    same("Q1's lane: its buckets, from both pieces of its transcript", dict(lane["buckets"]), {
        "before the first record": 0,
        "model: thinking": 2_000,
        "model: writing other tool input": 11_000,
        "model: writing commands": 3_500,
        "just gate quiet": 25_000,
        "just t": 15_000,
        "git status": 5_000,
        "model: prose": 500,
        "waiting for a message": 8_000,
        "StructuredOutput": 100,
        "after the last record": 100,
    })
    same("Q1's lane: a Read whose result came before its response ended is charged nothing, and keeps its own time",
         (lane["buckets"].get("Read"), lane["own"]["Read"]), (None, [500]))
    same("Q1's lane: own times of the two commands that ran at once",
         (lane["own"]["just t"], lane["own"]["git status"]), ([20_500], [10_000]))
    lander = steps[("wf_r1", "d1")]
    same("Q1's lander: the land lock ends at the time its hold line carries", dict(lander["buckets"]), {
        "before the first record": 0,
        "model: writing commands": 2_000,
        "land lock": 18_000,
        "just land": 69_000,
        "model: writing other tool input": 1_000,
        "StructuredOutput": 100,
        "after the last record": 8_900,
    })
    same("Q1's lander: a hold line with its time is no bound", sorted(lander["marks"]), [])
    row = rows["Q1"]
    same("Q1: the parts of its elapsed time", (row["elapsed"], row["parts"]), (205_000, {
        "waiting to start": 5_000, "agents": 198_600, "handoff": 900, "earlier merge in the same run": 500,
        "between attempts": 0, "after the run": 0}))
    same("Q1: its land lock", row["land lock"], 18_000)
    same("Q1: what each tree held: the lane's until verify-lane, the verifier's until the hold, the lander's to its end",
         [steps[("wf_r1", agent)]["held"] for agent in ("l1", "v1", "d1")], [71_000, 39_600, 99_000])
    same("Q1: its slot time, the wait to start included", row["slot"], 5_000 + 71_000 + 39_600 + 99_000)
    same("Q1: no marks", sorted(row["marks"]), [])


def check_a_bundle_and_a_shared_gate(assembled):
    rows = {row["id"]: row for row in assembled["rows"]}
    steps = assembled["steps"]
    for row_id in ("Q2", "Q3"):
        row = rows[row_id]
        same(f"{row_id}: the bundle's parts", (row["elapsed"], row["parts"]), (102_000, {
            "waiting to start": 1_000, "agents": 100_700, "handoff": 300, "earlier merge in the same run": 0,
            "between attempts": 0, "after the run": 0}))
    same("the bundle's lane serves two rows", steps[("wf_r2", "l2")]["served"], 2)
    same("the holder's lander serves the bundle's two rows and the lane it landed beside them",
         (steps[("wf_r2", "d2")]["served"], steps[("wf_r2", "d2")].get("landed_for")), (3, ["q4-landed-by-another"]))
    holder = steps[("wf_r2", "d2")]
    same("a hold line with no time ends the land lock at the start of the call that printed it",
         (holder["hold"], holder["buckets"].get("land lock"), holder["buckets"]["just land"]), (BASE + 321_000, None, 69_000))
    same("and is marked a bound", sorted(holder["marks"]), ["land lock bound"])
    landed = steps[("wf_r3", "d4")]
    same("a lane another run's lander landed names that run", landed["landed_by"],
         {"run": "wf_r2", "lane": "q2-two-rows"})
    same("and all its land time was the wait for the lock", dict(landed["buckets"]), {
        "before the first record": 0, "model: writing commands": 900, "land lock": 75_000,
        "after the last record": 1_000})
    row = rows["Q4"]
    same("Q4: the parts of its elapsed time", (row["elapsed"], row["parts"]), (93_000, {
        "waiting to start": 500, "agents": 92_200, "handoff": 300, "earlier merge in the same run": 0,
        "between attempts": 0, "after the run": 0}))
    same("Q4: its land lock", row["land lock"], 75_000)
    same("Q4: its verifier's tree is held until its own lander's hold, a bound",
         steps[("wf_r3", "v4")]["held"], 316_000 - 305_100)


def check_a_retried_row(assembled):
    rows = {row["id"]: row for row in assembled["rows"]}
    steps = assembled["steps"]
    row = rows["Q5"]
    same("Q5: three attempts, the second with no take of its own",
         [attempt["take"] for attempt in row["attempts"]], [BASE + 500_000, None, BASE + 600_000])
    same("Q5: the parts of its elapsed time", (row["elapsed"], row["parts"]), (200_000, {
        "waiting to start": 2_000, "agents": 27_900, "handoff": 100, "earlier merge in the same run": 0,
        "between attempts": 85_000, "after the run": 85_000}))
    same("the killed run's agent: no durationMs, so it ran to its run's end, and no transcript",
         dict(steps[("wf_r4", "l5a")]["buckets"]), {"no transcript": 4_000})
    same("Q5: trees never seen unlocked are held until their run's last agent ends, and the row says so",
         ([steps[key]["held"] for key in (("wf_r4", "l5a"), ("wf_r5", "l5b"), ("wf_r6", "l5c"), ("wf_r6", "v5"))],
          sorted(row["marks"])), ([4_000, 10_000, 10_000, 4_900], ["still locked when the run ended"]))
    same("Q5: its slot time", row["slot"], 1_000 + 4_000 + 10_000 + 1_000 + 10_000 + 4_900)


def check_a_relayed_lane(assembled):
    rows = {row["id"]: row for row in assembled["rows"]}
    row = rows["Q6"]
    same("Q6: the relay joins the lane's own attempt, both role lane",
         [agent["role"] for attempt in row["attempts"] for agent in attempt["agents"]], ["lane", "lane"])
    same("Q6: the parts of its elapsed time", (row["elapsed"], row["parts"]), (10_100, {
        "waiting to start": 1_000, "agents": 7_200, "handoff": 1_900, "earlier merge in the same run": 0,
        "between attempts": 0, "after the run": 0}))
    same("Q6: no tree is ever seen unlocked, so both are held to the run's end",
         (row["slot"], sorted(row["marks"])), (1_000 + 9_100 + 3_100, ["still locked when the run ended"]))


def check_a_prep_refused_row(assembled):
    rows = {row["id"]: row for row in assembled["rows"]}
    row = rows["Q7"]
    same("Q7: only a prep agent ran, no lane", [agent["role"] for attempt in row["attempts"]
                                               for agent in attempt["agents"]], ["prep"])
    same("Q7: the parts of its elapsed time", (row["elapsed"], row["parts"]), (3_100, {
        "waiting to start": 1_000, "agents": 2_100, "handoff": 0, "earlier merge in the same run": 0,
        "between attempts": 0, "after the run": 0}))
    same("Q7: prep holds no tree of its own, so its slot is only the wait to start", row["slot"], 1_000)


def check_a_close_at_base_row(assembled):
    rows = {row["id"]: row for row in assembled["rows"]}
    row = rows["Q8"]
    same("Q8: lane, verifier and close, close in the merge stage",
         [agent["role"] for attempt in row["attempts"] for agent in attempt["agents"]],
         ["lane", "verifier", "close"])
    same("Q8: the parts of its elapsed time, close's wait split like a lander's or a merger's",
         (row["elapsed"], row["parts"]), (16_000, {
             "waiting to start": 1_000, "agents": 12_900, "handoff": 1_600,
             "earlier merge in the same run": 500, "between attempts": 0, "after the run": 0}))
    same("Q8: no marks; every tree is freed by the closer's one unlock call", sorted(row["marks"]), [])
    same("Q8: its slot time: the lane freed by the verifier's call, the verifier by the closer's unlock, "
         "the closer by its own unlock of its own tree", row["slot"], 1_000 + 6_000 + 8_800 + 2_000)


def check_a_relayed_holder_lands_another_run(assembled):
    rows = {row["id"]: row for row in assembled["rows"]}
    steps = assembled["steps"]
    same("Q11: landed by a holder lander that itself relayed mid-`just land`, found by its raw label",
         steps[("wf_r21", "d11")]["landed_by"], {"run": "wf_r20", "lane": "q10-relayed-holder"})
    same("Q11: all its land time became the wait for the lock", dict(steps[("wf_r21", "d11")]["buckets"]), {
        "before the first record": 0, "model: writing commands": 900, "land lock": 9_000,
        "after the last record": 1_000})
    row = rows["Q10"]
    same("Q10: its own row holds only the lane, verifier and the lander's pre-relay segment",
         [agent["role"] for attempt in row["attempts"] for agent in attempt["agents"]],
         ["lane", "verifier", "lander"])
    same("Q10: the relay's own time is unattributed, a separate gap from the one this row fixes",
         [item for item in assembled["unattributed"] if item["lane"] == "land:q10-relayed-holder"],
         [{"run": "wf_r20", "lane": "land:q10-relayed-holder", "agents": 1, "time": 40_000}])


def check_a_missing_holder_lands_another_run(assembled):
    steps = assembled["steps"]
    landed = steps[("wf_r22", "d12")]
    same("Q12: a holder whose own run cannot be found gets a landed_by naming no run",
         landed["landed_by"], {"run": None, "lane": "q-vanished-holder"})
    same("Q12: its land time still folds into the land lock, marked rather than resolved",
         (dict(landed["buckets"]), sorted(landed["marks"])),
         ({"before the first record": 0, "model: writing commands": 900, "land lock": 75_000,
           "after the last record": 1_000},
          ["land lock bound", "landed by q-vanished-holder, whose own run is not in the retained transcripts"]))


def check_every_step_adds_up(assembled):
    for key, step in assembled["steps"].items():
        same(f"{key}: its buckets add up to its running time", sum(step["buckets"].values()), step["time"])


def check_the_printed_report(world):
    printed = io.StringIO()
    with contextlib.redirect_stdout(printed):
        code = row_time.main([world.root, REPO, world.claims_path, "--since", "2026-10-05", "--until", "2026-10-06"])
    same("the report prints and exits 0", code, 0)
    text = printed.getvalue()
    same("the report names eleven rows", text.splitlines()[0], "row-time: 11 rows, started 2026-10-05T00:00:00Z to "
                                                                "2026-10-06T00:00:00Z")
    printed = io.StringIO()
    with contextlib.redirect_stdout(printed):
        row_time.main([world.root, REPO, world.claims_path, "Q4", "--json"])
    report = json.loads(printed.getvalue())
    same("--json for one row", ([row["row"] for row in report["rows"]], report["rows"][0]["land lock"]), (["Q4"], 75_000))
    same("--json names the run that landed it", report["rows"][0]["attempts"][0]["steps"][2]["landed by"],
         {"run": "wf_r2", "lane": "q2-two-rows"})


def check_parts_that_do_not_add_up(scratch):
    """A verifier that starts before its lane ends: the row's parts cannot add up, and row-time must stop."""
    world = World(scratch)
    lane = Talk("/trees/r9-1")
    lane.prompt(1_000)
    verifier = Talk("/trees/r9-2")
    verifier.prompt(5_000)
    run = world.run("wf_r9", REPO, "sess-a", [lane_of("q9-overlap", "Q9")])
    run.agent("q9-overlap", "l9", 1_000, 10_000, lane)
    run.agent("verify:q9-overlap", "v9", 5_000, 12_000, verifier)
    world.claim("take", "Q9", 0, "lane-q9-overlap")
    world.write()
    printed, said = io.StringIO(), io.StringIO()
    with contextlib.redirect_stdout(printed), contextlib.redirect_stderr(said):
        code = row_time.main([world.root, REPO, world.claims_path, "Q9"])
    same("parts that do not add up: exit 1", code, 1)
    same("parts that do not add up: nothing printed", printed.getvalue(), "")
    same("parts that do not add up: the error names the row", said.getvalue().startswith("row-time: Q9: handoff runs "
                                                                                        "backwards"), True)


def check_command_keys():
    cases = [
        ("cd /trees/x && mise exec -- just gate quiet | tail -5", "just gate quiet"),
        ("mise exec -- just t how_often_a_fuse_with_a_cone_refuses", "just t"),
        ("mise exec -- just fmt && mise exec -- just t -p rcad-geom; mise exec -- just fmt", "just fmt+t"),
        ("timeout 600 mise exec -- just land wait", "just land"),
        ("mise exec -- just land --steps target/s.txt Q1 lane-q1 abc merge-q1 /t/1", "just land"),
        ("mise exec -- just verify-lane wait", "just verify-lane"),
        ("mise exec -- just queue take Q1 lane-q1", "just queue take"),
        ("mise exec -- just decision 411", "just decision"),
        ("A=1 B=2 git -C /trees/x log --oneline -3", "git log"),
        ("cargo +nightly build -p x", "cargo build"),
        ("grep -n foo bar | head\nls target", "grep"),
        ("cat > /tmp/x <<'EOF'\nit's a heredoc\nEOF", "cat"),
        ("git commit -q -m \"$(cat <<'EOF'\nit's\nEOF\n)\" 2>&1", "git commit"),
    ]
    for command, want in cases:
        same(f"the key of {command!r}", row_time.command_key(command), want)


def check_a_cut_off_line():
    whole = json.dumps({"parentUuid": "b", "type": "user", "timestamp": at(0)})
    records, lost = row_time.records_of_line('{"parentUuid":"a","type":"attachm' + whole)
    same("a line cut off mid-record keeps the record after the cut", (records, lost), ([json.loads(whole)], 1))


# ---------------------------------------------------------------------------
# Writing the fixtures


class World:
    """A projects root with run records and transcripts under it, and a claims file beside it."""

    def __init__(self, root):
        self.root = root
        self.claims_path = os.path.join(root, "claims")
        self.runs = []
        self.claims = []

    def run(self, run_id, folder, session, lanes, status="completed", start=None, end=None):
        run = Run(run_id, folder, session, lanes, status, start, end)
        self.runs.append(run)
        return run

    def claim(self, verb, row, ms, last):
        if verb == "take":
            self.claims.append(f"take\t{row}\t{second(ms)}\t/trees/dispatcher\t{last}\t-")
        else:
            self.claims.append(f"release\t{row}\t{second(ms)}\t/trees/dispatcher\t{last}")

    def write(self):
        for run in self.runs:
            run.write(self.root)
        with open(self.claims_path, "w", encoding="utf-8") as f:
            f.write("add\tQ0\t2026-09-26T09:29:46Z\t/trees/dispatcher\tQ0 1 todo opus - first event\n")
            f.write("".join(line + "\n" for line in self.claims))


class Run:
    def __init__(self, run_id, folder, session, lanes, status, start, end):
        self.id, self.folder, self.session, self.lanes, self.status = run_id, folder, session, lanes, status
        self.start, self.end = start, end
        self.agents = []

    def agent(self, label, agent_id, start, end, talk, queued=None, split_into=None):
        self.agents.append((label, agent_id, start, end, talk, start if queued is None else queued, split_into))

    def write(self, root):
        start = self.start if self.start is not None else min(agent[2] for agent in self.agents)
        end = self.end if self.end is not None else max(agent[3] for agent in self.agents)
        progress = []
        for index, (label, agent_id, agent_start, agent_end, talk, queued, split_into) in enumerate(self.agents):
            entry = {"type": "workflow_agent", "index": index + 1, "label": label, "agentId": agent_id,
                     "state": "done" if agent_end is not None else "progress",
                     "queuedAt": BASE + queued, "startedAt": BASE + agent_start}
            if agent_end is not None:
                entry["durationMs"] = agent_end - agent_start
            progress.append(entry)
            if talk is not None:
                talk.write(root, self, agent_id, split_into)
        record = {"runId": self.id, "workflowName": "lanes", "status": self.status, "startTime": BASE + start,
                  "durationMs": end - start, "args": {"base": "abc1234", "lanes": self.lanes},
                  "workflowProgress": [{"type": "workflow_phase", "index": 1, "title": "Work"}] + progress}
        folder = os.path.join(root, self.folder, self.session, "workflows")
        os.makedirs(folder, exist_ok=True)
        with open(os.path.join(folder, f"{self.id}.json"), "w", encoding="utf-8") as f:
            json.dump(record, f)


def lane_of(label, rows):
    return {"label": label, "queueId": rows, "brief": "do it", "model": "opus"}


class Talk:
    """One agent's transcript, written a record at a time."""

    def __init__(self, cwd):
        self.cwd = cwd
        self.records = []
        self.cut_at = None
        self.requests = 0

    def prompt(self, ms, text="the brief"):
        self.records.append({"type": "user", "timestamp": at(ms), "cwd": self.cwd,
                             "message": {"role": "user", "content": text}})

    def attachment(self, ms):
        self.records.append({"type": "attachment", "timestamp": at(ms), "cwd": self.cwd,
                             "attachment": {"type": "hook_success"}})

    def response(self, *blocks):
        self.requests += 1
        for ms, block in blocks:
            self.records.append({"type": "assistant", "timestamp": at(ms), "cwd": self.cwd,
                                 "requestId": f"req_{self.requests}",
                                 "message": {"id": f"msg_{self.requests}", "role": "assistant", "content": [block]}})

    def result(self, ms, tool_id, text="done"):
        self.records.append({"type": "user", "timestamp": at(ms), "cwd": self.cwd, "message": {
            "role": "user", "content": [{"type": "tool_result", "tool_use_id": tool_id, "content": text}]}})

    def cut(self):
        """Ends the first piece here; the rest goes to another project folder's session."""
        self.cut_at = len(self.records)

    def write(self, root, run, agent_id, split_into):
        """Writes the records in time order, as Claude Code does, in one piece or two."""
        pieces = [(run.folder, run.session, self.records)]
        if split_into is not None:
            assert self.cut_at is not None, "a split transcript needs a cut"
            pieces = [(run.folder, run.session, self.records[:self.cut_at]),
                      (split_into[0], split_into[1], self.records[self.cut_at:])]
        for folder, session, piece in pieces:
            where = os.path.join(root, folder, session, "subagents", "workflows", run.id)
            os.makedirs(where, exist_ok=True)
            in_order = sorted(piece, key=lambda record: record["timestamp"])
            with open(os.path.join(where, f"agent-{agent_id}.jsonl"), "w", encoding="utf-8") as f:
                f.write("".join(json.dumps(record) + "\n" for record in in_order))


def thinking():
    return {"type": "thinking", "thinking": ""}


def prose():
    return {"type": "text", "text": "..."}


def call(tool_id, name, **tool_input):
    return {"type": "tool_use", "id": tool_id, "name": name, "input": tool_input}


if __name__ == "__main__":
    sys.exit(main())
