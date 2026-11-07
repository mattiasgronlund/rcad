## 487. No hash map the kernel can walk — 2026-11-07

*Written by a session comparing rcad with solidcore, not from a queue row.
Mattias had not asked for the row it files; the session took the row as a
recommendation under `CLAUDE.md`'s "Decide it yourself", filed it, and
dispatched nothing. §17.4 is not amended: its rule stands, and this section is
how it gets checked.*

**In short.** §17.4 says "No `HashMap` or `HashSet` anywhere in the fuse's
working state", and `fuse/mod.rs` says it again, yet the fuse holds four std
`HashMap`s. None of them is walked, so no hash order reaches an answer today,
and nothing stops the next one. Settled: clippy's `disallowed-types` bans
`std::collections::HashMap` and `HashSet` in the kernel crates, a map that is
only looked up goes in a wrapper, `LookupMap`, that has no way to walk it, and
a map that is walked becomes a `BTreeMap` or `BTreeSet`. Clippy's
`disallowed-methods` was measured and rejected: it caught 6 of 14 ways to walk
a map.

### 487.1 Where the code stands

The four fuse maps, at `4471473b`:

| where | what it does with the map |
|---|---|
| `fuse/cuts.rs:391` `seen` | `entry(bits).or_insert(next)` with `next` the map's length, so the id is the order the bits first came in, not hash order |
| `fuse/cuts.rs:392` `answers` | `get`, then `insert` |
| `fuse/cuts.rs:729` `faces_along` | `entry(..).or_default().push(..)`, then `faces_along[key]` |
| `fuse/arrange.rs:836` `baked_corner_at` | built by `the_baked_corners_by_vertex`, read by key |

Outside the fuse, nine more kernel files hold one: `rcad-mesh`'s
`triangulate.rs`; `rcad-step`'s `brep.rs`, `graph.rs`, `read.rs` and
`write.rs`; `rcad-topo`'s `measure.rs`, `mesh.rs`, `overlap.rs`, `validate.rs`
and `view.rs`. A script listed every name declared there as a std hash
container and every line that walks one by that name — `iter`, `keys`,
`values`, `into_iter`, `drain`, `retain`, the set operations, `for … in`,
`extend`, `{:?}`. It found six walks, and none lets the order out:

| where | why the order cannot show |
|---|---|
| `measure.rs:2283` `for (face, cone) in cones` | each turn writes its own face's slot and reads nothing another turn wrote |
| `mesh.rs:5156` `on_the_same_points.values()` | the groups share no triangle, and the pairing inside a group follows that group's own `Vec` |
| `mesh.rs:1478` `walked_finer.extend(halved)` | a map extended from a map |
| `validate.rs:4752` `balance.values().any(..)` | `any` over integers |
| `triangulate.rs:3675` `for … in &used` | inside asserts |
| `mesh.rs:13275` | a unit test |

Matching on names is not a proof. A map passed to a function under another
name, or returned from one, is missed. Sorting every use is the row's work, not
this section's. The one measured case of hash order in the kernel so far is
§306.8's: `rcad-step`'s reader's `HashMap` moved coverage from process to
process, but the model it built was the same in all ten.

### 487.2 What was measured

A throwaway crate held 14 functions, each walking a std map or set one way,
three walks that cannot leak, and two controls with no hash container in them.
It ran under clippy on the pinned 1.96.0 with three configs, on 2026-11-07:

| config | caught | missed | false alarms |
|---|---|---|---|
| `disallowed-methods` on the walking methods of `HashMap` and `HashSet` | 6: `iter`, `keys`, `values`, `drain`, `retain`, `HashSet::union` | 8: `for … in &map`, `for … in map`, `for … in &set`, `.into_iter()`, `IntoIterator::into_iter(&map)`, `vec.extend(map)`, `Vec::from_iter(map)`, `format!("{map:?}")` | `values().any(..)` |
| the same, plus `std::iter::IntoIterator::into_iter` | 8: the two `into_iter` calls as well | 6: the three `for` loops, `extend`, `from_iter`, `{:?}` | `Vec::into_iter` (the control), and collecting a map into a `BTreeMap` |
| `disallowed-types` on `HashMap` and `HashSet` | every mention of either type in the 14 | none | the three harmless walks, which name the type too |

The methods config misses because a `for` loop, `extend`, `from_iter` and
`{:?}` reach the map through a trait — `IntoIterator`, `Debug` — and the lint
can only name the trait's method, which every type shares. Naming that method
flags every `Vec`, and still does not see a `for` loop.

`disallowed-types` reported nothing inside a module that says
`#![allow(clippy::disallowed_types)]`, where the wrapper lives, and nothing in
the four fuse uses rewritten on the wrapper.

The wrapper costs nothing that was measured:

- it is the size and alignment of `HashMap<u64, u32>`, in `Option` too, by a
  test;
- in a release build's assembly, LLVM wrote
  `look_up_in_a_lookup_map = look_up_in_a_hash_map` and
  `count_in_a_lookup_map = count_in_a_hash_map`: one body each, shared;
- walking it does not compile: `for (k, v) in &m` is E0277, `m.keys()` E0599,
  `m.0` E0616, `m.into_iter()` E0599.

And the order really does move: 20 maps of the same 64 keys, built the same way
in one process, walked in 20 different orders. So a test that runs the same
operation twice in one process is enough to catch a leak.

### 487.3 Settled

1. A root `clippy.toml` with `disallowed-types` for `std::collections::HashMap`
   and `std::collections::HashSet`, each with a reason naming the two ways out.
2. `LookupMap` in `rcad-geom`, the lowest crate `rcad-mesh`, `rcad-topo` and
   `rcad-step` all depend on besides `rcad-predicates`, which holds the
   predicates and nothing else. It is `#[repr(transparent)]` over `HashMap`,
   with `new`, `with_capacity`, `len`, `is_empty`, `get`, `get_mut`,
   `contains_key`, `insert`, `remove`, `entry`, `Index`, `FromIterator`,
   `Extend`, `Default` and `Clone`. Left out on purpose:
   - `Deref` to the map, which hands back every walking method;
   - `IntoIterator`, `iter`, `keys`, `values` and `drain`;
   - `retain`, whose closure sees the entries in hash order;
   - `Serialize`, because serde walks the map. A struct that holds one then
     cannot derive it, which is wanted;
   - a derived `Debug`, which prints the entries in hash order. It prints the
     count.

   A lookup-only set is built when a lookup-only `HashSet` turns up, not
   before.
3. Every use goes one of three ways: `LookupMap` if it is only looked up,
   `BTreeMap` or `BTreeSet` if it is walked, or an
   `#[allow(clippy::disallowed_types)]` with its reason beside it.
4. `the_same_fuse_twice_over_writes_the_same_bytes_twice`
   (`fusing_solids.rs:246`) fuses two cubes, which reaches none of the curved
   code. It gains a curved pair. Meshing already has its own,
   `meshing_the_same_solid_twice_gives_the_same_mesh`.

### 487.4 Rejected

- **`disallowed-methods`**, by 487.2's table.
- **A lint of our own** (dylint) that names `HashMap`'s own `IntoIterator`. It
  would see every walk, `for` loops included. It is a new tool for `mise` to
  pin and a lint crate to keep building against every clippy release, and the
  type ban plus the wrapper covers what it would: code that cannot name the
  type cannot walk it, and the wrapper cannot be walked.
- **A fixed-seed hasher**, std's `BuildHasherDefault<DefaultHasher>`. A walk
  would then repeat from run to run, but in an order set by an algorithm that
  std's documentation says is not to be relied on across releases, so a
  toolchain bump could change every written byte. And the order would still be
  one nobody chose.

What the type ban does not see: a hash map that comes in from another crate
without our code naming its type, and a hash map from a crate other than std
(`hashbrown`, `indexmap`). The row's lane checks whether any kernel dependency
hands one back, and adds any such type to the list if one does.

### 487.5 What the row leaves to the lane

**Which code the ban reaches.** `just lint` is
`cargo clippy --workspace --all-targets -- -D warnings`, so a root
`clippy.toml` reaches the tests and examples too, and 34 test and example
files use a std hash container. It also reaches the two fuzz-tool crates,
`rcad-fuzz-explain` and `rcad-fuzz-timing`, which use `HashMap` and clocks
freely. Three ways to scope it:

- **(a)** `#![allow(clippy::disallowed_types)]` at the top of each of the 34
  files, and of each fuzz-tool crate. That puts 34 files in the write set, many
  of them busy (`fusing_solids.rs`, `meshing_a_curved_solid.rs`).
- **(b)** `disallowed_types = "allow"` in `[workspace.lints.clippy]`, and
  `#![deny(clippy::disallowed_types)]` in the `lib.rs` of the six kernel
  crates: `rcad`, `rcad-predicates`, `rcad-geom`, `rcad-mesh`, `rcad-topo` and
  `rcad-step`. An integration test or an example is a crate of its own and gets
  only the workspace setting. A lib's own `#[cfg(test)]` module is inside the
  lib and is held to it. The fuzz-tool crates are left alone with nothing
  added.
- **(c)** Two clippy runs in `lint`: `--lib --bins` as now, and the test
  targets with `-A clippy::disallowed_types`. Different flags make cargo check
  every lib a second time.

Recommended: **(b)**, for the smallest write set. Unmeasured: the lane shows
that the crate-level `deny` wins over the workspace `allow`, with a std
`HashMap` put into a kernel file that `just lint` then fails on.

**What fails without the change.** Nothing leaks today, so no test fails at the
base. What fails is `just lint` on a std `HashMap` put back into a kernel file,
and the build on a walk of a `LookupMap`.
