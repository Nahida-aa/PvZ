#!/usr/bin/env python3
"""Pre-render animation frames matching Godot real renderer (render_godot_frames.gd).

The real Godot renderer loads the full PackedScene (e.g. plant_002_sun_flower.tscn)
which includes the node hierarchy: Plant → Body(0,0) → BodyCorrect(-43,-69) → Sprites.
So the tres positions are relative to BodyCorrect, and the actual canvas position is:
  canvas_pos = RootNode(100,100) + BodyCorrect(-43,-69) + tres_pos

Differences from Godot rendering that we accept:
- plant_composer does software rasterization vs Godot hardware GPU rendering
  (minor bilinear filtering / alpha blending differences)

Everything else must match exactly:
- 200x200 canvas, root at (100,100)
- centered=false (top-left positioning)
- BodyCorrect offset: (-43, -69)
- TimeScale: 1.2 (from AnimationTree, anim_time = real_time * 1.2, looped)
- Affine: R*Sk*S matching Godot Transform2D (skew uses sx)
- Z-order = tres track order (matches scene tree child index)
"""
import json
import math
import re
import subprocess
import sys
from pathlib import Path

VIEWPORT_SIZE = 200
ROOT_POSITION = (VIEWPORT_SIZE / 2.0, VIEWPORT_SIZE / 2.0)
BODY_CORRECT = (-43.0, -69.0)
TIME_SCALE = 1.2


def lerp(a, b, t):
    if isinstance(a, list):
        return [lerp(ai, bi, t) for ai, bi in zip(a, b)]
    return a + (b - a) * t


def interpolate_keyframes(keyframes, time):
    """Linear interpolation matching Godot _lerp_vec2 / _lerp_f32."""
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
    """Find index of closest keyframe to given time, matching Godot _closest()."""
    best = 0
    best_dist = float("inf")
    for i, (t, _) in enumerate(keyframes):
        d = abs(t - time)
        if d < best_dist:
            best_dist = d
            best = i
    return best


def make_affine(rot, skew, sx, sy):
    """Match Godot Transform2D: columns[1] += columns[0] * tan(skew).

    Godot source:
      columns[0] = (cos(r)*sx, sin(r)*sx)
      columns[1] = (-sin(r)*sy, cos(r)*sy)
      columns[1] += columns[0] * tan(skew)

    Result:
      a = cos(r)*sx
      b = cos(r)*sx*tan(k) - sin(r)*sy
      c = sin(r)*sx
      d = sin(r)*sx*tan(k) + cos(r)*sy
    """
    cos_r = math.cos(rot)
    sin_r = math.sin(rot)
    tan_k = math.tan(skew)
    a = cos_r * sx
    b = cos_r * sx * tan_k - sin_r * sy
    c = sin_r * sx
    d = sin_r * sx * tan_k + cos_r * sy
    return a, b, c, d


def top_left_to_center(tl_x, tl_y, tex_w, tex_h, rot, skew, sx, sy):
    """Convert Godot centered=false top-left position to plant_composer center position."""
    a, b, c, d = make_affine(rot, skew, sx, sy)
    hw, hh = tex_w / 2, tex_h / 2
    return tl_x + a * hw + b * hh, tl_y + c * hw + d * hh


def generate_frame_rig(jsonc_data, frame_time, parts_dir):
    """Generate a rig for one frame, matching Godot simplified renderer logic."""
    nodes = jsonc_data["nodes"]
    parts = []
    z_order = 0

    # Simplified renderer iterates _node_tracks in insertion order = tres track order.
    # In Godot, child 0 drawn first (behind), child N drawn last (on top).
    # plant_composer: lower z drawn first (behind), higher z drawn later (on top).
    for node_name, props in nodes.items():
        props = nodes[node_name]

        # Godot: uses _closest() to snap visible to nearest keyframe
        visible_track = props.get("visible_track")
        if visible_track is not None:
            idx = closest_keyframe(visible_track, frame_time)
            visible = visible_track[idx][1]
        else:
            visible = props.get("visible", True)
        if not visible:
            continue

        texture_track = props.get("texture_track")
        if texture_track:
            idx = closest_keyframe(texture_track, frame_time)
            texture = texture_track[idx][1]
        else:
            texture = props.get("texture")
        if not texture:
            continue

        pos = interpolate_keyframes(props.get("position", []), frame_time) or [0, 0]
        rot = interpolate_keyframes(props.get("rotation", []), frame_time) or 0
        scale = interpolate_keyframes(props.get("scale", []), frame_time) or [1, 1]
        skew = interpolate_keyframes(props.get("skew", []), frame_time) or 0

        # Godot: sprite.position = tres_value, centered=false
        # Canvas pos = RootNode + BodyCorrect + tres_pos
        tl_x = ROOT_POSITION[0] + BODY_CORRECT[0] + pos[0]
        tl_y = ROOT_POSITION[1] + BODY_CORRECT[1] + pos[1]

        from PIL import Image
        tex_path = parts_dir / texture
        if not tex_path.exists():
            continue
        img = Image.open(tex_path)
        tw, th = img.size

        cx, cy = top_left_to_center(tl_x, tl_y, tw, th, rot, skew, scale[0], scale[1])

        parts.append({
            "image": texture,
            "x": cx,
            "y": cy,
            "z": float(z_order),
            "scale_x": scale[0],
            "scale_y": scale[1],
            "rotation": rot,
            "skew": skew,
        })
        z_order += 1

    return {"scale": 1.0, "parts": parts}


def main():
    if len(sys.argv) != 3:
        print(f"Usage: {sys.argv[0]} <idle.jsonc> <output_dir>")
        sys.exit(1)

    jsonc_path = Path(sys.argv[1])
    output_dir = Path(sys.argv[2])
    output_dir.mkdir(parents=True, exist_ok=True)

    text = jsonc_path.read_text()
    text = re.sub(r'//.*', '', text)
    text = re.sub(r',\s*([}\]])', r'\1', text)
    data = json.loads(text)

    duration = data["duration"]
    fps = data["fps"]
    num_frames = int(duration * fps)
    parts_dir = jsonc_path.parent.parent / "parts"

    print(f"Duration: {duration}s, FPS: {fps}, Frames: {num_frames}, TimeScale: {TIME_SCALE}")
    print(f"Canvas: {VIEWPORT_SIZE}x{VIEWPORT_SIZE}, Root: {ROOT_POSITION}, BodyCorrect: {BODY_CORRECT}")

    for frame_idx in range(num_frames):
        real_time = frame_idx / fps
        frame_time = (real_time * TIME_SCALE) % duration
        rig = generate_frame_rig(data, frame_time, parts_dir)

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
                "--width", str(VIEWPORT_SIZE),
                "--height", str(VIEWPORT_SIZE),
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
