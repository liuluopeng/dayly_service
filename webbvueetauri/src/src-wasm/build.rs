use std::io::Write;

fn main() {
    let json_path = format!("{}/../../public/word_frequency_list.json",
        std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let json = std::fs::read_to_string(&json_path)
        .expect("Failed to read word_frequency_list.json");
    let data: Vec<serde_json::Value> = serde_json::from_str(&json)
        .expect("Failed to parse word_frequency_list.json");

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest = std::path::Path::new(&out_dir).join("words_data.rs");
    let mut f = std::fs::File::create(dest).unwrap();

    // Build a compact Vec<u8> — the entire data blob
    // Each entry: 24 bytes word + 40 bytes pinyin + 4 bytes frequency + 1 byte has_explanation + 3 pad = 72
    const ENTRY_SIZE: usize = 72;
    let total = data.len();
    let mut blob: Vec<u8> = Vec::with_capacity(total * ENTRY_SIZE);

    // Also build a blob of concatenated explanation texts
    let mut explanation_blob: Vec<u8> = Vec::new();
    // Per-entry: (offset, len) for explanation lookup — 4 + 4 bytes = 8
    let mut explanation_index: Vec<u8> = Vec::with_capacity(total * 8);

    for item in &data {
        let word = item["word"].as_str().unwrap_or("");
        let pinyin = item["pinyin_flat"].as_str().unwrap_or("");
        let freq = item["frequency"].as_u64().unwrap_or(0) as u32;
        let explanation = item["explanation"].as_str().unwrap_or("");

        let mut entry = [0u8; ENTRY_SIZE];
        // 按字符边界截断，避免产生非法 UTF-8
        let wb = word.as_bytes();
        let wc = wb.len().min(24);
        let wc = word.floor_char_boundary(wc);
        entry[..wc].copy_from_slice(&wb[..wc]);
        let pb = pinyin.as_bytes();
        let pc = pb.len().min(40);
        let pc = pinyin.floor_char_boundary(pc);
        entry[24..24 + pc].copy_from_slice(&pb[..pc]);
        entry[64..68].copy_from_slice(&freq.to_le_bytes());
        if !explanation.is_empty() {
            entry[68] = 1;
        }
        blob.extend_from_slice(&entry);

        // Store explanation
        let expl_offset = explanation_blob.len();
        let expl_bytes = explanation.as_bytes();
        explanation_blob.extend_from_slice(expl_bytes);
        explanation_index.extend_from_slice(&(expl_offset as u32).to_le_bytes());
        explanation_index.extend_from_slice(&(expl_bytes.len() as u32).to_le_bytes());
    }

    writeln!(f, "// Auto-generated — DO NOT EDIT").unwrap();
    writeln!(f, "pub const WORD_COUNT: usize = {};", total).unwrap();
    writeln!(f, "pub const ENTRY_SIZE: usize = {};", ENTRY_SIZE).unwrap();

    writeln!(f, "pub const WORDS_BLOB: &[u8] = &[").unwrap();
    for chunk in blob.chunks(72) {
        write!(f, "    ").unwrap();
        for b in chunk {
            write!(f, "{},", b).unwrap();
        }
        writeln!(f).unwrap();
    }
    writeln!(f, "];").unwrap();

    // Explanation blob
    writeln!(f, "pub const EXPLANATION_BLOB: &[u8] = &[").unwrap();
    for chunk in explanation_blob.chunks(100) {
        write!(f, "    ").unwrap();
        for b in chunk {
            write!(f, "{},", b).unwrap();
        }
        writeln!(f).unwrap();
    }
    writeln!(f, "];").unwrap();

    // Explanation index
    writeln!(f, "pub const EXPLANATION_INDEX: &[u8] = &[").unwrap();
    for chunk in explanation_index.chunks(100) {
        write!(f, "    ").unwrap();
        for b in chunk {
            write!(f, "{},", b).unwrap();
        }
        writeln!(f).unwrap();
    }
    writeln!(f, "];").unwrap();

    println!("cargo:rerun-if-changed={}", json_path);
    println!("cargo:warning=Generated words data: {} entries", total);
}
