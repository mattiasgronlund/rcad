## 69. A refusal that named the wrong decision, recorded 2026-09-07

A soak of `two_solids_fused_by_a_stranger` crashed `Model::fuse` in
`Record::new`:

```
thread '<unnamed>' panicked at crates/rcad-predicates/src/provenance.rs:190:17:
orient3d: a certified outcome does not get a provenance record
```

That assert enforces §5 — *detailed records only on non-certified outcomes* —
so a certified `orient3d` arriving with a record is a contradiction, and exactly
one of the two sides of it is wrong. **The caller is wrong.** The assert stands
untouched and §5 stands untouched; what is recorded here is the evidence that
settled it, so that the next person to meet this crash does not widen the assert
to make it go away.

### 69.1 What actually reached the assert

The walk in `crates/rcad-topo/src/holds.rs` holds *two* decisions at the point
it refuses and reports only one of them:

```rust
match crossing(sides[index], sides[next], along) {
    Some(count) => winding += count,
    None => return Err(gave_up_turning_a_corner(point, sides[index])),   // line 322
}
```

`crossing` returns `None` only through `above_the_line(from_side)?` or
`above_the_line(to_side)?`, and `above_the_line` returns `None` only for
`Decision::Undecidable`. So reaching that arm proves *one of the two sides* was
undecidable — and the arm hands over `sides[index]` whichever one it was.

The reproducer takes the other branch. A probe printed at the refusal, on the
crashing input:

```
index=1 next=2
sides[index] = Neg
sides[next]  = Undecidable { enclosure: 3.6537540933272573e47 }
along        = Pos
from  = Point([0.0, 0.0, 0.0])
to    = Point([0.0, 4.243991582e-314, 2.3611832414348226e21])
point = Exact([-0.0, 1.770887431076117e21, 5.902958103587057e20])
```

and a second probe in `Record::new` itself printed what it was then asked to
record:

```
predicate=orient3d level=Exact outcome=Neg
inputs=[Exact([-0.0, 1.770887431076117e21, 5.902958103587057e20])]
measures=Volume
```

`outcome=Neg` is the certified answer about corner 1. The refusal is about
corner 2. Both probes were temporary and neither is in the tree.

That input is §3.5's spread and nothing more exotic: coordinates around `1e21`
sitting beside a `4.2e-314` subnormal, so the interval on the determinant is
`3.7e47` wide and no scaling brings both ends into range at once. The
undecidable side is honest. Only the record made about it is not.

### 69.2 Why this is the caller's bug and not the assert's

The tempting reading is the other one: the *walk* gave up while this particular
predicate was certified, so perhaps a certified outcome may legitimately carry a
record after all. It may not, and the reason is arithmetic rather than taste.

A `Record` has an `enclosure`, and `Record::new` takes it out of the `Decision`
because that is the only place it exists. A certified `Neg` has no enclosure —
there is no interval that straddled zero, because nothing straddled zero. A
record built from one would have to invent the field, and `worst_wobble` would
then be a maximum over a number nobody measured. §15.3 is the same mistake one
level down: an enclosure whose units nobody can recover is not a smaller record,
it is a wrong one.

The record the walk actually wants already exists and is one line away:
`sides[next]` carries `Undecidable { enclosure: 3.65e47 }`, which is exactly the
number a reader of the refusal needs. Naming the certified neighbour throws away
a measurement that was in hand.

So this is a **gap and not a limit** in §16.10's terms — the answer exists, and
the code reached past it.

### 69.3 The other one in the same function, and it is reachable

Twelve lines up:

```rust
Decision::Zero => match lies_between(from, corners[next], point) {
    Some(true) => return Ok(None),
    Some(false) => (),
    None => return Err(gave_up_turning_a_corner(point, along)),          // line 315
},
```

`along` is bound to `Decision::Zero` by the arm it is in, so this call **cannot**
be given anything but a certified outcome: reaching it panics every time.

**It is reachable, and the demonstration is a test rather than a paragraph**
(§16.10). `SolidView::classify_uncertain_point` is public and takes a
`rcad_predicates::Point`, whose `Constructed([Interval; 3])` variant is public
and whose `Interval::new` is public — so *any* box is a caller's to hand over,
and no fuse, no fuzzer and no spread is needed to get one here.
`a_refusal_no_predicate_made_must_not_be_recorded_as_one` is that box: it lies
along the outline's first side with its `y` and `z` exact, so every `orient3d`
about it certifies and `along` is exactly `Zero`, and it runs a millimetre past
the corner at the far end, so the ordering cannot be settled. It panicked on
`314b453` under `Policy::certified()`.

The first note here said the reachability "was not demonstrated and is not
claimed either way", which was the right thing to write and the wrong thing to
leave. §16.10 asks the question of the **public API** rather than of today's
callers, and the public API answers it in one line.

**The fix is a different constructor, not a different argument.** Nothing was
undecided by a predicate: what could not be settled is the *ordering* of a box
against the ends of a side, which §17.4 answers by comparing one axis and asks
no predicate at all. So `gave_up_ordering_along_a_side` records the ordering
under its own name, at `Level::Interval`, with the overhang as its enclosure and
`Measures::Length` as its units — and `lies_between` now hands that overhang back
(`Result<bool, f64>`) instead of throwing away the only measurement of the
refusal there is. Filing it as an `orient3d` of `Measures::Volume` would have put
a width in units nothing on this path worked in, which is §15.3's mistake with
the degree wrong instead of the level.

### 69.4 Why the signature admits the mistake

`Record::new` takes an `outcome: Decision`, and three of that type's four values
are always a bug there. The type is wider than the contract, and the contract is
kept by a runtime assert that every call site has to remember — sixteen of them
outside tests, across three crates, on this tip.

Narrowing it — `Record::new(…, enclosure: f64, …)`, with the caller made to
destructure the `Undecidable` it claims to have — would move both faults above
to compile time, because neither call site can produce an `f64` it does not have.
That is the shape of the fix worth taking, and it is deliberately **not** taken
here: it changes every call site in `rcad-topo` and `rcad-geom`, which is a unit
of work rather than a bug fix, and §13's rule is that it waits for the consumer
that names it.

**The consumer has now arrived** — it is the `holds.rs` fix in §69.1 and §69.3,
and it is the reason this paragraph is an argument for the change rather than a
note that nothing needs it. It still did not happen in this lane, because the
call sites it touches run through files another lane held open at the time. It is
its own unit, and this section is its case.

### 69.5 The other fifteen, and the one held up by a distant invariant

Every other production call site was read, because "one caller got it wrong" is
worth nothing without knowing whether the rest did too. The fifteen away from
`holds.rs` are all safe, and mostly for the dull reason: seven build the
`Undecidable` themselves out of an `f64` and never take a `Decision` at all,
five sit inside a `Decision::Undecidable { .. }` match arm — `validate.rs:272`
binds it as `decision @ Decision::Undecidable { .. }`, which is what makes it
proof against the two guard arms above it — and three (`orient2d.rs:98`,
`orient3d.rs:110`, `intersect.rs:2713`) sit behind an
`if decision.is_certified() { return … }`.

The sixteenth site is `holds.rs:639`, the one inside `gave_up`, and five callers
reach it. Lines 315 and 322 are §69.1 and §69.3. Lines 277 and 545 destructure
`Undecidable` on the spot and are fine. Line 372 is fine too, and is worth
naming because it is *correct for a reason that is not local to it*:

```rust
match side {
    Decision::Zero => on_its_boundary = true,
    settled if settled == inwards => (),
    Decision::Pos | Decision::Neg => return Ok(OnAFace::NotHeld),
    other => return Err(gave_up_on_a_side(&trimming.surface, point, other)),
}
```

A guard arm can fall through, so `Pos | Neg` alone would not prove `other` is
undecidable — it does, but only because `inwards` came from
`the_side_this_face_lies_on`, which writes `found = Some(side)` inside its
`Decision::Pos | Decision::Neg` arm and nowhere else. The day that function
learns to return `Some(Undecidable)`, this line starts panicking, and nothing in
the file says so. Same shape as line 322: a `Decision` whose certifiedness is
argued at a distance rather than destructured on the spot. §69.4's narrowing
would settle this one too.

**No guard was added there**, and that is the decision rather than an omission.
There is no bug at line 372 today; a guard would defend against one that does
not exist and would put a second, quieter copy of the invariant beside the real
one. What the hazard needs is to be *written down*, which is this paragraph, and
to be settled by the type — which is §69.4. The name to look it up by: **a
certifiedness argued in another function**.

### 69.6 What it took to reproduce this without a fuzzer

The fuzzer needed a model whose coordinates ran from `4.2e-314` to `2.4e21`, and
neither test here needs a spread at all. Two things stood between a hand-built
model and the walk, and both are worth knowing before writing the next test that
has to reach it.

**A box on a spoke grazes both of its neighbours.** The walk is only asked about
a point already on the face's plane, and the way a polygon says so is one of the
triangles fanned from its first corner reporting that the ray starts on it. Every
fan triangle beside a spoke has that spoke for an edge, so a box sitting on a
spoke makes both of them refuse and neither can report anything — the walk is
never entered and the crash never happens. What fixes it is a **corner that
points back into the outline**: the fan then overlaps itself, and a spoke runs
through the middle of a triangle that is not beside it. That triangle answers.
`OUTLINE`'s `[6, 0]` is that corner, and it does the same work in both tests.

**A certified policy cannot get past the first corner.** The spoke from the first
corner to itself is a determinant with two equal rows — exactly zero, and interval
arithmetic cannot see that it is zero, because it does not know the two rows are
the same. So a *constructed* point refuses there and the walk stops having named
the undecided corner correctly, which is not the bug. The fuzzer's point was
`Exact`, which is why it got past. A hand-built box gets past instead by being
resolved: under `Policy::tolerant` a narrow enough enclosure reads as `Zero`.

That turns the test into an arithmetic problem with a real window. The budget is
`64 · ε · working_size`, which is `1.42e-12` at 100 mm, and a volume's enclosure
converts to a wobble as `enclosure / working_size²` (§15.3) — so the first
corner's enclosure has to come in at or under `1.42e-8` and the spoke's has to
come in over it. Both are linear in the width of the box, so shrinking the box
moves them together and separates nothing. What separates them is **moving one
corner and not the other**, because a spoke's enclosure grows with how far out
its corner sits: `[160, -80]` is ten times out, and the two enclosures measure

```
first corner  9.600000794307562e-9     resolves    (wobble 9.6e-13 ≤ 1.42e-12)
spoke         2.560002485552104e-7     does not    (wobble 2.56e-11)
```

a factor of about 27, with the budget between them. In plain terms: a fifth of a
nanometre of doubt about a point on a 160 mm part — an ordinary amount for a
constructed point to carry — and one corner of the outline far enough away that
the same doubt is twenty-seven times too much when asked about *that* corner.

Both numbers were measured. The first was also predicted, by scaling an earlier
probe, and the prediction was wrong by a factor of four — which is why the
measured pair is what is written here.

