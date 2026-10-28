#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
if [[ -f .wrong-number/caller.env ]]; then
    set -a
    source .wrong-number/caller.env
    set +a
fi
exec cargo run --locked -p wrong-number-caller-server
