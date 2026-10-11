## 416. A writing lane past its context budget hands its branch to a fresh lane — 2026-10-08

*Mattias asked on 2026-10-08 why lanes run over budget and how a lane's job
could be split into smaller pieces, then asked for the two recommended changes
to be built and landed. Measured and built in the dispatcher session of
`dispatch-2026-10-03`.*

In short: four lanes in five now pass the 150k lane budget. About half of a
lane's growth is reading the code it is about to change, and those files have
nearly tripled since 09-15. A lane makes its first commit at the very end, so
one stopped at the warning hands on nothing, and briefs told lanes to carry on
past it instead. Now a writing lane at the warning commits, lets go of its
branch and reports `relay`, and `lanes.js` starts a fresh lane on that branch,
at most three times a row. `lane.md` also has a lane commit its failing test
first, because what a lane commits is what the next one starts from.

  416.1   What was measured
  416.2   What a split buys
  416.3   What is built
  416.4   What is not built, and why

### 416.1 What was measured

The transcripts of the 354 Opus lanes since 10-01: each
`subagents/**/agent-*.jsonl` whose `.meta.json` says `agentType` lane and model
opus. A response's context is its input plus cache read plus cache write, one
response per message id. A lane's peak is the largest of them.

- **283 of 354 passed 150k.** The median peak was 203k, p90 269k, the largest
  413k. §170.3 measured a median of 134k for 09-12 to 09-21. The Sonnet
  verifiers are not the problem: median 85k, 1 of 346 past 150k.
- **About 57% of a lane's growth is reading code.** Each response's growth to
  the next, less its own output, is charged to the tool results it got back:
  `Read` 31%, grep 16%, `sed -n` 5%, `just src` 4%. The lane's own output and
  thinking is 14%, `just t` 7%, `Edit` 3%.
- **The reading is not waste.** Lines read a second time are 3.6% of what
  `Read` returned. 69% of the code a lane reads before its first edit is in
  files it then edits.
- **The files grew.** `holds.rs`, `fuse/arrange.rs`, `fuse/assemble.rs`,
  `mesh.rs` and `fuse/select.rs` were 17.6k lines together on 09-15, 28.6k on
  10-01 and 46.9k on 10-08 (`git show <sha>:<file> | wc -l`). Lanes that read
  one of them peaked at a median of 210k, 87% past 150k. Lanes that read none
  peaked at 166k, 56% past. Those are also the hard curved-fuse rows, so this
  does not separate the size of the files from the size of the rows.
- **The commit comes last.** The medians: first test run at 102k, first edit
  at 109k, first commit at 203k, first gate at 204k. After the gate starts, a
  lane grows a median of 9k more.
- **325 of 354 briefs** told the lane to carry on past the warning, as the
  memory note written after §235 said to, because a lane stopped at 150k had
  nothing committed.
- **Big lanes come back unfinished more often.** `partial` came back for 23%
  of the lanes that peaked between 150k and 250k, and 43% of those above 250k.

*§437.8 corrected the shares in the second bullet on 2026-10-11. A transcript
often records a response's output as a count taken early in the stream, so the
lane's own output was charged to the tool results after it. Estimated properly,
own output and thinking is about 38% of the growth; `Read` stays at 31%, and
grep falls to 7.3%, `sed -n` to 3.2%, `just src` to 1.8% and `just t` to 4.7%.
The bullet's point stands: reading code is the largest cost.*

### 416.2 What a split buys

Each lane was replayed as two agents, cut at a point. The cost is the sum, over
responses, of the context each read. The second agent starts at the lane's
start plus R tokens of re-reading, and then grows as the lane did after the
cut. One cut each:

| Cut | R | Cost against the lane as run | Lanes with a piece past 150k |
|---|---|---|---|
| at 150k | 20k | 0.66 | 58 |
| at 150k | 40k | 0.73 | 91 |
| at 150k | 60k | 0.79 | 150 |
| at the first edit | 20k | 0.70 | 175 |
| at the first edit | 60k | 0.89 | 264 |

As run, 283 lanes passed 150k. R was measured on rows that went out twice:
Q712-2 re-read 14k before its first edit, Q753-2 25k and Q649-2 26k, each after
a first lane that had committed code. Q1287 re-read 60k and Q1268 105k after a
first lane that committed nothing and whose findings were written into the
brief. Five rows is few, but the gap is a factor of three or more.

So the cut is at the warning, and a commit is what keeps R small. A cut between
investigating and fixing saves less, and leaves the fixing half past 150k for
most lanes.

### 416.3 What is built

- **`lanes.js`.** `worked()` runs the lane, and while it reports `relay`
  starts a fresh lane on its branch, up to `MAX_RELAYS`, 3. A relay's prompt
  holds the handoff, the row's brief, and its first commands: `git switch` onto
  the branch (not `-c`), `git rev-parse HEAD`, the ancestor check against the
  base, and `git worktree unlock` of the tree before. If the lane before did
  not let go of the branch, the relay runs `git -C <that tree> switch --detach`
  and switches again. The relay's report is the row's whole report: it carries
  forward each number, test, row step and file of the lane before that still
  holds. The run hands the row back instead of verifying it when a lane asks for
  a relay with no handoff, when a relay starts at another commit than the one
  handed over (`continued_from` against `landed_sha`), or when a lane asks for
  a fourth relay. The returned lines then carry the last handoff and its sha.
  The trees a row was relayed from count among its trees, so the lander and the
  merger unlock them and mark them finished.
- **`lane.md`.** Commit the failing test first, on its own, then the fix. When
  "Lane budget passed" comes with the row not done, hand over: commit
  everything, `git switch --detach`, report `relay` with a handoff that names
  places. This holds even when a brief says to carry on past the warning. A
  read-only lane has nothing to commit and reports what it has, as before.
- **The brief carries the rule too (Q1422).** A `claude` process keeps the
  agent types it started with through a `/clear`. A dispatcher cleared after
  this landed reads the new `dispatch-lanes` and memory, stops telling its
  lanes to carry on, and sends them with a `lane.md` that has no relay, so
  they would stop at 150k with nothing handed on. `lanes.js` is read from the
  tree at each call, so its `handOver()` sentence goes into every lane's brief
  and every relay's.
- **`dispatch-lanes`.** A writing lane's brief says nothing about the context
  hook. "Carry on past it" stays for a read-only brief that builds a probe or
  reads a listing.
- **`check-lanes-workflow`, a twenty-second arm, with eleven cuts.** Q1412's
  §414 arm landed first and took the twenty-first, so Q1415's row text, which
  says twenty-first, is one off. It fails
  on `main`'s `lanes.js` and `lane.md` with its first message and passes here.
  It was run against main's files by `git archive origin/main .claude CLAUDE.md
  tooling/verify-lane` into a scratch repo with the new justfile. `lanes.js`
  was also run under node against a stubbed `agent()`, which is not checked in
  because the repo pins no node. Six cases: two relays and then done, the
  verifier given the last report and all three lane trees counted among the
  row's trees; a fourth relay
  refused; a relay with an empty handoff refused before any relay started; a
  relay that started at another commit refused; a relay blocked at its switch,
  handed back with the last handoff; no relay at all, as before.

### 416.4 What is not built, and why

- **A read-only lane does not relay.** It commits nothing, so its handoff
  would be prose only, which re-read 60k and 105k in the two cases measured.
- **No investigate-then-fix stages.** See §416.2.
- **The big files are not split.** The 210k against 166k gap could be the
  rows, not the files, and a split would conflict with every lane working in
  them. A lead, not a finding.
- **The guardrails hook is unchanged.** It already tells a lane to commit and
  to report "what you learned that the next lane would otherwise rediscover",
  which is the handoff. Its last sentence lets a brief that says carry on
  override it; `lane.md` says the relay wins.
- **No real relay has run yet.** What to read on the first ones: whether the
  handoff names places, R against the 14–26k measured, and whether a verifier
  reads a branch made by several lanes well. `just row-time` shows a relayed
  row as more than one lane agent under one row.
