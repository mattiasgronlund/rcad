## 45. The door, closed — recorded 2026-09-03

*Eleven lanes over `b3e5670..HEAD`. Every number here was re-derived by a closing
gate at the final tree unless it says otherwise; where a lane and the gate
disagree, the gate's number stands and the lane's is kept beside it, because the
difference is usually the finding.*

### 45.1 What it bought

| | `b3e5670` | now |
|---|---|---|
| **KiCad imported** | 46 / 323 = 14.2% | **97 / 323 = 30.0%** |
| NIST imported | 0 / 44 | 0 / 44 |
| control | 88 / 88 | 88 / 88 |

Two sentences, and the split between them is the whole of §45.2:

- **A file whose corners are as close as an `f64` can put them is no longer
  refused for being off** — §44.3's first sentence, **+50 files**.
- **A face bounded by an arc and its chord is a face** — §44.3's second, **+1**.

Four things the corpus does not measure landed beside them: criterion 8 exists
(`overlap.rs`), a `Model` off the wire can no longer switch off the certified
check, the mesher stopped meshing a wrong region round a ball's pole, and a
green suite now means the corpus ran.

### 45.2 §44.3 priced the second sentence at 75 files. It is worth one.

Of the 75 files whose *first* refusal was the wire rule, **1 now imports and 74
meet a second blocker immediately** — 63 into the curved-face bucket, 11 into
`B_SPLINE_CURVE_WITH_KNOTS`. The gate instrumented `b3e5670` to emit per-file
reasons (that reporting does not exist at the old tree) and diffed 321
before-rows against 270 after-rows to get it, rather than trusting the totals.

**§44.11 item 5 was written about the curved item and governs both.** It says a
count of files a blocker stops *first* is an **upper bound on what removing it is
worth, never a projection**. Scored:

| item | priced at | delivered |
|---|---|---|
| curved writer's rounding | 88 | **50** |
| the two-step wire | 75 | **1** |

Both of §44's headline file counts were upper bounds wearing a projection's
clothes. The rule is now demonstrated twice and should be applied to every
future count of this shape — including the 68 B-spline files, which are the next
number somebody will be tempted to treat as a payoff.

### 45.3 The refusal name is not the reason, so option D cannot be scored

§44.6 wrote its option-D trigger as a fraction *"so it can fire"* and deferred
scoring to this gate. **The gate refuses to score it, and it is right to.**

| | KiCad refused | refused for a miss > 1 spacing | trigger |
|---|---|---|---|
| as published | 226 | 128 | 56.6% — **fires** |
| less the 57 relabelled | 226 | 71 | 31.4% — **does not fire** |

`SolidBuilder::build` failing for *any* reason sends `brep.rs` to
`a_corner_provably_off_a_curved_face`, and if a bare exact `side_of` finds any
corner off any curved face, that name wins ahead of the real reason. Since the
curved lane landed, **that criterion is strictly stricter than the one
`Model::validate` now uses**, because it carries no `NEAREST_F64_POINT_ROUNDOFF`
slack. `brep.rs` is not in this unit's diff; the divergence was made by changing
`validate` around it.

**The demonstration, which no lane had:** of the 63 ex-wire files that landed in
that bucket, **0 are inside one coordinate spacing**, against 40 of 168 (24%)
for the corpus as a whole. A bucket of genuine corner misses does not have a
zero there.

**So the first thing the next unit owes is a first-refusal count that `brep.rs`
has not overwritten.** Until then no statement about option D is available in
either direction, and §44.6's own arithmetic is unusable for the reason in §45.4.

### 45.4 §44.11 item 6, settled — and it was three mistakes, not one

§44.11 item 6 recorded §43.5's KiCad table as `153+59+29+13+8+23 = 285` against
277 refused, over by 8, and could not tell which row was wrong. Two lanes gave
different answers and both were right about different objects.

1. **The sum.** The measured tail at `b3e5670` is `6+3+2+2+1+1 = 15`, and
   §43.5's own text reads "… `SURFACE_OF_LINEAR_EXTRUSION` 1, **and 8 more**".
   `23 = 15 + 8`, where that 8 is the arc-plane row already listed above it.
   `153+59+29+13+8+15 = 277` exactly. **A double-count in the prose; the tree's
   counts were always sound.**
2. **The row.** It is also not a first-refusal count, for §45.3's reason. Both
   things are true at once.
3. **A third quantity nobody had checked.** §44.6 argues from *"161 KiCad files
   with a corner off a curved face"* exceeding *"the 153 whose first refusal it
   was"* by 8. **There is no such set of 161.** `corner_misses_in_spacings` is
   pushed for **both** corner-miss variants, so the reported 161 is `153 + 8` —
   the same 8 a third time. §44.6's 65–73 range and its "50% to 56%" rest on it.

The lesson is narrower than "check the arithmetic": three different sentences in
two sections all reused one number without asking what population it came from.

### 45.5 The `ModelData` table — §44.5 deliverable 4, published here

§44.5 asked for this table and §44.10 made publishing it a verification line.
The lane produced it in its report and **it never reached the tree**; the gate
caught that the line was unmet. It is published here. "Guard" means: does
something refuse a bad value **on the deserialize path**, which runs
`check_keys_resolve`, `check_nothing_is_used_more_than_once` and
`check_shape_is_possible`, and nothing else.

| field / payload | serde | guard at the door | what a hostile value cost |
|---|---|---|---|
| `ModelData.scale` | `try_from` | yes — `Scale::new` | — |
| the seven `SlotMap`s | derive | partly — slotmap refuses ≥ `u32::MAX` slots and rebuilds `num_elems` and the free list itself | — |
| `VertexData.point` | `from` — cannot refuse | **now yes** — `VertexNotFinite` | every predicate downstream asked about a coordinate that is not a number |
| `VertexData.approximation` | was plain derive | **now yes** — `try_from` | **an infinite magnitude switched the certified check off**: `not_provably_off` keeps any magnitude `> 0.0`, widening the corner to an interval reaching every surface, and `validate` returned `Ok` on a solid whose corners are nowhere near its faces |
| `EdgeData.start/end` | derive | yes — `check_keys_resolve` | — |
| `EdgeData.run` | derive | leaves yes — every `Surface` and `Picker` arm is `try_from` | a run naming a curve the edge is not on is caught in `validate` only, and correctly so: it needs a `Policy` and a predicate |
| `EdgeData.approximation` | was plain derive | **now yes** | as vertex |
| `CoedgeData.edge` / `.orientation` | derive | yes | — |
| `WireData.steps` | derive | keys yes; shape yes; **multiplicity now yes** | see §45.6 |
| `FaceData.surface` | `try_from` | yes | — |
| `FaceData.wires` | derive | keys yes; non-empty yes; **multiplicity now yes** | see §45.6 |
| `FaceData.approximation` | was plain derive | **now yes** | as vertex |
| `ShellData.faces` | derive | keys, `loc` and `orient` yes; **multiplicity now yes** | see §45.6 |
| `SolidData.shells` | derive | as `ShellData.faces` | see §45.6 |
| `Model.worst_approximation` | not stored | recomputed | a file cannot lie about it |
| `Model.id` | not stored | minted per read | old keys are refused against the new model |

`Point` and `Vector` keep `#[serde(from)]` deliberately (§44.5): a fallible
branch on every coordinate in the kernel is a real cost, and finiteness is a
condition a sweep already implements.

### 45.6 The hole that was not a field

§44.12 predicted lane 0's audit would find a third unguarded **field**. It found
something worse and of a different kind: **nothing at the door counted how many
times an entity is used.**

A cube whose second face names the first face's wire read back cleanly —
`check_keys_resolve` passed, `check_shape_is_possible` passed — and
`Index::rebuilt_from`'s `insert` **silently gave the wire to whichever face was
written last**. Six faces, one wire each, and the index disagreeing with the data
it was built from before anything asked a question. `Model::validate` caught it;
`validate` is not on that path. Found by a lane, reproduced independently by the
gate, and closed last.

**Where the check runs is the part worth recording.** §44.5's rule said it
belongs in `check_shape_is_possible`, and it does not: that is a method on
`Model` and needs `self.index()`, so it can only run *after* the index exists —
and the index is what the count protects. It runs in `Deserialize` beside
`check_keys_resolve`, before `from_parts`. Four counts — coedges by wires, wires
by faces, faces by shells, shells by solids — one `SecondaryMap` each, O(n).
`rebuilt_from`'s two inserts now **assert unconditionally** rather than
`debug_assert`, because by then the bytes are vetted, so a second claimant is a
broken internal invariant, and a `debug_assert` is absent in exactly the build
where a user opens a stranger's file.

**The symmetric hole is still open.** `Invalid::Orphan` — a face in no shell, a
wire on no face, a coedge in no wire, a shell in no solid — qualifies under the
same rule and is still not at the door.

### 45.7 Three measurements that changed a verdict

**The skip.** §44.10 asked for the hit count of
`check_which_side_of_its_trimming_each_curved_face_lies_on`'s `continue`, and
§44.12 framed the answer as a fork: *thousands makes it a correctness lane, zero
means two doc comments disagree.* It fires **33 004** times — and it is still not
a correctness lane, because in **32 965 of 33 005 raises the face had no whole
closed trimming at all**. A per-edge rewrite recovers **40 face-visits across the
entire suite**, and that 40 is identical in debug, in release and in the lane's
own log while the percentage moves with the denominator. Neither doc comment is
wrong: they are about different objects, and §44.12 read them as contradicting.
What survives is sharper — *"the fuse will not make a straddling face"* is an
unreachability claim **the sweep is in no position to have tested**, since it is
skipped on 96.1% of the fuse's own curved face-visits. The real finding is a
coverage number nobody had: **3.90%** over the fuse's own swept curved output,
2.18% over KiCad. The remedy is a second invariant that can speak about an arc,
and that is milestone-sized.

**A number that went stale inside the unit.** The same lane measured NIST at
**0** and was right. By the final tree it is **204**, because `door3-twostep`
let two NIST files build far enough to reach the skip while still being refused
at import. The gate proved it by rebuilding the probe at `d680434` and
reproducing the lane's `0` exactly. **A lane's number can be true when taken and
false when merged**, which is the argument for a gate re-deriving at the final
tree, demonstrated rather than asserted.

**The round-trip charge.** §43.2 scored criterion 12 unmet and cited
`fuse/select.rs:1045`. That site round-trips through a surface's *own two
coordinates* and is charged correctly; the uncharged trip was in
`rcad-topo/src/mesh.rs`, which §44.4 filed under "the mesher" while `rcad-mesh`
contains no `transformed` call at all. And **`18·ε` was never a claim about this
operation** — it is `Isometry::POINT_ROUNDOFF`, what the deflection floor
charges, so §41.8's "fourteen times short" compared two unrelated constants. The
derived cost is `36·ε·L` for a ball (a centre) and `72·ε·L` for a pipe, cone or
ring (a point *and* a direction); worst measured `4·ε·L`. A real gap, of the same
order as the charge rather than 14× it.

### 45.8 The predictions

| prediction | score |
|---|---|
| §44.12 #1 — the money goes to the two-step wire, not the curved rounding | **not borne out by cost, right about shape.** The curved lane is larger by insertions and paid 50 files; the two-step paid 1. But the two-step lane was the messier: two rules in two files, a winding exception, a fixture wrong since it was written, and a mesh gap it could not close |
| §44.12 #2 — lane 0's audit finds a third unguarded field | **wrong as worded, right in spirit** — see §45.6. It is a multiplicity, not a field |
| §44.12 #3 — criterion 8 goes unstarted a third unit | **wrong, and the gate got the outcome it wanted** |
| §43.10, inherited — corner slop first, criterion 8 second, the fuse's unjudged `expect`s third | **first two done, third untouched for the third unit running.** `.expect(` in `crates/rcad-topo/src/fuse/` is **147** at both ends of the unit, 30 of them production. `arrange.rs` was rewritten by +328 lines without adding or judging one |

**§12.1's lesson holds a third time in a new form.** The money did not go where
either the plan or the last gate said. It went to work nobody had scheduled at
all: a wrong region meshed silently round a ball's pole, a door that let a
self-contradicting model through, and a fixture that had been inside out since
the day it was written. All three were found by *removing a restriction* and
seeing what the tree did next.

### 45.9 Claims that did not survive

Every lane ran `claim-audit` on its own work and corrected itself — five
sentences in one lane, four in another, three in a third. Those are in the
commits. What a lane auditing itself could not see, and the gate found:

- **A §15.6 search with two assertions that cannot fail.** The headline
  constant's own test presented four checks per bracket as independent; two were
  tautologies (`spacing ≤ budget` holds for any point and names no surface).
  Now sorted explicitly into what tests the size, what proves the search is not
  vacuous, and what is bookkeeping — **and a search that can actually fail was
  built**: Pythagorean quadruples put a point exactly on a ball, pipe, cone or
  ring with no arithmetic, one IEEE division rounds it, and the distance moved
  is compared with the budget. Worst shares 0.508 to 0.584, and **all four
  surfaces fail when the constant is halved.**
- **A doc claiming a tighter ceiling than its code asserted** — `mesh.rs` said
  `36·ε·L` and asserted `72·ε·L`. Charging each surface its own ceiling
  uncovered what the shared number hid: **all 480 ball faces stray exactly
  zero**, because `add_ball` stores its sphere on the origin of its own
  placement and `M⁻¹(M(origin))` cancels to the bit. The ball's half of a
  "two roads must differ" test had been comparing a road with itself.
- **A ratchet that reset its own instrument.** `wildcard_arms` 90 → 87 reads as
  three arms going away and is one: measured both scans on both trees, the
  composite half moved entirely because it started counting uncovered variants
  instead of sites. The four bridging rows are now in the `justfile`.

### 45.10 What stands in the door now

**Correctness, in the order a next unit should take them:**

1. **A first-refusal count `brep.rs` has not overwritten** (§45.3). Everything
   published about why files are refused depends on it, and option D cannot be
   scored without it.
2. **`Invalid::Orphan` at the door** (§45.6) — the same rule, the other half.
3. **The fuse's 30 production `expect`s**, unjudged for a third unit. §43.3
   demonstrated eleven such clauses reachable with caller data in one milestone;
   `arrange.rs` grew by a third this unit without any of them being looked at.

**Named limits and gaps, each with its demonstration in the tree:**

- **A plane resting on a pipe leaves that crossing with no direction.**
  `∇S × ∇Q` is exactly the zero vector along the tangent ruling. Four families
  of fix are ruled out at the refusal site in `arrange.rs`, including the sharp
  one: storing a direction on `Crossing` is **necessary but not sufficient**,
  because `det(∇S, ∇Q₁, ∇Q₂)` vanishes identically once `∇Q₁ ∥ ∇S` and the
  curvature fallback dies with it. Both derivatives the engine takes are blind.
- **A quarter arc contributes no corner at `as_coarse_as_a_mesh_gets`**, so a
  plate bounded by one will not mesh. `a_plate_bounded_by_a_quarter_arc_and_its_chord_still_will_not_mesh`
  is the only `#[ignore]` in the suite and names its own fix. **Nothing runs
  `--ignored`**, so it will stay quiet after the fix lands.
- **A face bounded by two half-circles** sweeps exactly zero winding and is
  refused. A gap: the runs carry the answer and nothing reads them.
- **The writer has no `SURFACE_CURVE`** for the circle where a flat face meets a
  round one, so 52 of the 97 importing files do not round-trip. Nothing that
  used to round-trip stopped.
- **The sweep sees 3.90% of the fuse's own curved output** (§45.7).

**A prediction, for whoever scores it.** §44.2 dropped the a-priori criteria
list and replaced it with a verification list in which every line names
something in the tree that changes state. That worked: three of §44.10's lines
were found unmet or unmeetable by a gate, and one of them — the `ModelData`
table — would have been recorded as done on a lane's word. **So the guess is
that the next unit's biggest loss is again a number that was true when a lane
took it and false when the unit closed**, because that failure mode survived
this unit's own precautions and was caught only by re-running at the final tree.
The second guess is that `brep.rs` turns out to have been distorting the
refusal tables for longer than this unit, and that some earlier published rate
moves when it is fixed.

---

