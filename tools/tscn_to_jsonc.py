#!/usr/bin/env python3
"""Convert Godot .tscn scene file to JSONC for the Python renderer.

Extracts visual rendering data: node hierarchy, Sprite2D properties,
ext_resource texture paths. Preserves instance inheritance links.
Filters out non-visual nodes.
"""
import json
import re
import sys
from pathlib import Path

SKIP_TYPES = {
    "Timer", "Area2D", "Label", "Control", "ProgressBar",
    "CollisionShape2D", "AnimationPlayer", "AnimationTree",
    "Marker2D", "Button", "RichTextLabel",
}

RENDER_RELEVANT_KEYS = [
    "position", "scale", "rotation", "skew", "texture",
    "visible", "centered", "z_index", "modulate",
    "flip_h", "flip_v", "size", "stretch",
    "transparent_bg", "render_target_update_mode",
]


def parse_value(s):
    s = s.strip()
    if s == "true":
        return True
    if s == "false":
        return False
    m = re.match(r"Vector2\(([^)]+)\)", s)
    if m:
        return [float(x.strip()) for x in m.group(1).split(",")]
    m = re.match(r"Vector2i\(([^)]+)\)", s)
    if m:
        return [int(x.strip()) for x in m.group(1).split(",")]
    m = re.match(r"Color\(([^)]+)\)", s)
    if m:
        return [float(x.strip()) for x in m.group(1).split(",")]
    m = re.match(r'ExtResource\("([^"]+)"\)', s)
    if m:
        return {"_ext": m.group(1)}
    m = re.match(r'PackedStringArray\(([^)]*)\)', s)
    if m:
        inner = m.group(1).strip()
        return re.findall(r'"([^"]*)"', inner) if inner else []
    try:
        return float(s) if ("." in s or "e" in s.lower()) else int(s)
    except ValueError:
        pass
    if s.startswith('"') and s.endswith('"'):
        return s[1:-1]
    if s.startswith("[") and s.endswith("]"):
        inner = s[1:-1].strip()
        return [parse_value(x.strip()) for x in inner.split(",")] if inner else []
    return s


def parse_tscn(path):
    """Parse a .tscn file. Returns (ext_resources, nodes, instance_res_path).

    ext_resources: { id: "res://..." }  — full res:// paths preserved
    nodes: list of { name, type, parent, index, properties }
    instance_res_path: "res://..." if root node has instance=, else None
    """
    lines = Path(path).read_text(encoding="utf-8").split("\n")
    ext_resources = {}  # id -> res:// path
    nodes = []
    current_node = None
    current_props = {}
    instance_res_path = None

    for line in lines:
        line = line.strip()
        if not line or line.startswith(";"):
            continue

        m = re.match(r"^\[(\w+)\s*(.*)\]$", line)
        if m:
            stype, attrs_str = m.group(1), m.group(2)
            if current_node:
                current_node["properties"] = current_props
                nodes.append(current_node)
                current_node = None
                current_props = {}

            if stype == "ext_resource":
                attrs = dict(re.findall(r'(\w+)="([^"]*)"', attrs_str))
                ext_resources[attrs.get("id", "")] = attrs.get("path", "")
            elif stype == "node":
                attrs = dict(re.findall(r'(\w+)="([^"]*)"', attrs_str))
                # instance=ExtResource("...") — extract ref id
                inst_m = re.search(r'instance=ExtResource\("([^"]+)"\)', attrs_str)
                idx = attrs.get("index")
                current_node = {
                    "name": attrs.get("name", ""),
                    "type": attrs.get("type", ""),
                    "parent": attrs.get("parent", "."),
                    "index": int(idx) if idx is not None else None,
                    "instance": inst_m.group(1) if inst_m else None,
                }
                current_props = {}
            continue

        if current_node:
            km = re.match(r'^(\w+)\s*=\s*(.+)$', line)
            if km:
                key, val = km.group(1), parse_value(km.group(2))
                if isinstance(val, dict) and "_ext" in val:
                    val = ext_resources.get(val["_ext"], f"Unknown({val['_ext']})")
                current_props[key] = val

    if current_node:
        current_node["properties"] = current_props
        nodes.append(current_node)

    # Check root node for instance (inheritance)
    for node in nodes:
        if node["parent"] == ".":
            inst_ref = node.get("instance")  # ExtResource ref id like "1_e772o"
            if inst_ref and inst_ref in ext_resources:
                res_path = ext_resources[inst_ref]
                # Convert res://...tscn → res://...jsonc
                instance_res_path = re.sub(r'\.tscn$', '.jsonc', res_path)
            break

    return ext_resources, nodes, instance_res_path


def main():
    if len(sys.argv) < 3:
        print(f"Usage: {sys.argv[0]} <input.tscn> <output.jsonc>")
        sys.exit(1)

    input_path = Path(sys.argv[1])
    output_path = Path(sys.argv[2])
    output_path.parent.mkdir(parents=True, exist_ok=True)

    ext_resources, nodes, instance_res_path = parse_tscn(input_path)

    output = {"source": input_path.name}
    if instance_res_path:
        output["instance"] = instance_res_path
    output["nodes"] = {}

    for node in nodes:
        if node["type"] in SKIP_TYPES:
            continue
        entry = {"type": node["type"], "parent": node["parent"]}
        if node["index"] is not None:
            entry["index"] = node["index"]
        for key in RENDER_RELEVANT_KEYS:
            if key in node.get("properties", {}):
                entry[key] = node["properties"][key]
        output["nodes"][node["name"]] = entry

    json_str = json.dumps(output, indent=2, ensure_ascii=False) + "\n"
    output_path.write_text(json_str, encoding="utf-8")
    inst_str = f" (inherits {instance_res_path})" if instance_res_path else ""
    print(f"{input_path.name} -> {output_path} ({len(output['nodes'])} nodes){inst_str}")


if __name__ == "__main__":
    main()
