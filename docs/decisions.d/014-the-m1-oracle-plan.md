## 14. The M1 oracle plan

*Recorded 2026-08-30, before M1.*

§13 rule 1 makes the oracle list and the work breakdown the same document: a
piece of M1 can be worked on independently exactly when the property deciding it
is correct is written down before the work starts. So this list is not a test
plan appended to a design — it is what says which parts of M1 fan out and which
stay in the gate. A candidate algorithm with nothing here is gate work.

### 14.1 What a candidate has to answer

`rcad-predicates/tests/scaling_a_predicate_must_not_change_its_answer.rs` is the
worked example. Its module comment answers all four of these, and that is why it
found a bug the differential suite next door could not see.

1. **Is the relation exactly true, or does it need a bound?** Exactly-true
   relations cost nothing to trust. A bounded one is worth nothing until the
   bound is published beside the expression it bounds (§13 rule 4); without that
   the tolerance gets tuned until the test passes, and then the test is
   measuring the tolerance.
2. **Do the two runs take different paths through the dangerous code?** The
   discriminating question. Scaling by a power of two works because one run
   decides at level 1 and the other does not. Two runs down the same branch
   prove nothing, however pretty the algebra.
3. **What family of inputs does the generator make?** §12.3. Different per
   piece, deliberately, or two pieces report green together and the agreement
   means nothing.
4. **Which gap in §12 would this have caught?** If none, say why it is being
   written anyway.

### 14.2 What M0 already covers

| Kind | Stock |
|---|---|
| Metamorphic | scaling by 2ᵏ on predicates and surfaces; rigid-motion algebra — round trip, associativity, identity, distance preservation, homomorphism; volume under rotation; serialization round trip |
| Differential | `orient2d` and `orient3d` against `robust`, with the shared-lineage blind spot §12.2 records |
| Refusal | dynamic range, error shapes, undecidable through constructed points |

Nearly all of it is **invariance**. There is no closed-form oracle anywhere in
the tree, because planar M0 had nothing whose answer is known in advance. M1 is
the first milestone where that axis exists, and it is the cheapest new coverage
available.

### 14.3 Exactly true, so free to trust

Write these first.

- **Partition of unity.** `Σ Nᵢ,ₚ(u) = 1`. The sum has p+1 terms, so the bound
  is derivable rather than guessed. A direct test of Cox–de Boor.
- **Basis functions are never negative and never exceed one.** Together with
  partition of unity this gives the convex hull property, far cheaper than
  testing hull containment. Roadmap §6.2's Bézier clipping rests on that
  property, so this is M1 protecting M3.
- **Knot insertion is bit-exact outside the affected span.** Boehm's algorithm
  touches p+1 control points and copies the rest, so evaluation in an untouched
  span must agree to the last bit. This is the sharp form. "Insert a knot and
  the curve is unchanged" everywhere is the same idea needing a bound, and
  belongs in §14.5.
- **The split seam.** At a knot of multiplicity p the curve passes exactly
  through a control point, so both halves evaluated at the seam must agree bit
  for bit.
- **Scaling by 2ᵏ, extended to evaluation and derivatives.** A curve scaled by s
  has every derivative order scaled by exactly s. Reuses M0's generator whole
  and aims §12.2's bug family at the new arithmetic.

### 14.4 Closed form — the axis M0 could not have

- A **rational quadratic Bézier circle** is exact by construction, so evaluated
  points must satisfy M0's sphere sign predicate. No shared lineage at all.
- **Curvature of a circle is `1/r`, of a line `0`. Gaussian curvature of a
  sphere is `1/r²`, mean curvature `1/r`.** Tests roadmap §4.3's fundamental
  forms against known answers. Fix the mean-curvature sign convention in the
  gate — it is a coin flip, and two pieces of a fan-out will call it
  differently.
- **Arc length of a circular arc is `rθ`.** Tests roadmap §4.4.

### 14.5 Bounded, and therefore gated

Degree elevation preserves the curve; splitting reproduces the original on each
subdomain; all weights equal reproduces the non-rational B-spline. All three are
good relations and none can be written until §13 rule 4's bounds exist. They are
gate work, not fan-out work, and listing them here is how that stays visible.

### 14.6 Differential is weak here

de Boor against Cox–de Boor is tempting and close to worthless: one author, one
book, one set of conventions, so a misread convention appears in both. That is
§12.2's failure exactly. `curvo` and `truck-geometry` are genuinely independent
lineage and worth a low-weight check if their licences allow it. Neither may be
the only check on anything.

### 14.7 The acceptance oracle, and why it goes first

Every refusal-shaped test in the tree checks that bad input is refused. Nothing
checks that a refusal was **deserved** — that `Undecidable` came back because the
case is genuinely undecidable and not because a filter was timid.

§12.3 already stated the general form: a property suite must assert that valid
input is accepted, not only that invalid input is refused. M1 needs the two-sided
version — cases whose answer is exactly computable, which `Policy::certified()`
must decide. Build it first, not last. Without it a cascade that refuses far more
often than it must passes every other oracle on this list, and §1.7's exit
criterion 8 records a number about our filters rather than about the geometry.

### 14.8 Carried to M2.5

Not M1, recorded here because it came out of the same pass. §1.7's exit criteria
5–8 name the boolean oracles; three more belong with them. **Commutativity** of
union and intersection, which catches argument-order dependence — the defect
§9.2's bit-identical criterion exists to find. **Idempotence and absorption**:
`A ∪ A ≡ A`, `A ∩ A ≡ A`, `A \ A ≡ ∅`, `A ∪ (A ∩ B) ≡ A`. And
`fuse([A,B,C]) ≡ fuse([fuse([A,B]), C])`, which takes genuinely different paths
and tests the whole reason roadmap §7.2 exists.

---

