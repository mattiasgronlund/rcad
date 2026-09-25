# Test plan

What the tests search, and what to change so they find more. `docs/plan.md`
carries this as roadmap 11.3, *finding hard cases*, `partly`, and points here.

**The top open row always has a lane.** Whenever lanes are dispatched, one of
them takes the highest row still `todo`, whatever unit §0 names. A row here
frees no corpus file, so it never wins a ranking by what a lane frees
(§171.5); a reserved lane is the only way it gets one. `docs/decisions.md`
§226 says why, and what got stuck before this file existed.

One line per change, highest priority first. The number is a name, not a rank —
the order of the lines is the rank. The states are **todo**, **lane** (name the
branch), **done** (name the commit and the evidence) and **dropped** (name the
section that dropped it). A section that ends by naming a change to what the
tests search adds a row here in the same commit.

## Open

```
T1  lane   lane-t1-fuzz-runs — The cone-or-torus target fell from 160 M runs a night to 2.3 M between the nights of 2026-09-20 and 09-21, same flags; and the fuse gets 2.0 M runs on 20 cores where it got 3.0 M on one. Say why, for each. §226.3
T2  lane   lane-t2-fuse-both-orders — The fuse target asks each pair both ways round and compares the two answers, the way fuses_that_must_agree.rs's commutativity does. §129.5 item 3
T3  lane   lane-t3-fuzz-timeouts — Decode the first timeouts the fuse ever wrote (three, 2026-09-24/25) and the cone-or-torus one (2026-09-25): what each input is, how long it takes on main, a hang or only slow. §226.3
T4  todo   The morning line says what is new tonight, by kind and target, off the per-night counters, instead of "nothing found" beside a pile that grows by ~700 a night. §226.3
T5  todo   The writer target reads back what it wrote and compares it with the model that went in. §129.5 item 3
T6  todo   A dictionary of the STEP entity names Import::read handles, given to the reader target with -dict=. §226.1
T7  todo   Seed the reader target from the real STEP files under RCAD_STEP_CORPUS; its corpus is all fuzzer-grown. §226.1
T8  todo   A gate check that holds this file's rows to their shape, the way check-plan holds docs/plan.md's. §226.4
```

## Done

```
D1  done   0ac02509 §129 — the five fuzz targets run at once, each for the whole night
D2  done   22684dad §134 — fork mode: a crash no longer ends a target's night
D3  done   52ea7984 §135 — one input gets ten seconds
D4  done   e115b2ed §191 — three families aimed where KiCad found bugs: rounded STEP text, turned curved solids, solids that touch
D5  done   a736e351 §186 — the night fuzzes the tip of main, not the last green
```
