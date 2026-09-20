## 88. One number, one list of entries, recorded 2026-09-11

The overnight fuzzer found a pair of pipes that named two different crossings
depending on which was written first: `a` then `b` gave one curve, `b` then `a`
gave `Err(Undecidable)`. Both orders computed the *identical* enclosure — it
straddles zero, so the interval level decided nothing either way — and the whole
difference was at the exact level.

The quantity that broke is `W₁W₂`, the product of the two axes' squared lengths.
It is symmetric in every factor, and it came back as **eight entries reaching
239 binary orders down one way and nine entries reaching 284 down the other**.
The same number. Only the shorter one could be multiplied by what came next,
because `Exact::times` refuses as soon as one entry-by-entry product would land
in the subnormals — and that is a question about *entries*, not about values.

By §16.10 that is a **gap and not a limit**: the answer exists, the other order
produced it, and one order's arithmetic could not reach it.

So an `Exact` now settles its entries, and the entries are a function of the
number rather than of the route that reached it. `a·b` and `b·a` are then the
same list, and the refusal — which reads the list — is the same refusal.

### 88.1 Why an exact level still needed a normal form

This is worth stating plainly because it is easy to read as a contradiction.
Level 3 was never wrong. Every entry it produced was exact, both orders stood for
the same number, and any sign either of them managed to name was correct.

What was not a function of the value was **how far the arithmetic could go
next**. `product_is_exact` is the guard that keeps level 1's relative error bound
honest (§30.4), and it asks about the entries in front of it. A value written in
more entries than it needs reaches further down the range, so it refuses products
that the same value written shortly would have taken. Exactness and reach are
different properties, and only the first was ever a function of the number.

That is the general shape: *a level that always gives the right answer can still
give it only sometimes, and "sometimes" was decided by argument order.*

### 88.2 Sweeping once is not a normal form, which cost a detour

The obvious implementation is Shewchuk's `COMPRESS` — one sweep down the
expansion and back up, leaving each entry the correctly rounded total of itself
and everything below. It was written, it fixed the pipes, and the doc comment
claimed it was canonical.

It is not, and the tree said so within the hour. `M`, the leaning-pipes question
`r₁²W₂K + r₂²W₁K − T²W₁W₂`, comes out of one sweep as ten entries reaching
`2^-483`; sweeping *that* gives ten entries reaching `2^-504`, which is then
stable. So one sweep is a step, and `Exact::canonical` runs it to a fixed point.

The way this was caught is worth keeping: the round-trip property. Sliding an
expansion down the range by a power of two and back up is two exact scalings and
must land on the entries it started from. It did not — 485 orders wide became
505 — and that is what said the form was not settled, rather than any reading of
the algorithm.

`sweeping_twice_can_differ_from_sweeping_once` pins it on the smallest instance a
search turned up: six numbers whose expansion is four entries, which one sweep
takes to three with the smallest of them wrong, and a second sweep settles.

### 88.3 What is demonstrated and what is only searched

The claim the fix rests on is that a settled expansion is unique to its value.
Nobody here has written a proof of it, and §16.10's rule is that a claim of this
shape needs a demonstration rather than a paragraph. So it is searched, not
argued: `two_routes_to_one_number_end_at_the_same_entries` builds twenty thousand
pairs of pseudo-random expansions, multiplies each pair both ways round, and
requires the two to settle on the same list. It also counts how often the two
orders differ *before* settling and fails if that count is zero, because a search
where the two routes always agreed anyway would prove nothing.

The exactness of a sweep is checked separately and against arithmetic that knows
nothing about it: `compressing_does_not_change_the_number` judges the result with
`expansion_sum` alone, so a bug in one cannot hide inside the other.

### 88.4 Termination, and the one input that has no settled form

`Exact::canonical` sweeps until the entries stop changing, and panics if that
does not happen within one pass per entry. It is a broken internal invariant if
it ever fires (§4.1's middle row): a sweep cannot lengthen an expansion, so one
that never settles means the primitive is not doing what it says.

It fired immediately, on `[NaN, inf]`.

That is not a broken invariant, it is an overflow upstream, and this module has a
standing rule about those: every operation keeps a non-finite entry rather than
dropping it, so that `expansion_sign` is the single place that refuses to name a
sign for one. A `NaN` is never equal to itself, so the fixed-point loop could
never terminate on one. `canonical` now hands a non-finite expansion straight on
untouched, which is the same rule the rest of the module already followed.

The assertion earned its place on its first run, which is the argument for
writing it rather than assuming convergence.

### 88.5 What moved, and what did not

The refusal rate moved **down**, which §16.10 asks be reported rather than passed
over in silence: the crashing corpus seed — `crash-3f9c76a3…`, in
`fuzz/corpus/where_two_surfaces_cross/` and landed by the fix rather than ahead
of it — is decided now, in both orders, and no test anywhere refuses something it
used to decide.

§85.5 says why the file is named here rather than left to
`git log --diff-filter=A`: that section landed a seed against this same target a
day earlier, and on 2026-09-11 a session reading the two files side by side
attributed this one to §89 on the strength of its mtime.

Two unit tests in `exact.rs` changed and then changed back, and the round trip is
the interesting part. With one sweep, `M` settled shortly enough to square where
it stands, and both tests — which exist to demonstrate §24.17's claim that it
cannot — failed. With the fixed point, `M` reaches `2^-504` and cannot square in
place again, so the original claims are true as written. The published width in
the doc comment moved from "`2^-500` of its largest" to `2^-504`, which is the
settled figure and now a property of the value.

`Exact::square_minus` therefore keeps its consumer. It had looked, for about
twenty minutes, like a part with nothing left to do.

### 88.6 The fuzz crate now type-checks in the gate

Found alongside: `no_step_file_makes_the_reader_panic.rs` had not compiled since
§86 made `Model::solid` return a `Result`, and nothing noticed. `fuzz/Cargo.toml`
declares its own empty `[workspace]`, so `cargo check --workspace` does not reach
it, and a broken target is invisible until somebody runs `cargo +nightly fuzz
build` by hand — after a night of fuzzing has been spent.

That is the §77.6 shape: a check passing for a property of the environment it ran
in. The fix is `just check-fuzz-builds`, in `gate`, and the point worth recording
is that **the nightly compiler and the sanitizer are needed for linking an
instrumented binary, not for type-checking**. `cargo check --all-targets` from
`fuzz/` reads all four targets on pinned stable in about six seconds. The gap was
never that this could not be checked; it was that the reason it was out of the
workspace was taken to cover more than it did.

Verified by putting the §86 break back and watching the arm fail on it.

