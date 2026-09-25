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
D6  done   d2f1b4be §228 — the fuse's first timeouts decoded: two are one correct refusal that takes eight minutes; two finish in seconds here, why the night stopped them is open
D7  done   e14b14ab §227 — T1: both run counts fell because the corpus reached slow inputs, not the kernel or fork mode
D8  done   7e9d1b2c §129.5 — T2: the fuse target asks every two-sided union and intersection both ways round; no disagreement on the replay
```
