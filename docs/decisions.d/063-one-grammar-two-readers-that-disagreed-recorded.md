## 63. One grammar, two readers that disagreed, recorded 2026-09-07

§59 gave `open-questions.md` an address and left its grammar written twice: once
in the `question` recipe and once in `statusline.sh`, kept in step by a comment
in the justfile asserting that they matched. They did not, and §59.2 is the entry
where that cost two open questions. This section is the fix — one parser, a
grammar strict enough to be checked, and four bugs the work turned up — together
with what a census of every question file said, which corrected two of the
recommendations that led to it.

It is the first of three stages. The other two are named in §63.7 and neither is
built.

### 63.1 The comment was the only thing holding them together

Two implementations of one grammar have no single answer to "is this entry
open". The justfile said so out loud — *it reads labels exactly as
`statusline.sh` does* — which is a claim about two pieces of code that nothing
executed. §15.6 wants a test for a published bound; the same rule applied here
would have wanted one for a published *equality*, and there was none.

`.claude/questions.py` is now the answer and both callers shell into it. The
grammar cannot disagree with itself because there is one of it. That is the whole
of the design; everything below is what the rewrite exposed.

### 63.2 What 227 entries said, and what it corrected

The corpus at the time of writing: **30 files, 227 entries, 111 answered and 116
open**, across `rcad`, its worktrees and `rscene`. Two findings changed decisions
that had already been argued for on other grounds.

**Options are rare and were never marked.** Seventeen entries of 227 offer
lettered options at all, and **not one** wrote them as their own lines — eleven
inline only, six inline with a marked line beside them. The complaint that
started this work was that finding what `(a)` meant took reading the paragraph it
was buried in, and that is not a rendering problem: nothing can lift out of prose
what was never marked. But the case for the rule was overstated in the asking as
"the whole reason", when it governs 7.5% of entries.

**An answer almost never names a letter.** Four of 111 answered entries do. Free
text is not the escape hatch beside the letter, it is the normal case, which
inverts what a sidecar of answers (§63.7) should treat as its primary field.

Two more, both of which became reader bugs rather than format proposals: **all 30
files open with an H1** and eight use `##` sections, which no reader knew about;
and **the recommendation is not last** — of the 117 entries carrying one, 44 have
content after it, and 110 entries carry none at all. `➡️` and `→ *recommend`
are both live, 46 to 68. The grammar takes `➡️`, and the reader takes both
forever, because rewriting 68 entries buys nothing.

### 63.3 A `- [ ]` line is never dropped

The awk skipped any entry whose label it could not parse. Both readers did, so
the count and the listing agreed with each other and were both wrong. Two such
entries were live in the corpus — `**handoff Q1**`, whose label does not match
the pattern at all, and `**Q1–Q10**`, an en-dash range that yields `Q1` and
silently discards nine.

This is §59.2's failure mode and not a new one: *the line looked complete and was
not*. The rule that replaces it is absolute — **anything starting `- [ ]` is
reported**, as `??` with its line number in the listing and as `??×n` in the
status line, and counted as open. A grammar whose violations are invisible is not
enforced; it is described.

Two more entry-ending bugs, each with a case in `check-questions-selftest`:

- **A fenced example truncated the body.** Two of the three read paths tracked
  fences and the body extractor did not. The format is documented *as* a fenced
  entry, so an entry quoting the format broke the reader meant to show it.
- **A `##` heading was swallowed** into the entry above it, in ten entries. The
  file that first proposed this grammar became the eleventh within a turn of
  being written, which is how it was found.

### 63.4 No list of grandfathered sessions

The proposal on the table was a file naming the sessions written before the
grammar, exempt from it. It is not built, and the reason is the one §60 used for
the hub: **do not invent a second record beside the thing itself.**

A list of session ids also outlives what it names. `/usr/lib/tmpfiles.d/tmp.conf`
sweeps `/tmp` at `30d`, so every id in such a list dangles within a month — a
file that grows forever and describes nothing.

What replaces it is scope, not exemption: `just question-check` takes a session
hash and judges one file, which is the caller's own. A dead session's file cannot
be fixed, so judging it produces noise nobody can act on. A live session's can,
so it is judged — including a session older than the grammar, which is right,
because it is the one case that can still comply.

### 63.5 Python, and it was faster than what it replaced

`mise.toml` pins only `cargo:` tools and CLAUDE.md says never install a tool ad
hoc. This installs nothing: stdlib only, one file, system `python3`.

The expected objection was cost on a status line that redraws every five seconds.
Measured, it goes the other way. `just question` with no arguments took **368 ms**
because it spawned one `awk` per file over 30 files; one Python process over the
same 30 is **19 ms**. `statusline.sh` went from **27.8 ms to 15.6 ms**, because it
forked `awk`, `stat`, `date` and `printf` repeatedly and a fork costs more than an
interpreter start.

The claim that a scripting language is the slow choice was true of the *language*
and silent about the *process count*, which is §62's shape in a smaller key.

### 63.6 The checker's two false positives, both found by running it here

Written, then run against the file of the session that wrote it. Both times it
was wrong before it was right:

- It flagged `→ answered: **(a)**` on every settled entry, reading an answer that
  names its letter as an offer of unmarked options. Answered entries are now
  exempt from the option rules.
- Then it passed an entry that marked `(a)` and left `(b)`, `(c)` and `(d)` in
  the prose beneath it — the rule fired only when *no* option was marked. Any
  lettered mention left over after the marked ones are subtracted is now the
  violation.

`option(s)` and `transformed(m)` are both in the corpus and neither is an option,
so a mention needs a non-word character before the parenthesis, and the letters
stop at `(h)` — the largest set anyone has written is four, and `(i)` is as
likely to be a roman numeral as a ninth choice.

### 63.7 What is not built

Deliberately, and each with the consumer that would justify it (§13):

- **A plugin repo.** The reader is already cross-repo — it globs every session
  under the questions root, and the slug column exists to say which repo each
  came from — while its home is one repo's justfile. `rscene` has written seven
  question files and contains the string `question` in no recipe. The extraction
  is stage 2, and its consumer is the next time `rscene` needs to read its own
  questions.
- **A TUI, and a sidecar of prepared answers.** Stage 3. The design decision it
  waits on is already settled: answers go in a sidecar `answers.md` and the TUI
  never writes `open-questions.md`, because the session appends to that file
  while the answers are being typed and one of the two writers would lose
  silently.
- **A write-time hook.** `check-patching-hook.sh` shows the shape and it would
  catch a violation in the turn that wrote it rather than the turn that reads it
  back. It is not built because a rejecting hook costs a retry every time it
  fires, false positives included, and §63.6 is the only evidence about that rate
  — two, both in the first hour.
- **A recipe that writes a question.** Unchanged from §59.3.

### 63.8 What is still uncaught

The repo column of `just question` prints the last dash-separated word of the
session slug, so a worktree named `dispatch-2` shows as `2`. It is wrong in the
awk this replaced and wrong in the same way now; it was left alone because
changing it changes output that the selftest pins, and no session has ever failed
to find its own file because of it.

Nothing checks that CLAUDE.md's description of the grammar matches the parser.
`check-default-runs-every-check` exists because a claim in CLAUDE.md wants a test
(§15.6), and this is the same kind of claim with no such test. It has not drifted
yet, which is the only reason.

---

