## 320. The reader's refusal on 1 057 writer crash files is the triangulator's floor, and Q79 closes all of it — 2026-09-28

*Q167, from the writer fuzzer's 2026-09-26 night (§265, §313.4). Lane
`lane-q167-reader-refuses-its-own-solid`, off `b5325a75`. **Neither the writer
nor the reader is wrong.** Every one of the 1 057 files stops at spade refusing a
coordinate below `2⁻¹⁴²`, the refusal §246 found on one round bar, and the flush
§246.5 names and queue row Q79 holds clears all 1 057. The fix is in
`crates/rcad-mesh/src/triangulate.rs`, which this lane does not own, so nothing
here changes code.*

### 320.1 One cause, not several

The same test file as §313.1, with a second test that takes the harness's steps
by hand — `postcard::from_bytes`, `Model::validate`, `rcad_step::write`,
`StepFile::read`, `Import::read` — and prints the `rcad_topo::Error` inside each
`Refusal::ThisEntityOfTheSolidWasRefused`. It was not committed. All 1 057 files
carry the same reason, with numbers written as `N`:

```text
1057  ADraftFaceCouldNotBeMeshed { face: N, why: CoordinateOutOfRange { position: N, why: TooSmall } } on ADVANCED_FACE
```

So the entity the reader names is always an `ADVANCED_FACE`, and what refuses it
is not the reader. It is `Model::validate`, run by `SolidBuilder::build`,
meshing that face; `rcad_mesh::triangulate` hands the face's corner to spade;
and spade refuses a coordinate that is not zero and smaller than
`MIN_ALLOWED_VALUE = 1.793662034335766e-43`, which is `2⁻¹⁴²`. §246.2 walks
that path from the reader to spade in full.

### 320.2 The sample

`crash-003bf776d50b3ffe45b474ab579baab2ced32489` is five solids in inches. The
second is a round bar of radius 4 whose shell is placed at
`translation: [0.0, 2.91412e-318, 0.0]`, a subnormal move along `y`. The writer
writes its corners and its axis at `y = 2.91412E-318`, which is the placed
model's own number. The reader builds the bar with that move in every corner,
and `#166=ADVANCED_FACE('',(#165),#155,.T.)` is refused:
`triangulating: the corner at 0 is outside the range a triangulation takes:
TooSmall`. The model that went in passes `validate` because a placed face is
meshed in its placement's inverse frame, where the move cancels to zero
(§246.3).

### 320.3 Where the small numbers come from

For each of the 1 057, whether the written file holds a real `x` with
`0 < |x| < 2⁻¹⁴²`, and whether its import succeeds with every such real
rewritten as `0.0` in the text:

| the file holds a number below `2⁻¹⁴²` | import with those set to `0.0` | files |
|---|---|---|
| yes | imports | 1 045 |
| yes | still refused, the same way | 1 |
| no | still refused, the same way | 11 |

Of the 1 045, 752 have the small number in a shell's placement and 293 do not.
Of those 293, 6 have one in a stored corner. The other 287 have it in neither,
which leaves a stored surface or a number the writer works out itself, and this
lane did not tell those two apart. In 88 files a small number in the file is not
one of the source model's located surface or corner numbers, bit for bit. Those
are `CARTESIAN_POINT`s and `DIRECTION`s in placements. This lane did not trace
them to the line in `write.rs` that produces them.

**The 12 are not a second cause.** 11 of them hold no number below `2⁻¹⁴²` at
all, and the 12th, `crash-db081a115148aed639d97ad4f53dd5b8e80bda10`, is refused
the same way with its small numbers set to zero. So in all 12 the scaled corner
spade refuses is worked out on the way to spade, from numbers that are not
small. `crash-0cd61ccf8507dbc7b429597740747b74853c915a`, 549 bytes, is a round
bar of radius 8 turned `6.4e-30` radians about `x`. Reading the model, and
not measured: a corner's `(u, v)` on that turned bar is a rounding residue below
the floor where the exact answer is zero. 320.4 shows the same flush clears all
12.

### 320.4 The flush, measured on the whole directory

§246.5's one-line change is: in `triangulate`, pass each scaled corner through
`spade::mitigate_underflow` before `mesh.insert`. This lane applied it to a copy
of `b5325a75` exported with `git archive` into the scratchpad, not to this
worktree. It then replayed all 22 528 files with §313.1's replay on both trees:

| first panic line, numbers as `N` | `b5325a75` | with the flush |
|---|---|---|
| no panic | 19 905 | 20 954 |
| a different surface: the normal only (Q299) | 1 509 | 1 560 |
| the reader refuses: `#N, in the solid #N, is not one this kernel can build` (Q167) | 1 057 | 0 |
| does not measure after its own file: `TooSmall` (Q172) | 43 | 0 |
| the reader refuses: `what #N describes is not a solid this kernel can hold` (Q168) | 13 | 13 |
| the reader refuses: `#N, on the curved face #N, is N off the surface it sits on` (Q169) | 1 | 1 |

The `b5325a75` column matches §313.4's branch column. The comparison is file by
file, and only two rows lose files:

- **Q167's 1 057:** 1 007 now pass every check, which means the located numbers
  come back the same bit for bit and a flat or unplaced solid measures the same.
  50 go on to stop at a plane's normal, §313.3's gap. All 12 files from the
  end of 320.3 are among the 1 007.
- **Q172's 43:** 42 pass and 1 goes on to the normal row. So Q172 is the same
  floor too, and Q79's lane can close it by replaying the directory.

### 320.5 Whose fault it is

Not the writer's. It writes the placed model's numbers, and a real below
`2⁻¹⁴²` is a legal Part 21 real that names a legal position. Flushing it on the
way out would move the solid silently. Not the reader's either. It reads the
number exactly, and nothing in `rcad-step` refuses the face. §246.4's verdict
holds for every one of the 1 057: the reader refuses a file that correctly
describes a solid this kernel holds, and the refusal comes from the
triangulator. **A gap, closed by Q79**, whose write set already holds
everything the fix touches.

### 320.6 What this leaves

- **Q79** (`crates/rcad-mesh/src/triangulate.rs`) is the fix for all of Q167
  and all of Q172. Its seed candidates: `crash-00f8c3b43bc1aaa6cf3866f4ba144041df4c46bc`
  (549 bytes; one bar placed at a subnormal `y`) and
  `crash-0cd61ccf8507dbc7b429597740747b74853c915a` (549 bytes; one bar turned
  `6.4e-30` radians, with no small number in its file). Both panic
  `every_model_the_writer_fuzzer_kept_is_written_again.rs` until the flush
  lands, which is why neither is checked in here.
- Q167 and Q172 can be closed when Q79 merges, if a replay of this directory on
  Q79's branch shows both rows at 0.
- The 50 + 1 files that move to the normal row belong to Q299 (§313.3).
- The 88 files of 320.3 whose small numbers are not the model's located ones
  are not a defect as far as this lane measured. The flush clears them like the
  rest. This lane did not look at whether the writer should work out a
  placement point that is off zero by less than `2⁻¹⁴²` when the model has
  zero there.
- How the tally was taken: 320.1's detail list and 320.3's per-file table are
  `why-base.txt` and `tiny-base.txt`. 320.4's per-file columns are
  `replay-base.txt.per-file` and `replay-flush.txt.per-file`. All four are in
  this lane's scratchpad,
  `/tmp/claude-1000/-home-mattias-git-mattiasgronlund-rcad-worktrees-dispatch-next/ae41a571-41a0-4312-8ede-91633cba1982/scratchpad/`,
  and a reboot removes them.
