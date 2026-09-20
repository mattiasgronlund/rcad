## 107. What actually blocks the STEP corpus, measured 2026-09-15

Five lanes over two days asked which lever opens the corpus. Four of them
priced one hypothesis each and three of those bought a zero. The fifth stopped
asking one hypothesis at a time and measured the whole stack, and its table is
what §0's unit is now cut from.

**The baseline it is read against: 98 of 367 files import** — 97 of 323 KiCad,
1 of 44 NIST, `just t how_often_a_real_file_imports --no-capture`. The probe
agrees with that number by a different route: 106 files give it no reason at
all, 8 of those for a gate that stops the walk before it starts, and 106 − 8 =
98. That agreement is asserted per file in the example itself, which panics if
a file imports and the probe still finds a reason.

### 107.1 The number that prices a fix

§45.2 says the count of files a blocker stops **first** is an upper bound and
never a projection, and three lanes then paid for that sentence: upper bounds of
75, 55 and 663 edges delivered 1, 0 and 0 files. What prices a fix is the count
of files that have **only** that reason, and no walk that stops at the first
refusal can take it. `rcad_step::probe` is the walk that does not stop, and
this is what it found over both corpora:

| files where it is the only reason | reason |
|---:|---|
| **97** | a corner is not on the curved face the file says it is on |
| **44** | this crossing runs where its two surfaces are too nearly parallel to follow it |
| **18** | a face walks a closed edge against the way its surface winds |
| **17** | a curve this kernel does not hold: `B_SPLINE_CURVE_WITH_KNOTS` |
| **7** | this crossing comes in two pieces and no half-space square to both surfaces can be written down in `f64` |
| **6** | a surface this kernel does not hold: `SURFACE_OF_REVOLUTION` |
| **5** | could not settle the decisions the answer depends on |
| **4** | a loop this reader does not walk: `VERTEX_LOOP` |
| **3** | a solid with a void in it |
| **3** | a vertex of a face is too far from the face's plane |

with a tail of nine reasons worth one or two files each, and 8 files in the
fatal bucket — two length units, an assembly occurrence, a void, no size.

**Co-occurrence is what makes those numbers real, and it splits them in two.**
The 44 **never appear beside another reason in any file**: 44 files hang on that
one question and nothing else. (That is co-occurrence and it stands. What *kind*
of question those 44 hang on was never measured, and §109 is the correction —
the refusal names the whole solid, so nothing could read the pair that refused.) The 97 sit alone in 97 of the 99 files that have
them. Against that, `B_SPLINE_CURVE_WITH_KNOTS` is in 68 files and alone in 17 —
22 of them also want a B-spline *surface*, 18 a surface of revolution, 10 a
bounded surface, 7 a vertex loop. The spline path is not one lever, it is four,
and §50.6 already records the surface as built and reachable from nothing.

§47.3 named the 44, the 18 and the 7 a unit earlier, from the opposite
direction — the files that stopped being called a corner miss when `brep.rs`
stopped overwriting the builder's own refusal. Two independent routes, the same
three rows.

### 107.2 Two rows were published wrong, and the reason generalises

The table above is the second one. The first was taken on the lane's own
branch, which is based on two earlier measurement lanes whose commits were never
reverted: `75e32bee` accepts a closed `B_SPLINE_CURVE_WITH_KNOTS` edge as going
right round, and `89716dae` accepts *any* unheld curve between two distinct
planes as straight. Both are knowingly unsound and both were live under the
probe. Landing the probe on a clean tree and re-running moved two rows:

| | on the lane's tree | on `main` |
|---|---:|---:|
| `B_SPLINE_SURFACE_WITH_KNOTS` alone | 13 | **1** |
| files carrying the spline-edge refusal | 56 | **68** |
| KiCad files with exactly one reason | 196 | **184** |
| KiCad files with exactly two | 24 | **36** |

The accepts were removing the spline-edge refusal from 12 files, which promoted
the surface refusal beside it to *sole* reason in each. **The 97, the 44, the 18
and the 7 are identical on both trees**, so the unit this chose is not affected
— but a lane that had gone at the B-spline surface on the strength of 13 files
would have found 1.

**The probe's own honesty check could not catch it**, and that is the part worth
keeping. It asserts that a file `Import::read` imports gives the probe no
reason — but both halves ran on the same patched tree, so the check compared a
patched reader against a patched probe and passed. A measurement carries the
tree it was taken on, and the only defence is to re-run it on the tree the
number will be published against. It cost one re-run here because the probe was
landing anyway; it would have cost a lane if the number had gone into a brief.

The refusal's own *name* moved too, for the same reason:
`an arc this reader could not name` was the patched tree's invented refusal for
an open spline edge it would not guess at, and on `main` the reason reads `a
curve this kernel does not hold: B_SPLINE_CURVE_WITH_KNOTS`, which is what the
reader has always said.

### 107.3 Healing is the biggest bucket and its own trigger does not fire

§42.9 settled the healing fork as option A — refuse by name, heal nothing — and
§44.6 wrote D's trigger so it could fire: *D fires when, after the named gaps
close, more than half the still-refused files are refused for a miss larger than
`f64` rounding.* It scored the trigger at 50–56% on §43.5's data, called that
"exactly on its own boundary", and said it becomes decidable only when a lane
publishes a per-file refusal reason. That lane has now run.

269 files are refused. 99 of them carry a corner off a curved face. **That is
37%, and the trigger does not fire** — it has moved away from its boundary
rather than onto it, because the named gaps closed and took files out of the
denominator on the other side.

**Two things keep that from being the final score, and both are named here
rather than assumed away.** It reads all 99 as the beyond-`f64` kind, and
§43.5's 88/73 split of the then-161 was taken before the arithmetic gap closed,
so the split wants re-deriving against today's tree before healing is priced
again. And 37% is not a small number: this section scores the trigger, it does
not argue that healing is the wrong thing to build.

### 107.4 What landed, and what it costs the gate

`rcad_step::probe` and `ProbeReport` are public; `probe_solid` and `ProbeWalk`
sit beside `Walk` in `brep.rs`; `examples/blocker_depth_probe.rs` prints the
five deliverables above. `Walk::build` and `Walk::what_stopped_the_build` take
`&self` rather than `self`, which costs nothing — `build` already cloned the
builder.

**The rule that makes the counts mean anything is that nothing which lost an
entity is asked to assemble what is left of it.** A dropped edge leaves a wire
that cannot close and a dropped bound leaves a face with a hole in it, and
`add_wire`/`add_face` then report *that hole* as a second reason the file does
not have. Take the guard out and `the builder or the sweep: a wire does not
close` appears in two of the five tests immediately, which is how it is known to
be load-bearing rather than argued to be. At the shell level the same rule is
applied and **is not known to cost anything**: `add_shell` took a block's five
remaining faces without complaint when the sixth was dropped. It is kept there
anyway, so the numbers do not depend on which level a future check lands at.

Five tests in `what_the_depth_probe_finds_behind_the_first_refusal.rs`, all off
the `a_block` fixture that imports: a file that imports gives no reason, a file
broken two independent ways names both where the reader names one, one break is
one reason at the edge and at the face, and a gate before the solids reports
fatal and nothing else. `just gate` PASS at 1903 tests.

§56.7 is why the probe landed rather than dying with its lane: its four corpus
numbers were taken by a probe that was deleted, so nothing re-takes them and
nothing goes red when they stop being true. The three parse-level censuses from
the round before this one landed as examples on the same argument (`ac77965b`).

### 107.5 The unit this cut

The 44 crossings too nearly parallel to follow, with the 18 closed edges walked
against their surface's winding behind them. It is the biggest bucket that is a
*gap* rather than a decision to reopen: one reason, never mixed with another in
any file, inside the kernel's own arithmetic, with no §42.9 to argue down. And
§16.10 makes the demonstration part of the deliverable rather than an extra —
"no ordering of the arithmetic helps" was true of orderings and silent about
scalings once before (§16.10), so the unit names the family of fixes it
considered and says why that family is the only one, or it finds the answer.

**The word "gap" above is withdrawn as unearned — §109.4.** The unit stands and
so do the counts; what does not stand is the kind. The refusal names the whole
solid and never the pair it refused about, so nothing had classified these 44
when this was written, and two of the six sites that raise it are not about
tangency at all. Building a refusal that names its pair and its raise site
(§109.5) is this unit's first piece of work.

What it is **not** is the file count. Healing's 97 is that, and §107.3 is why
this unit is not it.

