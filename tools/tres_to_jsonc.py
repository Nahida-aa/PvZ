#!/usr/bin/env python3
"""Convert Godot .tres animation to .jsonc format for Bevy."""
import json
import re
import sys
from pathlib import Path


def parse_tres(path):
    """Parse a Godot .tres animation file."""
    text = Path(path).read_text()

    # Parse ext_resources
    ext_resources = {}
    for m in re.finditer(
        r'\[ext_resource type="Texture2D" uid="[^"]*" path="([^"]*)" id="([^"]*)"\]',
        text,
    ):
        ext_resources[m.group(2)] = m.group(1).split("/")[-1]

    # Parse header
    length = float(re.search(r"length = ([\d.]+)", text).group(1))
    loop_mode = int(re.search(r"loop_mode = (\d+)", text).group(1))
    step = float(re.search(r"step = ([\d.]+)", text).group(1))

    # Parse tracks
    tracks = []
    track_blocks = re.split(r"(?=\ntracks/\d+/type)", text)
    for block in track_blocks:
        m = re.search(r"tracks/(\d+)/path = NodePath\(\"([^\"]+)\"\)", block)
        if not m:
            continue
        idx = int(m.group(1))
        node_path = m.group(2)

        # Extract property from path (last segment after ":")
        prop_match = re.search(r":(\w+)$", node_path)
        if not prop_match:
            continue
        prop = prop_match.group(1)
        node = node_path.rsplit(":", 1)[0].split("/")[-1]

        # Extract values
        values_match = re.search(r'"values": \[(.*?)\]', block, re.DOTALL)
        if not values_match:
            continue
        raw_values = values_match.group(1).strip()
        values = parse_values(raw_values, prop)

        # Extract times
        times_match = re.search(r'"times": PackedFloat32Array\(([^)]+)\)', block)
        if not times_match:
            continue
        times = [float(x) for x in times_match.group(1).split(",")]

        tracks.append(
            {
                "node": node,
                "property": prop,
                "times": times,
                "values": values,
            }
        )

    return {
        "ext_resources": ext_resources,
        "length": length,
        "loop": loop_mode > 0,
        "step": step,
        "tracks": tracks,
    }


def parse_values(raw, prop):
    """Parse Godot value literals into Python objects."""
    if prop == "visible":
        return [v.strip() == "true" for v in raw.split(",") if v.strip()]
    elif prop == "texture":
        vals = []
        for v in raw.split(","):
            v = v.strip()
            m = re.search(r'ExtResource\("([^"]+)"\)', v)
            if m:
                vals.append(m.group(1))
            else:
                vals.append(None)
        return vals
    elif prop == "position":
        return [
            [float(x) for x in m.group(1).split(",")]
            for m in re.finditer(r"Vector2\(([^)]+)\)", raw)
        ]
    elif prop == "scale":
        return [
            [float(x) for x in m.group(1).split(",")]
            for m in re.finditer(r"Vector2\(([^)]+)\)", raw)
        ]
    elif prop in ("rotation", "skew"):
        return [float(v) for v in raw.split(",") if v.strip()]
    elif prop == "self_modulate":
        return [
            [float(x) for x in m.group(1).split(",")]
            for m in re.finditer(r"Color\(([^)]+)\)", raw)
        ]
    else:
        return raw


def is_animated(track):
    """Check if a track has changing values."""
    vals = track["values"]
    if len(vals) < 2:
        return False
    first = vals[0]
    return any(v != first for v in vals[1:])


def convert(tres_path, output_path):
    """Convert .tres to .jsonc."""
    data = parse_tres(tres_path)

    ext_resources = data["ext_resources"]

    # Build nodes dict
    nodes = {}
    for track in data["tracks"]:
        node = track["node"]
        prop = track["property"]

        if node not in nodes:
            nodes[node] = {}

        if prop == "texture":
            # Always include texture (static or animated)
            nodes[node][prop] = ext_resources.get(track["values"][0], None)
        elif prop == "visible":
            # Always include visible (static or animated)
            nodes[node][prop] = track["values"][0]
        elif is_animated(track):
            # Store animated properties as keyframes: [[time, value], ...]
            keyframes = []
            for t, v in zip(track["times"], track["values"]):
                keyframes.append([round(t, 6), v])
            nodes[node][prop] = keyframes
        # Skip constant transform properties (they match defaults)

    result = {
        "duration": data["length"],
        "fps": round(1.0 / data["step"]),
        "loop": data["loop"],
        "nodes": nodes,
    }

    # Write JSONC
    Path(output_path).parent.mkdir(parents=True, exist_ok=True)
    with open(output_path, "w") as f:
        json.dump(result, f, indent=2)
    print(f"Converted {tres_path} -> {output_path}")
    print(f"  Duration: {data['length']:.6f}s, FPS: {round(1.0/data['step'])}, Loop: {data['loop']}")
    print(f"  Animated nodes: {len(nodes)}")


if __name__ == "__main__":
    if len(sys.argv) != 3:
        print(f"Usage: {sys.argv[0]} <input.tres> <output.jsonc>")
        sys.exit(1)
    convert(sys.argv[1], sys.argv[2])
