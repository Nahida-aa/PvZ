#!/usr/bin/env python3
"""Convert Godot Animation .tres to our animation JSONC format."""
import json
import re
import sys
from pathlib import Path
from collections import OrderedDict


def parse_ext_resources(text):
    """Parse ext_resource lines, return {id: path_basename}."""
    resources = {}
    for m in re.finditer(
        r'\[ext_resource[^\]]*path="res://([^"]+)"[^\]]*id="([^"]+)"', text
    ):
        path, rid = m.group(1), m.group(2)
        resources[rid] = Path(path).name
    return resources


def parse_packed_array(text, dtype="float"):
    """Parse PackedFloat32Array(...) or similar."""
    m = re.search(r"PackedFloat32Array\(([^)]*)\)", text)
    if not m:
        return []
    vals = m.group(1).strip()
    if not vals:
        return []
    if dtype == "float":
        return [float(x.strip()) for x in vals.split(",") if x.strip()]
    elif dtype == "vector2":
        # Not directly in PackedFloat32Array — values come as [Vector2(...), ...]
        return vals
    return vals


def parse_values_block(text):
    """Parse the values array which can contain scalars, Vector2, or ExtResource refs."""
    # Find "values": [...] block
    m = re.search(r'"values"\s*:\s*\[(.*?)\]', text, re.DOTALL)
    if not m:
        return []
    raw = m.group(1).strip()
    if not raw:
        return []

    values = []
    # Split by comma, but respect nesting
    depth = 0
    current = ""
    for ch in raw:
        if ch in "([{" :
            depth += 1
            current += ch
        elif ch in ")]}":
            depth -= 1
            current += ch
        elif ch == "," and depth == 0:
            values.append(current.strip())
            current = ""
        else:
            current += ch
    if current.strip():
        values.append(current.strip())

    return values


def parse_vector2(text):
    """Parse Vector2(x, y) to [x, y]."""
    m = re.match(r"Vector2\(([^,]+),\s*([^)]+)\)", text.strip())
    if m:
        return [float(m.group(1)), float(m.group(2))]
    return None


def parse_ext_ref(text, ext_resources):
    """Parse ExtResource("id") to filename."""
    m = re.match(r'ExtResource\("([^"]+)"\)', text.strip())
    if m:
        return ext_resources.get(m.group(1), None)
    return None


def convert_tres_to_jsonc(tres_path):
    """Convert a Godot .tres Animation file to our animation JSONC format."""
    text = Path(tres_path).read_text()

    ext_resources = parse_ext_resources(text)

    # Parse metadata
    length_m = re.search(r"length\s*=\s*([\d.]+)", text)
    step_m = re.search(r"step\s*=\s*([\d.]+)", text)
    loop_m = re.search(r"loop_mode\s*=\s*(\d+)", text)

    duration = float(length_m.group(1)) if length_m else 2.0
    step = float(step_m.group(1)) if step_m else 0.083333
    fps = int(round(1.0 / step)) if step > 0 else 12
    loop_anim = int(loop_m.group(1)) if loop_m else 0

    # Parse tracks
    # Split by tracks/N/
    track_blocks = re.split(r"(?=\[node name=\"tracks/\d+/)", text) if False else []

    # Better: find all track blocks
    track_pattern = re.compile(
        r"tracks/(\d+)/type\s*=\s*\"([^\"]+)\".*?"
        r"tracks/\1/path\s*=\s*NodePath\(\"([^\"]+)\"\).*?"
        r"tracks/\1/interp\s*=\s*(\d+).*?"
        r"tracks/\1/keys\s*=\s*\{(.*?)\n\}",
        re.DOTALL,
    )

    nodes = OrderedDict()

    for m in track_pattern.finditer(text):
        track_type = m.group(2)
        node_path = m.group(3)
        interp = int(m.group(4))  # 0=Linear, 1=Step, 2=Cubic
        keys_text = m.group(5)

        if track_type != "value":
            continue

        # Parse path: "Body/BodyCorrect/NodeName:property"
        path_m = re.match(r"Body/BodyCorrect/([^:]+):(.+)", node_path)
        if not path_m:
            continue

        node_name = path_m.group(1)
        property = path_m.group(2)

        # Parse times
        times_m = re.search(r'"times"\s*:\s*PackedFloat32Array\(([^)]*)\)', keys_text)
        if not times_m:
            continue
        times_str = times_m.group(1).strip()
        if not times_str:
            continue
        times = [float(t.strip()) for t in times_str.split(",") if t.strip()]

        # Parse values
        values = parse_values_block(keys_text)

        if len(times) != len(values):
            # skip mismatched tracks
            continue

        if node_name not in nodes:
            nodes[node_name] = {
                "position": [],
                "scale": [],
                "rotation": [],
                "skew": [],
                "texture": None,
                "_texture_name": None,
                "position_interp": 0,
                "scale_interp": 0,
                "rotation_interp": 0,
                "skew_interp": 0,
            }

        node = nodes[node_name]

        if property == "position":
            node["position"] = [(t, parse_vector2(v)) for t, v in zip(times, values) if parse_vector2(v)]
            node["position_interp"] = interp
        elif property == "scale":
            node["scale"] = [(t, parse_vector2(v)) for t, v in zip(times, values) if parse_vector2(v)]
            node["scale_interp"] = interp
        elif property == "rotation":
            node["rotation"] = [(t, float(v)) for t, v in zip(times, values)]
            node["rotation_interp"] = interp
        elif property == "skew":
            node["skew"] = [(t, float(v)) for t, v in zip(times, values)]
            node["skew_interp"] = interp
        elif property == "texture":
            # Texture track — take first value as the texture
            ref = parse_ext_ref(values[0], ext_resources) if values else None
            node["_texture_name"] = ref
        elif property == "visible":
            # Take first value
            if values:
                node["visible"] = values[0].strip() == "true"

    # Build JSONC output
    out_nodes = OrderedDict()
    for name, node in nodes.items():
        out_node = OrderedDict()
        if "visible" in node:
            out_node["visible"] = node["visible"]
        if node["_texture_name"]:
            out_node["texture"] = node["_texture_name"]
        if node["position"]:
            out_node["position"] = [[t, v] for t, v in node["position"]]
            out_node["position_interp"] = node["position_interp"]
        if node["scale"]:
            out_node["scale"] = [[t, v] for t, v in node["scale"]]
            out_node["scale_interp"] = node["scale_interp"]
        if node["rotation"]:
            out_node["rotation"] = [[t, v] for t, v in node["rotation"]]
            out_node["rotation_interp"] = node["rotation_interp"]
        if node["skew"]:
            out_node["skew"] = [[t, v] for t, v in node["skew"]]
            out_node["skew_interp"] = node["skew_interp"]
        out_nodes[name] = out_node

    result = {
        "duration": duration,
        "fps": fps,
        "loop": loop_anim != 0,
        "nodes": out_nodes,
    }

    return result


def main():
    if len(sys.argv) != 3:
        print(f"Usage: {sys.argv[0]} <input.tres> <output.jsonc>")
        sys.exit(1)

    tres_path = sys.argv[1]
    out_path = sys.argv[2]

    result = convert_tres_to_jsonc(tres_path)

    with open(out_path, "w") as f:
        json.dump(result, f, indent=2)

    print(f"Converted {tres_path} -> {out_path}")
    print(f"  duration={result['duration']}, fps={result['fps']}, loop={result['loop']}")
    print(f"  nodes: {list(result['nodes'].keys())}")


if __name__ == "__main__":
    main()
