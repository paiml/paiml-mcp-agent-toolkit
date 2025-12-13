//! HotspotTable Widget - Replaces Grid.js
//!
//! A pure Presentar data table with:
//! - Sorting by any column
//! - Pagination
//! - Keyboard navigation (WCAG 2.1 AA)
//! - JSON/CSV export

use crate::state::{Hotspot, SortDirection};
use serde::Serialize;

/// Hotspot table widget
#[derive(Debug, Clone)]
pub struct HotspotTable {
    rows: Vec<Hotspot>,
    page_size: usize,
    current_page: usize,
    sort_column: Option<String>,
    sort_direction: SortDirection,
    keyboard_nav: bool,
    accessible_name: Option<String>,
    focusable: bool,
}

impl HotspotTable {
    /// Create a new hotspot table
    pub fn new(rows: Vec<Hotspot>) -> Self {
        Self {
            rows,
            page_size: 25,
            current_page: 0,
            sort_column: None,
            sort_direction: SortDirection::Ascending,
            keyboard_nav: false,
            accessible_name: None,
            focusable: false,
        }
    }

    /// Set page size for pagination
    pub fn with_page_size(mut self, size: usize) -> Self {
        self.page_size = size;
        self
    }

    /// Enable keyboard navigation
    pub fn with_keyboard_navigation(mut self, enabled: bool) -> Self {
        self.keyboard_nav = enabled;
        self.focusable = enabled;
        self
    }

    /// Set accessible name for screen readers
    pub fn with_accessible_name(mut self, name: &str) -> Self {
        self.accessible_name = Some(name.to_string());
        self
    }

    /// Sort by column
    pub fn sort_by(mut self, column: &str, direction: SortDirection) -> Self {
        self.sort_column = Some(column.to_string());
        self.sort_direction = direction;

        match column {
            "file" => {
                self.rows.sort_by(|a, b| {
                    let cmp = a.file.cmp(&b.file);
                    if direction == SortDirection::Descending {
                        cmp.reverse()
                    } else {
                        cmp
                    }
                });
            }
            "complexity" => {
                self.rows.sort_by(|a, b| {
                    let cmp = a.complexity.cmp(&b.complexity);
                    if direction == SortDirection::Descending {
                        cmp.reverse()
                    } else {
                        cmp
                    }
                });
            }
            "churn" => {
                self.rows.sort_by(|a, b| {
                    let cmp = a.churn.cmp(&b.churn);
                    if direction == SortDirection::Descending {
                        cmp.reverse()
                    } else {
                        cmp
                    }
                });
            }
            "score" => {
                self.rows.sort_by(|a, b| {
                    let cmp = a
                        .score
                        .partial_cmp(&b.score)
                        .unwrap_or(std::cmp::Ordering::Equal);
                    if direction == SortDirection::Descending {
                        cmp.reverse()
                    } else {
                        cmp
                    }
                });
            }
            _ => {}
        }
        self
    }

    /// Get all rows
    pub fn rows(&self) -> &[Hotspot] {
        &self.rows
    }

    /// Get total page count
    pub fn page_count(&self) -> usize {
        (self.rows.len() + self.page_size - 1) / self.page_size
    }

    /// Get rows for current page
    pub fn current_page_rows(&self) -> &[Hotspot] {
        let start = self.current_page * self.page_size;
        let end = (start + self.page_size).min(self.rows.len());
        &self.rows[start..end]
    }

    /// Check if focusable (keyboard navigation)
    pub fn is_focusable(&self) -> bool {
        self.focusable
    }

    /// Get accessible name
    pub fn accessible_name(&self) -> Option<&str> {
        self.accessible_name.as_deref()
    }

    /// Export as JSON
    pub fn export_json(&self) -> String {
        #[derive(Serialize)]
        struct ExportData<'a> {
            hotspots: &'a [Hotspot],
            total: usize,
        }

        serde_json::to_string_pretty(&ExportData {
            hotspots: &self.rows,
            total: self.rows.len(),
        })
        .unwrap_or_default()
    }

    /// Export as CSV
    pub fn export_csv(&self) -> String {
        let mut csv = String::from("file,complexity,churn,score\n");
        for row in &self.rows {
            csv.push_str(&format!(
                "{},{},{},{}\n",
                row.file, row.complexity, row.churn, row.score
            ));
        }
        csv
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_hotspots() -> Vec<Hotspot> {
        vec![
            Hotspot {
                file: "b.rs".into(),
                complexity: 20,
                churn: 3,
                score: 60.0,
            },
            Hotspot {
                file: "a.rs".into(),
                complexity: 10,
                churn: 5,
                score: 50.0,
            },
            Hotspot {
                file: "c.rs".into(),
                complexity: 15,
                churn: 10,
                score: 75.0,
            },
        ]
    }

    #[test]
    fn test_sort_by_complexity() {
        let table =
            HotspotTable::new(sample_hotspots()).sort_by("complexity", SortDirection::Descending);
        assert_eq!(table.rows()[0].file, "b.rs");
        assert_eq!(table.rows()[0].complexity, 20);
    }

    #[test]
    fn test_pagination() {
        let hotspots: Vec<Hotspot> = (0..100)
            .map(|i| Hotspot {
                file: format!("file_{i}.rs"),
                complexity: i,
                churn: i % 10,
                score: i as f64,
            })
            .collect();
        let table = HotspotTable::new(hotspots).with_page_size(25);
        assert_eq!(table.page_count(), 4);
        assert_eq!(table.current_page_rows().len(), 25);
    }

    #[test]
    fn test_export_json() {
        let table = HotspotTable::new(sample_hotspots());
        let json = table.export_json();
        assert!(json.contains("b.rs"));
        assert!(json.contains("hotspots"));
    }

    #[test]
    fn test_accessible_name() {
        let table = HotspotTable::new(vec![]).with_accessible_name("Test Table");
        assert_eq!(table.accessible_name(), Some("Test Table"));
    }

    #[test]
    fn test_keyboard_navigation() {
        let table = HotspotTable::new(vec![]).with_keyboard_navigation(true);
        assert!(table.is_focusable());
    }
}
