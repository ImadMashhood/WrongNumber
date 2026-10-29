"""Original stylized portraits for the fictional cast; no external photographs."""
from pathlib import Path
from PIL import Image,ImageDraw
root=Path(__file__).resolve().parents[2]
out=root/'assets/callers';out.mkdir(exist_ok=True)
for n in range(50):
 bg=[(45,77,89),(60,65,96),(75,85,65),(89,62,70),(65,78,102)][n%5]
 im=Image.new('RGB',(192,192),bg);d=ImageDraw.Draw(im)
 skin=[(225,177,139),(179,122,87),(120,79,61),(237,201,170),(155,104,75),(203,153,111)][n%6]
 hair=[(38,32,30),(77,47,32),(167,137,87),(86,83,80),(159,70,35)][(n//2)%5]
 d.ellipse((30,18,162,150),fill=tuple(v+12 for v in bg))
 d.polygon([(21,192),(35,150),(75,134),(117,134),(158,150),(175,192)],fill=[(37,48,66),(80,106,96),(126,85,62),(74,61,93)][n%4])
 d.rectangle((80,111,112,147),fill=skin)
 d.polygon([(56,58),(73,34),(120,34),(138,60),(133,107),(115,131),(78,131),(59,105)],fill=skin)
 d.polygon([(55,72),(52,48),(73,24),(116,22),(142,48),(139,80),(125,53),(95,48),(65,61)],fill=hair)
 if n%3==0:d.polygon([(54,57),(61,107),(48,131),(45,75)],fill=hair)
 d.line((72,80,82,80),fill=(45,36,32),width=3);d.line((110,80,120,80),fill=(45,36,32),width=3)
 d.line((96,84,92,99,99,100),fill=tuple(int(v*.8) for v in skin),width=2)
 d.line((84,113,107,113+(n%3-1)*2),fill=(115,61,53),width=2)
 if n%4==1:
  d.rectangle((64,70,87,89),outline=(40,42,44),width=3);d.rectangle((105,70,128,89),outline=(40,42,44),width=3);d.line((87,77,105,77),fill=(40,42,44),width=3)
 im.save(out/f'{n:02}.png')
