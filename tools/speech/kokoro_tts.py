#!/usr/bin/env python3
"""Resident, loopback-only Kokoro TTS and INPUT.txt OUTPUT.wav adapter.

The model stays warm for a game session. No text/audio leaves this machine.
"""
import argparse
import hmac
import http.client
import io
import json
import logging
import os
from pathlib import Path
import secrets
import threading
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

MAX_WAV = 2_000_000
VOICES = ("af_heart", "am_puck", "af_bella", "am_fenrir", "af_sarah", "am_michael", "af_nicole", "bm_george")

def selected_voice(caller):
    if not isinstance(caller, int) or not 1000 <= caller < 1050:
        return "af_heart"
    if caller >= 1040:
        return ("am_puck", "am_fenrir", "am_michael", "bm_george")[(caller - 1040) % 4]
    return VOICES[(caller - 1000) % len(VOICES)]

def validate_request(value):
    if not isinstance(value, dict) or not isinstance(value.get("text"), str):
        raise ValueError("Expected text")
    text = value["text"].strip()
    if not text or len(text.encode("utf-8")) > 512:
        raise ValueError("Text must contain 1-512 UTF-8 bytes")
    caller = value.get("caller", 0)
    if not isinstance(caller, int) or isinstance(caller, bool):
        raise ValueError("Invalid caller ID")
    return text, selected_voice(caller)

def serve(model_dir, state):
    import numpy as np
    import onnxruntime as rt
    import soundfile as sf
    from kokoro_onnx import Kokoro
    from kokoro_onnx.config import EspeakConfig
    import ctypes.util
    logging.disable(logging.CRITICAL)  # Upstream debug logs can include phonemes/text.
    options = rt.SessionOptions()
    options.intra_op_num_threads = 2
    options.inter_op_num_threads = 1
    options.log_severity_level = 3
    session = rt.InferenceSession(str(model_dir / "kokoro-v1.0.onnx"), sess_options=options, providers=["CPUExecutionProvider"])
    # Some espeakng-loader Linux wheels retain a build-machine data path.
    # Prefer the installed system phonemizer when its data is present.
    data = Path(os.environ.get("WRONG_NUMBER_ESPEAK_DATA", "/usr/share/espeak-ng-data"))
    library = ctypes.util.find_library("espeak-ng")
    espeak = EspeakConfig(lib_path=library, data_path=str(data)) if library and (data/"phontab").exists() else None
    engine = Kokoro.from_session(session, str(model_dir / "voices-v1.0.bin"), espeak_config=espeak)
    engine.create("Caller audio is ready.", voice="af_heart", speed=1.0, lang="en-us")
    token = secrets.token_urlsafe(32)
    busy = threading.Lock()

    class Handler(BaseHTTPRequestHandler):
        def setup(self):
            super().setup()
            self.connection.settimeout(5)

        def log_message(self, *args):
            pass

        def do_POST(self):
            if self.path != "/speech" or not hmac.compare_digest(self.headers.get("Authorization", ""), "Bearer " + token):
                self.send_error(403)
                return
            try:
                length = int(self.headers.get("Content-Length", "0"))
                if not 0 < length <= 4096:
                    raise ValueError("Invalid request length")
                text, voice = validate_request(json.loads(self.rfile.read(length)))
            except (ValueError, TimeoutError):
                self.send_error(400)
                return
            if not busy.acquire(blocking=False):
                self.send_error(429, "Speech worker busy")
                return
            try:
                start = time.monotonic()
                samples, rate = engine.create(text, voice=voice, speed=1.0, lang="en-gb" if voice.startswith("b") else "en-us")
                if not len(samples) or not np.isfinite(samples).all():
                    raise ValueError("Invalid synthesized samples")
                buffer = io.BytesIO()
                sf.write(buffer, samples, rate, format="WAV", subtype="PCM_16")
                wav = buffer.getvalue()
                if not 44 < len(wav) <= MAX_WAV:
                    raise ValueError("Audio exceeds game limit")
                self.send_response(200)
                self.send_header("Content-Type", "audio/wav")
                self.send_header("Content-Length", str(len(wav)))
                self.end_headers()
                self.wfile.write(wav)
                print(f"Kokoro voice={voice} generation={time.monotonic()-start:.2f}s audio={len(samples)/rate:.2f}s", flush=True)
            except (BrokenPipeError, ConnectionResetError, TimeoutError):
                pass
            except Exception:
                self.send_error(500, "Local synthesis failed")
            finally:
                busy.release()

    class Server(ThreadingHTTPServer):
        daemon_threads = True
        request_queue_size = 4

    server = Server(("127.0.0.1", 0), Handler)
    state.parent.mkdir(parents=True, exist_ok=True)
    # Per-launch file created by the launcher; only this user can read the token.
    fd = os.open(state, os.O_WRONLY | os.O_CREAT | os.O_TRUNC, 0o600)
    with os.fdopen(fd, "w") as file:
        json.dump({"port": server.server_port, "token": token}, file)
    print("Kokoro ready / local CPU / 2 inference threads", flush=True)
    try:
        server.serve_forever(poll_interval=0.2)
    finally:
        server.server_close()
        state.unlink(missing_ok=True)

def request(input_path, output_path):
    text = input_path.read_text(encoding="utf-8")
    caller = int(os.environ.get("WRONG_NUMBER_CALLER_ID", "0"))
    validate_request({"text": text, "caller": caller})
    state = json.loads(Path(os.environ["WRONG_NUMBER_TTS_STATE"]).read_text())
    port = int(state["port"])
    if not 1 <= port <= 65535:
        raise ValueError("Invalid local TTS port")
    connection = http.client.HTTPConnection("127.0.0.1", port, timeout=18)
    try:
        connection.request("POST", "/speech", json.dumps({"text": text, "caller": caller}), {"Authorization": "Bearer " + state["token"], "Content-Type": "application/json"})
        response = connection.getresponse()
        if response.status != 200:
            raise RuntimeError(f"Local TTS returned {response.status}")
        wav = response.read(MAX_WAV+1)
        if len(wav) > MAX_WAV or not wav.startswith(b"RIFF") or wav[8:12] != b"WAVE":
            raise ValueError("Invalid local speech audio")
        output_path.write_bytes(wav)
    finally:
        connection.close()

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("paths", nargs="*")
    parser.add_argument("--serve", action="store_true")
    parser.add_argument("--models", type=Path)
    parser.add_argument("--state", type=Path)
    args = parser.parse_args()
    if args.serve:
        if not args.models or not args.state:
            parser.error("--serve needs --models and --state")
        serve(args.models, args.state)
    else:
        if len(args.paths) != 2:
            parser.error("Expected INPUT.txt OUTPUT.wav")
        request(*map(Path, args.paths))

if __name__ == "__main__":
    main()
