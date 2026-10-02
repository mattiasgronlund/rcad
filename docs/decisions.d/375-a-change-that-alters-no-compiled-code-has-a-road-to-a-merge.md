## 375. A change that alters no compiled code has a road to a merge — 2026-10-02

*Queue row Q740, which replaces Q691. A change that alters nothing that runs
had no road to a merge through the `lanes` workflow. It now has one, on
evidence a command prints from the diff. A change to what runs still needs a
test that fails without it. This lane changed `.claude/workflows/lanes.js`,
`.claude/agents/verifier.md` and the justfile's `check-lanes-workflow`.*

In short: `lanes.js` hands every verifier one command, `LINES_THAT_RUN`. It
reads the lane's whole-file diff and prints every changed line that could
change what runs. When that print is empty, the verifier skips the revert, and
`unproven` lets the lane merge if its tests pass. A lane that names no test
merges on the empty print alone. When the print is not empty, everything works
as it did before this section.

### 375.1 How it failed

Three ways, and in each Mattias had to merge by hand:

- **A comment in a file that is not `.md`.** `verified()` gives a lane that
  names no failing test the moved-text steps, and `verifier.md`'s moved-text
  step 6 said any changed file not ending in `.md` rules out `merge`. A
  doc-comment edit in a `.rs` file, or a comment in the justfile, always
  came back `fix_first`. The bundle Q440,Q619,Q720,Q624,Q684 at `237cf663`
  is the worked case: `git diff c92a035b...237cf663 --stat` shows 6 files, 105
  insertions and 33 deletions. Four `.rs` files change only `///` and `//!`
  prose, and step 6 was the verifier's only `fix_first`.
- **A pin inside `src/`.** A test pinning behaviour the base already has
  passes with the source put back, so `unproven` refused it unless a mutation
  was reproduced. `pinsOnly` is no way round that when the test is a
  `#[cfg(test)]` unit test in `src/`, because §287 refuses `pinsOnly` beside a
  source path.
- **A pure code move (Q601).** Its verifier said `merge`. Then `unproven`
  refused it, because its tests pass with the source put back.

Q519 and Q601 waited on 2026-09-29, and the Q440 bundle on 2026-09-30.

### 375.2 What the print shows

`LINES_THAT_RUN` is an awk program held as one line in `lanes.js`.
`linesThatRunCommand` puts it behind
`git diff --no-color --no-ext-diff -M --src-prefix=a/ --dst-prefix=b/ -U1000000 <base> <branch>`.
The whole file is context, so the program sees each `#[cfg(test)]` and each
recipe header above a changed line. The flags are spelled out so no one's git
config can change what it reads. It prints each changed line as
`<path>: <+ or -><line>`, plus any rename, mode change or binary change. It
leaves out:

- every `.md` file. The moved-text steps still trace those (§266).
- comment lines and blank lines: `//` in `.rs` and `.js` files, and `#` in the
  justfile and in `.toml`, `.sh`, `.py` and `.yml` files. A `#!` line is not
  a comment, because a shebang decides what runs a script. `///` and `//!` are
  comments here too. A changed doctest is a changed test, and the gate runs
  the doctests.
- test code: files under `crates/*/tests/` and `fuzz/corpus/`, both before and
  after a rename. In a `.rs` file, the item after a column-0 `#[cfg(test)]`
  line, up to its column-0 closing `}`. In the justfile, the body of a
  `check-*` recipe.

It leaves out nothing else. A `#[cfg(test)]` line that is added or removed is
printed, because it moves an item into or out of the build. So is a recipe's
header or `[doc]` line, and any line in a file type it does not know. When git
prints no diff at all, the program prints a line that says so, so a failed
`git diff` can never read as an empty print.

Why awk inline, in `lanes.js`: the verifier's Bash guard runs
`git diff ... | awk '...'`, but it refuses `awk -f`. That was tried in this
lane's session, where it refused `awk -f <file>` and `awk --version` and
ran the inline command. A recipe would be easier to read, and it lies outside
this row's write set.

### 375.3 What the new path accepts

`unproven` (`.claude/workflows/lanes.js`):

- **A lane that names tests (`kind` `'tests'`)** and has an empty print merges
  when `tests_pass_on_branch` is true, with no revert and no mutation. A test
  that pins what the base already does is the §287 shape. The empty print is
  what shows there is nothing to revert, and it does that from the diff
  instead of from a field the dispatcher sets.
- **A lane that names no test (`'moved_text'`)** and has an empty print is no
  longer barred because a file does not end in `.md`. The moved-text steps 7
  and 8 still check its `.md` files. Test code it changed is shown to pass by
  the verifier's step 3 gate and again by the merger's gate.

The next bundle in the queue is the shape this exists for. Q655 adds a unit
test inside `brep.rs`'s `#[cfg(test)] mod tests`. Q738 renames a seed under
`fuzz/corpus/` and edits two files under `crates/rcad-step/tests/`. Q731
changes a comment and an `echo` inside `check-gate-runs-every-check`. Q732
adds an arm to that same recipe. All of that is test code or comment.

### 375.4 What it still refuses, and why

- **Anything the print shows.** A lane that names no test and has a non-empty
  print is refused, and the reason starts with the print's first line. A lane
  that names tests and has a non-empty print still needs
  `tests_fail_without_fix` or `mutationProven`, and a source path to revert,
  exactly as before. A mixed bundle (§352.4) whose rows change behaviour keeps
  the revert for all of its rows. A comment-only file in the same bundle does
  not waive it, because the print covers the whole branch.
- **A print the verifier did not report, or got from a different command.**
  The schema requires `lines_that_run_command` and `lines_that_run` for both
  kinds. `unproven` compares the command with the one it sent, ignoring
  whitespace. A missing print counts as not empty.
- **`pinsOnly` is unchanged.** Its four lines that §287 and arm 10 hold are
  where they were. It is still never inferred from an empty `sourcePaths`.
  The new path is a different inference: it is drawn from what the diff
  shows, not from what the dispatcher left out. A tooling lane whose
  dispatcher forgot `sourcePaths` still changes lines that run, so it still
  gets "the lane declared no source directory to revert".

### 375.5 What it does not catch

- **A line inside a string literal that looks like a comment** is read as a
  comment. An example is a `.rs` raw string with a line that starts with `//`,
  or a `#` line in a heredoc inside a justfile recipe. This lane fed the
  program a hand-written diff in which `// line two`, a line inside a Rust
  string literal, became `// line two, changed`, and it printed nothing.
  This is wrong/C: only a probe built it, and no lane has met it. Closing it
  needs a tokenizer, and awk is not one.
- **A test weakened or taken out.** Test code is never printed, so an
  assertion removed from a test leaves the print empty. `verifier.md`'s step 1
  now tells the verifier to read what a branch takes out of a test, and to
  say `fix_first` when the lane does not say why. That check is the
  verifier's reading, not the print. It guards the tests, not what runs.
- **A code move** is printed as lines that run. Q601's shape still needs a
  revert or a mutation. `git diff --color-moved` can mark moved blocks, but
  this row did not build a check on it. This lane's report asks for a row.

### 375.6 The test

`check-lanes-workflow`'s sixteenth arm takes the awk program out of `lanes.js`
and runs it on a small repo it builds with one base commit and one branch
commit. It wants back exactly these lines and no others: a code line changed
in a plain `mod`, a function added after the `#[cfg(test)]` module closes, a
shebang added to a recipe that is not a `check-*`, and a source file renamed.
It also checks the print is not empty for an empty diff, and it checks the
lines that use the print: the line that waives the revert only on an empty
print, the three refusals, the schema's `required`, the command `VERIFY`
hands over, and the new moved-text rule in `verifier.md` with the old rule
gone.

Twelve new cuts, each caught by this arm alone. Four of them change the awk
program toward letting code through: `src/` read as test code, a test
module that never ends, a shebang read as a comment, and no line for an empty
diff. `mise exec -- just check-lanes-workflow` reports "44 cuts, each caught"
on this branch. With `.claude/workflows/lanes.js` and
`.claude/agents/verifier.md` put back to `c94d6d22`, it fails at arm 16 with
"does not hand the verifier the lines-that-run command".

Like §287.5, a text arm cannot run the workflow. The first bundle that merges
on an empty print is the end-to-end proof.
