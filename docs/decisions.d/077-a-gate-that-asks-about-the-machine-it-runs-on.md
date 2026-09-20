## 77. A gate that asks about the machine it runs on, recorded 2026-09-08

`check-plugins` asks two questions. Are the three plugin directories there, and
has Claude Code been *told* about them. The first is about this checkout and a
runner can answer it. The second is about a Claude Code installation, and a
runner has none — so on CI it is not a question with a wrong answer, it is a
question with no answer at all.

That difference had never come up, because until 2026-09-08 CI ran `just test`.
The first push that ran the whole gate died on it.

### 77.1 The first `just default` on a runner, red in 31 seconds

Run `34260645261`, the push of `02367fc`. It printed `3 plugins, all where this
repo expects them` — so the marketplace clone worked and `CLAUDE_PLUGINS_ROOT`
resolved — and then failed on the arm after it: `marketplace
mattiasgronlund-local is not registered`, and all three plugins `on disk but
not declared to Claude Code`.

**The three greens above it do not cover this and never could.** They ran
`just test`: `git show d5bb95a:.github/workflows/ci.yml` ends `- run: mise exec
-- just test`, and `d5bb95a` already contained the commit that added the
declaration arm. So `02367fc` is the first `main` commit whose runner reached
it. Saying "CI is green" was true and covered nothing here, which is the same
shape §62 records — a sweep that could only ever return one answer.

The pre-flight sweep that was supposed to catch exactly this missed it by one
line. It pointed `CLAUDE_PLUGINS_ROOT` at an empty directory, which fails the
*presence* arm and stops there, one line above the arm that broke. Pointing it
at a real clone passed — on a machine that has the marketplace registered,
which is the reason a runner does not have.

### 77.2 The predicate is the config directory, not `CI`

Three predicates were on the table and two of them are worse.

`CI` was the first draft and is wrong in both directions: a human who exports it
loses the check silently, and a runner that does not set it gets a red nobody
can act on. It is a proxy for the thing rather than the thing.

**The config directory is the thing.** `~/.claude` *is* the installation —
`settings.json`, `plugins/known_marketplaces.json`,
`plugins/installed_plugins.json` are all inside it — so its absence is not
evidence that there is no Claude Code, it is what "no Claude Code" means. Any
machine a person works on has one, so the skip cannot quietly fire there. That
argument came from the peer session that landed `3e59648`, which fixed the red
in the `justfile` within the hour; this section keeps the argument and moves
where it lives (§77.3).

One correction to it. `CLAUDE_CONFIG_DIR` moves that directory, and the guard
and the file reads have to agree on where it is — which they could not while
`$HOME/.claude` was written out in four separate places and the guard asked
about a fifth. It is one `config` variable now, and two of the five selftest
cases are that agreement in its two halves.

**And it skips out loud.** Printing what it did not do rather than reporting a
pass it did not earn is §45's lesson: the corpus tests used to print "skipping"
and return green, and a green suite had then never read a real file. Skipping
and passing are both exit `0`, so the selftest checks the words too — without
that, an unconditional `exit 0` at the top of the script passes every
exit-code case.

### 77.3 Declaring the plugins in the tracked settings file, and why not

The obvious alternative was to commit an `enabledPlugins` block to
`.claude/settings.json` so the runner could read the declaration out of its own
checkout. `/plugin install` had in fact already written a partial one there,
which is how the option came up at all.

It is refused on two counts, and the second is the one that settles it.

`enabledPlugins` alone does not pass: the marketplace arm is separate and needs
`extraKnownMarketplaces` beside it. And `has($m)` is that arm's entire test, so
a placeholder value passes while asserting nothing — which is precisely the
failure `check-declared.sh` exists to catch, reintroduced in the file meant to
satisfy it. Measured against a throwaway repo root with an empty home, by the
peer session, before either of us wrote anything.

The deeper count: **it would be green for a reason the runner does not have.**
Declaring a plugin to a machine with no Claude Code enables nothing. The line
would say something true about this repo's intent and nothing about the runner,
and it would land for anyone cloning, naming a marketplace id that means
nothing to them.

The same rewrite had also deleted `worktree.baseRef: "head"` (`e09a134`) as
collateral. Restoring that is repair, not a settings change; dropping the
`enabledPlugins` block put the file back to what is committed.

### 77.4 The guard lives in the plugin, not in the justfile

Both repos ask this question, so the guard belongs where the question is
implemented. `3e59648` put four lines in `rcad`'s `check-plugins`; this moves
them into `check-declared.sh` and takes them back out of the `justfile`, which
leaves one copy instead of two — §64's whole subject, and `rscene` is the repo
that found the silent-plugin case in the first place.

`check-declared-selftest.sh` holds five cases and joins `bin/selftest.sh`, which
`just check-hooks` already runs and `just default` already gates. The cases are
not symmetric on purpose. **The direction that matters is the human one**: a
skip that leaks off a runner turns the check off for everybody and nothing says
so, which is the silent case the check was written for, restored by its own
fix. The runner direction is there because a test of one direction is a test of
neither.

Checked against mutants rather than by reading: the guard removed fails two
cases, the guard made unconditional fails three, and `CLAUDE_CONFIG_DIR`
ignored fails the two that exist for it.

The pin in `ci.yml` moved from `952672e` to `f5f2f48` in the same commit, and
that bump is load bearing rather than housekeeping — pinned below it, the
runner has no skip and `just default` is red again.

### 77.5 What a CI green now covers

Everything a local green covers except one arm, and the arm is named. The three
plugin directories, the questions parser selftest, every other `check-*` recipe
run on the runner exactly as they run here. Only "has Claude Code been told" is
skipped, and only where there is no Claude Code to have been told.

**Run `34266469454`, push of `e8ae455`, is the first `just default` to go green
on `main`.** It reported 1811 tests run, 1811 passed, 0 skipped — the same three
numbers a local `just default` gave on the same tree, which is the agreement a
runner exists to produce. It printed `no /home/runner/.claude, so no Claude Code
to have been told about mattiasgronlund-local: the declaration check is skipped,
not passed`, so the skip fired where it was meant to and said so.

It took two more red runs to get there, and neither was about plugins. §77.6.

### 77.6 A check that asked what kind of terminal it had

Runs `34264306999` (push of `3e59648`) and `34265339769` (push of `5b2413e`).
Both cleared `check-plugins` — the skip printed, the marketplace clone resolved
— and both died one recipe later on the same line:

    wildcard arms over our own enums: 5 (0 direct, 5 composite), expected 88

**Zero direct is a parse breaking, not eighty-three match arms going away.**
`ci.yml` sets `CARGO_TERM_COLOR: always`, so clippy writes

    ESC[1m ESC[33m warning ESC[0m ESC[1m ": wildcard match will also match ..."

and `check-wildcards` counted with `grep -c '^warning: wildcard match'`, anchored
at the start of the line. The escape sits where the anchor is looking, and every
direct warning stops matching at once.

**The number looked like a tree state, which is why it cost two pushes.** The 5
that survived are the composite arms, and they come from an `awk` pass over
`.rs` files that colour never touches. So the recipe reported a plausible count
and sent its reader to §40 to hunt for a new wildcard, rather than saying the
count had failed. A check that can be wrong should fail in a way that names
itself; this one failed in the vocabulary of a real defect.

**Two people measured it, and the second measurement is the better one.** The
dispatcher session reproduced the failure with `CARGO_TERM_COLOR=always just
check-wildcards`, which gives the runner's line byte for byte. A peer session
then read one clippy log two ways: anchored grep as the recipe writes it, 0;
the same log with the escapes stripped, 83. 83 + 5 = 88, the pinned count. The
first shows the failure happens; only the second shows the arms were in the log
the whole time.

**The fix is three parts and each does something the others do not.**
`_wildcard-direct-count` takes the count out of the clippy run and strips ANSI
before matching, so the answer is the same either way — the same split `t` and
`_test-digest` already use, and for the reason the comment there gives: a parser
kept apart from the run is one you can hold against logs whose answer is already
known. `CARGO_TERM_COLOR=never` on the clippy call keeps the saved log readable,
which matters because the failure branch greps `'^error'` out of it and was
anchored the same way. And `0 direct` against a non-zero ledger now says the
parse broke and names the log.

**The test is the coloured line itself.** `check-wildcards-selftest` carries the
same warning twice, once plain and once with the escapes `cargo clippy` actually
emitted under `CARGO_TERM_COLOR=always` — captured from a real run, not guessed
at. Both must count 2. Mutation-checked by taking the `sed` back out, which
reports `read 0 warnings from coloured.log, expected 2`: the runner's own number.

**This is the third instance in a day, and three is what makes it a rule.** Each
one is a check that passed for a property of the environment it ran in rather
than a property of the tree.

- §77.1's pre-flight sweep pointed `CLAUDE_PLUGINS_ROOT` at a real clone on a
  machine where the marketplace is registered.
- §77.2's declaration arm asked a question that only has an answer where Claude
  Code is installed, and every machine that ran it had one.
- This one anchored a grep at `^` and was only ever run where cargo had no
  terminal to colour for. A `$(...)` capture is not a tty, so a local green
  earned the anchor a pass a runner never would.

The technique that found all three is the same: **take away what the environment
supplies and run it again.** Point the variable at an empty directory. Point
`HOME` at one. Force the colour on. A green is evidence about the environment it
ran in and about no other.

One rule falls out of it that is worth stating on its own: **a recipe that parses
a tool's output owns the format it parses.** Reading a format the ambient
environment chooses is how a check ends up measuring the machine.

**What a green still cannot cover.** After the fix, every non-compiling `check-*`
recipe was run in a built runner shape — a plain `--no-local` clone with one
worktree, colour forced on, `HOME` empty — and all passed, `check-hub` correctly
answering that a lone checkout is not a hub. That is evidence and not proof:
`check-kache-off-on-ci` forces `CI` both ways itself and is indifferent, and the
four selftests that build their own repositories already pass `-c user.email`
explicitly, so the identity trap was closed before anyone looked. The gap that
remains is the one that has now bitten three times — an environment difference
nobody has thought to remove yet.

### 77.7 Three versions of the plugins, and the gate asserts a different one than CI

§77.4 called it a limit that nothing checks a human's installed plugins against
the CI pin. Measured on 2026-09-09, that limit is wider than the sentence
suggests, and in a direction the sentence does not mention: **the two gates run
different numbers of cases and neither says so.**

`just check-questions-selftest` here runs 20 parser + 22 status line + 22 TUI =
**64**, green. `ci.yml` clones the marketplace at `f5f2f483`; that tree's own
`selftest.sh`, extracted with `git archive` and run, is 20 + 22 = **42**, green.
Both greens, different suites, and the only difference in the output is a count
nobody reads back. `rscene` measured the same two numbers on its own machine and
runner the same day.

Only `questions` diverges. `guardrails` is byte-identical between `f5f2f483` and
the checkout, so `check-hooks` asserts the same 24 + 8 + 5 either way. "The pin
is stale" and "the pinned suite is smaller" are different claims and only the
second one is true.

**Where the code being run actually comes from.** `plugins_root` defaults to
`~/git/mattiasgronlund/claude-questions/plugins`, so every recipe here executes
scripts straight out of that working tree — a push to the marketplace changes
what `just gate` runs on this machine with no install step and no pin move. That
is what makes the 42/64 gap appear without anyone deciding anything.

`~/.claude` holds two other things and neither is the code: the declarations
`check-declared.sh` reads, and — for `questions` alone — a leftover copy at
`~/.claude/plugins/cache/mattiasgronlund-local/questions/0.1.0`, byte-identical
to `952672e`. `installed_plugins.json` names cache paths for all three plugins
at that sha, but the `guardrails` and `practices` directories it names **do not
exist**. The hooks run anyway: a `sed -i` on a tracked `.md` was refused by our
own hook while this was being written, and there is no cached `guardrails` for
that to have come from. So a directory-source marketplace does run in place, as
`CLAUDE.md` says, and the install metadata is a third number — `952672e` —
describing neither what the gate runs nor what CI runs.

Not established: which copy Claude Code loads the `questions` *skill* from, since
that plugin is the one with both a cache copy and a checkout. The hooks question
is settled by the probe; this one is not, and no claim is made.

`rscene` fixed its half by bumping its pin and making the recipe print the sha it
ran — `rscene` `0a97b18`, run `34327882376`, where the runner log now carries
`questions cases from … (4728682)` beside the three counts. rcad has not done
either. The pin here stays at `f5f2f483` until there is a reason of rcad's own to
move it, and a wider pinned suite is such a reason; that it is also `rscene`'s
reason does not make it rcad's.

