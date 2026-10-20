## 261. CLAUDE.md split by role — 2026-09-26

*Queue row Q141. `CLAUDE.md` is loaded into every session and every subagent —
lanes, verifiers and lookups included — so every line in it is paid for on every
call of every role. This section says what stayed, what moved to where only one
role loads it, and what was cut because a section or `ci.yml` already records
it.*

  261.1   What was measured
  261.2   Nested CLAUDE.md files do not reach a subagent
  261.3   What went where
  261.4   Every paragraph of the old file
  261.5   What no section recorded
  261.6   What this leaves

### 261.1 What was measured

| | bytes | command |
|---|---|---|
| `CLAUDE.md` at base `cc74200d` | 35 272 | `git show cc74200d:CLAUDE.md \| wc -c` |
| `CLAUDE.md` after | 14 349 | `wc -c CLAUDE.md` |
| of which "Top-level sessions only" | 993 | `awk '/^## Top-level sessions only/{f=1} f' CLAUDE.md \| wc -c` |
| `dispatch-lanes` skill at base | 8 024 | `git show cc74200d:.claude/skills/dispatch-lanes/SKILL.md \| wc -c` |
| `dispatch-lanes` skill after | 17 875 | `wc -c .claude/skills/dispatch-lanes/SKILL.md` |
| `.claude/agents/lane.md` | 678 | `wc -c .claude/agents/lane.md` |
| `.claude/agents/verifier.md` | 520 | `wc -c .claude/agents/verifier.md` |

The row quoted a split of the 35K by one reading — about 13K a lane needs, 12K
for the dispatcher alone, 10K history — and asked for it to be redone paragraph
by paragraph. Done over the 90 paragraphs of 261.4, each counted whole under
the role its main part now serves (a script in the lane's `target/`, which is
not kept; the table below is its input):

| where the paragraph's main part went | bytes |
|---|---|
| every role — stays in `CLAUDE.md` | 15 071 |
| top-level sessions only — stays, in its own section | 322 |
| the dispatcher — `dispatch-lanes` skill | 11 222 |
| history, cut: a section or `ci.yml` records it | 7 571 |
| a copy of the `asking-questions` skill, cut | 997 |

The dispatcher's share came out near the row's 12K. The lane's share did not:
what every role needs is about 15K, not 13K, and the row's aim of about 5 000
characters is not reachable without cutting something a lane uses. The core
principles, the design rules, plain language, top to bottom and comments are
already 5 464 of it (P1–P2, P34–P36, P81–P90, summed from the same count). The rest that stays is the gate
and a single test, `just src` and `just recipe`, the decisions files (261.2 is
why they could not go to `docs/`), the editing rules, and "Decide it yourself",
which a lane needs as much as anyone: take the recommendation, end finished or
written down, never be the hands for a refusal.

The dispatcher's start-of-session size the row quotes — 44K to 65K tokens,
measured by rcad-18 — was not re-run here.

### 261.2 Nested CLAUDE.md files do not reach a subagent

The probe, from inside this lane, which is a worktree-isolated subagent: a
marker line written to `docs/CLAUDE.md`, then three `Read`s of files under
`docs/` — `docs/plan.md`, `docs/queue.md` and `docs/decisions.d/00-preamble.md`.
The marker did not arrive in the lane's context after any of them. The file was
then deleted; it never existed at base and is not in the tree.

So the docs-only rules — one file per section, never append to
`docs/decisions.md`, `claim-section`, `LC_ALL=C`, `decisions-find` — stay in
`CLAUDE.md`. Lanes write sections and verifiers read them, so they are
every-role rules in any case.

### 261.3 What went where

- **`CLAUDE.md`** keeps the core principles, "Rules that come from the design",
  "Decide it yourself", the toolchain and `mise exec`, the gate and how to run
  it, `just t`, `just src`, `just recipe`, the decisions files, the editing
  rules, plain language, top to bottom and comments. A last section, "Top-level
  sessions only", holds the hub rule in one paragraph, where role rules now
  live, and a pointer to the `asking-questions` skill.
- **The hub** needs less prose than it had. `.claude/hooks/session-start-hub.sh`
  already tells a session that starts in the hub to call `EnterWorktree`, that
  git cannot lock the hub, and that `just check-hub` fails over work left there.
  What it does not say is that `main` lives in a worktree too, so that sentence
  stays.
- **`.claude/skills/dispatch-lanes/SKILL.md`** gains the queue file's shape and
  owner (intro), `status` and §0 and the branch row (step 1), "take `just lanes`
  whole" (step 2), the agent types to dispatch with (step 5), the §0-in-the-same-commit
  rule (step 8), and after step 8: the recipes the shared skills name, the plan,
  merging and the tree holding `main`, CI and the Actions allowance, the
  plugins, and lookup lanes. Steps 5 to 7 are otherwise untouched, because Q142
  rewrites them.
- **`.claude/agents/lane.md`** holds the one rule the old file addressed to a
  lane alone: never dispatch a lookup lane from inside one.
- **`.claude/agents/verifier.md`** holds no moved rule, because the old file had
  none that only a verifier needed. It exists so step 5 can name the type.
- **Questions** were four paragraphs, and every rule in them is in the
  `asking-questions` skill in the `questions` plugin
  (`plugins/questions/skills/asking-questions/SKILL.md` in the marketplace
  checkout): both places the same turn and why, `just question <hash>`, never
  `!cat`, `just question-tui`, `just question-check` refused, the `## Waiting on
  you` block. The one thing the skill does not say — that the status line
  carries the hash and the open labels — stays in `CLAUDE.md` beside a pointer.
- **What the justfile cites `CLAUDE.md` for stays in `CLAUDE.md`.**
  `check-gate-runs-every-check` fails with "CLAUDE.md says check-plugins runs
  first thing in just gate", so that sentence (P8's first) stays in Tooling
  when the rest of the plugin text left. The justfile's header calls `just t -E
  'test(/walk/)'` an example "this file and CLAUDE.md both give", and
  `decisions-find`'s comment says a line number is what "CLAUDE.md says never to
  address a decision by"; both sentences stay. The same check's other message,
  "CLAUDE.md says gate runs these four and every check-* besides", matched no
  sentence of the old file closely; "`just gate` is the whole gate" stays. No
  gate check reads `CLAUDE.md` itself: `grep -n 'CLAUDE\.md' justfile` finds
  only comments and `echo`s.
- **`AGENTS.md`** is a symlink to `CLAUDE.md` (`ls -la AGENTS.md`), so it
  follows the file with no change.
- **Agent frontmatter** copies `lookup.md`'s shape but leaves out `tools`.
  `lookup.md` lists its tools to make itself read-only; a lane and a verifier
  both edit files, and a list could drop a tool a harness adds, such as a
  workflow's `StructuredOutput`. `lane.md` says `model: inherit` because the row
  sets the model; `verifier.md` says `sonnet`, as step 6 does.

Where two copies of a rule stood in the old file — the CI arms that skip (P9,
P58) and `ff-main` finding its tree by branch (P15, P24) — one copy was kept.
Where the old file restated something the skill already said — the fill rule
and the six-slot cap, `lane:` and `done:`, the verifier's revert, "§0 no longer
lists what is open" — the skill's wording was kept and the old file's cut.

### 261.4 Every paragraph of the old file

Paragraphs are blank-line separated, a fenced block kept whole, numbered over
`git show cc74200d:CLAUDE.md`. "Skill" is `.claude/skills/dispatch-lanes/SKILL.md`;
"cut" says which record holds what left.

Q818 later moved four of those sections out of `SKILL.md` into a new
`.claude/skills/dispatch-lanes/reference.md` beside it: "The plan", "Merging,
and the tree that holds `main`", "CI and the Actions allowance" and "The
plugins". Every row below naming one of those four — skill "The plugins"
(P4–P8, P10–P13), skill "CI" (P9, P49–P51, P53, P58, P60, P62–P64, P66), skill
"The plan" (P20) and skill "Merging" (P24, P38) — points at `reference.md`
now, not at `SKILL.md`; the split moved on, this section did not.

| P | lines | begins | where it lives now |
|---|---|---|---|
| 1 | 1 | `## Core Principles` | `CLAUDE.md`, same heading |
| 2 | 3–8 | principles 1–6 | `CLAUDE.md`, verbatim |
| 3 | 10–12 | Always use the code-quality skill | first sentence `CLAUDE.md` top; "Big work goes to lanes" `CLAUDE.md` "Top-level sessions only" |
| 4 | 14 | `## Three plugins…` | skill "The plugins" |
| 5 | 16–20 | `.claude/` used to hold | cut: §64 records it; skill keeps "shared with `rscene` (§64)" |
| 6 | 22–27 | the four `/plugin` lines | skill "The plugins", verbatim |
| 7 | 29–36 | `questions`, `guardrails`, `practices` | skill "The plugins"; the `question-check` clause cut: §152.4 and the `asking-questions` skill record it |
| 8 | 38–41 | `just check-plugins` fails, first thing | first sentence `CLAUDE.md` Tooling; "A missing `guardrails`" skill "The plugins" |
| 9 | 43–53 | CI has no Claude Code | "skipped, not passed" and "keeps its place at the head of `gate`" merged into skill "CI", the rest duplicates P58; the clone's removal on 2026-09-13 cut: `ci.yml` gate-step comment records it |
| 10 | 55–60 | The version this repo asks for | skill "The plugins", verbatim |
| 11 | 62–63 | The marketplace repo is public | skill "The plugins", verbatim |
| 12 | 65–72 | That it is on GitHub | skill "The plugins", verbatim |
| 13 | 74–80 | What stays here | skill "The plugins", verbatim |
| 14 | 82–84 | A bare `just` lists the recipes | `CLAUDE.md` Tooling, verbatim |
| 15 | 86–99 | Recipes the shared skills refer to | "`just gate` is the whole gate, and what CI runs" `CLAUDE.md` Tooling; the rest skill "Recipes the shared skills name"; "they are reached for now…" cut: §157.3; "called `just default` until 2026-09-08" 261.5 |
| 16 | 101–110 | What replaced it is asking | "Take `just lanes` whole" and the re-ask reason skill step 2; 262 asks, 86 sessions, "mostly irreducible" cut: §157.3; 58% cut: §157.4; "one screen for fourteen worktrees" 261.5; the `kache` sentence `CLAUDE.md` Tooling, verbatim |
| 17 | 112–121 | `just status` answers | the §0 rule skill step 1; the list of what `status` prints was already step 1's; "twelve tool calls" cut: §102.1 |
| 18 | 123–130 | The §0 update goes in the same commit | skill step 8; "5 of 6", "27 of 44" cut: §172.5 |
| 19 | 132–135 | Do not read the branch row | skill step 1; the five-branch example cut: §102.4 |
| 20 | 137–144 | `just plan` is the roadmap | skill "The plan", verbatim |
| 21 | 146–155 | `just queue` is what to dispatch next | row shape, tiers, "the dispatcher writes the file", `check-queue` skill intro; fill rule and cap already in skill intro (§230.3, §230.8); `lane:`/`done:`/new rows already in steps 5 and 7; "§0 no longer lists" already in step 1 |
| 22 | 157 | `## Nobody works in the hub` | `CLAUDE.md` "Top-level sessions only" |
| 23 | 159–162 | …rcad is the hub | `CLAUDE.md` "Top-level sessions only", verbatim, plus the §60 pointer |
| 24 | 164–171 | That worktree is `rcad-worktrees/main` | both names and "Do not type either name" skill "Merging"; the rename and the sections left spelling `wave-1` cut: §210.1; "every recipe finds it by the branch" duplicates P15's `ff-main` sentence |
| 25 | 173–179 | This is not tidiness | cut: §60 and §60.2 record it |
| 26 | 181–185 | `main` leaving the hub | cut: §60.3 |
| 27 | 187–189 | `just check-hub` fails | cut: §60.6 records the one-worktree exemption; `session-start-hub.sh` says it at session start; skill "Recipes" keeps "`just check-hub` is the hub rule" |
| 28 | 191–200 | A hub nobody works in goes stale | cut: §146 records it (§146.3, §146.4) |
| 29 | 202–207 | The decisions are one file per section | `CLAUDE.md` "The decisions"; "the way you always did" and "every recipe survived the split" cut: §179 |
| 30 | 209–213 | A new section is a new file | `CLAUDE.md` "The decisions"; "four sessions' habit" cut: §179 |
| 31 | 215–221 | Never sort those filenames | `CLAUDE.md` "The decisions"; "785 lines" and "§173's shape" cut: §179.3 |
| 32 | 223–228 | Search with `just decisions-find` | `CLAUDE.md` "The decisions"; "which of 179 files" 261.5 |
| 33 | 230–235 | Take a new section's number | `CLAUDE.md` "The decisions"; the lost race cut: §159 |
| 34 | 237 | `## Rules that come from the design` | `CLAUDE.md`, same heading |
| 35 | 239 | These are the ones | `CLAUDE.md`, verbatim |
| 36 | 241–278 | the twelve design rules | `CLAUDE.md`, verbatim |
| 37 | 280 | `## Decide it yourself, and land it` | `CLAUDE.md` "Decide it yourself"; landing went to the skill |
| 38 | 282–287 | A green `just gate` on the merged tree | the rule, "Not a sentence from Mattias" and "Gate the tree you actually merged" skill "Merging"; "never write down a gate you have not read" `CLAUDE.md` Tooling; the reasons cut: §138 |
| 39 | 289–294 | take the recommendation | `CLAUDE.md`, verbatim |
| 40 | 296–298 | End finished, or end written down | `CLAUDE.md`, verbatim |
| 41 | 300–302 | Still not yours to take | `CLAUDE.md`, verbatim |
| 42 | 304 | `### The questions that are still questions` | `CLAUDE.md` "Top-level sessions only" |
| 43 | 306–309 | Every question goes in `open-questions.md` | cut: the `asking-questions` skill says it, and why the file exists |
| 44 | 311–314 | Read it back with `just question <hash>` | the status-line sentence `CLAUDE.md` "Top-level sessions only"; the rest cut: the `asking-questions` skill says it; §100 pointer kept |
| 45 | 316–317 | End any turn … `## Waiting on you` | cut: the `asking-questions` skill says it |
| 46 | 319–323 | The grammar, how a round is written | pointer kept in `CLAUDE.md` "Top-level sessions only"; the stale `rscene` copy cut: §64 |
| 47 | 325 | `## Tooling` | `CLAUDE.md`, same heading |
| 48 | 327–329 | `rust-toolchain.toml` pins | `CLAUDE.md` Tooling, verbatim |
| 49 | 331–333 | CI runs `just gate` | skill "CI"; "since 2026-09-07", "four recipes" cut: `ci.yml` header records it |
| 50 | 335–341 | Since 2026-09-20 there are two arms | skill "CI"; the date cut: §172; 1m07s and 9m46s cut: §172.1, §172.2 |
| 51 | 343–348 | This is because the runner is metered | skill "CI"; "it ran out", 171 runs, 2271 minutes cut: §172 |
| 52 | 350–352 | `just gate` is still what you run locally | `CLAUDE.md` Tooling, verbatim |
| 53 | 354–360 | `just gate-fresh` | skill "CI"; "found all three of the red runs" cut: §77.6, §173.6 |
| 54 | 362–367 | On its first working run it found a fourth | cut: §173 records it |
| 55 | 369–378 | Run it as `just gate quiet` | `CLAUDE.md` Tooling; "262 of 272", "122" cut: §147.1 |
| 56 | 380–381 | Never pipe it through `tail` | `CLAUDE.md` Tooling, verbatim |
| 57 | 383–392 | Never write a wait loop | `CLAUDE.md` Tooling; 56 runs, 98 s, 244 s, 171 runs cut: §147.4 |
| 58 | 394–402 | Three arms are not the same there | skill "CI", with P9's two phrases |
| 59 | 404–406 | It was one arm until 2026-09-13 | cut: `ci.yml` gate-step comment records it |
| 60 | 408–411 | `34266469454` is the first whole-gate green | skill "CI"; "still called `default`" cut: §77.5 |
| 61 | 413–418 | Three red runs came first | cut: §77.6 |
| 62 | 420–423 | CI is green, and the commit | skill "CI"; "the empty answer that command used to give" cut: `ci.yml` header records it |
| 63 | 425–432 | Read that answer as a list | the rule and "a named run does not go stale" skill "CI"; "26 of 26" and the three named runs cut: `ci.yml` header and §77.5 |
| 64 | 434–437 | The other green is a pull request build | "Say which commit" skill "CI"; the pull request story cut: `ci.yml` header records it |
| 65 | 439–441 | Three faults kept every earlier run red | cut: `ci.yml` header records all three |
| 66 | 443–445 | So there is a second opinion now | the rule skill "CI"; "agreed the first time it could" cut: §77.5 |
| 67 | 447–453 | `just t <args>` | `CLAUDE.md` Tooling, verbatim |
| 68 | 455–460 | `just src <name>` | `CLAUDE.md` Tooling; the `arrange.rs` figures cut: §57.7 |
| 69 | 462–472 | `just recipe <name>` | `CLAUDE.md` Tooling; why `--show` falls short cut: §153.1; 307 404 and 2 934 cut: §153.2 |
| 70 | 474–482 | If you cannot name the file | skill "Hunting"; "a third of a week's bill … a sixth" cut: §158 |
| 71 | 484–494 | "Past two probes" was the rule | "All the value is in … the first probe", "Never send a lane for…", `just spend` skill "Hunting"; "never dispatch one from inside a lane" `.claude/agents/lane.md`; the prices and the old trigger cut: §158.4, §158.6 |
| 72 | 496–501 | A call is billed the context | the rule and "print the whole thing" `CLAUDE.md` Editing; what it explains and the 2 200 / 73 arithmetic cut: §100.2 |
| 73 | 503 | `## Editing files` | `CLAUDE.md`, same heading |
| 74 | 505–510 | `Read` first, then `Edit` | `CLAUDE.md`, verbatim |
| 75 | 512–514 | This is a cost rule | `CLAUDE.md`; "145k tokens and ~70 minutes" cut: §50.8 |
| 76 | 516–517 | If `Edit` refuses | `CLAUDE.md`, verbatim |
| 77 | 519–520 | Bash keeps what Bash is for | `CLAUDE.md`, verbatim |
| 78 | 522–526 | Independent read-only probes go in one call | `CLAUDE.md`; "129 000 … 300 000" 261.5 |
| 79 | 528–536 | Most of the gap this rule used to quote | "The dependent pairs are a tool problem…" `CLAUDE.md` Editing; the rest cut: §144.7, §157.2 |
| 80 | 538–542 | Claude Code sometimes injects | `CLAUDE.md`, verbatim |
| 81–84 | 544–558 | plain language, with its example | `CLAUDE.md`, verbatim |
| 85–88 | 560–573 | code flows from top to bottom | `CLAUDE.md`, verbatim |
| 89–90 | 575–582 | comments only when necessary | `CLAUDE.md`, verbatim |

### 261.5 What no section recorded

These left `CLAUDE.md` and were found in no section — searched with `just
decisions-find` and with `grep -l` over `docs/decisions.d/` — so they are
written down here:

- **`just gate` was called `just default` until 2026-09-08**, so an older
  document or a session's memory may still say that. The justfile's comment
  above `check-skills` says the same.
- **`just lanes` was one screen for fourteen worktrees** when "take it whole"
  was written.
- **The `decisions-find` rule got stronger with the split**: a grep now also has
  to tell you which of 179 files it landed in.
- **A call bills the whole conversation again — a median of 129 000 tokens,
  and past 300 000 late in a session.** `.claude/agents/lookup.md` quotes the
  same two figures.

### 261.6 What this leaves

- `CLAUDE.md` is 14 349 bytes, not about 5 000. 261.1 is the argument: what
  every role needs is about 15K by the paragraph count, and nothing was left
  that a lane or a verifier does not use.
- **A lane dispatched through the `lanes` workflow does not load
  `.claude/agents/lane.md`**, because `lanes.js` sends its lanes and verifiers
  without an agent type. Its `COMMON` and `VERIFY` blocks are today's real lane
  and verifier rules. Q142 owns `lanes.js`; if it moves those blocks into the
  two agent files, it should say so there and cut them from the script.
- Step 6's description of the verifier's job and `lanes.js`'s `VERIFY` block say
  the same thing twice. Q142 rewrites step 6 and can keep one.
- `.claude/agents/lookup.md` repeats `just src`, `just recipe`, `just
  decision`, `decisions-find` and the 129 000-token figure from `CLAUDE.md`. A
  lookup loads both, so its copies are redundant; it was outside this row's
  write set.
- `.github/workflows/ci.yml` line 120 says a run tested "CLAUDE.md's exemption
  for a repository with one worktree". That exemption is §60.6 and no longer
  in `CLAUDE.md`. The comment is a record of a past run, true when written, so
  it was left alone.
