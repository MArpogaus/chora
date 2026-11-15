#!/bin/bash
# Fails when the packages of chora lack the programs it needs: the RPM
# and the Arch package require fuzzel 1.11 or later and fontconfig, and
# recommend the DejaVu fonts.
#
# Usage: tests/gate-package-deps.sh [--rpm FILE]
#   --rpm FILE  read the metadata of the built RPM with rpm -qp
#               (default: the metadata sources, crates/chora/Cargo.toml
#               and dist/arch/PKGBUILD)
# Exit status: 0 every entry found, 1 an entry is missing, 2 a usage error.
set -u

repo=$(cd "$(dirname "$0")/.." && pwd)
cd "$repo" || exit 2
rpm_file=
case ${1:-} in
    --rpm)
        rpm_file=${2:-}
        [ -f "$rpm_file" ] || { echo "usage: $0 [--rpm FILE]" >&2; exit 2; }
        ;;
    "") ;;
    *) echo "usage: $0 [--rpm FILE]" >&2; exit 2 ;;
esac

failed=0
expect() {
    local where=$1 pattern=$2 text=$3 what=$4
    if grep -Eq -- "$pattern" <<<"$text"; then
        echo "ok   $where: $what"
    else
        echo "FAIL $where: $what is missing"
        failed=1
    fi
}

if [ -n "$rpm_file" ]; then
    requires=$(rpm -qp --requires "$rpm_file")
    recommends=$(rpm -qp --recommends "$rpm_file")
    rpm_where=$rpm_file
    fuzzel_pattern='^fuzzel >= 1\.11'
    fontconfig_pattern='^fontconfig'
else
    requires=$(grep '^requires = ' crates/chora/Cargo.toml)
    recommends=$(grep '^recommends = ' crates/chora/Cargo.toml)
    rpm_where=crates/chora/Cargo.toml
    fuzzel_pattern='fuzzel = ">= 1\.11"'
    fontconfig_pattern='fontconfig = '
fi
expect "$rpm_where" "$fuzzel_pattern" "$requires" "Requires fuzzel >= 1.11"
expect "$rpm_where" "$fontconfig_pattern" "$requires" "Requires fontconfig"
for font in dejavu-sans-fonts dejavu-serif-fonts dejavu-sans-mono-fonts; do
    expect "$rpm_where" "$font" "$recommends" "Recommends $font"
done

depends=$(grep '^depends=' dist/arch/PKGBUILD)
optdepends=$(sed -n '/^optdepends=(/,/^)/p' dist/arch/PKGBUILD)
expect dist/arch/PKGBUILD "[ (]'fuzzel>=1\.11'[ )]" "$depends" "depends fuzzel>=1.11"
expect dist/arch/PKGBUILD "[ (]fontconfig[ )]" "$depends" "depends fontconfig"
expect dist/arch/PKGBUILD "'ttf-dejavu:" "$optdepends" "optdepends ttf-dejavu"

exit $failed
