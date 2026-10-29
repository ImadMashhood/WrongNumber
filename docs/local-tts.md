# Local neural caller voices

The default on the development PC is now **Piper**, using a resident CPU model for much lower delay. Kokoro remains an optional installed alternative. There are no per-call speech API charges. Dialogue still uses the configured AI provider. Model/runtime installation requires internet once; synthesis during gameplay is local.

```sh
python3 tools/speech/install_piper.py
bash scripts/run.sh
```

Keep using the AI launcher for live callers; it delegates to the same game launcher. The installer creates a Python environment, downloads the pinned LibriTTS-R medium model/config, verifies SHA-256 hashes, and writes ignored `.wrong-number/speech.env`. This PC is already configured. Re-running `install_kokoro.py` selects Kokoro instead; engine failures never silently switch engines.

A warm background process loads the model once per launch. Piper assigns the 50 recurring caller IDs to 50 stable stock speaker IDs. Parodies do not clone their real-world inspirations. The service binds only loopback, requires a private per-launch token, bounds input to 512 UTF-8 bytes and WAV output to 2 MB, and logs timings rather than transcripts. It stops with the launcher. The game permits external synthesis up to 20 seconds and audio up to 60 seconds.

The same two test lines that took Kokoro 4–5 seconds generated in **0.466 seconds each** with Piper (including adapter/HTTP overhead), producing about 6.3 seconds of speech. Shorter actual in-game fixture lines generated in 0.08–0.21 seconds. These are machine-specific measurements; model and speaking duration differ. Dialogue-provider and transcription time are separate. Actual game playback was also exercised. Output routing can be set to Scarlett Solo in Settings.

The optional neural check uses `WRONG_NUMBER_TTS_ENGINE=piper <venv-python> tools/speech/check_kokoro.py --models <piper-model-directory> --output <directory>`. The shared daemon/check retains its original filename but supports both engines.

## Sources and licenses

- [Kokoro-82M](https://huggingface.co/hexgrad/Kokoro-82M): Apache-2.0 model.
- [kokoro-onnx](https://github.com/thewh1teagle/kokoro-onnx): MIT wrapper; pinned Python dependencies are in `tools/speech/tts-requirements.txt`.
- [Stock voice documentation](https://huggingface.co/hexgrad/Kokoro-82M/blob/main/VOICES.md).

The installer pins `model-files-v1.0`; model and voice archive hashes are in `tools/speech/install_kokoro.py`. Models are downloaded locally, not checked into Git. The existing `--tts-command` executable interface remains available for other engines. Audio failure does not generate replacement AI dialogue.

### Piper attribution and distribution

- [Piper Python API/source](https://github.com/OHF-Voice/piper1-gpl): GPL-3.0; installed as the separate Python speech process, not vendored into the Rust executable. Distribution of that runtime must comply with its license.
- [LibriTTS-R medium voice model card](https://huggingface.co/rhasspy/piper-voices/blob/main/en/en_US/libritts_r/medium/MODEL_CARD): 904 speakers, 22,050 Hz. Dataset **LibriTTS-R**, published through [OpenSLR 141](https://www.openslr.org/141/), CC BY 4.0. The model card records fine-tuning from Lessac medium.
- Model/config are pinned to Hugging Face repository revision `5041a7a49ae5b3a0f30e6837769eb03fdbb231bd`, with SHA-256 checks in `tools/speech/install_piper.py`. Runtime is pinned in `piper-requirements.txt`. Neither models nor dependencies are committed.
