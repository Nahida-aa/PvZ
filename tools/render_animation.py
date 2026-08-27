#!/usr/bin/env python3
"""Pre-render animation frames matching Godot real renderer (render_godot_frames.gd).

Reads a scene JSONC file (with instance chain) + animation JSONC (from tres_to_jsonc.py).
Resolves the instance inheritance chain recursively and merges nodes.
"""
import json
import math
import re
import subprocess
import sys
from pathlib import Path

GODOT_DIR = Path("/home/aa/repos/game_ls/learn_ls/PVZ-Godot_dream_20260406_v1.2.0")
PROJECT_ROOT = Path(__file__).resolve().parent.parent


def lerp(a, b, t):
    if isinstance(a, list):
        return [lerp(ai, bi, t) for ai, bi in zip(a, b)]
    return a + (b - a) * t


def interpolate_keyframes(keyframes, time):
    if not keyframes:
        return None
    if len(keyframes) == 1:
        return keyframes[0][1]
    for i in range(len(keyframes) - 1):
        t0, v0 = keyframes[i]
        t1, v1 = keyframes[i + 1]
        if t0 <= time <= t1:
            if t1 - t0 < 1e-10:
                return v0
            return lerp(v0, v1, (time - t0) / (t1 - t0))
    return keyframes[-1][1]


def closest_keyframe(keyframes, time):
    best = 0
    best_dist = float("inf")
    for i, (t, _) in enumerate(keyframes):
        d = abs(t - time)
        if d < best_dist:
            best_dist = d
            best = i
    return best


def make_affine(rot, skew, sx, sy):
    cos_r = math.cos(rot)
    sin_r = math.sin(rot)
    cos_r_sk = math.cos(rot + skew)
    sin_r_sk = math.sin(rot + skew)
    a = cos_r * sx
    b = -sin_r_sk * sy
    c = sin_r * sx
    d = cos_r_sk * sy
    return a, b, c, d


def top_left_to_center(tl_x, tl_y, tex_w, tex_h, rot, skew, sx, sy):
    a, b, c, d = make_affine(rot, skew, sx, sy)
    hw, hh = tex_w / 2, tex_h / 2
    return tl_x + a * hw + b * hh, tl_y + c * hw + d * hh


def load_jsonc(path):
    text = Path(path).read_text()
    lines = text.split("\n")
    cleaned = []
    for line in lines:
        in_string = False
        escape = False
        comment_pos = len(line)
        for i, ch in enumerate(line):
            if escape:
                escape = False
                continue
            if ch == "\\":
                escape = True
                continue
            if ch == '"':
                in_string = not in_string
            elif ch == "/" and not in_string and i + 1 < len(line) and line[i + 1] == "/":
                comment_pos = i
                break
        cleaned.append(line[:comment_pos])
    text = "\n".join(cleaned)
    text = re.sub(r',\s*([}\]])', r'\1', text)
    return json.loads(text)


def load_scene_tree(scene_jsonc_path):
    """Recursively load scene JSONC via instance chain, merging nodes.

    Returns merged { source, nodes } with the full inherited node tree.
    Child nodes override parent nodes with the same name; new nodes are added.
    """
    data = load_jsonc(scene_jsonc_path)
    instance = data.get("instance")

    if not instance:
        # Base case: no parent, return as-is
        return {"source": data["source"], "nodes": dict(data["nodes"])}

    # Resolve instance path: "res://scenes/character/character_000_base.jsonc"
    # → PROJECT_ROOT / "scenes/character/character_000_base.jsonc"
    rel = instance[len("res://"):]
    parent_path = PROJECT_ROOT / rel
    if not parent_path.exists():
        raise FileNotFoundError(f"Instance target not found: {parent_path} (from {instance})")

    # Recursively load parent
    parent = load_scene_tree(parent_path)

    # Merge: parent nodes first, child overrides
    merged_nodes = dict(parent["nodes"])
    for name, props in data["nodes"].items():
        if name in merged_nodes:
            # Override: merge properties (child wins)
            merged = dict(merged_nodes[name])
            merged.update(props)
            merged_nodes[name] = merged
        else:
            merged_nodes[name] = props

    return {"source": data["source"], "nodes": merged_nodes}


def resolve_texture(texture_name, parts_dir):
    if texture_name.startswith("res://"):
        rel = texture_name[len("res://"):]
        godot_path = GODOT_DIR / rel
        if godot_path.exists():
            return godot_path
        bevy_path = PROJECT_ROOT / rel
        if bevy_path.exists():
            return bevy_path
    else:
        parts_path = parts_dir / texture_name
        if parts_path.exists():
            return parts_path
    return None


def build_scene_config(render_jsonc_path, merged_scene):
    """Extract render config from render_frames.jsonc + merged scene tree."""
    render = load_jsonc(render_jsonc_path)
    nodes = merged_scene["nodes"]

    vp_size = render["nodes"]["SubViewport"]["size"]
    root_pos = render["nodes"]["RootNode"]["position"]
    body_correct = nodes.get("BodyCorrect", {}).get("position", [0, 0])
    time_scale = 1.2

    shadow_node = nodes.get("Shadow", {})
    shadow_texture = shadow_node.get("texture")

    # Build child_index map for BodyCorrect children (z-order)
    child_indices = {}
    for name, props in nodes.items():
        parent = props.get("parent", "")
        if parent == "Body/BodyCorrect" and "index" in props:
            child_indices[name] = props["index"]

    return {
        "viewport_size": vp_size,
        "root_position": root_pos,
        "body_correct": body_correct,
        "time_scale": time_scale,
        "shadow_texture": shadow_texture,
        "child_indices": child_indices,
    }


def generate_frame_rig(jsonc_data, frame_time, parts_dir, scene, shadow_path=None, merged_scene=None):
    nodes = jsonc_data["nodes"]
    parts = []
    child_indices = scene["child_indices"]
    root_pos = scene["root_position"]
    body_correct = scene["body_correct"]
    scene_nodes = merged_scene["nodes"] if merged_scene else {}

    if shadow_path and shadow_path.exists():
        from PIL import Image as PILImage
        cx = root_pos[0]
        cy = root_pos[1]
        parts.append({
            "image": shadow_path.name,
            "x": cx,
            "y": cy,
            "z": 0.0,
            "scale_x": 1.0,
            "scale_y": 1.0,
            "rotation": 0,
            "skew": 0,
        })

    for node_name, props in nodes.items():
        props = nodes[node_name]
        scene_props = scene_nodes.get(node_name, {})

        # visible: 动画有 visible_track 用轨道值，否则用 scene 的 visible（默认 true）
        visible_track = props.get("visible_track")
        if visible_track is not None:
            idx = closest_keyframe(visible_track, frame_time)
            visible = visible_track[idx][1]
        else:
            visible = scene_props.get("visible", True)
        if not visible:
            continue

        texture_track = props.get("texture_track")
        if texture_track:
            idx = closest_keyframe(texture_track, frame_time)
            texture = texture_track[idx][1]
        else:
            texture = props.get("texture") or scene_props.get("texture")
        if not texture:
            continue

        pos = interpolate_keyframes(props.get("position", []), frame_time) or [0, 0]
        rot = interpolate_keyframes(props.get("rotation", []), frame_time) or 0
        scale = interpolate_keyframes(props.get("scale", []), frame_time) or [1, 1]
        skew = interpolate_keyframes(props.get("skew", []), frame_time) or 0

        # centered: 默认 true (Sprite2D 默认)，但 scene 里可能写了 false
        centered = scene_props.get("centered", True)

        tl_x = root_pos[0] + body_correct[0] + pos[0]
        tl_y = root_pos[1] + body_correct[1] + pos[1]

        from PIL import Image
        tex_path = resolve_texture(texture, parts_dir)
        if not tex_path:
            continue
        img = Image.open(tex_path)
        tw, th = img.size

        if centered:
            # 中心在 (tl_x, tl_y)
            cx, cy = tl_x, tl_y
        else:
            cx, cy = top_left_to_center(tl_x, tl_y, tw, th, rot, skew, scale[0], scale[1])

        z = child_indices.get(node_name, 0) + 1

        parts.append({
            "image": tex_path.name,
            "x": cx,
            "y": cy,
            "z": float(z),
            "scale_x": scale[0],
            "scale_y": scale[1],
            "rotation": rot,
            "skew": skew,
        })

    return {"scale": 1.0, "parts": parts}


def main():
    if len(sys.argv) != 4:
        print(f"Usage: {sys.argv[0]} <scene.jsonc> <animation.jsonc> <output_dir>")
        sys.exit(1)

    scene_path = Path(sys.argv[1])
    anim_path = Path(sys.argv[2])
    output_dir = Path(sys.argv[3])
    output_dir.mkdir(parents=True, exist_ok=True)

    # Load animation data
    data = load_jsonc(anim_path)
    duration = data["duration"]
    fps = data["fps"]
    num_frames = int(duration * fps)
    parts_dir = anim_path.parent.parent / "parts"

    # Load scene via instance chain
    print(f"Loading scene tree: {scene_path}")
    merged = load_scene_tree(scene_path)
    print(f"  Merged {len(merged['nodes'])} nodes from instance chain")

    # Build render config
    tools_dir = Path(__file__).resolve().parent
    scene = build_scene_config(tools_dir / "render_frames.jsonc", merged)

    viewport_size = scene["viewport_size"]
    root_pos = scene["root_position"]
    body_correct = scene["body_correct"]
    time_scale = scene["time_scale"]

    print(f"Duration: {duration}s, FPS: {fps}, Frames: {num_frames}, TimeScale: {time_scale}")
    print(f"Canvas: {viewport_size[0]}x{viewport_size[1]}, Root: {root_pos}, BodyCorrect: {body_correct}")

    shadow_tex = scene["shadow_texture"]
    shadow_path = resolve_texture(shadow_tex, parts_dir) if shadow_tex else None
    if shadow_path:
        # Shadow texture may live outside parts_dir; copy it in so plant_composer can find it
        import shutil
        dest = parts_dir / shadow_path.name
        if not dest.exists():
            shutil.copy(shadow_path, dest)
        shadow_path = dest
        print(f"Shadow: {shadow_path}")

    for frame_idx in range(num_frames):
        real_time = frame_idx / fps
        frame_time = (real_time * time_scale) % duration
        rig = generate_frame_rig(data, frame_time, parts_dir, scene, shadow_path if shadow_path else None, merged)

        rig_path = output_dir / f"_rig_{frame_idx:03d}.jsonc"
        with open(rig_path, "w") as f:
            json.dump(rig, f, indent=2)

        frame_path = output_dir / f"frame_{frame_idx:03d}.png"
        result = subprocess.run(
            [
                "cargo", "run", "-p", "plant_composer",
                "--", "--input", str(parts_dir.parent),
                "--rig", str(rig_path),
                "--output", str(frame_path),
                "--width", str(viewport_size[0]),
                "--height", str(viewport_size[1]),
            ],
            capture_output=True,
            text=True,
        )
        if result.returncode != 0:
            print(f"Frame {frame_idx} failed: {result.stderr}")
            continue

        rig_path.unlink()
        print(f"Frame {frame_idx:3d}/{num_frames} (t={frame_time:.4f}s) -> {frame_path.name}")

    print(f"\nDone! {num_frames} frames in {output_dir}")


if __name__ == "__main__":
    main()
