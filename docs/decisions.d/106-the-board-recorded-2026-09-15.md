## 106. The board, recorded 2026-09-15

### 106.1 Three decisions each took state out of a place, and none put it back

Asked why it is so hard to see the plan, the answer is not that anybody was
careless. It is that three decisions, each right on its own, between them left
nowhere for the state of the work to live.

- **`docs/roadmap.md` may not carry state.** This file's own preamble says it is
  the north star and *"a roadmap, not a specification of this codebase"*, and §1
  exists so a later reader does not see the code disagree with it and "fix" the
  code back. That is why it has one commit in the repository's whole history. A
  checkbox in it would be the first edit down a road that ends with the roadmap
  describing the tree, which is the one thing it must not do.
- **The M-numbers were abandoned.** §44.1 found that "M6" named three different
  bodies of work in three documents and replaced the numbering with named
  units — `the door`, `the cut`, `the six arms`. That was right, and it has a
  cost nobody priced: **a unit chain is sequential and a roadmap is a tree.** A
  chain shows the node it is on. It can never show the board.
- **This file is append-only.** Everything in it is true of the moment it was
  written. §43.8's *"what M5 hands to M6"* was an accurate list on its day and
  nothing updates it; §53 sat open for nine days and §103 is what finding that
  out cost.

So the plan was in a file forbidden to have state, the schedule was in a chain
that can only show one node, and the record cannot be edited. Nobody decided
that the state should be invisible. It fell between three decisions.

### 106.2 `docs/plan.md`, and why it is not a second record

§60 says a second record beside the thing itself is a second thing to disagree,
and that objection has to be answered before adding a file.

It is answered by what `plan.md` does **not** contain. It holds no titles and no
item list: a line is a roadmap number, a state and the evidence, nothing more.
`just plan` renders it against `docs/roadmap.md`'s own headings, so the titles
exist once, in the roadmap. `check-plan` derives the item list from the roadmap
on both sides of a comparison and fails when they differ, in either direction —
so a roadmap that grows a subsection cannot leave a plan quietly reading
complete over a roadmap that has moved.

What is left in `plan.md` is the one thing genuinely recorded nowhere else:
whether a thing is built. Before this, establishing that `4.3 Differential
geometry` does not exist meant grepping for fundamental forms, curvature types
and Frenet frames and reasoning from the absence. That is not a record, it is a
re-derivation, and it was paid for again every time anyone asked.

### 106.3 A box is ticked against evidence, not against an opinion

The states are `done`, `partly`, `todo`, `unaudited` and `n/a`. A line carries
the module or the test that stands behind its state, and for `done` the check
holds every path it names against the disk. That is §15.6's rule — a published
bound needs a test — applied to a published checkbox.

The check cannot decide whether the evidence *proves* the item, and does not
try. What it removes is the failure that kills a status file: a tick that
outlives the thing it was ticked for. `7.3 done on crates/rcad-topo/src/
classify.rs` fails the moment that file is renamed, which is exactly when a
hand-maintained board would have started lying.

`unaudited` exists so that the file can be honest about the difference between
*not built* and *nobody looked*. The first audit, 2026-09-15 against `6939147`,
left none — three read-only lanes covered all 26 items — but a roadmap that
grows an item gets `unaudited` by default rather than `todo`, because assuming
the new thing is unbuilt is a guess like any other.

### 106.4 What this is for: cutting lanes

`just plan open` is the list of everything unfinished, each line already naming
what is missing. A unit is a set of those lines; a lane is one of them. That is
the step that was missing between "§53 is the current unit" and a brief — the
unit sections name their lanes (§53.4 and its four predecessors all do), but
nothing said where the candidates come from, so each unit's arms were derived
afresh from whatever was in mind that week.

The first audit's answer, for the record: 13 done, 7 partly, 5 todo, 1 not a
build item. The five `todo` are differential geometry, approximation, local
operations, healing and the scratch allocator; the seven `partly` each say in
their own line what is missing, and three of those — the freeform intersection
path, `Extrema`, and the general curve–surface case — are deferred for want of
a consumer rather than unattempted, per §13 and §24.1.

### 106.5 It took the number that was already taken

This section was written as §105 and renumbered in the merge. §105 had been
picked, used and landed on `main` by another lane while this work was in
progress, and nothing noticed until `git merge main` put the two blocks against
each other.

The rule that would have caught it already exists and was followed for §102,
§103 and §104 and then not for this one: pick a number with `git log --all
-S'## N. '`, which sees every branch, rather than against the file in front of
you. `origin/main` under-counts what lanes have queued. The cost was a
three-edit merge resolution and every `§105` in this branch having to be
re-pointed; the cost of the check is one command.

