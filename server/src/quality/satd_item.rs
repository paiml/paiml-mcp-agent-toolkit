use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SatdItem {
    pub satd_type: String,
    pub line: usize,
    pub comment: String,
}

pub struct SatdDetectorWithItems;

impl Default for SatdDetectorWithItems {
    fn default() -> Self {
        Self::new()
    }
}

impl SatdDetectorWithItems {
    pub fn new() -> Self {
        Self
    }

    pub fn detect(&self, source: &str) -> Vec<SatdItem> {
        let mut items = Vec::new();

        for (line_num, line) in source.lines().enumerate() {
            if line.contains("TODO") {
                items.push(SatdItem {
                    satd_type: "TODO".to_string(),
                    line: line_num + 1,
                    comment: line.trim().to_string(),
                });
            } else if line.contains("FIXME") {
                items.push(SatdItem {
                    satd_type: "FIXME".to_string(),
                    line: line_num + 1,
                    comment: line.trim().to_string(),
                });
            } else if line.contains("HACK") {
                items.push(SatdItem {
                    satd_type: "HACK".to_string(),
                    line: line_num + 1,
                    comment: line.trim().to_string(),
                });
            }
        }

        items
    }
}