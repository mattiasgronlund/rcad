---
name: prep
description: Check one queue row, or a bundle of them, against the code at a lanes run's base, and answer what its lane starts from — the write set, the model, the marks and the facts — or which files the row is missing. Read-only. The lanes workflow sends it for a lane given as { label, queueId, notes }.
model: sonnet
tools: Read, Grep, Glob, Bash
---

# Check the row, and answer what its lane starts from

`CLAUDE.md` is already in your context, and this adds to it. You are the first
stage of a `lanes` run whose dispatcher sent its row as `{ label, queueId,
notes }`. `lanes.js` builds the lane's brief from the row's text, your facts
and the dispatcher's notes, and starts the lane with the write set, model and
marks you answer. What you get wrong, the lane starts from. These rules were
steps 4 and 5 of `dispatch-lanes`, which the dispatcher did for every row at
every fill; they live here now and nowhere else (`docs/decisions.md` §396).

## You read; you never write

You have no `Edit` and no `Write`. Do not commit, stage or create a file, and
run no `just queue` verb that writes: `show` is the one you need. You run in
the dispatcher's own tree, not in one of your own, so anything you change there
changes under the dispatcher. Every command stays in the foreground.

**Read the code at the run's base**, which your brief names, not at your tree's
`HEAD`. The dispatcher's tree carries its `take` commit past the base, and can
carry more. `git show <base>:<path>` and `git grep -n '<pattern>' <base>` read
the base; `git diff --stat <base> HEAD` says whether `HEAD` differs from it at
all. `just src` and the other one-call readers `CLAUDE.md` names read the tree,
so use them only for a file that diff does not name.

## The rows

Read each row with `mise exec -- just queue show <id>`, one id a call. A bundle,
`queueId` `Q1,Q2,Q3` (§352.4), is one prep for all its rows. Report each row in
`rows`: its id, its model column, its write-set column and its whole line, each
exactly as the command prints it. `lanes.js` cannot read the queue file, so
what you copy is what the lane is given, and what your answer is checked
against.

## The write set

The lane's write set is the union of the rows' write sets, leaving out `-`,
and nothing else; `lanes.js` builds that union itself from the `write_set`
column you copy into `rows`, so you report no `writeSet` of your own. **You
never widen it or narrow it, not even when it is wrong**: that is a `needs`.
Only `queue next` checks a write set against the rows already running, `take`
does not, and two preps in one fill could widen two rows onto one file at once
(§396.4). A slip in restating the union here used to be able to stop a run on
its own; now there is nothing of yours to restate.

Check the write set against what the work really touches. A row whose write set
is wrong is fixed in the file first, by the dispatcher: a lane that needs a file
its row does not name either stops or writes outside its set, and the fill has
already let another lane hold that file. So answer `needs`, with each missing
file and why.

A row that adds a `check-*` recipe or an `#[ignore]` moves a count the gate
pins — `check-gate-runs-every-check`'s `check-*` count, `check-ignored`'s
declared count, `wildcard_arms` — so the `justfile`, which holds the recipe that
pins that count, must be in its write set too, or the merger has to bump it
outside the write set on its own (`.claude/agents/merger.md`). When it is not,
the answer is `needs`.

A row whose write set names a file directly under `crates/rcad-topo/tests/`,
`crates/rcad-geom/tests/` or `crates/rcad-step/tests/` that does not exist
yet, with no entry of that crate's `tests/merged.rs` in the write set, is
missing one: `autotests` is off there, so nothing builds the file until
`merged.rs` declares it (§373.7). Answer `needs` with the item
`crates/rcad-topo/tests/merged.rs#<module>` (or the `rcad-geom` or `rcad-step`
equivalent), the module being the file's name with each `-` written `_` —
never the whole file, which would hold every other row adding a file there
back too (§373.7's "what this leaves").

## The model

Per row, from its `model` field. A bundle is `opus` if any of its rows says
`opus`, else `sonnet` (§352.4). `lanes.js` refuses any other answer.

The field was set by these rules, and they say what it should hold: Opus for
anything reasoning about a numerical refusal, a published bound, or whether a
claim of impossibility holds. Sonnet by name for: replaying a corpus,
reproducing and classifying a fuzz artifact, bisecting, running the gate and
reporting, sweeping references after a renumber, resolving a `decisions.md`
conflict down to a marker deletion, disk and worktree sweeps. When a row's field
disagrees with them, say so in a fact; the answer is still the field.

## `sourcePaths` and the marks

`sourcePaths` is what the verifier reverts to the base to watch the tests fail
without the change. Leave it empty and `lanes.js` defaults it to the
`crates/<c>/src` directories in the write set. Set it when that is not what the
verifier should revert: for a tooling row whose test is a recipe, the file the
recipe checks.

A row whose `lines-that-run` print will end `lines that run: 0` — `.md` files,
comments, test code — needs no mark. It names no failing test, and its verifier
traces every paragraph it moved instead; a row that adds a test, a
`#[cfg(test)]` pin in `src/` included, still names it. A bundle of such rows
merges on that print with no `pinsOnly` and no `sourcePaths` (§375, §382).

**`pinsOnly: true`** is for a row that only pins a test and writes a decisions
section, with no `src/` path anywhere in its work (§287). Say it explicitly;
it is never inferred from an empty `sourcePaths`, because that emptiness is
also what a tooling row shows when nobody set `sourcePaths`, and that must keep
refusing. A `pinsOnly` row with a `src/` path too — in `writeSet` or
`sourcePaths` — is refused: there is something to revert then.

**`sameBehaviour: true`** is for a row that changes lines that run and keeps
what they compute — one of two copies dropped for the other (Q854), a body
lifted into a function (Q601), a block moved (§399). Say it explicitly, for
`pinsOnly`'s reason: a lane naming no failing test is also what a lane that
changed behaviour and found no test looks like. Its lane names the tests that
reach the change, and its verifier shows they pass with the source reverted and
fail under a mutation of a line the change adds.

**Never both marks on one row**; `lanes.js` refuses that answer. Decide each
mark from what the row's work is, and put why it holds in `marks_why`, which
the lane's brief carries; leave `marks_why` empty when you set neither.

## The facts

`facts` go into the lane's brief as they stand, one bullet each. They are what
the lane would otherwise spend twenty minutes finding again:

- the verbatim API signatures and exact-arithmetic facts the lane needs, each
  with the file and line at the base it is from. Quote a signature; never
  paraphrase it, and never give a fact you did not read at the base;
- for a row that writes a section, `mise exec -- just claim-section '<what>'`
  before writing;
- for a row that compares two fuzz setups on coverage, how far two identical
  runs drift apart. After one 9 736 s slice, measured with `RCAD_FUZZ_TIMINGS`
  unset, one pair ended 60 edges apart on the reader, 10 on the writer and 0 to
  3 on the other three targets (§272.9). A smaller gap says nothing, and one
  pair is a floor, so ask for several runs a side. One measuring pass per
  corpus is enough: two passes over one corpus agreed to within 2 edges;
- for a row that names crash files, whether the latest fuzz night still saw
  them fail (§433.8). The night is the last field of the last line of
  `latest` under the fuzz log root, `RCAD_FUZZ_LOGS` or else
  `~/rcad-fuzz-logs`, as `_status-triage` reads it (`just recipe
  _status-triage`). Its crash files are `<night>/artifacts/<target>/crash-<hex>`.
  Its `fuzz-triage-members.tsv`, headed `target`, `group`, `file`, names by
  full path every crash file of each group that still failed on the `main`
  whose sha `fuzz-triage-replayed-main.txt` holds. A row's `crash-<hex>`
  names a file whose hash it starts, as `_status-triage` matches it. When
  every crash file the row names is under that night's `artifacts/` and none
  is in its `fuzz-triage-members.tsv`, say so in a fact with the night, that
  sha, and whether `git merge-base --is-ancestor <that sha> <base>` exits 0:
  the lane runs them on its base first, and may close the row without a fix
  (`lane.md`). A night with no `fuzz-triage-members.tsv` or no
  `fuzz-triage-replayed-main.txt` was not triaged that way, and gives no such
  fact. A named crash file that is not under the latest night's `artifacts/`
  but is listed in `removed-crash-files.tsv` under the same root passed on the
  sha its line names: the morning folded it into the corpus and removed it
  (§434.3). That file's columns are `night`, `target`, `file`,
  `replayed_main`, `kept`. Count such a file as passing, give the sha from its
  own line, and its night's copy at `<root>/<night>/artifacts/<target>/<file>`
  for the lane to replay;
- for a row whose write set names a `tests/*.rs` file, the open rows that
  name the same file in their write sets, each with its id and its line
  (§450.3 rule 4). That is the row's family: once its fix is committed, the
  lane runs each one's shapes on its branch and names those that now answer.
  Find them with `git grep -n -F '<file>' <base> -- docs/queue.md`, and leave
  out the row itself, a closed or parked row, and one a lane holds, which has
  `lane:` in its third field. Leave out `crates/rcad-topo/tests/merged.rs`,
  `crates/rcad-geom/tests/merged.rs` and `crates/rcad-step/tests/merged.rs`
  too, whole or as an item (`merged.rs#<module>`): a row names it only to
  declare a new file there (§373.7), not to say what shape it is on, so it
  is never the shared file a family is on. `just queue families` lists only
  families of three or more that no trace has named, so it is not this list.
  Say so when there are none;
- nothing about the context hook for a writing row. A lane that sees "Lane
  budget passed" is past 150k of its own context; it commits, hands its branch
  over, and `lanes.js` starts a fresh lane on it, up to three times a row
  (§416). "Carry on past it" would undo that. A read-only lane has nothing to
  hand over and stops at the warning, so for a read-only row that builds a
  probe or reads a listing, give "carry on past it" as a fact.

## What to hand back

`ok`, with `rows`, `model`, `sourcePaths`, `pinsOnly`, `sameBehaviour`,
`marks_why` and `facts`, and `needs` empty. Or `needs`, with
each file the work touches that its rows do not name and why, and the other
fields as far as you got. A `needs` stops the run before any lane starts, and
it comes back not merged with your `needs` in its lines. The dispatcher then
re-files the row, drop and add, then `queue next`, and decides it as Q813 says.
