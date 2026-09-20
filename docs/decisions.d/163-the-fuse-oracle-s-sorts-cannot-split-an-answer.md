## 163. The fuse oracle's sorts cannot split an answer, and one of them merged two — §151.5 answered, 2026-09-19

*§151.5 asked whether face order, wire order or solid order — each a
`sort_unstable` in `fuses_that_must_agree.rs` — can tell one answer from itself.
**They cannot**, and that is now a test rather than a paragraph. The other
direction had a hole: sorting **every** wire of a face forgot which one is the
outline, and a washer with its rim and its hole swapped wrote out as the same
face. The outline is kept first now and only the holes are sorted. What the
comparison still forgets — a two-corner ring's direction, and the edges' runs —
is pinned and left open (§163.4).*

### 163.1 Split: arena order cannot make one answer two

Sorting a list of `Vec<u64>` is a canonical form of the multiset it holds:
`Vec<u64>`'s order is total and says `Equal` only for identical vectors, so two
elements `sort_unstable` might put either way round are the same vector and
stability cannot show. The ring rule is §151's smallest rotation, a canonical
form of a cyclic list.

That argument is backed by two tests, both of which shuffle an answer and
insist `canonical()` does not move. Every solid's faces, every face's holes
and the solids themselves are dealt out in a new order (a Fisher–Yates shuffle
driven by SplitMix64), and every ring is started from a new corner:

- `dealing_an_answer_out_in_another_order_does_not_change_it` — a property over
  `a_lattice_pair`, all three rules, a fresh seed per case. Real answers, as
  the kernel hands them over.
- `every_order_of_one_answer_with_two_holes_in_a_face_writes_out_the_same` — one
  hand-built answer dealt 256 ways. A lattice pair only sometimes has two
  solids or a face with two holes, so the property alone could pass with a sort
  missing; this answer has two solids, three faces in one of them, a face with
  two holes and rings of four corners, so every level is certain to be reached.

### 163.2 Merge: which wire is first is part of the face

`FaceView::wires` says the first wire is the outline and the rest are holes.
That is not decoration:

- `fuse/assemble.rs`'s `rings_round` builds a joined face's wire list outline
  first, by winding, on purpose.
- `holds.rs`'s `what_a_flat_face_holds` counts a round wire `+1` in first place
  and `-1` anywhere else.
- `validate.rs`'s `check_flat_face` refuses an outline that winds against the
  normal and a hole that winds with it — **but only for a wire of three corners
  or more.** Fewer is skipped: a round wire has one corner and nothing to wind.

So for a polygonal flat face the winding, which the oracle does write down,
already gives away which ring is the outline in any model `validate` accepts.
For a round wire it does not: the oracle writes a round wire as its one corner,
and after sorting every wire, the order was the only place the difference
could have lived. `a_washer_whose_hole_is_handed_over_as_its_outline_is_another_answer`
builds that washer both ways; before the change it failed with

```text
assertion `left != right` failed: a washer with its rim and its hole swapped wrote out as the same face
  left: [[[0, 0, 0, 4607182418800017408, 0, 0, 2, 3, 4607182418800017408, 0, 0, 3, 4611686018427387904, 0, 0]]]
 right: [[[0, 0, 0, 4607182418800017408, 0, 0, 2, 3, 4607182418800017408, 0, 0, 3, 4611686018427387904, 0, 0]]]
```

`written_out` now writes the outline first and sorts the rest. **Nothing else in
the suite moved**: all 51 tests pass with it, including every swapping
property, so the kernel puts the same wire first whichever way round a pair is
handed over, on every family the file builds. Curved faces get the same rule.
The kernel reads no meaning into a curved face's wire order today, but it does
not promise any order either, and one rule for every face is cheaper to reason
about than two. The suite says that costs nothing now; if a curved family ever
splits on it, that is the place to look.

This was a hole in the oracle and not a defect found in the kernel. No fuse
has been seen handing a round hole over first.

### 163.3 That each piece discriminates

Checked by taking each piece out and running `just t -p rcad-topo --test
fuses_that_must_agree --no-fail-fast`, not by argument:

| taken out | tests that fail | of them new here |
|---|---|---|
| `faces.sort_unstable()` | 11 | both dealing tests |
| `solids.sort_unstable()` | 6 | both dealing tests |
| `holes.sort_unstable()` | 3 | both dealing tests |
| turning to the smallest rotation (start at corner 0) | 8 | both dealing tests and the two-corner pin |

The property's report with the face sort gone reads *"dealing the answer out in
another order changed the answer to a Union: 1 solids and 6 faces one way, 1
solids and 6 faces the other, solid 0 face 0 differs"*.

**§151.5's "none of them has a test of its own" was true, but they were not
unexercised.** The existing swapping properties already fail without the face
sort, the solid sort and the ring turn, because the kernel's arena order really
does differ between the two ways round. The hole sort was the thin one: only
`swapping_the_two_sides_of_a_degenerate_pair_changes_not_one_bit` noticed it
missing — so the kernel does hand a face's holes over in different orders
either way round, and the sort is load-bearing.

### 163.4 What the comparison still forgets, and why it is left open

The `Answer` records a face's surface, whether it is used reversed, and each
wire's corners. It does not record the **edges' runs**, and a ring's direction
is visible only through its corners:

- A **two-corner ring** walked backwards is a rotation of itself, so the
  smallest rotation reads it the same either way. Pinned by
  `a_ring_of_two_corners_reads_the_same_walked_either_way`, which fails the day
  a ring's direction is written down another way.
- A **one-corner round wire** has no order at all.
- Two faces bounded by the same corners along different arcs write out the same.

Whether any of that lets two **whole valid answers** write out the same has not
been shown in either direction. At the face level it plainly does: the two caps
of a ball cut in half by a plane sit on the same sphere, are used the same way
round, and each has one wire whose one corner is the same point, so they would
write out identically. This was reasoned, not run. At the answer level
those two caps sit beside a flat face whose plane is stored facing the other way,
and that plane's coefficients are written down, so the two halves are told apart.
That is one example, not a bound. Settling it needs the edge runs, and the
closed-edge direction `validate` already enforces, in the written-out form — a
bigger change than §151.5 asked for, and one that should come with an answer
pair the current form really confuses. Until someone finds one, it stays
written down here rather than fixed.

**Answered by §165**: two such pairs were built and run, and the runs are in the
written-out form now.

Write set: `crates/rcad-topo/tests/fuses_that_must_agree.rs` (the change to
`written_out`, four tests, and the `dealt_out_again` / `Dice` helpers), this
section, and a pointer in §151.5.

---
