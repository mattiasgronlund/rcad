## 40. Every public enum grows, and what actually checks that, recorded 2026-09-01

*Recorded after M4 merged. The rule in CLAUDE.md — `#[non_exhaustive]` on every
public enum — has stood unchanged since before any code was written, and has
never been checked by anything. This section says what it has always meant, what
was built to check it, why that did not work, and what does.*

### 40.1 What the rule buys, and what it quietly costs

The rule is one line: `#[non_exhaustive]` on every public enum, because they all
grow each milestone. M4 is the fifth milestone to prove the second half of that
sentence — `Containment` lost a variant and `SurfaceMeet` gained several.

What was never written down is that **`#[non_exhaustive]` does nothing inside the
crate that owns the enum**. `rcad-topo` matching its own `Error` is checked for
exhaustiveness exactly as if the attribute were absent. What the attribute
actually does is force a last arm on every match in the *other* four crates.

So the rule that reads as "our enums are free to grow" also reads as "every
cross-crate match on one of them carries a wildcard", and a wildcard is where a
new variant goes to be ignored. Those two readings have been the same decision
all along; only the first one was recorded.

### 40.2 How many places that is

Counted on this commit: **154** wildcard match arms across the five crates'
`src`. **97** of them panic or are `unreachable!`. **57 do not.**

Two of the 57, both correct today:

```rust
// rcad-topo/src/fuse/arrange.rs — on Decision
match onwards(at - self.nodes[from].point.enclosure()[axis]).sign() {
    Decision::Neg => continue,
    _ => return None,
}

// rcad-geom/src/walk.rs — on EdgeRun
let picked_by: &[Surface] = match run.run() {
    EdgeRun::PartOfWhereItCrosses { picked_by, .. } => picked_by,
    _ => &[],
};
```

The first folds `Zero`, `Pos` and `Undecidable` into one answer, which is what it
means to. The second says a run of any other kind is picked by nothing, which is
true of every kind there is. Neither is a bug. Both are places where the variant
M5 adds changes behaviour with no diagnostic, in a kernel where §2.1's whole
argument is that every call site must say what it does with `Undecidable` — and
where `Decision`'s own doc comment tells the reader "never let `Undecidable` land
in it". That is a rule enforced by attention.

### 40.3 A feature that takes the attribute off checks nothing, demonstrated

The first thing built here was a cargo feature that removes `non_exhaustive`:

```rust
#[cfg_attr(not(feature = "exhaustive-for-checking"), non_exhaustive)]
```

off by default, with `just check-exhaustive` building the workspace with it on,
so that the compiler would name every cross-crate match that had stopped
covering its input. It was written, wired through all five crates, and run.

**It passes on a clean tree, and it will pass forever.** The reason is
structural. A wildcard arm makes a match exhaustive whatever the enum holds, and
every cross-crate match here *has* a wildcard — it must, or it would not compile
in the default build, where the attribute is on. Removing the attribute cannot
make an existing site fail. When M5 adds a variant, all 154 wildcards absorb it
exactly as before, feature on or feature off.

The demonstration, since §16.10 asks for one rather than a paragraph: a variant
added to `Level` produced **zero** errors with the feature on and zero with it
off. What the feature does change is that a site enumerating *every* variant and
then carrying a wildcard begins to emit `unreachable_patterns` — and when a
variant is added, that warning silently goes away again. A signal that vanishes
when the thing it watches for happens is not a signal.

This also sinks the downstream story the feature was half built for. A consumer's
own wildcards absorb new variants the same way, so a CI job with the feature on
proves nothing to them either, unless they omit wildcards entirely and never
build without it.

The whole mechanism is therefore **not built**. It is recorded because it is a
plausible idea that a later reader will have again, and because the reason it
fails is not obvious until it is stated.

### 40.4 What does work

`clippy::wildcard_enum_match_arm`, on stable, in the default build, with no
feature at all:

```
warning: wildcard matches known variants and will also match future added variants
   --> crates/rcad-topo/src/validate.rs:837:9
837 |         other => panic!("a face with no side to its material: {other:?}"),
    |         ^^^^^ help: try: `other @ Decision::Zero | other @ Decision::Undecidable { .. } | _`
```

The suggestion is exactly the discipline this section wanted: name every variant
that exists, and keep `_` for the ones that do not yet. A site written that way
goes quiet. When M5 adds a variant, that site has an unnamed variant under its
wildcard again, and the lint fires at the site — which is the arm that would
otherwise have swallowed it.

Counted over the whole workspace including tests and benches: **106 sites**.
§40.8 is what they turned out to be and what was done about them.

**An `#[allow]` on the arm does not work**, which this section said it would.
Sixty-four were written, formatted and run, and clippy reported every one of
them anyway: rustc resolves a lint level at the enclosing *expression*, and a
match arm is not one, so the nearest attribute the level lookup reaches is the
whole function. Per-site granularity is not available on stable, and a
function-wide `#[allow]` would hide the next wildcard added to that function,
which is the thing being watched for.

### 40.5 What else was rejected

**`non_exhaustive_omitted_patterns`.** The rustc lint built for precisely this
question: warn when a `non_exhaustive` enum gains a variant a match does not name,
while still allowing the wildcard. It is unstable and needs nightly;
`rust-toolchain.toml` pins stable and §8.1 says why. It is what to use if it
stabilises, and it is better than the clippy lint because it asks the question
only of foreign enums, which is where the question is.

**Doing nothing, and relying on the panicking wildcard.** It is already the
majority idiom — 97 of 154 — and it is a decent one. But it converts a question
that can be asked at build time into a panic at run time, and §4.1's tiers say a
panic is for a broken internal invariant, not for a case nobody got round to
handling. The 57 that do not panic are the argument on their own.

### 40.6 What none of this buys

Stated because a check whose reach is overestimated is worse than no check.

- **Nothing inside a crate.** `rcad-topo`'s matches on `rcad-topo`'s enums are
  unaffected, because they were already exhaustive.
- **Nothing for a wildcard that folds variants on purpose.** `Decision::Neg =>
  continue, _ => return None` is silent under every mechanism here and should be.
  What the lint buys is that the folding becomes an `#[allow]` somebody wrote,
  rather than a `_` nobody looked at.
- **It does not stop a variant being added carelessly.** It guarantees the sites
  are *looked at*, not that the answer written at them is right.

### 40.7 How the rule itself is checked

`just check-enums`, in `just default`. A `pub enum` in any crate's `src` must
carry `#[non_exhaustive]`, or a doc comment saying **"Exhaustive on purpose"** —
which `Orientation` says, and is the only one that does (§3.6).

This is the part that would otherwise rot. A new public enum written without the
attribute compiles, passes every test, and is invisible; the rule has gone
unchecked for four milestones and happens to have held. It is a five-line `awk`,
and the marker it looks for is the sentence `Orientation`'s doc comment already
had, so the exception carries its own reason at the only place it applies.

### 40.8 What the 106 sites were, and what counts them now

Sorting them by what happens when the next variant arrives, rather than by
which enum they match, split them cleanly:

| | sites | what a new variant does there |
|---|---|---|
| the arm panics or is `unreachable!` | 68 | a loud stop, naming the variant |
| the arm returns an `Err` naming what it saw | 3 | a refusal a caller can read |
| the arm forwards a whole `Error` enum onward | 3 | forwarded, which is what forwarding means |
| the arm folds on purpose | 2 | correct: every surface that is not a plane is curved, including ones added later |
| **the arm absorbs it silently** | **30** | `return None`, `continue`, `&[]`, `1.0`, `Ok(false)` — no trace at all |

Only the last row is a defect, and §40.2's count of 57 was the right order and
the wrong set: what matters is not whether the arm binds a name but whether it
makes a noise. Binding `other` to print it in a panic message is a *symptom* of
being loud, not the thing itself.

**All 30 were fixed**, by naming the variants they fold — the one-line change
`_ => return None` →
`Decision::Zero | Decision::Pos | Decision::Undecidable { .. } | _ => return None`.
Nothing else changed: all 1113 tests pass unchanged, which is what a pattern-only
edit has to show.

Two of the thirty were inside the crate that owns the enum, and there the
compiler had the last word: with every variant named, `| _` is unreachable and
`rustc` says so. §40.1 argued that; those two lines are it happening.

**The check is a count, not a denial** — `just check-wildcards`, against
`wildcard_arms` in the justfile, because §40.4's per-site `#[allow]` is not
available. The count moves for both of the reasons worth knowing about:

- a **new wildcard** raises it. Demonstrated: putting one back in
  `measure.rs` reported 77 against 76 and failed.
- a **new variant** raises it too, because the thirty sites that now name every
  variant there is start matching a known one. The compiler gets there first
  inside the owning crate — a variant added to `Decision` stopped
  `rcad-predicates` compiling — so the recipe distinguishes "clippy found
  nothing" from "clippy never ran", and prints the build errors when it is the
  second.

A count is a baseline with a baseline's failure mode: a legitimate new panicking
wildcard fails the check until the number is committed with it. That is the same
deal a golden file makes, and it is the reason the number lives next to the
recipe rather than in a file nobody reads.

---
