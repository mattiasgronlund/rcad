## 151. The fuse oracle compares rings by their smallest rotation, fixed 2026-09-19

*§149's change, made. `turned_to_its_smallest` in `fuses_that_must_agree.rs`
takes the smallest **rotation** of a ring instead of starting at the first
corner equal to the smallest one. The two rules differ **only when the smallest
corner occurs more than once**, so the suite's other forty-six tests cannot
notice, and the gate says they do not. §149.4's prediction that the lane's
fourth test would fail the day this landed was **wrong**, and the reason it was
wrong is worth more than the change.*

### 151.1 What was built

```rust
fn turned_to_its_smallest(wire: &[[f64; 3]]) -> Vec<u64> {
    let corners: Vec<[u64; 3]> = wire.iter().map(|&at| at.map(bits)).collect();
    (0..corners.len())
        .map(|start| -> Vec<u64> {
            (0..corners.len())
                .flat_map(|step| corners[(start + step) % corners.len()])
                .collect()
        })
        .min()
        .unwrap_or_default()
}
```

Every rotation is written out and the smallest wins. The rings are face wires,
so `n` is small and the `n²` does not show in the suite's runtime — the gate's
test step reads 27.3 s against 27.0 s before.

Comparing the flattened `Vec<u64>` is the same comparison as comparing the
sequence of corners, because every rotation is the same length and a corner is
a fixed three numbers wide.

### 151.2 Why the other forty-six tests cannot see it

**The two rules differ only when the smallest corner occurs more than once.**
If it occurs once, at position `p`, then every rotation starting anywhere else
begins with a strictly larger corner and is therefore lexicographically larger,
so the smallest rotation *is* the one starting at `p` — which is what the old
rule returned. The old rule was right about every ring with a unique smallest
corner, which is every ring the suite draws except the ones §149 is about.

That is also why this was so hard to see: the oracle was correct on everything
anyone looked at.

### 151.3 The test, and that it discriminates

`a_ring_that_goes_through_one_corner_twice_reads_the_same_from_either_side`, in
the file that owns the function. One four-corner ring through the origin twice,
written down from each of its two repeated corners, has to come back as one
list.

**Checked by putting the old rule back and running it**, not by argument: it
fails with

```text
left:  [0, 0, 0, 4607182418800017408, 0, 0, 0, 0, 0, 0, 4607182418800017408, 0]
right: [0, 0, 0, 0, 4607182418800017408, 0, 0, 0, 0, 4607182418800017408, 0, 0]
```

and passes with the new rule. Those are the two halves of the one ring, which
is §149.2 in twelve numbers.

### 151.4 §149.4 was wrong about the lane's fourth test, and why that matters

§149.4 said the lane's fourth test "asserts today's behaviour on purpose, and
its doc comment says the day the rotation change lands this test fails". **It
does not fail, and it never would have.**
`the_turned_pair_whose_union_lost_a_face_one_way_round.rs` cannot call into
another test binary, so it carried its **own copy** of both rules —
`the_suites_starting_corner` beside its own `turned_to_its_smallest`. Changing
the suite left that copy untouched, so the test would have gone on passing,
green and meaningless, asserting a property of a rule nothing used any more.

**A test named for another file's behaviour, holding its own copy of it, cannot
notice that file changing.** It is the §77.6 shape once more — a check passing
for a property of its own surroundings rather than of the thing it names — and
the tell is the same: take away what the test supplies itself and see whether
anything is left.

So the deletion was done by reading, not by watching a test go red. Removed:
the fourth test, the fifth, `the_suites_way`, `the_suites_starting_corner`, and
the `ring` parameter that `fused_by` and `written_out` carried so that two rules
could be run side by side — `fused_by` folded back into `fused` with it. The
fifth test's surviving half, "the smallest rotation is canonical for a ring that
repeats a corner", is §151.3 above, now asserted against the real function
instead of a copy.

What stays in that file is the pair itself and the three tests that are about
the kernel: it unites and intersects to the same solid either way round, and its
union really does have a ring through one corner twice while its intersection
does not. Worth keeping, because the suite draws that pair about once in 2 300
runs and these ask it every time.

### 151.5 What this does not close

The suite's `same_answer` is one canonical form among several this file builds
by hand, and only the ring rule was checked. Face order, wire order and solid
order are all `sort_unstable` over written-out lists, and none of them has a
test of its own; whether any of *those* can tell one answer from itself has not
been asked. **Answered by §163.**

§149's counterexample is the only ring-with-a-repeated-corner anyone has
produced. That it took a turn about a point both solids have a corner at
suggests the family is small, but nothing here bounds it.

