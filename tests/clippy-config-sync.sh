#!/bin/bash
# pre-commit hook: clippy reads only the nearest clippy.toml, so
# plugins/clippy.toml repeats the settings of clippy.toml. Fails when a
# setting is in one file and not the other; the disallowed-methods and
# disallowed-types lists exist only in the plugin file.
#
# Usage: tests/clippy-config-sync.sh [ROOT_FILE PLUGIN_FILE]
set -u
root=${1:-clippy.toml}
plugin=${2:-plugins/clippy.toml}
settings() {
    awk '/^disallowed-/ { list = 1 } !list && !/^#/ && NF { print } list && /^\]/ { list = 0 }' "$1" | sort
}
diff=$(diff <(settings "$root") <(settings "$plugin"))
[ -z "$diff" ] && exit 0
echo "$root and $plugin differ (< only in $root, > only in $plugin):"
echo "$diff"
exit 1
