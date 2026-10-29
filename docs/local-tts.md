# Local neural caller voices

The Bash launcher supports **Kokoro-82M v1.0**, running on CPU through `kokoro-onnx`. There are no per-call speech API charges. Dialogue still uses the configured AI provider. Model/runtime installation requires internet once; synthesis during gameplay is local.

```sh
python3 tools/speech/install_kokoro.py
bash scripts/run.sh
```

Keep using the AI launcher for live callers; it delegates to the same game launcher. The installer creates a Python environment, downloads the pinned model and stock voice archive (about 355 MB combined), verifies SHA-256 hashes, and writes ignored `.wrong-number/speech.env`. The development PC already has this installed. On Linux, install `espeak-ng` if the phonemizer’s bundled data cannot be located. That component converts words to phonemes; Kokoro generates the sound.

A warm background process loads the model once per game launch. Eight stock voices are assigned deterministically by caller ID. Fictional parodies use stock voices, not imitations or clones of the people inspiring them. The service binds only loopback, requires a private per-launch token, bounds text to 512 UTF-8 bytes and WAV output to 2 MB, rejects concurrent synthesis rather than queueing indefinitely, and logs timings rather than transcripts. It stops with the launcher. The game allows external synthesis up to 20 seconds; errors remain visible and do not silently substitute a different voice. Playback permits up to 60 seconds.

Two local test lines produced 9–10 seconds of speech in 4–5 seconds of CPU synthesis. This is a machine-specific measurement; longer lines can take longer. The warm startup can add several seconds before the menu. Caller volume and output selection apply to playback; select Scarlett Solo when that is your listening device.

## Sources and licenses

- [Kokoro-82M](https://huggingface.co/hexgrad/Kokoro-82M): Apache-2.0 model.
- [kokoro-onnx](https://github.com/thewh1teagle/kokoro-onnx): MIT wrapper; pinned Python dependencies are in `tools/speech/tts-requirements.txt`.
- [Stock voice documentation](https://huggingface.co/hexgrad/Kokoro-82M/blob/main/VOICES.md).

The installer pins `model-files-v1.0`; model and voice archive hashes are in `tools/speech/install_kokoro.py`. Models are downloaded locally, not checked into Git. The existing `--tts-command` executable interface remains available for other engines. Audio failure does not generate replacement AI dialogue.
