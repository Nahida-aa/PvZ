use std::path::{Path, PathBuf};
use std::process::exit;

mod png;
mod tscn;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("用法:");
        eprintln!("  godot_analyzer png <目录>     分析目录下所有 png 的包围盒/重心");
        eprintln!("  godot_analyzer tscn <文件>    解析 .tscn 节点坐标");
        exit(1);
    }
    let cmd = &args[1];
    let path = &args[2];
    match cmd.as_str() {
        "png" => png::run(Path::new(path)),
        "tscn" => tscn::run(Path::new(path)),
        other => {
            eprintln!("未知子命令: {other}");
            exit(1);
        }
    }
}

/// 递归收集目录下所有指定扩展名的文件
pub fn collect_files(dir: &Path, ext: &str) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&d) else {
            continue;
        };
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
            } else if p.extension().and_then(|s| s.to_str()) == Some(ext) {
                out.push(p);
            }
        }
    }
    out.sort();
    out
}
