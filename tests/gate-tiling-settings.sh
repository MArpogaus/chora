#!/bin/bash
# The settings door of tiling (persona round 5, L2): Super+S, and the entry
# Edit settings of the launcher, open chora-tiling.yaml in the terminal
# editor: $EDITOR, else vi, in foot. The key list shows the entry above
# MOD+Ctrl+r, which reads the file again.
#
# Usage: tests/gate-tiling-settings.sh   BIN: directory with the binaries
set -u
. "$(dirname "$0")/gate-lib.sh"
need foot
need_bins chora choractl chora-tiling
export PATH=$bin:$PATH
export CHORA_FLAVOR_DIR=$repo/flavors/tiling
temp_config

printing=$("$bin/chora-tiling" --print-config)
grep -qx '  MOD+s: {run: chora-tiling --launch settings}' <<<"$printing" ||
    fail "the shipped bindings have no MOD+s: {run: chora-tiling --launch settings}"
grep -qx '  MOD+s: Edit settings' <<<"$printing" || fail "the shipped labels have no MOD+s: Edit settings"
entry=$repo/flavors/tiling/applications/chora-settings.desktop
grep -qx 'Name=Edit settings' "$entry" || fail "$entry has no Name=Edit settings"
grep -qx 'Exec=chora-tiling --launch settings' "$entry" || fail "$entry has no Exec=chora-tiling --launch settings"
ok "the bindings, the labels and the launcher entry name Edit settings and its command line"

fakes=$XDG_CONFIG_HOME/fakes
mkdir -p "$fakes"
for editor in editor vi; do
    cat >"$fakes/$editor" <<SH
#!/bin/sh
echo "$editor \$*" >>"$XDG_CONFIG_HOME/edited"
exec sleep 600
SH
    chmod +x "$fakes/$editor"
done
export PATH=$fakes:$PATH
file=$XDG_CONFIG_HOME/chora/chora-tiling.yaml
session_start 1280x800 -- "$bin/chora-tiling" --no-companions
grant input.inject
edited() { cat "$XDG_CONFIG_HOME/edited" 2>/dev/null; }
key() { echo "[{\"shortcut\": {\"keys\": \"$1\"}}]" | ctl inject - >/dev/null || fail "the key $1"; }

key Super+s
until_ok "grep -q '^vi ' '$XDG_CONFIG_HOME/edited' 2>/dev/null" 60 || fail "Super+s started no vi: $(edited)"
[ "$(edited)" = "vi $file" ] || fail "the editor was started as '$(edited)', not 'vi $file'"
[ -s "$file" ] || fail "the editor was started before $file existed"
ok "Super+s opens $file in vi when EDITOR is not set"

: >"$XDG_CONFIG_HOME/edited"
EDITOR=$fakes/editor sh "$repo/flavors/tiling/settings.sh" &
run_pid $!
until_ok "grep -q '^editor ' '$XDG_CONFIG_HOME/edited' 2>/dev/null" 60 || fail "settings.sh started no \$EDITOR: $(edited)"
[ "$(edited)" = "editor $file" ] || fail "\$EDITOR was started as '$(edited)', not 'editor $file'"
ok "the script opens $file in \$EDITOR when it is set"

key Super+slash
sleep 2
python3 -I -c '
import json, sys
words = {n["id"]: n["text"] for n in json.load(sys.stdin) if n["id"].startswith("tiling-keys") and n.get("text")}
caps = {i: t for i, t in words.items() if i.endswith("-k")}
labels = {i: t for i, t in words.items() if i.endswith("-l")}
def key_of(label):
    return words["tiling-keys-body-%d-k" % (int(label.split("-")[-2]) - 1)]
edit = next((i for i, t in labels.items() if t == "Edit settings"), None)
if edit is None:
    sys.exit("no row Edit settings: %r" % sorted(labels.values()))
if key_of(edit) != "Super+S":
    sys.exit("Edit settings has the keys %r" % key_of(edit))
after = "tiling-keys-body-%d-l" % (int(edit.split("-")[-2]) + 2)
if labels.get(after) != "Read the settings again" or key_of(after) != "Super+Ctrl+R":
    sys.exit("the row after Edit settings is %r %r" % (labels.get(after), caps.get(after[:-2] + "-k")))
' <<<"$(ctl query 'kind:text' 2>/dev/null)" || fail "the key list does not show Edit settings beside the reload key"
ok "the key list shows Edit settings (Super+S) and then Read the settings again (Super+Ctrl+R)"
