"""No paid requests: prove duplicate repair is live-provider-only and counts against budget."""
import http.server,json,os,socket,subprocess,threading,time
from pathlib import Path
requests=[]
old="I think I repeated an old message. I am not sure where it came from."
new="You're right to ask. I passed on a rumour without checking it, and that was careless."
class Fixture(http.server.BaseHTTPRequestHandler):
 def log_message(self,*args):pass
 def do_POST(self):
  body=json.loads(self.rfile.read(int(self.headers['Content-Length'])));requests.append(body)
  repair=body['messages'][-1]['role']=='system'
  reply=dict(spoken_text=new if repair else old,emotion='Confused',proposed_action='Answer',fact_references=['source'],call_complete=False)
  data=json.dumps({'choices':[{'message':{'content':json.dumps(reply)}}]}).encode()
  self.send_response(200);self.send_header('Content-Length',str(len(data)));self.end_headers();self.wfile.write(data)
server=http.server.ThreadingHTTPServer(('127.0.0.1',0),Fixture);threading.Thread(target=server.serve_forever,daemon=True).start()
with socket.socket() as sock:sock.bind(('127.0.0.1',0));port=sock.getsockname()[1]
env=os.environ.copy();env.update(CALLER_API_KEY='fixture-placeholder',CALLER_MODEL='fixture',CALLER_ENDPOINT=f'http://127.0.0.1:{server.server_port}/chat',CALLER_BIND=f'127.0.0.1:{port}',CALLER_MAX_REQUESTS='2')
binary=Path('target/debug/wrong-number-caller-server').resolve()
if os.name=='nt':binary=binary.with_suffix('.exe')
proc=subprocess.Popen([str(binary)],env=env,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
c=dict(version=4,report='Old rumour',personality='Embarrassed but honest',facts=[dict(id='source',question='Where did this come from?',answer='An unverified old message.')],history=[['What happened?',old]],question='Why are you repeating that?')
def ask():
 with socket.create_connection(('127.0.0.1',port),timeout=5) as sock:
  sock.sendall(json.dumps(c).encode()+b'\n');data=sock.makefile('rb').readline()
  return (lambda v: None if "error" in v else v)(json.loads(data)) if data else None
try:
 for _ in range(100):
  try:
   with socket.create_connection(('127.0.0.1',port),timeout=.1):break
  except OSError:time.sleep(.05)
 r=ask();assert r['reply']['spoken_text']==new
 assert len(requests)==2 and any(m['role']=='assistant' and m['content']==old for m in requests[0]['messages'])
 assert ask() is None and len(requests)==2
 print(json.dumps(dict(result='PASS',duplicate_rewritten=True,history_uses_chat_roles=True,both_attempts_budgeted=True,offline_fallback=False)))
finally:proc.terminate();proc.wait(timeout=3);server.shutdown()
