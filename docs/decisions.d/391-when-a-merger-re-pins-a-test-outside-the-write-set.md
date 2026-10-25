## 391. When a merger re-pins a test outside the write set — 2026-10-25

*Q924. Replaces Q816, which claimed this section under the title "a merger
re-pins no test outside the write set" and decided the opposite of what this
section settles.*

In short: a merger may re-pin a test outside the lane's write set — not just
apply a patch the lane already wrote and the verifier already ran — when the
verifier names the pin, the value it becomes and why that value is right, and
no open row holds the file. When the verifier cannot show the new value
right, the pin is not the merger's, and goes to a landing row instead.

### 391.1 The question

Two mergers on 2026-10-03 read `merger.md` step 2 two ways on a test pin
outside the lane's write set that the lane's own change moved:

- Q563's merger (`be0ddf78`) re-pinned two KiCad corpus counts, both 164 to
  165, as "a count the gate pins."
- Q650's merger refused to re-pin a test outside the write set and did not
  merge, which cost a second run of the same fix (Q814).

Q816 looked at the same two runs and took the narrow reading: a count the
gate pins means only the three recipes named in `merger.md` step 2 at the
time — `check-gate-runs-every-check`'s `check-*` count, `check-ignored`'s
declared count, `wildcard_arms` — whose right value the check itself
computes. A test's expected value, a corpus count included, is not one of
those, because whether the new value is right takes a verifier's judgement,
and a verifier judges only inside the write set. Q816 was filed to record
that reading and never implemented; it is re-filed here as Q924, deciding the
other way, because the record since built up more than those two runs to
weigh it against.

### 391.2 What the record since shows

Three more merges re-pinned a test's expected value outside the write set,
each saying why the new value was right:

- **Q503's merge (`f8b4cc67`)**, citing §338.4 and §387.4: a refusal's
  classification moved from `AFaceMeetsItsNeighbourOnItsOwnSurface` to
  `TwoCurvesTouchWithoutCrossing`, the one the lane's own fix produces for
  every draw of the shape, both ways round.
- **Q668's merger (`5d28a70c`)**: two point classifications moved to the
  value the shape's own stored plane gives — `Outside` for a point the test's
  own assertion already named, `Inside` for a point the module's two-and-one
  split becomes four-and-one. Its report asked that `merger.md` say a merger
  may do this; nothing had, until now.
- **Q2374's merge (`bb1d1786`)**, a larger case: a 164-line, 103-deletion
  re-pin the lane itself had written, formatted, linted and run green
  (`out_of_write_set_needs` named it, the verifier copied it into `fixes`),
  merged whatever its size (§414.4). That case is `merger.md` step 2's first
  bullet and is unchanged here; this section is about the merger writing a
  smaller re-pin by hand, not applying the lane's own tested patch.

Against these, one merger refused correctly: **Q453's merger**, asked to
re-pin a refusal to `Inside` outside its write set, could not show the new
answer was the right one — nobody had decoded the shape or checked it against
a closed form — and did not merge. That pin went to a landing row instead
(Q920), which decoded the shape, checked it against a closed form, and
landed it once the answer was shown right.

### 391.3 The rule

The merger re-pins a test outside the write set when all three hold:

1. The verifier's `fix_first` names the pin, the value it becomes, and the
   evidence that value is right — an exact sign, a closed form, or (for a
   corpus pin) the one file that moved and no other.
2. No row in `mise exec -- just queue running` holds the file.
3. The change is a few lines.

The merger says so in the merge commit. A pin whose new value the verifier
did not show right is not the merger's — as Q453's was not — and is a
landing row's job, not a merge's. `merger.md` step 2 and `verifier.md`'s "The
verdict" say this where each role reads it, and `dispatch-lanes` step 4 says
that a row which makes a corpus file import or stop importing names its two
corpus pins in its write set, so the row's own lane can move both counts
instead of leaving it to the merger.

### 391.4 Checking `be0ddf78`'s 165

Q563's merge re-pinned `KICAD_FILES_THAT_IMPORT` and
`KICAD_FILES_THAT_ROUND_TRIP` from 164 to 165. The only change to either
array, across the merge, is the one line added to each:

```
$ git diff 65a476a86..be0ddf78e0cbc188dc5b454fd467748a74c843d8 -- crates/rcad-step/tests/how_often_a_real_file_imports.rs crates/rcad-step/tests/a_real_file_written_out_and_read_back_is_the_same_solid.rs | grep -E '^[+-]    "'
+    "Relay_1-Form-C_Schrack-RYII_RM3.2mm.step",
+    "Relay_1-Form-C_Schrack-RYII_RM3.2mm.step",
```

One addition in each file, the Schrack RYII relay that §387.4 says the lane's
fix let import, nothing removed and no other file's name touched. 165 is 164
plus that relay and no other file.
