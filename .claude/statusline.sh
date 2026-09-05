#!/usr/bin/env bash
# Two lines that answer "how close am I to a handoff" and "what is waiting on
# me", without either question costing a token.
#
# This exists because the obvious way to check — `!cat` on the questions file —
# is not free. Shell mode adds its output to the session and Claude answers it:
# on 2026-09-05 that put 21 command outputs and 5,710 characters into context
# that nobody needed the model to read. The status line is terminal UI. It never
# reaches the model, so it can be re-read as often as you like.
#
# Line 1 tracks the same budget `context-budget.sh` enforces, so the handoff is
# visible on the way in rather than announced on arrival.
#
# Line 2 is the open question numbers and nothing else. The questions live in
# the session's own scratchpad, never in the tree: on 2026-09-05 four sessions
# were live across these two repos, and a tracked file would have had all four
# writing to it. Per session also means the numbering is the session's own, so
# "Q3-Q8" means what it says in the conversation you are looking at.
set -uo pipefail

payload=$(cat)
budget=${CLAUDE_CONTEXT_BUDGET:-300000}

get() { jq -r "$1 // empty" <<<"$payload" 2>/dev/null; }

model=$(get '.model.display_name')
used=$(get '.context_window.total_input_tokens')
root=$(get '.workspace.project_dir')
tree=$(get '.workspace.git_worktree')
[ -n "$root" ] || root=$(get '.cwd')
branch=$(git -C "$root" rev-parse --abbrev-ref HEAD 2>/dev/null)

dim=$'\033[2m'
off=$'\033[0m'
green=$'\033[32m'
yellow=$'\033[33m'
red=$'\033[31m'

# --- line 1: context against the handoff budget
line1="${model:-claude}"
[ -n "$tree" ] && line1="$line1 ${dim}·${off} $tree"
[ -n "$branch" ] && [ "$branch" != HEAD ] && line1="$line1 ${dim}·${off} $branch"

case "$used" in
'' | *[!0-9]*) used='' ;;
esac
if [ -n "$used" ]; then
	pct=$((used * 100 / budget))
	filled=$((pct / 10))
	[ "$filled" -gt 10 ] && filled=10
	[ "$filled" -lt 0 ] && filled=0
	colour=$green
	[ "$pct" -ge 70 ] && colour=$yellow
	[ "$pct" -ge 100 ] && colour=$red
	bar=""
	for i in $(seq 1 10); do
		if [ "$i" -le "$filled" ]; then bar="$bar▓"; else bar="$bar░"; fi
	done
	note=""
	[ "$pct" -ge 100 ] && note=" hand off"
	line1="$line1 ${dim}·${off} ${colour}$((used / 1000))k/$((budget / 1000))k ${bar} ${pct}%${note}${off}"
fi
printf '%s\n' "$line1"

# --- line 2: which questions are still open, by number
# The transcript sits at <projects>/<slug>/<session>.jsonl and the scratchpad at
# <tmp>/claude-<uid>/<slug>/<session>/scratchpad, so one names the other.
questions=${CLAUDE_QUESTIONS_FILE:-}
if [ -z "$questions" ]; then
	session=$(get '.session_id')
	transcript=$(get '.transcript_path')
	if [ -n "$session" ] && [ -n "$transcript" ]; then
		slug=$(basename "$(dirname "$transcript")")
		for base in "${TMPDIR:-/tmp}" /tmp; do
			candidate="$base/claude-$(id -u)/$slug/$session/scratchpad/open-questions.md"
			[ -r "$candidate" ] && questions=$candidate && break
		done
	fi
fi
[ -n "$questions" ] && [ -r "$questions" ] || exit 0

# The label is whatever letter and number sit right after the checkbox, not the
# first Q anywhere on the line: a body is free to mention M5 or Q7 in passing.
# The letter is kept because it is doing work — grilling numbers a round Q1, Q2,
# and a wave numbers a lane's findings by the lane's own letter, so K1-K4 came
# from "Lane K". Reading them as one undifferentiated series would lose that.
numbers=$(awk '
	/^[[:space:]]*```/ { fence = !fence; next }
	fence { next }
	/^- \[ \]/ {
		line = $0
		sub(/^- \[ \][[:space:]]*/, "", line)
		sub(/^\*+/, "", line)
		if (match(line, /^[A-Z]+[0-9]+/)) {
			label = substr(line, 1, RLENGTH)
			match(label, /[0-9]+/)
			print substr(label, 1, RSTART - 1), substr(label, RSTART)
		}
	}
' "$questions" | sort -k1,1 -k2,2n -u)

if [ -z "$numbers" ]; then
	printf '%s\n' "${green}✓${off} ${dim}no open questions${off}"
	exit 0
fi

# Consecutive runs collapse within a letter: Q2 Q4 Q5 Q6 K1 K2 K3 -> "K1-K3,
# Q2, Q4-Q6". A run of exactly two reads better spelled out than hyphenated.
compact=$(awk '
	{ letter[++c] = $1; n[c] = $2 }
	END {
		i = 1
		while (i <= c) {
			j = i
			while (j < c && letter[j + 1] == letter[i] && n[j + 1] == n[j] + 1) j++
			L = letter[i]
			if (j == i) seg = L n[i]
			else if (j == i + 1) seg = L n[i] ", " L n[j]
			else seg = L n[i] "-" L n[j]
			out = (out == "" ? seg : out ", " seg)
			i = j + 1
		}
		print out
	}
' <<<"$numbers")

printf '%s\n' "${yellow}❓${off} $compact"
