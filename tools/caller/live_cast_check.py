"""Opt-in paid, bounded multi-turn cast evaluation. No microphone audio or credentials logged."""
import json,os,socket,subprocess,time,argparse
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--output',required=True);a=p.parse_args()
out=Path(a.output);out.mkdir(parents=True,exist_ok=True)
env=os.environ.copy()
for line in Path('.wrong-number/caller.env').read_text().splitlines():
 if '=' in line and not line.startswith('#'):
  k,v=line.split('=',1);env[k]=v
with socket.socket() as sock:sock.bind(('127.0.0.1',0));port=sock.getsockname()[1]
env.update(CALLER_BIND=f'127.0.0.1:{port}',CALLER_MAX_REQUESTS='16')
log=(out/'service.log').open('w');proc=subprocess.Popen([str(Path('target/debug/wrong-number-caller-server').resolve())],env=env,stdout=log,stderr=subprocess.STDOUT)
cast=json.loads(Path('assets/scenarios/callers.json').read_text())['callers']
def brief(c):return f"{c['name']}. {c['occupation']}. Background: {c['backstory']} Speaking style: {c['voice']} Under stress: {c['stress']} Fictional adult character."
def ask(c):
 with socket.create_connection(('127.0.0.1',port),timeout=24) as sock:
  sock.sendall(json.dumps(c).encode()+b'\n');raw=sock.makefile('rb').readline(16385)
 if not raw:raise RuntimeError('Live caller disconnected; see sanitized service status')
 return json.loads(raw)['reply']
results=[]
try:
 for _ in range(100):
  try:
   with socket.create_connection(('127.0.0.1',port),timeout=.1):break
  except OSError:time.sleep(.05)
 for index in [13,40]:
  person=cast[index]
  c=dict(version=4,kind='Opening',caller_name=person['name'],location='Queen Drive 4',report='A rumour about a fight outside a shop',personality=brief(person),caller_private='You passed on an unverified old message. You have not witnessed a fight. If asked about the source, admit your mistake without inventing new details.',facts=[dict(id='report',question='What was reported?',answer='An old message claimed a fight outside a shop.'),dict(id='source',question='Did you witness this?',answer='I passed on an unverified old message and did not witness it.'),dict(id='location',question='Where?',answer='Queen Drive 4')],history=[],question='')
  turns=[]
  for question in ['', 'Did you actually see this, or is it a rumour?', 'What do you mean? Explain it plainly.', 'Why do you keep saying the same thing?', 'Okay, I understand. What should I call you?']:
   c['kind']='Opening' if not question else 'Conversation';c['question']=question
   reply=ask(c);line=reply['spoken_text']
   assert line not in [t['reply']['spoken_text'] for t in turns]
   assert not reply['call_complete']
   turns.append(dict(question=question,reply=reply));c['history'].append([question,line])
  results.append(dict(name=person['name'],turns=turns))
 (out/'report.json').write_text(json.dumps(dict(result='PASS',cases=results),indent=2))
 print(json.dumps(dict(result='PASS',callers=[r['name'] for r in results],unique_replies=sum(len(r['turns']) for r in results))))
finally:proc.terminate();proc.wait(timeout=3);log.close()
