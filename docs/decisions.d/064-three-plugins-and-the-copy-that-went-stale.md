## 64. Three plugins, and the copy that went stale, recorded 2026-09-08

§63 built one parser and named three stages. This is stage 2: the parser, the two
hooks and the nine shared skills leave `.claude/` and become plugins that `rcad`
and `rscene` both install from one marketplace.

§63.7 deferred it under §13 — a part with no named consumer is not built — and
named the consumer as "the next time `rscene` needs to read its own questions".
The consumer turned up as a defect rather than a wish. `rscene`'s copy of the
parser still dropped the second label of a paired entry, months after §59.2 fixed
that here. Nobody was going to notice, because the thing that would have noticed
was the copy.

`.claude/` lost **1 659 lines and kept 41** (`927eb58`, 18 files). What is left is
seven files, and §64.4 is the argument for each one.

### 64.1 What was actually duplicated

Eight files were shared between the two repos and every one of them differed. Six
were pure drift with `rcad` newer, and those moved whole. Two were mixed, each for
a different reason, and the two are what the design had to answer:

- `hooks/patching-cases.json` — the cases `rcad` added for variable expansion are
  drift, but every example path names its own repo's crates: `rcad-geom` and
  `rcad-topo` against `rscene-core` and `rscene-wgpu`.
- `skills/delegating/SKILL.md` — 137 lines apart, the widest of the eight, and
  nearly all of that is drift. Except one paragraph: `rscene` wires its compile
  cache through the justfile rather than mise's `[env]`, and said so by
  contrasting itself with `rcad`.

So a file can be almost entirely shared and still carry a fact that belongs to one
repo. "Move it to a plugin" does not answer that on its own, and a design that
ignores it produces a shared file that lies to one of its two readers.

### 64.2 The plugin ships machinery, the repo ships facts

The split is on the line that already existed. Shared machinery goes in the
plugin. A repo fact goes in that repo's `CLAUDE.md`, which is where repo facts
already live, or — where a *tool* has to read it rather than a person — in one
overlay file the tool loads beside its own data.

That is `.claude/patching-cases.local.json`: **3 cases** naming `rcad` crates,
`docs/decisions.md` and a `just` recipe of ours, appended to the plugin's **24**
repo-neutral ones. `just check-hooks` passes the overlay path to the plugin's
selftest, which runs **27 patching cases and 8 context-budget cases**.

**A named overlay that cannot be read is an error, not a skip.** The selftest
takes the path as an argument and fails when it is named and unreadable. The
alternative — carry on with the core cases — passes while testing something other
than what the repo asked for, which is the shape of failure this whole section
exists to remove.

Two designs were rejected. **Parameterising** the shared files, with crate names
injected from repo-local config, invents a configuration language for two files.
**Leaving those two per-repo** and moving the other six leaves `delegating`
forked at its 137-line spread, which is the drift that motivated the work.

### 64.3 Three plugins, not one

`questions` (6 files, 902 lines), `guardrails` (9 files, 599 lines) and
`practices` (10 files, 978 lines), all in one marketplace.

Three manifests is the whole cost, and it buys what a marketplace is for: a repo
enables what it wants. One plugin would also have meant renaming `questions` —
already built, committed and passing its selftest — into something that also held
the patching hook and the delegating skill, before it had been installed once.

The split is by what the thing is, not by who wrote it:

- **`questions`** — the parser, the status line, the `asking-questions` skill.
- **`guardrails`** — the two hooks: no in-place shell patching of tracked files,
  and the context budget that asks for a handoff.
- **`practices`** — the nine shared skills, `code-quality` and `delegating` among
  them.

### 64.4 What a repo cannot give away

Four hooks stay because they are this repo's rules and nobody else's: the hub
rule (§60) and the worktree-create hook, with a check beside each.

`settings.json` stays because it is the file that names all the others, and a
plugin does not ship one. It is the seventh file and the only one that is not a
choice.

The overlay stays for the reason in §64.2.

**The status line stays as a wrapper**, 31 lines that find the plugin and `exec`
into it. A plugin ships commands, agents, skills, hooks and MCP servers;
`statusLine` is a settings key, and a plugin ships no settings. So `settings.json`
names a path, and something has to be at that path. The wrapper prints the install
command instead of the status line when the plugin is missing, because a status
line that silently prints nothing is how a missing plugin would otherwise look.

That last claim rests on the manifest schema, not on an experiment: nobody wrote a
plugin that declares a status line and watched it fail to load. §16.10 wants the
demonstration, and this is the one claim in this section without one.

### 64.5 A directory is not an install

`just check-plugins` runs first in `just default` and it started by testing
`[ -d "$p" ]` for each plugin. That is not an install test, and `rscene` proved
it: three plugin directories present, `check-plugins` green, **nothing loaded** —
no hooks, no skills, and no message saying so.

The directory test stays, because it is the bootstrap and it is what has to run
when the plugin is missing. What it now calls afterwards is
`guardrails/bin/check-declared.sh`, which asks the second question — has Claude
Code been *told* about these? — and takes either answer:

- `enabledPlugins` in any of the three settings files, the declarative form, which
  is what a non-interactive session can write;
- an entry in `installed_plugins.json`, which is what the interactive
  `/plugin install` panel writes.

Either alone counts. An empty `installed_plugins.json` is **not** evidence of "not
installed", and reading it that way is a mistake that was made during this work.

The check is explicit about its limit: **it cannot prove the running session
loaded anything**, because settings are read at session start, so a declaration
written mid-session is true in the file and not yet true in the session reading
it. It rules out the silent case — never declared at all — and says so rather than
claiming more.

The declaration went in `~/.claude/settings.json`, at user scope, so it covers
both repos with no edit in either tree.

**The ordering inside `just default` was load-bearing and unheld.** With
`CLAUDE_PLUGINS_ROOT` pointed at an empty directory, `check-plugins` exits 1 and
names the install command, while `check-hooks` and `check-questions-selftest` exit
**127** each — a bare "command not found", which is the failure a CI log explains
worst. `check-default-runs-every-check` counts the **18** `check-*` recipes and
was blind to their order by construction: both sides of every comparison in it are
`sort -u`. It now asserts that `default`'s first dependency is `check-plugins`.
Those numbers were measured by the session that wrote that fix, not here.

### 64.6 Deleting a file races with improving it

Taking a file out of a repo is a delete, and a delete merges cleanly against
somebody else's improvement by throwing it away.

It happened once here. `skills/testing/SKILL.md` was deleted for the move while a
lane added a five-line paragraph to it on `main`; the merge offered modify/delete,
and taking the delete would have dropped the paragraph with nothing to show that
it had. The fix was to port the paragraph into the plugin **first**, check it byte
for byte against `main`'s version, and only then take the delete.

The rule that follows: when a file leaves this repo, diff the copy that is landing
against the tip of `main`, not against the revision the extraction started from.
Nine files moved; one of them had changed underneath in the days it took.

### 64.7 The shared skill was wrong in both repos at once

`delegating` told every lane that `isolation: "worktree"` branches from the
primary worktree's HEAD. It branches from **`origin/<default-branch>`** — the
`worktree.baseRef` default, `fresh`, which the settings schema describes as
"branches from `origin/<default-branch>` for a clean tree".

**So a repo that does not push is a repo whose lanes work in the past, by exactly
the amount it has not pushed.** `rcad`'s local `main` was 40 commits ahead of
`origin/main` when this was found, so a lane dispatched that day started before
this section's own work — nine deleted skill directories still present, no
`check-plugins` to say otherwise — and nothing in the lane's own output would have
said so.

`rscene` found it, and the evidence is what makes it a measurement rather than a
story: two lanes were handed trees at `52b212b`, which is `rscene`'s `origin/main`
exactly.

Both halves were done. `main` was pushed (`3511c7e..41b7160`, gap now zero), and
`worktree.baseRef` is `"head"` in `.claude/settings.json` (`e09a134`).

Neither half replaces the `delegating` rule of naming the base sha in the brief
and having the lane report back both the sha it landed on and the sha it checked
out — a rule this file does not otherwise record, which is why it is written out
here rather than pointed at.
Pushing does not hold, because `main` runs ahead again the moment anyone lands
anything — in this repo, four sessions in a day. And `"head"` only swaps "the last
pushed commit" for "whatever HEAD is", which is somebody's work in flight as often
as not: while one session listed five branch tips for a merge, **three of them
moved between one command and the next**.

This is the case for the extraction, stated twice over. The claim was wrong in
`rcad` and wrong in `rscene`, because it was the same paragraph copied; it was
fixed once. It is also the case against: a shared skill is a single point of being
wrong everywhere, and nothing here would have caught it. A lane in `rscene`
noticing its base sha did.

### 64.8 Why this section is §64

§71.1 records that §64 and §65 are the gap a renumbering left behind — three lane
briefs held those numbers while `main` landed its own §63 underneath them — and
§57.8 already rules that a gap is not a defect. So this is not a reserved slot
being claimed; it is a hole being filled, and `check-decisions` accepts it because
the numbers still ascend in file order.

It is placed here rather than at the end because it is stage 2 of §63 and reads
directly after it. **The cost is that this is the first section whose date runs
behind its neighbours'** — recorded 2026-09-08, sitting above six sections
recorded 2026-09-07. Section numbers in this file are not a timeline and this
entry is the proof. Read the dates in the headings.

### 64.9 What is not built, and what is still uncaught

Stage 3 is unchanged from §63.7 and unbuilt: a TUI, with answers in a sidecar
`answers.md` that the TUI writes and `open-questions.md` that it never does. So
are the write-time hook and the recipe that writes a question.

New with this section:

- **The GitHub repo is not the install path.** The marketplace is registered as a
  *directory* source pointing at the local checkout, so a plugin runs in place and
  `CLAUDE_PLUGINS_ROOT` resolves straight to it. Repointing at the `github` source
  would move the plugins to a version-stamped cache path and break that variable
  on the next version bump. The repo is a backup and a CI host. Its own CI went
  green in 13 seconds on the first run, which is worth recording only because
  `rcad`'s had never gone green at all until the day before.
- **Nothing checks `CLAUDE.md`'s "Three plugins" prose against the marketplace
  manifest.** This is the same untested claim §63 closed with, one file further
  out: a plugin renamed or added leaves the prose wrong and nothing fails.
- **Nothing checks that the two repos' overlays stay disjoint from the core
  cases.** A case duplicated into an overlay is tested twice and removed once.
- **`check-declared.sh` cannot see what the running session loaded**, per §64.5,
  and there is no evidence about how often that gap matters, because the only way
  it has been observed is the `rscene` case where nothing was declared at all.

---

