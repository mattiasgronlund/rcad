# `{{ args }}` interpolates a recipe's arguments into the shell *unquoted*, so a
# variadic recipe that passes them on splits them again on whitespace and hands
# the shell any punctuation they contain. That is not theoretical: `just t -E
# 'test(/walk/)'`, the example this file and CLAUDE.md both give, died on
# `Syntax error: "(" unexpected` from the day it was written, because `sh` saw
# the parentheses. With this setting the arguments arrive as `"$@"` with their
# original word boundaries, and `check-t-selftest` runs that exact example.
set positional-arguments

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
# `[private]` hides the nine `*-selftest` recipes, which nobody types.
[doc("Everything this repo can do, one line each. Bare `just` runs this.")]
list:
    @{{ just_executable() }} --list --unsorted

# Every step of the gate, in the order CI runs them, and the one place that
# order is written down. `check-gate-runs-every-check` reads this line to prove
# no `check-*` recipe was written and then never run.
#
# **A list rather than `gate`'s dependencies, which is what it used to be.** A
# dependency that fails takes `just` down with it, so a gate that failed printed
# whatever the failing step said and stopped — and the summary block below, the
# thing a reader actually wants, could only ever have been printed on a pass.
# Driving the steps from a loop is what lets the same block describe both.
gate_steps := "check-plugins check check-fuzz-builds check-enums check-wildcards check-wildcards-selftest check-ignored check-hooks check-kache-off-on-ci check-gate-runs-every-check check-gate-summary-selftest check-lanes-selftest check-release-lanes-selftest check-status-selftest check-hub check-hub-selftest check-clean-targets-selftest check-prune-worktrees-selftest check-decisions check-decisions-selftest check-where-the-work-is check-src-selftest check-questions-selftest check-corpus check-corpus-selftest check-fuzz-overnight-selftest check-fuzz-watermark-selftest check-fuzz-triage-selftest lint test check-release-build check-t-selftest deny"

# Where the whole run is kept, so the block at the end can be read back in full
# without running it again. A variable so `check-gate-summary-selftest` can
# point a nested run somewhere else: the selftest runs inside the gate, and a
# fixed path would have it truncate the log its own parent was still writing.
gate_log := "target/last-gate.log"

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
[doc("Everything, in the order CI runs it. ~9 minutes. Ends with a summary block.")]
gate:
    #!/usr/bin/env bash
    set -uo pipefail
    log='{{ gate_log }}'
    mkdir -p "$(dirname "$log")"
    : >"$log"
    failed=""
    status=0
    for step in {{ gate_steps }}; do
        {{ just_executable() }} "$step" 2>&1 | tee -a "$log"
        # Read before anything else runs. `[` is a command, so it rewrites
        # `PIPESTATUS` with its own 0 — which read as a clean gate over a step
        # that had just failed, the first time this loop was run.
        step_status=${PIPESTATUS[0]}
        if [ "$step_status" -ne 0 ]; then
            failed="$step"
            status=$step_status
            break
        fi
    done
    {{ just_executable() }} _gate-summary "$log" "$status" "$failed" | tee -a "$log"
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
wildcard_arms := "88"

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
    got=$(awk -v enums="Toy" -f crates/rcad-step/tools/composite_wildcards.awk \
        "$dir/bad_array.rs" "$dir/bad_tuple_ref.rs" "$dir/good_named.rs" \
        "$dir/good_single_enum_wildcard.rs" "$dir/commented.rs" "$dir/toy_enum.rs" | sort)
    want=$(printf '%s\n' \
        "$dir/bad_array.rs:2" \
        "$dir/bad_tuple_ref.rs:2" \
        "$dir/commented.rs:3" \
        "COUNT=3" | sort)
    if [ "$got" != "$want" ]; then
        echo "composite_wildcards.awk drifted from its own fixtures:"
        echo "--- got ---"; echo "$got"
        echo "--- want ---"; echo "$want"
        exit 1
    fi

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
ignored_tests := "0"

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
# The two hooks below it are rcad's own, need no plugin, and run either way —
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

# kache must be the wrapper here and no wrapper on a runner (`mise.toml`).
#
# Both halves are asserted, because each one alone passes for the wrong reason.
# Checking only the CI half passes if somebody deletes the variable outright,
# which would turn the cache off for every human as well. Checking only the
# local half passes on the state that kept CI red for ten pushes.
#
# It reads `mise env` rather than the file, because the file is a template and
# what breaks CI is the value mise computes, not the text. That also means this
# fails if the template stops parsing, which the eye does not catch.
[doc("kache wraps rustc here, and wraps nothing on a runner.")]
check-kache-off-on-ci:
    @on=$(CI=true mise env | sed -n 's/^export RUSTC_WRAPPER=//p' | tr -d "\"'"); \
    off=$(env -u CI mise env | sed -n 's/^export RUSTC_WRAPPER=//p' | tr -d "\"'"); \
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
    @steps=$(grep -m1 '^gate_steps := ' justfile | sed 's/^gate_steps := "//; s/"$//'); \
    defined=$(grep -oE '^check-[a-z0-9-]+' justfile | sort -u); \
    listed=$(printf '%s\n' "$steps" | tr ' ' '\n' | grep '^check-' | sort -u); \
    others=$(printf '%s\n' "$steps" | tr ' ' '\n' | grep -v '^check-' | sort -u | tr '\n' ' '); \
    n=$(printf '%s\n' "$defined" | grep -c .); \
    if [ "$n" != "29" ]; then \
        echo "found $n check-* recipes, expected 29"; \
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
    first=$(printf '%s\n' "$steps" | tr ' ' '\n' | grep -m1 .); \
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

# Where `decisions-toc` and `decision` read from. A variable so the selftest
# below can point them at fixtures rather than at whatever the real file happens
# to say today.
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
    @awk '/^## [0-9]/ { \
             if (num != "") printf "  %-5s %-56s %4d lines\n", num, title, n; \
             rest = substr($0, 4); p = index(rest, " "); \
             num = substr(rest, 1, p - 1); sub(/\.$/, "", num); \
             title = substr(rest, p + 1); n = 0; next \
         } \
         num != "" { n++ } \
         END { if (num != "") printf "  %-5s %-56s %4d lines\n", num, title, n }' \
        {{decisions_file}}

# One section of docs/decisions.md, by its number.
#
# `just decision 49` gives §49's own prose and then the list of its subsections,
# which is the cheap way in; `just decision 49.6` gives §49.6 in full. A leading
# section sign is accepted so a citation can be pasted as it is written.
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
            END { exit(found ? 0 : 3) }' "{{decisions_file}}" || {
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
            END { exit(found ? 0 : 3) }' "{{decisions_file}}" || {
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
            END { flush() }' "{{decisions_file}}"
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
[doc("Which sections mention this? Answers in section numbers, not line numbers.")]
decisions-find pattern:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ -z "$1" ]; then
        echo "find what? an empty pattern matches every line" >&2
        exit 2
    fi
    hits=$(mktemp)
    trap 'rm -f "$hits"' EXIT
    if ! grep -inE -- "$1" "{{decisions_file}}" | cut -d: -f1 > "$hits"; then
        echo "no line in {{decisions_file}} matches $1" >&2
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
        }' "{{decisions_file}}"

# `decision` finds a section by the number its heading starts with, so a heading
# that does not start with one is a section the lookup cannot reach — and it
# fails as silence, which reads as "nothing was decided about this".
#
# All 487 headings are numbered today, `## N.` and `### N.M`. `####` headings are
# free text on purpose: they sit inside a subsection and are never addressed.
#
# A heading hiding inside a code fence would fool the same match. None exists and
# none ever has, so per §13 that check is not built until something needs it.
#
# **Two sections sharing a number is the second way to be unreachable**, and it
# is the one that happens by itself. `decision` stops at the first heading whose
# number matches, so the second §56 is addressed by nothing at all — and it is
# not a typo that puts it there. Sections are appended at the tail by whoever
# finishes first, ten to thirty times a day from as many as four concurrent
# sessions, and two of them reaching for the same next number is the ordinary
# case rather than the unlucky one. It happened while §57 was being written
# (§57.4), and nothing in this file noticed; the collision was found by hand,
# at merge time, which is the expensive place to find it.
#
# **A `## N.` lower than the one before it, and a `### N.M` sitting under the
# wrong `## N.`, are two more ways to go silently unreachable.** A renumbering
# that moves later sections down leaves the file out of order if a stray
# earlier one is missed, and a subsection typed by hand during that renumbering
# can end up under the wrong parent — `### 64.3` left under `## 69.` after 64
# was renumbered to 67 is the shape it takes, and it happened four times in one
# session and was only caught by grepping for it. Neither is a duplicate, so
# the check above passes both. `docs/decisions.md` §71.
[doc("Every heading in docs/decisions.md can be reached by its number.")]
check-decisions:
    #!/usr/bin/env bash
    set -euo pipefail
    bad=$(grep -nE '^(## |### )' "{{decisions_file}}" \
        | grep -vE '^[0-9]+:## [0-9]+\. ' \
        | grep -vE '^[0-9]+:### [0-9]+\.[0-9]+ ' || true)
    if [ -n "$bad" ]; then
        echo "$bad"
        echo "^ a decisions.md heading that 'just decision' cannot address"
        exit 1
    fi

    dupes=$(grep -oE '^(## [0-9]+|### [0-9]+\.[0-9]+)' "{{decisions_file}}" \
        | awk '{ print $2 }' | sort | uniq -d)
    if [ -n "$dupes" ]; then
        for id in $dupes; do
            grep -nE "^(## ${id//./\\.}\. |### ${id//./\\.} )" "{{decisions_file}}"
        done
        echo "^ these sections share a number, and 'just decision' reaches only the first"
        exit 1
    fi

    order=$(awk '
        /^## [0-9]/ {
            rest = $0; sub(/^#+ /, "", rest)
            p = index(rest, " "); n = substr(rest, 1, p - 1) + 0
            if (n < top) print FNR ": ## " n " comes after ## " top
            top = n; lastsub = 0
            next
        }
        /^### [0-9]/ {
            rest = $0; sub(/^#+ /, "", rest)
            p = index(rest, " "); id = substr(rest, 1, p - 1)
            split(id, parts, ".")
            subtop = parts[1] + 0; subnum = parts[2] + 0
            if (subtop != top)
                print FNR ": ### " id " is not under its own ## " top
            else if (subnum < lastsub)
                print FNR ": ### " id " comes after ### " top "." lastsub
            lastsub = subnum
        }' "{{decisions_file}}")
    if [ -n "$order" ]; then
        echo "$order"
        echo "^ decisions.md section numbers are out of order (docs/decisions.md §71)"
        exit 1
    fi

# Pins the lookup against fixtures rather than against the real file, so it stays
# proven on a tree where the real §1.1 has been rewritten.
#
# `#### ` inside the fixture is the case worth having: it is a heading, and a
# lookup that stops at any heading would cut §1.1 in half without saying so.
[private]
check-decisions-selftest:
    #!/usr/bin/env bash
    set -euo pipefail
    dir=$(mktemp -d)
    trap 'rm -rf "$dir"' EXIT
    cat > "$dir/fixture.md" <<'MD'
    # Design decisions
    ## 1. First
    intro to one
    ### 1.1 Alpha
    alpha body
    #### A fourth-level heading that must not end the section
    still alpha
    ### 1.2 Beta
    beta body
    ## 2. Second
    intro to two
    MD
    j="{{just_executable()}} --set decisions_file $dir/fixture.md"

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

    # The duplicate-number check, against fixtures for the same reason: the real
    # file has no duplicate today, so pinning it there would prove nothing.
    $j check-decisions >/dev/null \
        || { echo "the clean fixture failed check-decisions"; exit 1; }

    cat > "$dir/twice.md" <<'MD'
    ## 1. First
    ## 1. First again
    MD
    if {{just_executable()}} --set decisions_file "$dir/twice.md" \
        check-decisions >/dev/null 2>&1; then
        echo "two sections numbered 1 passed check-decisions"; exit 1
    fi

    cat > "$dir/twice-sub.md" <<'MD'
    ## 1. First
    ### 1.1 Alpha
    ### 1.1 Alpha again
    MD
    if {{just_executable()}} --set decisions_file "$dir/twice-sub.md" \
        check-decisions >/dev/null 2>&1; then
        echo "two subsections numbered 1.1 passed check-decisions"; exit 1
    fi

    # Descending: §1 lands after §2, which a renumbering that skips one section
    # leaves behind.
    cat > "$dir/descending.md" <<'MD'
    ## 2. Second
    ## 1. First
    MD
    if {{just_executable()}} --set decisions_file "$dir/descending.md" \
        check-decisions >/dev/null 2>&1; then
        echo "## 1. after ## 2. passed check-decisions"; exit 1
    fi

    # Descending within a section: §1.1 lands after §1.2.
    cat > "$dir/descending-sub.md" <<'MD'
    ## 1. First
    ### 1.2 Beta
    ### 1.1 Alpha
    MD
    if {{just_executable()}} --set decisions_file "$dir/descending-sub.md" \
        check-decisions >/dev/null 2>&1; then
        echo "### 1.1 after ### 1.2 passed check-decisions"; exit 1
    fi

    # Wrong parent: §1.2 is left under §2 by a renumbering, exactly the way
    # `### 64.3` ended up under `## 69.` in the real file.
    cat > "$dir/wrong-parent.md" <<'MD'
    ## 1. First
    ### 1.1 Alpha
    ## 2. Second
    ### 1.2 Beta
    MD
    if {{just_executable()}} --set decisions_file "$dir/wrong-parent.md" \
        check-decisions >/dev/null 2>&1; then
        echo "### 1.2 under ## 2. passed check-decisions"; exit 1
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

# The five fuzz recipes the overnight run takes, in the order it takes them.
#
# Recipe names rather than fuzz-target names, because what each target wants on
# its command line is the recipe's business and not this list's.
fuzz_all := "fuzz fuzz-reader fuzz-writer fuzz-fuse fuzz-cones-and-tori"

# Where a night's logs go. One directory per date, plus a one-line `latest`.
fuzz_log_root := env_var_or_default("RCAD_FUZZ_LOGS", env_var("HOME") / "rcad-fuzz-logs")

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

# Fuzz every target in turn, inside a fixed window, unattended.
#
# `window` is seconds, and it is a *deadline* and not a budget: each target gets
# an equal share of whatever is left when its turn comes, so a slow build or a
# target that stops early is paid for out of the same window rather than added
# to the end of it. The 08:10 in `rcad-fuzz.timer` is that deadline, and it is
# real because the arithmetic is done against the clock each time round.
#
# A crash does not end the night. `cargo fuzz` exits non-zero and writes the
# input to `fuzz/artifacts/<target>/` (ignored, so it survives a `git clean`
# without dirtying anything), and a crash in the reader is no reason to cost the
# fuse its hours — so each target is reported and the run carries on, failing at
# the end if anything failed.
#
# Same nightly-compiler caveat as `fuzz` below.
[doc("Fuzz all five targets in turn, inside a window in seconds. What the timer runs.")]
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
        echo "nightly is not pinned by this repo (docs/decisions.md §29.9); \`rustup toolchain install nightly\` is the usual answer"
        printf '%s  BUILD FAILED  %s\n' "$(date '+%Y-%m-%d %H:%M')" "$night" >"{{fuzz_log_root}}/latest"
        exit 1
    fi

    recipes=({{fuzz_all}})
    left=${#recipes[@]}
    to_look_at=()
    for recipe in "${recipes[@]}"; do
        slice=$(( (deadline - $(date +%s)) / left ))
        left=$(( left - 1 ))
        # No floor beyond "there is time left". A minimum worth starting would
        # be a number nobody can defend — an 18 MB corpus takes seconds just to
        # load, so the honest floor depends on the target — and it would make
        # the window untestable at any size a test can afford to wait for.
        if [ "$slice" -lt 1 ]; then
            echo "$recipe: skipped, $slice s left in the window" | tee -a "$night/$recipe.log"
            to_look_at+=("$recipe:no-time")
            continue
        fi
        echo "$recipe: $slice s"
        # libFuzzer's own `-timeout` bounds one input and not the process, and a
        # target wedged outside the fuzzer's control would eat the whole night.
        # The grace is for the corpus write on the way out.
        status=0
        timeout --kill-after=60 "$(( slice + 300 ))" \
            {{just_executable()}} "$recipe" "$slice" "{{fuzz_corpus_root}}" \
            >"$night/$recipe.log" 2>&1 || status=$?
        case "$status" in
            0) ;;
            124|137)
                echo "$recipe: overran its slice and was killed"
                to_look_at+=("$recipe:overran") ;;
            *)
                echo "$recipe: exited $status — see $night/$recipe.log"
                to_look_at+=("$recipe:crash") ;;
        esac

        # The input a target found is kept beside the log that describes it,
        # because `fuzz/artifacts/` is the wrong place to leave the only copy of
        # anything (§91). Two reasons, and each one lost a finding.
        #
        # **libFuzzer never reads that directory back as a seed.** So the next
        # night starts from a corpus without the crash in it, rolls a fresh seed,
        # and does not re-find it: on 2026-09-13 a night reported "nothing found"
        # on the very commit the night before had crashed on, and nothing had
        # been fixed in between.
        #
        # **It lives in this worktree, and `.gitignore` hides it.** §89's slow
        # unit sat there as its only copy for three days in a tree that had been
        # unlocked and was due to be reclaimed.
        #
        # Copied after each target rather than once at the end, so a night killed
        # from outside still keeps what the targets that did run found. `-n`
        # makes the repeat cheap and keeps the first copy of a name that libFuzzer
        # rewrites.
        if [ -d fuzz/artifacts ]; then
            mkdir -p "$night/artifacts"
            cp -Rn fuzz/artifacts/. "$night/artifacts/"
        fi
    done

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
#     green gate answers it, where in §81.4 nothing did.
#   - **on `main`**, so a green run for some other branch cannot move it.
#   - **forward only**, so a tree somebody advanced by hand is never dragged
#     back, and neither is one already past the last green.
#
# Separate from `fuzz-overnight` because that recipe's selftest runs it for real
# in whatever worktree the gate was run from, and a checkout in there would move
# somebody's tree out from under them.
[doc("Which commit the overnight run should fuzz: the newest CI-green commit on main.")]
fuzz-watermark:
    #!/usr/bin/env bash
    set -euo pipefail

    staying() { echo "staying on $(git rev-parse --short HEAD): $1" >&2; exit 0; }

    git diff --quiet && git diff --cached --quiet \
        || staying "the tree has uncommitted changes"
    command -v gh >/dev/null 2>&1 \
        || staying "gh is not on PATH, so no run's conclusion can be read"

    green=$(gh run list --workflow ci.yml --branch main --status success \
        --limit 20 --json headSha --jq '.[].headSha' 2>/dev/null) \
        || staying "gh could not list the runs on main"
    [ -n "$green" ] || staying "gh lists no green run on main"

    # Fetched only once there is something to weigh against it, so that a gate
    # running this recipe does not need the network to pass — §77.6's shape, a
    # check that would be answering a question about its machine.
    git fetch --quiet origin 2>/dev/null \
        || staying "origin could not be reached"

    for sha in $green; do
        git cat-file -e "${sha}^{commit}" 2>/dev/null || continue
        git merge-base --is-ancestor "$sha" origin/main || continue
        git merge-base --is-ancestor HEAD "$sha" || continue
        [ "$(git rev-parse "$sha")" != "$(git rev-parse HEAD)" ] \
            || staying "already on the last green"
        echo "$sha"
        exit 0
    done
    staying "no green run on main is ahead of it"

# The property fuzzer, over a pair of surfaces (docs/decisions.md §24.14).
#
# NOT part of `just gate`, and not because anybody chose to leave it out:
# `cargo-fuzz` builds with a sanitizer that needs a nightly compiler and
# `rust-toolchain.toml` pins a stable one, so this is the one recipe here that
# uses a compiler the repo does not pin. What runs on every commit is the same
# property over the same decoder, in
# `crates/rcad-geom/tests/every_pair_of_surfaces_the_fuzzer_draws_is_named_honestly.rs`.
[doc("Fuzz the predicates over a pair of surfaces. Needs a nightly compiler.")]
fuzz duration="60" corpus_root="": (fuzz-target "where_two_surfaces_cross" duration corpus_root "71")

# The Part 21 reader, over arbitrary bytes (docs/decisions.md §42.12 criterion 5).
#
# Same nightly-compiler caveat as `fuzz` above. What runs on every commit
# instead is `crates/rcad-step/tests/what_the_reader_refuses_and_by_what_name.rs`,
# which covers the shapes somebody thought of; this covers the ones nobody did.
#
# No `-max_len`, unlike `fuzz`: the reader's promise is about any length of
# anything, and the truncation cases live at every length there is.
[doc("Fuzz the Part 21 reader over arbitrary bytes.")]
fuzz-reader duration="60" corpus_root="": (fuzz-target "no_step_file_makes_the_reader_panic" duration corpus_root)

# The Part 21 writer, over an arbitrary `Model` (docs/decisions.md §42.12).
#
# Same nightly-compiler caveat as `fuzz` above. `Model` is `Deserialize`
# (`crates/rcad-topo/src/serialize.rs`), so this target draws bytes through
# `postcard::from_bytes` — the same route a stranger's saved file takes — and
# whatever `Model` comes out, sound or not, goes straight into
# `rcad_step::write`, which owes an `Err` and never a panic either way.
[doc("Fuzz the Part 21 writer over an arbitrary Model.")]
fuzz-writer duration="60" corpus_root="": (fuzz-target "no_model_makes_the_writer_panic" duration corpus_root)

# The fuse, over two arbitrary `Model`s (docs/decisions.md §17).
#
# Same nightly-compiler caveat as `fuzz` above. Decodes two `Model`s per run
# and, for each that passes `Model::validate`, fuses a byte-chosen split of
# its own solids against each other — see
# `fuzz/fuzz_targets/two_solids_fused_by_a_stranger.rs` for why two `Model`s
# and not one cross-fused pair.
[doc("Fuzz the fuse over two arbitrary Models.")]
fuzz-fuse duration="60" corpus_root="": (fuzz-target "two_solids_fused_by_a_stranger" duration corpus_root)

# A cone or a torus crossing another surface, over its own byte decoder.
#
# Same nightly-compiler caveat as `fuzz` above. Needs its own `-max_len`,
# like `fuzz` above and unlike the reader/writer/fuse targets: a torus-vs-torus
# pair is this decoder's worst case, at 1 byte of choice plus two 8-byte
# doubles for each of a torus's five numbers, so 81 and not `fuzz`'s 71.
[doc("Fuzz the predicates over a cone or a torus and another surface.")]
fuzz-cones-and-tori duration="60" corpus_root="": (fuzz-target "where_a_cone_or_a_torus_crosses" duration corpus_root "81")

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
[private]
fuzz-target target duration corpus_root="" max_len="":
    #!/usr/bin/env bash
    set -euo pipefail
    cd fuzz
    corpus=()
    if [ -n "{{corpus_root}}" ]; then
        mkdir -p "{{corpus_root}}/{{target}}"
        corpus=("{{corpus_root}}/{{target}}" "corpus/{{target}}")
    fi
    RUSTFLAGS="-C target-cpu=x86-64-v3" \
        cargo +nightly fuzz run {{target}} ${corpus[@]+"${corpus[@]}"} -- \
        -max_total_time={{duration}} {{ if max_len == "" { "" } else { "-max_len=" + max_len } }}

# Pins `fuzz-overnight` against a `cargo` it brings with it, because the real one
# wants a sanitizer build and eight hours before it says anything at all. The
# stub sleeps for exactly the time it was granted, so the clock the recipe
# divides is a real clock and the whole thing still runs in about thirteen
# seconds.
#
# Four things, and the first is the one that would otherwise cost a night in
# silence: **the window is a deadline, not a budget.** A recipe that handed each
# target the full window would pass every other assertion here and run four
# times as long as it was told to, which nobody would notice until the fuzzer
# was still going at lunch.
#
# The other three are each a way the night could quietly do nothing: a crash in
# one target taking the other three down with it, the run reporting success
# after a target died, and — the reason the hub case is here rather than trusted
# to `check-hub` — a timer left pointing at the one tree whose owner cannot be
# written down, where `just lanes` would show `free` all night (§60).
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
    trap 'rm -rf "$dir"; rm -f fuzz/artifacts/*/crash-"$tag"' EXIT
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
    printf '%s [timings=%s]\n' "$*" "${RCAD_FUZZ_TIMINGS:-unset}" >>"$STUB_LOG"
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
    if [ -n "$target" ] && [ -n "${STUB_ARTIFACT_TAG:-}" ]; then
        mkdir -p "artifacts/$target"
        echo "what a target found" >"artifacts/$target/crash-$STUB_ARTIFACT_TAG"
    fi

    for arg in "$@"; do
        case "$arg" in
            -max_total_time=*)
                half=$(( ${arg#*=} / 2 ))
                [ "$half" -ge 1 ] || half=1
                sleep "$(( half + ${STUB_OVERRUN:-0} ))"
                ;;
        esac
    done
    for arg in "$@"; do
        if [ "$arg" = "${STUB_CRASH:-}" ]; then exit 1; fi
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
    # dying. Every one still runs, the run ends inside the window rather than
    # five times past it, and it reports the failure rather than swallowing it.
    #
    # A correct recipe spends about 9 s here and the budget mutation about 24 s,
    # so 16 separates them with room on both sides. There is deliberately no
    # assertion on the sum of the granted slices: that sum legitimately exceeds
    # the window, because a target finishing early hands what it did not use to
    # the ones after it. It only looked like an invariant while the stub was
    # consuming every second it was given.
    : >"$dir/log"
    started=$(date +%s)
    status=0
    STUB_CRASH=no_step_file_makes_the_reader_panic run 12 || status=$?
    elapsed=$(( $(date +%s) - started ))
    [ "$status" -ne 0 ] || fail "a target exited 1 and the night reported success"
    ran=$(grep -c 'fuzz run' "$dir/log" || true)
    [ "$ran" -eq 5 ] || fail "a crash in the second target left $ran of 5 targets run"
    [ "$elapsed" -ge 4 ] || fail "a twelve-second window was over in $elapsed s, so nothing was fuzzed"
    [ "$elapsed" -le 16 ] || fail "a twelve-second window took $elapsed s, so the window is a budget and not a deadline"
    grep -q 'TO LOOK AT.*fuzz-reader' "$dir/logs/latest" \
        || fail "the summary line does not name the target that died"

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
        [ -f "$night/artifacts/$target/crash-$tag" ] \
            || fail "$target's artifact was left in the worktree instead of kept in $night/artifacts"
    done

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

    # Four: when a target overruns, what is left of the window is what the rest
    # get — and when that is nothing, they are skipped and said to be skipped,
    # rather than each starting a fresh hour past the deadline.
    #
    # The overrun has to clear the whole window on its own, or the arithmetic
    # leaves a second behind and the last target runs on it, which is right and
    # is not what this case is about. Six and eight, so the first target is nine
    # seconds into a six-second window when it returns.
    : >"$dir/log"
    started=$(date +%s)
    status=0
    STUB_OVERRUN=8 run 6 || status=$?
    elapsed=$(( $(date +%s) - started ))
    [ "$status" -ne 0 ] || fail "three targets were skipped and the night reported success"
    [ "$elapsed" -le 18 ] || fail "an overrun in the first target let the rest run on for $elapsed s"
    [ "$(grep -c 'fuzz run' "$dir/log")" -eq 1 ] \
        || fail "the window was spent by the first target and the rest ran anyway"

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
        reason=$(mktemp)
        (cd "$work" && mise exec -- cargo run -q -p rcad-fuzz-explain -- "$target" "$file") 2>"$reason"
        decoded=$?
        if [ "$decoded" -eq 0 ]; then
            echo "still fails on {{ against }}: no — it decoded and replayed clean"
        else
            still=$((still + 1))
            echo "still fails on {{ against }}: exit $decoded, it did not come back clean — whether that is a failed decode or a failed replay is on stderr:"
            sed 's/^/    /' "$reason"
        fi
        rm -f "$reason"
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

[doc("The benchmarks.")]
bench:
    cargo bench --workspace

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
    echo "=== lanes ==="
    {{ just_executable() }} lanes '{{dir}}'

    echo
    echo "=== branches carrying work main does not have ==="
    {{ just_executable() }} _status-branches '{{dir}}' main

    echo
    echo "=== main against the last green ==="
    if ! command -v gh >/dev/null 2>&1; then
        echo "unknown: gh is not on PATH, so no run's conclusion can be read"
    else
        green=$(gh run list --workflow ci.yml --branch main --status success \
            --limit 1 --json headSha --jq '.[0].headSha' 2>/dev/null)
        if [ -z "$green" ]; then
            echo "unknown: gh lists no green run on main"
        elif ! git -C '{{dir}}' cat-file -e "${green}^{commit}" 2>/dev/null; then
            echo "last green $(cut -c1-7 <<<"$green") is not in this checkout; fetch to weigh main against it"
        else
            ahead=$(git -C '{{dir}}' rev-list --count "$green"..main 2>/dev/null)
            printf 'main tip %s, last green %s, %s commit(s) on main not yet green\n' \
                "$(git -C '{{dir}}' rev-parse --short main)" "$(cut -c1-7 <<<"$green")" "${ahead:-?}"
        fi
    fi

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
# screen, which is the length something gets re-read at.
[doc("docs/decisions.md §0 exists, parses, and points only at sections that do.")]
check-where-the-work-is:
    #!/usr/bin/env bash
    set -uo pipefail
    {{ just_executable() }} _where-the-work-is-check '{{ decisions_file }}'

# Held apart from the recipe above so the selftest can run it over fixtures
# whose answer is known, rather than over whatever §0 happens to say today.
[private]
_where-the-work-is-check file:
    #!/usr/bin/env bash
    set -uo pipefail
    file='{{file}}'

    block=$(awk '/^## 0\. /{on=1} on && /^## [1-9]/{exit} on{print}' "$file")
    if [ -z "$block" ]; then
        echo "$file has no '## 0.' section, so nothing says where the work is"
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
