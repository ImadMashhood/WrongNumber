# Wrong Number — image-derived city routing starter

This is a **first-pass traced graph**, not a production-ready road map. The source PNG is not GIS data; road intersections, bridge connections, shorelines, and some building edges are imperfectly detected. In particular, the yellow overlay shows **false-positive coastlines** and many disconnected street fragments. You must review/edit the network before relying on emergency dispatch routing.

## Setup

```bash
python -m pip install opencv-python numpy scikit-image networkx
python build_city.py source_map.png --out city --width-km 4 --seed 42
```

## Output

- `city/road_overlay.png`: detected graph over original PNG (yellow/orange tracing)
- `city/road_mask.png`: extracted candidate line pixels
- `city/road_graph.json`: node coordinates and road edges, lengths, speed, time
- `city/roads.csv`: spreadsheet-friendly road list
- `city/addresses.json`: generated dispatchable address points, tied to edges
- `city/city.geojson`: vector line features and initial boundary
- `city/city_config.json`: map scale and editable boundary
- `city/corrections.json`: proposed manual edit manifest (not yet applied by script)

## Test Dijkstra routing

```bash
python build_city.py --route city/road_graph.json 0 10
```

Disconnected networks can have no route. The pathfinder uses travel-time weights.

## Production follow-up

1. Create a small manual road-editor for splitting/merging intersections and removing shoreline traces.
2. Make the corrected road graph authoritative; rerun validation for connected components, reachable addresses, and crossing geometry.
3. Rebuild address numbers to be unique per street and add road-side access nodes.
4. Define a polygon jurisdiction border rather than the current image rectangle.
5. Store street naming by continuous centerline, assign road classes/speeds and direction, then support closures and turn restrictions.
6. Export the validated graph to Bevy/Rust; server handles A*/Dijkstra, client renders the route.

The initial synthetic addresses, names, speeds, and 4 km width are design placeholders, not facts recovered from the image.
