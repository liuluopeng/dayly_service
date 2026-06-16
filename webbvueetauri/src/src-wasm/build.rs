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
    // Each entry: 16 bytes word + 32 bytes pinyin + 4 bytes frequency + 1 byte has_explanation + 3 pad = 56
    const ENTRY_SIZE: usize = 56;
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
        let wb = word.as_bytes();
        entry[..wb.len().min(15)].copy_from_slice(&wb[..wb.len().min(15)]);
        let pb = pinyin.as_bytes();
        entry[16..16 + pb.len().min(31)].copy_from_slice(&pb[..pb.len().min(31)]);
        entry[48..52].copy_from_slice(&freq.to_le_bytes());
        if !explanation.is_empty() {
            entry[52] = 1;
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
    for chunk in blob.chunks(56) {
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
