## 177. Two files carry every merge conflict this repo has ever had

Every two-parent merge reachable from `main` — 331 of them — was replayed with
`git merge-tree` against its own recorded parents, so what follows is what each
merger actually faced and not a reconstruction. **53 conflicted, and
`docs/decisions.md` is in 50 of them.** The justfile is in 6. The remaining six
files are one conflict each: `.gitignore`, `crates/rcad/src/lib.rs`, a
`.proptest-regressions` file, the two `tooling/session-spend/*.py`, and a
modify/delete on `.claude/skills/testing/SKILL.md` from the plugin move.

**Only 15 distinct files have ever been edited on both sides of a merge at
all.** Lane file-ownership is doing its job: the whole conflict bill sits in the
two files no lane can own, because every lane has to write to them.

**This section's own commit is the fifty-first.** It was written on a branch off
`7243d894`, `main` moved to `e63d357e` while it was being written, and §176 had
appended to the same tail — so merging it conflicted in exactly the place and
for exactly the reason measured below. The justfile merged clean, because §176
touched `check-where-the-work-is` and this touched `gate_steps`; had both
touched the step list, the one-line form would have conflicted there too.

### 177.1 It is the position, not the prose

47 of the 50 `docs/decisions.md` conflicts land between 93% and 99% of the way
through the file. Nobody is disagreeing about a section — two lanes are
appending different sections to the same tail. `claim-section` (§159) already
stops them taking the same *number*; nothing stops them taking the same *lines*.

One hunk landed at 0% instead, and that one is going to grow: §172.5 requires
the §0 update to ride in the same commit as the section that closes it, so every
closing branch now writes to the top of the file and to the bottom.

The justfile's hunks cluster on the top-of-file preamble, on the tail where new
recipes land, and on the gate registry — `2da15de4` on 2026-09-16 conflicted on
the `gate_steps` line itself, both sides.

### 177.2 What it costs at more lanes than three

Exposure is the share of the 331 incoming branch sides that touch a file; the
chance a batch of `k` lanes contains a pair that both touch it runs over all
`C(k,2)` pairs. Multiplying by the measured conflict-given-co-edit rate — 50/74
for the decisions, 6/26 for the justfile — gives the last column.

| file | exposure | k=3 | k=6 | k=10 | conflict at k=6 |
| --- | --- | --- | --- | --- | --- |
| `docs/decisions.md` | .390 | 39% | 92% | >99% | **62%** |
| `justfile` | .236 | 16% | 58% | 92% | **13%** |
| `crates/rcad-topo/src/validate.rs` | .115 | 4% | 18% | 45% | 8% |
| `crates/rcad-topo/src/mesh.rs` | .109 | 4% | 16% | 42% | 7% |
| `CLAUDE.md` | .100 | 3% | 14% | 36% | 0% measured, 0 of 5 |
| `fuse/arrange.rs`, `geom/walk.rs` | .091 | 2% | 12% | 31% | 5% |

At three lanes the decisions file conflicts on about two batches in five, which
is the observed 53 of 331. At six it is every batch. **Nothing in `crates/` comes
close**, and the first source file that would matter is `validate.rs` past six
lanes — so the answer to "can we run more in parallel" is about these two files
and nothing else.

`docs/consumers.md` is the near miss worth naming: 17 lanes touched it and never
once concurrently. That is sequencing luck, not structure, and it is the next
file to check if the shape of the work changes.

### 177.3 `gate_steps` is one step per line now, and this commit does that half

43 step names on one line is the worst merge shape available: two lanes each
adding a check rewrite the whole line, so git has nothing to take from both.
It is now one name per source line, joined with `\` into the same single-line
value — `just --evaluate gate_steps` gives the same 45 steps in the same order
as the committed one-line declaration did, compared word for word.

**Continuation rather than a `"""` indented string**, which was tried first and
rejected on evidence: that form puts real newlines in the *value*, and
`for step in {{ gate_steps }}` is then a bash syntax error at the second name.
The continuation form leaves the value byte-identical, so every consumer —
`gate`, `gate-docs`, `_record-green`, the `--set gate_steps` overrides in three
selftests — is untouched.

Two readers did break, both loudly, and both were greps anchored at `^` that
read the first source line and stopped: `check-gate-docs-selftest`'s extraction
(caught by its own "parsed to nothing" guard, which is why that guard exists)
and `check-gate-runs-every-check`. Both now take the value from `just` by
interpolation instead of re-parsing the justfile's source, which is a thing that
cannot silently empty. The `defined` side of `check-gate-runs-every-check` is
still a grep over the file, and the pinned count of 41 still guards it — the two
sides stay independent, which was §77.6's point.

**A new arm holds the shape**: the declaration's source-line count must equal
its step count. Checked by mutation, not by running green — joining `lint` and
`test` onto one line gives "45 steps written on 44 source lines" and exit 1.

`gate_metered_steps` is deliberately left on one line. It gains a step only when
a *metered* step is added, which its own comment explains is rare by design —
a new `check-*` joins the docs arm by default and is never named there.

### 177.4 The decisions file becomes a directory, ported from rscene

rscene hit this first and fixed it in its own §39: `docs/decisions.d/`, one
section per file named `N-slug.md`, with `docs/decisions.md` kept as a short
pointer because the citation spelling `docs/decisions.md §N` is what the whole
tree already uses and the number is the address either way. Two lanes writing
`177-*.md` and `178-*.md` do not touch. The collision that remains — two lanes
taking one number — is `check-decisions`' job rather than git's, and here
`claim-section` already prevents it.

That design is taken as-is rather than re-argued. **What does not carry over is
subsections**: rscene's `###` headings are unnumbered free text, so its
`decision` recipe prints whole sections only. This file numbers every `###` as
`N.M` and `just decision 173.10` is how it is read, so that addressing has to
keep working against the one file the section now lives in.

rscene also proved the move lost nothing in the only way worth trusting:
`decisions-all` concatenated in number order was byte-identical to the file it
replaced, diffed rather than read. The same proof is required here, against
40 578 lines.

Not done in this commit — it is a lane, because it touches `decisions_file`,
`decision`, `decisions-toc`, `decisions-find`, `check-decisions`,
`check-decisions-selftest`, `check-where-the-work-is` and `claim-section`, all
of which read the single file today.

**A union merge driver was considered and refused.** `merge=union` on the
decisions file would take both appends with no conflict, and `claim-section`
makes the numbers distinct, so it looks like it fits. It does not order what it
takes, and silently interleaved prose in this file is worth more than the
conflicts are costing. §1 — crash over corrupt.

Write set: this section and `justfile` (`gate_steps`,
`check-gate-runs-every-check`, `check-gate-docs-selftest`).

