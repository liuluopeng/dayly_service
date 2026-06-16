use serde::{Deserialize, Serialize};
use std::fmt;

// 导入颜色工具
use crate::utils::color::{BLUE, CYAN, GREEN, YELLOW};
use crate::utils::table::TableFormatter;

// 仅在非 WebAssembly 环境中导入 sqlx
#[cfg(not(target_arch = "wasm32"))]
use sqlx::FromRow;

// 为所有环境定义结构体
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(FromRow))]
pub struct GgttCode {
    pub id: i32,
    pub char: String,
    pub code_86: String,
    pub has_diagram: bool,
    pub svg1: Option<String>,
    pub svg2: Option<String>,
    pub svg3: Option<String>,
    pub svg4: Option<String>,
}

impl GgttCode {
    // 使用svg_to_ansi函数显示SVG图像（兼容所有环境）
    pub fn display_with_images(&self) {
        // 打印基本信息
        println!("┌─────────────────────────────────────┐");
        println!("│ GgttCode                            │");
        println!("├─────────────────────────────────────┤");
        println!("│ ID:         {:<20} │", self.id);
        println!("│ Char:       {:<20} │", self.char);
        println!("│ Code 86:    {:<20} │", self.code_86);
        println!(
            "│ Has Diagram: {:<19} │",
            if self.has_diagram { "Yes" } else { "No" }
        );
        println!("├─────────────────────────────────────┤");

        // 显示SVG图像
        for (i, svg) in [&self.svg1, &self.svg2, &self.svg3, &self.svg4]
            .iter()
            .enumerate()
        {
            println!("│ SVG{}:                               │", i + 1);
            match svg {
                Some(svg_str) => {
                    // 创建PrintConfig配置
                    let config = PrintConfig::new()
                        .with_size(64, 64) // 设置宽度为64个字符，高度为64个字符
                        .with_truecolor(true) // 启用真彩色
                        .with_transparent(false); // 禁用透明

                    // 使用svg_to_ansi函数将SVG转换为ANSI转义序列
                    match svg_to_ansi(svg_str.as_bytes(), &config) {
                        Ok(ansi_str) => println!("{}", ansi_str), // 显示成功，打印ANSI转义序列
                        Err(e) => println!("SVG显示失败: {:?}", e), // 显示失败，打印错误信息
                    }
                }
                None => {
                    println!("None"); // SVG不存在，显示"None"
                }
            }
        }

        println!("└─────────────────────────────────────┘");
    }
}

impl fmt::Display for GgttCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let formatter = TableFormatter::new(60);

        writeln!(f, "{}", formatter.top_border())?;
        writeln!(f, "{}", formatter.format_title("GgttCode", GREEN))?;
        writeln!(f, "{}", formatter.separator())?;
        writeln!(
            f,
            "{}",
            formatter.format_row("ID", BLUE, &self.id.to_string())
        )?;
        writeln!(f, "{}", formatter.format_row("Char", GREEN, &self.char))?;
        writeln!(
            f,
            "{}",
            formatter.format_row("Code 86", YELLOW, &self.code_86)
        )?;
        writeln!(
            f,
            "{}",
            formatter.format_row(
                "Has Diagram",
                CYAN,
                if self.has_diagram { "Yes" } else { "No" }
            )
        )?;

        // 显示SVG图像
        for (i, svg) in [&self.svg1, &self.svg2, &self.svg3, &self.svg4]
            .iter()
            .enumerate()
        {
            writeln!(f, "{}", formatter.separator())?;
            writeln!(
                f,
                "{}",
                formatter.format_row(&format!("SVG{}", i + 1), BLUE, "")
            )?;
            match svg {
                Some(svg_str) => {
                    // 创建PrintConfig配置
                    let config = PrintConfig::new()
                        .with_size(64, 64) // 设置宽度为64个字符，高度为64个字符
                        .with_truecolor(true) // 启用真彩色
                        .with_transparent(false); // 禁用透明

                    // 使用svg_to_ansi函数将SVG转换为ANSI转义序列
                    match svg_to_ansi(svg_str.as_bytes(), &config) {
                        Ok(ansi_str) => {
                            // 显示完整的SVG内容
                            let lines: Vec<&str> = ansi_str.lines().collect();

                            for line in lines {
                                writeln!(f, "{}", line)?;
                            }
                        }
                        Err(e) => writeln!(f, "│ SVG显示失败: {:<22?} │", e)?,
                    }
                }
                None => {
                    writeln!(f, "│ {:<39} │", "None")?;
                }
            }
        }

        writeln!(f, "{}", formatter.bottom_border())
    }
}

use image::{DynamicImage, GenericImageView, ImageBuffer, Rgba};

#[derive(Debug, Clone, Copy)]
pub struct PrintConfig {
    pub width: u32,
    pub height: u32,
    pub truecolor: bool,
    pub transparent: bool,
}

impl Default for PrintConfig {
    fn default() -> Self {
        Self {
            width: 80,
            height: 24,
            truecolor: true,
            transparent: false,
        }
    }
}

impl PrintConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_size(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    pub fn with_truecolor(mut self, value: bool) -> Self {
        self.truecolor = value;
        self
    }

    pub fn with_transparent(mut self, value: bool) -> Self {
        self.transparent = value;
        self
    }
}

pub fn svg_to_ansi(svg_data: &[u8], config: &PrintConfig) -> Result<String, String> {
    let img = render_svg(svg_data)?;
    let img = crop_to_content(&img);
    image_to_ansi(&img, config)
}

pub fn print_svg(
    svg_data: &[u8],
    width: u32,
    height: u32,
    truecolor: bool,
    transparent: bool,
) -> Result<(), String> {
    let config = PrintConfig {
        width,
        height,
        truecolor,
        transparent,
    };
    let result = svg_to_ansi(svg_data, &config)?;
    Ok(())
}

pub fn render_svg(svg_data: &[u8]) -> Result<DynamicImage, String> {
    use resvg::usvg;

    let opt = usvg::Options::default();
    let tree =
        usvg::Tree::from_data(svg_data, &opt).map_err(|e| format!("SVG parse error: {}", e))?;

    let tree_size = tree.size();
    let target_width = 400;
    let target_height =
        (target_width as f32 * tree_size.height() / tree_size.width()).round() as u32;

    let mut pixmap = resvg::tiny_skia::Pixmap::new(target_width.max(1), target_height.max(1))
        .ok_or_else(|| "Failed to create pixmap".to_string())?;
    resvg::render(&tree, usvg::Transform::default(), &mut pixmap.as_mut());

    Ok(DynamicImage::ImageRgba8(
        image::RgbaImage::from_raw(pixmap.width(), pixmap.height(), pixmap.data().to_vec())
            .ok_or_else(|| "Failed to create image".to_string())?,
    ))
}

pub fn crop_to_content(img: &DynamicImage) -> DynamicImage {
    let (width, height) = img.dimensions();

    let mut min_x = width;
    let mut max_x = 0;
    let mut min_y = height;
    let mut max_y = 0;

    for y in 0..height {
        for x in 0..width {
            let pixel = img.get_pixel(x, y);
            if pixel.0[3] > 0 {
                if x < min_x {
                    min_x = x;
                }
                if x > max_x {
                    max_x = x;
                }
                if y < min_y {
                    min_y = y;
                }
                if y > max_y {
                    max_y = y;
                }
            }
        }
    }

    if min_x > max_x || min_y > max_y {
        return img.clone();
    }

    let crop_width = max_x - min_x + 1;
    let crop_height = max_y - min_y + 1;

    let mut cropped = ImageBuffer::new(crop_width, crop_height);
    for y in 0..crop_height {
        for x in 0..crop_width {
            let pixel = img.get_pixel(min_x + x, min_y + y);
            cropped.put_pixel(x, y, Rgba(pixel.0));
        }
    }

    DynamicImage::ImageRgba8(cropped)
}

pub fn image_to_ansi(img: &DynamicImage, config: &PrintConfig) -> Result<String, String> {
    let img = resize(img, Some(config.width), Some(config.height));
    let (width, height) = img.dimensions();
    let img_buffer = img.to_rgba8();

    let mut result = String::new();
    let mut row_buffer: Vec<(Option<(u8, u8, u8)>, Option<(u8, u8, u8)>)> =
        vec![(None, None); width as usize];

    for (curr_row, img_row) in img_buffer.enumerate_rows() {
        let is_even_row = curr_row % 2 == 0;
        let is_last_row = curr_row == height - 1;

        for pixel in img_row {
            let (col, _, rgba) = pixel;
            let color = if rgba.0[3] == 0 {
                if config.transparent {
                    None
                } else {
                    Some(checkerboard_color(curr_row, col))
                }
            } else {
                let rgb = (rgba.0[0], rgba.0[1], rgba.0[2]);
                if config.transparent || rgba.0[3] == 255 {
                    Some(rgb)
                } else {
                    let checker = checkerboard_color_raw(curr_row, col);
                    Some(alpha_blend(rgb, checker, rgba.0[3]))
                }
            };

            if is_even_row {
                row_buffer[col as usize].1 = color;
                if is_last_row {
                    let (fg, bg) = &row_buffer[col as usize];
                    write_half_block(&mut result, *fg, *bg, true, config.truecolor);
                    row_buffer[col as usize] = (None, None);
                }
            } else {
                row_buffer[col as usize].0 = color;
                let (fg, bg) = &row_buffer[col as usize];
                write_half_block(&mut result, *fg, *bg, false, config.truecolor);
                row_buffer[col as usize] = (None, None);
            }
        }

        if !is_even_row && !is_last_row {
            result.push_str("\x1b[0m\n");
        }
    }

    result.push_str("\x1b[0m\n");
    Ok(result)
}

fn write_half_block(
    out: &mut String,
    fg: Option<(u8, u8, u8)>,
    bg: Option<(u8, u8, u8)>,
    is_last_row: bool,
    truecolor: bool,
) {
    if is_last_row {
        if let Some(bg_color) = bg {
            write_fg(out, Some(bg_color), truecolor);
            out.push('\u{2580}');
        } else {
            out.push(' ');
        }
        return;
    }

    match (fg, bg) {
        (None, None) => {
            out.push(' ');
        }
        (Some(fg_color), None) => {
            write_fg(out, Some(fg_color), truecolor);
            out.push('\u{2584}');
        }
        (None, Some(bg_color)) => {
            write_fg(out, Some(bg_color), truecolor);
            out.push('\u{2580}');
        }
        (Some(fg_color), Some(bg_color)) => {
            write_color(out, Some(fg_color), bg_color, truecolor);
            out.push('\u{2584}');
        }
    }
}

fn write_color(out: &mut String, fg: Option<(u8, u8, u8)>, bg: (u8, u8, u8), truecolor: bool) {
    if truecolor {
        if let Some(fg_color) = fg {
            out.push_str(&format!(
                "\x1b[38;2;{};{};{}m",
                fg_color.0, fg_color.1, fg_color.2
            ));
        }
        out.push_str(&format!("\x1b[48;2;{};{};{}m", bg.0, bg.1, bg.2));
    } else {
        if let Some(fg_color) = fg {
            out.push_str(&format!("\x1b[38;5;{}m", ansi256_from_rgb(fg_color)));
        }
        out.push_str(&format!("\x1b[48;5;{}m", ansi256_from_rgb(bg)));
    }
}

fn write_fg(out: &mut String, fg: Option<(u8, u8, u8)>, truecolor: bool) {
    out.push_str("\x1b[0m");
    if let Some(fg_color) = fg {
        if truecolor {
            out.push_str(&format!(
                "\x1b[38;2;{};{};{}m",
                fg_color.0, fg_color.1, fg_color.2
            ));
        } else {
            out.push_str(&format!("\x1b[38;5;{}m", ansi256_from_rgb(fg_color)));
        }
    }
}

fn checkerboard_color(row: u32, col: u32) -> (u8, u8, u8) {
    checkerboard_color_raw(row, col)
}

fn checkerboard_color_raw(row: u32, col: u32) -> (u8, u8, u8) {
    if row % 2 == col % 2 {
        (102, 102, 102)
    } else {
        (153, 153, 153)
    }
}

fn alpha_blend(fg: (u8, u8, u8), bg: (u8, u8, u8), alpha: u8) -> (u8, u8, u8) {
    (
        ((fg.0 as u16 * alpha as u16 + bg.0 as u16 * (255 - alpha as u16)) / 255) as u8,
        ((fg.1 as u16 * alpha as u16 + bg.1 as u16 * (255 - alpha as u16)) / 255) as u8,
        ((fg.2 as u16 * alpha as u16 + bg.2 as u16 * (255 - alpha as u16)) / 255) as u8,
    )
}

fn ansi256_from_rgb(rgb: (u8, u8, u8)) -> u8 {
    ansi_colours::ansi256_from_rgb(rgb)
}

fn resize(img: &DynamicImage, width: Option<u32>, height: Option<u32>) -> DynamicImage {
    let (w, h) = find_best_fit(img, width, height);
    img.resize_exact(
        w,
        2 * h - img.height() % 2,
        image::imageops::FilterType::CatmullRom,
    )
}

fn find_best_fit(img: &DynamicImage, width: Option<u32>, height: Option<u32>) -> (u32, u32) {
    let (img_width, img_height) = img.dimensions();
    const DEFAULT_TERM_W: u32 = 80;
    const DEFAULT_TERM_H: u32 = 24;

    match (width, height) {
        (None, None) => {
            let (w, h) = fit_dimensions(img_width, img_height, DEFAULT_TERM_W, DEFAULT_TERM_H);
            (w, h.saturating_sub(1).max(1))
        }
        (Some(w), None) => fit_dimensions(img_width, img_height, w, img_height),
        (None, Some(h)) => fit_dimensions(img_width, img_height, img_width, h),
        (Some(w), Some(h)) => fit_dimensions(img_width, img_height, w, h),
    }
}

fn fit_dimensions(width: u32, height: u32, bound_width: u32, bound_height: u32) -> (u32, u32) {
    let bound_height_scaled = 2 * bound_height;

    if width <= bound_width && height <= bound_height_scaled {
        return (width, std::cmp::max(1, height / 2 + height % 2));
    }

    let ratio = width * bound_height_scaled;
    let nratio = bound_width * height;

    let use_width = nratio <= ratio;
    let intermediate = if use_width {
        height * bound_width / width
    } else {
        width * bound_height_scaled / height
    };

    if use_width {
        (bound_width, std::cmp::max(1, intermediate / 2))
    } else {
        (intermediate, std::cmp::max(1, bound_height))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ggtt_display() {
        let jsonn = r##"{
  "id": 326,
  "char": "地",
  "code_86": "fbn",
  "has_diagram": true,
  "svg1": "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"75\" height=\"75\" viewBox=\"0 0 1000 1000\"><g transform=\"scale(1, -1) translate(0, -1000) scale(0.9) translate(3.749999999999999, 3.749999999999999)\"><path d=\"M 292 431 Q 376 461 382 466 Q 389 475 384 482 Q 377 492 349 495 Q 321 496 295 483 Q 294 483 294 482 L 244 459 Q 154 429 124 423 Q 91 413 117 401 Q 151 389 224 409 Q 233 412 245 415 L 292 431 Z\" fill=\"#F00\" /><path d=\"M 286 229 Q 290 334 292 431 L 294 482 Q 294 599 315 684 Q 318 696 296 711 Q 259 730 232 734 Q 214 738 206 728 Q 199 721 207 704 Q 237 662 237 634 Q 241 550 244 459 L 245 415 Q 245 318 242 211 C 241 181 285 199 286 229 Z\" fill=\"#F00\" /><path d=\"M 242 211 Q 190 190 134 167 Q 118 160 88 157 Q 75 154 74 143 Q 73 130 84 122 Q 109 109 146 93 Q 156 92 168 102 Q 195 129 346 223 Q 367 236 382 250 Q 394 260 393 269 Q 387 273 375 270 Q 332 251 286 229 L 242 211 Z\" fill=\"#F00\" /><path d=\"M 473 369 Q 513 393 580 428 L 627 454 Q 675 482 738 514 Q 757 521 762 510 Q 772 494 735 332 Q 731 289 700 300 Q 681 306 661 310 Q 648 311 651 304 Q 657 294 719 240 Q 741 216 757 237 Q 791 280 798 343 Q 808 406 818 470 Q 825 504 853 535 Q 863 551 848 562 Q 829 572 780 579 Q 761 582 749 569 Q 724 545 630 489 L 583 463 Q 541 444 497 421 Q 485 417 475 411 L 432 391 Q 377 369 341 355 Q 328 352 328 343 Q 328 336 369 326 Q 394 319 415 334 Q 419 338 432 345 L 473 369 Z\" fill=\"#555\" /><path d=\"M 580 428 Q 565 263 574 233 Q 580 220 590 226 Q 612 250 617 331 Q 621 395 627 454 L 630 489 Q 643 699 658 753 Q 668 774 655 787 Q 639 803 609 819 Q 585 829 560 820 Q 547 811 562 794 Q 586 742 589 696 Q 593 597 583 463 L 580 428 Z\" fill=\"#555\" /><path d=\"M 973 161 Q 958 201 944 318 Q 944 334 937 338 Q 931 341 927 324 Q 905 201 885 162 Q 869 135 811 120 Q 682 86 560 140 Q 512 165 497 197 Q 476 234 473 300 Q 472 333 473 369 L 475 411 Q 478 472 487 539 Q 491 557 479 566 Q 466 579 442 586 Q 429 590 421 585 Q 414 581 420 563 Q 435 526 433 488 Q 432 436 432 391 L 432 345 Q 433 221 451 176 Q 460 139 493 108 Q 604 20 811 49 Q 836 55 862 62 Q 916 80 966 121 Q 985 136 973 161 Z\" fill=\"#555\" /></g></svg>",
  "svg2": "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"75\" height=\"75\" viewBox=\"0 0 1000 1000\"><g transform=\"scale(1, -1) translate(0, -1000) scale(0.9) translate(3.749999999999999, 3.749999999999999)\"><path d=\"M 292 431 Q 376 461 382 466 Q 389 475 384 482 Q 377 492 349 495 Q 321 496 295 483 Q 294 483 294 482 L 244 459 Q 154 429 124 423 Q 91 413 117 401 Q 151 389 224 409 Q 233 412 245 415 L 292 431 Z\" fill=\"#555\" /><path d=\"M 286 229 Q 290 334 292 431 L 294 482 Q 294 599 315 684 Q 318 696 296 711 Q 259 730 232 734 Q 214 738 206 728 Q 199 721 207 704 Q 237 662 237 634 Q 241 550 244 459 L 245 415 Q 245 318 242 211 C 241 181 285 199 286 229 Z\" fill=\"#555\" /><path d=\"M 242 211 Q 190 190 134 167 Q 118 160 88 157 Q 75 154 74 143 Q 73 130 84 122 Q 109 109 146 93 Q 156 92 168 102 Q 195 129 346 223 Q 367 236 382 250 Q 394 260 393 269 Q 387 273 375 270 Q 332 251 286 229 L 242 211 Z\" fill=\"#555\" /><path d=\"M 473 369 Q 513 393 580 428 L 627 454 Q 675 482 738 514 Q 757 521 762 510 Q 772 494 735 332 Q 731 289 700 300 Q 681 306 661 310 Q 648 311 651 304 Q 657 294 719 240 Q 741 216 757 237 Q 791 280 798 343 Q 808 406 818 470 Q 825 504 853 535 Q 863 551 848 562 Q 829 572 780 579 Q 761 582 749 569 Q 724 545 630 489 L 583 463 Q 541 444 497 421 Q 485 417 475 411 L 432 391 Q 377 369 341 355 Q 328 352 328 343 Q 328 336 369 326 Q 394 319 415 334 Q 419 338 432 345 L 473 369 Z\" fill=\"#F00\" /><path d=\"M 580 428 Q 565 263 574 233 Q 580 220 590 226 Q 612 250 617 331 Q 621 395 627 454 L 630 489 Q 643 699 658 753 Q 668 774 655 787 Q 639 803 609 819 Q 585 829 560 820 Q 547 811 562 794 Q 586 742 589 696 Q 593 597 583 463 L 580 428 Z\" fill=\"#F00\" /><path d=\"M 973 161 Q 958 201 944 318 Q 944 334 937 338 Q 931 341 927 324 Q 905 201 885 162 Q 869 135 811 120 Q 682 86 560 140 Q 512 165 497 197 Q 476 234 473 300 Q 472 333 473 369 L 475 411 Q 478 472 487 539 Q 491 557 479 566 Q 466 579 442 586 Q 429 590 421 585 Q 414 581 420 563 Q 435 526 433 488 Q 432 436 432 391 L 432 345 Q 433 221 451 176 Q 460 139 493 108 Q 604 20 811 49 Q 836 55 862 62 Q 916 80 966 121 Q 985 136 973 161 Z\" fill=\"#F00\" /></g></svg>",
  "svg3": "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"75\" height=\"75\" viewBox=\"0 0 1000 1000\"><g transform=\"scale(1, -1) translate(0, -1000) scale(0.9) translate(3.749999999999999, 3.749999999999999)\"></g></svg>",
  "svg4": "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"75\" height=\"75\" viewBox=\"0 0 1000 1000\"><g transform=\"scale(1, -1) translate(0, -1000) scale(0.9) translate(3.749999999999999, 3.749999999999999)\"></g></svg>"
}"##;

        // Deserialize the JSON into a GgttCode struct
        let ggcode: GgttCode = serde_json::from_str(jsonn).expect("Failed to deserialize JSON");

        // Print the struct
        println!("{}", ggcode);
    }
}
