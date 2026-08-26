#!/usr/bin/env python3
"""Pre-render animation frames from idle.jsonc using plant_composer."""
import json
import subprocess
import sys
from pathlib import Path


def lerp(a, b, t):
    """Linear interpolation."""
    if isinstance(a, list):
        return [lerp(ai, bi, t) for ai, bi in zip(a, b)]
    return a + (b - a) * t


def interpolate_keyframes(keyframes, time):
    """Interpolate keyframes at a given time."""
    if not keyframes:
        return None
    if len(keyframes) == 1:
        return keyframes[0][1]

    # Find surrounding keyframes
    for i in range(len(keyframes) - 1):
        t0, v0 = keyframes[i]
        t1, v1 = keyframes[i + 1]
        if t0 <= time <= t1:
            if t1 - t0 < 1e-10:
                return v0
            frac = (time - t0) / (t1 - t0)
            return lerp(v0, v1, frac)

    # Return last value
    return keyframes[-1][1]


def generate_frame_rig(jsonc_data, frame_time, parts_dir):
    """Generate a rig.jsonc for a specific frame time."""
    nodes = jsonc_data["nodes"]

    parts = []
    z_order = 0

    # Process nodes in a consistent order
    for node_name in sorted(nodes.keys()):
        props = nodes[node_name]

        visible = props.get("visible", True)
        if not visible:
            continue

        texture = props.get("texture")
        if not texture:
            continue

        # Get transform properties
        pos = interpolate_keyframes(props.get("position", []), frame_time) or [0, 0]
        rot = interpolate_keyframes(props.get("rotation", []), frame_time) or 0
        scale = interpolate_keyframes(props.get("scale", []), frame_time) or [1, 1]
        skew = interpolate_keyframes(props.get("skew", []), frame_time) or 0

        part = {
            "image": texture,
            "x": pos[0],
            "y": pos[1],
            "z": float(z_order),
            "scale_x": scale[0],
            "scale_y": scale[1],
            "rotation": rot,
            "skew": skew,
        }
        parts.append(part)
        z_order += 1

    return {"scale": 1.0, "parts": parts}


def compute_canvas_size(data, parts_dir):
    """Compute the bounding box across all frames for a fixed canvas size."""
    fps = data["fps"]
    duration = data["duration"]
    num_frames = int(duration * fps)

    min_x, min_y = float("inf"), float("inf")
    max_x, max_y = float("-inf"), float("-inf")

    for fi in range(num_frames):
        t = fi / fps
        nodes = data["nodes"]
        for node_name, props in nodes.items():
            if not props.get("visible", True):
                continue
            texture = props.get("texture")
            if not texture:
                continue

            pos = interpolate_keyframes(props.get("position", []), t) or [0, 0]
            scale = interpolate_keyframes(props.get("scale", []), t) or [1, 1]

            tex_path = parts_dir / texture
            if not tex_path.exists():
                continue
            from PIL import Image
            img = Image.open(tex_path)
            w, h = img.size
            sw, sh = w * scale[0], h * scale[1]
            x0 = pos[0] - sw / 2
            y0 = pos[1] - sh / 2
            x1 = pos[0] + sw / 2
            y1 = pos[1] + sh / 2
            min_x = min(min_x, x0)
            min_y = min(min_y, y0)
            max_x = max(max_x, x1)
            max_y = max(max_y, y1)

    return (min_x, min_y, max_x, max_y)


def main():
    if len(sys.argv) != 3:
        print(f"Usage: {sys.argv[0]} <idle.jsonc> <output_dir>")
        sys.exit(1)

    jsonc_path = Path(sys.argv[1])
    output_dir = Path(sys.argv[2])
    output_dir.mkdir(parents=True, exist_ok=True)

    # Load animation data
    with open(jsonc_path) as f:
        data = json.load(f)

    duration = data["duration"]
    fps = data["fps"]
    num_frames = int(duration * fps)

    # Find parts directory (sibling of animation/)
    parts_dir = jsonc_path.parent.parent / "parts"

    print(f"Duration: {duration:.6f}s, FPS: {fps}, Frames: {num_frames}")
    print(f"Parts dir: {parts_dir}")

    # Compute fixed canvas size across all frames
    bbox = compute_canvas_size(data, parts_dir)
    min_x, min_y, max_x, max_y = bbox
    import math
    canvas_w = math.ceil(max_x - min_x)
    canvas_h = math.ceil(max_y - min_y)
    offset_x = -min_x
    offset_y = -min_y
    print(f"Canvas: {canvas_w}x{canvas_h}, offset=({offset_x:.1f}, {offset_y:.1f})")

    for frame_idx in range(num_frames):
        frame_time = frame_idx / fps
        rig = generate_frame_rig(data, frame_time, parts_dir)

        # Apply offset to all parts so content fits in fixed canvas
        for part in rig["parts"]:
            part["x"] += offset_x
            part["y"] += offset_y

        # Write temporary rig.jsonc
        rig_path = output_dir / f"_rig_{frame_idx:03d}.jsonc"
        with open(rig_path, "w") as f:
            json.dump(rig, f, indent=2)

        # Run composer with fixed canvas size
        frame_path = output_dir / f"frame_{frame_idx:03d}.png"
        result = subprocess.run(
            [
                "cargo", "run", "-p", "plant_composer",
                "--", "--input", str(parts_dir.parent),
                "--rig", str(rig_path),
                "--output", str(frame_path),
                "--width", str(canvas_w),
                "--height", str(canvas_h),
            ],
            capture_output=True,
            text=True,
        )
        if result.returncode != 0:
            print(f"Frame {frame_idx} failed: {result.stderr}")
            continue

        # Remove temporary rig
        rig_path.unlink()

        print(f"Frame {frame_idx:3d}/{num_frames} (t={frame_time:.4f}s) -> {frame_path.name}")

    print(f"\nDone! {num_frames} frames in {output_dir}")


if __name__ == "__main__":
    main()
