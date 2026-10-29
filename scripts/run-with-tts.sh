#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
: "${WRONG_NUMBER_TTS_PYTHON:?Run the Kokoro installer first}"
: "${WRONG_NUMBER_TTS_MODELS:?Run the Kokoro installer first}"
umask 077
mkdir -p .wrong-number
export WRONG_NUMBER_TTS_STATE
WRONG_NUMBER_TTS_STATE=$(mktemp "$PWD/.wrong-number/tts-state.XXXXXX")
"$WRONG_NUMBER_TTS_PYTHON" tools/speech/kokoro_tts.py --serve --models "$WRONG_NUMBER_TTS_MODELS" --state "$WRONG_NUMBER_TTS_STATE" > .wrong-number/tts.log 2>&1 &
tts_pid=$!
cleanup() { kill "$tts_pid" 2>/dev/null || true; wait "$tts_pid" 2>/dev/null || true; rm -f "$WRONG_NUMBER_TTS_STATE"; }
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
for ((i=0; i<300; i++)); do
    if [[ -s "$WRONG_NUMBER_TTS_STATE" ]]; then break; fi
    if ! kill -0 "$tts_pid" 2>/dev/null; then echo 'Local Kokoro startup failed. See .wrong-number/tts.log.' >&2; exit 1; fi
    sleep 0.1
done
if [[ ! -s "$WRONG_NUMBER_TTS_STATE" ]]; then echo 'Local Kokoro startup timed out.' >&2; exit 1; fi
cargo run --locked -- --tts-command "$PWD/scripts/local-tts.sh" "$@"
