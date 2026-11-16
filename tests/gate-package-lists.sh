#!/bin/bash
# Fails when a path that dist/build-arch.sh expects in the Arch package has
# no install rule in dist/arch/PKGBUILD, or when a path under
# /usr/share/chora/ has no asset in crates/chora/Cargo.toml. It builds
# nothing, so a missing install rule shows before the package job.
#
# Usage: tests/gate-package-lists.sh
# Exit status: 0 every path has its rules, 1 a rule is missing.
set -u

repo=$(cd "$(dirname "$0")/.." && pwd)
cd "$repo" || exit 2

expected_paths() {
    sed -n '/^for path in /,/; do$/p' dist/build-arch.sh |
        sed 's/^for path in //; s/; do$//; s/\\$//' |
        tr -s ' \t' '\n' | grep -v '^$'
}

has_install_rule() {
    local candidate=$1
    while [ "$candidate" != . ] && [ "$candidate" != usr ] && [ "$candidate" != etc ]; do
        grep -Eq "pkgdir/${candidate}/?(\"| |\$)" dist/arch/PKGBUILD && return 0
        candidate=$(dirname "$candidate")
    done
    return 1
}

failed=0
while read -r path; do
    directory=$(dirname "$path")
    if ! has_install_rule "$path"; then
        echo "FAIL PKGBUILD: no install rule for $path"
        failed=1
    fi
    case $path in
        usr/share/chora/*)
            if ! grep -qF -e "dest = \"/$directory/\"" -e "dest = \"/$path\"" crates/chora/Cargo.toml; then
                echo "FAIL Cargo.toml: no asset for $path"
                failed=1
            fi
            ;;
    esac
done < <(expected_paths)

[ "$failed" = 0 ] && echo "ok   every expected path has its install rules"
exit "$failed"
