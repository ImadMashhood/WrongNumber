#!/usr/bin/env bash
set -euo pipefail
: "${WRONG_NUMBER_TTS_PYTHON:?Local TTS Python is not configured}"
exec "$WRONG_NUMBER_TTS_PYTHON" "$(dirname "${BASH_SOURCE[0]}")/../tools/speech/kokoro_tts.py" "$@"
