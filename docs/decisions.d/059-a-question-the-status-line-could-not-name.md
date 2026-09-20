## 59. A question the status line could not name, recorded 2026-09-06

The open questions of a session live in a file in that session's own scratchpad.
No section had recorded it before this one. It could be counted and it could not
be read: the status line said which labels were open, and getting the text of one
meant `!cat` on the whole file. This records the address that closed that, a
parser bug the work turned up, one recipe deliberately not built, and a number
this section corrects.

### 59.1 The address the file did not have

The file is `<tmp>/claude-<uid>/<slug>/<session>/scratchpad/open-questions.md`.
Every part of that is derivable except the session id, which only the harness
knows — so nothing a person can type names the file. The status line already had
the id, in its own input, and was throwing it away.

Six characters of it now print beside the labels, the width `just lanes` prints a
lane's ref in, and `just question <hash> <label>` takes that as the address.
`just question` alone lists the sessions that have open questions; with a hash
and no label it lists that session's labels and headings.

What this replaces is `!cat`, and the cost of `!cat` is not the reading. Shell
mode puts the output into the conversation, where it is charged again on every
later turn; `statusline.sh`'s own header records 21 command outputs and 5 710
characters put there that way on 2026-09-05. The file being read that way on
2026-09-06 held sixteen open questions, to get the text of one.

### 59.2 Fourteen of sixteen

An entry may carry two labels when one answer settles both, `**Q7 / Q11**`. The
parser matched the label once, so the second was neither counted nor shown.

Measured on the file as it stood at 10:18 on 2026-09-06: fourteen open checkbox
lines, two of them paired, so **sixteen open questions and fourteen labels on the
line**. The two that were invisible, Q11 and Q27, had been open all day. Run on
that file, the old parser prints fourteen labels and the new one sixteen.

**This corrects a published number.** `0828cb9`, the doc comment it added to
`statusline.sh`, and the paragraph it added to CLAUDE.md all say "thirteen of
fifteen". That is a miscount of the same file — the open lines were fourteen, not
thirteen — and it went in without being re-derived. The two files are corrected
in the commit that adds this section. The commit message cannot be, so this is
the record that it is wrong.

The failure is the shape §57.8 had recorded from another direction that same day:
a short answer that reads like a complete one. A status line whose whole job is
"what is waiting on you" is the last place that can under-report it, so the
labels are walked to the end of the run rather than matched once.

### 59.3 What is not built: a recipe that writes a question

A recipe that *asks* a question was proposed and declined. §13 is not what
declines it — the consumer exists, it is every session — so the argument belongs
here rather than in a deferral.

A body is paragraphs of prose carrying backticks, `§` refs and `**`. Passing one
through a shell argument means quoting all of that correctly, and it is never
cheaper than `Edit`: the text is written once either way, and whatever the recipe
prints is charged on top. The cost argument runs against the recipe, not for it.

The stronger reason is what the two operations are. Adding a question is an
append, which cannot silently match the wrong thing. Marking one answered is a
substitution, `- [ ]` to `- [x]`, and that is the shape §50.8 keeps out of the
shell: a pattern that does not match fails quietly, where `Edit` errors. A recipe
that did the append and not the substitution would be half a tool, and one that
did both would put the quiet half in the shell.

The model also has no addressing problem to solve — its own scratchpad path is in
its system prompt. The hash is for the person.

`just question <hash>` with no label took the place of a write-side check. It
reads labels through the same walk the status line uses, so an entry the line
would drop is missing there too.

### 59.4 The exemption the patching hook advertised but did not implement

Found while writing a fixture for the above, and fixed in the same run.

The hook refused `S=/tmp/…/scratchpad; cat > "$S/fixture.md"` and told the caller
"Writing under /tmp or a scratchpad is still fine". It judges the extracted
redirect fragment as text, so what it saw was `$S/fixture.md`, which matches
neither `^/tmp/` nor `scratchpad`. Failing closed is the right direction; saying
the opposite while doing it is not, and the caller it stops is a lane writing a
fixture to its own scratchpad — the case CLAUDE.md keeps Bash for.

Assignments made in the same command are now resolved before the path is judged.
Only the same command can matter: the Bash tool carries a working directory
between calls and no shell state, so a variable set in an earlier call is not set
here either. One that cannot be resolved stays unresolved, and unresolved is
still judged a tree file.

The guard is not loosened, and that is the half worth testing: a variable holding
`crates/rcad-geom/src` is still refused, and so is one this command never set.
Five cases, two of which the old hook gets wrong.

The hook then refused the command that would have added those five cases, because
it carried `> "$W/lib.rs"` as literal text. Its own header already describes
that: the cases live in JSON rather than in the runner for exactly this reason,
and it had blocked two earlier drafts of its own test.

### 59.5 What is still uncaught

`check-decisions` now fails on two sections sharing a number (§57.8, `cc969ad`),
so the collision this section's own numbering came close to repeating cannot
happen silently any more — §58 was taken between the number being chosen and the
section being written.

**It catches the collision at merge, which is later than it is made.** The check
reads one file, so it cannot see a number taken on a branch that has not landed.
On 2026-09-06 this section and "Nobody works in the hub" were both written as
§59, on two unmerged branches, and the duplicate was found by the other session
saying so — not by the check, which would have fired only when the second of them
merged. Four instances in one day, for a shape unnoticed across the fifty-seven
sections before it: one near-miss on 56, §58 taken mid-write, this collision, and
the renumber that settled it. Nothing watches the branches, and nothing is
proposed here that should: the check fires before anything is published, which is
the point at which being wrong costs something.

A heading deleted with its body left behind still can. The orphaned lines join
the section above, `decisions-toc` counts them there, and nothing says so. No
check is built for it, and no claim is made here that none is needed: it has not
cost anything yet, which is the only reason.

