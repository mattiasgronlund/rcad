#!/usr/bin/env bash
# Runs decisions-zero.sh against fixtures whose answer is known, the same
# shape as check-worktree-create.sh: a real git repo it builds itself, so the
# hook is exercised end to end rather than trusted on its own say-so.
#
# It copies this tree's own `justfile` into that fixture repo rather than
# reimplementing the cap, the reviewed-date check or the citation walk —
# §178.3's rule against two copies of a rule kept in step by a comment applies
# to a selftest fixture as much as to the hook itself. The copy is exercised,
# not trusted: `check-decisions-selftest` already pins `_where-the-work-is-check`
# against fixtures of its own, so this selftest's job is only the hook's own
# behaviour around that recipe — matching, silence, the block, the "just not
# found" case — not the awk a second time.
#
# Runs unconditionally, with no plugin skip: none of this needs Claude Code,
# and it must run on CI the same as here (docs/decisions.md §178.6).
set -uo pipefail

here=$(dirname "$(readlink -f "$0")")
hook="$here/decisions-zero.sh"
repo_root=$(git -C "$here" rev-parse --show-toplevel)

if ! command -v just >/dev/null 2>&1; then
	echo "check-decisions-zero.sh needs 'just' on PATH to build its fixture repo" >&2
	exit 1
fi

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

repo="$work/rcad"
mkdir -p "$repo/docs/decisions.d"
cp "$repo_root/justfile" "$repo/justfile"
git init -q -b main "$repo"
git -C "$repo" -c user.email=t@t -c user.name=t add -A
git -C "$repo" -c user.email=t@t -c user.name=t commit -q -m root

zero="$repo/docs/decisions.d/000-where-the-work-is.md"

write_zero() { cat >"$zero"; }

payload() {
	printf '{"tool_input":{"file_path":"%s"}}' "$1"
}

failed=0
total=0
wrong() {
	printf '%s\n' "$1"
	failed=$((failed + 1))
}

# A clean, deliberately un-saturated §0: 3 non-blank lines, well under the cap,
# so the headroom line names a real number rather than the real repo's current
# 0 (§0 is saturated there today, which is a fine demo but a bad fixture — the
# next section closed would flip it to a different count for no reason this
# selftest controls).
write_zero <<'MD'
## 0. Where the work is

reviewed: 2026-09-20

**unit:** the thing.

## 1. Something else
MD

# The whole field, not a substring of it. `additionalContext` is the one thing
# this hook puts in front of a model, so "contains the headroom line" is the
# wrong question — it passes just as happily when the line arrives with three
# lines of junk stapled to it, which is exactly what this fixture used to do
# and nothing noticed. The fixture repo has no `tooling/hub-align/align.sh`,
# so `hub_aligned` writes `No such file or directory` to stderr three times on
# every run through here; keeping that fixture unfixed is the point, because
# it makes this an assertion that the hook drops stderr rather than a wish.
want_clean='`just check-where-the-work-is` ran on 000-where-the-work-is.md as part of this write: §0 is 3 of 50 non-blank lines, reviewed 2026-09-20, every section it cites exists.'

total=$((total + 1))
out=$(payload "$zero" | "$hook" 2>"$work/err")
status=$?
got=$(jq -r '.hookSpecificOutput.additionalContext // empty' <<<"$out" 2>/dev/null)
if [ "$status" -ne 0 ]; then
	wrong "a clean §0 exited $status, wanted 0 — stderr: $(cat "$work/err")"
elif [ -z "$got" ]; then
	wrong "a clean §0 did not carry additionalContext: $out"
elif [ "$got" != "$want_clean" ]; then
	wrong "$(printf 'a clean §0 did not carry exactly the §178.4 line:\n--- got ---\n%s\n--- want ---\n%s' "$got" "$want_clean")"
fi

# Three lines over the cap: the heading, `reviewed:`, and 51 filler lines is
# 53 non-blank lines, 3 over 50.
{
	printf '## 0. Where the work is\n\nreviewed: 2026-09-20\n\n'
	for _ in $(seq 51); do printf 'filler\n'; done
	printf '\n## 1. Something else\n'
} >"$zero"

total=$((total + 1))
out=$(payload "$zero" | "$hook" 2>"$work/err")
status=$?
if [ "$status" -ne 0 ]; then
	wrong "an over-cap §0 exited $status, wanted 0 (a block is a JSON decision, not a nonzero exit) — stderr: $(cat "$work/err")"
elif ! grep -qF '"decision": "block"' <<<"$out"; then
	wrong "an over-cap §0 did not block: $out"
elif ! grep -qF 'cut 3 non-blank lines' <<<"$out"; then
	wrong "an over-cap §0's block reason did not name the cut: $out"
fi

# A write to some other file in the same directory: silent, exit 0.
other="$repo/docs/decisions.d/050-something.md"
printf '## 50. Something\n' >"$other"

total=$((total + 1))
out=$(payload "$other" | "$hook" 2>"$work/err")
status=$?
if [ "$status" -ne 0 ]; then
	wrong "a write to $other exited $status, wanted 0"
elif [ -n "$out" ]; then
	wrong "a write to $other was not silent: $out"
fi

# `just` missing from PATH: reported on stderr with exit 2, never swallowed —
# the same exposure check-question-file.sh's "checker not found" case covers.
# Built by dropping only the PATH entries that hold a `just`, so jq, git and
# every other tool the hook needs stay reachable.
nojust=""
IFS=':' read -r -a dirs <<<"$PATH"
for d in "${dirs[@]}"; do
	[ -x "$d/just" ] && continue
	nojust="$nojust:$d"
done
nojust="${nojust#:}"

total=$((total + 1))
out=$(payload "$zero" | PATH="$nojust" "$hook" 2>"$work/err")
status=$?
if [ "$status" -ne 2 ]; then
	wrong "with no 'just' on PATH the hook exited $status, wanted 2"
elif [ -n "$out" ]; then
	wrong "with no 'just' on PATH the hook wrote to stdout, which a caller reads as a decision: $out"
elif ! grep -qF 'not on PATH' "$work/err"; then
	wrong "with no 'just' on PATH the hook did not say so on stderr: $(cat "$work/err")"
fi

if [ "$failed" -gt 0 ]; then
	echo "$failed of $total checks came out wrong"
	exit 1
fi
echo "$total checks, all as expected"
