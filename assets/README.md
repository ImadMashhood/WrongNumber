# Asset provenance

The office and skyline are procedural Bevy primitives.

## Imported coastal map starter

Source: user-provided `Wrong_Number_Road_Map_Starter.zip`, supplied on October 27, 2026.

- `city/map.png`: the package's original `source_map.png`, copied without image edits; used as the M2 map background.
- `city/road_graph.json`, `addresses.json`, and `corrections.json`: imported data, compacted as JSON without changing values.
- `city/city_config.json`: original metadata, with the stale source-image path replaced by the repository-relative path.
- `city/import_report.json`: initial audit summary. The graph is a draft and is not used for M2 travel.
- `tools/maps/build_city.py` and `STARTER_README.md`: supplied tracing script and source documentation, preserved for future editing. The tracing script was not executed during import.

Names, addresses, 4 km scale, and station positions remain provisional. The import contains 694 disconnected weak components, so route connectivity and geometry need review. The larger original archive, overlay, road mask, CSV, and GeoJSON remain available in the supplied ZIP.

For future external art/audio, record source, author, license, and required attribution here and retain supplied license files.
