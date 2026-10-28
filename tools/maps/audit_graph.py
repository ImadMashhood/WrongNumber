"""Inspect an imported road graph without running its tracing script or altering it."""
import argparse
import json
import math
from collections import defaultdict
from pathlib import Path


def audit(path):
    graph = json.loads(Path(path).read_text())
    nodes = {node['id']: node for node in graph['nodes']}
    adjacency = defaultdict(set)
    errors = []
    if len(nodes) != len(graph['nodes']):
        errors.append('Duplicate node IDs')
    road_ids = set()
    for road in graph['roads']:
        if road['id'] in road_ids:
            errors.append(f"Duplicate road ID {road['id']}")
        road_ids.add(road['id'])
        a, b = road['from_node'], road['to_node']
        if a not in nodes or b not in nodes:
            errors.append(f"Road {road['id']} has a missing endpoint")
            continue
        for key in ('length_m', 'speed_limit_kmh', 'travel_time_s'):
            value = road.get(key)
            if not isinstance(value, (int, float)) or not math.isfinite(value) or value <= 0:
                errors.append(f"Road {road['id']} has invalid {key}")
        if not road.get('closed', False):
            # Weak connectivity is a necessary condition even when roads are directed.
            adjacency[a].add(b)
            adjacency[b].add(a)
    seen = set()
    components = []
    for node in nodes:
        if node in seen:
            continue
        stack = [node]
        seen.add(node)
        size = 0
        while stack:
            current = stack.pop()
            size += 1
            for neighbor in adjacency[current]:
                if neighbor not in seen:
                    seen.add(neighbor)
                    stack.append(neighbor)
        components.append(size)
    return {'nodes': len(nodes), 'roads': len(road_ids), 'errors': errors,
            'weak_components': len(components), 'largest_component_nodes': max(components, default=0),
            'citywide_connectivity': len(components) == 1,
            'manual_geometry_review_required': True}


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('graph', nargs='?', default='assets/city/road_graph.json')
    report = audit(parser.parse_args().graph)
    print(json.dumps(report, indent=2))
    raise SystemExit(1 if report['errors'] else 0)
