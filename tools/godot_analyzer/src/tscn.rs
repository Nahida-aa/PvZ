use std::path::Path;

/// 极简 .tscn 解析: 提取每个节点的 name/parent/type 及常见坐标字段
/// 用于人工核对 Godot 素材的世界/屏幕坐标 (Godot 世界原点在视口中心)
pub fn run(file: &Path) {
    let text = match std::fs::read_to_string(file) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("读取失败 {}: {}", file.display(), e);
            return;
        }
    };

    println!(
        "{:<34} {:<20} {:<20} {}",
        "节点", "parent", "type", "position / scale / offset"
    );
    println!("{}", "-".repeat(110));

    // 按行扫描, 把每个 [node ...] 与其后的属性行归一组
    let mut cur_name: Option<String> = None;
    let mut cur_parent: Option<String> = None;
    let mut cur_type: Option<String> = None;
    let mut cur_pos: Option<String> = None;
    let mut cur_scale: Option<String> = None;
    let mut cur_off: Vec<String> = Vec::new();
    let mut cur_anchor: Option<String> = None;
    let mut cur_layout: Option<String> = None;

    let flush = |name: &Option<String>,
                     parent: &Option<String>,
                     type_: &Option<String>,
                     pos: &Option<String>,
                     scale: &Option<String>,
                     off: &[String],
                     anchor: &Option<String>,
                     layout: &Option<String>| {
        if name.is_none() {
            return;
        }
        let mut s = String::new();
        if let Some(p) = pos {
            s.push_str(&format!("pos=({}) ", p));
        }
        if let Some(s2) = scale {
            s.push_str(&format!("scale=({}) ", s2));
        }
        for o in off {
            s.push_str(&format!("{} ", o));
        }
        if let Some(a) = anchor {
            s.push_str(&format!("anchor={} ", a));
        }
        if let Some(l) = layout {
            s.push_str(&format!("layout={} ", l));
        }
        println!(
            "{:<34} {:<20} {:<20} {}",
            name.as_deref().unwrap_or(""),
            parent.as_deref().unwrap_or(""),
            type_.as_deref().unwrap_or(""),
            s
        );
    };

    for raw in text.lines() {
        let line = raw.trim();
        if line.starts_with("[node name=") {
            flush(
                &cur_name, &cur_parent, &cur_type, &cur_pos, &cur_scale, &cur_off, &cur_anchor,
                &cur_layout,
            );
            let (name, parent, type_) = parse_header(line);
            cur_name = name;
            cur_parent = parent;
            cur_type = type_;
            cur_pos = None;
            cur_scale = None;
            cur_off.clear();
            cur_anchor = None;
            cur_layout = None;
        } else if line.starts_with('[') {
            // 其它块 ([sub_resource] 等) 结束当前节点
            flush(
                &cur_name, &cur_parent, &cur_type, &cur_pos, &cur_scale, &cur_off, &cur_anchor,
                &cur_layout,
            );
            cur_name = None;
        } else if let Some(v) = line.strip_prefix("position = Vector2(") {
            cur_pos = Some(v.trim_end_matches(')').to_string());
        } else if let Some(v) = line.strip_prefix("scale = Vector2(") {
            cur_scale = Some(v.trim_end_matches(')').to_string());
        } else if let Some(v) = line.strip_prefix("offset_left =") {
            cur_off.push(format!("offL={}", v.trim_end_matches(',').trim()));
        } else if let Some(v) = line.strip_prefix("offset_top =") {
            cur_off.push(format!("offT={}", v.trim_end_matches(',').trim()));
        } else if let Some(v) = line.strip_prefix("offset_right =") {
            cur_off.push(format!("offR={}", v.trim_end_matches(',').trim()));
        } else if let Some(v) = line.strip_prefix("offset_bottom =") {
            cur_off.push(format!("offB={}", v.trim_end_matches(',').trim()));
        } else if let Some(v) = line.strip_prefix("anchors_preset =") {
            cur_anchor = Some(v.trim_end_matches(',').trim().to_string());
        } else if let Some(v) = line.strip_prefix("layout_mode =") {
            cur_layout = Some(v.trim_end_matches(',').trim().to_string());
        }
    }
    flush(
        &cur_name, &cur_parent, &cur_type, &cur_pos, &cur_scale, &cur_off, &cur_anchor, &cur_layout,
    );
}

fn parse_header(line: &str) -> (Option<String>, Option<String>, Option<String>) {
    let body = line.strip_prefix("[node name=").unwrap_or(line);
    let body = body.trim_end_matches(']');
    let name = extract_quoted(body);
    let parent = body
        .split("parent=")
        .nth(1)
        .and_then(|s| extract_quoted(s.trim_start()));
    let type_ = body
        .split("type=")
        .nth(1)
        .and_then(|s| extract_quoted(s.trim_start()));
    (name, parent, type_)
}

fn extract_quoted(s: &str) -> Option<String> {
    let s = s.trim();
    if let Some(rest) = s.strip_prefix('"') {
        let end = rest.find('"')?;
        return Some(rest[..end].to_string());
    }
    None
}
