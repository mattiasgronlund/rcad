## 370. A verifier's step 3 already proves the tests pass; step 6 stops proving it twice — 2026-09-29

Re-files Q689, which asked the same question of `verifier.md` alone; this
answer also changes `lanes.js` and `check-lanes-workflow`, so the row moved to
Q703. Closes Q689 and Q703.

### 370.1 What step 6 cost, for no new information

`verifier.md`'s step 3 runs `mise exec -- just gate quiet` on the branch tip.
Its step 6 then ran the lane's named tests again, straight after: `mise exec
-- just t <name>`, or `mise exec -- just <recipe>`. §367.6 named this as
Q689: Q574's verifier paid about five minutes there, right after a gate that
had just run, or served, that same code.

The two runs are not independent measurements of two different things. `test`
(`cargo nextest run --workspace`, then `cargo test --workspace --doc`) is
itself one of `gate_steps` (`justfile` line 78-133), so step 3 already runs
every compiled test in the workspace, the lane's named ones included, unless
they are `#[ignore]`d. Step 6's `just t <name>` is `cargo nextest run "$@"`
with no `--run-ignored` -- read from the recipe itself -- so it applies the
same default filter as `test` and would have skipped an ignored test exactly
as step 3 does. Naming one there never proved anything different from naming
it nowhere; that candidate, raised in the row, is not a real gap.

A named `check-*` recipe is in `gate_steps` too, and not by convention:
`check-gate-runs-every-check` pins the count of `check-*` recipes defined in
the justfile against the count listed in `gate_steps`, so a new one that is
not wired in fails the gate in the same commit that adds it. `lint`, `test`
and `deny` are themselves always in `gate_steps`. So for every shape a lane's
`tests_that_fail_without_the_change` normally takes -- a test function, or a
`check-*` recipe -- step 3 has already run it, live or served, before step 6
gets a chance to run it again.

The other candidate the row raised, a test needing an env var the gate does
not set, was checked and found not to hold either: every role reaches `just`
through `mise exec --` (CLAUDE.md's tooling section), so step 3 and step 6 run
under the same `mise` environment, `RCAD_STEP_CORPUS` included.

### 370.2 The one thing step 6 could still be checking

`verifier.md` also lets a lane name a bare recipe, `mise exec -- just
<recipe>`, and not every recipe in the justfile is in `gate_steps` --
`queue`, `decision`, `claim-section`, `t` itself, and others besides. Nothing
pins those into `gate_steps` the way `check-gate-runs-every-check` pins a
`check-*` recipe, so a lane that named one of those as its proof would have a
real gap: step 3 never touches it, served or fresh.

Step 6 keeps exactly this case, and only this case: a named recipe that is
none of `test`, `lint`, `deny` or a `check-*` recipe gets its own run, `mise
exec -- just <recipe>`, once. Everything else -- a test name, or any of those
four kinds of recipe -- takes its `tests_pass_on_branch` from step 3's gate
summary instead, whether that step served or ran fresh.

### 370.3 Whether a served step 3 is independent evidence, or the lane's own word again

`CLAUDE.md` tells a verifier "do not trust its report" (`verifier.md` line 23),
and a lane gates its own tip before it reports (`lane.md`), so a served step 3
is usually serving a record the lane's own gate wrote. Read on its own that
looks circular: the verifier's proof of "pass" would rest on the lane having
said so.

It is not the lane's report that step 3 reads, though. `docs/decisions.d/337`
describes what the record actually is: one file per code fingerprint, in the
hub's shared `.git`, written by the `gate` recipe itself only when the run
was not served, every reusable step ran, and the fingerprint did not move
between the start and the end of the run (§337.4). A lane cannot put a record
there by writing JSON in its own report; the only way one exists is that
`gate` really ran `test` to completion, for real, on this exact tree, and it
passed. The verifier's own step 3 still checks the fingerprint against the
branch tip it just checked out, so a served record only applies when the code
is the same code, byte for byte outside `docs/` and `*.md`.

What is left to worry about is whether that record could be lying -- stale,
or from a run that was not deterministic. §337.4 bounds staleness (a day, and
`fresh` or an audited commit ignore it outright) and §337.6 is the answer to
the second worry: any reusable step's audit or `fresh` run that disagrees
with a standing record is treated as a finding, not noise -- it deletes the
record, appends a line to a shared violations file, and says to find the
cause before running the gate again. That is an active check for the thing
that would make serving unsafe, not silence. A served `tests:` line is
therefore independent of the lane's own claim in the sense that matters here:
it is evidence the `gate` recipe's own machinery wrote about a real
execution, checked against this tree's own fingerprint, under a mechanism
built to surface exactly the failure mode -- same code, different answer --
that would make trusting it wrong.

### 370.4 Why "always run step 3 fresh" was not taken instead

§337's whole point is that code already gated green on this machine is
served, not run again, and `docs/decisions.d/369` measured what a served
gate now costs. Telling every verifier to pass `fresh` at step 3 would make
every verifier pay the full, cold cost that serving exists to avoid,
including the fifteen-plus steps §369 already moved off it -- for the same
information a served record already gives, protected the way §370.3 above
describes. That is the same shape of cost Q689 flagged in the first place,
moved from step 6 to step 3 rather than removed. §369's argument for serving
already covers a verifier's own gate; nothing here argues the verifier is a
special case that needs to re-earn it.

### 370.5 What changed

`verifier.md`'s step 6 no longer runs the lane's named tests before the
revert. It instead says which of `test`, `lint`, `deny`, a `check-*` recipe or
a plain test name the lane named, and for all of those, takes
`tests_pass_on_branch: true` from step 3's `gate: PASS` line, and for a test
name its `tests:` line too. Only a recipe that
is none of those still gets its own run. Step 7 (the revert, the required
failure, and the restore) is unchanged except that the restore no longer
implies a second "watch it pass" run of its own: what it restores is exactly
what step 3 already gated.

A `pinsOnly` lane has no revert at all, so its whole proof used to be step
6's direct run. It is now step 6's reading of step 3's gate summary instead --
the same fact, taken from the same place every other lane's now is.
`lanes.js`'s `VERIFY` prompt and its `pinsOnlyNote` carry the identical
sentence verifier.md's step 6 does, so a verifier reading either cannot land
on the old double run by only half-reading the change. `check-lanes-workflow`
gains a fifteenth arm that checks that sentence survives, in both places in
`lanes.js` and in `verifier.md`, the way its eighth arm already does for the
verifier's own tree (§263.4).

`verdictSchema`'s `tests_pass_on_branch` property gained a one-line
description saying where it now comes from; the field itself, and
`unproven()`'s reading of it in `lanes.js`, are unchanged, because it is still
exactly the same boolean claim -- only where the verifier gets the number to
put there moved.
