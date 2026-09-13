#!/usr/bin/env bash
# The status line itself lives in the `questions` plugin. This file is the
# wrapper that has to stay here, and the reason is not style: a plugin cannot
# supply a `statusLine`. Only user or project `settings.json` can, and
# `settings.json` names a path — so something in this repo must be that path.
#
# What it must not become is a second copy of the line. `rscene` had one, and
# on 2026-09-07 it still had `if (match(...))` where this repo had `while`: on
# a file with an entry carrying two labels it printed `❓ Q7, Q12` where the
# fixed copy printed `❓ deadbe Q7, Q11, Q12`. Q11 was dropped, against seven
# live question files, and nothing noticed — a miscount reads exactly like a
# correct count of a different number. So this stays a wrapper: no parsing, no
# formatting, nothing that could disagree with the other repo's copy.
#
# `$HOME`-relative and overridable, never absolute: a tracked file naming
# `/home/mattias` breaks for anyone who clones this. The same variables the
# justfile reads, resolved in the same order, so there is one place to move:
# the override first, then the marketplace clone — the tree the `ref` in
# `.claude/settings.json` pins — then a plain checkout.
set -uo pipefail

config=${CLAUDE_CONFIG_DIR:-$HOME/.claude}
clone=$config/plugins/marketplaces/mattiasgronlund-local/plugins
[ -d "$clone" ] || clone=$HOME/git/mattiasgronlund/claude-questions/plugins
root=${CLAUDE_PLUGINS_ROOT:-$clone}
plugin=${CLAUDE_QUESTIONS_PLUGIN:-$root/questions}
line="$plugin/bin/statusline.sh"

# A missing plugin says so on the line itself. It cannot exit quietly: the
# status line is the only place this repo reports its context budget, and a
# blank line reads as "under budget" rather than as "not installed".
if [ ! -x "$line" ]; then
	cat >/dev/null
	printf '%s\n' "questions plugin not found at $plugin — claude plugin install -y questions@mattiasgronlund-local"
	exit 0
fi

exec "$line"
