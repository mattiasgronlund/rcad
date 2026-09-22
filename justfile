# `{{ args }}` interpolates a recipe's arguments into the shell *unquoted*, so a
# variadic recipe that passes them on splits them again on whitespace and hands
# the shell any punctuation they contain. That is not theoretical: `just t -E
# 'test(/walk/)'`, the example this file and CLAUDE.md both give, died on
# `Syntax error: "(" unexpected` from the day it was written, because `sh` saw
# the parentheses. With this setting the arguments arrive as `"$@"` with their
# original word boundaries, and `check-t-selftest` runs that exact example.
set positional-arguments

# Nothing moves the hub's HEAD on its own (docs/decisions.md §60.9), so the one
# tree nobody works in is the one tree that goes stale — and it goes stale
# invisibly, because its files match its own old commit and `check-hub` calls
# that "parked and clean". `just decision 144.5` typed in a hub parked five days
# back answers "no section 144.5", which is a property of that checkout and not
# of this repository (§146, and §77.6 for the shape).
#
# `just` evaluates every top-level assignment before it runs a recipe, whether
# or not the variable is used — measured on 1.42.4, not assumed — so this is the
# one hook the file has that fires on `just t` as well as on a bare `just`. The
# script exits within three git calls anywhere but a stale hub, so every lane
# pays only that; `questions_root` below has been paying a backtick since long
# before it.
#
# The value is unused and always empty. The work is the side effect, and what it
# has to say it says on stderr, which `just` passes straight through.
hub_aligned := shell('bash "$1/tooling/hub-align/align.sh" "$1" || true', justfile_directory())

# `just` with no arguments runs the **first** recipe in this file, not one named
# `default` — so putting the listing here is what makes a bare `just` show you
# what there is, and `just gate` the thing you actually wait nine minutes for.
#
# `--unsorted` keeps the file's own order, which is grouped by subject. Sorted
# alphabetically the ten `check-*` recipes lead and `gate` lands under `g`,
# between `fuzz-writer` and `lanes`.
#
# Every listed recipe carries a `[doc("...")]`. Without one, `just` takes the
# **last line of the comment block above the recipe** as its description, and
# every comment block here is an argument several paragraphs long — so the
# listing was 35 rows of fragments torn out of mid-sentence, `check-decisions #
# at merge time, which is the expensive place to find it.` among them.
# `[private]` hides the `*-selftest` recipes, which nobody types — it said
# "the nine" until 2026-09-19, when there were twenty-two of them. `just recipe`
# with no argument is the listing that shows them, §77's rule about tallies
# being the reason this sentence no longer carries one.
[doc("Everything this repo can do, one line each. Bare `just` runs this.")]
list:
    @{{ just_executable() }} --list --unsorted

# Every step of the gate, in the order CI runs them, and the one place that
# order is written down. `check-gate-runs-every-check` reads this block to prove
# no `check-*` recipe was written and then never run.
#
# **One step per line, joined by `\` into a single-line value.** The value is
# what it always was — every consumer below word-splits it and none can tell —
# but the *source* is now one line per step, and that is the whole point: git
# merges by line. As one line this was the single worst merge shape in the repo,
# a 43-name line that two lanes adding a check each rewrite in full, and it
# conflicted on 2026-09-16 (`2da15de4`) for exactly that reason. One name per
# line, two lanes appending two different checks, and git takes both.
#
# Continuation rather than a `"""` indented string, which was tried first: that
# form puts real newlines in the value, and `for step in {{ gate_steps }}` is
# then a bash syntax error at the second name. `docs/decisions.md` §177.
#
# **A list rather than `gate`'s dependencies, which is what it used to be.** A
# dependency that fails takes `just` down with it, so a gate that failed printed
# whatever the failing step said and stopped — and the summary block below, the
# thing a reader actually wants, could only ever have been printed on a pass.
# Driving the steps from a loop is what lets the same block describe both.
#
# **`check-wildcards-selftest` runs before `check-wildcards`, and the order is
# the finding of §173.** The ledger check judges the tree with an instrument it
# does not itself prove; the selftest proves the instrument against fixtures
# whose answer is known. With the ledger first, a broken scan is reported as
# "a new one, or a new variant reaching an old one" — a sentence about the tree,
# when what moved was the scan. That is the wrong place to start reading, and it
# is where a container's mawk put one session. Prove the instrument, then read it.
gate_steps := "check-plugins \
    check \
    check-fuzz-builds \
    check-enums \
    check-wildcards-selftest \
    check-wildcards \
    check-ignored \
    check-hooks \
    check-skills-selftest \
    check-skills \
    check-kache-off-on-ci \
    check-gate-runs-every-check \
    check-gate-summary-selftest \
    check-gate-docs-selftest \
    check-lanes-selftest \
    check-release-lanes-selftest \
    check-status-selftest \
    check-plan \
    check-plan-selftest \
    check-hub \
    check-hub-selftest \
    check-hub-align-selftest \
    check-clean-targets-selftest \
    check-prune-worktrees-selftest \
    check-decisions \
    check-decisions-selftest \
    check-claim-section-selftest \
    check-where-the-work-is \
    check-src-selftest \
    check-recipe-selftest \
    check-questions-selftest \
    check-corpus \
    check-corpus-selftest \
    check-coverage-selftest \
    check-fuzz-overnight-selftest \
    check-fuzz-watermark-selftest \
    check-gate-fresh-selftest \
    check-gate-fresh-cleanup-selftest \
    check-fuzz-triage-selftest \
    check-fuzz-unit \
    check-fuzz-unit-selftest \
    check-spend-selftest \
    lint \
    test \
    check-release-build \
    check-t-selftest \
    deny"

# The steps that cost a runner money, and that a change touching only `docs/`
# and `*.md` cannot move either way. `gate-docs` runs `gate_steps` **minus**
# this list, so a new `check-*` recipe joins the docs arm by default and only an
# explicitly named step is left out — the same direction
# `check-gate-runs-every-check` guards, for the same reason. A list of what to
# keep would have quietly dropped each new check from the docs arm instead.
#
# Two reasons land a step here, and "metered" rather than "compiling" is the
# name because of the second:
#
#   It needs a build. `check`, `lint`, `test`, `check-release-build`,
#   `check-fuzz-builds` and `check-fuzz-unit` obviously; `check-wildcards`
#   because it reads clippy's output; `deny` because it resolves the dependency
#   graph, and a prose commit cannot change `Cargo.lock`.
#
#   It needs the STEP corpus. `check-corpus` is 0.16 s and compiles nothing, and
#   it is here anyway: it **fails** when `RCAD_STEP_CORPUS` is not a directory,
#   by design (§44.10), and putting a corpus there is a 119 MiB fetch. A prose
#   commit cannot change the corpus or `corpus/manifest.sha256`, so the arm that
#   pays for one is the arm that might have broken it.
#   `check-corpus-selftest` stays in the docs arm, because it builds its own
#   fixture and never reads the real corpus.
#
# On a metered runner this is 1m07s against 9m46s. `docs/decisions.md` §172.
gate_metered_steps := "check check-fuzz-builds check-wildcards lint test check-release-build check-fuzz-unit deny check-corpus"

# Where the whole run is kept, so the block at the end can be read back in full
# without running it again. A variable so `check-gate-summary-selftest` can
# point a nested run somewhere else: the selftest runs inside the gate, and a
# fixed path would have it truncate the log its own parent was still writing.
gate_log := "target/last-gate.log"

# The gate a prose commit needs: `gate_steps` minus `gate_metered_steps`,
# driven through `gate` itself so the summary block and `quiet` come for free.
#
# **This exists because the runner is metered and half of `main` is prose.** Of
# `main`'s last 150 commits, 43 touched only `docs/` and `*.md` — 42 of them
# `docs/decisions.md` — and each one bought a 13-minute compile of a tree whose
# source had not moved. Every step it keeps finishes in under a second:
# `check-decisions` 0.08 s, `check-plan` 0.14 s, `check-where-the-work-is`
# 0.21 s, `check-enums` 0.08 s, `check-ignored` 0.54 s, `check-corpus` 0.16 s.
#
# It is the arm `ci.yml` picks by path, and the choice is made there rather than
# here: a recipe cannot see what a push touched, and one that guessed from
# `git diff HEAD~1` would be wrong on the first merge commit it met.
#
# Running the docs arm by hand on a change that touches `.rs` is not guarded
# against, and deliberately: the steps it drops are the ones that cost money, so
# the way to be sure is `just gate`, and this recipe says which steps it dropped
# so that a reader of the output can see what was not asked.
[doc("The gate a docs-only change needs: everything that does not compile. `quiet` works here too.")]
gate-docs mode="loud":
    #!/usr/bin/env bash
    set -euo pipefail
    steps=""
    dropped=""
    for step in {{ gate_steps }}; do
        skip=""
        for heavy in {{ gate_metered_steps }}; do
            if [ "$step" = "$heavy" ]; then skip=yes; break; fi
        done
        if [ -n "$skip" ]; then dropped="$dropped $step"; else steps="$steps $step"; fi
    done
    # An empty list would run nothing and exit 0, which is the one answer this
    # must never give: a gate that checked nothing reads exactly like a pass.
    if [ -z "$steps" ]; then
        echo "gate-docs would run no steps at all, which would pass by checking nothing" >&2
        exit 2
    fi
    echo "gate-docs: dropped${dropped:- nothing}"
    exec {{ just_executable() }} --set gate_steps "${steps# }" gate '{{ mode }}'

# The gate with what this machine supplies taken away, which is the one thing a
# local run cannot otherwise be: a fresh clone, in a container, with no `HOME`
# from here, no `CLAUDE_PLUGINS_ROOT`, no sibling hub and no warm `target/`.
#
# **This is `docs/decisions.md` §77.6's manoeuvre as a recipe.** Three red runs
# in 2026-09 were checks passing for a property of the environment they ran in
# rather than of the tree, and the way all three were found was to take away what
# the environment supplied and run it again. That was done by hand each time.
#
# It is not a replacement for the runner and does not pretend to be: it runs on
# the same kernel, the same docker daemon and the same disk, so it cannot catch
# what only a different machine would. What it does catch is the whole class the
# runner was bought for, and it costs no minutes — which is why it is the arm
# that survives a spent Actions allowance. §172.
#
# `check-plugins`, the plugin arm of `check-hooks` and `check-questions-selftest`
# all skip in here, because there is no Claude Code config directory. That is
# them working, not them broken: the predicate is that directory and never `CI`,
# `docs/decisions.md` §77.
#
# **The tools live in a named docker volume, the tree does not.** `mise install`
# builds eight `cargo:` tools from source, which is two minutes for
# `cargo-nextest` alone and dwarfs the gate it is setting up for. The volume
# keeps them across runs; the clone is thrown away every time. Taking `HOME`
# away is about this machine's `~/.claude` and its build cache, not about
# rebuilding `cargo-deny` on every run, and nothing of this repo's own
# compilation is in there. `docker volume rm rcad-gate-fresh-tools` resets it.
#
# Cold by construction, so expect the nine minutes the `gate` doc names rather
# than the median 98 seconds, and more than that on the first run of all. Pass
# `run_in_background` and be woken — and read the output file rather than piping
# it, because a pipe hands back the pipe's exit code and a failed run then reads
# as a pass. That happened on this recipe's own first run.
#
# **A pass here is written to the green record**, the same one a plain `just
# gate` writes and the one `fuzz-watermark` reads to choose what the overnight
# run fuzzes (§173.9). This is the strongest green this machine can produce —
# the whole gate, on a tree nothing of this environment has touched — so it is
# the last one that should have gone unrecorded, and it did.
[doc("The gate in a container on a fresh clone, with this machine's environment taken away.")]
gate-fresh image="rust:1-bookworm":
    #!/usr/bin/env bash
    set -euo pipefail
    command -v docker >/dev/null || {
        echo "gate-fresh needs docker, which is not on PATH" >&2; exit 2; }
    corpus="$RCAD_STEP_CORPUS"
    [ -d "$corpus" ] || {
        echo "gate-fresh needs the STEP corpus at $corpus; just fetch-corpus builds one" >&2
        exit 2; }
    # **A clone carries commits, so uncommitted work is invisible to it.** Left
    # unguarded this is the worst answer a gate can give: a green for a tree
    # nobody asked about, indistinguishable from a green for the tree in front of
    # you. Refusing is the only honest option, because the alternative — copying
    # the dirty tree in — would stop this being a *fresh* clone, which is the one
    # thing the recipe is for.
    if [ -n "$(git status --porcelain --untracked-files=no)" ]; then
        echo "gate-fresh clones, so it would gate HEAD and not your changes:" >&2
        git status --short --untracked-files=no >&2
        echo "commit first — a WIP commit is enough, and the branch is still on disk" >&2
        exit 2
    fi
    # An untracked file is invisible to a clone for the same reason, but it is
    # also not part of the commit anyone is about to hand over, so gating without
    # it is right rather than wrong. What it must not do is pass *silently*: a
    # new `.rs` file nobody has `git add`ed is exactly the work a reader of this
    # green would assume was covered. So it is named, and the run carries on.
    untracked=$(git ls-files --others --exclude-standard)
    if [ -n "$untracked" ]; then
        echo "gate-fresh: not in the clone, because they are not committed —" >&2
        printf '  %s\n' $untracked >&2
        echo "  this gate is of HEAD; git add them if they were meant to be in it" >&2
    fi
    # The clone is made here rather than in the container so that the container
    # needs no credentials and no network for it. `--no-local` forces a real
    # object transfer: a local clone hardlinks into the hub's object store, and
    # a container writing through a hardlink would be writing into it.
    work=$(mktemp -d)
    # **A clean-up that cannot finish must not fail the gate.** The container is
    # root and writes `target/` as root, so the host cannot delete the tree it
    # has just gated — which turned a green run into an exit 1 the moment the
    # trap started firing at all. The `chown` at the end of the container script
    # is the fix; this is what is left when the container dies before reaching
    # it — killed, OOM, a docker error — and it still owns the same docker
    # daemon this recipe already required, so it reaches for a second, throwaway
    # container to do as root what the host cannot, before it names the tree and
    # gives up.
    cleanup() {
        rc=$?
        chmod -R u+w "$work" 2>/dev/null || true
        if ! rm -rf "$work" 2>/dev/null; then
            docker run --rm -v "$work:/w" '{{ image }}' rm -rf /w/rcad 2>/dev/null || true
            rm -rf "$work" 2>/dev/null || {
                echo "gate-fresh: $work is left behind, owned by the container's root" >&2
                echo "  docker run --rm -v $work:/w {{ image }} rm -rf /w/rcad  removes it" >&2
            }
        fi
        exit "$rc"
    }
    trap cleanup EXIT
    git clone --no-local --quiet --single-branch "$(git rev-parse --show-toplevel)" "$work/rcad"
    echo "gate-fresh: cloned $(git -C "$work/rcad" rev-parse --short HEAD) into a single-worktree tree"
    # `HOME` inside is a scratch directory, so nothing of this machine's
    # `~/.claude`, `~/.cargo` or `~/.rustup` is visible. `CI=true` is what
    # `mise.toml` reads to empty `RUSTC_WRAPPER`, so the build cache — which
    # lives in the `HOME` we just took away — is not asked for.
    #
    # **Run, not `exec`ed, because the green is written down after it.** `exec`
    # replaces this shell, so nothing could follow the container — and the
    # `trap` above could never fire either, leaving every run's clone behind.
    status=0
    docker run --rm -t \
        -v "$work/rcad:/rcad" \
        -v "$corpus:/corpus:ro" \
        -v rcad-gate-fresh-tools:/tmp/home/.local/share/mise \
        -w /rcad \
        -e HOME=/tmp/home \
        -e CI=true \
        -e RCAD_STEP_CORPUS=/corpus \
        -e CARGO_TERM_COLOR=always \
        -e RUST_BACKTRACE=1 \
        -e HOST_UID="$(id -u)" \
        -e HOST_GID="$(id -g)" \
        '{{ image }}' \
        bash -eu -c '
            mkdir -p /tmp/home
            # `check-corpus-selftest` refuses without a locale that collates by
            # dictionary order, because the collation bug it exists to catch is
            # invisible under a codepoint collation and three of its four cases
            # stay green with the bug in place. A bare image has only `C.UTF-8`,
            # which is exactly the wrong one. `ci.yml` installs this too; §79.
            apt-get update -qq
            apt-get install -y -qq locales >/dev/null
            sed -i "s/^# en_US.UTF-8/en_US.UTF-8/" /etc/locale.gen
            locale-gen >/dev/null
            # The clone is bind-mounted from the host and owned by the host
            # user, while this container is root, so git calls it "dubious
            # ownership" and refuses. That is not cosmetic here: the `justfile`
            # derives `RCAD_STEP_CORPUS` from a backtick `git rev-parse` at parse
            # time, and just evaluates it whether or not the variable is already
            # set — so with git refusing, *every* `just` command in this
            # container fails before any recipe runs.
            git config --global --add safe.directory /rcad
            curl -fsSL https://mise.run | sh
            export PATH="/tmp/home/.local/bin:$PATH"
            eval "$(mise activate bash --shims)"
            # Materialise the pinned toolchain before mise installs anything.
            # This is fault 1 from the top of `ci.yml`, and the first run of this
            # recipe reproduced it exactly: mise installs the eight `cargo:`
            # tools in parallel, each triggers rustup to fetch the version
            # `rust-toolchain.toml` pins, they race over one `.partial`, and
            # seven of the eight died — `cargo-nextest` won. That the fresh
            # environment found a fault this repo had already documented is the
            # best evidence the recipe does what it claims.
            rustup show
            mise install
            # The gate runs as root and everything it writes is root-owned, so
            # the tree goes back to the invoking user before this container
            # exits. The status is carried by hand rather than left to `set -e`
            # so that the handover happens whether the gate passed or failed: a
            # red run whose tree cannot be deleted is the same mess as a green
            # one, and the host has no way to undo it afterwards.
            gate_status=0
            mise exec -- just gate quiet || gate_status=$?
            chown -R "$HOST_UID:$HOST_GID" /rcad
            exit $gate_status
        ' || status=$?
    [ "$status" -eq 0 ] || exit "$status"

    # **The green goes in the host's record, because the container's dies with
    # it.** `gate` in there reaches `_record-green` too, and writes into
    # `/rcad/.git` — the clone the `trap` deletes. Without this line the
    # strongest gate this machine can run would be the only one leaving no
    # trace, while a plain `just gate` in a warm tree counts.
    # `fuzz-watermark` reads that record to choose what the night fuzzes, and
    # since §172 spent the Actions allowance it is the only source that can
    # still answer. §173.9.
    #
    # The sha is taken here and not carried out of the clone, and it is the same
    # sha: this recipe refuses a dirty tree and the clone is of HEAD.
    # `_record-green` re-reads that for itself, which is what catches a tree
    # written to during the ten minutes the container was running.
    #
    # Both directories named rather than left to `just`'s defaults, so the tree
    # the record describes is the one `git clone` above read, whichever
    # directory of it this recipe was invoked from.
    {{ just_executable() }} --justfile "{{ justfile() }}" \
        --working-directory "$(git rev-parse --show-toplevel)" \
        _record-green "{{ gate_steps }}"

# The whole gate, and the same command CI runs. Named `gate` rather than
# `test-all` because it is two dozen checks, a lint and a `cargo-deny` around
# exactly one `test` — and this repo already says "gated green", "a gate agent",
# "the gate worktree". `test-all` would have shipped a false sentence on day one.
#
# **It ends with a summary block, and that is what the nine minutes are for.**
# In one week 145 of 415 gate runs were followed by a second call that grepped
# the redirected log for `Summary|FAIL|tests run` — a mean of 159k tokens of
# context to read back three numbers this recipe already had in hand. So it says
# them itself, last, on stdout: pass or fail, the counts, and the name of the
# first thing that broke.
#
# The exit code is untouched — CI reads it and nothing else.
#
# Each step's stderr is folded into stdout so that `tee` keeps one stream in the
# order it happened. That is the shape a redirected run already had, and the
# block is parsed back out of it.
#
# **`just gate quiet` is the mode a tool call wants; bare `gate` is the mode a
# terminal wants.** The block above was meant to end the second call that reads
# the log back, and it did not, because a whole run is 300 KB — 75k tokens — and
# no session can let that into a tool result. So 262 of the 272 gate runs in the
# week of 2026-09-12 redirected stdout to a file, which is the right thing to do
# with 300 KB and throws the block away with it; 122 of them then paid a second
# whole-context call to read back the eight lines already printed. `quiet` puts
# one line per step, the tail of whatever broke, and the block on stdout, and
# sends the rest to the log — so one unredirected call answers the question.
# `docs/decisions.md` §147.
[doc("Everything, in the order CI runs it. `just gate quiet` prints only the summary.")]
gate mode="loud":
    #!/usr/bin/env bash
    set -uo pipefail
    mode='{{ mode }}'
    # Named rather than defaulted, so a typo is a refusal and not a loud run.
    case "$mode" in
        loud|quiet) ;;
        *) echo "just gate takes 'quiet' or nothing, not '$mode'" >&2; exit 2 ;;
    esac
    log='{{ gate_log }}'
    mkdir -p "$(dirname "$log")"
    : >"$log"
    failed=""
    status=0
    started_at=0
    for step in {{ gate_steps }}; do
        if [ "$mode" = quiet ]; then
            # The one thing a truncated result still has to answer is how far
            # the run got, which is why this costs a line even though nothing
            # reads it until something goes wrong.
            echo "· $step"
            # Where this step's own output begins. A plain `tail` of the log
            # would spend its lines on the step before, which on a gate whose
            # previous step was `list` is forty lines of recipe names and one
            # line of the error that actually stopped it.
            started_at=$(wc -c <"$log")
            {{ just_executable() }} "$step" >>"$log" 2>&1
            step_status=$?
        else
            {{ just_executable() }} "$step" 2>&1 | tee -a "$log"
            # Read before anything else runs. `[` is a command, so it rewrites
            # `PIPESTATUS` with its own 0 — which read as a clean gate over a step
            # that had just failed, the first time this loop was run.
            step_status=${PIPESTATUS[0]}
        fi
        if [ "$step_status" -ne 0 ]; then
            failed="$step"
            status=$step_status
            break
        fi
    done
    # Only in `quiet`, and only before the block: `loud` has already shown these
    # lines once, and the block stays last because `tail -6` is where a reader
    # looks for it. Capped, because a step can fail with a megabyte behind it
    # and `full log:` names where the rest is.
    if [ "$mode" = quiet ] && [ "$status" -ne 0 ]; then
        echo
        echo "=== last 60 lines of $failed ==="
        tail -c "+$((started_at + 1))" "$log" | tail -60
    fi
    {{ just_executable() }} _gate-summary "$log" "$status" "$failed" | tee -a "$log"
    # After the block, so that a reader's `tail -6` still lands on the summary,
    # and only on a pass: what it records is that this commit was read end to
    # end. §173.9 — it is what moves the overnight fuzz now that no push can be
    # CI-green.
    if [ "$status" -eq 0 ]; then
        {{ just_executable() }} _record-green "{{ gate_steps }}"
    fi
    exit $status

# The block itself, kept apart from the run so `check-gate-summary-selftest` can
# hold it against logs whose answer is already known — the same split `t` and
# `_test-digest` have, and for the same reason.
#
# **The escape codes come off before anything is grepped.** `ci.yml` sets
# `CARGO_TERM_COLOR: always`, so on a runner nextest's summary line arrives as
# `\e[1m     Summary\e[0m ...` and an anchored pattern sees a colour code where
# it wanted a space. That is not a guess: it is what turned two CI runs red in
# `check-wildcards`, which anchored its grep the same way (docs/decisions.md
# §77.6).
#
# **A missing count reads as `not reached`, never as zero.** A gate that died in
# `check-enums` never got to `test`, and a block that printed `0 tests run`
# there would be describing the gate's own failure as a clean empty suite.
#
# **Every count line is reported, never the last one.** The gate has two nextest
# runs — `test` and `check-release-build` — and eight `test result:` lines, one
# per crate, out of `cargo test --doc`. Reading the last of each got `3 tests
# run` over a suite of 1869 and `2 passed` over 24 doc tests, on a real green
# gate. A number that small and that wrong is worse than none, because it reads
# as a real total. So the `Summary` lines are all printed, each under its own
# `tests:` label, and the doc tests are added up.
[private]
_gate-summary log status failed="":
    #!/usr/bin/env bash
    set -uo pipefail
    plain=$(sed 's/\x1b\[[0-9;]*m//g' '{{ log }}' 2>/dev/null)
    tests=$(grep -E '^ *Summary ' <<<"$plain")
    doctests=$(awk '/^test result: /{passed += $4; failed += $6; crates += 1} \
                    END {if (crates) printf "%d passed, %d failed, across %d crate(s)", passed, failed, crates}' \
                    <<<"$plain")
    echo
    echo "=== gate summary ==="
    if [ '{{ status }}' = "0" ]; then
        echo "gate: PASS"
    else
        echo "gate: FAIL  first failure: {{ failed }} (exit {{ status }})"
        first=$(grep -m1 -E '^ *FAIL \[' <<<"$plain" | sed 's/^ *//')
        [ -z "$first" ] || echo "first failing test: $first"
    fi
    if [ -z "$tests" ]; then
        echo "tests: not reached"
    else
        sed 's/^ *//; s/^/tests: /' <<<"$tests"
    fi
    echo "doc tests: ${doctests:-not reached}"
    echo "full log: {{ log }}"

# Writes down that this commit has been read end to end, so that something other
# than GitHub can say so.
#
# `fuzz-watermark` moves the overnight run onto a commit CI went green on, and
# §172's spent Actions allowance means no commit can be green in that sense any
# more. Left alone the night keeps fuzzing the last one that was — silently,
# because staying put is how that recipe is built to fail, and it was 18 commits
# behind within a day. **The condition §81.4 asked for is that nothing fuzzes a
# tree nobody has read end to end, and a passing `just gate` is exactly such a
# reading.** CI supplied that fact; it was never the only thing that could.
#
# Three conditions, and each is a way the record could otherwise lie:
#
#   - **the metered steps all ran.** `gate-docs` re-enters `gate` with a trimmed
#     list, so it lands here as a pass having compiled nothing at all. A green
#     that skipped `test` is not a reading of the code, and this is the one
#     mistake that would be invisible afterwards — the file would just name a
#     commit, with no trace of how little was run for it.
#   - **the tree is clean**, because the sha is the whole record. A green
#     recorded against a dirty tree names something nobody can get back.
#   - **it never fails the gate.** Bookkeeping that can turn a green run red is
#     worse than no bookkeeping; every giving-up path here says why and exits 0.
#
# The file sits in the common git dir, so every worktree writes to one list and
# the hub does not need to be checked out to have one. Untracked by construction:
# it is a fact about this machine, and a commit's greenness is not the sort of
# thing to carry in the tree it describes. `docs/decisions.md` §173.9.
[private]
_record-green steps:
    #!/usr/bin/env bash
    set -uo pipefail
    for heavy in {{ gate_metered_steps }}; do
        case " {{ steps }} " in
            *" $heavy "*) ;;
            *) echo "green not recorded: this run skipped $heavy, so it read no code"; exit 0 ;;
        esac
    done
    git diff --quiet 2>/dev/null && git diff --cached --quiet 2>/dev/null || {
        echo "green not recorded: the tree has uncommitted changes, so a sha would not name it"
        exit 0
    }
    sha=$(git rev-parse HEAD 2>/dev/null) || { echo "green not recorded: no HEAD here"; exit 0; }
    common=$(git rev-parse --path-format=absolute --git-common-dir 2>/dev/null) \
        || { echo "green not recorded: no git dir here"; exit 0; }
    file="$common/rcad-greens"
    if grep -q "^$sha " "$file" 2>/dev/null; then
        echo "green already recorded for $(git rev-parse --short HEAD)"
        exit 0
    fi
    printf '%s %s %s\n' "$sha" "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "$(hostname)" >>"$file"
    # Bounded, because nothing ever deletes from it and the readers are loops.
    # The tail is what a watermark could still move onto; the head is history no
    # night will ever walk back to.
    if [ "$(wc -l <"$file")" -gt 500 ]; then
        tail -200 "$file" >"$file.new" && mv "$file.new" "$file"
    fi
    echo "green recorded for $(git rev-parse --short HEAD) in $file"

# Holds the block and the loop that drives it against fixtures whose right
# answer is known, rather than against whatever the gate happens to do today.
#
# **The loop is exercised by moving `gate_steps`, not by running the real gate.**
# This selftest is itself a step of the gate, so a nested real run would not
# terminate. `--set` hands the driver a step list one of whose words is not a
# recipe at all, which is the cheapest failing step there is and needs no
# fixture recipe of its own.
#
# The five ways this can be wrong each get a fixture: a pass that does not print
# the block, a failure that prints no block at all (which is the whole defect
# this recipe exists to remove), a failure that does not name the step that
# broke, a log whose counts are behind colour codes, and a doc-test total read
# off one crate's line rather than added up.
#
# `quiet` gets four more, because the thing it promises is what it leaves out: a
# quiet pass that carried a step's own output into the result would be no better
# than a loud one, and a quiet failure that left the failing lines in the log
# would send the reader back for the second call this whole mode exists to
# remove. The fourth is the typo — a mode that is neither word must refuse,
# because a `quiet` misspelt into a 300 KB result is the defect wearing a
# different hat.
[private]
check-gate-summary-selftest:
    #!/usr/bin/env bash
    set -euo pipefail
    dir=$(mktemp -d)
    trap 'rm -rf "$dir"' EXIT
    j='{{ just_executable() }}'

    fail() {
        echo "the gate summary drifted from its own fixtures: $1"
        echo "--- got ---"; echo "$got"
        exit 1
    }

    # A pass. `list` is a real recipe that costs nothing, so the driver runs
    # something real without the gate running itself.
    got=$($j --set gate_steps list --set gate_log "$dir/pass.log" gate 2>&1) \
        || fail "a gate whose every step passed exited non-zero"
    [ "$(tail -6 <<<"$got" | grep -c '=== gate summary ===')" -eq 1 ] \
        || fail "the block is not at the tail, which is the one place a redirected run is read"
    grep -q '^gate: PASS$' <<<"$got" || fail "a passing gate did not say so"
    grep -q '^tests: not reached$' <<<"$got" \
        || fail "a run that never reached the suite reported a count anyway"

    # A failure. The step list names something that is not a recipe, so `just`
    # exits non-zero on it, and the two steps after it must not run.
    got=$($j --set gate_steps 'list no-such-step list' --set gate_log "$dir/fail.log" gate 2>&1) \
        && fail "a gate with a failing step exited 0"
    grep -q '=== gate summary ===' <<<"$got" \
        || fail "a failing gate printed no block, which is the defect this exists to remove"
    grep -q '^gate: FAIL  first failure: no-such-step' <<<"$got" \
        || fail "the block does not name the step that broke"
    [ "$(grep -c 'Everything this repo can do' "$dir/fail.log")" -eq 1 ] \
        || fail "the gate carried on past a failing step"

    # A quiet pass. The step still runs and the log still gets everything; what
    # must not appear on stdout is the step's own output.
    got=$($j --set gate_steps list --set gate_log "$dir/quiet-pass.log" gate quiet 2>&1) \
        || fail "a quiet gate whose every step passed exited non-zero"
    grep -q '^gate: PASS$' <<<"$got" || fail "a passing quiet gate did not say so"
    grep -q '^· list$' <<<"$got" || fail "a quiet gate did not say which step it was on"
    grep -q 'Everything this repo can do' <<<"$got" \
        && fail "a quiet gate put a step's own output in the result, which is the whole thing it exists not to do"
    grep -q 'Everything this repo can do' "$dir/quiet-pass.log" \
        || fail "a quiet gate dropped a step's output instead of logging it"

    # A quiet failure. Naming the step is not enough here: loud mode has already
    # shown the lines that explain it and quiet mode has not, so they come out
    # with the block — and the block stays last.
    got=$($j --set gate_steps 'list no-such-step list' --set gate_log "$dir/quiet-fail.log" gate quiet 2>&1) \
        && fail "a quiet gate with a failing step exited 0"
    grep -q '^gate: FAIL  first failure: no-such-step' <<<"$got" \
        || fail "a quiet failing gate does not name the step that broke"
    grep -q '=== last 60 lines of no-such-step ===' <<<"$got" \
        || fail "a quiet failure sent the reader back to the log for the lines that explain it"
    grep -q "Justfile does not contain recipe" <<<"$got" \
        || fail "the failing tail named the step but carried none of its output"
    grep -q 'Everything this repo can do' <<<"$got" \
        && fail "the failing tail spent its lines on the step before the one that broke"
    [ "$(tail -6 <<<"$got" | grep -c '=== gate summary ===')" -eq 1 ] \
        || fail "the failing tail pushed the block off the end, where no reader looks for it"

    # A mode that is neither word. Defaulting it to `loud` would answer a typed
    # `quite` with the 300 KB this mode exists to avoid.
    got=$($j --set gate_steps list --set gate_log "$dir/mode.log" gate quite 2>&1) \
        && fail "a misspelt mode ran the gate anyway"
    grep -q "not 'quite'" <<<"$got" || fail "a misspelt mode did not say what it was given"

    # The counts, read back out of a log written the way a runner writes one:
    # `CARGO_TERM_COLOR: always`, so every line arrives wrapped in escape codes.
    printf '\033[1m     Summary\033[0m [  12.345s] 1811 tests run: 1811 passed, 0 skipped\n' >"$dir/counts.log"
    printf 'test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out\n' >>"$dir/counts.log"
    printf 'test result: ok. 9 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out\n' >>"$dir/counts.log"
    printf '     Summary [   0.007s] 3 tests run: 3 passed, 0 skipped\n' >>"$dir/counts.log"
    got=$($j _gate-summary "$dir/counts.log" 0)
    grep -q '^tests: Summary \[  12.345s\] 1811 tests run: 1811 passed, 0 skipped$' <<<"$got" \
        || fail "the test counts did not survive the colour codes a runner writes"
    grep -q '^tests: Summary \[   0.007s\] 3 tests run: 3 passed, 0 skipped$' <<<"$got" \
        || fail "only one of the gate's two nextest runs was reported"
    grep -q '^doc tests: 24 passed, 1 failed, across 2 crate(s)$' <<<"$got" \
        || fail "the doc tests were read off one crate's line instead of added up over all of them"

    # A failing suite names the first failing test as well as the step, because
    # `test` is one step and the answer wanted under it is which test.
    printf '        FAIL [   0.011s] rcad-geom a_plane_and_a_sphere\n' >>"$dir/counts.log"
    got=$($j _gate-summary "$dir/counts.log" 1 test)
    grep -q '^first failing test: FAIL \[   0.011s\] rcad-geom a_plane_and_a_sphere$' <<<"$got" \
        || fail "a failing suite did not name the first test that failed"

# Holds `gate-docs` to the two things it claims: that it drops the compiles, and
# that it still runs everything a prose commit can break.
#
# **The second is the arm that matters, and it is the one nothing else guards.**
# Dropping a step too few costs money. Dropping a step too many means a
# `docs/decisions.md` commit lands on `main` having been checked by nothing that
# reads `docs/decisions.md` — and it would pass, loudly, printing a summary
# block. So `check-decisions`, `check-plan` and `check-where-the-work-is` are
# named here: move one of them into `gate_metered_steps` and this fails.
#
# The filter is driven through `--set` against fixture lists rather than against
# the real ones, for the same reason `check-gate-summary-selftest` is: this
# recipe is itself a step of the docs arm, so a nested real run would not
# terminate.
#
# Both directions are exercised. A filter that dropped nothing and a filter that
# dropped everything are both "the list came out wrong", and only one of them is
# visible from a run that passes.
[private]
check-gate-docs-selftest:
    #!/usr/bin/env bash
    set -euo pipefail
    dir=$(mktemp -d)
    trap 'rm -rf "$dir"' EXIT
    j='{{ just_executable() }}'
    got=""

    fail() {
        echo "gate-docs drifted from what it promises: $1"
        [ -n "$got" ] && { echo "--- got ---"; echo "$got"; }
        exit 1
    }

    # Interpolated rather than grepped out of the justfile. `gate_steps` is one
    # step per source line now (§177), and a regex anchored at `^` reads the
    # first name and stops — which is exactly what it did, loudly, the moment
    # the list was split. What `just` hands over cannot disagree with what
    # `gate-docs` reads, because it is the same substitution.
    all=$(printf '%s\n' {{ gate_steps }})
    heavy=$(printf '%s\n' {{ gate_metered_steps }})
    # An empty list would make every comparison below pass against nothing.
    # `check-gate-runs-every-check` says the same thing about itself.
    [ "$(grep -c . <<<"$all")" -ge 30 ] || fail "gate_steps came back empty"
    [ "$(grep -c . <<<"$heavy")" -ge 5 ] \
        || fail "gate_metered_steps came back empty"

    # A name in the exclusion list that is not a step excludes nothing, and says
    # so nowhere. That is how this list goes stale when a step is renamed.
    while read -r step; do
        grep -qx "$step" <<<"$all" \
            || fail "gate_metered_steps names '$step', which is not a step of the gate"
    done <<<"$heavy"

    docs_arm=$(grep -vxF -f <(printf '%s\n' "$heavy") <<<"$all")

    # What a prose commit can actually break. Named one by one rather than
    # counted, because the count would survive the swap that matters.
    for needed in check-decisions check-decisions-selftest check-claim-section-selftest \
                  check-plan check-plan-selftest check-where-the-work-is; do
        grep -qx "$needed" <<<"$docs_arm" \
            || fail "$needed is not in the docs arm, so a docs commit would be checked by nothing that reads docs"
    done

    # And the other half of the bargain: if these survived, the arm buys nothing.
    #
    # `check-corpus` is in this list for a different reason from the rest, and it
    # is the one that would not have been noticed by reading. It compiles nothing
    # and takes 0.16 s, and it **fails** when `RCAD_STEP_CORPUS` is not a
    # directory (§44.10) — so a docs arm that ran it would have gone red on the
    # runner, every time, until somebody added a 119 MiB fetch to the cheap job.
    for costly in check lint test check-release-build check-corpus; do
        grep -qx "$costly" <<<"$docs_arm" \
            && fail "$costly is still in the docs arm, which is the whole cost it exists to drop"
    done
    # The selftest beside it must stay, though: it builds its own fixture, so it
    # is the arm that would catch the collation bug in a prose-only commit.
    grep -qx check-corpus-selftest <<<"$docs_arm" \
        || fail "check-corpus-selftest left the docs arm, and it needs no corpus to run"

    # The filter itself. `list` is a real recipe that costs nothing; the word
    # beside it is not a recipe at all, which is the cheapest failing step there
    # is and needs no fixture of its own.
    got=$($j --set gate_steps 'list no-such-step' --set gate_metered_steps 'no-such-step' \
            --set gate_log "$dir/drop.log" gate-docs 2>&1) \
        || fail "the docs arm did not drop the step it was told to drop"
    grep -q '^gate-docs: dropped no-such-step$' <<<"$got" \
        || fail "the docs arm dropped a step without saying which"

    # The same fixture with the lists swapped. A non-zero exit alone would prove
    # nothing here — dropping every step also exits non-zero — so this checks
    # which step was dropped and which one the summary blames.
    got=$($j --set gate_steps 'list no-such-step' --set gate_metered_steps 'list' \
            --set gate_log "$dir/keep.log" gate-docs 2>&1) \
        && fail "the docs arm kept a failing step and exited 0, so it is filtering nothing"
    grep -q '^gate-docs: dropped list$' <<<"$got" \
        || fail "the docs arm dropped something other than the one step it was told to"
    grep -q '^gate: FAIL  first failure: no-such-step' <<<"$got" \
        || fail "the step the docs arm was told to keep never ran"

    # Every step excluded. Running no steps exits 0 through the loop in `gate`,
    # prints a passing summary block, and has checked nothing whatsoever.
    got=$($j --set gate_steps 'list' --set gate_metered_steps 'list' \
            --set gate_log "$dir/empty.log" gate-docs 2>&1) \
        && fail "an empty docs arm passed by checking nothing"
    grep -q 'no steps at all' <<<"$got" \
        || fail "an empty docs arm failed without saying that it had nothing to run"

# The STEP corpus lives outside the tree, because it is other people's CAD files
# and this repo has no business carrying them. Seven test files in `rcad-step`
# read this variable and **fail** rather than skip when the corpus is not there
# (§44.10): a green has to mean the corpus was read.
#
# Because this file defaults the variable, nothing that runs through `just` ever
# reaches those tests' "not set" panic — it trips their `root.is_dir()` assertion
# one line later, naming a path that looks plausible and is not there. A bare
# `cargo nextest run` gets the panic instead. Both are failures, which is the
# point; they just name different things, and the plausible path is the one that
# wastes an afternoon.
#
# Derived rather than hardcoded so that every worktree gets the same answer:
# `--git-common-dir` points at the *main* repo's `.git` from inside a worktree,
# so two `parent_directory` calls land beside the repo wherever it is run from.
# Before this, lanes exported it by hand — 412 times on 2026-09-05 — and the
# ones that forgot got four silent skips rather than a failure.
export RCAD_STEP_CORPUS := env_var_or_default("RCAD_STEP_CORPUS", parent_directory(parent_directory(`git rev-parse --path-format=absolute --git-common-dir`)) / "rcad-step-corpus")

[doc("Type-check the whole workspace, tests and benches included.")]
check:
    cargo check --workspace --all-targets

# The fuzz targets compile, which `check` above cannot tell you.
#
# `fuzz/Cargo.toml` declares its own empty `[workspace]`, deliberately: building
# a target needs a nightly compiler and a sanitizer, and `rust-toolchain.toml`
# pins stable. The cost of that was silence. `no_step_file_makes_the_reader_panic.rs`
# stopped compiling when §86 made `Model::solid` return a `Result`, and nothing
# said so until a night of fuzzing had already been spent — the §77.6 shape, a
# check passing for a property of the environment it ran in.
#
# **Type-checking needs none of that.** The nightly compiler and the sanitizer
# are for *linking* an instrumented binary; `cargo check` on stable reads the
# same four targets and finds the same break, in six seconds. So the gap was
# never that this could not be checked, only that nothing checked it.
#
# Run from `fuzz/` because it is its own workspace; `--all-targets` to reach the
# `fuzz_targets/` binaries rather than just the empty lib.
[doc("The four fuzz targets still compile, which the workspace check cannot see.")]
check-fuzz-builds:
    cd fuzz && cargo check --all-targets

# Every public enum either grows or says why it does not (docs/decisions.md §40).
#
# `#[non_exhaustive]`, or a doc comment saying "Exhaustive on purpose" — which
# `Orientation` is, and is the only one (§3.6). A new public enum written
# without either compiles, passes every test, and nothing else would notice.
#
# A `pub enum` is judged against what stands between it and the blank line above
# it, which is where rustfmt puts the item's own attributes and doc comment.
# How many wildcard match arms stand over one of our own enums, or over an
# array, a tuple, or a reference of them (docs/decisions.md §40). Raise or
# lower it only with the commit that does.
#
# Two mechanisms sit behind this one number, because they see different
# things. `clippy::wildcard_enum_match_arm` asks only what the scrutinee's
# own type is, so a match over `Surface` is seen and a match over
# `[Surface; 2]` or `(&Surface, &Surface)` is not — the scrutinee there is an
# array or a tuple, and clippy asks nothing about what is nested inside one.
# Two real defects hid in exactly that gap this milestone: a panic reachable
# from a real KiCad file through `Import::read`, and a fourth list of
# surface pairs that had drifted out of step with the other three. Both
# matched over a composite rather than over one enum.
#
# `crates/rcad-step/tools/composite_wildcards.awk` is the second mechanism: a
# line-based scan for an arm that names one of our enum's variants inside a
# `[`, a `(`, or a `&`, standing in the same match as a bare `_`. It counts how
# many of that enum's own variants the match never names anywhere in its arms
# — not the match itself — so a fully-named composite match, the same as a
# fully-named single-enum one, counts zero. The proper mechanism for this is
# rustc's own `non_exhaustive_omitted_patterns`, unstable on the 1.96 this repo
# pins; the script's own header comment says so first.
#
# `check-wildcards-selftest`, below, pins two small fixtures the scan must
# catch and two it must not, against a made-up enum rather than one of ours —
# so the scan keeps being proven even on a tree where nothing real trips it.
#
# A count and not a denial: rustc resolves a lint level at the enclosing
# expression, so `#[allow]` on a match arm never reaches it and the only
# granularity clippy leaves is the whole function; the text scan has no
# per-site silencing at all.
#
# The number moves for three reasons and all three want looking at. A new
# wildcard raises it, of either kind. So does a new *variant*, because a site
# that names every variant there is today starts matching a known one
# tomorrow — which is the whole point of naming them. That used to be true of
# the clippy half only; §43.8 item 8 found the composite half counted sites
# and not variants, so a fourth surface-pair list could drift out of step with
# the other three and this number would never say so. It counts uncovered
# variants now, and moves with a new one on the composite side too.
#
# It falls when a refusal becomes an answer, which is the fourth reason and the
# happiest one: a test that matched an `Error` it expected, with `other =>
# panic!` for the rest, stops matching anything once the case it named is
# answered. That is what took it from 88 to 87 — a block corner on a ball's
# skin, which the mesher used to refuse.
#
# **A fifth reason, and the one that makes this number not a ratchet: the
# instrument changed.** 90 → 87 over the unit called "the door" looks like three
# arms going away and is not. Measured both ways on both trees:
#
#     old awk, before   82 direct + 8 composite = 90
#     new awk, before   82 direct + 6 composite = 88
#     old awk, after    81 direct + 8 composite = 89
#     new awk, after    81 direct + 6 composite = 87
#
# So the composite half moved **entirely** because the scan started counting
# uncovered variants instead of sites, and not at all because the tree did. Only
# one point of the three — the direct half's 82 → 81 — is a real arm going away.
# A ratchet that resets its own instrument in the same commit range as the tree
# has lost its baseline, so the bridging figures are written here rather than
# left to be re-derived. Whoever changes the scan again owes the same four rows.
#
# **87 → 89 over the unit called "the endless bend", and every point of it was
# read.** Measured on three commits with one instrument:
#
#     06bc048  the unit's own starting main   81 direct + 6 composite = 87
#     198de07  after E, F and C merged        81 direct + 5 composite = 86
#     2c236f9  after A and B merged           84 direct + 5 composite = 89
#
# The composite point is the happy fourth reason: `is_on_a_knife_edge` in
# `the_same_pair_turned_names_the_same_kind_of_crossing.rs` grew a ball-and-pipe
# arm, so that match leaves two of `Surface`'s variants unnamed where it left
# three. The three direct points are all in `intersect.rs` and all belong to
# naming the coaxial cone and pipe: two are the `other => unreachable!("a sum of
# squares is never below zero")` arms `#[non_exhaustive]` forces at the two `ask`
# call sites in `where_a_cone_and_a_pipe_cross`, which is what twenty-three of the
# other twenty-four direct arms in that file already are — the scan counts 26 in
# `intersect.rs`, 25 of them `unreachable!` or `panic!`; the third is
# `leaning => leaning` in `whether_a_cone_and_a_pipe_share_an_axis`, a
# pass-through that hands any answer but `Zero` straight back. It is safe by its
# caller's shape and not by the enum's: `walk`'s cone-and-pipe cut asks
# `!= Decision::Zero` and refuses, so a variant `Decision` grows later lands on
# the refusing side. `same => same` in `Picker::side_of_point`
# (`half_space.rs:441`) is **not** the precedent for it — there `Zero` and
# `Undecidable` both genuinely mean "not on a side", and here they do not.
#
#     a2d4df5  after the measurement lanes   85 direct + 5 composite = 90
#
# The one new point is a **test-only label mapper**, and it is the case where a
# wildcard is right rather than tolerated: `what_the_refusal_was_called` in
# `whether_a_coarser_deflection_always_meshes.rs` turns a refusal into a short
# string for a report, and `rcad_topo::Error` has **64 variants it does not
# name**. Naming all 64 to satisfy the letter of §40 would be worse code than the
# wildcard, and the arm decides nothing — it returns "an unexpected refusal" and
# the test's real assertions are on the refusal kinds it *does* name. §40's
# hazard is a wildcard swallowing a variant that should have changed an answer;
# nothing here answers anything.
#
#     b51b39d  after the reach fix            86 direct + 5 composite = 91
#
# The one new point is `how_far_along_the_axis_the_other_surface_reaches` in
# `mesh.rs`. It asks the surface on the other side of a crossing how far along
# our own axis a point on both of them can be, and it names all five `Surface`
# variants there are — the `_` is only there because `#[non_exhaustive]` forces
# it. It answers `None`, which the caller reads as "the corners are all there
# is": a floor that is too low on a surface kind nobody has taught it yet, which
# is what every surface got before this function existed. A `panic!` there would
# refuse to mesh a shape the kernel could have cut, and that is the worse of the
# two.
#     lane-non-monotone  after the placement pin  87 direct + 5 composite = 92
#
# The one new point is `what_it_refused` in
# `how_far_out_a_ball_can_sit_and_still_mesh.rs`, and it is the same case as the
# one at 90: a test-only label mapper over `rcad_topo::Error`, naming the five
# refusals that file has actually seen and answering "an unexpected refusal" for
# the rest. Reaching the arm is the finding, not the pass.
#     lane-record-enclosure  after §73's narrowing  83 direct + 5 composite = 88
#
# Four go away and none arrive. Three are `holds.rs` 318, 377 and 550, where a
# `Decision` reached `gave_up_…` through a wildcard; `Record::new` takes an
# `f64` now, so each arm has to name `Undecidable` to get one, and the wildcard
# below it can only be reached by a variant that does not exist yet
# (`docs/decisions.md` §73.3). The fourth is `validate.rs:281`, and it was a
# false positive the ledger was carrying: the arm above it read
# `decision @ Decision::Undecidable { .. }`, and clippy does not see a `@`
# binding as covering the variant. Written plainly as
# `Decision::Undecidable { enclosure }` — which is what the narrowing wanted
# anyway — the same match stops warning. The arm itself did not move.
#
#     ef7cf664  the pipe-convention conversion  83 direct + 3 composite = 86
#
# Two go away and none arrive. **Both are composite and neither is a new kind of
# movement**: each was a `_ => return None` over `Decision` inside a tuple, in
# `intersect.rs`, leaving `Undecidable` the one variant its match never named.
# `how_two_squares_compare_exactly` is gone outright — with a pipe `r` wide
# rather than `r/‖w‖` wide, what two leaning pipes ask clears to `(r₁ ± r₂)²K`
# against `T²` and no longer needs the five-hundred-order expansion that
# function existed to square (`docs/decisions.md` §118.11). The other is in
# `how_two_radii_and_a_gap_compare_exactly`, where the radii's sign was read off
# an `Exact` difference and is now `other.radius().partial_cmp(&one.radius())`:
# both radii mean the same thing, so the sign of the difference *is* the sign of
# the comparison. `Ordering` is not one of ours, so the scan stops counting that
# site at all. The direct half does not move.
#
#     lane-settle-the-turned-pair  after the resting ball  83 direct + 4 composite = 87
#
# The one new point is the mirror of the composite point at 89: `is_on_a_knife_edge`
# in `the_same_pair_turned_names_the_same_kind_of_crossing.rs` **loses** the
# ball-and-pipe arm it grew there, so that match leaves three of `Surface`'s
# variants unnamed where it left two. A ball resting all the way round the
# inside of a pipe is not a knife edge any more — §118.11 leaves both radii
# untouched by a turn, so the resting survives it — and an arm that skips a pair
# the property can now compare is a hole in the property. Two more quartic pairs
# get compared for the point.
#
#     §191's three test lanes  after the merge  84 direct + 4 composite = 88
#
# One new direct arm, in test code. `fuses_that_touch.rs`'s `named` grew one
# that only singled out `Undecidable`, and it is a `let … else` now; what is
# left is a match that names the three variants it is asked about and hands
# everything else on — `Rule`'s three fuses in `name_of`, or
# `a_turned_curved_refusal` handing every other `Error` to `a_curved_refusal`,
# where the upright families' names live.
#
#     §194's ball-and-block lane  after the merge  85 direct + 7 composite = 92
#
# Four more, all in the cut of a ball-and-plane circle that starts at a pole.
# The direct one is `which_half_of_a_circle_cut_at_a_pole`'s material side,
# which panics on `Zero` or `Undecidable` as `which_arc_this_is` beside it does.
# The composite three are one site, `across_a_circle_that_starts_at_a_pole`'s
# pair of surfaces, which names the ball and the plane and leaves `Surface`'s
# other three kinds to `_`. A pair that is not a ball and a plane is not cut,
# and `None` there leaves the fuse where it was before — refusing by
# `AClosedEdgeOfTheAnswerWouldStartAtAPole` when the start is a pole.
#
#     §195's polar-axis lane  after the merge  84 direct + 7 composite = 91
#
# One fewer. `a_ball_cut_by_a_plane_down_its_polar_axis_is_refused_by_name`
# matched its refusal and sent every other `Error` to `panic!`; the cut down the
# polar axis now answers, and the test that replaced it has no match at all.
wildcard_arms := "97"

[doc("Count wildcard match arms over our own enums; fails when the count moves.")]
check-wildcards:
    #!/usr/bin/env bash
    set -uo pipefail
    find crates -name lib.rs -path '*/src/*' -exec touch {} +
    mkdir -p target
    log=target/last-wildcard-clippy.log
    if ! CARGO_TERM_COLOR=never cargo clippy --workspace --all-targets -- \
            -W clippy::wildcard_enum_match_arm > "$log" 2>&1; then
        grep '^error' "$log" | head -5
        echo "clippy did not finish, so this counted nothing"
        echo "full log: $log"
        exit 1
    fi
    direct=$({{ just_executable() }} _wildcard-direct-count "$log")
    files=$(find crates -name '*.rs' \( -path '*/src/*' -o -path '*/tests/*' \
            -o -path '*/benches/*' -o -path '*/examples/*' \))
    enums=$(grep -hoE '^[ \t]*pub enum [A-Za-z0-9_]+' $(find crates -path '*/src/*' -name '*.rs') \
            | awk '{print $NF}' | sort -u | paste -sd'|')
    composite=$(awk -v enums="$enums" -f crates/rcad-step/tools/composite_wildcards.awk $files \
            | grep '^COUNT=' | cut -d= -f2)
    found=$((direct + composite))
    if [ "$found" = "{{wildcard_arms}}" ]; then
        exit 0
    fi
    echo "wildcard arms over our own enums: $found ($direct direct, $composite composite), expected {{wildcard_arms}}"
    if [ "$direct" = "0" ] && [ "{{wildcard_arms}}" != "0" ]; then
        echo "0 direct is the parse breaking, not the arms going away: clippy ran and"
        echo "nothing it printed was matched. Read $log before touching the ledger."
        exit 1
    fi
    echo "a new one, or a new variant reaching an old one — see docs/decisions.md §40"
    exit 1

# The direct half of the count, kept apart from the clippy run so
# `check-wildcards-selftest` can hold it against logs whose answer is already
# known — the same split as `t` and `_test-digest`, for the same reason.
#
# ANSI escapes come off before anything is matched. `ci.yml` sets
# `CARGO_TERM_COLOR=always`, which puts an escape in front of every `warning:`,
# and every `^`-anchored pattern here then matches nothing at all. Run
# 34264306999 counted 0 of 83 that way and took `main` red while this machine
# stayed green, because cargo colours nothing when its stdout is a `$(...)`
# rather than a terminal. Stripping is what makes the count the same answer
# either way; `CARGO_TERM_COLOR=never` above is what keeps the saved log
# readable when a human opens it.
_wildcard-direct-count log:
    @sed 's/\x1b\[[0-9;]*m//g' '{{ log }}' | grep -c '^warning: wildcard match' || true

# Pins the composite-wildcard scan against fixtures it does not read off the
# tree, so it stays proven even on a tree where nothing real trips it.
#
# `Toy` is not one of our enums — the fixtures pass it to the script by name
# so the test is about the *script*, not about what today's `crates` happen
# to hold, and `toy_enum.rs` gives it a two-variant body of its own so the
# scan has a real total to judge the others against. `commented.rs` pins the
# bug the script had while this recipe was being written: a doc comment that
# says the word "match" opened a match this script never closed, and every
# line after it stopped being read. Every fixture below names only `Toy::A`
# and never `Toy::B`, so each is one variant short — 1 apiece, 3 in all — not
# because three sites were found, but because three fixtures each leave one
# of `Toy`'s two variants uncovered.
[private]
check-wildcards-selftest:
    #!/usr/bin/env bash
    set -euo pipefail
    dir=$(mktemp -d)
    trap 'rm -rf "$dir"' EXIT
    cat > "$dir/bad_array.rs" <<'RS'
    fn f(pair: [Toy; 2]) -> bool {
        match pair {
            [Toy::A, Toy::A] => true,
            _ => false,
        }
    }
    RS
    cat > "$dir/bad_tuple_ref.rs" <<'RS'
    fn f(one: &Toy, other: &Toy) -> bool {
        match (one, other) {
            (&Toy::A, &Toy::A) => true,
            _ => false,
        }
    }
    RS
    cat > "$dir/good_named.rs" <<'RS'
    fn f(pair: [Toy; 2]) -> bool {
        match pair {
            [Toy::A, Toy::A] => true,
            [Toy::A, Toy::B] | [Toy::B, Toy::A] | [Toy::B, Toy::B] => false,
        }
    }
    RS
    cat > "$dir/good_single_enum_wildcard.rs" <<'RS'
    fn f(one: Toy) -> bool {
        match one {
            Toy::A => true,
            _ => false,
        }
    }
    RS
    cat > "$dir/commented.rs" <<'RS'
    // Whatever the caller does, match it against Toy::A here.
    fn f(pair: [Toy; 2]) -> bool {
        match pair {
            [Toy::A, Toy::A] => true,
            _ => false,
        }
    }
    RS
    cat > "$dir/toy_enum.rs" <<'RS'
    pub enum Toy {
        A,
        B,
    }
    RS
    want=$(printf '%s\n' \
        "$dir/bad_array.rs:2" \
        "$dir/bad_tuple_ref.rs:2" \
        "$dir/commented.rs:3" \
        "COUNT=3" | sort)
    # Under **every** awk on this machine and not just the default one. The
    # script is not the gate's answer; what the gate counts is whatever the awk
    # in front of it says, and two of them disagreed by four arms —
    # `docs/decisions.md` §173. A bare container has mawk alone and this arm is
    # then one implementation wide, which is why it says which ones it ran.
    ran=
    for a in awk gawk mawk nawk "busybox awk"; do
        command -v ${a%% *} >/dev/null || continue
        got=$($a -v enums="Toy" -f crates/rcad-step/tools/composite_wildcards.awk \
            "$dir/bad_array.rs" "$dir/bad_tuple_ref.rs" "$dir/good_named.rs" \
            "$dir/good_single_enum_wildcard.rs" "$dir/commented.rs" "$dir/toy_enum.rs" | sort)
        if [ "$got" != "$want" ]; then
            echo "composite_wildcards.awk drifted from its own fixtures under $a:"
            echo "--- got ---"; echo "$got"
            echo "--- want ---"; echo "$want"
            echo "an answer that depends on which awk ran is not an answer —"
            echo "gawk's own regex extensions are the way this happened before"
            exit 1
        fi
        ran="$ran $a"
    done
    echo "composite scan agrees under:$ran"

    # The direct half, against the same line with and without cargo's colours.
    # The escapes below are the bytes `cargo clippy` actually emitted under
    # `CARGO_TERM_COLOR=always` on 2026-09-08, not a guess at their shape.
    w='wildcard match will also match any future added variants'
    printf 'warning: %s\n' "$w" "$w" > "$dir/plain.log"
    printf '\033[1m\033[33mwarning\033[0m\033[1m: %s\033[0m\n' "$w" "$w" > "$dir/coloured.log"
    for f in plain coloured; do
        got=$({{ just_executable() }} _wildcard-direct-count "$dir/$f.log")
        if [ "$got" != "2" ]; then
            echo "_wildcard-direct-count read $got warnings from $f.log, expected 2"
            echo "a count that only works on one of these is a count that only"
            echo "works off a runner — see the comment on the recipe"
            exit 1
        fi
    done

# How many tests are marked `#[ignore]` today. Move it with the commit that
# moves the tree, the same way `wildcard_arms` above works.
#
# Zero is a legitimate answer, and it is where this should end up: it says the
# last gap somebody parked behind an `#[ignore]` has been closed, and
# `test-ignored` then has nothing to run and says so.
ignored_tests := "6"

# An `#[ignore]`d test is a gap somebody parked, and this is what stops it being
# forgotten.
#
# Nothing ran the ignored tests at all before this: `cargo nextest run
# --workspace` skips them silently, so the one `#[ignore]` in the tree could be
# fixed and un-ignored, or a second one added, and no recipe here would say a
# word either way.
#
# **The count is pinned rather than the run being left to speak for itself**,
# because a run that selects nothing is the failure this recipe exists to
# prevent and an empty selection is exactly what a passing run looks like. A
# typo in the filter, a flag this nextest does not have, a crate that did not
# build — each of those selects nothing, and `cargo nextest run --run-ignored
# ignored-only` printing "0 tests run" would be read as good news. A number that
# has to match says which of the two happened.
#
# `test-ignored` below is what actually runs them, and it is not part of
# `gate`: today's one ignored test **fails**, which is why it is ignored, so
# a `gate` that ran it would be red on purpose.
[doc("How many tests are marked #[ignore]; fails when the count moves.")]
check-ignored:
    @out=$(cargo nextest list --workspace --run-ignored ignored-only 2>/dev/null) \
        || { echo "nextest could not list the ignored tests, so this counted nothing"; exit 1; }; \
    listed=$(echo "$out" | grep -c .); \
    if [ "$listed" != "{{ignored_tests}}" ]; then \
        echo "$out"; \
        echo "^ tests marked #[ignore]: $listed, expected {{ignored_tests}}"; \
        echo "a new one raises this number; one fixed and un-ignored lowers it"; \
        exit 1; \
    fi

# Run them. Fails loudly if the selection is empty rather than reporting a run
# of no tests as a pass.
[doc("Run the #[ignore]d tests. They fail; that is why they are ignored.")]
test-ignored:
    @if [ "{{ignored_tests}}" = "0" ]; then \
        echo "no test is marked #[ignore], so there is nothing here to run"; \
        exit 0; \
    fi; \
    cargo nextest run --workspace --run-ignored ignored-only --no-tests fail

# The hook that refuses shell patching of a .rs file, against its own cases
# (docs/decisions.md §50.8).
#
# It is in `gate` because the hook fails in the direction that is hard to
# notice: a pattern that stops matching lets the thing through silently, which
# is the same failure mode the hook exists to prevent. Two of its cases are
# drafts of itself that were wrong — a commit message describing the rule, and
# a recipe whose own body writes .rs fixtures — and five more came from the
# 2026-09-05 session review: three shapes it let through, two it wrongly
# refused in the scratchpad.
#
# The hook and most of its cases now live in the `guardrails` plugin, shared
# with `rscene`. What stayed here is `patching-cases.local.json`: the cases that
# name *this* repo — a `crates/rcad-*` path, `docs/decisions.md`, a `just`
# recipe of ours. The plugin's runner appends it and fails loudly if the file is
# named and missing, because a repo's own cases quietly not running is exactly
# the drift the plugin was made to end.
#
# The plugin's half is skipped where there is no Claude Code, which since
# 2026-09-13 includes CI: the runner used to clone the marketplace to get that
# script, and the hook it tests is a **PreToolUse** hook, which cannot fire in a
# job with no session. What is lost is this repo's three overlay cases, and it
# is named here rather than hidden. The shared 24 have a runner of their own in
# the marketplace repo now.
#
# The three hooks below it are rcad's own, need no plugin, and run either way —
# which is why the skip wraps one line and not the recipe.
[doc("The hooks against their own cases: shell patching, context budget, worktrees.")]
check-hooks:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ -d "{{claude_config}}" ]; then
        {{guardrails_plugin}}/bin/selftest.sh .claude/patching-cases.local.json
    else
        echo "no {{claude_config}}, so no Claude Code and no plugin here: the patching cases are skipped, not passed"
    fi
    .claude/hooks/check-worktree-create.sh
    .claude/hooks/check-session-start-hub.sh
    .claude/hooks/check-decisions-zero.sh

# Every `just` recipe and every repo path a project skill names still exists.
#
# A skill is prose that tells a session which commands to run, so it rots the
# way every document naming a recipe rots — and this repo has already paid for
# that once: `default` became `gate` on 2026-09-08 and every sentence naming it
# went quietly wrong, including ones in the shared skills. `check-decisions`
# holds that file to its own numbering for the same reason; this is that rule
# pointed at `.claude/skills/`, which is where a skill too repo-specific to give
# away has to live (§64.4).
#
# **Only code spans are read.** "just" is an ordinary English word, and a prose
# sentence containing it would otherwise invent a recipe name and fail here for
# nothing. A command a skill means literally is one it wrote in backticks or in
# a fenced block — the same rule the reader follows, so the check and the reader
# cannot disagree about what was a claim.
#
# A path is checked only when it carries a slash and an extension and no `<`, so
# `crates/<c>/src` is left alone: a placeholder is not a claim about the disk.
#
# Only public recipes are known, because `--summary` is what lists them. A skill
# naming a `[private]` recipe fails here, which is the right answer — those are
# the selftests, and a skill has no business sending a session into one.
[doc("Every recipe and path a .claude/skills SKILL.md names still exists.")]
check-skills:
    @{{ just_executable() }} _skills-check '.claude/skills'

# Held apart from the recipe above so the selftest can run it over fixtures
# whose answer is known, rather than over whatever the skills happen to say.
[private]
_skills-check dir:
    #!/usr/bin/env bash
    set -uo pipefail
    dir='{{dir}}'
    if [ ! -d "$dir" ]; then
        echo "no $dir, so there are no project skills to check"
        exit 0
    fi
    files=$(find "$dir" -name SKILL.md | sort)
    if [ -z "$files" ]; then
        echo "no SKILL.md under $dir, so there are no project skills to check"
        exit 0
    fi
    recipes=$({{ just_executable() }} --justfile {{ justfile() }} --summary 2>/dev/null \
        | tr ' ' '\n' | grep . | sort -u)
    if [ -z "$recipes" ]; then
        echo "just --summary listed no recipes, so every name a skill gave would pass"
        exit 1
    fi
    spans=0
    bad=0
    for f in $files; do
        claims=$(awk '
            /^[ \t]*```/ { fence = !fence; next }
            fence { print; next }
            {
                line = $0
                while (match(line, /`[^`]+`/)) {
                    print substr(line, RSTART + 1, RLENGTH - 2)
                    line = substr(line, RSTART + RLENGTH)
                }
            }
        ' "$f")
        spans=$((spans + $(printf '%s\n' "$claims" | grep -c .)))
        named=$(printf '%s\n' "$claims" \
            | awk '{ for (i = 1; i < NF; i++) \
                if ($i == "just" && $(i + 1) ~ /^[a-z][a-z0-9-]*$/) print $(i + 1) }' \
            | sort -u)
        for r in $named; do
            printf '%s\n' "$recipes" | grep -qx "$r" && continue
            echo "$f says \`just $r\`, which this justfile does not define"
            bad=1
        done
        paths=$(printf '%s\n' "$claims" \
            | awk '{ for (i = 1; i <= NF; i++) { t = $i; sub(/[,;:)]+$/, "", t); \
                if (t ~ /^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.\/-]*\.[a-z]+$/ && t !~ /</) print t } }' \
            | sort -u)
        for p in $paths; do
            [ -e "$p" ] && continue
            echo "$f names $p, which is not in the tree"
            bad=1
        done
    done
    if [ "$spans" -eq 0 ]; then
        echo "no code spans in any SKILL.md under $dir, so this scan compared nothing"
        echo "a skill with no backticks and no fenced block is not how any of them are written"
        exit 1
    fi
    [ "$bad" -eq 0 ] || exit 1

# Pins the scan against fixtures rather than against the skills in this tree, so
# it stays proven on a day they are rewritten.
#
# The passing fixture carries the two cases that would make this check worse than
# nothing if the code-span rule broke: an ordinary English "just" before a word
# that is not a recipe, and a `<`-placeholder path. Both are in the file that has
# to pass, so either one failing fails here rather than in a lane's brief.
[private]
check-skills-selftest:
    #!/usr/bin/env bash
    set -euo pipefail
    dir=$(mktemp -d)
    trap 'rm -rf "$dir"' EXIT
    j="{{ just_executable() }} --justfile {{ justfile() }}"

    mkdir -p "$dir/good"
    cat > "$dir/good/SKILL.md" <<'MD'
    Run `mise exec -- just gate quiet`, then read .claude/workflows/lanes.js.
    You can just read it first. The sources are crates/<c>/src/thing.rs.
    MD
    $j _skills-check "$dir" || { echo "a skill naming only real things failed"; exit 1; }

    mkdir -p "$dir/bad-recipe"
    printf 'Run `just no-such-recipe`.\n' > "$dir/bad-recipe/SKILL.md"
    if $j _skills-check "$dir" >/dev/null 2>&1; then
        echo "a skill naming a recipe this justfile does not define passed"; exit 1
    fi
    rm -rf "$dir/bad-recipe"

    mkdir -p "$dir/bad-path"
    printf 'Read `docs/no-such-file.md` first.\n' > "$dir/bad-path/SKILL.md"
    if $j _skills-check "$dir" >/dev/null 2>&1; then
        echo "a skill naming a path that is not in the tree passed"; exit 1
    fi
    rm -rf "$dir/bad-path"

    rm -rf "$dir/good"
    mkdir -p "$dir/silent"
    printf 'No code spans at all in this one.\n' > "$dir/silent/SKILL.md"
    if $j _skills-check "$dir" >/dev/null 2>&1; then
        echo "a scan that found no code spans passed, having compared nothing"; exit 1
    fi

    if ! $j _skills-check "$dir/no-such-dir" | grep -q 'no project skills'; then
        echo "a missing skills directory is not the same as a broken scan"; exit 1
    fi

# kache must be the wrapper here and no wrapper on a runner (`mise.toml`).
#
# Both halves are asserted, because each one alone passes for the wrong reason.
# Checking only the CI half passes if somebody deletes the variable outright,
# which would turn the cache off for every human as well. Checking only the
# local half passes on the state that kept CI red for ten pushes.
#
# It asks mise rather than reading the file, because the file is a template and
# what breaks CI is the value mise computes, not the text. That also means this
# fails if the template stops parsing, which the eye does not catch.
#
# **It asks by running a process under mise, not by parsing `mise env`.** Until
# 2026-09-20 it sed'd `^export RUSTC_WRAPPER=` out of `mise env`; mise 2026.9.11
# no longer prints that line in that shape, and this machine's 2026.7.5 does. So
# in a fresh container both halves extracted the empty string: the CI half passed
# vacuously — the one direction that took CI red for ten pushes — and the local
# half failed saying `RUSTC_WRAPPER is ''`, which reads as a broken template and
# was nothing of the sort. `mise exec` is the thing the gate actually relies on,
# and its answer is a value rather than a format. `docs/decisions.md` §173.8.
[doc("kache wraps rustc here, and wraps nothing on a runner.")]
check-kache-off-on-ci:
    @on=$(CI=true mise exec -- sh -c 'echo "${RUSTC_WRAPPER-}"'); \
    off=$(env -u CI mise exec -- sh -c 'echo "${RUSTC_WRAPPER-}"'); \
    if [ -n "$on" ]; then \
        echo "with CI set, RUSTC_WRAPPER is '$on' and should be empty"; \
        echo "mise applies [env] while installing [tools], so cargo would be asked"; \
        echo "to run kache in order to build kache, and every install fails"; \
        exit 1; \
    fi; \
    if [ "$off" != "kache" ]; then \
        echo "off CI, RUSTC_WRAPPER is '$off' and should be kache"; \
        echo "the build cache is now off for everyone, which is not what was wanted"; \
        exit 1; \
    fi

# `gate` runs `check`, `lint`, `test`, `deny` and every `check-*` there is,
# which is what CLAUDE.md publishes and so is a claim that wants a test (§15.6).
#
# Since 2026-09-07 CI runs `gate` itself rather than those four separately,
# which makes this recipe the thing that carries a new `check-*` onto the runner:
# wire one into `gate` and it is enforced there with no edit to `ci.yml`.
#
# The failure it is really for is the other direction: a `check-*` recipe
# written, committed and never wired into `gate` is a check nobody runs, and
# nothing else in this tree would notice. That has never happened, which is not
# the same as being prevented.
#
# The count is pinned rather than left to the comparison, and that is the whole
# design. Both sides are parsed out of this one file, so an extraction that
# stops matching empties both at once and two empty sets compare equal — the
# recipe would pass having compared nothing, and an empty diff prints nothing,
# which reads exactly like success. `check-ignored` says the same thing about
# itself a hundred lines up. It is not hypothetical, and it has now happened:
# renaming `default` to `gate` broke the anchor on the second line below, the
# pinned count is what turned that into a failure rather than a silent pass, and
# both arms were re-checked by mutation afterwards rather than by running green.
# The anchor moved a second time when `gate` stopped being a dependency list and
# its steps became `gate_steps`, so that a failing gate could still print a
# summary; the count caught that move too.
#
# So bumping the number is how a new `check-*` recipe gets added, deliberately,
# in the commit that adds it.
#
# The last arm pins `check-plugins` **first**, which the three above could not:
# both sides of every comparison here are `sort -u`, so this recipe was blind to
# order by construction. CLAUDE.md publishes "first thing in `just gate`" and
# `ci.yml` leans on it, and a published claim wants a test (§15.6) — it had none,
# and `check-plugins` ran first only because of where it happened to sit.
#
# What the order buys is the error message. Without plugins `check-plugins` exits
# 1 and names the install command, while `check-hooks` and
# `check-questions-selftest` exit **127** on a missing `selftest.sh` — no
# message, no remedy, and the failure a CI log explains worst. Found by rcad-c2,
# who also noticed that nothing held the ordering it depends on.
[doc("Every check-* recipe is in `gate`, and check-plugins is first.")]
check-gate-runs-every-check:
    @steps=$(printf '%s\n' {{ gate_steps }}); \
    defined=$(grep -oE '^check-[a-z0-9-]+' justfile | sort -u); \
    listed=$(printf '%s\n' "$steps" | grep '^check-' | sort -u); \
    others=$(printf '%s\n' "$steps" | grep -v '^check-' | grep . | sort -u | tr '\n' ' '); \
    n=$(printf '%s\n' "$defined" | grep -c .); \
    if [ "$n" != "43" ]; then \
        echo "found $n check-* recipes, expected 43"; \
        echo "if you added or removed one, change the number here in the same commit"; \
        echo "if you found none, the parse broke and this recipe just compared nothing"; \
        exit 1; \
    fi; \
    if [ "$defined" != "$listed" ]; then \
        echo "these check-* recipes are defined but not run by gate:"; \
        printf '%s\n' "$defined" | while read -r r; do \
            printf '%s\n' "$listed" | grep -qx "$r" || echo "  $r"; \
        done; \
        echo "a check nobody runs is not a check"; \
        exit 1; \
    fi; \
    if [ "$others" != "check deny lint test " ]; then \
        echo "gate's non-check-* recipes are '$others', expected 'check deny lint test '"; \
        echo "CLAUDE.md says gate runs these four and every check-* besides"; \
        echo "change that sentence in the same commit, or this is now false"; \
        exit 1; \
    fi; \
    decl=$(awk '/^gate_steps := /{f=1} f{print} f && !/\\$/{exit}' justfile); \
    lines=$(printf '%s\n' "$decl" | grep -c .); \
    steps_n=$(printf '%s\n' "$steps" | grep -c .); \
    if [ "$lines" != "$steps_n" ]; then \
        echo "gate_steps is $steps_n steps written on $lines source lines"; \
        echo "one step per line is what stops two lanes adding two checks from"; \
        echo "conflicting: git merges by line, and as one line this was the worst"; \
        echo "merge shape in the repo (docs/decisions.md §177)"; \
        exit 1; \
    fi; \
    first=$(printf '%s\n' "$steps" | grep -m1 .); \
    if [ "$first" != "check-plugins" ]; then \
        echo "gate runs '$first' first, expected 'check-plugins'"; \
        echo "CLAUDE.md says check-plugins runs first thing in just gate, and"; \
        echo "ci.yml leans on it: with no plugins, check-hooks and"; \
        echo "check-questions-selftest exit 127 on a missing selftest.sh, which is"; \
        echo "the failure a runner explains worst, while check-plugins exits 1 and"; \
        echo "names the command that installs them"; \
        exit 1; \
    fi

[doc("Every public enum is #[non_exhaustive] or says it is exhaustive on purpose.")]
check-enums:
    @! awk '/^$/ {ok = 0} \
            /^#\[non_exhaustive\]/ {ok = 1} \
            /Exhaustive on purpose/ {ok = 1} \
            /^[ \t]*pub enum/ {if (!ok) print FILENAME ":" FNR ": " $0; ok = 0}' \
            $(find crates -path '*/src/*' -name '*.rs') | grep . \
        || (echo "^ a public enum that neither grows nor says why not (docs/decisions.md §40)"; exit 1)

# Where `decisions-toc`, `decision` and the rest read from: one file per `## N.`
# section, named `NNN-<slug>.md` for the number it is cited as, plus
# `00-preamble.md` for the fifteen lines that stood above `## 0.`. A variable so
# the selftest below can point them at fixtures rather than at whatever the real
# sections happen to say today.
#
# **The preamble is two digits and a section is three, and that is the whole
# scheme.** Both wanted to be `000-`: the preamble comes first and §0 is a real
# section whose number is 0. Two digits sorts before three under byte order
# (`-` is 0x2D, below `0`), which is what `LC_ALL=C sort` gives, and it is the
# one distinction a filename can carry that `just decision 0` does not have to
# know about — the lookup matches three digits, so the preamble is invisible to
# it rather than special-cased in it.
#
# **Every listing here sorts under `LC_ALL=C`**, and that is not decoration: a
# plain `ls` collates under the user's locale, which ignores `-` entirely, and
# then `00-preamble.md` lands between `009-` and `010-` — which is exactly where
# the first attempt at this put it, silently, and the concatenation came back
# 15 lines out of order.
decisions_dir := "docs/decisions.d"

# The pointer file. Every citation in the tree spells `docs/decisions.md §N` and
# still does after the split (§179), so the path keeps its name and holds four
# lines of prose. `check-decisions` is what keeps it a pointer.
decisions_file := "docs/decisions.md"

# The map of docs/decisions.md: every top-level section, and what it costs to
# read.
#
# 27 000 lines under 843 headings, so the file is not a thing anyone holds at
# once, and the way it was being read was `sed -n '15690,15900p'` — a guessed
# line range, ~5 000 tokens a guess, landing beside the section about as often
# as on it. On 2026-09-05 that cost 123 calls and ~180 000 tokens in one day,
# more than any other file in the project.
#
# **The line count is the column that earns its place**, because it is the one
# thing the section number does not already tell you: it says whether `just
# decision 49` is 40 lines or 400 before you pay for it. There are deliberately
# no line *offsets* here. The sections are numbered, `just decision 49.6`
# addresses one by its number, and an offset would be a second address for the
# same section that goes stale the moment anybody inserts a paragraph.
[doc("The map of docs/decisions.md: every section, and what it costs to read.")]
decisions-toc:
    #!/usr/bin/env bash
    set -euo pipefail
    dir='{{decisions_dir}}'
    for f in $(ls "$dir" | LC_ALL=C sort); do
        head=$(head -n 1 "$dir/$f")
        case "$head" in '## '*) ;; *) continue ;; esac
        rest=${head#'## '}
        title=${rest#* }
        num=${rest%% *}; num=${num%.}
        printf "  %-5s %-56s %4d lines\n" \
            "$num" "$title" "$(( $(wc -l < "$dir/$f") - 1 ))"
    done

# Every section, concatenated, in the order the numbers run.
#
# This is the file the sections used to be, and it is how the split was checked
# to have lost nothing: at §179's own commit this recipe's output was
# byte-identical to the `docs/decisions.md` it replaced, diffed and empty.
#
# It is not what anybody should read — 40 000 lines — but a whole-file search
# needs it to exist, and `decisions-find` and `check-where-the-work-is` are both
# written against it rather than against a directory walk, because a hit or a
# citation has to be attributed across sections to mean anything.
[doc("Every section concatenated, in number order: the file the sections used to be.")]
decisions-all:
    #!/usr/bin/env bash
    set -euo pipefail
    dir='{{decisions_dir}}'
    for f in $(ls "$dir" | LC_ALL=C sort); do
        cat "$dir/$f"
    done

# One section of docs/decisions.md, by its number.
#
# `just decision 49` gives §49's own prose and then the list of its subsections,
# which is the cheap way in; `just decision 49.6` gives §49.6 in full. A leading
# section sign is accepted so a citation can be pasted as it is written.
#
# **The subsection address is what the split had to preserve** (§179). rscene's
# `###` headings are free text, so its port of this recipe prints whole sections
# and stops; here every subsection is numbered and `just decision 173.10` is how
# one is read. The filename picks the file — `173.10` and `173` both resolve to
# `173-*.md` — and then the awk below is the one that ran against the single
# file, unchanged, reading a file that now holds one section instead of 177.
#
# **A number that does not exist fails rather than printing nothing.** Silence
# reads as "the decisions say nothing about this", which is the opposite of what
# an unknown number means, and it is the same failure `check-ignored` above
# guards against: an empty selection looks exactly like a clean pass.
[doc("One section of docs/decisions.md by number: `just decision 49.6`.")]
decision want:
    #!/usr/bin/env bash
    set -euo pipefail
    want=$(printf '%s' '{{want}}' | sed 's/^§//; s/\.$//')
    if [ -z "$want" ]; then
        echo "which section? try 'just decisions-toc' for the map" >&2
        exit 2
    fi
    dir='{{decisions_dir}}'
    top=${want%%.*}
    file=""
    if [[ $top =~ ^[0-9]+$ ]]; then
        for f in $(ls "$dir" | LC_ALL=C sort); do
            n=${f%%-*}
            [[ $n =~ ^[0-9][0-9][0-9]$ ]] || continue
            [ "$((10#$n))" = "$((10#$top))" ] || continue
            file="$dir/$f"
            break
        done
    fi
    if [ -z "$file" ]; then
        echo "docs/decisions.md has no section $want" >&2
        exit 3
    fi
    case "$want" in
    *.*)
        awk -v want="$want" '
            /^## |^### / {
                if (on) exit
                rest = $0; sub(/^#+ /, "", rest)
                p = index(rest, " "); id = substr(rest, 1, p - 1); sub(/\.$/, "", id)
                if (id == want) { on = 1; found = 1 }
            }
            on { print }
            END { exit(found ? 0 : 3) }' "$file" || {
            echo "docs/decisions.md has no section $want" >&2
            exit 3
        }
        ;;
    *)
        awk -v want="$want" '
            /^## / {
                if (on) exit
                rest = substr($0, 4); p = index(rest, " ")
                id = substr(rest, 1, p - 1); sub(/\.$/, "", id)
                if (id == want) { on = 1; found = 1 }
            }
            /^### / { if (on) exit }
            on { print }
            END { exit(found ? 0 : 3) }' "$file" || {
            echo "docs/decisions.md has no section $want" >&2
            exit 3
        }
        awk -v want="$want" '
            function flush() {
                if (num != "") printf "  %-7s %-54s %4d lines\n", num, title, n
                num = ""
            }
            /^### / {
                flush()
                rest = substr($0, 5); p = index(rest, " ")
                id = substr(rest, 1, p - 1)
                if (index(id, want ".") == 1) { num = id; title = substr(rest, p + 1); n = 0 }
                next
            }
            /^## / { flush(); next }
            num != "" { n++ }
            END { flush() }' "$file"
        ;;
    esac

# Which sections mention this? A text search that answers in section numbers.
#
#     just decisions-find 'refusal rate'
#
# gives the subsections whose text matches, so the next call is `just decision
# 41.13`. A plain grep answers `16820:` instead, and a line number is the one
# thing CLAUDE.md says never to address a decision by — walking back up 17 000
# lines to the heading above the hit was the step that had no recipe.
#
# **The match is case-insensitive.** Of 886 grep commands measured across one
# day's sessions, 97 matched nothing at all, and a search that misses `Refusal`
# because the caller typed `refusal` fails in exactly that silent way. A caller
# who wants the distinction writes the bracket themselves.
#
# **No match fails rather than printing nothing**, for the same reason `decision`
# fails on a number that does not exist: an empty answer here reads as "the
# decisions say nothing about this", and that is a claim a typo can forge.
#
# **This searches the concatenation and not the directory**, which is what keeps
# a hit attributed to the `### N.M` it fell in rather than only to the `## N.`
# file that holds it. rscene's port counts hits per file with `grep -c`, because
# a file is the finest unit it has; here the subsection is finer than the file,
# so the search needs a stream with every heading in it. `decisions-all` is that
# stream, and the awk below is unchanged from the one that read the single file.
[doc("Which sections mention this? Answers in section numbers, not line numbers.")]
decisions-find pattern:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ -z "$1" ]; then
        echo "find what? an empty pattern matches every line" >&2
        exit 2
    fi
    all=$(mktemp)
    hits=$(mktemp)
    trap 'rm -f "$all" "$hits"' EXIT
    dir='{{decisions_dir}}'
    for f in $(ls "$dir" | LC_ALL=C sort); do cat "$dir/$f"; done > "$all"
    if ! grep -inE -- "$1" "$all" | cut -d: -f1 > "$hits"; then
        echo "no line in {{decisions_dir}} matches $1" >&2
        exit 1
    fi
    awk -v hits="$hits" '
        BEGIN {
            while ((getline ln < hits) > 0) wanted[ln + 0] = 1
            id = "(preamble)"
        }
        /^## |^### / {
            rest = $0; sub(/^#+ /, "", rest)
            p = index(rest, " "); id = substr(rest, 1, p - 1); sub(/\.$/, "", id)
            title = substr(rest, p + 1)
        }
        wanted[FNR] {
            if (!(id in count)) { order[++k] = id; name[id] = title }
            count[id]++
        }
        END {
            for (i = 1; i <= k; i++)
                printf "  %-7s %-54s %4d hits\n", order[i], name[order[i]], count[order[i]]
        }' "$all"

# Where `claim-section` writes down the numbers it hands out. Empty means a file
# in the repository's common git directory, which every worktree shares and no
# branch carries.
section_claims := ""

# Take the next section number for docs/decisions.md now, before writing it.
#
#     just claim-section 'the lookup trigger, measured'
#
# prints the number and nothing else. **The number is yours from that moment**:
# it is written to the claims file before it is printed, under a lock, so a
# second session asking a second later gets the one after.
#
# Checking by hand — `git log --all -S'## 151. '` — was correct every time and
# lost anyway, because it is a read against a value another session is about to
# write, and a section takes an hour to write. §144.5, §152.7 and §157.6 each
# lost that race while being written; §157.6 lost it three times. What was
# missing was a claim that outlives the read (§159).
#
# The number is one past the highest of: every `## N.` on every local and
# remote-tracking branch, every `## N.` in every worktree's copy of the file —
# committed or not, because a lane writes its section long before it commits —
# and every number claimed before. A claim that is never written leaves a gap,
# and a gap costs nothing: `check-decisions` asks for unique, ascending numbers,
# not consecutive ones. §54 and §65 are two such gaps today.
#
# **It reads both shapes, and that is not tidiness** (§179). The split put the
# sections in `docs/decisions.d/NNN-*.md`, and every lane branch cut before it
# still carries the single `docs/decisions.md` with its sections in it — for
# weeks, since a lane lives as long as its unit. A recipe that read only the new
# shape would see §178 as the top while a branch held §180, and hand §179 out
# twice; reading only the old shape would stop counting the moment the split
# landed. So both are read, on refs and in worktrees alike, and the highest of
# everything wins. This is the failure this repository has already paid for once
# by hand, which is what §159 was written about.
#
# Every worktree of this clone shares the claims, and nothing else does. A
# section written on another machine is seen once it reaches a branch here.
[doc("Take the next docs/decisions.md section number now, before writing it: `just claim-section 'what it is about'`.")]
claim-section what:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ -z "$1" ]; then
        echo "claim it for what? the claims file says who holds each number" >&2
        exit 2
    fi
    claims="{{section_claims}}"
    if [ -z "$claims" ]; then
        claims="$(git rev-parse --path-format=absolute --git-common-dir)/rcad-section-claims"
    fi
    exec 9>>"$claims.lock"
    flock 9
    touch "$claims"
    file="{{decisions_file}}"
    dir="{{decisions_dir}}"
    top=$({
        git for-each-ref --format='%(refname)' refs/heads refs/remotes \
            | while read -r ref; do git rev-parse -q --verify "$ref:$file" || true; done \
            | sort -u \
            | while read -r blob; do git cat-file blob "$blob"; done \
            | grep -E '^## [0-9]+\. ' || true
        git for-each-ref --format='%(refname)' refs/heads refs/remotes \
            | while read -r ref; do git ls-tree --name-only "$ref:$dir" 2>/dev/null || true; done \
            | sed -nE 's/^([0-9][0-9][0-9])-.*\.md$/## \1./p'
        git worktree list --porcelain | sed -n 's/^worktree //p' \
            | while read -r tree; do
                if [ -f "$tree/$file" ]; then grep -E '^## [0-9]+\. ' "$tree/$file" || true; fi
                if [ -d "$tree/$dir" ]; then
                    ls "$tree/$dir" | sed -nE 's/^([0-9][0-9][0-9])-.*\.md$/## \1./p'
                fi
            done
        cut -f1 "$claims" | sed 's/^/## /'
    } | awk '{ n = $2 + 0; if (n > top) top = n } END { print top + 0 }')
    next=$((top + 1))
    printf '%s\t%s\t%s\t%s\n' "$next" "$(date -u +%FT%TZ)" "{{invocation_directory()}}" "$1" >> "$claims"
    echo "$next"

# `decision` finds a section by the number its filename starts with and a
# subsection by the number its heading starts with, so either one written
# without a number is a section the lookup cannot reach — and it fails as
# silence, which reads as "nothing was decided about this".
#
# All 487 headings are numbered today, `## N.` and `### N.M`. `####` headings are
# free text on purpose: they sit inside a subsection and are never addressed.
#
# A heading hiding inside a code fence would fool the same match. None exists and
# none ever has, so per §13 that check is not built until something needs it.
#
# **The filename and the heading are two places to write the number**, so they
# are compared. A section file copied to start the next one keeps its old
# heading under a new name, and then two numbers address one text and one
# addresses none.
#
# **Two files sharing a number is the second way to be unreachable**, and it is
# the one that happens by itself. `decision` takes the first file whose number
# matches, so the second §56 is addressed by nothing at all — and it is not a
# typo that puts it there. Whoever finishes a section next takes the next
# number, ten to thirty times a day from as many as four concurrent sessions,
# and two of them reaching for the same one is the ordinary case rather than the
# unlucky one. It happened while §57 was being written (§57.4). The split (§179)
# is what stopped that being a merge conflict every time; the collision it
# cannot prevent is the one worth catching here.
#
# **A `### N.M` sitting in the wrong file, or after a higher one, is the
# third.** A subsection typed by hand during a renumbering can end up under the
# wrong parent — `### 64.3` left under `## 69.` after 64 was renumbered to 67 is
# the shape it takes, and it happened four times in one session and was only
# caught by grepping for it. `docs/decisions.md` §71.
#
# **And the pointer file must stay a pointer.** Every citation in the tree says
# `docs/decisions.md §N`, the habit of four sessions is to append a section to
# that path, and an append there is invisible to every recipe above — the text
# would be live, addressed by nothing, and colliding again. This is the failure
# the split invites and the one nothing else would catch. The pointer's path is
# derived from the directory's own name (`docs/decisions.d` → `docs/decisions.md`)
# so that the selftest below gets it too, against a fixture pointer of its own.
#
# **Contiguity is deliberately not demanded, unlike rscene's port of this.**
# `claim-section` hands a number out under a lock before the section is written,
# so a claim that is never used leaves a permanent hole: §54 and §65 are two
# such holes today and neither is a fault. A check that asked for 0..N would
# have been red on the tree it was written against.
[doc("Every heading in docs/decisions.md can be reached by its number.")]
check-decisions:
    #!/usr/bin/env bash
    set -euo pipefail
    dir="{{decisions_dir}}"
    [ -d "$dir" ] || { echo "$dir is not a directory"; exit 1; }

    ok=1
    nums=$(mktemp)
    trap 'rm -f "$nums"' EXIT
    for f in $(ls "$dir" | LC_ALL=C sort); do
        path="$dir/$f"
        if [ "$f" = "00-preamble.md" ]; then
            if grep -q '^## ' "$path"; then
                grep -n '^## ' "$path"
                echo "^ $path is the preamble and carries a '## ' heading"
                ok=0
            fi
            continue
        fi
        if [[ ! $f =~ ^([0-9][0-9][0-9])-[a-z0-9-]+\.md$ ]]; then
            echo "$path is not named NNN-<slug>.md, so 'just decision' cannot reach it"
            ok=0
            continue
        fi
        n=$((10#${BASH_REMATCH[1]}))
        echo "$n" >> "$nums"

        headings=$(grep -c '^## ' "$path" || true)
        if [ "$headings" -ne 1 ]; then
            echo "$path has $headings '## ' headings; a file is one section"
            ok=0
            continue
        fi
        if ! head -n 1 "$path" | grep -q "^## $n\. "; then
            head -n 1 "$path"
            echo "^ $path does not open with '## $n. ', so its name and its heading disagree"
            ok=0
            continue
        fi

        bad=$(grep -nE '^(## |### )' "$path" \
            | grep -vE '^[0-9]+:## [0-9]+\. ' \
            | grep -vE '^[0-9]+:### [0-9]+\.[0-9]+ ' || true)
        if [ -n "$bad" ]; then
            echo "$path"
            echo "$bad"
            echo "^ a decisions heading that 'just decision' cannot address"
            ok=0
            continue
        fi

        subs=$(awk -v n="$n" -v path="$path" '
            /^### [0-9]/ {
                rest = $0; sub(/^#+ /, "", rest)
                p = index(rest, " "); id = substr(rest, 1, p - 1)
                split(id, parts, ".")
                if (parts[1] + 0 != n)
                    print path ":" FNR ": ### " id " is not under its own ## " n
                else if (parts[2] + 0 <= lastsub)
                    print path ":" FNR ": ### " id " comes after ### " n "." lastsub
                lastsub = parts[2] + 0
            }' "$path")
        if [ -n "$subs" ]; then
            echo "$subs"
            echo "^ decisions subsection numbers are out of order (docs/decisions.md §71)"
            ok=0
        fi
    done
    [ "$ok" -eq 1 ] || exit 1

    [ -s "$nums" ] || { echo "$dir holds no sections at all"; exit 1; }
    dupes=$(sort -n "$nums" | uniq -d)
    if [ -n "$dupes" ]; then
        for id in $dupes; do ls "$dir" | { grep -E "^0*${id}-" || true; }; done
        echo "^ these files share a section number, and 'just decision' reaches only the first"
        exit 1
    fi

    pointer="${dir%.d}.md"
    if [ -e "$pointer" ] && grep -q '^## ' "$pointer"; then
        grep -n '^## ' "$pointer"
        echo "^ $pointer is the pointer, not a section; that heading is addressed by nothing"
        exit 1
    fi

# Pins the lookup against fixtures rather than against the real sections, so it
# stays proven on a tree where the real §1.1 has been rewritten.
#
# `#### ` inside the fixture is the case worth having: it is a heading, and a
# lookup that stops at any heading would cut §1.1 in half without saying so.
#
# **The fixture is a directory now** (§179), and every case below that used to
# be a second `.md` file is a second fixture directory. The cases the split
# added are the ones that were impossible before: a filename and a heading that
# disagree, two files claiming one number, a section appended to the pointer,
# and a `00-preamble.md` that has grown a section heading. The cases it made
# impossible went the other way — two `## 1.` in one file is now "a file is one
# section", and a descending `## 1.` after `## 2.` cannot be expressed at all,
# because the order is the filenames' and the filenames are sorted.
[private]
check-decisions-selftest:
    #!/usr/bin/env bash
    set -euo pipefail
    dir=$(mktemp -d)
    trap 'rm -rf "$dir"' EXIT
    mkdir -p "$dir/fixture.d"
    cat > "$dir/fixture.d/00-preamble.md" <<'MD'
    # Design decisions
    MD
    cat > "$dir/fixture.d/001-first.md" <<'MD'
    ## 1. First
    intro to one
    ### 1.1 Alpha
    alpha body
    #### A fourth-level heading that must not end the section
    still alpha
    ### 1.2 Beta
    beta body
    MD
    cat > "$dir/fixture.d/002-second.md" <<'MD'
    ## 2. Second
    intro to two
    MD
    j="{{just_executable()}} --set decisions_dir $dir/fixture.d"

    got=$($j decisions-all)
    want=$(printf '%s\n' "# Design decisions" "## 1. First" "intro to one" \
        "### 1.1 Alpha" "alpha body" \
        "#### A fourth-level heading that must not end the section" "still alpha" \
        "### 1.2 Beta" "beta body" "## 2. Second" "intro to two")
    [ "$got" = "$want" ] || { echo "decisions-all drifted:"; \
        echo "--- got ---"; echo "$got"; echo "--- want ---"; echo "$want"; exit 1; }

    got=$($j decision 1.1)
    want=$(printf '%s\n' "### 1.1 Alpha" "alpha body" \
        "#### A fourth-level heading that must not end the section" "still alpha")
    [ "$got" = "$want" ] || { echo "decision 1.1 drifted:"; \
        echo "--- got ---"; echo "$got"; echo "--- want ---"; echo "$want"; exit 1; }

    got=$($j decision 1)
    want=$(printf '%s\n' "## 1. First" "intro to one" \
        "  1.1     Alpha                                                     3 lines" \
        "  1.2     Beta                                                      1 lines")
    [ "$got" = "$want" ] || { echo "decision 1 drifted:"; \
        echo "--- got ---"; echo "$got"; echo "--- want ---"; echo "$want"; exit 1; }

    got=$($j decisions-toc)
    want=$(printf '%s\n' \
        "  1     First                                                       7 lines" \
        "  2     Second                                                      1 lines")
    [ "$got" = "$want" ] || { echo "decisions-toc drifted:"; \
        echo "--- got ---"; echo "$got"; echo "--- want ---"; echo "$want"; exit 1; }

    got=$($j decision §1.1)
    [ "$got" = "$($j decision 1.1)" ] || { echo "a pasted § was not accepted"; exit 1; }

    if $j decision 9 >/dev/null 2>&1; then
        echo "decision 9 found a section that does not exist"; exit 1
    fi

    # `decision 1` above pins the column widths, so what is left to pin here is
    # the attribution: which section a hit is reported under, and how many.
    squashed() { $j decisions-find "$1" | awk '{ $1 = $1; print }'; }

    got=$(squashed ALPHA)
    want=$(printf '%s\n' "1.1 Alpha 3 hits")
    [ "$got" = "$want" ] || { echo "decisions-find ALPHA drifted:"; \
        echo "--- got ---"; echo "$got"; echo "--- want ---"; echo "$want"; exit 1; }

    got=$(squashed intro)
    want=$(printf '%s\n' "1 First 1 hits" "2 Second 1 hits")
    [ "$got" = "$want" ] || { echo "decisions-find intro drifted:"; \
        echo "--- got ---"; echo "$got"; echo "--- want ---"; echo "$want"; exit 1; }

    got=$(squashed decisions)
    [ "$got" = "(preamble) 1 hits" ] || { \
        echo "a hit above the first heading was reported as '$got'"; exit 1; }

    if $j decisions-find zzzz >/dev/null 2>&1; then
        echo "decisions-find said nothing about a pattern nothing matches"; exit 1
    fi

    if $j decisions-find "" >/dev/null 2>&1; then
        echo "decisions-find accepted the empty pattern"; exit 1
    fi

    # Each way the sections can go unreachable, against fixtures for the same
    # reason as above: the real directory is clean today, so pinning the check
    # there would prove nothing.
    $j check-decisions >/dev/null \
        || { echo "the clean fixture failed check-decisions"; exit 1; }

    # `bad <name> <why>` builds a fixture directory from the clean one, lets the
    # caller break it, and fails if `check-decisions` is happy with the wreck.
    bad() {
        cp -r "$dir/fixture.d" "$dir/$1.d"
        "break_$1"
        if {{just_executable()}} --set decisions_dir "$dir/$1.d" \
            check-decisions >/dev/null 2>&1; then
            echo "$2 passed check-decisions"; exit 1
        fi
    }

    break_twice() { cp "$dir/twice.d/001-first.md" "$dir/twice.d/001-first-again.md"; }
    bad twice "two files numbered 001"

    break_twice_sub() {
        printf '%s\n' "### 1.1 Alpha again" >> "$dir/twice_sub.d/001-first.md"
    }
    bad twice_sub "two subsections numbered 1.1"

    # Descending within a section: §1.1 lands after §1.2.
    break_descending_sub() {
        printf '%s\n' "### 1.1 Alpha" >> "$dir/descending_sub.d/001-first.md"
    }
    bad descending_sub "### 1.1 after ### 1.2"

    # Wrong parent: §1.2 is left in §2's file by a renumbering, exactly the way
    # `### 64.3` ended up under `## 69.` in the single file.
    break_wrong_parent() {
        printf '%s\n' "### 1.2 Beta" >> "$dir/wrong_parent.d/002-second.md"
    }
    bad wrong_parent "### 1.2 in ## 2.'s file"

    # The name and the heading disagree: a file copied to start the next section
    # and renamed but not retitled.
    break_renamed() { mv "$dir/renamed.d/002-second.md" "$dir/renamed.d/003-second.md"; }
    bad renamed "a file named 003 whose heading says 2"

    # A file is one section: the old habit of appending, applied to a section
    # file instead of the whole document.
    break_two_in_one() {
        printf '%s\n' "## 3. Third" >> "$dir/two_in_one.d/002-second.md"
    }
    bad two_in_one "two sections in one file"

    # An unnumbered filename, which `just decision` can reach by nothing.
    break_unnumbered() { mv "$dir/unnumbered.d/002-second.md" "$dir/unnumbered.d/second.md"; }
    bad unnumbered "a file named without a number"

    # The preamble grows a section heading, which addresses it to nobody.
    break_preamble_section() {
        printf '%s\n' "## 3. Third" >> "$dir/preamble_section.d/00-preamble.md"
    }
    bad preamble_section "a section heading in the preamble"

    # The pointer grows a section, which is the habit of four sessions and the
    # one failure the split invites (§179).
    break_pointer() { printf '%s\n' "## 3. Third" > "$dir/pointer.md"; }
    bad pointer "a section appended to the pointer file"

# Pins `claim-section` against a throwaway repository with a section in each
# place the recipe must look: on the checked-out branch, on another branch only,
# and uncommitted in another worktree.
#
# **The concurrent round is the one that matters**, because it is the race the
# recipe exists for. Without the lock, eight claims started together hand out
# the same number several times over; that was checked by deleting the `flock`
# line and watching this fail.
[private]
check-claim-section-selftest:
    #!/usr/bin/env bash
    set -euo pipefail
    dir=$(mktemp -d)
    trap 'rm -rf "$dir"' EXIT
    repo="$dir/repo"
    g() { git -C "$repo" -c user.name=selftest -c user.email=selftest@invalid "$@"; }
    mkdir -p "$repo/docs"
    g init -q -b main
    printf '%s\n' "## 1. First" "## 2. Second" > "$repo/docs/decisions.md"
    g add docs/decisions.md
    g commit -q -m one
    g checkout -q -b elsewhere
    printf '%s\n' "## 5. Only on a branch" >> "$repo/docs/decisions.md"
    g commit -q -am five
    g checkout -q main
    claim() { {{just_executable()}} --justfile "{{justfile()}}" --working-directory "$repo" claim-section "$@"; }

    got=$(claim 'past a branch')
    [ "$got" = "6" ] || { echo "the first claim got '$got', want 6: one past §5 on the other branch"; exit 1; }

    g worktree add -q --detach "$dir/lane" main
    printf '%s\n' "## 9. Written, not committed" >> "$dir/lane/docs/decisions.md"
    got=$(claim 'past a worktree')
    [ "$got" = "10" ] || { echo "a claim got '$got', want 10: one past the uncommitted §9"; exit 1; }

    got=$({{just_executable()}} --justfile "{{justfile()}}" --working-directory "$dir/lane" \
        claim-section 'from the other worktree')
    [ "$got" = "11" ] || { echo "a claim from the other worktree got '$got', want 11: §10 was claimed from the first"; exit 1; }

    for i in 1 2 3 4 5 6 7 8; do claim "at once $i" > "$dir/at-once-$i" & done
    wait
    got=$(cat "$dir"/at-once-* | sort -n | tr '\n' ' ')
    [ "$got" = "12 13 14 15 16 17 18 19 " ] || {
        echo "eight claims at once got '$got', want 12 to 19 once each"; exit 1; }

    # Both shapes, which is the whole of §179.2's second point. Everything above
    # is the old single file, because that is what a lane branch cut before the
    # split still carries; these two are the new directory, committed on a
    # branch and uncommitted in a worktree. A recipe that had learnt the new
    # shape and forgotten the old would pass every case above and hand out a
    # number a branch already holds — so the old cases are the test for the new
    # code just as much as these are.
    g checkout -q elsewhere
    mkdir -p "$repo/docs/decisions.d"
    printf '%s\n' "## 40. Split, on a branch" > "$repo/docs/decisions.d/040-split.md"
    g add docs/decisions.d/040-split.md
    g commit -q -m forty
    g checkout -q main
    got=$(claim 'past a split branch')
    [ "$got" = "41" ] || {
        echo "a claim got '$got', want 41: one past §40 in the branch's decisions.d"; exit 1; }

    mkdir -p "$dir/lane/docs/decisions.d"
    printf '%s\n' "## 60. Split, not committed" > "$dir/lane/docs/decisions.d/060-split.md"
    got=$(claim 'past a split worktree')
    [ "$got" = "61" ] || {
        echo "a claim got '$got', want 61: one past the uncommitted §60 in decisions.d"; exit 1; }

    if claim '' >/dev/null 2>&1; then
        echo "claim-section accepted a claim for nothing"; exit 1
    fi

# One item of Rust source, by its name, with its doc comment and attributes.
#
#     just src arrange_faces
#     just src new crates/rcad-geom
#
# The habit this replaces is `grep -n 'fn arrange_faces'` followed by a `Read`
# with a hand-guessed `offset` and `limit` — two calls, and the guess either
# cuts the item in half or pays for the next one as well. Measured across 64
# transcripts, `.rs` files are 33% of every character `Read` has delivered in
# this repo, and 0.31 M of those arrived through calls that passed no `limit`
# at all. §57.6 left this recipe open rather than refused; this is it.
#
# **A name defined in more than one place lists the places instead of printing
# all of them**, because printing every `fn new` in the workspace is the cost
# the recipe exists to avoid. The second argument narrows the search to a path.
#
# The extent of an item is read off rustfmt's layout rather than parsed: back to
# the blank line above it, forward to the first `}` at the item's own
# indentation. `just fmt` is what makes that exact. **When the closing brace is
# not where rustfmt would put it this fails and says so**, rather than printing
# to the end of the file — an item that runs on for 7 000 lines is the failure
# this whole recipe is against.
[doc("One item of Rust source by name, with its doc comment: `just src Decision`.")]
src want filter="":
    #!/usr/bin/env bash
    set -euo pipefail
    want="$1"
    case "$want" in
    "")
        echo "which name? try 'just src arrange_faces'" >&2
        exit 2
        ;;
    *[!A-Za-z0-9_]*)
        echo "'$want' is not the name of a Rust item" >&2
        exit 2
        ;;
    esac

    root=$(git rev-parse --show-toplevel)
    filter="${2:-}"
    if [ -n "$filter" ]; then
        files=$(git -C "$root" ls-files -- "$filter" | grep '\.rs$' || true)
    else
        files=$(git -C "$root" ls-files -- '*.rs' || true)
    fi
    if [ -z "$files" ]; then
        echo "no Rust files are tracked${filter:+ under $filter}" >&2
        exit 1
    fi

    kw='fn|struct|enum|trait|type|union|mod|static'
    lead='^[[:space:]]*(pub(\([^)]*\))?[[:space:]]+)?(default[[:space:]]+)?((const|async|unsafe)[[:space:]]+)*(extern[[:space:]]+"[^"]*"[[:space:]]+)?'
    hits=$(cd "$root" && grep -nHE \
        "$lead(($kw)[[:space:]]+$want\b|const[[:space:]]+$want[[:space:]]*:|macro_rules![[:space:]]*$want\b)" \
        -- $files || true)

    if [ -z "$hits" ]; then
        echo "no Rust item named $want${filter:+ under $filter}" >&2
        exit 1
    fi

    if [ "$(printf '%s\n' "$hits" | wc -l)" -gt 1 ]; then
        printf '%s\n' "$hits" | awk '{
            p = index($0, ":"); file = substr($0, 1, p - 1); rest = substr($0, p + 1)
            q = index(rest, ":"); ln = substr(rest, 1, q - 1); code = substr(rest, q + 1)
            sub(/^[ \t]+/, "", code)
            printf "  %-52s %s\n", file ":" ln, code
        }'
        exit 0
    fi

    file=${hits%%:*}
    rest=${hits#*:}
    awk -v L="${rest%%:*}" -v path="$file" '
        { a[NR] = $0 }
        END {
            def = a[L]
            indent = match(def, /[^ ]/) - 1
            pad = substr(def, 1, indent)

            start = L
            while (start > 1) {
                t = a[start - 1]
                sub(/^[ \t]+/, "", t); sub(/[ \t]+$/, "", t)
                if (t == "" || t ~ /\{$/ || t ~ /^\}/) break
                start--
            }

            trimmed = def; sub(/[ \t]+$/, "", trimmed)
            if (trimmed ~ /;$/ || (index(def, "{") > 0 && trimmed ~ /\}$/)) {
                end = L
            } else {
                for (i = L + 1; i <= NR; i++)
                    if (substr(a[i], 1, indent + 1) == pad "}") { end = i; break }
            }
            if (end == 0) {
                print path ":" L " opens an item that has no closing brace at its own" \
                    " indentation — run just fmt, or read it by hand" > "/dev/stderr"
                exit 1
            }

            print path ":" L
            for (i = start; i <= end; i++) print a[i]
        }' "$root/$file"

# Pins the extraction against a fixture repo rather than against the tree, so it
# stays proven on a day the real `arrange_faces` is rewritten. `git ls-files` is
# what the recipe searches, so the fixture is a real index — `git add` with no
# commit is enough for that, and needs no identity configured.
[private]
check-src-selftest:
    #!/usr/bin/env bash
    set -euo pipefail
    dir=$(mktemp -d)
    trap 'rm -rf "$dir"' EXIT
    mkdir -p "$dir/repo/src"
    cat > "$dir/repo/src/a.rs" <<'RS'
    /// Doc for alpha.
    #[derive(Debug)]
    pub struct Alpha {
        pub x: f64,
    }

    pub fn shared() -> u8 {
        1
    }

    const LIMIT: usize = 3;

    impl Alpha {
        /// Doc for the method.
        pub fn beta(
            &self,
        ) -> f64 {
            self.x
        }
    }
    RS
    cat > "$dir/repo/src/b.rs" <<'RS'
    pub fn shared() -> u8 {
        2
    }
    RS
    git -C "$dir/repo" init -q
    git -C "$dir/repo" add -A
    j="{{just_executable()}} --justfile {{justfile()}} --working-directory $dir/repo"

    got=$($j src Alpha)
    want=$(printf '%s\n' "src/a.rs:3" "/// Doc for alpha." "#[derive(Debug)]" \
        "pub struct Alpha {" "    pub x: f64," "}")
    [ "$got" = "$want" ] || { echo "src Alpha drifted:"; \
        echo "--- got ---"; echo "$got"; echo "--- want ---"; echo "$want"; exit 1; }

    got=$($j src beta)
    want=$(printf '%s\n' "src/a.rs:15" "    /// Doc for the method." "    pub fn beta(" \
        "        &self," "    ) -> f64 {" "        self.x" "    }")
    [ "$got" = "$want" ] || { echo "src beta drifted:"; \
        echo "--- got ---"; echo "$got"; echo "--- want ---"; echo "$want"; exit 1; }

    got=$($j src LIMIT)
    want=$(printf '%s\n' "src/a.rs:11" "const LIMIT: usize = 3;")
    [ "$got" = "$want" ] || { echo "src LIMIT drifted:"; \
        echo "--- got ---"; echo "$got"; echo "--- want ---"; echo "$want"; exit 1; }

    got=$($j src shared | awk '{ $1 = $1; print }')
    want=$(printf '%s\n' "src/a.rs:7 pub fn shared() -> u8 {" "src/b.rs:1 pub fn shared() -> u8 {")
    [ "$got" = "$want" ] || { echo "a name defined twice did not list both:"; \
        echo "--- got ---"; echo "$got"; echo "--- want ---"; echo "$want"; exit 1; }

    got=$($j src shared src/b.rs)
    want=$(printf '%s\n' "src/b.rs:1" "pub fn shared() -> u8 {" "    2" "}")
    [ "$got" = "$want" ] || { echo "the path filter did not narrow to one:"; \
        echo "--- got ---"; echo "$got"; echo "--- want ---"; echo "$want"; exit 1; }

    if $j src no_such_item >/dev/null 2>&1; then
        echo "src found an item that does not exist"; exit 1
    fi

    if $j src 'fn beta' >/dev/null 2>&1; then
        echo "src accepted something that is not an item name"; exit 1
    fi

# Which file `recipe` below reads. A variable so `check-recipe-selftest` can
# point it at a fixture: the subject here is a justfile, and a selftest that
# read the real one would be pinned to whatever recipes happen to exist today.
recipes_file := "justfile"

# One recipe of this justfile, by name, with the whole comment block above it.
#
#     just recipe gate
#     just recipe hub          no such recipe: lists the three whose names contain it
#     just recipe              every definition, including the ones `list` hides
#
# `just --show <name>` already prints the attributes, the header and the body.
# What it prints of the comment block is **one line**, because that is just's
# own doc-comment rule — and in this file the comment block is where the
# argument lives: 22 lines above `src`, 6 above `gate_steps`. So `--show`
# truncates exactly the part a reader came for. And `just --show gate_steps`
# answers `error: Justfile does not contain recipe`: the twenty `:=`
# assignments are reachable by no reader at all. This prints both, whole.
#
# The habit it replaces is `grep -n '^check-hub' justfile` followed by a `Read`
# at a guessed offset — the two-call shape §144.6 measured, on the file it
# measured second. This justfile is 307 404 characters over 105 definitions, and
# the mean answer here is 2 934 of them: a hundredth of the file, in one call.
# The largest, `check-fuzz-overnight-selftest`, is 40 344.
#
# **The extent is exact rather than guessed**, which is the one place this beats
# `just src`. `src` reads a Rust item's end off rustfmt's layout and fails when
# the closing brace is not where rustfmt would put it. Here indentation *is*
# just's grammar: a body is every following line that is blank or indented, and
# no line inside one can begin at column 0. Nothing is guessed and there is no
# way to run on to the end of the file.
#
# **A name that matches nothing fails**, as `decisions-find` does and for the
# same reason: an empty answer reads as "this justfile has no such recipe", and
# that is a claim a typo can forge. A name that is not exact but is contained in
# several names lists them; contained in exactly one, it prints that one.
#
# **No name at all is the index rather than an error.** It is the only listing
# that shows the `_`-prefixed and `[private]` definitions — `just list` hides
# every selftest on purpose, and they are the ones nobody can find — and it is
# where the selftest's sweep over the real file gets its single call.
#
# **A listed row's one line is the `[doc]` text, or the comment block's *first*
# line — not its last, which is just's own rule.** `list` above records what the
# last line gives here: "35 rows of fragments torn out of mid-sentence". Every
# comment block in this file opens with its thesis, so the first line is a whole
# sentence, and the `[private]` definitions — which carry no `[doc]` precisely
# so `list` will hide them — get a description instead of nothing.
#
# Aliases are not resolved. This justfile has none, and §13 does not build a
# part with no consumer; one would fail here loudly rather than answer wrongly.
[doc("One recipe of this justfile by name, with its comment: `just recipe gate`.")]
recipe want="":
    #!/usr/bin/env bash
    set -euo pipefail
    want="$1"
    case "$want" in
    *[!A-Za-z0-9_-]*)
        echo "'$want' is not the name of a recipe or a variable" >&2
        exit 2
        ;;
    esac

    awk -v want="$want" -v path="{{recipes_file}}" '
        function defname(s,   n, rest, c) {
            sub(/^(export|unexport)[ \t]+/, "", s)
            if (match(s, /^[A-Za-z_][A-Za-z0-9_-]*/) == 0) return ""
            n = substr(s, 1, RLENGTH)
            rest = substr(s, RLENGTH + 1)
            if (rest ~ /^[ \t]*:=/) return n
            c = index(rest, ":")
            if (c == 0 || substr(rest, c + 1, 1) == "=") return ""
            return n
        }

        function blockstart(h,   s) {
            s = h
            while (s > 1 && (a[s - 1] ~ /^\[/ || a[s - 1] ~ /^#/)) s--
            return s
        }

        function blockend(h,   i, e) {
            e = h
            for (i = h + 1; i <= last; i++) {
                if (a[i] ~ /^[ \t]/ || a[i] == "") { e = i; continue }
                break
            }
            while (e > h && a[e] ~ /^[ \t]*$/) e--
            return e
        }

        function oneline(h,   i, t, p, q, first) {
            for (i = h - 1; i >= 1; i--) {
                t = a[i]
                if (t ~ /^\[doc\("/) {
                    p = index(t, "\"")
                    q = index(substr(t, p + 1), "\")]")
                    if (q > 0) return substr(t, p + 1, q - 1)
                }
                if (t ~ /^\[/) continue
                if (t ~ /^#/) {
                    t = substr(t, 2); sub(/^ /, "", t)
                    if (t != "") first = t
                    continue
                }
                break
            }
            return first
        }

        function show(j,   s, e, i) {
            s = blockstart(at[j]); e = blockend(at[j])
            print path ":" at[j]
            for (i = s; i <= e; i++) print a[i]
        }

        function row(j,   line) {
            line = sprintf("  %-26s %-32s %s", path ":" at[j], name[j], oneline(at[j]))
            sub(/[ \t]+$/, "", line)
            print line
        }

        { a[NR] = $0; last = NR }

        END {
            for (i = 1; i <= last; i++) {
                n = defname(a[i])
                if (n == "") continue
                k++
                name[k] = n
                at[k] = i
                if (n == want) exact = k
            }
            if (want == "") {
                for (j = 1; j <= k; j++) row(j)
                exit 0
            }
            if (exact) { show(exact); exit 0 }
            for (j = 1; j <= k; j++)
                if (index(tolower(name[j]), tolower(want)) > 0) hit[++m] = j
            if (m == 0) {
                print "no recipe or variable in " path " is named " want ", or contains it" \
                    > "/dev/stderr"
                exit 1
            }
            if (m == 1) { show(hit[1]); exit 0 }
            for (j = 1; j <= m; j++) row(hit[j])
        }' "{{recipes_file}}"

# Pins the extraction against a fixture rather than against this file, so it
# stays proven on a day `gate` is rewritten — the same reason
# `check-src-selftest` builds a fixture repo of its own.
#
# **The sweep at the end is the other half, and it is the half about silence.**
# A definition the scanner cannot see does not announce itself: it drops out of
# the index and out of every listing, and `just recipe <it>` then says the file
# has no such recipe — the empty answer the recipe above refuses to give. So the
# scanner's own list is held against two greps written a different way, which is
# what makes a new header shape — an alias, an attribute, a colon inside a
# default — fail here rather than disappear.
[private]
check-recipe-selftest:
    #!/usr/bin/env bash
    set -euo pipefail
    dir=$(mktemp -d)
    trap 'rm -rf "$dir"' EXIT
    cat > "$dir/fixture" <<'JF'
    # The first line of alpha's argument.
    # The last line of alpha's argument.
    [doc("Alpha does a thing.")]
    alpha:
        echo one

        echo two

    # The first line of beta's argument.
    # The last line of beta's argument.
    [private]
    beta arg="x":
        echo beta

    # What the answer is measured against.
    export answer := "42"

    alpha-extra:
        echo three
    JF
    j="{{just_executable()}} --justfile {{justfile()}} --set recipes_file $dir/fixture"
    f="$dir/fixture"

    got=$($j recipe alpha)
    want=$(printf '%s\n' "$f:4" "# The first line of alpha's argument." \
        "# The last line of alpha's argument." '[doc("Alpha does a thing.")]' \
        "alpha:" "    echo one" "" "    echo two")
    [ "$got" = "$want" ] || { echo "recipe alpha drifted:"; \
        echo "--- got ---"; echo "$got"; echo "--- want ---"; echo "$want"; exit 1; }

    got=$($j recipe beta)
    want=$(printf '%s\n' "$f:12" "# The first line of beta's argument." \
        "# The last line of beta's argument." "[private]" \
        'beta arg="x":' "    echo beta")
    [ "$got" = "$want" ] || { echo "a [private] recipe drifted:"; \
        echo "--- got ---"; echo "$got"; echo "--- want ---"; echo "$want"; exit 1; }

    got=$($j recipe answer)
    want=$(printf '%s\n' "$f:16" "# What the answer is measured against." \
        'export answer := "42"')
    [ "$got" = "$want" ] || { echo "an exported assignment drifted:"; \
        echo "--- got ---"; echo "$got"; echo "--- want ---"; echo "$want"; exit 1; }

    got=$($j recipe alph | awk '{ print $2 }')
    want=$(printf '%s\n' "alpha" "alpha-extra")
    [ "$got" = "$want" ] || { echo "an inexact name did not list both:"; \
        echo "--- got ---"; echo "$got"; echo "--- want ---"; echo "$want"; exit 1; }

    $j recipe alph | grep -q 'Alpha does a thing.' || { \
        echo "the listing dropped the [doc] text"; exit 1; }

    # A `[private]` recipe has no [doc], so its row is the comment block's first
    # line. just's own rule would take the last, which is `list`'s fragments.
    row=$($j recipe | awk '$2 == "beta"')
    case "$row" in
    *"The first line of beta's argument."*) ;;
    *) echo "a row without [doc] is not the comment block's first line: $row"; exit 1 ;;
    esac

    got=$($j recipe extra)
    want=$(printf '%s\n' "$f:18" "alpha-extra:" "    echo three")
    [ "$got" = "$want" ] || { echo "one inexact match did not print the recipe:"; \
        echo "--- got ---"; echo "$got"; echo "--- want ---"; echo "$want"; exit 1; }

    got=$($j recipe | awk '{ print $2 }')
    want=$(printf '%s\n' "alpha" "beta" "answer" "alpha-extra")
    [ "$got" = "$want" ] || { echo "the index is not every definition in order:"; \
        echo "--- got ---"; echo "$got"; echo "--- want ---"; echo "$want"; exit 1; }

    if $j recipe no_such_recipe >/dev/null 2>&1; then
        echo "recipe found something that does not exist"; exit 1
    fi

    if $j recipe 'alpha beta' >/dev/null 2>&1; then
        echo "recipe accepted something that is not a name"; exit 1
    fi

    # The sweep. Both sides name the real file; neither is allowed to be empty,
    # because two empty lists compare equal and prove nothing.
    got=$({{just_executable()}} --justfile {{justfile()}} recipe \
        | awk '{ split($1, p, ":"); print p[2], $2 }' | sort -n)
    want=$( { grep -nE '^(export[ \t]+)?[A-Za-z_][A-Za-z0-9_-]*[ \t]*:=' {{justfile()}}; \
              grep -nE '^[A-Za-z_][A-Za-z0-9_-]*([ \t][^:]*)?:([^=]|$)' {{justfile()}}; } \
        | sed -E 's/^([0-9]+):(export[ \t]+)?([A-Za-z_][A-Za-z0-9_-]*).*/\1 \3/' | sort -n)
    n=$(printf '%s\n' "$want" | grep -c .)
    if [ "$n" -lt 80 ]; then
        echo "the greps found $n definitions in {{justfile()}}, so the sweep compared nothing"
        exit 1
    fi
    if [ "$got" != "$want" ]; then
        echo "the scanner and a plain grep disagree about what this justfile defines:"
        diff <(printf '%s\n' "$want") <(printf '%s\n' "$got") || true
        echo "a definition only one side sees is one the index drops in silence"
        exit 1
    fi

[doc("The whole test suite, and the doc tests.")]
test:
    cargo nextest run --workspace
    cargo test --workspace --doc

# The tests that are about what a release build does, built the way it ships.
#
# `test` above runs every one of its 1820 tests with `debug_assertions` **on**:
# `Cargo.toml` has no `[profile.release]` section and `[profile.test]` sets only
# `opt-level`, so until this recipe nothing that gates had ever built the
# shipping code path. That is not a speed question — the two profiles measured
# within 3% of each other on the same workload — it is that every
# `debug_assert!` and every `cfg!(debug_assertions)` branch is in all 1820 and
# in none of what a user gets. `ModelId` in `crates/rcad-topo/src/key.rs` is a
# counter with the assertions on and a zero-sized type without them, which
# changes what the kernel *answers*, not how fast it answers.
#
# Filtered, rather than a release arm over the whole gate. The suite is the
# slowest thing anybody here waits for and a second copy of it would buy the
# handful of tests this is about at the price of every other test being run
# twice. So one crate is built in release, and one test binary in it.
#
# `--no-tests fail` is what makes the arm worth trusting. Its tests sit behind
# `#![cfg(not(debug_assertions))]`, so a run that quietly came up in the dev
# profile — or a filter that stopped matching — selects nothing, and a run of no
# tests otherwise reports itself as a pass. `check-ignored` above says the same
# thing about itself for the same reason.
[doc("The tests about what a release build does, with the assertions off.")]
check-release-build:
    cargo nextest run --cargo-profile release -p rcad-topo --test what_only_a_release_build_does --no-tests fail

# Where the open questions live: one file per session, under the session's own
# scratchpad and never in the tree. Overridable so the selftest can point at a
# directory it builds itself, rather than at whatever sessions are live today.
questions_root := env_var_or_default("CLAUDE_QUESTIONS_ROOT", env_var_or_default("TMPDIR", "/tmp") / ("claude-" + `id -u`))

# The parser, the hooks and the shared skills live in plugins and not in this
# repo, because a copy in each repo is a copy that goes stale: `rscene` carried
# a parser that still dropped the second label of a paired entry months after
# the fix landed here.
#
# The name Claude Code knows them by, which is not derivable from the paths
# below and is not this repo's to pick: it comes from the marketplace's own
# `marketplace.json`, and an `extraKnownMarketplaces` key that disagrees with it
# registers nothing, silently.
marketplace := env_var_or_default("CLAUDE_MARKETPLACE", "mattiasgronlund-local")

# The config directory is the installation — settings, `known_marketplaces.json`,
# `installed_plugins.json` — so its absence is what "there is no Claude Code
# here" means. The recipes that need a plugin skip on it, and on `CI` never: a
# human who exports `CI` would lose the check silently, and a runner that does
# not set it would go red with nothing to act on. `docs/decisions.md` §77.
claude_config := env_var_or_default("CLAUDE_CONFIG_DIR", home_directory() / ".claude")

# Three places can hold the plugins, in this order:
#
#   `CLAUDE_PLUGINS_ROOT` — the escape hatch. A checkout elsewhere, or the
#   working tree when the point is to change a plugin and see the change. It is
#   also how one repo tries a version the shared pin has not taken: the
#   marketplace name is fixed, so one name carries one ref and this is what
#   independence looks like instead.
#
#   The marketplace clone, which is the **pinned** tree. `claude plugin
#   marketplace add mattiasgronlund/claude-questions@v0.2.0` clones the tag here
#   and leaves it, so a recipe reading it runs the version the session loaded.
#
#   `$HOME/git/…` — a machine with the repo checked out that never added the
#   marketplace.
#
# Not the install path. A github source installs under
# `<config>/plugins/cache/<marketplace>/<plugin>/<version>` — measured, and
# version-stamped — so a recipe naming it breaks at the next bump in the
# direction of "no questions found", which is the failure this whole apparatus
# exists to prevent. The clone above does not move.
#
# `$HOME`-relative and overridable rather than absolute, so a clone somewhere
# else still resolves.
marketplace_clone := claude_config / "plugins/marketplaces" / marketplace / "plugins"
plugins_root := env_var_or_default("CLAUDE_PLUGINS_ROOT", if path_exists(marketplace_clone) == "true" { marketplace_clone } else { home_directory() / "git/mattiasgronlund/claude-questions/plugins" })
questions_plugin := env_var_or_default("CLAUDE_QUESTIONS_PLUGIN", plugins_root / "questions")
guardrails_plugin := env_var_or_default("CLAUDE_GUARDRAILS_PLUGIN", plugins_root / "guardrails")

# The open questions, in full, by the hash the status line prints beside them.
#
#     just question                 the sessions that have open questions
#     just question 766405          every open question in that session, in full
#     just question 766405 Q19 Q21  those two, answered or not
#     just question 766405 --labels one line per entry, the old default
#     just question 766405 --all    the answered ones too
#     just question 766405 --json   the whole file, for a program
#
# **This recipe is for an agent. `just question-tui` is the one a person types.**
# That split is what sets the default. Terseness here used to be justified by
# what "you came for" and by a finite screen — true of a human at a terminal,
# and false of the caller this actually has. A call is billed the context it is
# made at, not the bytes it returns: across the five longest transcripts in this
# repo the mean turn bills 144 k-182 k input against 850-1 300 output, 97 % of it
# cache reads. One avoidable follow-up costs about what carrying 2 200 extra
# resident tokens for 73 further turns costs, and the largest question file in
# the tree is 2 200 tokens whole. Drilling down is a human affordance and an
# agent tax. `docs/decisions.md` §100.
#
# **The stronger reason is that a malformed entry is only readable here.** It has
# no label to print, so `--labels` can say no more than `??` with a line number
# and the status line no more than `??×1`; the TUI drops it outright. Printing
# bodies by default is what puts a broken entry in front of someone able to fix
# it. That would be worth doing at zero token cost.
#
# **The hash exists because nothing else knows the address.** The file is named
# by the session id, which only the harness has, so reading one question meant
# `!cat` on the whole file, and shell mode then charges the model for every line
# of it on every later turn.
#
# **This and `statusline.sh` are the same parser**, so an entry one of them would
# drop cannot be one the other shows. They used to be two implementations of one
# grammar held in step by a comment, and they disagreed: an entry carrying two
# labels, `**Q7 / Q11**`, counted as one, and Q11 and Q27 stayed open and
# invisible through 2026-09-06. That entry is also why naming two labels prints
# one body: what is deduped is the entry, not the label.
#
# There is deliberately no recipe that *writes* a question. A body is paragraphs
# of prose with backticks and `§` refs in it, so passing one as a shell argument
# costs what `Edit` costs and echoes the whole thing back on top; and marking one
# answered is a substitution, which is the quiet-failure shape §50.8 is about.
# For the caller this recipe is written for, `Write` already is that API.
[doc("The open questions in full, for an agent. `question-tui` is the human one.")]
question hash="" *args:
    #!/usr/bin/env bash
    set -euo pipefail
    q="{{questions_plugin}}/bin/questions.py"

    if [ -z "{{hash}}" ]; then
        exec python3 "$q" sessions "{{questions_root}}"
    fi
    file=$(python3 "$q" find "{{questions_root}}" "{{hash}}")

    mode=open
    labels=()
    given=({{args}})
    for arg in ${given[@]+"${given[@]}"}; do
        case "$arg" in
        --labels) mode=labels ;;
        --all) mode=all ;;
        --json) mode=json ;;
        --*) printf 'unknown flag %s — try --labels, --all or --json\n' "$arg" >&2; exit 2 ;;
        *) labels+=("$arg") ;;
        esac
    done

    # Before anything is printed: a program asked for the file, so it gets the
    # file and nothing appended to it.
    [ "$mode" = json ] && exec python3 "$q" json "$file"

    # A named label bypasses the open filter. Asking for Q2 and being told there
    # is no such question, because Q2 was answered an hour ago, is a worse
    # failure than the one the filter saves.
    if [ ${#labels[@]} -gt 0 ]; then
        python3 "$q" entry "$file" "${labels[@]}"
    elif [ "$mode" = labels ]; then
        python3 "$q" labels "$file"
    elif [ "$mode" = all ]; then
        python3 "$q" entries "$file" --all
    else
        python3 "$q" entries "$file"
    fi

    # The violations are named, not counted. A count is not actionable, and the
    # second call that makes it actionable pays for the whole conversation again
    # to learn three lines — 150 k tokens of context for a few hundred bytes.
    problems=$(python3 "$q" check "$file" || true)
    if [ -n "$problems" ]; then
        printf '\n'
        sed 's/^/  ⚠ /' <<<"$problems"
    fi

# The same questions, to read and answer in rather than to print.
#
# **Run it in a terminal of its own, never in a session's shell mode.** It is
# full-screen curses, so a session running it takes every frame it draws into
# the conversation and pays for them on every later turn — the cost the status
# line and `just question` exist to avoid. That is why it is a recipe of its own
# and not a flag on `question`: a flag is one keystroke from being typed in the
# wrong terminal.
#
# It never writes `open-questions.md`. It composes the answers into the text you
# would have typed and puts it on your clipboard; you paste that into the
# session's own terminal, where it arrives as your turn. An answer to a grilling
# round is consent, and consent cannot arrive as a message some other process
# sent on your behalf — the `asking-questions` skill has the argument, and it is
# the same rule as "a relayed approval is not an approval".
[doc("Read and answer the open questions. A terminal of its own, not shell mode.")]
question-tui *hash:
    @python3 {{questions_plugin}}/bin/questions-tui.py {{hash}}

# The grammar violations in one session's file, in full.
#
# **It takes a hash rather than checking everything, and that is the exemption
# mechanism.** A list of grandfathered sessions would outlive what it names —
# `/tmp` is swept at 30 days, so every id in such a list dangles within a month —
# and it would be a second record beside the thing itself, which is the shape
# §60 rejected for the hub. A dead session's file cannot be fixed, so judging it
# is noise nobody can act on; a live session's can, so it is judged.
[doc("The grammar violations in one session's question file, named in full.")]
question-check hash:
    #!/usr/bin/env bash
    set -euo pipefail
    q="{{questions_plugin}}/bin/questions.py"
    file=$(python3 "$q" find "{{questions_root}}" "{{hash}}")
    exec python3 "$q" check "$file"

# The grammar's own 20 cases live with the parser, in the `questions` plugin.
# What is left here is the seam: that the plugin is where this repo thinks it
# is, and that `just question` — which is this repo's recipe, not the plugin's —
# still reads a file end to end and still announces a violation under the
# listing. The footer is rcad's, so rcad tests it.
#
# Keeping the recipe's name is deliberate: `check-gate-runs-every-check`
# counts `check-*` recipes and this one moving out of that set would be a
# silent change to a number that exists to be loud.
#
# Skipped where there is no Claude Code. These are the plugin's own 20 cases and
# the marketplace repo's CI runs them on every push there; running them again
# out of a clone this job made was checking that repo's tree rather than this
# one's. The seam this file does own is in `check-plugins`, which exercises
# `just question` end to end.
[private]
check-questions-selftest:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ ! -d "{{claude_config}}" ]; then
        echo "no {{claude_config}}, so no Claude Code and no plugin here: the questions cases are skipped, not passed"
        exit 0
    fi
    exec {{questions_plugin}}/bin/selftest.sh

# The three plugins this repo's `.claude/` was emptied into, checked for being
# where the recipes above think they are.
#
# It is its own recipe rather than a guard inside each caller because the answer
# is the same for all of them and it is four lines of install instructions. It
# runs early in `gate` so a missing plugin says so once, in words, instead of
# three times as "no such file or directory".
#
# `practices` ships only skills, so nothing here runs it — its directory
# existing is the whole check. A skill that fails to load fails visibly, in the
# session, which is not true of a hook or a parser.
#
# **Two questions, and the first one alone was not enough.** Where the files are
# is the bootstrap and has to be asked here, because it is what runs when the
# plugin is missing. Whether Claude Code has been *told* about them is a
# different question, and `rscene` answered it the hard way on 2026-09-08: three
# plugin directories present, this recipe green, and nothing loaded — no hooks,
# no skills, no message saying so. A directory is not an install.
#
# The second question is asked by the plugin, because the answer is the same for
# both repos and the copy is what goes stale. It cannot prove *this* session
# loaded anything — settings are read at session start — so it rules out the
# silent case and says exactly that rather than claiming more.
[doc("The three Claude plugins are on disk, and Claude Code has been told.")]
check-plugins:
    #!/usr/bin/env bash
    set -euo pipefail
    # Both questions below presuppose a Claude Code, and so does the seam test
    # at the end, which runs `just question` against the plugin's parser. Where
    # there is none, all three are skipped and this says which.
    if [ ! -d "{{claude_config}}" ]; then
        echo "no {{claude_config}}, so no Claude Code to have been told anything: the plugin checks are skipped, not passed"
        exit 0
    fi
    missing=0
    for p in "{{questions_plugin}}" "{{guardrails_plugin}}" "{{plugins_root}}/practices"; do
        [ -d "$p" ] && continue
        echo "not where this repo expects it: $p"
        missing=1
    done
    if [ "$missing" -eq 1 ]; then
        # The ref comes out of the committed settings file rather than being
        # written here, so this cannot name a version the repo is not asking
        # for.
        pin=$(jq -r '.extraKnownMarketplaces["{{marketplace}}"].source.ref // "main"' \
            .claude/settings.json 2>/dev/null || echo main)
        echo
        echo "install them with:"
        echo "  claude plugin marketplace add mattiasgronlund/claude-questions@$pin"
        echo "  claude plugin install -y questions@{{marketplace}}"
        echo "  claude plugin install -y guardrails@{{marketplace}}"
        echo "  claude plugin install -y practices@{{marketplace}}"
        echo "or point CLAUDE_PLUGINS_ROOT at your checkout's plugins directory"
        exit 1
    fi
    echo "3 plugins, all where this repo expects them"
    # On disk is not installed: clone the marketplace and never register it and
    # the files are all there while a session loads no hooks and no skills.
    # `rscene` hit exactly that. So this asks Claude Code whether it has been
    # *told* about them, which is a different question from the loop above.
    #
    # It skips where there is no Claude Code to have been told, and says so
    # rather than passing quietly. That guard used to be four lines here; it is
    # in the script now, with a selftest, because it is the same guard `rscene`
    # needs and a copy per repo is a copy that goes stale. `docs/decisions.md`
    # §77.
    "{{guardrails_plugin}}/bin/check-declared.sh" {{marketplace}} questions guardrails practices

    dir=$(mktemp -d)
    trap 'rm -rf "$dir"' EXIT
    scratch="$dir/-slug-rcad/76640573-f351-4e74/scratchpad"
    mkdir -p "$scratch"
    printf '%s\n' \
        '- [ ] **Q7 / Q11** — a floor unit with no record' \
        '' \
        '  An indented body, mentioning M5 in passing.' \
        '' \
        '- [ ] **Q1** — options in prose, so the footer has something to count: (a) or (b)?' \
        > "$scratch/open-questions.md"
    j="{{just_executable()}} --set questions_root $dir"

    got=$($j question)
    want="  766405   3 open  $(date -d "@$(stat -c %Y "$scratch/open-questions.md")" +%H:%M)  rcad"
    [ "$got" = "$want" ] || { echo "the session list drifted:"; \
        echo "--- got ---"; echo "$got"; echo "--- want ---"; echo "$want"; exit 1; }

    # Captured before it is matched: `grep -q` closes the pipe on its first
    # hit, and under `pipefail` that SIGPIPE reads exactly like the output
    # having failed.
    open=$($j question 766405)
    grep -q 'indented body' <<<"$open" \
        || { echo "the default did not print bodies, only headings"; exit 1; }

    # Named, not counted. The count used to be the whole footer and cost a
    # second call at full context to find out what it meant.
    grep -q 'stated in prose but not marked' <<<"$open" \
        || { echo "the footer did not name the violation it found"; exit 1; }
    if grep -q 'grammar violation' <<<"$open"; then
        echo "the footer counted violations instead of naming them"; exit 1
    fi

    listing=$($j question 766405 --labels)
    grep -q 'Q7, Q11' <<<"$listing" \
        || { echo "--labels did not reach the plugin's parser"; exit 1; }
    grep -q 'indented body' <<<"$listing" \
        && { echo "--labels printed bodies"; exit 1; }

    # One entry, two labels. Deduping by label rather than by entry prints the
    # body twice and reads as two questions that happen to be identical.
    both=$($j question 766405 Q7 Q11)
    [ "$(grep -c 'indented body' <<<"$both")" = 1 ] \
        || { echo "an entry with two labels printed more than once"; exit 1; }

    entry=$($j question 766405 Q11)
    grep -q 'indented body' <<<"$entry" \
        || { echo "an entry did not come back in full"; exit 1; }
    python3 -c 'import json,sys; json.load(sys.stdin)' <<<"$($j question 766405 --json)" \
        || { echo "--json did not come back as json"; exit 1; }
    if $j question 766405 --nonsense >/dev/null 2>&1; then
        echo "an unknown flag was accepted"; exit 1
    fi
    if $j question zzzz >/dev/null 2>&1; then
        echo "a hash matching no session was accepted"; exit 1
    fi

# One test, or a few, with the corpus and the build cache already in place.
#
# `just test` is the whole workspace and takes minutes, so a lane wanting a
# single test reached past `just` for `mise exec -- cargo nextest run ...` and
# hand-typed the corpus path with it. On 2026-09-05 that was 456 targeted runs
# against 104 full ones, which makes the fast path the common one and the one
# worth making correct. Everything after the recipe name goes to nextest:
#
#     just t how_often_a_fuse_with_a_cone_refuses
#     just t -p rcad-geom
#     just t -E 'test(/walk/)'
#
# **The run is captured and digested rather than streamed**, because what a
# reader needs from a test run is one line when it passes and the *first* panic
# when it does not. The habit this replaces was `... 2>&1 | tail -40`, which is
# backwards on both counts: on a pass it prints 40 lines of `PASS`, and on a
# failure it prints the summary block and cuts off the panic that explains it.
# The full log is always at `target/last-test.log`, so nothing is lost — reading
# it back is one more call, and one that is only paid when it is wanted.
#
# `--no-capture` streams instead. It exists to watch output as it happens, and
# it also changes nextest's format, so digesting it would defeat the flag and
# misread it at the same time.
[doc("One test or a few, with the corpus and the build cache in place.")]
t *args:
    #!/usr/bin/env bash
    set -uo pipefail
    for arg in "$@"; do
        case "$arg" in
        --no-capture|--nocapture) exec cargo nextest run "$@" ;;
        esac
    done
    mkdir -p target
    log=target/last-test.log
    CARGO_TERM_COLOR=never cargo nextest run "$@" > "$log" 2>&1
    status=$?
    {{ just_executable() }} _test-digest "$log" "$status"
    exit $status

# The digest itself, kept apart from the run so `check-t-selftest` can hold it
# against logs whose answer is already known.
_test-digest log status:
    #!/usr/bin/env bash
    set -uo pipefail
    log='{{ log }}'
    if ! grep -qE '^ +Summary ' "$log"; then
        echo "the run did not get as far as a summary — first 40 lines:"
        head -40 "$log"
        echo "full log: $log"
        exit 0
    fi
    grep -E '^ +Summary ' "$log" | tail -1
    if [ '{{ status }}' = "0" ]; then
        exit 0
    fi
    awk '/^ +Summary /{s = 1} s && /^ +FAIL \[/' "$log"
    echo
    echo "--- the first failure in full ---"
    awk '!started && /^ +FAIL \[/ { started = 1; print; next }
         started && (/^ +(PASS|FAIL|SLOW|TRY|SKIP) \[/ || /^─/) { exit }
         started' "$log"
    failures=$(awk '/^ +Summary /{s = 1} s && /^ +FAIL \[/' "$log" | grep -c .)
    if [ "$failures" -gt 1 ]; then
        echo "$((failures - 1)) further failure(s) not shown."
    fi
    echo "full log: $log"

# Holds the digest against three logs whose right answer is known, rather than
# against whatever the suite happens to do today.
#
# The failing fixture carries a `PASS` line *between* its two failures on
# purpose. That is the shape a real `--no-fail-fast` run has, and a digest that
# printed "the first failure" by reading to the next `FAIL` would swallow the
# passing test and the second failure's header along with it.
#
# The third fixture has no summary at all, which is what a compile error looks
# like. It is the case where the head and not the tail is the useful end: rustc
# reports the first error first, and every later one may be its consequence.
[private]
check-t-selftest:
    #!/usr/bin/env bash
    set -euo pipefail
    dir=$(mktemp -d)
    trap 'rm -rf "$dir"' EXIT

    printf '%s\n' \
        '────────────' \
        '    Starting 2 tests across 125 binaries' \
        '        PASS [   0.005s] (1/2) probe first' \
        '        PASS [   0.006s] (2/2) probe second' \
        '────────────' \
        '     Summary [   0.007s] 2 tests run: 2 passed, 0 skipped' > "$dir/pass.log"

    got=$({{ just_executable() }} _test-digest "$dir/pass.log" 0)
    want='     Summary [   0.007s] 2 tests run: 2 passed, 0 skipped'
    [ "$got" = "$want" ] || { echo "a passing run should be one line:"; \
        echo "--- got ---"; echo "$got"; echo "--- want ---"; echo "$want"; exit 1; }

    printf '%s\n' \
        '        FAIL [   0.005s] (1/3) probe alpha' \
        '  stderr ───' \
        '' \
        "    thread 'alpha' panicked at src/lib.rs:9:9:" \
        '    the first failure has a message' \
        '' \
        '        PASS [   0.006s] (2/3) probe gamma' \
        '        FAIL [   0.007s] (3/3) probe beta' \
        '  stderr ───' \
        '' \
        "    thread 'beta' panicked at src/lib.rs:14:9:" \
        '    the second failure' \
        '' \
        '────────────' \
        '     Summary [   0.010s] 3 tests run: 1 passed, 2 failed, 0 skipped' \
        '        FAIL [   0.005s] (1/3) probe alpha' \
        '        FAIL [   0.007s] (3/3) probe beta' \
        'error: test run failed' > "$dir/fail.log"

    got=$({{ just_executable() }} _test-digest "$dir/fail.log" 1)
    want=$(printf '%s\n' \
        '     Summary [   0.010s] 3 tests run: 1 passed, 2 failed, 0 skipped' \
        '        FAIL [   0.005s] (1/3) probe alpha' \
        '        FAIL [   0.007s] (3/3) probe beta' \
        '' \
        '--- the first failure in full ---' \
        '        FAIL [   0.005s] (1/3) probe alpha' \
        '  stderr ───' \
        '' \
        "    thread 'alpha' panicked at src/lib.rs:9:9:" \
        '    the first failure has a message' \
        '' \
        '1 further failure(s) not shown.' \
        "full log: $dir/fail.log")
    [ "$got" = "$want" ] || { echo "the failing digest drifted:"; \
        echo "--- got ---"; echo "$got"; echo "--- want ---"; echo "$want"; exit 1; }

    printf '%s\n' \
        '   Compiling rcad-geom v0.0.0' \
        'error[E0425]: cannot find value `nope` in this scope' \
        'error: could not compile `rcad-geom`' > "$dir/broken.log"

    got=$({{ just_executable() }} _test-digest "$dir/broken.log" 101)
    case "$got" in
    *'did not get as far as a summary'*'cannot find value'*"full log: $dir/broken.log")
        ;;
    *)
        echo "a run that never reached a summary lost its errors:"; echo "$got"; exit 1
        ;;
    esac

    # The documented example, run for real, because it is the one that was broken.
    # It is here rather than beside the fixtures because it needs the test
    # binaries, which is why `gate` runs this recipe after `test`.
    got=$({{ just_executable() }} t -E 'test(/a_cube_comes_back_sound_and_measuring_the_same/)')
    case "$got" in
    *"1 test run: 1 passed"*)
        ;;
    *)
        echo "a parenthesised filter did not survive the recipe:"; echo "$got"; exit 1
        ;;
    esac

# `corpus/fetch.sh` rebuilds the STEP corpus from upstream, so a runner can have
# one without this repo carrying 133 MiB of other people's CAD files. The pins
# live in the script and the evidence behind them in `corpus/sources.md`.
#
# It refuses a destination that is not empty, which is the whole safety story: a
# corpus is somebody's only copy, it is in no git repository, and a mistyped
# path must not be a way to lose it. So rebuilding over an existing corpus means
# deleting it first, deliberately.
#
# Rebuilds the STEP corpus from upstream into `RCAD_STEP_CORPUS`.
[doc("Rebuild the STEP corpus from upstream into RCAD_STEP_CORPUS.")]
fetch-corpus destination=RCAD_STEP_CORPUS:
    ./corpus/fetch.sh "{{destination}}"

# CI fetches the STEP corpus (see `RCAD_STEP_CORPUS` above) with that script
# rather than having it on disk, so "CI is green" and "my machine is green" have
# to mean the same corpus. `corpus/manifest.sha256` pins the
# sha256 of every `.stp`/`.step` file the test suite reads (`files_under` in
# `crates/rcad-step/tests/every_file_in_the_corpus_reads.rs` — the walk that
# picks those two extensions, case-insensitively, and nothing else); this
# recipe is what checks a corpus against it.
#
# `sha256sum --check` already tells missing and altered apart in its own
# output — a listed file it could not open ends its line "FAILED open or
# read", one it read but that hashed wrong just ends "FAILED" — so this reads
# that rather than re-implementing it. What it cannot tell us is a `.stp`/
# `.step` file on disk the manifest never heard of, which means the corpus
# grew since the manifest was made; that half is a plain `find` against the
# manifest's own path list.
#
# Checks the STEP corpus on this machine against `corpus/manifest.sha256`.
[doc("The STEP corpus on this machine matches corpus/manifest.sha256.")]
check-corpus manifest="corpus/manifest.sha256":
    #!/usr/bin/env bash
    set -euo pipefail
    root="$RCAD_STEP_CORPUS"
    if [ ! -d "$root" ]; then
        echo "RCAD_STEP_CORPUS points at $root, which is not a directory"
        exit 1
    fi
    if [ ! -f "{{manifest}}" ]; then
        echo "{{manifest}} does not exist"
        exit 1
    fi
    manifest_abs=$(readlink -f "{{manifest}}")

    set +e
    out=$(cd "$root" && sha256sum --check --quiet "$manifest_abs" 2>&1)
    rc=$?
    set -e

    missing=$(grep ': FAILED open or read$' <<<"$out" | sed 's/: FAILED open or read$//' || true)
    mismatched=$(grep -v ': FAILED open or read$' <<<"$out" | grep ': FAILED$' | sed 's/: FAILED$//' || true)

    manifest_paths=$(cut -c67- "$manifest_abs" | sort)
    disk_paths=$(cd "$root" && find . -type f \( -iname '*.stp' -o -iname '*.step' \) -printf '%P\n' | sort)
    extra=$(comm -13 <(echo "$manifest_paths") <(echo "$disk_paths"))

    ok=1
    if [ -n "$missing" ]; then
        ok=0
        echo "named in {{manifest}} but not on disk under $root:"
        sed 's/^/  /' <<<"$missing"
    fi
    if [ -n "$mismatched" ]; then
        ok=0
        echo "on disk under $root but the contents do not match {{manifest}}:"
        sed 's/^/  /' <<<"$mismatched"
    fi
    if [ -n "$extra" ]; then
        ok=0
        echo "STEP files on disk under $root that {{manifest}} does not name:"
        sed 's/^/  /' <<<"$extra"
        echo "the corpus grew; regenerate {{manifest}} to cover the new files"
    fi
    if [ "$ok" -eq 1 ] && [ "$rc" -ne 0 ]; then
        echo "sha256sum --check failed in a way this recipe does not parse:"
        sed 's/^/  /' <<<"$out"
        exit 1
    fi
    if [ "$ok" -eq 0 ]; then
        exit 1
    fi
    echo "corpus matches {{manifest}}: $(wc -l <"$manifest_abs") files, $root"

# `check-corpus` above passed the first time it ran, on a corpus that was
# right, and that proved less than it looked like: run against corpora that
# were *wrong*, `rscene`'s copy of it named no file at all. These are the
# mutations that found that, kept as a recipe, so the next thing to silence an
# arm is caught by the gate rather than by somebody thinking to try it.
# `rscene`'s `docs/decisions.md` §12 is the finding; §79 here is what porting
# it to this repo changed. Written there and copied here, the same way
# `corpus/fetch.sh` was and for the same reason: their §12 holds the case list.
#
# **A synthetic corpus of six files, not a hardlinked copy of the real one.**
# Nothing here touches `RCAD_STEP_CORPUS`, which also means it runs on a
# machine that has never fetched a corpus — so it sits in `gate` *beside*
# `check-corpus` rather than behind it.
#
# **The filenames are the point.** `ab.stp`, `a-b.stp`, `aB.stp` and `a_c.stp`
# sort one way by byte and another by dictionary. That difference is the only
# thing that catches the bug this exists for: path lists sorted `LC_ALL=C`
# against a `comm` left in the ambient locale, which reads its input as
# disordered, exits 1, and takes the recipe down with `set -e`. Four ordinary
# names collate identically under both and would pass with that bug in place.
# `a_new.stp` is named to land *inside* the run where the two orders disagree,
# between `a_c.stp` and `ab.stp` by byte and elsewhere by dictionary, because
# `comm`'s order check only fires where it compares unequal lines.
#
# **This repo has no such bug to delete, and that is the difference from
# `rscene`.** All three of `check-corpus`'s sorts and its one `comm` are
# ambient, so they agree by construction and it is not exposed. There the
# selftest holds a fix; here it is a tripwire for an edit nobody has made. So
# the mutation that proves the fourth case is not vacuous is *adding*
# `LC_ALL=C` to the two sorts above while leaving the `comm` alone — see §79.2
# for what each mutation did.
#
# **One mutation at a time, and each case asserts the file its arm names**
# rather than the exit code. One corpus carrying three faults stops after the
# first under `set -e` and hides the other two; and an exit code standing in
# for an assertion is the layer the original bug hid inside.
[doc("check-corpus names the file for each way a corpus can be wrong.")]
check-corpus-selftest:
    #!/usr/bin/env bash
    set -euo pipefail

    dictionary=""
    for candidate in $(locale -a 2>/dev/null); do
        case "$candidate" in C|C.*|POSIX) continue ;; esac
        if [ "$(printf 'ab\na_c\n' | LC_ALL="$candidate" sort 2>/dev/null | head -1)" = "ab" ]; then
            dictionary="$candidate"
            break
        fi
    done
    if [ -z "$dictionary" ]; then
        echo "No locale here collates by dictionary order, so the case that"
        echo "would catch a collation bug in check-corpus cannot run — and the"
        echo "other three pass with that bug in place, which is worse than not"
        echo "running them at all. docs/decisions.md section 79."
        echo "  sudo locale-gen en_US.UTF-8 && sudo update-locale"
        exit 1
    fi

    work=$(mktemp -d -- "${TMPDIR:-/tmp}/rcad-corpus-selftest.XXXXXX")
    trap 'rm -rf -- "$work"' EXIT

    pristine="$work/pristine"
    mkdir -p "$pristine/nested"
    for name in ab.stp a-b.stp aB.stp a_c.stp; do
        printf 'ISO-10303-21;\n/* %s */\nEND-ISO-10303-21;\n' "$name" >"$pristine/$name"
    done
    printf 'ISO-10303-21;\nEND-ISO-10303-21;\n' >"$pristine/nested/deep one.step"
    printf 'not a STEP file\n' >"$pristine/notes.txt"

    manifest="$work/manifest.sha256"
    (cd "$pristine" \
        && find . -type f \( -iname '*.stp' -o -iname '*.step' \) -printf '%P\0' \
        | LC_ALL=C sort -z | xargs -0 sha256sum) >"$manifest"

    failed=0
    every_output=""
    check() {
        local name=$1 root=$2 want_rc=$3
        shift 3
        local out rc needle
        set +e
        out=$(RCAD_STEP_CORPUS="$root" LC_ALL="$dictionary" \
            {{ just_executable() }} check-corpus "$manifest" 2>&1)
        rc=$?
        set -e
        every_output+="$out"$'\n'
        if [ "$rc" -ne "$want_rc" ]; then
            echo "FAIL  $name: check-corpus exited $rc, wanted $want_rc"
            sed 's/^/        /' <<<"$out"
            failed=1
            return
        fi
        for needle in "$@"; do
            if ! grep -qF -- "$needle" <<<"$out"; then
                echo "FAIL  $name: nothing in the output named '$needle'"
                sed 's/^/        /' <<<"$out"
                failed=1
                return
            fi
        done
        echo "ok    $name"
    }

    mutate() {
        local into="$work/$1"
        rm -rf -- "$into"
        cp -a -- "$pristine" "$into"
        echo "$into"
    }

    # The happy path asserts two fragments rather than one sentence: this
    # repo's success line reads "corpus matches <manifest>: 5 files, <root>",
    # where `rscene`'s reads "5 files match". Copying their needle across gave
    # a case that passes without checking anything — on the one arm that is
    # supposed to be green, which is where nobody looks.
    check "a corpus that matches its manifest" "$pristine" 0 "corpus matches" "5 files"

    gone=$(mutate gone)
    rm -- "$gone/nested/deep one.step"
    check "a file the manifest names and disk lacks" "$gone" 1 \
        "not on disk" "nested/deep one.step"

    altered=$(mutate altered)
    printf 'ISO-10303-21;\n/* not what it was */\nEND-ISO-10303-21;\n' >"$altered/aB.stp"
    check "a file whose bytes changed" "$altered" 1 \
        "do not match" "aB.stp"

    grown=$(mutate grown)
    printf 'ISO-10303-21;\nEND-ISO-10303-21;\n' >"$grown/a_new.stp"
    check "a STEP file on disk the manifest never heard of" "$grown" 1 \
        "does not name" "a_new.stp"

    # `notes.txt` sits in all four corpora and is not a STEP file. If it ever
    # surfaces in an arm, `check-corpus`'s walk has stopped agreeing with
    # `files_under` in `crates/rcad-step/tests/every_file_in_the_corpus_reads.rs`,
    # which is the list `corpus/manifest.sha256` is defined against.
    if grep -qF 'notes.txt' <<<"$every_output"; then
        echo "FAIL  a file that is not STEP was reported by some arm:"
        grep -nF 'notes.txt' <<<"$every_output" | sed 's/^/        /'
        failed=1
    fi

    # No arm may leave `sha256sum`'s own wording in what it prints. The four
    # cases above cannot tell a trimmed name from an untrimmed one — a needle
    # is a substring, and `nested/deep one.step` sits inside
    # `nested/deep one.step: FAILED open or read` just as happily. Dropping
    # either of `check-corpus`'s two trims therefore passed all four when this
    # was first ported, which is a mutation surviving a suite written to catch
    # exactly that. Both are verified against this guard in §79.3.
    #
    # It is safe only while no case exercises `check-corpus`'s "failed in a way
    # this recipe does not parse" arm, which prints `sha256sum`'s output raw and
    # means to. A case that reaches that arm must assert on it rather than be
    # excused from this guard.
    if grep -qE 'FAILED( open or read)?$' <<<"$every_output"; then
        echo "FAIL  an arm printed sha256sum's wording rather than a bare name:"
        grep -nE 'FAILED( open or read)?$' <<<"$every_output" | sed 's/^/        /'
        failed=1
    fi

    [ "$failed" -eq 0 ] || exit 1
    echo "check-corpus named its file in every case, under $dictionary"

[doc("Formatting and clippy, warnings denied. Rust and TOML both.")]
lint:
    cargo fmt --all -- --check
    taplo fmt --check
    cargo clippy --workspace --all-targets -- -D warnings

[doc("cargo-deny: advisories, bans, licences, sources. No LGPL at runtime.")]
deny:
    cargo deny check

# The five fuzz recipes the overnight run starts. They run together, so this is
# the order they are started and waited on and not the order they get the
# machine — but it is still the order the summary line names them in.
#
# Recipe names rather than fuzz-target names, because what each target wants on
# its command line is the recipe's business and not this list's.
fuzz_all := "fuzz fuzz-reader fuzz-writer fuzz-fuse fuzz-cones-and-tori"

# Where a night's logs go. One directory per date, plus a one-line `latest`.
fuzz_log_root := env_var_or_default("RCAD_FUZZ_LOGS", env_var("HOME") / "rcad-fuzz-logs")

# Where systemd looks for the timer. An argument only so `check-fuzz-unit`'s
# selftest can point it somewhere hermetic; nothing else sets it.
systemd_user_dir := env_var_or_default("RCAD_SYSTEMD_USER", env_var("HOME") / ".config/systemd/user")

# Where the overnight run accumulates what it finds worth keeping.
#
# **Outside the tree, and that is the whole of decision §81.** The corpus under
# `fuzz/corpus/` is tracked, 30 MB of it; eight hours a night would add
# thousands of files to it, so the tree would be dirty every morning and every
# lane would merge past a thousand-file commit it never read. So the tracked
# corpus is handed to libFuzzer as a *read-only seed* and the night's own
# findings land here. What is worth keeping is promoted into `fuzz/corpus/` by
# hand, which is a thing somebody decided rather than a thing that accrued.
fuzz_corpus_root := env_var_or_default("RCAD_FUZZ_CORPUS", env_var("HOME") / "rcad-fuzz-corpus")

# Fuzz every target at once, inside a fixed window, unattended.
#
# `window` is seconds, and it is a *deadline* and not a budget: the window is
# measured against the clock once the build is done, so a slow build is paid for
# out of the same window rather than added to the end of it. The 08:10 in
# `rcad-fuzz.timer` is that deadline.
#
# **All five together, each for the whole window.** Until 2026-09-18 they ran in
# turn on a fifth of the window each, which put one core of 24 to work and left
# the other 23 idle for eight hours. Nothing is shared between the five, so the
# only thing sequencing bought was politeness, and `Nice=10` buys that already.
#
# A crash does not end the night. `cargo fuzz` exits non-zero and writes the
# input to `fuzz/artifacts/<target>/` (ignored, so it survives a `git clean`
# without dirtying anything), and a crash in the reader is no reason to cost the
# fuse its hours — so each target is reported and the run carries on, failing at
# the end if anything failed. A crash still ends *that* target's night, which is
# the next thing to fix.
#
# Same nightly-compiler caveat as `fuzz` below.
[doc("Fuzz all five targets at once, inside a window in seconds. What the timer runs.")]
fuzz-overnight window="28800":
    #!/usr/bin/env bash
    set -euo pipefail

    # The same parse `check-hub` uses: git documents the main working tree as
    # the first entry. An eight-hour job in the hub is the one kind of work that
    # cannot be seen — `git worktree lock` refuses there, so `just lanes` would
    # show `free` all night beside a tree somebody is using (docs/decisions.md
    # §60). Give the timer a worktree of its own and lock it.
    listing=$(git worktree list --porcelain)
    hub=$(awk '/^worktree /{print $2; exit}' <<<"$listing")
    if [ "$(grep -c '^worktree ' <<<"$listing")" -gt 1 ] \
        && [ "$(git rev-parse --show-toplevel)" = "$hub" ]; then
        echo "this is the hub, and nobody works in the hub: $hub"
        echo "  an unattended run here can never be recorded as anyone's; point the"
        echo "  timer at a locked worktree instead:"
        echo "      git worktree add ../rcad-fuzz && git worktree lock ../rcad-fuzz --reason 'overnight fuzzing'"
        exit 1
    fi

    deadline=$(( $(date +%s) + {{window}} ))
    night="{{fuzz_log_root}}/$(date +%Y-%m-%d)"
    mkdir -p "$night"

    # Per-run timings, which libFuzzer keeps none of. Its `Slowest unit:` line
    # ratchets and is whole seconds, so a night that ran a thousand slow inputs
    # leaves one behind and says nothing about how the rest were spread. Each
    # target writes a percentile table and its slowest inputs under here, named
    # so they can be handed straight back to `cargo fuzz run`.
    #
    # Exported rather than passed, because it has to reach past `just`, `cargo`
    # and `cargo-fuzz` to the target's own process, and every one of those
    # passes the environment through untouched.
    export RCAD_FUZZ_TIMINGS="$night/timings"

    # The night follows the last commit CI went green on (docs/decisions.md §90).
    #
    # §81.4 refused to advance this tree at all and priced the cost as a timer
    # that can sit on one commit for a month. That cost came due on the third
    # night: the run of 2026-09-11 re-found §85's crash on a base from two days
    # earlier, twelve hours after the fix for it went green. What the refusal was
    # buying is a revert, and `main` has had one in 1003 commits.
    #
    # Green is the part that matters, not newness. CI runs the whole gate on
    # every push and answers in nine to twelve minutes, so "the last green" is a
    # tree something has already read end to end — which is what §81.4 wanted and
    # what an offset like `main~5` cannot give: lanes merge in bursts, so five
    # commits back was under fifteen minutes old in 21 of the last 115 positions.
    #
    # Deciding is separate from doing, in `fuzz-watermark` below, because this
    # recipe's selftest runs it in a real worktree and a checkout here would move
    # whichever tree the gate was run from.
    if [ -n "${RCAD_FUZZ_ALREADY_ADVANCED:-}" ]; then
        echo "standing where the night that started this one put us"
    else
        target=$({{just_executable()}} fuzz-watermark 2>"$night/watermark.log") || true
        cat "$night/watermark.log"
        if [ -n "$target" ]; then
            echo "advancing to $target, the last commit CI went green on"
            git checkout --quiet --detach "$target"

            # **Start again, from the justfile that was just checked out.** `just`
            # hands bash the whole recipe body before the first line of it runs,
            # so without this a change to this recipe takes effect the night
            # *after* it lands. That is not a hypothetical: the night of
            # 2026-09-14 kept none of the six artifacts it found, because the
            # body it was running had been read from a commit of 2026-09-11 and
            # §91's copy-out landed on 2026-09-13.
            #
            # **A night advances once.** The commit just checked out is the
            # newest green there was a moment ago, so the recipe taking over
            # would almost always find nothing to move to and stop of its own
            # accord — but "almost always" is a bound on CI's timing and not on
            # this loop, and a night that spent itself chasing a branch that
            # kept going green would say nothing about why. Once is also the
            # honest reading of §90: the night follows the last green commit as
            # of when it started.
            export RCAD_FUZZ_ALREADY_ADVANCED=1
            exec {{just_executable()}} fuzz-overnight "$(( deadline - $(date +%s) ))"
        fi
    fi

    # How far behind `main` the night is, which is the indicator §81.4 asked for
    # and did not get. It printed the commit's *age*, and a commit can be one day
    # old and sixty-eight commits behind — which is exactly what `latest` read on
    # the night that proved nothing. Age answers "when was this written"; the
    # question is "what has landed since".
    commit=$(git rev-parse --short HEAD)
    age=$(( ( $(date +%s) - $(git log -1 --format=%ct) ) / 86400 ))
    behind=$(git rev-list --count HEAD..origin/main 2>/dev/null || echo '?')
    echo "fuzzing $commit ($age days old, $behind behind main) until $(date -d "@$deadline" '+%H:%M:%S'), logs in $night"

    # Built once, before the window is divided, so that the compiler comes out
    # of the night as a whole and not out of whichever target went first.
    if ! (cd fuzz && RUSTFLAGS="-C target-cpu=x86-64-v3" \
            cargo +nightly fuzz build) >"$night/build.log" 2>&1; then
        tail -n 20 "$night/build.log"
        echo "the fuzz targets did not build, so nothing was fuzzed: $night/build.log"
        # Which advice is the right advice depends on whether there is a `cargo`
        # here at all. On 2026-09-15 there was not, and the night printed the
        # nightly sentence over a `build.log` reading `cargo: command not found`
        # — the one operator-facing line it wrote, naming the one thing that was
        # never the question (§108.2).
        if command -v cargo >/dev/null 2>&1; then
            echo "nightly is not pinned by this repo (docs/decisions.md §29.9); \`rustup toolchain install nightly\` is the usual answer"
        else
            echo "there is no \`cargo\` on this PATH at all, so the toolchain is not the question:"
            echo "  PATH=$PATH"
            echo "a systemd run gets no profile. rcad-fuzz.service sets PATH itself for this reason, and \`just check-fuzz-unit\` says whether the copy systemd runs is the one in the tree"
        fi
        printf '%s  BUILD FAILED  %s\n' "$(date '+%Y-%m-%d %H:%M')" "$night" >"{{fuzz_log_root}}/latest"
        exit 1
    fi

    # The input a target found is kept beside the log that describes it, because
    # `fuzz/artifacts/` is the wrong place to leave the only copy of anything
    # (§91). Two reasons, and each one lost a finding.
    #
    # **libFuzzer never reads that directory back as a seed.** So the next night
    # starts from a corpus without the crash in it, rolls a fresh seed, and does
    # not re-find it: on 2026-09-13 a night reported "nothing found" on the very
    # commit the night before had crashed on, and nothing had been fixed in
    # between.
    #
    # **It lives in this worktree, and `.gitignore` hides it.** §89's slow unit
    # sat there as its only copy for three days in a tree that had been unlocked
    # and was due to be reclaimed.
    #
    # `-n` makes the repeat cheap and keeps the first copy of a name that
    # libFuzzer rewrites.
    copy_out() {
        [ -d fuzz/artifacts ] || return 0
        mkdir -p "$night/artifacts"
        cp -Rn fuzz/artifacts/. "$night/artifacts/" 2>/dev/null || true
    }

    # **On a clock, and not at each target's end, because the ends are all in
    # the same minute now.** The reason §91 wanted this at all is a night killed
    # from outside, and the sequential loop got its protection for free: four
    # target boundaries spread across the night, about ninety minutes apart. Run
    # together, the five targets finish within seconds of each other and there
    # are no boundaries left before the deadline — so a night killed at 04:00
    # would keep nothing, which is worse than what this replaced rather than
    # better. An interval so a test can afford one.
    copy_every="${RCAD_FUZZ_COPY_EVERY:-300}"
    ( while sleep "$copy_every"; do copy_out; done ) &
    copier=$!
    trap 'kill "$copier" 2>/dev/null || true' EXIT

    # **Every target for the whole window, all five at once, rather than a fifth
    # of the window each.** One target at a time was one core of 24 (§81), and
    # the night of 2026-09-18 spent 8 core-hours of the 192 it could have had.
    # Nothing here is shared: each target has its own corpus directory, its own
    # log, and its own `RCAD_FUZZ_TIMINGS/<target>/` — so the five are
    # independent processes that happen to start together, and peak resident was
    # 689 MB apiece against 125 GB.
    #
    # `Nice=10` and `IOSchedulingClass=idle` in `rcad-fuzz.service` are what make
    # this polite rather than rude, and children inherit both: a lane compiling
    # at three in the morning still wins every core it asks for.
    #
    # The five `cargo fuzz run`s serialise for a few seconds at startup on the
    # build directory lock, each doing the no-op build that `cargo fuzz build`
    # above already did. That is the whole cost of not having a flag to skip it.
    recipes=({{fuzz_all}})
    to_look_at=()
    left=$(( deadline - $(date +%s) ))
    pids=()
    started=()

    # The machine, split five ways, because all five run at once. `nproc` and not
    # a number in the file: 24 here, and a recipe carrying `4` would quietly
    # under-use a bigger machine and thrash a smaller one. A floor of 1, since
    # `-fork=1` still isolates the crash even where there is no parallelism left
    # to win — and `-fork=0` would mean "don't fork", which is a different night.
    fork=$(( $(nproc) / ${#recipes[@]} ))
    [ "$fork" -ge 1 ] || fork=1

    # Which artifacts are **this night's**, as opposed to every night's.
    #
    # `fuzz/artifacts/` is gitignored, so it survives across nights in the tree
    # the timer keeps checked out, and `copy_out` below copies all of it. The
    # count in the summary has always been carry-over for that reason. It could
    # stay a wart while the count was only ever *reported*; now that a crash file
    # is what says a night failed (§134), a five-day-old crash would fail every
    # night from here on, which is the same "night nobody believes" §91 exists to
    # prevent, running the other way.
    #
    # `-newer` against a file made now, rather than a listing taken now and
    # diffed: libFuzzer writes an artifact once and never rewrites it, so mtime
    # answers exactly this question and costs one `touch`.
    since="$night/.started"
    : >"$since"
    for recipe in "${recipes[@]}"; do
        # No floor beyond "there is time left". A minimum worth starting would
        # be a number nobody can defend — an 18 MB corpus takes seconds just to
        # load, so the honest floor depends on the target — and it would make
        # the window untestable at any size a test can afford to wait for.
        if [ "$left" -lt 1 ]; then
            echo "$recipe: skipped, $left s left in the window" | tee -a "$night/$recipe.log"
            to_look_at+=("$recipe:no-time")
            continue
        fi
        echo "$recipe: $left s"
        # libFuzzer's own `-timeout` bounds one input and not the process, and a
        # target wedged outside the fuzzer's control would eat the whole night.
        # The grace is for the corpus write on the way out.
        timeout --kill-after=60 "$(( left + 300 ))" \
            {{just_executable()}} "$recipe" "$left" "{{fuzz_corpus_root}}" "$fork" \
            >"$night/$recipe.log" 2>&1 &
        pids+=("$!")
        started+=("$recipe")
    done

    # Waited in the order they were started, which is not the order they finish.
    # It costs nothing: they were all given the same deadline, so the last wait
    # returns within seconds of the first however they are ordered, and reading
    # them in a fixed order keeps the summary's list stable night to night.
    # **Under `-fork` the exit status says nothing about the night**, in either
    # direction, and reading it as "crash" is how a night stops being believed.
    #
    # libFuzzer's fork parent keeps one `ExitCode`, overwrites it with each job it
    # collects, never resets it, and exits with it — `FuzzerFork.cpp:400-461` and
    # the `exit(ExitCode)` at 490, under the comment *"Use the exit code from the
    # last child process"*. Every fate the night is built to survive leaves it
    # non-zero, so **70 is a timeout in the last job of forty thousand** and not a
    # crash; two runs of the same command on the same corpus exited 70 and 0,
    # differing only in which job happened to land last (§133.7). And the other
    # direction is worse: a crash in job 3 of 40 000 exits **0**, so a status
    # mapping that trusted zero would call that night clean.
    #
    # What is left that the status can still answer is the one thing no child
    # decides: `timeout` killing the whole target from outside.
    for i in "${!started[@]}"; do
        recipe="${started[$i]}"
        status=0
        wait "${pids[$i]}" || status=$?
        case "$status" in
            124|137)
                echo "$recipe: overran the window and was killed"
                to_look_at+=("$recipe:overran") ;;
            *)
                # The fuzzer ran, and ran to the end. `INFO: exiting:` is the last
                # line `FuzzWithFork` prints before `exit`, so it is present
                # exactly when the fork loop finished of its own accord — which is
                # the question the exit status used to be asked and can no longer
                # answer. A target whose build or launch failed prints nothing of
                # the sort, and that is a night that fuzzed four targets while
                # reporting five.
                grep -q '^INFO: exiting:' "$night/$recipe.log" \
                    || { echo "$recipe: never got fuzzing (exited $status) — see $night/$recipe.log"
                         to_look_at+=("$recipe:no-run"); } ;;
        esac
        copy_out
    done

    kill "$copier" 2>/dev/null || true
    trap - EXIT
    copy_out

    # Counted after the loop and named in the summary, because a copy nobody is
    # told about is very nearly as lost as the original was. A clean night writes
    # slow units too — the night of 2026-09-13 wrote three and said "nothing
    # found" — so this is reported whether or not anything failed.
    kept=0
    if [ -d "$night/artifacts" ]; then
        kept=$(find "$night/artifacts" -type f | wc -l)
    fi
    artifacts=""
    [ "$kept" -eq 0 ] || artifacts="  $kept artifacts in $night/artifacts"

    # **What failed the night, now that the exit status cannot say.** libFuzzer
    # names every input it writes for what happened to it (`FuzzerLoop.cpp`), and
    # three of those five names are a bug: `crash-`, `oom-`, `leak-`.
    #
    # The other two are a measurement and not a fault. `slow-unit-` means an input
    # was slower than its neighbours — the night of 2026-09-13 wrote three while
    # being an entirely good night. `timeout-` means it was slower than the
    # `-timeout=10` §135 sets, which on `where_two_surfaces_cross` is a nightly
    # certainty rather than news: its tracked corpus holds inputs of 17 to 19
    # seconds, so the replay writes one before any fuzzing starts. A standing
    # population cannot be what fails a night — that is §91's failure reached by
    # crying wolf instead of by staying quiet. Both are still copied out and still
    # counted in the summary, which is how a new one gets noticed.
    #
    # Read out of the worktree rather than out of `$night/artifacts`, and against
    # `$since`, because the copy is made with `cp -Rn` and carries no useful mtime
    # while the original has libFuzzer's own.
    #
    # Guarded rather than left to `find` to report: a night where no target wrote
    # anything has no `fuzz/artifacts` at all, `find` exits 1, and `pipefail`
    # takes the recipe down before the summary is written — the same trap arm Ten
    # of the selftest records falling into.
    found=0
    if [ -d fuzz/artifacts ]; then
        found=$(find fuzz/artifacts -type f -newer "$since" \
            \( -name 'crash-*' -o -name 'oom-*' -o -name 'leak-*' \) \
            | wc -l)
    fi
    if [ "$found" -gt 0 ]; then
        echo "$found crash-class artifacts written tonight — see $night/artifacts"
        to_look_at+=("crashed:$found")
    fi

    if [ "${#to_look_at[@]}" -eq 0 ]; then
        printf '%s  %s (%d days old, %s behind main)  %d targets, nothing found%s  %s\n' \
            "$(date '+%Y-%m-%d %H:%M')" "$commit" "$age" "$behind" "${#recipes[@]}" \
            "$artifacts" "$night" >"{{fuzz_log_root}}/latest"
        cat "{{fuzz_log_root}}/latest"
        exit 0
    fi
    printf '%s  %s (%d days old, %s behind main)  %d targets, %d TO LOOK AT (%s)%s  %s\n' \
        "$(date '+%Y-%m-%d %H:%M')" "$commit" "$age" "$behind" "${#recipes[@]}" \
        "${#to_look_at[@]}" "${to_look_at[*]}" "$artifacts" "$night" >"{{fuzz_log_root}}/latest"
    cat "{{fuzz_log_root}}/latest"
    exit 1

# Which commit the night should fuzz: the newest on `main` that CI went green on.
#
# Prints that sha, or prints nothing and says on stderr why it is staying put.
# **It never exits non-zero.** A night that cannot work out where to stand is a
# night that runs where it already is; failing here would trade a stale run for
# no run at all, which is the worse of the two.
#
# Three conditions, each of them a way the night could otherwise go wrong:
#
#   - **green**, so it never fuzzes a tree nothing has read end to end. This is
#     §81.4's objection kept rather than overturned — what changed is that a
#     green gate answers it, where in §81.4 nothing did. **Green now means a
#     green CI run or a recorded local `just gate`**, because since §172 the
#     first of those can no longer happen and asking only GitHub froze the night
#     on a commit that kept getting older. §173.9.
#   - **on `main`**, so a green run for some other branch cannot move it.
#   - **forward only**, so a tree somebody advanced by hand is never dragged
#     back, and neither is one already past the last green.
#
# Separate from `fuzz-overnight` because that recipe's selftest runs it for real
# in whatever worktree the gate was run from, and a checkout in there would move
# somebody's tree out from under them.
[doc("Which commit the overnight run should fuzz: the newest green commit on main, CI's or a local gate's.")]
fuzz-watermark:
    #!/usr/bin/env bash
    set -euo pipefail

    staying() { echo "staying on $(git rev-parse --short HEAD): $1" >&2; exit 0; }

    git diff --quiet && git diff --cached --quiet \
        || staying "the tree has uncommitted changes"
    # No longer fatal: a missing `gh` costs the CI half of the list, and since
    # §173.9 that is not the only half there is.
    command -v gh >/dev/null 2>&1 \
        || echo "gh is not on PATH, so only the local gate record can answer" >&2

    # Two sources, because since §172 one of them can no longer answer: the
    # Actions allowance is spent, so no push to `main` can go green and asking
    # only GitHub freezes the night on whatever was last green before the
    # billing ran out — 18 commits back inside a day, and silently, since
    # staying put is how this recipe is built to fail. A local `just gate` reads
    # the tree end to end, which is the thing §81.4 wanted; `_record-green`
    # writes each one down. §173.9.
    #
    # A `gh` that cannot answer is not a failure here and never was — it leaves
    # the list shorter, and the local half can still carry the night.
    green=$(gh run list --workflow ci.yml --branch main --status success \
        --limit 20 --json headSha --jq '.[].headSha' 2>/dev/null) || green=""
    recorded=$(cut -d' ' -f1 \
        "$(git rev-parse --path-format=absolute --git-common-dir)/rcad-greens" 2>/dev/null) || recorded=""
    # `|| green=""` and not a bare pipeline: `grep -v` exits 1 when it filters
    # every line away, which under `pipefail` is both sources being silent
    # reported as a failed recipe — the one thing this must never do, because a
    # night that cannot decide still has to run. Arm five caught it.
    green=$(printf '%s\n%s\n' "$green" "$recorded" | grep -v '^$' | sort -u) || green=""
    [ -n "$green" ] || staying "neither gh nor the local gate record names a green commit"

    # Fetched only once there is something to weigh against it, so that a gate
    # running this recipe does not need the network to pass — §77.6's shape, a
    # check that would be answering a question about its machine.
    git fetch --quiet origin 2>/dev/null \
        || staying "origin could not be reached"

    # The furthest forward of the candidates, not the first one seen. Two
    # sources merged and de-duplicated arrive in no useful order, where `gh`
    # alone arrived newest-first, so the walk has to establish that itself —
    # `--is-ancestor` between two candidates is which of them is further along.
    best=""
    on_head=""
    for sha in $green; do
        git cat-file -e "${sha}^{commit}" 2>/dev/null || continue
        git merge-base --is-ancestor "$sha" origin/main || continue
        git merge-base --is-ancestor HEAD "$sha" || continue
        if [ "$(git rev-parse "$sha")" = "$(git rev-parse HEAD)" ]; then
            on_head=yes
            continue
        fi
        if [ -z "$best" ] || git merge-base --is-ancestor "$best" "$sha"; then
            best=$sha
        fi
    done
    if [ -n "$best" ]; then
        echo "$best"
        exit 0
    fi
    [ -z "$on_head" ] || staying "already on the last green"
    staying "no green commit on main is ahead of it"

# The property fuzzer, over a pair of surfaces (docs/decisions.md §24.14).
#
# NOT part of `just gate`, and not because anybody chose to leave it out:
# `cargo-fuzz` builds with a sanitizer that needs a nightly compiler and
# `rust-toolchain.toml` pins a stable one, so this is the one recipe here that
# uses a compiler the repo does not pin. What runs on every commit is the same
# property over the same decoder, in
# `crates/rcad-geom/tests/every_pair_of_surfaces_the_fuzzer_draws_is_named_honestly.rs`.
[doc("Fuzz the predicates over a pair of surfaces. Needs a nightly compiler.")]
fuzz duration="60" corpus_root="" fork="0": (fuzz-target "where_two_surfaces_cross" duration corpus_root "71" fork)

# The Part 21 reader, over arbitrary bytes (docs/decisions.md §42.12 criterion 5).
#
# Same nightly-compiler caveat as `fuzz` above. What runs on every commit
# instead is `crates/rcad-step/tests/what_the_reader_refuses_and_by_what_name.rs`,
# which covers the shapes somebody thought of; this covers the ones nobody did.
#
# No `-max_len`, unlike `fuzz`: the reader's promise is about any length of
# anything, and the truncation cases live at every length there is.
[doc("Fuzz the Part 21 reader over arbitrary bytes.")]
fuzz-reader duration="60" corpus_root="" fork="0": (fuzz-target "no_step_file_makes_the_reader_panic" duration corpus_root "" fork)

# The Part 21 writer, over an arbitrary `Model` (docs/decisions.md §42.12).
#
# Same nightly-compiler caveat as `fuzz` above. `Model` is `Deserialize`
# (`crates/rcad-topo/src/serialize.rs`), so this target draws bytes through
# `postcard::from_bytes` — the same route a stranger's saved file takes — and
# whatever `Model` comes out, sound or not, goes straight into
# `rcad_step::write`, which owes an `Err` and never a panic either way.
[doc("Fuzz the Part 21 writer over an arbitrary Model.")]
fuzz-writer duration="60" corpus_root="" fork="0": (fuzz-target "no_model_makes_the_writer_panic" duration corpus_root "" fork)

# The fuse, over two arbitrary `Model`s (docs/decisions.md §17).
#
# Same nightly-compiler caveat as `fuzz` above. Decodes two `Model`s per run
# and, for each that passes `Model::validate`, fuses a byte-chosen split of
# its own solids against each other — see
# `fuzz/fuzz_targets/two_solids_fused_by_a_stranger.rs` for why two `Model`s
# and not one cross-fused pair.
[doc("Fuzz the fuse over two arbitrary Models.")]
fuzz-fuse duration="60" corpus_root="" fork="0": (fuzz-target "two_solids_fused_by_a_stranger" duration corpus_root "" fork)

# A cone or a torus crossing another surface, over its own byte decoder.
#
# Same nightly-compiler caveat as `fuzz` above. Needs its own `-max_len`,
# like `fuzz` above and unlike the reader/writer/fuse targets: a torus-vs-torus
# pair is this decoder's worst case, at 1 byte of choice plus two 8-byte
# doubles for each of a torus's five numbers, so 81 and not `fuzz`'s 71.
[doc("Fuzz the predicates over a cone or a torus and another surface.")]
fuzz-cones-and-tori duration="60" corpus_root="" fork="0": (fuzz-target "where_a_cone_or_a_torus_crosses" duration corpus_root "81" fork)

# The one place that knows how to start a fuzz target. The five recipes above
# are its five call sites and differ only in the name and `-max_len`.
#
# RUSTFLAGS has to be repeated: cargo-fuzz sets it as an environment variable,
# and that shadows the `.cargo/config.toml` entry `inari` needs to build at all.
#
# With no `corpus_root` — a hand-run `just fuzz 60` — libFuzzer gets no corpus
# argument and writes to the tracked `fuzz/corpus/<target>` as it always has.
# With one, that directory becomes the *second* argument and libFuzzer treats it
# as a read-only seed, writing only to the first (§81).
#
# `fork` is how many jobs libFuzzer runs at once, and **0 means don't fork at
# all**, which is what a hand-run `just fuzz 60` wants: one crash, one report, and
# the run stops so you can read it. Only the night asks for a number, because only
# the night would rather lose the report than lose the other eight hours (§134).
[private]
fuzz-target target duration corpus_root="" max_len="" fork="0":
    #!/usr/bin/env bash
    set -euo pipefail
    cd fuzz
    corpus=()
    if [ -n "{{corpus_root}}" ]; then
        mkdir -p "{{corpus_root}}/{{target}}"
        corpus=("{{corpus_root}}/{{target}}" "corpus/{{target}}")
    fi

    # `-ignore_crashes=1` is the flag that buys the night: without it the first
    # crash calls `StopJobs()` and the target is done for the evening, however
    # many hours were left (`FuzzerFork.cpp:441-450`).
    #
    # `-ignore_timeouts=1` and `-ignore_ooms=1` are already libFuzzer's defaults
    # (`FuzzerFlags.def:66-67`) and are passed anyway, because the night now
    # *depends* on them rather than merely tolerating them: with a per-input
    # `-timeout` the slow units of §91.3 time out on the corpus replay, and a
    # default that changed under us would end every target in its first minute.
    #
    # **`-timeout=10` is in here and not outside it, because it is only safe
    # inside it** (§133.7, §135). `-ignore_timeouts` is a fork-mode flag; without
    # `-fork` a timeout kills the process, and `fuzz/corpus/where_two_surfaces_cross`
    # holds inputs of 17 to 19 seconds — so a hand-run `just fuzz 60` carrying this
    # flag would die on its own corpus replay before fuzzing anything.
    #
    # Ten seconds is 1 270 times that target's p99 of 7.9 ms and caps a tail that
    # took 19% of its night in 0.025% of its inputs. Deliberately loose rather
    # than tight: an input killed here never enters the corpus, and on this target
    # the slow inputs are the nearly-parallel crossings §0's unit is about.
    fork_flags=()
    if [ "{{fork}}" -gt 0 ]; then
        fork_flags=(-fork={{fork}} -timeout=10 -ignore_crashes=1 -ignore_timeouts=1 -ignore_ooms=1)
    fi
    RUSTFLAGS="-C target-cpu=x86-64-v3" \
        cargo +nightly fuzz run {{target}} ${corpus[@]+"${corpus[@]}"} -- \
        -max_total_time={{duration}} {{ if max_len == "" { "" } else { "-max_len=" + max_len } }} \
        ${fork_flags[@]+"${fork_flags[@]}"}

# Pins `fuzz-overnight` against a `cargo` it brings with it, because the real one
# wants a sanitizer build and eight hours before it says anything at all. The
# stub sleeps half the time it was granted and logs when it started, so both the
# clock the recipe runs against and the order it starts things in are real, and
# the whole thing still runs in about thirty seconds.
#
# Two things about the window, and neither can be seen from the clock alone.
# **It is a deadline and not a budget**: a recipe that gave each of five targets
# a fresh window would run five times as long as it was told to, which nobody
# would notice until the fuzzer was still going at lunch. And **every target
# gets the whole of it**, at once — which is the same sentence read the other
# way, and is why the pair has to be asserted on what each target was *told*
# (arm one) and on when each one *started* (arm four) rather than on how long
# the night took. Running in turn on a fifth each and running together on the
# whole each take about the same wall-clock time; that is exactly how the first
# of them went unquestioned for nine nights while 23 of 24 cores sat idle.
#
# The rest are each a way the night could quietly do nothing: a crash in one
# target taking the other four down with it, one target's overrun starving them,
# the run reporting success after a target died, a finding left in a gitignored
# directory in a tree due to be reclaimed (§91), and — the reason the hub case is
# here rather than trusted to `check-hub` — a timer left pointing at the one tree
# whose owner cannot be written down, where `just lanes` would show `free` all
# night (§60).
[private]
check-fuzz-overnight-selftest:
    #!/usr/bin/env bash
    set -euo pipefail
    dir=$(mktemp -d)

    # The stub writes its artifacts into the real `fuzz/artifacts/`, because that
    # is the one path the recipe under test is not free to choose — `cargo-fuzz`
    # fixes `-artifact_prefix` and nothing in the environment moves it. So the tag
    # is unique per run, two lanes can gate at once, and the trap takes away
    # exactly what this run put there and nothing a real night left.
    tag="overnight-selftest-$$-${RANDOM}"
    trap 'rm -rf "$dir"; rm -f fuzz/artifacts/*/crash-"$tag" fuzz/artifacts/*/slow-unit-"$tag"' EXIT
    mkdir -p "$dir/bin"

    # Logs its whole command line, then sleeps **half** the slice it was given.
    #
    # Half, and not all of it, is what keeps this arm from being a race. Sleeping
    # the whole slice makes the five sleeps add up to the window by
    # construction, which leaves nothing for the arm's own overhead — `timeout`
    # around a nested `just`, five times over, about 1.2 s under load. `date +%s`
    # truncates to whole seconds and hides that until it crosses a second
    # boundary; then `(deadline - now) / left` truncates to 0 for the last
    # target, it is skipped, and the night correctly exits 1 for a reason the
    # fixture never meant to create. Four passes and a fail in five runs, on one
    # tree, found by rcad-15 on 2026-09-09.
    #
    # It still has to scale with the slice rather than being a fixed nap, or the
    # mutation this arm exists to catch — handing every target the whole window —
    # would sleep the same amount as the correct recipe and go unnoticed.
    cat >"$dir/bin/cargo" <<'STUB'
    #!/usr/bin/env bash
    set -euo pipefail
    # `at=` is when this target started, and it is the only thing in the log that
    # can say whether the five ran together. Elapsed time cannot: the night is as
    # long as its slowest target however the rest are arranged, so a recipe that
    # started one target, waited for it, and then ran the other four at once
    # would spend about what a correct one spends.
    printf '%s [timings=%s] [at=%s]\n' "$*" "${RCAD_FUZZ_TIMINGS:-unset}" "$(date +%s)" >>"$STUB_LOG"
    case " $* " in
        *" build "*) exit 0 ;;
    esac

    # Writes what libFuzzer would write, where libFuzzer would write it: the real
    # `cargo fuzz run` is handed `-artifact_prefix=<worktree>/fuzz/artifacts/<target>/`
    # and this stub's cwd is that `fuzz/` directory, because `fuzz-target` cd's
    # into it. Named with a tag the arm picks, so the assertion is about this run's
    # own file and not about whatever a real session left lying there.
    target=""
    prev=""
    for arg in "$@"; do
        if [ "$prev" = "run" ]; then
            target="$arg"
            break
        fi
        prev="$arg"
    done
    #
    # **`slow-unit-` by default, and `crash-` only when an arm asks.** A clean
    # night writes slow units — the night of 2026-09-13 wrote three and correctly
    # said "nothing found" — so that is what an unremarkable target leaves behind.
    # Since §134 a crash file is what fails a night, and a stub that wrote one
    # every time would leave no fixture able to express a good night at all.
    kind="${STUB_ARTIFACT_KIND:-slow-unit-}"
    if [ -n "${STUB_ARTIFACT_KIND_TARGET:-}" ] && [ "$target" != "$STUB_ARTIFACT_KIND_TARGET" ]; then
        kind="slow-unit-"
    fi
    if [ -n "$target" ] && [ -n "${STUB_ARTIFACT_TAG:-}" ]; then
        mkdir -p "artifacts/$target"
        echo "what a target found" >"artifacts/$target/$kind$STUB_ARTIFACT_TAG"
    fi

    # **A barrier, and not a stopwatch** (§139). Armed only by `STUB_RENDEZVOUS`,
    # because this stub is shared by every arm in the recipe, and an arm that
    # launches between one and four targets would wait out the deadline for
    # targets that were never coming. (`run 0` is not that case: a recipe with
    # under a second left is skipped before it is launched, so it never gets
    # here — §139.7.)
    #
    # Each target appends its name and then waits for the file to hold five
    # lines. Run together, all five arrive and every one of them sees five. Run
    # in turn, the first blocks here and the other four are never launched, so
    # the file ends at one line and no target sees five. That holds at every
    # timeout *above the instant the five have arrived* — the deadline has a
    # floor and no ceiling, where a bound on the launch spread had both and they
    # met. Loosening this costs the check nothing; loosening that cost it
    # everything.
    #
    # It sits after the `at=` printf above and before the sleep below. Before
    # the printf, the log would stop recording when a target launched and the
    # paired `max_total_time`/`at=` check would lose its input; after the sleep,
    # the five would have finished waiting for each other before they met.
    if [ -n "${STUB_RENDEZVOUS:-}" ] && [ -n "$target" ]; then
        echo "$target" >>"$STUB_RENDEZVOUS"
        want="${STUB_RENDEZVOUS_COUNT:-5}"
        # The deadline is a deadlock escape and not a tuning knob: without it a
        # recipe that launches one target hangs `just gate` instead of failing
        # it. A correct recipe never reaches it, provided it is set above the
        # time five targets take to arrive — at 0 even a correct recipe times
        # out, which is the floor §139.7 records.
        deadline=$(( $(date +%s) + ${STUB_RENDEZVOUS_TIMEOUT:-15} ))
        while [ "$(grep -c '' "$STUB_RENDEZVOUS")" -lt "$want" ] \
            && [ "$(date +%s)" -lt "$deadline" ]; do
            sleep 0.1
        done
        printf 'rendezvous %s saw %s of %s\n' \
            "$target" "$(grep -c '' "$STUB_RENDEZVOUS")" "$want" >>"$STUB_LOG"
    fi

    # `STUB_OVERRUN_TARGET` aims the overrun at one target and leaves the other
    # four alone, which is the only way to ask whether an overrun costs them
    # anything. While the five ran in turn it could not be asked: an overrun in
    # the first one spent the window, and that was the whole of the old arm.
    overrun=0
    if [ -n "${STUB_OVERRUN:-}" ] \
        && { [ -z "${STUB_OVERRUN_TARGET:-}" ] || [ "$target" = "$STUB_OVERRUN_TARGET" ]; }; then
        overrun="$STUB_OVERRUN"
    fi
    for arg in "$@"; do
        case "$arg" in
            -max_total_time=*)
                half=$(( ${arg#*=} / 2 ))
                [ "$half" -ge 1 ] || half=1
                sleep "$(( half + overrun ))"
                ;;
        esac
    done
    # The line libFuzzer's fork parent prints last, which since §134 is what says
    # the target got as far as fuzzing. `STUB_NO_EXITING_TARGET` withholds it from
    # one target, which is how an arm asks for a target that never got going —
    # the failure the exit status used to carry and can no longer.
    if [ "$target" != "${STUB_NO_EXITING_TARGET:-}" ]; then
        echo "INFO: exiting: 0 time: 1s"
    fi

    # Exits non-zero while still having fuzzed the whole window, which is fork
    # mode's ordinary ending and not a failure: the code is whichever of forty
    # thousand jobs happened to be collected last (§133.7).
    for arg in "$@"; do
        if [ "$arg" = "${STUB_CRASH:-}" ]; then exit "${STUB_CRASH_CODE:-70}"; fi
    done
    exit 0
    STUB
    chmod +x "$dir/bin/cargo"

    # The night asks `fuzz-watermark` where to stand, and this arm runs it for
    # real in whatever worktree the gate was run from. Answering nothing unless
    # an arm asks for an answer keeps that hermetic — no network, no account —
    # and the HEAD assertion at the end is what says a gate cannot move
    # somebody's tree. The arm that does want the night to move sets `GH_GREEN`,
    # and does it in a fixture repository of its own.
    cat >"$dir/bin/gh" <<'STUB'
    #!/usr/bin/env bash
    [ -n "${GH_GREEN:-}" ] || exit 1
    cat "$GH_GREEN"
    STUB
    chmod +x "$dir/bin/gh"
    head_before=$(git rev-parse HEAD)

    fail() {
        echo "fuzz-overnight drifted from its own fixtures: $1"
        [ -f "$dir/log" ] && sed 's/^/    /' "$dir/log"
        exit 1
    }

    run() {
        STUB_LOG="$dir/log" PATH="$dir/bin:$PATH" STUB_ARTIFACT_TAG="$tag" \
            RCAD_FUZZ_LOGS="$dir/logs" RCAD_FUZZ_CORPUS="$dir/corpus" \
            {{just_executable()}} fuzz-overnight "$1" >"$dir/out" 2>&1
    }

    # One: five targets inside a twelve-second window, with the second of them
    # never getting as far as fuzzing. Every one still runs, every one is given
    # the whole window, the run ends inside it rather than five times past it, and
    # it reports the failure rather than swallowing it.
    #
    # **The second target fails by printing no `INFO: exiting:`, not by exiting
    # non-zero**, and that is §134: under `-fork` a non-zero status is the last of
    # forty thousand jobs and says nothing about the night, so the fixture that
    # used to express "this target died" by `exit 1` now expresses a target that
    # exited 70 and was fine. Arm Eleven is that case, and this one is the failure
    # the status used to carry — a target whose build or launch never happened.
    #
    # **The clock alone cannot say whether they ran together**, and that is why
    # the `-max_total_time` assertions below are here rather than a tighter
    # bound. A correct recipe spends about 6 s, running five targets that each
    # sleep half of twelve; the recipe this replaced spent about 9 s, running
    # five targets that each slept half of a fifth. Both sit inside any bound
    # wide enough to survive a loaded machine. What separates them exactly is
    # what each target was *told*: one window, five times over, against five
    # shares that shrink. 16 still catches the mutation the clock can see —
    # handing every target the whole window and then running them in turn.
    : >"$dir/log"
    started=$(date +%s)
    status=0
    STUB_NO_EXITING_TARGET=no_step_file_makes_the_reader_panic run 12 || status=$?
    elapsed=$(( $(date +%s) - started ))
    [ "$status" -ne 0 ] || fail "a target never got fuzzing and the night reported success"
    ran=$(grep -c 'fuzz run' "$dir/log" || true)
    [ "$ran" -eq 5 ] || fail "a failure in the second target left $ran of 5 targets run"
    [ "$elapsed" -ge 4 ] || fail "a twelve-second window was over in $elapsed s, so nothing was fuzzed"
    [ "$elapsed" -le 16 ] || fail "a twelve-second window took $elapsed s, so the window is a budget and not a deadline"
    grep -q 'TO LOOK AT.*fuzz-reader' "$dir/logs/latest" \
        || fail "the summary line does not name the target that died"

    # Every target got the whole of what was left when *it* launched, and not a
    # fifth of it. This is the assertion that holds §129's change — five targets
    # at once rather than a fifth of the window each.
    #
    # **The yardstick is what was left, not the twelve the window was asked for.**
    # The night spends its preamble — the watermark, git, a nested `just` per
    # target — inside the window, and hands each target `deadline - now`, so what
    # arrives is twelve minus the preamble. Here the preamble rounds to nothing
    # and the grant is 12. On a GitHub runner it was **9 s of the 12**, so the
    # correct recipe granted 3 and a `-ge 8` assertion called it a share: run
    # 35399208851, red on a commit that changed one line of `docs/decisions.md`.
    # Worse than red — at 3 s a fifth of twelve is 2, so the arm had almost no
    # power left to tell the correct recipe from the mutation it exists to catch.
    #
    # **Asked per target, against that target's own launch instant, because no
    # bound on the five values together is sound.** Requiring one value forbids a
    # correct recipe: one computing `deadline - now` inside the launch loop hands
    # a target that started two seconds later two seconds less, which is right,
    # and a set-wide check calls it a divided window. Allowing the values to
    # differ is what lets a genuinely divided window through. Pairing each grant
    # with its own `at=` escapes the choice, and needs no new instrumentation —
    # the stub already logs its whole argv and its stamp on one line, so it is one
    # `sed` with two groups where there were two greps with one each.
    #
    # `started` is read before `fuzz-overnight` begins, so `started + 12` is at or
    # before the deadline the recipe sets, and each `whole` is a *lower bound* on
    # what a correct recipe grants that target. This cannot fail a merely slow
    # machine, only a recipe that shares the window out. A preamble long enough to
    # leave nothing to launch with is caught by the `ran -eq 5` check above.
    pairs=$(sed -n 's/.*max_total_time=\([0-9]*\).*\[at=\([0-9]*\)\].*/\1 \2/p' "$dir/log")
    [ "$(grep -c . <<<"$pairs")" -eq 5 ] \
        || fail "$(grep -c . <<<"$pairs") of 5 targets logged both a window and a start time, so what they were given cannot be judged"
    while read -r granted at; do
        whole=$(( started + 12 - at ))
        [ "$granted" -ge "$whole" ] \
            || fail "a target was given $granted s of the $whole s still left when it launched, so the window is still being shared"
    done <<<"$pairs"

    # Every target is told where to write its per-run timings, and the place is
    # this night's own directory. A target that was not told keeps nothing, and
    # the morning has libFuzzer's one ratcheted `Slowest unit:` line and no
    # spread — which is the state this replaced.
    timed=$(grep -c "fuzz run.*\[timings=$dir/logs/$(date +%Y-%m-%d)/timings\]" "$dir/log" || true)
    [ "$timed" -eq 5 ] \
        || fail "$timed of 5 targets were told where to write their timings"

    # What each target found is carried out of the worktree and into the night's
    # own directory (§91). `fuzz/artifacts/` is gitignored, lives in a tree that
    # gets unlocked and reclaimed, and is never read back as a seed — so an input
    # left only there is an input the next night cannot re-find and a sweep can
    # delete. A night on 2026-09-13 reported "nothing found" over a crash sitting
    # in it.
    night="$dir/logs/$(date +%Y-%m-%d)"
    for target in where_two_surfaces_cross no_step_file_makes_the_reader_panic \
            no_model_makes_the_writer_panic two_solids_fused_by_a_stranger \
            where_a_cone_or_a_torus_crosses; do
        [ -f "$night/artifacts/$target/slow-unit-$tag" ] \
            || fail "$target's artifact was left in the worktree instead of kept in $night/artifacts"
    done

    # Every target forks, and every one is told to survive a crash. Without
    # `-ignore_crashes=1` the first crash calls `StopJobs()` and that target's
    # night ends there, which is the whole of what §134 bought; without `-fork`
    # the flag is not even read. The number is the machine split five ways and so
    # cannot be asserted exactly, but it can be asserted to be one number, shared,
    # and at least 1 — a recipe handing out 0 would silently stop forking.
    forked=$(grep -o 'fork=[0-9]*' "$dir/log" | cut -d= -f2 | sort -un)
    [ "$(wc -l <<<"$forked")" -eq 1 ] \
        || fail "the five targets were given different fork counts: $(tr '\n' ' ' <<<"$forked")"
    [ "$forked" -ge 1 ] \
        || fail "the night asked for -fork=$forked, so nothing is isolated and one crash still ends a target"
    [ "$(grep -c 'ignore_crashes=1' "$dir/log")" -eq 5 ] \
        || fail "a target forks without -ignore_crashes=1, so its first crash still ends its night"

    # And every target bounds one input (§135). Without it `where_two_surfaces_cross`
    # spends 19% of its night on 0.025% of its inputs, at 17 to 19 seconds apiece.
    [ "$(grep -c 'timeout=10' "$dir/log")" -eq 5 ] \
        || fail "a target runs with no per-input timeout, so one input can still take twenty seconds of its night"

    # And the morning is told, because a copy nobody is told about is very nearly
    # as lost as the original. This arm's night also failed, so it proves the
    # count rides along with `TO LOOK AT` rather than replacing it.
    #
    # **At least five, and not exactly five.** The stub writes into the real
    # `fuzz/artifacts/`, so a tree where somebody has actually fuzzed has more in
    # it than this arm put there. Pinning the number would make the arm pass or
    # fail on a property of the machine rather than of the recipe, which is §77.6's
    # mistake made a third time.
    kept=$(sed -n 's/.*  \([0-9]*\) artifacts in .*/\1/p' "$dir/logs/latest")
    [ -n "$kept" ] \
        || fail "the summary line does not say how many artifacts were kept: $(cat "$dir/logs/latest")"
    [ "$kept" -ge 5 ] \
        || fail "five targets each left an artifact and the summary counted $kept"

    # Two: the tracked corpus is a read-only seed and never the directory
    # libFuzzer writes to, which is the whole of §81. Order is the only thing
    # that says so, and only `fuzz` carries `-max_len`.
    grep -q "run where_two_surfaces_cross $dir/corpus/where_two_surfaces_cross corpus/where_two_surfaces_cross " "$dir/log" \
        || fail "the night's own corpus is not first, so libFuzzer would write into the tracked one"
    [ "$(grep -c 'max_len=71' "$dir/log")" -eq 1 ] \
        || fail "-max_len=71 belongs to where_two_surfaces_cross and to nothing else"

    # Three: a clean night says so and exits 0. Without this, a recipe that
    # always failed would satisfy every assertion above. No clock assertions
    # here, so the window only has to be wide enough that every target gets a
    # slice worth having.
    : >"$dir/log"
    run 8 || fail "a night that found nothing exited non-zero"
    grep -q 'nothing found' "$dir/logs/latest" \
        || fail "a clean night did not say so in the summary line"
    grep -q "$(git rev-parse --short HEAD) ([0-9]* days old, [0-9?]* behind main)" "$dir/logs/latest" \
        || fail "the summary line does not say which commit was fuzzed and how far behind main it is, so a timer left on a stale one reads as a good night"

    # The tree the gate ran in is where it was. `fuzz-watermark` is the only
    # thing here that can move a worktree, and a night that cannot reach `gh`
    # must stay put rather than guess — otherwise `just gate` would check out
    # somebody else's commit under them.
    [ "$(git rev-parse HEAD)" = "$head_before" ] \
        || fail "the night moved the tree it ran in, from $head_before to $(git rev-parse HEAD)"

    # Four: an overrun in one target costs the other four nothing.
    #
    # This is the arm the change of 2026-09-18 turned round, and it is worth
    # saying what it used to assert: that an overrun in the first target spent
    # the window and the remaining four were skipped and said to be skipped.
    # That was correct of a recipe that ran them in turn, and it was the single
    # worst property of that recipe — one slow target could take four others'
    # hours with it. Run together they cannot, so the same fixture now asks the
    # opposite question and the old assertion is what would fail.
    #
    # Six and eight, so the overrunner is eleven seconds into a six-second
    # window when it returns while the other four have been done since three.
    # The skip path it used to reach on the way is arm Nine's now.
    : >"$dir/log"
    : >"$dir/rendezvous"
    started=$(date +%s)
    status=0
    STUB_OVERRUN=8 STUB_OVERRUN_TARGET=where_two_surfaces_cross \
        STUB_RENDEZVOUS="$dir/rendezvous" run 6 || status=$?
    elapsed=$(( $(date +%s) - started ))
    [ "$status" -eq 0 ] \
        || fail "one target overran and the night reported $status for the other four"
    [ "$(grep -c 'fuzz run' "$dir/log")" -eq 5 ] \
        || fail "one target overran and took the rest of the window with it"
    # And all five were alive at one instant, which is the assertion the clock
    # cannot make. A night is as long as its slowest target however the rest are
    # arranged, so total elapsed time is satisfied by a recipe that runs one
    # target, waits for it, and only then starts the other four. What is not
    # satisfied by that recipe is five targets meeting at a barrier.
    #
    # **This replaced a bound on the launch spread, and the reason is §139.**
    # `spread <= 3` had to sit between five `just` forks below and a sequential
    # recipe's spread above, so a loaded runner failed it and raising the number
    # deleted the check rather than weakening it. Worse, its margin came entirely
    # from the overrun target leading `fuzz_all`: point `STUB_OVERRUN_TARGET`
    # anywhere else and the adversary spread exactly 3, so the arm kept passing
    # and stopped discriminating. A barrier does not care which target leads, and
    # asserts the stronger property the comment above already claimed.
    #
    # **Before the clock below, and not after**, because a recipe that fails this
    # one fails it by waiting out the barrier's deadline — which is itself time on
    # the clock. Checked the other way round, every sequential recipe reports the
    # elapsed bound, and the symptom buries the cause.
    [ "$(grep -c '^rendezvous .* saw 5 of 5$' "$dir/log")" -eq 5 ] \
        || fail "the five targets never met at one instant, so they are still waiting for each other: $(grep '^rendezvous ' "$dir/log" | tr '\n' ' ')"

    [ "$elapsed" -le 20 ] || fail "one eight-second overrun cost the night $elapsed s"

    # Five: the hub is refused before anything is built, so a timer pointed at
    # it fails on the first night rather than on the night it matters.
    hub="$dir/hub"
    git init -q -b main "$hub"
    cp justfile "$hub/justfile"
    git -C "$hub" add justfile
    git -C "$hub" -c user.email=t@t -c user.name=t commit -q -m "root"
    git -C "$hub" worktree add -q --detach "$dir/lane"
    : >"$dir/log"
    status=0
    (cd "$hub" && STUB_LOG="$dir/log" PATH="$dir/bin:$PATH" \
        RCAD_FUZZ_LOGS="$dir/logs" RCAD_FUZZ_CORPUS="$dir/corpus" \
        {{just_executable()}} fuzz-overnight 4) >"$dir/out" 2>&1 || status=$?
    [ "$status" -ne 0 ] || fail "the hub was handed an eight-hour job and took it"
    grep -q 'nobody works in the hub' "$dir/out" \
        || fail "the hub was refused without saying why: $(cat "$dir/out")"
    [ ! -s "$dir/log" ] \
        || fail "the hub was refused only after cargo had been run"

    # Six: a night runs the justfile it checked out, and not the one it woke up
    # on. `just` reads the whole recipe body before the first line of it runs, so
    # a recipe that advanced the tree and carried on would be fuzzing one commit
    # with another commit's instructions — and every fix to this file would take
    # effect the night after it landed. That is what happened on 2026-09-14: six
    # artifacts kept nowhere, by a body read from a commit two days older than
    # §91's copy-out.
    #
    # The fixture says which body ran by making the two commits disagree: the
    # night starts on the real recipe and the commit it advances to carries a
    # `fuzz-overnight` that does nothing but name itself.
    night_repo="$dir/night"
    git init -q -b main "$night_repo"
    mkdir -p "$night_repo/fuzz"
    touch "$night_repo/fuzz/.keep"
    cp justfile "$night_repo/justfile"
    git -C "$night_repo" add justfile fuzz/.keep
    git -C "$night_repo" -c user.email=t@t -c user.name=t commit -q -m "the real recipe"
    woke_on=$(git -C "$night_repo" rev-parse HEAD)
    cat >"$night_repo/justfile" <<'STUB'
    fuzz-overnight window="28800":
        @echo "the checked-out justfile ran, with {{{{ window }} s left"
    STUB
    git -C "$night_repo" add justfile
    git -C "$night_repo" -c user.email=t@t -c user.name=t commit -q -m "a recipe that names itself"
    green=$(git -C "$night_repo" rev-parse HEAD)
    git init -q --bare "$dir/night-origin.git"
    git -C "$night_repo" remote add origin "$dir/night-origin.git"
    git -C "$night_repo" push -q origin main
    git -C "$night_repo" checkout -q --detach "$woke_on"
    echo "$green" >"$dir/night-green"

    # The status is read last, and not with `||`. A night that carried on with
    # the wrong body goes on to build and fuzz, and then fails or passes for
    # reasons of its own — so asserting on it first answers "did the fixture's
    # stubs behave" where the arm means to ask "whose recipe ran".
    : >"$dir/log"
    status=0
    (cd "$night_repo" && STUB_LOG="$dir/log" PATH="$dir/bin:$PATH" \
        GH_GREEN="$dir/night-green" \
        RCAD_FUZZ_LOGS="$dir/logs" RCAD_FUZZ_CORPUS="$dir/corpus" \
        {{just_executable()}} fuzz-overnight 20) >"$dir/out" 2>&1 || status=$?
    grep -q "advancing to $green" "$dir/out" \
        || fail "the night did not advance onto the green commit: $(cat "$dir/out")"
    grep -q 'the checked-out justfile ran' "$dir/out" \
        || fail "the night carried on with the recipe it woke up on: $(cat "$dir/out")"
    [ ! -s "$dir/log" ] \
        || fail "the recipe that woke up on the old commit built and fuzzed before handing over"
    [ "$(git -C "$night_repo" rev-parse HEAD)" = "$green" ] \
        || fail "the night ended on $(git -C "$night_repo" rev-parse HEAD) rather than the commit it advanced to"
    [ "$status" -eq 0 ] || fail "a night that advanced exited $status: $(cat "$dir/out")"

    # What is left of the window is handed on, rather than a fresh one. A night
    # that started the clock again would run until morning past its deadline,
    # which is arm one's mistake made where nothing else would catch it.
    left=$(sed -n 's/.*ran, with \([0-9-]*\) s left.*/\1/p' "$dir/out")
    [ -n "$left" ] || fail "the recipe taking over was told no window at all"
    [ "$left" -le 20 ] \
        || fail "the recipe taking over was given $left s of a twenty-second window"

    # Seven: a night advances once. The commit it lands on is the newest green
    # there was a moment ago, so the recipe taking over almost always finds
    # nothing to move to — but "almost always" is a bound on CI's timing and not
    # on this loop, and a night that spent itself chasing a branch that kept
    # going green would say nothing about why.
    git -C "$night_repo" checkout -q --detach "$woke_on"
    : >"$dir/log"
    (cd "$night_repo" && STUB_LOG="$dir/log" PATH="$dir/bin:$PATH" \
        GH_GREEN="$dir/night-green" RCAD_FUZZ_ALREADY_ADVANCED=1 \
        RCAD_FUZZ_LOGS="$dir/logs" RCAD_FUZZ_CORPUS="$dir/corpus" \
        {{just_executable()}} fuzz-overnight 8) >"$dir/out" 2>&1 \
        || fail "a night told it had already advanced exited non-zero: $(cat "$dir/out")"
    [ "$(git -C "$night_repo" rev-parse HEAD)" = "$woke_on" ] \
        || fail "a night told it had already advanced moved the tree a second time"
    # Counted off `fuzz_all` rather than written down. What this arm is about is
    # the hand-over, not the roster — arm one owns the roster and says the
    # number out loud there — and a literal here is a second place to remember
    # when a target is added. One was, between this arm being written and the
    # branch it was written on reaching `main`.
    targets=$(wc -w <<<"{{ fuzz_all }}")
    [ "$(grep -c 'fuzz run' "$dir/log")" -eq "$targets" ] \
        || fail "the night that took over did not go on to fuzz all $targets targets: $(cat "$dir/out")"

    # Eight: with no `cargo` on the PATH at all, the night says that and does not
    # offer to install a nightly toolchain. This is 2026-09-15 exactly (§108.2):
    # the real reason was in `build.log` and the only sentence addressed to a
    # person named the toolchain, which was never the question.
    #
    # **The PATH is built here rather than borrowed, and asserted empty of
    # `cargo` before it is used.** Borrowing `/usr/bin:/bin` passes on this
    # machine, where cargo lives under `~/.cargo`, and silently tests nothing
    # wherever cargo comes from a package — an arm passing for a property of the
    # environment it ran in, which is §77.6's mistake and which this repo has
    # now made three times.
    # Built by dropping every PATH entry that holds a `cargo` and keeping the
    # rest, rather than by listing the tools the night needs into a fresh
    # directory. That list was tried and it is not a list anybody can finish:
    # the first attempt died on `id`, which this justfile runs in a top-level
    # backtick before any recipe body starts.
    bare=""
    IFS=: read -ra path_parts <<<"$PATH"
    for part in "${path_parts[@]}"; do
        [ -n "$part" ] || continue
        if [ -x "$part/cargo" ]; then continue; fi
        bare="${bare:+$bare:}$part"
    done
    if PATH="$bare" command -v cargo >/dev/null 2>&1; then
        fail "the no-cargo fixture still has a cargo in it, so the arm below would prove nothing"
    fi
    # And the drop has to have taken cargo and nothing else. Where cargo ships in
    # the same directory as the coreutils — a distribution package rather than
    # rustup — this arm cannot run, and it says so instead of failing further
    # down for a reason that reads like the night's fault.
    for needed in bash git id date find sed awk grep tail wc timeout mktemp; do
        if ! PATH="$bare" command -v "$needed" >/dev/null 2>&1; then
            fail "dropping cargo from the PATH also dropped $needed, so this arm cannot run on this machine"
        fi
    done

    : >"$dir/log"
    status=0
    PATH="$bare" STUB_LOG="$dir/log" RCAD_FUZZ_ALREADY_ADVANCED=1 \
        RCAD_FUZZ_LOGS="$dir/logs" RCAD_FUZZ_CORPUS="$dir/corpus" \
        {{just_executable()}} fuzz-overnight 8 >"$dir/out" 2>&1 || status=$?
    [ "$status" -ne 0 ] || fail "a night with no cargo at all reported success"
    grep -q 'no `cargo` on this PATH at all' "$dir/out" \
        || fail "a night with no cargo did not say so: $(cat "$dir/out")"
    if grep -q 'rustup toolchain install nightly' "$dir/out"; then
        fail "a night with no cargo offered to install a nightly, which is the sentence that misled on 2026-09-15"
    fi
    grep -q 'BUILD FAILED' "$dir/logs/latest" \
        || fail "a night that could not build did not say BUILD FAILED in the summary line"

    # Nine: a window with nothing left in it skips every target and says so,
    # rather than starting five fresh eight-hour runs past the deadline.
    #
    # Arm four used to reach this path on its way past an overrun, and cannot
    # any more: the five no longer wait for each other, so there is no longer a
    # way for one of them to spend the window before the others are started.
    # A window of zero is what is left of that case, and it is the whole of it.
    : >"$dir/log"
    status=0
    run 0 || status=$?
    [ "$status" -ne 0 ] || fail "every target was skipped and the night reported success"
    [ "$(grep -c 'fuzz run' "$dir/log")" -eq 0 ] \
        || fail "a window with no seconds in it still started a target"
    grep -q 'TO LOOK AT.*no-time' "$dir/logs/latest" \
        || fail "a night that skipped every target did not say so: $(cat "$dir/logs/latest")"

    # Ten: what a target found is carried out of the worktree on a clock, and
    # not only when a target finishes.
    #
    # §91's reason for copying at all is a night killed from outside, and while
    # the five ran in turn the copy rode along on four target boundaries spread
    # across the night. Run together, every target finishes in the same minute
    # at the far end of the window, so those boundaries are gone and a copy that
    # waited for them would keep nothing from a night killed at 04:00 — strictly
    # worse than what it replaced.
    #
    # Its own log root, because the arms above have already written this date's
    # artifacts into `$dir/logs` and the question here is whether *this* run
    # copied anything yet. Four seconds into a twelve-second window: the targets
    # are six seconds from returning, so nothing but the clock can have done it.
    : >"$dir/log"
    copier_logs="$dir/logs-copier"
    RCAD_FUZZ_COPY_EVERY=1 STUB_LOG="$dir/log" PATH="$dir/bin:$PATH" \
        STUB_ARTIFACT_TAG="$tag" RCAD_FUZZ_ALREADY_ADVANCED=1 \
        RCAD_FUZZ_LOGS="$copier_logs" RCAD_FUZZ_CORPUS="$dir/corpus" \
        {{just_executable()}} fuzz-overnight 12 >"$dir/out" 2>&1 &
    night_pid=$!
    sleep 4
    # Guarded, and not left to `find` to report: with nothing copied the
    # directory does not exist at all, `find` exits 1, and `pipefail` takes the
    # recipe down before the assertion below can say which arm failed or why.
    # Which is what happened the first time this arm was run against a night
    # with its copier taken out — the test failed, correctly, and said nothing.
    carried=0
    copier_night="$copier_logs/$(date +%Y-%m-%d)/artifacts"
    if [ -d "$copier_night" ]; then
        carried=$(find "$copier_night" -name "slow-unit-$tag" | wc -l)
    fi
    wait "$night_pid" || fail "the night under the copier exited non-zero: $(cat "$dir/out")"
    [ "$carried" -ge 1 ] \
        || fail "four seconds into a twelve-second window nothing had been carried out of the worktree, so a night killed before its targets finish keeps nothing"

    # Eleven: exit 70 is not a crash. This is §133.7 as an arm, and it is the one
    # the recipe this replaced would fail: every target fuzzed the whole window,
    # found nothing, and exited 70 because the last of its jobs happened to time
    # out. Two real runs of the same command on the same corpus exited 70 and 0,
    # so a night that read the status would have called one of them a find.
    : >"$dir/log"
    status=0
    STUB_CRASH=run run 8 || status=$?
    [ "$status" -eq 0 ] \
        || fail "every target exited 70 after fuzzing its whole window and the night reported $status: $(cat "$dir/logs/latest")"
    grep -q 'nothing found' "$dir/logs/latest" \
        || fail "a night whose targets exited 70 did not say it found nothing: $(cat "$dir/logs/latest")"

    # Twelve: a crash is found by what it wrote, not by what anything exited.
    # **The night exits 0 everywhere here** — which is fork mode's other half and
    # the more dangerous one: a crash in job 3 of 40 000 leaves the parent's
    # `ExitCode` holding whichever job was collected last, so a status mapping
    # that trusted zero would report this night clean.
    : >"$dir/log"
    status=0
    STUB_ARTIFACT_KIND=crash- STUB_ARTIFACT_KIND_TARGET=two_solids_fused_by_a_stranger \
        run 8 || status=$?
    [ "$status" -ne 0 ] \
        || fail "a target wrote a crash artifact and the night reported success: $(cat "$dir/logs/latest")"
    grep -q 'TO LOOK AT.*crashed:' "$dir/logs/latest" \
        || fail "the summary line does not say a crash was written: $(cat "$dir/logs/latest")"
    rm -f fuzz/artifacts/two_solids_fused_by_a_stranger/crash-"$tag"

    # Thirteen: and it is **tonight's** crash that fails tonight, not one from a
    # week ago. `fuzz/artifacts/` is gitignored and never cleaned, so it
    # accumulates in the tree the timer keeps; a night that counted all of it
    # would fail every night from the first crash onwards and stop being read,
    # which is §91's failure with the sign flipped.
    #
    # Dated a day back rather than merely written before the run: the marker is
    # made seconds before the targets start, and a file written in the same second
    # cannot be told from one written just after it.
    : >"$dir/log"
    mkdir -p fuzz/artifacts/where_two_surfaces_cross
    stale="fuzz/artifacts/where_two_surfaces_cross/crash-stale-$tag"
    echo "what last week found" >"$stale"
    touch -d "1 day ago" "$stale"
    status=0
    run 8 || status=$?
    rm -f "$stale"
    [ "$status" -eq 0 ] \
        || fail "a crash left over from an earlier night failed this one: $(cat "$dir/logs/latest")"
    grep -q 'nothing found' "$dir/logs/latest" \
        || fail "a carried-over crash artifact was reported as tonight's find: $(cat "$dir/logs/latest")"

    # Fourteen: a `timeout-` artifact is a measurement and not a fault (§135).
    # `-timeout=10` makes one of these a nightly certainty on
    # `where_two_surfaces_cross`, whose tracked corpus holds inputs of 17 to 19
    # seconds — the replay writes one before any fuzzing starts. A night that
    # failed on it would fail every night, and a night that always fails is a
    # night nobody reads. It is still carried out and still counted.
    : >"$dir/log"
    status=0
    STUB_ARTIFACT_KIND=timeout- STUB_ARTIFACT_KIND_TARGET=where_two_surfaces_cross \
        run 8 || status=$?
    [ "$status" -eq 0 ] \
        || fail "a timed-out input failed the night, so every night fails from here on: $(cat "$dir/logs/latest")"
    grep -q 'nothing found' "$dir/logs/latest" \
        || fail "a timed-out input was reported as a find: $(cat "$dir/logs/latest")"
    night="$dir/logs/$(date +%Y-%m-%d)"
    [ -f "$night/artifacts/where_two_surfaces_cross/timeout-$tag" ] \
        || fail "a timed-out input was not carried out of the worktree, so nobody can see a new one appear"
    rm -f fuzz/artifacts/where_two_surfaces_cross/timeout-"$tag"

    # Fifteen: **a hand run gets neither flag**, and that is not tidiness.
    # `-ignore_timeouts` is a fork-mode flag, so without `-fork` a timeout kills
    # the process — and `fuzz/corpus/where_two_surfaces_cross` is where the 17-to-19
    # second inputs live. A `just fuzz 60` carrying `-timeout=10` would die on its
    # own corpus replay before fuzzing a single new input (§133.7).
    #
    # Run against `fuzz-target` through `fuzz` rather than through `fuzz-overnight`,
    # because the hand path is the one call site the night never takes. It lives in
    # this arm's file because this is where the stub `cargo` is.
    : >"$dir/log"
    STUB_LOG="$dir/log" PATH="$dir/bin:$PATH" {{just_executable()}} fuzz 2 >"$dir/out" 2>&1 \
        || fail "a hand-run \`just fuzz\` failed: $(cat "$dir/out")"
    grep -q 'fuzz run where_two_surfaces_cross' "$dir/log" \
        || fail "a hand-run \`just fuzz\` did not reach the target at all: $(cat "$dir/log")"
    # Negated with `!` and not written as `grep && fail`: the passing case is the
    # grep finding nothing, which is a non-zero status, and `set -e` would take
    # the recipe down on the arm that just succeeded.
    ! grep -q 'timeout=' "$dir/log" \
        || fail "a hand run was given a per-input timeout, which without -fork kills it on its own corpus replay"
    ! grep -q 'fork=' "$dir/log" \
        || fail "a hand run was given -fork, so a crash no longer stops it and prints"

# `fuzz-watermark` picks the night's commit, and every way it can pick wrongly
# costs a night: too far back and the fuzzer re-finds what is already fixed,
# too far forward and it fuzzes what nothing has read.
#
# Run against a fixture repository with a stub `gh`, so no arm here needs the
# network, an account, or the state of this tree.
[doc("`fuzz-watermark` moves onto green, forward only, and stays put when it cannot tell.")]
check-fuzz-watermark-selftest:
    #!/usr/bin/env bash
    set -euo pipefail
    dir=$(mktemp -d)
    trap 'rm -rf "$dir"' EXIT
    mkdir -p "$dir/bin"

    repo="$dir/repo"
    git init -q -b main "$repo"
    echo "a file to dirty later" >"$repo/README"
    git -C "$repo" add README
    at() {
        git -C "$repo" -c user.email=t@t -c user.name=t \
            commit -q --allow-empty -m "$1"
        git -C "$repo" rev-parse HEAD
    }
    first=$(at one)
    second=$(at two)
    third=$(at three)
    git init -q --bare "$dir/origin.git"
    git -C "$repo" remote add origin "$dir/origin.git"
    git -C "$repo" push -q origin main

    # Green, and not on `main`. A run for some other branch must not move the
    # night, and `--branch main` is a filter on the *name* a run was made under.
    git -C "$repo" checkout -q -b elsewhere "$first"
    aside=$(at aside)
    git -C "$repo" checkout -q main

    # Answers with whatever the fixture put in the file, newest first, which is
    # the order the real `gh run list` returns.
    cat >"$dir/bin/gh" <<'STUB'
    #!/usr/bin/env bash
    cat "$GH_GREEN"
    STUB
    chmod +x "$dir/bin/gh"

    fail() {
        echo "fuzz-watermark drifted from its own fixtures: $1"
        [ -f "$dir/why" ] && sed 's/^/    /' "$dir/why"
        exit 1
    }

    # Stdout is the answer and stderr is the reason for a blank one, so the
    # night can print the reason and still test the answer for emptiness.
    picked() {
        GH_GREEN="$dir/green" PATH="$dir/bin:$PATH" \
            {{just_executable()}} --justfile "{{justfile()}}" \
            --working-directory "$repo" fuzz-watermark 2>"$dir/why"
    }

    # One: the newest green on `main`, and not merely the first one that
    # qualifies. A loop that took the last match would pass every other arm here.
    git -C "$repo" checkout -q --detach "$first"
    printf '%s\n%s\n' "$third" "$second" >"$dir/green"
    got=$(picked) || fail "deciding exited non-zero, so a night would not have run"
    [ "$got" = "$third" ] \
        || fail "with two green commits ahead it chose ${got:-nothing}, not the newest"

    # Two: a green commit that is not on `main` is not a candidate, and the one
    # behind it on `main` is.
    printf '%s\n%s\n' "$aside" "$second" >"$dir/green"
    got=$(picked) || fail "deciding exited non-zero"
    [ "$got" = "$second" ] \
        || fail "it chose ${got:-nothing} over the green commit that is actually on main"

    # Three: forward only. A tree already standing on the last green stays, and
    # says so rather than checking out what it is already on.
    git -C "$repo" checkout -q --detach "$third"
    printf '%s\n' "$third" >"$dir/green"
    got=$(picked) || fail "deciding exited non-zero"
    [ -z "$got" ] || fail "it offered to advance to $got from the commit it is on"
    grep -q 'already on the last green' "$dir/why" \
        || fail "standing on the last green was not said out loud"

    # Four: never backwards. Somebody advanced this tree by hand — which is what
    # §81.4 asked for and still works — and the night must not undo it.
    printf '%s\n' "$second" >"$dir/green"
    got=$(picked) || fail "deciding exited non-zero"
    [ -z "$got" ] \
        || fail "it offered to move back to $got from a commit further along"

    # Five: a `gh` that cannot answer leaves the night where it is, and exits 0
    # so that the night still runs. A stale run beats no run.
    git -C "$repo" checkout -q --detach "$first"
    GH_GREEN="$dir/absent" PATH="$dir/bin:$PATH" \
        {{just_executable()}} --justfile "{{justfile()}}" \
        --working-directory "$repo" fuzz-watermark >"$dir/out" 2>"$dir/why" \
        || fail "a gh that cannot answer made the whole night fail"
    [ ! -s "$dir/out" ] || fail "it named a commit on an answer it never got"
    grep -q 'staying on' "$dir/why" || fail "it stayed put without saying so"

    # Six: a dirty tree is never checked out from under. The night is unattended
    # and whatever is uncommitted there is somebody's, however it got there.
    printf '%s\n' "$third" >"$dir/green"
    echo "something uncommitted" >>"$repo/README"
    got=$(picked) || fail "deciding exited non-zero"
    [ -z "$got" ] || fail "it offered to check out $got over uncommitted work"
    grep -q 'uncommitted changes' "$dir/why" \
        || fail "uncommitted work was passed over without saying why"
    git -C "$repo" checkout -q -- README

    # The arms below are §173.9's: a green recorded by a local `just gate`
    # counts, because since §172 no push can be CI-green and a night that waits
    # for one fuzzes a tree that only gets older.
    greens="$repo/.git/rcad-greens"

    # Seven: gh names nothing at all and the local record carries the night.
    # This is the arm that fails without §173.9 — before it, an empty `gh`
    # answer was the end of the question.
    git -C "$repo" checkout -q --detach "$first"
    : >"$dir/green"
    printf '%s 2026-09-20T00:00:00Z here\n' "$third" >"$greens"
    got=$(picked) || fail "deciding exited non-zero with a local green to go on"
    [ "$got" = "$third" ] \
        || fail "with gh silent and $third recorded green locally it chose ${got:-nothing}"

    # Eight: the local record is not a way around the other two conditions. A
    # gate that went green on a branch is still not a green commit on `main`.
    printf '%s 2026-09-20T00:00:00Z here\n' "$aside" >"$greens"
    got=$(picked) || fail "deciding exited non-zero"
    [ -z "$got" ] \
        || fail "a locally recorded green off main moved the night to $got"

    # Nine: the furthest forward of the two sources wins, whichever source it
    # came from and whichever order they merge in. Merging two lists and
    # de-duplicating them destroys gh's newest-first order, so the walk has to
    # work this out rather than take the first thing that qualifies.
    printf '%s\n' "$second" >"$dir/green"
    printf '%s 2026-09-20T00:00:00Z here\n' "$third" >"$greens"
    got=$(picked) || fail "deciding exited non-zero"
    [ "$got" = "$third" ] \
        || fail "gh said $second and the record said $third; it chose ${got:-nothing}"
    printf '%s\n' "$third" >"$dir/green"
    printf '%s 2026-09-20T00:00:00Z here\n' "$second" >"$greens"
    got=$(picked) || fail "deciding exited non-zero"
    [ "$got" = "$third" ] \
        || fail "with the sources swapped it chose ${got:-nothing}, so it took a side not a commit"

    rm -f "$greens"

    # Ten: what may be written down in the first place. `gate-docs` re-enters
    # `gate` with a trimmed step list and reaches the recorder as a pass, having
    # compiled nothing — and a green that skipped `test` is not a reading of the
    # code. This is the mistake that would leave no trace afterwards: the file
    # would name a commit and say nothing about how little ran for it.
    record() {
        {{just_executable()}} --justfile "{{justfile()}}" \
            --working-directory "$repo" _record-green "$1"
    }
    record "check-decisions check-hub" >"$dir/said" 2>&1
    [ ! -s "$greens" ] \
        || fail "a step list that compiles nothing recorded a green anyway"
    grep -q 'read no code' "$dir/said" \
        || fail "it declined to record without saying why: $(cat "$dir/said")"

    record "{{ gate_steps }}" >"$dir/said" 2>&1
    grep -q "^$(git -C "$repo" rev-parse HEAD) " "$greens" \
        || fail "a full gate's green was not written down: $(cat "$dir/said")"

    record "{{ gate_steps }}" >"$dir/said" 2>&1
    [ "$(wc -l <"$greens")" = "1" ] \
        || fail "the same commit was recorded twice"

    echo "something uncommitted" >>"$repo/README"
    rm -f "$greens"
    record "{{ gate_steps }}" >"$dir/said" 2>&1
    [ ! -s "$greens" ] \
        || fail "a green was recorded against a tree the sha does not name"
    grep -q 'uncommitted changes' "$dir/said" \
        || fail "it declined a dirty tree without saying why: $(cat "$dir/said")"
    git -C "$repo" checkout -q -- README

# `gate-fresh` writes its green down, which is what `exec docker` made
# impossible: `exec` replaces the shell, so there was no process left to record
# anything and the recipe still exited 0. The strongest green this machine can
# produce was the only one leaving no trace, and nothing said so.
#
# The container is stubbed rather than run. Ten minutes and a docker daemon are
# not what this is asking about — it asks whether a pass reaches `_record-green`
# on the host and a failure does not, and that is answered by a stub that exits
# with whichever status the arm wants.
[doc("`gate-fresh` records a green when the container passes, and not when it fails.")]
check-gate-fresh-selftest:
    #!/usr/bin/env bash
    set -euo pipefail
    dir=$(mktemp -d)
    trap 'rm -rf "$dir"' EXIT
    mkdir -p "$dir/bin" "$dir/corpus"

    repo="$dir/repo"
    git init -q -b main "$repo"
    echo "a tree to clone" >"$repo/README"
    git -C "$repo" add README
    git -C "$repo" -c user.email=t@t -c user.name=t commit -q -m one
    head=$(git -C "$repo" rev-parse HEAD)
    greens="$repo/.git/rcad-greens"

    cat >"$dir/bin/docker" <<'STUB'
    #!/usr/bin/env bash
    printf '%s\n' "$@" >>"$DOCKER_ARGV"
    exit "${DOCKER_STATUS:-0}"
    STUB
    chmod +x "$dir/bin/docker"

    fail() {
        echo "gate-fresh drifted from its own fixtures: $1"
        [ -f "$dir/said" ] && sed 's/^/    /' "$dir/said"
        exit 1
    }

    # The corpus is pointed at an empty directory rather than this machine's,
    # because the recipe only checks that one is there and a selftest that reads
    # the real one is answering a question about the machine (§77.6).
    fresh() {
        DOCKER_STATUS="$1" DOCKER_ARGV="$dir/argv" \
            PATH="$dir/bin:$PATH" RCAD_STEP_CORPUS="$dir/corpus" \
            {{ just_executable() }} --justfile "{{ justfile() }}" \
            --working-directory "$repo" gate-fresh >"$dir/said" 2>&1
    }

    # One: a container that passed is a green on the host, under the sha that was
    # cloned. This is the arm `exec docker` fails while exiting 0.
    fresh 0 || fail "it failed over a container that passed"
    grep -q "^$head " "$greens" 2>/dev/null \
        || fail "it passed and wrote no green for $(git -C "$repo" rev-parse --short HEAD)"

    # Two: a container that failed records nothing, and its status is what comes
    # out. Bookkeeping that ran anyway would write down a green for a gate that
    # went red, which is the one entry no reader could catch.
    rm -f "$greens"
    status=0
    fresh 3 || status=$?
    [ "$status" -eq 3 ] \
        || fail "a container exiting 3 came out as $status"
    [ ! -s "$greens" ] \
        || fail "a green was recorded for a container that failed"

    # Three: the clone is gone afterwards. `exec` discarded the EXIT trap with
    # the shell, so every run left its tree behind and nothing noticed for as
    # long as nothing else needed the trap. Read out of the arguments the stub
    # was handed, because the path is a `mktemp -d` this arm never sees
    # otherwise.
    clone=$(grep -o '^/.*/rcad:/rcad$' "$dir/argv" | head -1 | cut -d: -f1)
    [ -n "$clone" ] \
        || fail "the stub was handed no clone to mount, so this arm compared nothing"
    [ ! -e "$clone" ] \
        || fail "the clone was left behind at $clone"

# `cleanup`'s plain `rm -rf "$work"` is the one call in the trap that a died
# container can defeat: root wrote into the tree, `chown` never ran, and no
# `chmod -R u+w` by this uid reaches a file it does not own. A real root-owned
# file needs real root to make, which a selftest does not have — so a marker
# file stands in for it. The fake `docker` from the main run drops the marker
# inside the mounted tree instead of exiting cleanly, and a fake `rm` ahead of
# it on `PATH` refuses to remove anything that still carries it, the same
# refusal the host's own `rm` would hit. Only the fallback `docker run ... rm
# -rf /w/rcad` — told apart from the main run by its own `-v ...:/w` mount —
# clears the marker, exactly as a container really would, being root inside
# too. Everything the fake `rm` does not recognise it hands straight to the
# real one, so it cannot break anything outside the one case it models.
[doc("gate-fresh's cleanup reaches for docker when the plain rm on a leftover tree fails.")]
check-gate-fresh-cleanup-selftest:
    #!/usr/bin/env bash
    set -euo pipefail
    dir=$(mktemp -d)
    trap 'rm -rf "$dir"' EXIT
    mkdir -p "$dir/bin" "$dir/corpus"

    repo="$dir/repo"
    git init -q -b main "$repo"
    echo "a tree to clone" >"$repo/README"
    git -C "$repo" add README
    git -C "$repo" -c user.email=t@t -c user.name=t commit -q -m one

    # Captured before PATH is narrowed to $dir/bin, so the fake rm below can
    # still reach the real one, and the fake docker can delete for real.
    real_rm=$(command -v rm)

    cat >"$dir/bin/docker" <<'STUB'
    #!/usr/bin/env bash
    printf '%s\n' "$@" >>"$DOCKER_ARGV"
    fallback_mount=""
    clone_mount=""
    for a in "$@"; do
        case "$a" in
            *:/w) fallback_mount="$a" ;;
            *:/rcad) clone_mount="$a" ;;
        esac
    done
    if [ -n "$fallback_mount" ]; then
        host="${fallback_mount%%:*}"
        if [ "${DOCKER_CLEANUP_STATUS:-0}" -eq 0 ]; then
            "$REAL_RM" -rf "$host/rcad" 2>/dev/null || true
        fi
        exit "${DOCKER_CLEANUP_STATUS:-0}"
    fi
    host="${clone_mount%%:*}"
    touch "$host/.root-owned" 2>/dev/null || true
    exit "${DOCKER_STATUS:-1}"
    STUB
    chmod +x "$dir/bin/docker"

    cat >"$dir/bin/rm" <<'STUB'
    #!/usr/bin/env bash
    for a in "$@"; do
        case "$a" in -*) continue ;; esac
        if [ -e "$a" ] && find "$a" -name .root-owned -print -quit 2>/dev/null | grep -q .; then
            echo "rm: cannot remove '$a/.root-owned': Operation not permitted" >&2
            exit 1
        fi
    done
    exec "$REAL_RM" "$@"
    STUB
    chmod +x "$dir/bin/rm"

    fail() {
        echo "gate-fresh cleanup drifted from its own fixtures: $1"
        [ -f "$dir/said" ] && sed 's/^/    /' "$dir/said"
        exit 1
    }

    # The corpus is pointed at an empty directory rather than this machine's,
    # for the same reason `check-gate-fresh-selftest` does: the recipe only
    # checks that one is there (§77.6).
    fresh() {
        DOCKER_STATUS=5 DOCKER_CLEANUP_STATUS="$1" DOCKER_ARGV="$dir/argv" \
            REAL_RM="$real_rm" PATH="$dir/bin:$PATH" RCAD_STEP_CORPUS="$dir/corpus" \
            {{ just_executable() }} --justfile "{{ justfile() }}" \
            --working-directory "$repo" gate-fresh >"$dir/said" 2>&1
    }

    # One: the fallback docker removes what the plain rm could not, so the
    # tree is gone and the exit code is still the died container's own — the
    # cleanup neither swallows it nor overwrites it with the fallback's.
    status=0
    fresh 0 || status=$?
    [ "$status" -eq 5 ] \
        || fail "a died container's status 5 came out as $status"
    grep -qx '/w/rcad' "$dir/argv" \
        || fail "the fallback docker was never called"
    work=$(grep -o '^/.*/rcad:/rcad$' "$dir/argv" | head -1 | sed 's#/rcad:/rcad$##')
    [ -n "$work" ] || fail "no work directory recovered from the stub's log"
    [ ! -e "$work" ] \
        || fail "the tree is still at $work after the fallback docker cleaned it"

    # Two: the fallback docker fails too, so the tree is really left behind and
    # the message names it — but the exit code still carries the original rc,
    # not the fallback's own failure.
    rm -f "$dir/argv"
    status=0
    fresh 9 || status=$?
    [ "$status" -eq 5 ] \
        || fail "a died container's status 5 came out as $status over a failed fallback"
    grep -qx '/w/rcad' "$dir/argv" \
        || fail "the fallback docker was never called"
    work=$(grep -o '^/.*/rcad:/rcad$' "$dir/argv" | head -1 | sed 's#/rcad:/rcad$##')
    [ -n "$work" ] || fail "no work directory recovered from the stub's log"
    [ -e "$work" ] \
        || fail "the tree at $work is gone even though the fallback docker failed"
    grep -q "is left behind, owned by the container's root" "$dir/said" \
        || fail "no message named the leftover tree"

# The morning after a night's fuzzing, in one call.
#
# What this replaces is four or five calls that each had to read a directory
# somebody else wrote: the `latest` line, then the night's directory, then the
# artifacts under it, then the decoder on each one, then whether any of it still
# matters. Every one of those costs a fresh session the whole context of the
# ones before it.
#
# `night` defaults to whatever `latest` names, which is what the timer wrote
# when it finished. Giving one by hand reads an older night, and then the commit
# is only known if `latest` still describes it — a night's directory does not
# record its own commit, and this says "unknown" rather than reporting the wrong
# one.
#
# **Both places an input can be.** `fuzz-overnight` copies what a target found
# into `<night>/artifacts/` (§91), but the nights before that landed have their
# inputs in `<root>/artifacts-rescued-<date>/` beside the night instead, put
# there by hand — and the night of 2026-09-14 is one of them, because the body
# that ran had been read from a commit older than the copy-out. Both layouts are
# `<target-name>/<file>`, which is where the target name for the decoder comes
# from: it is the artifact's own parent directory, not a guess off the `TO LOOK
# AT` list, which names recipes rather than targets.
#
# **`against` is the tree the decoder runs in, and `main` is the point.** The
# night stands on an older commit by design (§90), so "it crashed" is yesterday's
# news and "it still crashes on `main`" is the question. When `main` is not what
# this worktree is standing on, a detached worktree of it is made and taken away
# again, so nobody's tree moves.
#
# A decode that fails is reported and not fatal: the exit code here says whether
# the triage ran, because a morning that stops on the first undecodable input
# tells you nothing about the other five.
#
# **What each input then did is `fuzz-triage-verdict` below, off the decoder's
# `outcome:` line.** It is a separate recipe because the gate can hand that one
# canned text and this one it cannot; §91.5 has the morning that bought the
# split.
#
# **The target comes from the artifact's own parent directory, and that name is
# checked against `fuzz/fuzz_targets/*.rs` before anything is decoded.** A flat
# directory of rescued artifacts — no target subdirectory at all — hands the
# decoder a name that is not a target, and every one of those used to come back
# as "did not come back clean", which reads exactly like a live crash and is
# none. That input gets a third outcome instead, counted apart from the ones
# that were actually decoded and actually failed.
[doc("The morning after: what the night found, decoded, and whether main still fails.")]
fuzz-triage night="" against="main":
    #!/usr/bin/env bash
    set -uo pipefail
    root='{{ fuzz_log_root }}'
    latest="$root/latest"
    night='{{ night }}'
    if [ -z "$night" ]; then
        if [ ! -f "$latest" ]; then
            echo "nothing to triage: $latest does not exist, so no run has reported a night"
            exit 1
        fi
        night=$(awk 'END{print $NF}' "$latest")
    fi
    if [ ! -d "$night" ]; then
        echo "nothing to triage: $night is not a directory"
        exit 1
    fi

    line=""
    if [ -f "$latest" ] && [ "$(awk 'END{print $NF}' "$latest")" = "$night" ]; then
        line=$(tail -1 "$latest")
    fi
    echo "night:    $night"
    if [ -z "$line" ]; then
        echo "commit:   unknown — $latest describes another night, and a night's directory does not record its own"
        echo "targets:  unknown for the same reason; each artifact below names its own"
    elif [ "$(awk '{print $3}' <<<"$line")" = "BUILD" ]; then
        echo "commit:   none — the targets did not build that night: $night/build.log"
        echo "targets:  none ran"
    else
        echo "commit:   $(awk '{print $3}' <<<"$line")"
        flagged=$(sed -n 's/.*TO LOOK AT (\([^)]*\)).*/\1/p' <<<"$line")
        echo "targets:  ${flagged:-nothing flagged}"
    fi
    echo "summary:  ${line:-(this night is not the one $latest describes)}"

    artifacts="$night/artifacts"
    rescued="$root/artifacts-rescued-$(basename "$night")"
    if [ -d "$artifacts" ]; then
        echo "inputs:   $artifacts"
    elif [ -d "$rescued" ]; then
        echo "inputs:   $rescued (rescued beside the night; $artifacts is not there)"
        artifacts="$rescued"
    else
        echo "inputs:   none — neither $artifacts nor $rescued exists"
        exit 0
    fi
    files=$(find "$artifacts" -type f | sort)
    if [ -z "$files" ]; then
        echo "no files under $artifacts, so there is nothing to decode"
        exit 0
    fi

    at=$(git rev-parse --verify --quiet '{{ against }}^{commit}')
    if [ -z "$at" ]; then
        echo "there is no commit named '{{ against }}' here, so nothing can be replayed against it"
        exit 1
    fi
    work="."
    if [ "$at" != "$(git rev-parse HEAD)" ]; then
        borrowed=$(mktemp -d)
        # Detached, so it never asks for a branch that is checked out elsewhere,
        # and taken away on the way out however this ends — a worktree left
        # behind would show up in `just lanes` as a lane nobody dispatched.
        trap 'git worktree remove --force "$borrowed/tree" >/dev/null 2>&1 || true; rm -rf "$borrowed"' EXIT
        if ! git worktree add --quiet --detach "$borrowed/tree" "$at"; then
            echo "could not borrow a worktree at {{ against }}, so nothing was replayed against it"
            exit 1
        fi
        work="$borrowed/tree"
    fi
    echo "replayed: {{ against }} ($(git rev-parse --short "$at")) in $work"

    valid_targets=""
    for target_rs in "$work"/fuzz/fuzz_targets/*.rs; do
        [ -e "$target_rs" ] || continue
        valid_targets="$valid_targets $(basename "$target_rs" .rs)"
    done

    still=0
    untellable=0
    while IFS= read -r file; do
        target=$(basename "$(dirname "$file")")
        echo
        echo "--- $target / $(basename "$file") ---"
        case " $valid_targets " in
            *" $target "*) ;;
            *)
                untellable=$((untellable + 1))
                echo "target:   unknown — $target names no fuzz_targets/*.rs on {{ against }}, so this could not be placed and was not decoded"
                continue
                ;;
        esac
        echo "\$ mise exec -- cargo run -q -p rcad-fuzz-explain -- $target $file"
        said=$(mktemp)
        reason=$(mktemp)
        (cd "$work" && mise exec -- cargo run -q -p rcad-fuzz-explain -- "$target" "$file") >"$said" 2>"$reason"
        decoded=$?
        cat "$said"
        if ! {{just_executable()}} fuzz-triage-verdict '{{ against }}' "$decoded" "$said"; then
            still=$((still + 1))
            [ "$decoded" -eq 0 ] || sed 's/^/    /' "$reason"
        fi
        rm -f "$said" "$reason"
    done <<<"$files"

    echo
    total=$(grep -c . <<<"$files")
    echo "$still of $total input(s) did not come back clean on {{ against }}"
    if [ "$untellable" -gt 0 ]; then
        echo "$untellable of $total input(s) could not be told which target they came from, and are not counted as failures above"
    fi

# Turns one artifact `fuzz-triage` found into a seed, which today is a `cp`
# somebody has to remember — and §98.5 is what happens when nobody does.
#
# **The kind comes off the source file's own name, not a hardcoded `crash`.**
# libFuzzer writes `crash-<sha1>`, `slow-unit-<sha1>`, `oom-<sha1>` and
# `timeout-<sha1>` — a slow-unit promoted under the crash kind would be a lie
# in the one place a later reader trusts, the file name. So the kind is read
# off, not invented: everything before the trailing sha1 and its separating
# dash. A name with nothing readable there — no dash before the sha1 at all —
# is refused rather than promoted under a guess.
#
# `name` is the human-readable slug, `crash-a-thin-pipe-on-a-fat-ones-skin` is
# the shape already in the corpus, but the sha1 on the end is what makes the
# name unique and it is never typed by hand: it is read off the file's own
# name and checked against the file's own bytes before anything is written, so
# a renamed or truncated artifact is refused rather than seeded under a false
# name. Leave `name` out and the promoted file is just `<kind>-<sha1>`.
[doc("Promote a fuzz artifact into fuzz/corpus/<target>/, sha1-checked, under its own kind.")]
fuzz-promote target file name="":
    #!/usr/bin/env bash
    set -euo pipefail
    target_rs="fuzz/fuzz_targets/{{target}}.rs"
    if [ ! -f "$target_rs" ]; then
        echo "no such target: $target_rs does not exist"
        exit 1
    fi
    file="{{file}}"
    if [ ! -f "$file" ]; then
        echo "no such file: $file"
        exit 1
    fi
    base=$(basename "$file")
    embedded=""
    if [ "${#base}" -ge 40 ]; then
        embedded="${base: -40}"
    fi
    if ! [[ "$embedded" =~ ^[0-9a-f]{40}$ ]]; then
        echo "refused: $base does not end in a 40-character sha1, so nothing was promoted"
        exit 1
    fi
    actual=$(sha1sum "$file" | awk '{print $1}')
    if [ "$actual" != "$embedded" ]; then
        echo "refused: $base names sha1 $embedded but the file hashes to $actual"
        exit 1
    fi
    kind="${base%-$embedded}"
    if [ "$kind" = "$base" ] || [ -z "$kind" ]; then
        echo "refused: $base has no <kind>-<sha1> shape, so no kind could be read from it"
        exit 1
    fi
    dest_dir="fuzz/corpus/{{target}}"
    name="{{name}}"
    if [ -n "$name" ]; then
        dest="$dest_dir/$kind-$name-$embedded"
    else
        dest="$dest_dir/$kind-$embedded"
    fi
    if [ -e "$dest" ]; then
        if cmp -s "$file" "$dest"; then
            echo "already there: $dest"
            exit 0
        fi
        echo "refused: $dest already exists with different contents"
        exit 1
    fi
    mkdir -p "$dest_dir"
    cp "$file" "$dest"
    echo "promoted: $dest"

# What one replayed input did, and whether it counts against the morning's
# tally. Non-zero says it counts, which is the only thing `fuzz-triage` above
# asks of it.
#
# **The answer is the decoder's `outcome:` line and never its exit code.**
# `rcad-fuzz-explain` exits 0 whenever the bytes decoded and the case replayed,
# "whatever the case then did" — its own `main.rs` says so in the sentence above
# `fn main` — so a replay that panicked exits 0 exactly like one that answered.
# Reading the exit code had `fuzz-triage` print `outcome: panicked` and `still
# fails on main: no` on the same entry, and total `0 of 8` on the morning of
# 2026-09-17 when the answer was two. `docs/decisions.md` §91.5.
#
# Split out of that recipe rather than left inline because this is the half the
# gate can test. Deciding needs only text; producing the text needs a workspace,
# a nightly build and an input that panics — and an input that panics for ever
# is the one thing this kernel is trying not to have, so the arm that owned one
# would go red the day the bug was fixed.
#
# `Outcome` is three variants and says it is exhaustive on purpose, so a fourth
# is a change to this contract. The last arm says so and counts the input rather
# than letting a name this recipe has never seen pass as clean.
[private]
fuzz-triage-verdict against decoded said:
    #!/usr/bin/env bash
    set -uo pipefail
    if [ '{{ decoded }}' -ne 0 ]; then
        echo "still fails on {{ against }}: exit {{ decoded }}, it never got as far as replaying — why is on stderr:"
        exit 1
    fi
    outcome=$(sed -n 's/^outcome: \([a-z]*\).*/\1/p' '{{ said }}')
    case "$outcome" in
        panicked)
            echo "still fails on {{ against }}: yes — it replayed and panicked"
            exit 1 ;;
        refused|succeeded)
            echo "still fails on {{ against }}: no — it replayed and $outcome"
            exit 0 ;;
        *)
            echo "still fails on {{ against }}: unknown — the decoder exited 0 and said '${outcome:-nothing}', which is not one of rcad-fuzz-explain's three outcomes, so its contract has moved and this recipe has not"
            exit 1 ;;
    esac

# Pins `fuzz-triage` against nights this builds itself: a `latest` line in the
# shape `fuzz-overnight` writes, artifacts in both of the two places they land,
# and a night nothing has reported.
#
# **The decoder is asserted by name, first, and that is not decoration.** It is
# another lane's crate, and without it every arm below would still pass: a
# decoder that cannot run reports as an input that did not come back clean,
# which is the right thing for the recipe to do and says nothing at all about
# whether the decoder was ever there. So this arm fails loudly until
# `rcad-fuzz-explain` exists, rather than going quietly green over a contract
# nothing implements.
#
# `HEAD` rather than `main` for `against`, because the arms are about the
# recipe's parse and not about which tree it borrows: pointing it at `main` from
# a lane would have every gate build a second worktree of the workspace to
# decode a fixture of twenty junk bytes.
[private]
check-fuzz-triage-selftest:
    #!/usr/bin/env bash
    set -euo pipefail
    cargo metadata --no-deps --format-version 1 2>/dev/null \
        | grep -q '"name":"rcad-fuzz-explain"' \
        || { echo "fuzz-triage decodes an artifact with rcad-fuzz-explain, and this workspace has no such crate"; \
             echo "it is lane-fuzz-explain's deliverable; this stays red until that branch is merged"; \
             exit 1; }

    dir=$(mktemp -d)
    trap 'rm -rf "$dir"' EXIT
    logs="$dir/logs"
    night="$logs/2026-09-14"
    crash="$night/artifacts/where_two_surfaces_cross/crash-deadbeef"
    mkdir -p "$(dirname "$crash")"
    printf 'not a real crash input' >"$crash"
    printf '%s  %s  %s  %s  %s\n' \
        "2026-09-14 08:10" "9b82e9e (0 days old, 0 behind main)" \
        "4 targets, 1 TO LOOK AT (fuzz:crash)" "1 artifacts in $night/artifacts" \
        "$night" >"$logs/latest"

    fail() {
        echo "fuzz-triage drifted from its own fixtures: $1"
        echo "--- got ---"; echo "$got"
        exit 1
    }
    triage() { RCAD_FUZZ_LOGS="$logs" just fuzz-triage "$1" HEAD 2>&1; }

    # One: no night given, so it reads the one `latest` names, and everything
    # the morning asks for comes out of that one line.
    got=$(triage "") || fail "a night with one artifact made the triage itself fail"
    grep -qF "night:    $night" <<<"$got" || fail "it did not say which night it read"
    grep -q '^commit:   9b82e9e$' <<<"$got" \
        || fail "it did not name the commit that was fuzzed, which is the whole of §90"
    grep -q '^targets:  fuzz:crash$' <<<"$got" || fail "it did not name the target that was flagged"
    grep -qF "\$ mise exec -- cargo run -q -p rcad-fuzz-explain -- where_two_surfaces_cross $crash" <<<"$got" \
        || fail "it did not run the decoder on the artifact, naming the target off the artifact's own directory"
    grep -q 'still fails on HEAD' <<<"$got" \
        || fail "it decoded the input without saying whether it still fails"

    # Two: the nights before §91's copy-out landed keep their inputs in a
    # rescued directory beside the night, and 2026-09-14 is one of them.
    rm -rf "$night/artifacts"
    rescued="$logs/artifacts-rescued-2026-09-14/two_solids_fused_by_a_stranger"
    mkdir -p "$rescued"
    printf 'nor is this one' >"$rescued/crash-c0ffee"
    got=$(triage "") || fail "a night whose inputs were rescued beside it made the triage fail"
    grep -qF "inputs:   $logs/artifacts-rescued-2026-09-14" <<<"$got" \
        || fail "it did not look beside the night for the rescued inputs"
    grep -qF "rcad-fuzz-explain -- two_solids_fused_by_a_stranger $rescued/crash-c0ffee" <<<"$got" \
        || fail "it did not decode the rescued input"

    # Three: a night nothing reported still says what it can, and says the
    # commit is unknown rather than borrowing the one `latest` happens to name.
    other="$logs/2026-09-09"
    mkdir -p "$other"
    got=$(triage "$other") || fail "a night with no artifacts at all made the triage fail"
    grep -q '^commit:   unknown' <<<"$got" \
        || fail "it read another night's commit off the latest line and reported it as this one's"
    grep -qF "inputs:   none" <<<"$got" \
        || fail "a night that kept nothing did not say so"

    # Four: a directory that is not there is a failure and not an empty report.
    if triage "$logs/2026-01-01" >/dev/null 2>&1; then
        got=$(triage "$logs/2026-01-01")
        fail "a night that does not exist was triaged anyway"
    fi

    # Five: a flat directory of artifacts, no target subdirectory at all — the
    # shape `~/rcad-fuzz-logs/artifacts-rescued-2026-09-13/` was in, 88 files
    # whose parent directory names no target. Every one of those used to read
    # as "did not come back clean", indistinguishable from a live crash.
    flat_night="$logs/2026-09-13"
    mkdir -p "$flat_night"
    flat_rescued="$logs/artifacts-rescued-2026-09-13"
    mkdir -p "$flat_rescued"
    printf 'flat, no target subdirectory' >"$flat_rescued/crash-baadf00d"
    got=$(triage "$flat_night") || fail "a flat directory of artifacts made the triage itself fail"
    grep -q '^target:   unknown' <<<"$got" \
        || fail "an artifact whose parent directory names no real target was decoded anyway"
    grep -qF "1 of 1 input(s) could not be told which target they came from" <<<"$got" \
        || fail "the untellable count did not reach the tail line"
    grep -q '^0 of 1 input(s) did not come back clean on HEAD$' <<<"$got" \
        || fail "an untellable input was folded into the count of inputs that did not come back clean"

    # Six: promoting a good artifact keeps the trailing sha1 and lands it under
    # a name — `crash-<name>-<sha1>` is the shape already in the corpus.
    promote_repo="$dir/promote-repo"
    mkdir -p "$promote_repo/fuzz/fuzz_targets" "$promote_repo/fuzz/corpus"
    touch "$promote_repo/fuzz/fuzz_targets/where_two_surfaces_cross.rs"
    git -C "$promote_repo" init -q
    promote() {
        {{just_executable()}} --justfile "{{justfile()}}" \
            --working-directory "$promote_repo" fuzz-promote "$@" 2>&1
    }
    src="$dir/crash-tmp"
    printf 'a thin pipe on a fat ones skin' >"$src"
    sha1=$(sha1sum "$src" | awk '{print $1}')
    mv "$src" "$dir/crash-$sha1"
    src="$dir/crash-$sha1"
    got=$(promote where_two_surfaces_cross "$src" a-thin-pipe-on-a-fat-ones-skin) \
        || fail "promoting a good artifact failed"
    dest="$promote_repo/fuzz/corpus/where_two_surfaces_cross/crash-a-thin-pipe-on-a-fat-ones-skin-$sha1"
    [ -f "$dest" ] || fail "promote did not land the file at $dest"
    cmp -s "$src" "$dest" || fail "the promoted file's contents do not match the source"

    # Seven: a target that does not exist is refused, and gets no corpus
    # directory of its own.
    if got=$(promote no_such_target "$src" 2>&1); then
        fail "promoting into a target that does not exist was not refused"
    fi
    grep -q 'no such target' <<<"$got" || fail "the refusal did not say the target does not exist"
    [ ! -d "$promote_repo/fuzz/corpus/no_such_target" ] \
        || fail "a target that does not exist got a corpus directory anyway"

    # Eight: a file whose contents do not hash to the sha1 in its own name is
    # refused, and nothing is written into the corpus for it.
    zeros=$(printf '%040d' 0)
    bad="$dir/crash-$zeros"
    printf 'these bytes do not hash to forty zeros' >"$bad"
    if got=$(promote where_two_surfaces_cross "$bad" 2>&1); then
        fail "promoting a file whose contents do not match its own sha1 was not refused"
    fi
    grep -q 'hashes to' <<<"$got" || fail "the refusal did not say the hash mismatched"
    [ ! -e "$promote_repo/fuzz/corpus/where_two_surfaces_cross/crash-$zeros" ] \
        || fail "a hash-mismatched file was written into the corpus anyway"

    # Nine: a slow-unit is not a crash, and does not get promoted as one — the
    # kind comes off the source name, not a hardcoded "crash".
    slow_content="$dir/slow-content"
    printf 'this one just took too long, it never crashed' >"$slow_content"
    slow_sha1=$(sha1sum "$slow_content" | awk '{print $1}')
    slow="$dir/slow-unit-$slow_sha1"
    mv "$slow_content" "$slow"
    got=$(promote where_two_surfaces_cross "$slow") || fail "promoting a slow-unit artifact failed"
    slow_dest="$promote_repo/fuzz/corpus/where_two_surfaces_cross/slow-unit-$slow_sha1"
    [ -f "$slow_dest" ] || fail "a slow-unit was not promoted under its own kind"
    cmp -s "$slow" "$slow_dest" || fail "the promoted slow-unit's contents do not match the source"
    [ ! -e "$promote_repo/fuzz/corpus/where_two_surfaces_cross/crash-$slow_sha1" ] \
        || fail "a slow-unit was promoted under the crash kind"

    # Ten: a source name with nothing readable before its sha1 — no dash, no
    # kind — is refused rather than promoted under an invented kind.
    noprefix_content="$dir/noprefix-content"
    printf 'the whole name is the sha1, nothing before it' >"$noprefix_content"
    noprefix_sha1=$(sha1sum "$noprefix_content" | awk '{print $1}')
    noprefix="$dir/$noprefix_sha1"
    mv "$noprefix_content" "$noprefix"
    if got=$(promote where_two_surfaces_cross "$noprefix" 2>&1); then
        fail "promoting a file with no kind before its sha1 was not refused"
    fi
    grep -q 'no kind could be read' <<<"$got" \
        || fail "the refusal did not say it could not read a kind"

    # Eleven: the verdict is the decoder's `outcome:` line and not its exit code.
    # Every arm that feeds the decoder real bytes feeds it bytes that do not
    # decode, so all of them pass on a recipe that reads the exit code, and all
    # of them did while it read one — the morning of 2026-09-17 called two live
    # crashes clean (§91.5).
    #
    # Canned text rather than a real input, because deciding is the half that
    # was wrong and the half a gate can hold still. The exit code is pinned at 0
    # in the arms that matter: that is the whole point, since the decoder exits
    # 0 whatever the replay then did.
    verdict() {
        printf '%s' "$2" >"$dir/said"
        just fuzz-triage-verdict main "$1" "$dir/said" 2>&1
    }

    got=$(verdict 0 'outcome: panicked — assertion `left == right` failed') \
        && fail "a replay that panicked exited 0 and was counted as coming back clean"
    grep -q '^still fails on main: yes' <<<"$got" \
        || fail "a replay that panicked did not say that it still fails"

    got=$(verdict 0 'outcome: succeeded — SurfaceMeet::Curves, 2 of them; the harness reached Asked') \
        || fail "a replay that answered was counted against the morning's tally"
    grep -q '^still fails on main: no' <<<"$got" \
        || fail "a replay that answered did not say that it is gone"

    got=$(verdict 0 'outcome: refused — Error::Undecidable; the harness reached Asked') \
        || fail "a replay that gave up by name was counted as a failure, and §17.4 says giving up is an answer"

    got=$(verdict 1 '') \
        && fail "an input that never got as far as replaying was counted as clean"

    got=$(verdict 0 'outcome: exploded — a fourth thing') \
        && fail "an outcome this recipe has never heard of passed as clean"
    grep -q 'contract has moved' <<<"$got" \
        || fail "a fourth outcome was counted but not named as a contract change"

# The timer's two unit files are tracked here and **copied** into
# `~/.config/systemd/user/`. That is two records of one thing, which §60 says is
# the shape to delete, and on 2026-09-15 it had a price: the night lost eight
# hours to a `cargo` that was not on its PATH, and the fix for that lands in
# `tooling/systemd/` where systemd will never read it. Nothing made the two
# agree and neither file said whether anyone had.
#
# **An installed copy must match the tracked one; a machine with no installed
# copy is not checked.** Almost nowhere runs the night — a runner has no
# `~/.config/systemd/user` at all — so absence is the ordinary case and not a
# finding.
#
# The gap that leaves is named rather than claimed away: this cannot tell a
# machine that never installed the timer from one where the installed file was
# removed, because both look the same from here and only the second is a
# problem. What it does catch is the case that cost the night — installed, and
# behind the tree.
#
# **Half an install is a failure, not an absence.** The pair goes in together;
# one of the two on disk means somebody copied by hand and stopped, which is the
# same mistake this recipe exists for, one file earlier.
[doc("The installed rcad-fuzz timer is the one tracked in tooling/systemd.")]
check-fuzz-unit tracked="tooling/systemd":
    #!/usr/bin/env bash
    set -euo pipefail
    units="rcad-fuzz.service rcad-fuzz.timer"
    present=""
    missing=""
    for unit in $units; do
        [ -f "{{tracked}}/$unit" ] \
            || { echo "there is no {{tracked}}/$unit here to compare anything against"; exit 1; }
        if [ -f "{{systemd_user_dir}}/$unit" ]; then
            present="$present $unit"
        else
            missing="$missing $unit"
        fi
    done

    if [ -z "$present" ]; then
        echo "no rcad-fuzz timer installed under {{systemd_user_dir}}, so there is nothing to drift"
        exit 0
    fi
    if [ -n "$missing" ]; then
        echo "half of the timer is installed under {{systemd_user_dir}}:"
        echo "  there:$present"
        echo "  not there:$missing"
        echo "the pair goes in together; copy both and reload:"
        echo "    cp {{tracked}}/rcad-fuzz.service {{tracked}}/rcad-fuzz.timer {{systemd_user_dir}}/ && systemctl --user daemon-reload"
        exit 1
    fi

    drifted=""
    for unit in $units; do
        cmp -s "{{tracked}}/$unit" "{{systemd_user_dir}}/$unit" || drifted="$drifted $unit"
    done
    if [ -n "$drifted" ]; then
        for unit in $drifted; do
            echo "installed $unit is not {{tracked}}/$unit:"
            diff -u "{{systemd_user_dir}}/$unit" "{{tracked}}/$unit" | sed 's/^/    /' || true
        done
        echo "systemd runs the installed copy, so the tree is not what fires tonight:"
        echo "    cp {{tracked}}/rcad-fuzz.service {{tracked}}/rcad-fuzz.timer {{systemd_user_dir}}/ && systemctl --user daemon-reload"
        exit 1
    fi
    echo "the installed rcad-fuzz timer matches {{tracked}}"

# Pins `check-fuzz-unit` against installs this builds itself, because the one
# case that matters cannot be arranged on the machine running the gate without
# breaking that machine's timer.
#
# **Each arm asserts the sentence its case produces, not the exit code.** A
# recipe that always failed would satisfy the two failing arms and nothing else;
# a recipe that always passed would satisfy the two passing ones. The pair of
# them together is what says it is reading the files.
[private]
check-fuzz-unit-selftest:
    #!/usr/bin/env bash
    set -euo pipefail
    work=$(mktemp -d -- "${TMPDIR:-/tmp}/rcad-fuzz-unit-selftest.XXXXXX")
    trap 'rm -rf -- "$work"' EXIT

    tracked="$work/tracked"
    installed="$work/installed"
    mkdir -p "$tracked" "$installed"
    printf '[Service]\nEnvironment=PATH=%%h/.cargo/bin:/usr/bin\n' >"$tracked/rcad-fuzz.service"
    printf '[Timer]\nOnCalendar=*-*-* 00:10:00\n' >"$tracked/rcad-fuzz.timer"

    failed=0
    check() {
        local name=$1 want_rc=$2 needle=$3
        local out rc=0
        out=$(RCAD_SYSTEMD_USER="$installed" \
            {{ just_executable() }} check-fuzz-unit "$tracked" 2>&1) || rc=$?
        if [ "$rc" -ne "$want_rc" ]; then
            echo "check-fuzz-unit drifted from its own fixtures: $name wanted exit $want_rc, got $rc"
            sed 's/^/    /' <<<"$out"
            failed=1
            return
        fi
        if ! grep -qF -- "$needle" <<<"$out"; then
            echo "check-fuzz-unit drifted from its own fixtures: $name did not say \"$needle\""
            sed 's/^/    /' <<<"$out"
            failed=1
        fi
    }

    # One: nothing installed. The ordinary case on every machine but this one,
    # and on every runner, so it has to pass and say why rather than pass in
    # silence that would look the same as a match.
    check "nothing installed" 0 "so there is nothing to drift"

    # Two: half of it. The pair goes in together.
    cp "$tracked/rcad-fuzz.service" "$installed/"
    check "half installed" 1 "not there: rcad-fuzz.timer"

    # Three: both, byte for byte. The passing case.
    cp "$tracked/rcad-fuzz.timer" "$installed/"
    check "installed and equal" 0 "matches"

    # Four: installed, and behind the tree. This is the night of 2026-09-15 —
    # the fix sitting in `tooling/systemd/` while systemd runs the old copy — so
    # the arm asserts the unit's name and the line that differs, not just a
    # non-zero exit.
    printf '[Service]\n' >"$installed/rcad-fuzz.service"
    check "installed and stale" 1 "installed rcad-fuzz.service is not"
    check "installed and stale names the missing line" 1 "Environment=PATH="

    [ "$failed" -eq 0 ]

[doc("The benchmarks.")]
bench:
    cargo bench --workspace

# What the provenance log costs the three fuzz targets that never read one.
#
# The §144.9 measurement, run on its own because it is the one bench somebody
# comes looking for by name rather than by sweeping them all. Two passes over
# each target's checked-in corpus and a 200 000-draw soak: the draw as the fuzz
# target runs it, then every record that draw made built and freed again. The
# share of the two is the whole of what a stubbed `ProvenanceLog` behind a cargo
# feature could have bought.
#
# **It asserts nothing**, the way §12.5 says a human-read figure should, and
# `docs/decisions.md` §162 records what it said on the day. The two doors it
# measures through are pinned by
# `a_fuzz_draw_hands_its_provenance_back_through_the_door.rs` in `rcad-geom` and
# its namesake in `rcad-topo` — without those a door that stopped collecting
# would read as nought percent, which is §77.6's shape.
#
# It writes each target's worst draw to `target/provenance-cost/` and names the
# path, because a count cannot be replayed and those bytes can.
[doc("What the provenance log costs a fuzz night: the §144.9 measurement.")]
provenance-cost:
    cargo bench -p rcad-step --bench what_the_provenance_log_costs_a_fuzz_night

# Which regions of `crates/*/src` the suite never runs.
#
# **Not in `gate`, and named `coverage` rather than `check-coverage` for that
# reason**: `check-gate-runs-every-check` says a check nobody runs is not a
# check, so a `check-` name would have to be paid on every commit. One run is
# about ten minutes and a `target/llvm-cov-target` of its own that shares
# nothing with the ordinary build, which is a thing to ask for. `bench` is out
# of the gate on the same argument and sits above.
#
# **It prints places and not a percentage.** The first run answered 94.0% and
# nobody could act on that; what the missing 6% *is* turned out to be the
# refusal half of this kernel — invariant panics, named `Error`s, and
# `Decision::Undecidable` arms at call sites nothing has ever reached. Every
# other check in this file pins a count of things rather than a ratio, and a
# ratio is the one number that moves for reasons nobody reads.
#
# The two costs worth knowing before you start it. Instrumentation is paid by
# the *suite*, not the build: 347 s against the 30.7 s critical path §95.5
# measures, one `rcad-step` test going from seconds to 342. And `llvm-tools` is
# in `rust-toolchain.toml` for this and nothing else.
[doc("Which regions of crates/*/src the suite never runs. Not in gate; ~10 minutes.")]
coverage:
    #!/usr/bin/env bash
    set -euo pipefail
    mkdir -p target
    cargo llvm-cov nextest --workspace --json --output-path target/coverage.json
    {{ just_executable() }} coverage-report target/coverage.json

# Read a coverage run back. `coverage` above calls this; it is separate so that
# the reading can be tested without a ten-minute run behind it, which is what
# `check-coverage-selftest` does.
#
# **Three things about llvm-cov's export make the naive read wrong, and two of
# them are silent.**
#
# A generic is exported once per instantiation and its regions are counted again
# each time, so the records have to be collapsed onto their spans: 199 081
# records over 35 368 real spans on this tree. Nothing about the un-collapsed
# number looks wrong.
#
# An inline `#[cfg(test)] mod tests` is ordinary lines of the file it sits in,
# and it is covered by construction — 42% of the region count here. The obvious
# filter is the function's name, and **that one fails silently**: names in the
# export are v0-mangled, so a test for `::tests::` matches nothing and keeps
# every one of them while looking like it worked. The first draft of this did
# exactly that and printed a confident "0 dropped". So the cut is geometric
# instead — the region's own file and start line against that file's first
# `#[cfg(test)]`.
#
# And `crates/*/tests/` needs no filter, because llvm-cov maps only `src` into
# its file summary. Measured, not assumed: the printed and src-only totals agree
# to the digit in all seven crates.
#
# **So it checks itself against the summary before it says anything.** The
# collapsed span count has to equal what llvm-cov's own per-file summary says,
# and a parse that drifts from that fails here rather than printing a plausible
# number. That identity is what catches the next version of the mangling bug.
[doc("Read a coverage run back: which regions of crates/*/src were never run.")]
coverage-report report:
    #!/usr/bin/env python3
    import json
    import re
    import sys
    from collections import defaultdict

    WHICH_CRATE = re.compile(r"/crates/([a-z0-9-]+)/")


    def main():
        with open("{{report}}") as handle:
            export = json.load(handle)["data"][0]

        widest = the_widest_count_for_each_span(export)
        say_so_if_the_summary_disagrees(export, widest)

        per_crate = defaultdict(lambda: [0, 0])
        per_file = defaultdict(lambda: [0, 0, []])
        cut_away = 0
        for (path, line, _, _, _), count in widest.items():
            starts_the_tests = where_the_tests_start(path)
            if starts_the_tests is not None and line >= starts_the_tests:
                cut_away += 1
                continue
            crate = per_crate[WHICH_CRATE.search(path).group(1)]
            here = per_file[path]
            crate[0] += 1
            here[0] += 1
            if count == 0:
                crate[1] += 1
                here[1] += 1
                here[2].append(line)

        print("never run, by crate")
        for crate in sorted(per_crate):
            total, missing = per_crate[crate]
            print(f"  {crate:<20} {missing:6d} of {total:6d}")

        print()
        print("never run, by file")
        worst = sorted(per_file.items(), key=lambda pair: -pair[1][1])
        for path, (total, missing, lines) in worst:
            if missing == 0:
                continue
            print(f"  {short(path):<44} {missing:5d} of {total:5d}  "
                  f"lines {as_runs(sorted(set(lines)))}")

        kept = sum(total for total, _ in per_crate.values())
        print()
        print(f"{kept} regions in crates/*/src, and {cut_away} more inside an inline")
        print("#[cfg(test)] module, which are covered by construction and were cut")
        print(f"{sum(1 for _ in every_record(export))} region records collapsed onto "
              f"{kept + cut_away} spans")


    def the_widest_count_for_each_span(export):
        """The same span is exported once per instantiation of a generic. Keep the
        highest count any of them reached: one instantiation running a branch is
        that branch having run."""
        widest = {}
        for span, count in every_record(export):
            if count > widest.get(span, -1):
                widest[span] = count
        return widest


    def every_record(export):
        for function in export["functions"]:
            paths = function["filenames"]
            for region in function["regions"]:
                line, column, last_line, last_column, count, which_file = region[:6]
                if region[7] != 0 or which_file >= len(paths):
                    continue
                path = paths[which_file]
                if WHICH_CRATE.search(path) is None or "/src/" not in path:
                    continue
                yield (path, line, column, last_line, last_column), count


    def say_so_if_the_summary_disagrees(export, widest):
        summed = 0
        for entry in export["files"]:
            path = entry["filename"]
            if WHICH_CRATE.search(path) is None or "/src/" not in path:
                continue
            summed += entry["summary"]["regions"]["count"]
        if summed != len(widest):
            sys.exit(
                f"llvm-cov's own summary counts {summed} regions in crates/*/src "
                f"and collapsing the records gave {len(widest)}.\n"
                "One of the two reads is wrong, and a number printed from here "
                "would look plausible either way.")


    read_so_far = {}


    def where_the_tests_start(path):
        """The line of the file's first `#[cfg(test)]`, or None if it has none."""
        if path not in read_so_far:
            found = None
            try:
                with open(path) as handle:
                    for number, text in enumerate(handle, 1):
                        if "#[cfg(test)]" in text:
                            found = number
                            break
            except OSError:
                found = None
            read_so_far[path] = found
        return read_so_far[path]


    def as_runs(lines):
        runs = []
        for line in lines:
            if runs and line - runs[-1][1] <= 3:
                runs[-1][1] = line
            else:
                runs.append([line, line])
        return ", ".join(str(a) if a == b else f"{a}-{b}" for a, b in runs)


    def short(path):
        return path.split("/crates/")[-1]


    main()

# `coverage-report` against a synthetic export, for each way the read can be
# wrong. It builds its own two-file tree under `mktemp` and writes its own JSON,
# so it needs no coverage run and sits in `gate` beside `check-corpus-selftest`.
#
# Verified by mutating the recipe above, one at a time, reverting between
# (docs/decisions.md §79's rule, applied to this):
#
# | Mutation | What failed |
# |---|---|
# | the collapse keeps the *last* count rather than the widest | the generic case, on a span that ran reading as never run |
# | `where_the_tests_start` always returns `None` | the cut case, on a line below the attribute being listed |
# | the `"/src/"` test dropped from `every_record` | the summary cross-check, before any output |
# | the cross-check's `sys.exit` deleted | the disagreement case, which then printed a plausible table |
#
# The third is why the cross-check is not merely a nicety: dropping the `src`
# filter is the shape of the mangling bug that already happened once, and the
# only thing that notices is the identity against llvm-cov's own summary.
[doc("coverage-report against a synthetic export, for each way the read can be wrong.")]
check-coverage-selftest:
    #!/usr/bin/env bash
    set -euo pipefail

    work=$(mktemp -d -- "${TMPDIR:-/tmp}/rcad-coverage-selftest.XXXXXX")
    trap 'rm -rf -- "$work"' EXIT

    mkdir -p "$work/crates/thing/src"
    # Twelve lines of kernel, then the inline tests. Written with printf because
    # it is a throwaway fixture under mktemp and not a tracked file.
    for line in $(seq 1 11); do printf 'fn kernel_%s() {}\n' "$line" >>"$work/crates/thing/src/thing.rs"; done
    printf '#[cfg(test)]\nmod tests {\n    fn a_test() {}\n}\n' >>"$work/crates/thing/src/thing.rs"

    thing="$work/crates/thing/src/thing.rs"

    # A region is [line, col, last_line, last_col, count, file, expanded, kind].
    # kind 0 is a code region; the summary count must match the collapsed spans.
    export_of() {
        printf '{"data":[{"files":[{"filename":"%s","summary":{"regions":{"count":%s,"covered":0},"lines":{"count":1,"covered":1},"functions":{"count":1,"covered":1}}}],"functions":[%s],"totals":{}}]}' \
            "$thing" "$2" "$3" >"$work/$1.json"
    }

    failed=0
    check() {
        local name=$1 want_rc=$2
        shift 2
        local out rc needle
        set +e
        out=$({{ just_executable() }} coverage-report "$work/$name.json" 2>&1)
        rc=$?
        set -e
        if [ "$rc" != "$want_rc" ]; then
            echo "FAIL $name: exit $rc, expected $want_rc"
            echo "$out" | sed 's/^/    /'
            failed=1
            return
        fi
        for needle in "$@"; do
            case "$needle" in
            !*)
                if grep -qF -- "${needle#!}" <<<"$out"; then
                    echo "FAIL $name: output says '${needle#!}' and must not"
                    echo "$out" | sed 's/^/    /'
                    failed=1
                fi
                ;;
            *)
                if ! grep -qF -- "$needle" <<<"$out"; then
                    echo "FAIL $name: output does not say '$needle'"
                    echo "$out" | sed 's/^/    /'
                    failed=1
                fi
                ;;
            esac
        done
    }

    # One: the geometric cut. Line 3 is kernel and never run, line 14 is inside
    # `mod tests` and never run. Only the first is anybody's business.
    export_of cut 2 '{"name":"_RNvCs0_5thing6kernel","count":0,"filenames":["'"$thing"'"],"regions":[[3,1,3,9,0,0,0,0],[14,1,14,9,0,0,0,0]]}'
    check cut 0 "lines 3" '!lines 3, 14' "and 1 more inside an inline"

    # Two: a generic exported twice, never run first and run second. Collapsing
    # on the widest count is what makes this a reached region; keeping the last
    # or the first record read would answer differently, and one of those two
    # orders would look right.
    export_of generic 1 '{"name":"_RNvCs0_5thing1a","count":0,"filenames":["'"$thing"'"],"regions":[[5,1,5,9,0,0,0,0]]},{"name":"_RNvCs0_5thing1b","count":7,"filenames":["'"$thing"'"],"regions":[[5,1,5,9,7,0,0,0]]}'
    check generic 0 "never run, by file" '!thing.rs' "1 regions in crates/*/src"

    # Three: the same two records the other way about. §79.2's lesson — a
    # mutation that leaves one order right can leave the other wrong.
    export_of generic-reversed 1 '{"name":"_RNvCs0_5thing1b","count":7,"filenames":["'"$thing"'"],"regions":[[5,1,5,9,7,0,0,0]]},{"name":"_RNvCs0_5thing1a","count":0,"filenames":["'"$thing"'"],"regions":[[5,1,5,9,0,0,0,0]]}'
    check generic-reversed 0 '!thing.rs' "1 regions in crates/*/src"

    # Four: a region of a kind other than code, and a region in a file the
    # summary does not describe. Both are dropped, so the summary still agrees.
    export_of kinds 1 '{"name":"_RNvCs0_5thing1a","count":0,"filenames":["'"$thing"'"],"regions":[[6,1,6,9,0,0,0,0],[7,1,7,9,0,0,0,2]]},{"name":"_RNvCs0_5thing1c","count":0,"filenames":["'"$work"'/crates/thing/tests/a.rs"],"regions":[[1,1,1,9,0,0,0,0]]}'
    check kinds 0 "lines 6" '!lines 6, 7' '!tests/a.rs'

    # Five: the cross-check itself. The summary says three regions and the
    # records collapse to one, which is the shape every parse bug here takes,
    # and it must refuse rather than print a table nobody can tell is wrong.
    export_of disagreement 3 '{"name":"_RNvCs0_5thing1a","count":0,"filenames":["'"$thing"'"],"regions":[[8,1,8,9,0,0,0,0]]}'
    check disagreement 1 "llvm-cov's own summary counts 3 regions" '!never run, by crate'

    if [ "$failed" != "0" ]; then
        echo "^ coverage-report read a synthetic export wrongly"
        exit 1
    fi
    echo "coverage-report: five synthetic exports read as they should be"

[doc("Format the Rust and the TOML in place.")]
fmt:
    cargo fmt --all
    taplo fmt

# Strip `target/` from every worktree git knows about. Always safe: kache holds
# the compilation, so a cleaned worktree refills from the cache instead of
# rebuilding, and nothing a lane wrote can be lost this way.
#
# Separate from `prune-worktrees` on purpose (docs/decisions.md §50.8). That one
# can only run once the work is merged; this one can run any time, and merging
# the two would make the safe half inherit the unsafe half's ceremony until
# nobody ran either. 38 worktrees held 166 GB of `target/` before this existed.
#
# **Asked git rather than guessed from a path.** This used to build the string
# `$(dirname $(git rev-parse --show-toplevel))/rcad-worktrees`, which is only the
# right answer when it is run from the hub — run from a lane it landed on
# `rcad-worktrees/rcad-worktrees`, found nothing there, and said *"there is
# nothing here to clean"*, which reads exactly like a tidy machine. Now that
# nobody works in the hub (§60) that was every run. The listing also reaches the
# sibling trees the glob never covered and the hub's own `target/`, which was the
# single largest at 14 GB.
#
# Two are skipped, and both for the same reason as `prune-worktrees`: the tree
# you are standing in, because you are using it, and any **locked** tree, because
# a lock means an agent is working in there and deleting its `target/` mid-build
# is exactly the interruption the lock exists to prevent.
[doc("Strip target/ from every worktree git knows about. Skips locked ones.")]
clean-targets dir=".":
    #!/usr/bin/env bash
    set -euo pipefail
    listing=$(git -C "{{dir}}" worktree list --porcelain)
    here=$(git -C "{{dir}}" rev-parse --show-toplevel)
    locked=$(awk '/^worktree /{tree = $2} /^locked/{print tree}' <<<"$listing")
    reclaimed=0
    kept=0
    for tree in $(awk '/^worktree /{print $2}' <<<"$listing"); do
        [ -d "$tree" ] || continue
        if [ "$tree" = "$here" ]; then
            echo "kept (you are standing in it): $tree"
            kept=$((kept + 1))
            continue
        fi
        if grep -qxF "$tree" <<<"$locked"; then
            echo "kept (locked, an agent is using it): $tree"
            kept=$((kept + 1))
            continue
        fi
        targets=$(find "$tree" -maxdepth 2 -type d -name target -prune)
        [ -n "$targets" ] || continue
        size=$(du -sb --files0-from=<(tr '\n' '\0' <<<"$targets") --total | awk '/total$/{print $1}')
        find "$tree" -maxdepth 2 -type d -name target -prune -exec rm -rf {} +
        echo "swept $(numfmt --to=iec "$size"): $tree"
        reclaimed=$((reclaimed + size))
    done
    echo "$(numfmt --to=iec "$reclaimed") reclaimed; $kept worktree(s) kept"

# Pins the two trees `clean-targets` must not touch, and the one it used to miss.
#
# A sweep that deletes too much is noticed within the minute; a sweep that
# deletes too little says "nothing to clean" and is believed for a month, which
# is what happened. So the fixture puts a `target/` in all three trees and runs
# the sweep from the lane: the **hub's** must go, because it is the biggest and
# the old path glob never reached it, the **locked** one must survive, and the
# tree the sweep is standing in must survive.
[private]
check-clean-targets-selftest:
    #!/usr/bin/env bash
    set -euo pipefail
    dir=$(mktemp -d)
    trap 'git -C "$dir/main" worktree unlock "$dir/held" 2>/dev/null || true; rm -rf "$dir"' EXIT
    git init -q -b main "$dir/main"
    git -C "$dir/main" -c user.email=t@t -c user.name=t commit -q --allow-empty -m root
    git -C "$dir/main" worktree add -q "$dir/held" -b held
    git -C "$dir/main" worktree add -q "$dir/free" -b free
    git -C "$dir/main" worktree lock --reason "agent-abc since 2026-09-06T00:00:00Z" "$dir/held"
    for tree in main held free; do
        mkdir -p "$dir/$tree/target/debug"
        head -c 4096 /dev/zero >"$dir/$tree/target/debug/artifact"
    done
    got=$(just clean-targets "$dir/free")
    fail() {
        echo "clean-targets drifted from its own fixtures: $1"
        echo "--- got ---"; echo "$got"
        exit 1
    }
    [ ! -d "$dir/main/target" ] || fail "the hub's target/ survived, which is the case the old path glob missed"
    [ -d "$dir/held/target" ] || fail "a locked worktree was swept while an agent was working in it"
    [ -d "$dir/free/target" ] || fail "the sweep deleted the target/ of the tree it was standing in"
    grep -qF "kept (locked, an agent is using it): $dir/held" <<<"$got" \
        || fail "the locked worktree was not reported as kept"
    grep -qF "kept (you are standing in it): $dir/free" <<<"$got" \
        || fail "the current worktree was not reported as kept"
    grep -qE "^[0-9.]+K reclaimed; 2 worktree\(s\) kept$" <<<"$got" \
        || fail "the tally is wrong, so a sweep that found nothing would read like a tidy machine"

# Remove lane worktrees whose branch has landed on main.
#
# A worktree is only removed when it is clean *and* its branch is an ancestor of
# main; anything else is listed and left alone. The branch itself is never
# deleted, so a spike that is never merged by design (§50.4) keeps its commits
# even after its worktree goes.
#
# The closing gate runs this, because a merge is the only moment when removing a
# worktree is provably safe, and this repo learned the hard way that nothing
# reclaims them on its own: the harness sweep keeps any worktree holding unpushed
# commits, so with no remote configured it kept all 38 for ever (§50.8).
#
# Two defects, both demonstrated before either fix was written (§76). First,
# `main` is an ancestor of itself, so the tree holding it used to fall through
# to `remove` like any merged lane — this actually happened on 2026-09-08. It
# gets its own check, ahead of the ancestry test, because §60 says `main` lives
# in a worktree and never the hub; this recipe honoring that is not optional.
# Second, a detached tree reports its branch as the literal string `HEAD`, and
# resolving that string as a ref used to go through whichever tree the command
# was standing in — from the tree holding `main`, `HEAD` resolved to `main`
# itself, trivially an ancestor, so a clean detached tree was removed no matter
# what it held. Its commits sit on no branch, so nothing else protects them: a
# detached tree is now judged by its own commit (`git -C "$tree" rev-parse
# HEAD`, no branch name involved at all) and kept unconditionally, the same way
# a locked or dirty tree is.
[doc("Remove lane worktrees whose branch has landed on main.")]
prune-worktrees dir=".":
    #!/usr/bin/env bash
    set -euo pipefail
    repo=$(git -C "{{dir}}" rev-parse --show-toplevel)
    kept=0
    # A lock means an agent is working in there right now — git holds one for the
    # life of the agent and releases it on exit. Never force past it: the pid in
    # the lock reason was a live 13-hour session the first time this ran.
    locked=$(git -C "$repo" worktree list --porcelain \
        | awk '/^worktree /{tree = $2} /^locked/{print tree}')
    for tree in $(git -C "$repo" worktree list --porcelain | awk '/^worktree /{print $2}'); do
        [ "$tree" = "$repo" ] && continue
        branch=$(git -C "$tree" rev-parse --abbrev-ref HEAD 2>/dev/null || echo "")
        commit=$(git -C "$tree" rev-parse HEAD 2>/dev/null || echo "")
        name="$branch"
        [ "$branch" = "HEAD" ] && name="detached at $(git -C "$tree" rev-parse --short HEAD 2>/dev/null || echo "$commit")"
        if grep -qxF "$tree" <<<"$locked"; then
            echo "kept (locked, an agent is using it): $tree"
            kept=$((kept + 1))
        elif [ -n "$(git -C "$tree" status --porcelain 2>/dev/null)" ]; then
            echo "kept (uncommitted changes): $tree"
            kept=$((kept + 1))
        elif [ "$branch" = "main" ]; then
            echo "kept (holds main, which a worktree carries by rule, never the hub -- §60): $tree"
            kept=$((kept + 1))
        elif [ "$branch" = "HEAD" ]; then
            echo "kept (detached; its commits are on no branch, so nothing else protects them): $tree [$name]"
            kept=$((kept + 1))
        elif [ -z "$commit" ] || ! git -C "$repo" merge-base --is-ancestor "$commit" main 2>/dev/null; then
            echo "kept (not merged into main): $tree [$name]"
            kept=$((kept + 1))
        else
            git -C "$repo" worktree remove --force "$tree"
            echo "removed: $tree [$name]"
        fi
    done
    git -C "$repo" worktree prune
    echo "$kept worktree(s) kept; their branches are untouched either way"

# Pins the two defects fixed above (§76) — `prune-worktrees` had no selftest at
# all, which is exactly how both survived to be found live.
#
# The fixture builds five worktrees off a throwaway `main` and runs the sweep
# from a sixth, so `main`'s own tree is judged rather than skipped as "the tree
# you are standing in". `held` is locked, `dirty` carries an uncommitted file,
# `loose` is detached, and `merged` is fast-forwarded into `main` before the
# sweep runs, so its commit really is `main`'s ancestor. A guard that kept
# everything would pass the first four and only the fifth catches it.
[private]
check-prune-worktrees-selftest:
    #!/usr/bin/env bash
    set -euo pipefail
    dir=$(mktemp -d)
    trap 'git -C "$dir/main" worktree unlock "$dir/held" 2>/dev/null || true; rm -rf "$dir"' EXIT
    git init -q -b main "$dir/main"
    git -C "$dir/main" -c user.email=t@t -c user.name=t commit -q --allow-empty -m root
    git -C "$dir/main" worktree add -q "$dir/runner" -b runner
    git -C "$dir/main" worktree add -q "$dir/held" -b held
    git -C "$dir/main" worktree add -q "$dir/merged" -b merged
    git -C "$dir/main" worktree add -q "$dir/dirty" -b dirty
    git -C "$dir/main" worktree add -q --detach "$dir/loose"
    git -C "$dir/merged" -c user.email=t@t -c user.name=t commit -q --allow-empty -m "landed work"
    git -C "$dir/main" merge -q merged
    echo change >"$dir/dirty/uncommitted.txt"
    git -C "$dir/main" worktree lock --reason "agent-abc since 2026-09-06T00:00:00Z" "$dir/held"
    got=$(just prune-worktrees "$dir/runner")
    fail() {
        echo "prune-worktrees drifted from its own fixtures: $1"
        echo "--- got ---"; echo "$got"
        exit 1
    }
    [ -d "$dir/main" ] || fail "the worktree holding main was removed"
    [ -d "$dir/held" ] || fail "a locked worktree was removed while an agent was using it"
    [ -d "$dir/dirty" ] || fail "a worktree with uncommitted changes was removed"
    [ -d "$dir/loose" ] || fail "a clean detached worktree was removed"
    [ ! -d "$dir/merged" ] || fail "a clean lane branch merged into main was not removed"
    grep -qF "kept (holds main" <<<"$got" \
        || fail "the tree holding main was not given its own kept reason"
    grep -qF "kept (locked, an agent is using it): $dir/held" <<<"$got" \
        || fail "the locked worktree was not reported as kept"
    grep -qF "kept (uncommitted changes): $dir/dirty" <<<"$got" \
        || fail "the dirty worktree was not reported as kept"
    grep -qF "kept (detached" <<<"$got" \
        || fail "the detached worktree was not reported as kept"
    grep -qF "removed: $dir/merged" <<<"$got" \
        || fail "a merged lane branch was not reported as removed"

# Where this repo stands, in one call.
#
# Collecting that from cold took twelve tool calls and a lookup lane on
# 2026-09-14, and every ingredient was already here and simply never assembled:
# `lanes` has who holds what, `gh` has the last green, `decisions-toc` has how
# far the record has got. The one fact nothing printed is the one that took
# longest — which *branches* carry something `main` does not — because `lanes`
# lists worktrees, and a worktree is not a branch.
#
# **Five rows, and the first of them is the one a recipe cannot compute.** What
# unit the work is in is a judgement, so it is written down in
# `docs/decisions.md` §0 and this prints it; §102 argues why there and not in a
# file of its own. The other four are facts, and they are read fresh every time.
#
# **It never fails.** A row that cannot be answered says so in place of its
# answer — the shape `fuzz-watermark` already has, and for the same reason: a
# survey that exits non-zero because `gh` was not on PATH is a survey nobody
# runs before deciding what to do next.
[doc("Where the work is, who holds what, and what is not landed on main yet.")]
status dir=".":
    #!/usr/bin/env bash
    set -uo pipefail

    echo "=== where the work is (docs/decisions.md §0) ==="
    block=$({{ just_executable() }} decision 0 2>&1) || block="docs/decisions.md has no §0; \`just check-where-the-work-is\` says what it should hold"
    echo "$block"
    reviewed=$(sed -n 's/^reviewed: *//p' <<<"$block" | head -1)
    if [ -n "$reviewed" ] && stamp=$(date -d "$reviewed" +%s 2>/dev/null); then
        echo "(last reviewed $(( ( $(date +%s) - stamp ) / 86400 )) days ago)"
    fi

    echo
    echo "=== the board ==="
    {{ just_executable() }} _plan-summary
    echo "(\`just plan\` for the roadmap with a box beside every item, \`just plan open\` for what is left)"

    echo
    echo "=== lanes ==="
    {{ just_executable() }} lanes '{{dir}}'

    echo
    echo "=== branches carrying work main does not have ==="
    {{ just_executable() }} _status-branches '{{dir}}' main

    echo
    echo "=== main against the last green ==="
    {{ just_executable() }} _status-green '{{dir}}' main

    echo
    echo "=== docs/decisions.md ==="
    {{ just_executable() }} decisions-toc | awk '
        { n = $1 + 0; if (n > top) top = n; seen[n] = 1; count += 1 }
        END {
            if (!count) { print "the toc printed nothing, so this counted no sections"; exit }
            gaps = ""
            for (i = 1; i < top; i++) if (!(i in seen)) gaps = gaps (gaps ? ", " : "") i
            printf "%d sections, highest is %d%s\n", count, top, \
                (gaps ? "; no section " gaps " — a gap is normal, a number is picked against every branch" : "")
        }'

plan_file := "docs/plan.md"
roadmap_file := "docs/roadmap.md"

# The roadmap with a box beside every item, and what fills each box.
#
# **`docs/roadmap.md` may not carry this and never will.** §1 of
# `docs/decisions.md` says it is the north star and not a description of this
# codebase — that is why it has one commit in the repository's whole history,
# and why a later reader is told not to "fix" the code back to match it. So the
# state goes beside it rather than in it, and this recipe renders the two
# together. `docs/decisions.md` §105.
#
# **`docs/plan.md` holds no titles.** It is a line per item: the roadmap number,
# a state, and the evidence. The titles come from the roadmap at render time, so
# there is exactly one copy of them and `plan.md` cannot drift into describing a
# roadmap that has been rewritten. The item list is not stored either — it is
# derived, and `check-plan` fails when the two disagree.
#
# `just plan open` leaves out what is finished, which is the list lanes are cut
# from.
[doc("The roadmap with a box beside every item: `just plan`, or `just plan open`.")]
plan filter="":
    #!/usr/bin/env bash
    set -uo pipefail
    {{ just_executable() }} _plan-items '{{ roadmap_file }}' | awk -F'\t' -v filter='{{filter}}' '
        FILENAME == "-" || started { }
        { items[$1] = $2; section[$1] = $3; order[++n] = $1 }
        END {
            while ((getline line < "{{ plan_file }}") > 0) {
                if (line !~ /^[0-9]/) continue
                split(line, f, /[ \t]+/)
                id = f[1]; state[id] = f[2]
                sub(/^[^ \t]+[ \t]+[^ \t]+[ \t]*/, "", line); why[id] = line
            }
            for (i = 1; i <= n; i++) {
                id = order[i]
                s = (id in state) ? state[id] : "unaudited"
                if (filter == "open" && (s == "done" || s == "n/a")) continue
                if (section[id] != lastsection) {
                    printf "\n%s\n", section[id]
                    lastsection = section[id]
                }
                printf "  [%s] %-5s %-10s %s\n", (s == "done" || s == "n/a") ? "x" : " ", id, s, items[id]
                if (why[id] != "") printf "              %s\n", why[id]
            }
        }'
    echo
    {{ just_executable() }} _plan-summary

# The counts, on one line, so `status` can carry the plan without printing it.
[private]
_plan-summary:
    #!/usr/bin/env bash
    set -uo pipefail
    {{ just_executable() }} _plan-items '{{ roadmap_file }}' | awk -F'\t' '
        { order[++n] = $1 }
        END {
            while ((getline line < "{{ plan_file }}") > 0) {
                if (line !~ /^[0-9]/) continue
                split(line, f, /[ \t]+/); state[f[1]] = f[2]
            }
            for (i = 1; i <= n; i++) {
                id = order[i]
                count[(id in state) ? state[id] : "unaudited"] += 1
            }
            printf "plan: %d items — %d done, %d partly, %d todo, %d unaudited, %d n/a\n", \
                n, count["done"], count["partly"], count["todo"], \
                count["unaudited"], count["n/a"]
        }'

# The roadmap's build items, derived rather than listed anywhere.
#
# An item is a **deepest heading under §3 to §11** — the layers and the testing
# apparatus. §1 and §2 are scope and foundational argument, §12 to §15 are the
# milestone list, the risk register and the reading, and none of those is a
# thing that gets built. A `## N.` with `### N.M` children contributes the
# children; one without contributes itself, which is how §3 and §8 get a line.
#
# Deriving it is the point. A stored list is a second copy of the roadmap's
# shape, and §60's argument applies to a shape exactly as it applies to a
# record: the copy is what goes stale when a section is added.
[private]
_plan-items file:
    #!/usr/bin/env bash
    set -uo pipefail
    awk -F'\n' '
        /^## [0-9]+\./ {
            flush()
            rest = substr($0, 4); p = index(rest, " ")
            top = substr(rest, 1, p - 1); sub(/\.$/, "", top)
            title = substr(rest, p + 1)
            want = (top + 0 >= 3 && top + 0 <= 11)
            heading = "roadmap §" top "  " title
            kids = 0
            next
        }
        /^### [0-9]+\.[0-9]+ / {
            if (!want) next
            kids = 1
            rest = substr($0, 5); p = index(rest, " ")
            printf "%s\t%s\t%s\n", substr(rest, 1, p - 1), substr(rest, p + 1), heading
        }
        END { flush() }
        function flush() {
            if (want && !kids && top != "") printf "%s\t%s\t%s\n", top, title, heading
        }' '{{file}}'

# `docs/plan.md` still describes `docs/roadmap.md`, and says nothing it cannot
# show you.
#
# **The item lists must agree exactly.** A roadmap gaining a subsection that
# `plan.md` never hears about is the whole failure this file could otherwise
# have: it would go on reading complete while covering a roadmap that has moved.
# So the list is derived on both sides of the comparison and a difference in
# either direction fails, naming the numbers.
#
# **A box is ticked only against evidence that exists.** `done` and `n/a` are
# the two states that tick, and a `done` whose evidence names a path under
# `crates/` or `docs/` is checked against the disk. That is §15.6's rule — a
# published bound needs a test — applied to a published checkbox. It cannot
# check that the evidence *proves* the item, which is why the audit that fills
# this file names a module or a test rather than an opinion.
[doc("docs/plan.md still covers docs/roadmap.md, and every ticked box names something real.")]
check-plan:
    #!/usr/bin/env bash
    set -uo pipefail
    {{ just_executable() }} _plan-check '{{ roadmap_file }}' '{{ plan_file }}'

# Held apart from the recipe above so the selftest can run it over fixtures
# whose answer is known.
[private]
_plan-check roadmap plan:
    #!/usr/bin/env bash
    set -uo pipefail
    roadmap='{{roadmap}}'
    plan='{{plan}}'
    [ -f "$plan" ] || { echo "$plan does not exist, so nothing records how far the roadmap has got"; exit 1; }

    wanted=$({{ just_executable() }} _plan-items "$roadmap" | cut -f1 | sort -u)
    [ -n "$wanted" ] || { echo "no build items were parsed out of $roadmap, so this compared nothing"; exit 1; }
    got=$(grep -E '^[0-9]' "$plan" | awk '{ print $1 }' | sort -u)

    missing=$(comm -23 <(echo "$wanted") <(echo "$got") | tr '\n' ' ')
    extra=$(comm -13 <(echo "$wanted") <(echo "$got") | tr '\n' ' ')
    if [ -n "${missing// }" ] || [ -n "${extra// }" ]; then
        [ -z "${missing// }" ] || echo "in $roadmap but not in $plan: $missing"
        [ -z "${extra// }" ] || echo "in $plan but not in $roadmap: $extra"
        echo "^ the plan no longer covers the roadmap, so its completeness means nothing"
        exit 1
    fi

    dupes=$(grep -E '^[0-9]' "$plan" | awk '{ print $1 }' | sort | uniq -d | tr '\n' ' ')
    if [ -n "${dupes// }" ]; then
        echo "$plan lists these items twice, and the second wins silently: $dupes"
        exit 1
    fi

    bad=0
    while read -r id state rest; do
        case "$state" in
        done|partly|todo|unaudited|n/a) ;;
        *) echo "$id: '$state' is not a state; they are done, partly, todo, unaudited, n/a"; bad=1; continue ;;
        esac
        if [ -z "$rest" ]; then
            echo "$id: state '$state' with no evidence after it"
            bad=1
            continue
        fi
        [ "$state" = "done" ] || continue
        for token in $(tr ',;' '  ' <<<"$rest"); do
            case "$token" in
            crates/*|docs/*|fuzz/*) [ -e "${token%%,}" ] || { echo "$id is done on '$token', which is not in the tree"; bad=1; } ;;
            esac
        done
    done < <(grep -E '^[0-9]' "$plan")
    [ "$bad" -eq 0 ] || { echo "^ a ticked box naming something that is not there is worse than an empty one"; exit 1; }

# Pins the derivation and the comparison against fixtures, because the live
# files cannot exercise either: they agree today by construction, so a
# `_plan-check` that had stopped comparing would pass every run.
#
# The fixtures are the four ways this goes wrong — a roadmap that grew an item,
# a plan that invented one, a box ticked against a path that is not there, and a
# state nobody defined — plus a good pair, because a check that rejects
# everything passes every negative case.
[private]
check-plan-selftest:
    #!/usr/bin/env bash
    set -uo pipefail
    dir=$(mktemp -d)
    trap 'rm -rf "$dir"' EXIT
    fail() {
        echo "plan drifted from its own fixtures: $1"
        echo "--- got ---"; echo "$got"
        exit 1
    }

    cat > "$dir/roadmap.md" <<'MD'
    # A roadmap
    ## 1. Scope
    ### 1.1 Not a build item
    ## 3. Layer 0
    ## 4. Layer 1
    ### 4.1 First
    ### 4.2 Second — with an em-dash in the title
    ## 12. Milestones
    ### 12.1 Also not a build item
    MD
    sed -i 's/^    //' "$dir/roadmap.md"

    got=$(just _plan-items "$dir/roadmap.md" | cut -f1 | tr '\n' ' ')
    [ "$got" = "3 4.1 4.2 " ] \
        || fail "the item list is '$got', expected '3 4.1 4.2 ' — §1 and §12 are not build items, and §3 has no children"

    printf '3 done crates\n4.1 todo nothing yet\n4.2 n/a prose only\n' > "$dir/plan.md"
    got=$(just _plan-check "$dir/roadmap.md" "$dir/plan.md" 2>&1) \
        || fail "a plan that covers the roadmap was rejected"

    printf '3 done crates\n4.1 todo nothing yet\n' > "$dir/short.md"
    got=$(just _plan-check "$dir/roadmap.md" "$dir/short.md" 2>&1) \
        && fail "a plan missing an item the roadmap has was accepted"
    grep -qF "but not in $dir/short.md: 4.2" <<<"$got" \
        || fail "the missing item was not named"

    printf '3 done crates\n4.1 todo x\n4.2 todo x\n9.9 todo invented\n' > "$dir/extra.md"
    got=$(just _plan-check "$dir/roadmap.md" "$dir/extra.md" 2>&1) \
        && fail "a plan naming an item the roadmap does not have was accepted"
    grep -qF "9.9" <<<"$got" || fail "the invented item was not named"

    printf '3 done crates/there-is-no-such-directory\n4.1 todo x\n4.2 todo x\n' > "$dir/liar.md"
    got=$(just _plan-check "$dir/roadmap.md" "$dir/liar.md" 2>&1) \
        && fail "a ticked box naming a path that is not in the tree was accepted"
    grep -qF "is not in the tree" <<<"$got" || fail "the absent path was not named"

    printf '3 finished crates\n4.1 todo x\n4.2 todo x\n' > "$dir/state.md"
    got=$(just _plan-check "$dir/roadmap.md" "$dir/state.md" 2>&1) \
        && fail "an undefined state was accepted"

    printf '3 done\n4.1 todo x\n4.2 todo x\n' > "$dir/bare.md"
    got=$(just _plan-check "$dir/roadmap.md" "$dir/bare.md" 2>&1) \
        && fail "an item with a state and no evidence was accepted"

    # As in check-status-selftest: every assertion above is `command && fail`,
    # so the last one leaves its own non-zero status behind when it passes.
    exit 0

# The branch half of `status`, kept apart so `check-status-selftest` can hold it
# against a repository whose right answer is known.
#
# **`git cherry` and not `rev-list main..<branch>`.** Ahead-of-main counts what
# is reachable; what a reader wants is what is *unlanded*, and the two disagree
# on every branch whose commits reached `main` by a cherry-pick, a rebase or a
# split into two commits. On 2026-09-14 that was three of the five branches the
# count called unmerged — one of them `merge/justfile`, whose recipe had been on
# `main` for days. `cherry` compares patch ids and marks those `-`, so counting
# only its `+` lines turns a five-branch answer into a two-branch one without a
# human reading five diffs. It skips merge commits, which is the same thing
# wanted here: a lane merging `main` into itself has landed nothing.
#
# **A branch with nothing new is counted, not listed.** The count is what says
# the sweep ran; listing twenty-three names nobody has to act on is how a survey
# stops being read.
[private]
_status-branches dir base:
    #!/usr/bin/env bash
    set -uo pipefail
    repo=$(git -C '{{dir}}' rev-parse --show-toplevel) || exit 1
    if ! git -C "$repo" rev-parse --verify --quiet '{{base}}^{commit}' >/dev/null; then
        echo "there is no branch named {{base}} here, so nothing could be weighed against it"
        exit 1
    fi

    trees=$(git -C "$repo" worktree list --porcelain \
        | awk '/^worktree /{t=substr($0,10)} /^branch /{print substr($0,19) "\t" t}')

    branches=$(git -C "$repo" for-each-ref --format='%(refname:short)' refs/heads/)
    if [ -z "$branches" ]; then
        echo "for-each-ref printed no branches, so this counted nothing"
        exit 1
    fi

    carrying=0
    quiet=0
    while IFS= read -r branch; do
        [ "$branch" != '{{base}}' ] || continue
        new=$(git -C "$repo" cherry '{{base}}' "$branch" 2>/dev/null | grep -c '^+')
        if [ "$new" -eq 0 ]; then
            quiet=$((quiet + 1))
            continue
        fi
        carrying=$((carrying + 1))
        where=$(awk -F'\t' -v b="$branch" '$1 == b { print $2 }' <<<"$trees")
        printf '%-46s %2d commit(s)  %s\n' "$branch" "$new" "${where:-no worktree}"
    done <<<"$branches"

    if [ "$carrying" -eq 0 ]; then
        echo "nothing: every branch here is already on {{base}}"
    fi
    echo "$quiet branch(es) carry nothing {{base}} does not already have"

# The green half of `status`, kept apart for the same reason as the branch half:
# the live repository's answer is whatever happened to it this week, so the ways
# this row goes wrong can only be pinned against a fixture.
#
# **Two sources, because since §172 one of them can no longer answer.** The
# Actions allowance is spent, so no push to `main` can go green on the runner,
# and a row that asks only GitHub reports a tip drifting further from a green
# that cannot move — 22 commits within a day, while a full `just gate` had in
# fact read the tip end to end. That is the same staleness §173.9 fixed for
# `fuzz-watermark`, and this was the other place it showed: the row a session
# reads first. `_record-green` writes each local gate down; this reads that file
# beside `gh`, and §173.9's rule about what may be written there — every metered
# step ran, on a clean tree — is what makes the two comparable at all.
#
# **It says which source answered, and that is the point rather than a detail.**
# A CI green is a different machine; a local green is *this* machine, the one
# §173 spent a day finding four environment-dependent faults in, so the two are
# not the same evidence and a row that blurred them would be the more expensive
# error now that almost every green is local. §173.9 left that caveat owed; this
# is where it gets paid, next to the number it qualifies.
[private]
_status-green dir base:
    #!/usr/bin/env bash
    set -uo pipefail

    file=""
    common=$(git -C '{{dir}}' rev-parse --path-format=absolute --git-common-dir 2>/dev/null) \
        && file="$common/rcad-greens"

    # Both sources normalised to full shas in *this* checkout, so that the
    # source test below is a string compare it cannot get subtly wrong, and so
    # a sha naming no commit here drops out before it can be chosen.
    keep() {
        for sha in $1; do
            git -C '{{dir}}' rev-parse --verify --quiet "${sha}^{commit}" 2>/dev/null
        done
    }
    raw_ci=""
    if command -v gh >/dev/null 2>&1; then
        raw_ci=$(gh run list --workflow ci.yml --branch '{{base}}' --status success \
            --limit 20 --json headSha --jq '.[].headSha' 2>/dev/null) || raw_ci=""
    fi
    raw_local=$(cut -d' ' -f1 "$file" 2>/dev/null) || raw_local=""
    ci=$(keep "$raw_ci")
    local_greens=$(keep "$raw_local")

    if [ -z "$raw_ci$raw_local" ]; then
        if command -v gh >/dev/null 2>&1; then
            echo "unknown: neither gh nor the local gate record names a green commit on {{base}}"
        else
            echo "unknown: gh is not on PATH and no local gate has been recorded"
        fi
        exit 0
    fi

    # The furthest forward of the candidates, not the first that qualifies: two
    # sources merged arrive in no useful order, so the walk establishes it here.
    best=""
    for sha in $ci $local_greens; do
        git -C '{{dir}}' merge-base --is-ancestor "$sha" '{{base}}' 2>/dev/null || continue
        if [ -z "$best" ] || git -C '{{dir}}' merge-base --is-ancestor "$best" "$sha" 2>/dev/null; then
            best=$sha
        fi
    done
    if [ -z "$best" ]; then
        echo "no green commit is on {{base}} in this checkout; fetch, or gate it here"
        exit 0
    fi

    case " $ci " in *" $best "*) from_ci=yes ;; *) from_ci=no ;; esac
    case " $local_greens " in *" $best "*) from_local=yes ;; *) from_local=no ;; esac
    if [ "$from_ci" = yes ] && [ "$from_local" = yes ]; then
        how="CI and a local gate"
    elif [ "$from_ci" = yes ]; then
        how="CI"
    else
        how="a local gate$(awk -v s="$best" '$1 == s { printf " on %s, %s", $3, $2 }' "$file" 2>/dev/null)"
    fi

    ahead=$(git -C '{{dir}}' rev-list --count "$best"..'{{base}}' 2>/dev/null)
    printf '%s tip %s, last green %s (%s), %s commit(s) on %s not yet green\n' \
        '{{base}}' "$(git -C '{{dir}}' rev-parse --short '{{base}}')" \
        "$(git -C '{{dir}}' rev-parse --short "$best")" "$how" "${ahead:-?}" '{{base}}'
    [ "$from_ci" = yes ] \
        || echo "  nothing independent has read it: a local gate is this machine, §173 and §173.9"

# §0 is the one row of `status` that a recipe cannot compute, so this is what
# stands behind it. It cannot check the judgement — whether the unit named there
# is still the unit — and it does not pretend to. It checks the three ways the
# block goes wrong without anyone noticing.
#
# **A cited section that does not exist.** §0 points at the sections that defer
# the work it lists, and those numbers get renumbered (§71 is four such moves in
# one session). A block pointing at a §43.8 that has become §44.8 is worse than
# no block, because it reads as current.
#
# **A `reviewed:` date that does not parse.** `status` prints how old the block
# is, and a date `date -d` cannot read makes that line silently vanish — which
# looks exactly like a block reviewed today.
#
# **A block that has grown into an essay.** The cap is the whole reason this
# stays true: `decisions.md` is 27 000 lines because nothing ever capped it, and
# that is correct for a record and fatal for a summary. Fifty lines is about a
# screen, which is the length something gets re-read at. The refusal names how
# many lines to cut, because the 21 trims of 2026-09-17..20 took 34 rounds
# between them — 67 to 56 to 54 to 51 is one of them — and a trim by eye aims
# short.
#
# **It reads the concatenation, not §0's own file** (§179). Only half of what it
# checks lives in §0: the `reviewed:` date and the 50-line cap do, but the last
# arm walks every `§N` and `§N.M` the block cites and asks whether that heading
# exists — and after the split those headings are in 177 other files. Pointing
# this at `docs/decisions.d/000-*.md` alone would have turned every correct
# citation in §0 into "cites sections that do not exist", which is louder than
# the failure it was written for and wrong every time. `decisions-all` is the
# same bytes the single file held, so the check below is the one that ran
# against it, unchanged, and its fixtures are unchanged with it.
[doc("docs/decisions.md §0 exists, parses, and points only at sections that do.")]
check-where-the-work-is:
    #!/usr/bin/env bash
    set -uo pipefail
    all=$(mktemp)
    trap 'rm -f "$all"' EXIT
    {{ just_executable() }} decisions-all > "$all" || exit 1
    {{ just_executable() }} _where-the-work-is-check "$all" '{{ decisions_file }}'

# Held apart from the recipe above so the selftest can run it over fixtures
# whose answer is known, rather than over whatever §0 happens to say today.
#
# `name` is what a failure calls the file, so the recipe above can hand over a
# temporary concatenation and still have the message name `docs/decisions.md`,
# which is the path a reader has to go and edit. It defaults to `file`, which is
# what the fixtures pass.
[private]
_where-the-work-is-check file name="":
    #!/usr/bin/env bash
    set -uo pipefail
    file='{{file}}'
    name='{{name}}'
    [ -n "$name" ] || name="$file"

    block=$(awk '/^## 0\. /{on=1} on && /^## [1-9]/{exit} on{print}' "$file")
    if [ -z "$block" ]; then
        echo "$name has no '## 0.' section, so nothing says where the work is"
        exit 1
    fi
    if ! head -1 <<<"$block" | grep -qx '## 0\. Where the work is'; then
        echo "$(head -1 <<<"$block")"
        echo "^ §0 is reserved for 'Where the work is'; \`just status\` prints it by that name"
        exit 1
    fi

    lines=$(grep -c . <<<"$block")
    if [ "$lines" -gt 50 ]; then
        echo "§0 is $lines non-blank lines; the cap is 50, because a summary nobody re-reads is not one"
        echo "^ cut $((lines - 50)) non-blank lines, in one pass: trimming by eye takes two to four (§102.3)"
        exit 1
    fi

    reviewed=$(sed -n 's/^reviewed: *//p' <<<"$block" | head -1)
    if [ -z "$reviewed" ]; then
        echo "§0 has no 'reviewed: <date>' line, so \`just status\` cannot say how stale it is"
        exit 1
    fi
    if ! date -d "$reviewed" +%s >/dev/null 2>&1; then
        echo "§0's reviewed date '$reviewed' is not one \`date -d\` can read, so its age would print as nothing"
        exit 1
    fi

    missing=""
    for id in $(grep -oE '§[0-9]+(\.[0-9]+)?' <<<"$block" | sed 's/^§//' | sort -u); do
        case "$id" in
        *.*) pattern="^### ${id//./\\.} " ;;
        *)   pattern="^## ${id}\. " ;;
        esac
        grep -qE "$pattern" "$file" || missing="$missing $id"
    done
    if [ -n "$missing" ]; then
        echo "§0 cites sections that do not exist:$missing"
        echo "^ a renumbering moved them; §0 reads as current whether or not it is (docs/decisions.md §71)"
        exit 1
    fi

    # Headroom, not a verdict (§178.4): a session that knows it has three lines
    # left writes the trim into the edit it is already making, where "clean"
    # alone sends it to find out at the gate instead. Printed on every clean run
    # of this check, including from `just gate` — `decisions-zero.sh` is what
    # this line was written for, but there is no reason `gate` should see less.
    echo "§0 is $lines of 50 non-blank lines, reviewed $reviewed, every section it cites exists."

# Pins both halves of `status` against fixtures, because the live repository
# cannot exercise either of them. Its §0 is right today by construction, and its
# branches are whatever happens to be lying around — a `_status-branches` that
# had lost the ability to tell a cherry-picked branch from an unlanded one would
# pass on most mornings and fail on the one that mattered.
#
# The branch fixture is the 2026-09-14 tree in miniature: a branch whose commit
# was cherry-picked onto main (ahead by the count, carrying nothing), a branch
# with real unlanded work, a branch fully merged, and a branch checked out in a
# worktree, which must be reported with the tree that holds it.
#
# The §0 fixtures are the three failures the check exists for, plus a good one,
# because a check that rejects everything passes every negative case.
[private]
check-status-selftest:
    #!/usr/bin/env bash
    set -uo pipefail
    dir=$(mktemp -d)
    trap 'rm -rf "$dir"' EXIT
    fail() {
        echo "status drifted from its own fixtures: $1"
        echo "--- got ---"; echo "$got"
        exit 1
    }

    git init -q -b main "$dir/repo"
    git -C "$dir/repo" config user.email t@t
    git -C "$dir/repo" config user.name t
    echo root > "$dir/repo/f"; git -C "$dir/repo" add f
    git -C "$dir/repo" commit -q -m root

    git -C "$dir/repo" checkout -q -b picked
    echo picked > "$dir/repo/p"; git -C "$dir/repo" add p
    git -C "$dir/repo" commit -q -m "a patch that reaches main another way"
    picked=$(git -C "$dir/repo" rev-parse HEAD)

    git -C "$dir/repo" checkout -q -b unlanded main
    echo unlanded > "$dir/repo/u"; git -C "$dir/repo" add u
    git -C "$dir/repo" commit -q -m "work that is genuinely not on main"

    git -C "$dir/repo" checkout -q main
    # Without an unrelated commit here first, cherry-picking onto the same
    # parent reproduces the byte-identical sha, `main..picked` is empty, and the
    # fixture quietly stops testing the thing it exists for: the broken
    # ahead-of-main implementation passed it.
    echo other > "$dir/repo/x"; git -C "$dir/repo" add x
    git -C "$dir/repo" commit -q -m "an unrelated commit, so the pick is not the same sha"
    git -C "$dir/repo" cherry-pick "$picked" >/dev/null 2>&1 \
        || { echo "the fixture's cherry-pick did not apply, so nothing here was tested"; exit 1; }
    [ "$(git -C "$dir/repo" rev-list --count main..picked)" -eq 1 ] \
        || { echo "the fixture's picked branch is reachable from main, so it cannot tell the two counts apart"; exit 1; }
    git -C "$dir/repo" branch -q merged main
    git -C "$dir/repo" worktree add -q "$dir/held" unlanded

    got=$(just _status-branches "$dir/repo" main)
    grep -q "^unlanded  *1 commit(s)  *$dir/held$" <<<"$got" \
        || fail "the branch with real work is not listed with the worktree holding it"
    grep -q '^picked' <<<"$got" \
        && fail "a branch whose patch is already on main was listed as carrying work"
    grep -q '^merged' <<<"$got" \
        && fail "a fully merged branch was listed as carrying work"
    grep -qx "2 branch(es) carry nothing main does not already have" <<<"$got" \
        || fail "the quiet count is wrong, so a broken sweep could pass as a clean tree"

    got=$(just _status-branches "$dir/repo" no-such-branch 2>&1) \
        && fail "weighing against a branch that does not exist did not fail"
    grep -qF "no branch named no-such-branch" <<<"$got" \
        || fail "a missing base branch did not say so"

    # The green row, against a stub `gh` so no arm needs the network or an
    # account — `check-fuzz-watermark-selftest`'s fixture, for the same reason.
    # What is pinned here is that the local record can answer *alone*: with the
    # Actions allowance spent that is the only source there is, and the version
    # of this row that asked GitHub and stopped reported a tip 22 commits from a
    # green while a full gate had read that very tip. §173.9.
    mkdir -p "$dir/bin"
    cat >"$dir/bin/gh" <<'STUB'
    #!/usr/bin/env bash
    cat "$GH_GREEN"
    STUB
    sed -i 's/^    //' "$dir/bin/gh"
    chmod +x "$dir/bin/gh"
    : >"$dir/ci"
    greens="$dir/repo/.git/rcad-greens"
    : >"$greens"

    git -C "$dir/repo" checkout -q main
    two=$(git -C "$dir/repo" rev-parse main~1)
    three=$(git -C "$dir/repo" rev-parse main)
    aside=$(git -C "$dir/repo" rev-parse unlanded)

    green_row() {
        GH_GREEN="$dir/ci" PATH="$dir/bin:$PATH" \
            just _status-green "$dir/repo" main 2>&1
    }

    printf '%s 2026-09-20T00:00:00Z somehost\n' "$two" >"$greens"
    got=$(green_row)
    grep -q "last green $(cut -c1-7 <<<"$two") (a local gate on somehost" <<<"$got" \
        || fail "with gh silent the local gate record did not answer, which is the only source left"
    grep -q "1 commit(s) on main not yet green" <<<"$got" \
        || fail "the local green was named but main was not weighed against it"
    grep -qF "nothing independent has read it" <<<"$got" \
        || fail "a green only this machine has seen was not said to be this machine's"

    # Furthest forward, not first seen: the two sources merged arrive in no
    # order, so a loop taking the first match would pass only by luck.
    printf '%s\n' "$two" >"$dir/ci"
    printf '%s 2026-09-20T00:00:00Z somehost\n' "$three" >"$greens"
    got=$(green_row)
    grep -q "last green $(cut -c1-7 <<<"$three")" <<<"$got" \
        || fail "it chose a green behind the furthest one that qualifies"

    # A CI green is a different machine, so it carries no caveat — and the row
    # must still be able to say so, or the caveat means nothing when it appears.
    printf '%s\n' "$three" >"$dir/ci"
    printf '%s 2026-09-20T00:00:00Z somehost\n' "$two" >"$greens"
    got=$(green_row)
    grep -qF "(CI)" <<<"$got" \
        || fail "a green CI run on the tip was not credited to CI"
    grep -qF "nothing independent has read it" <<<"$got" \
        && fail "an independent green was still called this machine's"

    printf '%s\n' "$three" >"$dir/ci"
    printf '%s 2026-09-20T00:00:00Z somehost\n' "$three" >"$greens"
    got=$(green_row)
    grep -qF "(CI and a local gate)" <<<"$got" \
        || fail "a commit both sources name was credited to only one of them"

    # Green somewhere that is not `main` moves nothing, and a sha naming no
    # commit here is dropped rather than crashing the row.
    printf '%s\n0000000000000000000000000000000000000000\n' "$aside" >"$dir/ci"
    : >"$greens"
    got=$(green_row)
    grep -qF "no green commit is on main in this checkout" <<<"$got" \
        || fail "a green belonging to another branch was counted as main's"

    good='## 0. Where the work is

    reviewed: 2026-09-14

    **unit:** the thing, opened in §3.1 and still open.

    ## 1. Something else
    ### 3.1 Not really under 1, but this fixture is not check-decisions
    '
    printf '%s\n' "$good" | sed 's/^    //' > "$dir/good.md"
    got=$(just _where-the-work-is-check "$dir/good.md" 2>&1) \
        || fail "a well-formed §0 was rejected"

    sed 's/§3\.1/§9.9/' "$dir/good.md" > "$dir/dangling.md"
    got=$(just _where-the-work-is-check "$dir/dangling.md" 2>&1) \
        && fail "§0 citing a section that does not exist was accepted"
    grep -qF "cites sections that do not exist: 9.9" <<<"$got" \
        || fail "a dangling citation was not named"

    sed 's/^reviewed: .*/reviewed: whenever/' "$dir/good.md" > "$dir/undated.md"
    got=$(just _where-the-work-is-check "$dir/undated.md" 2>&1) \
        && fail "§0 with an unparsable reviewed date was accepted"

    sed '/^## 1\./i\
    filler' "$dir/good.md" > "$dir/long.md"
    for _ in $(seq 6); do
        sed '/^filler$/a\
    filler\
    filler\
    filler\
    filler\
    filler\
    filler\
    filler\
    filler\
    filler' "$dir/long.md" > "$dir/long.next.md"
        mv "$dir/long.next.md" "$dir/long.md"
    done
    got=$(just _where-the-work-is-check "$dir/long.md" 2>&1) \
        && fail "a §0 grown past the cap was accepted"
    grep -qF "the cap is 50" <<<"$got" \
        || fail "an over-long §0 did not say what the cap is"

    fifty=$(for _ in $(seq 50); do printf 'filler\n'; done)
    awk -v f="$fifty" '/^## 1\./{print f} {print}' "$dir/good.md" > "$dir/three-over.md"
    got=$(just _where-the-work-is-check "$dir/three-over.md" 2>&1) \
        && fail "a §0 three lines over the cap was accepted"
    grep -qF "cut 3 non-blank lines" <<<"$got" \
        || fail "an over-long §0 did not say how many lines to cut"

    printf '## 1. No zero here\n' > "$dir/none.md"
    got=$(just _where-the-work-is-check "$dir/none.md" 2>&1) \
        && fail "a file with no §0 at all was accepted"

    # Every assertion above is `command && fail`, so the last one leaves its own
    # non-zero status behind when it passes. Without this the whole selftest
    # failed silently on a green run, printing nothing at all.
    exit 0

# Who is working where, right now. Run this before you touch a shared tree.
#
# **A worktree's lock is this repo's record of who owns it**, and it is the only
# record that survives the session that made it: it is a file in
# `.git/worktrees/<name>/locked`, not session state, so a `/clear` cannot lose it
# and a session that never dispatched the lane can still read it. `git worktree
# lock --reason` writes it and `git worktree unlock` clears it. Locking blocks
# only `worktree remove` and `worktree prune`, so it never gets in a lane's way.
#
# The alternative that was being used was `ps aux | grep claude`, and it cannot
# work: a subagent runs inside its parent's process, so two agents colliding in
# one worktree are one pid with one working directory, while every unrelated
# session on the machine — other repositories included — shows up as a hit.
#
# **An empty answer is a failure here, not a pass.** A repository always has at
# least its own worktree, so a listing that parses to nothing means the parse
# broke, and this says so instead of printing a reassuring silence.
#
# **The first tree is the hub, and it is not a lane.** Git refuses to lock the
# main working tree, so no reason string can ever appear beside it and printing
# `free` there would be a claim this command cannot make. It prints `hub`, leaves
# it out of the count, and says so when it is not parked and clean —
# `just check-hub` is the one that fails over it (docs/decisions.md §60).
[doc("Who is working where, right now. Run this before you touch a shared tree.")]
lanes dir=".":
    #!/usr/bin/env bash
    set -euo pipefail
    repo=$(git -C "{{dir}}" rev-parse --show-toplevel)
    listing=$(git -C "$repo" worktree list --porcelain)
    if [ -z "$listing" ]; then
        echo "git worktree list --porcelain printed nothing, so this counted no lanes"
        exit 1
    fi
    total=0
    held=0
    seen=0
    hub_warning=""
    tree=""
    branch=""
    locked=""
    emit() {
        [ -n "$tree" ] || return 0
        seen=$((seen + 1))
        if [ ! -d "$tree" ]; then
            work="missing"
        elif [ -n "$(git -C "$tree" status --porcelain 2>/dev/null)" ]; then
            work="dirty"
        else
            work="clean"
        fi
        if [ "$seen" -eq 1 ]; then
            printf '%-4s %-7s %s [%s]\n' "hub" "$work" "$tree" "$branch"
            if [ "$branch" != "(detached)" ] || [ "$work" != "clean" ]; then
                hub_warning="somebody is working in the hub, which is the one tree that cannot record an owner; \`just check-hub\` says what to do"
            fi
            return 0
        fi
        total=$((total + 1))
        state="free"
        if [ -n "$locked" ]; then
            state="HELD"
            held=$((held + 1))
        fi
        printf '%-4s %-7s %s [%s]\n' "$state" "$work" "$tree" "$branch"
        if [ -n "$locked" ]; then
            printf '         held by: %s\n' "$(why_it_is_held)"
        fi
    }
    # Not from the porcelain listing, which C-quotes any reason holding a
    # non-ASCII byte — a `·` comes back as `"\302\267"`. The lock file holds the
    # reason as it was written, and a reason naming an agent is the whole point.
    why_it_is_held() {
        [ -d "$tree" ] || { echo "(locked; its directory is gone, so the reason cannot be read)"; return 0; }
        note="$(git -C "$tree" rev-parse --git-dir)/locked"
        if [ -s "$note" ]; then
            tr '\n' ' ' <"$note" | sed 's/ *$//'
        else
            echo "(locked, no reason given)"
        fi
    }
    while IFS= read -r line || [ -n "$line" ]; do
        case "$line" in
            "worktree "*)
                emit
                tree=${line#worktree }
                branch="(detached)"
                locked=""
                ;;
            "branch "*) branch=${line#branch refs/heads/} ;;
            "locked"*) locked="yes" ;;
        esac
    done <<<"$listing"
    emit
    if [ "$seen" -eq 0 ]; then
        echo "no worktree was parsed out of a non-empty listing, so the parse is broken"
        exit 1
    fi
    echo "$held of $total lane(s) held; the hub is not one of them, because git cannot lock it"
    [ -z "$hub_warning" ] || echo "$hub_warning"

# Pins `lanes` against a repository it builds itself, so it stays proven on a
# tree where nothing real is locked.
#
# The cases that matter are the ones this builds: a **held** worktree whose
# reason must come back verbatim, a **free** one that must not be reported as
# held, the **hub**, which must be labelled as such and left out of the count,
# and the **count** line, which is what stops a broken parse from looking like a
# quiet tree. Reading the live repository instead would prove nothing — it is
# unlocked almost all of the time, so a `lanes` that had lost the ability to see
# a lock at all would pass every run.
#
# The fixture's hub is deliberately left on a branch, which is the state
# `check-hub` rejects, so the warning line is exercised here too.
[private]
check-lanes-selftest:
    #!/usr/bin/env bash
    set -euo pipefail
    dir=$(mktemp -d)
    trap 'git -C "$dir/main" worktree unlock "$dir/held" 2>/dev/null || true; rm -rf "$dir"' EXIT
    git init -q -b main "$dir/main"
    git -C "$dir/main" -c user.email=t@t -c user.name=t commit -q --allow-empty -m "root"
    git -C "$dir/main" worktree add -q "$dir/held" -b held
    git -C "$dir/main" worktree add -q "$dir/free" -b free
    git -C "$dir/main" worktree lock --reason "agent-abc · deliverable · 2026-09-06T00:00:00Z" "$dir/held"
    got=$(just lanes "$dir/main")
    fail() {
        echo "lanes drifted from its own fixtures: $1"
        echo "--- got ---"; echo "$got"
        exit 1
    }
    grep -q "^HELD .* $dir/held \[held\]$" <<<"$got" \
        || fail "the locked worktree is not reported as HELD"
    grep -qF "held by: agent-abc · deliverable · 2026-09-06T00:00:00Z" <<<"$got" \
        || fail "the lock reason did not come back verbatim"
    grep -q "^free .* $dir/free \[free\]$" <<<"$got" \
        || fail "the unlocked worktree is not reported as free"
    grep -q "^hub .* $dir/main \[main\]$" <<<"$got" \
        || fail "the main working tree is not reported as the hub"
    grep -qx "1 of 2 lane(s) held; the hub is not one of them, because git cannot lock it" <<<"$got" \
        || fail "the count line is wrong, so an empty parse could pass as a quiet tree"
    grep -qF "somebody is working in the hub" <<<"$got" \
        || fail "a hub sitting on a branch drew no warning"

# Hand back every lane matching a pattern, in one call.
#
# `git worktree unlock` takes one tree, so releasing a batch of lanes was one
# call per lane: fifteen of them in a row on 2026-09-12, at about 137k of
# context each, which is two million tokens to clear fifteen locks. The locks
# are what `lanes` reads as ownership (§55), so a batch of lanes finishing
# together is the ordinary case and not an unusual one.
#
# **The pattern is a glob, and it is matched against the branch name and against
# the worktree's own directory name.** Both, because a lane is addressed by
# either depending on who is asking — `release-lanes 'lane-fuzz-*'` after a
# batch of briefs, `release-lanes 'agent-*'` after a batch of sessions. It is
# required and has no default: a recipe that released everything when told
# nothing is a recipe that unlocks a tree somebody is still working in.
#
# **The hub is never touched**, and it is left out by position rather than by
# name. Git documents the main working tree as the first entry of
# `--porcelain`, which is the same parse `lanes` and `check-hub` use, and it
# answers *"fatal: The main working tree cannot be locked or unlocked"* — so a
# pattern matching the hub has to be reported rather than attempted.
[doc("Unlock every lane matching a pattern: `just release-lanes 'lane-fuzz-*'`.")]
release-lanes pattern dir=".":
    #!/usr/bin/env bash
    set -euo pipefail
    repo=$(git -C "{{dir}}" rev-parse --show-toplevel)
    listing=$(git -C "$repo" worktree list --porcelain)
    if [ -z "$listing" ]; then
        echo "git worktree list --porcelain printed nothing, so this released nothing"
        exit 1
    fi
    pattern='{{ pattern }}'
    released=0
    left=0
    seen=0
    tree=""
    branch=""
    locked=""
    consider() {
        [ -n "$tree" ] || return 0
        seen=$((seen + 1))
        if [ "$seen" -eq 1 ]; then
            printf 'hub      left alone, git cannot lock it and nobody works there: %s\n' "$tree"
            return 0
        fi
        if ! matches "$branch" && ! matches "$(basename "$tree")"; then
            left=$((left + 1))
            printf 'skipped  no match for %s: %s [%s]\n' "'$pattern'" "$tree" "$branch"
            return 0
        fi
        if [ -z "$locked" ]; then
            left=$((left + 1))
            printf 'skipped  already free: %s [%s]\n' "$tree" "$branch"
            return 0
        fi
        was="$(why_it_is_held)"
        git -C "$repo" worktree unlock "$tree"
        released=$((released + 1))
        printf 'RELEASED %s [%s]\n' "$tree" "$branch"
        printf '         was held by: %s\n' "$was"
    }
    matches() {
        case "$1" in
            $pattern) return 0 ;;
            *) return 1 ;;
        esac
    }
    # The same reason `lanes` prints, read from the lock file rather than from
    # the porcelain listing, which C-quotes any non-ASCII byte in it. Read before
    # the unlock, because the unlock is what deletes it.
    why_it_is_held() {
        [ -d "$tree" ] || { echo "(locked; its directory is gone, so the reason cannot be read)"; return 0; }
        note="$(git -C "$tree" rev-parse --git-dir)/locked"
        if [ -s "$note" ]; then
            tr '\n' ' ' <"$note" | sed 's/ *$//'
        else
            echo "(locked, no reason given)"
        fi
    }
    while IFS= read -r line || [ -n "$line" ]; do
        case "$line" in
            "worktree "*)
                consider
                tree=${line#worktree }
                branch="(detached)"
                locked=""
                ;;
            "branch "*) branch=${line#branch refs/heads/} ;;
            "locked"*) locked="yes" ;;
        esac
    done <<<"$listing"
    consider
    if [ "$seen" -eq 0 ]; then
        echo "no worktree was parsed out of a non-empty listing, so the parse is broken"
        exit 1
    fi
    echo "$released lane(s) released, $left left as they were; the hub is never either"

# Pins `release-lanes` against a repository it builds itself: what matches, what
# does not, and the hub.
#
# The case that costs the most to get wrong is the third one. A pattern that
# names the hub cannot be answered by unlocking it — git refuses, and `set -e`
# would take the whole batch down with the fatal — so the hub has to be
# recognised by position and reported. That arm passes only if the recipe never
# reached for `git worktree unlock` on it.
[private]
check-release-lanes-selftest:
    #!/usr/bin/env bash
    set -euo pipefail
    dir=$(mktemp -d)
    trap 'for t in alpha beta unrelated agent-xyz; do git -C "$dir/main" worktree unlock "$dir/$t" 2>/dev/null || true; done; rm -rf "$dir"' EXIT
    git init -q -b main "$dir/main"
    git -C "$dir/main" -c user.email=t@t -c user.name=t commit -q --allow-empty -m root
    git -C "$dir/main" worktree add -q "$dir/alpha" -b lane-alpha
    git -C "$dir/main" worktree add -q "$dir/beta" -b lane-beta
    git -C "$dir/main" worktree add -q "$dir/quiet" -b lane-quiet
    git -C "$dir/main" worktree add -q "$dir/unrelated" -b keep-me
    git -C "$dir/main" worktree add -q "$dir/agent-xyz" -b nothing-like-a-lane
    for t in alpha beta unrelated agent-xyz; do
        git -C "$dir/main" worktree lock --reason "agent-$t · deliverable · 2026-09-12T00:00:00Z" "$dir/$t"
    done

    fail() {
        echo "release-lanes drifted from its own fixtures: $1"
        echo "--- got ---"; echo "$got"
        exit 1
    }
    still_locked() {
        git -C "$dir/main" worktree list --porcelain \
            | awk -v want="worktree $1" '/^worktree /{found = ($0 == want)} \
                                          found && /^locked/ {print "yes"; exit}'
    }

    # One: the pattern matches two branches out of five.
    got=$(just release-lanes 'lane-*' "$dir/main")
    [ -z "$(still_locked "$dir/alpha")" ] || fail "a lane matching the pattern was left locked"
    [ -z "$(still_locked "$dir/beta")" ] || fail "only the first matching lane was released"
    [ -n "$(still_locked "$dir/unrelated")" ] \
        || fail "a tree that does not match the pattern was unlocked anyway"
    [ -n "$(still_locked "$dir/agent-xyz")" ] \
        || fail "a tree that does not match the pattern was unlocked anyway"
    grep -qF "RELEASED $dir/alpha [lane-alpha]" <<<"$got" \
        || fail "a released lane was not reported"
    grep -qF "was held by: agent-alpha · deliverable · 2026-09-12T00:00:00Z" <<<"$got" \
        || fail "the lock reason was thrown away rather than reported before the unlock"
    grep -qF "skipped  no match for 'lane-*': $dir/unrelated [keep-me]" <<<"$got" \
        || fail "a skipped tree was not said to be skipped, so silence reads as success"
    grep -qF "skipped  already free: $dir/quiet [lane-quiet]" <<<"$got" \
        || fail "a matching tree that was not locked was not reported as already free"
    grep -qx "2 lane(s) released, 3 left as they were; the hub is never either" <<<"$got" \
        || fail "the count line is wrong, so a parse that saw nothing could pass as a quiet batch"

    # Two: the directory name is an address too, because a batch of sessions is
    # named that way and a batch of briefs is named the other way.
    got=$(just release-lanes 'agent-*' "$dir/main")
    [ -z "$(still_locked "$dir/agent-xyz")" ] \
        || fail "a pattern matching the worktree's directory released nothing"
    [ -n "$(still_locked "$dir/unrelated")" ] \
        || fail "matching on the directory name released a tree that matched neither"

    # Three: a pattern naming the hub leaves it alone and says so, rather than
    # dying on git's refusal to unlock the main working tree.
    got=$(just release-lanes 'main' "$dir/main") \
        || fail "a pattern matching the hub took the whole batch down with it"
    grep -qF "hub      left alone, git cannot lock it and nobody works there: $dir/main" <<<"$got" \
        || fail "the hub was not reported as left alone"
    grep -qx "0 lane(s) released, 5 left as they were; the hub is never either" <<<"$got" \
        || fail "the hub was counted as a lane"

# Nobody works in the main working tree. It owns `.git` and nothing else.
#
# §55 made a worktree's lock this repo's record of who owns it, and the one tree
# that record structurally cannot describe is the tree every session lands in by
# default: `git worktree lock` answers *"fatal: The main working tree cannot be
# locked or unlocked"*, by path or by `.`, with no flag to override. This closes
# that by deleting the case rather than by inventing a second record beside the
# first — a tree nobody works in needs no owner, and `free` becomes the truth
# about it instead of a guess (docs/decisions.md §60).
#
# **The load-bearing half is that `main` is not checked out here.** Git refuses
# to move a branch that is checked out somewhere else — *"refusing to fetch into
# branch 'refs/heads/main' checked out at ..."*, and *"! [remote rejected] lane
# -> main (branch is currently checked out)"* — so a hub holding `main` drags
# every merge back into the one tree whose owner cannot be recorded, however
# carefully the lanes were dispatched. Parked on a detached HEAD, `main` is
# checkable out, and therefore lockable, in a worktree like any lane.
#
# **A repository with one worktree is exempt**, because then there is no hub and
# no lane, only a checkout, and no second session to collide with. That is a
# fresh clone, and it is the shape a CI checkout has. This is no longer a
# prediction: CI runs the whole `gate` since 2026-09-07, so a runner reaches
# this recipe, and run `34266469454` took the exemption and said so — *"is the
# only worktree, so it is a checkout and not a hub"*. The exemption is
# structural rather than an environment variable on purpose: one that has to be
# set is one that gets set, while this lapses by itself the moment a second tree
# exists, which is the moment ownership starts being a question.
[doc("Nobody works in the hub: it owns .git and nothing else.")]
check-hub dir=".":
    #!/usr/bin/env bash
    set -euo pipefail
    listing=$(git -C "{{dir}}" worktree list --porcelain)
    if [ -z "$listing" ]; then
        echo "git worktree list --porcelain printed nothing, so this checked no hub"
        exit 1
    fi
    # Git documents the main working tree as the first entry of the listing.
    hub=$(awk '/^worktree /{print $2; exit}' <<<"$listing")
    trees=$(grep -c '^worktree ' <<<"$listing")
    if [ -z "$hub" ]; then
        echo "no worktree was parsed out of a non-empty listing, so the parse is broken"
        exit 1
    fi
    if [ "$trees" -eq 1 ]; then
        echo "$hub is the only worktree, so it is a checkout and not a hub"
        exit 0
    fi
    wrong=0
    branch=$(git -C "$hub" symbolic-ref --quiet --short HEAD || true)
    if [ -n "$branch" ]; then
        echo "the hub has the branch '$branch' checked out: $hub"
        echo "    git -C $hub checkout --detach"
        echo "  while a branch lives here, git will not let a lane move it, so every merge has to come back to the one tree it cannot lock"
        wrong=1
    fi
    dirt=$(git -C "$hub" status --porcelain)
    if [ -n "$dirt" ]; then
        echo "the hub has $(wc -l <<<"$dirt") path(s) of uncommitted work in it: $hub"
        head -n 10 <<<"$dirt" | sed 's/^/    /'
        if [ "$(wc -l <<<"$dirt")" -gt 10 ]; then
            echo "    ..."
        fi
        # A hub parked at an older commit shows work that has since landed as
        # untracked, which is a *stale* hub and not leftovers. Same symptom,
        # opposite fix: telling the reader to commit files that are already
        # committed sends them to duplicate somebody's merge. The two are told
        # apart by asking whether every path is already on `main`, which has to
        # mean the same bytes and not the same name. A name-only answer sends a
        # reader to delete a file whose content is on `main` under that name but
        # is not the content in front of them (docs/decisions.md §60.10).
        untracked=$(grep '^??' <<<"$dirt" | cut -c4- || true)
        modified=$(grep -v '^??' <<<"$dirt" || true)
        unmatched=""
        if [ -z "$modified" ] && [ -n "$untracked" ]; then
            # Enumerated rather than read off `status`, which reports an
            # untracked directory as one entry and quotes an awkward name. Every
            # file the `clean` would take is hashed; an empty directory is the
            # only thing it takes that this does not see, and it holds nothing.
            while IFS= read -r -d '' path; do
                on_main=$(git -C "$hub" rev-parse --verify --quiet "main:$path" || true)
                if [ -n "$on_main" ] \
                    && [ "$on_main" = "$(git -C "$hub" hash-object -- "$hub/$path")" ]; then
                    continue
                fi
                unmatched="$unmatched$path"$'\n'
            done < <(git -C "$hub" ls-files --others --exclude-standard -z)
        fi
        if [ -z "$modified" ] && [ -n "$untracked" ] && [ -z "$unmatched" ]; then
            echo "  Every one of those is already committed on \`main\`, byte for byte —"
            echo "  compared by content, not by name — so nothing here is lost work; the"
            echo "  hub is just parked at an older commit. Re-park it:"
            # One line, because `check-hub-selftest` greps it back out and runs
            # it. Wrapping it for width is caught rather than silent: the grep
            # then hands bash a dangling `&&` and the selftest fails on it.
            echo "      git -C $hub clean -fd && git -C $hub checkout --detach main"
            echo "  The clean is not optional. Checkout refuses to overwrite an untracked"
            echo "  file even when its content already matches the commit being moved to,"
            echo "  so the removal has to come first; the checkout then restores every one"
            echo "  of them byte-identical. Ignored files are left alone, so fuzz artifacts"
            echo "  survive it. Nothing moves the hub's HEAD on its own."
        else
            # The hub is shared, so this is usually somebody else's leftovers and
            # the reader's first thought is that their own worktree is broken.
            # Say whose it might be before saying what to do about it.
            echo "  This is very likely not yours. The hub belongs to nobody, so anyone's"
            echo "  leftovers fail this for everyone, and the tree you are working in is fine."
            echo "  Whoever left it should commit it from a worktree of their own; \`just lanes\`"
            echo "  will not name them, which is the reason for the rule rather than a gap in it."
        fi
        wrong=1
    fi
    if [ "$wrong" -ne 0 ]; then
        exit 1
    fi
    echo "the hub is parked and clean: $hub"

# Pins `check-hub` against a repository it builds itself, walking a hub from the
# state this repo was in on 2026-09-06 to the state the rule wants.
#
# Four cases, and the first is the one that would otherwise take CI down: a lone
# checkout must **pass**, because there it is not a hub. Then a second worktree
# appears and the same tree must start **failing** — on its branch, then on its
# dirt, then not at all once it is parked and clean. Reading the live repository
# instead would prove nothing: it is in exactly one of these states at a time,
# and a `check-hub` that had lost the ability to fail would pass every run.
[private]
check-hub-selftest:
    #!/usr/bin/env bash
    set -euo pipefail
    dir=$(mktemp -d)
    trap 'rm -rf "$dir"' EXIT
    git init -q -b main "$dir/main"
    git -C "$dir/main" -c user.email=t@t -c user.name=t commit -q --allow-empty -m "root"
    fail() {
        echo "check-hub drifted from its own fixtures: $1"
        echo "--- got ---"; echo "$got"
        exit 1
    }
    got=$(just check-hub "$dir/main") \
        || fail "a repository with one worktree is not exempt, so this would fail in CI"
    grep -qF "the only worktree" <<<"$got" || fail "the lone checkout passed for the wrong reason"

    git -C "$dir/main" worktree add -q "$dir/lane" -b lane
    got=$(just check-hub "$dir/main" 2>&1) && fail "a hub holding a branch was accepted"
    grep -qF "the branch 'main' checked out" <<<"$got" || fail "the branch was not named"

    git -C "$dir/main" checkout -q --detach
    echo loose >"$dir/main/untracked"
    got=$(just check-hub "$dir/main" 2>&1) && fail "a hub with uncommitted work in it was accepted"
    grep -qF "1 path(s) of uncommitted work" <<<"$got" || fail "the dirt was not counted"
    grep -qF "?? untracked" <<<"$got" || fail "the dirt was not listed"
    grep -qF "very likely not yours" <<<"$got" \
        || fail "the failure does not say it is a shared condition, so it reads as the runner's fault"

    rm "$dir/main/untracked"
    got=$(just check-hub "$dir/main") || fail "a parked, clean hub was rejected"
    grep -qF "parked and clean" <<<"$got" || fail "the parked hub passed for the wrong reason"

    # A hub left behind by a merge: the file is on `main` and the hub's own HEAD
    # predates it, so `status` calls it untracked. Telling this reader to commit
    # it would send them to duplicate a merge that has already happened.
    echo landed >"$dir/lane/landed"
    git -C "$dir/lane" add landed
    git -C "$dir/lane" -c user.email=t@t -c user.name=t commit -q -m "landed"
    git -C "$dir/main" branch -f main lane
    echo landed >"$dir/main/landed"
    got=$(just check-hub "$dir/main" 2>&1) && fail "a stale hub was accepted"
    grep -qF "already committed on \`main\`" <<<"$got" \
        || fail "a hub parked behind main was reported as somebody's leftovers"
    grep -qF "checkout --detach main" <<<"$got" || fail "the re-park command was not given"
    if grep -qF "very likely not yours" <<<"$got"; then
        fail "the stale hub got the leftovers message as well as the stale one"
    fi

    # Run what it printed. A remedy nobody executes is a claim without a test,
    # and this one was wrong when it was first written: `checkout --detach` aborts
    # over an untracked file even when the content already matches the commit
    # being moved to, so a re-park with no `clean` in it leaves the hub where it
    # was and the reader distrusting a check that had the diagnosis right.
    remedy=$(grep -F 'clean -fd' <<<"$got" | sed 's/^ *//')
    [ -n "$remedy" ] || fail "no runnable remedy line was printed"
    bash -c "$remedy" >/dev/null 2>&1 || fail "the remedy the check prints does not run: $remedy"
    [ "$(git -C "$dir/main" rev-parse HEAD)" = "$(git -C "$dir/main" rev-parse main)" ] \
        || fail "the printed remedy ran but the hub did not move onto main"
    [ -z "$(git -C "$dir/main" status --porcelain)" ] \
        || fail "the hub is still dirty after its own remedy ran"
    [ "$(cat "$dir/main/landed")" = "landed" ] \
        || fail "the remedy did not restore the file it removed"
    got=$(just check-hub "$dir/main") \
        || fail "check-hub still fails after the reader did exactly what it said"

    # The same name on `main`, different bytes in the hub. Everything above this
    # point passes with a name-only comparison, because the fixture writes the
    # same content on both sides — so the fixture has to differ for the check to
    # be able to fail. Here the hub holds work that is in no commit anywhere, and
    # calling it "already committed on `main`" sends the reader to delete it with
    # the `clean -fd` the same message prints (docs/decisions.md §60.10).
    git -C "$dir/main" checkout -q --detach "main^"
    printf 'work nobody has committed\n' >"$dir/main/landed"
    got=$(just check-hub "$dir/main" 2>&1) && fail "a hub holding uncommitted work was accepted"
    if grep -qF "already committed on \`main\`" <<<"$got"; then
        fail "content that is on no branch was called committed, and the remedy deletes it"
    fi
    grep -qF "very likely not yours" <<<"$got" \
        || fail "differing content got neither the stale message nor the leftovers one"
    [ "$(cat "$dir/main/landed")" = "work nobody has committed" ] \
        || fail "the check itself changed the file it was asked about"

# Pins `tooling/hub-align/align.sh` against a repository it builds itself.
#
# The script runs ahead of every recipe in this file, in every worktree, so the
# cases that matter most are the ones where it must do **nothing**: a lone
# checkout, a hub holding a branch, and a hub holding anything that is not on
# `main`. A re-park deletes untracked files, so a wrong "yes" here deletes work,
# which is why the fixture writes differing bytes under a name `main` also
# carries (docs/decisions.md §60.10) and asserts they survive.
#
# Reading the live repository instead would prove nothing: it is in one of these
# states at a time, and a script that had lost the ability to act would pass.
[private]
check-hub-align-selftest:
    #!/usr/bin/env bash
    set -euo pipefail
    align="{{ justfile_directory() }}/tooling/hub-align/align.sh"
    [ -x "$align" ] || [ -f "$align" ] || { echo "no align.sh at $align"; exit 1; }
    dir=$(mktemp -d)
    trap 'rm -rf "$dir"' EXIT
    git init -q -b main "$dir/main"
    commit() { git -C "$1" -c user.email=t@t -c user.name=t commit -q "${@:2}"; }
    got=""
    fail() {
        echo "hub-align drifted from its own fixtures: $1"
        echo "--- got ---"; echo "$got"
        exit 1
    }
    run() {
        got=$(bash "$align" "$dir/main" 2>&1) \
            || fail "the script exited non-zero, which would take every recipe down with it"
    }

    commit "$dir/main" --allow-empty -m "root"
    root=$(git -C "$dir/main" rev-parse HEAD)
    echo one >"$dir/main/f"
    git -C "$dir/main" add f
    commit "$dir/main" -m "one"
    one=$(git -C "$dir/main" rev-parse HEAD)

    # A lone checkout is a checkout and not a hub, however far behind it is.
    git -C "$dir/main" checkout -q --detach "$root"
    run
    [ -z "$got" ] || fail "a repository with one worktree was spoken to"
    [ "$(git -C "$dir/main" rev-parse HEAD)" = "$root" ] \
        || fail "a lone checkout was re-parked, and it is somebody's working directory"

    # The same tree, once a second worktree makes it a hub.
    git -C "$dir/main" worktree add -q "$dir/lane" -b lane
    run
    grep -qF "re-parked it detached" <<<"$got" || fail "a stale hub was left behind"
    [ "$(git -C "$dir/main" rev-parse HEAD)" = "$(git -C "$dir/main" rev-parse main)" ] \
        || fail "the script said it re-parked and the hub did not move"
    [ "$(cat "$dir/main/f")" = "one" ] || fail "the file that main carries was not restored"
    [ -z "$(git -C "$dir/main" status --porcelain)" ] || fail "the hub is dirty after a re-park"

    run
    [ -z "$got" ] || fail "a hub already on main was spoken to, and this runs before every recipe"

    # Behind `main`, holding only files that have landed since. This is the
    # shape `check-hub` calls stale rather than leftovers, and the one the whole
    # feature exists for.
    echo landed >"$dir/lane/landed"
    git -C "$dir/lane" add landed
    commit "$dir/lane" -m "landed"
    git -C "$dir/main" branch -f main lane
    echo landed >"$dir/main/landed"
    run
    grep -qF "re-parked it detached" <<<"$got" || fail "a hub behind main with only landed files was left"
    [ "$(cat "$dir/main/landed")" = "landed" ] || fail "the re-park did not restore the file it removed"

    # A justfile arriving with the re-park makes this run's recipes yesterday's.
    printf 'a:\n    @echo one\n' >"$dir/lane/justfile"
    git -C "$dir/lane" add justfile
    commit "$dir/lane" -m "justfile"
    git -C "$dir/main" branch -f main lane
    run
    grep -qF "run it again" <<<"$got" \
        || fail "the justfile changed under a run in progress and nothing said so"

    # Differing bytes under a name `main` also carries. Everything above passes
    # with a name-only comparison; this is the case where a wrong answer runs a
    # `clean -fd` over the only copy of somebody's work.
    # Parked before `landed` existed, so the file below is untracked here and on
    # `main` under the same name with different bytes — which is the branch of
    # the script that compares content, and the one a name-only answer fails.
    git -C "$dir/main" checkout -q --detach "$one"
    printf 'work nobody has committed\n' >"$dir/main/landed"
    [ "$(git -C "$dir/main" status --porcelain)" = "?? landed" ] \
        || fail "the fixture meant to build an untracked file and did not"
    run
    grep -qF "left where it is" <<<"$got" || fail "a hub holding uncommitted work was re-parked"
    [ "$(cat "$dir/main/landed")" = "work nobody has committed" ] \
        || fail "content that is on no branch was deleted by the re-park"
    [ "$(git -C "$dir/main" rev-parse HEAD)" = "$one" ] || fail "the hub moved anyway"

    # A modified tracked file is the other half of that, and reaches it by a
    # different branch of the script.
    git -C "$dir/main" clean -fdq
    echo changed >"$dir/main/f"
    run
    grep -qF "a modified file" <<<"$got" || fail "a hub with a modified file was not named as such"
    [ "$(cat "$dir/main/f")" = "changed" ] || fail "a modified file was thrown away"

    # A branch checked out in the hub is `check-hub`'s failure, and it names the
    # fix. Detaching it here would move somebody off a branch without saying so.
    git -C "$dir/main" checkout -q -- f
    git -C "$dir/main" checkout -q -B parked "main^"
    run
    [ -z "$got" ] || fail "a hub holding a branch was spoken to by the wrong check"
    [ "$(git -C "$dir/main" symbolic-ref --short HEAD)" = "parked" ] \
        || fail "the script moved a hub off the branch it was holding"

# What a stretch of sessions cost, and what it was spent on.
#
# `just spend` covers the last seven days and prints every report. A window and
# one report narrow it: `just spend 2026-09-12 2026-09-20 ritual`, and a sixth
# argument keeps only the lanes whose path contains it.
#
# The reports are `summary` (token classes, batching, context, the cost of
# finding), `finding` (which of that a lookup lane could take, and what handing
# it over would cost), `classes` (which part of the tree the money went into),
# `ritual` (the recurring work no kernel change needs), `sessions`, `carry`
# (what fills the context every later response pays to re-read), `ceiling`
# (what handing off at a context cap would have saved), `warnings` (what the
# budget hook that exists actually caught) and `pairs` (a file searched in one
# response and read in the next — §144.6's two-call shape, priced).
#
# **`pairs` is what refused a tool change**: making `decisions-find` print the
# section when it matches one would have reached 4 of a week's 119 pairs on
# `docs/decisions.md`, and the whole shape there is under 1% of the bill.
# `docs/decisions.md` §161.
#
# **`summary`'s finding line is the one that gets misread**, and `finding`
# exists because it was: it counts every probe, wherever made and whatever
# about, and two thirds of what it counts is nobody's to delegate — probes
# inside lanes, which are cheap already, and §144.5's ritual, which is a session
# asking about its own state. `docs/decisions.md` §158.
#
# **`ceiling` and `warnings` ask different questions** and answered differently
# on the week of 2026-09-12: `ceiling` prices a cap nothing enforces, `warnings`
# prices the one installed, and the money turned out to be in sessions warned on
# time that kept going rather than in where the line was drawn.
# `docs/decisions.md` §160.
#
# **This reads transcripts, never the repository.** It cannot be a `check-` and
# is not in `gate_steps`: its answer moves with what the sessions did this week,
# so there is no fixed thing for a check to hold. What it is for is §144 — every
# finding there names the report that measures it, because a number written into
# a document is stale on the next session and a command is not.
#
# Transcripts live under `$CLAUDE_CONFIG_DIR/projects/<repo key>`, one directory
# per worktree. They are swept by nothing, so the window can reach back further
# than `/tmp` does — but a reboot takes the scratchpads and leaves these, which
# is why the evidence for a finding belongs here rather than in a handoff.
[doc("What a week of sessions cost and where it went: `just spend 2026-09-12 2026-09-20 ritual`.")]
spend since="" until="" report="all" lane="":
    #!/usr/bin/env bash
    set -euo pipefail
    since='{{since}}'; until='{{until}}'
    [ -n "$since" ] || since=$(date -u -d '7 days ago' +%F)
    [ -n "$until" ] || until=$(date -u -d 'tomorrow' +%F)
    exec python3 tooling/session-spend/spend.py \
        '{{claude_config}}/projects' \
        '-home-mattias-git-mattiasgronlund-rcad' \
        "$since" "$until" '{{report}}' '{{lane}}'

# The part of `spend` a check can hold: how it classifies a response, and what
# it read out of a hook's own message.
#
# Everything else there answers with this week's sessions in it and moves when
# they do, which is why `spend` is not in `gate_steps`. The classifiers are pure
# functions of a response's tool calls, and `ritual_of` was wrong for a week in
# the direction that mattered: a handoff listing the open questions counted as
# question-file work, so §144.5 reasoned about the question file from a figure
# a third too large. A number a document reasons from is worth a case.
#
# `warning_said` and `bucket_of` were wrong the same way a fortnight later: a
# warning recorded only as its summary line names no budget, and a parser that
# required one read three of the 39 sessions warned that week as sessions the
# hook never reached — a broken hook's signature, written by a broken parser.
# `docs/decisions.md` §160.
#
# `is_lookup`, `finding_runs` and `delegation_net` are held for the same reason
# one step on. §158's argument is entirely which probes a lane could have taken
# and what handing them over would have cost, and those three decide it — the
# obvious wrong version of each is caught by a case, which was checked rather
# than assumed.
#
# It needs no transcripts and no Claude Code, so it runs on a runner like any
# other check. `docs/decisions.md` §152.
# `PYTHONDONTWRITEBYTECODE` because the cases import `spend` as a module, and a
# `__pycache__` beside it is a directory the gate leaves behind in a tree the
# hub rule expects to be clean. Ignoring it would be a second record of the same
# thing; not writing it is one word.
[private]
check-spend-selftest:
    @PYTHONDONTWRITEBYTECODE=1 python3 tooling/session-spend/check-spend.py
