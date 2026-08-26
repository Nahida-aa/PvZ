use clap::Parser;
use png::Transformations;
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "plant_composer")]
#[command(about = "将植物/物品身体部件 PNG 合成为完整图像")]
struct Cli {
    #[arg(short, long)]
    input: PathBuf,
    #[arg(short, long)]
    output: PathBuf,
    #[arg(short = 'W', long)]
    width: Option<u32>,
    #[arg(short = 'H', long)]
    height: Option<u32>,
    #[arg(long, default_value = "0,0,0,0")]
    bg: String,
}

#[derive(Deserialize)]
struct RigConfig {
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
    /// 均匀缩放 (scale_x = scale_y)
    #[serde(default)]
    scale: Option<f32>,
    /// 非均匀缩放 X
    #[serde(default)]
    scale_x: Option<f32>,
    /// 非均匀缩放 Y
    #[serde(default)]
    scale_y: Option<f32>,
    #[serde(default)]
    rotation: Option<f32>,
    #[serde(default)]
    skew: Option<f32>,
    #[serde(default = "default_visible")]
    visible: bool,
}

fn default_visible() -> bool {
    true
}

/// 2x3 仿射矩阵: 源像素 (u,v) → 画布 (a*u + b*v + tx, c*u + d*v + ty)
struct Affine {
    a: f32,
    b: f32,
    c: f32,
    d: f32,
    tx: f32,
    ty: f32,
}

impl Affine {
    fn apply(&self, u: f32, v: f32) -> (f32, f32) {
        (self.a * u + self.b * v + self.tx, self.c * u + self.d * v + self.ty)
    }

    fn bounding_box(&self, w: f32, h: f32) -> (i32, i32, i32, i32) {
        let corners = [self.apply(0.0, 0.0), self.apply(w, 0.0), self.apply(0.0, h), self.apply(w, h)];
        let min_x = corners.iter().map(|p| p.0).fold(f32::INFINITY, f32::min);
        let min_y = corners.iter().map(|p| p.1).fold(f32::INFINITY, f32::min);
        let max_x = corners.iter().map(|p| p.0).fold(f32::NEG_INFINITY, f32::max);
        let max_y = corners.iter().map(|p| p.1).fold(f32::NEG_INFINITY, f32::max);
        (min_x.floor() as i32, min_y.floor() as i32, max_x.ceil() as i32, max_y.ceil() as i32)
    }
}

struct PartImage {
    px: Vec<u8>,
    w: u32,
    h: u32,
    /// 精灵中心在 Godot 世界坐标 (x右, y下)
    cx: f32,
    cy: f32,
    z: f32,
    scale_x: f32,
    scale_y: f32,
    rotation: f32,
    skew: f32,
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
    let parts_dir = cli.input.join("parts");

    let jsonc_path = cli.input.join("rig.jsonc");
    let rig = if jsonc_path.exists() {
        let text = std::fs::read_to_string(&jsonc_path)?;
        let val = jsonc_parser::parse_to_serde_value(&text, &Default::default())?
            .ok_or("无法解析 rig.jsonc")?;
        serde_json::from_value(val)?
    } else {
        let dir = if parts_dir.exists() { &parts_dir } else { &cli.input };
        let entries = collect_pngs(dir)?;
        if entries.is_empty() {
            return Err("目录下没有 PNG 文件".into());
        }
        RigConfig {
            scale: 1.0,
            parts: entries.iter().enumerate().map(|(i, p)| PartDef {
                image: p.file_name().unwrap().to_string_lossy().to_string(),
                x: 0.0, y: 0.0, z: i as f32,
                scale: Some(1.0), scale_x: None, scale_y: None,
                rotation: None, skew: None, visible: true,
            }).collect(),
        }
    };

    let mut images: Vec<PartImage> = Vec::new();
    for part in &rig.parts {
        if !part.visible { continue; }
        let path = if parts_dir.join(&part.image).exists() {
            parts_dir.join(&part.image)
        } else {
            cli.input.join(&part.image)
        };
        let data = std::fs::read(&path).map_err(|e| format!("读取 {} 失败: {e}", path.display()))?;
        let (w, h, px) = load_rgba(&data).ok_or_else(|| format!("无法解码: {}", path.display()))?;
        let uniform_scale = part.scale.unwrap_or(rig.scale);
        let sx = part.scale_x.unwrap_or(uniform_scale);
        let sy = part.scale_y.unwrap_or(uniform_scale);
        images.push(PartImage {
            px, w, h,
            cx: part.x, cy: part.y, z: part.z,
            scale_x: sx,
            scale_y: sy,
            rotation: part.rotation.unwrap_or(0.0),
            skew: part.skew.unwrap_or(0.0),
        });
    }

    images.sort_by(|a, b| a.z.partial_cmp(&b.z).unwrap());

    let (canvas_w, canvas_h, offset_x, offset_y) = if let (Some(w), Some(h)) = (cli.width, cli.height) {
        (w, h, 0.0f32, 0.0f32)
    } else {
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;
        for img in &images {
            let aff = make_affine(img, 0.0, 0.0);
            let (bx0, by0, bx1, by1) = aff.bounding_box(img.w as f32, img.h as f32);
            min_x = min_x.min(bx0 as f32);
            min_y = min_y.min(by0 as f32);
            max_x = max_x.max(bx1 as f32);
            max_y = max_y.max(by1 as f32);
        }
        let w = (max_x - min_x).ceil() as u32;
        let h = (max_y - min_y).ceil() as u32;
        (w.max(1), h.max(1), -min_x, -min_y)
    };

    let mut canvas = vec![0u8; (canvas_w * canvas_h * 4) as usize];
    for pixel in canvas.chunks_exact_mut(4) {
        pixel.copy_from_slice(&bg);
    }

    for img in &images {
        let aff = make_affine(img, offset_x, offset_y);
        let det = aff.a * aff.d - aff.b * aff.c;
        if det.abs() < 1e-10 { continue; }
        let inv_det = 1.0 / det;

        let (bx0, by0, bx1, by1) = aff.bounding_box(img.w as f32, img.h as f32);
        let cy_start = by0.max(0) as u32;
        let cy_end = (by1 as u32).min(canvas_h);
        let cx_start = bx0.max(0) as u32;
        let cx_end = (bx1 as u32).min(canvas_w);

        for cy in cy_start..cy_end {
            for cx in cx_start..cx_end {
                let dx = cx as f32 - aff.tx;
                let dy = cy as f32 - aff.ty;
                let u = (aff.d * dx - aff.b * dy) * inv_det;
                let v = (-aff.c * dx + aff.a * dy) * inv_det;

                let u0 = u.floor() as i32;
                let v0 = v.floor() as i32;
                if u0 < 0 || v0 < 0 || u0 + 1 >= img.w as i32 || v0 + 1 >= img.h as i32 {
                    continue;
                }
                let uf = u - u0 as f32;
                let vf = v - v0 as f32;

                let si = |pu: i32, pv: i32| ((pv * img.w as i32 + pu) * 4) as usize;
                let ci = ((cy * canvas_w + cx) * 4) as usize;

                let s00 = &img.px[si(u0, v0)..si(u0, v0) + 4];
                let s10 = &img.px[si(u0 + 1, v0)..si(u0 + 1, v0) + 4];
                let s01 = &img.px[si(u0, v0 + 1)..si(u0, v0 + 1) + 4];
                let s11 = &img.px[si(u0 + 1, v0 + 1)..si(u0 + 1, v0 + 1) + 4];

                let mut src = [0f32; 4];
                for ch in 0..4 {
                    src[ch] = s00[ch] as f32 * (1.0 - uf) * (1.0 - vf)
                        + s10[ch] as f32 * uf * (1.0 - vf)
                        + s01[ch] as f32 * (1.0 - uf) * vf
                        + s11[ch] as f32 * uf * vf;
                }

                let sa = src[3] / 255.0;
                if sa < 0.001 { continue; }
                let da = canvas[ci + 3] as f32 / 255.0;
                let out_a = sa + da * (1.0 - sa);
                if out_a > 0.0 {
                    for ch in 0..3 {
                        canvas[ci + ch] = ((src[ch] * sa + canvas[ci + ch] as f32 * da * (1.0 - sa)) / out_a) as u8;
                    }
                    canvas[ci + 3] = (out_a * 255.0) as u8;
                }
            }
        }
    }

    write_png(&cli.output, &canvas, canvas_w, canvas_h)?;
    Ok(cli.output.clone())
}

/// 创建从纹理到画布的仿射变换
///
/// rig.jsonc 中 x,y 是精灵中心在 Godot 世界坐标 (x右, y下)
/// 纹理像素 (u,v): u向右, v向下, 原点在左上角
/// 画布坐标: x向右, y向下 (与 Godot 一致)
///
/// Godot Sprite2D (centered=false) 变换链: Scale → Skew → Rotate → Translate
/// 矩阵: [[a,b],[c,d]] = R * Sk * S
fn make_affine(img: &PartImage, offset_x: f32, offset_y: f32) -> Affine {
    let cos_r = img.rotation.cos();
    let sin_r = img.rotation.sin();
    let tan_k = img.skew.tan();
    let sx = img.scale_x;
    let sy = img.scale_y;
    let a = cos_r * sx;
    let b = (cos_r * tan_k - sin_r) * sy;
    let c = sin_r * sx;
    let d = (sin_r * tan_k + cos_r) * sy;
    let hw = img.w as f32 * 0.5;
    let hh = img.h as f32 * 0.5;
    Affine {
        a,
        b,
        c,
        d,
        tx: img.cx - a * hw - b * hh + offset_x,
        ty: img.cy - c * hw - d * hh + offset_y,
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

fn write_png(path: &Path, data: &[u8], w: u32, h: u32) -> Result<(), Box<dyn std::error::Error>> {
    let file = std::fs::File::create(path)?;
    let mut encoder = png::Encoder::new(file, w, h);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header()?;
    writer.write_image_data(data)?;
    Ok(())
}

fn parse_rgba(s: &str) -> Result<[u8; 4], Box<dyn std::error::Error>> {
    let parts: Vec<u8> = s.split(',').map(|p| p.trim().parse::<u8>()).collect::<Result<_, _>>()?;
    if parts.len() != 4 { return Err("需要4个值 R,G,B,A".into()); }
    Ok([parts[0], parts[1], parts[2], parts[3]])
}
