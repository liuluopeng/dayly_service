use std::collections::{HashMap, HashSet};

pub fn luminance(r: u8, g: u8, b: u8) -> f64 {
    let rl = (r as f64 / 255.0).powf(2.2);
    let gl = (g as f64 / 255.0).powf(2.2);
    let bl = (b as f64 / 255.0).powf(2.2);
    0.2126 * rl + 0.7152 * gl + 0.0722 * bl
}

fn saturation(r: u8, g: u8, b: u8) -> f64 {
    let max = r.max(g).max(b) as f64;
    let min = r.min(g).min(b) as f64;
    if max == 0.0 { 0.0 } else { (max - min) / max }
}

fn is_dark(r: u8, g: u8, b: u8) -> bool { luminance(r, g, b) < 0.15 }

fn color_dist(c1: (u8, u8, u8), c2: (u8, u8, u8)) -> f64 {
    let dr = (c1.0 as f64 - c2.0 as f64);
    let dg = (c1.1 as f64 - c2.1 as f64);
    let db = (c1.2 as f64 - c2.2 as f64);
    (dr * dr + dg * dg + db * db).sqrt()
}

fn contrast_ratio(c1: (u8, u8, u8), c2: (u8, u8, u8)) -> f64 {
    let l1 = luminance(c1.0, c1.1, c1.2);
    let l2 = luminance(c2.0, c2.1, c2.2);
    let lt = l1.max(l2); let dk = l1.min(l2);
    (lt + 0.05) / (dk + 0.05)
}

fn to_int(r: u8, g: u8, b: u8) -> i64 {
    ((255i64 << 24) | ((r as i64) << 16) | ((g as i64) << 8) | (b as i64)) as i64
}

fn invert(r: u8, g: u8, b: u8) -> (u8, u8, u8) { (255 - r, 255 - g, 255 - b) }

/// 从图片字节数据提取主色和次色
pub fn extract_colors(image_bytes: &[u8]) -> (Option<i64>, Option<i64>) {
    let img = match image::load_from_memory(image_bytes) {
        Ok(i) => i,
        Err(_) => return (None, Some(0xFF2196F3)),
    };

    let mosaic = 20u32;
    let small = img.resize_exact(mosaic, mosaic, image::imageops::FilterType::Nearest);
    let rgba = small.to_rgba8();

    let mut counts: HashMap<(u8, u8, u8), usize> = HashMap::new();
    for y in 0..mosaic { for x in 0..mosaic {
        let p = rgba.get_pixel(x, y);
        *counts.entry((p[0], p[1], p[2])).or_insert(0) += 1;
    }}

    // 合并相似颜色
    let mut merged: HashMap<(u8, u8, u8), usize> = HashMap::new();
    let mut done: HashSet<(u8, u8, u8)> = HashSet::new();
    for &color in counts.keys() {
        if done.contains(&color) { continue; }
        let mut total = 0u32; let mut sr = 0u32; let mut sg = 0u32; let mut sb = 0u32;
        for (&oc, &cnt) in &counts {
            if done.contains(&oc) { continue; }
            if color_dist(color, oc) < 30.0 {
                total += cnt as u32; sr += oc.0 as u32 * cnt as u32;
                sg += oc.1 as u32 * cnt as u32; sb += oc.2 as u32 * cnt as u32;
                done.insert(oc);
            }
        }
        if total > 0 { merged.insert(((sr / total) as u8, (sg / total) as u8, (sb / total) as u8), total as usize); }
    }

    let mut sorted: Vec<_> = merged.into_iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1));
    let total_px = (mosaic * mosaic) as usize;

    let filtered: Vec<_> = sorted.into_iter().filter(|(c, _)| !is_dark(c.0, c.1, c.2)).collect();
    if filtered.is_empty() { return (None, Some(0xFF2196F3)); }

    // 最佳主色
    let mut best = 0; let mut best_score = 0.0f64;
    for (i, (c, cnt)) in filtered.iter().enumerate() {
        let s = saturation(c.0, c.1, c.2); let l = luminance(c.0, c.1, c.2);
        let score = (*cnt as f64 / total_px as f64) * (1.0 + s * 2.0) * (0.3 + l * 2.0);
        if score > best_score { best_score = score; best = i; }
    }
    let primary = to_int(filtered[best].0 .0, filtered[best].0 .1, filtered[best].0 .2);

    // 次色：高对比度
    let (pr, pg, pb) = filtered[best].0;
    let mut sec_best = None; let mut sec_ct = 0.0f64;
    for (i, (c, _)) in filtered.iter().enumerate() {
        if i == best { continue; }
        let ct = contrast_ratio((c.0, c.1, c.2), (pr, pg, pb));
        if ct > sec_ct && ct >= 5.0 { sec_ct = ct; sec_best = Some(i); }
    }
    let secondary = if let Some(i) = sec_best {
        to_int(filtered[i].0 .0, filtered[i].0 .1, filtered[i].0 .2)
    } else {
        let inv = invert(pr, pg, pb);
        to_int(inv.0, inv.1, inv.2)
    };

    (Some(primary), Some(secondary))
}
