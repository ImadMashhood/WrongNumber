#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
software=false
for arg in "$@"; do [[ "$arg" != --software ]] || software=true; done
if $software; then
    selected_mesa=
    for mesa_icd_path in /usr/share/vulkan/icd.d/*lvp*.json; do
        if [[ -f "$mesa_icd_path" ]]; then selected_mesa="$mesa_icd_path"; break; fi
    done
    if [[ -z "$selected_mesa" ]]; then echo "Mesa software Vulkan driver was not found." >&2; exit 1; fi
    export VK_DRIVER_FILES="$selected_mesa"
    export WGPU_BACKEND=vulkan
    echo 'Software rendering requested; reduced graphics enabled.'
elif [[ -z "${VK_DRIVER_FILES:-}" && -f /usr/share/vulkan/icd.d/nvidia_icd.x86_64.json ]] && command -v nvidia-smi >/dev/null && nvidia-smi >/dev/null 2>&1; then
    export VK_DRIVER_FILES=/usr/share/vulkan/icd.d/nvidia_icd.x86_64.json
    export WGPU_BACKEND=vulkan
fi
# Local neural speech is opt-in via the installer; explicit adapters take priority.
if [[ -f .wrong-number/speech.env ]]; then
    set -a
    source .wrong-number/speech.env
    set +a
fi
for arg in "$@"; do
    if [[ "$arg" == --tts-command || "$arg" == --smoke-test || "$arg" == --menu-test || "$arg" == --playtest || "$arg" == --benchmark || "$arg" == --network-playtest || "$arg" == --conversation-test ]]; then
        exec cargo run --locked -- "$@"
    fi
done
if [[ -n "${WRONG_NUMBER_TTS_PYTHON:-}" && -n "${WRONG_NUMBER_TTS_MODELS:-}" ]]; then
    exec bash scripts/run-with-tts.sh "$@"
fi
exec cargo run --locked -- "$@"
