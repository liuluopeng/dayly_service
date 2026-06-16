use crate::frb_generated::StreamSink;
use image::codecs::gif::GifDecoder;
use image::AnimationDecoder;
use image::ImageEncoder;
use std::fs::File;
use std::io::BufReader;

#[derive(Debug, Clone)]
pub struct GifFrame {
    pub png_bytes: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub delay_ms: u32,
}

#[derive(Debug, Clone)]
pub struct GifFrameRgba {
    pub rgba: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub delay_ms: u32,
}

pub fn decode_gif_frame_count(path: String) -> usize {
    let Ok(file) = File::open(&path) else { return 0 };
    let Ok(decoder) = GifDecoder::new(BufReader::new(file)) else { return 0 };
    decoder.into_frames().count()
}

pub fn decode_gif_frame(path: String, index: usize) -> Option<GifFrame> {
    let file = File::open(&path).ok()?;
    let decoder = GifDecoder::new(BufReader::new(file)).ok()?;
    let frames = decoder.into_frames();
    let frame = frames.enumerate().find_map(|(i, f)| if i == index { f.ok() } else { None })?;

    let delay = frame.delay();
    let delay_ms = delay.numer_denom_ms().0;

    let buffer = frame.into_buffer();
    let width = buffer.width();
    let height = buffer.height();
    let rgba = buffer.into_raw();

    let mut png_buf = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new(&mut png_buf);
    encoder.write_image(&rgba, width, height, image::ExtendedColorType::Rgba8).ok()?;

    Some(GifFrame {
        png_bytes: png_buf,
        width,
        height,
        delay_ms,
    })
}

pub fn decode_gif_all_frames(path: String) -> Vec<GifFrame> {
    let Ok(file) = File::open(&path) else { return vec![] };
    let Ok(decoder) = GifDecoder::new(BufReader::new(file)) else { return vec![] };

    let mut result = Vec::new();
    for frame in decoder.into_frames() {
        let Ok(frame) = frame else { continue };
        let delay = frame.delay();
        let delay_ms = delay.numer_denom_ms().0;

        let buffer = frame.into_buffer();
        let width = buffer.width();
        let height = buffer.height();
        let rgba = buffer.into_raw();

        let mut png_buf = Vec::new();
        let encoder = image::codecs::png::PngEncoder::new(&mut png_buf);
        if encoder.write_image(&rgba, width, height, image::ExtendedColorType::Rgba8).is_err() {
            continue;
        }

        result.push(GifFrame {
            png_bytes: png_buf,
            width,
            height,
            delay_ms,
        });
    }
    result
}

pub fn decode_gif_preview_frames(path: String, max_frames: usize) -> Vec<GifFrameRgba> {
    let Ok(file) = File::open(&path) else { return vec![] };
    let Ok(decoder) = GifDecoder::new(BufReader::new(file)) else { return vec![] };

    let mut result = Vec::new();
    for (i, frame) in decoder.into_frames().enumerate() {
        if i >= max_frames { break }
        let Ok(frame) = frame else { continue };
        let delay = frame.delay();
        let delay_ms = delay.numer_denom_ms().0;

        let buffer = frame.into_buffer();
        let width = buffer.width();
        let height = buffer.height();
        let rgba = buffer.into_raw();

        result.push(GifFrameRgba { rgba, width, height, delay_ms });
    }
    result
}

pub fn decode_gif_stream(sink: StreamSink<GifFrameRgba>, path: String, max_frames: u32) {
    let Ok(file) = File::open(&path) else { return };
    let Ok(decoder) = GifDecoder::new(BufReader::new(file)) else { return };

    for (i, frame) in decoder.into_frames().enumerate() {
        if i >= max_frames as usize { break }
        let Ok(frame) = frame else { continue };
        let delay = frame.delay();
        let delay_ms = delay.numer_denom_ms().0;

        let buffer = frame.into_buffer();
        let width = buffer.width();
        let height = buffer.height();
        let rgba = buffer.into_raw();

        let _ = sink.add(GifFrameRgba { rgba, width, height, delay_ms });
    }
}
