## 175. `/dispatch-lanes`, and the check that keeps the commands a skill names true — 2026-09-20

*§170 made the lanes workflow a file instead of a retyped script. What was still
retyped was the paragraph that drives it: pin a base, watch it go green, build
the write-set table, call the workflow by `meta.name`. That paragraph is now
`.claude/skills/dispatch-lanes/SKILL.md`, triggered by name. It stays in this
repo rather than joining the nine shared skills, because it names `lanes.js`,
`just claim-section` and §0, none of which `rscene` has. A skill is prose naming
commands, and prose naming commands rots — `just check-skills` holds every
recipe and path it names against the tree.*

### 175.1 Why it is not in `practices`

§64.4 draws the line and it draws it here without needing a new argument: a repo
keeps what is its rules and nobody else's. Every load-bearing name in this skill
is this repo's — `.claude/workflows/lanes.js`, `just status`, `just plan open`,
`just claim-section`, §0's "Next and unclaimed", the §0-in-the-same-commit rule.
Moving it to the marketplace would give `rscene` a skill whose every command
fails there, which is the inverse of what §64 was for.

The `delegating` skill stays where it is and this one does not replace it.
`delegating` is the argument — write-set tables, three at once, the 45 minutes,
why a lane's own numbers are not evidence — and is true of any repo with lanes.
This is the button, and it is true only of this one. §64.7 is the warning
against the other arrangement: the shared skill that was wrong in both repos at
once, because it was carrying what only one of them knew.

### 175.2 Only what a skill wrote in backticks is a claim

`check-skills` reads code spans — fenced blocks and inline backticks — and
nothing else. "just" is an ordinary English word, and a check that took every
occurrence of it would have read "you can just read it first" as naming a recipe
called `read` and failed on a sentence that claims nothing. The rule is the one
the reader already follows, which is the point: the check and the reader cannot
disagree about which words were meant literally.

Paths are held to the same standard from the other side. A path is checked when
it has a slash, an extension, and no `<`, so `crates/<c>/src` is left alone — a
placeholder is not a claim about the disk.

Only **public** recipes are known, because `just --summary` is what lists them.
A skill naming a `[private]` recipe therefore fails, and that is the right
answer rather than a limitation: the private ones are the selftests, and a skill
has no business sending a session into one.

The vacuity guard is the part that matters most and it is four lines. Both sides
of this comparison are extracted from the same files, so an extraction that
stops matching compares nothing and an empty result prints nothing — which reads
exactly like success. `check-ignored` and `check-gate-runs-every-check` both say
this about themselves, and §173 is the session that watched a container's mawk
do it for real. So a scan that finds no code span in any `SKILL.md` fails rather
than passes, on the ground that a skill with no backticks and no fenced block is
not how any of them are written.

### 175.3 What it was checked with, and what is not checked

`check-skills-selftest` runs the scan over fixtures: a file naming only real
things passes, one naming `just no-such-recipe` fails, one naming
`docs/no-such-file.md` fails, one with no code spans fails, and a missing
directory is reported as no skills rather than as a broken scan. The passing
fixture carries the English "just" and the `<`-placeholder deliberately, so that
either rule breaking fails in the selftest and not later in a lane's brief.

The real skill was then broken by hand — `claim-section` to `claim-sektion` —
and the check named the file, the command and the reason, which is the mutation
§77.6 asks for rather than a green run.

Two things are not checked, and neither is worth a recipe yet.

- **Whether the skill's advice is still right.** It can only be held to names
  that resolve. That §0 is where the unit lives, that a green base matters, that
  three or four writing lanes is the batch — those are claims about judgement,
  and `check-where-the-work-is` already says the same about §0 itself.
- **Whether the skill resolves by name.** Claude Code registers project skills
  at session start, the same memoisation §170.2 found for the workflow registry,
  so the session that writes one cannot see it. The first dispatch is the test,
  and it costs a session to run — the same trade §170.5 took for `lanes.js`.

Write set: this section, `.claude/skills/dispatch-lanes/SKILL.md`, and
`justfile` (`check-skills`, `_skills-check`, `check-skills-selftest`,
`gate_steps`, and the pinned count in `check-gate-runs-every-check`, 39 to 41).

