## 342. An edge walked once from each of two shells is refused at the door — 2026-09-28

*Queue row Q403, from Q84 and the last item of §242.6. Lane
`lane-q403-an-edge-walked-from-two-shells`, off `9fc2b3d6`. A shell has to
close on its own. The door, `validate`, `validate_solid` and the touched checks
now refuse an edge whose two coedges are in two shells, as
`Invalid::AnEdgeIsWalkedOnceFromEachOfTwoShells`. **A gap** (§16.10): a
caller's file made a fuse panic, and the right answer, a refusal, was one index
lookup away.*

### 342.1 What was wrong

A shell is a closed skin (`ShellData`'s doc). `check_edge_is_walked_by_two_opposite_coedges`
counted an edge's coedges over the whole model. Share a cube's six faces out
between two solids and every edge along the cut still has two coedges running
opposite ways, one from each side. Every key resolves, every face is in one
shell and every shell is in one solid, so every other door check passes too.

- **The door let all 34 cuts in**: the 31 ways to cut a cube's six faces in
  two, and the 3 ways to cut a round bar's three. Measured on the base: the
  three door tests in
  `tests/a_skin_that_closes_only_with_another_never_reads_back.rs` fail there
  with `None` where they want `SerdeDeCustom`.
- **A fuse of a bar half panicked.** On the base, the half holding the wall and
  the top cap panicked with "no entry found for key" at `fuse/bake.rs:874:41`,
  fused with nothing under `Difference` and with a cube 10 mm clear under
  `Union`. That is two calls through a throwaway probe on `9fc2b3d6`. The fuse
  looked for the face across a cut edge among the faces of its own half. Q84's
  row has the wider count, from its own probe on `91896051`.

### 342.2 What changed

- **The new refusal is in the one helper**, after the count and the
  direction. `check_shape_is_possible` (the door), `validate`,
  `validate_solid` and `check_touched_edge` all call that helper, so one change
  reaches all four.
- **`Index` gains `shell_of_face`**, derived from the downward links like the
  other three. `rebuilt_from` asserts that no face is in two shells: the door has
  already refused that as `UsedMoreThanOnce` by then, so the assert is the same
  kind the wire and coedge links have. `add_shell` writes it.
- **Where a link is missing there is nothing to compare.** A coedge no wire
  walks, a wire on no face or a face in no shell has no shell, and the helper
  leaves it to the orphan checks. `Deserialize` refuses all three before it asks
  (`check_everything_has_an_owner`), and `validate` still refuses them as
  `Orphan`, so no existing refusal changed name. The one rcad-topo test that
  had to change is the copy test in §342.3.
- **It belongs at the door under §44.5's rule.** It needs no `Policy` and no
  arithmetic. It costs three lookups per coedge, so it is a question about one
  edge and not a sweep over pairs.

### 342.3 `copy_solid` asks nothing first now

`copy_solid` used to ask whether every edge of the solid was walked only from
inside it, and refused with `EdgeCoedgeCount` otherwise (§242.6). With the new
check that cannot happen. Both coedges of every edge are in one shell, and a
shell is in one solid (`UsedMoreThanOnce`), so every edge a solid reaches is
walked from inside it alone.

**The test that reached the old refusal shows it cannot be reached.**
`a_solid_whose_skin_only_closes_with_another_is_refused_a_copy` built the only
model that got there, a cube split three and three, and read it from a file.
It is now `a_solid_whose_skin_only_closes_with_another_never_reaches_a_copy`,
which shows the door refusing those bytes and `check_shape_is_possible` naming
the cut.

What else could hand `copy_solid` such a solid: the public ways to change a
model. `SolidBuilder` describes one shell and sweeps a scratch model holding
only the new solid before it writes. Each `add_*` shape builder writes one
closed shell and runs `check_touched_solid`. The fuse sweeps its throwaway
answer before writing and checks what it wrote. `place_solid` and the three
approximation setters do not change which shell a face is in. If a split skin
turned up anyway, it would be a bug in this crate, and `copy_solid`'s own
`check_touched_solid` would panic on the copy.

### 342.4 The pins

- `tests/a_skin_that_closes_only_with_another_never_reads_back.rs`: all 31
  cube cuts and all 3 bar cuts refused at the door, the bar half whose fuse
  panicked refused while the whole bar still fuses, and a control. The control
  shows that rewriting the end of the file with every face in one shell gives
  back the file byte for byte. The halves are made by rewriting the shell and
  solid arenas at the end of the postcard bytes, which is how Q84's probe built
  them.
- `validate.rs`'s
  `a_skin_shared_out_between_two_solids_is_caught_every_way_it_can_be_cut`: the
  same 34, built through the test-only `data_mut`. Each is refused by name by
  `validate`, by `validate_solid` of each half, and by `check_shape_is_possible`.
- **The door's refusal carries no name.** postcard's `de::Error::custom` drops
  the message, so a refused file is `SerdeDeCustom` whatever refused it. So the
  file pins *that* the door refuses, and the unit tests pin *why* by asking the
  door's own check of the same data.
