# `{{ args }}` interpolates a recipe's arguments into the shell *unquoted*, so a
# variadic recipe that passes them on splits them again on whitespace and hands
# the shell any punctuation they contain. That is not theoretical: `just t -E
# 'test(/walk/)'`, the example this file and CLAUDE.md both give, died on
# `Syntax error: "(" unexpected` from the day it was written, because `sh` saw
# the parentheses. With this setting the arguments arrive as `"$@"` with their
# original word boundaries, and `check-t-selftest` runs that exact example.
set positional-arguments

default: check check-enums check-wildcards check-wildcards-selftest check-ignored check-hooks check-lanes-selftest check-decisions check-decisions-selftest check-src-selftest lint test check-t-selftest deny

# The STEP corpus lives outside the tree, because it is other people's CAD files
# and this repo has no business carrying them. Four tests in `rcad-step` skip
# without it, so `just default` quietly runs four tests short.
#
# Derived rather than hardcoded so that every worktree gets the same answer:
# `--git-common-dir` points at the *main* repo's `.git` from inside a worktree,
# so two `parent_directory` calls land beside the repo wherever it is run from.
# Before this, lanes exported it by hand — 412 times on 2026-09-05 — and the
# ones that forgot got four silent skips rather than a failure.
export RCAD_STEP_CORPUS := env_var_or_default("RCAD_STEP_CORPUS", parent_directory(parent_directory(`git rev-parse --path-format=absolute --git-common-dir`)) / "rcad-step-corpus")

check:
    cargo check --workspace --all-targets

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
wildcard_arms := "91"

check-wildcards:
    @find crates -name lib.rs -path '*/src/*' -exec touch {} +
    @out=$(cargo clippy --workspace --all-targets -- \
            -W clippy::wildcard_enum_match_arm 2>&1) \
        || { echo "$out" | grep '^error' | head -5; \
             echo "clippy did not finish, so this counted nothing"; exit 1; }; \
    direct=$(echo "$out" | grep -c '^warning: wildcard match'); \
    files=$(find crates -name '*.rs' \( -path '*/src/*' -o -path '*/tests/*' \
            -o -path '*/benches/*' -o -path '*/examples/*' \)); \
    enums=$(grep -hoE '^[ \t]*pub enum [A-Za-z0-9_]+' $(find crates -path '*/src/*' -name '*.rs') \
            | awk '{print $NF}' | sort -u | paste -sd'|'); \
    composite=$(awk -v enums="$enums" -f crates/rcad-step/tools/composite_wildcards.awk $files \
            | grep '^COUNT=' | cut -d= -f2); \
    found=$((direct + composite)); \
    if [ "$found" != "{{wildcard_arms}}" ]; then \
        echo "wildcard arms over our own enums: $found ($direct direct, $composite composite), expected {{wildcard_arms}}"; \
        echo "a new one, or a new variant reaching an old one — see docs/decisions.md §40"; \
        exit 1; \
    fi

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
# `default`: today's one ignored test **fails**, which is why it is ignored, so
# a `default` that ran it would be red on purpose.
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
test-ignored:
    @if [ "{{ignored_tests}}" = "0" ]; then \
        echo "no test is marked #[ignore], so there is nothing here to run"; \
        exit 0; \
    fi; \
    cargo nextest run --workspace --run-ignored ignored-only --no-tests fail

# The hook that refuses shell patching of a .rs file, against its own cases
# (docs/decisions.md §50.8).
#
# It is in `default` because the hook fails in the direction that is hard to
# notice: a pattern that stops matching lets the thing through silently, which
# is the same failure mode the hook exists to prevent. Two of its cases are
# drafts of itself that were wrong — a commit message describing the rule, and
# a recipe whose own body writes .rs fixtures — and five more came from the
# 2026-09-05 session review: three shapes it let through, two it wrongly
# refused in the scratchpad.
check-hooks:
    @.claude/hooks/check-patching-hook.sh
    @.claude/hooks/check-context-budget.sh
    @.claude/hooks/check-statusline.sh
    @.claude/hooks/check-worktree-create.sh

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
# 17 000 lines under 487 headings, so the file is not a thing anyone holds at
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

# Pins the lookup against fixtures rather than against the real file, so it stays
# proven on a tree where the real §1.1 has been rewritten.
#
# `#### ` inside the fixture is the case worth having: it is a heading, and a
# lookup that stops at any heading would cut §1.1 in half without saying so.
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

test:
    cargo nextest run --workspace
    cargo test --workspace --doc

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
    # binaries, which is why `default` runs this recipe after `test`.
    got=$({{ just_executable() }} t -E 'test(/a_cube_comes_back_sound_and_measuring_the_same/)')
    case "$got" in
    *"1 test run: 1 passed"*)
        ;;
    *)
        echo "a parenthesised filter did not survive the recipe:"; echo "$got"; exit 1
        ;;
    esac

lint:
    cargo fmt --all -- --check
    taplo fmt --check
    cargo clippy --workspace --all-targets -- -D warnings

deny:
    cargo deny check

# The property fuzzer, over a pair of surfaces (docs/decisions.md §24.14).
#
# NOT part of `just default`, and not because anybody chose to leave it out:
# `cargo-fuzz` builds with a sanitizer that needs a nightly compiler and
# `rust-toolchain.toml` pins a stable one, so this is the one recipe here that
# uses a compiler the repo does not pin. What runs on every commit is the same
# property over the same decoder, in
# `crates/rcad-geom/tests/every_pair_of_surfaces_the_fuzzer_draws_is_named_honestly.rs`.
#
# RUSTFLAGS has to be repeated: cargo-fuzz sets it as an environment variable,
# and that shadows the `.cargo/config.toml` entry `inari` needs to build at all.
fuzz duration="60":
    cd fuzz && RUSTFLAGS="-C target-cpu=x86-64-v3" \
        cargo +nightly fuzz run where_two_surfaces_cross -- \
        -max_total_time={{duration}} -max_len=71

# The Part 21 reader, over arbitrary bytes (docs/decisions.md §42.12 criterion 5).
#
# Same nightly-compiler caveat as `fuzz` above, and the same repeated RUSTFLAGS.
# What runs on every commit instead is
# `crates/rcad-step/tests/what_the_reader_refuses_and_by_what_name.rs`, which
# covers the shapes somebody thought of; this covers the ones nobody did.
#
# No `-max_len`, unlike `fuzz`: the reader's promise is about any length of
# anything, and the truncation cases live at every length there is.
fuzz-reader duration="60":
    cd fuzz && RUSTFLAGS="-C target-cpu=x86-64-v3" \
        cargo +nightly fuzz run no_step_file_makes_the_reader_panic -- \
        -max_total_time={{duration}}

# The Part 21 writer, over an arbitrary `Model` (docs/decisions.md §42.12).
#
# Same nightly-compiler caveat as `fuzz` above. `Model` is `Deserialize`
# (`crates/rcad-topo/src/serialize.rs`), so this target draws bytes through
# `postcard::from_bytes` — the same route a stranger's saved file takes — and
# whatever `Model` comes out, sound or not, goes straight into
# `rcad_step::write`, which owes an `Err` and never a panic either way.
fuzz-writer duration="60":
    cd fuzz && RUSTFLAGS="-C target-cpu=x86-64-v3" \
        cargo +nightly fuzz run no_model_makes_the_writer_panic -- \
        -max_total_time={{duration}}

# The fuse, over two arbitrary `Model`s (docs/decisions.md §17).
#
# Same nightly-compiler caveat as `fuzz` above. Decodes two `Model`s per run
# and, for each that passes `Model::validate`, fuses a byte-chosen split of
# its own solids against each other — see
# `fuzz/fuzz_targets/two_solids_fused_by_a_stranger.rs` for why two `Model`s
# and not one cross-fused pair.
fuzz-fuse duration="60":
    cd fuzz && RUSTFLAGS="-C target-cpu=x86-64-v3" \
        cargo +nightly fuzz run two_solids_fused_by_a_stranger -- \
        -max_total_time={{duration}}

bench:
    cargo bench --workspace

fmt:
    cargo fmt --all
    taplo fmt

# Strip `target/` from every lane worktree. Always safe: kache holds the
# compilation, so a cleaned worktree refills from the cache instead of
# rebuilding, and nothing a lane wrote can be lost this way.
#
# Separate from `prune-worktrees` on purpose (docs/decisions.md §50.8). That one
# can only run once the work is merged; this one can run any time, and merging
# the two would make the safe half inherit the unsafe half's ceremony until
# nobody ran either. 38 worktrees held 166 GB of `target/` before this existed.
clean-targets:
    #!/usr/bin/env bash
    set -euo pipefail
    base="$(dirname "$(git rev-parse --show-toplevel)")/rcad-worktrees"
    if [ ! -d "$base" ]; then
        echo "no worktrees at $base, so there is nothing here to clean"
        exit 0
    fi
    before=$(du -sh "$base" | cut -f1)
    find "$base" -maxdepth 2 -type d -name target -prune -exec rm -rf {} +
    echo "$base: $before -> $(du -sh "$base" | cut -f1)"

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
prune-worktrees:
    #!/usr/bin/env bash
    set -euo pipefail
    repo=$(git rev-parse --show-toplevel)
    kept=0
    # A lock means an agent is working in there right now — git holds one for the
    # life of the agent and releases it on exit. Never force past it: the pid in
    # the lock reason was a live 13-hour session the first time this ran.
    locked=$(git -C "$repo" worktree list --porcelain \
        | awk '/^worktree /{tree = $2} /^locked/{print tree}')
    for tree in $(git -C "$repo" worktree list --porcelain | awk '/^worktree /{print $2}'); do
        [ "$tree" = "$repo" ] && continue
        branch=$(git -C "$tree" rev-parse --abbrev-ref HEAD 2>/dev/null || echo "")
        if grep -qxF "$tree" <<<"$locked"; then
            echo "kept (locked, an agent is using it): $tree"
            kept=$((kept + 1))
        elif [ -n "$(git -C "$tree" status --porcelain 2>/dev/null)" ]; then
            echo "kept (uncommitted changes): $tree"
            kept=$((kept + 1))
        elif [ -z "$branch" ] || ! git -C "$repo" merge-base --is-ancestor "$branch" main 2>/dev/null; then
            echo "kept (not merged into main): $tree [$branch]"
            kept=$((kept + 1))
        else
            git -C "$repo" worktree remove --force "$tree"
            echo "removed: $tree"
        fi
    done
    git -C "$repo" worktree prune
    echo "$kept worktree(s) kept; their branches are untouched either way"

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
    tree=""
    branch=""
    locked=""
    emit() {
        [ -n "$tree" ] || return 0
        total=$((total + 1))
        state="free"
        if [ -n "$locked" ]; then
            state="HELD"
            held=$((held + 1))
        fi
        if [ ! -d "$tree" ]; then
            work="missing"
        elif [ -n "$(git -C "$tree" status --porcelain 2>/dev/null)" ]; then
            work="dirty"
        else
            work="clean"
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
    if [ "$total" -eq 0 ]; then
        echo "no worktree was parsed out of a non-empty listing, so the parse is broken"
        exit 1
    fi
    echo "$held of $total worktree(s) held"

# Pins `lanes` against a repository it builds itself, so it stays proven on a
# tree where nothing real is locked.
#
# The three cases that matter are the three this builds: a **held** worktree
# whose reason must come back verbatim, a **free** one that must not be reported
# as held, and the **count** line, which is what stops a broken parse from
# looking like a quiet tree. Reading the live repository instead would prove
# nothing — it is unlocked almost all of the time, so a `lanes` that had lost the
# ability to see a lock at all would pass every run.
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
    grep -qx "1 of 3 worktree(s) held" <<<"$got" \
        || fail "the count line is wrong, so an empty parse could pass as a quiet tree"
