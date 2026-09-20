## 70. A model that decides how closely it will be checked, recorded 2026-09-07

§67.4 left two questions open and called them a decision rather than a patch.
This is the decision. One of them turned out to have a fix and the other turned
out to be answered by measurement rather than by taste.

### 70.1 The number a file was allowed to write about itself

`Model::allowed_wobble` is how far off its own plane a face's corner may sit,
and it was two things added:

```
Scale::default_linear_tolerance()   +   what_writing_a_plane_down_costs()
= nominal_size × 1e-9                   = 16 × SIGNED_DISTANCE_ROUNDOFF
                                          × largest_coordinate()
```

The second term is measured — `largest_coordinate` walks the model's own
corners. The first was **declared**: `Scale` is a field of the model, so it
arrives with the file, and a file that says how big it is has thereby said how
closely it will be checked. That is a self-certifying input, and the fuzzer
found it.

Measured on §67's corpus entry, `crash-7d309176caedc7140756f167d7db5e7740eff1c3`
bytes `2382..4763`, decoded and swept:

```
nominal_size            1.8446744073709552e21     declared
default_linear_tolerance  1.8446744073709553e12   declared × 1e-9
largest coordinate        1.0359375e1             measured over its corners
faces off their own plane  3.593845367340691e-1, 1e1, 2e1
Model::validate           passes
```

Two hundred light-years of declaration, ten millimetres of block, and twenty
millimetres of wobble walking through a check that thought it had `1.8e12` to
play with.

### 70.2 The fix, and the family of alternatives it beat

**`allowed_wobble`'s first term is now the smaller of what the model says and
what its corners show.** In `crates/rcad-topo/src/validate.rs`:

```rust
let what_the_model_says = self.scale().default_linear_tolerance();
let what_its_corners_show = Scale::RELATIVE_LINEAR_TOLERANCE * self.largest_coordinate();
what_the_model_says.min(what_its_corners_show) + self.what_writing_a_plane_down_costs()
```

§16.10 asks for the family, not a paragraph. Five scalings were on the table.

**(a) Leave it.** The hole is unbounded: `nominal_size` is any positive finite
`f64`, so a stranger buys `1e300` of room for the cost of one field.

**(b) Drop `Scale` and scale on `largest_coordinate` alone.** This closes the
overstating direction and opens the understating one: a model declaring `1` with
one vertex parked at `1e21` would then *earn* `1e12` of wobble on the geometry
beside the origin. Measured, not argued — `crash-f984851…` is a model whose
largest coordinate really is `2.3611832414348226e21`.

**(c) An absolute floor or cap in millimetres.** This is the thing `Scale`
exists to avoid. A number in millimetres cannot be right for a bracket and for a
bridge at once, and §15.6 would want the bound demonstrated for both.

**(d) Refuse the model outright when `nominal_size` and `largest_coordinate`
disagree by more than some ratio.** A second threshold to argue about, and it
refuses models that are merely mis-declared rather than broken. The ratio on
§67's entry is `1.8e20`; on `crash-f984851…` it is `100`, and that second one is
an ordinary overstatement that nothing should refuse.

**(e) The minimum, which is what was taken.** It is the tightest of (a) and (b)
in both directions at once and introduces no new number: `min` is not a
threshold. What it costs a legitimate model is the case where the declaration
honestly exceeds the geometry — a model holding one small fragment of a large
part. That model's tolerance drops to what its own corners earn, and the
direction is the safe one `what_writing_a_point_down_costs` already picked: too
small a budget refuses a sound corner, too large a one accepts a corner nobody
placed.

**So: yes, it may scale with a number the model declares — but never with that
number alone, and never above what the model's own corners show.**

### 70.3 Where the missing comparison goes: `bake`, measured

§67.4's other question: nothing compares a baked corner with the corner the
model stored, and does that check belong in `bake` or in `validate`?

**`bake`.** Two reasons, one structural and one measured.

The structural one is that with §70.2 in place `validate` *already makes* the
comparison for every well-conditioned corner: a corner within `allowed_wobble`
of all three planes through it is within that of their meet. What survives is
the ill-conditioned case — three surfaces at a grazing angle, where a corner
close to all three can still be far from where they cross. That is a question
about a *re-derivation*, and only the thing that re-derives has both points to
compare. `validate` does not re-derive corners and has no reason to start.

The measured one is what re-deriving costs. On this machine, `Policy::certified`:

```
Plane::signed_distance_to                            3 ns
meet_of_three_surfaces, three perpendicular planes  571 ns
Model::validate over one block                    13.519 µs
Model::fuse of two blocks                          1.353 ms
```

The distance is what the sweep pays per corner per face today and the meet is
what a re-derivation would pay per corner per triple — 190 times dearer. A box's
eight corners are eight triples, `4.6 µs`, which is a **34% increase on the whole
sweep** of that box. And `derive_a_corner` tries *every* triple, so the cost is
cubic in the faces round a corner where the distance check is linear: a corner
where six faces meet is twenty meets, `11.4 µs`, most of one sweep for one
corner. §4.3 says the full sweep runs at every import and test boundary; this is
the shape of cost it was written about.

The comparison itself is one line where it belongs. `place_a_corner` already
holds `model.data().vertices[slot].point` (`bake.rs:500`) and `derive_a_corner`
returns the rebuilt point beside it. **This is not done here** — it is a fuse
change with its own test to write, and the lane that did §70.2 ran out of budget
before it. It is owed, and it is owed to `bake.rs`.

While reading for that: `derive_a_corner`'s doc comment says "the stored corner
is what says which: it is a cached sample of the truth (§17.1), so the box it
falls inside is the one. Two boxes that both hold it are two answers to one
question and the triple is passed over." The code consults no stored corner at
all — it takes `let [only] = places.as_slice() else { continue; }`, passing over
*every* multi-place triple regardless of where the stored corner sits. The
sentence describes a filter that is not there, which is the §16.10 shape again.

### 70.4 What this refuses now, and what it still does not

Both of this session's crash reproducers were replayed against the fix with
`cargo +nightly fuzz run two_solids_fused_by_a_stranger <path>`. Both run clean,
as they did before — but one of them now stops a stage earlier:

`crash-7d309176caedc7140756f167d7db5e7740eff1c3` is **refused at the sweep**,
which is the gate the fuzz target runs before it fuses at all:
`Invalid(VertexOffItsFacePlane { distance: 9.536734069115482e-6 })`, against a
budget that fell from `1.8446744073709553e12` to `1.0359375e-8`. The fuse it
crashed is never reached.

`crash-f984851ca89020e1ff939c087667a0aa86bc352d` is **unchanged**. It declares
`2.3611832414348226e23` with a largest coordinate of `2.3611832414348226e21`, so
the cap tightens its budget from `2.3611832414348228e14` to
`2.3611832414348228e12` — and its faces sit at most `4.021529366771898e-87` off
their own planes, so it passes as it should. §69's refusal is what that one was
about and this does not touch it.

**What is still open.** `largest_coordinate` is model-wide, so a model holding
one solid with a corner at `1e21` still buys `1e12` of wobble for a second,
small solid beside the origin. Narrowing it per face or per solid was
considered and not taken: a part's tolerance is a property of the part, and a
face near the origin of a large part is entitled to the large part's tolerance.
Closing that needs the §70.3 comparison, not a smaller radius.

### 70.5 Two tests it moves, and why they were not adjusted

`crates/rcad-topo/tests/a_piece_of_a_face_bounded_by_two_straight_sides.rs` —
§67's own test file — carries §67's bytes and asserts, at line 88, that *"the
sweep lets these blocks through, which is how they reach the fuse"*. §70.2 makes
that false, which is the point of §70.2. Both tests in that file fail on the
premise rather than on the assertion:

```
a_wall_wearing_its_neighbours_plane_is_refused_and_does_not_crash_the_fuse   FAIL
a_certified_policy_gives_up_before_it_gets_that_far                          FAIL
  the sweep lets these blocks through, which is how they reach the fuse:
  Invalid(VertexOffItsFacePlane { distance: 9.536734069115482e-6 })
```

They were left failing rather than edited. What §67 tested is still worth
testing — a two-sided piece is a refusal and not an assert — and the bytes are
no longer a way to reach it, so somebody has to decide whether that file gets a
model that reaches the walk another way or whether the assert's own test is
enough. That is §67's lane's call, not this one's.

---

