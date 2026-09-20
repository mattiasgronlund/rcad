#!/usr/bin/env bash
#
# Split a single-file `docs/decisions.md` into one file per `## N.` section.
#
#     tooling/decisions-split/split.sh [<decisions.md>] [<outdir>]
#
# Defaults to `docs/decisions.md` and `docs/decisions.d`. The text above the
# first `## ` heading becomes `00-preamble.md`; section N becomes
# `NNN-<slug>.md`, zero-padded to three digits. Nothing is reflowed, reordered
# or rewritten: every byte of the input lands in exactly one output file, in
# order, so `just decisions-all` concatenates back to the input byte for byte.
# That equality is the whole correctness argument for the split (§179).
#
# **This is committed because `main` moves.** Sections land in this repository
# daily, and a lane that splits the file at one tip is stale by the time it
# merges. Whoever merges re-runs this against the new tip rather than redoing
# the split by hand — which is also why the preamble is written last, after the
# old files are cleared, so a re-run cannot leave a section from the old tip
# behind under a slug the new one no longer uses.
set -euo pipefail

src=${1:-docs/decisions.md}
dir=${2:-docs/decisions.d}

[ -f "$src" ] || { echo "$src is not a file" >&2; exit 2; }

sections=$(grep -cE '^## [0-9]+\. ' "$src" || true)
if [ "$sections" -eq 0 ]; then
    echo "$src holds no '## N.' section, so there is nothing to split" >&2
    echo "^ it is already the pointer; split the tip's single file instead" >&2
    exit 2
fi

mkdir -p "$dir"
rm -f "$dir"/*.md

awk -v dir="$dir" '
    function shorten(s,   i) {
        if (length(s) <= 48) return s
        s = substr(s, 1, 48)
        i = length(s)
        while (i > 1 && substr(s, i, 1) != "-") i--
        if (i > 1) s = substr(s, 1, i - 1)
        sub(/-+$/, "", s)
        return s
    }
    BEGIN { out = dir "/00-preamble.md" }
    /^## [0-9]+\. / {
        close(out)
        rest = substr($0, 4)
        p = index(rest, " ")
        num = substr(rest, 1, p - 1)
        title = substr(rest, p + 1)
        slug = tolower(title)
        gsub(/[^a-z0-9]+/, "-", slug)
        sub(/^-+/, "", slug)
        sub(/-+$/, "", slug)
        slug = shorten(slug)
        if (slug == "") slug = "section"
        out = sprintf("%s/%03d-%s.md", dir, num + 0, slug)
        if (out in seen) {
            printf "two sections would be written to %s\n", out > "/dev/stderr"
            exit 1
        }
        seen[out] = 1
    }
    { print > out }
' "$src"

wrote=$(ls "$dir" | wc -l)
echo "$src -> $dir: $wrote files ($sections sections and the preamble)"
