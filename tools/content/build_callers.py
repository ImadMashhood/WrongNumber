"""Build fictional cast; references describe inspiration, not real biographies."""
import json
from pathlib import Path
root=Path(__file__).resolve().parents[2]
rows=[]
for n,line in enumerate((root/'tools/content/callers.txt').read_text().splitlines()):
 name,kind,occupation,backstory,voice,stress=line.split('|')
 rows.append(dict(id=1000+n,name=name,kind=kind,occupation=occupation,backstory=backstory,voice=voice,stress=stress))
assert len(rows)==50 and sum(r['kind']=='Original' for r in rows)==40
assert len({r['name'] for r in rows})==50
(root/'assets/scenarios/callers.json').write_text(json.dumps({'version':1,'callers':rows},indent=2)+'\n')
lines=['# Recurring caller cast','','40 original adults and 10 clearly fictional public-persona parodies. Every biography below is invented. Parodies are not claims about the real people, endorsements or voice clones. Character identity is independent of incident truth/severity.','']
for r in rows:
 lines += [f"## {r['id']} — {r['name']}",'',f"**{r['kind']} · {r['occupation']}**",'',r['backstory'],'',f"**Voice:** {r['voice']}",f"**Under stress:** {r['stress']}",'']
(root/'docs/design/caller-cast.md').write_text('\n'.join(lines)+'\n')
print('Built 50 caller profiles')
