## 84. A key knows which model made it in every build, recorded 2026-09-10

§83.1 wrote the hole down and left it open on purpose. `ModelId` was a `u64`
from an atomic counter under `#[cfg(debug_assertions)]` and `pub(crate) struct
ModelId;` — one zero-sized value that every model shared — without them, so a
key from another live model resolved against a release build without a word.

There is one type now, the `u64`, in both builds
(`crates/rcad-topo/src/key.rs:43`). Two tests in
`crates/rcad-topo/tests/what_only_a_release_build_does.rs` say so, and the arm
§83 built is what runs them.

The reason it had been two types was cost, and most of this unit was pricing
that. The price is one eight-byte field on each `Model`, nothing per entity,
and nothing anybody could measure in time or in peak memory.

### 84.1 The check never disappeared. The question it asked did

"The assertion is compiled out in release" is the natural way to say what was
wrong, and it is wrong. `slot_in` opens with a plain `assert!` and never a
`debug_assert!` (`crates/rcad-topo/src/key.rs:83`), so it was in the shipping
binary the whole time and it ran on every single key lookup. What went away was
not the check but the thing it compared: both sides were values of a type with
one inhabitant, so `self.model == model` was `true` by construction. The
assertion went on faithfully checking a question that could no longer be asked.

Keep the two apart, because they point at different fixes. An assertion that
gets stripped is fixed by making it survive the strip. This one had survived;
what it needed was something to look at.

### 84.2 What the old release build answered, and what §82.10 predicted

Two models, each holding one block as its first solid, so both keys name slot
zero and the wrong answer is a plausible one rather than a crash. Measured in a
release build with the old `cfg`-split type put back:

    mine.solid(cube)    volume = 1
    theirs.solid(block) volume = 1000
    theirs.solid(cube)  volume = 1000   <- another model's key
    theirs.fuse(&[cube], &[block], Union) = Ok, 0 solid(s), volume -0

The third line is what §82.10 said it would be: a key naming a 1 mm³ cube read
1000 mm³ out of the other model, and nothing said so.

**The fourth line is not, and the difference is worth having.** §82.10
predicted a union of two of the model's own solids, "volume 1500 where the
caller asked about a 1 mm³ cube". What the fuse really did was take
`theirs`'s block twice — the caller's cube resolved to it too — fuse it against
itself, and hand back `Ok` with a `Fused` naming **no solids at all**. Not a
wrong volume: no volume. A prediction written down and then measured is worth
more than either on its own, and this is the case in point, because a reader
hunting for a 1500 would have found nothing wrong and stopped.

### 84.3 The arena grows by zero bytes

Nothing a `Model` stores holds a key. Every arena payload names its neighbours
by `*Slot` — `VertexData` through `ShellUse`, `crates/rcad-topo/src/model.rs`
`:1161` to `:1268` — and the derived `Index` at `:1075` is four `SecondaryMap`s
off slots. So widening the id widens nothing the arena holds, and the whole
change to a `Model` is the one `id: ModelId` field on the model itself
(`model.rs:33`), which went from a zero-sized type to a `u64`. Once per model,
nothing per entity: a model with a million faces grows by the same one field as
an empty one.

What widens is a key in flight: `{slot, model}` went from 8 bytes to 16. The
widest transient population is the mesher's key-keyed maps —
`at_vertex: HashMap<VertexKey, u32>` and two `HashMap<EdgeKey, _>` at
`crates/rcad-topo/src/mesh.rs:49`, `:50` and `:398`. On the largest model in
the corpus that reads, that is 2 812 vertices plus 4 530 edges counted twice,
11 872 keys, so about 95 kB more against a peak of 51 MB for the whole import.

**That 95 kB is a floor, not the figure.** It counts the key bytes and nothing
else, and a `std::collections::HashMap` stores `(K, V)` pairs: a
`(VertexKey, u32)` entry goes from 12 bytes to 24, not from 8 to 16, because the
key's alignment goes with its width. Two or three times 95 kB is still nothing
against 51 MB, which is why the floor is enough to argue from and why §84.4
measures rather than adds this up.

### 84.4 The measurement, and the control that makes it evidence

Five release runs a side on `kicad/PinHeader_2x39_P2.54mm_Horizontal.step` —
2 901 763 bytes, 65 967 STEP entities, 20 312 topology entities (2 812
vertices, 4 530 edges, 9 060 coedges, 2 032 wires, 1 876 faces, one shell, one
solid). Medians, and the full range where it matters:

| | before (zero-sized) | after (`u64`) |
|---|---|---|
| parse ms | 55.5 | 56.6 |
| import ms | 64.7 | 63.6 |
| validate ms | 33.5 | 33.7 |
| mesh ms | 24.1 | 23.5 |
| peak RSS kB | 51 400 (51 240–51 624) | 51 552 (51 440–51 596) |

Import and mesh came out nominally *faster* with the wider key, and the whole
after-range for peak memory sits inside the before-range.

**The parse column is the control and it is the argument.** Parsing a STEP file
mints no key at all — `StepFile::read` goes through `graph.rs`, `read.rs`,
`token.rs` and `text.rs`, none of which name `rcad_topo` — so nothing this
change touched can move it. It moved by 1.1 ms anyway, which is exactly what
the largest of the three key-touching columns moved (import, 1.1 ms the other
way) and more than the other two. That is what licenses reading the table as
"the machine, not the change". Without that column these are four numbers that
moved, with no way to tell which movements meant anything, which is noise
presented as evidence.

### 84.5 Why the biggest model that reads is a 2.9 MB pin header

Five corpus files are bigger and all five are NIST parts, and not one of them
ever becomes a `Model`. `how_often_a_real_file_imports` on this base gives the
reasons: `nist_ftc_09_asme1_ap242-e1.stp`, `nist_stc_10_asme1_ap242-e2.stp` and
`nist_ctc_02_asme1_ap203.stp` on `B_SPLINE_CURVE_WITH_KNOTS`,
`nist_stc_09_asme1_ap242-e3.stp` on a corner 9 874 spacings off the curved face
the file puts it on, and `nist_ftc_08_asme1_ap242-e2.stp` on a swept area that
could not settle one decision it depends on.

So the largest thing this kernel can hold out of 367 corpus files is a KiCad
connector housing. That says more about what the reader can read than about
this change, and it is written down because the next person looking for a big
model to measure against will otherwise go hunting through those five.

### 84.6 A `u32` id was considered and lost

It halves the widening. What it asks in exchange is a counter that wraps after
four billion models, and a wrapped id does not fail loudly — it re-aliases, and
the model that draws the recycled number is exactly as invisible to `slot_in`
as every model was before this change. Four bytes that §84.4 could not measure
the absence of even at eight, against bringing the bug back in a rarer form, is
not a trade.

### 84.7 The two tests, and how they were made to fail

Both live in `crates/rcad-topo/tests/what_only_a_release_build_does.rs`, which
is where §83.6 says a release-only test goes, and both are `#[should_panic]` on
`"this solid key was made by a different model"`:
`a_solid_key_from_another_model_names_a_stranger_and_must_not_be_taken` is
§84.2's third line, and `a_fuse_given_another_models_solid_must_not_answer_ok`
is its fourth.

They were shown to fail without the fix by putting the old `cfg`-split type
back and running `just check-release-build`: both reported **"test did not
panic as expected"**, which is the exact shape of the defect — no crash, the
call simply went through.

The third test in the file is what makes those two failures mean what they
look like. `these_tests_are_built_with_the_debug_assertions_off` passed in that
same run, so the two failures were the absent fix and not a build that had
quietly dropped back to the dev profile, where the counter is real and both
tests pass for the wrong reason. That is §83.3's guard doing the one job it
exists for, on the first unit that needed it.

### 84.8 Still open: §4.1 would rather this were an `Err`

**Closed by §86**, on 2026-09-10, built from the inventory below and one item
wider: the three `set_*_approximation` already returned a `Result` and had the
same panic inside them, and `Model::fuse` now asks the question of every
argument and tool key before it does anything else. The rest of this subsection
is what was known when it was still open, kept as written.

A key from another model is **caller input**, and §4.1's table answers bad
caller input with `Err`. The panic that went in is the smaller fix: it turns a
wrong answer into a crash, which is the direction `CLAUDE.md`'s "crash >
corrupt" asks for, and it moves no signature. But §4.1's amendment under §66 is
explicit that a panic is for where the operation has already written into the
caller's model and cannot take it back — and `Model::solid` writes nothing at
all. It is the clean case for the other door, and the other door is not built.

It is a lane and not a paragraph. What it moves:

- the seven view accessors on `Model` — `solid`, `shell`, `face`, `wire`,
  `coedge`, `edge`, `vertex`, `crates/rcad-topo/src/model.rs:150` to `:232` —
  each handing back a `Result` instead of a view;
- `place_solid` at `:280`, which returns nothing today;
- a new variant on `Error`, `crates/rcad-topo/src/error.rs:14`, which is
  `#[non_exhaustive]`, so that part costs callers outside the crate nothing;
- `Fused::images_of_face`, `images_of_edge` and `images_of_vertex`,
  `crates/rcad-topo/src/fuse/history.rs:47` to `:71`;
- every call site of all eleven, in `rcad-topo`, `rcad-step` and `rcad`.

**That is recorded as open, and nothing here decides it.** The inventory is
written down so the next reader prices it instead of re-deriving it.

§66.7's first item is the same shape and is also still open: `Assembly::commit`
wants a scratch model so a fuse can refuse *before* it writes into the caller's
model, and it panics today for want of anywhere to fail earlier. Both are the
same question — where a check with no pre-write door can grow one — and
whoever takes one should look at the other.

**§86 closed this one and left that one open**, and the two turned out not to be
the same question after all. §86.5 says why: this check needs nowhere to fail
earlier, because it reads nothing before it fails.
---

