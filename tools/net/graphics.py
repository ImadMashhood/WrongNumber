"""Two rendered Bevy processes, actual lobby button and complete shared shift."""
import argparse, json, os, socket, subprocess, time
from pathlib import Path
parser=argparse.ArgumentParser();parser.add_argument('--catalogue-test',action='store_true');parser.add_argument('--output',default='target/network-graphics');parser.add_argument('--binary',default='target/debug/wrong-number');args=parser.parse_args()
out=Path(args.output).resolve();out.mkdir(parents=True,exist_ok=True)
binary=Path(args.binary).resolve();binary=binary.with_suffix('.exe') if os.name=='nt' else binary
env=os.environ.copy();env['BEVY_ASSET_ROOT']=str(Path.cwd());env['WGPU_BACKEND']='vulkan'
nvidia=Path('/usr/share/vulkan/icd.d/nvidia_icd.x86_64.json')
if nvidia.exists():env['VK_DRIVER_FILES']=str(nvidia)
with socket.socket() as s:s.bind(('127.0.0.1',0));address=f'127.0.0.1:{s.getsockname()[1]}'
processes=[];files=[]
try:
 for role,flag,name in [('host','--host','Host'),('client','--join','Partner')]:
  capture=out/role;capture.mkdir(exist_ok=True);log=(out/f'{role}.log').open('w');files.append(log)
  processes.append(subprocess.Popen([str(binary),flag,address,'--name',name,'--network-playtest','--fast-shift','40','--capture-dir',str(capture),'--network-report',str(out/f'{role}.json'),'--resume',str(out/'resume.json'),*(['--catalogue-test','--dialogue-test'] if args.catalogue_test else [])],env=env,stdout=log,stderr=subprocess.STDOUT))
  time.sleep(1)
 deadline=time.monotonic()+150
 while any(p.poll() is None for p in processes) and time.monotonic()<deadline:time.sleep(.5)
 assert all(p.poll()==0 for p in processes),f'Graphics failure/timeout: {out}'
 reports=[json.loads((out/f'{role}.json').read_text()) for role in ['host','client']]
 assert len({r['digest'] for r in reports})==1,reports
 assert {r['player'] for r in reports}=={1,2} and all(r['shift']==2 and r['mirror_matches'] for r in reports),reports
 print(json.dumps({'result':'PASS','reports':reports},indent=2))
finally:
 for p in processes:
  if p.poll() is None:p.terminate()
 for p in processes:
  try:p.wait(timeout=3)
  except subprocess.TimeoutExpired:p.kill()
 for f in files:f.close()
 (out/'resume.json').unlink(missing_ok=True)
