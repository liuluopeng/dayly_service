// 字词学习——复用 common::front_can_do::zici（与 Vue 共享的生字/生词数据）

use common::front_can_do::zici::{new_chars_for_grade, new_words};

/// 按年级/学期返回生字表（grade 1-6，term 1-2）
pub fn zici_new_chars(grade: u8, term: u8) -> Vec<String> {
    new_chars_for_grade(grade as usize, term as usize)
        .into_iter()
        .map(|c| c.to_string())
        .collect()
}

/// 返回全部生词表（2182 个）
pub fn zici_new_words() -> Vec<String> {
    new_words().iter().map(|s| s.to_string()).collect()
}

/// 词频搜索（返回 word/pinyin/frequency/explanation）
#[flutter_rust_bridge::frb]
pub struct WordFrequencyEntry {
    pub word: String,
    pub pinyin: String,
    pub frequency: u32,
    pub explanation: String,
}

/// 词频搜索（首次调用解析 3.2MB json 并缓存）
pub fn zici_word_frequency_search(query: String, limit: u32) -> Vec<WordFrequencyEntry> {
    common::front_can_do::zici::word_frequency_search(&query, limit as usize)
        .into_iter()
        .map(|e| WordFrequencyEntry {
            word: e.word,
            pinyin: e.pinyin,
            frequency: e.frequency,
            explanation: e.explanation,
        })
        .collect()
}
