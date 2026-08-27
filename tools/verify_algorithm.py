#!/usr/bin/env python3
"""
逐像素验证渲染算法是否与 Godot 一致。

用法: python3 verify_algorithm.py <frame_index>
取 frame_XXX 用 Godot 渲染的参考图，与我们的算法逐像素比对，
输出差异的统计分布和具体位置。
"""
import sys, json, math, re, os
from PIL import Image

# ── 读取场景树 + 动画 ─────────────────────────────────────────────
def load_jsonc(path):
    with open(path) as f:
        text = f.read()
    result = []
    i = 0
    in_string = False
    while i < len(text):
        ch = text[i]
        if in_string:
            result.append(ch)
            if ch == '\\' and i + 1 < len(text):
                result.append(text[i + 1])
                i += 2
                continue
            if ch == '"':
                in_string = False
            i += 1
            continue
        if ch == '"':
            in_string = True
            result.append(ch)
            i += 1
            continue
        if ch == '/' and i + 1 < len(text) and text[i + 1] == '/':
            while i < len(text) and text[i] != '\n':
                i += 1
            continue
        result.append(ch)
        i += 1
    cleaned = ''.join(result)
    cleaned = re.sub(r',\s*([}\]])', r'\1', cleaned)
    return json.loads(cleaned)

def resolve_instances(scene_path):
    scene = load_jsonc(scene_path)
    if 'instance' in scene:
        inst = scene['instance']
        if inst.startswith("res://"):
            inst = inst[len("res://"):]
        base = resolve_instances(inst)
        if 'nodes' in base:
            if 'nodes' not in scene:
                scene['nodes'] = {}
            for k, v in base['nodes'].items():
                if k not in scene['nodes']:
                    scene['nodes'][k] = v
        del scene['instance']
    return scene

def sample_track(track, t):
    if isinstance(track, dict) and 'frames' in track:
        frames = track['frames']
    elif isinstance(track, list):
        frames = track
    else:
        return track
    if len(frames) == 1:
        return frames[0][1]
    if t <= frames[0][0]:
        return frames[0][1]
    if t >= frames[-1][0]:
        return frames[-1][1]
    for i in range(len(frames) - 1):
        t0, v0 = frames[i]
        t1, v1 = frames[i + 1]
        if t0 <= t <= t1:
            dt = t1 - t0
            if dt < 1e-12:
                return v0
            alpha = (t - t0) / dt
            if isinstance(v0, list) and isinstance(v0[0], (int, float)):
                return [a + (b - a) * alpha for a, b in zip(v0, v1)]
            else:
                return v0 + (v1 - v0) * alpha
    return frames[-1][1]

# ── Bilinear 采样 ──────────────────────────────────────────────────
def sample_bilinear(img_data, w, h, u, v):
    """GPU LINEAR filter: pixel (i,j) 覆盖 [i-0.5, i+0.5)，uv 是采样坐标"""
    # GPU 采样: floor(u - 0.5) 和 frac(u - 0.5)
    fu = u - 0.5
    fv = v - 0.5
    u0 = int(math.floor(fu))
    v0 = int(math.floor(fv))
    # CLAMP_TO_EDGE
    u0 = max(0, min(u0, w - 1))
    v0 = max(0, min(v0, h - 1))
    u1 = min(u0 + 1, w - 1)
    v1 = min(v0 + 1, h - 1)
    fu_f = max(0.0, min(fu - math.floor(fu), 1.0))
    fv_f = max(0.0, min(fv - math.floor(fv), 1.0))
    p00 = img_data[v0 * w + u0]
    p10 = img_data[v0 * w + u1]
    p01 = img_data[v1 * w + u0]
    p11 = img_data[v1 * w + u1]
    r = (1 - fu_f) * (1 - fv_f) * p00[0] + fu_f * (1 - fv_f) * p10[0] + (1 - fu_f) * fv_f * p01[0] + fu_f * fv_f * p11[0]
    g = (1 - fu_f) * (1 - fv_f) * p00[1] + fu_f * (1 - fv_f) * p10[1] + (1 - fu_f) * fv_f * p01[1] + fu_f * fv_f * p11[1]
    b = (1 - fu_f) * (1 - fv_f) * p00[2] + fu_f * (1 - fv_f) * p10[2] + (1 - fu_f) * fv_f * p01[2] + fu_f * fv_f * p11[2]
    a = (1 - fu_f) * (1 - fv_f) * p00[3] + fu_f * (1 - fv_f) * p10[3] + (1 - fu_f) * fv_f * p01[3] + fu_f * fv_f * p11[3]
    return (r, g, b, a)

def alpha_composite(bg, fg):
    """Standard SRC_OVER on [0..255]"""
    ab = bg[3] / 255.0
    af = fg[3] / 255.0
    ao = af + ab * (1.0 - af)
    if ao < 1e-9:
        return (0, 0, 0, 0)
    r = (fg[0] * af + bg[0] * ab * (1.0 - af)) / ao
    g = (fg[1] * af + bg[1] * ab * (1.0 - af)) / ao
    b = (fg[2] * af + bg[2] * ab * (1.0 - af)) / ao
    return (r, g, b, ao * 255)

# ── 仿射矩阵 ──────────────────────────────────────────────────────
def make_affine(pos, rot, scale, skew, offset_x, offset_y, sprite_w, sprite_h):
    sx, sy = scale
    cos_r = math.cos(rot)
    sin_r = math.sin(rot)
    cos_rsk = math.cos(rot + skew)
    sin_rsk = math.sin(rot + skew)
    a = cos_r * sx
    b = -sin_rsk * sy
    c = sin_r * sx
    d = cos_rsk * sy
    hw = sprite_w * 0.5
    hh = sprite_h * 0.5
    tx = pos[0] - a * hw - b * hh + offset_x
    ty = pos[1] - c * hw - d * hh + offset_y
    return a, b, c, d, tx, ty

# ── 主程序 ──────────────────────────────────────────────────────────
def main():
    frame_idx = int(sys.argv[1]) if len(sys.argv) > 1 else 12
    fps = 12
    time_scale = 1.2
    anim_length = 2.0
    canvas_w, canvas_h = 200, 200
    root_x, root_y = 100.0, 100.0

    scene = resolve_instances('scenes/character/plant/plant_002_sun_flower.jsonc')
    anim = load_jsonc('assets/plants/sun_flower/animation/idle.jsonc')
    body_correct = {'position': [-43.0, -69.0]}

    real_time = frame_idx * (1.0 / fps)
    anim_time = (real_time * time_scale) % anim_length
    print(f"Frame {frame_idx}: real_time={real_time:.4f}, anim_time={anim_time:.4f}")

    # 合并所有 Sprite2D 节点 — scene 是基础，animation 覆盖属性
    # 绘制规则: 1) 在 idle 动画里的节点  2) Shadow（基础场景恒定存在）
    # Zzz 等睡眠符号属于独立 sleep 动画，idle 时不显示
    nodes_data = scene.get('nodes', {})
    anim_nodes = set(anim.get('nodes', {}).keys())
    transforms = []
    for node_name, props in nodes_data.items():
        if props.get('type') != 'Sprite2D':
            continue
        # 跳过不在 idle 动画里且不是 Shadow 的节点（如 Zzz 睡眠符号）
        if node_name not in anim_nodes and node_name != 'Shadow':
            continue

        # 基础值来自 scene
        base_pos = props.get('position', [0, 0])
        base_rot = props.get('rotation', 0)
        base_scale = props.get('scale', [1, 1])
        base_skew = props.get('skew', 0)
        base_visible = props.get('visible', True)

        # animation 覆盖
        if node_name in anim.get('nodes', {}):
            anim_node = anim['nodes'][node_name]
            if 'visible' in anim_node:
                base_visible = sample_track(anim_node['visible'], anim_time)
            if 'position' in anim_node:
                base_pos = sample_track(anim_node['position'], anim_time)
            if 'rotation' in anim_node:
                base_rot = sample_track(anim_node['rotation'], anim_time)
            if 'scale' in anim_node:
                base_scale = sample_track(anim_node['scale'], anim_time)
            if 'skew' in anim_node:
                base_skew = sample_track(anim_node['skew'], anim_time)

        if not base_visible:
            continue

        # 计算画布位置：RootNode.position + 节点 transform
        # Shadow 的 parent="." → 直接在 RootNode 下，没有 BodyCorrect 偏移！
        parent = props.get('parent', '.')
        if parent.endswith('BodyCorrect'):
            px = root_x + body_correct['position'][0] + base_pos[0]
            py = root_y + body_correct['position'][1] + base_pos[1]
        else:
            px = root_x + base_pos[0]
            py = root_y + base_pos[1]

        # centered: 默认 true (Sprite2D 默认)，但 JSONC/tscn 里可能写了 false
        centered = props.get('centered', True)

        transforms.append({
            'name': node_name,
            'pos': (px, py),
            'rot': base_rot,
            'scale': base_scale,
            'skew': base_skew,
            'texture': props.get('texture', ''),
            'index': props.get('index', -1),
            'centered': centered,
        })

    # 按 z-order (child_index) 排序 — Godot child 0 先画
    transforms.sort(key=lambda n: n['index'])

    # 读取参考图 (Godot 渲染)
    ref_path = f'tmp/godot_sunflower/frame_{frame_idx:03d}.png'
    if not os.path.exists(ref_path):
        print(f"参考图不存在: {ref_path}")
        sys.exit(1)
    ref_img = Image.open(ref_path).convert('RGBA')
    ref_pixels = list(ref_img.getdata())

    # 用我们的算法渲染
    canvas = [(0.0, 0.0, 0.0, 0.0)] * (canvas_w * canvas_h)

    for node in transforms:
        node_name = node['name']
        # 优先使用 scene 里的 texture 字段
        tex = node.get('texture', '')
        if tex:
            if tex.startswith("res://"):
                tex = tex[len("res://"):]
            sprite_path = tex
        else:
            sprite_path = os.path.join('assets', f'{node_name}.png')
        if not os.path.exists(sprite_path):
            sprite_path = f'tmp/bevy_render/parts/{node_name}.png'
        if not os.path.exists(sprite_path):
            sprite_path = f'assets/plants/sun_flower/parts/{node_name}.png'
        if not os.path.exists(sprite_path):
            print(f"  跳过 {node_name}: 纹理不存在")
            continue

        sprite_img = Image.open(sprite_path).convert('RGBA')
        sprite_w, sprite_h = sprite_img.size
        sprite_data = list(sprite_img.getdata())

        # Godot Sprite2D: 如果 centered=false，纹理左上角 (0,0) 映射到 pos
        # 如果 centered=true，纹理中心 (w/2, h/2) 映射到 pos
        sx, sy = node['scale']
        cos_r = math.cos(node['rot'])
        sin_r = math.sin(node['rot'])
        cos_rsk = math.cos(node['rot'] + node['skew'])
        sin_rsk = math.sin(node['rot'] + node['skew'])
        a = cos_r * sx
        b = -sin_rsk * sy
        c = sin_r * sx
        d = cos_rsk * sy

        hw = sprite_w * 0.5
        hh = sprite_h * 0.5
        if node['centered']:
            # 中心在 pos：top-left = pos - (a*hw + b*hh, c*hw + d*hh)
            tx = node['pos'][0] - a * hw - b * hh
            ty = node['pos'][1] - c * hw - d * hh
        else:
            tx = node['pos'][0]
            ty = node['pos'][1]

        # 逆矩阵
        det = a * d - b * c
        if abs(det) < 1e-9:
            continue
        inv_a = d / det
        inv_b = -b / det
        inv_c = -c / det
        inv_d = a / det

        inv_tx = -(inv_a * tx + inv_b * ty)
        inv_ty = -(inv_c * tx + inv_d * ty)

        # 逐像素采样 + 混合
        # GPU 采样: fragment 在像素中心 (x+0.5, y+0.5)，不是像素角落！
        for cy in range(canvas_h):
            for cx in range(canvas_w):
                su = inv_a * (cx + 0.5) + inv_b * (cy + 0.5) + inv_tx
                sv = inv_c * (cx + 0.5) + inv_d * (cy + 0.5) + inv_ty
                if su < -0.5 or sv < -0.5 or su > sprite_w - 0.5 or sv > sprite_h - 0.5:
                    continue
                sampled = sample_bilinear(sprite_data, sprite_w, sprite_h, su, sv)
                if sampled[3] < 1.0:
                    continue
                bg = canvas[cy * canvas_w + cx]
                canvas[cy * canvas_w + cx] = alpha_composite(bg, sampled)

    # 转成 RGBA u8
    our_pixels = []
    for p in canvas:
        r = max(0, min(255, int(p[0] + 0.5)))
        g = max(0, min(255, int(p[1] + 0.5)))
        b = max(0, min(255, int(p[2] + 0.5)))
        a = max(0, min(255, int(p[3] + 0.5)))
        our_pixels.append((r, g, b, a))

    # 保存我们的渲染结果
    out_img = Image.new('RGBA', (canvas_w, canvas_h))
    out_img.putdata(our_pixels)
    out_path = f'tmp/verify/frame_{frame_idx:03d}_our.png'
    os.makedirs('tmp/verify', exist_ok=True)
    out_img.save(out_path)
    print(f"  我们的渲染: {out_path}")

    # 逐像素比较
    total_ref = sum(1 for p in ref_pixels if p[3] > 5)
    total_our = sum(1 for p in our_pixels if p[3] > 5)
    both = sum(1 for r, o in zip(ref_pixels, our_pixels) if r[3] > 5 and o[3] > 5)

    # 只在两者都有内容的像素比较
    close_count = 0
    diff_count = 0
    for i, (r, o) in enumerate(zip(ref_pixels, our_pixels)):
        if r[3] <= 5 and o[3] <= 5:
            continue
        max_diff = max(abs(r[j] - o[j]) for j in range(4))
        if max_diff <= 5:
            close_count += 1
        else:
            diff_count += 1

    print(f"  参考图有内容像素: {total_ref}")
    print(f"  我们有内容像素: {total_our}")
    print(f"  差异 > 5 的像素: {diff_count} ({100*diff_count/max(close_count+diff_count,1):.1f}%)")
    print(f"  差异 <= 5 的像素: {close_count}")

    # 分析差异类型
    if diff_count > 0:
        print("\n  差异分布:")
        alpha_diff = 0
        color_diff = 0
        both_diff = 0
        for r, o in zip(ref_pixels, our_pixels):
            if r[3] <= 5 and o[3] <= 5:
                continue
            max_diff = max(abs(r[j] - o[j]) for j in range(4))
            if max_diff <= 5:
                continue
            if abs(r[3] - o[3]) > 5 and all(abs(r[j] - o[j]) <= 5 for j in range(3)):
                alpha_diff += 1
            elif abs(r[3] - o[3]) <= 5 and any(abs(r[j] - o[j]) > 5 for j in range(3)):
                color_diff += 1
            else:
                both_diff += 1
        print(f"    仅 alpha 差异: {alpha_diff}")
        print(f"    仅颜色差异: {color_diff}")
        print(f"    颜色+alpha 差异: {both_diff}")

        # 找差异最大的像素
        max_pixel_diff = 0
        max_pixel_pos = (0, 0)
        for i, (r, o) in enumerate(zip(ref_pixels, our_pixels)):
            d = max(abs(r[j] - o[j]) for j in range(4))
            if d > max_pixel_diff:
                max_pixel_diff = d
                max_pixel_pos = (i % canvas_w, i // canvas_w)
        print(f"\n  最大差异像素: ({max_pixel_pos[0]},{max_pixel_pos[1]})")
        x, y = max_pixel_pos
        r = ref_pixels[y * canvas_w + x]
        o = our_pixels[y * canvas_w + x]
        print(f"    参考: RGBA({r[0]},{r[1]},{r[2]},{r[3]})")
        print(f"    我们: RGBA({o[0]},{o[1]},{o[2]},{o[3]})")

if __name__ == '__main__':
    main()
