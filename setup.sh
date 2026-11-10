#!/bin/bash
# setup.sh: prepares a fresh cloud container for a PM run of the chora
# session. Idempotent; run it from anywhere at the start of a run.
# Usage: bash /home/user/wt/log/setup.sh   (or from a fresh clone of the log branch)
set -u
log() { echo "setup: $*"; }

need_apt=()
build_pkgs="libdisplay-info-dev libdrm-dev libinput-dev libseat-dev libsystemd-dev libudev-dev libwayland-dev libxkbcommon-dev libegl1-mesa-dev libgbm-dev libegl-mesa0 libgl1-mesa-dri pkg-config"
e2e_pkgs="sway swaybg foot grim waybar fuzzel wlrctl swaylock wlr-randr python3-dbusmock jq"
for p in $build_pkgs $e2e_pkgs; do
    dpkg -s "$p" >/dev/null 2>&1 || need_apt+=("$p")
done
if [ "${#need_apt[@]}" -gt 0 ]; then
    log "apt: ${need_apt[*]}"
    apt-get update -qq
    DEBIAN_FRONTEND=noninteractive apt-get install -y -qq "${need_apt[@]}" >/dev/null ||
        for p in "${need_apt[@]}"; do
            DEBIAN_FRONTEND=noninteractive apt-get install -y -qq "$p" >/dev/null || log "FAILED apt: $p"
        done
fi

for t in pre-commit vale git-filter-repo; do
    command -v "$t" >/dev/null 2>&1 || { log "uv tool: $t"; uv tool install -q "$t"; }
done
command -v cz >/dev/null 2>&1 || { log "uv tool: commitizen"; uv tool install -q commitizen; }

if ! command -v lychee >/dev/null 2>&1; then
    log "lychee 0.24.2"
    tmp=$(mktemp -d)
    curl -fsSL -o "$tmp/l.tgz" "https://github.com/lycheeverse/lychee/releases/download/lychee-v0.24.2/lychee-x86_64-unknown-linux-gnu.tar.gz" &&
        tar -xzf "$tmp/l.tgz" -C "$tmp" && install -m 755 "$(find "$tmp" -name lychee -type f | head -1)" "$HOME/.local/bin/lychee"
    rm -rf "$tmp"
fi

command -v sccache >/dev/null 2>&1 || { log "cargo install sccache"; cargo install -q --locked sccache; }
for c in cargo-deny cargo-machete; do
    cargo "${c#cargo-}" --version >/dev/null 2>&1 || { log "cargo install $c"; cargo install -q --locked "$c"; }
done

grep -q 'incremental' "$HOME/.cargo/config.toml" 2>/dev/null || {
    mkdir -p "$HOME/.cargo"
    printf '[build]\nincremental = false\nrustc-wrapper = "sccache"\n\n[profile.dev]\ndebug = "line-tables-only"\n' >>"$HOME/.cargo/config.toml"
}

if [ -d /home/user/chora/.git ]; then
    git -C /home/user/chora config --local user.name "Marcel Arpogaus"
    git -C /home/user/chora config --local user.email "38564291+MArpogaus@users.noreply.github.com"
    git -C /home/user/chora config --local core.bare false
    (cd /home/user/chora && pre-commit install -t pre-commit -t commit-msg >/dev/null 2>&1)
fi
missing=()
for t in cargo pre-commit vale cz lychee sway swaybg foot grim jq gh; do
    command -v "$t" >/dev/null 2>&1 || missing+=("$t")
done
for c in deny machete; do cargo "$c" --version >/dev/null 2>&1 || missing+=("cargo-$c"); done
pkg-config --exists libdisplay-info libdrm libinput libseat libudev wayland-server xkbcommon egl gbm || missing+=("build-packages")
[ "$(git -C /home/user/chora config --local user.email 2>/dev/null)" = "38564291+MArpogaus@users.noreply.github.com" ] || missing+=("git-identity")
[ -x /home/user/chora/.git/hooks/commit-msg ] || missing+=("hooks")
if [ "${#missing[@]}" -gt 0 ]; then log "MISSING: ${missing[*]}"; exit 1; fi
log "done, all present"
