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
    "kokoro-v1.0.onnx": "7d5df8ecf7d4b1878015a32686053fd0eebe2bc377234608764cc0ef3636a6c5",
    "voices-v1.0.bin": "bca610b8308e8d99f32e6fe4197e7ec01679264efed0cac9140fe9c29f1fbf7d",
}
BASE = "https://github.com/thewh1teagle/kokoro-onnx/releases/download/model-files-v1.0/"

def digest(path):
    with path.open("rb") as f:
        return hashlib.file_digest(f,"sha256").hexdigest()

def main():
    repo = Path(__file__).resolve().parents[2]
    p = argparse.ArgumentParser()
    p.add_argument("--runtime", type=Path, default=repo/".wrong-number/tts-runtime")
    p.add_argument("--models", type=Path, default=repo/".wrong-number/tts-models")
    a = p.parse_args()
    runtime, models = a.runtime.resolve(), a.models.resolve()
    python = runtime/("Scripts/python.exe" if sys.platform == "win32" else "bin/python")
    if not python.exists():
        venv.create(runtime, with_pip=True)
    subprocess.run([str(python),"-m","pip","install","-r",str(repo/"tools/speech/tts-requirements.txt")],check=True)
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
    config.write_text("WRONG_NUMBER_TTS_ENGINE=kokoro\nWRONG_NUMBER_TTS_PYTHON="+shlex.quote(str(python))+"\nWRONG_NUMBER_TTS_MODELS="+shlex.quote(str(models))+"\n")
    config.chmod(0o600)
    print("Installed Kokoro. The Bash game launcher now uses local neural speech.")

if __name__ == "__main__":
    main()
