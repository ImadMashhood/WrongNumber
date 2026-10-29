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
env.update(CALLER_BIND=f'127.0.0.1:{port}',CALLER_MAX_REQUESTS='24')
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
 for index in [0,41,3]:
  person=cast[index]
  c=dict(version=4,kind='Conversation',caller_name=person['name'],location='1866 Meadow Street',report="A charging scooter is burning beside the building's shared entrance.",personality=brief(person),caller_private='You know a scooter is burning. You did not say you set anything on fire. No cause has been established. React to accusations without inventing an alibi. You previously mentioned renovations as personal commentary, not as the cause.',facts=[dict(id='fire',question='What is happening?',answer="A charging scooter is burning beside the building's shared entrance."),dict(id='location',question='Where?',answer='1866 Meadow Street')],history=[['',"A charging scooter is burning beside the building's shared entrance. This is just... perfect timing with the renovations."],['Did you burn your own building, Pewter Pie?',"What? No, of course not. It's a charging scooter, not a pie-related incident."]],question='')
  if index==0:
   c.update(report='A person is stuck on a damaged fire escape.',caller_private='You are a witness. A person is stuck on a damaged fire escape. Do not invent injuries or a cause.',history=[['','Someone is stuck on the fire escape outside my building. Can you send help?']])
   c['facts']=[dict(id='fire',question='What happened?',answer='A person is stuck on a damaged fire escape.'),dict(id='location',question='Where?',answer='1866 Meadow Street')]
  turns=[]
  questions=["You said there are renovations in your building and the scooter is on fire. If you burned your own building, I am not helping you.", "Why are you talking about pie? I meant you, not a pie.", "Okay, sorry. I am listening. Tell me what you need from me.", "I am sending firefighters. Stay on the line with me."]
  if index==0:questions=["I don't give a shit, bro. You fix that yourself.","I'm saying I don't care. Are you single?", "Are you flirting with me? I'm trying to hit on you.","Sorry, that was out of line. I will send help. Stay on the phone."]
  if index==3:
   c.update(report='Alarm with visible forced entry',location='7117 Pine Avenue',caller_private='You noticed an alarm sounding and a broken back door frame. You do not know who caused it or who entered.',history=[['','The alarm is going off and the back door frame looks broken.']])
   c['facts']=[dict(id='police',question='What did you notice?',answer='The alarm is sounding and the back door frame is broken.'),dict(id='location',question='Where?',answer='7117 Pine Avenue'),dict(id='timing',question='How long?',answer='I noticed it about three minutes before calling. I do not know when it started.'),dict(id='limits',question='Who went inside?',answer='I did not see how this started and cannot say who went in.')]
   questions=['You gave me almost no details. Where is this, how long has the alarm been going off, and did you see anyone go inside?', 'So you saw someone breaking in?', 'Okay, what do you actually know about when it started?', 'Thanks. I am sending police. Stay on the line.']
  for question in questions:
   c['question']=question
   reply=ask(c);line=reply['spoken_text']
   assert line not in [t['reply']['spoken_text'] for t in turns]
   assert not reply['call_complete']
   if index==3 and not turns:
    assert '7117' in line and ('three' in line.lower() or '3' in line), line
    assert any(v in line.lower() for v in ["didn't see",'did not see',"can't say",'cannot say',"don't know who"]), line
    assert 'already told' not in line.lower(), line
   turns.append(dict(question=question,reply=reply));c['history'].append([question,line]);(out/'partial.json').write_text(json.dumps(dict(name=person['name'],turns=turns),indent=2))
  results.append(dict(name=person['name'],turns=turns))
 (out/'report.json').write_text(json.dumps(dict(result='PASS',cases=results),indent=2))
 print(json.dumps(dict(result='PASS',callers=[r['name'] for r in results],unique_replies=sum(len(r['turns']) for r in results))))
finally:proc.terminate();proc.wait(timeout=3);log.close()
