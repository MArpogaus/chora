#!/bin/bash

# Runs one Gufo model server for the voice pipeline (M6) in an isolated
# Podman container on the AMD GPU, after /gufo.sh. One model per process.

set -euo pipefail

SCRIPT_NAME=$(basename "$(realpath "$0")")
IMAGE="ghcr.io/gufo-org/toolboxes/gufo-runtime:latest"
MODELS_DIR="/var/models"
NETWORK_NAME_INTERN="ai-net-intern"
NETWORK_NAME_OFFLINE="ai-net-offline"

usage() {
    cat <<EOF
Usage: $SCRIPT_NAME (asr|tts|llm) MODEL [ADDITIONAL_ARGS...]

Runs 'gufo serve KIND' on the AMD GPU (image: $IMAGE). The container is
gufo-KIND; the ports are asr 8881, tts 8882 and llm 8888.

Arguments:
  MODEL              Model under $MODELS_DIR: a checkpoint directory for
                     asr and tts, a GGUF file (first shard) for llm.
  ADDITIONAL_ARGS... Extra flags for the KIND command of 'gufo serve'.

Examples:
  $SCRIPT_NAME asr $MODELS_DIR/Qwen/Qwen3-ASR-1.7B
  $SCRIPT_NAME tts $MODELS_DIR/Qwen/Qwen3-TTS-12Hz-1.7B-CustomVoice
EOF
    exit 1
}

if [ $# -lt 2 ]; then
    usage
fi

KIND=$1
case "$KIND" in
asr) PORT=8881 ;;
tts) PORT=8882 ;;
llm) PORT=8888 ;;
*) usage ;;
esac

MODEL=$(realpath -s "$2")
shift 2

case "$MODEL" in
"$MODELS_DIR"/*) ;;
*)
    echo "Error: Model must be under $MODELS_DIR." >&2
    exit 1
    ;;
esac

if [ ! -e "$MODEL" ]; then
    echo "Error: Model '$MODEL' not found." >&2
    exit 1
fi

podman network create --internal --ignore "$NETWORK_NAME_INTERN" >/dev/null
podman network create --internal --ignore "$NETWORK_NAME_OFFLINE" >/dev/null

podman run --rm -it \
    --userns=keep-id \
    --user "$(id -u):$(id -g)" \
    --group-add keep-groups \
    --read-only \
    --cap-drop=all \
    --security-opt label=disable \
    --tmpfs /tmp \
    --network "$NETWORK_NAME_INTERN" \
    --network "$NETWORK_NAME_OFFLINE" \
    --name "gufo-$KIND" \
    --device /dev/kfd \
    --device /dev/dri \
    -v "$MODELS_DIR":"$MODELS_DIR":ro \
    -p "$PORT":"$PORT" \
    "$IMAGE" \
    gufo serve \
    --host 0.0.0.0 \
    --port "$PORT" \
    "$KIND" \
    --model "$MODEL" \
    --served-model-name "gufo" \
    "$@"
