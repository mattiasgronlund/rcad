## 4. Errors and invariants

### 4.1 Three tiers

The roadmap says "never panic, return `EvalError`" (§4.5). `CLAUDE.md` says
"crash > corrupt" and "assert invariants". One rule:

| Situation | Response | Why |
|---|---|---|
| Bad caller input — parameter outside the domain, zero weight, empty knot vector | `Err` | It is data, not a bug |
| Broken internal invariant — edge with three coedges, key resolving to a freed slot, approximation grew | **panic** | It is a bug; emitting a corrupt solid is worse than dying |
| Cannot decide | `Decision::Undecidable` | It is a result, not a failure |

**The middle row is amended by §66, and the amendment is rewritten by §87.** The
row holds where the operation has already written into the caller's model and
cannot take it back. Where the check can be made *before* the write, the
response is an `Err` that says "this kernel has a bug" —
`Error::TheFuseWroteABrokenAnswer`, a third door and not `Error::Undecidable`.

**Which door it is does not depend on the build.** §66's wording said "in a
release build the response is a panic only where…", and that was written when
the check did not run in a release build at all (§66.1). It does now (§82.1),
and §87 makes the fuse answer `Err` in both builds rather than panicking in one
and refusing in the other. A kernel that reports its own bug differently
depending on how the caller happened to compile it is the shape §82.1 was
written to close, and a `cfg!(debug_assertions)` in the answer would reopen it.

Until §82 there was a worse misreading available and it was the one that
mattered: that the row described what a release build did. It did neither of the
two, because the check did not run at all (§66.1). That is closed — the four
touched checks and the fuse's post-write check now run in every build (§82.1),
and `mise exec -- just t -p rcad-topo --cargo-profile release -E
'test(a_mutation_notices)'` runs its two `#[should_panic]` tests with debug
assertions off and passes both, which is the closure demonstrated rather than
argued. The row now describes something that happens in release. ~~Which of the
two doors it opens there is a separate question and only one of them is built:
the panic is what the fuse and `place_solid` do today, and the pre-write `Err`
waits on the scratch model §66.7's first item is about.~~

**Both doors are built now (§87).** The fuse takes the `Err` one: it works its
answer out in a throwaway model and refuses before the caller's arena is
touched. `place_solid` keeps the panic, because it overwrites existing entities
in place and has nothing to work out in a throwaway — §66.7's second item, still
open. The fuse itself still panics for the two things that fire *after* its real
write (§87.4), so the line this row draws now runs through one function twice
over rather than between two functions.

### 4.2 Error types

`thiserror` enums per crate. No `anyhow` in library code. A `#[non_exhaustive]
rcad::Error` at the facade wraps the crate errors.

`Undecidable` is a variant at the *operation* level (`Error::Undecidable { records }`,
carrying the §5 records that caused it) even though it is a `Decision` variant at
the *predicate* level. A predicate that cannot decide is a result; an operation
that cannot decide is a failure.

### 4.3 Validation is local by default

The roadmap §5.4 says assert invariants "after every operation, not at the end".
Taken literally with a full-model sweep, a test performing *n* operations becomes
O(n²) and the suite stops being runnable long before M4.

- **After every mutation, in debug**: cheap checks on *touched entities only* —
  keys resolve, coedge count, orientation agrees, approximation did not grow.
  **Amended by §66: "in debug" is withdrawn and they run always.** The O(n²)
  above is the full sweep's and was never measured for these; measured, they cost
  50–65 ns per face of the solid they are handed and the full sweep costs 3.4–4.4
  µs per face of the whole model (§66.2).
- **At test and import boundaries**: full `Model::validate()`, public API, called
  by `proptest` and by anyone importing untrusted data.
- **Always, including release**: the approximation-growth check. One comparison,
  and §11.2's whole argument is that catching inflation at the source turns a
  bisection session into a line number.

---

