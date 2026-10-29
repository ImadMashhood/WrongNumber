#!/usr/bin/env python3
"""Local STT adapter: transcribe.py INPUT.wav OUTPUT.txt. Model must exist locally."""
import os,sys,wave
import numpy as np
from pathlib import Path
from faster_whisper import WhisperModel
if len(sys.argv)!=3:raise SystemExit('Usage: transcribe.py input.wav output.txt')
model_path=os.environ.get('WRONG_NUMBER_STT_MODEL')
if not model_path or not Path(model_path).is_dir():raise SystemExit('Set WRONG_NUMBER_STT_MODEL to a downloaded local Faster Whisper model folder')
model=WhisperModel(model_path,device='cpu',compute_type='int8',cpu_threads=4,local_files_only=True)
with wave.open(sys.argv[1],'rb') as wav:
 if wav.getsampwidth()!=2 or wav.getnframes()/wav.getframerate()>12.5:raise SystemExit('Expected bounded PCM16 speech')
 rate=wav.getframerate();channels=wav.getnchannels()
 audio=np.frombuffer(wav.readframes(wav.getnframes()),dtype='<i2').astype(np.float32)/32768
 audio=audio.reshape(-1,channels).mean(axis=1)
 if rate!=16000:audio=np.interp(np.arange(int(len(audio)*16000/rate))*rate/16000,np.arange(len(audio)),audio).astype(np.float32)
segments,_=model.transcribe(audio,language='en',beam_size=5,vad_filter=False,condition_on_previous_text=False)
text=' '.join(s.text.strip() for s in segments).strip()
if not text or len(text.encode())>1024:raise SystemExit('No usable question recognized')
Path(sys.argv[2]).write_text(text,encoding='utf-8')
