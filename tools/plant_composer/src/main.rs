use clap::Parser;
use png::Transformations;
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "plant_composer")]
#[command(about = "将植物/物品身体部件 PNG 合成为完整图像")]
struct Cli {
    /// 身体部件目录 (包含 PNG 和 skeleton.jsonc)
    #[arg(short, long)]
    input: PathBuf,

    /// 输出文件路径
    #[arg(short, long)]
    output: PathBuf,

    /// 画布宽度 (默认自动计算)
    #[arg(short = 'W', long)]
    width: Option<u32>,

    /// 画布高度 (默认自动计算)
    #[arg(short = 'H', long)]
    height: Option<u32>,

    /// 背景颜色 R,G,B,A (默认透明)
    #[arg(long, default_value = "0,0,0,0")]
    bg: String,
}

#[derive(Deserialize)]
struct ReanimConfig {
    parts: Vec<PartDef>,
    #[serde(default = "default_scale")]
    scale: f32,
}

fn default_scale() -> f32 {
    1.0
}

#[derive(Deserialize)]
struct PartDef {
    image: String,
    x: f32,
    y: f32,
    z: f32,
    #[serde(default)]
    scale: Option<f32>,
}

fn main() {
    let cli = Cli::parse();
    match compose(&cli) {
        Ok(path) => println!("输出: {}", path.display()),
        Err(e) => eprintln!("错误: {e}"),
    }
}

fn compose(cli: &Cli) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let bg = parse_rgba(&cli.bg)?;

    // 部件目录结构: <input>/skeleton.jsonc + <input>/reanim/<image>
    let reanim_dir = cli.input.join("reanim");

    // 尝试加载 skeleton.jsonc
    let jsonc_path = cli.input.join("skeleton.jsonc");
    let parts = if jsonc_path.exists() {
        let text = std::fs::read_to_string(&jsonc_path)?;
        let val = jsonc_parser::parse_to_serde_value(&text, &Default::default())?
            .ok_or("无法解析 skeleton.jsonc")?;
        serde_json::from_value(val)?
    } else {
        // 无配置时，按文件名排序水平排列
        let dir = if reanim_dir.exists() {
            &reanim_dir
        } else {
            &cli.input
        };
        let entries = collect_pngs(dir)?;
        if entries.is_empty() {
            return Err("目录下没有 PNG 文件".into());
        }
        ReanimConfig {
            scale: 1.0,
            parts: entries
                .iter()
                .enumerate()
                .map(|(i, _)| PartDef {
                    image: entries[i]
                        .file_name()
                        .unwrap()
                        .to_string_lossy()
                        .to_string(),
                    x: 0.0,
                    y: 0.0,
                    z: i as f32,
                    scale: Some(1.0),
                })
                .collect(),
        }
    };

    // 加载所有引用的 PNG
    let mut images: Vec<(String, Vec<u8>, u32, u32, f32, f32, f32, f32)> = Vec::new(); // name, px, raw_w, raw_h, x, y, z, scale
    for part in &parts.parts {
        let path = if reanim_dir.join(&part.image).exists() {
            reanim_dir.join(&part.image)
        } else {
            cli.input.join(&part.image)
        };
        let data = std::fs::read(&path)
            .map_err(|e| format!("读取 {} 失败: {e}", path.display()))?;
        let (w, h, px) =
            load_rgba(&data).ok_or_else(|| format!("无法解码: {}", path.display()))?;
        let scale = part.scale.unwrap_or(parts.scale);
        images.push((part.image.clone(), px, w, h, part.x, part.y, part.z, scale));
    }

    // 按 z-order 排序
    images.sort_by(|a, b| a.6.partial_cmp(&b.6).unwrap());

    // 计算画布尺寸
    let (canvas_w, canvas_h) = if let (Some(w), Some(h)) = (cli.width, cli.height) {
        (w, h)
    } else {
        // 计算包围盒 (x,y 为中心点, 缩放后的尺寸)
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;
        for (_, _, w, h, x, y, _, scale) in &images {
            let sw = *w as f32 * scale;
            let sh = *h as f32 * scale;
            min_x = min_x.min(x - sw / 2.0);
            min_y = min_y.min(y - sh / 2.0);
            max_x = max_x.max(x + sw / 2.0);
            max_y = max_y.max(y + sh / 2.0);
        }
        let w = (max_x - min_x).ceil() as u32;
        let h = (max_y - min_y).ceil() as u32;
        (w.max(1), h.max(1))
    };

    let mut canvas = vec![0u8; (canvas_w * canvas_h * 4) as usize];

    // 填充背景色
    for pixel in canvas.chunks_exact_mut(4) {
        pixel.copy_from_slice(&bg);
    }

    // 计算偏移使所有图片都在画布内 (x,y 为中心点)
    let mut min_x = f32::MAX;
    let mut min_y = f32::MAX;
    for (_, _, w, h, x, y, _, scale) in &images {
        let sw = *w as f32 * scale;
        let sh = *h as f32 * scale;
        min_x = min_x.min(x - sw / 2.0);
        min_y = min_y.min(y - sh / 2.0);
    }
    let offset_x = -min_x;
    let offset_y = -min_y;

    // 合成 (x,y 为中心点, y 向上)
    for (_, px, w, h, x, y, _, scale) in &images {
        let sw = *w as f32 * scale;
        let sh = *h as f32 * scale;
        // 中心点 -> 左上角 (canvas 坐标: y 向下)
        let dx = (x + offset_x - sw / 2.0) as u32;
        let dy = (canvas_h as f32 - (y + offset_y) - sh / 2.0) as u32;
        blit_scaled(&mut canvas, canvas_w, canvas_h, px, *w, *h, dx, dy, *scale);
    }

    // 写入 PNG
    write_png(&cli.output, &canvas, canvas_w, canvas_h)?;

    Ok(cli.output.clone())
}

/// 将源图像合成到目标画布的指定位置 (带缩放)
fn blit_scaled(
    canvas: &mut [u8],
    cw: u32,
    ch: u32,
    src: &[u8],
    sw: u32,
    sh: u32,
    dx: u32,
    dy: u32,
    scale: f32,
) {
    let out_w = (sw as f32 * scale).ceil() as u32;
    let out_h = (sh as f32 * scale).ceil() as u32;
    for oy in 0..out_h {
        let cy = dy + oy;
        if cy >= ch {
            continue;
        }
        let sy = (oy as f32 / scale) as u32;
        if sy >= sh {
            continue;
        }
        for ox in 0..out_w {
            let cx = dx + ox;
            if cx >= cw {
                continue;
            }
            let sx = (ox as f32 / scale) as u32;
            if sx >= sw {
                continue;
            }
            let si = ((sy * sw + sx) * 4) as usize;
            let ci = ((cy * cw + cx) * 4) as usize;
            // Alpha 混合
            let sa = src[si + 3] as f32 / 255.0;
            let da = canvas[ci + 3] as f32 / 255.0;
            let out_a = sa + da * (1.0 - sa);
            if out_a > 0.0 {
                for c in 0..3 {
                    let s = src[si + c] as f32;
                    let d = canvas[ci + c] as f32;
                    canvas[ci + c] = ((s * sa + d * da * (1.0 - sa)) / out_a) as u8;
                }
                canvas[ci + 3] = (out_a * 255.0) as u8;
            }
        }
    }
}

fn collect_pngs(dir: &Path) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut out = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("png") {
            out.push(path);
        }
    }
    out.sort();
    Ok(out)
}

fn load_rgba(data: &[u8]) -> Option<(u32, u32, Vec<u8>)> {
    let mut decoder = png::Decoder::new(data);
    decoder.set_transformations(Transformations::EXPAND);
    let mut reader = decoder.read_info().ok()?;
    let info = reader.info().clone();
    let mut buf = vec![0u8; info.width as usize * info.height as usize * 4];
    reader.next_frame(&mut buf).ok()?;
    Some((info.width, info.height, buf))
}

fn write_png(
    path: &Path,
    data: &[u8],
    w: u32,
    h: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    let file = std::fs::File::create(path)?;
    let mut encoder = png::Encoder::new(file, w, h);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header()?;
    writer.write_image_data(data)?;
    Ok(())
}

fn parse_rgba(s: &str) -> Result<[u8; 4], Box<dyn std::error::Error>> {
    let parts: Vec<u8> = s
        .split(',')
        .map(|p| p.trim().parse::<u8>())
        .collect::<Result<_, _>>()?;
    if parts.len() != 4 {
        return Err("需要4个值 R,G,B,A".into());
    }
    Ok([parts[0], parts[1], parts[2], parts[3]])
}
