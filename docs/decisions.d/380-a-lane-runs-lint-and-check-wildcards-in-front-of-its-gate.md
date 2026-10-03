## 380. A lane runs lint and check-wildcards in front of its gate — 2026-10-03

*Not a queue row. Mattias asked where the time in a `lanes` run goes, then
asked for the pros and cons of two fixes for gates that fail only on the cheap
checks. This section records the measurements and the choice. It changes
`.claude/agents/lane.md` and nothing else.*

In short: a lane now gates with `mise exec -- just lint check-wildcards gate
quiet`. `just` runs the three recipes in turn and stops at the first that
fails, so a clippy warning or a new wildcard arm comes back in about 16 s, not
after a whole gate. On a gate that passes, the two checks add about 9 s. The
gate itself is unchanged, and §372.9 still holds.

### 380.1 What the cheap failures cost

The 14 `lanes` runs from `wf_4373c43c-235` (Q740, 2026-10-02 evening) to
`wf_cfa0186e-82d` (Q774, 2026-10-03 morning) took 392 minutes and had 24 lane
gates. Four of those gates failed only on `check-wildcards` or `lint`, and the
tests passed in all four:

| Row  | Run               | Gate  | Failed on                 |
|------|-------------------|-------|---------------------------|
| Q763 | `wf_65d9ee08-4c2` | 99 s  | `check-wildcards`         |
| Q766 | `wf_0feaf8af-cd7` | 161 s | `check-wildcards`, `lint` |
| Q767 | `wf_c84930ee-141` | 164 s | `check-wildcards`, `lint` |
| Q767 | `wf_c84930ee-141` | 157 s | `check-wildcards`         |

That is 581 s. Q649's lane gates also failed, but on a test, so they are not in
it. The numbers come from reading each run's agent transcripts for the `just
gate` calls, with each call's time and summary block. The method is in memory
`timing-a-workflow-run-from-its-transcripts`.

### 380.2 The two options

**A rule in `lane.md`: run the two checks before the gate.** The gate still
runs every step afterwards, so nothing is hidden. The cost is paid on every
gate, including the 20 of 24 that would have passed. It reaches only lanes, and
only new `claude` processes.

**A gate that stops at a cheap failure.** It costs nothing on a pass and
reaches every role. But stopping is not enough. §372.9 says the cargo chain and
the slowest selftest start in the first second, so the gate must kill the steps
that are already running. Killed steps report nothing. A lane that broke both a
lint and a test would then find the test failure one gate later, which costs
about what the early stop saved. CI runs the same command, so it would lose its
full report or need a mode of its own. The selftests that pin the gate
(`check-gate-runs-every-check` among them) would have to change. All of that
buys at most 581 s in 392 minutes.

The rule pays off if it costs less than about 24 s a gate (581 s over 24
gates). 380.3 measures it at about 9 s.

### 380.3 What the two checks cost after an edit

The worktree was warmed with `just gate quiet fresh` at `260bbd9c`. Each run
then changed one word in the first line of `crates/rcad-topo/src/lib.rs`, a new
word every time. `rcad-topo` is the crate lanes changed most in the four days
before. In runs A the lane runs `just lint`, then `just check-wildcards`, then
`just gate quiet`. In runs B it runs `just gate quiet` alone. Run C is 380.4's
other order. Before each run, clippy's last run used `lint`'s flags, the way a
gate leaves it.

| Run | before the gate              | gate    | gate's `check-wildcards` | gate's `lint` | load |
|-----|------------------------------|---------|--------------------------|---------------|------|
| A1  | `lint` 9.6 s, `check-wildcards` 7.7 s | 142.5 s | 6.2 s       | 9.4 s         | 6.1  |
| A2  | `lint` 7.4 s, `check-wildcards` 7.3 s | 138.5 s | 6.0 s       | 10.4 s        | 6.7  |
| A3  | `lint` 7.5 s, `check-wildcards` 8.1 s | 108.7 s | 6.0 s       | 9.8 s         | 9.7  |
| B1  | —                            | 113.8 s | 12.9 s                   | 9.6 s         | 9.6  |
| B2  | —                            | 117.9 s | 13.3 s                   | 10.7 s        | 22.0 |
| C1  | `check-wildcards` 7.6 s, `lint` 9.2 s | 115.4 s | 12.4 s      | 10.0 s        | 2.4  |

The two checks take about 16 s together. In return, the gate's own
`check-wildcards` drops from about 13 s to about 6 s. In A3, clippy re-checked
no crate in that step: the log it writes, `target/last-wildcard-clippy.log`,
has no `Checking` line. The same recipe on unchanged code, run alone, took
1.1 s. In the gate it runs beside the gate's other steps, and the gap between
1.1 s and 6 s was not looked into further. The gate's `lint` re-checked all 8
crates in every run whose log was kept (A2, A3, B2 and C1), because cargo counts clippy's flags as part
of what it caches (`check-wildcards`'s recipe comment records that this was
confirmed). So a passing gate costs about 9 s more.

A1 and A2's whole-gate totals are high because of two numbers that did not
repeat. `check-ignored` took 41.0 s in A1 and 9.4 to 10.9 s in every other run.
`test` took 94.8 s in A2 and 66.2 to 67.4 s in every other run; the load was 22
when B2 started, so other sessions were busy during the end of A2. A3 has
neither, and its gate is 5 to 9 s shorter than B1 and B2. The choice does not
depend on this anyway: if the gate got no faster at all, the rule would cost
16 s a gate, under the 24 s it can afford.

On the 24 gates of 380.1 the rule would have saved about 129 s on each of the
four (a 16 s check in place of a 145 s gate), 517 s in all. It would have cost
about 9 s on each of the other 20, about 180 s. That is about five and a half
minutes saved over 14 runs. It breaks even when about 6.5% of gates fail only
on the cheap checks; 4 of 24 is 17%.

### 380.4 One call, and in this order

**One call** because two calls would be one more model turn on every gate, and
a turn bills the whole context again (§100). `just` with several recipe names
runs them in order and stops at the first that fails. That was checked both
ways in this worktree. With `mod  build;` written into `rcad-topo`'s `lib.rs`,
`just lint check-wildcards gate quiet` printed `cargo fmt`'s diff and `Recipe
'lint' failed`, exited 1, and never started the gate. On the clean tree it ran
`lint`, then `check-wildcards`, and then `gate` in `quiet` mode: `quiet` reaches
`gate` as its first argument.

**`lint` first**, so that `check-wildcards` is the last clippy run before the
gate. `check-wildcards` is also the first clippy step in the gate's cargo
chain, so it finds its own flags and the same code, and re-checks nothing. Run
C1 is the other order. There the lane's `lint` re-checked all 8 crates instead
of the 4 the edit reaches (A3), and so did the gate's `check-wildcards`
(12.4 s). That order costs about 16 s a passing gate, not 9.

§381 replaces this rule: `just pre-gate` runs the checks at once, each clippy
in a target directory of its own, so the order no longer matters.

### 380.5 What this leaves

- **When `lint` fails, `check-wildcards` does not run in that call.** A lane that
  broke both, like Q766, finds the second failure one cheap call later, not one
  gate later.
- **A lane gate that would be served pays the 16 s for nothing.** One of the 24
  was served (Q708's second run).
- **Verifiers, mergers and dispatchers are unchanged.** In these 14 runs, 19 of
  their 26 gates were served. Of the 7 that ran in full, the only failure was
  Q649's verifier, on a test.
