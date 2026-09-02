# A wildcard arm over an array, a tuple, or a reference of our own enums
# (docs/decisions.md §40), the half `clippy::wildcard_enum_match_arm` cannot
# see: that lint only asks what the scrutinee's own type is, and the
# scrutinee of `match (first, second) { ... }` is a tuple, not `Surface`, so
# it asks nothing about what is nested inside. Called from `just
# check-wildcards`, with the list of our own enum names passed in as `enums`
# (pipe-separated) and one or more `.rs` files as arguments.
#
# What it looks for, per match: an arm whose pattern opens with `[`, `(`, or
# `&` immediately followed by one of our enum names and `::` — naming a
# variant of ours inside a composite — standing in the same match as a bare
# `_` arm. Line-based, not a parse: a match is tracked from the line holding
# the word `match` to the line its brace depth returns to zero, and an arm is
# read off its own first line after leading whitespace is stripped. Comment
# lines are skipped outright, because a doc comment that happens to use the
# word "match" would otherwise open a match this file never closes.
BEGIN {
    n = split(enums, names, "|")
    for (i = 1; i <= n; i++) want[names[i]] = 1
    state = "idle"
}
FNR == 1 { state = "idle" }
{
    trimmed = $0
    sub(/^[ \t]*/, "", trimmed)
    if (trimmed ~ /^(\/\/|\*)/) next
}
state == "idle" {
    if ($0 !~ /\<match\>/) next
    state = "header"
    match_line = FNR
}
state == "header" {
    if (index($0, "{") == 0) next
    state = "body"
    depth = 0
    saw_composite = 0
    saw_wildcard = 0
}
state == "body" {
    tmp1 = $0; opens = gsub(/\{/, "{", tmp1)
    tmp2 = $0; closes = gsub(/\}/, "}", tmp2)
    depth += opens - closes
    line = $0
    sub(/^[ \t]*/, "", line)
    if (match(line, /^(\[&?|\(&?|&)[A-Za-z_][A-Za-z0-9_]*::/)) {
        name = line
        sub(/^[\[(&]+/, "", name)
        sub(/::.*/, "", name)
        if (name in want) saw_composite = 1
    }
    if (line ~ /^_[ \t]*(=>|,|if[ \t])/) saw_wildcard = 1
    if (depth <= 0) {
        if (saw_composite && saw_wildcard) {
            print FILENAME ":" match_line
            count++
        }
        state = "idle"
    }
}
END { print "COUNT=" count + 0 }
