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
exec cargo run --locked -- "$@"
