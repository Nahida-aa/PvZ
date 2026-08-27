# 渲染 Godot 参考帧

本项目的 `tools/render_godot_frames.gd` 是一个未完成的简化版（直接加载 tres Animation，
缺少 BodyCorrect 等场景树偏移）。**不要使用它。**

生成参考帧的真正脚本在 Godot 项目中：

```
PVZ-Godot_dream_20260406_v1.2.0/tools/render_godot_frames.gd
```

## 用法

```bash
cd /path/to/PVZ-Godot_dream_20260406_v1.2.0

# 渲染植物动画帧
xvfb-run godot "res://tools/render_frames.tscn" -- \
  res://scenes/character/plant/plant_002_sun_flower.tscn \
  /path/to/PvZ/tmp/godot_sunflower \
  200 200 12
```

参数: `<plant_scene.tscn> <output_dir> <width> <height> <fps>`

## 渲染流程

1. 加载 `render_frames.tscn`（包含 SubViewport + RootNode）
2. 实例化植物 PackedScene（包含完整节点树: Plant → Body → BodyCorrect → Sprites）
3. 通过 AnimationPlayer 逐帧 seek，截取 SubViewport 图像
4. 输出 200×200 透明背景 PNG

## 关键细节

- `render_frames.tscn` 定义了 SubViewport(200×200, transparent_bg) + RootNode(100,100)
- 植物场景实例化后置于 RootNode 下，position=(0,0)
- BodyCorrect(-43, -69) 在场景树中，tres 动画的 position 是相对于 BodyCorrect 的
- 使用 AnimationTree 的 TimeScale 参数
- Z-order = 场景树子节点顺序（不是 tres track 顺序）
