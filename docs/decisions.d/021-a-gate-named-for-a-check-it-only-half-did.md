## 21. A gate named for a check it only half did

*Recorded 2026-08-31.*

`product_is_exact(a, b)` answers whether [`two_product`] on `a` and `b` is
exact. It was `true` for `1e200` squared.

Its doc was accurate — it described the subnormal precondition and nothing else,
and that is genuinely what it tested. The **name** is the problem. At
twenty-seven call sites it reads as *the* gate that keeps unrepresentable
products out of the expansion arithmetic, and it only ever guarded the bottom of
the range. It now rejects overflow too.

### 21.1 Nothing downstream changes, and that was checked rather than hoped

The prediction going in was that no outcome moves, because an overflowed product
reaches `expansion_sign`, which refuses to name a sign for a non-finite
expansion. A prediction is not a measurement, so each site was checked:

Every guarded call site sits in an `exactly()` — level 3 of the cascade — and
every one of them maps `Decision::Undecidable` to `None`: `orient2d`,
`orient3d`, `plane`, `sphere`, `cylinder`, `surface`, and
`validate.rs`'s `AreaSweptRoundTheNormal`. The cascade turns that `None` into
`Undecidable { enclosure: straddle }` at `Level::Interval`. So both routes to
the refusal — the gate declining early, and the arithmetic running and the sign
refusing — converge on the same `None` and produce the same `Decision`.

**The cascade would have caught it if they did not.** `decide` treats
`Some(Decision::Undecidable { .. })` from level 3 as `unreachable!`, because
exact arithmetic must certify a sign or say it cannot run. A site that mapped an
overflow to `Undecidable` instead of `None` would panic, not drift.

`de_boor`'s `quotient_is_exact` is the one gate that is not an `exactly()`. It
returns `false` on a failed gate, and with an overflowed product it returned
`false` anyway, because `two_product` hands back an infinity and the `high ==
top` comparison against a finite `top` fails. Same answer, one step sooner.

740 tests pass, including §17.7's refusal-rate bound, which is the oracle that
would notice a rate moving the wrong way.

### 21.2 The zero test is on the arguments, and has to be

The obvious tightening — read the zero off the product rather than off the
arguments — is wrong, and quietly. `1e-200` squared underflows to zero with
neither argument zero, and calling that exact would open the **subnormal** end of
the gate while closing the overflow end. That end is the one `robust` gets wrong
and returns a confident wrong sign for. The two ends are separate tests and stay
separate.

### 21.3 Two claims carried into this work that did not reproduce

Both came from §18.6 and were checked at `898ac58` before being acted on.

1. **"Two call sites bypass the gate entirely and lean on `expansion_sign`
   alone."** They do not. Every use of `two_product` or `scale_expansion`
   outside `expansion.rs` itself is preceded by a `product_is_exact` in the same
   function — `validate.rs`'s path, named as one of the two, gates both its
   minor and its scaling. Nothing was left to add, so the "legibility, not
   correctness" change this predicted has no subject.
2. **"The zero-elimination invariant is documented on `two_diff` and needs tests
   at the two functions that provide it."** It has them.
   `a_non_finite_entry_cannot_be_dropped_on_its_way_to_a_sign` already runs an
   overflowed pair through `expansion_sum` *and* through `scale_expansion`, and
   through the scale-by-zero case specifically — which is the subtle one, since
   a short-circuit on a zero factor is what would break the backstop. That work
   was already done.

Recorded because a plan is a claim like any other, and §16.10 does not exempt
the ones written a day earlier.
---

