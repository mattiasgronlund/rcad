## 73. A record that cannot be born certified, recorded 2026-09-08

§69.4 named the change and did not take it: `Record::new` took an
`outcome: Decision`, three of whose four values were always a bug in that
position, and the contract was kept by a runtime assert that sixteen production
call sites had to remember. Two of them forgot, and §69.1 and §69.3 are the
faults that came of it.

This is that change. The constructor now takes `enclosure: f64` and builds the
`Undecidable` itself:

```rust
pub fn new(
    predicate: &'static str,
    inputs: &[Point],
    level: Level,
    enclosure: f64,
    measures: Measures,
) -> Self {
    Self {
        predicate,
        inputs: inputs.to_vec(),
        level,
        outcome: Decision::Undecidable { enclosure },
        enclosure,
        measures,
        approximation: None,
    }
}
```

The `panic!` §69 leaned on is gone, and nothing can reach where it stood: a
caller with a `Decision` in hand has to destructure it to get the `f64`, and the
only arm that yields one is `Undecidable`.

### 73.1 What did *not* change

**The `outcome` field stays.** It is what a policy writes a sign into —
`Record::resolve_to` sets it, and `Record::outcome` says it "starts out
`Decision::Undecidable` and becomes a sign if a policy resolved it". Only the
*parameter* narrowed. The invariant "a record is born undecidable" moved out of
a runtime assert and into the constructor's own body, where no caller is asked
to remember it.

**The mirror assert in `resolve_to` stays too**, and for the opposite reason: it
checks that resolving *produces* a certified sign, which is a claim about the
policy rather than about the caller's destructuring, and no type here makes it
by construction.

### 73.2 The three sites that leaned on an early return

§69.5 grouped the sixteen. The seven that already had an `f64` in hand got
shorter — `Decision::Undecidable { enclosure: slop }` became `slop`. The five in
a `Decision::Undecidable { .. }` match arm changed `..` to `enclosure` and passed
it on; `validate.rs`'s `decision @ Decision::Undecidable { .. }` binding is gone,
because there is nothing left that wants the whole `Decision`.

The interesting three were `orient2d.rs:98`, `orient3d.rs:110` and
`intersect.rs:2713`, all behind

```rust
if decision.is_certified() {
    return decision;
}
```

All three destructure on the spot now, and the early return is the `else`:

```rust
let Decision::Undecidable { enclosure } = decision else {
    return decision;
};
```

No site needed an `unreachable!`, and none needed a made-up number. Every one of
the sixteen had the enclosure available where it stood; what it lacked was a
signature that asked for it.

### 73.3 `holds.rs`, and the certifiedness argued in another function

§69.5 named this hazard and left it standing on purpose: the match arm in
`what_a_curved_face_holds` was correct only because `the_side_this_face_lies_on`
happens to write `found = Some(side)` inside its `Decision::Pos | Decision::Neg`
arm and nowhere else. Before:

```rust
match side {
    Decision::Zero => on_its_boundary = true,
    settled if settled == inwards => (),
    Decision::Pos | Decision::Neg => return Ok(OnAFace::NotHeld),
    other => return Err(gave_up_on_a_side(&trimming.surface, point, other)),
}
```

`other` is a `Decision`, a guard arm above it can fall through, and the
certifiedness of what reached `gave_up_on_a_side` was argued a function away.
After:

```rust
match side {
    Decision::Zero => on_its_boundary = true,
    settled if settled == inwards => (),
    Decision::Pos | Decision::Neg => return Ok(OnAFace::NotHeld),
    Decision::Undecidable { enclosure } => {
        return Err(gave_up_on_a_side(&trimming.surface, point, enclosure));
    }
    other => panic!("a side came back with a decision nothing here knows: {other:?}"),
}
```

The distant argument is now **unnecessary for this line's safety**. Whatever
`the_side_this_face_lies_on` learns to return, `inwards` only affects which of
the arms a `Pos`, `Neg` or `Zero` takes — and none of those three can reach the
refusal, because the arm that reaches it names `Undecidable` and the wildcard
below it is `panic!`. The day that function returns `Some(Undecidable)`, this
loop treats a matching undecided side as "inside" rather than panicking in
`Record::new`; that is a different bug, and a worse-behaved one, so the distant
argument still earns its paragraph in §69.5 — it just no longer holds up a
record.

The narrowing travelled the whole way down: `gave_up`, `gave_up_on_a_side` and
`gave_up_turning_a_corner` all take an `f64` now, which is what forced each of
the five callers to destructure. `whichever_of_them_was_not_read` returns `f64`
instead of `Decision`; its `assert!` became a `let … else { panic! }` on the
same invariant, one arm rather than two, and it no longer has a second copy of
itself waiting inside `Record::new`.

Three wildcard arms became `panic!` where they had been `return Err(gave_up…)`.
That is not a new refusal being invented: those arms are now reachable only by a
`Decision` variant that does not exist yet, which §40 says is a stop rather than
a guess.

### 73.4 The test, and what it does not prove

The old failure was a runtime panic; the new one is a compile error, which no
running test can exercise. So there are two, and they prove different halves.

The compile-time half is a `compile_fail` doctest on `Record::new`, paired with
an identical one that is not `compile_fail`. The pair is the point: same five
lines, same imports, only the fourth argument differs — `1e-30` in the one that
compiles, `Decision::Pos` in the one that must not. A `compile_fail` doctest
passes when the snippet fails to build *for any reason*, including a typo in an
import, so on its own it proves almost nothing. Its twin passing is what rules
that out. Compiled directly, the failing snippet gives:

```
error[E0308]: mismatched types
 --> src/main.rs:4:68
  |
4 |     let record = Record::new("orient3d", &origin, Level::Interval, Decision::Pos, Measures::Volume);
  |                  -----------                                       ^^^^^^^^^^^^^ expected `f64`, found `Decision`
```

The runtime half replaces `a_certified_outcome_may_not_be_recorded`, which had
no panic left to catch. `a_record_is_born_undecidable` asserts that a fresh
record's `outcome()` is `Decision::Undecidable` carrying the enclosure it was
handed. **That test does not prove the compile-time property.** It would pass
just as well against the old signature. It is there because the field is still
a `Decision` and still mutable through `resolve_to`, so "born undecidable" is
worth pinning as behaviour and not only as a type.

Neither test proves the sixteen call sites are right. Nothing is asked to: the
signature is, which is the whole of what §69.4 wanted.

