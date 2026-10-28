"""Opt-in live provider/Whisper/TTS check. Reads only the ignored local server config."""
import argparse,json,os,socket,subprocess,time,wave
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--output',default='target/live-caller');p.add_argument('--stt-command',required=True);args=p.parse_args()
out=Path(args.output).resolve();out.mkdir(parents=True,exist_ok=True)
env=os.environ.copy()
for line in Path('.wrong-number/caller.env').read_text().splitlines():
 if '=' in line and not line.startswith('#'):
  k,v=line.split('=',1);env[k]=v
with socket.socket() as s:s.bind(('127.0.0.1',0));port=s.getsockname()[1]
env['CALLER_BIND']=f'127.0.0.1:{port}';env['CALLER_MAX_REQUESTS']='8'
binary=Path('target/debug/wrong-number-caller-server').resolve();log=(out/'service.log').open('w');proc=subprocess.Popen([str(binary)],env=env,stdout=log,stderr=subprocess.STDOUT)
catalog=json.loads(Path('assets/scenarios/complaints.json').read_text())['complaints'];reports=[]
def ask(c):
 start=time.monotonic()
 with socket.create_connection(('127.0.0.1',port),timeout=24) as s:
  s.sendall(json.dumps(c).encode()+b'\n');line=s.makefile('rb').readline(16385)
 if not line:raise RuntimeError('Live provider returned no accepted response; conversation disconnected')
 reply=json.loads(line)['reply'];assert 0<len(reply['spoken_text'])<=512
 assert set(reply['fact_references'])<=set(f['id'] for f in c['facts'])
 return reply,round((time.monotonic()-start)*1000)
def speak(text,path):
 txt=path.with_suffix('.txt');txt.write_text(text);subprocess.run(['espeak-ng','-s','140','-f',str(txt),'-w',str(path)],check=True,timeout=8)
 with wave.open(str(path)) as wav:assert wav.getnframes()>0
try:
 deadline=time.monotonic()+5
 while True:
  try:
   with socket.create_connection(('127.0.0.1',port),timeout=.1):break
  except OSError:
   if time.monotonic()>deadline:raise
   time.sleep(.05)
 for index,(card_index,fake) in enumerate([(103,False),(72,False),(1,True)]):
  card=catalog[card_index];address='Queen Drive 4 / junction 1010, 260'
  verification='I made this report up; there is no such event here.' if fake else ('I am witnessing this here now.' if card['needs_response'] else 'The complaint is real but nobody is in danger and no urgent response is needed.')
  c={'version':2,'kind':'Opening','caller_name':'Jordan Ellis','report':card['title'],'location':address,'personality':'Jordan Ellis, an adult local resident. Nervous, easily flustered, using short everyday sentences.','caller_private':('This is a deliberately fabricated report inside a fictional game. Maintain the initial claimed story, but admit it is invented if directly asked to verify it. Do not add any other alleged events.' if fake else 'This is a genuine report. Describe only the supplied observations; do not invent a prank or additional events.'),'facts':[{'id':'observation','question':'What can you see?','answer':card['observation']},{'id':'verification','question':'Did you witness it here now? Is this report real?','answer':verification},{'id':'location','question':'Where are you?','answer':address}],'history':[],'question':''}
  opening,opening_ms=ask(c);assert opening['proposed_action']=='Opening' and not opening['fact_references'] and not opening['call_complete'];assert opening['spoken_text']!=card['title']
  speak(opening['spoken_text'],out/f'{index+1}-opening.wav')
  question='Is the car on fire? Is anybody still inside?' if index==0 else ('Did you really witness this here now, or did you make the report up?' if fake else 'What exactly is happening? Is anybody in danger?')
  if index==0:
   speak(question,out/'dispatcher-question.wav');text=out/'whisper-question.txt';subprocess.run([args.stt_command,str(out/'dispatcher-question.wav'),str(text)],check=True,timeout=20);question=text.read_text().strip();assert question
  c['kind']='Conversation';c['question']=question;c['history']=[('',opening['spoken_text'])]
  reply,reply_ms=ask(c);assert reply['proposed_action'] in ['Answer','Clarify'];assert not reply['call_complete']
  if fake:assert 'verification' in reply['fact_references']
  speak(reply['spoken_text'],out/f'{index+1}-reply.wav')
  reports.append({'catalogue_id':card['id'],'complaint':card['title'],'false_report':fake,'opening':opening,'question':question,'reply':reply,'opening_ms':opening_ms,'reply_ms':reply_ms,'not_verbatim':reply['spoken_text'] not in [f['answer'] for f in c['facts']]})
 result={'result':'PASS','model':env['CALLER_MODEL'],'cases':reports,'pipeline':'synthetic dispatcher audio -> local Whisper -> live caller provider -> local TTS'}
 (out/'report.json').write_text(json.dumps(result,indent=2));print(json.dumps(result,indent=2))
finally:
 proc.terminate();proc.wait(timeout=3);log.close()
