## 311. Defects outrank finding defects better: queue tiers 1 and 2 swap — 2026-09-27

*Mattias, 2026-09-27, in the session that asked how many nightly-fuzz finds had
been fixed. Amends §230.1's order. §307's tier 2.2 stays below both, as it was.*

### 311.1 What changed

Tier 1 is now **new defects**, §230.1's old tier 2, and tier 2 is now **finding
defects better**, its old tier 1. The rest of §230.1's table stands, so the
order is:

| tier | what |
|---|---|
| 1 | New defects: a wrong answer, a crash, a regression, a wrong answer kept behind `#[ignore]`, a defect a lane reports, a fuzz find that still fails on `main` |
| 2 | Finding defects better: what the tests and the nights search |
| 2.2 | How the work is done (§307) |
| 2.5 | Long execution times |
| 3 | STEP corpus files that fail |
| 4 | New functionality, under §13 |

A lane that fixes a wrong answer the kernel gives now gets a slot before a lane
that measures the fuzzer.

### 311.2 Why

Asked how many nightly-fuzz finds had been fixed in the 22 hours to 20:27Z on
09-27, the answer was none. The 09-26 night's triage filed Q166–Q172 in tier 2
at 11:30Z on 09-26 (the claims file's `add` events), and none had been taken.
In the same 22 hours 21 rows about the fuzzer itself closed: 7 changed the night
(Q161, Q156, Q237, Q263, Q53, Q232, and Q179, a comment), 13 measured it and
built nothing (Q148, Q162, Q155, Q178, Q214, Q157, Q188, Q175, Q238, Q244, Q239,
Q236, Q261), and one copied data out of `/tmp` (Q212). No commit to `crates/` in
that window came from a fuzz find. The fill did what §230.1 told it: tier 1
took every slot it could fill.

**Tier 1 refilled itself.** A lane's "what this leaves" becomes rows (§230.7,
`merger.md` step 4), and a follow-up to a lane that measured the search is
itself finding defects better, so it went back into tier 1. Between 09-26
09:29Z, when the claims file starts, and 09-27 20:32Z, 96 rows were added in
tier 1 and 38 in tier 2; 56 of the 96 came from a merger's tree. Q236's merger
added Q277–Q280, and Q261's added Q282, each citing the section the lane had
just written. Mattias had expected the fuzz work to fade unless he brought new
ideas for the night's yield. Under §230.1 it grew instead, and it outranked
every fix.

### 311.3 What moved, and what did not

- 79 open rows went from tier 2 to 1 and 26 from 1 to 2, each through `just
  queue tier`'s own code, so `check-queue` passed after every move. Each tier
  keeps its rows' order.
- The eight fuzz-find rows went first — Q166–Q172 from the 09-26 night, then
  Q7 — so they head the new tier 1's ready rows. Mattias did not ask for that
  ranking. It follows from the question that led here, and `just queue rank`
  undoes it.
- Rows held by a lane stayed where they were: Q287, which is in tier 1 at
  Mattias's word (§309), and Q276, Q270, Q177, Q268 and Q250. Each closes into
  `## Closed` shortly. A row one of them gives back as `todo` needs `just queue
  tier` by hand.
- Closed rows keep the tier they closed in. That field is history.

### 311.4 What this leaves

- `merger.md` step 4 and the `dispatch-lanes` skill name the new order. A
  merger started before this change still files by §230.1's, so a follow-up
  that measures the search can land in tier 1 until the running lanes are done.
  Move it with `just queue tier <id> 2`.
- Follow-up rows about the fuzzer are still filed; now they wait in tier 2
  behind the defects. Filing them only when Mattias asks would mean changing
  §230.7's rule that everything a lane leaves becomes a row. That is not done
  here.
