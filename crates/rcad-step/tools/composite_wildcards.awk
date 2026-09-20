# A wildcard arm over an array, a tuple, or a reference of our own enums
# (docs/decisions.md §40), the half `clippy::wildcard_enum_match_arm` cannot
# see: that lint only asks what the scrutinee's own type is, and the
# scrutinee of `match (first, second) { ... }` is a tuple, not `Surface`, so
# it asks nothing about what is nested inside. Called from `just
# check-wildcards`, with the list of our own enum names passed in as `enums`
# (pipe-separated) and one or more `.rs` files as arguments.
#
# The proper mechanism for this is rustc's own
# `non_exhaustive_omitted_patterns` lint, which sees exactly what a text scan
# cannot -- but it is unstable on the 1.96 this repo pins
# (`rust-toolchain.toml`), so this is a stand-in for it and not the real
# thing.
#
# What counts, per match: not the match itself, but how many of a named
# enum's own variants the match never names anywhere in its arms, for every
# enum a composite arm names at least one variant of, in the same match as a
# bare `_`. A match that names every variant the enum has today counts zero,
# whatever its wildcard is there to catch tomorrow -- so this moves for a new
# *variant* the same way the direct half already does, which counting sites
# alone never did (`docs/decisions.md` §43.8 item 8).
#
# Line-based, not a parse: a match is tracked from the line holding the word
# `match` to the line its brace depth returns to zero, and an arm is read off
# its own first line after leading whitespace is stripped. So is an enum's own
# body, from `pub enum Name {` to its matching `}`, one top-level line per
# variant. Comment lines are skipped outright, and so is the inside of a
# string literal on any other line -- both because either can hold `{`, `}`
# or a word that looks like a variant, and neither is code.
#
# A name and not a path: `total["Error"]` is one number, and four crates each
# have their own `pub enum Error`. Whichever this scan reads last for a given
# name is the one every match against that name is judged against. Harmless
# today -- nothing in the tree matches a composite of `Error` -- and worth
# knowing before it is not: `check-wildcards-selftest`'s own `Toy` is picked
# to collide with nothing real for exactly this reason.
BEGIN {
    n = split(enums, names, "|")
    for (i = 1; i <= n; i++) want[names[i]] = 1

    # First pass: how many variants each of our enums actually has, read off
    # its own definition before a single match is judged against it. Reads
    # every file twice on purpose -- once here, once more as the main loop
    # below -- because a match earlier in argument order than the enum it
    # names can only be judged against a total known in advance.
    for (a = 1; a < ARGC; a++) {
        file = ARGV[a]
        ec_state = "idle"
        ec_in_attr = 0
        while ((getline ec_line < file) > 0) {
            ec_stripped = ec_line
            gsub(/"([^"\\]|\\.)*"/, "", ec_stripped)
            ec_trimmed = ec_stripped
            sub(/^[ \t]*/, "", ec_trimmed)
            if (ec_trimmed ~ /^(\/\/|\*)/) continue
            if (ec_in_attr) {
                t1 = ec_stripped; o = gsub(/\[/, "[", t1)
                t2 = ec_stripped; c = gsub(/\]/, "]", t2)
                ec_attr_depth += o - c
                if (ec_attr_depth <= 0) ec_in_attr = 0
                continue
            }
            if (ec_trimmed ~ /^#\[/) {
                t1 = ec_stripped; o = gsub(/\[/, "[", t1)
                t2 = ec_stripped; c = gsub(/\]/, "]", t2)
                ec_attr_depth = o - c
                if (ec_attr_depth > 0) ec_in_attr = 1
                continue
            }
            if (ec_state == "idle") {
                if (ec_line ~ /pub enum [A-Za-z0-9_]+/) {
                    ec_name = ec_line
                    sub(/.*pub enum /, "", ec_name)
                    sub(/[^A-Za-z0-9_].*/, "", ec_name)
                    if (ec_name in want) {
                        ec_state = "header"
                        ec_current = ec_name
                    } else {
                        continue
                    }
                } else {
                    continue
                }
            }
            if (ec_state == "header") {
                if (index(ec_line, "{") == 0) continue
                ec_state = "body"
                ec_depth = 1
                ec_count = 0
                continue
            }
            if (ec_state == "body") {
                if (ec_depth == 1 && ec_trimmed ~ /^[A-Z][A-Za-z0-9_]*[ \t]*[,({]?/) ec_count++
                t1 = ec_stripped; o = gsub(/\{/, "{", t1)
                t2 = ec_stripped; c = gsub(/\}/, "}", t2)
                ec_depth += o - c
                if (ec_depth <= 0) {
                    total[ec_current] = ec_count
                    ec_state = "idle"
                }
            }
        }
        close(file)
    }

    state = "idle"
}
FNR == 1 { state = "idle"; in_attr = 0 }
{
    stripped = $0
    gsub(/"([^"\\]|\\.)*"/, "", stripped)
    trimmed = stripped
    sub(/^[ \t]*/, "", trimmed)
    if (trimmed ~ /^(\/\/|\*)/) next
    if (in_attr) {
        t1 = stripped; o = gsub(/\[/, "[", t1)
        t2 = stripped; c = gsub(/\]/, "]", t2)
        attr_depth += o - c
        if (attr_depth <= 0) in_attr = 0
        next
    }
    if (trimmed ~ /^#\[/) {
        t1 = stripped; o = gsub(/\[/, "[", t1)
        t2 = stripped; c = gsub(/\]/, "]", t2)
        attr_depth = o - c
        if (attr_depth > 0) in_attr = 1
        next
    }
}
state == "idle" {
    # Spelled out rather than as `\<match\>`, which is gawk's alone: mawk is
    # Debian's default `awk` and every container this gate runs in has it, and
    # there `\<` is just a literal `<`, so no line ever opens a match and the
    # whole scan answers 0 without failing. `docs/decisions.md` §173.
    if ($0 !~ /(^|[^A-Za-z0-9_])match([^A-Za-z0-9_]|$)/) next
    state = "header"
    match_line = FNR
}
state == "header" {
    if (index($0, "{") == 0) next
    state = "body"
    depth = 0
    saw_composite = 0
    saw_wildcard = 0
    delete named
}
state == "body" {
    t1 = stripped; opens = gsub(/\{/, "{", t1)
    t2 = stripped; closes = gsub(/\}/, "}", t2)
    depth += opens - closes

    if (match(trimmed, /^(\[&?|\(&?|&)[A-Za-z_][A-Za-z0-9_]*::/)) {
        name = trimmed
        sub(/^[\[(&]+/, "", name)
        sub(/::.*/, "", name)
        if (name in want) saw_composite = 1
    }
    if (trimmed ~ /^_[ \t]*(=>|,|if[ \t])/) saw_wildcard = 1

    # Only a line that opens a pattern -- an array, a tuple, a reference, a
    # bare wildcard, or an `|` continuation of one -- is scanned for the
    # variants it names. A line that is arm *body*, reached once a pattern's
    # own `=>` or an earlier line already closed it, is not a pattern and is
    # not read as one: `Point::origin()` two lines into an arm's body is a
    # struct's associated function, not a variant of the unrelated
    # `rcad_predicates::Point` enum that happens to share its name.
    if (trimmed ~ /^([\[(&_]|\|)/) {
        scan = stripped
        while (match(scan, /[A-Za-z_][A-Za-z0-9_]*::[A-Za-z_][A-Za-z0-9_]*/)) {
            token = substr(scan, RSTART, RLENGTH)
            split(token, parts, "::")
            if (parts[1] in want) named[parts[1] "::" parts[2]] = 1
            scan = substr(scan, RSTART + RLENGTH)
        }
    }

    if (depth <= 0) {
        if (saw_composite && saw_wildcard) {
            delete seen
            for (key in named) {
                split(key, kv, "::")
                seen[kv[1]]++
            }
            uncovered = 0
            for (e in seen) {
                if ((e in total) && seen[e] < total[e]) uncovered += total[e] - seen[e]
            }
            if (uncovered > 0) {
                print FILENAME ":" match_line
                count += uncovered
            }
        }
        state = "idle"
    }
}
END { print "COUNT=" count + 0 }
