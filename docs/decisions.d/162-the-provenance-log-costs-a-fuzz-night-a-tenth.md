## 162. The provenance log costs a fuzz night a tenth of a percent — §144.9 measured, no feature built, 2026-09-19

*§144.9 asked for a stub `ProvenanceLog` behind the workspace's first cargo
feature, and asked for a measurement first. The measurement says no: recording is
**0.097% at worst** of what the four recording fuzz targets spend, and the worst
single draw in any corpus holds **99 kB**. The premise was half right — draws that
go deep do record, about thirty decisions each — and the conclusion does not
follow, because thirty records is nothing beside the fuse that made them. It is
the ratio that kills the feature, not a scarcity of records. No `[features]` was
added, and both of the two questions §144.9 told the lane to settle dissolve with
it.*

### 162.1 What was claimed, and which half of it held

`ProvenanceLog`'s doc comment said an empty log does not allocate, "so a
certified operation pays nothing for carrying one". §144.9 read that as
optimising the path a fuzz night never takes: `Record` owns a `Vec<Point>`, so
every recorded decision allocates, and finding refusals is what a night is for.
The cost the design avoids is avoided where it was never going to be paid.

Both halves of that are true as written. What does not follow is the conclusion,
and §144.9 knew it might not — which is why it said **measure before building**
and cited §16.10: "logging is expensive here" is a claim, and a claim owes a
demonstration.

### 162.2 How it was measured

`crates/rcad-step/benches/what_the_provenance_log_costs_a_fuzz_night.rs`, run by
`just provenance-cost`. Two passes over each target's draws:

- **fuzzing** — the draw run exactly as the fuzz target runs it, through the same
  shared harness file the target reaches into with `#[path]`. Nothing stubbed.
- **recording** — every record that draw made, built and kept and freed again:
  the `Vec<Point>` per record, the log's own growth, and dropping both.

Every field of every replayed record is read back off the real one, so the
predicate names, the input counts and the log's growth are the draw's own rather
than a guess at their shape. **That is precisely the work a stub `push` would not
do**, so the share of the two is the whole of what the feature could have bought
rather than an estimate of it.

The draws are each target's checked-in corpus, plus — for the two surface targets
— the 200 000-draw soak from
`every_pair_of_surfaces_the_fuzzer_draws_is_named_honestly.rs`, same generator and
same seed, so the distribution is one that file has already characterized. The
fuse gets no soak because random bytes do not decode as a `Model`: postcard
refuses them and the draw stops at the door.

**Getting the log out needed a door in each harness**, because every one of them
drops what it records — no fuzz target has ever read a log. Each harness now has
a `_keeping_the_provenance` form that takes the caller's log, with the old name as
a one-line wrapper over it, so the draw being measured is the same lines of code
as the draw being fuzzed.

### 162.3 What it said, on 2026-09-19

```
one Record is 96 bytes and one Point is 64, before the heap behind the inputs

target                                  how   draws recorded    fuzzing   records     rec/s   recording    share
where_two_surfaces_cross             corpus     834       10     6.383s        34         5     0.001ms  0.0000%
where_a_cone_or_a_torus_crosses      corpus      38        0     0.001s         0         0     0.000ms  0.0000%
two_solids_fused_by_a_stranger       corpus    1751      207     0.801s     16036     20016     0.774ms  0.0966%
no_step_file_makes_the_reader_panic  corpus     909        0     0.003s         0         0     0.000ms  0.0000%
where_two_surfaces_cross               soak  200000        6     1.666s        18        11     0.000ms  0.0000%
where_a_cone_or_a_torus_crosses        soak  200000       32     0.364s       128       352     0.003ms  0.0008%

the worst single draw, which is the one an rss limit would stop
two_solids_fused_by_a_stranger       corpus        434      99.3 kB
every other row                                   ≤ 4       0.6 kB
```

Three runs of the fuse row gave 0.0944%, 0.0964% and 0.0966%, so the figure is
stable to the precision it is quoted at.

**Memory is reported beside time because it is the reading that can fail a night
outright**, and a share of the wall clock says nothing about it. libFuzzer stops
a target at `-rss_limit_mb`, 2048 by default. The worst draw anywhere is 99 kB —
four orders of magnitude clear.

### 162.4 Where the records come from, which is why the ratio is what it is

The interesting number is not that records are rare. It is where they sit.
Measured over the fuse corpus by
`rcad-topo/tests/a_fused_draw_hands_its_provenance_back_through_the_door.rs`: of
1751 seeds, the **1216 that never got a model past `validate` recorded 4
decisions between them**, and the **535 that reached the fuse recorded the other
16 032** — about thirty apiece.

So §144.9's premise held. A draw that goes deep does record, and provenance is
not free on the path a night takes. What kills the feature is that thirty records
is nothing beside the fuse that made them: 20 000 records a second of fuzzing, at
a cost of 0.0966% of that second. A stub would hand back a thousandth of the run.

### 162.5 Why 0.097% is decisive rather than merely small

The share is an **upper bound on the saving** and a **lower bound on the cost**,
and both directions matter.

Upper bound on the saving, because a stub cannot give back more than the work it
removes, and the predicates underneath still run whatever the log does. Lower
bound on the cost, because a million live allocations push the rest of the kernel
out of cache and a replay into a warm allocator is not charged for that.

The largest honest correction runs the other way, and it is the profile. This is
the `bench` profile; a night runs a sanitized, coverage-instrumented build where
allocation is dearer relative to arithmetic. Call it a tenfold error in the
direction that favours the feature and the share is **under one percent**, against
a second build configuration `just gate` would have to run or let rot. Measuring
it under the sanitizer needs a nightly build of the bench and would move a number
that cannot reach the decision, so it was not done and this paragraph is the
reason.

### 162.6 §144.9's two questions dissolve

Both were conditional on the feature existing, and there is no feature.

**"A feature that is off compiles a second configuration that nothing exercises,
so `just gate` has to run both or the off path rots."** Nothing to run. The
workspace still declares no `[features]` and there is still no `cfg(feature = …)`
in any `src/`.

**"With provenance stubbed out, `Error::Undecidable` can no longer say why, so the
feature must never be off in a build anyone reasons from; whether that is
enforceable, or only documented, is the lane's question."** This was the harder
one and it is the better reason to be glad of the answer: the invariant that
would have been difficult to enforce is now simply not needed. Every build says
why.

### 162.7 The tests, and the one seam none of them holds

**A measurement taken through a door that stopped collecting reads nought
percent** — the answer it went looking for, arrived at by measuring nothing. That
is §77.6's shape exactly: a check passing for a property of how it was run rather
than of the tree. So each door is pinned:

- `rcad-geom/tests/a_fuzz_draw_hands_its_provenance_back_through_the_door.rs` —
  the corpus draw that records the most, a ball and a pipe out at `4e138`, found
  by the bench and decoded into eleven numbers by `rcad-fuzz-explain` rather than
  pasted in as 56 bytes of hex. The door answers what the front answers, and its
  log is not empty.
- `rcad-topo/tests/a_fused_draw_hands_its_provenance_back_through_the_door.rs` —
  the whole fuse corpus, because that target's worst draw is four and a half
  kilobytes of `Model` and a hex blob that long says nothing about what is being
  asked. It also holds §162.4's concentration: the records belong to the deep
  draws, asserted as a ratio rather than as either count, since both move whenever
  the corpus does.

**The reader's door is the seam no test can hold, and that is worth saying rather
than papering over.** Its row is a measured zero because none of its 909 seeds
imports at all — `reader.rs` records the same thing from the other side, 135 of
907 parsed and none imported — so there is no non-empty log to assert. That is a
property of today's corpus, unlike the writer's zero, which is a property of the
code: `writer.rs` names no `Policy` and no log, and every door that records in
this kernel takes a `&mut ProvenanceLog`, so a harness that never mentions one
cannot reach a record. `every_file_the_reader_fuzzer_kept_is_read_again.rs` still
covers the reader's *body*, because the front door delegates to it; what is
unheld is only the threading of the log, and the day a seed imports is the day it
becomes checkable.

The bench writes each target's worst draw to `target/provenance-cost/` and names
the path, for `rcad-fuzz-timing`'s reason for keeping its slowest units in full: a
count cannot be replayed and those bytes can.

### 162.8 Two things found on the way

**The reader was nearly written off on an argument.** The first cut of this bench
left it out with the sentence "neither goes near a predicate, so neither records
anything", about it and the writer together. That was true of the writer and false
of the reader, which imports and then sweeps, both through a log. The row was
added, and it came back at zero anyway — so the conclusion would have survived,
and the evidence for it would have been a guess wearing a measurement's clothes.
§16.10 again, in the small.

**A bench that reaches a test module gets `cfg(test)` without the test harness.**
`#[test]` bodies are stripped while the helpers beside them stay, so an import
used only inside one reads as unused. `fuzzing/mod.rs`'s `scaling_by_one` calls
`super::scaled` rather than importing it for that reason, which is the sort of
thing worth writing down once so the next reader does not delete the qualification
as noise.

Write set: `crates/rcad-step/benches/what_the_provenance_log_costs_a_fuzz_night.rs`
(new) and `Cargo.toml`; the `_keeping_the_provenance` doors in
`rcad-geom/tests/fuzzing/mod.rs`, `rcad-topo/tests/fuzzing/fuse.rs` and
`rcad-step/tests/fuzzing/reader.rs`; the two new door tests; `ProvenanceLog`'s doc
comment in `rcad-predicates/src/provenance.rs`; and the `provenance-cost` recipe.

---
