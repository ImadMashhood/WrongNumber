"""Generate the project's original telephone ring (no external recording)."""
import math,struct,wave
from pathlib import Path
p=Path(__file__).resolve().parents[2]/'assets/audio/telephone.wav'
rate=22050
with wave.open(str(p),'wb') as out:
 out.setparams((1,2,rate,0,'NONE','not compressed'))
 samples=[]
 for n in range(rate*3):
  t=n/rate;phase=t%3;on=phase<.45 or .7<=phase<1.15
  envelope=min(1,(phase% .7)*60,max(0,(.45-phase% .7)*60)) if on else 0
  value=(math.sin(2*math.pi*440*t)+math.sin(2*math.pi*480*t))*.22*envelope
  samples.append(struct.pack('<h',int(value*32767)))
 out.writeframes(b''.join(samples))
print(p)
