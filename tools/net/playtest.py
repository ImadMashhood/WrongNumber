"""Run real host/client OS processes through a complete accelerated co-op shift."""
import argparse,json,socket,subprocess,time,os
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--reconnect',action='store_true');p.add_argument('--dialogue-test',action='store_true');p.add_argument('--players',type=int,choices=[2,6],default=2);p.add_argument('--binary',default='target/debug/wrong-number');p.add_argument('--output',default='work/net-playtest');args=p.parse_args()
out=Path(args.output).resolve();out.mkdir(parents=True,exist_ok=True)
with socket.socket() as s:s.bind(('127.0.0.1',0));port=s.getsockname()[1]
address=f'127.0.0.1:{port}';binary=str(Path(args.binary).resolve().with_suffix('.exe') if os.name=='nt' else Path(args.binary).resolve());processes=[];files=[];clients={};reconnected=False;terminated=set()
def launch(name,argv):
 log=(out/f'{name}.log').open('w');files.append(log);proc=subprocess.Popen([binary,*argv,*(['--dialogue-test'] if args.dialogue_test else [])],stdout=log,stderr=subprocess.STDOUT);processes.append(proc);return proc
try:
 launch('host',['--headless-host',address,'--name','Host','--expected-players',str(args.players),'--fast-shift','40','--one-shift','--network-report',str(out/'host.json')]);time.sleep(.5)
 for n in range(2,args.players+1):clients[n]=launch(f'client{n}',['--bot',address,'--name',f'Player{n}','--resume',str(out/f'resume{n}.json'),'--network-report',str(out/f'client{n}.json')])
 deadline=time.monotonic()+100
 while any(p.poll() is None for p in processes) and time.monotonic()<deadline:
  if args.reconnect and not reconnected and 'NETWORK PHASE Running' in (out/'host.log').read_text():
   victim=clients[2];victim.terminate();victim.wait(timeout=3);terminated.add(victim);time.sleep(.5)
   clients[2]=launch('client2-reconnected',['--bot',address,'--name','Player2','--resume',str(out/'resume2.json'),'--network-report',str(out/'client2.json')]);reconnected=True
  time.sleep(.25)
 assert all(p.poll()==0 for p in processes if p not in terminated),f'Process failure/timeout; see {out}'
 reports=[json.loads((out/name).read_text()) for name in ['host.json',*[f'client{n}.json' for n in range(2,args.players+1)]]]
 assert len({r['digest'] for r in reports})==1,reports
 assert all(r['peak_players']==args.players and r['completed_shift']==1 and r['clock']==0 and r['mirror_matches'] for r in reports),reports
 assert len({r['player'] for r in reports})==args.players,reports
 print(json.dumps({'players':args.players,'result':'PASS','reconnect_verified':reconnected,'dialogue_test':args.dialogue_test,'shared_digest':f"{reports[0]['digest']:016x}",'reports':str(out)},indent=2))
finally:
 for proc in processes:
  if proc.poll() is None:proc.terminate()
 for proc in processes:
  try:proc.wait(timeout=3)
  except subprocess.TimeoutExpired:proc.kill()
 for f in files:f.close()
 # Never deliver or upload reconnect credentials with test reports.
 for credential in out.glob('resume*.json'):credential.unlink(missing_ok=True)
