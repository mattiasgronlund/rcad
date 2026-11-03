#!/usr/bin/env bash
# Makes again, after a boot, the destination of every link that points into the
# zram disk's artifacts directory (docs/decisions.d/476).
#
#   make-artifact-dirs.sh [<artifacts> [<where to look>...]]
#
# A reboot empties the zram disk, and a build through a link whose destination
# is missing fails: `File exists (os error 17)` for rcad's `target/debug`, `Not
# a directory (os error 20)` for inari's whole `target` (476.1, 476.5). rcad
# links the directories inside `target/` and inari links `target` whole, both
# under `<artifacts>/<xx>/<hash>`, so this one step serves both.
#
# Installed as a copy owned by root, and run from zram-kache.service's
# `ExecStartPost` as mattias, through `runuser`: 476.5 has the line. As mattias,
# because the destinations are his and so is the disk once mounted; and so that
# a link a session wrote can never have root make a directory anywhere.
#
# It looks under ~/git and under /tmp/claude-<uid>, where inari keeps scratch
# trees, unless told where. It does not look inside `.git`, nor more than two
# levels inside a directory named `target`: cargo's output directories sit at
# most two levels in (`target/debug`, `target/<triple>/release`), and the real
# `target/` of a tree made before Q3070 holds hundreds of thousands of files.
#
# A link whose text climbs out with `..`, or names a part with any character
# but letters, digits, `.`, `_` and `-`, is refused and counted, never
# followed. It exits 0 whatever it finds, so the unit's start never fails on
# it; the drop-in's `-` says the same to systemd.
set -uo pipefail

artifacts=${1:-/var/cache/zram/artifacts}
[ "$#" -eq 0 ] || shift
if [ "$#" -gt 0 ]; then
    roots=("$@")
else
    roots=("$HOME/git" "/tmp/claude-$(id -u)")
fi
umask 022

made=0
there=0
refused=0
failed=0
for root in "${roots[@]}"; do
    [ -d "$root" ] || continue
    while IFS= read -r -d '' link; do
        text=$(readlink "$link") || continue
        rest=${text#"$artifacts"/}
        if ! grep -qxE '[A-Za-z0-9._-]+(/[A-Za-z0-9._-]+)+' <<<"$rest" \
            || grep -qE '(^|/)\.\.?(/|$)' <<<"$rest"; then
            echo "refused (not a plain path under $artifacts): $link -> $text"
            refused=$((refused + 1))
            continue
        fi
        if [ -d "$text" ]; then
            there=$((there + 1))
        elif mkdir -p -m 755 "$text"; then
            made=$((made + 1))
        else
            echo "could not make $text, for $link"
            failed=$((failed + 1))
        fi
    done < <(find "$root" \( -name .git -o -path '*/target/*/*/*' \) -prune \
        -o -type l -lname "$artifacts/*" -print0 2>/dev/null)
done
echo "made $made destinations, $there already there, $refused links refused, $failed could not be made"
exit 0
