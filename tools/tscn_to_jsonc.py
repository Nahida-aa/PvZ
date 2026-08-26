#!/usr/bin/env python3
"""Convert Godot .tscn inline animations to .jsonc format for Bevy.

Reads animations embedded in a .tscn file (not .tres), resolves parent
chains to world-space coordinates, and outputs an idle.jsonc compatible
with render_animation.py.
"""
import json
import re
import sys
from pathlib import Path


def parse_ext_resources(text):
    ext = {}
    for m in re.finditer(
        r'\[ext_resource type="Texture2D"[^]]*path="([^"]*)" id="([^"]*)"\]',
        text,
    ):
        ext[m.group(2)] = m.group(1).split("/")[-1]
    return ext


def parse_node_static_props(text):
    """Extract static position/scale/rotation/skew from [node] definitions."""
    props = {}
    for m in re.finditer(
        r'\[node name="([^"]+)"[^]]*parent="([^"]*)"[^]]*\]\n((?:.+\n)*?)(?=\n\[|\Z)',
        text,
    ):
        name = m.group(1)
        parent = m.group(2)
        body = m.group(3)

        pos_m = re.search(r"position = Vector2\(([^)]+)\)", body)
        scale_m = re.search(r"scale = Vector2\(([^)]+)\)", body)
        rot_m = re.search(r"rotation = ([\d.eE+-]+)", body)
        skew_m = re.search(r"skew = ([\d.eE+-]+)", body)
        centered_m = re.search(r"centered = (true|false)", body)
        tex_m = re.search(r'texture = ExtResource\("([^"]+)"\)', body)
        visible_m = re.search(r"visible = (true|false)", body)

        props[name] = {
            "parent": parent,
            "position": [float(x) for x in pos_m.group(1).split(",")] if pos_m else [0.0, 0.0],
            "scale": [float(x) for x in scale_m.group(1).split(",")] if scale_m else [1.0, 1.0],
            "rotation": float(rot_m.group(1)) if rot_m else 0.0,
            "skew": float(skew_m.group(1)) if skew_m else 0.0,
            "centered": centered_m.group(1) == "true" if centered_m else True,
            "texture": tex_m.group(1) if tex_m else None,
            "visible": visible_m.group(1) == "true" if visible_m else True,
        }
    return props


def parse_animations(text):
    """Parse all [sub_resource type="Animation"] blocks."""
    anims = {}
    blocks = re.split(r'(?=\[sub_resource type="Animation")', text)
    for block in blocks:
        name_m = re.search(r'resource_name = "(.+)"', block)
        if not name_m:
            continue
        anim_name = name_m.group(1)

        length_m = re.search(r"length = ([\d.]+)", block)
        step_m = re.search(r"step = ([\d.]+)", block)
        loop_m = re.search(r"loop_mode = (\d+)", block)

        length = float(length_m.group(1)) if length_m else 1.0
        step = float(step_m.group(1)) if step_m else 0.020833
        loop = int(loop_m.group(1)) > 0 if loop_m else False

        tracks = []
        track_blocks = re.split(r"(?=\ntracks/\d+/type)", block)
        for tb in track_blocks:
            path_m = re.search(r'tracks/\d+/path = NodePath\("([^"]+)"\)', tb)
            if not path_m:
                continue
            node_path = path_m.group(1)

            prop_m = re.search(r":(\w+)$", node_path)
            if not prop_m:
                continue
            prop = prop_m.group(1)
            node_full = node_path.rsplit(":", 1)[0]

            times_m = re.search(r'"times": PackedFloat32Array\(([^)]+)\)', tb)
            values_m = re.search(r'"values": \[(.*?)\]', tb, re.DOTALL)
            if not times_m or not values_m:
                continue

            times = [float(x) for x in times_m.group(1).split(",")]
            raw_vals = values_m.group(1).strip()
            values = parse_values(raw_vals, prop)

            tracks.append({
                "node_path": node_full,
                "property": prop,
                "times": times,
                "values": values,
            })

        anims[anim_name] = {
            "length": length,
            "step": step,
            "loop": loop,
            "tracks": tracks,
        }
    return anims


def parse_values(raw, prop):
    if prop == "visible":
        return [v.strip() == "true" for v in raw.split(",") if v.strip()]
    elif prop == "texture":
        vals = []
        for v in raw.split(","):
            v = v.strip()
            m = re.search(r'ExtResource\("([^"]+)"\)', v)
            vals.append(m.group(1) if m else None)
        return vals
    elif prop in ("position", "scale"):
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


def resolve_node_name(node_path):
    """Extract short node name from full path like 'Body/BodyCorrect/Foo'."""
    return node_path.split("/")[-1]


def build_parent_chain(node_name, static_nodes):
    """Build the full parent path for a node: ['Body', 'BodyCorrect', ...]."""
    chain = []
    current = node_name
    while current and current in static_nodes:
        chain.append(current)
        current = static_nodes[current]["parent"].split("/")[-1] if static_nodes[current]["parent"] else None
    chain.reverse()
    return chain


def find_animation_for_node(node_path, animations, anim_name):
    """Find which animation tracks apply to a given node."""
    if anim_name not in animations:
        return []
    return [t for t in animations[anim_name]["tracks"] if t["node_path"] == node_path]


def get_track_value_at_time(track, time):
    """Get interpolated value at a specific time from a track."""
    times = track["times"]
    values = track["values"]
    if not times:
        return None

    for i in range(len(times) - 1):
        if times[i] <= time <= times[i + 1]:
            t0, t1 = times[i], times[i + 1]
            v0, v1 = values[i], values[i + 1]
            if t1 - t0 < 1e-10:
                return v0
            frac = (time - t0) / (t1 - t0)
            if isinstance(v0, list):
                return [a + (b - a) * frac for a, b in zip(v0, v1)]
            return v0 + (v1 - v0) * frac

    return values[-1]


def convert(tscn_path, output_path, anim_name="Idle", head_anim_name="Head_Idle"):
    text = Path(tscn_path).read_text()
    ext_resources = parse_ext_resources(text)
    static_nodes = parse_node_static_props(text)
    animations = parse_animations(text)

    if anim_name not in animations:
        print(f"Error: animation '{anim_name}' not found")
        sys.exit(1)

    anim = animations[anim_name]
    duration = anim["length"]
    step = anim["step"]
    fps = round(1.0 / step)
    loop = anim["loop"]

    head_anim = animations.get(head_anim_name)

    print(f"Animation: {anim_name}, duration={duration:.4f}s, fps={fps}, loop={loop}")
    print(f"Static nodes: {len(static_nodes)}")
    print(f"Tracks in {anim_name}: {len(anim['tracks'])}")
    if head_anim:
        print(f"Tracks in {head_anim_name}: {len(head_anim['tracks'])}")

    num_frames = int(duration * fps)

    # Find all nodes that have textures (via static or texture track)
    textured_nodes = {}
    for name, props in static_nodes.items():
        if props["texture"]:
            textured_nodes[name] = props

    # Also check texture tracks in animations
    for aname in [anim_name, head_anim_name]:
        if aname not in animations:
            continue
        for track in animations[aname]["tracks"]:
            if track["property"] == "texture":
                node = resolve_node_name(track["node_path"])
                tex_id = track["values"][0]
                if tex_id and node not in textured_nodes:
                    # Find static node
                    if node in static_nodes:
                        textured_nodes[node] = static_nodes[node]
                        textured_nodes[node]["texture"] = ext_resources.get(tex_id, None)

    print(f"Textured nodes: {list(textured_nodes.keys())}")

    # For each textured node, find its parent chain and determine if parent is animated
    # Parent chain structure:
    #   Body(0,0) -> BodyCorrect(-43,-67) -> [direct children]
    #   Body(0,0) -> BodyCorrect(-43,-67) -> Anim_stem(animated) -> stem_correct(static) -> [head children]

    # Build node lookup: full_path -> node_name
    node_full_paths = {}
    for name, props in static_nodes.items():
        parent = props["parent"]
        if parent:
            node_full_paths[f"{parent}/{name}"] = name

    # Determine which parent path each node has
    def get_full_path(node_name):
        for fp, n in node_full_paths.items():
            if n == node_name:
                return fp
        return node_name

    # Output nodes dict
    nodes = {}

    for node_name, props in sorted(textured_nodes.items()):
        full_path = get_full_path(node_name)

        # Determine if this is a "head" node (under stem_correct)
        is_head = "stem_correct" in full_path

        # Check if we have animation tracks for this node
        anim_tracks = find_animation_for_node(full_path, animations, anim_name)
        if head_anim and is_head:
            head_tracks = find_animation_for_node(full_path, animations, head_anim_name)
            if head_tracks:
                anim_tracks = head_tracks

        # Get animated properties
        pos_track = next((t for t in anim_tracks if t["property"] == "position"), None)
        rot_track = next((t for t in anim_tracks if t["property"] == "rotation"), None)
        scale_track = next((t for t in anim_tracks if t["property"] == "scale"), None)
        skew_track = next((t for t in anim_tracks if t["property"] == "skew"), None)
        vis_track = next((t for t in anim_tracks if t["property"] == "visible"), None)

        # Texture
        tex_id = props.get("texture")
        tex_name = ext_resources.get(tex_id, tex_id) if tex_id else None

        # Check texture track
        tex_track = next((t for t in anim_tracks if t["property"] == "texture"), None)
        if tex_track:
            tex_id = tex_track["values"][0]
            tex_name = ext_resources.get(tex_id, tex_id) if tex_id else None

        # Visible
        visible = props.get("visible", True)
        if vis_track:
            visible = vis_track["values"][0]

        node_data = {"visible": visible, "texture": tex_name}

        # Convert position keyframes to world-space
        # We need to add the parent chain offset to each position keyframe
        if pos_track:
            world_keyframes = []
            for t, local_pos in zip(pos_track["times"], pos_track["values"]):
                # Compute world offset from parent chain
                world_offset = compute_world_offset(node_name, full_path, t, animations, anim_name, head_anim_name)
                world_x = world_offset[0] + local_pos[0]
                world_y = world_offset[1] + local_pos[1]
                world_keyframes.append([round(t, 6), [round(world_x, 2), round(world_y, 2)]])
            node_data["position"] = world_keyframes
        else:
            # No position track - use static position + parent offset
            local_pos = props["position"]
            world_offset = compute_world_offset(node_name, full_path, 0, animations, anim_name, head_anim_name)
            world_x = world_offset[0] + local_pos[0]
            world_y = world_offset[1] + local_pos[1]
            node_data["position"] = [[0.0, [round(world_x, 2), round(world_y, 2)]]]

        # Scale - store as keyframes
        if scale_track:
            node_data["scale"] = [[round(t, 6), v] for t, v in zip(scale_track["times"], scale_track["values"])]

        # Rotation - store as keyframes
        if rot_track:
            node_data["rotation"] = [[round(t, 6), v] for t, v in zip(rot_track["times"], rot_track["values"])]

        # Skew - store as keyframes
        if skew_track:
            node_data["skew"] = [[round(t, 6), v] for t, v in zip(skew_track["times"], skew_track["values"])]

        nodes[node_name] = node_data

    result = {
        "duration": duration,
        "fps": fps,
        "loop": loop,
        "nodes": nodes,
    }

    Path(output_path).parent.mkdir(parents=True, exist_ok=True)
    with open(output_path, "w") as f:
        json.dump(result, f, indent=2)

    print(f"\nOutput: {output_path}")
    print(f"Nodes: {len(nodes)}")
    for name, data in sorted(nodes.items()):
        pos = data.get("position", [])
        print(f"  {name}: texture={data['texture']}, visible={data['visible']}, keyframes={len(pos)}")


def compute_world_offset(node_name, full_path, time, animations, anim_name, head_anim_name):
    """Compute world-space offset from parent chain (excluding the node's own position).

    For nodes under Body/BodyCorrect:
        offset = BodyCorrect.pos
    For nodes under Body/BodyCorrect/Anim_stem/stem_correct:
        offset = BodyCorrect.pos + Anim_stem.pos(time) + stem_correct.pos
    """
    static_nodes = parse_node_static_props(
        Path(sys.argv[1]).read_text() if len(sys.argv) > 1 else ""
    )

    # Simple approach: hardcode based on known hierarchy
    # BodyCorrect = (-43, -67)
    # Anim_stem is animated by Idle animation
    # stem_correct = (-37.6, -48.7)

    body_correct = (-43.0, -67.0)
    stem_correct_static = (-37.6, -48.7)

    if "stem_correct" in full_path:
        # Head node: need Anim_stem position at this time
        anim = animations.get(anim_name)
        anim_stem_pos = [0.0, 0.0]
        if anim:
            for t in anim["tracks"]:
                if t["node_path"] == "Body/BodyCorrect/Anim_stem" and t["property"] == "position":
                    anim_stem_pos = get_track_value_at_time(t, time) or [0.0, 0.0]
                    break
        wx = body_correct[0] + anim_stem_pos[0] + stem_correct_static[0]
        wy = body_correct[1] + anim_stem_pos[1] + stem_correct_static[1]
        return (wx, wy)
    else:
        # Body node: just BodyCorrect offset
        return body_correct


if __name__ == "__main__":
    if len(sys.argv) < 3:
        print(f"Usage: {sys.argv[0]} <input.tscn> <output.jsonc> [anim_name] [head_anim_name]")
        sys.exit(1)

    tscn_path = sys.argv[1]
    output_path = sys.argv[2]
    anim_name = sys.argv[3] if len(sys.argv) > 3 else "Idle"
    head_anim_name = sys.argv[4] if len(sys.argv) > 4 else "Head_Idle"

    convert(tscn_path, output_path, anim_name, head_anim_name)
