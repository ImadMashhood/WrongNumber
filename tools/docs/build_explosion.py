"""Generate original deterministic distant-boom audio; no external recording."""
from pathlib import Path
import math,random,struct,wave
rate=22050;rng=random.Random(42);samples=[];low=0
for n in range(int(rate*1.4)):
 t=n/rate;low=.94*low+.06*rng.uniform(-1,1)
 value=(low*1.5+math.sin(2*math.pi*(60*t-12*t*t))*.3)*math.exp(-t*4)*min(1,t*150)
 samples.append(struct.pack('<h',int(max(-1,min(1,value))*.8*32767)))
p=Path(__file__).resolve().parents[2]/'assets/audio/structure-boom.wav'
with wave.open(str(p),'wb') as out:out.setparams((1,2,rate,0,'NONE','not compressed'));out.writeframes(b''.join(samples))
