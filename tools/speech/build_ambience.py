#!/usr/bin/env python3
"""Derive seamless mono loops from LEGIT Audio's CC0 OpenGameArt samples."""
import argparse, io, zipfile
from pathlib import Path
import numpy as np
import soundfile as sf
p=argparse.ArgumentParser();p.add_argument('source_zip',type=Path);p.add_argument('--output',type=Path,default=Path('assets/audio'));a=p.parse_args()
a.output.mkdir(parents=True,exist_ok=True)
with zipfile.ZipFile(a.source_zip) as archive:
    for source,dest in [('TheShopCollection_convenience_store_drinks_fridge_drone.wav','office-ambience.wav'),('TheShopCollection_convenience_store_drinks_fridge_drone_2.wav','breakroom-ambience.wav')]:
        data,rate=sf.read(io.BytesIO(archive.read(source)),always_2d=True)
        mono=data.mean(axis=1)
        # Eight seconds plus a one-second crossfade. Resample the low room drone.
        rate_out=16000
        n=min(len(mono),rate*9)
        mono=np.interp(np.arange(9*rate_out)/rate_out,np.arange(n)/rate,mono[:n])
        fade=np.linspace(0,1,rate_out,endpoint=False)
        loop=np.concatenate([mono[rate_out:-rate_out],mono[-rate_out:]*(1-fade)+mono[:rate_out]*fade])
        loop-=loop.mean()
        peak=max(.001,float(np.max(np.abs(loop))))
        loop=loop/peak*.35
        sf.write(a.output/dest,loop,rate_out,subtype='PCM_16')
        print(dest,len(loop)/rate_out,'seconds')
