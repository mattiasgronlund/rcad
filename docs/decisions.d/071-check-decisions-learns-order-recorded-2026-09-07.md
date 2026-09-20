## 71. `check-decisions` learns order, recorded 2026-09-07

`check-decisions` (§57.8) already refused two sections sharing a number. It said
nothing about a section landing in the wrong place, and two ways to land in the
wrong place happened on the same day.

### 71.1 The two shapes

A duplicate and an out-of-order section look alike at a glance — both are a
number that does not say what it should — but they are different mistakes with
different causes, and the file carried both on 2026-09-07.

**The duplicate.** Two sessions each appended a `## 62.` hours apart — one "What
a release build hands back when the kernel is wrong", one "A sweep that could
only ever return one answer" — because each read "the next free number" off a
tree that disagreed with the other about what was next. Nothing noticed until
the merge, and the merge was resolved by taking one side of the file whole,
which silently dropped two amendments the other side had made to §4.1 and §4.3.
That second loss was caught by a line count, not by reading it.

**Out of order.** Three lane briefs were given §63, §64, §65 while `main` landed
its own §63 underneath them. They were renumbered to §67, §68, §69 by hand, and
the renumbering is why the file now reads 61, 62, 63, 66, 67, 68, 69 — a gap is
what a renumbering leaves behind, and §57.8's own doc comment already says a gap
is not a defect. What the renumbering also left, four times over in one session,
was a `### 64.3` still sitting under the new `## 69.` after its own `## 64.`
had become `## 67.` — a subsection typed once and never moved when its parent's
number changed underneath it. That was caught by grepping for it by hand, which
is the same expensive place §57.8 was written to move the duplicate check away
from.

### 71.2 What `check-decisions` checks now

Two more passes, after the existing format and duplicate checks and before
either exits:

- **Descending is an error.** A `## N.` lower than the `## N.` before it in file
  order.
- **A misnested subsection is an error.** A `### N.M` whose `N` does not match
  the `## N.` it currently sits under, or whose `M` is lower than the `### N.M`
  before it within that same section.

Gaps are still fine, at both levels — nothing here re-litigates §57.8's own
point, and the real file's 61, 62, 63, 66, 67, 68, 69 is exactly what a correct
file with a gap looks like, so it is the fixture the check has to pass rather
than one of the ones it has to fail.

### 71.3 Fixtures, not the real file

`check-decisions-selftest` proves five broken shapes against small fixture
files built for the purpose, the same way §57.8's own selftest proved the
duplicate check: the two duplicate checks §57.8 already had (a repeated `##`
and a repeated `###`), plus the three this section adds (a descending `##`, a
`###` descending within its own section, and a `###` sitting under the wrong
`##`). Editing the real `docs/decisions.md` to demonstrate a defect would be
demonstrating the defect into the file that must not have it.

---

