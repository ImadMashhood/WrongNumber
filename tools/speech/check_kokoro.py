#!/usr/bin/env python3
"""Real local model/adapter checks; produces samples without recording a person."""
import argparse
import http.client
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time
import wave

p=argparse.ArgumentParser();p.add_argument('--models',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
a.output.mkdir(parents=True,exist_ok=True)
script=Path(__file__).with_name('kokoro_tts.py')
with tempfile.TemporaryDirectory(prefix='wrong-number-tts-') as scratch:
    state=Path(scratch)/'state.json'
    with (a.output/'synthesis.log').open('w') as log:
        process=subprocess.Popen([sys.executable,str(script),'--serve','--models',str(a.models),'--state',str(state)],stdout=log,stderr=subprocess.STDOUT)
        try:
            deadline=time.monotonic()+45
            while (not state.exists() or not state.stat().st_size) and time.monotonic()<deadline:
                assert process.poll() is None,'TTS failed to start'
                time.sleep(.1)
            config=json.loads(state.read_text())
            conn=http.client.HTTPConnection('127.0.0.1',config['port'],timeout=5)
            conn.request('POST','/speech','{}');assert conn.getresponse().status==403;conn.close()
            conn=http.client.HTTPConnection('127.0.0.1',config['port'],timeout=5)
            conn.request('POST','/speech',json.dumps({'text':'a'*513}),{'Authorization':'Bearer '+config['token']});assert conn.getresponse().status==400;conn.close()
            reports=[]
            for caller,text in [(1000,"Hi, yes, I'm at the laundromat on Bridge Drive. There's smoke coming from one of the dryers. I've got everyone outside, but I don't know if anyone's still in the back."),(1041,"Okay, listen. I thought it was just a prank at first, but someone's really trying to force the shop door open. I'm across the street. Please send someone.")]:
                input_path=Path(scratch)/'text.txt';input_path.write_text(text)
                wav=a.output/f'caller-{caller}.wav'
                start=time.monotonic()
                subprocess.run([sys.executable,str(script),str(input_path),str(wav)],env={**os.environ,'WRONG_NUMBER_TTS_STATE':str(state),'WRONG_NUMBER_CALLER_ID':str(caller)},check=True,timeout=20)
                elapsed=time.monotonic()-start
                with wave.open(str(wav)) as audio:
                    assert audio.getnchannels()==1 and audio.getsampwidth()==2
                    duration=audio.getnframes()/audio.getframerate()
                    assert duration>1
                    assert any(audio.readframes(audio.getnframes()))
                reports.append({'caller':caller,'generation_seconds':round(elapsed,3),'audio_seconds':round(duration,3)})
            (a.output/'report.json').write_text(json.dumps({'result':'PASS','checks':['authorization','text bound','two real voices','PCM WAV'],'samples':reports},indent=2))
            print(json.dumps(reports,indent=2))
        finally:
            process.terminate()
            try:process.wait(timeout=3)
            except subprocess.TimeoutExpired:process.kill();process.wait()
