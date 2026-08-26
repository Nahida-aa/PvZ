#!/usr/bin/env python3
"""Pre-render animation frames from idle.jsonc using plant_composer."""
import json
import subprocess
import sys
import tempfile
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

    for frame_idx in range(num_frames):
        frame_time = frame_idx / fps
        rig = generate_frame_rig(data, frame_time, parts_dir)

        # Write temporary rig.jsonc
        rig_path = output_dir / f"_rig_{frame_idx:03d}.jsonc"
        with open(rig_path, "w") as f:
            json.dump(rig, f, indent=2)

        # Run composer
        frame_path = output_dir / f"frame_{frame_idx:03d}.png"
        result = subprocess.run(
            [
                "cargo", "run", "-p", "plant_composer",
                "--", "--input", str(parts_dir.parent),
                "--rig", str(rig_path),
                "--output", str(frame_path),
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
