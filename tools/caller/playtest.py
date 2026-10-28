"""Exercise the actual trusted caller daemon against a local HTTP provider fixture."""
import argparse,http.server,json,os,socket,subprocess,threading,time
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--output',default='target/caller-test');a=p.parse_args();out=Path(a.output).resolve();out.mkdir(parents=True,exist_ok=True)
requests=[]
class Fixture(http.server.BaseHTTPRequestHandler):
 def log_message(self,*args):pass
 def do_POST(self):
  body=json.loads(self.rfile.read(int(self.headers['Content-Length'])));requests.append(body)
  assert body['max_tokens']==256 and body['response_format']['type']=='json_schema'
  context=json.loads(body['messages'][1]['content']);fact=context['facts'][0]
  answer={'spoken_text':fact['answer'],'emotion':'Calm','proposed_action':'Answer','fact_references':[fact['id']],'call_complete':False}
  if 'malformed' in context['question']:answer['spoken_text']='An invented disaster and secret dispatch order'
  data=json.dumps({'choices':[{'message':{'content':json.dumps(answer)}}]}).encode();self.send_response(200);self.send_header('Content-Length',str(len(data)));self.end_headers();self.wfile.write(data)
api=http.server.ThreadingHTTPServer(('127.0.0.1',0),Fixture);threading.Thread(target=api.serve_forever,daemon=True).start()
with socket.socket() as sock:sock.bind(('127.0.0.1',0));port=sock.getsockname()[1]
env=os.environ.copy()
for key in ['CALLER_API_KEY','OPENROUTER_API_KEY','CALLER_MODEL','CALLER_ENDPOINT','CALLER_BIND']:env.pop(key,None)
env.update(CALLER_API_KEY='local-fixture-placeholder',CALLER_MODEL='fixture-model',CALLER_ENDPOINT=f'http://127.0.0.1:{api.server_port}/chat',CALLER_BIND=f'127.0.0.1:{port}',CALLER_MAX_REQUESTS='2')
binary=Path('target/debug/wrong-number-caller-server').resolve();binary=binary.with_suffix('.exe') if os.name=='nt' else binary
log=(out/'service.log').open('w');proc=subprocess.Popen([str(binary)],env=env,stdout=log,stderr=subprocess.STDOUT)
context={'version':1,'report':'Smoke in laundromat','personality':'anxious','facts':[{'id':'smoke','question':'What can you see?','answer':'Black smoke from the dryer.'}],'history':[],'question':'What can you see?'}
def ask(question):
 c=dict(context,question=question)
 with socket.create_connection(('127.0.0.1',port),timeout=6) as sock:
  sock.sendall(json.dumps(c).encode()+b'\n');return json.loads(sock.makefile('rb').readline(16385))
try:
 deadline=time.monotonic()+5
 while True:
  try:
   with socket.create_connection(('127.0.0.1',port),timeout=.1):break
  except OSError:
   if time.monotonic()>deadline:raise
   time.sleep(.05)
 first=ask('What can you see?');assert first['source']=='provider' and first['reply']['spoken_text']==context['facts'][0]['answer']
 invalid=ask('What malformed event can you see?');assert invalid['source']=='scripted fallback' and invalid['reply']['spoken_text']==context['facts'][0]['answer']
 budget=ask('What can you see?');assert budget['source']=='scripted fallback' and len(requests)==2
 result={'result':'PASS','valid_provider':True,'hallucination_rejected':True,'request_budget_enforced':True,'paid_requests':0}
 (out/'report.json').write_text(json.dumps(result,indent=2));print(json.dumps(result,indent=2))
finally:
 proc.terminate();proc.wait(timeout=3);log.close();api.shutdown()
