#!/bin/bash
# setup.sh: prepares a fresh cloud container for a PM run of the chora
# session. Idempotent; run it from anywhere at the start of a run.
# Usage: bash /home/user/wt/log/setup.sh   (or from a fresh clone of the log branch)
set -u
log() { echo "setup: $*"; }

need_apt=()
build_pkgs="libdisplay-info-dev libdrm-dev libinput-dev libseat-dev libsystemd-dev libudev-dev libwayland-dev libxkbcommon-dev libegl1-mesa-dev libgbm-dev libegl-mesa0 libgl1-mesa-dri pkg-config"
e2e_pkgs="sway swaybg foot grim waybar fuzzel wlrctl swaylock wlr-randr python3-dbusmock jq mako-notifier libnotify-bin bubblewrap"
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

export PATH="$HOME/.local/bin:$PATH"
fuzzel_wanted=1.11.1
fuzzel_sha256=8f7d00c912d5b46d09ad726347644f38b295b586d76ac8113c7a24da962dfb54
fuzzel_ok() {
    local v
    v=$(fuzzel --version 2>/dev/null | sed -n 's/^fuzzel version: \([0-9.]*\).*/\1/p')
    [ -n "$v" ] && [ "$(printf '%s\n1.11\n' "$v" | sort -V | head -1)" = "1.11" ]
}
if ! fuzzel_ok; then
    log "fuzzel $fuzzel_wanted under \$HOME/.local"
    (
        set -e
        dpkg -s wayland-protocols libfcft-dev libtllist-dev >/dev/null 2>&1 ||
            DEBIAN_FRONTEND=noninteractive apt-get install -y -qq wayland-protocols libfcft-dev libtllist-dev >/dev/null
        command -v meson >/dev/null 2>&1 || pip3 install -q --user --break-system-packages meson
        tmp=$(mktemp -d)
        trap 'rm -rf "$tmp"' EXIT
        curl -fsSL -o "$tmp/f.tgz" "https://codeberg.org/dnkl/fuzzel/archive/$fuzzel_wanted.tar.gz"
        echo "$fuzzel_sha256  $tmp/f.tgz" | sha256sum -c --quiet -
        tar -xzf "$tmp/f.tgz" -C "$tmp"
        # the man pages need scdoc, which the container lacks (fuzzel r2); errors stay visible
        sed -i "/^subdir('doc')/d" "$tmp/fuzzel/meson.build"
        meson setup "$tmp/fuzzel/build" "$tmp/fuzzel" --prefix="$HOME/.local" --buildtype=release >/dev/null
        ninja -C "$tmp/fuzzel/build" install >/dev/null
    ) || log "FAILED fuzzel build (apt fuzzel stays)"
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
if [ ! -d /home/user/agent_skills/.git ]; then
    log "clone agent_skills"
    GIT_TERMINAL_PROMPT=0 timeout 120 git clone -q https://github.com/MArpogaus/agent_skills /home/user/agent_skills || log "FAILED clone agent_skills"
fi
GIT_TERMINAL_PROMPT=0 timeout 120 git -C /home/user/agent_skills fetch -q origin dev 2>/dev/null || log "FAILED fetch agent_skills dev"
missing=()
for t in cargo pre-commit vale cz lychee sway swaybg foot grim jq gh; do
    command -v "$t" >/dev/null 2>&1 || missing+=("$t")
done
for c in deny machete; do cargo "$c" --version >/dev/null 2>&1 || missing+=("cargo-$c"); done
pkg-config --exists libdisplay-info libdrm libinput libseat libudev wayland-server xkbcommon egl gbm || missing+=("build-packages")
if [ -d /home/user/chora/.git ]; then
    [ "$(git -C /home/user/chora config --local user.email 2>/dev/null)" = "38564291+MArpogaus@users.noreply.github.com" ] || missing+=("git-identity")
    [ -x /home/user/chora/.git/hooks/commit-msg ] || missing+=("hooks")
else
    log "no /home/user/chora: repo checks skipped (a session without the repo, such as the heartbeat)"
fi
git -C /home/user/agent_skills rev-parse -q --verify origin/dev >/dev/null 2>&1 || log "WARNING: no agent_skills dev; read the skills with add_repo MArpogaus/agent_skills or the GitHub tools (conventions.md)"
if [ "${#missing[@]}" -gt 0 ]; then log "MISSING: ${missing[*]}"; exit 1; fi
log "done, all present"
