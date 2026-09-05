#!/usr/bin/env bash
# Runs the status line against its cases. It is checked because it is the one
# piece of this that nothing else would notice going wrong: it renders below the
# prompt, it never fails a build, and a miscount reads exactly like a correct
# count of a different number.
set -uo pipefail

here=$(dirname "$(readlink -f "$0")")
line=$(dirname "$here")/statusline.sh
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
mkdir -p "$work/docs"

failed=0
total=0

render() {
	jq -n --arg d "$work" --argjson t "$1" \
		'{model: {display_name: "Opus"}, context_window: {total_input_tokens: $t}, workspace: {project_dir: $d}}' |
		CLAUDE_CONTEXT_BUDGET=300000 "$line" | sed -e 's/\x1b\[[0-9;]*m//g'
}

want() {
	local why=$1 tokens=$2 pattern=$3
	total=$((total + 1))
	local got
	got=$(render "$tokens")
	if ! grep -qE "$pattern" <<<"$got"; then
		printf 'wanted /%s/, got: %s\n  (%s)\n' "$pattern" "$(tr '\n' '|' <<<"$got")" "$why"
		failed=$((failed + 1))
	fi
}

reject() {
	local why=$1 tokens=$2 pattern=$3
	total=$((total + 1))
	local got
	got=$(render "$tokens")
	if grep -qE "$pattern" <<<"$got"; then
		printf 'did not want /%s/, got: %s\n  (%s)\n' "$pattern" "$(tr '\n' '|' <<<"$got")" "$why"
		failed=$((failed + 1))
	fi
}

want "a missing file is said plainly, not crashed on" 1000 'no docs/open-questions.md'

# The file documents its own format in a fenced example. An example is not a
# question, and counting it was a real bug.
cat >"$work/docs/open-questions.md" <<'FILE'
# Open questions

```
- [ ] **Q11** — Short question in one line? → *recommend (a)*
```

## Waiting

<!-- none right now -->

## Answered

- [x] **Q0** — Was this asked? → answered: yes
FILE
want "an empty Waiting section says nothing is waiting" 1000 'nothing waiting on you'
reject "the fenced example is not counted" 1000 'open'
reject "an answered row is not counted" 1000 'Q0'

cat >"$work/docs/open-questions.md" <<'FILE'
## Waiting

- [ ] **Q11** — Run the sweep now or after the merge? → *recommend now*
- [ ] **Q12** — Bump the pin before the cap? → *recommend after*
- [ ] **Q13** — Does the section stay open? → *recommend yes*
- [ ] **Q14** — Push tonight? → *recommend yes*

## Answered

- [x] **Q0** — Was this asked? → answered: yes
FILE
want "four open are counted as four" 1000 '4 open'
want "only three are shown, the rest counted" 1000 '\+1 more'
want "the recommendation is not shown, only the question" 1000 'Q11 — Run the sweep now'
reject "the arrow and its argument stay in the file" 1000 'recommend now'

want "under budget shows the percentage" 150000 '150k/300k .* 50%'
reject "under budget does not say hand off" 150000 'hand off'
want "over budget says hand off" 340000 'hand off'
want "over budget passes 100%" 340000 '11[0-9]%'

if [ "$failed" -gt 0 ]; then
	echo "$failed of $total cases came out wrong"
	exit 1
fi

echo "$total cases, all as expected"
