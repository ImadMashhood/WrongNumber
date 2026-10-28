"""Build the reviewable catalogue from 300 individually authored complaint records."""
import json
from pathlib import Path
root=Path(__file__).resolve().parents[2]
rows=[]
for service in ['police','fire','medical']:
 for line in (root/f'tools/content/{service}.txt').read_text().splitlines():
  severity,disposition,title,observation=line.split('|')
  rows.append({'id':len(rows)+1,'title':title,'service':service.capitalize(),'severity':severity,'needs_response':disposition=='E','observation':observation,'can_explode':service=='fire' and disposition=='E' and any(w in observation.lower() for w in ['burning','flame','caught fire','gas smell','gas escaping'])})
assert len(rows)>=300 and len({r['title'] for r in rows})==len(rows)
(root/'assets/scenarios/complaints.json').write_text(json.dumps({'version':1,'complaints':rows},indent=2)+'\n')
lines=['# Complaint catalogue','',f'{len(rows)} individually authored complaints. The host randomizes the case, reachable location, caller persona and false-report slot. Real minor complaints are distinct from false reports.','', '| ID | Service | Severity | Real incident needs response? | Complaint |','| --- | --- | --- | --- | --- |']
lines += [f"| {r['id']:03} | {r['service']} | {r['severity']} | {'Yes' if r['needs_response'] else 'No'} | {r['title']} |" for r in rows]
(root/'docs/design/complaint-catalogue.md').write_text('\n'.join(lines)+'\n')
print(f'Built {len(rows)} unique complaints')
