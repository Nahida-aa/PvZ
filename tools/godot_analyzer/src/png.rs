use std::path::Path;

use crate::collect_files;

/// 分析 png: 尺寸、非透明像素包围盒、视觉重心, 以及重心相对几何中心的偏移
pub fn run(dir: &Path) {
    let files = collect_files(dir, "png");
    if files.is_empty() {
        eprintln!("未找到 png 文件: {}", dir.display());
        return;
    }
    println!(
        "{:<60} {:>10} {:>10} {:>10} {:>10} {:>10} {:>10}",
        "文件", "宽", "高", "bbox_l", "bbox_t", "bbox_r", "bbox_b"
    );
    println!("{}", "-".repeat(120));
    for f in &files {
        let data = match std::fs::read(f) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("读取失败 {}: {}", f.display(), e);
                continue;
            }
        };
        let img = match load_rgba(&data) {
            Some(i) => i,
            None => {
                eprintln!("无法解码: {}", f.display());
                continue;
            }
        };
        let (w, h, px) = img;
        let mut minx = w;
        let mut miny = h;
        let mut maxx = 0usize;
        let mut maxy = 0usize;
        let mut sx = 0u64;
        let mut sy = 0u64;
        let mut n = 0u64;
        for y in 0..h {
            for x in 0..w {
                let a = px[(y * w + x) * 4 + 3];
                if a > 10 {
                    if x < minx {
                        minx = x;
                    }
                    if x > maxx {
                        maxx = x;
                    }
                    if y < miny {
                        miny = y;
                    }
                    if y > maxy {
                        maxy = y;
                    }
                    sx += x as u64;
                    sy += y as u64;
                    n += 1;
                }
            }
        }
        let name = f.file_name().and_then(|s| s.to_str()).unwrap_or("");
        if n == 0 {
            println!("{:<60} {:>10} {:>10} (全透明)", name, w, h);
            continue;
        }
        let bbox_l = minx as f32;
        let bbox_t = miny as f32;
        let bbox_r = maxx as f32;
        let bbox_b = maxy as f32;
        println!(
            "{:<60} {:>10} {:>10} {:>10.1} {:>10.1} {:>10.1} {:>10.1}",
            name, w, h, bbox_l, bbox_t, bbox_r, bbox_b
        );
        // 额外输出视觉重心相对几何中心的偏移 (y 向下, 与 Godot 一致)
        let centroid_x = sx as f32 / n as f32;
        let centroid_y = sy as f32 / n as f32;
        let geom_cx = w as f32 / 2.0;
        let geom_cy = h as f32 / 2.0;
        println!(
            "    几何中心=({:.1},{:.1}) 视觉重心=({:.1},{:.1}) 重心偏移=({:+.1},{:+.1})",
            geom_cx, geom_cy, centroid_x, centroid_y, centroid_x - geom_cx, centroid_y - geom_cy
        );
    }
}

/// 解码 png 为 (宽, 高, RGBA 像素 vec)
fn load_rgba(data: &[u8]) -> Option<(usize, usize, Vec<u8>)> {
    use png::Transformations;
    let mut decoder = png::Decoder::new(data);
    decoder.set_transformations(Transformations::EXPAND);
    let mut reader = decoder.read_info().ok()?;
    let info = reader.info().clone();
    let mut buf = vec![0u8; info.width as usize * info.height as usize * 4];
    reader.next_frame(&mut buf).ok()?;
    Some((info.width as usize, info.height as usize, buf))
}
