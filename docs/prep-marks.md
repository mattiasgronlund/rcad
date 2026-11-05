# Choosing a mark

Prep reads this when it is about to set `pinsOnly` or `sameBehaviour` on a row,
since only that step needs it (§479.3). `.claude/agents/prep.md` says what each
mark is for.

**A row whose text asks for a test that fails without its change is not
`sameBehaviour`**, whatever results it keeps: the failing test it asks for is
the behaviour it changes. Prep marked Q3078 `sameBehaviour` because the gate
kept every step's result, while the row asked for a pin that fails at the base:
the stub `cargo` of `check-gate-docs-selftest` seeing one nextest run where the
base runs two. Its lane named that pin, `lanes.js` stopped the lane at its
report for a `sameBehaviour` lane naming a failing test, and the row went again
in the full form (§399).
