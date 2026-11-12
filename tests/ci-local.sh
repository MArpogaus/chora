#!/bin/bash
# Runs on this machine what the jobs of .github/workflows run, so that a
# draft PR is not the first run: msrv (cargo check with the rust-version
# of Cargo.toml), docs (cargo doc, and the mkdocs site when uvx exists),
# pre-commit (all files, with the SKIP list of pre-commit.yaml), deny,
# machete and packaging (the syntax of dist/build-*.sh, and the RPM build
# when rpm-build and cargo-generate-rpm exist). The Fedora container,
# nextest and the e2e jobs are not replayed; tests/merge-gate.sh covers
# them. A job whose tool is missing prints SKIP and its reason.
#
# Usage: tests/ci-local.sh [JOB...]
#   JOB  msrv, docs, pre-commit, deny, machete, packaging (default all)
#   CI_LOCAL_TARGET  target directory of the msrv check
#                    (default target/ci-local-msrv; delete it after use)
# Exit status: 0 no job failed, 1 a job failed, 2 a usage error.
set -u

repo=$(cd "$(dirname "$0")/.." && pwd)
cd "$repo" || exit 2
all="msrv docs pre-commit deny machete packaging"
jobs=${*:-$all}
logs=$(mktemp -d /tmp/ci-local.XXXXXX)

usage() { sed -n '2,/^set -u/p' "$0" | sed '$d; s/^# \{0,1\}//'; }
case ${1:-} in -h | --help)
    usage
    exit 0
    ;;
esac

have() { command -v "$1" >/dev/null 2>&1; }
skip() {
    echo "SKIP: $*"
    return 77
}

job_msrv() {
    have rustup || skip "no rustup"
    local msrv
    msrv=$(sed -n 's/^rust-version = "\(.*\)"/\1/p' Cargo.toml)
    rustup toolchain install "$msrv" --profile minimal || return 1
    CARGO_TARGET_DIR=${CI_LOCAL_TARGET:-$repo/target/ci-local-msrv} \
        cargo "+$msrv" check --workspace --all-targets --locked
}

job_docs() {
    cargo doc --workspace --no-deps --locked || return 1
    if [ -f mkdocs.yml ] && have uvx; then
        uvx --with-requirements docs/requirements.txt --from mkdocs mkdocs build --strict -d "$logs/site"
    else
        echo "mkdocs site not built: no mkdocs.yml or no uvx"
    fi
}

job_pre_commit() {
    have pre-commit || skip "no pre-commit"
    SKIP=cargo-clippy,lychee,vale pre-commit run --all-files
}

job_deny() {
    have cargo-deny || skip "no cargo-deny"
    cargo deny check
}

job_machete() {
    have cargo-machete || skip "no cargo-machete"
    cargo machete
}

job_packaging() {
    [ -f dist/build-rpm.sh ] || skip "no dist/build-rpm.sh"
    bash -n dist/build-rpm.sh || return 1
    [ ! -f dist/build-arch.sh ] || bash -n dist/build-arch.sh || return 1
    if have rpm && have cargo-generate-rpm; then
        dist/build-rpm.sh "$logs/rpm"
    else
        echo "RPM not built: needs rpm-build and cargo-generate-rpm; syntax checked only"
    fi
}

failed=0
summary=()
for job in $jobs; do
    case " $all " in *" $job "*) ;; *)
        echo "ci-local: unknown job $job" >&2
        exit 2
        ;;
    esac
    started=$SECONDS
    {
        case $job in
        msrv) job_msrv ;;
        docs) job_docs ;;
        pre-commit) job_pre_commit ;;
        deny) job_deny ;;
        machete) job_machete ;;
        packaging) job_packaging ;;
        esac
    } >"$logs/$job.log" 2>&1
    code=$?
    case $code in
    0) result=PASS ;;
    77) result=SKIP ;;
    *) result=FAIL ;;
    esac
    if [ "$result" = FAIL ]; then
        failed=1
        echo "--- $job: last 30 lines of $logs/$job.log"
        tail -n 30 "$logs/$job.log"
    fi
    summary+=("$(printf '%-11s %s %5ss  %s' "$job" "$result" "$((SECONDS - started))" "$logs/$job.log")")
done
printf '%s\n' "${summary[@]}"
exit "$failed"
