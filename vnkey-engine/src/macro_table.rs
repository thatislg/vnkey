//! Bảng macro: ánh xạ viết tắt sang văn bản tiếng Việt đầy đủ.

use std::collections::HashMap;

const MAX_MACRO_LEN: usize = 256;
const MAX_MACRO_ENTRIES: usize = 1024;

/// Một mục macro
#[derive(Debug, Clone)]
struct MacroEntry {
    key: String,
    value: String,
}

/// Bảng tra macro, so khớp không phân biệt hoa/thường
pub struct MacroTable {
    entries: Vec<MacroEntry>,
    lookup: HashMap<String, usize>,
}

impl MacroTable {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            lookup: HashMap::new(),
        }
    }

    /// Thêm mục macro. Trả false nếu bảng đầy hoặc key quá dài.
    pub fn add(&mut self, key: &str, value: &str) -> bool {
        if self.entries.len() >= MAX_MACRO_ENTRIES || key.len() > MAX_MACRO_LEN {
            return false;
        }
        let lower_key = key.to_lowercase();
        if let Some(&idx) = self.lookup.get(&lower_key) {
            self.entries[idx].value = value.to_string();
        } else {
            let idx = self.entries.len();
            self.entries.push(MacroEntry {
                key: key.to_string(),
                value: value.to_string(),
            });
            self.lookup.insert(lower_key, idx);
        }
        true
    }

    /// Tra macro theo key (không phân biệt hoa/thường)
    pub fn lookup(&self, key: &str) -> Option<&str> {
        let lower_key = key.to_lowercase();
        self.lookup.get(&lower_key).map(|&idx| self.entries[idx].value.as_str())
    }

    /// Xóa macro theo key
    pub fn remove(&mut self, key: &str) -> bool {
        let lower_key = key.to_lowercase();
        if let Some(&idx) = self.lookup.get(&lower_key) {
            self.entries.swap_remove(idx);
            self.lookup.remove(&lower_key);
            // Cập nhật chỉ mục của mục đã hoán đổi
            if idx < self.entries.len() {
                let swapped_key = self.entries[idx].key.to_lowercase();
                self.lookup.insert(swapped_key, idx);
            }
            true
        } else {
            false
        }
    }

    /// Xóa tất cả macro
    pub fn clear(&mut self) {
        self.entries.clear();
        self.lookup.clear();
    }

    /// Số lượng macro
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Tải macro từ văn bản. Định dạng: mỗi dòng "key\tvalue"
    pub fn load_from_text(&mut self, text: &str) {
        self.clear();
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((key, value)) = line.split_once('\t') {
                let key = key.trim();
                let value = value.trim();
                if !key.is_empty() && !value.is_empty() {
                    self.add(key, value);
                }
            }
        }
    }
}

impl Default for MacroTable {
    fn default() -> Self {
        Self::new()
    }
}
