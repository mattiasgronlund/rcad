## 66. What a release build hands back when the kernel is wrong, recorded 2026-09-07

§4.1 gives a broken internal invariant one response, a panic, and the reason is
that emitting a corrupt solid is worse than dying. §4.3 then wrote "in debug"
beside the checks that would notice, and gave a reason about **test-suite
runtime**: a full sweep after every operation makes an *n*-operation test O(n²).

Those two sentences are about different things and the second quietly became the
policy for the first. In a release build `Model::check_touched_solid` and its
three siblings open with `if !cfg!(debug_assertions) { return; }`
(`crates/rcad-topo/src/model.rs:547, 564, 575, 632`) and
`Assembly::commit`'s post-write sweep is gated the same way
(`crates/rcad-topo/src/fuse/assemble.rs:300`). So the release answer to a broken
invariant is neither of §4.1's two: no panic, no `Err`, and a solid that does not
validate comes back as `Ok`. §61 is that bug found by a fuzzer and fixed at the
door; this section is about the door being the only thing that was between it and
a caller.

The proposal this section settles is Mattias's: keep the panic in debug, and in
release record enough for a post-mortem and return an error that says *internal
error*. What follows measures the reason §4.3 gave, so that the policy is decided
by a number rather than by an argument about a different check.

### 66.1 What a release build does today, replayed

Three runs of one harness against `a6f03b8`, built with plain `cargo build
--release` — the release profile has `debug-assertions` off, and
`.cargo/config.toml` supplies the `-C target-cpu=x86-64-v3` that `inari` needs,
so no `RUSTFLAGS` is involved and nothing clobbers it. The harness is the fuzz
target's own decode and split (`fuzz/fuzz_targets/two_solids_fused_by_a_stranger.rs`),
with the one difference that it **keeps** the fuse's answer instead of doing
`let _ =` on it, and asks `Model::validate` about every model the fuse called
`Ok`.

- **341 crash artifacts** from the two soak trees: 12 halves reached the fuse, 7
  answered `Ok`, and **0** of those fail `Model::validate`. §61 and the four
  lanes merged into `a6f03b8` closed what was there.
- **The 9 975-input corpus** (4 835 + 5 140 across the same two trees): 2 923
  halves reached the fuse, 1 060 answered `Ok`, and **0** of those fail
  `Model::validate` *under the policy the fuse itself ran under*.
- The same 1 060, validated a second time under `Policy::certified()`: **16**
  fail, all sixteen fused under `Policy::tolerant`, **fifteen under
  `Rule::Difference` and one under `Rule::Union`**, ten with a span of `inf`.
  The complaint is always `Invalid::DynamicRangeTooWide`.

  The rule count was first written here as thirteen, which was wrong. The rule
  is the input's first byte mod 3 (`fuzz_targets/two_solids_fused_by_a_stranger.rs:97`),
  and fifteen of the sixteen carry `0xb9` against `2fb467fc`'s `0x84` — counted
  off the bytes, twice, by two people who did not share a harness.

**Those 16 are not a release-build hole, and saying so is the point of measuring
them.** They were carried as an unexplained residue — sixteen wrong answers on
both trees, unchanged by the fixes. What the second validation shows is that they
are the *same* answer read under a stricter policy than the one that produced it:
`Assembly::commit` sweeps under the operation's own `policy` (`assemble.rs:300`
and `:309`), so a debug build accepts these too, and a debug build and a release
build agree about all sixteen. Nothing here is decided by how the caller was
compiled.

What they do show is a separate thing worth its own row: **"valid" is not a
property of a model on its own.** A `Policy::tolerant` fuse can build a solid
whose coordinates span `inf` and both the fuse and `Model::validate` will call it
sound, and the same model handed to a certified caller is broken. That is left
open in §66.7.

### 66.2 What the touched checks cost in release — the measurement

Three release binaries from the same tree, differing only in a forced-on gate,
run interleaved so that machine noise falls on all three alike, best of five:

| Workload | Today | Touched checks on | Both on |
|---|---|---|---|
| 144 unions of cubes, growing to a 98-face solid | 7.349 s | 7.817 s | 7.712 s |
| 200 000 `place_solid` on a 202-face solid | 3.03 ms | 2.001 s | 2.019 s |
| 2 000 `add_extrusion` of a 62-face solid | 112.8 ms | 113.6 ms | 115.0 ms |

The extrusion row is the only one where five runs are not enough — the gap is
smaller than the spread — so it was run again on its own, fifteen interleaved
pairs, giving 103.7 ms against 111.9 ms. That is the figure quoted below.

"Touched checks on" replaces the four `!cfg!(debug_assertions)` guards with
`false`. "Both on" also removes the `cfg!(debug_assertions) &&` from
`assemble.rs:300`, so the fuse sweeps the whole model twice per call in release
as well.

Two figures come out of that, and they are what the whole argument turns on.

**The touched checks cost 50–65 ns per face of the solid they are handed.**
`place_solid` does nothing else at all — it rewrites one isometry per shell — so
it isolates them: 15.2 ns per call becomes 10.0 µs on a 202-face solid, which is
49 ns a face. `add_extrusion`, timed on its own over fifteen interleaved pairs,
goes from 51.9 µs to 56.0 µs per 62-face solid, which is 66 ns a face.

**The full sweep costs 3.4–4.4 µs per face of the whole model.** Measured with
no source edit at all, since `Model::validate` is public: 209 µs at 62 faces,
2.14 ms at 620, 22.9 ms at 6 200, 108 ms at 24 800. Linear, as it should be.

So the checks §4.3 sent to debug are **seventy times cheaper per face** than the
sweep §4.3's O(n²) is about, and they are taken over *one solid* rather than over
the whole model, which is the half that makes them not quadratic at all. On a
model of 400 solids a sweep per operation is 108 ms and a touched check is 4 µs:
a factor of 25 000, and only one of the two grows with the model.

What that buys, per operation:

- **A fuse: 0.01%.** 51 ms per union in the chain above, against about 5 µs of
  checking. The measured +6.4% is smaller than the baseline's own spread across
  five runs (7.35 s to 9.17 s), so the honest statement is that it is below this
  machine's noise floor, and the per-face figure says why.
- **Building a solid from scratch: +7.9%.** The one real cost. `add_extrusion`
  spends 52 µs on a 62-face solid and 4.1 µs of checking is a visible fraction of
  it. This is the number that would show up in an importer.
- **`Model::place_solid`: ×660.** 15.2 ns becomes 10.0 µs. In ratio this is
  enormous and in absolute terms it is ten microseconds on a 202-face solid, so
  an application moving a solid sixty times a second spends 0.06% of a frame on
  it. It is still the one call where the checks are the entire cost, and a
  transform-heavy loop over a large assembly is where somebody would notice.

**§4.3's stated reason does not survive this.** "The test suite becomes
quadratic" is true of `Model::validate` and false of the touched checks; it was
never measured for them, and measured it is between nothing and eight percent.
A decision about test-suite runtime is not a decision about what a released
kernel hands a caller, and this is the number that says so.

### 66.3 So the checks run always, and then the question is what they do

The gates go. `check_touched_solid`, `check_touched_face`, `check_touched_wire`
and `check_touched_edge` run in every build, at 50–65 ns a face, and
`assemble.rs:300`'s post-write sweep runs in every build too — it is one
`Model::validate` of a model the fuse has already spent tens of milliseconds on,
and both binaries above say it is free at that scale.

That is the half of the proposal the number decides on its own, and it is worth
separating from the half it does not: **running the check is not the same
decision as what the check does when it fails.** Today a release build cannot
panic because it never looks. Once it looks, §4.1's panic is back by default, and
Mattias's proposal is that release should instead record and return. §66.4 is
where that stops being uniform.

**Implemented 2026-09-09 by §82, two days late and not quite as written.** The
four touched checks lost their gates exactly as this subsection says. The
post-write sweep did not: **the headline figure above does not cover a model
holding geometry the fuse never touched.** "A model the fuse has already spent
tens of milliseconds on" is true of the workloads §66.2 measured, where the
fuse built the model itself and its own cost grows faster than the sweep's. Fuse
two small blocks in a model of 5040 faces and the sweep costs about 20 ms where
the fuse costs a millisecond or so (§82.7) — quote the 20 ms and not a ratio,
because three machines put the fuse itself at 0.82, 1.34 and 1.44 ms, which
moves "how many times the fuse" from fourteen to twenty-five while the sweep's
own 1.9–2.1 µs a face barely moves at all. Worse, a whole-model sweep can
*fail* for a solid the fuse never touched, because the coordinate range is a
fold over every solid there is — now
a test in the tree (§82.5).

So what runs after every fuse, in every build, is
`Model::check_what_was_just_written`: eleven of the sweep's thirteen checks,
asked of the entities the fuse wrote and of nothing else, with the sweep's own
tolerances so that an answer it accepts is one `Model::validate` accepts. §82.4
is the argument that this asks the same questions. The half separated out above
is unchanged — the check still panics, and §66.7's first item is still what
stands between it and an `Err`.

### 66.4 Which operations can return `Err`, and which cannot

The line is not "does this operation document `# Atomic`". It is **can the check
be made before anything the caller can see has changed** — structural
confinement, not a paragraph about blast radius. Every entry point that mutates a
`Model`, with where its commit is:

| Entry point | Where it writes | Verdict |
|---|---|---|
| `SolidBuilder::build` (`builder.rs:320`) | scratch `Model`, swept, then the caller's | **Convert.** Structural |
| `Model::fuse` (`fuse/mod.rs:149`) | ~~the caller's model, `assemble.rs:303`~~ throwaway `Model`, swept, then the caller's (§87) | ~~Panic, until it gets a scratch~~ **Converted, §87** |
| `Model::add_cube` (`build.rs:495`) | the caller's model, additive | Panic, until it gets a scratch |
| `Model::add_extrusion` (`build.rs:533`) | the caller's model, additive | Panic, until it gets a scratch |
| `Model::add_round_bar` (`build.rs:623`) | the caller's model, additive | Panic, until it gets a scratch |
| `Model::add_ball` (`build.rs:719`) | the caller's model, additive | Panic, until it gets a scratch |
| `Model::add_bar_cut_off_by` (`build.rs:805`) | the caller's model, additive | Panic, until it gets a scratch |
| `Model::place_solid` (`model.rs:280`) | existing entities, in place | **Panic.** Nothing to undo it with |
| `Model::set_vertex_approximation` (`model.rs:308`) | one field, in place | **Panic** |
| `Model::set_edge_approximation` (`model.rs:337`) | one field, in place | **Panic** |
| `Model::set_face_approximation` (`model.rs:381`) | one field, in place | **Panic** |

`SolidBuilder::build` is the only one that is already right, and it is worth
reading because it is the shape every other row wants: it writes the draft into a
`Model::new(model.scale())`, runs the full sweep on *that*, returns the sweep's
`Err` if it fails, and only then writes into the caller's model
(`builder.rs:330-340`). Its `check_touched_solid` on the real model afterwards is
a second write of a description that just passed a full sweep, so it should never
fire — and if it did, the caller's model would already have it.

**The fuse is not confined the way it looks, and this is the correction that cost
the most to find.** `Model::fuse` documents `# Atomic` — "the whole answer is
worked out in scratch and written in one go. Any `Err` leaves the model
byte-identical" — and it is true of every error the operation raises, because
`Scratch`, `Assembly` and `Chosen` are plain data and every refusal happens
before `commit`. The scratch `Model` at `assemble.rs:1112` is **not** the answer
being built: it is a throwaway used to ask which shell nests inside which, by
measuring volumes. `Assembly::commit` takes `model: &mut Model` and
`write_into(model, …)` at `assemble.rs:303` writes straight into the caller's
arena. The sweep at `:309` that would catch a corrupt answer runs **after** that
write.

So converting the fuse's panic to an `Err` today would hand the caller an error
*and* leave the corrupt solid in their model. The write is purely additive —
`write_into` only calls `add_vertex`, `add_edge_running`, `add_coedge`,
`add_wire_of_steps`, `add_face`, `add_shell`, `add_solid`, and the doc comment's
"nothing is removed from the arena" holds — so the *inputs* survive. That is not
enough: the orphan entities stay in the arena, and every later `Model::validate`
fails on them. An `Err` that permanently poisons the model is worse than the
panic, not better.

**Nothing in this crate can undo a write.** `grep -n '\.remove(' crates/rcad-topo/src/`
returns thirteen hits and every one is either a test helper reaching through
`data_mut()` or `serialize.rs` tidying a model it is still building. There is no
rollback and no arena free-list walk, so "additive" does not mean "undoable".

The fix is the shape `SolidBuilder::build` already has, and the same measurement
pays for it: `Assembly::commit` writes into a scratch `Model`, sweeps it, and
writes into the caller's only if the sweep passed. The cost is one extra *write*
of the answer's entities plus one sweep of a model the size of the answer,
against a fuse that costs 51 ms. The `add_*` builders in `build.rs` are the same change,
and they are the ones where 66.2's +7.9% would matter — an importer that pays a
scratch-and-copy per solid pays more than 4 µs. That is a real trade and it is
why those rows say "until", not "never".

> **Amended 2026-09-10, §87.** The fuse row is built. "Copies" is the word this
> paragraph gets wrong and the precedent it cites in the same breath gets right:
> `SolidBuilder::build` writes its description twice rather than copying, and so
> does `Assembly::commit` now. `Assembly::write_into` reads nothing out of the
> model it writes into and names no entity that was already there, which is what
> makes a second call available; §87.5's second test demonstrates that the two
> writes agree rather than arguing it. Costed at 34.2 µs on a 12-face flat
> answer and 55.0 µs on an 18-face one (§87.7), not at the 51 ms this paragraph
> budgeted against. The five `add_*` rows still say "until", and §87.8 now has a
> measured number for what converting them would cost rather than a proxy.

`place_solid` is the one row where the answer is a settled panic. It overwrites
`used.loc` on every shell of an existing solid before checking anything
(`model.rs:289-293`), and the previous isometries are gone. The inverse isometry
would restore them — `movement` is invertible or `Isometry::translation` would
not have built it — so this is a *gap* and not a *limit* in §16.10's sense, and
§66.7 keeps it open rather than closing it with a paragraph. Until somebody
writes that, there is nothing to return an `Err` *to*: the model is already
wrong.

### 66.5 Two doors, and why they must not be one

`Error::Undecidable { records }` means the arithmetic honestly could not settle
something, and a caller is *supposed* to answer it by retrying under a more
tolerant policy — that is what `Policy` is for and what §4.2 says the variant is.
"This kernel has a bug" arriving through that same door puts the retry loop on
top of a computation that is already wrong, and a tolerant retry is exactly the
thing most likely to accept the wrong answer. So the two must be
distinguishable without reading a string.

**A new variant on `Error`, not a new error type.** `Error::TheKernelIsWrong`,
carrying an owned report. Considered and rejected: a separate `KernelBug` type
returned as `Result<T, Either<Error, KernelBug>>` from every entry point. It is
the more honest shape in the type system and it is worse here, because every
public entry point already returns `Result<_, Error>`, a second type forces a sum
at every call site in `rcad` and in every consumer, and the `From` impl that
would make that bearable is precisely the place the two get merged back into one
again. `Error` is `#[non_exhaustive]` (§40), so adding the variant is not a
breaking change, and the wildcard-arm rule means every existing match keeps
compiling while `just check-wildcards` still names what is unhandled.

**The name must not be able to be read as a refusal.** `Error::Internal` is what
this would usually be called and it is the wrong name here, because a caller
skimming variants sorts them into "the input was bad", "it could not decide" and
"something else", and "something else" is where a retry loop puts its default
arm. The variant says what happened to *the kernel*, not to the request.

**What a caller does with it: report it, and do not retry.** In that order and
nothing else. Retrying under a tolerant policy re-runs an operation whose
intermediate state is already known to be wrong. Retrying under the *same* policy
is deterministic and will fail the same way. Falling back to another operation
uses a model that may already hold what the check tripped on. The doc comment on
the variant says this, and so does its `Display`, because the string is what ends
up in somebody's bug report.

### 66.6 Where the post-mortem record goes, and what it costs when nothing is wrong

`ProvenanceLog` is the obvious sink and it is the wrong one, for the same reason
§66.5 gives about the door. It is a `Vec<Record>` of decisions the arithmetic
could not certify (§5), it is what `Error::Undecidable` carries, and
`log.worst_wobble` is read to *decide* whether an operation refuses
(`fuse/mod.rs:183`). A kernel bug written into it is a record that reads as one
more undecided decision, and any consumer summing wobble over the log would be
summing a bug into a tolerance. It also breaks the rule the log exists to keep:
provenance is recorded per decision, and a broken invariant is not a decision.

So the variant carries its own owned report, built only on the failing branch:

- **Which check tripped**, as a name, not a formatted sentence.
- **Which entities**, as the keys the check was handed — a `SolidKey` and the
  `EdgeKey` under it, say. Keys, because they are what a caller can hand back and
  what a later `Model::validate` will name too.
- **The operation's `ProvenanceLog` as it stood**, moved in rather than pointed
  at. §5's reason for splitting the log off the model is that a pointer into a
  log dangles the moment the model changes, and a report that outlives the
  operation is exactly that case. It is a move of a `Vec` that the operation was
  about to drop.

**The cost when nothing goes wrong is zero, and it is zero for the same reason
`ProvenanceLog` is.** The report is a struct built inside the `else` of a check
that passed; a certified operation constructs nothing, allocates nothing and logs
nothing. What is *not* free is the check itself, and §66.2 is the answer to that:
50–65 ns a face, always, in every build.

### 66.7 What this leaves open

**§82 closed none of these.** What it closed is §66.3 — the checks now run in
every build — and running the check was always the half separated from what the
check does when it fails. All four items below stand, with one sentence in the
first of them corrected.

**~~`Assembly::commit` needs a scratch model before the fuse's panic can become
an `Err`.~~ Closed by §87.** §66.4 says why and says what shape it takes. Nobody
has measured the copy — the sweep half is measured above and is free at a fuse's
scale, and the copy is an unknown that is probably also free at that scale and
certainly is not at `add_extrusion`'s. **Still open after §82**, which put the
check in and left the `expect` where it was: Mattias chose the branch where a
failure still panics, so the scratch-and-copy is unbuilt and unpriced. "The
sweep half is free at a fuse's scale" is the sentence §82.7 corrects — it holds
where the fuse built the model and not where the model holds geometry the fuse
never touched, and the check that actually went in is local for that reason
(§66.3's amendment).

> **Amended 2026-09-10, §87.** Built, and "nobody has measured the copy" is now
> false twice over. There is no copy — the fuse calls `write_into` a second time
> — and it is measured: 34.2 µs on a 12-face answer against a fuse of 1.42 ms,
> 55.0 µs on an 18-face answer against 2.94 ms, about 3 µs a face on a flat
> answer and four times that on a curved one (§87.7, whose shapes are named
> there and whose first cut had the 18-face fuse at 4.75 ms without saying which
> three cubes it meant). A broken answer is
> `Error::TheFuseWroteABrokenAnswer` and the caller's model is untouched. Of the
> three items below this one, the fourth-listed `add_extrusion` figure is now
> measured rather than a proxy; the other two stand.

**Whether `place_solid` should roll back with the inverse isometry.** It can:
`movement` is invertible. Whether an operation that discovers a broken solid
should put the *previous* broken-in-a-different-way state back is not obvious,
and §4.1's answer for an in-place mutation is still a panic until somebody argues
the other side.

**Whether "valid" should depend on a policy at all.** §66.1's sixteen are one
model that `Model::validate` calls sound under `Policy::tolerant` and broken
under `Policy::certified()`, with a coordinate span of `inf` in ten of them. That
is not a release-build question and it is not answered here, but a solid that is
valid only for the caller who made it is a thing a file format has to have an
opinion about, and `Invalid::DynamicRangeTooWide` is where that opinion would go.

**The +7.9% on `add_extrusion` has no consumer to weigh it against.** It is
measured on a synthetic loop of 2 000 extrusions. Whether it matters is a
question for the importer, and §13's rule says the answer waits for the importer
to exist rather than being guessed here.

> **Amended 2026-09-10, §87.** The +7.9% was never a measurement of this shape
> and is withdrawn. What a throwaway-and-write-twice `add_extrusion` would
> actually cost is **about 14 µs added to a ~3 µs operation** — a 6-face box
> takes 2.79–2.90 µs to build into a fresh `Model`, and the rehearsal (a fresh
> `Model`, the same build again, the twelve-check sweep) adds 13.8–14.2 µs
> (§87.7's second table). Call it roughly six times the present cost; the
> denominator is the part that moves, so a two-figure percentage does not
> survive re-measurement. **The +640% first published here is withdrawn too**:
> it divided the rehearsal by 2.2 µs, which is `add_extrusion` into a model
> whose arenas have already grown, and a rehearsal always starts from
> `Model::new`. The proxy was still out by nearly two orders of magnitude, which
> is the part that matters. The rest of this item stands unchanged: there is
> still no consumer to weigh it against, and §13 still says the importer
> decides. What has changed is that the trade is now known to be a bad one
> rather than assumed to be a cheap one.

---

