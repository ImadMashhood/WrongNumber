# Local neural caller voices

The development launcher now selects **Pocket TTS 4.0.1**, one of the local engines named in Scam With Your Friends’ official 0.1.95 notes. This integration uses Kyutai’s **english_18m_2026-10** model on CPU, with two inference threads. It does not claim to reproduce the other game’s exact settings or voice assets. Whisper still handles microphone transcription; the configured cloud provider still writes dialogue.

```sh
python3 tools/speech/install_pocket.py
bash scripts/run.sh
```

The installer is for the current Linux CPU setup. It installs pinned Pocket/PyTorch versions into an isolated environment, downloads model dependencies, exports eight stock voice embeddings and runs a real synthesis check before atomically selecting the engine in ignored `.wrong-number/speech.env`. Internet is needed for installation only. Gameplay sets `HF_HUB_OFFLINE=1`; the warmed process uses cached models and local voice embeddings. No caller text/audio is sent to a speech API, and no speech API key is required.

The 50 personas map deterministically across eight stock voices: alba, anna, charles, george, jane, paul, marius and vera. They are not 50 unique voices, and parodies do not clone the celebrities they reference. Voice embeddings stay warm throughout a session. Missing models or synthesis failures produce visible errors, not an automatic engine switch.

The larger April model measured 3.9–4.7 seconds for the same two caller samples; the selected small model measured approximately 1.9–2.6 seconds for 9–10.5 seconds of spoken output on this PC. The old Piper was faster at about 0.47 seconds, so this is a different voice-quality/latency tradeoff, not a claim that Pocket beats Piper on speed. These measurements exclude the dialogue provider and STT. The adapter currently generates a whole WAV before playback; upstream Pocket supports streaming, but this game adapter does not yet stream.

The loopback service requires a private per-launch token, accepts at most 512 UTF-8 text bytes, bounds WAV output to 2 MB, logs timings rather than transcripts, and exits with the launcher. The game retains its 20-second external synthesis deadline and 60-second audio limit. It never changes the no-offline-caller-fallback rule.

Validation: `WRONG_NUMBER_TTS_ENGINE=pocket <venv-python> tools/speech/check_kokoro.py --models <pocket-model-directory> --output <directory>`. The shared adapter/check retains its historical filename but now supports Pocket, Piper and Kokoro.

## Sources and attribution

- [Official game patch notes](https://steamcommunity.com/app/4954910?l=swedish): Pocket TTS, MOSS TTS Nano and Qwen3 are selectable voice models.
- [Pocket TTS source/API](https://github.com/kyutai-labs/pocket-tts) and [model card](https://huggingface.co/kyutai/pocket-tts): MIT model/code licensing; see upstream notices when packaging. Runtime dependencies have their own licenses.
- [Kyutai voice repository](https://huggingface.co/kyutai/tts-voices): Alba MacKenna casual voice, CC BY 4.0. Anna/Charles/George/Jane/Paul/Vera are VCTK voices, CC BY 4.0, from the [CSTR VCTK Corpus](https://datashare.ed.ac.uk/handle/10283/3443), with the enhanced source recordings supplied by Kyutai. Marius is a volunteer voice donation, CC0. The game derives local voice embeddings from these source recordings. Noncommercial EARS/Expresso voices are not selected.

## Previously installed alternatives

Piper and Kokoro remain explicitly selectable by their installers, never automatic fallbacks. Their models are local and not committed. The older Piper benchmark remains useful as a speed baseline.

## Sources and licenses

- [Kokoro-82M](https://huggingface.co/hexgrad/Kokoro-82M): Apache-2.0 model.
- [kokoro-onnx](https://github.com/thewh1teagle/kokoro-onnx): MIT wrapper; pinned Python dependencies are in `tools/speech/tts-requirements.txt`.
- [Stock voice documentation](https://huggingface.co/hexgrad/Kokoro-82M/blob/main/VOICES.md).

The installer pins `model-files-v1.0`; model and voice archive hashes are in `tools/speech/install_kokoro.py`. Models are downloaded locally, not checked into Git. The existing `--tts-command` executable interface remains available for other engines. Audio failure does not generate replacement AI dialogue.

### Piper attribution and distribution

- [Piper Python API/source](https://github.com/OHF-Voice/piper1-gpl): GPL-3.0; installed as the separate Python speech process, not vendored into the Rust executable. Distribution of that runtime must comply with its license.
- [LibriTTS-R medium voice model card](https://huggingface.co/rhasspy/piper-voices/blob/main/en/en_US/libritts_r/medium/MODEL_CARD): 904 speakers, 22,050 Hz. Dataset **LibriTTS-R**, published through [OpenSLR 141](https://www.openslr.org/141/), CC BY 4.0. The model card records fine-tuning from Lessac medium.
- Model/config are pinned to Hugging Face repository revision `5041a7a49ae5b3a0f30e6837769eb03fdbb231bd`, with SHA-256 checks in `tools/speech/install_piper.py`. Runtime is pinned in `piper-requirements.txt`. Neither models nor dependencies are committed.
