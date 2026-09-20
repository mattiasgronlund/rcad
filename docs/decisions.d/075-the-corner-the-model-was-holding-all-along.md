## 75. The corner the model was holding all along, recorded 2026-09-08

### 75.1 A paragraph describing a filter that was not there

`derive_a_corner` in `crates/rcad-topo/src/fuse/bake.rs` carried this sentence:

> Three surfaces can meet at more than one point, and the stored corner is what
> says which: it is a cached sample of the truth (§17.1), so the box it falls
> inside is the one. Two boxes that both hold it are two answers to one question
> and the triple is passed over.

The solving loop was:

```rust
let Ok(MeetOfThree::At(places)) =
    meet_of_three_surfaces([&triple[0], &triple[1], &triple[2]], policy, log)
else {
    continue;
};
let [only] = places.as_slice() else {
    continue;
};
```

`let [only] … else { continue }` passes over **every** multi-place triple, not
the ambiguous ones the paragraph describes. And the function could not have done
what the paragraph said: `vertex: VertexKey` was read once in the whole body, to
be copied into the returned `BakedCorner`, so the loop that places the point had
no stored corner to consult. §70.3 spotted this while measuring something else
and called it "the §16.10 shape again".

**It is a gap and not a limit.** The answer exists: two planes and a pipe cross
at both ends of a chord, `MeetOfThree::At` hands back both boxes, and which of
them a corner is, is a fact the model has been holding since it was built. A
corner like that could only be placed by some *other* triple round it, and
refused as `Error::Undecidable` when there was none — a representable answer
turned away.

The consumer is named, so §13 does not defer it: the fuzz target
`two_solids_fused_by_a_stranger` decodes an arbitrary model, and a stranger's
solid is under no obligation to keep its curved corners on well-conditioned
triples. Nothing this milestone *builds* has such a corner — every builder in
`build.rs` makes curved corners with exactly two surfaces round them (a cap
circle) and flat ones with three — which is why the demonstration below is a
unit test on the three surfaces and not a solid.

### 75.2 The fix: an overlap of two boxes, which needs no tolerance

`the_place_the_model_points_at` is the whole of it.

One place and the model is not consulted at all — the surfaces gave a single
answer and there is nothing to choose. That is every corner of every flat solid,
because three planes meet at one point or at none, so nothing about the
polyhedral half of the kernel turns on any of what follows.

More than one, and the place whose box **overlaps** the box the model's own
corner is in is the one. Both are certified enclosures of true points, so:

- the comparison needs no tolerance, no constant and no policy — it is
  `lo <= other.hi && other.lo <= hi` per axis, the same shape as
  `Bounds::provably_misses`;
- a pair that misses is a **proof** that the model's corner is not that meet,
  not a hint that it is far from it.

That is what the trap in this lane's brief asked for. §70 is about a model that
declares how closely it will be checked; nothing here reads a number the model
declares, and nothing here scales with one.

Two places overlapping, or none overlapping, both give `None`: the first is two
answers to one question, the second is the model's corner and its surfaces
disagreeing by more than either box allows. **Neither is a refusal.** It is that
triple saying nothing about this corner while another may still place it, which
is the same standing `MeetOfThree::Nowhere` and `MoreThanAPoint` already had
(§34.9).

**The model's corner never places the point.** It selects among places the
surfaces already agree on, and the geometry comes from the meet. A wrong cached
corner therefore cannot move a right answer — the worst it can do is fail to
pick, which costs a refusal and never a wrong solid. That is the §17.1 reading
of "cached sample" taken seriously in both directions.

**The doc now matches the code, and the change went that way round.** The
paragraph as written was true of the design and false of the code, so the code
moved. Two sentences were added rather than rewritten: that a single place is
taken without consulting anything, and that no place overlapping is also a pass.
Neither was in the original and both are behaviour a reader has to know.

### 75.3 Where the model's corner comes from, and an assert that moved

`place_a_corner` already worked out where the model puts a corner — that is what
a corner with fewer than three surfaces round it *is*, read out and moved (§31.3,
§34.6). It is now `where_the_model_puts_a_corner`, and the two-surface answer and
the three-surface selector are the same call:

- placement is `Isometry::identity()` → the stored triple, exactly, because a box
  round a move that rounded nothing would turn an exact corner inexact and that is
  the whole of §17.7's lattice control;
- otherwise `where_a_corner_lands_after_moving`, whose box is what the move cost,
  through `as_exact_as_it_is`.

Two things follow that are worth writing down.

**The placement assert now covers both routes.** `bake`'s doc has always said it
panics when the faces round one corner sit at different placements, and the
assert lived inside the fewer-than-three branch, so the documented panic was only
half enforced. It is now in `where_the_model_puts_a_corner`, which both routes
call. A corner in two places at once has no world position for anything to
select against; a solid like that is broken rather than awkward, and §4.3's
"corrupt is worse than a crash" is why this is an assert and not an `Err`.

**The joined-face path pays for it too.** `corner_of_every_vertex_left` handles a
solid whose coplanar faces were joined on the way in, and it now builds the same
`PlacedSurface` set as the ordinary route to get each corner. That is work it did
not do before and it buys nothing on that path — everything there is a plane, so
the selection is never consulted. It was taken anyway, because the alternative is
`derive_a_corner` having a second contract for one caller, and a second spelling
of one question is how two callers come to disagree (the argument
`side_of_surface` is already written on).

### 75.4 A prune, and why it changes no answer

The search stops at the first triple that solves to an exact point:

```rust
if point.uncertainty() == 0.0 {
    break 'every_triple;
}
```

This is not an approximation of the old behaviour, it is the old behaviour. The
widths were compared with `<`, so once `best` held a box of no width no later
triple could displace it; the loop was running `meet_of_three_surfaces` for
triples whose answers it had already decided to throw away. §70.3's cost argument
is that `derive_a_corner` is cubic in the faces round a corner — twenty meets for
a corner where six faces meet — and this makes the well-conditioned case stop at
the first.

It bites on nothing this milestone builds. Every corner of every solid in the
suite has exactly three faces round it, so there is exactly one triple and
nothing to break out of. It is written for the corner §70.3 named and the fuzzer
can reach.

### 75.5 The benchmark, re-run

§70.3's four numbers, re-measured on this machine with a throwaway probe (a
crate under the session scratchpad that path-depends on this worktree, built
`--release`, each figure the mean of a warmed loop). Nine runs of the binary,
alternating between the base `bake.rs` and this one:

```
                                                    §70.3     measured here
Plane::signed_distance_to                            3 ns     1.6 – 8.8 ns
meet_of_three_surfaces, three perpendicular planes  571 ns     549 – 743 ns
Model::validate over one block                   13.519 µs     11.2 – 13.4 µs
Model::fuse of two blocks                         1.353 ms     1.29 – 1.39 ms
```

`Model::validate` and `Model::fuse` land on §70.3's numbers. The two nanosecond
figures do not settle to a single value on this machine — `signed_distance_to`
reads between 1.6 ns and 8.8 ns depending on what the surrounding loop got
hoisted, and `meet_of_three_surfaces` swung 30% between two runs of the *same*
binary. **So the 190:1 ratio in §70.3 is not reproducible to two figures here**
and nothing below is argued from it; what survives re-measurement is the order,
which is two to three hundred times, and §70.3's conclusion rests on that.

The 34% figure survives too, and gets worse rather than better: eight corners of
one block at one triple each is `8 × 549 ns = 4.4 µs` against a sweep of
`11.2 µs`, which is 39%.

**Before and after this section, `Model::fuse` of two blocks:**

```
base 41b7160    1.347 ms   1.389 ms   1.288 ms
this section    1.386 ms   1.307 ms   1.302 ms
```

Indistinguishable, and that is the expected result rather than a lucky one: a
block's corners have three faces each, so the prune has nothing to break out of
and the only added work is one `where_a_corner_lands_after_moving` per corner of
a placed solid — sixteen calls against a millisecond. **The consultation is free
on the shape the kernel builds today**, which is what makes it affordable to put
in the path of every corner rather than behind a flag.

The command, for whoever re-runs it:

```
mise exec -- cargo build --release --manifest-path <scratchpad>/corner-cost/Cargo.toml
<scratchpad>/corner-cost/target/release/corner-cost
```

The probe is not checked in. It measures four things across three crates and
would be a fifth `benches/` file for one paragraph; §13's rule about a part with
no named consumer covers a benchmark as well as a feature. What is checked in is
this section, which is what the next reader needs.

### 75.6 What §70.3 owed, and what is still owed

§70.3 owed two things to `bake.rs`. This settles one and a half of them.

**Settled: the consultation.** `derive_a_corner` reads the model's corner, the
doc and the code agree, and there is a test either way — a corner the model
places at one of two poles, and a corner whose box holds both and is therefore
passed over.

**Settled: the comparison, as a selector.** §70.3 wanted "nothing compares a
baked corner with the corner the model stored" fixed, and the overlap test is
that comparison, made where §70.3 said it belonged — in the thing that
re-derives, which is the only thing holding both points.

**Was owed, and the three reasons it was left owing were all wrong.** What stood
here said the overlap test could not be made universal, and gave three reasons
for it. A *single*-place triple was indeed still taken without asking whether the
model's corner was anywhere near it, so the ill-conditioned corner §70.3
describes — three surfaces at a grazing angle, a corner close to all three and
far from where they cross — did still pass. That much was true. Each of the three
reasons was disproved by measurement in §78, which took the one line and is where
the detail lives. The three corrections belong here, at the sentences that were
wrong.

*The evidence was not evidence.* This paragraph read the `1e-12` of slack in
`re_deriving_the_corners_of_a_turned_cube_gives_boxes_that_hold_them` as a box
that sometimes fails to overlap a box. It is not that comparison at all:
`corners_as_stored` goes through `VertexView::point()`, which hands back three
bare `f64` and no box, so the test holds a certified box against a plain point
and the slack sits between those two. Set the slack to `0.0` and the test passes
— **72 coordinates checked, every one strictly inside, worst margin 1.665e-16**.
Two lanes measured that independently and the second corrected the first, which
had reported 1.776e-15: that figure is the *largest* low-side margin, the
loosest of the 72 and not the tightest. Whether a box can miss a box on curved or
grazing-angle input is untouched by any of this — the `1e-12` was never evidence
either way, in either direction.

*The named error was the wrong one.* Refusing a single-place triple does not
produce `Invalid::VertexOffItsFacePlane`. It leaves `derive_a_corner` with no
placement at all, which is `Error::Undecidable` with a record naming
`WHERE_THREE_SURFACES_MEET`. `VertexOffItsFacePlane` comes from the
`side_of_surface` sweep further down the same function, and that sweep tests the
**derived** point — a stored corner sitting off its own plane never reaches that
arm.

*And "solids that fuse today" is not the set `validate` already refuses.* This
paragraph reads as though a stricter `bake` could only refuse what
`Model::validate` refuses anyway. Nothing gates `bake` behind `validate`.
`bake()` is the first step of `Model::fused_under`; the self-check
`Assembly::commit` runs *after* it is `Model::check_what_was_just_written`,
which asks eleven of the sweep's thirteen questions of the entities that one
fuse wrote and of nothing else, so no build runs a `validate()` of the model
inside `commit()` at all (§82.4); and
`a_union_can_be_united_again_because_every_corner_of_it_still_has_three_planes`
(`crates/rcad-topo/tests/fusing_solids.rs:799`) feeds one fuse's raw output
straight into the next fuse's `bake()`. Whatever a stricter `bake` refuses, it
refuses on its own account and nothing upstream has filtered it first.

**What the one line actually cost.** Bit for bit, 30 solids the kernel had
already accepted, plus a changed reason for 3 already refused. With the stored
corner given the room that writing it down costs, none — no published number
moved. §78 has the sweep, the mechanism worked out in exact rational arithmetic,
and the shape of the 33.

Every `how_often_*_refuses.rs` suite still publishes the rate it published
before — `mise exec -- just t -E 'test(/how_often/)'` is green, and those suites
assert their own numbers, so green *is* the statement that nothing moved. That
is what "a single place is taken exactly as before" predicts. The whole gate was
green at **1808 tests, 0 skipped** on this unit's tree, up two from 1806 — the
two new ones. Read that paragraph as the record of this unit and not as the
current shape of the code: a single place is no longer taken as before, and
§78.7's sweep is why the rates are unmoved anyway.

**That caveat was aimed at what the code does, and it did not reach the count or
the command.** Both stood here in the present tense, which reads as a claim about
today: the recipe was `just default` until 2026-09-08 and is `just gate` now, and
the tree §78 closes on is at **1815 tests, 0 skipped**. Neither figure is a bound
anything asserts — each is the record of the tree it was measured on, and the
next unit's will differ again.

