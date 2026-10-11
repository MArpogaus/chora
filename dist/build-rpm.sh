#!/bin/bash
# Build the chora RPM for the Fedora release of this host.
# Usage, from the repository root: dist/build-rpm.sh [OUTDIR]
# Needs the packages in dist/fedora-build-requires.txt, rpm-build and
# cargo-generate-rpm.
set -euo pipefail
out=${1:-target/rpm}
cargo build --release --locked --workspace
release="1$(rpm --eval '%{?dist}')"
cargo generate-rpm -p crates/chora --set-metadata "release = \"$release\"" -o "$out/"
(cd "$out" && sha256sum ./*.rpm >SHA256SUMS)
