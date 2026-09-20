## 178. §0's cap is checked at gate time, and the write already knows — 2026-09-20

*Mattias asked whether the 50-line cap on §0 is efficient as it stands. It is
not, and the waste is not the trimming. Over 2026-09-17 to 2026-09-20 the cap
refused **21 separate trims, 34 rounds between them**, and **11 of the 21 were
found by a whole `just gate`** rather than by the check that costs a second.
The cap itself is right and §102.3's argument for it stands; what is wrong is
that it speaks at the end of the work instead of at the write. §152 already
solved this exact shape for `open-questions.md` — the write runs the checker, so
the round trip disappears — and §0 is the same file-shaped property asked at the
same moment. One change has landed already; the hook is a lane.*

### 178.1 Twenty-one trims in four days, and the cap is not catching mistakes

Every `rcad` transcript, every tool result carrying the refusal, deduplicated by
record and with this session's own reads of them excluded:

- **34 refusals in 19 sessions**, all between 2026-09-17 and 2026-09-20.
- They group into **21 trim episodes** by session and prompt. §0 was edited in
  roughly the same number of commits over those four days, so **the cap fires on
  almost every edit of §0**. A check that refuses most of the writes to a
  section is not catching mistakes; it is doing the compressing, one refusal at
  a time.
- **8 of the 21 needed two to four rounds.** One went 67 → 56 → 54 → 51, four
  refusals in 1.7 minutes; another 60 → 53 → 51; another 54 → 52 → 51. A trim by
  eye aims short.
- **11 of the 21 arrived from `just gate`**, 9 from the direct check, 1 from a
  saved gate log. The gate-found share rises across the four days: every episode
  from 2026-09-19 17:00 onward but one began at a gate. That is the expensive
  route — a 98-second median run and a whole-context call to be told you are
  three lines over.
- `2b15f320` is a commit whose entire content is a trim: *"§0 back under its
  50-line cap, by compressing what closed"*.
- **§0 stands at exactly 50 today.** It is saturated, and both of its accreting
  lists — what closed, what is deferred — only grow. Every ordinary edit to it
  is therefore over the cap before it starts.

### 178.2 What landed first: the refusal now says how many lines to cut

`e63d357e`. The refusal named the count and left the subtraction, and the choice
of how brave to be, to the reader:

    §0 is 53 non-blank lines; the cap is 50, because a summary nobody re-reads is not one
    ^ cut 3 non-blank lines, in one pass: trimming by eye takes two to four (§102.3)

Pinned by a fixture in `check-decisions-selftest` that is exactly three lines
over and asserts `cut 3`; the pre-change recipe prints no such line, run side by
side rather than argued. This is aimed at the 13 extra rounds, not at the 11
gate discoveries — those are the hook's.

### 178.3 The hook is §152's, pointed at a different file

§152's `check-question-file.sh` is a **PostToolUse** hook on `Write|Edit` that
fires when the written path ends `open-questions.md`, runs the checker the write
would otherwise be asked about later, and reports in the same turn. §0's three
properties — the cap, a `reviewed:` date `date -d` can read, and citations that
resolve — are exactly as file-shaped and exactly as cheap to ask.

`_where-the-work-is-check` is already held apart from its recipe so something
other than the gate can run it over a named file, which is the whole of what the
hook needs. **The hook runs that recipe rather than reimplementing its awk**: two
copies of a rule kept in step by a comment is the drift this repo keeps ending,
and §173 is what it costs when one copy reads the environment differently.

**Two ways it can fail silently, both already paid for elsewhere.**

- **A hook that cannot find what it runs looks exactly like a clean file.**
  `check-question-file.sh` reports a missing checker on stderr with exit 2
  rather than swallowing it, and says in its own comment why. The same applies
  here to a `just` that is not on PATH or a recipe that has been renamed.
- **A hook that says nothing when all is well looks exactly like a hook that is
  not installed.** §152 measured that directly: 148 hand `question-check` calls
  were made *after* the hook landed, three of which found anything. So a clean
  §0 must say so, in one line, through `additionalContext` — a PostToolUse
  hook's plain stdout goes to the debug log and the model never sees it, which
  is the same silence wearing a different name.

### 178.4 What it says when §0 is clean is the headroom, not "ok"

The one line should carry the number the next edit needs:

    §0 is 47 of 50 non-blank lines, reviewed 2026-09-20, every section it cites exists.

Headroom is what turns the cap from a refusal into a budget. A session that
knows it has three lines left writes the trim into the edit it is already
making; a session that knows only "clean" writes 55 lines and finds out at the
gate. That is the whole of the 11.

### 178.5 The unit stays lines, and the wrap is named rather than fixed

The cap counts hard-wrapped non-blank lines, and **nothing in the gate enforces
a wrap width**. §0 today is 3 711 non-blank characters and 604 words over 50
lines — a mean of 74 columns. Rewrapped at 100 columns it is about 38 lines and
passes with nothing cut at all. So the bound the check enforces is a property of
the wrapping, not of the content, and "about a screen" holds only because
`decisions.md` is hand-wrapped at roughly 79 by habit.

**Left alone deliberately.** Nobody has reached for that in 34 refusals, and
closing it properly means a character or word cap *plus* a written wrap rule
across 40 000 lines — a rule change for a hole that has never been fallen
through. It is written down here so the next reader who spots it knows it was
seen. If it is ever exercised, the fix is a character cap of about 3 800 with a
wrap check beside it, not a character cap alone.

### 178.6 The lane

One lane, Sonnet, off a base that is pushed, so `isolation: "worktree"` reaches
it. Write set: `.claude/hooks/decisions-zero.sh` and
`.claude/hooks/check-decisions-zero.sh`, both new; `.claude/settings.json`, the
`PostToolUse` entry only; the `check-hooks` recipe in `justfile`; and this
section's `178.7`. It writes no crate, so it declares no source paths and its
verifier has nothing to revert.

The write set overlaps `justfile` and `docs/decisions.md`, which §177 has just
measured as the two files carrying every merge conflict this repo has had, so
**it runs alone or beside lanes that write neither**.

Done when `mise exec -- just gate quiet` is green — `check-hooks` is already in
`gate_steps`, so the new selftest is gated by being called from it — and when
the lane has quoted the hook's own output from a real `Edit` of
`docs/decisions.md` in its own session, clean case and over-cap case both.
A hook registered in `settings.json` mid-session may not load until the session
restarts; if it does not, that is reported as the limit it is, and the selftest
carries the evidence instead.

Two things for the lane to measure rather than assume:

- **What the hook costs per write.** It shells out to `just`, and a bare `just`
  in this repo evaluates every top-level assignment first — `hub_aligned` spends
  three git calls, `RCAD_STEP_CORPUS` another (§146). If that is over about half
  a second, report it; do not quietly inline the awk to get under it.
- **Whether it should speak on a write to `decisions.md` that does not touch
  §0.** Always running is the simpler rule and keeps one code path; the risk is
  a line of noise on every section a session writes. Whichever way it goes, the
  selftest pins it.

### 178.7 The hook landed, matched narrowly, and did not fire this session

`.claude/hooks/decisions-zero.sh` matches only
`docs/decisions.d/000-where-the-work-is.md` — §0's own file, after §179 moved
it out of `docs/decisions.md`, which is now a 17-line pointer carrying no
sections at all. It runs `just check-where-the-work-is` rather than a second
copy of `_where-the-work-is-check`'s awk: that recipe already builds the whole
`decisions-all` concatenation, which the citation arm needs, and copying it
into the hook is the drift §178.3 exists to stop.

**Narrow, not wide.** §178.6 asked whether the hook should also speak on a
write to any other file under `docs/decisions.d/`, since a heading rename
anywhere can dangle a citation §0 makes. Kept to §0's own file: the round trip
this hook removes is the one §178.1 measured, 21 trim episodes on edits to §0
itself, 11 found only at `just gate`. A rename elsewhere breaking one of §0's
citations is a different and rarer failure — `just gate` already catches it on
the commit that made the rename — and a wide matcher would speak on every
section a session writes or closes, which §179's own split did 179 times over.
`check-decisions-zero.sh` pins the narrow match: a write to
`docs/decisions.d/050-something.md` is silent, exit 0.

**Measured cost.** Five runs of the hook against the real §0 file, timed with
`date +%s.%N` around each call: 0.394s, 0.429s, 0.410s, 0.408s, 0.406s — 0.39
to 0.43s, under the "about half a second" `check-hooks` asked to be told
about. Almost all of it is `just` evaluating its top-level assignments before
any recipe runs — `hub_aligned`'s git calls, the plugin-root lookups — the
same cost §146 already named; `check-where-the-work-is` alone was clocked at
0.21s in `gate`'s own comment. Left as a shell-out rather than inlined, per
§178.3 and §178.6's own instruction not to chase this number by duplicating
the awk.

**The literal clean line**, from a real `Edit` of the real §0 file in this
session — it happens to be exactly saturated today, so this is the file's true
state and not a fixture:

    §0 is 50 of 50 non-blank lines, reviewed 2026-09-20, every section it cites exists.

**The literal block**, from the same file with three lines added by hand and
reverted before committing (`git checkout -- docs/decisions.d/000-where-the-work-is.md`):

    §0 is 53 non-blank lines; the cap is 50, because a summary nobody re-reads is not one
    ^ cut 3 non-blank lines, in one pass: trimming by eye takes two to four (§102.3)

**Mid-session registration did not take effect.** After the `PostToolUse`
entry was added to `.claude/settings.json`, a real `Edit` of §0 later in this
same session produced no `additionalContext` and no block — the tool result
was the plain "file has been updated successfully," nothing from the hook.
Both lines above came from piping the payload a `PostToolUse` call would send
straight into `decisions-zero.sh`, not from Claude Code invoking it.
`check-decisions-zero.sh` is what proves the hook's behaviour unconditionally
and on CI, rather than resting on a session restart nobody can force to order.

`just gate` now prints the headroom line too, on every clean run:
`_where-the-work-is-check` emits it once, and both the hook and `gate` read it
from the same place rather than the hook computing it a second time. This
lane's own green run carries `§0 is 50 of 50 non-blank lines, reviewed
2026-09-20, every section it cites exists.` at line 9 of
`target/last-gate.log`. That is deliberate, not a leak: the whole point of
turning "clean" into a count was that `gate` should not know less than the
write does.
