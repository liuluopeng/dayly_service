use crate::api::logger_bridge::log_to_dart;

#[derive(Debug, Clone)]
pub struct ColorInfo {
    pub background_color: Option<u32>,
    pub secondary_color: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct ImageColorResult {
    pub name: String,
    pub color_info: ColorInfo,
}

pub fn luminance(r: u8, g: u8, b: u8) -> f64 {
    let r_linear = (r as f64 / 255.0).powf(2.2);
    let g_linear = (g as f64 / 255.0).powf(2.2);
    let b_linear = (b as f64 / 255.0).powf(2.2);
    0.2126 * r_linear + 0.7152 * g_linear + 0.0722 * b_linear
}

pub fn contrast_ratio(color1: (u8, u8, u8), color2: (u8, u8, u8)) -> f64 {
    let lum1 = luminance(color1.0, color1.1, color1.2);
    let lum2 = luminance(color2.0, color2.1, color2.2);
    let lighter = lum1.max(lum2);
    let darker = lum1.min(lum2);
    (lighter + 0.05) / (darker + 0.05)
}

pub fn color_to_int(r: u8, g: u8, b: u8, a: u8) -> u32 {
    ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}

pub fn is_vibrant(r: u8, g: u8, b: u8) -> bool {
    let max = r.max(g).max(b) as f64;
    let min = r.min(g).min(b) as f64;
    let saturation = if max == 0.0 { 0.0 } else { (max - min) / max };
    saturation > 0.3
}

pub fn saturation(r: u8, g: u8, b: u8) -> f64 {
    let max = r.max(g).max(b) as f64;
    let min = r.min(g).min(b) as f64;
    if max == 0.0 {
        0.0
    } else {
        (max - min) / max
    }
}

pub fn is_too_dark(r: u8, g: u8, b: u8) -> bool {
    luminance(r, g, b) < 0.15
}

pub fn color_distance(c1: (u8, u8, u8), c2: (u8, u8, u8)) -> f64 {
    let dr = (c1.0 as f64 - c2.0 as f64).abs();
    let dg = (c1.1 as f64 - c2.1 as f64).abs();
    let db = (c1.2 as f64 - c2.2 as f64).abs();
    (dr * dr + dg * dg + db * db).sqrt()
}

pub fn are_colors_similar(c1: (u8, u8, u8), c2: (u8, u8, u8), threshold: f64) -> bool {
    color_distance(c1, c2) < threshold
}

pub fn merge_similar_colors(
    color_counts: &mut std::collections::HashMap<(u8, u8, u8), usize>,
    threshold: f64,
) -> std::collections::HashMap<(u8, u8, u8), usize> {
    let mut merged: std::collections::HashMap<(u8, u8, u8), usize> =
        std::collections::HashMap::new();
    let mut processed: std::collections::HashSet<(u8, u8, u8)> = std::collections::HashSet::new();

    let mut colors: Vec<(u8, u8, u8)> = color_counts.keys().cloned().collect();

    for color in colors {
        if processed.contains(&color) {
            continue;
        }

        let mut total_count = 0;
        let mut sum_r = 0u32;
        let mut sum_g = 0u32;
        let mut sum_b = 0u32;

        for (&other_color, &count) in color_counts.iter() {
            if processed.contains(&other_color) {
                continue;
            }

            if are_colors_similar(color, other_color, threshold) {
                total_count += count;
                sum_r += other_color.0 as u32 * count as u32;
                sum_g += other_color.1 as u32 * count as u32;
                sum_b += other_color.2 as u32 * count as u32;
                processed.insert(other_color);
            }
        }

        if total_count > 0 {
            let avg_r = (sum_r / total_count as u32) as u8;
            let avg_g = (sum_g / total_count as u32) as u8;
            let avg_b = (sum_b / total_count as u32) as u8;
            merged.insert((avg_r, avg_g, avg_b), total_count);
        }
    }

    merged
}

pub fn invert_color(r: u8, g: u8, b: u8) -> (u8, u8, u8) {
    (255 - r, 255 - g, 255 - b)
}

pub fn is_monochromatic(colors: &Vec<((u8, u8, u8), usize)>, total_pixels: usize) -> bool {
    if colors.is_empty() {
        return false;
    }

    let dominant_count = colors[0].1;
    let dominance_ratio = dominant_count as f64 / total_pixels as f64;

    if dominance_ratio > 0.80 {
        return true;
    }

    if colors.len() <= 3 {
        let mut total_contrast = 0.0;
        let mut count = 0;

        for i in 0..colors.len().min(5) {
            for j in (i + 1)..colors.len().min(5) {
                let contrast = contrast_ratio(colors[i].0, colors[j].0);
                total_contrast += contrast;
                count += 1;
            }
        }

        if count > 0 {
            let avg_contrast = total_contrast / count as f64;
            if avg_contrast < 2.5 {
                return true;
            }
        }
    }

    false
}

#[flutter_rust_bridge::frb]
pub fn extract_colors(image_data: Vec<u8>) -> Result<ColorInfo, String> {
    let img =
        image::load_from_memory(&image_data).map_err(|e| format!("Failed to load image: {}", e))?;

    let img_rgba = img.to_rgba8();
    let (width, height) = img_rgba.dimensions();

    let mosaic_width = 20;
    let mosaic_height = 20;

    let img = img.resize_exact(
        mosaic_width,
        mosaic_height,
        image::imageops::FilterType::Nearest,
    );
    let rgba = img.to_rgba8();

    let mut color_counts: std::collections::HashMap<(u8, u8, u8), usize> =
        std::collections::HashMap::new();
    let mut total_pixels = 0;

    for y in 0..mosaic_height {
        for x in 0..mosaic_width {
            let pixel = rgba.get_pixel(x, y);
            let r = pixel[0];
            let g = pixel[1];
            let b = pixel[2];
            *color_counts.entry((r, g, b)).or_insert(0) += 1;
            total_pixels += 1;
        }
    }

    let merged_colors = merge_similar_colors(&mut color_counts, 30.0);

    let mut sorted_colors: Vec<_> = merged_colors.into_iter().collect();
    sorted_colors.sort_by(|a, b| b.1.cmp(&a.1));

    let filtered_colors: Vec<((u8, u8, u8), usize)> = sorted_colors
        .into_iter()
        .filter(|(color, _)| !is_too_dark(color.0, color.1, color.2))
        .collect();

    let mut background_color: Option<u32> = None;
    let mut secondary_color: Option<u32> = None;

    if !filtered_colors.is_empty() {
        let mut best_idx = 0;
        let mut best_score = 0.0;

        for (idx, (color, count)) in filtered_colors.iter().enumerate() {
            let sat = saturation(color.0, color.1, color.2);
            let lum = luminance(color.0, color.1, color.2);
            let dominance = *count as f64 / total_pixels as f64;
            let score = dominance * (1.0 + sat * 2.0) * (0.3 + lum * 2.0);
            if score > best_score {
                best_score = score;
                best_idx = idx;
            }
        }

        let (r, g, b) = filtered_colors[best_idx].0;
        background_color = Some(color_to_int(r, g, b, 255));

        let dom_r = r;
        let dom_g = g;
        let dom_b = b;

        let is_mono = is_monochromatic(&filtered_colors, total_pixels);

        if is_mono {
            let inverted = invert_color(dom_r, dom_g, dom_b);
            secondary_color = Some(color_to_int(inverted.0, inverted.1, inverted.2, 255));
        } else {
            let mut best_secondary_idx = None;
            let mut best_contrast = 0.0;
            let min_contrast_threshold = 5.0;

            for (idx, (color, _)) in filtered_colors.iter().enumerate() {
                if idx == best_idx {
                    continue;
                }
                let contrast = contrast_ratio((color.0, color.1, color.2), (dom_r, dom_g, dom_b));
                if contrast > best_contrast && contrast >= min_contrast_threshold {
                    best_contrast = contrast;
                    best_secondary_idx = Some(idx);
                }
            }

            if let Some(sec_idx) = best_secondary_idx {
                let (r, g, b) = filtered_colors[sec_idx].0;
                secondary_color = Some(color_to_int(r, g, b, 255));
            } else {
                let inverted = invert_color(dom_r, dom_g, dom_b);
                secondary_color = Some(color_to_int(inverted.0, inverted.1, inverted.2, 255));
            }
        }
    }

    if secondary_color.is_none() {
        secondary_color = Some(0xFF2196F3);
    }

    Ok(ColorInfo {
        background_color,
        secondary_color,
    })
}

pub fn extract_colors_from_path(image_path: String) -> Result<ColorInfo, String> {
    let image_data =
        std::fs::read(&image_path).map_err(|e| format!("Failed to read image file: {}", e))?;
    extract_colors(image_data)
}

pub fn extract_colors_from_paths(image_paths: Vec<String>) -> Vec<ImageColorResult> {
    image_paths
        .iter()
        .filter_map(|path| {
            extract_colors_from_path(path.clone())
                .ok()
                .map(|color_info| ImageColorResult {
                    name: path.clone(),
                    color_info,
                })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_to_int() {
        assert_eq!(color_to_int(255, 0, 0, 255), 0xFFFF0000);
        assert_eq!(color_to_int(0, 255, 0, 255), 0xFF00FF00);
        assert_eq!(color_to_int(0, 0, 255, 255), 0xFF0000FF);
        assert_eq!(color_to_int(128, 128, 128, 255), 0xFF808080);
        assert_eq!(color_to_int(0, 0, 0, 0), 0x00000000);
    }

    #[test]
    fn test_is_vibrant() {
        assert!(is_vibrant(255, 0, 0));
        assert!(is_vibrant(0, 255, 0));
        assert!(is_vibrant(0, 0, 255));
        assert!(!is_vibrant(128, 128, 128));
        assert!(!is_vibrant(200, 200, 200));
    }

    #[test]
    fn test_saturation() {
        let sat = saturation(255, 0, 0);
        assert_eq!(sat, 1.0);
        let sat = saturation(128, 128, 128);
        assert_eq!(sat, 0.0);
        let sat = saturation(255, 128, 128);
        assert!(sat > 0.0 && sat < 1.0);
    }

    #[test]
    fn test_is_too_dark() {
        assert!(is_too_dark(10, 10, 10));
        assert!(is_too_dark(50, 50, 50));
        assert!(!is_too_dark(100, 100, 100));
        assert!(!is_too_dark(255, 255, 255));
    }

    #[test]
    fn test_luminance() {
        let lum = luminance(255, 255, 255);
        assert_eq!(lum, 1.0);
        let lum = luminance(0, 0, 0);
        assert_eq!(lum, 0.0);
        let lum = luminance(128, 128, 128);
        assert!(lum > 0.0 && lum < 1.0);
    }

    #[test]
    fn test_contrast_ratio() {
        let contrast = contrast_ratio((255, 255, 255), (0, 0, 0));
        assert!(contrast > 20.0);
        let contrast = contrast_ratio((128, 128, 128), (128, 128, 128));
        assert_eq!(contrast, 1.0);
    }
}
