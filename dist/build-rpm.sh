#!/bin/bash
# Build the chora RPM for the Fedora release of this host.
# Usage, from the repository root: dist/build-rpm.sh [OUTDIR]
# Needs the packages in dist/fedora-build-requires.txt, rpm-build and
# cargo-generate-rpm.
set -euo pipefail
out=${1:-target/rpm}
cargo build --release --locked --workspace
release="1$(rpm --eval '%{?dist}')"
# An RPM version takes no hyphen; rpm sorts 0.6.0~rc.1 before 0.6.0.
cargo_version=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)
version=${cargo_version/-/\~}
mkdir -p "$out"
# GitHub replaces a tilde in the name of an asset, so the file name keeps
# the version of Cargo.
cargo generate-rpm -p crates/chora -o "$out/chora-$cargo_version-$release.x86_64.rpm" \
    --set-metadata "version = \"$version\"" --set-metadata "release = \"$release\""
(cd "$out" && sha256sum ./*.rpm >SHA256SUMS)
