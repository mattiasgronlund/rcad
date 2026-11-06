## 483. What the Gappa work paid off, and its case runs parked — 2026-11-06

*Mattias asked, of §464 and the rows that came from Q2647, verbatim: "So what
is your verdict of this, have Gappa proved valuable, or would we have found
these faults using the other methods we use?", then "park the case-by-case
Gappa rows", and "How do 'Keep the habit that paid off most' relate to
Gappa?". This is the answer, and the rows it parks.*

### 483.1 What it cost

At `294f34f7`, 160 commits touch §464, from 2026-10-22 to 2026-11-06, and the
section is 32 481 lines. Q2647's family on the queue page is 103 rows, 79 of
them closed. One row could take many lanes: the sphere six (464.14), the cone
six (464.16), the torus seven (464.17) and the cylinder ten (464.15). What the
lanes spent in tokens is not measured.

### 483.2 What each finding came from

- **The proof.** All nine sign filters are proved at the code's constants on
  the sum `decide` reads (464.4, 464.8, 464.12 to 464.17), and Coq checks each
  proof but for its hints. No test can show this. §16.3's test passed a bound
  wrong by a factor of two, and §18.5 took 10⁸ inputs to find one worst case.
- **The cone's wrong sign** (Q2779) was found reading `on_floats` line by line
  to write `cone.g` (464.16), not by a proof failing. The fuzz target
  `where_a_cone_or_a_torus_crosses` draws numbers across 1 300 powers of two
  and does call `cone.side_of`, but on boxes round points that lie on both
  surfaces, and it has not reported this.
- **Comments that claimed more than was proved**: `value` within `7ε`
  (464.4), the term counts of five constants, `product_is_normal`'s doc
  (Q2767), `NORMAL_LENGTH_ROUNDOFF`'s `63.4ε`, the exact sum `bend_up` is not
(Q2847), and the
  `4u` of the parabola's directions (464.41). Tests carry too much slack to see
  these. No answer moved: every constant held.
- **The faults users can meet** came from a hypothesis a script needed and an
  ordinary probe of what the code does without it. 483.3 is about this.
- **The case runs**, a proof run again over more branches, zero factors or
  products below the normals, found three faults, all at numbers outside
  §3.5's `1e11`: Q3209 (wrong/C, from 464.35), Q2897 and Q3194 (refuses/D,
  from 464.31 and 464.33). Each of orient3d's five (464.11, 464.25, 464.26,
  464.43, 464.44) ends "nothing in `orient3d.rs` needs to change". The rest
  restated comments.

### 483.3 The habit, and why it is Gappa's

A Gappa script does not prove until every condition the bound needs is written
as a hypothesis. Leave one out and the proof fails: 464.4's 3 drops the
products being normal and the goal fails. A derivation by hand does not fail
that way: §12.2 and §16.9 went on without their condition, and §15.6's was
wrong with nothing to run it. So each script ends with a list of what the code
must make sure of.
Each item on it is a question: does the code make sure of it? Where the answer
was no, an ordinary test or probe found the fault:

- 464.20's `square.g` takes a normal within a constructor's `ε` before its
  turn. Q2826 asked whether a normal turned twice is, and it is not (Q3219).
  Q3220 then built a designer's shape on that path, fused, placed turned, fused
  again, written and read. It found Q3224 (refuses/B), Q3226 and Q3234
  (wrong/B), all fixed.
- 464.19 takes the turn's `q·q` within `6ε` of one. Q2803 found the
  `From<na::Isometry3>` route took a turn of any length, and it is `TryFrom` now
  (Q2852).
- 464.35 needs `L` normal. Its probe found `apply_to_point` `1.8e15 u·L` off
  below that (Q3209, fixed).
- The derivation 464.23 proves assumes `|s|` at most `K`. Q2849 found nothing
  holds it there (Q2855, open).

The fuse fuzzer would not have flagged the class B three. Its target hands
`two_models_split_and_fused_noting_everything` a refusal callback that does
nothing, it fuses once, and it never reads a model back twice.
A test of that shape would have, and that is the method this house already
uses. What it lacked was a reason to build that shape, and that came from the
hypothesis. So the probe needs no prover, but what to probe came from one. Read
the hypotheses a proof needed, ask of each whether the code makes sure of it,
and where it does not, build a shape without it.

### 483.4 What is parked, and what is not

The case runs' yield so far is class C and D faults and comment edits, which
the fill hardly reaches (§450.4). Each has taken from two lanes (464.43,
464.44) to seven (464.31). Parked here:

- Q2881, the cylinder's checked products below the normals.
- Q2895 and Q2896, the cone's halves and other base states below the normals.
- Q3193, the torus's checked products below the normals.
- Q3210, 464.21's pipe with a point's slips at 464.35's `34.73u`.
- Q3213, `square.g` with a component between 0 and `2⁻¹⁰¹²`.

A lane that changes which products a filter checks unparks that filter's row.
Q2779's fix in `cone.rs` is the first that may change Q2895's and Q2896's.

**Not parked.** Q3223 and Q3232 are not more cases of a finished proof. They
ask whether a published bound holds on ordinary numbers, and nothing has
proved it. A corner on a moved cone comes to about `290u` on paper against
`CORNER_AFTER_MOVING_ROUNDOFF`'s `256u`, and the search finds `60u` (464.21,
464.38). The parabola's directions pass `4u` from slope 3 (464.41). Also open:
the tooling that keeps the proofs true as the Rust moves (Q2699, pinned;
Q2700, Q2701, Q2761), the faults (Q2779, pinned; Q2855, Q2897, Q3194, Q3219,
Q3225, Q3227), the comments (Q3218, Q3231, Q3237), and the two tools not yet
tried (Q2696 Lean, Q2697 FPTaylor or Lean).

From here, Gappa is for a bound constant that is new or rewritten, behind
`check-proofs` once Q2699 builds it.
