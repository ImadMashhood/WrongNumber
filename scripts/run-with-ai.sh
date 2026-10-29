#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
cargo build --locked --workspace
umask 077
mkdir -p .wrong-number
chmod 700 .wrong-number
# Overwritten on launch; diagnostics contain no keys or spoken transcripts.
(
    if [[ -f .wrong-number/caller.env ]]; then
        set -a
        source .wrong-number/caller.env
        set +a
    fi
    exec target/debug/wrong-number-caller-server > >(tee .wrong-number/caller.log) 2>&1
) &
caller_pid=$!
trap 'kill "$caller_pid" 2>/dev/null || true; wait "$caller_pid" 2>/dev/null || true' EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
sleep 0.3
if ! kill -0 "$caller_pid" 2>/dev/null; then
    echo 'Caller server could not start. Check configuration and port 47005.' >&2
    echo 'Opening the menu without AI. Joining another host still works; hosting live callers requires a configured caller service.' >&2
    bash scripts/run.sh "$@" 2>&1 | tee .wrong-number/game.log
    exit "${PIPESTATUS[0]}"
fi
bash scripts/run.sh --caller-service 127.0.0.1:47005 "$@" 2>&1 | tee .wrong-number/game.log
