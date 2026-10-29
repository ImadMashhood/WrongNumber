#!/usr/bin/env python3
"""Build locally packaged recorded city layers; see assets/audio/AMBIENCE_CREDITS.md."""
import argparse
import subprocess
import urllib.request
from pathlib import Path

parser = argparse.ArgumentParser()
parser.add_argument('--cache', type=Path, required=True)
parser.add_argument('--output', type=Path, default=Path('assets/audio'))
args = parser.parse_args()
args.cache.mkdir(parents=True, exist_ok=True)
args.output.mkdir(parents=True, exist_ok=True)
for name, url, filters, duration in [
    ('street-muffled', 'https://soundimage.org/wp-content/uploads/2019/09/Street-Corner_P29_2.mp3',
     'highpass=f=100,lowpass=f=1100,volume=1.1,afade=t=in:d=1,afade=t=out:st=47:d=1', ['-t', '48']),
    ('car-engine', 'https://opengameart.org/sites/default/files/loop_0.wav',
     'highpass=f=45,lowpass=f=900,volume=0.7', []),
]:
    source = args.cache / url.rsplit('/', 1)[1]
    if not source.exists():
        urllib.request.urlretrieve(url, source)
    subprocess.run(['ffmpeg', '-y', '-i', str(source), *duration, '-ac', '1', '-ar', '16000',
                    '-af', filters, str(args.output / (name + '.wav'))], check=True)
