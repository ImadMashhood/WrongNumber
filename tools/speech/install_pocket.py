#!/usr/bin/env python3
"""Install CPU Pocket TTS, cache its model/stock voices, then select it atomically."""
import argparse
import os
from pathlib import Path
import shlex
import subprocess
import sys
import venv

VOICES = ('alba', 'anna', 'charles', 'george', 'jane', 'paul', 'marius', 'vera')
LANGUAGE = 'english_18m_2026-10'

def prepare(models):
    os.environ['HF_HOME'] = str(models / 'cache')
    import torch
    from pocket_tts import TTSModel, export_model_state
    torch.set_num_threads(2)
    model = TTSModel.load_model(language=LANGUAGE)
    for voice in VOICES:
        state = model.get_state_for_audio_prompt(voice)
        export_model_state(state, str(models / (voice + '.safetensors')))
    audio = model.generate_audio(model.get_state_for_audio_prompt(str(models/'alba.safetensors')), 'Caller audio is ready.')
    assert audio.numel() and torch.isfinite(audio).all()

def main():
    repo = Path(__file__).resolve().parents[2]
    parser = argparse.ArgumentParser()
    parser.add_argument('--runtime',type=Path,default=repo/'.wrong-number/pocket-runtime')
    parser.add_argument('--models',type=Path,default=repo/'.wrong-number/pocket-models')
    parser.add_argument('--prepare',action='store_true')
    a = parser.parse_args()
    models = a.models.resolve(); models.mkdir(parents=True,exist_ok=True)
    if a.prepare:
        prepare(models); return
    runtime = a.runtime.resolve()
    python = runtime/('Scripts/python.exe' if sys.platform=='win32' else 'bin/python')
    if not python.exists(): venv.create(runtime,with_pip=True)
    subprocess.run([str(python),'-m','pip','install','-r',str(repo/'tools/speech/pocket-requirements.txt'),'--extra-index-url','https://download.pytorch.org/whl/cpu'],check=True)
    subprocess.run([str(python),str(Path(__file__).resolve()),'--prepare','--models',str(models)],check=True)
    config=repo/'.wrong-number/speech.env'; config.parent.mkdir(parents=True,exist_ok=True)
    temp=config.with_suffix('.tmp')
    temp.write_text('WRONG_NUMBER_TTS_ENGINE=pocket\nWRONG_NUMBER_TTS_PYTHON='+shlex.quote(str(python))+'\nWRONG_NUMBER_TTS_MODELS='+shlex.quote(str(models))+'\n')
    temp.chmod(0o600); temp.replace(config)
    print('Pocket TTS installed and selected. Runtime synthesis is local.')

if __name__ == '__main__': main()
