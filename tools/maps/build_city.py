#!/usr/bin/env python3
"""Extract a first-pass, editable dispatch road graph from a dark 2D city PNG.
Dependencies: pip install opencv-python numpy scikit-image networkx
Usage: python build_city.py input.png --out ./city --width-km 4 --seed 42
Coordinates are pixel-space (x right, y down); distance is calibrated by --width-km.
This is image tracing, NOT guaranteed correct GIS data. Review overlay and edit corrections.json.
"""
import argparse,csv,json,math,random
from pathlib import Path
import cv2,numpy as np,networkx as nx
from skimage.morphology import skeletonize,remove_small_objects

DIRECTIONS=[(-1,-1),(-1,0),(-1,1),(0,-1),(0,1),(1,-1),(1,0),(1,1)]

def extract_mask(img, threshold=20):
    # Roads tend to be narrow, bright gray-blue ridges. Suppress broad bright buildings.
    lab=cv2.cvtColor(img,cv2.COLOR_BGR2LAB)
    light=lab[:,:,0]
    tophat=cv2.morphologyEx(light,cv2.MORPH_TOPHAT,cv2.getStructuringElement(cv2.MORPH_ELLIPSE,(11,11)))
    # Narrow high-contrast road lines, including local roads in darker regions.
    gray=cv2.cvtColor(img,cv2.COLOR_BGR2GRAY)
    blur=cv2.GaussianBlur(gray,(0,0),3)
    ridge=cv2.subtract(gray,blur)
    mask=((tophat>threshold)&(ridge>3)).astype('uint8')
    mask=cv2.morphologyEx(mask,cv2.MORPH_CLOSE,np.ones((2,2),np.uint8))
    mask=remove_small_objects(mask.astype(bool),min_size=12)
    return mask

def make_graph(mask, min_length=9, snap_radius=5):
    sk=skeletonize(mask)
    h,w=sk.shape
    ys,xs=np.nonzero(sk)
    pixels={(int(y),int(x)) for y,x in zip(ys,xs)}
    if not pixels:raise ValueError('No roads detected; lower --threshold')
    # Build pixel adjacency with diagonal corner-cut prevention.
    adj={}
    for y,x in pixels:
        neigh=[]
        for dy,dx in DIRECTIONS:
            q=(y+dy,x+dx)
            if q in pixels:
                if dy and dx and ((y+dy,x) in pixels or (y,x+dx) in pixels):continue
                neigh.append(q)
        adj[(y,x)]=neigh
    # Group junction clusters into representative nodes.
    key={p for p,nb in adj.items() if len(nb)!=2}
    # Connected key-pixel groups (reduces huge junction clouds).
    groups=[]; unseen=set(key)
    while unseen:
        p=unseen.pop(); group=[p];stack=[p]
        while stack:
            u=stack.pop()
            for v in adj[u]:
                if v in unseen:unseen.remove(v);stack.append(v);group.append(v)
        groups.append(group)
    node_at={};nodes=[]
    for group in groups:
        cy=int(round(sum(p[0] for p in group)/len(group)))
        cx=int(round(sum(p[1] for p in group)/len(group)))
        nid=len(nodes);nodes.append({'id':nid,'x':cx,'y':cy,'kind':'junction' if len(group)>1 else 'endpoint'})
        for p in group:node_at[p]=nid
    # Pure loops have no endpoint or junction; add one anchor per loop.
    for p in pixels:
        if p in node_at:continue
        # These are normally corridors; handled by tracing from existing nodes.
    graph=nx.MultiGraph();graph.add_nodes_from(range(len(nodes)))
    visited=set();roads=[]
    for p,nid in list(node_at.items()):
        for nb in adj[p]:
            e=frozenset((p,nb))
            if e in visited:continue
            visited.add(e);path=[p,nb];prev=p;cur=nb
            while cur not in node_at:
                options=[q for q in adj[cur] if q!=prev and frozenset((cur,q)) not in visited]
                if not options:break
                nxt=options[0];visited.add(frozenset((cur,nxt)));path.append(nxt);prev,cur=cur,nxt
            if cur not in node_at:continue
            end=node_at[cur]
            if nid==end:continue
            length=sum(math.hypot(b[0]-a[0],b[1]-a[1]) for a,b in zip(path,path[1:]))
            if length<min_length:continue
            road={'id':len(roads),'from_node':nid,'to_node':end,'points':[[int(x),int(y)] for y,x in path],'length_px':round(length,2)}
            roads.append(road);graph.add_edge(nid,end,key=road['id'],weight=length)
    # Remove isolated nodes and remap indices.
    active=sorted({r['from_node'] for r in roads}|{r['to_node'] for r in roads})
    remap={old:i for i,old in enumerate(active)}
    nodes=[dict(nodes[old],id=i) for i,old in enumerate(active)]
    for r in roads:r['from_node']=remap[r['from_node']];r['to_node']=remap[r['to_node']]
    return nodes,roads,sk

def road_names(nodes,roads,seed):
    # Connected nearly-collinear segments inherit names at junctions.
    rng=random.Random(seed)
    bases=['Oak','Maple','Pine','Cedar','Elm','Willow','River','Hill','Lake','Park','Church','Mill','Union','Washington','Lincoln','Main','Market','Harbor','Forest','Meadow','Birch','Ash','Walnut','Chestnut','Spring','Valley','Sunset','Bridge','Grove','North','South','West','East','Highland','King','Queen','Broad','Center','Garden','Station','Orchard','Magnolia','Fairview','Liberty','Franklin','Jackson','Jefferson','Adams']
    # Graph road-segment connectivity: merge if segment direction continues straight at junction.
    ends={}
    for r in roads:
        pts=r['points']
        if len(pts)<2:continue
        for node,vec in [(r['from_node'],(pts[1][0]-pts[0][0],pts[1][1]-pts[0][1])),(r['to_node'],(pts[-2][0]-pts[-1][0],pts[-2][1]-pts[-1][1]))]:
            d=math.hypot(*vec)
            if d:ends.setdefault(node,[]).append((r['id'],(vec[0]/d,vec[1]/d)))
    parent=list(range(len(roads)))
    def root(x):
        while parent[x]!=x:parent[x]=parent[parent[x]];x=parent[x]
        return x
    for group in ends.values():
        for i,(a,va) in enumerate(group):
            choices=[(va[0]*vb[0]+va[1]*vb[1],b) for j,(b,vb) in enumerate(group) if i!=j]
            if choices:
                dot,b=min(choices)
                if dot<-.90:parent[root(a)]=root(b)
    grouped={}
    for r in roads:grouped.setdefault(root(r['id']),[]).append(r)
    ordered=sorted(grouped.values(),key=lambda g:-sum(r['length_px'] for r in g))
    for i,group in enumerate(ordered):
        base=bases[i%len(bases)]
        suffix=['Street','Avenue','Road','Drive','Boulevard','Lane'][i//len(bases)%6]
        name=f'{base} {suffix}' if i<len(bases) else f'{base} {suffix} {i//len(bases)+1}'
        for r in group:r['name']=name
    return roads

def generate(img,args):
    h,w=img.shape[:2];m_per_px=args.width_km*1000/w
    mask=extract_mask(img,args.threshold)
    nodes,roads,sk=make_graph(mask,args.min_length)
    roads=road_names(nodes,roads,args.seed)
    rng=random.Random(args.seed)
    for r in roads:
        r['length_m']=round(r.pop('length_px')*m_per_px,2)
        r['road_type']='local';r['speed_limit_kmh']=40;r['one_way']=False;r['closed']=False
        r['travel_time_s']=round(r['length_m']/ (40/3.6),2)
    # Address points placed along longer detected road segments; each links to its edge.
    addresses=[]
    for r in roads:
        if r['length_m']<45:continue
        pts=r['points']; count=min(5,max(1,int(r['length_m']//85)))
        for j in range(count):
            ix=round((j+1)* (len(pts)-1)/(count+1));x,y=pts[ix]
            num=100+2*(len(addresses)%450)
            addresses.append({'id':f'addr_{len(addresses):05d}','address':f'{num} {r["name"]}', 'x':x,'y':y,'road_id':r['id'],'district':'Unassigned','building_type':rng.choice(['residential','residential','commercial'])})
    # Border: whole image extent; editable city border polygon. Do not infer jurisdiction from the image.
    border=[[0,0],[w-1,0],[w-1,h-1],[0,h-1],[0,0]]
    out=Path(args.out);out.mkdir(parents=True,exist_ok=True)
    meta={'name':'Wrong Number — Initial Traced City','source_image':str(args.image),'image_width_px':w,'image_height_px':h,'map_width_m':args.width_km*1000,'map_height_m':round(h*m_per_px,2),'meters_per_pixel':m_per_px,'coordinate_system':'image pixels: x right, y down','border_polygon_px':border,'warnings':['Automatically traced road network; visually verify all bridges, crossings, and road connections.','Street names and addresses are synthetic.','Pixel-scale distances are estimates based on specified map width.','City border is initially image rectangle; edit to set actual jurisdiction.']}
    for fn,data in [('city_config.json',meta),('road_graph.json',{'nodes':nodes,'roads':roads}),('addresses.json',addresses),('corrections.json',{'road_closures':[],'road_renames':{},'remove_road_ids':[],'border_polygon_px':border})]:
        (out/fn).write_text(json.dumps(data,indent=2))
    with (out/'roads.csv').open('w',newline='') as f:
        wr=csv.DictWriter(f,fieldnames=['id','name','from_node','to_node','length_m','speed_limit_kmh','one_way','road_type']);wr.writeheader();wr.writerows([{k:r[k] for k in wr.fieldnames} for r in roads])
    features=[{'type':'Feature','properties':{k:r[k] for k in ['id','name','length_m','speed_limit_kmh']},'geometry':{'type':'LineString','coordinates':r['points']}} for r in roads]
    features.append({'type':'Feature','properties':{'name':'Editable city boundary'},'geometry':{'type':'Polygon','coordinates':[border]}})
    (out/'city.geojson').write_text(json.dumps({'type':'FeatureCollection','features':features}))
    overlay=img.copy()
    for r in roads:
        pts=np.asarray(r['points'],np.int32).reshape(-1,1,2)
        cv2.polylines(overlay,[pts],False,(80,220,255),1,cv2.LINE_AA)
    for n in nodes:cv2.circle(overlay,(n['x'],n['y']),2,(10,190,255),-1)
    cv2.imwrite(str(out/'road_overlay.png'),overlay)
    cv2.imwrite(str(out/'road_mask.png'),(mask*255).astype(np.uint8))
    print(f'Generated {len(nodes)} nodes, {len(roads)} road edges, {len(addresses)} addresses in {out}')

def route(graph_path,start,end):
    data=json.loads(Path(graph_path).read_text());g=nx.Graph()
    for r in data['roads']:
        g.add_edge(r['from_node'],r['to_node'],weight=r['travel_time_s'],road_id=r['id'])
    try:
        path=nx.dijkstra_path(g,int(start),int(end),weight='weight')
        cost=nx.dijkstra_path_length(g,int(start),int(end),weight='weight')
        print(json.dumps({'nodes':path,'travel_time_s':round(cost,1)},indent=2))
    except (nx.NetworkXNoPath,nx.NodeNotFound):print('No route: disconnected network or invalid node ID')

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('image',nargs='?');p.add_argument('--out',default='city');p.add_argument('--width-km',type=float,default=4.0);p.add_argument('--seed',type=int,default=42);p.add_argument('--threshold',type=int,default=20);p.add_argument('--min-length',type=float,default=9);p.add_argument('--route',nargs=3,metavar=('GRAPH_JSON','START_NODE','END_NODE'))
    a=p.parse_args()
    if a.route:route(*a.route)
    elif a.image:
        im=cv2.imread(a.image)
        if im is None:p.error('Could not open image')
        generate(im,a)
    else:p.error('Provide image or --route')
