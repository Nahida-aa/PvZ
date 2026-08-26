use clap::Parser;
use png::Transformations;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "plant_composer")]
#[command(about = "将植物身体部件 PNG 合成为完整植物图")]
struct Cli {
    /// 身体部件 PNG 所在目录
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

fn main() {
    let cli = Cli::parse();
    match compose(&cli) {
        Ok(path) => println!("输出: {}", path.display()),
        Err(e) => eprintln!("错误: {e}"),
    }
}

fn compose(cli: &Cli) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let entries = collect_pngs(&cli.input)?;
    if entries.is_empty() {
        return Err("目录下没有 PNG 文件".into());
    }

    let bg = parse_rgba(&cli.bg)?;

    // 加载所有 PNG
    let mut images: Vec<(String, Vec<u8>, u32, u32)> = Vec::new();
    for entry in &entries {
        let data = std::fs::read(entry)?;
        let (w, h, px) = load_rgba(&data)
            .ok_or_else(|| format!("无法解码: {}", entry.display()))?;
        let name = entry.file_stem().unwrap().to_string_lossy().to_string();
        images.push((name, px, w, h));
    }

    // 计算画布尺寸 (包围所有图片)
    let (canvas_w, canvas_h) = if let (Some(w), Some(h)) = (cli.width, cli.height) {
        (w, h)
    } else {
        // 找出最宽和最高的图片，用于估算画布
        // 简单策略：所有图片按列排列，居中对齐
        let total_w: u32 = images.iter().map(|(_, _, w, _)| *w).sum();
        let max_h: u32 = images.iter().map(|(_, _, _, h)| *h).max().unwrap_or(100);
        // 用总宽度作为画布宽度，最大高度+一些边距
        (total_w, max_h + 40)
    };

    let mut canvas = vec![0u8; (canvas_w * canvas_h * 4) as usize];

    // 填充背景色
    for pixel in canvas.chunks_exact_mut(4) {
        pixel.copy_from_slice(&bg);
    }

    // 将每张图片居中放置
    // 按文件名排序以确保一致的叠加顺序
    images.sort_by(|a, b| a.0.cmp(&b.0));

    // 计算每张图片的放置位置 (水平排列，垂直居中)
    let mut x_offset = 0u32;
    for (_, px, w, h) in &images {
        let y_offset = (canvas_h - h) / 2;
        blit(&mut canvas, canvas_w, canvas_h, px, *w, *h, x_offset, y_offset);
        x_offset += w;
    }

    // 写入 PNG
    write_png(&cli.output, &canvas, canvas_w, canvas_h)?;

    Ok(cli.output.clone())
}

/// 将源图像合成到目标画布的指定位置
fn blit(
    canvas: &mut [u8],
    cw: u32,
    ch: u32,
    src: &[u8],
    sw: u32,
    sh: u32,
    dx: u32,
    dy: u32,
) {
    for y in 0..sh {
        let cy = dy + y;
        if cy >= ch {
            continue;
        }
        for x in 0..sw {
            let cx = dx + x;
            if cx >= cw {
                continue;
            }
            let si = ((y * sw + x) * 4) as usize;
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
