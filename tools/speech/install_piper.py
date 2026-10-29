#!/usr/bin/env python3
"""Install the pinned local CPU TTS runtime/models and configure the game launcher."""
import argparse
import hashlib
from pathlib import Path
import shlex
import subprocess
import sys
import urllib.request
import venv

FILES = {
    "en_US-libritts_r-medium.onnx": "10bb85e071d616fcf4071f369f1799d0491492ab3c5d552ec19fb548fac13195",
    "en_US-libritts_r-medium.onnx.json": "b471dc60d2d8335e819c393d196d6fbf792817f40051257b269878505bc9afb3",
}
BASE = "https://huggingface.co/rhasspy/piper-voices/resolve/5041a7a49ae5b3a0f30e6837769eb03fdbb231bd/en/en_US/libritts_r/medium/"

def digest(path):
    with path.open("rb") as f:
        return hashlib.file_digest(f,"sha256").hexdigest()

def main():
    repo = Path(__file__).resolve().parents[2]
    p = argparse.ArgumentParser()
    p.add_argument("--runtime", type=Path, default=repo/".wrong-number/piper-runtime")
    p.add_argument("--models", type=Path, default=repo/".wrong-number/piper-models")
    a = p.parse_args()
    runtime, models = a.runtime.resolve(), a.models.resolve()
    python = runtime/("Scripts/python.exe" if sys.platform == "win32" else "bin/python")
    if not python.exists():
        venv.create(runtime, with_pip=True)
    subprocess.run([str(python),"-m","pip","install","-r",str(repo/"tools/speech/piper-requirements.txt")],check=True)
    models.mkdir(parents=True,exist_ok=True)
    for name, expected in FILES.items():
        path = models/name
        if not path.exists() or digest(path) != expected:
            temp = path.with_suffix(path.suffix+".part")
            print("Downloading", name, flush=True)
            with urllib.request.urlopen(BASE+name, timeout=60) as response, temp.open("wb") as out:
                while chunk := response.read(1024*1024):
                    out.write(chunk)
            if digest(temp) != expected:
                temp.unlink(missing_ok=True)
                raise RuntimeError("Model checksum mismatch")
            temp.replace(path)
    config = repo/".wrong-number/speech.env"
    config.parent.mkdir(parents=True,exist_ok=True)
    config.write_text("WRONG_NUMBER_TTS_ENGINE=piper\nWRONG_NUMBER_TTS_PYTHON="+shlex.quote(str(python))+"\nWRONG_NUMBER_TTS_MODELS="+shlex.quote(str(models))+"\n")
    config.chmod(0o600)
    print("Installed Piper. The Bash game launcher now uses local neural speech.")

if __name__ == "__main__":
    main()
